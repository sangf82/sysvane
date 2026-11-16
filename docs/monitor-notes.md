# Monitor module notes

Moved out of `docs/handoff.md` on 2026-11-16 so `/status` stays small. Historical until re-verified.

### Displays redesign (2026-11-16, `4aa203f`; review fixes uncommitted)
- Under the monitor cards, one panel per section, full width, in this order: Profiles (first, for a quick pick) / Display (Windows: resolution, refresh rate, orientation, Identify) / Picture (DDC: brightness, contrast, volume) / Colour.
- Colour has a switch, Monitor (DDC) | Graphics card. Both stay in force; the switch only picks which sliders show. Default: Graphics card when its colour is changed or the monitor has no DDC colour, Monitor otherwise. A note says when the other one is also changed.
- Section headers carry a 16 px Lucide icon from `components/glyphs.tsx` (bookmark, monitor, sun, palette); Identify and Reset carry scan and undo. Header class is `.sechead`: `.mhead` already belongs to the meter rows.
- Sliders are drawn from tokens (`--pct` fills the track); the number beside each is an input (Enter or blur sends, Esc drops, held to 0–100). No white-point swatches in the headers: they read as a radio or a checkbox. The card swatch shows the monitor white point through the GPU one.

### Review of the redesign (impeccable audit, 15/20)
- Fixed: slider fill and thumb use `--control` (`#1f9d6a` in light, about 3.3:1 on `#fafafa`; `#3ecf8e` in dark); `.sechead` wraps (checked at 700 and 560 px, both tabs); `.mnum` focus has the 2 px emerald ring; the thumb shadow comes from `--ink`. The empty track stays light (`--hairline-strong`): the thumb and fill identify the control.
- Open:
1. Thumb 14 px and number field about 22 px tall, under 24 px.
2. `.problem` (`styles.css:285`) uses a red left rule; the detector flags it.
- The detector ran on source only; the rendered scan needs the app started with a DevTools port. Typing into the number field is not tried on the real app.

### M4 summary (what exists)
- `core/src/modules/monitor/`: `vcp.rs` (allowlist, the one guarded write, `Caps` parser, `Bus` trait, test `Fake`), `ddc.rs` (dxva2, capabilities, EDID from the registry, gamma ramp), `profile.rs` (JSON profiles), `mode.rs` (resolution, refresh rate, orientation, desktop-shape check), `gamma.rs` (GPU colour), `edid.rs` (name and serial), `mod.rs` (`Monitor` module).
- Levels (brightness, contrast, gains, 6-axis, black levels, volume) are a percentage of each monitor's maximum in the UI, in profiles and in `vcp::write`. Preset, picture mode and 0x0C stay raw.
- Controls follow the capabilities string: the six named ones plus `ControlId::Vcp(code)` for every other listed, scaled code. The preset list comes from the capabilities; one option shows as text.
- Profiles carry `model`; 0xB6 (picture mode) is written only to the same model. Order: 0xB6, 0x0C, 0x14, the rest.
- Resolution, refresh rate and orientation are 15 s trials (keep or revert; a failed revert is retried every second; quit from the tray and Scan stop the modules first, so their `Drop` reverts).
- GPU colour through `SetDeviceGammaRamp` (brightness, contrast, gamma, R/G/B), written only when not neutral, again at start, resume and after mode changes; saved per monitor by EDID serial, or by display number when the serial is zero. `GpuReset` is one command. `Identify` dims the monitor 3 times through the ramp.
- Monitor names come from the EDID name ("AL258Q · Display 1"); the capabilities' `model` ("CVTE", the scaler) gates model-specific profile codes.
- UI: Displays page, see "Displays redesign" above. Apply/Save spinners via `Device.busy`.

### Checked on the real PC (2026-11-16)
- Both calibrated profiles applied from the app with no error: Display 2 (left) B70 C50 R50 G50 B50 preset 5; Display 1 (right) B70 C50 R47 G49 B51 preset 5.
- Sliders, profile save/apply, Volume (0x62), GPU colour (B255 = 55705 at blue 85, Reset to identity), Identify, Portrait trial reverting at 15 s, EDID names.
- After the zero-serial fix, the core running on the PC was not restarted: the next start keys GPU colour by display number.

## Decisions
- Writes use an allowlist of image codes (`vcp::WRITABLE`), not a deny list: resets, power and input are absent, so no profile or UI value can reach them.
- 0x14 is checked against MCCS 1..=13, every other code against the monitor's reported maximum.
- These AL258Q (picture mode 3) take preset 5 only over DDC: capabilities say `14(05)`; writes of 1, 4 and 11 are ignored. `Calibrated-Right` uses preset 5.
- Monitors keep their own DDC settings, so re-apply after sleep only reads them; the GPU ramp is written again.
- DDC calls run on the module thread under the host watchdog, not in a worker process (review finding 2026-11-16, kept: a hung dxva2 call leaves one thread behind, logged, and the module shows "not answering").
- Default profiles live in `core/src/modules/monitor/profiles/` and are compiled in. User profiles and GPU colour files: `%ProgramData%\SysVane\config\monitor\`.
- Exe icons are read by the Tauri shell (on a blocking thread), not the core.
- No raw DDC writes from scripts: test writes go through the app or the command pipe. `tools/probe-ddc.ps1` only reads.

## Not done
1. `DeviceId::Monitor(u8)` is the enumeration position: after a Scan or hot-plug a command can reach another monitor. Key devices by a stable id (needs a wire change).
2. Gains at preset 5: written and read back, but whether they change the picture (the old "preset 5 locks the DAC" note) is not checked by eye.
3. One UI run on 2026-11-16 left Display 2 in portrait while the core read landscape; not reproduced. `mode::set` now checks the desktop shape and sets the mode again once.
4. While a display is in portrait, the resolution list from Windows is not checked.
5. Hot-plug: no automatic detection; Scan re-detects.
6. From the refactor: flyout, core-start alert and elevated restart are checked by compiling only; `tools\probe-temp.cmd` not run after the PawnIO path move (CPU temperature itself reads fine through the elevated core).
7. Test files left by the owner: `%ProgramData%\SysVane\config\monitor\Test-Standard-User.json`, `gpu-Display 2.toml` (neutral).

## Watch out
- `desktop/src/wire.ts` stays at that path: two tests in `wire/src/lib.rs` read it by path.
- Run clippy in release as well as debug.
- `trail check` flags `desktop/package-lock.json`, `AGENTS.md:71`, `devices.rs` and `config.rs` by keyword: all false so far.
- `vcp::Fake` is test-only. The debug fake device (`modules/fake.rs`) is unrelated and stays.
- A dev core started by `npm run tauri dev` is unelevated: no CPU temperature. Start `target\debug\sysvane-core.exe` with `-Verb RunAs` (after stopping the unelevated one). An elevated core can only be stopped from an elevated shell.
- `agent-desktop` on this PC: use `find --app sysvane-ui.exe --window-id <id>`, not `snapshot`. Anything with `aria-pressed` (monitor cards, the Colour switch, `.seg` buttons) is a `switch`: use `toggle`. After a hot reload the tree can lag; pass `--timeout-ms 15000`. Open dropdowns with `click`, then pick the `option`; picking the current value sends nothing.
- `System.Windows.Forms.Screen` caches in a long-running PowerShell: read screen sizes from a fresh process.
- `tools/probe-ddc.ps1` sees a monitor only while it is on; it can read while the core runs.
