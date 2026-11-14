import { AnimateIcon, IconPathStyle } from "@animate-ui/react/icons";
import { spring, transition } from "@animate-ui/react/utils/motion";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useState, useSyncExternalStore } from "react";
import { Flyout } from "./Flyout";
import { Alert } from "./Alert";
import { type CoreStart, coreStartStatus, isFlyout, startCore } from "./host";
import { type Msg, resolved, t, useLang } from "./i18n";
import { RES } from "./metrics";
import { Overview } from "./pages/Overview";
import { Performance } from "./pages/Performance";
import { Processes } from "./pages/Processes";
import { Settings } from "./pages/Settings";
import { getState, subscribe } from "./store";
import { install, useUpdate } from "./update";
import { useTheme } from "./theme";
import { PROTOCOL } from "./wire";
import { Icon, Logo, type IconName } from "./ui";
import "./styles.css";

type Page = "overview" | "performance" | "processes" | "settings";

const NAV: { page: Page; label: Msg; icon: IconName }[] = [
  { page: "overview", label: "Overview", icon: "overview" },
  { page: "performance", label: "Performance", icon: "performance" },
  { page: "processes", label: "Processes", icon: "processes" },
];

function Waiting({ connected, onStart }: { connected: boolean; onStart: (r: CoreStart) => void }) {
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
          startCore().then(onStart);
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
  useTheme();
  const [lang] = useLang();
  const [flyout, setFlyout] = useState(false);
  const [page, setPage] = useState<Page>("overview");
  const [resource, setResource] = useState(RES.cpu);
  const [update, recheck] = useUpdate();
  const [installing, setInstalling] = useState(false);
  const [notesOpen, setNotesOpen] = useState(false);
  const [coreProblem, setCoreProblem] = useState<CoreStart>("ok");
  const [problemSeen, setProblemSeen] = useState(false);
  const restarted = (r: CoreStart) => {
    setCoreProblem(r);
    setProblemSeen(false);
  };
  useEffect(() => {
    isFlyout().then(setFlyout);
    // The first start happens in the shell before this window listens, so ask for its result.
    coreStartStatus().then(setCoreProblem);
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
  if (flyout) return <main className="flyout-main">{st.snap ? <Flyout s={st.snap} h={st.hist} /> : <Waiting connected={st.connected} onStart={restarted} />}</main>;
  const s = st.snap;
  const open = (id: string) => {
    setResource(id);
    setPage("performance");
  };
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
        <AnimateIcon animateOnHover asChild>
          <button className={`nav settings ${page === "settings" ? "on" : ""}`} aria-current={page === "settings" ? "page" : undefined} aria-label={t("Settings")} title={t("Settings")} onClick={() => setPage("settings")}>
            {page === "settings" && <motion.span layoutId="nav-glide" className="nav-glide" transition={spring.highlight} aria-hidden />}
            <Icon name="settings" size={18} />
            <span className="label">{t("Settings")}</span>
          </button>
        </AnimateIcon>
      </aside>
      <main className="content">
        <div className="corner">
          {!st.connected && <span className="alert-chip" role="alert">{t("Core disconnected")}</span>}
          {update && (
            <button className="update-chip" onClick={() => setNotesOpen(true)}>
              {t("Update {v}", { v: update.version })}
            </button>
          )}
        </div>
        {update && (
          <Alert
            open={notesOpen}
            onClose={() => setNotesOpen(false)}
            title={t("SysVane {v} is available.", { v: update.version })}
            confirm={{ label: installing ? t("Installing…") : t("Install and restart"), busy: installing, onClick: () => { setInstalling(true); install(update).catch(() => setInstalling(false)); } }}
          >
            <p className="notes">{(update.body ?? "").split("**").join("") || t("No changelog.")}</p>
          </Alert>
        )}
        <Alert open={coreProblem !== "ok" && !problemSeen} onClose={() => setProblemSeen(true)} title={coreProblem === "failed" ? t("The background program did not start") : t("Administrator rights are missing")}>
          <p>{coreProblem === "failed" ? t("sysvane-core.exe could not be started. Reinstall SysVane, or check that the file is next to sysvane-ui.exe.") : t("The background program started without administrator rights, so CPU temperature and other hardware readings stay off. Reinstall SysVane to register its scheduled task, which starts the program with the rights it needs.")}</p>
        </Alert>
        {!s ? (
          <Waiting connected={st.connected} onStart={restarted} />
        ) : (
          <>
            <div className={st.connected ? "" : "stale"}>
              <AnimatePresence mode="wait" initial={false}>
                <motion.div key={page} initial={{ opacity: 0 }} animate={{ opacity: 1, transition: transition.panelIn }} exit={{ opacity: 0, transition: transition.panelOut }}>
                  {page === "overview" ? <Overview s={s} h={st.hist} open={open} /> : page === "performance" ? <Performance s={s} h={st.hist} sel={resource} setSel={setResource} /> : page === "processes" ? <Processes list={s.processes} /> : <Settings update={update} recheck={recheck} />}
                </motion.div>
              </AnimatePresence>
            </div>
          </>
        )}
      </main>
    </div>
  );
}
