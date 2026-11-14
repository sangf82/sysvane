import { AnimateIcon, IconPathStyle } from "@animate-ui/react/icons";
import { spring, transition } from "@animate-ui/react/utils/motion";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useRef, useState, useSyncExternalStore } from "react";
import { Flyout } from "./Flyout";
import { isFlyout, startCore } from "./host";
import { type Msg, resolved, t, useLang } from "./i18n";
import { RES } from "./metrics";
import { Overview } from "./pages/Overview";
import { Performance } from "./pages/Performance";
import { Processes } from "./pages/Processes";
import { getState, subscribe } from "./store";
import { install, useUpdate } from "./update";
import { useTheme } from "./theme";
import { PROTOCOL } from "./wire";
import { Icon, Logo, type IconName } from "./ui";
import "./styles.css";

type Page = "overview" | "performance" | "processes";

const NAV: { page: Page; label: Msg; icon: IconName }[] = [
  { page: "overview", label: "Overview", icon: "overview" },
  { page: "performance", label: "Performance", icon: "performance" },
  { page: "processes", label: "Processes", icon: "processes" },
];

function Waiting({ connected }: { connected: boolean }) {
  const [tried, setTried] = useState(false);
  if (connected) return <p className="empty">{t("Reading the first sensor values…")}</p>;
  return (
    <div className="empty">
      <h1>{t("sysvane-core is not running")}</h1>
      <p>{t("SysVane reads sensors in a separate background program so this window stays light. It normally starts by itself and shows in the tray.")}</p>
      <button
        className="primary"
        onClick={() => {
          setTried(true);
          startCore();
        }}
      >
        {t("Start sysvane-core")}
      </button>
      {tried && <p className="muted">{t("Started. This window fills in within a few seconds. If nothing changes, sysvane-core.exe is missing from the folder next to sysvane-ui.exe.")}</p>}
    </div>
  );
}

export default function App() {
  const st = useSyncExternalStore(subscribe, getState);
  const [theme, cycleTheme] = useTheme();
  const [lang, cycleLang] = useLang();
  const [flyout, setFlyout] = useState(false);
  const [page, setPage] = useState<Page>("overview");
  const [resource, setResource] = useState(RES.cpu);
  const update = useUpdate();
  const [installing, setInstalling] = useState(false);
  const notes = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    isFlyout().then(setFlyout);
  }, []);
  useEffect(() => {
    document.documentElement.lang = resolved();
  }, [lang]);
  if (st.snap && st.snap.protocol !== PROTOCOL)
    return (
      <main className="empty">
        <h1>{t("SysVane needs an update")}</h1>
        <p>{t("The window and the background program are different versions, so the window cannot talk to it safely. Update SysVane, then open it again.")}</p>
      </main>
    );
  if (flyout) return <main className="flyout-main">{st.snap ? <Flyout s={st.snap} h={st.hist} /> : <Waiting connected={st.connected} />}</main>;
  const s = st.snap;
  const open = (id: string) => {
    setResource(id);
    setPage("performance");
  };
  const themeLabel = t("Theme: {mode}", { mode: t(theme) });
  const langLabel = t("Language: {mode}", { mode: lang === "auto" ? t("auto") : lang === "en" ? t("English") : t("Vietnamese") });
  return (
    <div className="shell">
      <IconPathStyle />
      <aside className="sidebar">
        <div className="brand">
          <Logo />
          <span className="label">SysVane</span>
        </div>
        <nav aria-label={t("Pages")}>
          {NAV.map((n) => (
            <AnimateIcon key={n.page} animateOnHover asChild>
              <button className={`nav ${page === n.page ? "on" : ""}`} aria-current={page === n.page ? "page" : undefined} aria-label={t(n.label)} title={t(n.label)} onClick={() => setPage(n.page)}>
                {page === n.page && <motion.span layoutId="nav-glide" className="nav-glide" transition={spring.highlight} aria-hidden />}
                <Icon name={n.icon} size={18} />
                <span className="label">{t(n.label)}</span>
              </button>
            </AnimateIcon>
          ))}
        </nav>
        <div className="side-foot">
          <span className="status" role="status">
            <i className={`dot ${st.connected ? "on" : ""}`} />
            <span className="label">{st.connected ? t("Core connected") : t("Core disconnected")}</span>
          </span>
          <button className="theme" onClick={cycleLang} aria-label={langLabel} title={langLabel}>
            <span className="label">{langLabel}</span>
            <span className="short" aria-hidden>
              {resolved().toUpperCase()}
            </span>
          </button>
          <button className="theme" onClick={cycleTheme} aria-label={themeLabel} title={t("Theme: auto follows Windows")}>
            <span className="label">{themeLabel}</span>
            <span className="short" aria-hidden>
              {theme[0].toUpperCase()}
            </span>
          </button>
        </div>
      </aside>
      <main className="content">
        {update && (
          <>
            <button className="update-chip" onClick={() => notes.current?.showModal()}>
              {t("Update {v}", { v: update.version })}
            </button>
            <dialog ref={notes} className="update-dialog">
              <h2>{t("SysVane {v} is available.", { v: update.version })}</h2>
              <p className="notes">{(update.body ?? "").split("**").join("") || t("No changelog.")}</p>
              <div className="row">
                <button onClick={() => notes.current?.close()} disabled={installing}>{t("Later")}</button>
                <button className="primary" disabled={installing} onClick={() => { setInstalling(true); install(update).catch(() => setInstalling(false)); }}>
                  {installing ? t("Installing…") : t("Install and restart")}
                </button>
              </div>
            </dialog>
          </>
        )}
        {!s ? (
          <Waiting connected={st.connected} />
        ) : (
          <>
            <AnimatePresence initial={false}>
              {!st.connected && (
                <motion.div key="banner" className="banner" role="alert" initial={{ opacity: 0, y: -8 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -8 }} transition={transition.reveal}>
                  {t("The core stopped answering. Showing the last reading; reconnecting.")}
                </motion.div>
              )}
            </AnimatePresence>
            <div className={st.connected ? "" : "stale"}>
              <AnimatePresence mode="wait" initial={false}>
                <motion.div key={page} initial={{ opacity: 0 }} animate={{ opacity: 1, transition: transition.panelIn }} exit={{ opacity: 0, transition: transition.panelOut }}>
                  {page === "overview" ? <Overview s={s} h={st.hist} open={open} /> : page === "performance" ? <Performance s={s} h={st.hist} sel={resource} setSel={setResource} /> : <Processes list={s.processes} />}
                </motion.div>
              </AnimatePresence>
            </div>
          </>
        )}
      </main>
    </div>
  );
}
