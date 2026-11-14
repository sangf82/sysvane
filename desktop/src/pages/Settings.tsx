import { getVersion } from "@tauri-apps/api/app";
import { isTauri } from "@tauri-apps/api/core";
import type { Update } from "@tauri-apps/plugin-updater";
import { useEffect, useState } from "react";
import { type Lang, t, useLang } from "../i18n";
import { type Theme, useTheme } from "../theme";

/** What the user can set from the window. Startup, fans and other core-owned settings arrive with the command channel (M3). */
export function Settings({ update, recheck }: { update: Update | null; recheck: () => Promise<Update | null> }) {
  const [version, setVersion] = useState("");
  const [checked, setChecked] = useState<"idle" | "busy" | "none">("idle");
  const [theme, setTheme] = useTheme();
  const [lang, setLang] = useLang();
  useEffect(() => {
    if (isTauri()) getVersion().then(setVersion);
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
          <select className="select" value={lang} onChange={(e) => setLang(e.target.value as Lang)}>
            <option value="auto">{t("auto")}</option>
            <option value="en">{t("English")}</option>
            <option value="vi">{t("Vietnamese")}</option>
          </select>
        </div>
        <div className="row">
          <span>{t("Theme")}</span>
          <select className="select" value={theme} onChange={(e) => setTheme(e.target.value as Theme)}>
            <option value="auto">{t("auto")}</option>
            <option value="light">{t("light")}</option>
            <option value="dark">{t("dark")}</option>
          </select>
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
