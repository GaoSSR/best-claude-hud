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
npm --prefix packaging/npm run check
npm --prefix packaging/npm run test
npm --prefix packaging/npm run pack:dry-run
```

## Releases

Follow the complete, approval-gated checklist in [RELEASING.md](RELEASING.md).
Do not create a tag until the Cargo and npm manifests declare the intended
version and the release-preparation commit has passed CI.

## Pull Requests

Before opening a PR:

- run the checks above
- explain the user-facing behavior change
- include screenshots or terminal output for visual changes
- keep unrelated refactors out of the PR

## Upstream Work

The initial rebuild is based on `Haleclipse/CCometixLine`. When porting upstream PRs, prefer small, reviewable patches and document the upstream PR/issue number in the commit or PR description.
