import { transition } from "@animate-ui/react/utils/motion";
import { motion } from "motion/react";
import { useEffect, useMemo, useRef, useState } from "react";
import { t } from "../lib/i18n";
import { bytes, dur, rate } from "../lib/format";
import { Icon } from "../components/layout";
import { Dropdown } from "../components/Dropdown";
import { ProcIcon } from "../components/ProcIcon";
import { Glyph, type GlyphName } from "../components/glyphs";
import type { Proc } from "../wire";
import { COLUMNS, type Col, type Column, type Row, type Sort, arrange, rows, search, tree } from "./processTable";

const COLUMNS_KEY = "sysvane-proc-cols";
const isLocked = (id: Col) => COLUMNS.find((c) => c.id === id)?.locked;
const BY_CPU: Sort = { col: "cpu", desc: true };
/** Hidden until the user picks columns: the tree shows the parent, and virtual size is rarely the question. */
const HIDDEN_FIRST: Col[] = ["ppid", "virt"];
/** One-click orders for the two questions people ask most: what uses the most, and the least. */
const QUICK: { value: string; label: "Most CPU first" | "Least CPU first" | "Most memory first" | "Least memory first" }[] = [
  { value: "cpu:desc", label: "Most CPU first" },
  { value: "cpu:asc", label: "Least CPU first" },
  { value: "mem:desc", label: "Most memory first" },
  { value: "mem:asc", label: "Least memory first" },
];
type Mode = "list" | "name" | "tree";
const MODES: { id: Mode; label: "List" | "By name" | "Tree"; icon: GlyphName }[] = [
  { id: "list", label: "List", icon: "list" },
  { id: "name", label: "By name", icon: "layers" },
  { id: "tree", label: "Tree", icon: "tree" },
];

/** Hidden columns are remembered per user. */
function useHiddenColumns(): [Set<Col>, (c: Col) => void] {
  const [hidden, setHidden] = useState<Set<Col>>(() => {
    try {
      const raw = localStorage.getItem(COLUMNS_KEY);
      const saved = raw == null ? HIDDEN_FIRST : (JSON.parse(raw) as Col[]);
      return new Set(saved.filter((c) => !isLocked(c)));
    } catch {
      return new Set();
    }
  });
  const toggle = (c: Col) =>
    setHidden((prev) => {
      const next = new Set(prev);
      if (next.has(c)) next.delete(c);
      else next.add(c);
      localStorage.setItem(COLUMNS_KEY, JSON.stringify([...next]));
      return next;
    });
  return [hidden, toggle];
}

function Detail({ p, all, close }: { p: Proc; all: Proc[]; close: () => void }) {
  useEffect(() => {
    const esc = (e: KeyboardEvent) => e.key === "Escape" && close();
    window.addEventListener("keydown", esc);
    return () => window.removeEventListener("keydown", esc);
  }, [close]);
  const parent = p.ppid != null ? all.find((x) => x.pid === p.ppid) : undefined;
  const item = (label: string, value: React.ReactNode) => (
    <div>
      <dt>{label}</dt>
      <dd className="num">{value}</dd>
    </div>
  );
  return (
    <motion.section className="pdetail" aria-label={t("Process details")} initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={transition.fade}>
      <header>
        <ProcIcon exe={p.exe} />
        <h2>
          {p.name} <span className="muted num">PID {p.pid}</span>
        </h2>
        <button className="iconbtn" aria-label={t("Close")} title={t("Close")} onClick={close}>
          <Glyph name="x" size={16} />
        </button>
      </header>
      <dl className="stats">
        {item(t("User"), p.user ?? t("n/a"))}
        {item(t("Parent"), p.ppid != null ? `${parent?.name ?? t("exited")} (${p.ppid})` : t("none"))}
        {item(t("Started"), p.start != null ? new Date(p.start * 1000).toLocaleString() : t("n/a"))}
        {item(t("Run time"), p.run_s != null ? dur(p.run_s) : t("n/a"))}
        {item("CPU", `${p.cpu.toFixed(1)}%`)}
        {item(t("Memory (working set)"), bytes(p.mem))}
        {item(t("Virtual"), bytes(p.virt))}
        {item(t("Disk read"), rate(p.read_bps))}
        {item(t("Disk write"), rate(p.write_bps))}
      </dl>
      <div className="paths">
        <div>
          <span className="muted">{t("Path")}</span>
          <code>{p.exe ?? t("not readable")}</code>
        </div>
        <div>
          <span className="muted">{t("Command line")}</span>
          <code>{p.cmd || t("not readable")}</code>
        </div>
      </div>
    </motion.section>
  );
}

export function Processes({ list }: { list: Proc[] }) {
  const [sort, setSort] = useState<Sort>(BY_CPU);
  const [q, setQ] = useState("");
  const [mode, setMode] = useState<Mode>("list");
  const group = mode === "name";
  const [collapsed, setCollapsed] = useState<Set<number>>(new Set());
  const fold = (pid: number, open: boolean) =>
    setCollapsed((prev) => {
      const next = new Set(prev);
      if (open) next.delete(pid);
      else next.add(pid);
      return next;
    });
  const [sel, setSel] = useState<number | null>(null);
  const [hidden, toggleColumn] = useHiddenColumns();
  const shown = COLUMNS.filter((c) => !hidden.has(c.id));
  // A sort column that was hidden falls back to CPU, so the order stays explainable.
  const by = hidden.has(sort.col) ? BY_CPU : sort;
  // The row order is fixed: values update in place every second, but rows only move when the
  // sort, the filter or the grouping changes. New processes join
  // at the bottom. "Pause" also freezes the values.
  const [paused, setPaused] = useState(false);
  const held = useRef(list);
  if (!paused) held.current = list;
  const source = held.current;
  const order = useRef<{ sig: string; keys: string[] }>({ sig: "", keys: [] });
  const view = `${by.col}:${by.desc}|${q}|${group}`;
  // The tree takes its sibling order from the flat order, so it keeps the same stable rows.
  const data = useMemo(() => {
    const found = search(source, q);
    const next = arrange(rows(found, group), by, order.current.sig === view ? order.current.keys : null);
    order.current = { sig: view, keys: next.map((r) => r.key) };
    return mode === "tree" ? tree(source, found, order.current.keys, collapsed) : next;
  }, [source, view, mode, collapsed]);
  const top = Math.max(1, ...data.map((r) => r.mem));
  const cell = (c: Column, r: Row) => {
    const v = c.value(r);
    const w = c.bar === "pct" ? Math.min(100, v as number) : c.bar === "rel" ? ((v as number) / top) * 100 : 0;
    const cls = [`c-${c.id}`, c.right && "r num", c.cell, w > 0.5 && "bar", c.right && v === 0 && "zero"].filter(Boolean).join(" ");
    const style = w > 0.5 ? ({ "--w": `${w}%` } as React.CSSProperties) : undefined;
    if (c.id !== "name") return <td key={c.id} className={cls} style={style}>{c.show(r)}</td>;
    const tree = mode === "tree";
    const open = !collapsed.has(r.pid!);
    return (
      <td key={c.id} className={cls}>
        <span className="pcell" style={tree ? { paddingLeft: (r.depth ?? 0) * 16 } : undefined}>
          {tree &&
            (r.kids ? (
              <button className="twist" aria-expanded={open} aria-label={open ? t("Collapse") : t("Expand")} tabIndex={-1} onClick={(e) => { e.stopPropagation(); fold(r.pid!, !open); }}>
                <Icon name="sortDesc" size={12} />
              </button>
            ) : (
              <span className="twist" />
            ))}
          <ProcIcon exe={r.exe} />
          <span className="ptext">{c.show(r)}</span>
          {tree && !open && <span className="muted num">+{r.kids}</span>}
        </span>
      </td>
    );
  };
  const selected = sel != null ? source.find((p) => p.pid === sel) : undefined;
  const header = (c: Column) => (
    <th key={c.id} className={`c-${c.id}${c.right ? " r" : ""}`} aria-sort={by.col === c.id ? (by.desc ? "descending" : "ascending") : "none"}>
      <button onClick={() => setSort((s) => ({ col: c.id, desc: s.col === c.id ? !s.desc : !c.text }))}>
        {t(c.label)}
        <span className="sort">{by.col === c.id && <Icon name={by.desc ? "sortDesc" : "sortAsc"} size={12} />}</span>
      </button>
    </th>
  );
  return (
    <div className="page fill">
      <div className="toolbar">
        <h1>{t("Processes")}</h1>
        <label className="filter">
          <Icon name="search" size={14} />
          <input aria-label={t("Filter processes")} placeholder={t("Filter by name, PID, user or path")} value={q} onChange={(e) => setQ(e.target.value)} />
        </label>
      </div>
      {/* Second line: how the list is shown. Pause sits apart at the end with the count. */}
      <div className="toolbar controls">
        <div className="seg" role="group" aria-label={t("View")}>
          {MODES.map((m) => (
            <button key={m.id} className={mode === m.id ? "on" : ""} aria-pressed={mode === m.id} onClick={() => setMode(m.id)}>
              <Glyph name={m.icon} />
              {t(m.label)}
            </button>
          ))}
        </div>
        <Dropdown
          label={t("Sort")}
          icon={<Glyph name="sort" />}
          value={`${by.col}:${by.desc ? "desc" : "asc"}`}
          onChange={(v) => {
            const [col, dir] = v.split(":");
            setSort({ col: col as Col, desc: dir === "desc" });
          }}
          options={[
            ...QUICK.map((o) => ({ value: o.value, label: t(o.label) })),
            ...(QUICK.some((o) => o.value === `${by.col}:${by.desc ? "desc" : "asc"}`)
              ? []
              : [{ value: `${by.col}:${by.desc ? "desc" : "asc"}`, label: t("By {c}", { c: t(COLUMNS.find((c) => c.id === by.col)!.label) }) }]),
          ]}
        />
        <details className="colmenu">
          <summary>
            <Glyph name="columns" />
            {t("Columns")}
          </summary>
          <div className="menu">
            {COLUMNS.filter((c) => !c.locked).map((c) => (
              <label key={c.id} className="check">
                <input type="checkbox" checked={!hidden.has(c.id)} onChange={() => toggleColumn(c.id)} />
                {t(c.label)}
              </label>
            ))}
          </div>
        </details>
        <span className="muted num count">
          {data.length}
          {group ? ` ${t("groups")}` : ` / ${source.length}`}
        </span>
        <button className="btn" aria-pressed={paused} title={t("Pause updates. Values stay as they are.")} onClick={() => setPaused((p) => !p)}>
          <Glyph name={paused ? "play" : "pause"} />
          {paused ? t("Resume") : t("Pause")}
        </button>
      </div>
      <div className={`pbody${selected ? " open" : ""}`}>
      <div className="tablewrap">
        <table>
          <thead>
            <tr>{shown.map(header)}</tr>
          </thead>
          <tbody>
            {data.map((r) => (
              <tr
                key={r.key}
                tabIndex={0}
                aria-selected={r.pid != null && r.pid === sel}
                className={r.pid != null && r.pid === sel ? "sel" : ""}
                onClick={() => setSel(r.pid)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    setSel(r.pid);
                  } else if (mode === "tree" && r.kids && (e.key === "ArrowRight" || e.key === "ArrowLeft")) {
                    e.preventDefault();
                    fold(r.pid!, e.key === "ArrowRight");
                  } else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
                    e.preventDefault();
                    const next = e.key === "ArrowDown" ? e.currentTarget.nextElementSibling : e.currentTarget.previousElementSibling;
                    (next as HTMLElement | null)?.focus();
                  }
                }}
              >
                {shown.map((c) => cell(c, r))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {selected && <Detail key={selected.pid} p={selected} all={source} close={() => setSel(null)} />}
      </div>
    </div>
  );
}
