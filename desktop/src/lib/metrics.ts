// Numbers and names derived from a snapshot, so every page reads them the same way.

import type { Disk, Net, Snapshot, Temp } from "../wire";

export const percentOf = (used: number, total: number) => (total ? (used / total) * 100 : 0);
export const memPercent = (s: Snapshot) => percentOf(s.mem.used, s.mem.total);

/** The CPU package temperature, when the sensor helper is running. */
export const cpuTemp = (s: Snapshot): Temp | undefined => s.temps.find((t) => t.kind === "cpu");
/** The package and every die. */
export const cpuTemps = (s: Snapshot): Temp[] => s.temps.filter((t) => t.kind !== "sensor");

export const netTotals = (s: Snapshot) => ({
  rx: s.nets.reduce((sum, n) => sum + n.rx_bps, 0),
  tx: s.nets.reduce((sum, n) => sum + n.tx_bps, 0),
});
/** Adapters, the one that moved the most data first. */
export const netsByTraffic = (s: Snapshot): Net[] => [...s.nets].sort((a, b) => b.rx_total + b.tx_total - (a.rx_total + a.tx_total));

/** "C:" from the mount point "C:\". */
export const diskName = (d: Disk) => d.mount.replace("\\", "");

/** Ids of the things the Performance page can show. The Overview uses them to open one. */
export const RES = {
  cpu: "cpu",
  mem: "mem",
  gpu: "gpu",
  disk: (d: Disk) => `disk:${d.mount}`,
  net: (n: Net) => `net:${n.name}`,
};
