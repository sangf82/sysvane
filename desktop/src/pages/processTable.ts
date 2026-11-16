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
  exe: string | null;
  /** Tree view only: nesting level and number of direct children. */
  depth?: number;
  kids?: number;
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
  /** Draws a bar behind the value: `pct` is already 0 to 100, `rel` is scaled to the largest row. */
  bar?: "pct" | "rel";
  value: (r: Row) => string | number;
  show: (r: Row) => string | number;
};

const NO_RUN = -1;

const speed = (bps: number) => (bps ? rate(bps) : "0");

export const COLUMNS: Column[] = [
  { id: "name", label: "Name", locked: true, cell: "pname", text: true, value: (r) => r.name, show: (r) => (r.count > 1 ? `${r.name} (${r.count})` : r.name) },
  { id: "cpu", label: "CPU", right: true, bar: "pct", value: (r) => r.cpu, show: (r) => `${r.cpu.toFixed(1)}%` },
  { id: "mem", label: "Memory", right: true, bar: "rel", value: (r) => r.mem, show: (r) => bytes(r.mem) },
  { id: "read", label: "Disk read", right: true, value: (r) => r.read, show: (r) => speed(r.read) },
  { id: "write", label: "Disk write", right: true, value: (r) => r.write, show: (r) => speed(r.write) },
  { id: "pid", label: "PID", right: true, value: (r) => r.pid ?? 0, show: (r) => r.pid ?? "" },
  { id: "user", label: "User", text: true, value: (r) => r.user, show: (r) => r.user },
  { id: "run", label: "Run time", right: true, value: (r) => r.run, show: (r) => (r.run === NO_RUN ? "–" : dur(r.run)) },
  { id: "ppid", label: "Parent", right: true, value: (r) => r.ppid ?? 0, show: (r) => r.ppid ?? "" },
  { id: "virt", label: "Virtual", right: true, value: (r) => r.virt, show: (r) => bytes(r.virt) },
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
    exe: p.exe,
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

/** Parent of `p` among `byPid`, if it is still running. Windows reuses PIDs, so a "parent" that
 *  started after its child is a different process and does not count. */
function parentOf(p: Proc, byPid: Map<number, Proc>): Proc | undefined {
  if (p.ppid == null || p.ppid === p.pid) return undefined;
  const parent = byPid.get(p.ppid);
  if (!parent || (parent.start != null && p.start != null && parent.start > p.start)) return undefined;
  return parent;
}

/** Processes as a parent/child tree. `shown` are the ones the filter kept; their ancestors are added
 *  so each stays under its parent. Siblings follow `order` (the flat order's row keys). Children of a
 *  PID in `collapsed` are left out. */
export function tree(all: Proc[], shown: Proc[], order: string[], collapsed: Set<number>): Row[] {
  const byPid = new Map(all.map((p) => [p.pid, p]));
  const keep = new Set<number>();
  for (const p of shown) {
    // The guard ends a PID cycle that a reused PID could still form.
    for (let x: Proc | undefined = p; x && !keep.has(x.pid); x = parentOf(x, byPid)) keep.add(x.pid);
  }
  const rank = new Map(order.map((key, i) => [key, i]));
  const at = (p: Proc) => rank.get(String(p.pid)) ?? Infinity;
  const children = new Map<number | null, Proc[]>();
  for (const pid of keep) {
    const p = byPid.get(pid)!;
    const parent = parentOf(p, byPid);
    const key = parent && keep.has(parent.pid) ? parent.pid : null;
    children.set(key, [...(children.get(key) ?? []), p]);
  }
  for (const list of children.values()) list.sort((a, b) => at(a) - at(b));
  const out: Row[] = [];
  const seen = new Set<number>();
  const walk = (p: Proc, depth: number, emit: boolean) => {
    if (seen.has(p.pid)) return;
    seen.add(p.pid);
    const kids = children.get(p.pid) ?? [];
    if (emit) out.push({ ...rowOf(p, false), depth, kids: kids.length });
    for (const k of kids) walk(k, depth + 1, emit && !collapsed.has(p.pid));
  };
  for (const root of children.get(null) ?? []) walk(root, 0, true);
  // Processes caught in a PID cycle have no root above them: list them at the top level.
  for (const pid of keep) walk(byPid.get(pid)!, 0, true);
  return out;
}
