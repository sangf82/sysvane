---
title: "The root is split by process: one background core with all device modules, an on-demand UI, and a shared contract"
author: Sangf82
date: 2026-11-15
branch: main
cost: { minutes: 16, tokens: 121556 }
id: l-7b42d5e6
scope: repo
status: active
src: agent
touches:
  - core/src/sensors/cputemp.rs
  - tools/probe-temp.ps1
  - AGENTS.md
  - README.md
  - modules/ddc/docs/CALIBRATION.md
  - docs/architecture.md
  - PRODUCT.md
  - docs/plan.md
  - docs/handoff.md
  - desktop/src/lib/host.ts
---

## Decided
- The repo root stays split by process, not by language: core/ is the only background process and holds every device module (fan, RGB, LCD, monitor) compiled in; desktop/ is the on-demand UI (Tauri shell + React together); wire/ is the contract between them and would also serve a future web UI.
- Device modules go kind first, then vendor: core/src/modules/<kind>/<vendor>.rs, shared vendor protocol in vendors/<vendor>/. No folder until its first real module.
- DESIGN.md and PRODUCT.md stay at the root: the impeccable skill generates and reads DESIGN.md at the project root, and the design.md lint runs on it there. skills-lock.json stays at root for the skills CLI.
- Docs, the handoff and device notes live in docs/; the handoff is committed so the next agent knows what was in progress.

## Tried and ruled out
- Moving DESIGN.md into branding/: impeccable would create a second DESIGN.md at the root.
- Naming the Rust core desktop/ and splitting React into fe/ now: the core is the shared backend, not the desktop app, and the UI is web-ready only when Tauri calls are behind lib/host.ts, not by moving folders. Split into ui/, desktop/, web/ only when a second shell exists.

## Watch out
- Moving React out of desktop/ touches tauri.conf.json (frontendDist, beforeDevCommand), CI working-directory, bump-version.ps1, the lint-on-stop hook and the wire.ts path test in wire/src/lib.rs.

In the code: docs/architecture.md, docs/handoff.md, desktop/src/lib/host.ts, desktop/src/lib/update.ts.
