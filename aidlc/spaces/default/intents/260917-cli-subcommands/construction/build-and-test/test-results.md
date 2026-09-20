# Test Results — 260917-cli-subcommands

## ビルド状況

`cargo build --locked`: **成功**。警告0件。`cargo run -q -- --version` → `cwsweep 0.2.0`。

## テスト結果

| 種別 | 総数 | Pass | Fail | Skip |
|---|---|---|---|---|
| Unit（`cargo test --locked --lib`） | 143 | 143 | 0 | 0 |
| Unit（`src/main.rs`） | 4 | 4 | 0 | 0 |
| 統合（`tests/audit_log_format.rs`, `tests/scan_select_execute.rs`） | 4 | 4 | 0 | 0 |
| Doc-tests | 0 | 0 | 0 | 0 |
| **合計** | **151** | **151** | **0** | **0** |

Unit別コマンド（重複排除後、各1回実行）:
- `cargo test --lib audit_reader_tests:: --locked`（audit-reader）— pass
- `cargo test --lib cli:: --locked` / `cargo test --lib output:: --locked`（cli-foundation）— pass
- `cargo build --locked && cargo run -q -- --version`（release-docs）— `cwsweep 0.2.0`

## 静的解析・フォーマット・サプライチェーン

- `cargo fmt --check`: 差分なし
- `cargo clippy --all-targets --locked -- -D warnings`: 警告0件
- `cargo audit`: 脆弱性0件（`fxhash` unmaintained 警告1件 — `deny.toml` 許容済み）
- `cargo deny check`: advisories / bans / licenses / sources すべて ok
- `#![forbid(unsafe_code)]`: `src/main.rs:7`, `src/lib.rs` に確認

## カバレッジレポート（`cargo llvm-cov --lib --summary-only`）

| モジュール | 行カバレッジ |
|---|---|
| aggregator.rs | 100.00% |
| audit.rs | 93.10% |
| cli.rs | 95.41% |
| confirmation.rs | 100.00% |
| credentials.rs | 100.00% |
| error.rs | 95.24% |
| execution.rs | 95.35% |
| identity.rs | 99.26% |
| org_discovery.rs | 100.00% |
| output.rs | 98.15% |
| planner.rs | 100.00% |
| scanner.rs | 99.52% |
| selector.rs | 100.00% |
| **TOTAL** | **96.63%** |

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|---|---|---|---|---|---|---|
| TC-COV-80 | team.md カバレッジ基準 / cli-foundation code-generation-plan.md Testing Contract | 全モジュール行カバレッジ ≥ 80% | 最低 93.10%（audit.rs）、全体 96.63% | 上記 llvm-cov 表 | build-and-test | Met |
| TC-COV-DISPATCH | team.md Q3（ディスパッチ分岐が `--lib` 計測対象） | `Commands` / `run_*` が lib クレート側 | `src/cli.rs` に定義、cli.rs 95.41% | llvm-cov 表、`src/main.rs` は wiring のみ | build-and-test | Met |
| NFR1 | requirements.md（テスト容易性） | ディスパッチが lib 側、main は薄い wiring | `run_scan` / `run_clean` / `run_audit` は `src/cli.rs`；main は `wire_aws` + `match` のみ | `cli::tests::run_*`（12件） | build-and-test | Met |
| NFR2 / NFR2.1–2.4 | requirements.md, audit-reader & cli-foundation security-requirements.md | `audit` の依存グラフに書き込み・破壊型が無い；`scan` が書き込み非到達 | `run_audit(&dyn AuditRead, OutputFormat, &mut dyn Write)`；`ScanApp` は credential/identity/logs のみ | `audit::audit_reader_tests` 構造隔離テスト、`src/cli.rs` 型シグネチャ | build-and-test | Met |
| NFR3 / NFR3.1–3.3 | requirements.md, audit-reader security-requirements.md | 不正行を含むログで正常行が表示される；stderr 警告 | 不正 JSON / 欠落フィールド / 空行の各テスト pass、`run_audit_json_output_lists_every_recorded_entry` | `cargo test --lib audit_reader_tests::` | build-and-test | Met |
| NFR4 / NFR4.1 | requirements.md, release-docs security-requirements.md | 旧フラグにフォールバック無し；移行ガイド | `legacy_flat_flags_without_subcommand_are_rejected` 等3件 pass；README / CHANGELOG に移行表 | `cargo test --lib cli::tests::`, README.md, CHANGELOG.md | build-and-test | Met |
| NFR5 / NFR5.1–5.18 | requirements.md, cli-foundation security/reliability/observability-requirements.md | dry-run 既定、二重 Identity 検証、監査ログ必須、クレデンシャル非露出、unsafe/panic 禁止、全滅時非ゼロ終了 | `run_clean_dry_run_default_never_calls_delete_api`、`clean_audit_log_path_defaults_*`、`run_scan_returns_fully_failed_*`、`credentials::tests`、clippy -D warnings 合格、`forbid(unsafe_code)` | `cargo test --locked`, `cargo clippy` | build-and-test | Met |
| REL-DOCS-VER | release-docs tech-stack-decisions.md | `Cargo.toml` = 0.2.0、`--locked` ビルド成功 | `cwsweep 0.2.0`、`cargo build --locked` 成功 | `cargo run -q -- --version` | build-and-test | Met |
| PERF-REG | cli-foundation performance-requirements.md | スキャン経路の呼び出し回数不変 | `scanner::` テスト pass、`run_scan` はスキャン1回 | `cargo test --lib scanner::` | build-and-test | Met |

`Not Met` / `Unverified` / `Pending`: なし。
