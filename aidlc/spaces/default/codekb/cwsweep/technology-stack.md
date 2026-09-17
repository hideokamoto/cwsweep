# technology-stack.md — cwsweep

## 言語・ツールチェーン

- **言語**: Rust, edition 2021
- **ツールチェーン**: `channel = "1.98.1"`（`rust-toolchain.toml` で固定）、
  `rustfmt` / `clippy` コンポーネント込み
- **ビルドシステム**: Cargo（`Cargo.toml` に `[[bin]] cwsweep` と `[lib] cwsweep`
  を同一パッケージ内に定義）。`Cargo.lock` はコミット済み、CI は `--locked`。

## 依存クレート（`[dependencies]`）

| クレート | バージョン | 用途 |
|---|---|---|
| `aws-config` | 1.x（`default-https-client`, `rt-tokio`, `credentials-process`, `sso`） | AWS SDK 設定・クレデンシャル解決 |
| `aws-sdk-organizations` | 1.x（`rt-tokio`） | Organizations API（`ListAccounts`） |
| `aws-sdk-sts` | 1.x（`rt-tokio`） | STS API（`AssumeRole` / `GetCallerIdentity`） |
| `aws-sdk-cloudwatchlogs` | 1.x（`rt-tokio`） | CloudWatch Logs API |
| `tokio` | 1.x（`rt-multi-thread`, `macros`） | 非同期ランタイム（`#[tokio::main]`） |
| `clap` | 4.x（`derive`） | CLI 引数解析 |
| `inquire` | 0.7 | 対話式マルチセレクト・確認・単一選択・数値入力プロンプト（`main.rs` のアダプタでのみ使用） |
| `serde` | 1.x（`derive`） | シリアライズ |
| `serde_json` | 1.x | JSON シリアライズ（監査ログ、JSON出力） |
| `comfy-table` | 7.x | table 出力フォーマット |
| `thiserror` | 1.x | エラー型定義 |
| `secrecy` | 0.8（`serde`） | 一時クレデンシャルのマスキング |
| `tracing` | 0.1 | ログ出力 |
| `tracing-subscriber` | 0.3（`env-filter`） | ログ出力設定 |
| `uuid` | 1.x（`v4`） | 実行ごとの `run_id` 生成 |
| `async-trait` | 0.1 | 非同期トレイトの記述 |
| `chrono` | 0.4（`serde`） | 監査ログのタイムスタンプ（RFC3339） |

## 開発依存（`[dev-dependencies]`）

| クレート | バージョン | 用途 |
|---|---|---|
| `tempfile` | 3 | テスト用一時ディレクトリ/ファイル |

## ビルドプロファイル

- `[profile.release] opt-level = 3`

## CI/CD 基盤

- **CircleCI 2.1**（`.circleci/config.yml`）、orb `circleci/rust@1.6.0`、
  executor は `cimg/rust:1.98.1` の Docker イメージ。
- ジョブ: `fmt`, `clippy`, `test`, `coverage`, `audit`（週次のみ）,
  `deny`（通常 `bans licenses sources`、週次に `advisories` 追加）,
  `release-linux` / `release-macos`（タグ駆動）, `publish-github-release`。
- ワークフロー: `build-and-test`（push/PR時）、`weekly-security`
  （scheduled pipeline パラメータ `run-weekly-security` 経由）、
  `release`（`v\d+\.\d+\.\d+` タグ push 時）。

## 品質・セキュリティツール

- `cargo llvm-cov`（カバレッジ計測、`--lib --fail-under-lines 80`）
- `cargo audit`（依存脆弱性スキャン、週次）
- `cargo deny check`（advisories/licenses/bans/sources、`deny.toml`）

## 配布形態

サーバーデプロイなし。バージョンタグ push をトリガーに CircleCI が
クロスプラットフォーム（Linux/macOS、x86_64/arm64）バイナリをビルドし
GitHub Releases に添付するタグ駆動リリース方式（`team.md` Deployment）。
crates.io 公開は v1 スコープ外。
