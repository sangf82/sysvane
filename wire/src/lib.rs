//! What `sysvane-core` and `sysvane-ui` agree on: the pipe, the process names and the snapshot
//! the pipe carries. `desktop/src/wire.ts` mirrors the snapshot types and `TICK`; the test below
//! fails when a field or the tick differs between the two.

use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const PIPE_NAME: &str = r"\\.\pipe\sysvane-core";
/// UI to core, one JSON `Command` per line. The core never answers on it: the result shows in the next snapshot.
pub const COMMAND_PIPE_NAME: &str = r"\\.\pipe\sysvane-core-cmd";
/// A command line longer than this is refused and its client dropped.
pub const MAX_COMMAND_BYTES: usize = 512;
pub const CORE_EXE: &str = "sysvane-core.exe";
pub const UI_EXE: &str = "sysvane-ui.exe";
/// The scheduled task that starts the core with administrator rights.
pub const CORE_TASK: &str = "SysVane Core";
/// Opens the UI as the small tray flyout. Without it the UI opens the dashboard.
pub const FLYOUT_ARG: &str = "--flyout";

/// Version of the pipe protocol: the snapshot and, later, the commands. Raise it when either changes
/// shape. The UI refuses to talk to a core that reports another number.
pub const PROTOCOL: u32 = 2;

/// Time between snapshots. The UI keeps a fixed number of them, so its "2 minutes" labels depend on this.
pub const TICK: Duration = Duration::from_secs(1);

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

/// Every device the core can be told to change. A command names one of these, never a free string.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum DeviceId {
    /// The core itself: settings that live in the core, such as starting with Windows.
    Core,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ControlId {
    Autostart,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(untagged)]
pub enum ControlValue {
    Bool(bool),
    Int(i64),
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
    pub status: DeviceStatus,
    pub error: Option<String>,
    pub controls: Vec<Control>,
}

#[derive(Serialize, Clone, PartialEq, Debug)]
pub struct Control {
    pub id: ControlId,
    pub value: ControlValue,
}

/// One line on the command pipe. Unknown fields are refused, and so is another protocol number.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub protocol: u32,
    pub device: DeviceId,
    pub control: ControlId,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::collections::{BTreeMap, BTreeSet};

    /// Every `Option` is `None` and `gpu` is `Some`, so a field is null exactly when it is an `Option`.
    fn sample() -> Snapshot {
        Snapshot {
            protocol: PROTOCOL,
            host: Host {
                name: String::new(),
                os_version: String::new(),
                kernel: String::new(),
                arch: String::new(),
                uptime_s: 0,
                processes: 0,
                threads: 0,
                handles: 0,
            },
            cpu: Cpu {
                name: String::new(),
                vendor: String::new(),
                logical: 0,
                physical: None,
                usage: 0.0,
                cores: vec![],
                mhz: None,
                base_mhz: None,
            },
            mem: Mem {
                total: 0,
                used: 0,
                available: 0,
                cached: 0,
                swap_total: 0,
                swap_used: 0,
                commit_used: 0,
                commit_limit: 0,
                paged_pool: 0,
                nonpaged_pool: 0,
            },
            disks: vec![Disk {
                mount: String::new(),
                name: String::new(),
                fs: String::new(),
                kind: String::new(),
                removable: false,
                used: 0,
                total: 0,
                read_bps: 0,
                write_bps: 0,
                active_pct: None,
                resp_ms: None,
            }],
            nets: vec![Net {
                name: String::new(),
                mac: String::new(),
                ips: vec![],
                mtu: 0,
                rx_bps: 0,
                tx_bps: 0,
                rx_total: 0,
                tx_total: 0,
            }],
            gpu: Some(Gpu {
                name: String::new(),
                driver: None,
                util: 0,
                mem_used: 0,
                mem_total: 0,
                temp_c: None,
                power_w: None,
                power_limit_w: None,
                clock_gr_mhz: None,
                clock_mem_mhz: None,
                fan_pct: None,
                enc_util: None,
                dec_util: None,
                pcie_gen: None,
                pcie_width: None,
            }),
            temps: vec![Temp {
                label: String::new(),
                celsius: 0.0,
                kind: TempKind::Cpu,
            }],
            processes: vec![Proc {
                pid: 0,
                ppid: None,
                name: String::new(),
                user: None,
                cpu: 0.0,
                mem: 0,
                virt: 0,
                read_bps: 0,
                write_bps: 0,
                start: None,
                run_s: None,
                exe: None,
                cmd: String::new(),
            }],
            devices: vec![Device {
                id: DeviceId::Core,
                status: DeviceStatus::Ok,
                error: None,
                controls: vec![Control {
                    id: ControlId::Autostart,
                    value: ControlValue::Bool(false),
                }],
            }],
        }
    }

    /// Fields of `export type <name> = { ... }` in wire.ts as name to type text, one field per line.
    fn ts_fields(ts: &str, name: &str) -> BTreeMap<String, String> {
        let head = format!("export type {name} = {{");
        let start = ts
            .find(&head)
            .unwrap_or_else(|| panic!("wire.ts has no type {name}"))
            + head.len();
        let body = &ts[start..start + ts[start..].find("};").expect("type block ends with };")];
        body.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with(['/', '*']))
            .filter_map(|l| l.split_once(':'))
            .map(|(field, ty)| {
                (
                    field.trim().to_owned(),
                    ty.trim().trim_end_matches(';').to_owned(),
                )
            })
            .collect()
    }

    fn check(ts: &str, name: &str, value: &Value) {
        let fields = ts_fields(ts, name);
        let object = value.as_object().expect("a struct");
        let ts_names: BTreeSet<&str> = fields.keys().map(String::as_str).collect();
        let rust_names: BTreeSet<&str> = object.keys().map(String::as_str).collect();
        assert_eq!(ts_names, rust_names, "{name} fields in wire.ts");
        for (field, v) in object {
            let ty = &fields[field];
            assert_eq!(
                ty.ends_with("| null"),
                v.is_null(),
                "{name}.{field}: `| null` exactly when the Rust field is an Option"
            );
            let kind_matches = match v {
                Value::Number(_) => ty.starts_with("number"),
                Value::String(_) => {
                    ty.starts_with("string")
                        || matches!(
                            ty.as_str(),
                            "TempKind" | "DeviceId" | "ControlId" | "DeviceStatus"
                        )
                }
                Value::Bool(_) => ty.starts_with("boolean") || ty == "ControlValue",
                Value::Array(_) => ty.ends_with("[]"),
                Value::Object(_) => ty.starts_with(char::is_uppercase),
                Value::Null => true,
            };
            assert!(
                kind_matches,
                "{name}.{field}: wire.ts says `{ty}`, the snapshot sends {v}"
            );
        }
    }

    #[test]
    fn ui_types_match_the_snapshot() {
        let ts = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../desktop/src/wire.ts"
        ))
        .expect("desktop/src/wire.ts");
        let v = serde_json::to_value(sample()).unwrap();
        let mut without_gpu = sample();
        without_gpu.gpu = None;
        check(&ts, "Snapshot", &serde_json::to_value(without_gpu).unwrap());
        for (name, value) in [
            ("Host", &v["host"]),
            ("Cpu", &v["cpu"]),
            ("Mem", &v["mem"]),
            ("Disk", &v["disks"][0]),
            ("Net", &v["nets"][0]),
            ("Gpu", &v["gpu"]),
            ("Temp", &v["temps"][0]),
            ("Proc", &v["processes"][0]),
            ("Device", &v["devices"][0]),
            ("Control", &v["devices"][0]["controls"][0]),
        ] {
            check(&ts, name, value);
        }
    }

    #[test]
    fn commands_are_validated_by_type() {
        let ok = r#"{"protocol":2,"device":"core","control":"autostart","value":true}"#;
        let c: Command = serde_json::from_str(ok).unwrap();
        assert_eq!(
            (c.device, c.control, c.value),
            (
                DeviceId::Core,
                ControlId::Autostart,
                ControlValue::Bool(true)
            )
        );
        for bad in [
            r#"{"protocol":2,"device":"core","control":"autostart","value":true,"path":"x"}"#,
            r#"{"protocol":2,"device":"gpu","control":"autostart","value":true}"#,
            r#"{"protocol":2,"device":"core","control":"format","value":true}"#,
            r#"{"protocol":2,"device":"core","control":"autostart","value":"on"}"#,
            r#"{"device":"core","control":"autostart","value":true}"#,
        ] {
            assert!(serde_json::from_str::<Command>(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn ui_knows_the_temperature_kinds_and_the_tick() {
        let ts = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../desktop/src/wire.ts"
        ))
        .expect("desktop/src/wire.ts");
        let line = |start: &str| {
            ts.lines()
                .find(|l| l.starts_with(start))
                .unwrap_or_else(|| panic!("wire.ts has no `{start}`"))
                .to_owned()
        };
        let in_ts: BTreeSet<String> = line("export type TempKind =")
            .split('"')
            .skip(1)
            .step_by(2)
            .map(str::to_owned)
            .collect();
        let in_rust: BTreeSet<String> = [TempKind::Cpu, TempKind::Ccd, TempKind::Sensor]
            .into_iter()
            .map(|k| {
                serde_json::to_value(k)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_owned()
            })
            .collect();
        assert_eq!(in_ts, in_rust, "TempKind in wire.ts");
        let protocol = line("export const PROTOCOL =");
        assert_eq!(
            protocol.trim_end_matches(';').rsplit(' ').next().unwrap(),
            PROTOCOL.to_string(),
            "PROTOCOL in wire.ts"
        );
        let tick = line("export const TICK_S =");
        assert_eq!(
            tick.trim_end_matches(';').rsplit(' ').next().unwrap(),
            TICK.as_secs().to_string(),
            "TICK_S in wire.ts"
        );
    }
}
