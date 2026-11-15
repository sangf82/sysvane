// The history the UI keeps: one array per plotted series, newest sample last.

import { cpuTemp, netTotals, percentOf } from "./metrics";
import { type Snapshot, TICK_S } from "../wire";

export const HISTORY = 120;
/** How far back the graphs reach, for their time labels. */
export const HISTORY_MINUTES = (HISTORY * TICK_S) / 60;

/** Keys of the history map. */
export const K = {
  cpu: "cpu",
  cpuMhz: "cpu.mhz",
  cpuTemp: "cpu.temp",
  mem: "mem",
  commit: "commit",
  netRx: "net.rx",
  gpu: "gpu",
  gpuVram: "gpu.vram",
  gpuTemp: "gpu.temp",
  gpuPower: "gpu.power",
  core: (i: number) => `core:${i}`,
  disk: (mount: string, series: "act" | "r" | "w") => `disk:${mount}.${series}`,
  net: (name: string, dir: "rx" | "tx") => `net:${name}.${dir}`,
};

export type History = Record<string, number[]>;

const push = (a: number[] | undefined, v: number) => [...(a ?? []).slice(-(HISTORY - 1)), v];

/** The history after one more snapshot. */
export function record(prev: History, s: Snapshot): History {
  const samples: [string, number][] = [
    [K.cpu, s.cpu.usage],
    [K.mem, percentOf(s.mem.used, s.mem.total)],
    [K.commit, percentOf(s.mem.commit_used, s.mem.commit_limit)],
    [K.netRx, netTotals(s).rx],
  ];
  if (s.cpu.mhz != null) samples.push([K.cpuMhz, s.cpu.mhz]);
  const tctl = cpuTemp(s);
  if (tctl) samples.push([K.cpuTemp, tctl.celsius]);
  s.cpu.cores.forEach((v, i) => samples.push([K.core(i), v]));
  s.disks.forEach((d) => {
    samples.push([K.disk(d.mount, "act"), d.active_pct ?? 0], [K.disk(d.mount, "r"), d.read_bps], [K.disk(d.mount, "w"), d.write_bps]);
  });
  s.nets.forEach((n) => samples.push([K.net(n.name, "rx"), n.rx_bps], [K.net(n.name, "tx"), n.tx_bps]));
  if (s.gpu) {
    samples.push([K.gpu, s.gpu.util], [K.gpuVram, percentOf(s.gpu.mem_used, s.gpu.mem_total)]);
    if (s.gpu.temp_c != null) samples.push([K.gpuTemp, s.gpu.temp_c]);
    if (s.gpu.power_w != null) samples.push([K.gpuPower, s.gpu.power_w]);
  }
  const next = { ...prev };
  for (const [key, v] of samples) next[key] = push(prev[key], v);
  return next;
}
