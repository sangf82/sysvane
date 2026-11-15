// The process table's data: rows, columns, searching and ordering. The page only draws it.

import type { Msg } from "../lib/i18n";
import { bytes, dur, rate } from "../lib/format";
import type { Proc } from "../wire";

export type Col = "name" | "pid" | "ppid" | "user" | "cpu" | "mem" | "virt" | "read" | "write" | "run";
export type Sort = { col: Col; desc: boolean };

export type Row = {
  key: string;
  name: string;
  count: number;
  pid: number | null;
  ppid: number | null;
  user: string;
  cpu: number;
  mem: number;
  virt: number;
  read: number;
  write: number;
  run: number;
};

export type Column = {
  id: Col;
  label: Msg;
  right?: boolean;
  /** Always shown: the column menu cannot hide it. */
  locked?: boolean;
  /** Sorts A to Z on the first click; the others sort largest first. */
  text?: boolean;
  /** A class for its cells, besides the alignment. */
  cell?: string;
  value: (r: Row) => string | number;
  show: (r: Row) => string | number;
};

const NO_RUN = -1;

const speed = (bps: number) => (bps ? rate(bps) : "0");

export const COLUMNS: Column[] = [
  { id: "name", label: "Name", locked: true, cell: "pname", text: true, value: (r) => r.name, show: (r) => (r.count > 1 ? `${r.name} (${r.count})` : r.name) },
  { id: "pid", label: "PID", right: true, value: (r) => r.pid ?? 0, show: (r) => r.pid ?? "" },
  { id: "ppid", label: "Parent", right: true, value: (r) => r.ppid ?? 0, show: (r) => r.ppid ?? "" },
  { id: "user", label: "User", text: true, value: (r) => r.user, show: (r) => r.user },
  { id: "cpu", label: "CPU", right: true, value: (r) => r.cpu, show: (r) => `${r.cpu.toFixed(1)}%` },
  { id: "mem", label: "Memory", right: true, value: (r) => r.mem, show: (r) => bytes(r.mem) },
  { id: "virt", label: "Virtual", right: true, value: (r) => r.virt, show: (r) => bytes(r.virt) },
  { id: "read", label: "Disk read", right: true, value: (r) => r.read, show: (r) => speed(r.read) },
  { id: "write", label: "Disk write", right: true, value: (r) => r.write, show: (r) => speed(r.write) },
  { id: "run", label: "Run time", right: true, value: (r) => r.run, show: (r) => (r.run === NO_RUN ? "–" : dur(r.run)) },
];

function rowOf(p: Proc, grouped: boolean): Row {
  return {
    key: grouped ? p.name : String(p.pid),
    name: p.name,
    count: 1,
    pid: grouped ? null : p.pid,
    ppid: grouped ? null : p.ppid,
    user: p.user ?? "",
    cpu: p.cpu,
    mem: p.mem,
    virt: p.virt,
    read: p.read_bps,
    write: p.write_bps,
    run: p.run_s ?? NO_RUN,
  };
}

/** One row per process, or one per name with the figures added up. */
export function rows(list: Proc[], group: boolean): Row[] {
  if (!group) return list.map((p) => rowOf(p, false));
  const byName = new Map<string, Row>();
  for (const p of list) {
    const next = rowOf(p, true);
    const g = byName.get(p.name);
    if (!g) {
      byName.set(p.name, next);
    } else {
      g.count += 1;
      g.cpu += next.cpu;
      g.mem += next.mem;
      g.virt += next.virt;
      g.read += next.read;
      g.write += next.write;
      g.run = Math.max(g.run, next.run);
    }
  }
  return [...byName.values()];
}

export function search(list: Proc[], query: string): Proc[] {
  const q = query.trim().toLowerCase();
  if (!q) return list;
  return list.filter((p) => p.name.toLowerCase().includes(q) || String(p.pid).includes(q) || (p.user ?? "").toLowerCase().includes(q) || (p.exe ?? "").toLowerCase().includes(q));
}

const compare = (a: string | number, b: string | number) => (typeof a === "string" ? a.localeCompare(b as string) : a - (b as number));

/** The order the table shows. Given the keys of the previous order, rows keep their place and new ones
 *  go last, busiest first. Given null, the rows are sorted. */
export function arrange(list: Row[], sort: Sort, previous: string[] | null): Row[] {
  if (previous === null) {
    const column = COLUMNS.find((c) => c.id === sort.col)!;
    const dir = sort.desc ? -1 : 1;
    return [...list].sort((a, b) => dir * compare(column.value(a), column.value(b)) || a.name.localeCompare(b.name));
  }
  const place = new Map(previous.map((key, i) => [key, i]));
  const known = list.filter((r) => place.has(r.key)).sort((a, b) => place.get(a.key)! - place.get(b.key)!);
  const fresh = list.filter((r) => !place.has(r.key)).sort((a, b) => b.cpu - a.cpu);
  return [...known, ...fresh];
}
