import { t } from "../lib/i18n";
import { ActivityIcon } from "../icons/activity";
import { ArrowDownIcon } from "../icons/arrow-down";
import { ArrowUpIcon } from "../icons/arrow-up";
import { ChevronDownIcon } from "../icons/chevron-down";
import { ChevronUpIcon } from "../icons/chevron-up";
import { LayoutDashboardIcon } from "../icons/layout-dashboard";
import { ListIcon } from "../icons/list";
import { MonitorIcon } from "../icons/monitor";
import { SearchIcon } from "../icons/search";
import { SlidersIcon } from "../icons/sliders";

/** Lucide icons with Animate UI's motion, one set for the whole app. Hover animation comes from the parent AnimateIcon. */
const ICONS = {
  overview: LayoutDashboardIcon,
  performance: ActivityIcon,
  processes: ListIcon,
  displays: MonitorIcon,
  down: ArrowDownIcon,
  up: ArrowUpIcon,
  sortDesc: ChevronDownIcon,
  sortAsc: ChevronUpIcon,
  search: SearchIcon,
  settings: SlidersIcon,
} as const;
export type IconName = keyof typeof ICONS;

export function Icon({ name, size }: { name: IconName; size: number }) {
  const Glyph = ICONS[name];
  return <Glyph className="icon" size={size} aria-hidden />;
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

/** Shown where a CPU temperature would be while the sensor helper is not set up. */
export function TempHint() {
  return <p className="muted">{t("CPU temperature needs administrator rights. The installed SysVane starts with them.")}</p>;
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
