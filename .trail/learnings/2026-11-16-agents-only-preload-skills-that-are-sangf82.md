---
title: Agents only preload skills that are not marked user-invocable-only
author: Sangf82
date: 2026-11-16
branch: main
cost: { minutes: 1, tokens: 1113 }
id: l-cda76550
scope: repo
status: active
src: agent
touches:
  - AGENTS.md
---

## Decided
rust-core and frontend-ui preload their skills, so those skills must not be hidden from the main session. impeccable and rust-best-practices were removed from the hiding list; unslop and logo-design stay hidden.

## Tried and ruled out
Hiding the preloaded skills to keep the main session small: a skill set to user-invocable-only is not injected into the agent either. rust-core did not see rust-best-practices and frontend-ui did not see impeccable until the setting was removed.

## Watch out
Check a new preload by asking the agent, with no tools, whether the skill text is in its context. Hiding a skill that an agent lists under `skills:` silently breaks that agent.

In the code: `.claude/settings.json` (`skillOverrides`), `.claude/agents/rust-core.md`, `.claude/agents/frontend-ui.md`, `AGENTS.md` (Agents and skills).
