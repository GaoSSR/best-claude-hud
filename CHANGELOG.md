# Changelog

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
