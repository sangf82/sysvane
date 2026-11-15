//! AMD CPU temperature through the PawnIO kernel driver, which only an elevated process may open.
//! It is read in a worker process (`worker.rs`).
//! `tools/probe-temp.ps1` repeats the register numbers below for use without the core.

use crate::system::win::wide;
use libloading::Library;
use std::ffi::c_void;
use sysvane_wire::{Temp, TempKind};
use windows_sys::Win32::Foundation::{CloseHandle, WAIT_ABANDONED, WAIT_OBJECT_0};
use windows_sys::Win32::System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject};

/// Signed PawnIO module for AMD family 17h to 1Ah (LGPL-2.1, see core/pawnio/COPYING).
static MODULE: &[u8] = include_bytes!("../../pawnio/AMDFamily17.bin");

const TCTL_REG: u32 = 0x0005_9800;
/// CCD temperature registers: Zen 2 to Zen 4 start at the first, Zen 5 at the second.
/// Only one range answers with valid bits on a given CPU.
const CCD_BASES: [u32; 2] = [0x0005_9954, 0x0005_9B08];
const MAX_CCD: u32 = 8;

/// Tctl in degrees. Bit 19 means the scale is shifted down by 49 C.
fn tctl_c(raw: u32) -> f32 {
    (raw >> 21) as f32 * 0.125 - if raw & (1 << 19) != 0 { 49.0 } else { 0.0 }
}

/// CCD temperature in degrees; None when the valid bit (11) is clear.
fn ccd_c(raw: u32) -> Option<f32> {
    (raw & 0x800 != 0).then_some((raw & 0x7FF) as f32 * 0.125 - 49.0)
}

type OpenFn = unsafe extern "system" fn(*mut *mut c_void) -> i32;
type LoadFn = unsafe extern "system" fn(*mut c_void, *const u8, usize) -> i32;
type ExecFn = unsafe extern "system" fn(
    *mut c_void,
    *const u8,
    *const u64,
    usize,
    *mut u64,
    usize,
    *mut usize,
) -> i32;
type CloseFn = unsafe extern "system" fn(*mut c_void) -> i32;

struct PawnIo {
    _lib: Library,
    handle: *mut c_void,
    execute: ExecFn,
    close: CloseFn,
}

impl PawnIo {
    fn open() -> Option<Self> {
        let dir = std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into());
        // SAFETY: loading the signed vendor DLL, and the signatures below are the ones in PawnIOLib.h.
        unsafe {
            let lib = Library::new(format!(r"{dir}\PawnIO\PawnIOLib.dll")).ok()?;
            let open: OpenFn = *lib.get(b"pawnio_open\0").ok()?;
            let load: LoadFn = *lib.get(b"pawnio_load\0").ok()?;
            let execute: ExecFn = *lib.get(b"pawnio_execute\0").ok()?;
            let close: CloseFn = *lib.get(b"pawnio_close\0").ok()?;
            let mut handle = std::ptr::null_mut();
            if open(&mut handle) < 0 {
                return None;
            }
            let this = Self {
                _lib: lib,
                handle,
                execute,
                close,
            };
            // On failure `this` is dropped here and closes the handle.
            (load(handle, MODULE.as_ptr(), MODULE.len()) >= 0).then_some(this)
        }
    }

    /// Reads one 32-bit SMN register. Holds the system-wide PCI mutex, as other
    /// hardware monitors do, because the read goes through shared PCI index/data ports.
    fn read_smn(&self, reg: u32) -> Option<u32> {
        let name = wide("Global\\Access_PCI");
        let input = [u64::from(reg)];
        let mut out = [0u64];
        let mut returned = 0usize;
        // SAFETY: buffers are sized as declared; the mutex handle is closed on every path.
        unsafe {
            let mutex = CreateMutexW(std::ptr::null(), 0, name.as_ptr());
            if mutex.is_null() {
                return None;
            }
            let waited = WaitForSingleObject(mutex, 200);
            let held = waited == WAIT_OBJECT_0 || waited == WAIT_ABANDONED;
            let hr = if held {
                (self.execute)(
                    self.handle,
                    c"ioctl_read_smn".as_ptr().cast(),
                    input.as_ptr(),
                    1,
                    out.as_mut_ptr(),
                    1,
                    &mut returned,
                )
            } else {
                -1
            };
            if held {
                ReleaseMutex(mutex);
            }
            CloseHandle(mutex);
            (hr >= 0).then_some(out[0] as u32)
        }
    }

    fn temps(&self) -> Vec<Temp> {
        let mut temps = Vec::new();
        if let Some(raw) = self.read_smn(TCTL_REG) {
            temps.push(Temp {
                label: "CPU (Tctl)".into(),
                celsius: tctl_c(raw),
                kind: TempKind::Cpu,
            });
        }
        let mut n = 0;
        for base in CCD_BASES {
            for i in 0..MAX_CCD {
                if let Some(c) = self.read_smn(base + 4 * i).and_then(ccd_c) {
                    n += 1;
                    temps.push(Temp {
                        label: format!("CCD {n}"),
                        celsius: c,
                        kind: TempKind::Ccd,
                    });
                }
            }
        }
        temps
    }
}

impl Drop for PawnIo {
    fn drop(&mut self) {
        // SAFETY: `handle` came from pawnio_open and is closed once.
        unsafe { (self.close)(self.handle) };
    }
}

/// Worker entry point: see `worker::run`. Without administrator rights PawnIO cannot be opened and the reading stays `null`.
pub fn worker_main() {
    let io = PawnIo::open();
    crate::system::worker::run(|| io.as_ref().map(PawnIo::temps).filter(|t| !t.is_empty()));
}

#[cfg(test)]
mod tests {
    use super::*;

    // Values read from a Ryzen 9 9900X with PawnIO on 2026-11-09.
    #[test]
    fn decodes_tctl_and_ccd_registers() {
        assert_eq!(tctl_c(0x6DDB_0000), 60.75);
        assert_eq!(ccd_c(0x0B60), Some(59.0));
        assert_eq!(ccd_c(0x0AE0), Some(43.0));
        assert_eq!(ccd_c(0), None, "valid bit clear means no CCD here");
    }
}
