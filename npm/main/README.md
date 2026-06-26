# best-claude-hud

Minimal Claude Code statusline HUD written in Rust.

## Installation

```bash
npm install -g best-claude-hud
```

## Features

- 🚀 **Fast**: Written in Rust for maximum performance
- 🌍 **Cross-platform**: Works on Windows, macOS, and Linux
- 📦 **Easy installation**: One command via npm
- 🔄 **Auto-update**: Built-in update notifications
- 🎨 **Beautiful**: Nerd Font icons and colors

## Usage

After installation, configure Claude Code with:

```json
{
  "statusLine": {
    "type": "command",
    "command": "best-claude-hud"
  }
}
```

The package does not copy binaries into `~/.claude`; it runs the matching
platform binary from npm optional dependencies.

You can also use it directly:

```bash
best-claude-hud --help
best-claude-hud --version
```

## For Users in China

Use npm mirror for faster installation:

```bash
npm install -g best-claude-hud --registry https://registry.npmmirror.com
```

## More Information

- GitHub: https://github.com/GaoSSR/best-claude-hud
- Issues: https://github.com/GaoSSR/best-claude-hud/issues
- License: MIT
