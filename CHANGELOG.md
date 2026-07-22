# Changelog

## 0.1.9 - 2026-07-23

- Clear stale Ultracode state as soon as a new `/effort` command appears, so
  cancelled, overridden, or unrecognized command output cannot replace the
  effective statusLine effort and ordinary `xhigh` remains distinct.
- Version effort caches and rebuild incompatible cache data from the active
  process transcript, preventing state written by older HUD versions from
  leaking into the upgraded session.
- Render the model name and reasoning effort through one shared path with
  exactly one ASCII space in both background and non-background themes.

## 0.1.8 - 2026-07-22

- Display Claude Code's live reasoning mode next to the model name in a
  dedicated bright purple color.
- Distinguish `low`, `medium`, `high`, `xhigh`, `max`, and `ultracode` without
  adding another statusline segment, icon, or separator.
- Recognize Ultracode from successful `/effort` events in the active Claude
  Code process because the official statusline payload reports it as `xhigh`.
- Ignore Ultracode events from an earlier process when a conversation is
  resumed, while treating `CLAUDE_CODE_EFFORT_LEVEL=xhigh` as compatible and
  preventing incompatible environment overrides from being misreported.
- Keep the model-only display for models and Claude Code versions that do not
  provide reasoning effort.

## 0.1.7 - 2026-07-18

- Prefer Claude Code's official `context_window` statusLine data for context
  usage and window size.
- Preserve the last valid context usage when an interrupted response writes an
  all-zero usage placeholder to the active transcript.
- Keep transcript parsing as a compatibility fallback when official context
  data is absent, null, or temporarily zero.

## 0.1.6 - 2026-07-18

- Add Nix flake packaging and development shell support.
- Run Nix flake checks in CI.
- Optimize release builds for smaller native binaries.
- Keep the displayed workspace and Git status anchored to Claude Code's launch
  directory when skills, subagents, or shell commands change the temporary
  working directory.
- Fall back to `workspace.current_dir` for older Claude Code versions that do
  not provide `workspace.project_dir`.

## 0.1.5 - 2026-06-26

- Add `best-claude-hud --setup` to configure Claude Code `statusLine`
  automatically.
- Document the one-line install plus setup flow so users do not have to edit
  `~/.claude/settings.json` manually.

## 0.1.4 - 2026-06-26

- Make the unscoped `best-claude-hud` npm package the only documented install
  entry.
- Publish platform binaries as prerelease versions of the unscoped package and
  keep platform packages internal through npm alias optional dependencies.

## 0.1.3 - 2026-06-26

- Publish the previous scoped npm package through GitHub Actions trusted
  publishing.
- Keep npm distribution aligned with the Kiri-style single scoped package
  layout.

## 0.1.2 - 2026-06-26

- Migrate npm distribution to the Kiri-style scoped package layout.
- Package platform binaries as prerelease versions of the same npm package and
  install them through npm alias optional dependencies.
- Replace the old multi-package npm build scripts with `packaging/npm`.
- Prepare the publish workflow for GitHub Actions trusted publishing.

## 0.1.1 - 2026-06-26

- Publish the first npm registry version of `best-claude-hud`.
- Replace temporary README logo assets with the generated project logo.
- Align release metadata and npm publishing workflow for the public package.

## 0.1.0 - 2026-06-26

- Start `best-claude-hud` from the `Haleclipse/CCometixLine` source snapshot.
- Rename Rust crate, CLI command, npm package, configuration path, and release
  assets to `best-claude-hud`.
- Use Apache-2.0 for this project and preserve upstream attribution in `NOTICE`.
- Stop npm install from copying/linking binaries into `~/.claude`.
- Prefer Claude Code stdin `rate_limits` data before OAuth/API usage polling.
- Accept both string and object forms of the Claude Code `model` field.
- Improve context window parsing for complete assistant messages.
