//! Per-module settings in `%ProgramData%\SysVane\config\<name>.toml`. The file is data from disk, so a
//! missing, unreadable or malformed one gives the default and a log line; modules still validate the values.

use serde::{Serialize, de::DeserializeOwned};
use std::path::{Path, PathBuf};

pub fn dir() -> PathBuf {
    let base = std::env::var_os("ProgramData")
        .map_or_else(|| PathBuf::from(r"C:\ProgramData"), PathBuf::from);
    base.join("SysVane").join("config")
}

pub fn load<T: DeserializeOwned + Default>(path: &Path) -> T {
    match std::fs::read_to_string(path) {
        Ok(text) => toml::from_str(&text).unwrap_or_else(|e| {
            crate::log::error(&format!("{}: {e}; using defaults", path.display()));
            T::default()
        }),
        Err(_) => T::default(),
    }
}

pub fn save<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let text = toml::to_string(value).map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, text).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
    #[serde(deny_unknown_fields, default)]
    struct C {
        level: u8,
    }

    #[test]
    fn round_trips_and_falls_back_to_default() {
        let dir = std::env::temp_dir().join(format!("sysvane-config-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("x.toml");
        assert_eq!(load::<C>(&path), C::default(), "missing file");
        save(&path, &C { level: 7 }).unwrap();
        assert_eq!(load::<C>(&path), C { level: 7 });
        std::fs::write(&path, "level = \"loud\"").unwrap();
        assert_eq!(load::<C>(&path), C::default(), "malformed file");
        std::fs::write(&path, "level = 1\nrun = \"x\"").unwrap();
        assert_eq!(load::<C>(&path), C::default(), "unknown key");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
