---
title: The AL258Q ignores DDC writes to picture mode and to colour presets outside the range its picture mode allows
author: Sangf82
date: 2026-11-15
branch: main
cost: { minutes: 14, tokens: 57030 }
id: l-63c7e2ad
scope: repo
status: active
replaces: l-2abab4a4
src: agent
touches:
  - tools/probe-ddc.ps1
  - docs/handoff.md
---

## Decided
The guarded DDC write checks VCP 0x14 (colour preset) against the MCCS values 1 to 13, not the maximum the monitor reports: 0x14 is a list, and the AL258Q reported max 5 while the May scan showed 14. Picture mode 0xB6 is on the allowlist and a profile writes it first, then 0x14, then the gains.

## Tried and ruled out
- Checking 0x14 against the reported maximum: refused Calibrated-Right (0x14 = 11) before sending.
- Switching picture mode over DDC to unlock User mode: a profile with 0xB6 = 1 then 0x14 = 11, applied through the app, returned no error, but the monitor kept 0xB6 = 3 and 0x14 = 5. Brightness writes on the same monitor took, so DDC itself works.
- Raw SetVCPFeature from a one-off PowerShell script: the auto-mode permission classifier refused it. Test writes go through the app's guarded write (a user profile or a slider).

## Watch out
With picture mode 3 the AL258Q allows presets 1 to 5 only, so Calibrated-Right (0x14 = 11, User) cannot take until the picture mode is changed in the monitor's own menu. Both Calibrated profiles store 0xB6 = 3 and may need that value changed. Not yet confirmed that Standard in the OSD unlocks 0x14 = 11.

In the code: core/src/modules/monitor/vcp.rs (`write`, `PRESETS`, `PICTURE_MODE`, `WRITABLE`), profile.rs (`entries` order). Read-only probe: tools/probe-ddc.ps1. Diagnostic profile: %ProgramData%\SysVane\config\monitor\Test-Standard-User.json.
