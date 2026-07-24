<h4 align="right"><a href="./README.md">English</a> | <a href="./README_CN.md">简体中文</a> | <strong><a href="./README_JA.md">日本語</a></strong></h4>

<p align="center">
  <a href="https://github.com/GaoSSR/best-claude-hud">
    <img src="assets/best-claude-hud-logo.png" alt="best-claude-hud" width="480">
  </a>
</p>

<h3 align="center"><nobr>Rust 製のミニマルな Claude Code ステータスライン HUD</nobr></h3>

---

<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-CLI-orange" />
  <img alt="MacOS Linux Windows 対応" src="https://img.shields.io/badge/MacOS%20%7C%20Linux%20%7C%20Windows-supported-brightgreen" />
  <img alt="コマンド: best-claude-hud" src="https://img.shields.io/badge/command-best--claude--hud-8A2BE2" />
  <img alt="ライセンス: Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-blue" />
</p>

## best-claude-hud の概要

`best-claude-hud` は、Rust で書かれた高性能な Claude Code ステータスラインツールです。ターミナルで Claude Code を使用するときに実際に必要なステータス情報、すなわちモデルとリアルタイムの推論強度、ワークスペース、Git ブランチと状態、コンテキストウィンドウの使用量、さらにオプションの使用量とレート制限のメタデータを表示します。

<p align="center">
  <img src="assets/best-claude-hud-preview.png" alt="best-claude-hud ステータスラインのプレビュー" width="1200">
</p>

デフォルトのステータスラインは、次の情報を重視しています。

- Claude モデル。利用可能な場合はリアルタイムの推論強度も表示
- 一時的に作業ディレクトリが変わっても維持される Claude Code の起動ディレクトリ
- Git ブランチ、clean/dirty/conflict 状態、ahead/behind のコミット数
- Claude Code 公式の statusLine データから取得したコンテキストウィンドウ使用量。利用できない場合はアクティブな transcript にフォールバック
- オプションの使用量とレート制限、コスト、セッション、出力スタイルの各セグメント

## インストール

`best-claude-hud` は npm を通じて配布されています。npm パッケージはビルド済みのネイティブバイナリを使用するため、ユーザーが Rust をインストールする必要はありません。

次の 1 行で `best-claude-hud` をインストールし、Claude Code を設定できます。

```bash
npm install -g best-claude-hud@latest && best-claude-hud --setup
```

設定後に Claude Code を再起動してください。既存のセッションでは `~/.claude/settings.json` が自動的に再読み込みされません。

インストールのみを行う場合:

```bash
npm install -g best-claude-hud@latest
```

yarn または pnpm を使用する場合:

```bash
yarn global add best-claude-hud@latest
pnpm add -g best-claude-hud@latest
```

中国国内のユーザー向け:

```bash
npm install -g best-claude-hud@latest --registry https://registry.npmmirror.com && best-claude-hud --setup
```

既存のインストールを更新する場合:

```bash
npm install -g best-claude-hud@latest
```

アンインストール:

```bash
npm uninstall -g best-claude-hud
```

## Nix

`best-claude-hud` は、宣言的で再現可能な環境向けに Nix flake も提供しています。

グローバルにインストールせず実行する場合:

```bash
nix run github:GaoSSR/best-claude-hud -- --help
```

Nix profile にインストールする場合:

```bash
nix profile install github:GaoSSR/best-claude-hud
best-claude-hud --setup
```

home-manager または別の宣言的な設定を使用する場合は、Claude Code から Nix store のバイナリを直接参照します。以下の例は `~/.claude/settings.json` ファイル全体を宣言的に管理し、既存の設定はマージしません。新規ファイルを作成する場合、または Claude Code のすべての設定を同じ Nix 構成で管理する場合にのみ使用してください。

```nix
# In your flake inputs:
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

`~/.claude/settings.json` を手動で管理し続ける場合は、`best-claude-hud --setup` を実行するか、`statusLine` ブロックを直接追加してください。この `home.file` 宣言は使用しないでください。すでに Nix で管理している場合は、既存の Nix 式に `statusLine` を追加してください。管理対象外のファイルを Home Manager に移行する場合は、既存の設定をすべて Nix に移してから、アクティベーション前に元のファイルを別の場所へ移動してください（バックアップ名への変更など）。

開発シェル:

```bash
nix develop
```

## Claude Code の設定

`npm install -g best-claude-hud@latest` はコマンドをインストールするだけです。`statusLine` を設定するまで、Claude Code に HUD は表示されません。

推奨設定:

```bash
best-claude-hud --setup
```

セットアップコマンドは、既存の設定を維持したまま `statusLine` ブロックを `~/.claude/settings.json` に書き込みます。可能な場合は、インストール済みのコマンドを絶対パスに解決します。

```json
{
  "statusLine": {
    "type": "command",
    "command": "/path/to/best-claude-hud",
    "padding": 0
  }
}
```

Claude Code のセッションがシェルと同じ PATH を継承する場合、手動設定で `"command": "best-claude-hud"` を使用することもできます。`statusLine` がすでに存在する場合、`--setup` は置き換える前に `settings.json` と同じ場所へタイムスタンプ付きのバックアップを作成します。このファイルを変更した後は Claude Code を再起動してください。

npm パッケージは、意図的に `~/.claude` へバイナリをインストールしません。グローバル npm コマンドを使用し、Kiri 形式の npm alias optional dependencies から対応するネイティブバイナリを解決します。

## コマンド

```bash
best-claude-hud                    # open the interactive menu when run in a terminal
best-claude-hud --help             # print command help
best-claude-hud --version          # print version
best-claude-hud --setup            # configure Claude Code statusLine
best-claude-hud --config           # open the TUI configuration interface
best-claude-hud --theme minimal    # temporarily render with a built-in theme
best-claude-hud --patch <cli.js>   # patch Claude Code cli.js context warnings
```

## テーマ

設定済みのテーマを一時的に上書きする場合:

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

カスタムテーマは次のディレクトリに保存できます。

```text
~/.claude/best-claude-hud/themes/
```

保存後は次のように使用します。

```bash
best-claude-hud --theme my-custom-theme
```

## 設定

設定ファイルは次のディレクトリに保存されます。

```text
~/.claude/best-claude-hud/
```

重要なファイル:

- `config.toml`: HUD とセグメントのメイン設定
- `models.toml`: モデルの表示名とコンテキストウィンドウ上限
- `themes/*.toml`: カスタムテーマのプリセット
- `.api_usage_cache.json`: オプションの使用量 API キャッシュ
- `.update_state.json`: 更新チェックの状態

TUI 設定ツールを実行する場合:

```bash
best-claude-hud --config
```

使用可能なセグメントの種類:

- `model`
- `directory`
- `git`
- `context_window`
- `usage`
- `cost`
- `session`
- `output_style`
- `update`

## モデル設定

`models.toml` は初回実行時に自動的に作成されます。

```text
~/.claude/best-claude-hud/models.toml
```

このファイルでモデルの表示名とコンテキスト上限を制御します。Claude のモデルファミリーは自動的に認識され、サードパーティ製モデルはカスタマイズできます。

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

## ステータスラインのデータ

Claude Code は、statusLine データを stdin 経由でコマンドに送信します。`best-claude-hud` は次の項目を読み取ります。

- `model`
- `effort.level`: 現在のモデルが推論強度をサポートしている場合は独立した項目として表示
- `workspace.project_dir`: 安定した Claude Code 起動ディレクトリ
- `workspace.current_dir`: 古いバージョンの Claude Code 向けフォールバック
- `transcript_path`
- `session_id`: Ultracode の検出範囲を現在の Claude Code プロセスに限定するために使用
- `context_window`
- `cost`
- `output_style`
- `rate_limits`

推論強度の項目はモデル名の後に続き、明示的な ASCII パイプと脳のアイコンで区切られます。たとえば `Kimi K2.7 | 🧠 max` のように表示されます。ラベルは明るい紫色（`#B45CFF`）で描画されます。HUD は `low`、`medium`、`high`、`xhigh`、`max`、`ultracode` を表示し、Nerd Font モードと Powerline モードでは対応する Nerd Font の脳グリフを使用します。

Claude Code 公式のステータスラインペイロードでは、Ultracode は `xhigh` として報告されます。通常の `xhigh` と Ultracode を区別するため、HUD は現在の Claude Code プロセスで成功した `/effort` イベントのみを照合します。会話を再開しても、以前のプロセスで発生したイベントは無視されます。`xhigh` の `CLAUDE_CODE_EFFORT_LEVEL` オーバーライドは Ultracode と互換性がありますが、その他の有効なオーバーライドがある場合は Ultracode として報告されません。Claude Code が推論強度のデータを提供しない場合、非対応モデルとサードパーティ製モデルでは従来どおりモデル名のみが表示されます。Claude Code のバージョンフィールドは意図的に表示されません。

コンテキストウィンドウの使用量については、HUD は Claude Code 公式の `context_window` フィールドを優先します。これらのフィールドが存在しない、null、または一時的にゼロの場合に限り、アクティブな transcript を互換性のためのフォールバックとして使用します。応答が中断された後に書き込まれる全項目がゼロの使用量プレースホルダーは無視されるため、`Esc` を押しても最後の有効なコンテキスト表示は消去されません。実際に新しいセッションで使用量がまだない場合は `0% · 0 tokens` と表示され、古いプロジェクト履歴が検索されることもありません。

## Git の状態表示

- `✓`: clean な作業ツリー
- `●`: dirty な作業ツリー
- `⚠`: conflict あり
- `↑n`: upstream より n コミット先行
- `↓n`: upstream より n コミット遅れ

Git コマンドは `--no-optional-locks` 付きで実行されるため、HUD の動作中に不要な `.git/index.lock` の競合を発生させません。

## Claude Code パッチユーティリティ

継承したパッチツールを使用して Claude Code の `cli.js` にパッチを適用し、コンテキスト警告のノイズを減らすことができます。

```bash
best-claude-hud --patch /path/to/claude-code/cli.js
```

例:

```bash
best-claude-hud --patch ~/.local/share/fnm/node-versions/v24.4.1/installation/lib/node_modules/@anthropic-ai/claude-code/cli.js
```

パッチツールは書き込み前に対象ファイルと同じ場所へバックアップを作成します。

## 対応プラットフォーム

| プラットフォーム | ネイティブバイナリの提供元 | 状態 |
| --- | --- | --- |
| MacOS arm64 | npm によりネイティブバイナリを自動選択 | 対応 |
| MacOS x64 | npm によりネイティブバイナリを自動選択 | 対応 |
| Linux x64 musl | npm によりネイティブバイナリを自動選択 | 対応 |
| Windows x64 | npm によりネイティブバイナリを自動選択 | 対応 |
| Linux arm64 / Windows arm64 | - | 対応予定 |

## 要件

- `statusLine` をサポートする Claude Code
- ブランチと状態の表示に使用する Git
- ANSI カラーをサポートするターミナル
- Nerd Font または Powerline テーマを選択する場合は Nerd Font

## 開発

ソースから作業するメンテナーとコントリビューター向け:

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo build --release
cargo run -- --help
npm --prefix packaging/npm run check
npm --prefix packaging/npm run test
```

## リリース

メンテナーは [RELEASING.md](./RELEASING.md) に記載された、承認ゲート付きの完全なチェックリストに従う必要があります。この文書が、バージョン更新、ローカル検証、Git タグ、GitHub Releases、npm への公開、ローカル環境でのアップグレードに関する正式な手順です。

## プロジェクトリソース

- [変更履歴](./CHANGELOG.md)
- [コントリビューションガイド](./CONTRIBUTING.md)
- [リリース手順](./RELEASING.md)
- [セキュリティポリシー](./SECURITY.md)
- [アップストリームのトリアージ](./docs/triage.md)

## 謝辞

サードパーティの帰属情報は [NOTICE](./NOTICE) に保持されています。

## ライセンス

[Apache License 2.0](./LICENSE) の下でライセンスされています。
