---
title: Module code is allowed to be unused in release builds until the first real module ships
author: Sangf82
date: 2026-11-14
branch: main
cost: { minutes: 4, tokens: 84935 }
id: l-a7340816
scope: repo
status: active
src: agent
touches:
  - core/src/module.rs
  - core/src/config.rs
  - core/src/fake.rs
  - core/src/devices.rs
  - core/src/main.rs
  - core/src/commands.rs
  - core/src/hub.rs
---

## Decided
The fake device is compiled in debug builds only, so a release build has no module and the module host and config loader look unused. They stay, with dead-code allowed only in release, until M4 adds the DDC module.

## Tried and ruled out
Pushing the fake device into a `Vec` after creating it: clippy (`vec_init_then_push`) refuses it. A `vec![]` with a cfg-gated element passes.

## Watch out
Run clippy in release too (`cargo clippy --release -p sysvane-core -- -D warnings`); the debug run alone passes while release fails with "never used".
In the code: `core/src/main.rs` (`#[cfg_attr(not(debug_assertions), allow(dead_code))]` on `mod config` and `mod module`, and `modules()`). Remove the allows when M4 registers a release module.
