// M2: look for a signed update on GitHub, show it, install only after the user agrees.
import { isTauri } from "@tauri-apps/api/core";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { useEffect, useState } from "react";

const DAY = 24 * 60 * 60 * 1000;

/** Checks at start and once a day while the window stays open. A failed check (offline, no release) is silent. */
export function useUpdate() {
  const [update, setUpdate] = useState<Update | null>(null);
  const recheck = (): Promise<Update | null> => (isTauri() ? check().then((u) => (setUpdate(u), u), () => null) : Promise.resolve(null));
  useEffect(() => {
    if (!isTauri()) return;
    recheck();
    const id = setInterval(recheck, DAY);
    return () => clearInterval(id);
  }, []);
  return [update, recheck] as const;
}

/** Downloads, verifies the signature, then runs the installer, which stops the core and starts it again. */
export const install = (u: Update) => u.downloadAndInstall();
