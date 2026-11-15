//! Fake device: no hardware, one `level` control, saved in `fake.toml`. It is the reference for every later
//! module (config load, validation, re-apply) and what the UI is tested against. Debug builds only.

use crate::modules::Module;
use crate::modules::config;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use sysvane_wire::{Control, ControlId, ControlValue, DeviceId};

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(deny_unknown_fields, default)]
struct Config {
    level: u8,
}

pub struct Fake {
    path: PathBuf,
    level: u8,
    /// What the "hardware" holds; `reapply` writes `level` here again.
    device: u8,
}

impl Fake {
    pub fn new(path: PathBuf) -> Self {
        // A saved level out of range is a corrupt file, not a value to apply.
        let level = Some(config::load::<Config>(&path).level)
            .filter(|l| *l <= 100)
            .unwrap_or(0);
        Self {
            path,
            level,
            device: level,
        }
    }
}

impl Module for Fake {
    fn id(&self) -> DeviceId {
        DeviceId::Fake
    }

    fn controls(&self) -> Vec<Control> {
        vec![Control {
            id: ControlId::Level,
            value: ControlValue::Int(self.level.into()),
        }]
    }

    fn set(&mut self, control: ControlId, value: ControlValue) -> Result<(), String> {
        let (ControlId::Level, ControlValue::Int(n @ 0..=100)) = (control, value) else {
            return Err("level is a whole number from 0 to 100".to_owned());
        };
        self.level = n as u8;
        self.device = self.level;
        config::save(&self.path, &Config { level: self.level })
    }

    fn reapply(&mut self) -> Result<(), String> {
        self.device = self.level;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("sysvane-fake-{name}-{}.toml", std::process::id()))
    }

    #[test]
    fn the_level_is_validated_saved_and_restored() {
        let p = path("a");
        let _ = std::fs::remove_file(&p);
        let mut f = Fake::new(p.clone());
        assert!(f.set(ControlId::Level, ControlValue::Int(101)).is_err());
        assert!(f.set(ControlId::Level, ControlValue::Int(-1)).is_err());
        assert!(f.set(ControlId::Level, ControlValue::Bool(true)).is_err());
        assert!(f.set(ControlId::Level, ControlValue::Int(60)).is_ok());
        assert_eq!(Fake::new(p.clone()).level, 60);
        let _ = std::fs::remove_file(p);
    }

    #[test]
    fn a_corrupt_saved_level_is_ignored_and_reapply_rewrites_the_device() {
        let p = path("b");
        std::fs::write(&p, "level = 200").unwrap();
        assert_eq!(Fake::new(p.clone()).level, 0);
        let mut f = Fake::new(p.clone());
        f.level = 30;
        f.device = 0; // the device lost its state in sleep
        f.reapply().unwrap();
        assert_eq!(f.device, 30);
        let _ = std::fs::remove_file(p);
    }
}
