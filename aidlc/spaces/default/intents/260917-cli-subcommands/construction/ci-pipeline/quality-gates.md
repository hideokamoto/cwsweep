# Quality Gates — 260917-cli-subcommands

マージ前必須ゲート（既存 `.circleci/config.yml`、すべてブロッキング、変更なし）:

| ゲート | コマンド | Build and Test での結果 |
|---|---|---|
| フォーマット | `cargo fmt --check` | 合格 |
| Lint | `cargo clippy --all-targets --locked -- -D warnings` | 合格（警告0） |
| ビルド | `cargo build --locked` | 合格（0.2.0） |
| テスト | `cargo test --locked` | 151 / 151 pass |
| カバレッジ | `cargo llvm-cov --lib --locked --fail-under-lines 80 --summary-only` | 96.63%（最低モジュール audit.rs 93.10%） |
| 依存脆弱性 | `cargo audit` | 脆弱性0（fxhash unmaintained 警告は deny.toml 許容済み） |
| サプライチェーン | `cargo deny check` | 合格 |

## 本intent固有の追加ゲート（テストとして CI に組み込まれている）

| 保証したい性質 | 強制手段 |
|---|---|
| 旧フラグ形式が復活しない（FR6.1 / NFR4） | `cli::tests::legacy_flat_flags_without_subcommand_are_rejected` 等が `test` ジョブで失敗する |
| `audit` が破壊的型に依存しない（FR4.5 / NFR2） | `run_audit` シグネチャ + `audit::audit_reader_tests` 構造隔離テスト |
| dry-run 既定（FR3.3） | `run_clean_dry_run_default_never_calls_delete_api` |
| 監査ログ無効化オプションが存在しない（FR3.5） | `clean_audit_log_path_defaults_*`、clap 定義に off スイッチなし |

## CI実行から除外される項目

intent 260910 と同じ（破壊的パスの100%カバレッジは人間レビュー、`#[ignore]` の実AWSテストは通常実行外）。

## リリースゲート

0.2.0 は CLI の破壊的変更を含むため、`v0.2.0` タグ作成前に CHANGELOG の移行ガイドを確認し、team.md の M6 サインオフ（サンドボックスでの `clean --execute` 動作確認）を行う。本intentではタグを打たない。

## Assumptions & Open Questions

None.
