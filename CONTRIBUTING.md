# Contributing

Thanks for helping improve `best-claude-hud`.

## Project Direction

This project keeps the default Claude Code statusline compact. New functionality should be:

- useful during normal terminal work
- single-line by default
- configurable when it adds visual weight
- covered by tests when it changes parsing, rendering, release packaging, or Claude Code compatibility

## Development

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo build --release
cargo run -- --help
```

For npm packaging checks:

```bash
node npm/scripts/prepare-packages.js 0.1.0
cp target/release/best-claude-hud npm-publish/darwin-arm64/best-claude-hud
chmod +x npm-publish/darwin-arm64/best-claude-hud
(cd npm-publish/darwin-arm64 && npm pack --dry-run)
(cd npm-publish/main && npm pack --dry-run)
```

## Pull Requests

Before opening a PR:

- run the checks above
- explain the user-facing behavior change
- include screenshots or terminal output for visual changes
- keep unrelated refactors out of the PR

## Upstream Work

The initial rebuild is based on `Haleclipse/CCometixLine`. When porting upstream PRs, prefer small, reviewable patches and document the upstream PR/issue number in the commit or PR description.
