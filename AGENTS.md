# Project Agent Instructions

## Release Work

- Before any release-related change or external action, read
  `/Users/gaossr/.codex/AGENTS.md` and this repository's `RELEASING.md` in full.
- Treat `RELEASING.md` as the source of truth for the release sequence and
  verification commands.
- Never create a release tag while `Cargo.toml`, `Cargo.lock`, and
  `packaging/npm/package.json` still declare an older version.
- Keep these approval checkpoints separate: release-preparation commit; push;
  the tag push and its automatically triggered GitHub Release; npm publish; and
  local machine upgrade.
- A pushed tag triggers the GitHub Release workflow. Wait for that workflow and
  verify its assets before requesting approval to publish npm packages.
- Never delete, move, replace, or force-update a tag to recover from a failed
  release without explicit user approval.
- Issue and pull request state changes are separate actions and always require
  explicit approval.
