# Toushou Monitor TUI

Multi-monitor DDC/CI controller with a Textual TUI and a Typer CLI.

## Requirements

- Windows with DDC/CI enabled in your monitor OSD settings.
- `uv` installed.
- Python 3.12. The repo includes `.python-version`, so `uv` can manage this automatically.

## Setup

From the project root:

```powershell
cd C:\Projects\toushou-monitor-tui
uv sync
```

`uv sync` creates the local virtual environment in `.venv` and installs the app dependencies from `pyproject.toml` / `uv.lock`.

## Run The TUI

Launch the full-screen terminal interface:

```powershell
uv run toushou-tui
```

Equivalent direct Python entry point:

```powershell
uv run python main.py
```

## OSD-Style TUI Guide

The app opens directly on the Controls screen. Monitor detection happens in stages: Windows EDID/DDC handles appear first, then DDC/CI capabilities and current values load in the background. EDID names from Windows are preferred, so AL258Q monitors appear as `Display 1: AL258Q` and `Display 2: AL258Q` even when DDC/CI reports the vendor model as `CVTE`.

Basic workflow:

- Select a monitor in the left sidebar and press `Enter`.
- Use the top status chips for quick state: refresh rate, Adaptive-Sync candidate, brightness, mode candidate, and input.
- Use the OSD-style categories: Picture, Color, Game, and System.
- Each row includes a short note describing what the setting changes or why it is read-only/unmapped.
- Use the `-` and `+` buttons to change confirmed DDC/CI controls such as brightness, contrast, color preset, RGB gain, saturation, hue, input source, volume, and power mode.
- For input source, color preset, power mode, and other discrete controls, `-` and `+` cycle through known values.
- Controls marked `EXP` are experimental OSD candidates. The first click only arms the control and shows a warning; click again to write the candidate VCP value.
- Unmapped experimental controls, such as ECO, Black Level, and Aspect Ratio, are shown as `Unmapped` until the scanner confirms their VCP code.
- Refresh rate is read-only in the TUI because DDC/CI only reports the active signal timing. Change refresh rate in Windows Display Settings or the NVIDIA control panel.
- Press `R` when you want to refresh monitor detection and current control values. This can take a few seconds because DDC/CI reads are slow.
- Use `F2` for the manufacturer-register scanner. To map an OSD-only function, take Snapshot 1, change exactly one setting in the physical OSD, take Snapshot 2, enter the OSD label, and press `Confirm Mapping`.
- Use `F3` for saved profiles (`p` for quick apply).
- Press `Q` to quit.

DDC/CI notes:

- Enable DDC/CI in the monitor's physical OSD menu.
- Windows EDID may report the friendly product name while DDC/CI reports a vendor model. The app shows both in the details pane.
- Some VCP values are unsupported even when a monitor responds to DDC/CI.
- Input switching can temporarily disconnect the active display path; this is monitor behavior, not an app crash.
- The app keeps UI actions responsive by doing DDC/CI reads and writes in background workers.
- The TUI can clone all DDC/CI-exposed OSD controls and can discover additional vendor-only controls, but it cannot control OSD functions that the monitor never exposes over DDC/CI.

## Run The CLI

Use the Typer CLI entrypoint (`toushou` or `python -m src.cli`):

```powershell
uv run toushou --help
```

Available commands:

```powershell
# List all detected monitors
uv run toushou list-monitors

# Read a register value (e.g. Brightness 0x10)
uv run toushou get 0x10 --monitor 0

# Set a register value
uv run toushou set-value 0x10 70 --monitor 0

# List saved profiles
uv run toushou list-profiles

# Apply a profile to a specific monitor
uv run toushou apply-profile Calibrated-Left --monitor 0
uv run toushou apply-profile Calibrated-Right --monitor 1

# Apply dual calibration to BOTH monitors simultaneously
# (input source 0x60 is skipped by default; add --include-input to switch ports)
uv run toushou apply-dual
```

Common VCP codes:

- `0x10`: brightness / luminance
- `0x12`: contrast
- `0x14`: color preset (5 = 6500K D65, 11 = User Mode)
- `0x16`: red video gain
- `0x18`: green video gain
- `0x1A`: blue video gain
- `0x60`: input source
- `0x62`: audio volume
- `0xC6`: power mode (vendor)
- `0xD6`: power mode (standard)

## Dual Monitor Hardware Color Calibration

For detailed calibration notes, panel variance compensation, and technical findings on Acer AL258Q (CVTE scaler), see:
👉 **[`docs/CALIBRATION.md`](docs/CALIBRATION.md)**

Quick summary of the calibrated setup:

- **Monitor 0 (Left - DP1)**: Brightness 70, Contrast 50, Color Preset 5 (6500K), R50 G50 B50.
- **Monitor 1 (Right - DP2)**: Brightness 70, Contrast 50, Color Preset 11 (User Mode), R47 G49 B51 _(Red reduced to eliminate reddish tint, Green balanced to prevent yellowing, Blue boosted for crisp white match)_.

## Desktop Prototype (optional)

A pywebview desktop window reuses the same Python DDC/CI core and safety checks. It is a prototype, not a packaged installer.

```powershell
uv sync --extra desktop
uv run toushou-desktop
```

The UI lives in `src/ui/desktop.html`; the bridge is `src/desktop.py`. It follows the Supabase design system in `supabase/DESIGN.md` (white canvas, near-black ink, emerald primary, hairline borders). Monitor writes still go through `MonitorManager` and `VCPCatalog`, so factory-reset, power-off, and input-source protections apply.

## Development

Run tests:

```powershell
uv run python -m pytest
```

Check typing and formatting:

```powershell
uvx basedpyright
uvx ruff check .
```

## monitorcontrol Integration

This project uses `monitorcontrol` for DDC/CI access. The app follows the upstream API rules:

- `monitorcontrol.get_monitors()` returns closed monitor handles.
- Monitor operations must run inside `with monitor:`.
- High-level reads and writes use methods such as `get_luminance()`, `set_luminance()`, `get_contrast()`, `set_input_source()`, and `get_vcp_capabilities()`.
- Low-level VCP reads may return `(current, maximum)`, so the app stores the current value.
- `get_vcp_capabilities()` can be slow, so startup first lists detected handles and then loads detailed labels and current values in the background.

References:

- https://github.com/newAM/monitorcontrol
- https://newam.github.io/monitorcontrol/api.html
