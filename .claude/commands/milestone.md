---
description: Start the next SysVane milestone: read the plan, grill the owner on open questions, then plan, delegate, test and review.
argument-hint: "[M5|M6|...; default: Next in docs/handoff.md]"
disable-model-invocation: true
---

Run only because the user typed /milestone. Lead (you, Sonnet) decides and writes; `scout`/`verify` (Haiku) read and test; the advisor (Opus) checks the plan and risky code.

1. **Orient.** Read `docs/handoff.md` (State, Next, Not verified) and the milestone `$ARGUMENTS` in `docs/plan.md`. Use `scout` for code lookups. If the tree has uncommitted work, ask whether to commit first.
2. **Grill.** List the milestone's "Open questions" and "Not verified" items. Ask the owner with AskUserQuestion, one decision at a time (max 4 per round), recommending an option. Do not guess an answer the plan says to ask about. Record answers in the handoff.
3. **Plan.** Break the milestone into 3 to 8 tasks (TaskCreate), each with a "done when" check from the plan. Call the advisor on the plan before writing code.
4. **Develop.** Rust in `core/` or `wire/`: delegate to `rust-core`; UI in `desktop/`: delegate to `frontend-ui`; small single-file edits you do yourself. One task at a time; code that writes to hardware gets an advisor check before it is declared done.
5. **Test.** `verify` runs the checks after each task. Fix failures yourself or send them back to the writer agent. Tests use fake devices only; real-hardware checks are listed for the owner to run and are never run by an agent.
6. **Review.** `code-review` on the diff; advisor on safety-relevant changes.
7. **Close.** Update `docs/handoff.md` (State, Next, Not verified), `trail note` for decisions and dead ends, then ask the owner whether to commit. Do not mention release; that is `/release`.
