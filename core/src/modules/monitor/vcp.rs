//! The VCP codes the DDC module may write, and the one guarded write every DDC write goes through.
//! An allowlist: a code not listed is never written, whatever a profile or the UI says. Factory resets
//! (0x04/05/06/08), power (0xD6/0xC6) and input source (0x60) are not in it, and no flag adds them.

use super::mode::Mode;

pub const BRIGHTNESS: u8 = 0x10;
pub const CONTRAST: u8 = 0x12;
/// Colour preset. CVTE scalers lock the RGB gains until it is set to User (11), so it is written first.
pub const PRESET: u8 = 0x14;
/// MCCS colour presets: 1 sRGB … 10 11500K, 11 to 13 User 1 to 3.
const PRESETS: u16 = 13;
/// Picture mode (AL258Q: 1 Standard … 9). It limits which colour presets the monitor takes, so a profile
/// writes it before the preset.
pub const PICTURE_MODE: u8 = 0xB6;
pub const RED: u8 = 0x16;
pub const GREEN: u8 = 0x18;
pub const BLUE: u8 = 0x1A;

/// Image settings: colour temperature request, brightness, contrast, preset, RGB gains, 6-axis saturation,
/// volume, black levels, 6-axis hue, picture mode.
#[rustfmt::skip]
const WRITABLE: [u8; 24] = [
    0x0C, BRIGHTNESS, CONTRAST, PRESET, RED, GREEN, BLUE,
    0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E,
    0x62, 0x6C, 0x6E, 0x70,
    0x9B, 0x9C, 0x9D, 0x9E, 0x9F, 0xA0,
    PICTURE_MODE,
];

pub fn writable(code: u8) -> bool {
    WRITABLE.contains(&code)
}

/// Codes kept as a percentage of the monitor's own maximum in the UI and in profiles, so 70 means the same
/// on a monitor that counts to 100 and one that counts to 255. Lists (preset, picture mode) and the colour
/// temperature request (steps of a size the monitor sets) stay raw.
pub fn scaled(code: u8) -> bool {
    writable(code) && ![0x0C, PRESET, PICTURE_MODE].contains(&code)
}

/// Codes whose values mean something only on one model: written only by a profile captured on that model.
/// MCCS defines 0xB6 as read only; the AL258Q uses it for its picture mode.
pub fn model_specific(code: u8) -> bool {
    code == PICTURE_MODE
}

pub fn to_percent(raw: u16, max: u16) -> u16 {
    if max == 0 {
        return 0;
    }
    ((u32::from(raw) * 100 + u32::from(max) / 2) / u32::from(max)) as u16
}

fn from_percent(percent: u16, max: u16) -> u16 {
    u16::try_from((u32::from(percent) * u32::from(max) + 50) / 100).unwrap_or(u16::MAX)
}

/// What a monitor's capabilities string says: its model, and every VCP code it lists with the values it offers
/// (empty for a code without a list).
#[derive(Default, Debug, PartialEq)]
pub struct Caps {
    pub model: Option<String>,
    pub values: std::collections::BTreeMap<u8, Vec<u16>>,
}

impl Caps {
    /// `(prot(monitor)model(AL258Q)vcp(02 10 12 14(05 06 08 0B) 16)...)`. Codes may come without spaces
    /// between them (`vcp(101214)`); anything unreadable is left out.
    pub fn parse(caps: &str) -> Self {
        let model = group(caps, "model(")
            .map(str::trim)
            .filter(|m| !m.is_empty())
            .map(str::to_owned);
        let mut values = std::collections::BTreeMap::new();
        let (mut rest, mut last) = (group(caps, "vcp(").unwrap_or(""), None);
        while let Some(c) = rest.chars().next() {
            if c == '(' {
                let end = rest.find(')').unwrap_or(rest.len());
                if let Some(code) = last.take() {
                    values.insert(code, hex_pairs(&rest[1..end]).map(u16::from).collect());
                }
                rest = rest.get(end + 1..).unwrap_or("");
            } else if let Some(code) = rest.get(..2).and_then(|h| u8::from_str_radix(h, 16).ok()) {
                values.entry(code).or_insert_with(Vec::new);
                last = Some(code);
                rest = &rest[2..];
            } else {
                rest = &rest[c.len_utf8()..];
            }
        }
        Self { model, values }
    }
}

/// The text inside `key(...)`, nested brackets included.
fn group<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let start = text.find(key)? + key.len();
    let mut depth = 1;
    for (i, c) in text[start..].char_indices() {
        depth += match c {
            '(' => 1,
            ')' => -1,
            _ => 0,
        };
        if depth == 0 {
            return Some(&text[start..start + i]);
        }
    }
    None
}

fn hex_pairs(text: &str) -> impl Iterator<Item = u8> + '_ {
    text.split_whitespace().flat_map(|word| {
        (0..word.len() / 2)
            .filter_map(move |i| u8::from_str_radix(word.get(2 * i..2 * i + 2)?, 16).ok())
    })
}

/// How the core reaches one monitor: DDC/CI for its settings, Windows for its display mode. The real one is
/// `ddc::Physical`; tests use `Fake`.
pub trait Bus: Send + 'static {
    /// Current and maximum value.
    fn read(&mut self, code: u8) -> Result<(u16, u16), String>;
    /// Unchecked. Only `write` may call it.
    fn write_raw(&mut self, code: u8, value: u16) -> Result<(), String>;
    /// The raw capabilities string, if the monitor reports one.
    fn capabilities(&mut self) -> Option<String> {
        None
    }
    /// Modes Windows lists for the display, best first.
    fn modes(&mut self) -> Vec<Mode> {
        Vec::new()
    }
    fn mode(&mut self) -> Option<Mode> {
        None
    }
    /// Quarter turns clockwise from landscape (`mode::turn`).
    fn turn(&mut self) -> Option<u8> {
        None
    }
    /// Only `Monitor` calls it, with a mode from `modes` sized for `turn`. `keep` saves it past this session.
    fn set_mode(&mut self, _: Mode, _turn: u8, _keep: bool) -> Result<(), String> {
        Err("this display has no mode to set".to_owned())
    }
    /// The GPU gamma ramp of the display this monitor shows.
    fn set_gamma(&mut self, _: &super::gamma::Ramp) -> Result<(), String> {
        Err("this display has no gamma ramp".to_owned())
    }
}

#[derive(Debug)]
pub enum WriteError {
    /// The monitor does not answer for this code: nothing was sent.
    Unsupported(u8),
    Failed(String),
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Unsupported(code) => write!(f, "0x{code:02X} is not supported by this monitor"),
            Self::Failed(e) => f.write_str(e),
        }
    }
}

/// The only way to write a VCP code. `level` is a percentage of the monitor's maximum for a `scaled` code,
/// the raw value otherwise. Reads the code first, so an unsupported code or a value above the monitor's
/// own maximum is refused before anything is sent.
pub fn write(bus: &mut dyn Bus, code: u8, level: u16) -> Result<(), WriteError> {
    if !writable(code) {
        return Err(WriteError::Failed(format!(
            "0x{code:02X} is not a setting SysVane writes"
        )));
    }
    if scaled(code) && level > 100 {
        return Err(WriteError::Failed(format!(
            "0x{code:02X} = {level}% is above 100%"
        )));
    }
    let (_, max) = bus.read(code).map_err(|_| WriteError::Unsupported(code))?;
    let value = if scaled(code) {
        from_percent(level, max)
    } else {
        level
    };
    // 0x14 is a list, not a range: the "maximum" a monitor reports for it means nothing (the AL258Q says 5
    // and takes 11, User 1). Only the MCCS presets 1 (sRGB) to 13 (User 3) are allowed.
    let allowed = if code == PRESET {
        (1..=PRESETS).contains(&value)
    } else {
        value <= max
    };
    if !allowed {
        return Err(WriteError::Failed(format!(
            "0x{code:02X} = {value} is above the maximum {max}"
        )));
    }
    bus.write_raw(code, value)
        .map_err(|e| WriteError::Failed(format!("0x{code:02X}: {e}")))
}

/// Monitor stand-in: holds values in memory and records every raw write.
#[cfg(test)]
#[derive(Default)]
pub struct Fake {
    pub values: std::collections::BTreeMap<u8, (u16, u16)>,
    pub writes: Vec<(u8, u16)>,
    /// Codes whose write fails, as a monitor that ignores a code would.
    pub broken: Vec<u8>,
    pub modes: Vec<Mode>,
    pub mode: Option<Mode>,
    /// Every mode set, with whether it was kept.
    pub mode_writes: Vec<(Mode, bool)>,
    pub turn: u8,
    pub caps: Option<String>,
    /// Every gamma ramp written.
    pub ramps: Vec<super::gamma::Ramp>,
}

#[cfg(test)]
impl Fake {
    /// Brightness, contrast, preset 5 and RGB gains at 50, as the AL258Q reports them.
    pub fn monitor() -> Self {
        let mut f = Self::default();
        for (code, value, max) in [
            (BRIGHTNESS, 70, 100),
            (CONTRAST, 50, 100),
            (PRESET, 5, 13),
            (PICTURE_MODE, 3, 9),
            (RED, 50, 100),
            (GREEN, 50, 100),
            (BLUE, 50, 100),
        ] {
            f.values.insert(code, (value, max));
        }
        f
    }
}

#[cfg(test)]
impl Bus for Fake {
    fn read(&mut self, code: u8) -> Result<(u16, u16), String> {
        self.values
            .get(&code)
            .copied()
            .ok_or_else(|| "unsupported".to_owned())
    }
    fn write_raw(&mut self, code: u8, value: u16) -> Result<(), String> {
        self.writes.push((code, value));
        if self.broken.contains(&code) {
            return Err("no reply".to_owned());
        }
        self.values.entry(code).or_insert((0, u16::MAX)).0 = value;
        Ok(())
    }
    fn capabilities(&mut self) -> Option<String> {
        self.caps.clone()
    }
    fn modes(&mut self) -> Vec<Mode> {
        self.modes.clone()
    }
    fn mode(&mut self) -> Option<Mode> {
        self.mode
    }
    fn turn(&mut self) -> Option<u8> {
        self.mode.map(|_| self.turn)
    }
    fn set_mode(&mut self, m: Mode, turn: u8, keep: bool) -> Result<(), String> {
        self.mode_writes.push((m, keep));
        self.mode = Some(m);
        self.turn = turn;
        Ok(())
    }
    fn set_gamma(&mut self, ramp: &super::gamma::Ramp) -> Result<(), String> {
        self.ramps.push(*ramp);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resets_power_and_input_are_never_written() {
        let mut bus = Fake::monitor();
        for code in [0x04, 0x05, 0x06, 0x08, 0xD6, 0xC6, 0x60] {
            bus.values.insert(code, (1, 255));
            for value in [0, 1, 2, 4, 5, 90] {
                assert!(
                    write(&mut bus, code, value).is_err(),
                    "0x{code:02X}={value}"
                );
            }
        }
        for code in 0..=u8::MAX {
            if writable(code) {
                assert!(
                    ![0x04, 0x05, 0x06, 0x08, 0xD6, 0xC6, 0x60].contains(&code),
                    "0x{code:02X} in the allowlist"
                );
            }
        }
        assert!(bus.writes.is_empty());
    }

    #[test]
    fn a_value_is_checked_against_what_the_monitor_reports() {
        let mut bus = Fake::monitor();
        assert!(write(&mut bus, BRIGHTNESS, 101).is_err(), "above max");
        assert!(write(&mut bus, 0x62, 10).is_err(), "volume not reported");
        assert!(write(&mut bus, 0xE0, 1).is_err(), "manufacturer code");
        write(&mut bus, BRIGHTNESS, 40).unwrap();
        assert_eq!(bus.writes, [(BRIGHTNESS, 40)]);
        assert_eq!(bus.values[&BRIGHTNESS].0, 40);
    }

    #[test]
    fn a_preset_is_checked_against_the_mccs_list_not_the_reported_maximum() {
        let mut bus = Fake::monitor();
        bus.values.insert(PRESET, (5, 5)); // what the AL258Q reports
        write(&mut bus, PRESET, 11).unwrap();
        assert!(write(&mut bus, PRESET, 0).is_err());
        assert!(write(&mut bus, PRESET, 14).is_err());
        assert_eq!(bus.writes, [(PRESET, 11)]);
    }

    #[test]
    fn a_level_is_a_percentage_of_the_monitors_maximum() {
        let mut bus = Fake::monitor();
        bus.values.insert(BRIGHTNESS, (0, 255));
        write(&mut bus, BRIGHTNESS, 70).unwrap();
        assert_eq!(bus.values[&BRIGHTNESS].0, 179);
        assert_eq!(to_percent(179, 255), 70);
        assert!(write(&mut bus, BRIGHTNESS, 101).is_err());
        write(&mut bus, PRESET, 11).unwrap();
        assert_eq!(bus.values[&PRESET].0, 11, "a list code stays raw");
        assert_eq!(to_percent(5, 0), 0);
    }

    #[test]
    fn capabilities_give_the_model_and_the_listed_values() {
        let caps = Caps::parse(
            "(prot(monitor)type(LCD)model(AL258Q)cmds(01 02 03)vcp(02 10 12 14(05 06 08 0B) 16 18 1A 60( 0F 10 ) B6)mccs_ver(2.2))",
        );
        assert_eq!(caps.model.as_deref(), Some("AL258Q"));
        assert_eq!(caps.values[&PRESET], [5, 6, 8, 11]);
        assert_eq!(caps.values[&0x60], [15, 16]);
        assert!(caps.values[&BRIGHTNESS].is_empty(), "listed without values");
        assert_eq!(caps.values.len(), 9);
        assert_eq!(Caps::parse("vcp(101214(0105)16)").values[&PRESET], [1, 5]);
        assert_eq!(Caps::parse("junk(("), Caps::default());
    }
}
