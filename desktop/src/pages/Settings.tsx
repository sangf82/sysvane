import { useEffect, useState } from "react";
import { Dropdown } from "../components/Dropdown";
import { appVersion, sendCommand } from "../lib/host";
import type { Update } from "../lib/update";
import { Switch } from "../components/Switch";
import type { Snapshot } from "../wire";
import { t, useLang } from "../lib/i18n";
import { useTheme } from "../lib/theme";

/** What the user can set from the window. Startup, fans and other core-owned settings arrive with the command channel (M3). */
export function Settings({ update, recheck, devices }: { update: Update | null; recheck: () => Promise<Update | null>; devices: Snapshot["devices"] }) {
  const core = devices.find((d) => d.id === "core");
  const autostart = core?.controls.find((c) => c.id === "autostart")?.value === true;
  const fake = devices.find((d) => d.id === "fake");
  const level = Number(fake?.controls.find((c) => c.id === "level")?.value ?? 0);
  const [sendError, setSendError] = useState<string | null>(null);
  const [version, setVersion] = useState("");
  const [checked, setChecked] = useState<"idle" | "busy" | "none">("idle");
  const [theme, setTheme] = useTheme();
  const [lang, setLang] = useLang();
  useEffect(() => {
    appVersion().then(setVersion);
  }, []);
  return (
    <div className="page">
      <h1>{t("Settings")}</h1>
      <section className="section">
        <header>
          <h2>{t("About")}</h2>
        </header>
        <div className="row">
          <span>{t("Version")}</span>
          <span className="num">{version || "—"}</span>
        </div>
        <div className="row">
          <span>{update ? t("SysVane {v} is available.", { v: update.version }) : checked === "none" ? t("You have the latest version.") : t("Updates")}</span>
          <button
            className="theme"
            disabled={checked === "busy"}
            onClick={() => {
              setChecked("busy");
              recheck().then((u) => setChecked(u ? "idle" : "none"));
            }}
          >
            {checked === "busy" ? t("Checking…") : t("Check for updates")}
          </button>
        </div>
      </section>
      <section className="section">
        <header>
          <h2>{t("Appearance")}</h2>
        </header>
        <div className="row">
          <span>{t("Language")}</span>
          <Dropdown label={t("Language")} value={lang} onChange={setLang} options={[{ value: "auto", label: t("auto") }, { value: "en", label: t("English") }, { value: "vi", label: t("Vietnamese") }]} />
        </div>
        <div className="row">
          <span>{t("Theme")}</span>
          <Dropdown label={t("Theme")} value={theme} onChange={setTheme} options={[{ value: "auto", label: t("auto") }, { value: "light", label: t("light") }, { value: "dark", label: t("dark") }]} />
        </div>
      </section>
      <section className="section">
        <header>
          <h2>{t("Startup")}</h2>
        </header>
        <div className="row">
          <span>{t("Start with Windows")}</span>
          <Switch checked={autostart} label={t("Start with Windows")} disabled={!core} onChange={(on) => { setSendError(null); sendCommand("core", "autostart", on).catch((e) => setSendError(String(e))); }} />
        </div>
        {(sendError || core?.error) && <p className="problem" role="alert">{t("Could not change this: {why}", { why: sendError ?? core?.error ?? "" })}</p>}
      </section>
      {fake && (
        <section className="section">
          <header>
            <h2>{t("Test device")}</h2>
          </header>
          <div className="row">
            <span>{t("Level")}</span>
            <input type="range" min={0} max={100} value={level} aria-label={t("Level")} onChange={(e) => { setSendError(null); sendCommand("fake", "level", Number(e.target.value)).catch((err) => setSendError(String(err))); }} />
          </div>
          {fake.error && <p className="problem" role="alert">{t("Could not change this: {why}", { why: fake.error })}</p>}
        </section>
      )}
      <section className="section">
        <header>
          <h2>{t("Licences")}</h2>
        </header>
        <p className="muted">{t("CPU temperature uses PawnIO by namazso (driver GPL-2.0, modules LGPL-2.1), downloaded from its official release when SysVane is installed.")}</p>
      </section>
    </div>
  );
}
