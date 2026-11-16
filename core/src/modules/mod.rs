//! A device module and the thread that hosts it. Every module runs on its own thread, so one that hangs
//! costs only its own device: the core, the pipe and the other modules keep going. The watchdog is the
//! clock on the call in progress; a module is reported failed while a call outlasts `DEADLINE`.

pub mod config;
pub mod devices;
#[cfg(debug_assertions)]
pub mod fake;
pub mod monitor;

use std::sync::mpsc::{RecvTimeoutError, SyncSender, sync_channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use sysvane_wire::{Control, ControlId, ControlValue, Device, DeviceId, DeviceStatus};

/// A call that takes longer than this marks the module failed.
pub(crate) const DEADLINE: Duration = Duration::from_secs(5);
/// Jobs waiting for a busy module. Beyond this they are dropped, so a hung module cannot grow a queue.
const QUEUE: usize = 4;

pub trait Module: Send + 'static {
    fn id(&self) -> DeviceId;
    /// Shown in the UI. Empty when the UI names the device itself.
    fn name(&self) -> String {
        String::new()
    }
    /// First call on the module's own thread, under the watchdog: read the device here, not in `new`.
    fn start(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn controls(&self) -> Vec<Control>;
    fn profiles(&self) -> Vec<String> {
        Vec::new()
    }
    /// Choices for a control that takes one of a list, such as a monitor's display modes.
    fn modes(&self) -> Vec<String> {
        Vec::new()
    }
    /// Colour presets the device lists. Empty when it lists none.
    fn presets(&self) -> Vec<u16> {
        Vec::new()
    }
    /// How long until the module wants `tick` without a job, for work on a clock such as a revert.
    fn wake(&self) -> Option<Duration> {
        None
    }
    fn tick(&mut self) -> Result<(), String> {
        Ok(())
    }
    /// Validates `value` and writes it. The module owns its device's safety rules.
    fn set(&mut self, control: ControlId, value: ControlValue) -> Result<(), String>;
    /// After sleep or a device change: write the saved state to the device again.
    fn reapply(&mut self) -> Result<(), String>;
}

enum Job {
    Start,
    Set(ControlId, ControlValue),
    Reapply,
    /// Not queued: the module's `wake` ran out.
    Tick,
}

#[derive(Default)]
struct Shared {
    name: String,
    controls: Vec<Control>,
    profiles: Vec<String>,
    modes: Vec<String>,
    presets: Vec<u16>,
    error: Option<String>,
    /// Start of the call in progress, if any.
    busy_since: Option<Instant>,
}

pub struct Host {
    id: DeviceId,
    deadline: Duration,
    shared: Arc<Mutex<Shared>>,
    tx: SyncSender<Job>,
    thread: std::thread::JoinHandle<()>,
}

impl Host {
    /// Ends the module after its queued jobs and waits up to `wait` for its thread, so the next owner of the
    /// device starts only once this one has let go, and the module's `Drop` (a display-mode revert) runs.
    /// False when the thread is still in a call: a hung module cannot be stopped, only left behind.
    pub fn stop(self, wait: Duration) -> bool {
        let Self { tx, thread, .. } = self;
        drop(tx);
        let start = Instant::now();
        while !thread.is_finished() {
            if start.elapsed() > wait {
                return false;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        true
    }

    pub fn spawn(module: impl Module) -> Self {
        Self::with_deadline(module, DEADLINE)
    }

    fn with_deadline(mut module: impl Module, deadline: Duration) -> Self {
        let id = module.id();
        let shared = Arc::new(Mutex::new(Shared::default()));
        let (tx, rx) = sync_channel(QUEUE);
        tx.try_send(Job::Start)
            .expect("an empty queue takes the first job");
        let state = shared.clone();
        let thread = std::thread::spawn(move || {
            loop {
                let job = match module.wake() {
                    None => rx.recv().map_err(|_| RecvTimeoutError::Disconnected),
                    Some(wait) => rx.recv_timeout(wait),
                };
                let job = match job {
                    Ok(job) => job,
                    Err(RecvTimeoutError::Timeout) => Job::Tick,
                    Err(RecvTimeoutError::Disconnected) => break,
                };
                let tick = matches!(job, Job::Tick);
                state.lock().unwrap().busy_since = Some(Instant::now());
                let result = match job {
                    Job::Start => module.start(),
                    Job::Set(control, value) => module.set(control, value),
                    Job::Reapply => module.reapply(),
                    Job::Tick => module.tick(),
                };
                if let Err(e) = &result {
                    crate::system::log::error(&format!("{id:?}: {e}"));
                }
                let (name, controls, profiles, modes, presets) = (
                    module.name(),
                    module.controls(),
                    module.profiles(),
                    module.modes(),
                    module.presets(),
                );
                let mut s = state.lock().unwrap();
                s.name = name;
                s.controls = controls;
                s.profiles = profiles;
                s.modes = modes;
                s.presets = presets;
                // A quiet tick keeps the last error on show; only its own failure replaces it.
                if !(tick && result.is_ok()) {
                    s.error = result.err();
                }
                s.busy_since = None;
            }
        });
        Self {
            id,
            deadline,
            shared,
            tx,
            thread,
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
            name: s.name.clone(),
            status: if error.is_some() {
                DeviceStatus::Failed
            } else {
                DeviceStatus::Ok
            },
            error,
            controls: s.controls.clone(),
            profiles: s.profiles.clone(),
            modes: s.modes.clone(),
            presets: s.presets.clone(),
            busy: s.busy_since.is_some(),
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

    #[test]
    fn stop_waits_for_the_thread_and_gives_up_on_a_hung_one() {
        let h = Host::spawn(Probe::default());
        assert!(h.stop(Duration::from_secs(2)), "an idle module stops");
        let probe = Probe::default();
        let hang = probe.hang.clone();
        let h = Host::spawn(probe);
        hang.store(true, Ordering::SeqCst);
        assert!(h.set(ControlId::Level, ControlValue::Int(1)));
        wait("busy", || h.device().busy);
        assert!(
            !h.stop(Duration::from_millis(50)),
            "a hung module is left behind"
        );
        hang.store(false, Ordering::SeqCst);
    }

    /// None until the module has started.
    fn level(h: &Host) -> Option<ControlValue> {
        h.device().controls.first().map(|c| c.value.clone())
    }

    #[test]
    fn a_set_shows_in_the_device_and_a_bad_value_marks_it_failed() {
        let h = Host::spawn(Probe::default());
        assert!(h.set(ControlId::Level, ControlValue::Int(40)));
        wait("level 40", || level(&h) == Some(ControlValue::Int(40)));
        assert_eq!(h.device().status, DeviceStatus::Ok);
        assert!(h.set(ControlId::Level, ControlValue::Int(900)));
        wait("failure shown", || {
            h.device().status == DeviceStatus::Failed
        });
        assert_eq!(level(&h), Some(ControlValue::Int(40)), "old value kept");
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
