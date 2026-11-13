# SysVane plan

**Updated:** 2026-11-13
**Version:** `0.0.2` (beta); the tag `v0.0.2` is not cut yet. `1.0.0` when the app is stable on the owner's ASUS hardware, or handles universal devices well.
**Status:** M0 and M1 done. `sysvane-core` samples CPU, RAM, disks, network, processes and the NVIDIA GPU (worker), reads CPU temperature through a core-owned worker (PawnIO), shows a tray icon, serves the data pipe (protocol 1) and writes rotating logs with a diagnostics export. It runs elevated from the scheduled task "SysVane Core", installed by an NSIS installer into `Program Files\SysVane`; "Start with Windows" and uninstall were tested on the owner's PC. `sysvane-ui` has Overview, Performance, Processes and a 320x480 flyout. Not built yet: auto update (M2), command pipe and module framework (M3), and every device module (DDC, LCD, RGB, fans). `modules/ddc/` holds only profiles and notes for the Rust port.
**Evidence:** [research notes](research/). RAM and timing numbers are targets unless marked measured.

## Decisions (grilling session, 2026-11-12)

| Topic | Decision |
|---|---|
| Audience | The owner's PC now. Public and universal later, once the owner's hardware is stable. |
| Feature order | DDC, then LCD, then RGB, then fans (fans only if BIOS Q-Fan is not enough). |
| Process model | Two exes: `sysvane-ui.exe` and `sysvane-core.exe`. The core exe also plays worker roles (GPU worker, hang-prone code) through flags. |
| Privilege | The core runs as administrator. The UI runs as a normal user. All safety checks live in the core. |
| Start with Windows | A user switch, off by default, also offered in the installer. Implemented as a scheduled task "run at logon, highest privileges", created when switched on and deleted when off. A Run key cannot start an elevated exe. |
| UI stack | Keep Tauri 2 + React. The UI exists only while a window is open. Background RAM is the core's. A Rust-only UI is rejected: large rework, no gain in background RAM. |
| Versions | SemVer, one version for the whole workspace. `main` is always releasable. Release = tag `vX.Y.Z` + GitHub Release. Pre-release (beta) channel only until `1.0.0`. |
| UI to core compatibility | An integer protocol version in every snapshot, every command and every profile (`schema`). On mismatch the UI says "update needed" and sends no commands. SemVer is for people only. |
| Pipes | Two pipes. Data pipe: core to UI, read only. Command pipe: UI to core, one JSON line per command, `device_id` + `control_id` + value, access limited to the current user by ACL, validated in the core. |
| Config | One file per module, never one merged config. Defaults ship with the release (read only). The user's saved settings live in `%ProgramData%\SysVane\<module>\` and override them. Updates never touch user files. |
| Profiles | Local only. The user configures and saves them in the app. No profile downloads. New device support arrives through app updates. Profiles are data, never code, and every load is validated: the module's hard safety rules always win. |
| Logs | Rotating files in `%ProgramData%\SysVane\logs`: errors and device state changes, raw commands only in debug mode. Buttons to open the folder and export a diagnostics pack. No telemetry, ever. |
| Install scope | All users only, into `C:\Program Files\SysVane`. No per-user install: an elevated core in a user-writable folder would let a normal process replace the exe and gain administrator rights. A "just me" mode may come later with an unelevated core, without fans, some RGB and PawnIO. |
| Conflict with other suites | If Armoury Crate (or another suite that owns the same devices) is installed or running, SysVane shows a prompt on first use of a device: take control from it, or leave it. On accept SysVane stops that suite's services for the session; it never uninstalls or edits them, and it hands control back when the user asks. Without a yes, SysVane does not touch the device. |
| Code signing | None for beta: installers are signed only with the updater key, so SmartScreen warns and the owner clicks Run anyway. Buy a code-signing certificate near `1.0.0` or when other people install. |
| Fast user switching | Another Windows user logging in counts as closing the app and starting it again for that user. Nothing special. |
| Uninstall | Removes exes, scheduled task, Run key, and anything SysVane installed. Asks whether to delete `%ProgramData%\SysVane`. Removes PawnIO only if SysVane installed it (remembered), never if it was already there. |
| Test strategy | Every module has a fake device. Logic, safety rules and profiles are tested without hardware. Real-device checks are manual scripts in `docs/`. |

## Architecture

```
sysvane-core.exe (Rust, no Tauri, administrator, started by a scheduled task)
 ├─ tray icon, sensors, config, logs
 ├─ device modules (compiled in): DDC, LCD, RGB, fans; each owns its device rules,
 │   its default profile data, its user config and its fake device for tests
 ├─ watchdog per module; hang-prone or driver code in a supervised child process
 ├─ power and device-arrival events: re-scan and re-apply saved state after sleep or replug
 └─ data pipe (out) and command pipe (in), both JSON lines, ACL for the current user
sysvane-ui.exe (Tauri 2 + React, normal user): flyout and dashboard, exits when closed
```

- **`sysvane-wire`** (`wire/`) is the only code shared by both exes: pipe names, process names, protocol version, tick, snapshot and command types. `desktop/src/wire.ts` mirrors the types and a test fails if they differ.
- **Modules are compiled in.** No DLL or WASM plugins. Per device: a Rust driver for the protocol, plus data profiles for the parameters. A new device with the same protocol only needs a new profile. A universal backend (OpenRGB, VESA DDC/CI, standard HID) is one more module with its own profiles, used when it works well.
- **Write the first shared interface from real modules.** Build DDC and LCD for one device each, then extract what they share. Do not design the trait first.
- **Safety lives in the core.** The UI only sends `device_id`, `control_id` and a value. The core refuses anything unsafe, whatever the UI or a profile says.
- **One owner per bus.** A third-party tool may only run on a bus the core does not touch.
- **Sensors run on demand.** One sampler parks when no consumer holds a lease. Stale data counts as no data. Sensors can report "unavailable".
- **Module failure is contained.** A command past its deadline marks that module failed, stops sending to that device, shows the state in the UI and retries a limited number of times.
- **Release builds keep `panic = "unwind"`**, so a panic stays inside one module thread.

**Targets to measure:** core with LCD under 25 MB private, no WebView2 process after the UI closes, flyout opens in under 1 s.

**Measured (2026-11-13, release build, no LCD yet, dev folder):** core alone 1.9 MB private (12.5 MB working set). With the flyout open: core 8.1 MB, GPU worker 22.3 MB (exists only while connected), temp worker 1.5 MB, UI 6 MB plus 6 WebView2 processes of about 164 MB private in total. After the UI closes: no sysvane WebView2 process and no GPU worker, but the core stays at 7.9 MB instead of returning to 1.9: 22 DLLs (WMI, PDH, network, user APIs) stay loaded and the heap is not returned. Stable over 6 connect and close cycles, so not a leak; accepted. Flyout window visible 457 ms after launch (window shown; content paint not measured). Earlier (2026-11-09): NVML inside the core cost about 20 MB that was never returned, hence the worker.

## Tech stack

| Part | Choice | Status |
|---|---|---|
| UI shell | Tauri 2 in `desktop/` | Working |
| Frontend | React 19, Vite 8, TypeScript 6; CSS variables from `DESIGN.md`, bundled Inter | Working |
| Core | Rust 2024, `std::thread` and channels, no async runtime | Working |
| Sensors | `sysinfo`, `nvml-wrapper` in a worker, PawnIO for AMD CPU temperature | Working. ADL untried |
| Tray | `tray-icon` | Built |
| Installer | Tauri bundler, NSIS, into `C:\Program Files\SysVane`, asks for administrator once | Working (M1) |
| Auto update | `tauri-plugin-updater` in the UI only, signed `latest.json` on GitHub Releases | Planned (M2) |
| Config and profiles | TOML with `serde`, one set per module | Planned (M3) |
| LCD | `nusb`, `image`, `fontdue` | Untried, protocol from source only |
| DDC/CI | `ddc-winapi` or direct Win32 | Untried, decide when porting |

## Design system

`DESIGN.md` describes a marketing site, so the app adapts it: bundled Inter 13 to 14 px, canvas `#ffffff` and `#fafafa`, ink `#171717`, emerald `#3ecf8e` only for the primary action and the active state, derived dark theme on `#1c1c1c`, yellow and red for temperature warnings only. UI safety: input switching goes through ConfirmDialog, and there is no factory-reset or display-off control.

## Milestones

Each milestone ends with something you can run and check. Order matters: M1 and M2 come first so every later build reaches your PC the same way.

**M0 Monitoring (done, measure left)**
- Done: sensors, tray, pipe, dashboard, flyout, CPU temperature helper, wire crate.
- Measured 2026-11-13, see Measured below. Done.

**M1 Install, privilege, lifecycle (`0.0.2`) — done 2026-11-13**
- Verified 2026-11-13 on the owner's PC: silent install, in-place reinstall as manual update, core elevated from the task, exe not writable by Users, clean uninstall (default No on data), reboot with "Start with Windows". Version is `0.0.2`; the tag is not cut.
- Accepted by the owner with two gaps: PawnIO redistribution licence unchecked; uninstall "delete data = Yes" and "remove PawnIO we installed" branches read from code only, never run.
- NSIS installer into `Program Files`, shipping `sysvane-core.exe` next to the UI.
- Core runs as administrator through a scheduled task; remove the separate temperature helper and its task.
- "Start with Windows" switch (default off) creating and deleting the logon task.
- PawnIO: the installer detects it. If missing, install it and record that SysVane did so; if already present, skip the install and never remove it. The uninstaller removes PawnIO only when SysVane installed it. Check the PawnIO licence for redistribution before shipping.
- Data pipe ACL for the current user; protocol version in snapshot and UI mismatch message.
- Logging and the diagnostics pack; clean uninstall with the "delete data" option.
- Done when: install, reboot, run, update by hand, uninstall leave no trace, and a normal process cannot overwrite the core exe.

**M2 Auto update, beta channel (`0.0.3`)**
- Updater key pair; the private key stays off the repo and is backed up.
- GitHub Actions: tag `vX.Y.Z` builds the installer, signs it, publishes a pre-release and `latest.json`.
- UI checks at dashboard start and daily while open, shows version and notes, asks before installing.
- Installer stops the core and any worker, replaces files, starts the core again. A failed update leaves the old version working.
- Done when: a tag produces a release and your installed copy updates itself without uninstalling.

**M3 Command channel and module framework (`0.0.4`)**
- Command pipe with ACL, validation, protocol version.
- Module registry (static list), per-module config and profile loader with validation and `schema`.
- Watchdog, failed-module state shown in the UI, re-apply after sleep and device change.
- Fake device harness used by every later module.
- Done when: a fake module can be controlled from the UI, survives a simulated hang and a simulated resume.

**M4 DDC module (`0.1.0`)**
- Port from commit `30b7852`: `vcp_catalog.py`, `profiles.py`, `monitor_manager.py` and the safety tests. Profiles stay in `modules/ddc/profiles/` as module defaults.
- One guarded write function; the rules below; fix the known EDID-by-position issue.
- UI: Displays page with brightness, contrast, colour per monitor, apply and save profile.
- Done when: the two Acer AL258Q monitors take the calibrated profile from the app, and no test can write a refused code.

**M5 LCD module (`0.2.0`)**
- Device: Thermalright Levita Vision 360 ARGB (AIO with LCD). UI and features follow the TRCC help file (`LCDHelp.pdf`):
  - Information module: live values in cards (CPU, GPU, memory, disk, network, fan: temperature, usage, clock, power, rates), with editable names. SysVane already samples most of these.
  - Themes: a theme is a background (image, video or built-in), an optional transparent PNG mask, and cards (value, text, clock, date, weekday) with position, font and colour.
  - Preview box mirrors the screen; drag cards, nudge with WASD (1 px) and Shift+WASD (10 px).
  - Display angle, brightness, save as local theme, export and import theme files, carousel of themes with a time interval.
  - Left out on purpose: the online theme and gallery download (profiles are local only). Ship a few built-in themes instead.
- Needs the Levita Vision USB protocol first (from source or capture only; not verified). 150 ms resend that never blocks on a sensor. Decide whether WinUSB calls can hang, which decides whether the LCD runs in a supervised child.
- Never send OTA or firmware commands.

**M6 RGB module (`0.3.0`)**
- Devices: ROG Azoth keyboard, ROG Keris II Ace mouse, Corsair Vengeance RGB memory, plus the AIO and case fans' lighting.
- One backend decision per device after listing what each exposes: `OpenRGB --profile` as a third-party tool, or per-device HID. Armoury Crate (Aura) and iCUE currently own these buses and must be stopped before SysVane touches them.
- Saved colours re-applied after sleep.

**M7 Fan module (`0.4.0`)**
- Scope: the AIO fans and pump, and the case and cooler fans on the motherboard headers.
- Only if BIOS Q-Fan is not enough. Super I/O writes are the riskiest part of the project; it needs its own safety review and a hard floor on fan and pump speed before any code.

**M8 Universal and stable (`1.0.0`)**
- Universal backends where they work well, a second device per module as proof, documentation for adding a device, signed releases.
- Stable channel appears with `1.0.0`. Before that, beta only.

Until M4 lands SysVane cannot control monitors, including the calibration profiles.

## DDC behaviour to carry into Rust

- One guarded write function. It refuses factory-reset codes `0x04/05/06/08` and power-off values on `0xD6/0xC6`. No flag unlocks those.
- Input `0x60` needs an explicit per-call confirmation.
- Malformed profile entries are skipped. Corrupt files do not crash. Saves are atomic.
- A negative or out-of-range monitor index is an error. Applying a profile that writes nothing is an error.
- Filenames strip Windows-illegal characters.
- CVTE scalers: write `0x14 = 11` before `0x16/0x18/0x1A`.
- Known issue to fix in the port: EDID names were matched to monitors by position, so labels could be wrong.

## Open questions

Ask the owner before starting the milestone named.
- M1 (done): the installer installs PawnIO when missing and skips it when present. Still to check before a public release: its redistribution licence.
- M2: decided: public repository, so the updater needs no token.
- M5: how the Levita Vision talks over USB, and which built-in themes to ship.
- M7: which Super I/O chip the motherboard uses, and which headers the fans use.
- Which Armoury Crate features are used weekly? Do Omni pairing and firmware updates before uninstalling Armoury Crate.

## Not verified

- Whether WinUSB calls can hang past their timeout (decides the LCD process model).
- Whether the Windows driver policy keeps allowing PawnIO.
- Whether device-arrival notifications fire for `87AD:70DB`.
- `nvmlDeviceSetFanSpeed_v2` support on Windows.
- That the installer can stop the elevated core and replace files during an update with one UAC prompt, and that the updater can drive a `Program Files` install.
- Whether an unsigned installer is blocked by SmartScreen on the owner's PC.
- Flyout cold-start time with a separate UI process.
