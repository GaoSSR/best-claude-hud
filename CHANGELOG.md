# Changelog

## 0.1.0 - Unreleased

- Start `best-claude-hud` from the `Haleclipse/CCometixLine` source snapshot.
- Rename Rust crate, CLI command, npm package, configuration path, and release
  assets to `best-claude-hud`.
- Add MIT `LICENSE` and upstream attribution in `NOTICE`.
- Stop npm install from copying/linking binaries into `~/.claude`.
- Prefer Claude Code stdin `rate_limits` data before OAuth/API usage polling.
- Accept both string and object forms of the Claude Code `model` field.
- Improve context window parsing for complete assistant messages.
