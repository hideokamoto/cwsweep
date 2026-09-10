# Integration Test Instructions — cwsweep

Standard戦略に基づき、主要境界（スキャン→選択→削除の一連の流れ、監査ログ出力）を検証する統合テストを実行する。

## テストフレームワーク

`cargo test`の標準統合テストランナー（`tests/`直下の各ファイルが独立バイナリとしてコンパイルされる）。モックのみで完結し、実AWSアカウントへは一切接触しない。

## 実行コマンド

```bash
cd /home/user/cwsweep
cargo test --test scan_select_execute --test audit_log_format
```

## テスト内容

- `tests/scan_select_execute.rs`:
  - `full_pipeline_scan_select_confirm_dry_run_never_calls_delete_api` — dry-run既定でスキャン→選択→確認の一連の流れを通し、削除APIが一切呼ばれないことを検証（NFR2.4のクロスモジュール検証）
  - `full_pipeline_with_execute_true_calls_delete_and_writes_audit_log` — `--execute`時に削除APIが呼ばれ、監査ログが書き込まれることを検証
- `tests/audit_log_format.rs`:
  - `audit_log_is_valid_json_lines_with_all_mandated_fields` — 監査ログの必須フィールド（対象アカウントID・リージョン・ログループ名・実行時刻・成功/失敗）を検証
  - `audit_log_write_failure_is_reported_as_error_not_silently_ignored` — 監査ログ書き込み失敗時に操作が中断されエラーとして扱われることを検証

## カバレッジ目標

統合テストはユニットテストのカバレッジ集計（`cargo llvm-cov --lib`）には含めない。クロスモジュール境界（scanner→aggregator→selector→planner→confirmation→execution→audit）の1件以上のend-to-endシナリオカバレッジを目標とする（本stage実行時点で2シナリオ達成: dry-run経路、--execute経路）。

## テストデータ管理

各統合テストはテスト関数内でモックのAWSクライアント（トレイト差し替え）とダミーのアカウントID・リージョン・ログループ名をインラインで構築する。共有ミュータブル状態は使用しない。監査ログの書き込み先は一時ディレクトリ（テストごとに一意）。

## Assumptions & Open Questions

None.
