//! Everything the UI can change goes through here: one owner thread applies commands one at a time, so
//! the command pipe never touches a device and two commands never run together. The result shows in the
//! next snapshot (`snapshot()`); there is no reply.

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
    tx: SyncSender<Command>,
}

impl Devices {
    /// Starts the owner thread. `initial` is what the scheduled task holds now.
    pub fn start(initial: bool) -> Arc<Self> {
        Self::with(initial, crate::install::register)
    }

    fn with(initial: bool, register: fn(bool) -> Result<(), String>) -> Arc<Self> {
        let (tx, rx) = sync_channel(QUEUE);
        let devices = Arc::new(Self {
            autostart: AtomicBool::new(initial),
            error: Mutex::new(None),
            register,
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
            crate::log::error(&format!("command refused: protocol {}", command.protocol));
            return false;
        }
        self.tx.try_send(command).is_ok()
    }

    fn apply(&self, command: Command) {
        let result = match (command.device, command.control, command.value) {
            (DeviceId::Core, ControlId::Autostart, ControlValue::Bool(on)) => {
                self.set_autostart(on)
            }
            _ => Err("that control does not take this value".to_owned()),
        };
        let error = result.err();
        if let Some(e) = &error {
            crate::log::error(&format!("command failed: {e}"));
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

    pub fn snapshot(&self) -> Vec<Device> {
        let error = self.error.lock().unwrap().clone();
        vec![Device {
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
        }]
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
    use std::time::{Duration, Instant};

    fn wait(what: &str, mut ok: impl FnMut() -> bool) {
        let start = Instant::now();
        while !ok() {
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "timed out: {what}"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn a_command_changes_the_control_and_shows_in_the_snapshot() {
        let d = Devices::with(false, |_| Ok(()));
        assert!(d.submit(cmd(true)));
        wait("autostart on", || d.autostart());
        assert_eq!(d.snapshot()[0].status, DeviceStatus::Ok);
    }

    #[test]
    fn a_failed_command_marks_the_device_failed_and_keeps_the_old_value() {
        let d = Devices::with(false, |_| Err("no rights".into()));
        assert!(d.submit(cmd(true)));
        wait("failure shown", || {
            d.snapshot()[0].status == DeviceStatus::Failed
        });
        assert!(!d.autostart());
        assert_eq!(d.snapshot()[0].error.as_deref(), Some("no rights"));
    }

    #[test]
    fn another_protocol_is_refused_and_a_flood_is_bounded() {
        let d = Devices::with(false, |_| {
            std::thread::sleep(Duration::from_millis(50));
            Ok(())
        });
        let mut old = cmd(true);
        old.protocol = PROTOCOL - 1;
        assert!(!d.submit(old));
        let queued = (0..100).filter(|_| d.submit(cmd(true))).count();
        assert!(queued <= QUEUE + 1, "{queued} commands queued");
    }
}
