# Handoff — M4 DDC module

Updated 2026-11-16. Read this before starting; update or remove a step when it ends. Tree and rules: `docs/architecture.md`. Milestones: `docs/plan.md`.

## State
The file tree refactor is committed (`518d473` … `9a25025`). M4 code and the UI redesigns below pass every check and are **not committed**. M4 is blocked on one thing: Display 2 does not take preset 11 (step E2).

| Step | What | State |
|---|---|---|
| A | `core/src/modules/monitor/vcp.rs`: allowlist + the one guarded write (reads the code first; a value above the monitor's max is refused, except 0x14, checked against MCCS 1..=13). `profile.rs`: JSON profiles, `schema: 1`, malformed entries skipped, written in the order picture mode 0xB6, preset 0x14, rest; unsupported codes skipped, capture, atomic save, name cleaning, built-ins read only | Done, tested with `vcp::Fake` |
| B | `monitor/ddc.rs`: dxva2 enumerate, read, write, model from the capabilities string; display number from the HMONITOR (fixes EDID-by-position) | Done, used on the real PC |
| C | `monitor/mod.rs`: one `Monitor` module per physical monitor. Wire protocol 4: `DeviceId::Monitor(u8)`, monitor `ControlId`s, `ControlValue::Text`, `Device.name` and `Device.profiles`. Host runs `Module::start` on the module thread | Done, tested |
| D | `desktop/src/pages/Displays.tsx`: sliders (sent 250 ms after the drag pauses), colour preset, profile apply and save; nav entry and `icons/monitor.tsx` | Done, used on the real PC |
| E1 | Real monitors, read only (2026-11-15): both found, named `CVTE · Display 1` / `CVTE · Display 2` (capabilities `model()` is the scaler, not `AL258Q`). Display 1: B70 C50 R50 G50 B50, preset 5. Display 2: B70 C50 R47 G49 B51, preset reads **5**, not 11 as the calibration notes say | Done |
| E2 | Apply `Calibrated-Left` to Display 1 and `Calibrated-Right` to Display 2; Display 2 must then read preset 11 | See "E2 findings" below. Display 1 done. Display 2: preset stays 5. **Next** |
| F | Displays page redesign: one card per monitor (swatch tinted by its gains and brightness), settings for the selected one in Picture / Colour / Profiles; sidebar brand row removed | Done, checked on screen |
| G | Scan: `ControlId::Scan` on `Core` re-runs `monitors()` and replaces every `Monitor` host (`Devices::rescan`) | Done, checked on the real PC |
| H | Resolution and refresh rate: `monitor/mode.rs`, `ControlId::DisplayMode` + `KeepDisplayMode`, `Device.modes`. A new mode is a trial, reverted after 15 s (`Module::wake`/`tick`) or when the module is dropped; kept = saved with `CDS_UPDATEREGISTRY`. UI: two dropdowns + keep/revert dialog | Done, checked on the real PC by the owner (2026-11-16) |
| I | Overview redesign (outside M4): one panel per module on a 6-column grid, CPU 2/3 wide (usage chart + per-core bars), GPU (load line, VRAM and power meters), Memory (stacked bar, commit, swap), Disks (capacity bar + read/write spark per drive), Network (mirrored rx/tx, new `K.netTx` history), Top processes, Other sensors. Container queries: halves under 1040 px, one column under 640 px. `MemSplit` moved to `components/meters.tsx` | Done, checked on screen by the owner |
| J | Processes redesign (outside M4): List / By name / Tree views (`processTable.ts::tree`, PID reuse guarded by start time), Sort dropdown (most/least CPU or memory), load bars behind CPU and memory, zeros muted, columns reordered (Parent and Virtual hidden by default), columns hidden on narrow tables, details in a side pane (on top under 1100 px, closed with X or Esc), toolbar in two lines. Overview top-process panel: same style, click the sorted column again for least first | Done, checked on screen by the owner |
| K | Process icons: `desktop/src-tauri/src/icon.rs` (`exe_icon` command, ExtractIconExW, RGBA to the UI, drawn to a data URL and cached per path in `components/ProcIcon.tsx`). `windows-sys` added to `sysvane-ui` | Done, tested (`explorer.exe` icon, missing file) |
| L | Shared UI: `components/glyphs.tsx` (static Lucide icons for toolbar controls), dropdown uses drawn chevron/check, `.dd-pos` carries its z-index (popup was under sticky table headers), dropdown opens on a click without pointer (`detail === 0`) so screen readers and UI Automation can open it | Done |
| M | Universal profiles (2026-11-16): continuous codes (`vcp::scaled`: brightness, contrast, gains, 6-axis, black levels, volume) are a percentage of each monitor's maximum in the UI, in profiles and in `vcp::write`; preset, picture mode and 0x0C stay raw. Capabilities parsed once (`vcp::Caps`: model + listed values); `Device.presets` limits the UI preset list. `Profile.model`: a model-specific code (0xB6) is written only to a monitor of the profile's model; the Calibrated profiles carry `"model": "CVTE"`, Gaming and D65 are universal | Done, checked on the real PC (2026-11-16): sliders show 70/50/47/49/51, brightness 70→65→70 through the slider, `Test-Universal` saved (model CVTE, percentages) and applied (65 read back), then deleted. Both AL258Q capabilities list `14(05)` only, so the preset dropdown shows 6500K alone |
| N | Busy state: `Device.busy` (the module is running a job). Displays: Apply/Save show a spinner and "Applying…"/"Saving…" until a snapshot at least 1.5 ticks after the click shows the monitor idle, or an error | Done, checked on the real PC: Calibrated-Left applied, "Applying…" for about 1.5 s, values unchanged |
| O | Controls from capabilities: `ControlId::Vcp(code)` for every listed code that `vcp::scaled` allows beyond the six named ones (volume, black levels, 6-axis saturation and hue); `Caps.values` keeps every listed code. UI labels them (`MORE_PICTURE`, `MORE_COLOUR`). A colour preset with one option shows as text | Done, checked on the real PC: Volume appears (0x62), 50→45→50 read back; preset shows "6500K" as text |
| P | GPU colour and orientation (2026-11-16): `monitor/gamma.rs` (brightness, contrast, gamma, R/G/B gain → `SetDeviceGammaRamp`, written only when not neutral, saved to `config/monitor/gpu-<Display n>.toml`, written again at start, resume and after every mode change). `ControlId::Orientation` (0..=3) is a trial like `DisplayMode` (`Before { mode, turn }`). UI: Orientation dropdown, "GPU colour" group with Reset | Done, checked on the real PC: blue 85 read back as B255=55705, Reset back to identity; Portrait trial reverted by itself at 15 s (pipe and UI). **One UI run left Display 2 in portrait while the core read landscape; not reproduced, cause unknown** |

Checked 2026-11-16: `cargo fmt --check`, clippy `-D warnings` on `sysvane-core` (debug) and `sysvane-ui` (debug, release), tests 55 core + 1 ui + 3 wire, `npm run build`, impeccable detector (no findings).

### E2 findings (2026-11-15/16)
1. First apply of `Calibrated-Right` to Display 2 was refused before sending: `0x14 = 11 is above the maximum 5`. 0x14 is a list, so its reported maximum is meaningless; `vcp::write` now checks 0x14 against MCCS 1..=13.
2. Second apply: no error, gains and brightness read back right, preset still 5 after a rescan.
3. Read only, Display 1 (`tools/probe-ddc.ps1`): picture mode `0xB6` = 3, preset `0x14` = 5 with max **5**. The May scan (`VCP_REFERENCE.md` in `30b7852`) had `0xB6` = 1 and 0x14 max **14**.
4. `0xB6` added to `vcp::WRITABLE` and written first. Test profile `%ProgramData%\SysVane\config\monitor\Test-Standard-User.json` (`0xB6` = 1, `0x14` = 11) applied to Display 1 through the app: no error, **both ignored** (still 3 and 5). Brightness 70 → 65 → 70 through the app took, so writes reach the monitor.
5. Conclusion so far: this AL258Q ignores DDC writes to picture mode and to presets above its current range; the range seems to follow the picture mode set in its own menu. Both Calibrated profiles store `0xB6` = 3.
6. Display 2 was switched off by the owner during the test (not caused by a write).
7. 2026-11-16, read only: both monitors' capabilities are `...vcp(02 0C 10 12 14(05) 16 18 1A 52 60(11 12 0F 10) 62 8D(01 02) AC AE B2 B6 C6 C8 C9 CC(02) D6(01) DF)...`. The scaler declares preset 5 only, which fits finding 5. The R47 G49 B51 panel is now `Display 1` in the app (was Display 2 on 2026-11-15).
8. 2026-11-16, through the app on Display 2 (profile with only `0x14`): 4 (5000K) and 1 (sRGB) sent with no error, both ignored, preset still 5. The capabilities are right: over DDC this AL258Q takes preset 5 only; other presets only from its OSD.

**Next**: add `"model": "CVTE"` to `Test-Standard-User.json` (without it 0xB6 is now skipped, step M); owner sets the left monitor's picture mode to Standard in the OSD; then re-apply `Test-Standard-User` and read with the probe. If 0x14 = 11 sticks, change both Calibrated profiles' `0xB6` to the Standard value (or drop it) and repeat on Display 2. Delete the test profile afterwards.

## Decisions
- Writes use an allowlist of image codes (`vcp::WRITABLE`), not a deny list: resets, power and input are absent, so no profile or UI value can reach them. Input switching is not offered in M4. `0xB6` (picture mode) was added 2026-11-16: image only.
- 0x14 is checked against MCCS 1..=13, every other code against the monitor's reported maximum.
- Monitors keep their own settings, so re-apply after sleep only reads them again; nothing is written on resume.
- Default profiles live in `core/src/modules/monitor/profiles/` and are compiled in. `Calibrated-Dual-Setup.json` stays in `docs/devices/ddc/profiles/` as history.
- User profiles: `%ProgramData%\SysVane\config\monitor\*.json`. A built-in name cannot be overwritten.
- DDC calls run on the module thread under the host watchdog, not in a worker process.
- Exe icons are read by the Tauri shell, not the core: the core's RAM stays as it is, and reading an icon from a file is not hardware.
- No raw DDC writes from scripts: the auto-mode classifier refused one (2026-11-16). Test writes go through the app (a user profile or a slider). `tools/probe-ddc.ps1` only reads.

## Not done
1. Step E2 above (blocks M4).
2. Hot-plug: no automatic detection; Scan re-detects. A hung DDC call on the same monitor delays a trial-mode revert.
3. Review passes (`/code-review`) on the refactor, M4 and the UI redesigns.
4. From the refactor: flyout, core-start alert and elevated restart are checked by compiling only; `tools\probe-temp.cmd` not run after the PawnIO path move.
5. Nothing committed since `9a25025`.

## Watch out
- `desktop/src/wire.ts` stays at that path: two tests in `wire/src/lib.rs` read it by path.
- Run clippy in release as well as debug. `config::load`/`save` are allowed dead in release: only the debug fake uses them.
- `trail check` flags `desktop/package-lock.json` (version 2→4) against a Tailwind rule, and `AGENTS.md:71` against the per-edit clippy learning: both false.
- `vcp::Fake` is test-only. The debug fake device (`modules/fake.rs`) is unrelated and stays.
- `agent-desktop` on this PC: `snapshot` returns a WebView2 browser-chrome tree; use `find --app sysvane-ui.exe --window-id <id>` instead. Monitor cards are `switch` elements: use `toggle`, not `click`. Headless `press` fails (window focus unconfirmed); open dropdowns with `click` and pick the `option`.
- `tools/probe-ddc.ps1` sees a monitor only while it is on; it can read while the core runs.
