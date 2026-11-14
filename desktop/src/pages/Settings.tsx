import { getVersion } from "@tauri-apps/api/app";
import { isTauri } from "@tauri-apps/api/core";
import type { Update } from "@tauri-apps/plugin-updater";
import { useEffect, useState } from "react";
import { Dropdown } from "../Dropdown";
import { t, useLang } from "../i18n";
import { useTheme } from "../theme";

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
          <Dropdown label={t("Language")} value={lang} onChange={setLang} options={[{ value: "auto", label: t("auto") }, { value: "en", label: t("English") }, { value: "vi", label: t("Vietnamese") }]} />
        </div>
        <div className="row">
          <span>{t("Theme")}</span>
          <Dropdown label={t("Theme")} value={theme} onChange={setTheme} options={[{ value: "auto", label: t("auto") }, { value: "light", label: t("light") }, { value: "dark", label: t("dark") }]} />
        </div>
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
