import { getVersion } from "@tauri-apps/api/app";
import { isTauri } from "@tauri-apps/api/core";
import type { Update } from "@tauri-apps/plugin-updater";
import { useEffect, useState } from "react";
import { resolved, t, useLang } from "../i18n";
import { useTheme } from "../theme";

/** What the user can set from the window. Startup, fans and other core-owned settings arrive with the command channel (M3). */
export function Settings({ update, recheck }: { update: Update | null; recheck: () => Promise<Update | null> }) {
  const [version, setVersion] = useState("");
  const [checked, setChecked] = useState<"idle" | "busy" | "none">("idle");
  const [theme, cycleTheme] = useTheme();
  const [lang, cycleLang] = useLang();
  useEffect(() => {
    if (isTauri()) getVersion().then(setVersion);
  }, []);
  const langName = lang === "auto" ? t("auto") : lang === "en" ? t("English") : t("Vietnamese");
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
          <button className="theme" onClick={cycleLang} aria-label={t("Language: {mode}", { mode: langName })}>
            {langName} ({resolved().toUpperCase()})
          </button>
        </div>
        <div className="row">
          <span>{t("Theme")}</span>
          <button className="theme" onClick={cycleTheme} aria-label={t("Theme: {mode}", { mode: t(theme) })}>
            {t(theme)}
          </button>
        </div>
      </section>
      <section className="section">
        <header>
          <h2>{t("Background program")}</h2>
        </header>
        <p className="muted">{t("Start with Windows is in the tray menu, because the background program owns it.")}</p>
        <p className="muted">{t("Settings and logs are kept in the SysVane folder under ProgramData.")}</p>
      </section>
      <section className="section">
        <header>
          <h2>{t("Licences")}</h2>
        </header>
        <p className="muted">{t("CPU temperature uses PawnIO by namazso (driver GPL-2.0, modules LGPL-2.1), downloaded from its official release when SysVane is installed.")}</p>
      </section>
    </div>
  );
}
