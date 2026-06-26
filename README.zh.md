# best-claude-hud

用 Rust 写的极简 Claude Code statusline HUD。

`best-claude-hud` 是基于
[`Haleclipse/CCometixLine`](https://github.com/Haleclipse/CCometixLine)
源代码的维护型 fork/rebuild。目标不是堆满信息，而是保留真正高价值的单行状态栏：
少污染屏幕、信息稳定、适合长期日常使用。

![Preview](assets/img1.png)

## 安装

```bash
npm install -g best-claude-hud
```

国内网络可用 npm 镜像：

```bash
npm install -g best-claude-hud --registry https://registry.npmmirror.com
```

源码安装：

```bash
git clone https://github.com/GaoSSR/best-claude-hud.git
cd best-claude-hud
cargo install --path .
```

## 配置 Claude Code

在 `~/.claude/settings.json` 中配置：

```json
{
  "statusLine": {
    "type": "command",
    "command": "best-claude-hud"
  }
}
```

npm 包不会把二进制复制到 `~/.claude`。命令会通过 npm 的全局 bin shim 运行，并从
optional dependencies 中加载当前平台对应的二进制。

## 使用

```bash
best-claude-hud --help
best-claude-hud --config
best-claude-hud --theme minimal
```

配置目录：

```text
~/.claude/best-claude-hud/
```

默认 HUD 关注这些信息：

- 当前模型
- 项目目录
- Git 分支/状态
- context window 占用
- 可选的 usage/rate limit、cost、session、output style

## 维护方向

这个 fork 的第一版目标是稳定、简洁、可发布。新增功能默认必须保持单行、可配置、
不污染状态栏。上游 PR/Issue 的初始接收策略见 [docs/triage.md](docs/triage.md)。

## 发布

CI 会跑格式检查、clippy、测试和 release 构建冒烟。GitHub Release 与 npm publish
拆成两个 workflow，便于单独控制 npm 凭证或 trusted publishing。

```bash
git tag v0.1.0
git push origin v0.1.0
```

## 许可证与致谢

MIT licensed. See [LICENSE](LICENSE).

本项目基于 `Haleclipse/CCometixLine` 的源代码，该项目在 Cargo metadata 中声明为
MIT。版权与来源说明见 [NOTICE](NOTICE)。
