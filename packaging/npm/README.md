# best-claude-hud

Minimal Claude Code statusline HUD powered by Rust.

## Install

```bash
npm install -g best-claude-hud
```

For users in China:

```bash
npm install -g best-claude-hud --registry https://registry.npmmirror.com
```

## Claude Code Configuration

```json
{
  "statusLine": {
    "type": "command",
    "command": "best-claude-hud",
    "padding": 0
  }
}
```

## Commands

```bash
best-claude-hud --help
best-claude-hud --version
best-claude-hud --config
best-claude-hud --theme minimal
```

The npm package installs the `best-claude-hud` command and resolves the matching native binary through platform-specific optional dependencies.

## Links

- GitHub: https://github.com/GaoSSR/best-claude-hud
- Issues: https://github.com/GaoSSR/best-claude-hud/issues
- License: Apache-2.0
