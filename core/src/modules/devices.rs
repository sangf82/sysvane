//! Everything the UI can change goes through here: one owner thread applies commands one at a time, so
//! the command pipe never touches a device and two commands never run together. The result shows in the
//! next snapshot (`snapshot()`); there is no reply.

use crate::modules::Host;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, Mutex};
use sysvane_wire::{
    Command, Control, ControlId, ControlValue, Device, DeviceId, DeviceStatus, PROTOCOL,
};

/// A client that floods the pipe loses its commands instead of growing a queue.
const QUEUE: usize = 16;

pub struct Devices {
    autostart: AtomicBool,
    error: Mutex<Option<String>>,
    register: fn(bool) -> Result<(), String>,
    modules: Mutex<Vec<Host>>,
    /// Detects monitors again, for `ControlId::Scan`.
    scan: fn() -> Vec<Host>,
    tx: SyncSender<Command>,
}

impl Devices {
    /// Starts the owner thread. `initial` is what the scheduled task holds now. `scan` lists the monitors;
    /// it runs once here and again on every `ControlId::Scan`.
    pub fn start(initial: bool, mut modules: Vec<Host>, scan: fn() -> Vec<Host>) -> Arc<Self> {
        let mut all = scan();
        all.append(&mut modules);
        Self::with(initial, crate::system::install::register, all, scan)
    }

    fn with(
        initial: bool,
        register: fn(bool) -> Result<(), String>,
        modules: Vec<Host>,
        scan: fn() -> Vec<Host>,
    ) -> Arc<Self> {
        let (tx, rx) = sync_channel(QUEUE);
        let devices = Arc::new(Self {
            autostart: AtomicBool::new(initial),
            error: Mutex::new(None),
            register,
            modules: Mutex::new(modules),
            scan,
            tx,
        });
        let owner = devices.clone();
        std::thread::spawn(move || owner.run(rx));
        devices
    }

    fn run(&self, rx: Receiver<Command>) {
        while let Ok(command) = rx.recv() {
            self.apply(command);
        }
    }

    /// Called from the pipe thread: checks the protocol and queues. Never blocks.
    pub fn submit(&self, command: Command) -> bool {
        if command.protocol != PROTOCOL {
            crate::system::log::error(&format!("command refused: protocol {}", command.protocol));
            return false;
        }
        self.tx.try_send(command).is_ok()
    }

    fn apply(&self, command: Command) {
        if command.device != DeviceId::Core {
            // A module reports its own failure in its own device entry.
            let sent = self
                .modules
                .lock()
                .unwrap()
                .iter()
                .find(|m| m.id() == command.device)
                .is_some_and(|m| m.set(command.control, command.value));
            if !sent {
                crate::system::log::error(&format!(
                    "command dropped: {:?} is missing or busy",
                    command.device
                ));
            }
            return;
        }
        let result = match (command.device, command.control, command.value) {
            (DeviceId::Core, ControlId::Autostart, ControlValue::Bool(on)) => {
                self.set_autostart(on)
            }
            (DeviceId::Core, ControlId::Scan, ControlValue::Bool(true)) => {
                self.rescan();
                Ok(())
            }
            _ => Err("that control does not take this value".to_owned()),
        };
        let error = result.err();
        if let Some(e) = &error {
            crate::system::log::error(&format!("command failed: {e}"));
        }
        *self.error.lock().unwrap() = error;
    }

    /// The tray menu calls this directly; the command pipe reaches it through `apply`.
    pub fn set_autostart(&self, on: bool) -> Result<(), String> {
        (self.register)(on)?;
        self.autostart.store(on, Ordering::Relaxed);
        Ok(())
    }

    /// Replaces every monitor with what `scan` finds now. The old monitor threads are stopped first, so a
    /// monitor never has two owners and a display-mode trial is reverted before the new module reads the
    /// mode; numbers follow the new order.
    fn rescan(&self) {
        let old = self.take(|id| matches!(id, DeviceId::Monitor(_)));
        stop_all(old);
        let mut found = (self.scan)();
        crate::system::log::info(&format!("scan: {} monitor(s)", found.len()));
        let mut modules = self.modules.lock().unwrap();
        found.append(&mut modules);
        *modules = found;
    }

    /// Stops every module, so their `Drop` runs (a display-mode trial is reverted) before the core exits.
    pub fn shutdown(&self) {
        stop_all(self.take(|_| true));
    }

    fn take(&self, which: impl Fn(DeviceId) -> bool) -> Vec<Host> {
        let mut modules = self.modules.lock().unwrap();
        let (taken, kept) = std::mem::take(&mut *modules)
            .into_iter()
            .partition(|m| which(m.id()));
        *modules = kept;
        taken
    }

    pub fn autostart(&self) -> bool {
        self.autostart.load(Ordering::Relaxed)
    }

    /// After sleep or a device change: every module writes its saved state again.
    pub fn resume(&self) {
        for m in self.modules.lock().unwrap().iter() {
            if !m.reapply() {
                crate::system::log::error(&format!("re-apply dropped: {:?} is busy", m.id()));
            }
        }
    }

    pub fn snapshot(&self) -> Vec<Device> {
        let error = self.error.lock().unwrap().clone();
        let core = Device {
            id: DeviceId::Core,
            name: String::new(),
            profiles: Vec::new(),
            modes: Vec::new(),
            presets: Vec::new(),
            busy: false,
            status: if error.is_some() {
                DeviceStatus::Failed
            } else {
                DeviceStatus::Ok
            },
            error,
            controls: vec![Control {
                id: ControlId::Autostart,
                value: ControlValue::Bool(self.autostart()),
            }],
        };
        std::iter::once(core)
            .chain(self.modules.lock().unwrap().iter().map(Host::device))
            .collect()
    }
}

/// Stops the modules side by side, each with the watchdog's time; one that is hung is logged and left behind.
fn stop_all(hosts: Vec<Host>) {
    let ids: Vec<DeviceId> = hosts.iter().map(Host::id).collect();
    let stopped: Vec<bool> = std::thread::scope(|s| {
        let waits: Vec<_> = hosts
            .into_iter()
            .map(|h| s.spawn(move || h.stop(crate::modules::DEADLINE)))
            .collect();
        waits
            .into_iter()
            .map(|w| w.join().unwrap_or(false))
            .collect()
    });
    for (id, ok) in ids.into_iter().zip(stopped) {
        if !ok {
            crate::system::log::error(&format!("{id:?} did not stop; left behind"));
        }
    }
}

#[cfg(test)]
fn cmd(on: bool) -> Command {
    Command {
        protocol: PROTOCOL,
        device: DeviceId::Core,
        control: ControlId::Autostart,
        value: ControlValue::Bool(on),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::tests::{Probe, wait};
    use std::time::Duration;

    #[test]
    fn a_command_changes_the_control_and_shows_in_the_snapshot() {
        let d = Devices::with(false, |_| Ok(()), vec![], Vec::new);
        assert!(d.submit(cmd(true)));
        wait("autostart on", || d.autostart());
        assert_eq!(d.snapshot()[0].status, DeviceStatus::Ok);
    }

    #[test]
    fn a_failed_command_marks_the_device_failed_and_keeps_the_old_value() {
        let d = Devices::with(false, |_| Err("no rights".into()), vec![], Vec::new);
        assert!(d.submit(cmd(true)));
        wait("failure shown", || {
            d.snapshot()[0].status == DeviceStatus::Failed
        });
        assert!(!d.autostart());
        assert_eq!(d.snapshot()[0].error.as_deref(), Some("no rights"));
    }

    #[test]
    fn another_protocol_is_refused_and_a_flood_is_bounded() {
        let d = Devices::with(
            false,
            |_| {
                std::thread::sleep(Duration::from_millis(50));
                Ok(())
            },
            vec![],
            Vec::new,
        );
        let mut old = cmd(true);
        old.protocol = PROTOCOL - 1;
        assert!(!d.submit(old));
        let queued = (0..100).filter(|_| d.submit(cmd(true))).count();
        assert!(queued <= QUEUE + 1, "{queued} commands queued");
    }

    fn fake_cmd(level: i64) -> Command {
        Command {
            protocol: PROTOCOL,
            device: DeviceId::Fake,
            control: ControlId::Level,
            value: ControlValue::Int(level),
        }
    }

    #[test]
    fn a_hung_module_does_not_stop_the_core_and_resume_reaches_every_module() {
        let probe = Probe::default();
        let (hang, reapplied) = (probe.hang.clone(), probe.reapplied.clone());
        let d = Devices::with(false, |_| Ok(()), vec![Host::spawn(probe)], Vec::new);
        hang.store(true, Ordering::SeqCst);
        assert!(d.submit(fake_cmd(20)));
        assert!(d.submit(cmd(true)));
        wait("core command done while the module hangs", || d.autostart());
        hang.store(false, Ordering::SeqCst);
        wait("module answers again", || {
            d.snapshot()[1].controls[0].value == ControlValue::Int(20)
        });
        d.resume();
        wait("re-applied after resume", || {
            reapplied.load(Ordering::SeqCst) == 1
        });
    }
}
