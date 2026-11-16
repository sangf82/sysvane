---
title: "The colour preset code is checked against the MCCS list, not the monitor's reported maximum"
author: Sangf82
date: 2026-11-15
branch: main
cost: { minutes: 68, tokens: 644407 }
id: l-2abab4a4
scope: repo
status: active
src: agent
touches:
  - docs/handoff.md
  - desktop/src/pages/Displays.tsx
  - core/src/modules/monitor/mode.rs
  - core/src/modules/monitor/ddc.rs
  - desktop/src/pages/Performance.tsx
  - desktop/src/pages/Overview.tsx
  - desktop/src/styles.css
  - desktop/src/pages/processTable.ts
  - desktop/src/components/ProcIcon.tsx
  - desktop/src/components/glyphs.tsx
---

## Decided
The guarded DDC write checks VCP 0x14 (colour preset) against the MCCS values 1 to 13, not against the maximum the monitor reports. Every other writable code is still checked against the reported maximum.

## Tried and ruled out
Checking 0x14 against the reported maximum: the Acer AL258Q (CVTE scaler) reports max 5 for 0x14, so Calibrated-Right (0x14 = 11, User 1) was refused before anything was sent. 0x14 is a list of values, not a range, so its "maximum" means nothing.

## Watch out
After writing 0x14 = 11 the AL258Q still read back 5, even after a rescan (2026-11-16). Whether the monitor really switched to User mode is not confirmed yet: check the monitor's own menu.

In the code: core/src/modules/monitor/vcp.rs, `write` and `PRESETS`; test `a_preset_is_checked_against_the_mccs_list_not_the_reported_maximum`.
