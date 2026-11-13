# Armoury Crate replacement: target, needs, plan and LCD add-on research

**Date:** 2026-11-08
**Status:** Historical research. The current plan is [docs/plan.md](../plan.md), under the new name sysvane.
**Companion note:** [2026-11-08-universal-pc-control-tools.md](2026-11-08-universal-pc-control-tools.md) holds the tool-by-tool survey and its sources. [2026-11-04-low-ram-react-monitor-stack.md](2026-11-04-low-ram-react-monitor-stack.md) covers UI shell RAM.

**Evidence labels.** *Measured here* means read on this PC on 2026-11-08. *Source* means read in a project's code or docs. *User-reported* means the owner told us. *Estimate* means judgement with no measurement.

## 1. Where the target ended up

| Step | Direction | Outcome |
|---|---|---|
| 1 | Review and optimise the Python TUI and desktop monitor app | Review done, no fixes applied (section 8) |
| 2 | Replace Armoury Crate with one very light app for monitoring, displays, peripherals, RGB and fans | Survey found no single tool does this |
| 3 | TUI or desktop? | Desktop, tray-first. TUI cannot live in the tray or run in the background without a terminal |
| 4 | Universal for every PC and laptop? | Only as "detect and enable what the machine supports". Full coverage is impossible because peripherals, laptop fans and board fan mapping have no shared standard |
| 5 | ASUS-first replacement with add-ons for the owner's devices | **Current scope** |
| 6 | Survey verdict: combine existing tools plus one thin "apply my setup" command | Recommended, but the owner has not chosen between this and a custom app |
| 7 | Add the Thermalright AIO LCD, with PNG or GIF backgrounds and a live stats overlay | **Requested add-on, researched below** |
| 8 | Research only for now | **Current phase** |

## 2. Needs

**Hard goals**
- Stop paying for Armoury Crate (about 848 MB) and TRCC (about 300 MB).
- Very low resident RAM. Background RAM matters more than window-open RAM.

**Features wanted**
- Hardware monitoring.
- Monitor control over DDC/CI for two AL258Q panels, keeping the existing safety rules.
- ASUS keyboard and mouse settings through the ROG Omni receiver.
- Headset settings for the Arctis Nova 3P.
- Aura RGB.
- Fan control.
- AIO LCD: static image or GIF background, with an overlay showing PC stats such as CPU and GPU temperature.
- Add-ons per device, so new hardware can be added later.

**Non-goals, agreed or implied**
- Full Armoury Crate parity: game library, news, deals, ROG Live and update agents are dropped.
- Support for every PC and laptop in the first version.
- macOS and Linux.
- Any write that risks the hardware. See the safety rules in `AGENTS.md` and section 6.6.

## 3. This PC

| Item | Value | Evidence |
|---|---|---|
| Board | ASUS TUF GAMING B850M-PLUS WIFI | Measured here |
| CPU | Ryzen 9 9900X | Measured here |
| GPU | NVIDIA RTX 5070 Ti, plus AMD iGPU | Measured here |
| RAM | 31.6 GB | Measured here |
| Aura controller | USB `0B05:19AF` | Measured here. The ID is in OpenRGB's detection list |
| ROG Omni receiver | USB `0B05:1ACE` | Measured here. Keyboard and mouse models unknown |
| Headset | SteelSeries Arctis Nova 3P, USB `1038:2269` | Measured here. Matches HeadsetControl's device file |
| AIO LCD | USB `87AD:70DB`, "USBDISPLAY", driver WinUSB | Measured here. AIO model unknown |
| Monitors | 2x Acer AL258Q over DisplayPort | Measured here |
| Vulnerable Driver Blocklist | Enabled | Measured here |
| HVCI / Memory Integrity | Off | Measured here |
| PawnIO | Service running, start type Manual, `PawnIOLib.dll` present | Measured here |
| TRCC | "TRCCCAP" 2.1.6 installed, startup task `TRCCAppStartup` present, no process running | Measured here |
| Rust toolchain | cargo 1.99.0, stable MSVC | Measured here |

**Armoury Crate RAM (private MB, measured here):** `asus_framework` x9 = 594.3, ArmouryCrate.Service 71.3, ROGLiveService 37.7, ROGKBPlugin 36.1, ArmouryCrate.UserSessionHelper 33.9, AsusFanControlService 23.7, ArmourySocketServer 17.0, ArmourySwAgent 15.4, LightingService 13.7, GameSDK 2.1, AsusCertService 1.6, ArmouryHtmlDebugServer 1.3. **Total about 848 MB**, 70% of it in `asus_framework`, which is probably the UI and plugin host (inferred from the name).

**TRCC:** about 300 MB, user-reported, and turned off for that reason. Not measured.

## 4. Plan per feature

| Feature | Proposed path | Resident RAM | Status of evidence |
|---|---|---|---|
| Fans | BIOS Q-Fan curves | 0 MB | ASUS manual excerpts. LibreHardwareMonitor and FanControl look unreliable on this board family |
| Sensors on demand | HWiNFO, opened when needed | 0 MB (estimate) | Current version and blocklist status unverified |
| Monitors | Existing `toushou` CLI and profiles | 0 MB | Works today on this PC per repo docs |
| Mouse, keyboard | Onboard profiles, G-Helper only when changing settings | 0 MB | G-Helper source supports the Omni receiver. Exact models need checking |
| Headset | HeadsetControl, run once | 0 MB | Device file matches this USB ID |
| RGB | BIOS Stealth, or `OpenRGB --profile` at logon | 0 MB | Detection ID matches. Persistence unverified, and hardware modes are reported broken on a sibling board |
| **AIO LCD** | **Small headless daemon, the only resident part** | **10–20 MB (estimate)** | Section 6 |

**Corrected expectation.** The survey's "about 0 MB resident" no longer holds once the LCD is included, because this panel needs continuous frames (section 6.2). The realistic target is the LCD daemon alone, compared with about 1.15 GB for Armoury Crate plus TRCC.

## 5. Decisions

**Made**
- Target is a desktop or headless Windows program, not the TUI.
- ASUS-first, plus the owner's other devices.
- The LCD add-on is wanted, with static and animated backgrounds plus overlay.
- No full Armoury Crate clone.

**Recommended, awaiting the owner**
- Combine existing tools rather than build a custom app with a UI.
- Build only two things: a one-shot "apply my setup" command, and the LCD daemon.
- Write the LCD daemon in Rust as a headless program.
- Split it into a shared part (sensors, compositing) and a per-panel part (one send function). Use a compiled-in module, not a plugin system, because Rust has no stable plugin ABI and there is one user.

## 6. LCD add-on research

Primary reference: the [TRCC Linux](https://github.com/Lexonight1/thermalright-trcc-linux) source (GPL-3.0), whose protocol notes are decompiled from Thermalright's `USBLCDNEW` binary. Nothing here has been tested against this panel.

### 6.1 Protocol for `87AD:70DB`

From [`bulk_lcd.py`](https://github.com/Lexonight1/thermalright-trcc-linux/blob/main/src/trcc/adapters/device/bulk_lcd.py) and [`PROTOCOL_USBLCDNEW.md`](https://github.com/Lexonight1/thermalright-trcc-linux/blob/main/doc/PROTOCOL_USBLCDNEW.md):

- **Transport:** raw USB bulk, endpoint `0x01` OUT and EP01 IN.
- **Handshake:** write 64 bytes (`12 34 56 78`, zeros, byte 56 = `01`), then read 1024 bytes. The reply is valid when byte 24 is non-zero. **PM = byte 24, SUB = byte 36.**
- **Panel identity:** one USB ID covers many coolers, and the handshake's PM and SUB bytes decide resolution, rotation and encoding. The default is 480x480, and 854x480 panels exist on this ID.
- **Frame header (64 bytes):** magic at 0, command at 4 (LE u32), width at 8, height at 12, value `2` at 56, payload length at 60.
- **Payload:** JPEG with command 2 for most panels. PM 32 and 50 use RGB565 with command 3, and PM 50 is big-endian.
- **Sending:** write in 16 KiB chunks. Send a zero-length packet when the total is a multiple of 512. The vendor code sleeps 15 ms after each frame.
- **Timeouts:** 1000 ms for handshake, 5000 ms for writes.

### 6.2 The panel needs continuous frames

The source states this family's firmware "reverts to the logo ~2-3 s after the last frame", and the vendor app resends every 150 ms ([`_base.py`](https://github.com/Lexonight1/thermalright-trcc-linux/blob/main/src/trcc/adapters/device/_base.py)).

- A one-shot "send image and exit" will not work, even for a static image. Something must stay resident.
- Whether a slower resend, such as 1 second, is safe is untested. It would cut wake-ups from about 7 per second to 1, so it is worth trying.

### 6.3 Why TRCC is heavy, partly

TRCC is .NET, with a separate `USBLCDNEW` process, and maps a 34,560,000-byte shared memory block (50 slots of 691,200 bytes) according to the protocol notes. The rest of its 300 MB is not explained. A headless program needs none of this.

### 6.4 Background and overlay design

Arithmetic at 480x480, the likely default. The real size comes from the handshake.

| Item | Size |
|---|---|
| One RGB frame | 691,200 bytes |
| One RGBA frame | 921,600 bytes |
| One RGB565 frame | 460,800 bytes |
| 100-frame GIF held decoded as RGBA | about 92 MB |

**Key consequence: caching decoded GIF frames would throw away the RAM win.** Hold the compressed GIF and keep one decode canvas, decoding as it plays.

| Mode | Work per update | Notes |
|---|---|---|
| Static PNG or JPEG | Composite only when a displayed number changes, at most once a second. Between updates, resend the last JPEG | Almost no CPU |
| GIF | Decode the next frame, draw the cached overlay layer, encode JPEG | CPU cost unmeasured. Cap at about 15 fps |

- Redraw the overlay layer only when its values change, not per frame.
- The panel takes whole frames only, so there are no partial updates.
- Video files are out of scope, and are a likely part of what makes TRCC heavy.
- A small config should hold the background path and each overlay item's metric, position, font size and colour. TRCC Linux's overlay model has the same three element types: text, metric and clock.
- Editing layouts needs no resident UI: a config file, or a small tool that opens and exits.

### 6.5 Sensors

| Metric | Source | Driver? |
|---|---|---|
| CPU temperature (Tctl) | SMN register `0x00059800`, bits 31:21 x 0.125 °C, minus 49 °C when the range-select bits are set ([LHM `Amd17Cpu.cs`](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/blob/master/LibreHardwareMonitorLib/Hardware/Cpu/Amd17Cpu.cs)) | PawnIO `AMDFamily17` module (LGPL-2.1, [modules](https://github.com/namazso/PawnIO.Modules)). Already running here |
| CCD temperature, Ryzen 9000 (model 0x44) | SMN base `0x00059B08`, raw x 0.125 − 305 °C (same file) | PawnIO |
| GPU temperature, load, power, fan | NVML, via [`nvml-wrapper`](https://docs.rs/nvml-wrapper/latest/nvml_wrapper/) 0.13, which loads the library at runtime | None. Temperature support comes from NVML generally, and was not stated on the crate's front page |
| CPU load, RAM use | Windows APIs | None (not researched in detail) |
| Fan or pump RPM | Super I/O | PawnIO, and unreliable on this board family. Leave out at first |

Open points: calling PawnIOLib from Rust and the exact loading steps need the [PawnIO wiki](https://github.com/namazso/PawnIO.Modules); whether admin rights are required is expected but unconfirmed. LHM's file does not list Granite Ridge as tested. HWiNFO's shared memory could replace PawnIO, but HWiNFO would then have to stay running.

### 6.6 Candidate libraries (not yet tried)

- USB: [`nusb`](https://docs.rs/nusb) 0.2.7 uses WinUSB on Windows, and this panel is already bound to WinUSB, so Zadig should not be needed.
- Images: the `image` crate for PNG, GIF and JPEG. Its RAM and JPEG speed are unmeasured.
- Text: a small font rasteriser such as `fontdue` or `ab_glyph`, not evaluated.

### 6.7 Safety rules for the future build

- Only ever send the documented handshake and picture frames.
- The protocol lists other command types: 3 Logo, 4 OTA, 5 Upgrade State, 6 Rotate, 7 Screen Settings, 8 Backlight, 9 Logo State. Never send OTA or upgrade commands.
- **Conflict to resolve:** the vendor constants name command 3 "Logo", while `bulk_lcd.py` uses command 3 for RGB565 frames on PM 32 and 50. If this panel reports PM 32 or 50, stop and investigate before sending any frame, because a frame could be stored as the boot logo.
- Validate the handshake reply and take resolution from it. Never guess.
- Only one program can own the device, so TRCC and its `TRCCAppStartup` task must stay off while the daemon runs.
- Handle sleep, resume and USB reconnect with a fresh handshake.
- If sensors fail, show "--". Never show a stale number as if it were live.

### 6.8 Licensing

TRCC Linux is GPL-3.0 and derives from a decompiled vendor binary. Personal use is fine. Copying its code would make a distributed program GPL-3.0, and publishing would deserve a closer look at the protocol's provenance. PawnIO modules are LGPL-2.1.

## 7. Proposed phases (none started)

1. **Research (current).**
2. **Owner decisions** (section 9).
3. **Armoury Crate exit prep:** finish Omni pairing, firmware updates and onboard profiles first, because new pairings need Armoury Crate. Set Q-Fan curves. Uninstall with ASUS's tool, then stress test the fans.
4. **LCD test:** handshake only, read PM and SUB, confirm resolution and command type. Then one static frame.
5. **LCD daemon:** keepalive, static background, overlay with GPU stats, then CPU temperature through PawnIO, then GIF. Measure RAM and CPU at each step.
6. **"Apply my setup" command** and logon task.
7. **Optional:** a small tray or settings tool, only if something is missing after real use.

## 8. Code review findings for the existing app (not fixed)

Tests: 78 passed. Ruff: 68 errors.

**Core and CLI, which matter if `toushou` stays**
- `ProfileStore.load` crashes on malformed JSON, which breaks every profile listing (`src/core/profiles.py`). `ExperimentalMappingStore.load` has the same problem.
- A bad register key crashes `Profile.apply` (`int(key, 16)`).
- Profile names containing `? * " < > |` raise `OSError` on Windows, and profile saves are not atomic.
- `--force` lets the CLI write factory-reset codes and power-off values, against the hard rule in `AGENTS.md` (`src/cli.py`).
- `--monitor -1` silently writes to monitor 0.
- `apply-profile` reports success even when 0 codes were written.
- Catalog ranges cap at 100 although monitors report their own maximum. `_coerce_value` drops it (`src/core/monitor_manager.py`).
- EDID names are matched by position, so names can land on the wrong monitor, and they are cached forever.
- `.claude/settings.json` runs the safety hook on Bash only, not PowerShell.

**TUI, which matter only if it stays in use (`src/ui/screens.py`)**
- After applying a profile the cache is not re-read, so the next key press writes old value + 1.
- A slow read from a previously selected monitor can overwrite the cache for the current one, and later writes use those values.
- `profile.apply` resolves the active monitor per write, so switching monitor mid-apply splits the profile.
- Input-switch confirmation is not tied to monitor or target value.
- Rapid adjustments can land out of order.
- `Profile.capture` runs on the UI thread.
- Saving under an existing name overwrites it without asking, including calibrated profiles.
- Up and Down are priority-bound, so the keyboard cannot change the selected monitor.

**To verify before relying on the VCP catalog.** From memory only, standard MCCS uses `0x87` for sharpness, `0x8D` for audio mute and screen blank, `0xB0` for settings store/restore, and `0xB2`/`0xB6` as read-only info. The catalog differs, and `0x8D` and `0xB0` may need blocking. Check against the MCCS spec or ddcutil.

**Removable:** legacy `ScannerScreen` and `ProfilesScreen`, unused manager passthroughs, the `desktop/` Tauri template and pywebview prototype if not pursued. Deletion needs confirmation, and this folder is not a git repository, so back up first.

## 9. Open questions for the owner

1. Script-and-LCD-daemon only, or a custom app with a UI?
2. AIO model. The handshake can supply resolution, but the model helps.
3. Keyboard and mouse models on the Omni receiver.
4. Which Armoury Crate features are used weekly?
5. Which stats should the overlay show?
6. Is RGB wanted at all, or is BIOS Stealth acceptable?

## 10. Not verified

- No tool or protocol here was tested on this PC.
- This panel's PM, SUB, resolution and encoding.
- Whether a resend interval slower than 150 ms is safe.
- The 10–20 MB daemon figure, and GIF CPU cost.
- PawnIO use from Rust, admin needs, and Microsoft signing status.
- Whether attestation-signed drivers or PawnIO pass the 2026 Windows driver policy. Windows 11 24H2 and later are moving to WHCP-signed drivers plus an allow list, so a future update could break PawnIO-based CPU temperature. GPU stats would be unaffected. See section 8 of the survey note.
- Laptops are out of the current scope. Section 8 of the survey note covers them from documentation only, untested on hardware.
- RGB persistence after removing Armoury Crate.

## 11. Sources

Accessed 2026-11-08.

- TRCC Linux: https://github.com/Lexonight1/thermalright-trcc-linux
- `bulk_lcd.py`: https://github.com/Lexonight1/thermalright-trcc-linux/blob/main/src/trcc/adapters/device/bulk_lcd.py
- `_base.py`: https://github.com/Lexonight1/thermalright-trcc-linux/blob/main/src/trcc/adapters/device/_base.py
- `PROTOCOL_USBLCDNEW.md`: https://github.com/Lexonight1/thermalright-trcc-linux/blob/main/doc/PROTOCOL_USBLCDNEW.md
- thermalright-lcd-control listing: https://awesome.ecosyste.ms/projects/github.com%2Frejeb%2Fthermalright-lcd-control
- LibreHardwareMonitor `Amd17Cpu.cs`: https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/blob/master/LibreHardwareMonitorLib/Hardware/Cpu/Amd17Cpu.cs
- PawnIO modules: https://github.com/namazso/PawnIO.Modules
- nusb: https://docs.rs/nusb
- nvml-wrapper: https://docs.rs/nvml-wrapper/latest/nvml_wrapper/
- Tool survey and its sources: [2026-11-08-universal-pc-control-tools.md](2026-11-08-universal-pc-control-tools.md)
