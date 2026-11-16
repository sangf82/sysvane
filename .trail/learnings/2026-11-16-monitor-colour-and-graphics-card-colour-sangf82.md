---
title: "Monitor colour and graphics card colour are shown one at a time behind a switch, both stay in force"
author: Sangf82
date: 2026-11-16
branch: main
cost: { minutes: 12, tokens: 562247 }
id: l-49bf8009
scope: repo
status: active
src: agent
touches:
  - core/Cargo.toml
  - core/src/modules/devices.rs
  - core/src/modules/mod.rs
  - core/src/modules/monitor/ddc.rs
  - core/src/modules/monitor/edid.rs
  - core/src/modules/monitor/mod.rs
  - core/src/modules/monitor/mode.rs
  - core/src/modules/monitor/profile.rs
  - core/src/modules/monitor/vcp.rs
  - core/src/system/tray.rs
---

## Decided
The Displays page keeps both colour sources: DDC gains live in the monitor and are what profiles save; the GPU gamma ramp works on any monitor and sits on top. A switch (Monitor (DDC) | Graphics card) picks which sliders show; it changes nothing on the hardware. Profiles sit first under the monitor cards so the owner can pick one quickly; then Display, Picture, Colour, each its own full-width panel.

## Tried and ruled out
Showing both colour groups side by side or stacked: read as duplicate settings. A coloured dot or square for the white point in the section header: the owner took it for a radio button, then a checkbox, and clicked it. Three columns of sections and one 760 px column inside a wide panel: both looked off-centre.

## Watch out
`.mhead` is already the meter-row header in components/meters.tsx; the section header is `.sechead`. Native range inputs draw a near-black track on the light theme, so the slider is drawn from tokens with `--pct`. The light-theme slider still sits under 3:1 contrast (see handoff).

In the code: desktop/src/pages/Displays.tsx (Settings, shownSource, Slider), desktop/src/styles.css (.sechead, .mslider, .mnum).
