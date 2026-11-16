//! Monitor profiles: a name and a set of VCP values, stored as JSON. Defaults ship inside the exe and are
//! read only; the user's own live in `%ProgramData%\SysVane\config\monitor\`. A file or a name from the UI
//! is untrusted: a malformed entry is skipped, and only allowlisted codes reach `vcp::write`.

use super::vcp::{self, Bus};
use crate::modules::config;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub const SCHEMA: u32 = 1;
const MAX_NAME: usize = 48;

const DEFAULTS: [&str; 4] = [
    include_str!("profiles/Calibrated-Left.json"),
    include_str!("profiles/Calibrated-Right.json"),
    include_str!("profiles/Color-Accurate-D65.json"),
    include_str!("profiles/Gaming.json"),
];

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Profile {
    pub schema: u32,
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// The model it was captured on. Model-specific codes (`vcp::model_specific`) are written only to a
    /// monitor of this model; without one the profile is universal and they are skipped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// `"0x10": 70`: a percentage of the monitor's maximum for `vcp::scaled` codes, the raw value for the
    /// rest. Kept as raw JSON so one bad entry does not reject the whole file.
    pub settings: BTreeMap<String, serde_json::Value>,
}

impl Profile {
    pub fn parse(text: &str) -> Result<Self, String> {
        let p: Self = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if p.schema != SCHEMA {
            return Err(format!("schema {} (this version reads {SCHEMA})", p.schema));
        }
        Ok(p)
    }

    /// The well-formed entries: picture mode first (it decides which presets the monitor takes), then the
    /// colour preset (CVTE scalers ignore RGB gains until it is set), then the rest.
    pub fn entries(&self) -> Vec<(u8, u16)> {
        let mut entries: Vec<(u8, u16)> = self
            .settings
            .iter()
            .filter_map(|(key, value)| {
                let hex = key.strip_prefix("0x").or_else(|| key.strip_prefix("0X"))?;
                let code = u8::from_str_radix(hex, 16).ok()?;
                Some((code, u16::try_from(value.as_u64()?).ok()?))
            })
            .collect();
        entries.sort_by_key(|&(code, _)| match code {
            vcp::PICTURE_MODE => 0,
            // A colour-temperature request can switch some scalers to their temperature mode, so it goes
            // before the preset, never between the preset and the gains.
            0x0C => 1,
            vcp::PRESET => 2,
            _ => 3,
        });
        entries
    }

    /// Writes every allowlisted entry to a monitor of `model` and skips the rest (read-only codes a capture
    /// recorded, model-specific codes from another model). Writing nothing, or any write failing, is an
    /// error. A code this monitor does not support is skipped.
    pub fn apply(&self, bus: &mut dyn Bus, model: Option<&str>) -> Result<(), String> {
        let same_model = self.model.is_some() && self.model.as_deref() == model;
        let mut written = 0;
        let mut errors = Vec::new();
        for (code, value) in self
            .entries()
            .into_iter()
            .filter(|(c, _)| vcp::writable(*c) && (same_model || !vcp::model_specific(*c)))
        {
            match vcp::write(bus, code, value) {
                Ok(()) => written += 1,
                Err(vcp::WriteError::Unsupported(_)) => {}
                Err(e) => errors.push(e.to_string()),
            }
        }
        match (written, errors.is_empty()) {
            (0, true) => Err(format!("{} has no setting SysVane writes", self.name)),
            (_, true) => Ok(()),
            _ => Err(format!("{}: {}", self.name, errors.join("; "))),
        }
    }

    /// Reads every writable code the monitor answers for.
    pub fn capture(name: &str, bus: &mut dyn Bus, model: Option<&str>) -> Result<Self, String> {
        let name = clean_name(name)?;
        let settings: BTreeMap<String, serde_json::Value> = (0..=u8::MAX)
            .filter(|c| vcp::writable(*c))
            .filter_map(|c| {
                let (value, max) = bus.read(c).ok()?;
                let level = if vcp::scaled(c) {
                    vcp::to_percent(value, max)
                } else {
                    value
                };
                Some((format!("0x{c:02X}"), level.into()))
            })
            .collect();
        if settings.is_empty() {
            return Err("the monitor answered no setting".to_owned());
        }
        Ok(Self {
            schema: SCHEMA,
            name,
            description: String::new(),
            model: model.map(str::to_owned),
            settings,
        })
    }
}

/// A name the UI sent, made safe to show and to use as a file name. Refused when nothing is left.
fn clean_name(name: &str) -> Result<String, String> {
    let kept: String = name
        .chars()
        .filter(|c| !c.is_control() && !r#"<>:"/\|?*"#.contains(*c))
        .take(MAX_NAME)
        .collect();
    let kept = kept.trim().trim_end_matches('.').trim().to_owned();
    let stem = kept.split('.').next().unwrap_or("").to_ascii_uppercase();
    let reserved = ["CON", "PRN", "AUX", "NUL"].contains(&stem.as_str())
        || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.len() == 4
            && stem.ends_with(|c: char| c.is_ascii_digit()));
    if kept.is_empty() || reserved {
        return Err(format!("{name:?} is not a usable profile name"));
    }
    Ok(kept)
}

pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// A file of the monitor module's own next to the profiles. Not `.json`, so it is never read as a profile.
    pub fn file(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    fn defaults() -> impl Iterator<Item = Profile> {
        DEFAULTS
            .iter()
            .map(|t| Profile::parse(t).expect("a shipped profile parses"))
    }

    /// The user's files. A corrupt one is logged and left out.
    fn saved(&self) -> Vec<Profile> {
        let Ok(dir) = std::fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        dir.flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .filter_map(|p| {
                let text = std::fs::read_to_string(&p).ok()?;
                Profile::parse(&text)
                    .inspect_err(|e| crate::system::log::error(&format!("{}: {e}", p.display())))
                    .ok()
            })
            .collect()
    }

    /// Shipped profiles first, then the user's, each group sorted.
    pub fn names(&self) -> Vec<String> {
        let mut defaults: Vec<String> = Self::defaults().map(|p| p.name).collect();
        let mut saved: Vec<String> = self.saved().into_iter().map(|p| p.name).collect();
        defaults.sort();
        saved.sort();
        defaults.extend(saved);
        defaults
    }

    pub fn get(&self, name: &str) -> Result<Profile, String> {
        Self::defaults()
            .chain(self.saved())
            .find(|p| p.name == name)
            .ok_or_else(|| format!("no profile named {name:?}"))
    }

    /// Saves under the profile's (already cleaned) name. A shipped profile cannot be overwritten.
    pub fn save(&self, profile: &Profile) -> Result<(), String> {
        if Self::defaults().any(|p| p.name.eq_ignore_ascii_case(&profile.name)) {
            return Err(format!(
                "{} is a built-in profile; pick another name",
                profile.name
            ));
        }
        let json = serde_json::to_string_pretty(profile).map_err(|e| e.to_string())?;
        config::write_atomic(
            &self.dir.join(format!("{}.json", profile.name)),
            json.as_bytes(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::monitor::vcp::{BLUE, BRIGHTNESS, Fake, PRESET, RED};

    fn profile(settings: &str) -> Profile {
        Profile::parse(&format!(
            r#"{{"schema":1,"name":"t","settings":{settings}}}"#
        ))
        .unwrap()
    }

    #[test]
    fn shipped_profiles_parse_and_skip_power_and_input() {
        for p in Store::defaults() {
            let mut bus = Fake::monitor();
            for code in [0x60, 0xD6, 0xC6] {
                bus.values.insert(code, (1, 255));
            }
            let _ = p.apply(&mut bus, Some("CVTE"));
            assert!(
                bus.writes.iter().all(|(c, _)| vcp::writable(*c)),
                "{}: {:?}",
                p.name,
                bus.writes
            );
        }
    }

    #[test]
    fn picture_mode_then_preset_are_written_before_the_gains() {
        let mut bus = Fake::monitor();
        let mut p = profile(r#"{"0x16":47,"0x10":70,"0x14":11,"0xB6":1,"0x1A":51}"#);
        p.model = Some("AL258Q".into());
        p.apply(&mut bus, Some("AL258Q")).unwrap();
        assert_eq!(bus.writes[..2], [(vcp::PICTURE_MODE, 1), (PRESET, 11)]);
        assert_eq!(bus.values[&RED].0, 47);
        assert_eq!(bus.values[&BLUE].0, 51);
    }

    #[test]
    fn model_specific_codes_reach_only_the_same_model() {
        let p = profile(r#"{"0x14":11,"0xB6":1}"#);
        for (profile_model, monitor_model) in
            [(None, None), (None, Some("X")), (Some("Y"), Some("X"))]
        {
            let mut p = p.clone();
            p.model = profile_model.map(str::to_owned);
            let mut bus = Fake::monitor();
            p.apply(&mut bus, monitor_model).unwrap();
            assert_eq!(
                bus.writes,
                [(PRESET, 11)],
                "{profile_model:?} on {monitor_model:?}"
            );
        }
    }

    #[test]
    fn levels_follow_the_monitors_maximum_both_ways() {
        let mut bus = Fake::monitor();
        bus.values.insert(BRIGHTNESS, (179, 255));
        let p = Profile::capture("Wide", &mut bus, None).unwrap();
        assert_eq!(p.settings["0x10"], 70, "stored as a percentage");
        assert_eq!(p.settings["0x14"], 5, "a list code stays raw");
        let mut narrow = Fake::monitor();
        p.apply(&mut narrow, None).unwrap();
        assert_eq!(narrow.values[&BRIGHTNESS].0, 70);
    }

    #[test]
    fn malformed_entries_are_skipped_and_an_empty_apply_is_an_error() {
        let p =
            profile(r#"{"0x10":"high","16":5,"0xZZ":1,"0x12":-3,"0x14":70000,"0x04":1,"0x60":15}"#);
        assert!(p.entries().iter().all(|(c, _)| [0x04, 0x60].contains(c)));
        let mut bus = Fake::monitor();
        assert!(p.apply(&mut bus, None).is_err());
        assert!(bus.writes.is_empty());
        assert!(Profile::parse(r#"{"schema":2,"name":"x","settings":{}}"#).is_err());
        assert!(Profile::parse(r#"{"name":"x","monitors":[]}"#).is_err());
    }

    #[test]
    fn a_failed_write_is_reported() {
        let mut bus = Fake::monitor();
        bus.broken.push(RED);
        let e = profile(r#"{"0x10":60,"0x16":40}"#)
            .apply(&mut bus, None)
            .unwrap_err();
        assert!(e.contains("no reply"), "{e}");
        assert_eq!(bus.values[&BRIGHTNESS].0, 60, "the other write still lands");
    }

    #[test]
    fn names_from_the_ui_are_cleaned() {
        assert_eq!(clean_name(" My/Game:Mode? ").unwrap(), "MyGameMode");
        assert_eq!(
            clean_name("x".repeat(100).as_str()).unwrap().len(),
            MAX_NAME
        );
        for bad in ["", "  ", "...", "CON", "com1", "Lpt9.txt", "\u{7}"] {
            assert!(clean_name(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn a_capture_saves_loads_and_cannot_replace_a_default() {
        let dir = std::env::temp_dir().join(format!("sysvane-profiles-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let store = Store::new(dir.clone());
        let mut bus = Fake::monitor();
        let p = Profile::capture("Night", &mut bus, Some("CVTE")).unwrap();
        assert_eq!(p.model.as_deref(), Some("CVTE"));
        assert_eq!(p.settings["0x10"], 70);
        store.save(&p).unwrap();
        assert_eq!(store.get("Night").unwrap(), p);
        assert!(store.names().ends_with(&["Night".to_owned()]));
        std::fs::write(dir.join("bad.json"), "{").unwrap();
        assert_eq!(
            store.names().len(),
            DEFAULTS.len() + 1,
            "corrupt file left out"
        );
        let mut default = p.clone();
        default.name = "gaming".to_owned();
        assert!(store.save(&default).is_err());
        let _ = std::fs::remove_dir_all(dir);
    }
}
