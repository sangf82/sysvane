// What the core sends, as the `sysvane-wire` crate defines it. A Rust test fails when a field
// is added, removed or renamed on one side only. Keep one field per line, with no comments inside a type.

export type TempKind = "cpu" | "ccd" | "sensor";

export type DeviceId = "core";

export type ControlId = "autostart";

export type ControlValue = boolean | number;

export type DeviceStatus = "ok" | "failed";

// Protocol version, as `PROTOCOL` in the wire crate.
export const PROTOCOL = 2;

// Seconds between snapshots, as TICK in the wire crate.
export const TICK_S = 1;

export type Host = {
  name: string;
  os_version: string;
  kernel: string;
  arch: string;
  uptime_s: number;
  processes: number;
  threads: number;
  handles: number;
};

export type Cpu = {
  name: string;
  vendor: string;
  logical: number;
  physical: number | null;
  usage: number;
  cores: number[];
  mhz: number | null;
  base_mhz: number | null;
};

export type Mem = {
  total: number;
  used: number;
  available: number;
  cached: number;
  swap_total: number;
  swap_used: number;
  commit_used: number;
  commit_limit: number;
  paged_pool: number;
  nonpaged_pool: number;
};

export type Disk = {
  mount: string;
  name: string;
  fs: string;
  kind: string;
  removable: boolean;
  used: number;
  total: number;
  read_bps: number;
  write_bps: number;
  active_pct: number | null;
  resp_ms: number | null;
};

export type Net = {
  name: string;
  mac: string;
  ips: string[];
  mtu: number;
  rx_bps: number;
  tx_bps: number;
  rx_total: number;
  tx_total: number;
};

export type Temp = {
  label: string;
  celsius: number;
  kind: TempKind;
};

export type Gpu = {
  name: string;
  driver: string | null;
  util: number;
  mem_used: number;
  mem_total: number;
  temp_c: number | null;
  power_w: number | null;
  power_limit_w: number | null;
  clock_gr_mhz: number | null;
  clock_mem_mhz: number | null;
  fan_pct: number | null;
  enc_util: number | null;
  dec_util: number | null;
  pcie_gen: number | null;
  pcie_width: number | null;
};

export type Proc = {
  pid: number;
  ppid: number | null;
  name: string;
  user: string | null;
  cpu: number;
  mem: number;
  virt: number;
  read_bps: number;
  write_bps: number;
  start: number | null;
  run_s: number | null;
  exe: string | null;
  cmd: string;
};

export type Control = {
  id: ControlId;
  value: ControlValue;
};

export type Device = {
  id: DeviceId;
  status: DeviceStatus;
  error: string | null;
  controls: Control[];
};

export type Snapshot = {
  protocol: number;
  host: Host;
  cpu: Cpu;
  mem: Mem;
  disks: Disk[];
  nets: Net[];
  gpu: Gpu | null;
  temps: Temp[];
  processes: Proc[];
  devices: Device[];
};
