---
description: Brief a new session: where the project stands, what is uncommitted, what is next. Read-only.
argument-hint: "[optional focus, e.g. M5 or displays]"
disable-model-invocation: true
---

Run only because the user typed /status. Read-only: change nothing, run nothing that writes.

1. Read `docs/handoff.md`, the current milestone in `docs/plan.md`, and `git status --short`, `git log --oneline -5`, `git tag --sort=-v:refname` (first one). For a focus in `$ARGUMENTS`, run `trail ask "<focus>"` and read the learnings it lists. Use `scout` for lookups.
2. Treat handoff and plan as historical: if a claim is cheap to check (a file exists, a commit is there, the tree matches "Uncommitted"), check it and say where it differs.
3. Answer in at most 20 lines:
   - **Now**: milestone, branch, last release tag, last commit.
   - **Uncommitted**: files and whether they match the handoff.
   - **Next**: the handoff's next step, and which command starts it (`/milestone`, or the task itself).
   - **Waiting on the owner**: open questions.
   - **Risks**: unverified items and relevant learnings (one line each, with the file).
4. End by asking what to do first. Do not start work.
