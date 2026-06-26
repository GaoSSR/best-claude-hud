# best-claude-hud

Minimal Claude Code statusline HUD powered by Rust.

## Install

```bash
npm install -g best-claude-hud && best-claude-hud --setup
```

For users in China:

```bash
npm install -g best-claude-hud --registry https://registry.npmmirror.com && best-claude-hud --setup
```

## Claude Code Configuration

`npm install -g best-claude-hud` only installs the command. Claude Code will not show the HUD until `statusLine` is configured.

Recommended:

```bash
best-claude-hud --setup
```

Manual configuration:

```json
{
  "statusLine": {
    "type": "command",
    "command": "/path/to/best-claude-hud",
    "padding": 0
  }
}
```

`best-claude-hud --setup` resolves the installed command to an absolute path when possible.

## Commands

```bash
best-claude-hud --help
best-claude-hud --version
best-claude-hud --setup
best-claude-hud --config
best-claude-hud --theme minimal
```

The npm package installs the `best-claude-hud` command and resolves the matching native binary through platform-specific optional dependencies.

## Links

- GitHub: https://github.com/GaoSSR/best-claude-hud
- Issues: https://github.com/GaoSSR/best-claude-hud/issues
- License: Apache-2.0
