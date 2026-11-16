# SysVane

Very low-RAM, tray-first Windows app replacing ASUS Armoury Crate and vendor device suites. Rust core plus a React UI. Status and plan: `docs/handoff.md` (read first, update when a step ends), `docs/plan.md`, `docs/architecture.md` (tree and rules).

## Start a task

1. `trail ask "<task>" --source` before grepping. Past decisions, dead ends and gotchas live in `.trail/learnings/`; they replace long rule lists here.
2. Work in the right place: Rust in `core/`, `wire/` (agent `rust-core`); UI in `desktop/`, `DESIGN.md`, `branding/` (agent `frontend-ui`). `wire/` and `desktop/src/wire.ts` change together.
3. Before commit, push or PR: `trail check`. Save what the code cannot say (a cause found after failed attempts, an approach dropped and why) with `trail note`.

## Commands

- `cargo test -p sysvane-core -p sysvane-wire`, `cargo clippy -p sysvane-core -- -D warnings`
- `cd desktop; npm run build` (type-check and build), `npm run tauri dev` (run; build the core too, the UI starts `sysvane-core.exe` from its own folder)
- Release: only when the user asks (`/release`, `.claude/commands/release.md`). It drafts `docs/releases/vX.Y.Z.md`, waits for the user's review, and only then bumps, tags, pushes and lets CI build. During normal work ask about commits, never about releases. Signing key and update channel: see trail learnings on updates.

## Invariants

- No Tauri, WebView, React or async runtime in `sysvane-core`; background RAM is the goal.
- The UI never touches hardware. Each module enforces its own safety in the core. Validate every command and profile from the UI or disk in the core.
- Tests use fake devices, never real hardware.

## Hardware safety (irreversible, always applies)

- Monitor DDC: never write `0x04`/`0x05`/`0x06`/`0x08` (factory reset), `0xD6`/`0xC6` off or standby, or `0x60` (input) unless the user explicitly accepts losing video. All writes go through one guarded function; no flag unlocks them. Never guess a value before reading it.
- LCD: never send command types 4 or 5 (OTA); stop if the panel reports PM 32 or 50.
- `.claude/hooks/guard-monitor.ps1` blocks these in shell calls. `SYSVANE_ALLOW_UNSAFE=1` bypasses it only when the user accepts losing the display.

## UI testing

`agent-desktop` only on SysVane: `agent-desktop snapshot --app sysvane-ui.exe -i`, headless (`set-value`, not `type`), `--timeout-ms 30000` on cold start. Run `agent-desktop skills get desktop` and `skills get platform` first. Never another app or the whole desktop.

## Model roles

- Sonnet leads and decides. Opus advises: call the advisor before choosing an approach for a milestone, before finishing code that writes to hardware (DDC, LCD, fans) or changes `guard-monitor`, and before pushing a release. Haiku does cheap legwork: `scout` (read, find, trace, answers with `file:line`) and `verify` (runs fmt, clippy, tests, build; returns only failures). `rust-core` and `frontend-ui` write code on Sonnet.
- Delegate lookups to `scout` and test runs to `verify` instead of reading files or test output in the main session.
- Flow: `/status` (new session briefing) → `/milestone` (grill the owner, plan, develop, test, review) → `/handoff` (end the session: rewrite `docs/handoff.md`, trail notes) → `/release` (changelog review, tag, CI). Between them, ask only about commits.

## Agents and skills

- `rust-core` preloads `rust-best-practices`; `frontend-ui` preloads `impeccable`, `animate-ui`, `agent-desktop`. `impeccable` and `rust-best-practices` are not hidden (hiding them blocks preload); prefer delegating to the agents. The user types `/unslop` for a review pass.
- Hooks (`.claude/hooks/`): `format-rust` after `.rs` edits; `lint-on-stop` runs fmt, clippy and `tsc` on files edited since the last clean pass, and blocks the stop once on failure.
- Claude Code questions: use the `claude-code-guide` MCP first.
