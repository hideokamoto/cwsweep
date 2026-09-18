# Developer Code Scan — cwsweep

## Developer Code Scan Results

### Scan Coverage
- **Analyzed deeply**:
  - `./`
  - `Cargo.toml`
  - `Cargo.lock`
  - `rust-toolchain.toml`
  - `deny.toml`
  - `README.md`
  - `.circleci/config.yml`
  - `src/main.rs`
  - `src/lib.rs`
  - `src/cli.rs`
  - `src/scanner.rs`
  - `src/aggregator.rs`
  - `src/audit.rs`
  - `src/confirmation.rs`
  - `src/credentials.rs`
  - `src/error.rs`
  - `src/execution.rs`
  - `src/identity.rs`
  - `src/org_discovery.rs`
  - `src/output.rs`
  - `src/planner.rs`
  - `src/selector.rs`
  - `tests/scan_select_execute.rs`
  - `tests/audit_log_format.rs`
- **Skimmed only**: なし（`.claude/`、`.cursor/`、`aidlc/`配下のAI-DLCフレームワーク自体の資材はアプリケーションコードではないため対象外とし、読み込みもしていない）

### Packages Found
- `cwsweep` — binary crate（`src/main.rs`、バイナリ名`cwsweep`） — Rust — 実AWS SDKクライアントを配線し、CLIエントリポイントとして`cwsweep`ライブラリのコンポーネントを実行する
- `cwsweep` — library crate（`src/lib.rs`） — Rust — Domain Designの12コンポーネント（1モジュール1コンポーネント）を実装する本体ロジック。バイナリcrateとテストの両方から利用される

### Build System
- **Type**: Cargo（Rust標準ビルドシステム）。`[[bin]]`（`cwsweep`、`src/main.rs`）と`[lib]`（`cwsweep`、`src/lib.rs`）を同一パッケージ内に定義する構成
- **Config Files**: `Cargo.toml`（依存関係・プロファイル）、`Cargo.lock`（コミット済み、CIは`--locked`でビルド・テスト）、`rust-toolchain.toml`（`channel = "1.98.1"`、`rustfmt`/`clippy`コンポーネント固定）、`deny.toml`（`cargo deny`のadvisories/licenses/bansポリシー）
- **Build Dependencies**（内部モジュール間の依存関係。`src/lib.rs`の`pub mod`宣言順）:
  - `main.rs`（バイナリ） → `cli`, `aggregator`, `audit`, `confirmation`, `credentials`, `error`, `execution`, `identity`, `org_discovery`, `planner`, `scanner`, `selector`（全コンポーネントを配線するエントリポイント）
  - `cli`（`CliApp`） → `aggregator`, `confirmation`, `credentials`, `error`, `execution`, `identity`, `org_discovery`, `output`, `planner`, `scanner`, `selector`（オーケストレーション層。ほぼ全モジュールに依存する唯一のハブ）
  - `scanner`（`LogGroupScanner`） → `aggregator`（`LogGroupRecord`生成）, `credentials`, `error`, `identity`
  - `execution`（`ExecutionEngine`） → `audit`, `credentials`, `error`, `identity`, `planner`
  - `confirmation`（`ConfirmationPresenter`） → `planner`
  - `output`（`OutputFormatter`） → `aggregator`
  - `selector`（`InteractiveSelector`） → `aggregator`
  - `credentials`（`CredentialProvider`） → `error`
  - `identity`（`IdentityVerifier`） → `credentials`, `error`
  - `org_discovery`（`OrgDiscovery`） → `error`
  - `audit`（`AuditLogger`） → `error`, `planner`（`ActionKind`を監査エントリに埋め込む）
  - `planner`（`ActionPlanner`） → `aggregator`（`LogGroupRecord`から`PlannedAction`を構築）, `error`
  - `aggregator`, `error` はどちらも他の内部モジュールに依存しない葉ノード

### APIs Discovered
- **CLI（clap）** — `src/cli.rs` `struct Cli` — フラグ7種:
  - `--regions <REGIONS>`（必須、複数指定可・カンマ区切り、重複除去あり）
  - `--role-name <ROLE_NAME>`（既定値 `OrganizationAccountAccessRole`）
  - `--output <table|json>`（既定値 `table`）
  - `--execute`（真偽フラグ、既定 `false`。dry-run既定を担保）
  - `--scan-only`（真偽フラグ、既定 `false`、`--execute`と`conflicts_with`）
  - `--audit-log-path <PATH>`（既定値 `cwsweep-audit.jsonl`）
  - （`clap`が自動生成する `--version` / `--help` を含む）
  - 現状はこのフラット構造の単一コマンドのみで、サブコマンド（`scan`/`clean`/`audit`等）は未実装。本intentの再構成対象そのもの。
- **内部コンポーネント境界（トレイト経由でDI可能な内部API）** — 12コンポーネント、いずれも`async_trait`または同期traitで抽象化され、テストではスタブ/フェイクを注入:
  - `CliApp`（`cli.rs`） — `scan_all`, `render_output`, `scan_fully_failed`, `select`, `plan`, `confirm`, `execute` の7メソッド（パイプライン統括の唯一の入口群）
  - `LogGroupScanner`（`scanner.rs`） — `scan_account_region`（`DescribeLogGroupsOperations`トレイト経由でCloudWatch Logs `describe-log-groups`をページ完了まで辿る）
  - `ScanAggregator`（`aggregator.rs`） — `add_all`, `records`, `is_empty`, `len`, `total_bytes`, `sorted_by_size_desc`, `filter_by_account_region`
  - `InteractiveSelector`（`selector.rs`） — `select`（`MultiSelectPrompt`トレイト経由。初期選択は常に空配列）
  - `ActionPlanner`（`planner.rs`） — `plan`（`ActionKind::Delete` / `SetRetention{days}`、retention許容値検証込み）
  - `ConfirmationPresenter`（`confirmation.rs`） — `build_summary`, `confirm`（`ConfirmPrompt`トレイト経由）
  - `ExecutionEngine`（`execution.rs`） — `execute_plan`（`ActionApiOperations`トレイト経由でCloudWatch Logs `delete-log-group` / `put-retention-policy`）
  - `CredentialProvider`（`credentials.rs`） — `credentials_for`（`AssumeRoleOperations`トレイト経由。管理アカウントはAssumeRoleせず現行クレデンシャルを使用）
  - `IdentityVerifier`（`identity.rs`） — `verify_identity` / `IdentityCheck::verify`（`CallerIdentityOperations`トレイト経由でSTS `get-caller-identity`）
  - `OrgDiscovery`（`org_discovery.rs`） — `list_active_accounts`（`ListAccountsOperations`トレイト経由でOrganizations `list-accounts`、ページネーション込み、`Status=ACTIVE`フィルタ）
  - `AuditLogger`（`audit.rs`） — `append`（`AuditWrite`トレイト経由、JSON Lines・fsync付き、`Intent`/`Result`二段階記録）
  - `OutputFormatter`（`output.rs`） — `format`（table/json切り替え）
- **外部AWS API（`src/main.rs`のアダプタ経由で呼び出す実サービス呼び出し）**:
  - STS: `AssumeRole`（`StsAssumeRoleAdapter`）, `GetCallerIdentity`（`StsCallerIdentityAdapter`）
  - Organizations: `ListAccounts`（`OrganizationsListAccountsAdapter`、ページネーション完全対応）
  - CloudWatch Logs: `DescribeLogGroups`（ページネーション対応）, `DeleteLogGroup`, `PutRetentionPolicy`（`CloudWatchLogsAdapter`）

### Frameworks & Libraries
- `aws-config` 1.x（`default-https-client`, `rt-tokio`, `credentials-process`, `sso`機能） — AWS SDK設定・クレデンシャル解決
- `aws-sdk-organizations` 1.x（`rt-tokio`） — Organizations API
- `aws-sdk-sts` 1.x（`rt-tokio`） — STS API（AssumeRole / GetCallerIdentity）
- `aws-sdk-cloudwatchlogs` 1.x（`rt-tokio`） — CloudWatch Logs API
- `tokio` 1.x（`rt-multi-thread`, `macros`） — 非同期ランタイム（`#[tokio::main]`）
- `clap` 4.x（`derive`） — CLI引数解析
- `inquire` 0.7 — 対話式マルチセレクト・確認・単一選択・数値入力プロンプト（`main.rs`の`InquireMultiSelectPrompt`/`InquireConfirmPrompt`等でのみ使用、ライブラリcrate本体はトレイト経由で抽象化しdirect依存しない）
- `serde` 1.x（`derive`） / `serde_json` 1.x — シリアライズ（監査ログJSON Lines、JSON出力フォーマット）
- `comfy-table` 7.x — table出力フォーマット
- `thiserror` 1.x — エラー型定義（`src/error.rs`）
- `secrecy` 0.8（`serde`機能） — 一時クレデンシャルのマスキング（`AccountCredentials`の`Debug`実装で`[REDACTED]`）
- `tracing` 0.1 / `tracing-subscriber` 0.3（`env-filter`） — ログ出力（stderr、スキャン失敗の`warn!`等）
- `uuid` 1.x（`v4`） — 実行ごとの`run_id`生成
- `async-trait` 0.1 — 非同期トレイトの記述（全内部境界トレイトで使用）
- `chrono` 0.4（`serde`機能） — 監査ログのタイムスタンプ（RFC3339）
- `tempfile` 3（dev-dependency） — テスト用一時ディレクトリ/ファイル

### Test Coverage
- **Test Directories**: `tests/`（統合テスト2ファイル: `scan_select_execute.rs`, `audit_log_format.rs`）。加えて全12ソースモジュールが`#[cfg(test)] mod tests`をファイル内に持つ（ユニットテスト）
- **Test Frameworks**: 標準の`#[test]` / `#[tokio::test]`（`tokio`の`macros`機能経由）。モックは全て手書きスタブ/フェイク（外部モックライブラリ不使用、各コンポーネント境界トレイトを直接実装）
- **Coverage Config**: 存在する。CIの`coverage`ジョブが`cargo llvm-cov --lib --locked --fail-under-lines 80 --summary-only`を実行（`--lib`のみ、統合テストはカバレッジ計測対象外）。README.mdにもローカル実行コマンドの記載あり（`cargo llvm-cov --lib --summary-only`、要`rustup component add llvm-tools-preview`）
- 実AWSアカウントに接触するテストは本リポジトリには存在しない（全てモック境界のみ）。README.mdとteam.mdの記述（`#[ignore]`分離）は将来の実アカウント接触テスト追加時の方針であり、現状のテストコード自体は全てモック完結

### Code Quality Indicators
- **Linting**: `clippy`（`.circleci/config.yml`の`clippy`ジョブで`cargo clippy --all-targets --locked -- -D warnings`）。フォーマットは`fmt`ジョブで`cargo fmt --check`。`src/main.rs`・`src/lib.rs`双方に`#![forbid(unsafe_code)]`と`#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`（`lib.rs`側は`#[cfg_attr(not(test), deny(...))]`でテストコードのみ除外）
- **CI/CD**: `.circleci/config.yml`（CircleCI 2.1）。ジョブ: `fmt`, `clippy`, `test`, `coverage`, `audit`（`cargo audit`、週次のみ）, `deny`（`cargo deny check`、通常は`bans licenses sources`、週次に`advisories`追加）, `release-linux` / `release-macos`（タグ駆動、`v\d+\.\d+\.\d+`パターンでクロスプラットフォームバイナリビルド）, `publish-github-release`。ワークフロー: `build-and-test`（push/PR時）, `weekly-security`（スケジュール実行時のみ）, `release`（タグpush時のみ）
- **Documentation**: README.md（日本語、使い方・スコープ制約・開発コマンド・AI-DLC説明を包括）。各ソースファイル冒頭にモジュールdocコメント（`//!`）でコンポーネントの責務とMandated/Forbidden規則の参照が付記されている。関数レベルのdocコメントも要所（特に安全パス関連: identity検証・監査ログ・実行エンジン）に充実

### Technical Debt Signals
- CLIは現状フラグのみのフラット構造（`--scan-only`/`--execute`の相互排他フラグ）であり、本intentが目指すサブコマンド化（`scan`/`clean`/`audit`）は未着手。`Cli`構造体・`main.rs`のフロー分岐（`cli.scan_only` → `stdin().is_terminal()`判定 → 選択/確認/実行）が今回の再構成の直接の対象になる
- `main.rs`の`action_kind_from_prompt`（delete/set-retention選択、`inquire::Select`/`inquire::CustomType`）はテストされていない（`inquire`への直接依存のためユニットテスト対象外。既存の設計判断としては妥当だが、将来のサブコマンド化でCLI引数から直接`ActionKind`を指定する経路を設ける場合はテスト可能な形に切り出す余地がある）
- `audit-log-path`に無効化オプションを設けない設計（project.md Mandated）のため、`audit`サブコマンドを新設する場合は既存の`AuditLogger`/`AuditWrite`をそのまま読み取り専用ビューアとして再利用できるか（現状は追記専用APIのみで読み取りAPIが存在しない）が設計上の空白点
- それ以外に深刻な技術的負債の兆候（TODO残置、デッドコード、重複ロジック等）は確認されなかった。エラーハンドリングは`thiserror`ベースで一貫し、`unwrap`/`expect`/`panic`は本番コードパスから`#![deny(...)]`で構造的に排除されている

## Handoff Summary
- **Intent-relevant finding**: サブコマンド化の対象は`src/cli.rs`の`Cli`構造体（`--regions`必須、`--execute`/`--scan-only`が`conflicts_with`の相互排他フラグ）と`src/main.rs`の`main()`内フロー分岐（`cli.scan_only`判定 → 非TTY時のフォールバック `!std::io::stdin().is_terminal()` → 対話式選択・確認・実行）。この2箇所が現在の「フラグのみ」構造の核であり、`scan`/`clean`サブコマンドへの分割時にこの判定ロジック（`src/main.rs:554-560`、`src/cli.rs:52`の`conflicts_with = "execute"`）をどう再配置するかが設計上の主眼になる。`audit`ビューアサブコマンドは、現状追記専用の`AuditLogger`（`src/audit.rs`）に読み取りAPIが存在しないため新規追加が必要。
- **Risks / follow-up**: (1) `--audit-log-path`は「無効化オプションを設けない」というproject.md Mandated制約があり、将来`audit`サブコマンドを新設する際もこの監査ログ自体を無効化する経路を作ってはならない。(2) `--scan-only`と`--execute`は現在`clap`の`conflicts_with`で相互排他だが、サブコマンド化後は「`scan`サブコマンド=スキャンのみ」「`clean`サブコマンド=選択・削除フロー、`--execute`で実行/dry-run切替」という自然な分割になるはずで、既存の`scan_fully_failed`終了コード判定（README.md記載のCI向け契約）とdry-run既定（`--execute`未指定時は`delete-log-group`/`put-retention-policy`を呼ばない、project.md Forbidden）を両サブコマンドで壊さないよう回帰テスト（`src/cli.rs`・`src/execution.rs`の既存テスト群）を維持すること。(3) `config`サブコマンドは本intentのスコープ外と明示されており、`--role-name`等の設定系フラグは既存フラグのまま各サブコマンドに残す前提で設計する必要がある。
