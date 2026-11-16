---
title: Two monitors of one model are told apart by their EDID serial only when the maker filled it in
author: Sangf82
date: 2026-11-16
branch: main
cost: { minutes: 22, tokens: 137883 }
id: l-b7f2ef9d
scope: repo
status: active
src: agent
touches:
  - desktop/src-tauri/Cargo.toml
  - desktop/src-tauri/src/icon.rs
  - desktop/src-tauri/src/lib.rs
  - desktop/src/components/ProcIcon.tsx
  - desktop/src/components/meters.tsx
  - desktop/src/lib/series.ts
  - desktop/src/pages/Overview.tsx
  - desktop/src/pages/Performance.tsx
  - desktop/src/pages/Processes.tsx
  - desktop/src/pages/processTable.ts
---

## Decided
A monitor's GPU colour file is keyed by its EDID id (maker, product, serial) so the setting follows the monitor to another port. When the serial is empty or all zeros the display number is used instead.

## Tried and ruled out
Keying by EDID id alone: both AL258Q report the serial "0000000000000" (maker ICD, product 2450), so they shared one file and one monitor's colour went to the other.

## Watch out
The EDID name ("AL258Q") is the right display name; the capabilities "model" is the scaler ("CVTE") and only gates model-specific profile codes. `DeviceId::Monitor(u8)` is still the enumeration position, so commands can reach the wrong monitor after a Scan.

In the code: core/src/modules/monitor/edid.rs (Edid::parse, id None for a zero serial), mod.rs gpu_file.
