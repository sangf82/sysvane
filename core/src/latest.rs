//! The newest reading from a source that runs in another process. A reading older than
//! `STALE_AFTER` counts as no data, so a stuck worker can never show a frozen number.

use serde::de::DeserializeOwned;
use std::io::{BufRead, BufReader, Read};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use sysvane_wire::TICK;

/// A worker that has missed three ticks in a row.
const STALE_AFTER: Duration = Duration::from_secs(3 * TICK.as_secs());

pub struct Latest<T>(Arc<Mutex<Option<(Instant, T)>>>);

impl<T> Clone for Latest<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Clone + DeserializeOwned> Latest<T> {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }

    /// Takes one line from the source: JSON of `Option<T>`, where `null` means unreadable.
    /// A line that does not parse also clears the reading.
    pub fn update(&self, line: &str) {
        let value = serde_json::from_str::<Option<T>>(line).ok().flatten();
        *self.0.lock().unwrap() = value.map(|v| (Instant::now(), v));
    }

    /// Feeds `update` from `source`, line by line, until it ends or `go` returns false.
    pub fn follow(&self, source: impl Read, go: impl Fn() -> bool) {
        for line in BufReader::new(source).lines().map_while(Result::ok) {
            self.update(&line);
            if !go() {
                break;
            }
        }
    }

    /// The last reading, if it is fresh. Never blocks on the source.
    pub fn get(&self) -> Option<T> {
        self.get_at(Instant::now())
    }

    fn get_at(&self, now: Instant) -> Option<T> {
        let slot = self.0.lock().unwrap();
        slot.as_ref()
            .filter(|(at, _)| now.duration_since(*at) < STALE_AFTER)
            .map(|(_, v)| v.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn null_and_garbage_clear_the_reading() {
        let latest = Latest::<u32>::new();
        assert_eq!(latest.get(), None, "nothing before the first line");
        latest.update("7");
        assert_eq!(latest.get(), Some(7));
        latest.update("null");
        assert_eq!(latest.get(), None);
        latest.update("8");
        latest.update("not json");
        assert_eq!(latest.get(), None);
    }

    #[test]
    fn an_old_reading_is_no_data() {
        let latest = Latest::<u32>::new();
        latest.update("7");
        let now = Instant::now();
        assert_eq!(latest.get_at(now + STALE_AFTER / 2), Some(7));
        assert_eq!(latest.get_at(now + STALE_AFTER * 2), None);
    }

    #[test]
    fn follow_keeps_the_last_line_and_stops_when_told() {
        let latest = Latest::<u32>::new();
        latest.follow(Cursor::new("1\n2\n3\n"), || true);
        assert_eq!(latest.get(), Some(3));
        let once = Latest::<u32>::new();
        once.follow(Cursor::new("1\n2\n3\n"), || false);
        assert_eq!(once.get(), Some(1));
    }

    #[test]
    fn clones_share_one_reading() {
        let latest = Latest::<u32>::new();
        latest.clone().update("5");
        assert_eq!(latest.get(), Some(5));
    }
}
