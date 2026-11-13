# SysVane

Very low-RAM, tray-first Windows app replacing ASUS Armoury Crate and vendor device suites: hardware monitoring, displays, peripherals, RGB, fans and the AIO LCD. All SysVane code is Rust, except the React UI. Plan and status: `docs/plan.md`.

## Commands

- `cargo check -p sysvane-core` — type-check the core.
- `cargo test -p sysvane-core -p sysvane-wire` — run core and wire tests; keep them green.
- `cargo clippy -p sysvane-core` — lint the core.
- `cd desktop; npm run build` — type-check and build the UI.
- `cd desktop; npm run tauri dev` — run the UI.

## Layout

- `core/` — `sysvane-core`, the headless background program. Sensors, device detection, config and built-in modules go here.
- `desktop/` — `sysvane-ui`, Tauri 2 + React/Vite. Started on demand and exits when closed. Never talks to hardware.
- `wire/` — `sysvane-wire`, all that the core and the UI share: pipe name, process names, snapshot types. `desktop/src/wire.ts` mirrors the types; a test fails if the field names differ.
- `modules/ddc/` — calibration profiles and VCP notes kept for the Rust DDC module.
- `DESIGN.md` — UI design system, adapted for the app in `docs/plan.md`.
- `branding/logo/` — the Crate logo kit (`kit/`, rules in `kit/GUIDELINES.md`) and the scripts in `scripts/` that regenerate it and write the Tauri icons. The tray icon is `core/assets/tray-32.rgba`.
- `tools/` — read-only hardware probes for use without the core.
- `docs/research/` — dated research notes. Historical; they use the old project name.
- Commit `30b7852` holds the removed Python implementation to port from.

## Architecture rules

- Keep Tauri, WebView and React out of `sysvane-core`. Background RAM is the main goal.
- Modules are compiled in. Do not add DLL or WASM plugin loading.
- Code that needs a kernel driver or can hang runs in a supervised worker process.
- One owner per hardware bus. A third-party tool may only run on a bus the core does not touch, and only through a fixed command line.
- Each module owns its device's safety rules, enforced in the core, not the UI.
- The core and the UI share only `sysvane-wire`; the UI starts `sysvane-core.exe` from its own folder, so build both.
- Keep hardware I/O off the UI and any thread with a deadline.
- Target privilege model (`docs/plan.md`, M1): the core runs as administrator from a scheduled task, installed under `Program Files`; the UI runs as a normal user. Until M1, the core runs unelevated and only the temperature helper is elevated. Never trust a command or profile from the UI or disk: validate it in the core.
- Each device module keeps its own config, default profiles and fake device. No merged config file. Profiles are data, never code.

## Monitor safety (hard rules)

A wrong DDC write blacks out the display, and DDC/CI has no undo.

- Never write factory-reset codes `0x04`, `0x05`, `0x06`, `0x08`.
- Never write power codes `0xD6`/`0xC6` with an off or standby value.
- Never write input source `0x60` unless the user explicitly asks and accepts losing video.
- All DDC writes go through one guarded function. No flag may unlock reset or power-off codes.
- Profile apply skips power and input by default.
- Do not guess a current value before it is read.

## LCD safety

- Never send OTA or firmware-upgrade commands to the panel (command types 4 and 5).
- Stop if the panel reports PM 32 or 50 until the Logo/RGB565 command conflict is resolved.

## Calibration notes

- CVTE scalers (Acer AL258Q) lock the RGB gain DAC while a factory color preset is active. Write `0x14 = 11` (User) before `0x16`/`0x18`/`0x1A`.
- Left panel: `0x14=5`, R50 G50 B50. Right panel: `0x14=11`, R47 G49 B51. Both: brightness 70, contrast 50.
- Details: `modules/ddc/docs/CALIBRATION.md`.

## Testing the UI

Use `agent-desktop`, a CLI that reads and operates apps through Windows UI Automation. Before first use in a session, run `agent-desktop skills get desktop` and `agent-desktop skills get platform`, and follow them.

- Only operate SysVane: `agent-desktop snapshot --app sysvane-ui.exe -i`. Never another app or the whole desktop, because output can hold private content.
- Quote refs in PowerShell: `'@s8f3k2p9:e8'`.
- Stay headless. Write text with `set-value`, not `type`.
- The Chromium tree fills in slowly on a cold start: `snapshot --timeout-ms 30000`.

## Hooks and tools

- `.claude/hooks/guard-monitor.ps1` runs before every Bash and PowerShell call. It blocks unsafe VCP writes and keeps `agent-desktop` on sysvane. `SYSVANE_ALLOW_UNSAFE=1` bypasses it; use that only when the user explicitly accepts losing the display. Keep its patterns in step with the Rust DDC command line once that exists.
- `.claude/hooks/format-rust.ps1` runs `rustfmt` after each edit of a `.rs` file.
- For Claude Code questions, use the `claude-code-guide` MCP first.

## Agents and release

- `rust-core` agent (`.claude/agents/`): `core/`, `wire/`, `modules/`; loads `rust-best-practices`; runs fmt, clippy `-D warnings` and tests before it reports.
- `frontend-ui` agent: `desktop/`, `DESIGN.md`, `branding/`; loads `impeccable`, `unslop`, `animate-ui`, `agent-desktop`; runs `npm run build`.
- `.claude/hooks/lint-on-stop.ps1` runs at Stop: fmt + clippy if Rust changed, `tsc --noEmit` if `desktop/src` changed; failure blocks the stop once.
- CI: `.github/workflows/ci.yml` on push to `main` and PRs. Release: `powershell -NoProfile -File scripts\bump-version.ps1 X.Y.Z`, commit, `git tag vX.Y.Z`, `git push --tags`; `release.yml` checks the tag equals the Cargo and package versions, builds the NSIS installer and publishes a pre-release. Updater signing and `latest.json` arrive in M2.

## Conventions

- Rust 2024 edition, `std::thread` and channels in the core, no async runtime unless a module proves it needs one.
- Tests use fake devices; never touch real hardware in tests.
- Add a dependency only when a module needs it.
