# Upstream Triage

This project starts from the `Haleclipse/CCometixLine` source snapshot and does
not preserve upstream Git history. The initial maintenance policy is to accept
low-risk stability fixes and reject default-on statusline noise.

## Accepted In Initial Rebuild

- `#47` Git commands use `--no-optional-locks`.
- `#99` context window parsing now prefers complete assistant messages with
  `stop_reason`, avoids stale project-history fallback for new sessions, and
  adds newer third-party model mappings.
- `#112/#105` usage reads `rate_limits` from Claude Code stdin before falling
  back to OAuth/API polling.
- `#118/#93/#90/#76` model parsing is tolerant of both string and object model
  fields.
- `#128` npm install no longer copies or links binaries into `~/.claude`.

## Deferred

- `#41/#42/#44/#74/#82/#87/#100/#104/#106/#107/#124` need separate review.
  Several are useful, but they either expand displayed information, change TUI
  behavior, or require more compatibility tests.

## Rule

New segments or extra fields must be disabled by default unless they are
required for correctness. Default output should remain a compact single line.
