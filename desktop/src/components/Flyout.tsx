import { openDashboard } from "./host";
import { t } from "./i18n";
import { cpuTemp, memPercent, netTotals } from "./metrics";
import { type History, K } from "./series";
import { bytes, pct, rate } from "./format";
import { Flow, Graph, Meter } from "./ui";
import type { Snapshot } from "./wire";

function Block({ title, right, children }: { title: string; right?: string; children: React.ReactNode }) {
  return (
    <section className="fblock">
      <header>
        <h2>{title}</h2>
        {right && <span className="num">{right}</span>}
      </header>
      {children}
    </section>
  );
}

export function Flyout({ s, h }: { s: Snapshot; h: History }) {
  const tctl = cpuTemp(s);
  const { rx, tx } = netTotals(s);
  return (
    <div className="flyout">
      <Block title="CPU" right={`${pct(s.cpu.usage)}${tctl ? `, ${tctl.celsius.toFixed(0)} °C` : ""}`}>
        <div className="spark">
          <Graph data={h[K.cpu] ?? []} max={100} />
        </div>
      </Block>
      {s.gpu && (
        <Block title="GPU" right={`${pct(s.gpu.util)}${s.gpu.temp_c != null ? `, ${s.gpu.temp_c} °C` : ""}`}>
          <div className="spark">
            <Graph data={h[K.gpu] ?? []} max={100} />
          </div>
        </Block>
      )}
      <Block title={t("Memory")}>
        <Meter label="RAM" value={memPercent(s)} right={t("{a} of {b}", { a: bytes(s.mem.used), b: bytes(s.mem.total) })} />
      </Block>
      <Block title={t("Network")}>
        <div className="row">
          <Flow dir="down">{t("Receive")}</Flow>
          <span className="num">{rate(rx)}</span>
        </div>
        <div className="row">
          <Flow dir="up">{t("Send")}</Flow>
          <span className="num">{rate(tx)}</span>
        </div>
      </Block>
      <button className="primary" onClick={() => openDashboard()}>
        {t("Open dashboard")}
      </button>
    </div>
  );
}
