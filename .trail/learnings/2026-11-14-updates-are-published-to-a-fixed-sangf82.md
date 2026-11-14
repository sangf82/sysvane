---
title: "Updates are published to a fixed release tag because GitHub's latest link skips pre-releases"
author: Sangf82
date: 2026-11-14
branch: main
cost: { minutes: 13, tokens: 88718 }
id: l-41f59636
scope: repo
status: active
src: agent
touches:
  - ".github/workflows/release.yml"
  - AGENTS.md
  - Cargo.lock
  - Cargo.toml
  - desktop/package-lock.json
  - desktop/package.json
  - desktop/src-tauri/Cargo.toml
  - desktop/src-tauri/capabilities/default.json
  - desktop/src-tauri/installer-hooks.nsh
  - desktop/src-tauri/src/main.rs
---

## Decided
The beta channel ships only pre-releases, and GitHub's "releases/latest" link ignores pre-releases, so the updater manifest lives on a fixed release named `updates` that every tag replaces.
The uninstaller deletes the shared data folder through the process environment, because NSIS has no common-app-data variable and Tauri switches the shell context to the current user before the hook runs.

## Tried and ruled out
Using the common-app-data name in the uninstall hook: it expands to nothing, so "delete data = Yes" silently left the logs behind. Using the roaming app-data variable would hit the wrong folder.

## Watch out
Local release builds now fail without the signing key in the environment. The key must never enter the repo, and losing it stops every installed copy from updating.
In the code: desktop/src-tauri/installer-hooks.nsh (`$%ProgramData%`), .github/workflows/release.yml (`updates` tag, `latest.json`), desktop/src-tauri/tauri.conf.json (updater endpoint), env `TAURI_SIGNING_PRIVATE_KEY`.
