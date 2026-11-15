import { onCoreStatus, onSnapshot } from "./host";
import { type History, record } from "./series";
import type { Snapshot } from "./wire";

export type State = { connected: boolean; snap: Snapshot | null; hist: History };

let state: State = { connected: false, snap: null, hist: {} };
const subs = new Set<() => void>();
const set = (next: State) => {
  state = next;
  subs.forEach((f) => f());
};

export const subscribe = (f: () => void) => {
  subs.add(f);
  return () => subs.delete(f);
};
export const getState = () => state;

onSnapshot((snap) => set({ connected: true, snap, hist: record(state.hist, snap) }));
// On disconnect the last reading stays, so the pages can show it as stale instead of going blank.
onCoreStatus((connected) => set({ ...state, connected }));
