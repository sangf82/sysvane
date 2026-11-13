---
title: "Devices are components with one kind each, and tool hooks stay out of per-edit"
author: Sangf82
date: 2026-11-13
branch: main
cost: { minutes: 1, tokens: 12092 }
id: l-b89aa2ef
scope: repo
status: active
src: agent
touches:
  - ".github/workflows/ci.yml"
  - ".github/workflows/release.yml"
  - AGENTS.md
  - rust-toolchain.toml
  - scripts/bump-version.ps1
  - Cargo.lock
  - Cargo.toml
  - PRODUCT.md
  - README.md
---

## Decided
- A device is made of components and each component has one kind (fan, rgb, lcd, mouse, keyboard, monitor, headset). The UI has one panel per kind, built from the module's data descriptor, so a new vendor in an existing kind needs only Rust. The AIO (pump, fan, LCD, RGB) and RGB on keyboard/mouse/headset break a one-kind-per-device model.
- Write the descriptor contract when the first module (DDC, M4) lands; do not scaffold empty folders per kind.
- Lint runs at Stop, not on every edit. Use rust-analyzer LSP only; skip the zircote/rust-lsp hooks (cargo check, clippy and test on each write, 19 hooks, no Windows note).
- Toolchain pinned to 1.99.0 so the local hook and CI clippy agree.
- Repo restructure (backend/frontend split) and Ultracite go in separate commits so each reverts alone.

## Tried and ruled out
- Per-edit clippy hooks: slow and contend with rust-analyzer for the target lock.

## Watch out
- Do not let Ultracite write its rules into the shared project instructions; they load into every session including Rust ones. Put them in the frontend agent.
- Moving desktop/ breaks the wire.ts mirror test (reads by path) and tauri.conf.json relative paths; keep the frontend one level deep.

In the code: .claude/hooks/lint-on-stop.ps1, .claude/agents/, rust-toolchain.toml, .github/workflows/
