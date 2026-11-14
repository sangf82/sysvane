---
title: An unelevated core needs write rights on the snapshot pipe to open its next instance
author: Sangf82
date: 2026-11-14
branch: main
cost: { minutes: 7, tokens: 44580 }
id: l-af1c5d15
scope: repo
status: active
src: agent
touches:
  - core/src/pipe.rs
---

## Decided
The snapshot pipe gives the current user read only when the core is elevated, and read plus write when it is not. An unelevated core is the same user as the UI, so there is no boundary to protect, and without write rights Windows refuses the second pipe instance (error 5) as soon as one client is connected.

## Tried and ruled out
Read plus the create-instance bit (`0x12008d`), and other masks without the full write set (`0x12018d`, `0x12009f`): a probe test showed they all fail on the second instance. Only the full read-write mask (`GRGW`, `0x12019f`) and `GA` pass. Searching the exe for the SDDL string with `strings -el` proved nothing because Rust literals are UTF-8.

## Watch out
An elevated shell or CI runner passes the regression test even with read-only rights, so only a normal shell catches this. Errors repeat every second in the log while a client is connected and stop when the UI closes. In shell heredocs a `\.\pipe\` name loses a backslash; check the test pipe name (error 123 means invalid name).
In the code: `core/src/pipe.rs` (`data_pipe_rights`, `is_elevated`, test `a_second_instance_opens_with_the_rights_chosen_for_this_process`).
