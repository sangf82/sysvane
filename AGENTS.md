# Toushou Monitor TUI

Windows DDC/CI controller for multi-monitor hardware settings: a Textual TUI (`src/app.py`, `src/ui/`) and a Typer CLI (`src/cli.py`) over `monitorcontrol`.

## Commands

- `uv sync` — install dependencies (Python 3.12 via `.python-version`).
- `uv run pytest` — run the test suite; keep it green.
- `uv run toushou-tui` — launch the TUI.
- `uv run toushou --help` — CLI commands.
- `uv sync --extra desktop` then `uv run toushou-desktop` — desktop prototype.
- `uvx basedpyright` / `uvx ruff check .` — typing and lint.

## Layout

- `src/core/monitor_manager.py` — monitor discovery and the only DDC read/write gateway.
- `src/core/controls.py` — capability-driven `ControlDefinition` list for the TUI.
- `src/core/profiles.py` — capture/apply profiles and safety filtering.
- `src/core/vcp_catalog.py` — VCP names, ranges, `DANGEROUS_CODES`, `POWER_CODES`.
- `src/ui/screens.py` — dashboard, scanner, and profiles views.
- `src/desktop.py` + `src/ui/desktop.html` — optional pywebview desktop prototype.
- `supabase/DESIGN.md` — design system for the desktop UI (white canvas, emerald primary, hairlines).
- `profiles/` — saved JSON profiles; `docs/CALIBRATION.md` — AL258Q/CVTE findings.

## Monitor safety (hard rules)

A wrong DDC write blacks out the display, and DDC/CI has no undo.

- Never write factory-reset codes `0x04`, `0x05`, `0x06`, `0x08`.
- Never write power codes `0xD6`/`0xC6` with an off or standby value (`VCPCatalog.is_power_off_value`).
- Never write input source `0x60` unless the user explicitly asks and accepts losing video.
- Keep these checks in every write path: CLI `set`, TUI adjustment, `Profile.apply`, scanner.
- `Profile.apply` skips power and input by default; `--include-power` / `--include-input` are opt-in.
- Do not guess a current value before it is read. The TUI blocks adjustment until `_register_values_loaded`.
- Prefer `MonitorManager.set_vcp` over calling `monitorcontrol` directly.

## Calibration notes

- CVTE scalers (Acer AL258Q) lock the RGB gain DAC while a factory color preset is active. Write `0x14 = 11` (User) before `0x16`/`0x18`/`0x1A`; `Profile.apply` enforces this order.
- Left panel: `0x14=5`, R50 G50 B50. Right panel: `0x14=11`, R47 G49 B51. Both: brightness 70, contrast 50.

## Hooks

`.claude/settings.json` runs `.claude/hooks/guard-monitor.ps1` before every Bash call. It blocks unsafe VCP writes. `TOUSHOU_ALLOW_UNSAFE=1` bypasses it; use that only when the user explicitly accepts losing the display.

## Conventions

- Python 3.11+ syntax, `from __future__ import annotations`, dataclasses for value objects.
- Tests use fake managers; never touch real hardware in tests.
- Keep hardware reads and writes off the event loop (Textual workers).
