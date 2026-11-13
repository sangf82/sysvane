//! A small rotating log in `%ProgramData%\SysVane\logs`. The core has no console, so this is where
//! errors and device state changes go. Raw device commands are not logged. No telemetry: nothing leaves the PC.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_BYTES: u64 = 1024 * 1024;
const KEEP: u32 = 3;

static LOG: Mutex<Option<Log>> = Mutex::new(None);

struct Log {
    dir: PathBuf,
    file: File,
}

pub fn dir() -> PathBuf {
    let base = std::env::var_os("ProgramData")
        .map_or_else(|| PathBuf::from(r"C:\ProgramData"), PathBuf::from);
    base.join("SysVane").join("logs")
}

/// Opens the log. Without it, `error` and `info` do nothing.
pub fn init() {
    *LOG.lock().unwrap() = open(&dir());
}

pub fn info(message: &str) {
    write("INFO", message);
}

pub fn error(message: &str) {
    write("ERROR", message);
}

fn open(dir: &Path) -> Option<Log> {
    fs::create_dir_all(dir).ok()?;
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("sysvane.log"))
        .ok()?;
    Some(Log {
        dir: dir.to_owned(),
        file,
    })
}

fn write(level: &str, message: &str) {
    let mut guard = LOG.lock().unwrap();
    let Some(log) = guard.as_mut() else { return };
    let line = format!("{} {level} {message}\n", timestamp(SystemTime::now()));
    let _ = log.file.write_all(line.as_bytes());
    if log.file.metadata().is_ok_and(|m| m.len() >= MAX_BYTES) {
        rotate(&log.dir);
        if let Some(fresh) = open(&log.dir) {
            *log = fresh;
        }
    }
}

/// `sysvane.log` becomes `.1`, `.1` becomes `.2`, and the oldest is dropped.
fn rotate(dir: &Path) {
    let name = |n: u32| {
        dir.join(if n == 0 {
            "sysvane.log".to_owned()
        } else {
            format!("sysvane.log.{n}")
        })
    };
    let _ = fs::remove_file(name(KEEP));
    for n in (0..KEEP).rev() {
        let _ = fs::rename(name(n), name(n + 1));
    }
}

/// UTC as `2026-11-12T10:15:30Z`.
fn timestamp(now: SystemTime) -> String {
    let secs = now.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs()) as i64;
    let (days, rest) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Days since 1970-01-01 to a civil date (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn timestamps_are_utc_dates() {
        assert_eq!(timestamp(UNIX_EPOCH), "1970-01-01T00:00:00Z");
        assert_eq!(
            timestamp(UNIX_EPOCH + Duration::from_secs(1_782_000_000)),
            "2026-06-21T00:00:00Z"
        );
        assert_eq!(
            timestamp(UNIX_EPOCH + Duration::from_secs(951_782_399)),
            "2000-02-28T23:59:59Z"
        );
    }

    #[test]
    fn rotation_keeps_a_few_old_files() {
        let dir = std::env::temp_dir().join(format!("sysvane-log-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        for round in 0..6 {
            fs::write(dir.join("sysvane.log"), format!("round {round}")).unwrap();
            rotate(&dir);
        }
        let mut names: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        assert_eq!(names, ["sysvane.log.1", "sysvane.log.2", "sysvane.log.3"]);
        assert_eq!(
            fs::read_to_string(dir.join("sysvane.log.1")).unwrap(),
            "round 5"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
