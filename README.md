# best-claude-hud

Minimal Claude Code statusline HUD written in Rust.

`best-claude-hud` is a maintained fork/rebuild inspired by
[`Haleclipse/CCometixLine`](https://github.com/Haleclipse/CCometixLine). It keeps
the useful single-line statusline idea while taking a stricter stance on screen
noise: default output should stay compact, readable, and useful during real work.

![Preview](assets/img1.png)

## Install

```bash
npm install -g best-claude-hud
```

For users in China:

```bash
npm install -g best-claude-hud --registry https://registry.npmmirror.com
```

From source:

```bash
git clone https://github.com/GaoSSR/best-claude-hud.git
cd best-claude-hud
cargo install --path .
```

## Claude Code Setup

Add a statusLine command to `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "best-claude-hud"
  }
}
```

The npm package does not copy binaries into `~/.claude`. The command runs from
the npm global binary shim and loads the correct platform binary from npm
optional dependencies.

## Usage

```bash
best-claude-hud --help
best-claude-hud --config
best-claude-hud --theme minimal
```

Configuration files are stored under:

```text
~/.claude/best-claude-hud/
```

The default HUD focuses on:

- current model
- project directory
- Git branch/status
- context window usage
- optional usage/rate limit, cost, session, and output style segments

## Maintenance Direction

The first-party goal for this fork is stability and compact output. Feature
additions should be accepted only when they stay single-line by default, are
configuration driven, and do not make the common statusline noisy.

Initial upstream triage is tracked in [docs/triage.md](docs/triage.md).

## Release

CI runs formatting, clippy, tests, and release build smoke checks. Tagged
releases build platform archives and npm packages. npm publish is handled by a
separate workflow so registry credentials/trusted publishing can be controlled
independently.

```bash
git tag v0.1.0
git push origin v0.1.0
```

## License and Attribution

MIT licensed. See [LICENSE](LICENSE).

This project is based on source code from `Haleclipse/CCometixLine`, also
published as MIT in its Cargo metadata. See [NOTICE](NOTICE) for attribution.
