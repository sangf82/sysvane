//! Fan-out of snapshots to connected clients. Sampling runs only while someone listens.

use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

type Line = Arc<str>;

/// A client that falls this many lines behind is dropped, so one that stops reading cannot grow memory.
const CLIENT_BACKLOG: usize = 4;

#[derive(Default)]
pub struct Hub {
    clients: Mutex<Vec<SyncSender<Line>>>,
    wake: Condvar,
}

impl Hub {
    pub fn subscribe(&self) -> Receiver<Line> {
        let (tx, rx) = sync_channel(CLIENT_BACKLOG);
        self.clients.lock().unwrap().push(tx);
        self.wake.notify_all();
        rx
    }

    /// Sends to every client and drops the ones that hung up or fell behind. Returns how many remain.
    pub fn broadcast(&self, line: &Line) -> usize {
        let mut clients = self.clients.lock().unwrap();
        clients.retain(|c| c.try_send(line.clone()).is_ok());
        clients.len()
    }

    #[cfg(test)]
    pub fn has_clients(&self) -> bool {
        !self.clients.lock().unwrap().is_empty()
    }

    fn wait_for_client(&self) {
        let mut clients = self.clients.lock().unwrap();
        while clients.is_empty() {
            clients = self.wake.wait(clients).unwrap();
        }
    }

    /// Parks with no clients. `make` builds the sampler on wake-up and it is dropped
    /// when the last client leaves, so an idle core holds no sensor state.
    pub fn run<S>(
        &self,
        interval: Duration,
        mut make: impl FnMut() -> S,
        mut sample: impl FnMut(&mut S, f64) -> String,
    ) -> ! {
        loop {
            self.session(interval, &mut make, &mut sample);
        }
    }

    /// One listening period: waits for a client, then samples until the last one leaves.
    /// `sample` gets the real seconds since the previous tick and returns one JSON line.
    pub fn session<S>(
        &self,
        interval: Duration,
        make: &mut impl FnMut() -> S,
        sample: &mut impl FnMut(&mut S, f64) -> String,
    ) {
        self.wait_for_client();
        let mut state = make();
        let mut last = Instant::now();
        loop {
            std::thread::sleep(interval);
            let now = Instant::now();
            let line: Line = sample(&mut state, now.duration_since(last).as_secs_f64()).into();
            last = now;
            if self.broadcast(&line) == 0 {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc::channel;

    const WAIT: Duration = Duration::from_secs(5);

    /// Runs one session on a thread. The receiver gets a message when the session ends.
    fn session_in_background(hub: &Arc<Hub>, made: &Arc<AtomicUsize>) -> Receiver<()> {
        let (hub, made) = (hub.clone(), made.clone());
        let (done, finished) = channel();
        std::thread::spawn(move || {
            hub.session(
                Duration::from_millis(5),
                &mut || {
                    made.fetch_add(1, Ordering::SeqCst);
                    0u32
                },
                &mut |n, _| {
                    *n += 1;
                    n.to_string()
                },
            );
            let _ = done.send(());
        });
        finished
    }

    #[test]
    fn dropped_client_is_removed() {
        let hub = Hub::default();
        let a = hub.subscribe();
        let b = hub.subscribe();
        drop(b);
        assert_eq!(hub.broadcast(&"x".into()), 1);
        assert_eq!(&*a.recv().unwrap(), "x");
        drop(a);
        assert_eq!(hub.broadcast(&"y".into()), 0);
    }

    #[test]
    fn a_client_that_stops_reading_is_dropped() {
        let hub = Hub::default();
        let _stalled = hub.subscribe();
        for _ in 0..CLIENT_BACKLOG {
            assert_eq!(hub.broadcast(&"x".into()), 1);
        }
        assert_eq!(hub.broadcast(&"x".into()), 0, "the backlog is full");
    }

    #[test]
    fn each_session_builds_a_fresh_sampler_and_ends_with_its_last_client() {
        let hub = Arc::new(Hub::default());
        let made = Arc::new(AtomicUsize::new(0));
        for round in 1..=2 {
            let finished = session_in_background(&hub, &made);
            assert_eq!(
                made.load(Ordering::SeqCst),
                round - 1,
                "no sampler before a client subscribes"
            );
            let rx = hub.subscribe();
            assert_eq!(
                &*rx.recv_timeout(WAIT).unwrap(),
                "1",
                "round {round} starts from a fresh sampler"
            );
            drop(rx);
            finished
                .recv_timeout(WAIT)
                .expect("the session ends when its last client leaves");
        }
        assert_eq!(made.load(Ordering::SeqCst), 2);
    }
}
