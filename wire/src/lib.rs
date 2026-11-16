//! What `sysvane-core` and `sysvane-ui` agree on: the pipe, the process names and the snapshot
//! the pipe carries. `desktop/src/wire.ts` mirrors the snapshot types and `TICK`; the test below
//! fails when a field or the tick differs between the two.

mod command;
mod consts;
mod snapshot;

pub use command::*;
pub use consts::*;
pub use snapshot::*;

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
                name: String::new(),
                status: DeviceStatus::Ok,
                error: None,
                controls: vec![Control {
                    id: ControlId::Autostart,
                    value: ControlValue::Bool(false),
                }],
                profiles: vec![],
                modes: vec![],
                presets: vec![],
                busy: false,
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
        let ok = r#"{"protocol":4,"device":"core","control":"autostart","value":true}"#;
        let c: Command = serde_json::from_str(ok).unwrap();
        assert_eq!(
            (c.device, c.control, c.value),
            (
                DeviceId::Core,
                ControlId::Autostart,
                ControlValue::Bool(true)
            )
        );
        let ok = r#"{"protocol":4,"device":{"monitor":1},"control":"profile","value":"Gaming"}"#;
        let c: Command = serde_json::from_str(ok).unwrap();
        assert_eq!(
            (c.device, c.value),
            (DeviceId::Monitor(1), ControlValue::Text("Gaming".into()))
        );
        let ok = r#"{"protocol":4,"device":{"monitor":0},"control":{"vcp":98},"value":40}"#;
        assert_eq!(
            serde_json::from_str::<Command>(ok).unwrap().control,
            ControlId::Vcp(0x62)
        );
        for bad in [
            r#"{"protocol":4,"device":"core","control":"autostart","value":true,"path":"x"}"#,
            r#"{"protocol":4,"device":"gpu","control":"autostart","value":true}"#,
            r#"{"protocol":4,"device":{"monitor":256},"control":"brightness","value":1}"#,
            r#"{"protocol":4,"device":{"monitor":-1},"control":"brightness","value":1}"#,
            r#"{"protocol":4,"device":"core","control":"format","value":true}"#,
            r#"{"protocol":4,"device":{"monitor":0},"control":{"vcp":256},"value":1}"#,
            r#"{"protocol":4,"device":"core","control":"autostart","value":[1]}"#,
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
