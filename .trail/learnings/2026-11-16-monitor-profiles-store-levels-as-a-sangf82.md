---
title: "Monitor profiles store levels as a percentage of each monitor's maximum, and the AL258Q declares only colour preset 5"
author: Sangf82
date: 2026-11-16
branch: main
cost: { minutes: 14, tokens: 143919 }
id: l-f048b167
scope: repo
status: active
src: agent
touches:
  - core/src/modules/monitor/ddc.rs
  - core/src/modules/monitor/vcp.rs
  - core/src/modules/monitor/profile.rs
  - core/src/modules/monitor/mod.rs
  - core/src/modules/mod.rs
  - core/src/modules/devices.rs
  - desktop/src/pages/Displays.tsx
  - docs/handoff.md
---

## Decided
Continuous VCP codes (brightness, contrast, gains, 6-axis, black levels, volume) are a percentage of the monitor's own maximum in the UI, in profiles and in the guarded write, so a profile means the same on a monitor that counts to 255. Preset, picture mode and 0x0C stay raw. Picture mode (0xB6) is model specific: a profile writes it only to a monitor of the model it was captured on. The UI preset list comes from the capabilities string.

## Tried and ruled out
Raw values in profiles: 70 on a 0..255 monitor is 27%, and a value above that monitor's maximum was refused.

## Watch out
Both AL258Q (scaler CVTE) report `14(05)` in their capabilities: DDC offers preset 5 (6500K) only, which explains why writing 11 (User 1) is ignored. A user profile without "model" no longer writes 0xB6.

In the code: core/src/modules/monitor/vcp.rs (scaled, model_specific, Caps, write), profile.rs (Profile.model), wire Device.presets.
