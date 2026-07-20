<h4 align="right"><a href="./README.md">English</a> | <strong><a href="./README_CN.md">简体中文</a></strong></h4>

<p align="center">
  <a href="https://github.com/GaoSSR/best-claude-hud">
    <img src="assets/best-claude-hud-logo.png" alt="best-claude-hud" width="480">
  </a>
</p>

<h3 align="center"><nobr>极简 Claude Code 状态栏 HUD，由 Rust 驱动</nobr></h3>

---

<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-CLI-orange" />
  <img alt="MacOS Linux Windows supported" src="https://img.shields.io/badge/MacOS%20%7C%20Linux%20%7C%20Windows-supported-brightgreen" />
  <img alt="Command: best-claude-hud" src="https://img.shields.io/badge/command-best--claude--hud-8A2BE2" />
  <img alt="License: Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-blue" />
</p>

## best-claude-hud 概览

`best-claude-hud` 是一个用 Rust 写的高性能 Claude Code 状态栏工具。它在终端中展示真正有用的 Claude Code 工作状态：当前模型、工作目录、Git 分支/状态、上下文窗口占用，以及可选的 usage/rate limit 信息。

<p align="center">
  <img src="assets/best-claude-hud-preview.png" alt="best-claude-hud statusline preview" width="1200">
</p>

默认状态栏关注：

- Claude 模型显示
- Claude Code 启动时的项目目录，不受临时工作目录变化影响
- Git 分支、clean/dirty/conflict 状态和 ahead/behind 计数
- 当前 Claude Code transcript 的 context window token 占用
- 可选的 usage/rate-limit、cost、session、output style 段落

## 安装

`best-claude-hud` 通过 npm 分发。npm 包使用预构建原生二进制，用户不需要本地安装 Rust。

一行完成安装并配置 Claude Code：

```bash
npm install -g best-claude-hud && best-claude-hud --setup
```

配置完成后需要重启 Claude Code。已经打开的会话通常不会自动重新读取 `~/.claude/settings.json`。

仅安装：

```bash
npm install -g best-claude-hud
```

使用 yarn 或 pnpm：

```bash
yarn global add best-claude-hud
pnpm add -g best-claude-hud
```

国内网络可使用 npm 镜像：

```bash
npm install -g best-claude-hud --registry https://registry.npmmirror.com && best-claude-hud --setup
```

更新：

```bash
npm update -g best-claude-hud
```

卸载：

```bash
npm uninstall -g best-claude-hud
```

## Nix

`best-claude-hud` 也提供 Nix flake，适合声明式、可复现环境。

不全局安装，直接运行：

```bash
nix run github:GaoSSR/best-claude-hud -- --help
```

安装到 Nix profile：

```bash
nix profile install github:GaoSSR/best-claude-hud
best-claude-hud --setup
```

如果使用 home-manager 或其他声明式配置，可以让 Claude Code 直接指向 Nix store 中的二进制：

```nix
# 在你的 flake inputs 中添加：
# best-claude-hud.url = "github:GaoSSR/best-claude-hud";

{ inputs, pkgs, ... }:

let
  hud = inputs.best-claude-hud.packages.${pkgs.system}.default;
in
{
  home.packages = [ hud ];

  home.file.".claude/settings.json".text = builtins.toJSON {
    statusLine = {
      type = "command";
      command = "${hud}/bin/best-claude-hud";
      padding = 0;
    };
  };
}
```

如果你已经用 Nix 管理 `~/.claude/settings.json`，请把上面的 `statusLine` 合并进现有 JSON，不要直接替换整个文件。

开发环境：

```bash
nix develop
```

## Claude Code 配置

`npm install -g best-claude-hud` 只会安装命令本身。Claude Code 不会自动调用它，必须配置 `statusLine` 后才会显示 HUD。

推荐方式：

```bash
best-claude-hud --setup
```

`--setup` 会保留现有配置，并把 `statusLine` 写入 `~/.claude/settings.json`。它会尽量把已安装命令解析成绝对路径：

```json
{
  "statusLine": {
    "type": "command",
    "command": "/path/to/best-claude-hud",
    "padding": 0
  }
}
```

如果你确认 Claude Code 会继承当前 shell 的 PATH，也可以手动写 `"command": "best-claude-hud"`。如果原本已经存在 `statusLine`，`--setup` 会先在 `settings.json` 同目录创建带时间戳的备份文件，然后再替换。修改后需要重启 Claude Code。

npm 包不会把二进制安装到 `~/.claude`。它使用 npm 全局命令，并通过 Kiri 风格的 npm alias optional dependencies 解析当前平台对应的原生二进制。

## 命令

```bash
best-claude-hud                    # 在终端中直接运行时打开交互式菜单
best-claude-hud --help             # 查看帮助
best-claude-hud --version          # 查看版本
best-claude-hud --setup            # 配置 Claude Code statusLine
best-claude-hud --config           # 打开 TUI 配置界面
best-claude-hud --theme minimal    # 临时使用指定内置主题
best-claude-hud --patch <cli.js>   # patch Claude Code cli.js 的 context warning
```

## 主题

临时覆盖当前主题：

```bash
best-claude-hud --theme cometix
best-claude-hud --theme minimal
best-claude-hud --theme gruvbox
best-claude-hud --theme nord
best-claude-hud --theme powerline-dark
best-claude-hud --theme powerline-light
best-claude-hud --theme powerline-rose-pine
best-claude-hud --theme powerline-tokyo-night
```

自定义主题目录：

```text
~/.claude/best-claude-hud/themes/
```

然后运行：

```bash
best-claude-hud --theme my-custom-theme
```

## 配置

配置文件存放在：

```text
~/.claude/best-claude-hud/
```

关键文件：

- `config.toml`：主配置与 segment 配置
- `models.toml`：模型显示名称与 context window limit
- `themes/*.toml`：自定义主题
- `.api_usage_cache.json`：可选 usage API 缓存
- `.update_state.json`：更新检查状态

打开 TUI 配置器：

```bash
best-claude-hud --config
```

支持的 segment：

- `model`
- `directory`
- `git`
- `context_window`
- `usage`
- `cost`
- `session`
- `output_style`
- `update`

## 模型配置

`models.toml` 会在首次运行时自动创建：

```text
~/.claude/best-claude-hud/models.toml
```

它用于控制模型显示名称和上下文窗口上限。Claude 模型族会自动识别，第三方模型可以手动配置：

```toml
[[models]]
pattern = "kimi-k2.7"
display_name = "Kimi K2.7"
context_limit = 262144

[[models]]
pattern = "glm-5"
display_name = "GLM-5"
context_limit = 200000

[[models]]
pattern = "qwen3-coder"
display_name = "Qwen Coder"
context_limit = 256000

[[context_modifiers]]
pattern = "[1m]"
display_suffix = " 1M"
context_limit = 1000000
```

## 状态栏数据来源

Claude Code 会通过 stdin 把 statusLine 数据传给命令。`best-claude-hud` 会读取：

- `model`
- `workspace.project_dir`，用于稳定表示 Claude Code 启动目录
- `workspace.current_dir`，用于兼容未提供 `project_dir` 的旧版 Claude Code
- `transcript_path`
- `cost`
- `output_style`
- `rate_limits`

对于 context window 占用，HUD 只读取当前活跃 transcript 文件。如果新终端/新会话还没有 transcript，它会显示 `0% · 0 tokens`，不会扫描项目目录里的旧历史文件。这修复了“新开终端仍沿用上一个 Claude Code 会话 token 占用”的旧行为。

## Git 状态标识

- `✓`：工作树干净
- `●`：存在未提交变更
- `⚠`：存在冲突
- `↑n`：领先 upstream n 个 commit
- `↓n`：落后 upstream n 个 commit

Git 命令使用 `--no-optional-locks`，避免状态栏刷新时造成不必要的 `.git/index.lock` 竞争。

## Claude Code Patch 工具

继承自上游的 patcher 可用于降低 Claude Code context warning 噪音：

```bash
best-claude-hud --patch /path/to/claude-code/cli.js
```

示例：

```bash
best-claude-hud --patch ~/.local/share/fnm/node-versions/v24.4.1/installation/lib/node_modules/@anthropic-ai/claude-code/cli.js
```

patcher 会在写入前创建同目录备份文件。

## 平台支持

| 平台 | 原生二进制来源 | 状态 |
| --- | --- | --- |
| MacOS arm64 | npm 自动选择原生二进制 | 支持 |
| MacOS x64 | npm 自动选择原生二进制 | 支持 |
| Linux x64 musl | npm 自动选择原生二进制 | 支持 |
| Windows x64 | npm 自动选择原生二进制 | 支持 |
| Linux arm64 / Windows arm64 | - | 计划中 |

## 系统要求

- 支持 `statusLine` 的 Claude Code
- Git，用于分支与状态显示
- 支持 ANSI color 的终端
- 如果使用 Nerd Font 或 Powerline 主题，需要配置 Nerd Font

## 开发

维护者与贡献者可从源码运行：

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo build --release
cargo run -- --help
npm --prefix packaging/npm run check
npm --prefix packaging/npm run test
```

发布前可做 npm dry-run：

```bash
cargo build --release
mkdir -p release-artifacts
tar -C target/release -czf release-artifacts/best-claude-hud-darwin-arm64.tar.gz best-claude-hud
node packaging/npm/scripts/build-packages.js \
  --version 0.1.6 \
  --release-dir release-artifacts \
  --output-dir npm-tarballs
```

## 发布

发布拆成两个 workflow：

- `Release`：构建 GitHub Release artifacts 和 npm tarballs
- `npm publish`：在 release artifacts 存在后手动发布 npm 包

创建 GitHub Release：

```bash
git tag v0.1.6
git push origin v0.1.6
```

npm trusted publishing 配置完成后发布：

```bash
gh workflow run "npm publish" --repo GaoSSR/best-claude-hud -f version=0.1.6
```

## 项目资源

- [Changelog](./CHANGELOG.md)
- [贡献指南](./CONTRIBUTING.md)
- [安全策略](./SECURITY.md)
- [上游 PR/Issue 接收策略](./docs/triage.md)

## 致谢

第三方版权归属保留在 [NOTICE](./NOTICE)。

## License

本项目采用 [Apache License 2.0](./LICENSE)。
