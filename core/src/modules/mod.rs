//! A device module and the thread that hosts it. Every module runs on its own thread, so one that hangs
//! costs only its own device: the core, the pipe and the other modules keep going. The watchdog is the
//! clock on the call in progress; a module is reported failed while a call outlasts `DEADLINE`.

pub mod config;
pub mod devices;
#[cfg(debug_assertions)]
pub mod fake;

use std::sync::mpsc::{SyncSender, sync_channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use sysvane_wire::{Control, ControlId, ControlValue, Device, DeviceId, DeviceStatus};

/// A call that takes longer than this marks the module failed.
const DEADLINE: Duration = Duration::from_secs(5);
/// Jobs waiting for a busy module. Beyond this they are dropped, so a hung module cannot grow a queue.
const QUEUE: usize = 4;

pub trait Module: Send + 'static {
    fn id(&self) -> DeviceId;
    fn controls(&self) -> Vec<Control>;
    /// Validates `value` and writes it. The module owns its device's safety rules.
    fn set(&mut self, control: ControlId, value: ControlValue) -> Result<(), String>;
    /// After sleep or a device change: write the saved state to the device again.
    fn reapply(&mut self) -> Result<(), String>;
}

enum Job {
    Set(ControlId, ControlValue),
    Reapply,
}

#[derive(Default)]
struct Shared {
    controls: Vec<Control>,
    error: Option<String>,
    /// Start of the call in progress, if any.
    busy_since: Option<Instant>,
}

pub struct Host {
    id: DeviceId,
    deadline: Duration,
    shared: Arc<Mutex<Shared>>,
    tx: SyncSender<Job>,
}

impl Host {
    pub fn spawn(module: impl Module) -> Self {
        Self::with_deadline(module, DEADLINE)
    }

    fn with_deadline(mut module: impl Module, deadline: Duration) -> Self {
        let id = module.id();
        let shared = Arc::new(Mutex::new(Shared {
            controls: module.controls(),
            ..Shared::default()
        }));
        let (tx, rx) = sync_channel(QUEUE);
        let state = shared.clone();
        std::thread::spawn(move || {
            while let Ok(job) = rx.recv() {
                state.lock().unwrap().busy_since = Some(Instant::now());
                let result = match job {
                    Job::Set(control, value) => module.set(control, value),
                    Job::Reapply => module.reapply(),
                };
                if let Err(e) = &result {
                    crate::system::log::error(&format!("{id:?}: {e}"));
                }
                let controls = module.controls();
                let mut s = state.lock().unwrap();
                s.controls = controls;
                s.error = result.err();
                s.busy_since = None;
            }
        });
        Self {
            id,
            deadline,
            shared,
            tx,
        }
    }

    pub fn id(&self) -> DeviceId {
        self.id
    }

    /// Never blocks. False when the module is too far behind to take it.
    pub fn set(&self, control: ControlId, value: ControlValue) -> bool {
        self.tx.try_send(Job::Set(control, value)).is_ok()
    }

    pub fn reapply(&self) -> bool {
        self.tx.try_send(Job::Reapply).is_ok()
    }

    pub fn device(&self) -> Device {
        let s = self.shared.lock().unwrap();
        let hung = s.busy_since.is_some_and(|t| t.elapsed() > self.deadline);
        let error = if hung {
            Some("not answering".to_owned())
        } else {
            s.error.clone()
        };
        Device {
            id: self.id,
            status: if error.is_some() {
                DeviceStatus::Failed
            } else {
                DeviceStatus::Ok
            },
            error,
            controls: s.controls.clone(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    /// Controllable stand-in for hardware: can be told to hang, and counts re-applies.
    #[derive(Default)]
    pub struct Probe {
        pub hang: Arc<AtomicBool>,
        pub reapplied: Arc<AtomicU32>,
        level: u8,
    }

    impl Module for Probe {
        fn id(&self) -> DeviceId {
            DeviceId::Fake
        }
        fn controls(&self) -> Vec<Control> {
            vec![Control {
                id: ControlId::Level,
                value: ControlValue::Int(self.level.into()),
            }]
        }
        fn set(&mut self, _: ControlId, value: ControlValue) -> Result<(), String> {
            while self.hang.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(5));
            }
            match value {
                ControlValue::Int(n @ 0..=100) => {
                    self.level = n as u8;
                    Ok(())
                }
                _ => Err("level is 0 to 100".to_owned()),
            }
        }
        fn reapply(&mut self) -> Result<(), String> {
            self.reapplied.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    pub fn wait(what: &str, mut ok: impl FnMut() -> bool) {
        let start = Instant::now();
        while !ok() {
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "timed out: {what}"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    fn level(h: &Host) -> ControlValue {
        h.device().controls[0].value
    }

    #[test]
    fn a_set_shows_in_the_device_and_a_bad_value_marks_it_failed() {
        let h = Host::spawn(Probe::default());
        assert!(h.set(ControlId::Level, ControlValue::Int(40)));
        wait("level 40", || level(&h) == ControlValue::Int(40));
        assert_eq!(h.device().status, DeviceStatus::Ok);
        assert!(h.set(ControlId::Level, ControlValue::Int(900)));
        wait("failure shown", || {
            h.device().status == DeviceStatus::Failed
        });
        assert_eq!(level(&h), ControlValue::Int(40), "old value kept");
    }

    #[test]
    fn a_hung_module_is_reported_failed_and_recovers() {
        let probe = Probe::default();
        let hang = probe.hang.clone();
        let h = Host::with_deadline(probe, Duration::from_millis(50));
        hang.store(true, Ordering::SeqCst);
        assert!(h.set(ControlId::Level, ControlValue::Int(10)));
        wait("reported not answering", || {
            h.device().error.as_deref() == Some("not answering")
        });
        let queued = (0..50)
            .filter(|_| h.set(ControlId::Level, ControlValue::Int(10)))
            .count();
        assert!(queued <= QUEUE, "{queued} jobs queued behind a hung module");
        hang.store(false, Ordering::SeqCst);
        wait("recovered", || h.device().status == DeviceStatus::Ok);
    }

    #[test]
    fn reapply_reaches_the_module() {
        let probe = Probe::default();
        let count = probe.reapplied.clone();
        let h = Host::spawn(probe);
        assert!(h.reapply());
        wait("reapplied", || count.load(Ordering::SeqCst) == 1);
    }
}
