import { useState } from "react";
import { t } from "../lib/i18n";
import { RES, cpuTemp, cpuTemps, diskName, memPercent, netsByTraffic } from "../lib/metrics";
import { type History, K } from "../lib/series";
import { bytes, dur, ghz, pct, rate, temp, watts } from "../lib/format";
import { Chart, Graph, PercentChart, TempChart } from "../components/charts";
import { Flow, Stats, TempHint } from "../components/layout";
import { MemSplit } from "../components/meters";
import type { Disk, Gpu, Net, Snapshot } from "../wire";

const of = (a: string, b: string) => t("{a} of {b}", { a, b });

type Item = { key: string; title: string; sub: string; value: string; hist: number[]; max?: number; detail: React.ReactNode };

function items(s: Snapshot, h: History): Item[] {
  const tctl = cpuTemp(s);
  const list: Item[] = [
    { key: RES.cpu, title: "CPU", sub: `${s.cpu.mhz != null ? ghz(s.cpu.mhz) : t("clock n/a")}${tctl ? `, ${temp(tctl.celsius)}` : ""}`, value: pct(s.cpu.usage), hist: h[K.cpu] ?? [], max: 100, detail: <CpuDetail s={s} h={h} /> },
    { key: RES.mem, title: t("Memory"), sub: of(bytes(s.mem.used), bytes(s.mem.total)), value: pct(memPercent(s)), hist: h[K.mem] ?? [], max: 100, detail: <MemDetail s={s} h={h} /> },
  ];
  if (s.gpu) list.push({ key: RES.gpu, title: "GPU", sub: s.gpu.name.replace("NVIDIA GeForce ", ""), value: pct(s.gpu.util), hist: h[K.gpu] ?? [], max: 100, detail: <GpuDetail g={s.gpu} h={h} /> });
  s.disks.forEach((d) => list.push({ key: RES.disk(d), title: t("Disk {d}", { d: diskName(d) }), sub: `${d.kind}, ${d.fs}`, value: d.active_pct != null ? pct(d.active_pct) : t("n/a"), hist: h[K.disk(d.mount, "act")] ?? [], max: 100, detail: <DiskDetail d={d} h={h} /> }));
  netsByTraffic(s).forEach((n) => list.push({ key: RES.net(n), title: n.name, sub: t("down {r}, up {t}", { r: rate(n.rx_bps), t: rate(n.tx_bps) }), value: rate(n.rx_bps + n.tx_bps), hist: h[K.net(n.name, "rx")] ?? [], detail: <NetDetail n={n} h={h} /> }));
  return list;
}

function Head({ title, sub }: { title: string; sub?: string }) {
  return (
    <header className="dhead">
      <h2>{title}</h2>
      {sub && <span className="muted">{sub}</span>}
    </header>
  );
}

function CpuDetail({ s, h }: { s: Snapshot; h: History }) {
  const [view, setView] = useState<"overall" | "cores">("overall");
  const temps = cpuTemps(s);
  const tctl = cpuTemp(s);
  const mhzMax = Math.max((s.cpu.base_mhz ?? 0) * 1.4, ...(h[K.cpuMhz] ?? [0]), 1000);
  return (
    <>
      <Head title="CPU" sub={s.cpu.name} />
      <Stats
        items={[
          { label: t("Utilization"), value: pct(s.cpu.usage), big: true },
          { label: t("Speed"), value: s.cpu.mhz != null ? ghz(s.cpu.mhz) : t("n/a"), big: true },
          { label: t("Temperature"), value: tctl ? temp(tctl.celsius) : t("n/a"), big: true },
        ]}
      />
      <div className="seg" role="group" aria-label={t("CPU view")}>
        <button className={view === "overall" ? "on" : ""} aria-pressed={view === "overall"} onClick={() => setView("overall")}>
          {t("Overall")}
        </button>
        <button className={view === "cores" ? "on" : ""} aria-pressed={view === "cores"} onClick={() => setView("cores")}>
          {t("Logical processors")}
        </button>
      </div>
      {view === "overall" ? (
        <PercentChart title={t("Utilization")} data={h[K.cpu] ?? []} tall />
      ) : (
        <div className="coregrid">
          {s.cpu.cores.map((c, i) => (
            <Chart key={i} title={`CPU ${i}`} scale={pct(c)}>
              <Graph data={h[K.core(i)] ?? []} max={100} />
            </Chart>
          ))}
        </div>
      )}
      {s.cpu.mhz != null && (
        <Chart title={t("Clock speed")} scale={`${(mhzMax / 1000).toFixed(1)} GHz`}>
          <Graph data={h[K.cpuMhz] ?? []} max={mhzMax} />
        </Chart>
      )}
      {tctl && <TempChart title={t("Temperature (Tctl)")} data={h[K.cpuTemp] ?? []} />}
      <Stats
        items={[
          { label: t("Base speed"), value: s.cpu.base_mhz != null ? ghz(s.cpu.base_mhz) : t("n/a") },
          { label: t("Cores"), value: s.cpu.physical ?? t("n/a") },
          { label: t("Logical processors"), value: s.cpu.logical },
          { label: t("Processes"), value: s.host.processes },
          { label: t("Threads"), value: s.host.threads.toLocaleString() },
          { label: t("Handles"), value: s.host.handles.toLocaleString() },
          { label: t("Up time"), value: dur(s.host.uptime_s) },
          { label: t("Vendor"), value: s.cpu.vendor },
          { label: t("Architecture"), value: s.host.arch },
          ...temps.filter((x) => x !== tctl).map((x) => ({ label: x.label, value: temp(x.celsius) })),
        ]}
      />
      {!tctl && <TempHint />}
    </>
  );
}

function MemDetail({ s, h }: { s: Snapshot; h: History }) {
  const m = s.mem;
  return (
    <>
      <Head title={t("Memory")} sub={bytes(m.total)} />
      <Stats
        items={[
          { label: t("In use"), value: bytes(m.used), big: true },
          { label: t("Available"), value: bytes(m.available), big: true },
          { label: t("Committed"), value: of(bytes(m.commit_used), bytes(m.commit_limit)), big: true },
        ]}
      />
      <Chart title={t("Memory in use")} scale={bytes(m.total)} tall>
        <Graph data={h[K.mem] ?? []} max={100} />
      </Chart>
      <MemSplit m={m} />
      <Chart title={t("Committed")} scale={bytes(m.commit_limit)}>
        <Graph data={h[K.commit] ?? []} max={100} muted />
      </Chart>
      <Stats
        items={[
          { label: t("Cached"), value: bytes(m.cached) },
          { label: t("Paged pool"), value: bytes(m.paged_pool) },
          { label: t("Non-paged pool"), value: bytes(m.nonpaged_pool) },
          { label: t("Swap"), value: of(bytes(m.swap_used), bytes(m.swap_total)) },
          { label: t("Total"), value: bytes(m.total) },
        ]}
      />
    </>
  );
}

function GpuDetail({ g, h }: { g: Gpu; h: History }) {
  return (
    <>
      <Head title="GPU" sub={g.name} />
      <Stats
        items={[
          { label: t("Utilization"), value: pct(g.util), big: true },
          { label: t("Dedicated memory"), value: of(bytes(g.mem_used), bytes(g.mem_total)), big: true },
          { label: t("Temperature"), value: g.temp_c != null ? temp(g.temp_c) : t("n/a"), big: true },
        ]}
      />
      <PercentChart title={t("Utilization")} data={h[K.gpu] ?? []} tall />
      <Chart title={t("Dedicated memory")} scale={bytes(g.mem_total)}>
        <Graph data={h[K.gpuVram] ?? []} max={100} muted />
      </Chart>
      {g.temp_c != null && <TempChart title={t("Temperature")} data={h[K.gpuTemp] ?? []} />}
      {g.power_w != null && (
        <Chart title={t("Power draw")} scale={g.power_limit_w ? t("{w} W limit", { w: g.power_limit_w.toFixed(0) }) : undefined}>
          <Graph data={h[K.gpuPower] ?? []} max={g.power_limit_w ?? undefined} muted />
        </Chart>
      )}
      <Stats
        items={[
          { label: t("Power"), value: g.power_w != null ? (g.power_limit_w ? t("{a} W of {b} W", { a: g.power_w.toFixed(0), b: g.power_limit_w.toFixed(0) }) : watts(g.power_w)) : t("n/a") },
          { label: t("Graphics clock"), value: g.clock_gr_mhz != null ? `${g.clock_gr_mhz} MHz` : t("n/a") },
          { label: t("Memory clock"), value: g.clock_mem_mhz != null ? `${g.clock_mem_mhz} MHz` : t("n/a") },
          { label: t("Fan"), value: g.fan_pct != null ? `${g.fan_pct}%` : t("n/a") },
          { label: t("Video encode"), value: g.enc_util != null ? pct(g.enc_util) : t("n/a") },
          { label: t("Video decode"), value: g.dec_util != null ? pct(g.dec_util) : t("n/a") },
          { label: "PCI Express", value: g.pcie_gen != null ? `Gen ${g.pcie_gen} x${g.pcie_width ?? "?"}` : t("n/a") },
          { label: t("Driver version"), value: g.driver ?? t("n/a") },
        ]}
      />
    </>
  );
}

function DiskDetail({ d, h }: { d: Disk; h: History }) {
  const r = h[K.disk(d.mount, "r")] ?? [];
  const w = h[K.disk(d.mount, "w")] ?? [];
  const top = Math.max(1024 * 1024, ...r, ...w);
  return (
    <>
      <Head title={t("Disk {d}", { d: diskName(d) })} sub={`${d.name || d.mount}, ${d.kind}`} />
      <Stats
        items={[
          { label: t("Active time"), value: d.active_pct != null ? pct(d.active_pct, 1) : t("n/a"), big: true },
          { label: t("Average response time"), value: d.resp_ms != null ? `${d.resp_ms.toFixed(2)} ms` : t("n/a"), big: true },
          { label: t("Capacity"), value: bytes(d.total), big: true },
        ]}
      />
      <PercentChart title={t("Active time")} data={h[K.disk(d.mount, "act")] ?? []} tall />
      <Chart title={t("Transfer rate, read above and write below")} scale={rate(top)}>
        <div className="split">
          <Graph data={r} max={top} />
          <Graph data={w} max={top} flip muted />
        </div>
      </Chart>
      <Stats
        items={[
          { label: t("Read speed"), value: rate(d.read_bps) },
          { label: t("Write speed"), value: rate(d.write_bps) },
          { label: t("Used"), value: bytes(d.used) },
          { label: t("Free"), value: bytes(d.total - d.used) },
          { label: t("Type"), value: d.kind },
          { label: t("File system"), value: d.fs },
          { label: t("Volume name"), value: d.name || t("none") },
          { label: t("Mount point"), value: d.mount },
          { label: t("Removable"), value: d.removable ? t("Yes") : t("No") },
        ]}
      />
    </>
  );
}

function NetDetail({ n, h }: { n: Net; h: History }) {
  const rx = h[K.net(n.name, "rx")] ?? [];
  const tx = h[K.net(n.name, "tx")] ?? [];
  const top = Math.max(100 * 1024, ...rx, ...tx);
  const v4 = n.ips.filter((ip) => !ip.includes(":"));
  const v6 = n.ips.filter((ip) => ip.includes(":"));
  return (
    <>
      <Head title={n.name} sub={n.mac} />
      <Stats
        items={[
          { label: t("Receive"), value: <Flow dir="down">{rate(n.rx_bps)}</Flow>, big: true },
          { label: t("Send"), value: <Flow dir="up">{rate(n.tx_bps)}</Flow>, big: true },
        ]}
      />
      <Chart title={t("Throughput, receive above and send below")} scale={rate(top)} tall>
        <div className="split">
          <Graph data={rx} max={top} />
          <Graph data={tx} max={top} flip muted />
        </div>
      </Chart>
      <Stats
        items={[
          { label: t("Total received"), value: bytes(n.rx_total) },
          { label: t("Total sent"), value: bytes(n.tx_total) },
          { label: "MTU", value: n.mtu },
          { label: t("MAC address"), value: n.mac },
          { label: t("IPv4 address"), value: v4.length ? v4.join(", ") : t("none") },
          { label: t("IPv6 addresses"), value: v6.length ? v6.map((ip) => <div key={ip}>{ip}</div>) : t("none") },
        ]}
      />
    </>
  );
}

export function Performance({ s, h, sel, setSel }: { s: Snapshot; h: History; sel: string; setSel: (resource: string) => void }) {
  const list = items(s, h);
  const current = list.find((i) => i.key === sel) ?? list[0];
  return (
    <div className="perf">
      <nav className="reslist" aria-label={t("Resources")}>
        {list.map((i) => (
          <button key={i.key} className={`res ${i.key === current.key ? "on" : ""}`} aria-current={i.key === current.key} onClick={() => setSel(i.key)}>
            <span className="res-graph">
              <Graph data={i.hist} max={i.max} />
            </span>
            <span className="res-text">
              <b>{i.title}</b>
              <span className="muted num">{i.sub}</span>
            </span>
            <span className="num res-val">{i.value}</span>
          </button>
        ))}
      </nav>
      <div className="detail">{current.detail}</div>
    </div>
  );
}
