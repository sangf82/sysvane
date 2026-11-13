import { t } from "./i18n";
import { ActivityIcon } from "./icons/activity";
import { ArrowDownIcon } from "./icons/arrow-down";
import { ArrowUpIcon } from "./icons/arrow-up";
import { ChevronDownIcon } from "./icons/chevron-down";
import { ChevronUpIcon } from "./icons/chevron-up";
import { LayoutDashboardIcon } from "./icons/layout-dashboard";
import { ListIcon } from "./icons/list";
import { SearchIcon } from "./icons/search";
import { HISTORY, HISTORY_MINUTES } from "./series";

/** Fill colour for a usage meter. Text is never coloured by state: emerald and yellow fail contrast on the canvas. */
export const heat = (v: number) => (v < 60 ? "var(--accent)" : v < 85 ? "var(--warn)" : "var(--hot)");
/** Temperature where the meter turns red and the graphs draw their warning line. */
export const TEMP_WARN = 90;
export const heatTemp = (c: number) => (c < 75 ? "var(--accent)" : c < TEMP_WARN ? "var(--warn)" : "var(--hot)");

/** Lucide icons with Animate UI's motion, one set for the whole app. Hover animation comes from the parent AnimateIcon. */
const ICONS = {
  overview: LayoutDashboardIcon,
  performance: ActivityIcon,
  processes: ListIcon,
  down: ArrowDownIcon,
  up: ArrowUpIcon,
  sortDesc: ChevronDownIcon,
  sortAsc: ChevronUpIcon,
  search: SearchIcon,
} as const;
export type IconName = keyof typeof ICONS;

export function Icon({ name, size }: { name: IconName; size: number }) {
  const Glyph = ICONS[name];
  return <Glyph className="icon" size={size} aria-hidden />;
}

/** The Crate mark, small cut. Paths are a copy of branding/logo/kit/sysvane-symbol-small.svg: update them when the kit is rebuilt.
 *  Side faces follow --ink so they stay visible in both themes. */
export function Logo() {
  return (
    <svg className="logo" viewBox="0 0 256 256" aria-hidden>
      <path fill="var(--ink)" d="M118,242.23 L24.08,188 L24.08,79.55 L118,133.77Z M231.92,79.55 L231.92,188 L138,242.23 L138,133.77Z" />
      <path fill="var(--primary)" d="M128,8 L221.92,62.23 L128,116.45 L34.08,62.23Z" />
    </svg>
  );
}

/** Direction of traffic, named in text for screen readers. */
export function Flow({ dir, children }: { dir: "down" | "up"; children: React.ReactNode }) {
  return (
    <span className="flow">
      <Icon name={dir} size={12} />
      <span className="sr">{dir === "down" ? t("Down") : t("Up")} </span>
      {children}
    </span>
  );
}

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

/** Shown where a CPU temperature would be while the sensor helper is not set up. */
export function TempHint() {
  return <p className="muted">{t("CPU temperature needs administrator rights. The installed SysVane starts with them.")}</p>;
}

/** A filled bar, 0 to 100 %. Given a label it is announced as a meter. */
export function Bar({ value, color, label }: { value: number; color: string; label?: string }) {
  const v = Math.min(100, Math.max(0, value));
  return (
    <div className="track" {...(label ? { role: "meter", "aria-label": label, "aria-valuemin": 0, "aria-valuemax": 100, "aria-valuenow": Math.round(v) } : {})}>
      <div className="fill" style={{ width: `${v}%`, background: color }} />
    </div>
  );
}

export function Meter({ label, value, right }: { label: string; value: number; right: string }) {
  return (
    <div className="meter">
      <div className="mhead">
        <span>{label}</span>
        <span className="num">{right}</span>
      </div>
      <Bar value={value} color={heat(value)} label={label} />
    </div>
  );
}

export function Section({ title, right, children }: { title: string; right?: React.ReactNode; children: React.ReactNode }) {
  return (
    <section className="section">
      <header>
        <h2>{title}</h2>
        {right && <span className="muted num">{right}</span>}
      </header>
      {children}
    </section>
  );
}

/** Label and value pairs. `big` ones are the headline numbers. */
export function Stats({ items }: { items: { label: string; value: React.ReactNode; big?: boolean }[] }) {
  return (
    <dl className="stats">
      {items.map((i) => (
        <div key={i.label} className={i.big ? "big" : ""}>
          <dt>{i.label}</dt>
          <dd className="num">{i.value}</dd>
        </div>
      ))}
    </dl>
  );
}
