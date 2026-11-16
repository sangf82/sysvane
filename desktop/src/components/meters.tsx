import { bytes } from "../lib/format";
import { t } from "../lib/i18n";
import type { Mem } from "../wire";
/** Fill colour for a usage meter. Text is never coloured by state: emerald and yellow fail contrast on the canvas. */
export const heat = (v: number) => (v < 60 ? "var(--accent)" : v < 85 ? "var(--warn)" : "var(--hot)");
/** Temperature where the meter turns red and the graphs draw their warning line. */
export const TEMP_WARN = 90;
export const heatTemp = (c: number) => (c < 75 ? "var(--accent)" : c < TEMP_WARN ? "var(--warn)" : "var(--hot)");

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

/** Physical memory as one bar: in use, cached, free, with its legend. */
export function MemSplit({ m }: { m: Mem }) {
  const cached = Math.min(m.cached, Math.max(0, m.total - m.used));
  const free = Math.max(0, m.total - m.used - cached);
  const parts = [
    { label: t("In use"), v: m.used, color: "var(--accent)" },
    { label: t("Cached"), v: cached, color: "var(--series-2)" },
    { label: t("Free"), v: free, color: "var(--hairline-cool)" },
  ];
  return (
    <>
      <div className="compose" role="img" aria-label={parts.map((p) => `${p.label} ${bytes(p.v)}`).join(", ")}>
        {parts.map((p) => (
          <div key={p.label} style={{ width: `${(p.v / (m.total || 1)) * 100}%`, background: p.color }} />
        ))}
      </div>
      <div className="legend">
        {parts.map((p) => (
          <span key={p.label}>
            <i style={{ background: p.color }} />
            {p.label}
          </span>
        ))}
      </div>
    </>
  );
}
