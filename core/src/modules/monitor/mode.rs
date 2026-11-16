//! Resolution and refresh rate, set through Windows as Settings > Display does. Only a mode the display
//! itself lists is accepted. A new mode is a trial: Windows holds it without saving it, and the monitor
//! module reverts it after `TRIAL` unless the user keeps it, so a mode the monitor cannot show comes back
//! without the user having to see the screen.

use std::fmt;
use std::str::FromStr;
use std::time::Duration;
use windows_sys::Win32::Graphics::Gdi::{
    CDS_TEST, CDS_UPDATEREGISTRY, ChangeDisplaySettingsExW, DEVMODEW, DISP_CHANGE_SUCCESSFUL,
    DM_DISPLAYFREQUENCY, DM_DISPLAYORIENTATION, DM_INTERLACED, DM_PELSHEIGHT, DM_PELSWIDTH,
    ENUM_CURRENT_SETTINGS, EnumDisplaySettingsW,
};

/// As long as Windows Settings waits before it reverts.
pub const TRIAL: Duration = Duration::from_secs(15);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Mode {
    pub width: u32,
    pub height: u32,
    pub hz: u32,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}x{}@{}", self.width, self.height, self.hz)
    }
}

impl FromStr for Mode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        let bad = || format!("{s:?} is not a display mode");
        let (size, hz) = s.split_once('@').ok_or_else(bad)?;
        let (w, h) = size.split_once('x').ok_or_else(bad)?;
        let n = |v: &str| v.parse::<u32>().map_err(|_| bad());
        Ok(Self {
            width: n(w)?,
            height: n(h)?,
            hz: n(hz)?,
        })
    }
}

fn devmode() -> DEVMODEW {
    // SAFETY: DEVMODEW is plain data; all-zero is valid once dmSize is set.
    let mut dm: DEVMODEW = unsafe { std::mem::zeroed() };
    dm.dmSize = size_of::<DEVMODEW>() as u16;
    dm
}

fn read(device: &[u16], index: u32) -> Option<DEVMODEW> {
    let mut dm = devmode();
    // SAFETY: `device` is a NUL-terminated GDI name; `dm` has dmSize set.
    (unsafe { EnumDisplaySettingsW(device.as_ptr(), index, &mut dm) } != 0).then_some(dm)
}

fn mode(dm: &DEVMODEW) -> Mode {
    Mode {
        width: dm.dmPelsWidth,
        height: dm.dmPelsHeight,
        hz: dm.dmDisplayFrequency,
    }
}

pub fn current(device: &[u16]) -> Option<Mode> {
    read(device, ENUM_CURRENT_SETTINGS).map(|dm| mode(&dm))
}

/// Quarter turns clockwise from landscape: 0 landscape, 1 portrait, 2 landscape flipped, 3 portrait flipped.
pub fn turn(device: &[u16]) -> Option<u8> {
    // SAFETY: the display half of the union, which a display mode fills.
    read(device, ENUM_CURRENT_SETTINGS)
        .map(|dm| unsafe { dm.Anonymous1.Anonymous2.dmDisplayOrientation } as u8)
}

/// Progressive modes at the current colour depth, largest and fastest first.
pub fn list(device: &[u16]) -> Vec<Mode> {
    let Some(now) = read(device, ENUM_CURRENT_SETTINGS) else {
        return Vec::new();
    };
    let mut modes: Vec<Mode> = (0..)
        .map_while(|i| read(device, i))
        // SAFETY: dmDisplayFlags is the display half of the union, which a display mode fills.
        .filter(|dm| {
            dm.dmBitsPerPel == now.dmBitsPerPel
                && unsafe { dm.Anonymous2.dmDisplayFlags } & DM_INTERLACED == 0
                && dm.dmDisplayFrequency > 1
        })
        .map(|dm| mode(&dm))
        .collect();
    modes.sort_unstable_by(|a, b| b.cmp(a));
    modes.dedup();
    modes
}

/// `m` is in the size `turn` gives (portrait swaps width and height). `keep` saves it for the next sign-in;
/// without it Windows only holds it until it is changed again.
pub fn set(device: &[u16], m: Mode, turn: u8, keep: bool) -> Result<(), String> {
    let mut dm = devmode();
    dm.dmFields = DM_PELSWIDTH | DM_PELSHEIGHT | DM_DISPLAYFREQUENCY | DM_DISPLAYORIENTATION;
    dm.Anonymous1.Anonymous2.dmDisplayOrientation = turn.into();
    dm.dmPelsWidth = m.width;
    dm.dmPelsHeight = m.height;
    dm.dmDisplayFrequency = m.hz;
    let flags = if keep { CDS_UPDATEREGISTRY } else { 0 };
    for flags in [CDS_TEST, flags] {
        // SAFETY: valid NUL-terminated name and DEVMODEW; no window, no extra parameter.
        let r = unsafe {
            ChangeDisplaySettingsExW(
                device.as_ptr(),
                &dm,
                std::ptr::null_mut(),
                flags,
                std::ptr::null(),
            )
        };
        if r != DISP_CHANGE_SUCCESSFUL {
            return Err(format!("Windows refused {m} (code {r})"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mode_round_trips_and_junk_is_refused() {
        let m: Mode = "2560x1440@165".parse().unwrap();
        assert_eq!((m.width, m.height, m.hz), (2560, 1440, 165));
        assert_eq!(m.to_string(), "2560x1440@165");
        for bad in ["", "2560x1440", "2560@165", "ax1440@165", "2560x1440@-1"] {
            assert!(bad.parse::<Mode>().is_err(), "{bad}");
        }
    }
}
