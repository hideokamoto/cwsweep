# Integration Test Instructions — 260917-cli-subcommands

## フレームワーク

`cargo test`（`tests/` 配下の統合テスト、および `src/cli.rs` 内のハンドラ結合テスト）。モックはトレイト実装で注入し、実AWSには接触しない。

## 実行コマンド

```bash
cargo test --locked --test scan_select_execute --test audit_log_format   # 既存統合シナリオ（回帰）
cargo test --locked --lib cli::tests::run_                               # run_scan / run_clean / run_audit の結合
cargo test --locked --lib audit::audit_reader_tests::                    # audit-reader
```

## クロスUnit境界テスト

| 境界 | テスト | 検証内容 |
|---|---|---|
| cli-foundation ↔ audit-reader | `run_audit_json_output_lists_every_recorded_entry` | `AuditLogger` で書いたエントリを `AuditReader` 経由で `run_audit` が全件表示 |
| cli-foundation ↔ audit-reader | `run_audit_with_missing_file_succeeds_with_empty_table` | ファイル不在 → 空結果・正常終了 |
| cli-foundation ↔ audit-reader | `run_audit_reports_error_when_path_is_a_directory` | I/Oエラーの伝播 |
| cli-foundation ↔ execution | `run_clean_dry_run_default_never_calls_delete_api` | `--execute` 無しで削除APIが呼ばれない |
| cli-foundation ↔ execution | `run_clean_with_execute_and_confirmation_calls_delete_api` | `--execute`＋確認で1回だけ実行 |
| cli-foundation ↔ selector | `run_clean_without_tty_prints_scan_and_never_reaches_execution` | 非TTYで選択・確認・実行に到達しない |
| cli-foundation ↔ scanner | `run_scan_returns_fully_failed_when_every_account_region_fails` | 全滅時の終了コード方針 |

## テストデータ

一時ディレクトリ（`tempfile`）に JSON Lines を書き出して読み戻す。固定フィクスチャは不要。
