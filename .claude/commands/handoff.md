---
description: End the session: write the current state into docs/handoff.md so a new Claude Code session can continue without questions.
argument-hint: "[free-text note for the next session]"
disable-model-invocation: true
---

Run only because the user typed /handoff. The next session starts cold and reads `docs/handoff.md` first, so it must be enough on its own.

1. **Collect.** `git status --short`, `git log --oneline -5`, `git diff --stat`, open tasks (TaskList), and what this session did. Add `$ARGUMENTS` as the owner's note. Read the current `docs/handoff.md`.
2. **Rewrite `docs/handoff.md`** (update in place, keep it short; delete finished items, do not append history):
   - `Updated <date>` and one-line **State**: milestone, what works, branch.
   - **Uncommitted**: each changed file or group and why; say whether it is ready to commit.
   - **Next**: the single next step, then 2 to 4 more, each concrete (file, command, "done when").
   - **Decisions**: only choices made this session that the code does not show, with the reason.
   - **Open questions**: what the owner must answer, with the options and your recommendation.
   - **Not verified**: what was not run or only partly checked, and on which hardware.
   - **Gotchas**: dead ends and traps seen this session, so the next session does not repeat them.
3. **Trail.** Save each lasting decision or dead end with `trail note` (not status, not what the diff shows).
4. **Check.** Run nothing heavy; if code changed and was not tested, say so under Not verified. Do not run release steps.
5. **Close.** Show the owner a 5-line summary and ask whether to commit the handoff (and any ready changes). Do not commit or push without a yes. Finish with: "Next session: run /status."
