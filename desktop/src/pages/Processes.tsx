import { transition } from "@animate-ui/react/utils/motion";
import { motion } from "motion/react";
import { useMemo, useRef, useState } from "react";
import { t } from "../lib/i18n";
import { bytes, dur, rate } from "../lib/format";
import { Icon } from "../components/layout";
import type { Proc } from "../wire";
import { COLUMNS, type Col, type Column, type Sort, arrange, rows, search } from "./processTable";

const COLUMNS_KEY = "sysvane-proc-cols";
const isLocked = (id: Col) => COLUMNS.find((c) => c.id === id)?.locked;
const BY_CPU: Sort = { col: "cpu", desc: true };

/** Hidden columns are remembered per user. */
function useHiddenColumns(): [Set<Col>, (c: Col) => void] {
  const [hidden, setHidden] = useState<Set<Col>>(() => {
    try {
      const saved = JSON.parse(localStorage.getItem(COLUMNS_KEY) ?? "[]") as Col[];
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

function Detail({ p, all }: { p: Proc; all: Proc[] }) {
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
        <h2>
          {p.name} <span className="muted num">PID {p.pid}</span>
        </h2>
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
  const [group, setGroup] = useState(false);
  const [sel, setSel] = useState<number | null>(null);
  const [hidden, toggleColumn] = useHiddenColumns();
  const shown = COLUMNS.filter((c) => !hidden.has(c.id));
  // A sort column that was hidden falls back to CPU, so the order stays explainable.
  const by = hidden.has(sort.col) ? BY_CPU : sort;
  // The row order is fixed: values update in place every second, but rows only move when the
  // sort, the filter or the grouping changes, or when "Sort now" is pressed. New processes join
  // at the bottom. "Pause" also freezes the values.
  const [paused, setPaused] = useState(false);
  const [resort, setResort] = useState(0);
  const held = useRef(list);
  if (!paused) held.current = list;
  const source = held.current;
  const order = useRef<{ sig: string; keys: string[] }>({ sig: "", keys: [] });
  const view = `${by.col}:${by.desc}|${q}|${group}|${resort}`;
  const data = useMemo(() => {
    const next = arrange(rows(search(source, q), group), by, order.current.sig === view ? order.current.keys : null);
    order.current = { sig: view, keys: next.map((r) => r.key) };
    return next;
  }, [source, view]);
  const selected = sel != null ? source.find((p) => p.pid === sel) : undefined;
  const header = (c: Column) => (
    <th key={c.id} className={c.right ? "r" : ""} aria-sort={by.col === c.id ? (by.desc ? "descending" : "ascending") : "none"}>
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
        <label className="check">
          <input type="checkbox" checked={group} onChange={(e) => setGroup(e.target.checked)} />
          {t("Group by name")}
        </label>
        <button className="btn" aria-pressed={paused} title={t("Pause updates. Values stay as they are.")} onClick={() => setPaused((p) => !p)}>
          {paused ? t("Resume") : t("Pause")}
        </button>
        <button className="btn" title={t("Rows keep their place between sorts. Press to sort by the current column.")} onClick={() => setResort((n) => n + 1)}>
          {t("Sort now")}
        </button>
        <details className="colmenu">
          <summary>{t("Columns")}</summary>
          <div className="menu">
            {COLUMNS.filter((c) => !c.locked).map((c) => (
              <label key={c.id} className="check">
                <input type="checkbox" checked={!hidden.has(c.id)} onChange={() => toggleColumn(c.id)} />
                {t(c.label)}
              </label>
            ))}
          </div>
        </details>
        <span className="muted num">
          {data.length}
          {group ? ` ${t("groups")}` : ` / ${source.length}`}
        </span>
      </div>
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
                  } else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
                    e.preventDefault();
                    const next = e.key === "ArrowDown" ? e.currentTarget.nextElementSibling : e.currentTarget.previousElementSibling;
                    (next as HTMLElement | null)?.focus();
                  }
                }}
              >
                {shown.map((c) => (
                  <td key={c.id} className={`${c.right ? "r num" : ""} ${c.cell ?? ""}`}>
                    {c.show(r)}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {selected ? <Detail p={selected} all={source} /> : <p className="hint">{group ? t("Turn off grouping to see one process's details.") : t("Select a process to see its path, command line, parent and user.")}</p>}
    </div>
  );
}
