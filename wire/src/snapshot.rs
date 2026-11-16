use crate::command::{ControlId, ControlValue, DeviceId};
use serde::{Deserialize, Serialize};

/// Everything one tick sends to the UI. A reading that cannot be taken is `None`, never a guessed value.
#[derive(Serialize, Clone, Debug)]
pub struct Snapshot {
    pub protocol: u32,
    pub host: Host,
    pub cpu: Cpu,
    pub mem: Mem,
    pub disks: Vec<Disk>,
    pub nets: Vec<Net>,
    /// First NVIDIA GPU only. None when NVML is missing or the read failed.
    pub gpu: Option<Gpu>,
    /// Empty when no sensor is readable.
    pub temps: Vec<Temp>,
    /// The busiest ones, the most CPU first.
    pub processes: Vec<Proc>,
    /// What the core controls: itself and, later, device modules.
    pub devices: Vec<Device>,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DeviceStatus {
    Ok,
    /// The last command failed or the module stopped answering; `Device::error` says why.
    Failed,
}

#[derive(Serialize, Clone, PartialEq, Debug)]
pub struct Device {
    pub id: DeviceId,
    /// What the user sees, such as "AL258Q · Display 2". Empty for the core and the fake device.
    pub name: String,
    pub status: DeviceStatus,
    pub error: Option<String>,
    pub controls: Vec<Control>,
    /// Profiles `ControlId::Profile` can apply, built-in first.
    pub profiles: Vec<String>,
    /// Display modes `ControlId::DisplayMode` accepts, best first. Empty for devices without one.
    pub modes: Vec<String>,
    /// Colour presets (MCCS 1..=13) `ControlId::ColorPreset` lists. Empty: the device lists none.
    pub presets: Vec<u16>,
    /// A command or read is running on the device now, such as a profile being written.
    pub busy: bool,
}

#[derive(Serialize, Clone, PartialEq, Debug)]
pub struct Control {
    pub id: ControlId,
    pub value: ControlValue,
}

#[derive(Serialize, Clone, Debug)]
pub struct Host {
    pub name: String,
    pub os_version: String,
    pub kernel: String,
    pub arch: String,
    pub uptime_s: u64,
    pub processes: u32,
    pub threads: u32,
    pub handles: u32,
}

#[derive(Serialize, Clone, Debug)]
pub struct Cpu {
    pub name: String,
    pub vendor: String,
    pub logical: usize,
    pub physical: Option<usize>,
    pub usage: f32,
    pub cores: Vec<f32>,
    /// Effective clock including boost.
    pub mhz: Option<u64>,
    pub base_mhz: Option<u32>,
}

#[derive(Serialize, Clone, Debug)]
pub struct Mem {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub cached: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub commit_used: u64,
    pub commit_limit: u64,
    pub paged_pool: u64,
    pub nonpaged_pool: u64,
}

#[derive(Serialize, Clone, Debug)]
pub struct Disk {
    pub mount: String,
    pub name: String,
    pub fs: String,
    pub kind: String,
    pub removable: bool,
    pub used: u64,
    pub total: u64,
    pub read_bps: u64,
    pub write_bps: u64,
    pub active_pct: Option<f32>,
    pub resp_ms: Option<f32>,
}

#[derive(Serialize, Clone, Debug)]
pub struct Net {
    pub name: String,
    pub mac: String,
    pub ips: Vec<String>,
    pub mtu: u64,
    pub rx_bps: u64,
    pub tx_bps: u64,
    pub rx_total: u64,
    pub tx_total: u64,
}

/// What a temperature reading belongs to, so the UI never has to guess from the label.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TempKind {
    /// The CPU package (Tctl).
    Cpu,
    /// One core complex die.
    Ccd,
    Sensor,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Temp {
    pub label: String,
    pub celsius: f32,
    pub kind: TempKind,
}

#[derive(Serialize, Clone, Debug)]
pub struct Proc {
    pub pid: u32,
    pub ppid: Option<u32>,
    pub name: String,
    pub user: Option<String>,
    /// Percent of the whole CPU, like Task Manager (not per core).
    pub cpu: f32,
    pub mem: u64,
    pub virt: u64,
    pub read_bps: u64,
    pub write_bps: u64,
    /// Seconds since the Unix epoch, and the seconds since then.
    pub start: Option<u64>,
    pub run_s: Option<u64>,
    pub exe: Option<String>,
    pub cmd: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Gpu {
    pub name: String,
    pub driver: Option<String>,
    pub util: u32,
    pub mem_used: u64,
    pub mem_total: u64,
    pub temp_c: Option<u32>,
    pub power_w: Option<f32>,
    pub power_limit_w: Option<f32>,
    pub clock_gr_mhz: Option<u32>,
    pub clock_mem_mhz: Option<u32>,
    pub fan_pct: Option<u32>,
    pub enc_util: Option<u32>,
    pub dec_util: Option<u32>,
    pub pcie_gen: Option<u32>,
    pub pcie_width: Option<u32>,
}
