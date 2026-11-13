//! Windows performance counters and system totals that `sysinfo` does not give.
//! - Effective CPU clock: the base clock scaled by "% Processor Performance" (the Task Manager
//!   source). `sysinfo` reports 0 and the power API's "current" clock is just the base clock.
//! - Per-volume active time and response time from the LogicalDisk counters.
//! - Process, thread and handle counts and the commit/cache/pool figures.

use crate::win::wide;
use std::collections::HashMap;
use std::ffi::c_void;
use windows_sys::Win32::System::Performance::{
    PDH_FMT_COUNTERVALUE, PDH_FMT_DOUBLE, PdhAddEnglishCounterW, PdhCloseQuery,
    PdhCollectQueryData, PdhGetFormattedCounterValue, PdhOpenQueryW,
};
use windows_sys::Win32::System::Power::{
    CallNtPowerInformation, PROCESSOR_POWER_INFORMATION, ProcessorInformation,
};
use windows_sys::Win32::System::ProcessStatus::{GetPerformanceInfo, PERFORMANCE_INFORMATION};

const CPU_PERF: &str = r"\Processor Information(_Total)\% Processor Performance";

#[derive(Clone, Copy)]
struct Counter(*mut c_void);

/// One PDH query. Rate counters need two collections, so values appear from the second tick.
struct Pdh(*mut c_void);

impl Pdh {
    fn new() -> Option<Self> {
        let mut query = std::ptr::null_mut();
        // SAFETY: the out-pointer is valid; the query is closed in Drop.
        (unsafe { PdhOpenQueryW(std::ptr::null(), 0, &mut query) } == 0).then_some(Self(query))
    }

    fn add(&self, path: &str) -> Option<Counter> {
        let path = wide(path);
        let mut counter = std::ptr::null_mut();
        // SAFETY: `path` is NUL-terminated; the counter lives as long as the query.
        (unsafe { PdhAddEnglishCounterW(self.0, path.as_ptr(), 0, &mut counter) } == 0)
            .then_some(Counter(counter))
    }

    fn collect(&self) {
        // SAFETY: valid query handle.
        unsafe { PdhCollectQueryData(self.0) };
    }

    fn value(&self, counter: Counter) -> Option<f64> {
        // SAFETY: valid counter handle; `v` is a valid out-struct.
        unsafe {
            let mut v: PDH_FMT_COUNTERVALUE = std::mem::zeroed();
            let status = PdhGetFormattedCounterValue(
                counter.0,
                PDH_FMT_DOUBLE,
                std::ptr::null_mut(),
                &mut v,
            );
            (status == 0 && v.CStatus == 0).then_some(v.Anonymous.doubleValue)
        }
    }
}

impl Drop for Pdh {
    fn drop(&mut self) {
        // SAFETY: closing the query once also releases its counters.
        unsafe { PdhCloseQuery(self.0) };
    }
}

/// Base clock in MHz from the first logical processor.
fn base_mhz(logical_cpus: usize) -> Option<u32> {
    let zero = PROCESSOR_POWER_INFORMATION {
        Number: 0,
        MaxMhz: 0,
        CurrentMhz: 0,
        MhzLimit: 0,
        MaxIdleState: 0,
        CurrentIdleState: 0,
    };
    let mut info = vec![zero; logical_cpus.max(1)];
    let bytes = (info.len() * size_of::<PROCESSOR_POWER_INFORMATION>()) as u32;
    // SAFETY: the output buffer holds `info.len()` entries and `bytes` is exactly its size.
    let status = unsafe {
        CallNtPowerInformation(
            ProcessorInformation,
            std::ptr::null(),
            0,
            info.as_mut_ptr().cast(),
            bytes,
        )
    };
    (status == 0)
        .then_some(info[0].MaxMhz)
        .filter(|&mhz| mhz > 0)
}

/// The volume letter (`C:`) of a volume root such as `C:\`. Other mount points have no LogicalDisk counter.
fn drive_letter(mount: &str) -> Option<&str> {
    mount
        .strip_suffix('\\')
        .filter(|m| m.len() == 2 && m.ends_with(':'))
}

/// Task Manager's clock: the base clock scaled by "% Processor Performance".
fn effective_mhz(base_mhz: u32, performance_pct: f64) -> u64 {
    (f64::from(base_mhz) * performance_pct / 100.0).round() as u64
}

fn busy_pct(idle_pct: f64) -> f32 {
    (100.0 - idle_pct).clamp(0.0, 100.0) as f32
}

/// System-wide totals from GetPerformanceInfo, in bytes where they were pages.
#[derive(Default, Clone, Copy)]
pub struct SysTotals {
    pub processes: u32,
    pub threads: u32,
    pub handles: u32,
    pub commit_used: u64,
    pub commit_limit: u64,
    pub cached: u64,
    pub paged_pool: u64,
    pub nonpaged_pool: u64,
}

pub fn sys_totals() -> SysTotals {
    // SAFETY: `info.cb` is set to the struct size as the API requires.
    unsafe {
        let mut info: PERFORMANCE_INFORMATION = std::mem::zeroed();
        info.cb = size_of::<PERFORMANCE_INFORMATION>() as u32;
        if GetPerformanceInfo(&mut info, info.cb) == 0 {
            return SysTotals::default();
        }
        let page = info.PageSize as u64;
        SysTotals {
            processes: info.ProcessCount,
            threads: info.ThreadCount,
            handles: info.HandleCount,
            commit_used: info.CommitTotal as u64 * page,
            commit_limit: info.CommitLimit as u64 * page,
            cached: info.SystemCache as u64 * page,
            paged_pool: info.KernelPaged as u64 * page,
            nonpaged_pool: info.KernelNonpaged as u64 * page,
        }
    }
}

struct DiskCounters {
    idle: Option<Counter>,
    resp: Option<Counter>,
}

/// Per-volume readings: active time in percent, average response time in milliseconds.
#[derive(Default, Clone, Copy)]
pub struct DiskPerf {
    pub active_pct: Option<f32>,
    pub resp_ms: Option<f32>,
}

#[derive(Default)]
pub struct PerfSample {
    pub cpu_mhz: Option<u64>,
    pub base_mhz: Option<u32>,
    pub disks: HashMap<String, DiskPerf>,
}

pub struct Perf {
    pdh: Pdh,
    cpu_perf: Option<Counter>,
    base_mhz: Option<u32>,
    disks: HashMap<String, DiskCounters>,
}

impl Perf {
    pub fn new(logical_cpus: usize) -> Option<Self> {
        let pdh = Pdh::new()?;
        let cpu_perf = pdh.add(CPU_PERF);
        pdh.collect();
        Some(Self {
            pdh,
            cpu_perf,
            base_mhz: base_mhz(logical_cpus),
            disks: HashMap::new(),
        })
    }

    /// `mounts` are volume roots like `C:\`. Counters are added the first time a volume is seen.
    pub fn sample(&mut self, mounts: &[String]) -> PerfSample {
        for mount in mounts {
            let Some(letter) = drive_letter(mount) else {
                continue;
            };
            self.disks
                .entry(mount.clone())
                .or_insert_with(|| DiskCounters {
                    idle: self
                        .pdh
                        .add(&format!(r"\LogicalDisk({letter})\% Idle Time")),
                    resp: self
                        .pdh
                        .add(&format!(r"\LogicalDisk({letter})\Avg. Disk sec/Transfer")),
                });
        }
        self.pdh.collect();
        let cpu_mhz = self
            .base_mhz
            .zip(self.cpu_perf.and_then(|c| self.pdh.value(c)))
            .map(|(base, pct)| effective_mhz(base, pct));
        let disks = self
            .disks
            .iter()
            .map(|(mount, c)| {
                let active_pct = c.idle.and_then(|c| self.pdh.value(c)).map(busy_pct);
                let resp_ms = c
                    .resp
                    .and_then(|c| self.pdh.value(c))
                    .map(|s| (s * 1000.0) as f32);
                (
                    mount.clone(),
                    DiskPerf {
                        active_pct,
                        resp_ms,
                    },
                )
            })
            .collect();
        PerfSample {
            cpu_mhz,
            base_mhz: self.base_mhz,
            disks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_drive_letter_roots_have_a_disk_counter() {
        assert_eq!(drive_letter(r"C:\"), Some("C:"));
        assert_eq!(drive_letter("C:"), None);
        assert_eq!(drive_letter(r"D:\data\"), None);
        assert_eq!(drive_letter(r"\\?\Volume{1234}\"), None);
    }

    #[test]
    fn readings_are_scaled_and_clamped() {
        assert_eq!(effective_mhz(4400, 120.0), 5280);
        assert_eq!(busy_pct(97.5), 2.5);
        assert_eq!(busy_pct(120.0), 0.0);
        assert_eq!(busy_pct(-5.0), 100.0);
    }
}
