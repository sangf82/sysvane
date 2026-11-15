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
    modules: Vec<Host>,
    tx: SyncSender<Command>,
}

impl Devices {
    /// Starts the owner thread. `initial` is what the scheduled task holds now.
    pub fn start(initial: bool, modules: Vec<Host>) -> Arc<Self> {
        Self::with(initial, crate::system::install::register, modules)
    }

    fn with(
        initial: bool,
        register: fn(bool) -> Result<(), String>,
        modules: Vec<Host>,
    ) -> Arc<Self> {
        let (tx, rx) = sync_channel(QUEUE);
        let devices = Arc::new(Self {
            autostart: AtomicBool::new(initial),
            error: Mutex::new(None),
            register,
            modules,
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

    pub fn autostart(&self) -> bool {
        self.autostart.load(Ordering::Relaxed)
    }

    /// After sleep or a device change: every module writes its saved state again.
    pub fn resume(&self) {
        for m in &self.modules {
            if !m.reapply() {
                crate::system::log::error(&format!("re-apply dropped: {:?} is busy", m.id()));
            }
        }
    }

    pub fn snapshot(&self) -> Vec<Device> {
        let error = self.error.lock().unwrap().clone();
        let core = Device {
            id: DeviceId::Core,
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
            .chain(self.modules.iter().map(Host::device))
            .collect()
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
        let d = Devices::with(false, |_| Ok(()), vec![]);
        assert!(d.submit(cmd(true)));
        wait("autostart on", || d.autostart());
        assert_eq!(d.snapshot()[0].status, DeviceStatus::Ok);
    }

    #[test]
    fn a_failed_command_marks_the_device_failed_and_keeps_the_old_value() {
        let d = Devices::with(false, |_| Err("no rights".into()), vec![]);
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
        let d = Devices::with(false, |_| Ok(()), vec![Host::spawn(probe)]);
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
