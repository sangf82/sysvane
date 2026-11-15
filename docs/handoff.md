# Handoff — file tree refactor

Updated 2026-11-15. Read this before starting; update or remove a step when it ends. Target tree and rules: `docs/architecture.md`.

## Why
Each PC has a different mainboard and peripherals. Fan, RGB and peripheral logic depends on the vendor and model, so the tree must let a new device be added as one file plus a data profile before the code grows. Sensors are generic. Regression risk accepted by the user.

## Done
| Commit | What |
|---|---|
| `518d473` | WIP baseline (module host, config, power, fake device, shorter `AGENTS.md`) |
| `5408406` | `core/src` grouped into `sensors/`, `ipc/` (+ `access.rs`), `system/`, `modules/` |
| `87e05d5` | `wire/src/lib.rs` split into `consts.rs`, `command.rs`, `snapshot.rs` (re-exported) |
| `06840d3` | `desktop/src` into `components/` and `lib/`; `temp()`/`watts()` helpers in `format.ts` |
| `f1b3904` | Tauri shell split: `main.rs`, `lib.rs`, `core_proc.rs`, `relay.rs`, `window.rs`; `CORE_START` is now managed state |
| (next commit) | Root tree: `core/pawnio/` → `core/assets/pawnio/`; `modules/ddc/` → `docs/devices/ddc/`; `docs/architecture.md`; this file moved to `docs/`; `AGENTS.md`/`README.md` Layout updated; DDC CLI section marked historical; `Settings.tsx` Tauri calls moved behind `lib/host.ts` / `lib/update.ts`; trail learning for the tree |

Checked after `f1b3904`: `cargo fmt --check`, clippy `-D warnings` (debug and release), `cargo test -p sysvane-core -p sysvane-wire` (29 + 3), `cd desktop; npm run build`. After the root move: fmt, clippy `-D warnings` (debug and release), tests (29 + 3), `npm run build`.

## Not done
1. **Run the app** — dashboard checked 2026-11-15 with `tauri dev` + `agent-desktop`: Overview shows live core data, Settings shows the version through `appVersion()`. Still checked by compiling only: flyout, core-start alert, elevated restart. Also run `tools\probe-temp.cmd` elevated once (PawnIO path changed).
2. **Review passes**: `simplify` and `/code-review` on the refactor commits.

`trail check` (2026-11-15): two hard flags, both false. `AGENTS.md:71` describes the Stop-time lint hook, not a per-edit clippy hook. `desktop/package-lock.json` is the known Tailwind false match. The other notes match code that implements the learnings themselves.

## Decisions
- Sensors (cputemp, gpu, perf, collector, latest) are common: `core/src/sensors/`. No sensor trait until a second source exists.
- Device modules go kind first, then vendor/model: `modules/<kind>/<vendor>.rs` (fan, rgb, lcd, mouse, keyboard, monitor, headset). Shared vendor protocol lives in `vendors/<vendor>/` with no `Module` impl, so an AIO's pump, fan, LCD and RGB share it.
- Each module exposes `detect()`; `modules/mod.rs` holds a static list. No dynamic plugins, no per-module cargo features. Board variants are data in `profiles/`.
- No empty folders until the first real module of a kind exists (learning 2026-11-13).
- `DESIGN.md` and `PRODUCT.md` stay at root: the impeccable skill and the design.md lint look for them there. `skills-lock.json` stays at root for the skills CLI.
- `desktop/` keeps the Tauri shell and React together until a second shell (web) exists; then split into `ui/`, `desktop/`, `web/`.
- Third-party tools (OpenRGB, liquidctl, LibreHardwareMonitor): native Rust by default; wrap one only as an optional backend in its own process on a bus the core does not touch; no GPL code linked into core. License, RAM and Windows support are unverified (from memory).
- Bus/transport abstraction waits for a second vendor on the same bus.

## Watch out
- `#[cfg_attr(not(debug_assertions), allow(dead_code))]` now sits on `mod modules;` in `core/src/main.rs`, so it also covers `devices.rs` (it did not before). Narrow it if that hides real dead code.
- `desktop/src/wire.ts` stays at that path: two tests in `wire/src/lib.rs` read it by path.
- Run clippy in release as well as debug (learning 2026-11-14).
- `Overview` used a missing i18n key `clock unavailable`; now `clock n/a` (the one visible change).
- `trail check` flags `desktop/package-lock.json` (version 2→4) against a Tailwind rule: false conflict, say so in the commit message.
- Most `core/` file bodies were not read; the duplicate-code claims came from structure only.
