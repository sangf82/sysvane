# Handoff

Updated 2026-11-16. Language: vi. Đọc file này trước; cây thư mục và quy tắc: `docs/architecture.md`; milestone: `docs/plan.md`; ghi chú module monitor (M4, thiết kế Displays, "Not done", mẹo agent-desktop): `docs/monitor-notes.md`.

## State
Nhánh `main`, tag mới nhất `v0.0.4`. M4 (DDC) xong. Quy trình Claude Code (agents, lệnh, hooks, release) đã commit (`43a6665`). Chưa push gì. Chưa đổi Rust trong phiên này.

## Uncommitted (sẵn sàng commit, hỏi chủ trước)
- `.claude/commands/handoff.md`, `status.md`: `/handoff` hỏi ngôn ngữ, viết lại gọn (<40 dòng); `/status` trả lời theo dòng `Language:`.
- `docs/handoff.md`, `docs/monitor-notes.md`: handoff rút gọn, phần dài chuyển sang notes.
- `.trail/hook.cjs`: do `trail init` (trail 0.30.2) ghi lại. Commit cùng `.trail/` nếu muốn.

## Next
1. Commit nhóm trên (chủ nói yes).
2. `/milestone M5` (module LCD): hỏi chủ về giao thức USB Levita Vision và theme dựng sẵn trước khi viết code. Xong khi mục "done when" của M5 trong `docs/plan.md` đúng.
3. Sửa 2 mục review UI khi đụng `styles.css`: thumb 14 px và ô số cao ~22 px (<24 px); `.problem` (`styles.css:285`) dùng vạch đỏ bên trái.
4. Nếu dùng `animate-ui`: chạy `npx shadcn init` trong `desktop/` chỉ sau khi `frontend-ui` liệt kê file sẽ đổi.

## Decisions (phiên này, code không thể hiện)
- `/release` hỏi yes lần nữa trước mọi `git push`; duyệt changelog chưa đủ.
- Hook trail `tool-savings` giữ lại (chạy `node` cục bộ, không tốn token model).
- Sonnet dẫn dắt, Opus cố vấn, Haiku cho `scout`/`verify`; đã gỡ `CLAUDE_CODE_SUBAGENT_MODEL=haiku` toàn cục (backup `~/.claude/settings.json.bak-subagent`).

## Open questions
- Không có.

## Not verified
- `rust-core`/`frontend-ui` nạp được skill preload khi skill là `user-invocable-only`.
- `scout`/`verify` thật sự chạy Haiku (xem status line).
- `/status`, `/milestone`, `/release`, `/handoff` mới và `--notes-file` trong `release.yml` chưa chạy thật; lần `/release` đầu sẽ thử workflow.
- `skills-lock.json` khi clone mới (`.claude/skills/` bị gitignore).
- Core dev đang chạy trên PC chưa khởi động lại sau bản sửa serial = 0.

## Gotchas
- `guard-monitor` từng chặn mọi lệnh Bash chứa "agent-desktop" và "find"/"press"; đã sửa, nếu lại vướng thì ghi file bằng Write/Edit.
- Hook trail nằm trong `.claude/settings.local.json` (gitignore), chỉ có trên máy này. Sao lưu file trước khi chạy `trail init`.
- PowerShell 5.1 `Set-Content` không có `-NoNewline`; dùng Edit cho file hook.
