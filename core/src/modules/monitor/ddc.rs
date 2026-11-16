//! Real monitors over DDC/CI, through dxva2. `detect` only lists handles and is quick; every I2C call
//! (`Bus` and `model`) runs on the module's own thread, where the host's watchdog sees a hang.

use super::gamma::Ramp;
use super::mode::{self, Mode};
use super::vcp::Bus;
use windows_sys::Win32::Devices::Display::{
    CapabilitiesRequestAndCapabilitiesReply, DestroyPhysicalMonitor, GetCapabilitiesStringLength,
    GetNumberOfPhysicalMonitorsFromHMONITOR, GetPhysicalMonitorsFromHMONITOR,
    GetVCPFeatureAndVCPFeatureReply, PHYSICAL_MONITOR, SetVCPFeature,
};
use windows_sys::Win32::Foundation::{HANDLE, LPARAM, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    CreateDCW, DISPLAY_DEVICEW, DeleteDC, EnumDisplayDevicesW, EnumDisplayMonitors,
    GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW,
};
use windows_sys::Win32::System::Registry::{HKEY_LOCAL_MACHINE, RRF_RT_REG_BINARY, RegGetValueW};
use windows_sys::Win32::UI::ColorSystem::SetDeviceGammaRamp;
use windows_sys::Win32::UI::WindowsAndMessaging::EDD_GET_DEVICE_INTERFACE_NAME;
use windows_sys::core::BOOL;

pub struct Physical {
    handle: HANDLE,
    /// GDI name of the display it shows, `\\.\DISPLAY2`, NUL-terminated, for its display mode.
    device: Vec<u16>,
}

// SAFETY: the handle is owned by one `Monitor`, which only its module thread touches.
unsafe impl Send for Physical {}

impl Drop for Physical {
    fn drop(&mut self) {
        // SAFETY: `handle` came from GetPhysicalMonitorsFromHMONITOR and is destroyed once.
        unsafe { DestroyPhysicalMonitor(self.handle) };
    }
}

/// One physical monitor and the Windows display it belongs to ("Display 2").
pub struct Found {
    pub display: String,
    pub bus: Physical,
}

unsafe extern "system" fn collect(m: HMONITOR, _: HDC, _: *mut RECT, list: LPARAM) -> BOOL {
    // SAFETY: `list` is the `Vec` passed by `detect`, alive for the whole enumeration.
    unsafe { (*(list as *mut Vec<HMONITOR>)).push(m) };
    1
}

/// Every physical monitor Windows knows. A display with no DDC/CI still appears; its reads fail later.
pub fn detect() -> Vec<Found> {
    let mut hmonitors: Vec<HMONITOR> = Vec::new();
    // SAFETY: the callback only pushes into `hmonitors`, which outlives the call.
    unsafe {
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null(),
            Some(collect),
            &mut hmonitors as *mut _ as LPARAM,
        )
    };
    let mut found = Vec::new();
    for hm in hmonitors {
        let (display, device) = display_name(hm);
        let mut count = 0u32;
        // SAFETY: `hm` came from EnumDisplayMonitors; `count` is a valid out pointer.
        if unsafe { GetNumberOfPhysicalMonitorsFromHMONITOR(hm, &mut count) } == 0 || count == 0 {
            continue;
        }
        // SAFETY: PHYSICAL_MONITOR is plain data; all-zero is a valid value.
        let mut list = vec![unsafe { std::mem::zeroed::<PHYSICAL_MONITOR>() }; count as usize];
        // SAFETY: `list` holds `count` entries.
        if unsafe { GetPhysicalMonitorsFromHMONITOR(hm, count, list.as_mut_ptr()) } == 0 {
            continue;
        }
        found.extend(list.into_iter().map(|p| Found {
            display: display.clone(),
            bus: Physical {
                handle: p.hPhysicalMonitor,
                device: device.clone(),
            },
        }));
    }
    found
}

/// "Display 2" from the GDI device name `\\.\DISPLAY2`, the number Windows Settings shows, and that name
/// NUL-terminated.
fn display_name(hm: HMONITOR) -> (String, Vec<u16>) {
    // SAFETY: MONITORINFOEXW is plain data; cbSize says which struct it is.
    let mut info: MONITORINFOEXW = unsafe { std::mem::zeroed() };
    info.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
    // SAFETY: `info` is a MONITORINFOEXW with cbSize set, which GetMonitorInfoW accepts.
    if unsafe { GetMonitorInfoW(hm, &mut info as *mut _ as *mut MONITORINFO) } == 0 {
        return ("Display".to_owned(), vec![0]);
    }
    let len = info.szDevice.iter().position(|&c| c == 0).unwrap_or(32);
    let device = String::from_utf16_lossy(&info.szDevice[..len]);
    let number: String = device.chars().filter(char::is_ascii_digit).collect();
    let mut name = info.szDevice[..len].to_vec();
    name.push(0);
    (format!("Display {number}"), name)
}

impl Bus for Physical {
    /// Slow (up to a second): called once, at start.
    fn capabilities(&mut self) -> Option<String> {
        let mut len = 0u32;
        // SAFETY: valid handle and out pointer.
        if unsafe { GetCapabilitiesStringLength(self.handle, &mut len) } == 0 || len == 0 {
            return None;
        }
        let mut buf = vec![0u8; len as usize];
        // SAFETY: `buf` holds `len` bytes.
        if unsafe { CapabilitiesRequestAndCapabilitiesReply(self.handle, buf.as_mut_ptr(), len) }
            == 0
        {
            return None;
        }
        Some(
            String::from_utf8_lossy(&buf)
                .trim_end_matches('\0')
                .to_owned(),
        )
    }

    fn read(&mut self, code: u8) -> Result<(u16, u16), String> {
        let (mut kind, mut current, mut max) = (0, 0u32, 0u32);
        // SAFETY: valid handle and out pointers.
        let ok = unsafe {
            GetVCPFeatureAndVCPFeatureReply(self.handle, code, &mut kind, &mut current, &mut max)
        };
        if ok == 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let clamp = |v: u32| u16::try_from(v).unwrap_or(u16::MAX);
        Ok((clamp(current), clamp(max)))
    }

    fn write_raw(&mut self, code: u8, value: u16) -> Result<(), String> {
        // SAFETY: valid handle. `vcp::write` has checked the code and the value.
        if unsafe { SetVCPFeature(self.handle, code, value.into()) } == 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        Ok(())
    }

    fn modes(&mut self) -> Vec<Mode> {
        mode::list(&self.device)
    }

    fn mode(&mut self) -> Option<Mode> {
        mode::current(&self.device)
    }

    fn turn(&mut self) -> Option<u8> {
        mode::turn(&self.device)
    }

    /// The first monitor on this display output: its interface name `\\?\DISPLAY#ACR0A4B#5&..#{guid}` gives
    /// the registry key Windows keeps its EDID under.
    fn edid(&mut self) -> Option<Vec<u8>> {
        // SAFETY: DISPLAY_DEVICEW is plain data; cb says which struct it is.
        let mut dd: DISPLAY_DEVICEW = unsafe { std::mem::zeroed() };
        dd.cb = size_of::<DISPLAY_DEVICEW>() as u32;
        // SAFETY: NUL-terminated adapter name and a DISPLAY_DEVICEW with cb set.
        let ok = unsafe {
            EnumDisplayDevicesW(
                self.device.as_ptr(),
                0,
                &mut dd,
                EDD_GET_DEVICE_INTERFACE_NAME,
            )
        };
        if ok == 0 {
            return None;
        }
        let len = dd.DeviceID.iter().position(|&c| c == 0).unwrap_or(128);
        let id = String::from_utf16_lossy(&dd.DeviceID[..len]);
        let parts: Vec<&str> = id.trim_start_matches(r"\\?\").split('#').collect();
        let [class, hardware, instance, ..] = parts[..] else {
            return None;
        };
        let key: Vec<u16> = format!(
            r"SYSTEM\CurrentControlSet\Enum\{class}\{hardware}\{instance}\Device Parameters"
        )
        .encode_utf16()
        .chain([0])
        .collect();
        let value: Vec<u16> = "EDID".encode_utf16().chain([0]).collect();
        let mut buf = vec![0u8; 512];
        let mut size = buf.len() as u32;
        // SAFETY: NUL-terminated key and value names; `buf` holds `size` bytes.
        let r = unsafe {
            RegGetValueW(
                HKEY_LOCAL_MACHINE,
                key.as_ptr(),
                value.as_ptr(),
                RRF_RT_REG_BINARY,
                std::ptr::null_mut(),
                buf.as_mut_ptr().cast(),
                &mut size,
            )
        };
        (r == 0).then(|| {
            buf.truncate(size as usize);
            buf
        })
    }

    fn set_mode(&mut self, m: Mode, turn: u8, keep: bool) -> Result<(), String> {
        mode::set(&self.device, m, turn, keep)
    }

    fn set_gamma(&mut self, ramp: &Ramp) -> Result<(), String> {
        // SAFETY: a display DC for a NUL-terminated GDI name, deleted below; no driver, port or DEVMODE.
        let dc = unsafe {
            CreateDCW(
                std::ptr::null(),
                self.device.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
            )
        };
        if dc.is_null() {
            return Err(format!(
                "no device context: {}",
                std::io::Error::last_os_error()
            ));
        }
        // SAFETY: `ramp` is 3 × 256 WORDs, the layout SetDeviceGammaRamp reads.
        let ok = unsafe { SetDeviceGammaRamp(dc, ramp.as_ptr().cast()) } != 0;
        // SAFETY: `dc` came from CreateDCW and is deleted once.
        unsafe { DeleteDC(dc) };
        // Windows refuses a ramp too far from neutral unless the GdiIcmGammaRange registry value allows it.
        ok.then_some(()).ok_or_else(|| {
            "Windows refused this colour change (too far from neutral for its gamma limit)"
                .to_owned()
        })
    }
}
