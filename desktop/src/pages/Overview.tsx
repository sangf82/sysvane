import { useState } from "react";
import { t } from "../lib/i18n";
import { RES, cpuTemp, diskName, memPercent, netTotals, netsByTraffic, percentOf } from "../lib/metrics";
import { type History, K } from "../lib/series";
import { bytes, dur, ghz, pct, rate, temp, watts } from "../lib/format";
import { Chart, Graph } from "../components/charts";
import { Flow, Icon, Stats, TempHint } from "../components/layout";
import { ProcIcon } from "../components/ProcIcon";
import { Bar, MemSplit, Meter, heat, heatTemp } from "../components/meters";
import type { Snapshot } from "../wire";

type Open = (resource: string) => void;

/** One module of the machine. The title opens its Performance page. */
function Panel({ title, sub, open, wide, children }: { title: string; sub?: string; open?: () => void; wide?: boolean; children: React.ReactNode }) {
  return (
    <section className={`opanel${wide ? " wide" : ""}`} aria-label={title}>
      <header>
        <h2>
          {open ? (
            <button className="link" onClick={open}>
              {title}
            </button>
          ) : (
            title
          )}
        </h2>
        {sub && <span className="muted">{sub}</span>}
      </header>
      {children}
    </section>
  );
}

const of = (a: string, b: string) => t("{a} of {b}", { a, b });

/** Load of every logical core as a bar, so one pinned core shows at a glance. */
function Cores({ cores }: { cores: number[] }) {
  if (cores.length === 0) return null;
  const top = cores.reduce((b, v, i) => (v > cores[b] ? i : b), 0);
  const busiest = t("Busiest core {n}, {v}", { n: top, v: pct(cores[top]) });
  return (
    <figure className="chart">
      <figcaption>
        <span>{t("Cores")}</span>
        <span className="num">{busiest}</span>
      </figcaption>
      <div className="cores" role="img" aria-label={`${t("Cores")}: ${busiest}`}>
        {cores.map((v, i) => (
          <span key={i} title={`${t("Core {n}", { n: i })}: ${pct(v)}`}>
            <i style={{ height: `${Math.min(100, v)}%`, background: heat(v) }} />
          </span>
        ))}
      </div>
    </figure>
  );
}

function Cpu({ s, h, open }: { s: Snapshot; h: History; open: Open }) {
  const tctl = cpuTemp(s);
  return (
    <Panel title="CPU" sub={s.cpu.name} open={() => open(RES.cpu)} wide>
      <Stats
        items={[
          { label: t("Usage"), value: pct(s.cpu.usage), big: true },
          { label: t("Clock speed"), value: s.cpu.mhz != null ? ghz(s.cpu.mhz) : t("n/a"), big: true },
          { label: t("Temperature"), value: tctl ? temp(tctl.celsius) : t("n/a"), big: true },
        ]}
      />
      <Chart title={t("Usage")} scale="100%" tall>
        <Graph data={h[K.cpu] ?? []} max={100} />
      </Chart>
      <Cores cores={s.cpu.cores} />
      {!tctl && <TempHint />}
    </Panel>
  );
}

function Gpu({ s, h, open }: { s: Snapshot; h: History; open: Open }) {
  const g = s.gpu;
  if (!g) return null;
  return (
    <Panel title="GPU" sub={g.name} open={() => open(RES.gpu)}>
      <Stats
        items={[
          { label: t("Utilization"), value: pct(g.util), big: true },
          { label: t("Temperature"), value: g.temp_c != null ? temp(g.temp_c) : t("n/a"), big: true },
        ]}
      />
      <Chart title={t("Utilization")} scale="100%">
        <Graph data={h[K.gpu] ?? []} max={100} />
      </Chart>
      <Meter label={t("Dedicated memory")} value={percentOf(g.mem_used, g.mem_total)} right={of(bytes(g.mem_used), bytes(g.mem_total))} />
      {g.power_w != null &&
        (g.power_limit_w ? (
          <Meter label={t("Power")} value={percentOf(g.power_w, g.power_limit_w)} right={t("{a} W of {b} W", { a: g.power_w.toFixed(0), b: g.power_limit_w.toFixed(0) })} />
        ) : (
          <p className="row">
            <span>{t("Power")}</span>
            <span className="num">{watts(g.power_w)}</span>
          </p>
        ))}
      <p className="muted num">{[g.clock_gr_mhz != null && `${t("Graphics clock")} ${g.clock_gr_mhz} MHz`, g.fan_pct != null && `${t("Fan")} ${g.fan_pct}%`].filter(Boolean).join(" · ")}</p>
    </Panel>
  );
}

function Memory({ s, open }: { s: Snapshot; open: Open }) {
  const m = s.mem;
  return (
    <Panel title={t("Memory")} sub={bytes(m.total)} open={() => open(RES.mem)}>
      <Stats
        items={[
          { label: t("In use"), value: pct(memPercent(s)), big: true },
          { label: t("Used"), value: bytes(m.used), big: true },
        ]}
      />
      <MemSplit m={m} />
      <Meter label={t("Committed")} value={percentOf(m.commit_used, m.commit_limit)} right={of(bytes(m.commit_used), bytes(m.commit_limit))} />
      {m.swap_total > 0 && <Meter label={t("Swap")} value={percentOf(m.swap_used, m.swap_total)} right={of(bytes(m.swap_used), bytes(m.swap_total))} />}
    </Panel>
  );
}

/** Capacity as a bar, transfers as a mirrored spark: read above, write below. */
function Disks({ s, h, open }: { s: Snapshot; h: History; open: Open }) {
  return (
    <Panel title={t("Disks")}>
      <div className="dlist">
        {s.disks.map((d) => {
          const r = h[K.disk(d.mount, "r")] ?? [];
          const w = h[K.disk(d.mount, "w")] ?? [];
          const top = Math.max(1024 * 1024, ...r, ...w);
          const used = percentOf(d.used, d.total);
          const name = t("Disk {d}", { d: diskName(d) });
          return (
            <div key={d.mount} className="drow">
              <div className="row">
                <button className="link" onClick={() => open(RES.disk(d))}>
                  {name}
                </button>
                <span className="muted num">{of(bytes(d.used), bytes(d.total))}</span>
              </div>
              <Bar value={used} color={heat(used)} label={`${name} ${t("Used")}`} />
              <div className="dspark">
                <div className="split">
                  <Graph data={r} max={top} />
                  <Graph data={w} max={top} flip muted />
                </div>
              </div>
              <p className="muted num">
                {t("read {r}, write {w}", { r: rate(d.read_bps), w: rate(d.write_bps) })}
                {d.active_pct != null && ` · ${t("Active time")} ${pct(d.active_pct)}`}
              </p>
            </div>
          );
        })}
      </div>
    </Panel>
  );
}

function Network({ s, h, open }: { s: Snapshot; h: History; open: Open }) {
  const { rx, tx } = netTotals(s);
  const busiest = netsByTraffic(s)[0];
  const rxs = h[K.netRx] ?? [];
  const txs = h[K.netTx] ?? [];
  const top = Math.max(100 * 1024, ...rxs, ...txs);
  return (
    <Panel title={t("Network")} sub={busiest?.name} open={busiest ? () => open(RES.net(busiest)) : undefined}>
      <Stats
        items={[
          { label: t("Receive"), value: <Flow dir="down">{rate(rx)}</Flow>, big: true },
          { label: t("Send"), value: <Flow dir="up">{rate(tx)}</Flow>, big: true },
        ]}
      />
      <Chart title={t("Throughput, receive above and send below")} scale={rate(top)}>
        <div className="split">
          <Graph data={rxs} max={top} />
          <Graph data={txs} max={top} flip muted />
        </div>
      </Chart>
    </Panel>
  );
}

/** The busiest processes, styled like the Processes table: the sorted column carries the load bar. */
function TopProcesses({ s, wide }: { s: Snapshot; wide: boolean }) {
  const [by, setBy] = useState<"cpu" | "mem">("cpu");
  // A second click on the sorted column turns it round: least first.
  const [desc, setDesc] = useState(true);
  const list = [...s.processes].sort((a, b) => (desc ? b[by] - a[by] : a[by] - b[by]) || a.name.localeCompare(b.name)).slice(0, 8);
  const most = Math.max(1, ...list.map((p) => p.mem));
  const head = (id: "cpu" | "mem", label: string) => (
    <th className="r" aria-sort={by === id ? (desc ? "descending" : "ascending") : "none"}>
      <button
        onClick={() => {
          setDesc(by === id ? !desc : true);
          setBy(id);
        }}
      >
        {label}
        <span className="sort">{by === id && <Icon name={desc ? "sortDesc" : "sortAsc"} size={12} />}</span>
      </button>
    </th>
  );
  const val = (on: boolean, w: number, text: string) => (
    <td className={`r num${on && w > 0.5 ? " bar" : ""}${on ? " strong" : ""}`} style={on ? ({ "--w": `${Math.min(100, w)}%` } as React.CSSProperties) : undefined}>
      {text}
    </td>
  );
  return (
    <section className={`opanel${wide ? " full" : " wide"}`} aria-label={t("Top processes")}>
      <header>
        <h2>{t("Top processes")}</h2>
        <span className="muted num">{t("{n} running", { n: s.processes.length })}</span>
      </header>
      <table className="ptop">
        <thead>
          <tr>
            <th>{t("Name")}</th>
            {head("cpu", "CPU")}
            {head("mem", t("Memory"))}
          </tr>
        </thead>
        <tbody>
          {list.map((p) => (
            <tr key={p.pid}>
              <td className="pname">
                <span className="pcell">
                  <ProcIcon exe={p.exe} />
                  <span className="ptext">{p.name}</span>
                </span>
              </td>
              {val(by === "cpu", p.cpu, pct(p.cpu, 1))}
              {val(by === "mem", (p.mem / most) * 100, bytes(p.mem))}
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}

export function Overview({ s, h, open }: { s: Snapshot; h: History; open: Open }) {
  // CPU and GPU temperatures sit in their own panels; this lists the rest.
  const tctl = cpuTemp(s);
  const sensors = s.temps.filter((x) => x !== tctl);
  return (
    <div className="page">
      <header className="phead">
        <h1>{t("Overview")}</h1>
        <p className="muted num">{t("{host} · {os} (build {build}) · up {up}", { host: s.host.name, os: s.host.os_version, build: s.host.kernel, up: dur(s.host.uptime_s) })}</p>
      </header>
      <div className={`ogrid${s.gpu ? "" : " nogpu"}`}>
        <Cpu s={s} h={h} open={open} />
        <Gpu s={s} h={h} open={open} />
        <Memory s={s} open={open} />
        <Disks s={s} h={h} open={open} />
        <Network s={s} h={h} open={open} />
        <TopProcesses s={s} wide={sensors.length === 0} />
        {sensors.length > 0 && (
          <Panel title={t("Other sensors")}>
            <table className="mini-table">
              <tbody>
                {sensors.map((x) => (
                  <tr key={x.label}>
                    <td>{x.label}</td>
                    <td className="bar-cell">
                      <Bar value={x.celsius} color={heatTemp(x.celsius)} />
                    </td>
                    <td className="r num">{temp(x.celsius)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </Panel>
        )}
      </div>
    </div>
  );
}
