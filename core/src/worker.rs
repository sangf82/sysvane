//! Readings that must not live in the core process run in a child of this exe: NVML leaks about
//! 20 MB it never gives back, and the PawnIO driver can hang. A child of an elevated core is
//! elevated too, so no separate helper is needed. The child writes one JSON line per tick on
//! stdout (`null` when unreadable); the core keeps the newest one and starts the child again if it dies.

use crate::latest::Latest;
use crate::mode::Mode;
use crate::win::hidden;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::io::Write;
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};
use sysvane_wire::TICK;

/// A worker that keeps dying is not started more often than this.
const RESPAWN_EVERY: Duration = Duration::from_secs(10);

/// Worker side. Returns when the parent goes away and the pipe breaks.
pub fn run<T: Serialize>(mut read: impl FnMut() -> Option<T>) {
    let mut out = std::io::stdout();
    loop {
        let line = serde_json::to_string(&read()).expect("a reading serializes");
        if writeln!(out, "{line}").and_then(|_| out.flush()).is_err() {
            return;
        }
        std::thread::sleep(TICK);
    }
}

/// Core side. Dropping it kills the worker.
pub struct Worker<T> {
    mode: Mode,
    child: Child,
    started: Instant,
    latest: Latest<T>,
}

fn spawn<T: Clone + DeserializeOwned + Send + 'static>(
    mode: Mode,
    latest: &Latest<T>,
) -> Option<Child> {
    let mut child = hidden(std::env::current_exe().ok()?)
        .arg(mode.flag())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let out = child.stdout.take()?;
    let slot = latest.clone();
    std::thread::spawn(move || slot.follow(out, || true));
    Some(child)
}

impl<T: Clone + DeserializeOwned + Send + 'static> Worker<T> {
    pub fn start(mode: Mode) -> Option<Self> {
        let latest = Latest::new();
        let child = spawn(mode, &latest)?;
        Some(Self {
            mode,
            child,
            started: Instant::now(),
            latest,
        })
    }

    /// The newest fresh reading. A worker that died is started again, at most once per `RESPAWN_EVERY`.
    pub fn latest(&mut self) -> Option<T> {
        let died = self.child.try_wait().ok().flatten().is_some();
        if died
            && self.started.elapsed() >= RESPAWN_EVERY
            && let Some(child) = spawn(self.mode, &self.latest)
        {
            crate::log::info(&format!(
                "worker {} died; started a new one",
                self.mode.flag()
            ));
            self.child = child;
            self.started = Instant::now();
        }
        self.latest.get()
    }
}

impl<T> Drop for Worker<T> {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
