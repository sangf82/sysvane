import { useEffect, useRef, useState } from "react";
import { Alert } from "../components/Alert";
import { Dropdown } from "../components/Dropdown";
import { sendCommand } from "../lib/host";
import { type Msg, t } from "../lib/i18n";
import { type ControlId, type ControlValue, type Device, TICK_S } from "../wire";

type Slide = { id: ControlId; label: Msg };
const PICTURE: Slide[] = [
  { id: "brightness", label: "Brightness" },
  { id: "contrast", label: "Contrast" },
];
const GAINS: Slide[] = [
  { id: "red_gain", label: "Red gain" },
  { id: "green_gain", label: "Green gain" },
  { id: "blue_gain", label: "Blue gain" },
];
// Colour through the GPU gamma ramp, which the core offers for every display.
const GPU: Slide[] = [
  { id: "gpu_brightness", label: "Brightness" },
  { id: "gpu_contrast", label: "Contrast" },
  { id: "gpu_gamma", label: "Gamma" },
  { id: "gpu_red", label: "Red" },
  { id: "gpu_green", label: "Green" },
  { id: "gpu_blue", label: "Blue" },
];
const GPU_NEUTRAL: Record<string, number> = { gpu_brightness: 50, gpu_contrast: 50, gpu_gamma: 50, gpu_red: 100, gpu_green: 100, gpu_blue: 100 };
// Quarter turns clockwise, as Windows numbers them.
const TURNS: Msg[] = ["Landscape", "Portrait", "Landscape (flipped)", "Portrait (flipped)"];
// Codes the core offers as `{ vcp }` controls only when the monitor lists them, as a percentage.
const vcp = (code: number, label: Msg): Slide => ({ id: { vcp: code }, label });
const MORE_PICTURE: Slide[] = [vcp(0x62, "Volume")];
const MORE_COLOUR: Slide[] = [
  vcp(0x6c, "Red black level"),
  vcp(0x6e, "Green black level"),
  vcp(0x70, "Blue black level"),
  vcp(0x59, "Red saturation"),
  vcp(0x5a, "Yellow saturation"),
  vcp(0x5b, "Green saturation"),
  vcp(0x5c, "Cyan saturation"),
  vcp(0x5d, "Blue saturation"),
  vcp(0x5e, "Magenta saturation"),
  vcp(0x9b, "Red hue"),
  vcp(0x9c, "Yellow hue"),
  vcp(0x9d, "Green hue"),
  vcp(0x9e, "Cyan hue"),
  vcp(0x9f, "Blue hue"),
  vcp(0xa0, "Magenta hue"),
];

// MCCS colour presets. A monitor ignores one it lacks, and the next read shows what it kept.
const presets = () => ["sRGB", t("Native"), "4000K", "5000K", "6500K", "7500K", "8200K", "9300K", "10000K", "11500K", t("User {n}", { n: 1 }), t("User {n}", { n: 2 }), t("User {n}", { n: 3 })];

/** "2560x1440@165", as the core names a display mode. */
const parseMode = (m: string) => {
  const [size, hz] = m.split("@");
  return { size, hz: Number(hz) };
};
const sizeLabel = (size: string) => size.replace("x", " × ");
const same = (a: ControlId, b: ControlId) => JSON.stringify(a) === JSON.stringify(b);
const text = (d: Device, id: ControlId) => {
  const v = d.controls.find((c) => same(c.id, id))?.value;
  return typeof v === "string" ? v : undefined;
};

const isMonitor = (d: Device) => typeof d.id === "object";
const key = (d: Device) => JSON.stringify(d.id);
const num = (d: Device, id: ControlId) => {
  const v = d.controls.find((c) => same(c.id, id))?.value;
  return typeof v === "number" ? v : undefined;
};

/** The monitor's white point as its gains and brightness push it, relative to the strongest channel. Only values read back are used. */
function tint(d: Device): string | undefined {
  const [r, g, b] = GAINS.map(({ id }) => num(d, id));
  if (r == null || g == null || b == null) return undefined;
  const top = Math.max(r, g, b, 1);
  const light = 0.35 + 0.65 * ((num(d, "brightness") ?? 100) / 100);
  const ch = (v: number) => Math.round((v / top) * 255 * light);
  return `rgb(${ch(r)} ${ch(g)} ${ch(b)})`;
}

/** Holds the dragged value until the monitor reports it, and sends only after the drag pauses: a DDC write takes tens of ms. */
function Slider({ label, value, send }: { label: string; value: number; send: (n: number) => void }) {
  const [pending, setPending] = useState<number | null>(null);
  const timer = useRef(0);
  useEffect(() => {
    if (pending === value) setPending(null);
  }, [value, pending]);
  const change = (n: number) => {
    setPending(n);
    clearTimeout(timer.current);
    timer.current = window.setTimeout(() => {
      send(n);
      window.setTimeout(() => setPending((p) => (p === n ? null : p)), 3000);
    }, 250);
  };
  const shown = pending ?? value;
  return (
    <label className="mslider">
      <span>{label}</span>
      <input type="range" min={0} max={100} value={shown} onChange={(e) => change(Number(e.target.value))} />
      <span className="num">{shown}</span>
    </label>
  );
}

function Card({ d, on, pick }: { d: Device; on: boolean; pick: () => void }) {
  const [model, display] = d.name.includes(" · ") ? d.name.split(" · ") : [d.name, ""];
  const b = num(d, "brightness");
  const c = num(d, "contrast");
  const mode = text(d, "display_mode");
  const m = mode && parseMode(mode);
  const ok = !d.error && d.controls.length > 0;
  return (
    <button className={`mcard${on ? " on" : ""}`} aria-pressed={on} onClick={pick}>
      <span className="mscreen" aria-hidden>
        <span style={{ background: tint(d) }} />
      </span>
      <span className="mtext">
        <b>{display || model}</b>
        <span className="muted">{[display && model, m && `${sizeLabel(m.size)} · ${m.hz} Hz`].filter(Boolean).join(" · ")}</span>
        <span className="status muted">
          <i className={`dot${ok ? " on" : ""}`} />
          {d.error ? t("Not answering") : !ok ? t("Reading the monitor…") : [b != null && `${t("Brightness")} ${b}`, c != null && `${t("Contrast")} ${c}`].filter(Boolean).join(" · ")}
        </span>
      </span>
    </button>
  );
}

function Settings({ d }: { d: Device }) {
  const [error, setError] = useState<string | null>(null);
  const [picked, setProfile] = useState("");
  // The list arrives after the first read, so the choice falls back to its first entry until one is picked.
  const profile = d.profiles.includes(picked) ? picked : (d.profiles[0] ?? "");
  const [name, setName] = useState("");
  const send = (control: ControlId, value: ControlValue) => {
    setError(null);
    sendCommand(d.id, control, value).catch((e) => setError(String(e)));
  };
  // A profile write takes a second or more. The pipe only says the command left, so wait until a snapshot taken
  // after it reached the core shows the monitor idle again.
  const [pending, setPending] = useState<{ control: "profile" | "save_profile"; at: number } | null>(null);
  useEffect(() => {
    if (pending && (d.error || (!d.busy && Date.now() - pending.at > TICK_S * 1500))) setPending(null);
  }, [d, pending]);
  const run = (control: "profile" | "save_profile", value: string) => {
    setError(null);
    setPending({ control, at: Date.now() });
    sendCommand(d.id, control, value).catch((e) => {
      setError(String(e));
      setPending(null);
    });
  };
  const busyButton = (control: "profile" | "save_profile", idle: string, busy: string, disabled = false) => (
    <button className="btn" disabled={pending != null || disabled} aria-busy={pending?.control === control} onClick={() => run(control, control === "profile" ? profile : name.trim())}>
      {pending?.control === control && <span className="spinner" aria-hidden />}
      {pending?.control === control ? busy : idle}
    </button>
  );
  const sliders = (list: Slide[]) =>
    list.map(({ id, label }) => {
      const v = num(d, id);
      return v != null && <Slider key={JSON.stringify(id)} label={t(label)} value={v} send={(n) => send(id, n)} />;
    });
  const preset = num(d, "color_preset");
  const turn = num(d, "orientation");
  const presetOptions = presets()
    .map((label, i) => ({ value: String(i + 1), label }))
    .filter((o) => !d.presets.length || d.presets.includes(Number(o.value)) || Number(o.value) === preset);
  const why = error ?? d.error;
  const read = d.controls.length > 0;
  const mode = text(d, "display_mode");
  const left = num(d, "keep_display_mode");
  const now = mode ? parseMode(mode) : undefined;
  const sizes = [...new Set(d.modes.map((m) => parseMode(m).size))];
  const rates = d.modes.filter((m) => parseMode(m).size === now?.size);
  return (
    <section className="mpanel" aria-label={d.name}>
      {!read && !d.error && <p className="muted">{t("Reading the monitor…")}</p>}
      {/* Dims this monitor three times, to tell two of one model apart. */}
      {num(d, "gpu_gamma") != null && (
        <button className="btn mident" onClick={() => send("identify", true)} disabled={d.busy}>
          {t("Identify")}
        </button>
      )}
      {read && (
        <div className="mgroups">
          <div className="mgroup">
            <h2>{t("Picture")}</h2>
            {now && d.modes.length > 0 && (
              <>
                <div className="mslider">
                  <span>{t("Resolution")}</span>
                  {/* The fastest rate the new size has, as Windows Settings picks. */}
                  <Dropdown label={t("Resolution")} value={now.size} onChange={(size) => send("display_mode", d.modes.find((m) => parseMode(m).size === size) ?? mode!)} options={sizes.map((size) => ({ value: size, label: sizeLabel(size) }))} />
                </div>
                <div className="mslider">
                  <span>{t("Refresh rate")}</span>
                  <Dropdown label={t("Refresh rate")} value={mode!} onChange={(m) => send("display_mode", m)} options={rates.map((m) => ({ value: m, label: `${parseMode(m).hz} Hz` }))} />
                </div>
              </>
            )}
            {turn != null && (
              <div className="mslider">
                <span>{t("Orientation")}</span>
                <Dropdown label={t("Orientation")} value={String(turn)} onChange={(v) => send("orientation", Number(v))} options={TURNS.map((label, i) => ({ value: String(i), label: t(label) }))} />
              </div>
            )}
            {sliders(PICTURE)}
            {sliders(MORE_PICTURE)}
          </div>
          <div className="mgroup">
            <h2>{t("Colour")}</h2>
            {preset != null && (
              <div className="mslider">
                <span>{t("Colour preset")}</span>
                {/* One preset listed: nothing to pick, so it is shown, not offered. */}
                {presetOptions.length > 1 ? <Dropdown label={t("Colour preset")} value={String(preset)} onChange={(v) => send("color_preset", Number(v))} options={presetOptions} /> : <span className="mfixed">{presetOptions[0]?.label ?? preset}</span>}
              </div>
            )}
            {sliders(GAINS)}
            {sliders(MORE_COLOUR)}
          </div>
          {num(d, "gpu_gamma") != null && (
            <div className="mgroup">
              <h2>{t("GPU colour")}</h2>
              <p className="muted">{t("Works on any monitor. Leave colour at default in the GPU control panel, or the two will undo each other.")}</p>
              {sliders(GPU)}
              <div className="mfield">
                <span />
                <button className="btn" disabled={GPU.every(({ id }) => num(d, id) === GPU_NEUTRAL[id as string])} onClick={() => send("gpu_reset", true)}>
                  {t("Reset")}
                </button>
              </div>
            </div>
          )}
          {d.profiles.length > 0 && (
            <div className="mgroup">
              <h2>{t("Profiles")}</h2>
              <div className="mfield">
                <Dropdown label={t("Profile")} value={profile} onChange={setProfile} options={d.profiles.map((p) => ({ value: p, label: p }))} />
                {busyButton("profile", t("Apply"), t("Applying…"))}
              </div>
              <div className="mfield">
                <input className="minput" aria-label={t("Profile name")} placeholder={t("Save current settings as")} maxLength={48} value={name} onChange={(e) => setName(e.target.value)} />
                {busyButton("save_profile", t("Save"), t("Saving…"), !name.trim())}
              </div>
            </div>
          )}
        </div>
      )}
      {why && <p className="problem" role="alert">{t("Could not change this: {why}", { why })}</p>}
      {/* The core reverts on its own when the countdown ends, even if this window is on the blank screen. */}
      <Alert open={left != null} onClose={() => send("keep_display_mode", false)} title={t("Keep these display settings?")} closeLabel={t("Revert")} confirm={{ label: t("Keep changes"), busy: false, onClick: () => send("keep_display_mode", true) }}>
        {t("The display goes back to the previous settings in {n} s.", { n: left ?? 0 })}
      </Alert>
    </section>
  );
}

/** Monitors over DDC/CI: one card per monitor, settings for the selected one. The core checks every value; this page only sends what the user picked. */
export function Displays({ devices }: { devices: Device[] }) {
  const monitors = devices.filter(isMonitor);
  const [sel, setSel] = useState<string | null>(null);
  const [scanning, setScanning] = useState(false);
  const current = monitors.find((d) => key(d) === sel) ?? monitors[0];
  const scan = () => {
    setScanning(true);
    sendCommand("core", "scan", true).finally(() => window.setTimeout(() => setScanning(false), 2000));
  };
  return (
    <div className="page">
      <div className="row">
        <div className="phead">
          <h1>{t("Displays")}</h1>
          <p className="muted">{t("Changes go straight to the monitor and stay there. SysVane never switches the input, turns a monitor off or resets it.")}</p>
        </div>
        <button className="btn" disabled={scanning} onClick={scan}>
          {scanning ? t("Scanning…") : t("Scan")}
        </button>
      </div>
      {current ? (
        <>
          <div className="mcards" role="group" aria-label={t("Displays")}>
            {monitors.map((d) => (
              <Card key={key(d)} d={d} on={d === current} pick={() => setSel(key(d))} />
            ))}
          </div>
          <Settings key={key(current)} d={current} />
        </>
      ) : (
        <p className="muted">{t("No monitor answers over DDC/CI. Turn on DDC/CI in the monitor's own menu, then press Scan.")}</p>
      )}
    </div>
  );
}
