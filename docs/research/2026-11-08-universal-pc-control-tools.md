# Universal low-RAM PC control tools versus Armoury Crate

**Date:** 2026-11-08
**Scope:** One Windows 11 desktop: ASUS TUF GAMING B850M-PLUS WIFI, Ryzen 9 9900X, RTX 5070 Ti, ROG OMNI RECEIVER, SteelSeries Arctis Nova 3P Wireless, 2x Acer AL258Q over DisplayPort. Web and source research only. Nothing was installed or changed, and no hardware was probed. The UI-shell question is settled in [2026-11-04-low-ram-react-monitor-stack.md](2026-11-04-low-ram-react-monitor-stack.md) and is not repeated here. Section 8 widens the target to a tray-first app for any Windows PC or laptop. macOS and Linux are out of scope; Linux sources are cited only where they document hardware behavior.

## 1. Direct answer

**No single tool does this.** The closest claim is RIGStats, which "started as a replacement for ROG Armoury Crate" and covers sensors, fan curves, Aura and ROG mouse settings ([README](https://github.com/dvalfrid/rigstats)). A text search of its README and device list found no DDC/CI control and no Arctis Nova 3 entry ([README](https://raw.githubusercontent.com/dvalfrid/rigstats/main/README.md), [devices](https://raw.githubusercontent.com/dvalfrid/rigstats/main/docs/supported-devices.md)). It is also not low-RAM: the project's own baseline is about 220 MB working set and 355 MB private for the app ([architecture.md](https://raw.githubusercontent.com/dvalfrid/rigstats/main/docs/architecture.md)).

A truly universal tool cannot exist today, because mouse, keyboard and headset settings have no shared protocol (section 4.7). The realistic target is **almost nothing resident**: let the firmware and devices hold their settings, and run small tools only when changing something.

**How to read the RAM numbers.** "Measured" means someone recorded it with a stated method. "Anecdotal" means an issue-tracker report. "Vendor" means the project's own claim. "Estimate" is my judgement with no source. I found **no independent, methodical idle-RAM measurement** for OpenRGB, FanControl, LibreHardwareMonitor, HWiNFO, G-Helper, Monitorian or HeadsetControl.

## 2. Comparison table

| Tool | Controls | RAM | License | Last release (checked 2026-11-08) | Kernel driver |
|---|---|---|---|---|---|
| **Armoury Crate** | Aura, Fan Xpert, device updates, scenario profiles, system monitor ([ASUS](https://www.asus.com/us/support/faq/1055115/)) | **Measured on this PC:** about 848 MB private across the process tree | Proprietary | Not checked | ASUS drivers; which ones are active here was not checked |
| **OpenRGB** | RGB only ([site](https://openrgb.org/)) | Not found. Vendor says only that it aims to use little RAM | GPL-2.0-or-later ([site](https://openrgb.org/), [repo](https://codeberg.org/OpenRGB/OpenRGB)) | 1.1, 2026-10-16 ([releases](https://codeberg.org/OpenRGB/OpenRGB/releases)) | PawnIO, for "RAM, older motherboards, and certain other devices" ([1.1 notes](https://codeberg.org/OpenRGB/OpenRGB/releases/tag/release_1.1)) |
| **FanControl** (Rem0o) | Fan curves, with sensors via LHM ([README](https://github.com/Rem0o/FanControl.Releases)) | **Anecdotal:** 20+ MB just after closing the window, 400+ MB hours later on V284 ([#4247](https://github.com/Rem0o/FanControl.Releases/issues/4247)); 100 MB growing to 900 MB in 2025 ([#2981](https://github.com/Rem0o/FanControl.Releases/issues/2981)). Vendor: "Low resource usage" | Proprietary, closed, personal non-commercial, no redistribution ([LICENSE](https://raw.githubusercontent.com/Rem0o/FanControl.Releases/master/LICENSE)) | V285, 2026-11-02 ([releases](https://github.com/Rem0o/FanControl.Releases/releases)) | PawnIO since V238 ([README](https://github.com/Rem0o/FanControl.Releases)) |
| **LibreHardwareMonitor** | Sensors, plus fan control through its library ([README](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor)) | Not found for current builds. **Anecdotal, 2019:** about 20 MB rising to 120 MB from sensor history ([#74](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/issues/74)) | MPL-2.0 ([README](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor)) | v0.9.6, 2026-02-14 ([release](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/releases/tag/v0.9.6)); master is active | PawnIO; the app bundles its installer ([tree](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/tree/master/LibreHardwareMonitor.Windows.Forms/Resources)) |
| **HWiNFO** | Sensors, read-only monitoring | Not found | Freeware plus paid Pro. Terms not read, site returned 403 | Could not confirm. Third-party listings show 8.50, July 2026 ([PC-Welt](https://www.pcwelt.de/?p=1082662)) | Own driver, `HWiNFO64A.SYS`, which the blocklist names (section 4.1) |
| **G-Helper** | ASUS laptop controls, plus ASUS mice, keyboards, headsets ([README](https://github.com/seerge/g-helper)) | Not found. Vendor: "much smaller footprint" | GPL-3.0 | v0.291, 2026-11-02 ([releases](https://github.com/seerge/g-helper/releases)) | Uses the ASUS ATK driver if present, but desktops are allowed to run without it ([Program.cs](https://github.com/seerge/g-helper/blob/main/app/Program.cs)). PawnIO for RyzenSMU |
| **RIGStats** | Sensors, fan curves, Aura, Dynamic Lighting, ROG mouse and headset settings ([README](https://github.com/dvalfrid/rigstats)) | **Measured by the project:** about 220 MB working set, 355 MB private ([architecture.md](https://raw.githubusercontent.com/dvalfrid/rigstats/main/docs/architecture.md)) | MIT | v1.53.0, 2026-10-31 ([releases](https://github.com/dvalfrid/rigstats/releases)) | PawnIO through embedded LHM; installs a Windows service ([README](https://raw.githubusercontent.com/dvalfrid/rigstats/main/README.md)) |
| **OpenCrate** | Certain ASUS lighting, fan, power ([README](https://github.com/yibudak/opencrate)) | Not found | MIT | v0.1.3, 2026-09-14 ([releases](https://github.com/yibudak/opencrate/releases)) | Not stated. Fan control **requires the ASUS fan service** to stay installed |
| **SignalRGB** | RGB; Pro adds fan curves and dashboards ([docs](https://docs.signalrgb.com/)) | **Anecdotal:** leak-like growth over days ([TechEnclave](https://techenclave.com/t/signalrgb-unusual-memory-consumption-issues-memory-leak/259515)) | Proprietary, freemium | Not checked | Not found |
| **Windows Dynamic Lighting** | HID LampArray lighting only ([Microsoft](https://learn.microsoft.com/en-us/windows/apps/develop/devices-sensors/lighting-dynamic-lamparray)) | Part of Windows | Part of Windows | Part of Windows | None |
| **Twinkle Tray** | DDC/CI brightness, contrast, arbitrary VCP ([README](https://github.com/xanderfrangos/twinkle-tray)) | **Estimate:** Electron class, empty Electron measured about 279 MB in the [prior note](2026-11-04-low-ram-react-monitor-stack.md) | MIT | v1.17.2, 2025-12-25 | None found. Its CLI needs the app running |
| **Monitorian** | DDC/CI brightness, contrast; CLI is a paid add-on ([README](https://github.com/emoacht/Monitorian)) | Not found | MIT | 4.16.11, 2026-11-08 | None found |
| **ClickMonitorDDC** | DDC/CI brightness, contrast | Not found | Freeware | 7.2, June 2020; listed as abandoned ([Softpedia](https://www.softpedia.com/progChangelog/ClickMonitorDDC-Changelog-250670.html), [AlternativeTo](https://alternativeto.net/software/clickmonitordcc/about/)) | None found |
| **ControlMyMonitor** (NirSoft) | Any VCP code, one-shot CLI ([NirSoft](https://www.nirsoft.net/utils/control_my_monitor.html)) | Zero resident in one-shot use | Freeware, no commercial bundling | 1.43 | None found |
| **winddcutil** | ddcutil-style CLI over `monitorcontrol` ([repo](https://github.com/scottaxcell/winddcutil)) | Zero resident | MIT | Last commit 2023-09-15 | None. Same backend this repo already uses |
| **HeadsetControl** | Headset settings per device ([README](https://raw.githubusercontent.com/Sapd/HeadsetControl/master/README.md)) | Zero resident in one-shot use | GPL-3.0 | 4.2.0, 2026-10-20 ([releases](https://github.com/Sapd/HeadsetControl/releases)) | None found; talks HID |
| **Halo Battery** | Battery tray for many brands ([README](https://github.com/HeyOkay/HaloBattery)) | Not found | MIT | v1.15.0, 2026-10-12 | None found. Omni [PR #185](https://github.com/HeyOkay/HaloBattery/pull/185) is unmerged |
| **liquidctl** | AIOs, some Aura LED boards, one-shot CLI ([README](https://github.com/liquidctl/liquidctl)) | Zero resident | GPL-3.0 | v1.16.0, 2026-03-03 | None, but some devices need WinUSB via Zadig |

**Not useful here.** OpenLinkHub is Corsair on [Linux](https://github.com/jurkovic-nikola/OpenLinkHub). asusctl is for [ROG laptops on Linux](https://gitlab.com/asus-linux/asusctl). libratbag's Omni support is an [open Linux request](https://github.com/libratbag/libratbag/issues/1793). Solaar is Logitech and Bazecor is Dygma; for those two I checked only that they are active, not their feature lists. ROG Ally tools target handheld firmware and I did not pursue them. I could not identify any ASUS-related tool called "AATool".

## 3. What each claim rests on

- Release dates come from each project's own release API on 2026-11-08.
- Source-code claims were read from the current default branch on 2026-11-08 and may change.
- The two ASUS PDF manuals are encrypted against text extraction here. Their content below comes from search-engine excerpts of those PDFs, so check the printed manual before relying on exact menu names.

## 4. The seven specific questions

### 4.1 Driver risk: WinRing0 and PawnIO

**WinRing0.**
- CVE-2020-14979 covers WinRing0 drivers 1.2.0 as shipped in EVGA Precision X1, and allows local privilege escalation through physical memory access ([CVE record](https://cveawg.mitre.org/api/cve/CVE-2020-14979)).
- Microsoft says the `VulnerableDriver:WinNT/Winring0` detection "is valid" and names HWiNFO, Libre Hardware Monitor, OpenRGB, SteelSeries Engine and others as possibly affected. It does not recommend the exclusion workaround ([Microsoft Support](https://support.microsoft.com/en-us/windows/security/threat-malware-protection/microsoft-defender-antivirus-alert-vulnerabledriver-winnt-winring0)).
- Microsoft's downloadable blocklist (file dated 2026-05-12, policy 10.0.29545.0) contains 29 WinRing0 hash deny rules and a `WinRing0.sys` file rule up to version 2.0.0.0, plus an `OpenLibSys` rule ([blocklist zip](https://aka.ms/VulnerableDriverBlockList)).
- Microsoft says the Windows-shipped list is usually less complete than that download ([Microsoft Learn](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/design/microsoft-recommended-driver-block-rules)). So a given WinRing0 copy may or may not be blocked on this PC. Treat it as unusable regardless.

**PawnIO.**
- It is a scriptable kernel driver that runs signed Pawn modules and exposes IOCTLs, so programs do not get raw hardware access ([site](https://pawnio.eu/), [OpenRGB MR 2833](https://gitlab.com/CalcProgrammer1/OpenRGB/-/merge_requests/2833)).
- License is GPL-2.0 with an exception for independent modules using its IOCTL interface, and commercial licensing is available ([repo](https://github.com/namazso/PawnIO)).
- Latest setup is 2.2.0 ([release](https://github.com/namazso/PawnIO.Setup/releases/tag/2.2.0)).
- **Signing:** the author, namazso, signs the official build and the modules. In 2025 FACEIT blocked PawnIO because its signing certificate was shared with cheat software, and the author planned a dedicated certificate ([issue #1](https://github.com/namazso/PawnIO.Setup/issues/1)). FanControl's README says that was fixed with PawnIO 2.1.0 ([README](https://github.com/Rem0o/FanControl.Releases)). **I could not verify** whether the driver has Microsoft attestation or WHQL signing, or any anti-cheat's current stance beyond that line.
- **Adopters confirmed in source or release notes:** LibreHardwareMonitor, FanControl V238+, OpenRGB 1.0+, RIGStats, G-Helper (RyzenSMU), and ZenTimings 1.36 ([discussion](https://github.com/irusanov/ZenTimings/discussions/90)).
- PawnIO has zero entries in the May 2026 blocklist.

**What this means here.**
- The blocklist is enabled, and Microsoft says it can be on independently of memory integrity ([Microsoft Learn](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/design/microsoft-recommended-driver-block-rules)). Any tool still on WinRing0 should be expected to fail. PawnIO-based tools should load.
- HVCI is off on this PC, so PawnIO under HVCI does not matter today, and I found no authoritative answer for it.
- The blocklist also names `HWiNFO64A.SYS` up to file version 8.98.0.0 under specific signers. I could not tell whether current HWiNFO is affected. Launching it will show this quickly.
- The simplest way to remove this risk is to not install a third-party kernel driver at all. The recommended setup needs none.

### 4.2 The B850M-PLUS WIFI board

**Super I/O: very likely Nuvoton NCT6701D, not confirmed for this exact model.** LHM's source marks the sibling `TUF_GAMING_B850M_PLUS_II` and `TUF_GAMING_B850_BTF_WIFI_W` as NCT6701D ([SuperIOHardware.cs](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/blob/master/LibreHardwareMonitorLib/Hardware/Motherboard/SuperIOHardware.cs)), and the WIFI7 W sibling reports the same chip ([#2613](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/issues/2613)). HWiNFO on this PC will confirm it. ASUS's spec page does not name the chip ([spec](https://www.asus.com/motherboards-components/motherboards/tuf-gaming/tuf-gaming-b850m-plus-wifi/techspec/)).

**LHM and FanControl: unreliable on this board family.**
- This exact name is not in LHM's model table, so it falls to `Model.Unknown` ([Identification.cs](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/blob/master/LibreHardwareMonitorLib/Hardware/Motherboard/Identification.cs)).
- The issue for this board, [#1672](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/issues/1672), is still open. A May 2025 comment reported CPU fan, pump and chassis 1 detected, chassis 2 to 4 missing.
- An NCT6701D fix merged in August 2026 ([PR #2417](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/pull/2417)), after the latest tagged release.
- The WIFI7 W sibling still reads 0 RPM on all 7 fans and 100% on all controls in LHM master and FanControl V284, while HWiNFO reads correctly ([#2613](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/issues/2613), opened 2026-10-26, no maintainer reply).
- **A possible cause worth testing:** on another NCT6701D board, turning off Fan Xpert control in Armoury Crate and rebooting fixed empty readings ([#1610 comment](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/issues/1610#issuecomment-2645197554)), and a B850M user reported that fix worked ([FanControl #3250](https://github.com/Rem0o/FanControl.Releases/issues/3250)). So readings may improve after Armoury Crate is gone. Whether PWM writes work on this board is unverified.

**OpenRGB and Aura: probably yes over USB HID, with limits.**
- OpenRGB detects ASUS mainboard controllers by USB PID, including `0x18F3`, `0x1939`, `0x19AF`, `0x1AA6` and `0x1BED` ([detector](https://codeberg.org/OpenRGB/OpenRGB/src/branch/master/Controllers/AsusAuraUSBController/AsusAuraUSBController/AsusAuraUSBControllerDetect.cpp)). That path needs no PawnIO.
- The sibling TUF B850M-PLUS reports `0B05:1BED` ([#5304](https://gitlab.com/CalcProgrammer1/OpenRGB/-/issues/5304)). This exact board's PID is unverified.
- On that sibling, under Linux on 1.0, **Direct mode works but no hardware effect mode does** ([#5755](https://gitlab.com/CalcProgrammer1/OpenRGB/-/issues/5755), open). Newer NEO boards use `0B05:1CD2`, which is not in the list.
- Addressable headers (three Gen 2 per [ASUS](https://www.asus.com/motherboards-components/motherboards/tuf-gaming/tuf-gaming-b850m-plus-wifi/techspec/)) are read from the controller's config table, so they should appear if the controller is detected.

### 4.3 ROG OMNI RECEIVER

**RGB via OpenRGB: no native support.** The request for `0B05:1ACE` has been open since August 2023 ([#3558](https://gitlab.com/CalcProgrammer1/OpenRGB/-/issues/3558)), and neither the mouse device table nor the Aura keyboard detector mentions it ([source](https://codeberg.org/OpenRGB/OpenRGB/src/branch/master/Controllers/AsusAuraUSBController)). OpenRGB does have a generic HID LampArray detector ([source](https://github.com/CalcProgrammer1/OpenRGB/blob/master/Controllers/HIDLampArrayController/HIDLampArrayControllerDetect.cpp)), so a device behind the receiver could appear if it exposes LampArray. I could not verify that.

**G-Helper: yes, and it is the best fit.**
- Current source reads the receiver's pair list and maps paired devices to models ([PeripheralsProvider.cs](https://github.com/seerge/g-helper/blob/main/app/Peripherals/PeripheralsProvider.cs)).
- Listed over Omni: Harpe Ace Aim Lab, Harpe Ace Mini, Harpe Ace Extreme, Keris II Ace, Keris II Origin, Strix Impact III Wireless, Gladius III AimPoint, Keris Wireless AimPoint; keyboards Azoth, Azoth Extreme, Strix Scope II 96 and 96 RX, Falchion RX Low Profile.
- It exposes battery, DPI, polling rate, profiles and lighting, and has a save-to-device command ([AsusMouse.cs](https://github.com/seerge/g-helper/blob/main/app/Peripherals/Mouse/AsusMouse.cs)).
- It starts on desktops without the ATK driver ([Program.cs](https://github.com/seerge/g-helper/blob/main/app/Program.cs)). The README still markets it for laptops, and the author warned that most other UI will have no effect there ([#1676](https://github.com/seerge/g-helper/issues/1676)).

**Other options.** RIGStats has hardware-verified Omni support for Azoth X, Harpe Ace Aim Lab and Harpe Ace Mini, and marks most other models unverified ([devices](https://raw.githubusercontent.com/dvalfrid/rigstats/main/docs/supported-devices.md)). SignalRGB is contradictory: a user reported Omni as unsupported on 2.4 GHz ([forum](https://forum.signalrgb.com/t/strix-scope-ii-96-wireless/1786)).

**Can Armoury Crate go after configuration? Very likely yes, with one exception.** ASUS documents onboard profiles on these mice, for example 5 profiles on the Harpe II Ace ([ASUS FAQ](https://rog.asus.com/me-en/support/faq/1055698/)), and the Keris II Ace page lists onboard DPI and polling adjustment ([ASUS](https://rog.asus.com/mice-mouse-pads/mice/wireless/rog-keris-ii-ace/)). Onboard memory does not cover macros or Windows shortcuts on those models. The exception is **pairing**: other pairings need Armoury Crate, and the pairing then persists ([#3558](https://gitlab.com/CalcProgrammer1/OpenRGB/-/issues/3558)). Do any pairing and firmware update before removal. The exact models here are unknown, so this needs checking against them.

### 4.4 Arctis Nova 3P Wireless

**Yes, HeadsetControl supports it**, for sidetone, battery, inactive time, EQ presets, EQ, parametric EQ and microphone volume ([README](https://raw.githubusercontent.com/Sapd/HeadsetControl/master/README.md)). The device file covers USB PID `0x2269` and calls a save-state command after writes, so settings appear to persist in the headset ([source](https://github.com/Sapd/HeadsetControl/blob/master/lib/devices/steelseries_arctis_nova_3p_wireless.hpp)). The latest release has a fix specifically for this model. SteelSeries GG features outside that list, such as Sonar, are not replaced.

### 4.5 Removing Armoury Crate

**How.** Use ASUS's Armoury Crate Uninstall Tool from the support page's Driver & Utility tab, then restart. ASUS advises against Add or Remove Programs and registry cleaners ([ASUS FAQ](https://www.asus.com/support/faq/1041654/), [ROG guide](https://rog.asus.com/articles/guides/how-to-cleanly-uninstall-and-reinstall-armoury-crate/)). Its "don't permanently uninstall" warning is about laptops. Afterwards disable BIOS Tool > ASUS Armoury Crate > Download & Install ARMOURY CRATE app, or Windows may prompt to reinstall it ([ASUS FAQ](https://zentalk.asus.com/t5/faq/motherboard-how-to-disable-automatic-download-of-armoury-crate/ta-p/408076)). I could not verify that the tool removes `asComSvc`, `AsusCertService` or `GameSDK Service`. Check with `Get-Service` after it finishes.

**What is lost.** Fan Xpert software curves, Aura Sync and Aura Creator effects, device pairing, the firmware and driver Update Center, scenario profiles and game library, System Monitor and WiFi Q-Antenna ([ASUS](https://www.asus.com/us/support/faq/1055115/)). ASUS DriverHub is named as the direction for driver updates.

**What keeps working.**
- **Q-Fan:** ASUS's manual lists CPU_FAN, CPU_OPT and chassis headers as default "Q-Fan Controlled", while AIO_PUMP defaults to full speed ([manual](https://dlcdnets.asus.com/pub/ASUS/mb/SocketAM5/TUF_GAMING_B850M-PLUS_WIFI/E24115_TUF_GAMING_B850M-PLUS_WIFI_EM_WEB.pdf?model=TUF+GAMING+B850M-PLUS+WIFI), excerpt). Qfan Control sets a profile or manual curve in firmware ([BIOS manual](https://dlcdnets.asus.com/pub/ASUS/mb/13MANUAL/E25269_AMD_800_Series_BIOS_manual_EM_WEB.pdf?model=PRIME+B840M-A), excerpt). No resident process is needed for this, so **a resident fan tool is likely unnecessary** unless curves must follow GPU temperature. Confirm that BIOS curves resume on the first boot under load.
- **Aura:** the BIOS AURA key offers on, Stealth (all LEDs off) and Aura Only modes ([BIOS manual](https://dlcdnets.asus.com/pub/ASUS/mb/13MANUAL/E25269_AMD_800_Series_BIOS_manual_EM_WEB.pdf?model=PRIME+B840M-A), excerpt). Whether the last Armoury Crate effect stays in the controller after removal is **unverified**. Stealth is the only outcome documented without software.

### 4.6 Reuse from a custom app, and one-shot use

| Backend | How to reuse | Needs a process left running? | Apply and exit? |
|---|---|---|---|
| OpenRGB | SDK is a TCP protocol, default port 6742, version 6 in 1.0 ([docs](https://codeberg.org/OpenRGB/OpenRGB/src/branch/master/Documentation/OpenRGBSDK.md)), with clients for Python, C#, Rust and more. CLI has `--profile`, `--mode`, `--color`, `--device` ([cli.cpp](https://codeberg.org/OpenRGB/OpenRGB/src/branch/master/cli.cpp)) | **SDK yes:** it talks to a running `--server`. RAM cost not found. **CLI no** | **Yes via CLI**, but `cli.cpp` has no `SaveMode` call, so colors may revert after power loss or sleep. Needs a Task Scheduler re-apply |
| LibreHardwareMonitor | NuGet library for .NET ([README](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor)); the GUI app has `/data.json` and `/Sensor` Get and Set on port 8085 ([HttpServer.cs](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/blob/master/LibreHardwareMonitor.Windows.Forms/Utilities/HttpServer.cs)) | Library no, web interface yes | **Read once, yes.** A fan curve is a loop, so software curves stay resident. Needs PawnIO and admin |
| FanControl | Plugins only: DLLs that it discovers and loads to add sensors and controls ([wiki](https://github.com/Rem0o/FanControl.Releases/wiki/Plugins)) | It is a **host, not a library** | No. Closed source with no documented CLI. Not usable as a backend |
| HeadsetControl | CLI with `-o json`, plus C and C++ library ([README](https://raw.githubusercontent.com/Sapd/HeadsetControl/master/README.md)) | No | **Yes** |
| G-Helper | GUI application; no CLI found. GPL-3.0 if code is reused | While open | Run, set, close. Settings are written to the device |
| Windows `dxva2` | `SetVCPFeature` and friends, already used through `monitorcontrol` ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/lowlevelmonitorconfigurationapi/nf-lowlevelmonitorconfigurationapi-setvcpfeature)) | No | **Yes**, and it needs no extra driver |

### 4.7 Is there a universal peripheral protocol?

**Only in fragments. Per-vendor reverse engineering is unavoidable for peripheral settings.**

- **Monitors:** VESA DDC/CI and MCCS is a genuine standard, though Microsoft warns many monitors implement it incompletely ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/lowlevelmonitorconfigurationapi/nf-lowlevelmonitorconfigurationapi-setvcpfeature)).
- **Lighting:** HID LampArray (usage page 0x59) is an open standard that Windows drives natively ([Microsoft](https://learn.microsoft.com/en-us/windows-hardware/design/component-guidelines/dynamic-lighting-devices)). RIGStats verified a Harpe Ace through the Omni receiver with this path, after setting a device toggle ([devices](https://raw.githubusercontent.com/dvalfrid/rigstats/main/docs/supported-devices.md)).
- **Battery:** the HID usage tables define a Battery System page, 0x85 ([HUT 1.4](https://www.usb.org/sites/default/files/hut1_4.pdf), table of contents). Real tools here still use vendor reports.
- **DPI, polling rate, sidetone, EQ, onboard profiles:** the HUT table of contents shows no such chapter, though I could not text-search the whole PDF. The evidence from code is stronger: G-Helper has one class per model, HeadsetControl has one file per device on vendor usage page `0xFFC0`, and the Omni receiver needs its own report ID and endpoint ([libratbag #1793](https://github.com/libratbag/libratbag/issues/1793)).
- **Fans:** Super I/O chips differ per board, and even the same chip needs per-board mapping, as section 4.2 shows.

Any "universal" product is therefore an aggregator of per-device drivers. It will always lag new hardware, and this PC's board and receiver are both at that edge.

## 5. Recommendation for this exact PC

| Need | Choice | Resident RAM |
|---|---|---|
| Fans | BIOS Q-Fan curves | 0 MB |
| Sensors | HWiNFO, opened only when needed | 0 MB (estimate) |
| Monitors | This repo's existing `toushou` CLI and profiles | 0 MB |
| Mouse, keyboard | Configure onboard, using G-Helper only when changing settings | 0 MB |
| Headset | HeadsetControl one-shot | 0 MB |
| RGB | BIOS Stealth, or `OpenRGB --profile` at logon if colors are wanted | 0 MB |
| **Total resident** | No third-party kernel driver needed if OpenRGB stays on USB HID and sensors use HWiNFO on demand | **About 0 MB (estimate) versus 848 MB measured** |

**If tray comforts are wanted later,** add per item and measure:

| Addition | RAM |
|---|---|
| FanControl, for GPU-temperature curves | 20 to 100 MB anecdotal, with reports of 400 to 900 MB after hours; PawnIO required and this board is doubtful |
| OpenRGB left running, if Direct mode will not persist | 30 to 80 MB estimate, no measurement found |
| Monitorian tray | 30 to 60 MB estimate, no measurement found |

That comes to roughly 80 to 240 MB in estimates. FanControl's reported drift could push it much higher.

**Suggested order, none of it done yet.**
1. In Armoury Crate, finish pairing, firmware updates and onboard profiles, and set Aura as desired.
2. Set Q-Fan curves in BIOS, including the AIO pump header if one is connected.
3. Run the official Uninstall Tool, restart, disable the BIOS auto-install option, and check leftover services.
4. Stress the PC and check that fans respond. Read the Super I/O name in HWiNFO.
5. Check the board's Aura PID with `Get-PnpDevice -PresentOnly | Where-Object InstanceId -match 'VID_0B05'`, and test OpenRGB only if RGB matters.

## 6. Build versus combine

**Combine. Build only a very thin script.**

**Worth building:** extend the existing `toushou` CLI, or keep a short script, so that one "apply my setup" command runs the DDC profile, `headsetcontrol`, and optionally `OpenRGB --profile`, triggered at logon by Task Scheduler. It is a small amount of glue, runs and exits, and adds nothing resident. The DDC/CI safety layer in this repo remains its real value, since no other tool here enforces the AL258Q color-preset order or the hard write bans.

**Not worth building:**
- A custom universal shell. A window costs about 190 to 310 MB of WebView2 while open ([prior note](2026-11-04-low-ram-react-monitor-stack.md)), for settings changed rarely, and every button would just launch a tool that already has its own UI.
- A fan-control backend. Needs a kernel driver, per-board mapping and a resident loop, and LHM itself is struggling on this chip. Q-Fan already does the job in firmware.
- Omni or Arctis protocol drivers. G-Helper, RIGStats and HeadsetControl have done that work.
- An OpenRGB SDK integration. It forces a resident server to save a one-line CLI call.
- Wrapping FanControl. It is closed and is a plugin host only.

**Reconsider only if** several settings need live cross-device logic that firmware cannot hold, such as per-game profiles spanning monitors, headset and lighting. Even then, keep it headless.

## 7. Risks and unknowns

- **Uninstalling is a real change.** Rollback is reinstalling Armoury Crate, but re-pairing and restoring Aura depend on it being available. Custom profile retention across uninstall is undocumented ([ROG guide](https://rog.asus.com/articles/guides/how-to-cleanly-uninstall-and-reinstall-armoury-crate/)).
- **Fan safety.** If BIOS curves are misconfigured, or a fan is on a header with an unexpected default, cooling suffers. Test under load right after removal. Any software fan tool can also fail in ways firmware does not.
- **Exact models unconfirmed.** Super I/O chip, Aura USB PID, and the keyboard and mouse models were not identified. Several conclusions rely on sibling boards.
- **RGB persistence is unverified.** Effects may not survive sleep or power loss without something re-applying them, and OpenRGB hardware modes are reported broken on a sibling board.
- **G-Helper is built for laptops.** It can stop and disable ASUS services through its own Services action, and re-stops them on launch if that was used ([AsusService.cs](https://github.com/seerge/g-helper/blob/main/app/Helpers/AsusService.cs)). Most of its UI will do nothing on a desktop.
- **PawnIO signing and anti-cheat.** Microsoft signing status and current anti-cheat acceptance are unverified. A driver with this capability stays attack surface, even if safer than WinRing0.
- **HWiNFO unverified.** Its current version, driver status against the blocklist, and free-tier terms could not be read because hwinfo.com returned 403.
- **RAM numbers are thin.** The "0 MB" total is by design, but all optional tray figures are estimates or issue reports. The 848 MB baseline is private bytes, so a before and after comparison should use the same counter.
- **Fast-moving projects.** FanControl, G-Helper, RIGStats and OpenRGB all shipped within the last month, so support details may change soon.

## 8. Extension: a tray-first app for any Windows PC or laptop

**Bottom line.** A universal app is feasible only as a capability-detecting shell. The standards cover load metrics, GPU telemetry, monitors, laptop panel brightness, Bluetooth battery and LampArray lighting, all without a kernel driver. CPU temperature, board fans, laptop fans, and most peripheral settings are per-device, and several need a kernel driver whose signing future is now uncertain.

### 8.1 Laptop fan control

**Windows has no fan-control API.** `Win32_Fan.SetSpeed` is documented as "Not implemented" ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/win32-fan)). Every tool below goes through a vendor firmware method or the embedded controller (EC).

| Vendor | Interface used by open tools | Third-party kernel driver? | Status and limits |
|---|---|---|---|
| **ASUS** | `\\.\ATKACPI` IOCTLs with `DSTS`/`DEVS` calls, plus WMI class `AsusAtkWmi_WMNB`, with fan curve IDs per fan ([AsusACPI.cs](https://github.com/seerge/g-helper/blob/main/app/AsusACPI.cs)) | No, uses the ASUS System Control Interface driver | G-Helper v0.291, 2026-11-02. Actively maintained |
| **Lenovo Legion** | WMI `root\WMI`: `LENOVO_FAN_METHOD.Fan_Set_Table`, `LENOVO_GAMEZONE_DATA` ([WMI.LenovoFanMethod.cs](https://github.com/LenovoLegionToolkit-Team/LenovoLegionToolkit/blob/master/LenovoLegionToolkit.Lib/System/Management/WMI.LenovoFanMethod.cs)) | No, but needs Lenovo's Vantage Gaming Feature Driver ([README](https://github.com/BartoszCichecki/LenovoLegionToolkit)) | Original project archived July 2025. Team fork v2.36.0.0, 2026-09-08. Compatibility is per model |
| **Alienware and Dell G** | Alienware functions in ACPI BIOS, the same calls AWCC uses. Explicitly avoids direct EC access ([alienfx-tools](https://github.com/T-Troll/alienfx-tools)) | No | v9.6.2.1, 2026-10-14. Says some models, such as m15/m17 R1, have no EC control |
| **Other Dell** | SMM. Linux documents per-model whitelists for disabling automatic BIOS fan control ([kernel docs](https://raw.githubusercontent.com/torvalds/linux/master/Documentation/hwmon/dell-smm-hwmon.rst)) | Likely yes on Windows; inferred from [DellFanManagement](https://github.com/AaronKelley/DellFanManagement) library names, not read | Last release 2021 |
| **HP Omen** | BIOS interface through `ACPI\PNP0C14`, plus EC routines ([OmenMon](https://github.com/OmenMon/OmenMon)) | Not stated for the BIOS path. A newer tool, OmenSuperHub, uses PawnIO for EC ([#106](https://github.com/breadeding/OmenSuperHub/issues/106), which reports raised `acpi.sys` DPC latency; anecdotal) | OmenMon last released 2024 |
| **MSI** | EC maps that differ per firmware; open work is Linux only ([msi-ec](https://github.com/BeardOverflow/msi-ec)) | Windows EC access would need one | No maintained open Windows path found |
| **Framework** | Official tool talks to `cros_ec` through a Framework Windows EC driver ([framework-system](https://github.com/FrameworkComputer/framework-system)); LHM has CrOS EC code | Vendor's own | BSD-3-Clause, active |
| **Generic EC** | NBFC per-model EC configs: 210 in the [Windows project](https://github.com/hirschmann/nbfc), 324 in [nbfc-linux](https://github.com/nbfc-linux/nbfc-linux) | Yes on Windows | Windows NBFC last released 2019 |

**Laptops with no control.** I found no authoritative list, and no vendor publishes one. Treat "no fan control" as the default for any laptop that does not match a verified adapter. Ordinary office and thin laptops generally have nothing documented; that is my inference from the absence of any interface above.

**EC write risk.**
- The Linux kernel labels EC writes "Dangerous" and warns they can confuse the EC so that "a normal reboot is not enough", requiring power off and battery removal ([ec_sys.c](https://github.com/torvalds/linux/blob/master/drivers/acpi/ec_sys.c), [Kconfig](https://github.com/torvalds/linux/blob/master/drivers/acpi/Kconfig)).
- EC layouts vary by firmware, and msi-ec warns "Use it at your own risk" even within one brand ([msi-ec](https://github.com/BeardOverflow/msi-ec)).
- alienfx-tools gives the safety case for firmware methods: "BIOS still monitors fans" so there is no risk of fans stopping under full load.
- **Rule for the product:** never write an EC on an unverified model. Use vendor firmware methods only, gated on an allowlist of DMI model plus BIOS version, always with a fallback that restores automatic control. This mirrors the repo's DDC safety rules, because neither has an undo.

### 8.2 Brightness: internal panels and where DDC/CI fails

**Internal panels use WMI, confirmed.** Microsoft's driver docs say `Monitor.sys` implements the WMI interface applications use for brightness on integrated panels, backed by the display driver or ACPI `_BCM` ([Microsoft](https://learn.microsoft.com/en-us/windows-hardware/drivers/display/supporting-brightness-controls-on-integrated-display-panels)). The app call is `WmiSetBrightness` on `WmiMonitorBrightnessMethods` in `root\wmi`, taking percent and timeout ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/wmicoreprov/wmisetbrightness-method-in-class-wmimonitorbrightnessmethods)). Monitorian reads through this same class ([MSMonitor.cs](https://github.com/emoacht/Monitorian/blob/master/Source/Monitorian.Core/Models/Monitor/MSMonitor.cs)).

Details that affect the app:
- Levels are 0 to 100, but a laptop need not support 0, and steps need not be uniform.
- WinRT `BrightnessOverride.GetDefaultForSystem` also works without a window, but the system may decline the request ([Microsoft](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.display.brightnessoverride)).

**Where DDC/CI fails on externals.**
- Monitorian lists the causes: the connector, or a cable, converter or dock that is not DDC/CI compatible. It says Thunderbolt and USB-C cables are generally fine, converters are not, and Surface Dock and Dock 2 are known to work ([README](https://github.com/emoacht/Monitorian)).
- **DisplayLink:** DisplayLink says its Windows software from V9.0 lets DDC/CI utilities adjust brightness and contrast ([DisplayLink](https://support.displaylink.com/knowledgebase/articles/1096210-usb-monitor-brightness-and-contrast-control-via-sc)). In practice Twinkle Tray has an open request because it cannot detect DisplayLink monitors ([#1115](https://github.com/xanderfrangos/twinkle-tray/issues/1115)), and Plugable says support varies with monitor, device and software ([Plugable](https://kb.plugable.com/docking-stations-and-video/can-i-adjust-the-brightness-of-a-display-connected-to-my-displaylink-based-docking-station-or-video-adapter-via-the-windows-%E2%80%98display-settings%E2%80%99-application)).
- **MST hubs:** I found no primary source on DDC/CI pass-through.
- **Other tray-app hazards:** requesting a capabilities string after hibernation caused partial freezes of several minutes on one Samsung ([Monitorian #800](https://github.com/emoacht/Monitorian/issues/800)). AMD reliability problems are reported ([#794](https://github.com/emoacht/Monitorian/issues/794), [NirSoft notice](https://www.nirsoft.net/utils/control_my_monitor.html)). Monitor identifiers can change with the connection type, including USB-C ([README](https://github.com/emoacht/Monitorian)).

### 8.3 Laptop keyboard RGB

Coverage is per model and there is no common path.

- **OpenRGB, current tree, Windows-capable by file or detector:** ASUS Aura Core laptops, ASUS TUF laptops (`_Windows` sources), HP Omen laptops (WMI, Windows only), MSI laptops through SteelSeries HID IDs such as Raider A18 and GE68HX, Lenovo 4-zone USB, Clevo ITE 8291 and 829x, Gigabyte Aorus laptops, Alienware, Acer Nitro HID, Razer, plus generic HID LampArray ([Controllers](https://github.com/CalcProgrammer1/OpenRGB/tree/master/Controllers)). Each is keyed to specific IDs or firmware interfaces, so a new model usually does not work until added.
- **Vendor tools:** G-Helper for ASUS, Lenovo Legion Toolkit for Spectrum and 4-zone ([README](https://github.com/BartoszCichecki/LenovoLegionToolkit)), alienfx-tools for Alienware over USB and ACPI.
- **Not researched:** a cross-vendor API for single-color keyboard backlight level. I did not look for one.

### 8.4 Standards versus per-device code

| Capability | Standard or documented interface | Real limits |
|---|---|---|
| CPU, memory, disk, network load | [PDH / performance counters](https://learn.microsoft.com/en-us/windows/win32/perfctrs/about-performance-counters), or lighter direct APIs | Not designed for more than one sample per second |
| GPU temperature, fan RPM, power | [`D3DKMT_ADAPTER_PERFDATA`](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/d3dkmthk/ns-d3dkmthk-_d3dkmt_adapter_perfdata) in the WDK | Cross-vendor, but depends on what each GPU driver fills in. Minimum version not stated |
| NVIDIA GPU | [NVML](https://docs.nvidia.com/deploy/nvml-api/nvml-api-reference.html), ships with the driver | NVIDIA only. NVIDIA says GeForce has "limited support" |
| AMD GPU | [ADLX](https://gpuopen.com/adlx/), with ADL as legacy | Windows 10+. GitHub reports no standard license id, so read its terms before shipping |
| Intel GPU | [IGCL](https://github.com/intel/drivers.gpu.control-library) | Not researched further |
| External monitors | VESA MCCS over DDC/CI through `dxva2` | Implementations are incomplete; see 8.2 |
| Bluetooth battery | [Battery Service 1.1](https://www.bluetooth.com/specifications/specs/battery-service/), [`GattServiceUuids.Battery`](https://learn.microsoft.com/en-us/uwp/api/windows.devices.bluetooth.genericattributeprofile.gattserviceuuids.battery) | Windows caches the level on the device node, which RIGStats reads with no device traffic ([BluetoothBatteries.cs](https://github.com/dvalfrid/rigstats/blob/main/sensor-sidecar/Control/BluetoothBatteries.cs)). No charging state |
| USB HID battery | Usages exist: Battery Strength `0x060020`, Battery page `0x85` ([Linux hid.h](https://github.com/torvalds/linux/blob/master/include/linux/hid.h)) | The gaming dongles here use vendor reports instead (sections 4.3, 4.4) |
| Lighting | HID LampArray, usage `0x59/0x01` | See below |
| CPU temperature | **None.** `Win32_TemperatureProbe.CurrentReading` is not populated ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/win32-temperatureprobe)) | LHM reads AMD sensors through a PawnIO module ([Amd17Cpu.cs](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/blob/master/LibreHardwareMonitorLib/Hardware/Cpu/Amd17Cpu.cs)) |

**Dynamic Lighting: real coverage is unknowable from Microsoft and is limited.**
- Devices qualify only through native HID LampArray firmware or a VHF driver. Microsoft's current page names partners, not models: all Logitech G LIGHTSYNC devices, "ASUS ROG plans to introduce", and "more details to come" for Acer and SteelSeries ([Microsoft](https://learn.microsoft.com/en-us/windows-hardware/design/component-guidelines/dynamic-lighting-devices), updated August 2024). Laptop and motherboard coverage has no current official list, so **detect at runtime** with `LampArray.GetDeviceSelector`.
- A device can have the capability switched off: ROG devices ignore changes until a "Cross-device Lighting Toggle" is set in Gear Link or Armoury Crate ([RIGStats devices](https://raw.githubusercontent.com/dvalfrid/rigstats/main/docs/supported-devices.md)).

**Can a third-party app control it? Yes, with constraints that hit a tray app directly.**
- The WinRT API applies state "only while application is in focus" ([LampArray](https://learn.microsoft.com/en-us/uwp/api/windows.devices.lights.lamparray)). A tray app has no focus.
- Background control needs Windows 11, package identity (MSIX or sparse package), the `com.microsoft.windows.lighting` AppExtension, and the user ranking the app in Settings. A foreground app wins by default ([Microsoft](https://learn.microsoft.com/en-us/windows/apps/develop/devices-sensors/lighting-dynamic-lamparray)).
- The alternative is raw HID on usage `0x59/0x01`, as OpenRGB and RIGStats do. It avoids packaging but must yield to Windows per device. RIGStats verified this on a Harpe Ace through the Omni receiver.

### 8.5 Shipping constraints

**Can a third party bundle PawnIO? It is done widely, but permission is not clearly stated.**
- The driver is GPL-2.0 with an exception for independent programs using only its IOCTL interface, and redistribution carries GPL source duties ([README](https://github.com/namazso/PawnIO)). Modules are LGPL-2.1 and must be signed ([modules](https://github.com/namazso/PawnIO.Modules)).
- LibreHardwareMonitor and RIGStats both embed the official installer; RIGStats pins it by SHA-256 ([installer.nsi](https://github.com/dvalfrid/rigstats/blob/main/build/installer.nsi)).
- However, the `PawnIO.Setup` repo has no license file detected by GitHub, and a request asking whether redistribution is permitted is unanswered as of today ([#13](https://github.com/namazso/PawnIO.Setup/issues/13)). The author offers separate licensing to commercial users ([site](https://pawnio.eu/)).
- **Safest options:** have the app download the official release and verify its hash, or get written permission. New hardware access needs a module signed by the author, because the official driver checks signatures.

**Kernel-driver signing today.**
- Since Windows 10 1607, new kernel drivers must be signed through Microsoft's portal, and a dashboard account needs an EV certificate ([Microsoft](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/kernel-mode-code-signing-policy--windows-vista-and-later-)). The PawnIO author notes EV certificates require a company ([#1](https://github.com/namazso/PawnIO.Setup/issues/1)).
- Attestation needs no HLK tests, but Microsoft now titles it "for testing scenarios", and says such drivers are not Windows Certified and cannot go to retail Windows Update ([Microsoft](https://learn.microsoft.com/en-us/windows-hardware/drivers/dashboard/driver-signing-offerings)).
- **April 2026 change:** Windows 11 24H2, 25H2, 26H1 and Server 2025 are moving to loading only WHCP-signed drivers plus an allow list, after an audit period of 250 hours and 3 reboots. Violations log Event 3076 or 3077 ([blog](https://techcommunity.microsoft.com/blog/windows-itpro-blog/advancing-windows-driver-security-removing-trust-for-the-cross-signed-driver-pro/4504818), [policy](https://support.microsoft.com/en-us/windows/hardware/drivers/the-windows-driver-policy)).
- **Unresolved and important:** neither Microsoft page says whether attestation-signed drivers pass this policy, and I could not determine how PawnIO's driver is signed. The string "namazso" appears in the installer, but the driver is packed inside it. Test on a machine past the audit period, or ask the author, before depending on PawnIO.

**Licensing.** This is my reading, not legal advice. The FSF FAQ was rate-limited and I could not read it.

| Component | In an open-source app | In a closed-source app |
|---|---|---|
| OpenRGB (GPL-2.0-or-later) | Fine, with compatible licensing | Ship or require it as a **separate program** driven by CLI or the SDK socket, include its license and source offer. Do not link it, or copy its controller code |
| LibreHardwareMonitor (MPL-2.0) | Fine | Allowed. Changes to MPL files stay MPL, and you must say where the source is ([MPL FAQ](https://www.mozilla.org/en-US/MPL/2.0/FAQ/)). Carry its [third-party notices](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/blob/master/THIRD-PARTY-NOTICES.txt). Still needs PawnIO |
| G-Helper, Legion Toolkit, OmenMon, HeadsetControl (GPL-3.0) | Fine under GPL-3.0 | Run as separate programs only. Do not port their code or device tables without legal advice |
| alienfx-tools, RIGStats (MIT), framework-system (BSD-3) | Fine | Fine, keep notices |
| FanControl | Cannot redistribute | Cannot redistribute ([LICENSE](https://raw.githubusercontent.com/Rem0o/FanControl.Releases/master/LICENSE)) |

### 8.6 Per-capability table

"Per-device" means code or data keyed to specific models or chips. "Driver" means a third-party kernel driver beyond what Windows or the vendor already installed.

| Capability | PC | Laptop | Standard or per-device | Kernel driver |
|---|---|---|---|---|
| CPU, RAM, disk, network load | Yes | Yes | Standard (PDH, Win32) | No |
| GPU load, temperature, fan RPM | Yes | Mostly; depends on GPU driver | Standard plus vendor SDKs | No |
| CPU temperature | Yes | Yes | Per CPU family; no Windows API | **Yes** |
| Board or EC sensors | Mostly | Partial | Per board or model | **Yes**, or vendor WMI |
| Fan curves | Firmware always; software varies by board | **Verified vendors only; default none** | Per device | Firmware no; software **yes**. Laptop vendor WMI no; EC **yes** |
| External monitor brightness and VCP | Yes | Yes, dock-dependent | Standard (MCCS) | No |
| Internal panel brightness | Not applicable | Yes | Standard (WMI) | No |
| Ambient-light adaptive brightness | Rare sensor; content fallback | Where a sensor exists | Standard (`LightSensor`, prior note) | No |
| System battery | Not applicable | Yes | Standard Windows API (not re-read today) | No |
| Charge limit, performance mode | Not applicable | Per vendor | Per device | Usually no |
| Bluetooth peripheral battery | Yes | Yes | Standard (GATT, Windows cache) | No |
| 2.4 GHz dongle battery | Per device | Per device | Per device | No |
| Peripheral RGB | LampArray where present, else per device | Same | Partial standard | No |
| Motherboard and ARGB | USB HID controllers, per board | Not applicable | Per device | No for USB; **yes** for SMBus |
| RAM RGB | Per module | Rare | Per device | **Yes** (SMBus) |
| Laptop keyboard RGB | Not applicable | Per model, or LampArray | Per device | Usually no |
| Mouse DPI, polling, profiles | Per device | Per device | Per device | No |
| Headset sidetone, EQ | Per device | Per device | Per device | No |

**Product consequence.** Ship a no-kernel-driver baseline: load metrics, GPU telemetry, DDC/CI with the existing safety layer, WMI panel brightness, Bluetooth battery and LampArray. Add vendor adapters one at a time, each showing up only when detected. Keep CPU temperature and desktop fan control as an optional, explicitly installed PawnIO module, and treat that module as at risk until its status under the 2026 driver policy is confirmed.

## 9. Sources

All accessed **2026-11-08**. Links cited only inline in section 8 are included there rather than repeated here.

**Drivers and security**
- Microsoft recommended driver block rules: https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/design/microsoft-recommended-driver-block-rules
- Microsoft blocklist download: https://aka.ms/VulnerableDriverBlockList
- Defender WinRing0 alert: https://support.microsoft.com/en-us/windows/security/threat-malware-protection/microsoft-defender-antivirus-alert-vulnerabledriver-winnt-winring0
- CVE-2020-14979: https://cveawg.mitre.org/api/cve/CVE-2020-14979
- PawnIO: https://pawnio.eu/ , https://github.com/namazso/PawnIO , https://github.com/namazso/PawnIO.Setup/releases/tag/2.2.0
- PawnIO signer and FACEIT: https://github.com/namazso/PawnIO.Setup/issues/1
- ZenTimings migration: https://github.com/irusanov/ZenTimings/discussions/90

**Sensors and fans**
- LibreHardwareMonitor: https://github.com/LibreHardwareMonitor/LibreHardwareMonitor
- LHM v0.9.6: https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/releases/tag/v0.9.6
- LHM board issues: https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/issues/1672 , https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/issues/2613 , https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/issues/1610#issuecomment-2645197554 , https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/pull/2417 , https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/issues/74
- LHM source: https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/blob/master/LibreHardwareMonitorLib/Hardware/Motherboard/Identification.cs , https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/blob/master/LibreHardwareMonitorLib/Hardware/Motherboard/SuperIOHardware.cs , https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/blob/master/LibreHardwareMonitor.Windows.Forms/Utilities/HttpServer.cs
- FanControl: https://github.com/Rem0o/FanControl.Releases , https://raw.githubusercontent.com/Rem0o/FanControl.Releases/master/LICENSE , https://github.com/Rem0o/FanControl.Releases/wiki/Plugins
- FanControl reports: https://github.com/Rem0o/FanControl.Releases/issues/4247 , https://github.com/Rem0o/FanControl.Releases/issues/2981 , https://github.com/Rem0o/FanControl.Releases/issues/3250
- HWiNFO listing, third party: https://www.pcwelt.de/?p=1082662

**RGB**
- OpenRGB: https://openrgb.org/ , https://codeberg.org/OpenRGB/OpenRGB , https://codeberg.org/OpenRGB/OpenRGB/releases/tag/release_1.1
- SDK docs: https://codeberg.org/OpenRGB/OpenRGB/src/branch/master/Documentation/OpenRGBSDK.md
- CLI: https://codeberg.org/OpenRGB/OpenRGB/src/branch/master/cli.cpp
- Aura USB source: https://codeberg.org/OpenRGB/OpenRGB/src/branch/master/Controllers/AsusAuraUSBController
- PawnIO merge request: https://gitlab.com/CalcProgrammer1/OpenRGB/-/merge_requests/2833
- OpenRGB issues: https://gitlab.com/CalcProgrammer1/OpenRGB/-/issues/3558 , https://gitlab.com/CalcProgrammer1/OpenRGB/-/issues/5304 , https://gitlab.com/CalcProgrammer1/OpenRGB/-/issues/5755
- SignalRGB: https://docs.signalrgb.com/ , https://forum.signalrgb.com/t/strix-scope-ii-96-wireless/1786 , https://techenclave.com/t/signalrgb-unusual-memory-consumption-issues-memory-leak/259515
- Dynamic Lighting: https://learn.microsoft.com/en-us/windows/apps/develop/devices-sensors/lighting-dynamic-lamparray , https://learn.microsoft.com/en-us/windows-hardware/design/component-guidelines/dynamic-lighting-devices
- HID Usage Tables 1.4: https://www.usb.org/sites/default/files/hut1_4.pdf

**Peripherals**
- G-Helper: https://github.com/seerge/g-helper , https://github.com/seerge/g-helper/releases , https://github.com/seerge/g-helper/issues/1676
- G-Helper source: https://github.com/seerge/g-helper/blob/main/app/Program.cs , https://github.com/seerge/g-helper/blob/main/app/Peripherals/PeripheralsProvider.cs , https://github.com/seerge/g-helper/blob/main/app/Peripherals/Mouse/AsusMouse.cs , https://github.com/seerge/g-helper/blob/main/app/Helpers/AsusService.cs
- RIGStats: https://github.com/dvalfrid/rigstats , https://raw.githubusercontent.com/dvalfrid/rigstats/main/docs/supported-devices.md , https://raw.githubusercontent.com/dvalfrid/rigstats/main/docs/architecture.md , https://github.com/dvalfrid/rigstats/releases
- OpenCrate: https://github.com/yibudak/opencrate
- HeadsetControl: https://raw.githubusercontent.com/Sapd/HeadsetControl/master/README.md , https://github.com/Sapd/HeadsetControl/blob/master/lib/devices/steelseries_arctis_nova_3p_wireless.hpp , https://github.com/Sapd/HeadsetControl/releases
- Halo Battery: https://github.com/HeyOkay/HaloBattery , https://github.com/HeyOkay/HaloBattery/pull/185
- libratbag Omni: https://github.com/libratbag/libratbag/issues/1793
- liquidctl: https://github.com/liquidctl/liquidctl
- OpenLinkHub: https://github.com/jurkovic-nikola/OpenLinkHub
- asusctl: https://gitlab.com/asus-linux/asusctl
- ASUS mice: https://rog.asus.com/me-en/support/faq/1055698/ , https://rog.asus.com/mice-mouse-pads/mice/wireless/rog-keris-ii-ace/

**Monitors**
- Monitorian: https://github.com/emoacht/Monitorian
- Twinkle Tray: https://github.com/xanderfrangos/twinkle-tray
- ControlMyMonitor: https://www.nirsoft.net/utils/control_my_monitor.html
- winddcutil: https://github.com/scottaxcell/winddcutil
- ClickMonitorDDC: https://www.softpedia.com/progChangelog/ClickMonitorDDC-Changelog-250670.html , https://alternativeto.net/software/clickmonitordcc/about/
- `SetVCPFeature`: https://learn.microsoft.com/en-us/windows/win32/api/lowlevelmonitorconfigurationapi/nf-lowlevelmonitorconfigurationapi-setvcpfeature

**ASUS**
- Armoury Crate FAQ: https://www.asus.com/support/faq/1041654/
- Clean uninstall guide: https://rog.asus.com/articles/guides/how-to-cleanly-uninstall-and-reinstall-armoury-crate/
- Armoury Crate 6 introduction: https://www.asus.com/us/support/faq/1055115/
- Disable automatic download: https://zentalk.asus.com/t5/faq/motherboard-how-to-disable-automatic-download-of-armoury-crate/ta-p/408076
- Board spec: https://www.asus.com/motherboards-components/motherboards/tuf-gaming/tuf-gaming-b850m-plus-wifi/techspec/
- Board manual (encrypted PDF, search excerpts only): https://dlcdnets.asus.com/pub/ASUS/mb/SocketAM5/TUF_GAMING_B850M-PLUS_WIFI/E24115_TUF_GAMING_B850M-PLUS_WIFI_EM_WEB.pdf?model=TUF+GAMING+B850M-PLUS+WIFI
- AMD 800 Series BIOS manual (encrypted PDF, search excerpts only): https://dlcdnets.asus.com/pub/ASUS/mb/13MANUAL/E25269_AMD_800_Series_BIOS_manual_EM_WEB.pdf?model=PRIME+B840M-A

**Repo-local context**
- UI shell research: [2026-11-04-low-ram-react-monitor-stack.md](2026-11-04-low-ram-react-monitor-stack.md)
- Safety rules: [AGENTS.md](../../AGENTS.md)
