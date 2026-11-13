# SysVane

A very low-RAM, tray-first Windows app for hardware monitoring, displays, peripherals, RGB, fans and AIO LCD screens. It is built to replace ASUS Armoury Crate and vendor device suites.

Early stage: monitoring works (CPU, memory, GPU, disks, network, processes, a tray icon, a flyout and a dashboard). Device control (displays, LCD, RGB, fans) is not built yet. See [docs/plan.md](docs/plan.md).

## Layout

- `core/` — background program, Rust.
- `desktop/` — flyout and dashboard, Tauri 2 + React/Vite.
- `wire/` — what the core and the UI share: pipe name, process names, snapshot types.
- `modules/ddc/` — monitor calibration profiles and VCP notes.
- `branding/logo/` — logo kit and the scripts that build it.
- `DESIGN.md` — design system.

## Requirements

- Windows 11.
- Rust with the MSVC toolchain and Visual Studio C++ Build Tools.
- Node.js.

## Development

```powershell
cargo build -p sysvane-core   # the UI starts sysvane-core.exe from its own folder
cd desktop
npm install
npm run tauri dev
```
