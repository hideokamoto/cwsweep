# Security Test Instructions — 260917-cli-subcommands

## 目的

構造的安全性（NFR2）・後方互換の意図的放棄（NFR4）・既存安全機構の継続（NFR5）を自動テストと静的検査で確認する。

## 実行コマンド

```bash
cargo clippy --all-targets --locked -- -D warnings      # unwrap/expect/panic 禁止ポリシー
grep -n 'forbid(unsafe_code)' src/main.rs src/lib.rs   # unsafe 禁止
cargo audit                                            # 脆弱性（fxhash unmaintained は deny.toml で許容済みの既知警告）
cargo deny check
cargo test --locked --lib cli::tests::                 # 下記の安全テスト群
cargo test --locked --lib audit::                      # 読み取り側が書き込み型に依存しない構造テスト
```

## 安全テスト対応表

| 要件 | テスト |
|---|---|
| NFR2 / FR4.5 `audit` が破壊的・書き込み型に依存しない | `run_audit(reader: &dyn AuditRead, ...)` のシグネチャ、`audit::audit_reader_tests` の構造隔離テスト、`ScanApp` に `ActionApiOperations`/`AuditWrite` フィールドが無い |
| FR3.3 dry-run 既定 | `run_clean_dry_run_default_never_calls_delete_api`, `clean_execute_defaults_to_false_and_is_true_only_when_supplied` |
| FR2.2 `scan` が監査ログを書かない | `scan_rejects_execute_and_audit_log_path`, `ScanApp` に audit_logger 無し |
| FR3.5 監査ログ必須 | `clean_audit_log_path_defaults_to_cwsweep_audit_jsonl_and_can_be_overridden`（無効化オプション無し） |
| FR5.1 / FR5.2 二重 Identity 検証 | 既存 `identity::` / `execution::` テスト（本intentで変更なし）|
| NFR4 / FR6.1 旧フラグ拒否 | `missing_subcommand_is_a_parse_error`, `legacy_flat_flags_without_subcommand_are_rejected`, `scan_only_flag_no_longer_exists_on_any_subcommand` |
| FR4.4 不正行スキップ | `audit_reader_tests::*skips*` |
| クレデンシャル非露出 | `credentials::tests`（`[REDACTED]`） |

## 手動確認（実AWS不要）

```bash
cargo run -q -- --regions us-east-1          # error: unexpected argument '--regions' found
cargo run -q -- audit --audit-log-path /nonexistent.jsonl   # 空結果・exit 0、AWS呼び出し無し
```
