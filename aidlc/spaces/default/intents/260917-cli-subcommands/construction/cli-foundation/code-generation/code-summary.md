# Code Summary — Unit: cli-foundation

## 実装したもの

- `src/cli.rs`
  - `Cli { command: Commands }`、`Commands::{Scan(ScanArgs), Clean(CleanArgs), Audit(AuditArgs)}`。サブコマンド未指定・所属外オプション・旧フラグはすべて clap のパースエラー。
  - `ScanApp`（credential_provider / identity / logs_client のみ）と `ScanApp::scan_all`。`CliApp::scan_all` は委譲。
  - `ExitDisposition::{Success, ScanFullyFailed { attempted }, Error(String)}`。
  - `ScanApp::run_scan`、`CliApp::run_clean`（`stdin_is_tty` 注入、`CleanInteraction` で対話依存を受け取る）、`run_audit(&dyn AuditRead, OutputFormat, &mut dyn Write)`。
  - `format_outcome_line` を main から移動。
- `src/output.rs`: `OutputFormatter::format_audit`（table は run_id を除く 8 列、intent 行の Success は `-`; json は `AuditReadEntry` 配列）。
- `src/main.rs`: `wire_aws` / `exit_with` / `match cli.command`。`Audit` 分岐は AWS SDK を一切実体化しない。`Clean` 分岐は監査ログ open を最初に行う。

## 安全性の担保

- `run_audit` / `ScanApp` のシグネチャに `AuditWrite` / `ActionApiOperations` / `ExecutionEngine` は現れない（コンパイル時制約、NFR2.3 / NFR2.4）。
- `--execute` 未指定時に delete API が呼ばれないことを `run_clean_dry_run_default_never_calls_delete_api` で固定化。
- 非 TTY の `clean` は選択・確認・実行に進まない（`run_clean_without_tty_prints_scan_and_never_reaches_execution`）。

## デビエーション

- 設計の `ScanDeps` / `CleanDeps` は `ScanApp` / 既存 `CliApp` として実現（nfr-design レビュー R-01 で許容済み）。
- 0 件メッセージ: `scan` は「ロググループは見つかりませんでした。」、`clean` は従来どおり「削除対象のロググループはありません。」。

## テストの実行方法

```bash
cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

結果: lib 143 passed、bin 4 passed、integration 4 passed、0 failed。

## 変更しなかったもの

`src/execution.rs`、`src/scanner.rs`、`src/credentials.rs`、`src/identity.rs`、`src/audit.rs`（書き込み側）、`Cargo.toml` 依存。
