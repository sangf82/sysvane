# Modular Rust application patterns for a low-RAM Windows hardware tool

**Date:** 2026-11-09
**Scope:** How existing modular Rust applications structure modules and plugins, and what a tray-first Windows replacement for Armoury Crate should copy. Source reading and official docs only. Repositories were shallow-cloned at the commits in section 3 and read. No downloaded code was built or run, and no hardware was touched. Sensor, driver and tool facts are in [2026-11-08-universal-pc-control-tools.md](2026-11-08-universal-pc-control-tools.md) and [2026-11-08-armoury-replacement-plan.md](2026-11-08-armoury-replacement-plan.md) and are not repeated here.

## 1. Direct answer

**The prior recommendation holds in shape and needs four changes.** A small headless Rust core with compiled-in modules, plus external programs that run and exit, is what the closest projects converge on. No hardware-control project I read loads Rust DLL plugins.

The four changes:

1. **Two of the three reasons against DLLs do not separate DLLs from compiled-in modules.** A compiled-in module can also crash or hang the core, and it also runs with core privileges. Only the ABI reason is specific to DLLs (section 8).
2. **Add a third tier: a long-lived supervised worker process, but only for hang-prone or privileged code.** CPU temperature through PawnIO is the first candidate. RIGStats, CoolerControl and fan-control all put this kind of code in a child process, and two of them say why: a stuck call cannot be cancelled in-process.
3. **Sensors need a shared, demand-driven source.** "Universal core" should mean one sampler that runs only while a consumer holds a lease.
4. **One owner per physical bus, and no raw write path to external tools.** RIGStats forbids helper processes on the buses it owns. The `toushou` CLI has a `set --force` path, so the core must call a closed list of fixed command lines.

Build order stands: LCD first, `toushou` second. The two share so little that the first shared interface should be a static table, with no trait (section 9).

**How to read the labels.** "Observed" means I read it in source or official docs at the cited commit. "Inferred" is my own conclusion. RAM figures are labelled "project-measured" (method stated), "vendor claim" (number with no method), or "unmeasured". I took no measurements.

## 2. Comparison table

RAM is for the whole app unless stated. "Isolation" means what survives if one module panics or hangs.

| Project | Mechanism | Interface (path) | Discovery, lifecycle | Lazy start/stop | Isolation | Safety boundary | Compatibility | RAM or size |
|---|---|---|---|---|---|---|---|---|
| **CoolerControl** (Linux) | Compiled-in `Repository` backends, Python `liqctld` sidecar, gRPC service plugins | [`repository.rs:32`](https://gitlab.com/coolercontrol/coolercontrol/-/blob/2288bc10cdcaf8b64bb27f0501b58111c0f6a524/coolercontrold/daemon/src/repositories/repository.rs#L32), [`device_service.proto`](https://gitlab.com/coolercontrol/coolercontrol/-/blob/2288bc10cdcaf8b64bb27f0501b58111c0f6a524/coolercontrold/daemon/resources/proto/coolercontrol/device_service/v1/device_service.proto) | Plugin folder with `manifest.toml`; systemd or OpenRC runs each | No consumer-based stop found | Sidecar and plugins: process. Built-in repos: none | Manifest `privileged` flag; manifest root-owned | Proto package `v1`; no version gate found | Unmeasured |
| **RIGStats** (Windows; egui + .NET) | Rust UI, LocalSystem .NET service, child workers, `IControlProvider` set | [`ControlProtocol.cs:21`](https://github.com/dvalfrid/rigstats/blob/ab6110e7bf8b0e08ddd9217a0b5b2b067e2e15bf/sensor-sidecar/Control/ControlProtocol.cs#L21) | Compiled-in providers, probed per request | Sample on request with max age | Hardware and GPU workers: process | Pipe client path check; firmware release on stop | App and service ship together | Project-measured: about 220 MB working set, 355 MB private for the app ([architecture.md](https://github.com/dvalfrid/rigstats/blob/ab6110e7bf8b0e08ddd9217a0b5b2b067e2e15bf/docs/architecture.md#L897)) |
| **asusctl** (Linux) | Compiled-in controllers, optional traits, D-Bus to separate GUI | [`asusd/src/lib.rs:143`](https://gitlab.com/asus-linux/asusctl/-/blob/321b1114a0ac3782d67aa39201682effb749f4d9/asusd/src/lib.rs#L143) | Hardcoded in `daemon.rs`; udev for USB | No | None; release is `panic = "abort"` | D-Bus policy by group | Single workspace | Vendor claim: "less than 1Mb" ([README](https://gitlab.com/asus-linux/asusctl/-/blob/321b1114a0ac3782d67aa39201682effb749f4d9/README.md)) |
| **fan-control** (Linux, Windows) | One `HardwareBridge` trait; Windows backend is a C# LHM child over localhost TCP | [`hardware/src/lib.rs:133`](https://github.com/wiiznokes/fan-control/blob/fba9921982d69c63a1a5f4a45994a554fe1f30ea/hardware/src/lib.rs#L133) | `cfg(target_os)` | No | Windows backend: process | Bridge has no authentication that I found | Same repo | Unmeasured |
| **RustCooling** (Linux, Windows) | Monolith, one device | [`src/`](https://github.com/Qyzom/RustCooling/tree/e4fd0ed384ef5c5a408fc64a980d4811ebf04428/src) | CLI flags | No | None | None | Not applicable | Vendor claim: 4.2 MB RSS daemon, 12 to 14 MB Windows GUI ([README](https://github.com/Qyzom/RustCooling/blob/e4fd0ed384ef5c5a408fc64a980d4811ebf04428/README.md)) |
| **LACT** (Linux) | Compiled-in `GpuController` per vendor, JSON socket to GUI | [`gpu_controller.rs:41`](https://github.com/ilya-zlobintsev/LACT/blob/56fd661cab90b3735d72958bd5c86427fa30bb8c/lact-daemon/src/server/gpu_controller.rs#L41) | PCI discovery | No | None | Socket group; auto-revert unless confirmed | Same repo | Unmeasured |
| **system76-power** (Linux) | Monolithic daemon | [`src/daemon/mod.rs`](https://github.com/pop-os/system76-power/blob/9a334723fbc96b993bcbe9ade1914224f8170bf7/src/daemon/mod.rs) | Fixed | No | None | Polkit on threshold writes | Same repo | Unmeasured |
| **cosmic-applets** (Linux) | One multicall binary, one process per applet | [`cosmic-applets/src/main.rs:19`](https://github.com/pop-os/cosmic-applets/blob/ef6c9e62f64dab0e4a3d03c75d369f75279fea51/cosmic-applets/src/main.rs#L19) | Panel spawns configured applets from `.desktop` entries | iced `subscription()` | Process | Wayland security context | Same repo | Unmeasured |
| **Ironbar** (Linux) | Cargo features, `Module` trait, shared `Client` registry | [`clients/mod.rs:69`](https://github.com/JakeStanger/ironbar/blob/57d091d3af461c53bee6eab18b204b44f4035a8e/src/clients/mod.rs#L69) | Config lists modules | Client created on first use | None | None | Same repo | Unmeasured |
| **Eww** (Linux) | External scripts as variables | [`script_var_handler.rs`](https://github.com/elkowar/eww/blob/48f5aa8b379adf29da0b0bb9ca04164f65d8bdaa/crates/eww/src/script_var_handler.rs) | Config file | Yes: starts when an open window uses it | Process | None | Text output | Unmeasured |
| **Nushell** | External process, JSON or MessagePack | [`nu-plugin/src/plugin/mod.rs:99`](https://github.com/nushell/nushell/blob/09f275bd004a2d7113952327beeca19a1da2a3d9/crates/nu-plugin/src/plugin/mod.rs#L99), [`nu-plugin-protocol/src/lib.rs:299`](https://github.com/nushell/nushell/blob/09f275bd004a2d7113952327beeca19a1da2a3d9/crates/nu-plugin-protocol/src/lib.rs#L299) | `plugin add` caches signatures | Spawn on use, stop after 10 s idle | Process | None; runs as the user | Semver handshake | Unmeasured |
| **Zellij** | WASM guest on wasmi | [`zellij-tile/src/lib.rs:36`](https://github.com/zellij-org/zellij/blob/43c8710b3a9845f60d5a567c7410fc1197fabb66/zellij-tile/src/lib.rs#L36) | URL or file, layouts | Not traced | Sandbox | User-granted permissions, preopened directories | Plugin can query host version | Project-measured: idle session private 38.1 to 15.2 MB, whole server on Linux ([#5662](https://github.com/zellij-org/zellij/pull/5662)). Guest memory cap 16 MB |
| **Vector** | Cargo features plus `typetag` | [`src/config/source.rs:85`](https://github.com/vectordotdev/vector/blob/962ab2f804d18c99941bfddf050e8fe4061cae97/src/config/source.rs#L85) | TOML/YAML names a component `type` | Config diff reload | None (inferred) | Build-time feature set | Same repo | Unmeasured |
| **Bevy** | Compile-time `Plugin` trait | [`crates/bevy_app/src/plugin.rs:57`](https://github.com/bevyengine/bevy/blob/526f9e40253348a52a0d0380db98e287e4b5e0b3/crates/bevy_app/src/plugin.rs#L57) | Added in code; groups can disable members | No | None | None | Same compiler build | Unmeasured |
| **Tauri 2** | Compile-time crate plus frontend ACL | [`crates/tauri/src/plugin.rs:37`](https://github.com/tauri-apps/tauri/blob/2e23492ab9bcec06bab041e0e47447aaa0da298c/crates/tauri/src/plugin.rs#L37) | Registered on the app builder | No | None | Capabilities constrain the webview only | Cargo version | See [prior note](2026-11-04-low-ram-react-monitor-stack.md) |
| **abi_stable** | Rust DLL with checked layout | [`abi_stable/src/library.rs`](https://github.com/rodrimati1992/abi_stable_crates/blob/9966b8f0084fc768e3fb557bf81affea0b5868d8/abi_stable/src/library.rs) | Loaded at startup, never unloaded | No unload | None; panics at the boundary abort | None | Own ABI per `0.y.0` | Unmeasured |
| **stabby** | Rust DLL with compact stable types | [`stabby/src/libloading.rs:18`](https://github.com/ZettaScaleLabs/stabby/blob/8bfa5cbead9c63a8ed9293eed5e5151d72f53733/stabby/src/libloading.rs#L18) | `libloading` | Not stated | None | None | Reports or canaries | Unmeasured |
| **Lapce** | WASI "volts" on Wasmtime 14 | [`lapce-proxy/src/plugin/wasi.rs:416`](https://github.com/lapce/lapce/blob/f66ffaa95387794205c35864517de70958b9ea8d/lapce-proxy/src/plugin/wasi.rs#L416) | Volt directory with metadata | Activation by language or workspace file | Sandbox | Preopened directory; inherits environment | Volt metadata | Unmeasured |
| **Helix** | None on master | Not applicable | Not applicable | Not applicable | Not applicable | Not applicable | Not applicable | Not applicable |

## 3. What each claim rests on

| Repository | Commit | Commit date |
|---|---|---|
| CoolerControl | `2288bc10cdca` | 2026-11-08 |
| RIGStats | `ab6110e7bf8b` | 2026-11-07 |
| asusctl (GitLab copy; the README says development moved to the OGC GitHub) | `321b1114a0ac` | 2026-06-02 |
| fan-control | `fba9921982d6` | 2026-08-16 |
| RustCooling | `e4fd0ed384ef` | 2026-11-06 |
| trv | `d72c1b35883f` | 2026-04-01 |
| LACT | `56fd661cab90` | 2026-11-07 |
| system76-power | `9a334723fbc9` | 2026-07-13 |
| cosmic-applets, cosmic-panel | `ef6c9e62f64d`, `69d506ca58d5` | 2026-11-07, 2026-11-06 |
| Ironbar, Eww | `57d091d3af46`, `48f5aa8b379a` | 2026-11-07, 2026-07-17 |
| Nushell (0.116.2) | `09f275bd004a` | 2026-11-08 |
| Zellij | `43c8710b3a98` | 2026-11-06 |
| Vector (0.62.0-dev; sparse checkout) | `962ab2f804d1` | 2026-11-06 |
| Bevy (0.20.0-dev; sparse checkout) | `526f9e402533` | 2026-11-07 |
| Tauri (2.12.5), plugins-workspace | `2e23492ab9bc`, `92e47a1c0866` | 2026-11-05, 2026-11-06 |
| abi_stable (0.11.3) | `9966b8f0084f` | 2023-10-12 |
| stabby (72.1.16) | `8bfa5cbead9c` | 2026-09-27 |
| Helix, Lapce | `ba40e547426b`, `f66ffaa95387` | 2026-09-29, 2026-09-30 |

These are default-branch heads, so most are ahead of the last tagged release. Web docs and pull requests were read on 2026-11-09.

## 4. Hardware and system tools

### 4.1 CoolerControl: the closest architecture

**Observed.**
- Five kinds of backend implement one `Repository` trait: CPU, GPU, hwmon, liquidctl and service plugins. Each initializes independently and a failure only logs ([`main.rs`](https://gitlab.com/coolercontrol/coolercontrol/-/blob/2288bc10cdcaf8b64bb27f0501b58111c0f6a524/coolercontrold/daemon/src/main.rs#L975)). The daemon must run as root.
- `liqctld` is the existing Python liquidctl wrapped as a child process over a Unix socket, optional at runtime ([README](https://gitlab.com/coolercontrol/coolercontrol/-/blob/2288bc10cdcaf8b64bb27f0501b58111c0f6a524/coolercontrold/daemon/resources/liqctld/README.md)). A supervisor restarts it after a 2 s pause ([`liqctld_service.rs:27`](https://gitlab.com/coolercontrol/coolercontrol/-/blob/2288bc10cdcaf8b64bb27f0501b58111c0f6a524/coolercontrold/daemon/src/repositories/liquidctl/liqctld_service.rs#L27)).
- The trait documents why the process boundary matters: "Only repositories that isolate their device IO can report this; the rest cannot tell a wedged device from a slow one" ([`repository.rs:134`](https://gitlab.com/coolercontrol/coolercontrol/-/blob/2288bc10cdcaf8b64bb27f0501b58111c0f6a524/coolercontrold/daemon/src/repositories/repository.rs#L134)). Recovery restarts the service because it can reap a worker "wedged in a call that cannot be interrupted from inside it" ([`liquidctl_repo.rs:92`](https://gitlab.com/coolercontrol/coolercontrol/-/blob/2288bc10cdcaf8b64bb27f0501b58111c0f6a524/coolercontrold/daemon/src/repositories/liquidctl/liquidctl_repo.rs#L92)).
- Each device has one `Semaphore(1)` serializing requests ([`liquidctl_repo.rs:273`](https://gitlab.com/coolercontrol/coolercontrol/-/blob/2288bc10cdcaf8b64bb27f0501b58111c0f6a524/coolercontrold/daemon/src/repositories/liquidctl/liquidctl_repo.rs#L273)). The project says any other program regularly driving such a device "has to go through CoolerControl, or it competes with sensor polling" ([VISION.md](https://gitlab.com/coolercontrol/coolercontrol/-/blob/2288bc10cdcaf8b64bb27f0501b58111c0f6a524/VISION.md#L29)).
- Service plugins are separate executables over gRPC, with 12 RPCs: health, list, init, status, reset, duty, profile, lighting, LCD and so on. Calls get eight poll intervals, and shutdown gets 2 s.
- The manifest declares `privileged`, and is forced root-owned because "a plugin-writable manifest lets a plugin grant itself `privileged = true`" ([`trust.rs:41`](https://gitlab.com/coolercontrol/coolercontrol/-/blob/2288bc10cdcaf8b64bb27f0501b58111c0f6a524/coolercontrold/daemon/src/repositories/service_plugin/trust.rs#L41)).
- After 8 missed statuses the daemon reports failsafe values, temperature 100 and fan 0 ([`failsafe.rs:20`](https://gitlab.com/coolercontrol/coolercontrol/-/blob/2288bc10cdcaf8b64bb27f0501b58111c0f6a524/coolercontrold/daemon/src/repositories/failsafe.rs#L20)).

**A warning from its own history.** VISION.md puts the built-in LCD renderers "on a deprecation path": the daemon owns the serialized device connection and hardware commands, and rendering belongs elsewhere. The `Repository` trait also grew device-specific methods such as `apply_setting_lcd` and `apply_setting_pwm_mode`, most with no meaning for most backends.

**Inferred.** Keep frame transport and frame rendering as separate functions inside the LCD module. A trait designed around every feature ends up wide, which supports extracting interfaces late.

### 4.2 RIGStats: the closest Windows product

**Observed.** Details are in [architecture.md](https://github.com/dvalfrid/rigstats/blob/ab6110e7bf8b0e08ddd9217a0b5b2b067e2e15bf/docs/architecture.md) and [control-architecture.md](https://github.com/dvalfrid/rigstats/blob/ab6110e7bf8b0e08ddd9217a0b5b2b067e2e15bf/docs/control-architecture.md).
- Rust egui app, plus a LocalSystem .NET service, plus `--hardware-worker` and `--gpu-worker` child processes of the same executable. LHM and PawnIO run only in the hardware worker.
- The worker restarts with backoff 0, 1, 2, 5, 15, 60 s and an 8 s call timeout ([`HardwareWorkerClient.cs:30`](https://github.com/dvalfrid/rigstats/blob/ab6110e7bf8b0e08ddd9217a0b5b2b067e2e15bf/sensor-sidecar/Control/Workers/HardwareWorkerClient.cs#L30)).
- `FanWatchdog` ends the worker when the fan loop stalls for 10 s, because "A hung call can't be cancelled in-process" ([`FanWatchdog.cs:19`](https://github.com/dvalfrid/rigstats/blob/ab6110e7bf8b0e08ddd9217a0b5b2b067e2e15bf/sensor-sidecar/Control/FanWatchdog.cs#L19)).
- **Counter-evidence for external tools:** "One process owns the hardware... no external helper processes (no OpenRGB server, no ZenStates tool, no second driver)" ([control-architecture.md:64](https://github.com/dvalfrid/rigstats/blob/ab6110e7bf8b0e08ddd9217a0b5b2b067e2e15bf/docs/control-architecture.md#L64)). The stated reason is concurrent access to Super I/O and SMU.
- `IControlProvider` is `Probe`, `Validate`, `Capture`, `Apply`, `Verify`, `Restore`, so each write is a transaction with rollback. `ControlBroker` fixes the order across domains.
- `BootCrashGuard` writes a marker before a risky change and clears it after 3 minutes of uptime. A marker found at start means that change is not re-applied.
- `PipeClientVerifier` checks the connecting process's kernel-reported image path. Its comments say the signer check guards against a mismatched install, not a forged binary.
- `media.rs` starts its watcher on the first `latest()` and ends it 5 s after the last, "so nothing runs while the panel is hidden".

**Why its RAM is not a target.** The 355 MB private figure belongs to a full wgpu dashboard. Its service and workers are counted separately and their RAM is not in that baseline.

### 4.3 asusctl and rog-control-center

**Observed.**
- Controllers are compiled in, and implement only the traits they need: `Reloadable`, `ZbusRun`, `CtrlTask`, `GetSupported` ([design-patterns.md](https://gitlab.com/asus-linux/asusctl/-/blob/321b1114a0ac3782d67aa39201682effb749f4d9/design-patterns.md), [`lib.rs:143`](https://gitlab.com/asus-linux/asusctl/-/blob/321b1114a0ac3782d67aa39201682effb749f4d9/asusd/src/lib.rs#L143)).
- `daemon.rs` tries each controller and warns on failure, so missing hardware disables only that feature.
- `DeviceManager` watches udev, rejects vendors other than `0b05`, opens one shared `HidRaw` per device node, then tries each device kind by product ID ([`aura_manager.rs:99`](https://gitlab.com/asus-linux/asusctl/-/blob/321b1114a0ac3782d67aa39201682effb749f4d9/asusd/src/aura_manager.rs#L99)).
- LED capability is data: `aura_support.ron` lists modes and zones per laptop. A model missing from the file gets a default with only the Static mode, and a missing file gets brightness-only control ([`aura_detection.rs`](https://gitlab.com/asus-linux/asusctl/-/blob/321b1114a0ac3782d67aa39201682effb749f4d9/rog-aura/src/aura_detection.rs)).
- Release uses `panic = "abort"`, so any panic ends the daemon. GUI is a separate process, `rog-control-center`, with its own tray.

### 4.4 fan-control

**Observed.** The whole hardware abstraction is one small trait: `hardware()`, sensor and control reads, `set_value`, `set_mode`, and `update()` for batch refresh ([`lib.rs:133`](https://github.com/wiiznokes/fan-control/blob/fba9921982d69c63a1a5f4a45994a554fe1f30ea/hardware/src/lib.rs#L133)). On Windows it spawns a C# LibreHardwareMonitor wrapper, then scans localhost ports with a fixed check string ([`windows.rs:74`](https://github.com/wiiznokes/fan-control/blob/fba9921982d69c63a1a5f4a45994a554fe1f30ea/hardware/src/windows.rs#L74)).

**Inferred.** The trait size is worth copying. The port scan with a constant string is not, because any local process could answer it.

### 4.5 RustCooling and trv: the closest LCD tools

Neither supports `87AD:70DB`, and I found no Rust project that lists it.
- **RustCooling** is for ID-COOLING `1A86:E317`, a 2-digit display, so frame rate and payload differ greatly. Its claimed 4.2 MB daemon is the best available hint for a headless USB loop in Rust, with no method given. It embeds `WinRing0x64.sys` ([`driver.rs:24`](https://github.com/Qyzom/RustCooling/blob/e4fd0ed384ef5c5a408fc64a980d4811ebf04428/src/telemetry/driver.rs#L24)), which the [survey](2026-11-08-universal-pc-control-tools.md) treats as unusable. Do not copy that part.
- **trv** targets Frozen Vision over ADB and TCP, per its [DATAFLOW.md](https://github.com/linketh/trv/blob/d72c1b35883fb4be7da49ef6585f6dca66a246c2/DATAFLOW.md). It is a different transport from WinUSB bulk. Its backoff on send failure is a reasonable loop shape.

### 4.6 LACT and system76-power

**Observed.**
- LACT has one `GpuController` trait per vendor and a newline-delimited JSON socket ([API.md](https://github.com/ilya-zlobintsev/LACT/blob/56fd661cab90b3735d72958bd5c86427fa30bb8c/docs/API.md)). A settings change applies first and stays unsaved. Without confirmation within 5 s by default, it reverts ([`handler.rs:356`](https://github.com/ilya-zlobintsev/LACT/blob/56fd661cab90b3735d72958bd5c86427fa30bb8c/lact-daemon/src/server/handler.rs#L356)).
- system76-power is one daemon with no module system. Polkit guards charge-threshold writes ([`mod.rs:317`](https://github.com/pop-os/system76-power/blob/9a334723fbc96b993bcbe9ade1914224f8170bf7/src/daemon/mod.rs#L317)).

### 4.7 cosmic-applets, Ironbar, Eww

**Observed.**
- **cosmic-applets** is one binary dispatching on `argv[0]`, one process per applet, `panic = "abort"` ([`main.rs:19`](https://github.com/pop-os/cosmic-applets/blob/ef6c9e62f64dab0e4a3d03c75d369f75279fea51/cosmic-applets/src/main.rs#L19)). The panel restarts an applet that exits with an error ([`wrapper_space.rs:681`](https://github.com/pop-os/cosmic-panel/blob/69d506ca58d5fcd05205cc77aec5a9e58aac2df6/cosmic-panel-bin/src/space/wrapper_space.rs#L681)). Each applet declares its event sources through iced's `subscription()` ([`app.rs:983`](https://github.com/pop-os/cosmic-applets/blob/ef6c9e62f64dab0e4a3d03c75d369f75279fea51/cosmic-applet-battery/src/app.rs#L983)).
- **Ironbar** feature-gates each module. A `Client` singleton is registered once and handed to every module that wants it, with `subscribe()` returning a broadcast receiver. Its `stop()` is marked `dead_code`, so that path is unused today ([`clients/mod.rs:69`](https://github.com/JakeStanger/ironbar/blob/57d091d3af461c53bee6eab18b204b44f4035a8e/src/clients/mod.rs#L69)).
- **Eww** starts a script variable when an open window references it, and stops it when no open window does ([`app.rs:404`](https://github.com/elkowar/eww/blob/48f5aa8b379adf29da0b0bb9ca04164f65d8bdaa/crates/eww/src/app.rs#L404)). `defpoll` also takes `:run-while`.

**Inferred.** A same-executable worker gets a process boundary with no second binary to ship. Windows should use a subcommand rather than `argv[0]`. Its code pages can be shared between the two processes, since Microsoft describes a page already in another process's working set as resolved by a soft fault ([Working Set](https://learn.microsoft.com/en-us/windows/win32/memory/working-set)).

## 5. Distinct plugin mechanisms

### 5.1 Nushell: external process over a serialized protocol

**Observed.**
- A plugin is an executable. Plugin and engine exchange a `Hello` containing protocol version and features, then calls over stdio or a local socket, in JSON or MessagePack ([protocol reference](https://www.nushell.sh/contributor-book/plugin_protocol_reference.html)).
- Local sockets use a named pipe path on Windows through the `interprocess` crate, and fall back to stdio if setup fails ([`local_socket/mod.rs`](https://github.com/nushell/nushell/blob/09f275bd004a2d7113952327beeca19a1da2a3d9/crates/nu-plugin-core/src/communication_mode/local_socket/mod.rs)).
- Compatibility is a caret semver comparison, with pre-release tags ignored ([`protocol_info.rs:33`](https://github.com/nushell/nushell/blob/09f275bd004a2d7113952327beeca19a1da2a3d9/crates/nu-plugin-protocol/src/protocol_info.rs#L33)).
- `plugin add` caches signatures, so the shell knows the commands without starting the plugin ([book](https://www.nushell.sh/book/plugins.html)).
- The plugin is spawned on use by `PersistentPlugin::get` ([`persistent.rs:80`](https://github.com/nushell/nushell/blob/09f275bd004a2d7113952327beeca19a1da2a3d9/crates/nu-plugin-engine/src/persistent.rs#L80)). It is stopped after 10 s idle by default, configurable per plugin ([`plugin_gc.rs:58`](https://github.com/nushell/nushell/blob/09f275bd004a2d7113952327beeca19a1da2a3d9/crates/nu-protocol/src/config/plugin_gc.rs#L58)).

**What to take.** This is the right design if third parties ever add modules. Spawn on use plus idle stop is the best-specified lazy lifecycle here.

### 5.2 Zellij: WASM sandbox

**Observed.**
- `ZellijPlugin` is `load`, `update`, `pipe`, `render`. Plugins subscribe to event types and get only those ([`shim.rs:59`](https://github.com/zellij-org/zellij/blob/43c8710b3a9845f60d5a567c7410fc1197fabb66/zellij-tile/src/shim.rs#L59)). Messages are protobuf.
- Host access goes through user-granted [permissions](https://zellij.dev/documentation/plugin-api-permissions). The filesystem is preopened at `/host`, `/data` and `/tmp` ([`plugin_loader.rs:474`](https://github.com/zellij-org/zellij/blob/43c8710b3a9845f60d5a567c7410fc1197fabb66/zellij-server/src/plugins/plugin_loader.rs#L474)).
- Each guest memory is limited to 16 MB and traps on failure ([`plugin_loader.rs:526`](https://github.com/zellij-org/zellij/blob/43c8710b3a9845f60d5a567c7410fc1197fabb66/zellij-server/src/plugins/plugin_loader.rs#L526)). A crash goes to `handle_plugin_crash`, which shows the error in that plugin's pane ([`wasm_bridge.rs:3736`](https://github.com/zellij-org/zellij/blob/43c8710b3a9845f60d5a567c7410fc1197fabb66/zellij-server/src/plugins/wasm_bridge.rs#L3736)).
- Zellij moved from wasmer to wasmtime to the wasmi interpreter. The author cited removing the plugin cache, complexity and Debian packaging, and said memory and binary size fell, with no figures ([#4449](https://github.com/zellij-org/zellij/pull/4449)).
- [#5662](https://github.com/zellij-org/zellij/pull/5662) measured "+46.7 MB" before and "+1.1 MB" after when attaching 2 more clients, because each client had its own copy of every plugin.

**What to take.** Per-instance state is the real cost of a plugin system, and sharing fixed it. WASM isolation gives no help for this app, because every interesting module needs USB, HID, DDC or a kernel driver. Each would become a host import that holds the risky code anyway (inferred).

### 5.3 Vector: Cargo features plus configuration

**Observed.** Each component is a `#[cfg(feature = "...")]` module ([`sources/mod.rs`](https://github.com/vectordotdev/vector/blob/962ab2f804d18c99941bfddf050e8fe4061cae97/src/sources/mod.rs)), with features declared in [`Cargo.toml`](https://github.com/vectordotdev/vector/blob/962ab2f804d18c99941bfddf050e8fe4061cae97/Cargo.toml#L792). `SourceConfig` is registered through `typetag`, and config selects it by `type`. Reload diffs old and new config and rebuilds only what changed ([`running.rs:313`](https://github.com/vectordotdev/vector/blob/962ab2f804d18c99941bfddf050e8fe4061cae97/src/topology/running.rs#L313)). `resources()` declares things only one component may own, such as ports ([`mod.rs:353`](https://github.com/vectordotdev/vector/blob/962ab2f804d18c99941bfddf050e8fe4061cae97/src/config/mod.rs#L353)).

**What to take.** `Resource` is a clean model for bus ownership. Declare `usb:87AD:70DB` or `ddc` and reject conflicts before starting. Feature gates trim the binary, but what RAM a tool uses depends on what it starts (inferred, unmeasured).

### 5.4 Bevy: compile-time trait

**Observed.** `Plugin` has `build`, `ready`, `finish`, `cleanup`, `name`, `is_unique`. Any `Fn(&mut App)` is also a plugin ([`plugin.rs:57`](https://github.com/bevyengine/bevy/blob/526f9e40253348a52a0d0380db98e287e4b5e0b3/crates/bevy_app/src/plugin.rs#L57)). Groups support `cfg` per member, and builders can disable members ([`plugin_group.rs:282`](https://github.com/bevyengine/bevy/blob/526f9e40253348a52a0d0380db98e287e4b5e0b3/crates/bevy_app/src/plugin_group.rs#L282)). Dynamic linking exists for compile speed, and its docs say not to ship it in release builds ([`bevy_dylib`](https://github.com/bevyengine/bevy/blob/526f9e40253348a52a0d0380db98e287e4b5e0b3/crates/bevy_dylib/src/lib.rs)).

**What to take.** A function can be a module. A large Rust project that has dynamic linking available still tells users not to ship it.

### 5.5 Tauri 2

**Observed.** `Plugin<R>` is a compiled-in trait with hooks such as `initialize` and `window_created` ([`plugin.rs:37`](https://github.com/tauri-apps/tauri/blob/2e23492ab9bcec06bab041e0e47447aaa0da298c/crates/tauri/src/plugin.rs#L37)). Capabilities map permissions to windows. Tauri's own docs list "Malicious or insecure Rust code" under what they do not protect against ([docs](https://v2.tauri.app/security/capabilities/)).

**What to take.** Use it for the window, if at all. Capabilities are no safety boundary for hardware writes, since all Rust code is outside them.

### 5.6 abi_stable and stabby

**Observed.**
- Rust's default layout guarantees almost nothing: "There are no other guarantees of data layout made by this representation" ([Reference](https://doc.rust-lang.org/reference/type-layout.html)). Both crates exist to work around this.
- abi_stable checks types recursively at load, and each `0.y.0` has its own incompatible ABI ([readme](https://github.com/rodrimati1992/abi_stable_crates/blob/9966b8f0084fc768e3fb557bf81affea0b5868d8/readme.md)). It leaks libraries because unloading is unsound for it ([`root_mod_trait.rs`](https://github.com/rodrimati1992/abi_stable_crates/blob/9966b8f0084fc768e3fb557bf81affea0b5868d8/abi_stable/src/library/root_mod_trait.rs)). A panic reaching its FFI guard aborts the process ([`macros.rs:179`](https://github.com/rodrimati1992/abi_stable_crates/blob/9966b8f0084fc768e3fb557bf81affea0b5868d8/abi_stable/src/macros.rs#L179)). Its last commit is 2023-10-12.
- stabby's README warns that a Rust 1.78 change put vtables in a leaked global set with linear lookup on stable compilers. Its source says canaries "don't guarantee" compatibility ([`libloading.rs`](https://github.com/ZettaScaleLabs/stabby/blob/8bfa5cbead9c63a8ed9293eed5e5151d72f53733/stabby/src/libloading.rs#L18)).

**What to take.** Stable-ABI loading is possible, but it keeps in-process crash risk, adds a macro-heavy type system, and gives a sole author nothing, because module and core ship together.

### 5.7 Wasmtime Component Model, Lapce, Helix

**Observed.**
- Wasmtime lists `component-model` and `x86_64-pc-windows-msvc` as Tier 1 ([tiers](https://docs.wasmtime.dev/stability-tiers.html)). Normal releases get 2 months of support and LTS releases 24 ([release process](https://docs.wasmtime.dev/stability-release.html)). The Component Model docs give WASI 0.3.2 as current, listing standard streams, environment variables and sockets as examples ([intro](https://component-model.bytecodealliance.org/introduction.html)).
- **Lapce** pins Wasmtime 14.0.0 ([`Cargo.toml`](https://github.com/lapce/lapce/blob/f66ffaa95387794205c35864517de70958b9ea8d/lapce-proxy/Cargo.toml)), which under that policy is long out of support (inferred from the version number). Plugins can activate by language or by workspace file, and `start_volt` inherits the host environment.
- **Helix** master has no plugin system, and its book still says "future plugin system". The Steel Scheme PR is open as a draft with activity on 2026-10-18, labelled `S-experimental`: "won't be merged in its current state" ([#8675](https://github.com/helix-editor/helix/pull/8675)).

**What to take.** One actively used editor still has no merged plugin runtime, and another is pinned to an old runtime version. A plugin runtime is a long-term commitment, which a sole author should avoid until needed.

### 5.8 Dropped

Waybar source (C++, and Eww and Ironbar cover the idea). Wasmtime source (docs only). Zellij plugin load triggers. Tauri's mobile plugin path. COSMIC panel config format.

## 6. Patterns worth copying

### 6.1 Capabilities

| Pattern | Source | Use here |
|---|---|---|
| Small trait, batch `update()` | fan-control | Sensors read in one batch |
| Probe per feature, absent means hidden | asusctl `GetSupported`, RIGStats `Probe` | "Show only what is detected" |
| Capability as data | asusctl `aura_support.ron` | Future device tables |
| Transaction: validate, capture, apply, verify, restore | RIGStats `IControlProvider` | Later, for risky writes. Overkill for the first two modules |

### 6.2 Shared sensors with demand-driven polling

Three working shapes:

1. **Pull with maximum age.** RIGStats: "One sample per SampleMaxAgeMs, however many telemetry clients and service-side consumers ask for it" ([`HardwareHost.cs:253`](https://github.com/dvalfrid/rigstats/blob/ab6110e7bf8b0e08ddd9217a0b5b2b067e2e15bf/sensor-sidecar/Control/HardwareHost.cs#L253)). It needs no thread when nobody asks. The sample runs in the background so a hung worker does not stall the caller.
2. **Reference by use.** Eww stops a script when no open window uses it. RIGStats `media.rs` stops 5 s after the last read.
3. **Channel closure.** `tokio::sync::watch::Sender::closed` "Completes when all receivers have dropped", and `receiver_count` gives the count ([docs, 1.54.1](https://docs.rs/tokio/latest/tokio/sync/watch/struct.Sender.html)).

**Recommendation (inferred).** Use leases with one sampler thread that parks while no lease is held. Plain pull is smaller, but the LCD must resend every 150 ms and a slow NVML or PawnIO call on that thread could push it toward the firmware's 2 to 3 s logo timeout. The LCD should read the last sample without blocking. RIGStats does the same by treating a sample older than 5 s as no data.

### 6.3 Device detection by USB ID

asusctl filters by vendor and then product. On Windows, call `CM_Register_Notification`, then `CM_Get_Device_Interface_List`, in that order, because the former does not report existing devices. Microsoft says to keep the callback fast and move blocking work to another thread ([docs](https://learn.microsoft.com/en-us/windows/win32/api/cfgmgr32/nf-cfgmgr32-cm_register_notification)). It needs no window (inferred from the callback signature), which suits a headless core. `RegisterDeviceNotification` needs a window or service handle ([docs](https://learn.microsoft.com/en-us/windows/win32/devio/registering-for-device-notification)).

### 6.4 Config-driven enablement

Hardware detection decides if a module can run and config decides if it should. Ironbar config names modules, Vector config names component types, and CoolerControl uses a disable list for plugins plus environment switches for subsystems. Start with a flat list of disabled module IDs (inferred).

### 6.5 Per-device safety

| Rule | Source | Use here |
|---|---|---|
| One serialized queue per device | CoolerControl `device_permits`, Vector `Resource` | One DDC action at a time; one LCD owner |
| Hand control back on stop | RIGStats `SafetyGuard`, CoolerControl `shutdown` | LCD firmware reverts on its own; later fan control needs this |
| Auto-revert unless confirmed | LACT, 5 s default | Worth considering for any future risky write that has a known previous value. It does not help banned DDC codes, which have no undo |
| Do not re-apply after a crash | RIGStats `BootCrashGuard` | Any future PawnIO or CPU writes |
| Stale data is no data | RIGStats 5 s, CoolerControl 8 missed | LCD shows blank rather than a stale temperature |
| Safety settings are not plugin-writable | CoolerControl `secure_manifest` | Bans live in code |
| Verify write-path clients | RIGStats `PipeClientVerifier` | When a pipe exists |

**The gap in this repo (observed).** [`src/cli.py`](../../src/cli.py) `set` has a `--force` option that overrides the checks for dangerous codes, power-off and input source. [`MonitorManager.set_vcp`](../../src/core/monitor_manager.py), the stated single write gateway, has no check of its own. `Profile.apply` does filter, so `apply-profile` and `apply-dual` without `--include-*` are safe paths. The Bash hook protects only agent shells and does not bind a Rust core that spawns a process. To make the rules unbypassable as required, move the check into `set_vcp` or below it, and remove or restrict `--force`. This research did not change either.

## 7. Linux-only mechanisms and Windows equivalents

| Linux | Used by | Windows equivalent | Note |
|---|---|---|---|
| D-Bus plus policy files | asusctl, system76-power | Named pipe with an explicit DACL | Default pipe ACL gives Everyone read access, and Microsoft says to use the logon SID to exclude other sessions ([docs](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights)) |
| Polkit | system76-power, LACT | UAC, or a privileged service plus client verification | RIGStats uses [`GetNamedPipeClientProcessId`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getnamedpipeclientprocessid) plus image path |
| udev, netlink | asusctl, CoolerControl | `CM_Register_Notification` | Section 6.3 |
| hwmon, sysfs, `/proc` | CoolerControl, LACT, RustCooling | NVML, PDH, WinUSB, PawnIO, vendor WMI | No unified layer; see the survey section 8.4 |
| systemd service, watchdog, restart | Most daemons | Service Control Manager or Task Scheduler, plus own supervisor | Process tree cleanup: job object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` ([docs](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects)) |
| logind sleep signals | asusctl, CoolerControl | [`PowerRegisterSuspendResumeNotification`](https://learn.microsoft.com/en-us/windows/win32/api/powerbase/nf-powerbase-powerregistersuspendresumenotification) with a callback | LCD must resume frames after sleep |
| Unix sockets | Most | Named pipes | Nushell's `interprocess` use shows the mapping |
| `SIGTERM`, `setsid` | RustCooling, liqctld | `CREATE_NO_WINDOW`, job objects | Observed in RustCooling and fan-control |

## 8. Testing the prior recommendation

| Claim | Result | Evidence |
|---|---|---|
| Rust has no stable ABI, so no Rust DLLs | **Holds** | Reference; abi_stable and stabby exist because of it and carry real costs |
| A crashing plugin takes down the core | **True, but not a reason to prefer compiled-in** | A compiled-in panic with `panic = "abort"` ends the process, and asusctl and cosmic-applets ship that. `catch_unwind` "only catches unwinding panics" ([std docs](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html)), and cannot end a hang |
| A plugin runs with core privileges | **True, same objection** | Compiled-in code has the same privileges. Only a process split changes this (RIGStats, CoolerControl) |
| Compiled-in modules enabled by config and detection | **Holds** | asusctl, LACT, RIGStats, Ironbar, Vector |
| Small headless core, UI elsewhere | **Holds** | asusd, coolercontrold, lactd, RIGStats service all separate it. Background RAM matches this |
| External run-and-exit programs | **Holds, conditionally** | CoolerControl wraps Python liquidctl. RIGStats rejects helpers on shared buses. Allowed only where the core does not touch that bus |
| Build two, then extract | **Holds** | CoolerControl's wide trait shows the opposite risk. fan-control shows how small a real interface can be |
| No third process tier needed | **Falsified** | RIGStats and CoolerControl isolate exactly the code that can wedge |
| "Everything is a plugin" (owner proposal) | **Rejected for now** | No hardware tool here does it. Nushell-style external plugins are the upgrade path if others must add modules |

**What would change this verdict.**
- Third parties need to add modules: adopt a Nushell-style protocol, not DLLs.
- Measured per-module RAM is significant while disabled: move toward Cargo features or separate processes.
- PawnIO or NVML proves stable under long runs: keep the worker tier unbuilt.
- The core needs to run elevated: a privilege split becomes mandatory earlier. Whether PawnIO requires admin is still unconfirmed in the plan.

**Tier rule (inferred).**
1. **Compiled-in, own OS thread:** user-mode USB and API code that a thread can hold, such as the LCD and NVML.
2. **Run-and-exit process:** existing tools in another runtime, for rare actions, on a bus nothing else owns, such as `toushou`, HeadsetControl, `OpenRGB --profile`.
3. **Supervised long-lived worker, same executable:** only for code that needs elevation or can hang in a driver. PawnIO is the expected first case. Do not build it for the first two modules.

## 9. Smallest interface for the first two modules

**The LCD and `toushou` have almost nothing in common**, so forcing a `Module` trait over them would invent an abstraction. One is resident, needs USB and sensors, and has a 150 ms deadline. The other runs for seconds on a tray click. What they share is an ID, a detection need, and being enabled by config. A `static` table captures that. Add a trait when a third module shows a repeated shape.

**Untested design.** I type-checked this with `rustc 1.99 --emit=metadata`. It was not linked because this machine has no MSVC linker, so its unit tests never ran. It has never touched hardware. The hub below tracks demand only, and the sampler is omitted.

```rust
use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Sensor { GpuTempC, GpuLoadPct }

/// One shared sampler. It polls only sensors with a live Lease.
#[derive(Clone, Default)]
pub struct SensorHub(Arc<Mutex<HashMap<Sensor, usize>>>);
pub struct Lease { hub: SensorHub, sensors: Vec<Sensor> }

impl SensorHub {
    pub fn lease(&self, sensors: &[Sensor]) -> Lease {
        let mut wanted = self.0.lock().unwrap();
        for s in sensors { *wanted.entry(*s).or_default() += 1; }
        Lease { hub: self.clone(), sensors: sensors.to_vec() }
    }
    /// The sampler reads this each tick and parks while it is empty.
    pub fn wanted(&self) -> Vec<Sensor> { self.0.lock().unwrap().keys().copied().collect() }
}

impl Drop for Lease {
    fn drop(&mut self) {
        let mut wanted = self.hub.0.lock().unwrap_or_else(|e| e.into_inner());
        for s in &self.sensors {
            if let Some(n) = wanted.get_mut(s) {
                *n -= 1;
                if *n == 0 { wanted.remove(s); }
            }
        }
    }
}

pub enum Cmd { Stop }
pub struct Ctx { pub sensors: SensorHub, pub cmds: Receiver<Cmd> }
pub enum Need { Usb { vid: u16, pid: u16 }, Exe(&'static str) }

/// A fixed command line. No VCP code or flag can be passed at runtime.
pub struct Action {
    pub id: &'static str,
    pub argv: &'static [&'static str],
    pub exclusive: &'static str, // one running action per bus
    pub timeout: Duration,
}

pub enum Kind {
    Resident(fn(&Ctx) -> Result<(), String>),
    Oneshot(&'static [Action]),
}

pub struct Module { pub id: &'static str, pub need: Need, pub kind: Kind }

pub static MODULES: &[Module] = &[
    Module {
        id: "thermalright-lcd",
        need: Need::Usb { vid: 0x87AD, pid: 0x70DB },
        kind: Kind::Resident(lcd_run),
    },
    Module {
        id: "toushou-ddc",
        need: Need::Exe("toushou"),
        kind: Kind::Oneshot(&[Action {
            id: "apply-calibrated",
            argv: &["apply-dual", "--name", "Calibrated-Dual-Setup"],
            exclusive: "ddc",
            timeout: Duration::from_secs(30),
        }]),
    },
];

fn lcd_run(ctx: &Ctx) -> Result<(), String> {
    let _lease = ctx.sensors.lease(&[Sensor::GpuTempC, Sensor::GpuLoadPct]);
    loop {
        match ctx.cmds.recv_timeout(Duration::from_millis(150)) {
            Ok(Cmd::Stop) | Err(RecvTimeoutError::Disconnected) => return Ok(()),
            Err(RecvTimeoutError::Timeout) => { /* resend the current frame over USB */ }
        }
    }
}

/// Runs on the module's own thread. Needs panic = "unwind" in the release profile.
pub fn supervise(run: fn(&Ctx) -> Result<(), String>, ctx: &Ctx) -> Result<(), String> {
    catch_unwind(AssertUnwindSafe(|| run(ctx))).unwrap_or_else(|_| Err("panicked".into()))
}
```

**Design consequences.**
- **Safety by construction.** No runtime value reaches `argv`, so tray code cannot request `set`, `--force` or `--include-*`. A unit test should assert that the table contains none of them, and the existing guards in `Profile.apply` still apply.
- **`exclusive`** copies Vector's `Resource` and stops a double click from launching two DDC writers.
- **Run-and-exit launcher, not shown:** spawn with `CREATE_NO_WINDOW` inside a kill-on-close job, enforce `timeout`, and resolve `toushou` to an absolute path. A relative search path is a risk once any part runs elevated.
- **`panic = "unwind"` is required**, unlike asusctl and cosmic-applets, because this design isolates by thread.
- **A hung USB call defeats `supervise`.** Section 4.1 and 4.2 show this cannot be cancelled in-process. If it occurs in practice, move the LCD to tier 3.
- **Frame caching.** The 150 ms tick should resend an already encoded frame, and re-encode only when the 1 Hz data changes (inferred).

## 10. Could not be verified

- **Any comparative RAM measurement between mechanisms.** No project publishes compiled-in versus DLL versus process versus WASM. The tier rule rests on failure behavior, not RAM data. RAM cost of threads, a worker process and disabled compiled-in modules must be measured on this PC, using private bytes to match the 848 MB baseline.
- **RustCooling's and asusd's RAM.** Both are vendor numbers. RustCooling's 4.2 MB has no method and appears near a Linux tip, and its protocol is far simpler than a 150 ms image stream.
- **Whether WinUSB transfer calls can hang past a timeout.** This drives whether the LCD can stay in tier 1. I did not research it, and the `nusb` behavior is unknown.
- **Whether PawnIO requires admin**, and how an elevated worker should be started for a single user. Scheduled task versus service was not researched.
- **Whether `CM_Register_Notification` fires for this WinUSB device interface.** Documentation covers the API. The right interface GUID for `87AD:70DB` is unconfirmed.
- **Zellij lifecycle triggers and version gating.** I read the loader and shim but did not trace what starts a plugin, and found only a host-version query.
- **Whether Vector isolates a panicking component.** Inferred no, since I did not read its task runner.
- **CoolerControl plugin versioning.** Health reports a version, and I found no compatibility gate in what I read, which does not show there is none.
- **Other Rust projects.** Search found no Rust Armoury Crate replacement beyond RIGStats, and no Rust driver for `87AD:70DB`. Absence from two web searches is weak evidence.
- **Vanta**, a Tauri-based Windows monitor found by search, could not be cloned from the URL given.
- **asusctl's new home.** I read the GitLab copy, whose README says future work is on GitHub under OGC. Newer changes were not read.
- **Current `rustc` ABI plans.** I did not check whether a stable `crabi`-style ABI has progressed.

## 11. Sources

All accessed **2026-11-09**. Source permalinks are inline, keyed to the commits in section 3.

**Repositories**
- CoolerControl: https://gitlab.com/coolercontrol/coolercontrol
- RIGStats: https://github.com/dvalfrid/rigstats
- asusctl: https://gitlab.com/asus-linux/asusctl
- fan-control: https://github.com/wiiznokes/fan-control
- RustCooling: https://github.com/Qyzom/RustCooling
- trv: https://github.com/linketh/trv
- LACT: https://github.com/ilya-zlobintsev/LACT
- system76-power: https://github.com/pop-os/system76-power
- cosmic-applets: https://github.com/pop-os/cosmic-applets , cosmic-panel: https://github.com/pop-os/cosmic-panel
- Ironbar: https://github.com/JakeStanger/ironbar , Eww: https://github.com/elkowar/eww
- Nushell: https://github.com/nushell/nushell , Zellij: https://github.com/zellij-org/zellij
- Vector: https://github.com/vectordotdev/vector , Bevy: https://github.com/bevyengine/bevy
- Tauri: https://github.com/tauri-apps/tauri , plugins: https://github.com/tauri-apps/plugins-workspace
- abi_stable: https://github.com/rodrimati1992/abi_stable_crates , stabby: https://github.com/ZettaScaleLabs/stabby
- Helix: https://github.com/helix-editor/helix , Lapce: https://github.com/lapce/lapce

**Official docs and pull requests**
- Rust type layout: https://doc.rust-lang.org/reference/type-layout.html
- `catch_unwind`: https://doc.rust-lang.org/std/panic/fn.catch_unwind.html
- Tokio `watch::Sender`: https://docs.rs/tokio/latest/tokio/sync/watch/struct.Sender.html
- Nushell protocol: https://www.nushell.sh/contributor-book/plugin_protocol_reference.html , plugins: https://www.nushell.sh/book/plugins.html
- Zellij permissions: https://zellij.dev/documentation/plugin-api-permissions , PRs: https://github.com/zellij-org/zellij/pull/4449 , https://github.com/zellij-org/zellij/pull/5662
- Tauri capabilities: https://v2.tauri.app/security/capabilities/
- Wasmtime: https://docs.wasmtime.dev/stability-tiers.html , https://docs.wasmtime.dev/stability-release.html
- Component Model: https://component-model.bytecodealliance.org/introduction.html
- Helix Steel PR: https://github.com/helix-editor/helix/pull/8675
- Windows `CM_Register_Notification`: https://learn.microsoft.com/en-us/windows/win32/api/cfgmgr32/nf-cfgmgr32-cm_register_notification
- Windows device notification: https://learn.microsoft.com/en-us/windows/win32/devio/registering-for-device-notification
- Windows job objects: https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects
- Windows working set: https://learn.microsoft.com/en-us/windows/win32/memory/working-set
- Windows named pipe security: https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights
- `GetNamedPipeClientProcessId`: https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getnamedpipeclientprocessid
- `PowerRegisterSuspendResumeNotification`: https://learn.microsoft.com/en-us/windows/win32/api/powerbase/nf-powerbase-powerregistersuspendresumenotification

**Repo-local context**
- [2026-11-08-universal-pc-control-tools.md](2026-11-08-universal-pc-control-tools.md)
- [2026-11-08-armoury-replacement-plan.md](2026-11-08-armoury-replacement-plan.md)
- [2026-11-04-low-ram-react-monitor-stack.md](2026-11-04-low-ram-react-monitor-stack.md)
- Safety rules: [AGENTS.md](../../AGENTS.md)
