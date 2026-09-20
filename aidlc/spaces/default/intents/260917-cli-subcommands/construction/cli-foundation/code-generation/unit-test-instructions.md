# Unit Test Instructions — Unit: cli-foundation

## テストフレームワーク

Rust 標準 `#[test]` / `#[tokio::test]`、既存モックアダプタ（`src/cli.rs` tests）、`tempfile`。

## 本Unitのテスト実行コマンド（厳密スコープ）

```bash
cargo test --lib cli:: --locked
cargo test --lib output:: --locked
```

## 期待されるテストケース

| 領域 | テスト |
|---|---|
| FR1.1 / FR6.1 / NFR4 | `missing_subcommand_is_a_parse_error`, `legacy_flat_flags_without_subcommand_are_rejected`, `scan_only_flag_no_longer_exists_on_any_subcommand` |
| FR2 | `scan_requires_regions`, `scan_accepts_comma_separated_regions_and_dedupes_preserving_order`, `scan_defaults_role_name_and_table_output`, `scan_accepts_json_output_and_custom_role_name`, `scan_rejects_execute_and_audit_log_path`, `run_scan_json_output_is_a_single_json_document`, `run_scan_returns_fully_failed_when_every_account_region_fails`, `run_scan_with_no_targets_succeeds_and_prints_table_notice` |
| FR3 | `clean_requires_regions_and_dedupes`, `clean_execute_defaults_to_false_and_is_true_only_when_supplied`, `clean_audit_log_path_defaults_to_cwsweep_audit_jsonl_and_can_be_overridden`, `clean_rejects_output_flag`, `run_clean_without_tty_prints_scan_and_never_reaches_execution`, `run_clean_dry_run_default_never_calls_delete_api`, `run_clean_with_execute_and_confirmation_calls_delete_api`, `run_clean_aborts_without_execution_when_confirmation_is_declined` |
| FR4 | `audit_defaults_and_overrides`, `audit_rejects_regions_and_execute`, `run_audit_with_missing_file_succeeds_with_empty_table`, `run_audit_json_output_lists_every_recorded_entry`, `run_audit_reports_error_when_path_is_a_directory`, `audit_table_hides_run_id_and_marks_intent_success_as_undetermined`, `audit_json_is_an_array_with_every_field` |
| 既存維持 | `scan_all_*`, `execute_*`, `scan_fully_failed_*`, `outcome_line_*` |

## カバレッジ目標

lib 全体 80% ライン floor（既存）。ディスパッチ層は全分岐を単体テストで網羅。

## モック/スタブ方針

AWS I/O は既存の `StubAssumeRole` / `StubIdentityOk` / `StubIdentityFailAll` / `StubDescribeLogGroups` / `RecordingApi`。対話は `FixedSelectAll` / `AlwaysConfirmPrompt` / `NeverConfirmPrompt`。実 AWS には接続しない。
