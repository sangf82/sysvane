# Lowest-RAM React stack for a Windows DDC/CI monitor controller

**Date:** 2026-11-04
**Scope:** Windows-first. “Universal” here means **all monitors attached to one Windows PC**, not all operating systems. No stack in this note gives DDC/CI or ambient-light sensing on macOS/Linux without separate native backends.

## 1. Question and constraints

The existing app is a Python `monitorcontrol` core with a Textual TUI, a Typer CLI, and a pywebview desktop prototype ([README](../../README.md)). It already detects monitors, scans VCP capabilities, applies JSON profiles, and enforces hard safety rules for factory-reset, power-off, and input-source writes ([AGENTS.md](../../AGENTS.md), [VCP_REFERENCE.md](../../VCP_REFERENCE.md)).

The rewrite decision is: **lowest RAM while keeping a React UI**, on Windows, with:

- auto-detect monitors;
- auto-scan VCP capabilities and current values;
- adaptive brightness (ambient-light sensor or content-based);
- per-monitor mode profiles (save/apply);
- safe writes (never power off, factory-reset, or black-screen via input switching).

Two different “lowest RAM” questions matter:

1. **Idle/background RAM** — the app runs all day as a brightness/profile daemon. This is the common case for this app.
2. **UI-open RAM** — the React window is visible. This is dominated by the webview engine, not the shell.

The recommendation below is optimized for the first, and is honest that the second is not decisively won by any stack.

## 2. Comparison table

**How to read the RAM numbers.** I found no official per-app Windows RAM figure from any of these vendors. The numbers marked “measured” are from the closest reproducible public benchmark I found, the Elanis web-to-desktop comparison ([repo](https://github.com/Elanis/web-to-desktop-framework-comparison), [README](https://raw.githubusercontent.com/Elanis/web-to-desktop-framework-comparison/main/README.md)). It measures an **empty app on GitHub CI, Windows x64, release build**, reports the **“Median of used memory for main process and children ones”** and the **“Median of difference between system measured free memory before execution and during execution.”** It does not define RSS/shared-page accounting, and the README gives no run date or versions; the manifests show Tauri 2 ([Cargo.toml](https://raw.githubusercontent.com/Elanis/web-to-desktop-framework-comparison/main/benchmark/01-empty-app/tauri/src-tauri/Cargo.toml)), Wails v2.17.1 ([go.mod](https://raw.githubusercontent.com/Elanis/web-to-desktop-framework-comparison/main/benchmark/01-empty-app/wails/go.mod)), Electron ^45.1.0 ([package.json](https://raw.githubusercontent.com/Elanis/web-to-desktop-framework-comparison/main/benchmark/01-empty-app/electron/package.json)), and Neutralino 6.10.0 ([config](https://raw.githubusercontent.com/Elanis/web-to-desktop-framework-comparison/main/benchmark/01-empty-app/neutralinojs/neutralino.config.json)). Treat these as **directional measurements, not official specs**.

| Stack | Backend language | React support | RAM expectation (measured / estimated / source) | Installer / build size | WebView dependency | DDC/CI access | Auto-detect / scan fit | Adaptive brightness fit | Profiles fit |
|---|---|---|---|---|---|---|---|---|---|
| **Tauri 2** | Rust | Official React template in `create-tauri-app` ([docs](https://v2.tauri.app/start/create-project/)) | **Measured:** ~312 MB process tree / ~187 MB system-free delta, empty app, Win x64 release ([bench](https://github.com/Elanis/web-to-desktop-framework-comparison)). **No official Tauri RAM figure found.** | **Measured:** ~3 MB empty build ([bench](https://github.com/Elanis/web-to-desktop-framework-comparison)). Installer adds 0 MB (download bootstrapper), ~1.8 MB (embed), ~127 MB (offline), or ~180 MB (fixed WebView2) ([Tauri](https://v2.tauri.app/distribute/windows-installer/)). | WebView2, system Evergreen; Windows 11 includes it ([Tauri prereqs](https://v2.tauri.app/start/prerequisites/), [Microsoft](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution)). | Rust `windows` crate `Win32::Devices::Display` + `ddc-winapi` / `ddc-hi` ([windows-rs](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Devices/Display/fn.GetPhysicalMonitorsFromHMONITOR.html), [ddc-winapi](https://docs.rs/ddc-winapi/latest/ddc_winapi/), [ddc-hi](https://docs.rs/ddc-hi/latest/ddc_hi/)). | **Best.** Async Tauri commands run off the main thread; Rust owns Win32 enumeration/scan ([Tauri commands](https://v2.tauri.app/develop/calling-rust/)). | **Best.** WinRT `LightSensor` via `windows` crate, plus Desktop Duplication fallback; DDC brightness is direct ([windows-rs](https://microsoft.github.io/windows-docs-rs/doc/windows/Devices/Sensors/struct.LightSensor.html), [Microsoft](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api)). | JSON via `serde_json` or Tauri Store; SQLite via Tauri SQL plugin ([Store](https://v2.tauri.app/plugin/store/), [SQL](https://v2.tauri.app/plugin/sql/)). |
| **Wails v3** | Go | React starter templates ([Wails intro](https://wails.io/docs/introduction)); v3 stable since Oct 2026 ([releases](https://github.com/wailsapp/wails/releases)). | **Measured:** ~330 MB process tree / ~193 MB delta for v2.17.1 ([bench](https://github.com/Elanis/web-to-desktop-framework-comparison)); v3 uses the same WebView2 architecture. **No official number**; Wails says only “small runtime.” | **Measured:** ~11 MB empty build ([bench](https://github.com/Elanis/web-to-desktop-framework-comparison)); single executable; WebView2 required. | WebView2, system; Wails calls it “the new Microsoft Webview2 library, built on Chromium” ([Wails intro](https://wails.io/docs/introduction)). | Go can load `dxva2.dll` through `golang.org/x/sys/windows` `LazyDLL` and call the Win32 exports ([Go docs](https://pkg.go.dev/golang.org/x/sys/windows#LazyDLL)); no official Go DDC/CI wrapper found. | **Good, but manual.** Goroutines help, but every struct/signature is hand-written via syscall/cgo. | **Harder.** Go can call WinRT/COM and DXGI, but there is no first-party sensor/capture binding; more hand-written COM. | Go `encoding/json` plus a SQLite driver; fine. |
| **Electron** | Node.js / TypeScript (+ native addon) | Electron Forge Vite template plus Vite’s official React plugin ([Forge](https://www.electronforge.io/templates), [Vite React](https://github.com/vitejs/vite-plugin-react)). | **Measured:** ~279 MB process tree / ~110 MB delta ([bench](https://github.com/Elanis/web-to-desktop-framework-comparison)). **No official number**; Electron documents its multi-process model but publishes no RAM figure ([process model](https://www.electronjs.org/docs/latest/tutorial/process-model)). | **Measured:** ~373 MB unpacked build ([bench](https://github.com/Elanis/web-to-desktop-framework-comparison)); bundles Chromium and Node, so no WebView dependency ([Electron docs](https://www.electronjs.org/docs/latest/)). | None; Chromium and Node are embedded in the app ([Electron docs](https://www.electronjs.org/docs/latest/)). | Native Node-API addon or a sidecar process; Electron has no built-in DDC/CI ([Node-API](https://nodejs.org/api/n-api.html)). | **Workable, extra bridge.** All Win32 calls live in a native addon; async work can run in the main process or a worker. | **Workable, most integration code.** Addon/sidecar for WinRT ALS and Desktop Duplication; same DDC layer as others. | Node `fs`/SQLite; fine. |
| **Neutralinojs** | C++ core + JS/TS app | Community React template `codezri/neutralinojs-react` ([README](https://github.com/neutralinojs/neutralinojs), [template](https://github.com/codezri/neutralinojs-react)). | **Measured:** ~502 MB process tree / ~327 MB delta ([bench](https://github.com/Elanis/web-to-desktop-framework-comparison)). **No official number**; only “Less resource intensive compared to chromium-node-based frameworks” ([docs](https://neutralino.js.org/docs/)). | **Measured:** ~2 MB empty build ([bench](https://github.com/Elanis/web-to-desktop-framework-comparison)); WebView2 required. | WebView2 via the bundled `lib/webview` library ([repo](https://github.com/neutralinojs/neutralinojs), [webview](https://github.com/webview/webview)); “chrome” mode launches installed Edge/Chrome instead ([chrome.cpp](https://github.com/neutralinojs/neutralinojs/blob/main/chrome.cpp)). | Native extension or sidecar; no built-in DDC/CI ([extensions loader](https://github.com/neutralinojs/neutralinojs/blob/main/extensions_loader.cpp)). | **Workable, but adds a native extension** on top of the static server + WebSocket architecture ([docs](https://neutralino.js.org/docs/)). | **Workable, native extension required.** | Filesystem API or extension; fine. |
| **Direct WebView2 host** (Rust/C++/C#) | Rust, C++, or C# | Any React/Vite build; you own the bridge via `postMessage`/host objects ([Microsoft Win32 sample](https://learn.microsoft.com/en-us/microsoft-edge/webview2/get-started/win32)). | **Estimate, unverified:** same WebView2 process-group floor as Tauri/Wails when a window is open; a Rust/C++ host adds little, a .NET/WinForms/WPF host adds CLR overhead. No official or third-party per-app number found. | Host binary size depends on language (unverified); WebView2 bootstrapper ~2 MB, or fixed runtime **over 250 MB** ([Microsoft](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution)). | WebView2, system Evergreen ([Microsoft](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution)). | Native host: Rust `windows`/`webview2-com`, C++ `Dxva2.lib`, or C# P/Invoke ([webview2-com](https://docs.rs/webview2-com/latest/webview2_com/)). | **Best control; you build everything** (enumeration, scan, tray, updates). | **Best control; you own WinRT/DXGI integration.** | You own JSON/SQLite and the schema. |
| **React Native Windows** | C++/C# native modules + TS | **React Native, not React DOM**; native Fabric UI, not a WebView ([RNW](https://microsoft.github.io/react-native-windows/docs/getting-started)). | **Estimate, unverified:** no WebView2 process group and native UI should be lower than the webview stacks, but no official or third-party Windows RAM number found. | APPX package; size unverified. | None; native UI ([RNW](https://microsoft.github.io/react-native-windows/docs/getting-started)). | Native module in C++/C# calling `dxva2.dll` ([RNW native modules](https://microsoft.github.io/react-native-windows/docs/native-platform)). | **Good, native module required.** | **Good, native module required.** | Native module / storage; fine. |

**Out of scope:** GPUI ([Zed](https://github.com/zed-industries/zed)), egui ([egui](https://github.com/emilk/egui)), and Slint ([Slint](https://github.com/slint-ui/slint)) are Rust-native UI toolkits with no React/DOM support.

### Reading the RAM numbers

- The benchmark’s Windows result is **not** the usual “Tauri is much lighter than Electron” story: Electron measured ~279 MB and Tauri ~312 MB with an empty window open. The system-free delta also favors Electron (~110 MB vs ~187 MB). Tauri is far lighter on macOS/Linux in the same benchmark (~96 MB / ~94 MB), so the Windows result is specific to WebView2 and CI measurement.
- The benchmark does not define shared-page accounting, so summing process RSS can double-count shared WebView2/Chromium pages. The system-free delta is a second metric that is less dependent on process accounting, but it is also noisy on CI. Use these numbers to rank, not to promise.
- The apps are empty. A real React dashboard with tables, charts, and event streams adds renderer memory.
- Therefore: **no stack wins the “UI-open Windows RAM” contest decisively from public data.** The decisive difference for this app is whether the webview is running at all.

### WebView2 is system-shared, but per-app RAM is not automatically shared

- The Evergreen WebView2 Runtime is included in Windows 11 and present on the vast majority of Windows 10 devices; all Evergreen apps use a shared copy on disk, and Edge/WebView2 binaries are hard-linked when on the same version, which Microsoft says benefits “disk footprint, memory, and performance” ([Microsoft distribution](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution)).
- However, the process model is a **WebView2 process group per user data folder**: one browser process, one or more renderer processes, plus GPU/audio/utility processes ([Microsoft process model](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/process-model)). A process group is tied to a user data folder; apps can share a user data folder, but Microsoft warns about performance and management implications. **Do not assume that installing one runtime makes all WebView2 apps share one app’s RAM.** The runtime files are shared; the processes are not free.
- For a tray-first app, this is an advantage: if no WebView2 window exists, there is no process group to pay for. Verify that your shell actually tears down the controller/window (see Risks).

### Backend DDC/CI path, briefly

All six stacks can reach the same Windows monitor API; the difference is how much native glue you write.

- **Tauri:** `windows` crate or `ddc-winapi`/`ddc-hi` — enumeration, VCP read, VCP write, capabilities string, all first-class Rust calls.
- **Wails:** Go `LazyDLL` on `dxva2.dll` — the same five calls, but every signature/struct is hand-written syscall/cgo.
- **Electron:** Node-API addon or sidecar — the same five calls in native code; no built-in DDC/CI.
- **Neutralino:** native extension or sidecar — the same five calls in native code.
- **Direct WebView2 host:** native host code — the same five calls; you own the event loop and bridge.
- **React Native Windows:** native module in C++/C# — the same five calls.

### Official vs measured claims

- **Tauri:** official docs describe the system-webview architecture ([start](https://v2.tauri.app/start/)) and require WebView2 on Windows ([prerequisites](https://v2.tauri.app/start/prerequisites/)); no official per-app RAM figure was found in those docs.
- **Wails:** official docs say it “does not embed a browser, so it delivers a small runtime” ([intro](https://wails.io/docs/introduction)), but give **no number**.
- **Electron:** official docs describe main/renderer/utility processes ([process model](https://www.electronjs.org/docs/latest/tutorial/process-model)) but give **no RAM number**.
- **Neutralino:** official docs claim “less resource intensive” than Chromium/Node frameworks ([docs](https://neutralino.js.org/docs/)) but give **no number**; the benchmark’s ~502 MB is the worst measured result here.
- **Direct WebView2 / RNW:** no official per-app RAM figure found; both entries are estimates and marked unverified.

## 3. Recommendation and runner-up

### Recommendation: Tauri 2 + React/Vite, tray-first with an on-demand webview window

This is the best fit for the app and the lowest **practical** RAM, for five reasons:

1. **Idle cost is the Rust backend only.** Keep no window in `tauri.conf.json`; create the React window lazily with `WebviewWindowBuilder` when the user opens it ([builder docs](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html)). Tauri’s tray API and runtime window builder allow a tray-first app with no window until the user opens one ([Tauri tray](https://v2.tauri.app/learn/system-tray/)). No WebView2 process group should exist while the window is closed (expected from the process model; verify on target hardware).
2. **Rust is the shortest path to the Windows APIs this app needs.** The Microsoft `windows` crate exposes the monitor APIs directly in `Win32::Devices::Display`: `GetPhysicalMonitorsFromHMONITOR`, `GetVCPFeatureAndVCPFeatureReply`, `SetVCPFeature`, `GetCapabilitiesStringLength`, `DestroyPhysicalMonitors` ([windows-rs](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Devices/Display/fn.GetPhysicalMonitorsFromHMONITOR.html)). The `ddc-winapi` and `ddc-hi` crates wrap the same Windows monitor API for higher-level use ([ddc-winapi](https://docs.rs/ddc-winapi/latest/ddc_winapi/), [ddc-hi](https://docs.rs/ddc-hi/latest/ddc_hi/)).
3. **One language for DDC/CI, ALS, and content sampling.** `windows` also exposes WinRT `LightSensor` ([windows-rs](https://microsoft.github.io/windows-docs-rs/doc/windows/Devices/Sensors/struct.LightSensor.html)), and `windows`/DXGI can reach Desktop Duplication for the content-based fallback ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api)). No second runtime or native addon is needed.
4. **Small install, patched webview.** Tauri measured ~3 MB for an empty build vs ~373 MB for Electron ([benchmark](https://github.com/Elanis/web-to-desktop-framework-comparison)); the default installer downloads the ~2 MB WebView2 bootstrapper only if the runtime is missing, and Windows 11 already includes it ([Tauri installer](https://v2.tauri.app/distribute/windows-installer/), [Microsoft](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution)). WebView2 security patches come with the Evergreen runtime, not your installer.
5. **Safety is easier to enforce in one typed backend.** Every write path (UI, tray, profile apply, scanner, adaptive loop) can go through one Rust `SafetyPolicy` + `MonitorHandle` layer. The existing rules from [AGENTS.md](../../AGENTS.md) map directly: block factory-reset codes, block power-off values, require explicit confirmation for input-source `0x60`, read before write, and never guess a current value.

**The honest caveat:** if the user opens the React window, Tauri is **not** the measured RAM winner on Windows in the only public benchmark (Electron ~279 MB vs Tauri ~312 MB). That gap is small relative to the benchmark’s unknown error bars and comes with a ~373 MB unpacked build for Electron. **Do not choose Electron on that one number without measuring the real dashboard on the target PC.** For an always-on monitor daemon, the idle design matters more than the window-open number.

### Runner-up: Wails v3 (Go)

Wails v3 is a credible runner-up if the team prefers Go: it is stable as of October 2026 ([releases](https://github.com/wailsapp/wails/releases)), has React starter templates and a single executable ([intro](https://wails.io/docs/introduction)), and uses the same WebView2 runtime. Its measured Windows RAM (~330 MB) is in the same band as Tauri, and its build is ~11 MB ([benchmark](https://github.com/Elanis/web-to-desktop-framework-comparison)). The cost is that Go must call `dxva2.dll` through `LazyDLL`/cgo and hand-write every Win32/WinRT signature ([Go docs](https://pkg.go.dev/golang.org/x/sys/windows#LazyDLL)); there is no Rust-style first-party binding. Choose Wails only if Go expertise clearly outweighs that extra FFI work.

**Direct WebView2 host** is the “minimum layers” variant, not the recommendation: it removes Tauri’s runtime and IPC, but the WebView2 process group still dominates RAM, no public measurement shows a meaningful win, and you must build tray, IPC, updates, packaging, and security yourself. **Electron** is not recommended for this app: it bundles Chromium/Node (~373 MB build), still needs a native addon for DDC/CI, and has no RAM advantage that survives the benchmark caveats. **Neutralino** is not recommended: its ~2 MB binary is attractive, but it measured ~502 MB RAM on Windows and needs a native extension for DDC/CI. **RNW** is only relevant if you accept a native React Native UI rewrite; it is not a drop-in for the existing React web UI and has no verified RAM number.

## 4. Architecture sketch for Tauri 2

```text
Tauri app (Rust, no window at startup)
├─ Tray menu + optional global hotkey
│    ├─ Brightness up/down (active monitor)
│    ├─ Apply profile
│    └─ Open React window (lazy WebviewWindowBuilder)
├─ Tauri command layer (async)
│    ├─ list_monitors() -> Vec<MonitorInfo>          # fast: EDID/name/port
│    ├─ scan_monitor(id) -> ScanResult               # slow: capabilities + values, emits progress
│    ├─ get_value(id, code) -> VcpValue
│    ├─ set_value(id, code, value, confirm_token) -> WriteOutcome
│    ├─ save_profile(name, monitor_ids) / apply_profile(name, opts)
│    └─ adaptive_status() / set_adaptive(config)
├─ core::monitor
│    ├─ EnumDisplayMonitors -> HMONITOR list
│    ├─ GetNumberOfPhysicalMonitorsFromHMONITOR
│    ├─ GetPhysicalMonitorsFromHMONITOR -> PHYSICAL_MONITOR handles
│    └─ RAII wrapper: Drop -> DestroyPhysicalMonitors
├─ core::ddc
│    ├─ read_vcp: GetVCPFeatureAndVCPFeatureReply (~40 ms)
│    ├─ write_vcp: SetVCPFeature (~50 ms)
│    ├─ capabilities: GetCapabilitiesStringLength + CapabilitiesRequestAndCapabilitiesReply
│    └─ per-monitor serialization (one queue/mutex per physical monitor)
├─ core::safety
│    ├─ allowlist: only codes advertised in the capabilities string
│    ├─ read-before-write; clamp to the capability range (or [0, max] for continuous codes)
│    ├─ block factory-reset 0x04/0x05/0x06/0x08
│    ├─ block power-off/standby values for 0xD6/0xC6
│    ├─ input 0x60 only with explicit confirmation token
│    └─ log {monitor, code, old, new, result}; best-effort restore on failure
├─ core::scanner
│    ├─ parse ASCII capabilities string -> supported VCP codes/ranges
│    ├─ read each supported code on a worker; emit progress events
│    └─ cache per monitor identity (EDID + port/device path), invalidate on display change
├─ core::profiles
│    ├─ JSON schema v1 in %APPDATA%/.../profiles/*.json
│    ├─ atomic write: temp file + rename
│    └─ optional SQLite only for scan history, not for a handful of profiles
└─ core::adaptive
     ├─ WinRT LightSensor.GetDefault(); null -> no ALS
     ├─ ReadingChanged / ReportInterval; map lux -> per-monitor brightness curve
     ├─ fallback: Desktop Duplication average luminance (low rate, user opt-in)
     ├─ hysteresis + minimum write interval; pause while user drags sliders
     └─ apply via SetMonitorBrightness or SetVCPFeature(0x10)
```

**React UI.** Keep the existing dashboard shape: monitor list, controls table, profiles, adaptive toggle. The UI only calls `invoke` and listens to events; it never calls DDC directly. The tray menu can apply the last profile and adjust brightness without opening the window.

**Backend commands and threading.** Tauri async commands run on a separate async task ([Tauri commands](https://v2.tauri.app/develop/calling-rust/)); nevertheless, Win32 DDC calls are blocking, so put them on a dedicated DDC worker thread (or `spawn_blocking`) rather than blocking the async executor. Serialize all commands to the same physical monitor: DDC/CI is a slow I2C transaction, and overlapping reads/writes on one monitor are a reliability risk. Emit scan progress events so the UI can show the same staged detection the current app uses.

**DDC/CI layer.** Enumerate with `EnumDisplayMonitors` ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaymonitors)); note it includes pseudo-monitors, and one `HMONITOR` can map to more than one physical monitor ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/physicalmonitorenumerationapi/nf-physicalmonitorenumerationapi-getphysicalmonitorsfromhmonitor)). Always destroy physical monitor handles ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/physicalmonitorenumerationapi/nf-physicalmonitorenumerationapi-destroyphysicalmonitors)). VCP reads return current + maximum but no minimum; for brightness prefer `GetMonitorBrightness`/`SetMonitorBrightness`, which take a physical monitor handle, with `GetMonitorBrightness` returning min/current/max ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/highlevelmonitorconfigurationapi/nf-highlevelmonitorconfigurationapi-getmonitorbrightness), [Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/highlevelmonitorconfigurationapi/nf-highlevelmonitorconfigurationapi-setmonitorbrightness)). Microsoft’s warning is important: many monitors do not fully implement MCCS, so arbitrary VCP writes can produce undefined behavior; validate physically ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/lowlevelmonitorconfigurationapi/nf-lowlevelmonitorconfigurationapi-setvcpfeature)).

**Safety guards.** Keep the existing rules as a single Rust module, not as UI checks. A write should require: a successful current-value read, a supported code from the capabilities string, a value within the reported range, and a confirmation token for power/input. Hide factory-reset and power-off from the UI entirely. Debounce slider writes (e.g. write on release, or at most every 100–200 ms) because each `SetVCPFeature` takes about 50 ms ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/lowlevelmonitorconfigurationapi/nf-lowlevelmonitorconfigurationapi-setvcpfeature)). Log every write with the previous value and attempt a best-effort restore on failure; DDC/CI has no transaction or undo.

**Profile store.** Start with JSON: it matches the existing `profiles/*.json`, is human-editable, and is trivial to back up. Tauri’s Store plugin is a JSON key-value store ([Store](https://v2.tauri.app/plugin/store/)); SQLite is available through the SQL plugin ([SQL](https://v2.tauri.app/plugin/sql/)). Use SQLite only if you need scan history or queries over many profiles; for a handful of per-monitor profiles, JSON plus atomic temp-file/rename is the smaller complete change. Key profiles by EDID/device path, not by `HMONITOR` (a runtime handle). Keep the existing CVTE apply order: color preset `0x14 = 11` before RGB gains `0x16`/`0x18`/`0x1A`, because the scaler locks the gain DAC while a factory preset is active ([AGENTS.md](../../AGENTS.md)).

**Auto-detect and auto-scan.** Enumerate at startup, then listen for `WM_DISPLAYCHANGE`; Microsoft notes it is sent only to top-level windows ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/gdi/wm-displaychange)). A tray-only app with no window either keeps a hidden top-level window for the message or polls every few seconds as a fallback. Do not rescan every launch: `GetCapabilitiesStringLength` “usually returns quickly, but sometimes it can take several seconds” ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/lowlevelmonitorconfigurationapi/nf-lowlevelmonitorconfigurationapi-getcapabilitiesstringlength)), and each VCP read is ~40 ms ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/lowlevelmonitorconfigurationapi/nf-lowlevelmonitorconfigurationapi-getvcpfeatureandvcpfeaturereply)). For two monitors × 30 supported codes, the floor is ~2.4 s before capability-string latency; cache the result per monitor and invalidate only on display change or explicit refresh.

**Adaptive brightness.** Use the WinRT `LightSensor` class for ambient lux; `GetDefault()` returns **null if no integrated light sensor is found** ([Microsoft](https://learn.microsoft.com/en-us/uwp/api/windows.devices.sensors.lightsensor)). The older COM Sensor API is deprecated in favor of the UWP Sensor API ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/sensorsapi/portal)). Desktop apps can call WinRT APIs, subject to a documented unsupported/package-identity list ([Microsoft](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/desktop-to-uwp-enhance), [support list](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/winrt-api-desktop-app-support)); `LightSensor` is not on that list, but test on the target hardware. The sensor reports one system ambient reading, not one per monitor; DDC/CI brightness is per physical monitor ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/highlevelmonitorconfigurationapi/nf-highlevelmonitorconfigurationapi-setmonitorbrightness)). If no ALS exists, fall back to content-based luminance sampling via Desktop Duplication ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api)), a time-of-day curve, or manual profiles. Use hysteresis and a minimum interval to avoid a stream of 50 ms DDC writes.

## 5. Risks and unknowns

- **The RAM benchmark is not a spec.** It is a third-party empty-app CI benchmark with no run date or versions in the README, and undefined shared-page accounting. The Windows result contradicts the usual Tauri-vs-Electron narrative. Measure the real React dashboard on the target PC, both idle and window-open, before committing to a rewrite.
- **Tauri’s idle memory is unverified.** The design assumes no window means no WebView2 process group, and that destroying the window releases the controller. Microsoft documents `CoreWebView2Controller.Close` to shut down associated browser/renderer processes ([process model](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/process-model)), but Tauri’s window lifecycle should be measured with Task Manager. If the WebView2 process persists, the idle advantage shrinks.
- **WebView2 updates and enterprise policy.** The Evergreen runtime updates independently; enterprise admins can freeze it. Fixed-version deployment adds ~180 MB (Tauri) or over 250 MB (Microsoft) to the installer ([Tauri](https://v2.tauri.app/distribute/windows-installer/), [Microsoft](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution)).
- **Ambient-light hardware may be absent.** `LightSensor.GetDefault()` returns null when no integrated sensor is found. The content-based fallback needs screen capture, which has privacy and performance costs, and may fail on protected/secure content. Windows exposes `LightSensor` as a system sensor; there is no documented per-monitor ALS association for external monitors.
- **DDC/CI is unreliable by nature.** Microsoft warns that many monitors do not fully implement MCCS and that arbitrary writes can produce undefined behavior ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/lowlevelmonitorconfigurationapi/nf-lowlevelmonitorconfigurationapi-setvcpfeature)). A write can black out the display, and there is no undo. Keep the current app’s hard rules and hardware validation.
- **Monitor identity across ports.** `HMONITOR` is a runtime handle, not a stable profile key. EDID may be missing, duplicated, or changed by a KVM/port switch. Profiles need a fallback key (device path + EDID hash + position) and a clear “apply to which monitor?” confirmation.
- **Wails v3 is newly stable.** v3.0.0 shipped 26 Oct 2026 and v3.1.0 on 01 Nov 2026 ([releases](https://github.com/wailsapp/wails/releases)); verify templates, plugins, and third-party DDC/CI code before committing. Go’s Win32/WinRT calls are manual.
- **Direct WebView2 host and RNW have no measured RAM.** Both entries are estimates. Direct host shifts a lot of engineering (tray, IPC, updates, packaging, security) onto you; RNW requires rewriting the React web UI as React Native components and writing a native DDC module.
- **Cross-OS “universal” is not solved.** Tauri/Wails are cross-platform shells, but DDC/CI, ALS, and Desktop Duplication here are Windows APIs. A macOS/Linux port needs separate backends.
- **Neutralino’s binary size is misleading for RAM.** ~2 MB build but ~502 MB measured process tree; the static server + WebSocket architecture and WebView2 dominate. Do not pick it on binary size alone.

## 6. Sources

All accessed **2026-11-04**.

**Tauri 2**
- What Tauri is / system webview: https://v2.tauri.app/start/
- React template: https://v2.tauri.app/start/create-project/
- Windows prerequisites / WebView2: https://v2.tauri.app/start/prerequisites/
- Windows installer / WebView2 modes: https://v2.tauri.app/distribute/windows-installer/
- Commands / async: https://v2.tauri.app/develop/calling-rust/
- System tray: https://v2.tauri.app/learn/system-tray/
- Store plugin: https://v2.tauri.app/plugin/store/
- SQL plugin: https://v2.tauri.app/plugin/sql/
- `WebviewWindowBuilder`: https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html

**Wails**
- Introduction (React templates, WebView2, “small runtime”): https://wails.io/docs/introduction (source: https://raw.githubusercontent.com/wailsapp/wails/master/website/docs/introduction.mdx)
- How it works: https://wails.io/docs/howdoesitwork (source: https://raw.githubusercontent.com/wailsapp/wails/master/website/docs/howdoesitwork.mdx)
- Releases / v3 stable dates: https://github.com/wailsapp/wails/releases
- WebView2 Go loader: https://github.com/wailsapp/go-webview2

**Electron**
- What Electron is (embeds Chromium and Node.js): https://www.electronjs.org/docs/latest/
- Process model: https://www.electronjs.org/docs/latest/tutorial/process-model
- Electron Forge templates: https://www.electronforge.io/templates
- Vite official React plugin: https://github.com/vitejs/vite-plugin-react
- Node-API native addons: https://nodejs.org/api/n-api.html

**Neutralinojs**
- Docs home (static server, WebSocket, resource claim): https://neutralino.js.org/docs/
- Repo / `lib/webview` / extensions loader: https://github.com/neutralinojs/neutralinojs
- WebView library (Edge WebView2 on Windows): https://github.com/webview/webview
- Chrome mode source: https://github.com/neutralinojs/neutralinojs/blob/main/chrome.cpp
- Community React template: https://github.com/codezri/neutralinojs-react

**WebView2 (Microsoft)**
- Process model: https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/process-model
- Distribution / Evergreen vs fixed: https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution
- Win32 host getting started: https://learn.microsoft.com/en-us/microsoft-edge/webview2/get-started/win32

**DDC/CI and Windows monitor APIs**
- `EnumDisplayMonitors`: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaymonitors
- `GetNumberOfPhysicalMonitorsFromHMONITOR`: https://learn.microsoft.com/en-us/windows/win32/api/physicalmonitorenumerationapi/nf-physicalmonitorenumerationapi-getnumberofphysicalmonitorsfromhmonitor
- `GetPhysicalMonitorsFromHMONITOR`: https://learn.microsoft.com/en-us/windows/win32/api/physicalmonitorenumerationapi/nf-physicalmonitorenumerationapi-getphysicalmonitorsfromhmonitor
- `DestroyPhysicalMonitors`: https://learn.microsoft.com/en-us/windows/win32/api/physicalmonitorenumerationapi/nf-physicalmonitorenumerationapi-destroyphysicalmonitors
- `GetVCPFeatureAndVCPFeatureReply`: https://learn.microsoft.com/en-us/windows/win32/api/lowlevelmonitorconfigurationapi/nf-lowlevelmonitorconfigurationapi-getvcpfeatureandvcpfeaturereply
- `SetVCPFeature`: https://learn.microsoft.com/en-us/windows/win32/api/lowlevelmonitorconfigurationapi/nf-lowlevelmonitorconfigurationapi-setvcpfeature
- `GetCapabilitiesStringLength`: https://learn.microsoft.com/en-us/windows/win32/api/lowlevelmonitorconfigurationapi/nf-lowlevelmonitorconfigurationapi-getcapabilitiesstringlength
- `CapabilitiesRequestAndCapabilitiesReply`: https://learn.microsoft.com/en-us/windows/win32/api/lowlevelmonitorconfigurationapi/nf-lowlevelmonitorconfigurationapi-capabilitiesrequestandcapabilitiesreply
- `GetMonitorBrightness`: https://learn.microsoft.com/en-us/windows/win32/api/highlevelmonitorconfigurationapi/nf-highlevelmonitorconfigurationapi-getmonitorbrightness
- `SetMonitorBrightness`: https://learn.microsoft.com/en-us/windows/win32/api/highlevelmonitorconfigurationapi/nf-highlevelmonitorconfigurationapi-setmonitorbrightness
- `WM_DISPLAYCHANGE`: https://learn.microsoft.com/en-us/windows/win32/gdi/wm-displaychange
- VESA standards (DDC/CI 1.1, MCCS 2.2a): https://vesa.org/vesa-standards/

**Rust crates / bindings**
- `windows` `EnumDisplayMonitors`: https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Graphics/Gdi/fn.EnumDisplayMonitors.html
- `windows` `GetPhysicalMonitorsFromHMONITOR`: https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Devices/Display/fn.GetPhysicalMonitorsFromHMONITOR.html
- `windows` `SetVCPFeature`: https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Devices/Display/fn.SetVCPFeature.html
- `windows` `GetVCPFeatureAndVCPFeatureReply`: https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Devices/Display/fn.GetVCPFeatureAndVCPFeatureReply.html
- `windows` `GetCapabilitiesStringLength`: https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Devices/Display/fn.GetCapabilitiesStringLength.html
- `windows` `DestroyPhysicalMonitors`: https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Devices/Display/fn.DestroyPhysicalMonitors.html
- `windows` `LightSensor`: https://microsoft.github.io/windows-docs-rs/doc/windows/Devices/Sensors/struct.LightSensor.html
- `ddc-winapi`: https://docs.rs/ddc-winapi/latest/ddc_winapi/ ; repo: https://github.com/arcnmx/ddc-winapi-rs
- `ddc-hi`: https://docs.rs/ddc-hi/latest/ddc_hi/
- `webview2-com`: https://docs.rs/webview2-com/latest/webview2_com/

**Adaptive brightness**
- `Windows.Devices.Sensors.LightSensor`: https://learn.microsoft.com/en-us/uwp/api/windows.devices.sensors.lightsensor
- Sensor API (deprecated in favor of UWP): https://learn.microsoft.com/en-us/windows/win32/sensorsapi/portal
- Call WinRT APIs in desktop apps: https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/desktop-to-uwp-enhance
- WinRT APIs not supported in desktop apps: https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/winrt-api-desktop-app-support
- Desktop Duplication API: https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api

**React Native Windows**
- Getting started: https://microsoft.github.io/react-native-windows/docs/getting-started
- System requirements: https://microsoft.github.io/react-native-windows/docs/rnw-dependencies
- Native modules: https://microsoft.github.io/react-native-windows/docs/native-platform

**RAM / size benchmark**
- Repo: https://github.com/Elanis/web-to-desktop-framework-comparison
- README / tables / methodology: https://raw.githubusercontent.com/Elanis/web-to-desktop-framework-comparison/main/README.md
- Tauri manifest (v2): https://raw.githubusercontent.com/Elanis/web-to-desktop-framework-comparison/main/benchmark/01-empty-app/tauri/src-tauri/Cargo.toml
- Electron manifest (^45.1.0): https://raw.githubusercontent.com/Elanis/web-to-desktop-framework-comparison/main/benchmark/01-empty-app/electron/package.json
- Wails manifest (v2.17.1): https://raw.githubusercontent.com/Elanis/web-to-desktop-framework-comparison/main/benchmark/01-empty-app/wails/go.mod
- Neutralino config (6.10.0): https://raw.githubusercontent.com/Elanis/web-to-desktop-framework-comparison/main/benchmark/01-empty-app/neutralinojs/neutralino.config.json

**Go / non-React toolkits**
- Go `LazyDLL`: https://pkg.go.dev/golang.org/x/sys/windows#LazyDLL
- GPUI (Zed): https://github.com/zed-industries/zed
- egui: https://github.com/emilk/egui
- Slint: https://github.com/slint-ui/slint

**Repo-local context**
- Current app overview: [README.md](../../README.md)
- Safety rules: [AGENTS.md](../../AGENTS.md)
- VCP reference: [VCP_REFERENCE.md](../../VCP_REFERENCE.md)
