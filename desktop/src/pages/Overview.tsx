import { t } from "../i18n";
import { RES, cpuTemp, diskName, memPercent, netTotals, netsByTraffic } from "../metrics";
import { HISTORY_MINUTES, type History, K } from "../series";
import { bytes, dur, ghz, pct, rate } from "../format";
import { Bar, Flow, Graph, Section, Stats, TEMP_WARN, TempHint, heat, heatTemp } from "../ui";
import type { Proc, Snapshot } from "../wire";

type Open = (resource: string) => void;

type Row = {
  key: string;
  /** The Performance page resource this row opens, if it has one. */
  open: string | null;
  label: string;
  /** 0-100 for a meter, null when the resource has no capacity (network). */
  usage: number | null;
  value: string;
  detail: React.ReactNode;
  hist: number[];
  max?: number;
};

function Ledger({ rows, open }: { rows: Row[]; open: Open }) {
  return (
    <table className="ledger">
      <thead>
        <tr>
          <th>{t("Resource")}</th>
          <th>{t("Usage")}</th>
          <th className="r">{t("Now")}</th>
          <th>{t("Detail")}</th>
          <th>{t("Last {n} minutes", { n: HISTORY_MINUTES })}</th>
        </tr>
      </thead>
      <tbody>
        {rows.map((r) => (
          <tr key={r.key}>
            <th scope="row">
              {r.open ? (
                <button className="link" onClick={() => open(r.open!)}>
                  {r.label}
                </button>
              ) : (
                r.label
              )}
            </th>
            <td>
              {r.usage != null && <Bar value={r.usage} color={heat(r.usage)} label={`${r.label} ${t("Usage")}`} />}
            </td>
            <td className="r num strong">{r.value}</td>
            <td className="num muted">{r.detail}</td>
            <td className="trend">
              <Graph data={r.hist} max={r.max} />
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function TopProcesses({ title, list, value, bar }: { title: string; list: Proc[]; value: (p: Proc) => string; bar: (p: Proc) => number }) {
  return (
    <Section title={title}>
      <table className="mini-table">
        <tbody>
          {list.map((p) => (
            <tr key={p.pid}>
              <td className="pname">{p.name}</td>
              <td className="bar-cell">
                <Bar value={bar(p)} color="var(--accent)" />
              </td>
              <td className="r num">{value(p)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </Section>
  );
}

function ledgerRows(s: Snapshot, h: History): Row[] {
  const memPct = memPercent(s);
  const tctl = cpuTemp(s);
  const { rx, tx } = netTotals(s);
  const busiestNet = netsByTraffic(s)[0];
  const rows: Row[] = [
    { key: RES.cpu, open: RES.cpu, label: "CPU", usage: s.cpu.usage, value: pct(s.cpu.usage), detail: `${s.cpu.mhz != null ? ghz(s.cpu.mhz) : t("clock unavailable")}${tctl ? `, ${tctl.celsius.toFixed(0)} °C` : ""}`, hist: h[K.cpu] ?? [], max: 100 },
    { key: RES.mem, open: RES.mem, label: t("Memory"), usage: memPct, value: pct(memPct), detail: t("{used} of {total}, {cached} cached", { used: bytes(s.mem.used), total: bytes(s.mem.total), cached: bytes(s.mem.cached) }), hist: h[K.mem] ?? [], max: 100 },
  ];
  if (s.gpu)
    rows.push({
      key: RES.gpu,
      open: RES.gpu,
      label: "GPU",
      usage: s.gpu.util,
      value: pct(s.gpu.util),
      detail: `${t("{v} VRAM", { v: bytes(s.gpu.mem_used) })}${s.gpu.temp_c != null ? `, ${s.gpu.temp_c} °C` : ""}${s.gpu.power_w != null ? `, ${s.gpu.power_w.toFixed(0)} W` : ""}`,
      hist: h[K.gpu] ?? [],
      max: 100,
    });
  s.disks.forEach((d) =>
    rows.push({
      key: RES.disk(d),
      open: RES.disk(d),
      label: t("Disk {d}", { d: diskName(d) }),
      usage: d.active_pct,
      value: d.active_pct != null ? pct(d.active_pct) : t("n/a"),
      detail: t("read {r}, write {w}", { r: rate(d.read_bps), w: rate(d.write_bps) }),
      hist: h[K.disk(d.mount, "act")] ?? [],
      max: 100,
    }),
  );
  rows.push({
    key: "net",
    open: busiestNet ? RES.net(busiestNet) : null,
    label: t("Network"),
    usage: null,
    value: rate(rx + tx),
    detail: (
      <>
        <Flow dir="down">{rate(rx)}</Flow> <Flow dir="up">{rate(tx)}</Flow>
      </>
    ),
    hist: h[K.netRx] ?? [],
  });
  return rows;
}

export function Overview({ s, h, open }: { s: Snapshot; h: History; open: Open }) {
  const tctl = cpuTemp(s);
  const topCpu = [...s.processes].sort((a, b) => b.cpu - a.cpu).slice(0, 8);
  const topMem = [...s.processes].sort((a, b) => b.mem - a.mem).slice(0, 8);
  const temps = [...(s.gpu?.temp_c != null ? [{ label: "GPU", celsius: s.gpu.temp_c }] : []), ...s.temps];
  return (
    <div className="page">
      <header className="phead">
        <h1>{t("Overview")}</h1>
        <p className="muted num">{t("{host} · {os} (build {build}) · up {up}", { host: s.host.name, os: s.host.os_version, build: s.host.kernel, up: dur(s.host.uptime_s) })}</p>
      </header>
      <Ledger rows={ledgerRows(s, h)} open={open} />
      <Section title={t("Temperatures")} right={t("red line at {t} °C", { t: TEMP_WARN })}>
        {temps.length > 0 ? (
          <table className="mini-table">
            <tbody>
              {temps.map((x) => (
                <tr key={x.label}>
                  <td>{x.label}</td>
                  <td className="bar-cell">
                    <Bar value={x.celsius} color={heatTemp(x.celsius)} />
                  </td>
                  <td className="r num">{x.celsius.toFixed(0)} °C</td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <p className="muted">{t("No sensor is readable.")}</p>
        )}
        {!tctl && <TempHint />}
      </Section>
      <div className="two">
        <TopProcesses title={t("Top CPU")} list={topCpu} value={(p) => pct(p.cpu, 1)} bar={(p) => p.cpu} />
        <TopProcesses title={t("Top memory")} list={topMem} value={(p) => bytes(p.mem)} bar={(p) => (p.mem / (topMem[0]?.mem || 1)) * 100} />
      </div>
      <Section title={t("System")}>
        <Stats
          items={[
            { label: t("Processor"), value: s.cpu.name },
            { label: t("Cores / threads"), value: `${s.cpu.physical ?? t("n/a")} / ${s.cpu.logical}` },
            ...(s.gpu ? [{ label: t("Graphics"), value: s.gpu.name }, { label: t("Graphics driver"), value: s.gpu.driver ?? t("n/a") }] : []),
            { label: t("Memory"), value: bytes(s.mem.total) },
            { label: t("Processes"), value: s.host.processes },
            { label: t("Threads"), value: s.host.threads.toLocaleString() },
            { label: t("Handles"), value: s.host.handles.toLocaleString() },
          ]}
        />
      </Section>
    </div>
  );
}
