import { t } from "../lib/i18n";
import { HISTORY, HISTORY_MINUTES } from "../lib/series";
import { TEMP_WARN } from "./meters";

/** History graph, newest sample at the right edge. `flip` hangs it from the top, `muted` draws the secondary series. */
export function Graph({ data, max, flip, muted, refs }: { data: number[]; max?: number; flip?: boolean; muted?: boolean; refs?: number[] }) {
  if (data.length < 2) return <svg className="graph" aria-hidden />;
  const top = max ?? Math.max(1, ...data);
  const x = (i: number) => ((HISTORY - data.length + i) / (HISTORY - 1)) * 100;
  const y = (v: number) => {
    const f = (Math.min(v, top) / top) * 96;
    return flip ? f : 100 - f;
  };
  const pts = data.map((v, i) => `${x(i)},${y(v)}`);
  const base = flip ? 0 : 100;
  const color = muted ? "var(--series-2)" : "var(--accent)";
  return (
    <svg className="graph" viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden>
      <path d={`M${x(0)},${base} L${pts.join(" L")} L${x(data.length - 1)},${base} Z`} fill={color} opacity={muted ? 0.1 : 0.18} />
      <polyline points={pts.join(" ")} fill="none" stroke={color} strokeWidth="1.5" vectorEffect="non-scaling-stroke" />
      {refs?.map((r) => (
        <line key={r} x1="0" x2="100" y1={y(r)} y2={y(r)} stroke="var(--hot)" strokeWidth="1" strokeDasharray="4 4" vectorEffect="non-scaling-stroke" />
      ))}
    </svg>
  );
}

/** A graph in a framed plot with a title, the scale maximum and a time axis. The grid under it is the measuring scale. */
export function Chart({ title, scale, children, tall }: { title: string; scale?: string; children: React.ReactNode; tall?: boolean }) {
  return (
    <figure className={`chart ${tall ? "tall" : ""}`}>
      <figcaption>
        <span>{title}</span>
        <span className="num">{scale}</span>
      </figcaption>
      <div className="plot">{children}</div>
      <div className="axis">
        <span>{t("{n} minutes ago", { n: HISTORY_MINUTES })}</span>
        <span>{t("now")}</span>
      </div>
    </figure>
  );
}

/** A usage history on a 0 to 100 % scale. */
export function PercentChart({ title, data, tall }: { title: string; data: number[]; tall?: boolean }) {
  return (
    <Chart title={title} scale="100%" tall={tall}>
      <Graph data={data} max={100} />
    </Chart>
  );
}

/** A temperature history on a 0 to 100 °C scale, with the warning line. */
export function TempChart({ title, data }: { title: string; data: number[] }) {
  return (
    <Chart title={title} scale="100 °C">
      <Graph data={data} max={100} refs={[TEMP_WARN]} />
    </Chart>
  );
}
