---
description: Prepare a SysVane release: draft the changelog for review, then bump, tag, push and watch CI.
argument-hint: "[X.Y.Z, default: next patch]"
disable-model-invocation: true
---

Release flow. Run only because the user typed /release. Never start it on your own, and during normal work ask about commits only.

1. **Check.** Must be on `main`, tree clean (list anything else and stop), `git fetch` shows nothing to pull. Run `trail check`, `cargo test -p sysvane-core -p sysvane-wire`, `cd desktop; npm run build`. Stop on any failure and report it.
2. **Version.** `$ARGUMENTS`, else latest `vX.Y.Z` tag (`git tag --sort=-v:refname`) with the patch + 1. State it.
3. **Changelog.** Read `git log <last tag>..HEAD` and the diff stats. Write `docs/releases/v<version>.md` for users, not for developers: short sections (Added, Changed, Fixed), one line each, plain words, no commit hashes. Mention hardware or safety changes explicitly. Show the full text to the user.
4. **Wait.** Stop and ask the user to review. Edit the file on request. Do nothing further until they approve.
5. **Release (after approval only).**
   - `powershell -NoProfile -File scripts\bump-version.ps1 <version>`
   - Commit the bump and the changelog: `Release v<version>`
   - `git tag v<version>`, `git push origin main`, `git push origin v<version>`
   - Call the advisor on the changelog and the diff since the last tag before the push; fix what it finds.
   - Before either `git push`, ask the user once more and wait for a yes. The changelog approval alone is not enough.
   - `gh run watch` the Release workflow (CI builds the signed installer and publishes the pre-release with the changelog as notes).
6. **Report.** Release URL, or the failing CI step. If CI fails, do not retag or force-push: report and ask. Rollback for a bad release: delete the release and tag (`gh release delete`, `git push --delete origin v<version>`) only on the user's word.
