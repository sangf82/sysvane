# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

The owner, on one ASUS Windows 11 desktop, working at it every day. Other people may install it later, since the code is meant to be open source, but the first version is built for this one machine and the owner's devices. The owner reads English and Vietnamese.

## Product Purpose

SysVane replaces ASUS Armoury Crate (about 848 MB measured) and Thermalright TRCC (about 300 MB, user-reported) with one very low-RAM, tray-first Windows app. It covers hardware monitoring, displays, peripherals, RGB, fans and the AIO LCD. Success is background RAM near zero when nobody looks at it, and a dashboard that answers "what is this machine doing" in seconds.

Confirmed in the 2026-11-09 interview: the job the dashboard must do best is controlling devices (monitors, RGB, fans, AIO LCD). Monitoring is the part built first and works today; control modules are not built yet.

## Positioning

One compiled-in Rust core that owns every hardware bus, with a UI that never touches hardware and exits when closed. A vendor suite cannot copy that without dropping its own background services. Each device module enforces its own safety rules in the core, not in the UI.

## Operating Context

- Windows 11, AMD Ryzen 9 9900X, NVIDIA GeForce RTX 5070 Ti, two Acer AL258Q monitors on DDC/CI, an AIO cooler with an LCD panel.
- The core runs in the tray all day; the flyout (320x480) and the dashboard open on demand.
- CPU temperature needs the PawnIO driver and an elevated helper, set up once from the tray menu.
- Documents: `docs/plan.md` (plan and status), `AGENTS.md` (rules), `DESIGN.md` (visual tokens), `docs/architecture.md` (file tree), `docs/devices/ddc/` (calibration and VCP notes).

## Capabilities and Constraints

- Working now: CPU, memory, GPU, disks, network and process monitoring; tray icon; autostart toggle; flyout; dashboard with Overview, Performance and Processes; CPU temperature through the elevated helper.
- Not built: LCD, DDC and calibration profiles, RGB, fans.
- Hard safety rules: no DDC factory-reset codes, no power-off values, no input-source writes without explicit consent; no OTA or firmware commands to the LCD panel. All enforced in the core.
- The core stays headless: no Tauri, WebView or React. Background RAM is the main goal.
- Interface languages: English and Vietnamese, switchable (confirmed by the owner). Hardware terms such as CPU, GPU, RAM and VRAM stay as they are.
- Open decision: whether other users get installers, and for which hardware beyond NVIDIA and AMD Ryzen.

## Brand Commitments

The name is SysVane, written exactly that way in display text. Identifiers stay lowercase (`sysvane-core`, `sysvane-ui`). Visual tokens come from `DESIGN.md`; emerald is the only chromatic accent.

## Evidence on Hand

- Measured on this machine (release build): core idle 1.8 MB private, about 7.6 MB with the dashboard connected, GPU worker only while connected.
- No user testimonials, benchmarks against other tools, or install counts exist. Do not invent any.

## Product Principles

1. Background cost first: nothing runs, samples or loads unless a window is looking at it.
2. Say "n/a" when a value is unreadable. Never show a guessed number.
3. The core owns every write to hardware. The UI sends intent, and the core decides whether it is safe.
4. Dense and readable beats decorative: one screen should answer a question without scrolling through chrome.
5. Anything that can blank a display or brick a device needs an explicit, named confirmation.

## Accessibility & Inclusion

Open decision: no formal standard is required yet. Working rules since the 2026-11 redesign: text contrast of at least 4.5:1 in both themes, full keyboard use of the sidebar, resource list and process table, and no state shown by colour alone.
