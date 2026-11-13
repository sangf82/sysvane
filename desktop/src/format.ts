const KIB = 1024;
const MIB = KIB ** 2;
const GIB = KIB ** 3;
const TIB = KIB ** 4;

export const bytes = (b: number) => (b >= TIB ? `${(b / TIB).toFixed(2)} TiB` : b >= GIB ? `${(b / GIB).toFixed(1)} GiB` : b >= MIB ? `${(b / MIB).toFixed(0)} MiB` : `${(b / KIB).toFixed(0)} KiB`);
export const rate = (b: number) => (b >= GIB ? `${(b / GIB).toFixed(2)} GiB/s` : b >= MIB ? `${(b / MIB).toFixed(1)} MiB/s` : `${(b / KIB).toFixed(0)} KiB/s`);
export const pct = (v: number, digits = 0) => `${v.toFixed(digits)}%`;
export const ghz = (mhz: number) => `${(mhz / 1000).toFixed(2)} GHz`;
export const dur = (s: number) => {
  const d = Math.floor(s / 86400);
  const hms = [Math.floor((s % 86400) / 3600), Math.floor((s % 3600) / 60), Math.floor(s % 60)].map((n) => String(n).padStart(2, "0")).join(":");
  return d ? `${d}d ${hms}` : hms;
};
