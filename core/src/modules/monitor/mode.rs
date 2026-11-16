//! Resolution and refresh rate, set through Windows as Settings > Display does. Only a mode the display
//! itself lists is accepted. A new mode is a trial: Windows holds it without saving it, and the monitor
//! module reverts it after `TRIAL` unless the user keeps it, so a mode the monitor cannot show comes back
//! without the user having to see the screen.

use std::fmt;
use std::str::FromStr;
use std::time::Duration;
use windows_sys::Win32::Foundation::{LPARAM, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    CDS_TEST, CDS_UPDATEREGISTRY, ChangeDisplaySettingsExW, DEVMODEW, DISP_CHANGE_SUCCESSFUL,
    DM_DISPLAYFREQUENCY, DM_DISPLAYORIENTATION, DM_INTERLACED, DM_PELSHEIGHT, DM_PELSWIDTH,
    ENUM_CURRENT_SETTINGS, EnumDisplayMonitors, EnumDisplaySettingsW, GetMonitorInfoW, HDC,
    HMONITOR, MONITORINFO, MONITORINFOEXW,
};
use windows_sys::core::BOOL;

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
    let change = |flags| {
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
        (r == DISP_CHANGE_SUCCESSFUL)
            .then_some(())
            .ok_or_else(|| format!("Windows refused {m} (code {r})"))
    };
    change(CDS_TEST)?;
    change(flags)?;
    // Once (2026-11-16) a revert to landscape was reported done while the desktop stayed in portrait, so
    // the desktop's own shape is checked and the change made once more if it differs.
    let portrait = m.height > m.width;
    if desktop_portrait(device).is_some_and(|p| p != portrait) {
        crate::system::log::error(&format!(
            "{m}: the desktop did not follow; setting it again"
        ));
        change(flags)?;
        if desktop_portrait(device).is_some_and(|p| p != portrait) {
            return Err(format!("Windows took {m} but the desktop did not change"));
        }
    }
    Ok(())
}

/// Whether the desktop on this display is taller than wide, from the monitor rectangle Windows lays out.
fn desktop_portrait(device: &[u16]) -> Option<bool> {
    unsafe extern "system" fn each(m: HMONITOR, _: HDC, _: *mut RECT, found: LPARAM) -> BOOL {
        // SAFETY: `found` is the `Vec` passed below, alive for the whole enumeration.
        unsafe { (*(found as *mut Vec<HMONITOR>)).push(m) };
        1
    }
    let mut monitors: Vec<HMONITOR> = Vec::new();
    // SAFETY: the callback only pushes into `monitors`, which outlives the call.
    unsafe {
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null(),
            Some(each),
            &mut monitors as *mut _ as LPARAM,
        )
    };
    let name = &device[..device.len().saturating_sub(1)];
    monitors.into_iter().find_map(|hm| {
        // SAFETY: MONITORINFOEXW is plain data; cbSize says which struct it is.
        let mut info: MONITORINFOEXW = unsafe { std::mem::zeroed() };
        info.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
        // SAFETY: `info` is a MONITORINFOEXW with cbSize set.
        if unsafe { GetMonitorInfoW(hm, &mut info as *mut _ as *mut MONITORINFO) } == 0 {
            return None;
        }
        let len = info.szDevice.iter().position(|&c| c == 0).unwrap_or(32);
        (&info.szDevice[..len] == name).then(|| {
            let r = info.monitorInfo.rcMonitor;
            r.bottom - r.top > r.right - r.left
        })
    })
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
