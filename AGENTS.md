# SysVane

Very low-RAM, tray-first Windows app replacing ASUS Armoury Crate and vendor device suites: hardware monitoring, displays, peripherals, RGB, fans and the AIO LCD. All Rust except the React UI. Plan and status: `docs/plan.md`.

## Commands

- `cargo check -p sysvane-core` / `cargo clippy -p sysvane-core` — type-check / lint the core.
- `cargo test -p sysvane-core -p sysvane-wire` — keep green.
- `cd desktop; npm run build` — type-check and build the UI.
- `cd desktop; npm run tauri dev` — run the UI.

## Layout

- `core/` — `sysvane-core`, the headless background program. `src/sensors/` (generic monitoring), `src/ipc/` (pipes, commands, access), `src/system/` (tray, install, power, workers, Win32 helpers), `src/modules/` (device module host, config, fake device). `assets/`: tray icon, `pawnio/` driver module.
- Device modules: `core/src/modules/<kind>/<vendor>.rs` (kind: fan, rgb, lcd, mouse, keyboard, monitor, headset), each with `detect()`, listed statically in `modules/mod.rs`. A vendor protocol shared across kinds goes in `vendors/<vendor>/` with no `Module` impl. Board variants are data profiles. No folder until its first real module.
- `desktop/` — `sysvane-ui`, Tauri 2 + React/Vite. Started on demand, exits when closed, never touches hardware. `src/pages/`, `src/components/`, `src/lib/`; Tauri shell in `src-tauri/src/` (`core_proc`, `relay`, `window`).
- `wire/` — `sysvane-wire`, all the core and UI share: pipe name, process names, snapshot types (`consts`, `command`, `snapshot`). `desktop/src/wire.ts` mirrors them; a test reads it by path and fails if field names differ.
- `docs/` — `architecture.md` (tree and rules), `handoff.md` (work in progress: read it first, update it when a step ends), `plan.md`, `devices/ddc/` (calibration, VCP notes, profiles for the future Rust DDC module).
- `DESIGN.md` — UI design system.
- `branding/logo/` — Crate logo kit (`kit/`, rules in `kit/GUIDELINES.md`) and `scripts/` that regenerate it and the Tauri icons. Tray icon: `core/assets/tray-32.rgba`.
- `tools/` — read-only hardware probes, usable without the core.
- `scripts/` — release helpers (`bump-version.ps1`).
- `docs/research/` — dated, historical notes (old project name).
- Commit `30b7852` holds the removed Python implementation to port from.

## Architecture rules

- Keep Tauri, WebView and React out of `sysvane-core`. Background RAM is the main goal.
- Modules are compiled in. No DLL or WASM plugin loading.
- Code that needs a kernel driver or can hang runs in a supervised worker process.
- One owner per hardware bus. A third-party tool may run only on a bus the core does not touch, through a fixed command line.
- Each module owns its device's safety rules, enforced in the core, not the UI.
- The UI starts `sysvane-core.exe` from its own folder: build both.
- Keep hardware I/O off the UI and any thread with a deadline.
- Target privilege model (`docs/plan.md`, M1): core runs as administrator from a scheduled task under `Program Files`; UI runs as a normal user. Until M1 the core is unelevated and only the temperature helper is elevated. Never trust a command or profile from the UI or disk: validate it in the core.
- Each module keeps its own config, default profiles and fake device. No merged config. Profiles are data, never code.

## Monitor safety (hard rules)

A wrong DDC write blacks out the display, and DDC/CI has no undo.

- Never write factory-reset codes `0x04`, `0x05`, `0x06`, `0x08`.
- Never write power codes `0xD6`/`0xC6` with an off or standby value.
- Never write input source `0x60` unless the user explicitly asks and accepts losing video.
- All DDC writes go through one guarded function. No flag unlocks reset or power-off codes.
- Profile apply skips power and input by default.
- Do not guess a current value before it is read.

## LCD safety

- Never send OTA or firmware-upgrade commands to the panel (command types 4 and 5).
- Stop if the panel reports PM 32 or 50 until the Logo/RGB565 command conflict is resolved.

## Calibration notes

- CVTE scalers (Acer AL258Q) lock the RGB gain DAC while a factory color preset is active. Write `0x14 = 11` (User) before `0x16`/`0x18`/`0x1A`.
- Left panel: `0x14=5`, R50 G50 B50. Right panel: `0x14=11`, R47 G49 B51. Both: brightness 70, contrast 50.
- Details: `docs/devices/ddc/CALIBRATION.md`.

## Testing the UI

Use `agent-desktop` (Windows UI Automation). Before first use in a session run `agent-desktop skills get desktop` and `agent-desktop skills get platform`, and follow them.

- Operate only SysVane: `agent-desktop snapshot --app sysvane-ui.exe -i`. Never another app or the whole desktop (private content).
- Quote refs in PowerShell: `'@s8f3k2p9:e8'`.
- Stay headless: `set-value`, not `type`.
- Cold start fills the Chromium tree slowly: `snapshot --timeout-ms 30000`.

## Hooks, agents, tools

- `.claude/hooks/guard-monitor.ps1` runs before every Bash/PowerShell call: blocks unsafe VCP writes, keeps `agent-desktop` on sysvane. `SYSVANE_ALLOW_UNSAFE=1` bypasses it, only when the user explicitly accepts losing the display. Keep its patterns in step with the Rust DDC command line once that exists.
- `.claude/hooks/format-rust.ps1` runs `rustfmt` after each `.rs` edit.
- `.claude/hooks/lint-on-stop.ps1` at Stop: fmt + clippy if Rust changed, `tsc --noEmit` if `desktop/src` changed; failure blocks the stop once.
- `rust-core` agent (`.claude/agents/`): `core/`, `wire/`, `modules/`; runs fmt, clippy `-D warnings` and tests before reporting.
- `frontend-ui` agent: `desktop/`, `DESIGN.md`, `branding/`; runs `npm run build`.
- Trail: `trail ask "<task>"` before grepping; run `trail check` before commit, push or PR; learnings live in `.trail/learnings/`.
- For Claude Code questions, use the `claude-code-guide` MCP first.

## Release

- CI: `.github/workflows/ci.yml` on push to `main` and PRs.
- Release: `powershell -NoProfile -File scripts\bump-version.ps1 X.Y.Z`, commit, `git tag vX.Y.Z`, `git push --tags`. `release.yml` checks the tag equals the Cargo and package versions, builds the NSIS installer, publishes a pre-release.
- The installer is signed for the updater: set `TAURI_SIGNING_PRIVATE_KEY` (path or text; local key `~/.sysvane/updater.key`, never in the repo) before `npm run tauri build`, and keep the same value in the GitHub secret of that name. `latest.json` lives on the fixed release tag `updates`.

## Conventions

- Rust 2024 edition; `std::thread` and channels in the core, no async runtime unless a module proves it needs one.
- Tests use fake devices; never touch real hardware.
- Add a dependency only when a module needs it.
