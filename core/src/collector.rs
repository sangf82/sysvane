//! Samples the whole system once per tick into a wire `Snapshot`.

use crate::devices::Devices;
use crate::mode::Mode;
use crate::perf::{Perf, PerfSample, SysTotals, sys_totals};
use crate::worker::Worker;
use std::sync::Arc;
use sysinfo::{
    Components, DiskKind, Disks, Networks, ProcessRefreshKind, ProcessesToUpdate, System,
    UpdateKind, Users,
};
use sysvane_wire::{Cpu, Disk, Gpu, Host, Mem, Net, PROTOCOL, Proc, Snapshot, Temp, TempKind};

const MAX_PROCESSES: usize = 600;
const MAX_CMD_CHARS: usize = 260;

/// Everything the Processes page shows. The default also reads threads for every process, which it does not.
fn process_kind() -> ProcessRefreshKind {
    ProcessRefreshKind::nothing()
        .with_cpu()
        .with_memory()
        .with_disk_usage()
        .with_user(UpdateKind::OnlyIfNotSet)
        .with_exe(UpdateKind::OnlyIfNotSet)
        .with_cmd(UpdateKind::OnlyIfNotSet)
}

fn per_second(bytes: u64, elapsed_s: f64) -> u64 {
    (bytes as f64 / elapsed_s.max(0.001)) as u64
}

fn mount(d: &sysinfo::Disk) -> String {
    d.mount_point().to_string_lossy().into_owned()
}

fn host(totals: &SysTotals) -> Host {
    Host {
        name: System::host_name().unwrap_or_default(),
        os_version: System::long_os_version().unwrap_or_default(),
        kernel: System::kernel_version().unwrap_or_default(),
        arch: System::cpu_arch(),
        uptime_s: System::uptime().unwrap_or(0),
        processes: totals.processes,
        threads: totals.threads,
        handles: totals.handles,
    }
}

/// Holds the sampler state between ticks. Create it only while a client is connected.
pub struct Collector {
    sys: System,
    disks: Disks,
    nets: Networks,
    comps: Components,
    users: Users,
    gpu: Option<Worker<Gpu>>,
    cpu_temp: Option<Worker<Vec<Temp>>>,
    perf: Option<Perf>,
    devices: Arc<Devices>,
}

impl Collector {
    pub fn new(devices: Arc<Devices>) -> Result<Self, sysinfo::Error> {
        let mut sys = System::new()?;
        // Usage needs two readings, so take the first one now.
        sys.refresh_cpu_usage();
        sys.refresh_processes_specifics(ProcessesToUpdate::All, true, process_kind());
        let logical = sys.cpus().len();
        Ok(Self {
            sys,
            disks: Disks::new_with_refreshed_list()?,
            nets: Networks::new_with_refreshed_list()?,
            comps: Components::new_with_refreshed_list()?,
            users: Users::new_with_refreshed_list()?,
            gpu: Worker::start(Mode::GpuWorker),
            cpu_temp: Worker::start(Mode::TempWorker),
            perf: Perf::new(logical),
            devices,
        })
    }

    /// `elapsed_s` is the real time since the last call, used for rates.
    pub fn sample(&mut self, elapsed_s: f64) -> Snapshot {
        self.refresh();
        let perf = self.sample_perf();
        let totals = sys_totals();
        Snapshot {
            protocol: PROTOCOL,
            host: host(&totals),
            cpu: self.cpu(&perf),
            mem: self.mem(&totals),
            disks: self.disks(&perf, elapsed_s),
            nets: self.nets(elapsed_s),
            gpu: self.gpu.as_mut().and_then(Worker::latest),
            temps: self.temps(),
            processes: self.processes(elapsed_s),
            devices: self.devices.snapshot(),
        }
    }

    fn refresh(&mut self) {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        self.sys
            .refresh_processes_specifics(ProcessesToUpdate::All, true, process_kind());
        self.disks.refresh(true);
        self.nets.refresh(true);
        self.comps.refresh(true);
    }

    fn sample_perf(&mut self) -> PerfSample {
        let mounts: Vec<String> = self.disks.iter().map(mount).collect();
        match self.perf.as_mut() {
            Some(p) => p.sample(&mounts),
            None => PerfSample::default(),
        }
    }

    fn cpu(&self, perf: &PerfSample) -> Cpu {
        let cpus = self.sys.cpus();
        Cpu {
            name: cpus
                .first()
                .map(|c| c.brand().trim().to_owned())
                .unwrap_or_default(),
            vendor: cpus
                .first()
                .map(|c| c.vendor_id().to_owned())
                .unwrap_or_default(),
            logical: cpus.len(),
            physical: System::physical_core_count().ok(),
            usage: self.sys.global_cpu_usage(),
            cores: cpus.iter().map(|c| c.usage()).collect(),
            mhz: perf.cpu_mhz,
            base_mhz: perf.base_mhz,
        }
    }

    fn mem(&self, totals: &SysTotals) -> Mem {
        Mem {
            total: self.sys.total_memory(),
            used: self.sys.used_memory(),
            available: self.sys.available_memory(),
            cached: totals.cached,
            swap_total: self.sys.total_swap(),
            swap_used: self.sys.used_swap(),
            commit_used: totals.commit_used,
            commit_limit: totals.commit_limit,
            paged_pool: totals.paged_pool,
            nonpaged_pool: totals.nonpaged_pool,
        }
    }

    fn disks(&self, perf: &PerfSample, elapsed_s: f64) -> Vec<Disk> {
        self.disks
            .iter()
            .map(|d| {
                let mount = mount(d);
                let io = d.usage();
                let p = perf.disks.get(&mount).copied().unwrap_or_default();
                Disk {
                    name: d.name().to_string_lossy().into_owned(),
                    fs: d.file_system().to_string_lossy().into_owned(),
                    kind: match d.kind() {
                        DiskKind::SSD => "SSD".into(),
                        DiskKind::HDD => "HDD".into(),
                        _ => "Unknown".into(),
                    },
                    removable: d.is_removable(),
                    used: d.total_space().saturating_sub(d.available_space()),
                    total: d.total_space(),
                    read_bps: per_second(io.read_bytes, elapsed_s),
                    write_bps: per_second(io.written_bytes, elapsed_s),
                    active_pct: p.active_pct,
                    resp_ms: p.resp_ms,
                    mount,
                }
            })
            .collect()
    }

    fn nets(&self, elapsed_s: f64) -> Vec<Net> {
        self.nets
            .iter()
            .filter(|(_, n)| n.total_received() + n.total_transmitted() > 0)
            .map(|(name, n)| Net {
                name: name.to_string_lossy().into_owned(),
                mac: n.mac_address().to_string(),
                ips: n
                    .ip_networks()
                    .iter()
                    .map(|ip| format!("{}/{}", ip.addr, ip.prefix))
                    .collect(),
                mtu: n.mtu(),
                rx_bps: per_second(n.received(), elapsed_s),
                tx_bps: per_second(n.transmitted(), elapsed_s),
                rx_total: n.total_received(),
                tx_total: n.total_transmitted(),
            })
            .collect()
    }

    fn temps(&mut self) -> Vec<Temp> {
        self.comps
            .iter()
            .filter_map(|c| {
                Some(Temp {
                    label: c.label().to_owned(),
                    celsius: c.temperature()?,
                    kind: TempKind::Sensor,
                })
            })
            .chain(
                self.cpu_temp
                    .as_mut()
                    .and_then(Worker::latest)
                    .unwrap_or_default(),
            )
            .collect()
    }

    /// The busiest `MAX_PROCESSES`, by CPU and then memory.
    fn processes(&self, elapsed_s: f64) -> Vec<Proc> {
        let logical = self.sys.cpus().len().max(1);
        let mut processes: Vec<Proc> = self
            .sys
            .processes()
            .values()
            .map(|p| {
                let io = p.disk_usage();
                let started = p.start_time() != 0;
                Proc {
                    pid: p.pid().as_u32(),
                    ppid: p.parent().map(|pid| pid.as_u32()),
                    name: p.name().to_string_lossy().into_owned(),
                    user: p
                        .user_id()
                        .and_then(|id| self.users.get_user_by_id(id))
                        .map(|u| u.name().to_owned()),
                    cpu: p.cpu_usage() / logical as f32,
                    mem: p.memory(),
                    virt: p.virtual_memory(),
                    read_bps: per_second(io.read_bytes, elapsed_s),
                    write_bps: per_second(io.written_bytes, elapsed_s),
                    start: started.then(|| p.start_time()),
                    run_s: started.then(|| p.run_time()),
                    exe: p.exe().map(|e| e.display().to_string()),
                    cmd: p
                        .cmd()
                        .iter()
                        .map(|a| a.to_string_lossy())
                        .collect::<Vec<_>>()
                        .join(" ")
                        .chars()
                        .take(MAX_CMD_CHARS)
                        .collect(),
                }
            })
            .collect();
        processes.sort_by(|a, b| b.cpu.total_cmp(&a.cpu).then(b.mem.cmp(&a.mem)));
        processes.truncate(MAX_PROCESSES);
        processes
    }
}
