import { type CSSProperties, useEffect, useRef, useState } from "react";
import { Alert } from "../components/Alert";
import { Dropdown } from "../components/Dropdown";
import { Glyph } from "../components/glyphs";
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

/** A white point as R/G/B gains (0–100) and a brightness push it, relative to the strongest channel. */
function white(gains: (number | undefined)[], brightness = 100): [number, number, number] | undefined {
  const [r, g, b] = gains;
  if (r == null || g == null || b == null) return undefined;
  const top = Math.max(r, g, b, 1);
  const light = 0.35 + 0.65 * (brightness / 100);
  return [r, g, b].map((v) => (v / top) * light) as [number, number, number];
}
const rgb = (w: [number, number, number] | undefined) => w && `rgb(${w.map((v) => Math.round(v * 255)).join(" ")})`;
/** The monitor's own white point (DDC gains and brightness), from values read back only. */
const monitorWhite = (d: Device) => white(GAINS.map(({ id }) => num(d, id)), num(d, "brightness"));
/** The graphics card's white point for this display (gamma-ramp gains and brightness, 50 = unchanged). */
const gpuWhite = (d: Device) => white(["gpu_red", "gpu_green", "gpu_blue"].map((id) => num(d, id as ControlId)), 50 + (num(d, "gpu_brightness") ?? 50));
/** What reaches the eye: the card's colour shown through the monitor's. */
function seen(d: Device) {
  const m = monitorWhite(d);
  const g = gpuWhite(d);
  return rgb(m && g ? (m.map((v, i) => v * g[i]) as [number, number, number]) : (m ?? g));
}

/** Holds the dragged or typed value until the device reports it, and sends only after the drag pauses: a DDC write takes tens of ms. */
function Slider({ label, value, send }: { label: string; value: number; send: (n: number) => void }) {
  const [pending, setPending] = useState<number | null>(null);
  const [draft, setDraft] = useState<string | null>(null);
  const timer = useRef(0);
  useEffect(() => {
    if (pending === value) setPending(null);
  }, [value, pending]);
  const change = (n: number, wait = 250) => {
    setPending(n);
    clearTimeout(timer.current);
    timer.current = window.setTimeout(() => {
      send(n);
      window.setTimeout(() => setPending((p) => (p === n ? null : p)), 3000);
    }, wait);
  };
  // A typed number goes out at once, rounded and held to 0–100; an empty or non-number entry is dropped.
  const commit = () => {
    if (draft == null) return;
    const n = Number(draft);
    setDraft(null);
    if (draft.trim() !== "" && Number.isFinite(n)) change(Math.min(100, Math.max(0, Math.round(n))), 0);
  };
  const shown = pending ?? value;
  return (
    <div className="mslider">
      <span>{label}</span>
      <input type="range" aria-label={label} min={0} max={100} value={shown} style={{ "--pct": `${shown}%` } as CSSProperties} onChange={(e) => change(Number(e.target.value))} />
      <input
        className="mnum"
        type="number"
        inputMode="numeric"
        aria-label={t("{name} value", { name: label })}
        min={0}
        max={100}
        value={draft ?? shown}
        onChange={(e) => setDraft(e.target.value)}
        onFocus={(e) => e.target.select()}
        onBlur={commit}
        onKeyDown={(e) => {
          if (e.key === "Enter") commit();
          if (e.key === "Escape") setDraft(null);
        }}
      />
    </div>
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
        <span style={{ background: seen(d) }} />
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
  const [source, setSource] = useState<"monitor" | "gpu" | null>(null);
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
  const ddcColour = preset != null || GAINS.some(({ id }) => num(d, id) != null);
  const gpuColour = num(d, "gpu_gamma") != null;
  const gpuChanged = gpuColour && GPU.some(({ id }) => num(d, id) !== GPU_NEUTRAL[id as string]);
  // Both stay in force; the switch only picks which one is shown. The card's own when nothing was picked.
  const shownSource = source ?? (!ddcColour || gpuChanged ? "gpu" : "monitor");
  return (
    <section className="mgroups" aria-label={d.name}>
      {!read && !d.error && <p className="muted">{t("Reading the monitor…")}</p>}
      {read && d.profiles.length > 0 && (
        <div className="mpanel">
          <div className="sechead">
            <Glyph name="bookmark" size={16} />
            <h2>{t("Profiles")}</h2>
            <span className="muted">{t("Monitor settings only")}</span>
          </div>
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
      {read && now && d.modes.length > 0 && (
        <div className="mpanel">
          <div className="sechead">
            <Glyph name="monitor" size={16} />
            <h2>{t("Display")}</h2>
            <span className="muted">{t("Set by Windows")}</span>
            {/* Dims this monitor three times, to tell two of one model apart. */}
            {gpuColour && (
              <button className="btn" onClick={() => send("identify", true)} disabled={d.busy}>
                <Glyph name="scan" />
                {t("Identify")}
              </button>
            )}
          </div>
          <div className="mslider">
            <span>{t("Resolution")}</span>
            {/* The fastest rate the new size has, as Windows Settings picks. */}
            <Dropdown label={t("Resolution")} value={now.size} onChange={(size) => send("display_mode", d.modes.find((m) => parseMode(m).size === size) ?? mode!)} options={sizes.map((size) => ({ value: size, label: sizeLabel(size) }))} />
          </div>
          <div className="mslider">
            <span>{t("Refresh rate")}</span>
            <Dropdown label={t("Refresh rate")} value={mode!} onChange={(m) => send("display_mode", m)} options={rates.map((m) => ({ value: m, label: `${parseMode(m).hz} Hz` }))} />
          </div>
          {turn != null && (
            <div className="mslider">
              <span>{t("Orientation")}</span>
              <Dropdown label={t("Orientation")} value={String(turn)} onChange={(v) => send("orientation", Number(v))} options={TURNS.map((label, i) => ({ value: String(i), label: t(label) }))} />
            </div>
          )}
        </div>
      )}
      {read && num(d, "brightness") != null && (
        <div className="mpanel">
          <div className="sechead">
            <Glyph name="sun" size={16} />
            <h2>{t("Picture")}</h2>
            <span className="muted">{t("Set in the monitor over DDC/CI")}</span>
          </div>
          {sliders(PICTURE)}
          {sliders(MORE_PICTURE)}
        </div>
      )}
      {read && (ddcColour || gpuColour) && (
        <div className="mpanel">
          <div className="sechead">
            <Glyph name="palette" size={16} />
            <h2>{t("Colour")}</h2>
            {ddcColour && gpuColour && (
              <div className="seg" role="group" aria-label={t("Colour from")}>
                {(["monitor", "gpu"] as const).map((s) => (
                  <button key={s} className={shownSource === s ? "on" : ""} aria-pressed={shownSource === s} onClick={() => setSource(s)}>
                    {s === "monitor" ? t("Monitor (DDC)") : t("Graphics card")}
                  </button>
                ))}
              </div>
            )}
            {shownSource === "gpu" && (
              <button className="btn" disabled={!gpuChanged} onClick={() => send("gpu_reset", true)}>
                <Glyph name="undo" />
                {t("Reset")}
              </button>
            )}
          </div>
          {shownSource === "monitor" ? (
            <>
              <p className="muted">{t("Kept in the monitor over DDC/CI; profiles save these")}</p>
              {preset != null && (
                <div className="mslider">
                  <span>{t("Colour preset")}</span>
                  {/* One preset listed: nothing to pick, so it is shown, not offered. */}
                  {presetOptions.length > 1 ? <Dropdown label={t("Colour preset")} value={String(preset)} onChange={(v) => send("color_preset", Number(v))} options={presetOptions} /> : <span className="mfixed">{presetOptions[0]?.label ?? preset}</span>}
                </div>
              )}
              {sliders(GAINS)}
              {sliders(MORE_COLOUR)}
              {gpuChanged && <p className="muted">{t("Graphics card colour is also changed and shows on top of this")}</p>}
            </>
          ) : (
            <>
              <p className="muted">{t("Applied by the graphics card on top of the monitor's own colour; works on any monitor. Leave colour at default in the GPU control panel, or the two will undo each other.")}</p>
              {sliders(GPU)}
            </>
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
