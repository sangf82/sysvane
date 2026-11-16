# Architecture and file tree

Decided 2026-11-15. Marked **[later]** = created only when real code for it exists (no empty folders).

## Processes

- `sysvane-core.exe` (`core/`) — the only background process. Sensors, tray, pipes and every device module (fan, RGB, LCD, monitor…) run here. Target: elevated from a scheduled task (M1).
- `sysvane-ui.exe` (`desktop/`) — started on demand, exits when closed, never touches hardware. Talks to the core over named pipes only.
- `wire/` — the contract between the two: pipe and process names, commands, snapshot types. Both Rust sides depend on it; `desktop/src/wire.ts` mirrors it and a test fails on drift. A future web UI would use the same contract.

## Tree

```
sysvane/
├─ AGENTS.md  README.md  Cargo.toml  rust-toolchain.toml  skills-lock.json
├─ DESIGN.md  PRODUCT.md        root on purpose: the impeccable skill and the design.md lint expect them here
│
├─ core/                         sysvane-core
│  ├─ assets/                    tray-32.rgba, pawnio/ (driver module + COPYING)
│  └─ src/
│     ├─ sensors/                generic monitoring: cputemp, gpu, perf, collector, latest
│     ├─ modules/                device modules, compiled in
│     │  ├─ mod.rs               Module trait, host, static detect() list
│     │  ├─ config.rs  devices.rs  fake.rs
│     │  ├─ monitor/             M4: mod (Module), vcp (allowlist + guarded write), profile, ddc (dxva2), profiles/*.json
│     │  ├─ fan/ rgb/ lcd/ mouse/ keyboard/ headset/   [later] <vendor>.rs each
│     │  └─ vendors/<vendor>/    [later] protocol shared across kinds (e.g. an AIO's pump, fan, LCD, RGB); no Module impl
│     ├─ ipc/                    pipe, commands, access, hub
│     └─ system/                 tray, install, power, mode, worker, log, win
│
├─ wire/src/                     consts, command, snapshot
│
├─ desktop/                      sysvane-ui
│  ├─ src-tauri/src/             lib, core_proc (starts the core), relay (pipe → UI), window
│  └─ src/
│     ├─ pages/                  Overview, Performance, Processes, Settings
│     ├─ components/             Alert, Dropdown, Flyout, Switch, charts, layout, meters
│     ├─ lib/                    host.ts + update.ts (the only places that call Tauri), i18n, format, metrics, series, store, theme
│     └─ wire.ts                 TS mirror of wire/
│
├─ branding/logo/                logo kit and the scripts that build it and the Tauri icons
│
├─ docs/
│  ├─ architecture.md            this file
│  ├─ handoff.md                 current work in progress; read before starting, update when a step ends
│  ├─ plan.md                    plan and status
│  ├─ devices/ddc/               calibration, VCP notes, profiles/ (move next to the DDC code at M4)
│  └─ research/                  dated, historical notes
│
├─ tools/                        read-only hardware probes, usable without the core
├─ scripts/                      release helpers (bump-version.ps1)
└─ .github/  .claude/  .trail/
```

## Rules

- A new device = one `.rs` file under `core/src/modules/<kind>/` + one data profile. The UI draws one panel per kind from the module's descriptor, so it does not change.
- Sensors stay generic; no sensor trait until a second source exists. Bus/transport abstraction waits for a second vendor on the same bus.
- No dynamic plugins, no per-module cargo features. Board variants are data, never code.
- React code calls Tauri only through `desktop/src/lib/host.ts` (and `lib/update.ts` for the updater plugin). That keeps a later split into a shared `ui/` + `desktop/` shell + `web/` shell to a folder move.
- Do not split `desktop/` into Tauri shell and React until a second shell exists: Tauri expects them together, and the move touches `tauri.conf.json`, CI, `bump-version.ps1`, the lint hook and the `wire.ts` path test.
