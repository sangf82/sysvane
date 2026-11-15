// Everything the UI says to, or hears from, the Tauri shell (desktop/src-tauri/src/main.rs).
// Outside Tauri (plain `vite` in a browser) there is no bridge: commands answer a default and no events arrive.

import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ControlId, ControlValue, DeviceId, Snapshot } from "./wire";

const call = <T>(command: string, fallback: T): Promise<T> => (isTauri() ? invoke<T>(command) : Promise.resolve(fallback));

export const isFlyout = () => call("is_flyout", false);
export const openDashboard = () => call("open_dashboard", undefined);
export type CoreStart = "ok" | "unelevated" | "failed";
export const coreStartStatus = () => call<CoreStart>("core_start_status", "ok");
export const restartCoreElevated = () => call<CoreStart>("restart_core_elevated", "ok");
/** The core answers in its next snapshot (`devices`), so there is nothing to wait for. */
export const sendCommand = (device: DeviceId, control: ControlId, value: ControlValue) => (isTauri() ? invoke<void>("send_command", { device, control, value }) : Promise.resolve());
export const startCore = () => call<CoreStart>("start_core", "ok");

export function onSnapshot(f: (s: Snapshot) => void) {
  if (isTauri()) listen<string>("snapshot", (e) => f(JSON.parse(e.payload) as Snapshot));
}

export function onCoreStatus(f: (connected: boolean) => void) {
  if (isTauri()) listen<boolean>("core-status", (e) => f(e.payload));
}
