# Code Summary — cwsweep (zero-Unit / stage-level)

## 作成/変更ファイル

### アプリケーションコード（ワークスペースルート）

- `Cargo.toml`, `Cargo.lock` — 依存クレート定義（`aws-config`, `aws-sdk-organizations`, `aws-sdk-sts`, `aws-sdk-cloudwatchlogs`, `tokio`, `clap`, `inquire`, `serde`, `serde_json`, `comfy-table`, `thiserror`, `secrecy`, `tracing`, `tracing-subscriber`, `uuid`）
- `src/main.rs` — バイナリエントリポイント。`#![forbid(unsafe_code)]`, `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`
- `src/lib.rs` — クレートルート、モジュール宣言
- `src/error.rs` — `thiserror`ベースの共通エラー型（`IdentityMismatchError`, `AssumeRoleError`, `AuditWriteError`, `PaginationError`等）
- `src/cli.rs` — `CliApp`: `clap`によるサブコマンド/フラグ解析（`--regions`必須, `--execute`既定false, `--role-name`）、全体オーケストレーション
- `src/org_discovery.rs` — `OrgDiscovery`: `organizations:list-accounts`によるACTIVEアカウント列挙
- `src/credentials.rs` — `CredentialProvider`: `AccountCredentials`（`secrecy::SecretString`ラップ + 手動`Debug`マスク）、管理アカウントは現在の認証情報をそのまま使用、メンバーアカウントは`AssumeRole`
- `src/identity.rs` — `IdentityVerifier`: `sts:get-caller-identity`によるアカウントID検証、不一致時は即時失敗
- `src/scanner.rs` — `LogGroupScanner`: `describe-log-groups`の全ページ取得後に集計へ渡す
- `src/aggregator.rs` — `ScanAggregator`: `LogGroupRecord`保持、集計・ソート・合計バイト数計算
- `src/output.rs` — `OutputFormatter`: `comfy-table`によるtable出力、`serde_json`によるjson出力
- `src/selector.rs` — `InteractiveSelector`: `inquire`マルチセレクト、初期状態は常に全チェックOFF
- `src/planner.rs` — `ActionPlanner`: `PlannedAction`エンティティ、選択結果からの実行計画構築
- `src/confirmation.rs` — `ConfirmationPresenter`: 実行前確認画面、`PlannedAction`を直接ミューテートしない確認済みコピー生成
- `src/execution.rs` — `ExecutionEngine`: `--execute`既定false（dry-run）、実行直前の二重目Identity検証、`delete-log-group`/`put-retention-policy`呼び出し
- `src/audit.rs` — `AuditLogger`: JSON Linesをfsync付きで追記、書き込み失敗時は操作を中断、無効化オプションなし
- `tests/scan_select_execute.rs` — スキャン→選択→削除の統合シナリオテスト（モックのみ、dry-run既定と`--execute`時の両方を検証）
- `tests/audit_log_format.rs` — 監査ログのフォーマット・必須フィールド・書き込み失敗時の中断を検証する専用テスト
- `README.md` — 使用方法・開発コマンド（`cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo llvm-cov`, `cargo audit`, `cargo deny check`の実行手順）を追記
- `.gitignore` — `/target`, `cwsweep-audit.jsonl`を追加

### 計画成果物（record dir）

- `code-generation-plan.md` — Step 1〜18のチェックボックスを`[x]`化（本stageのフィンガープリント対象外の唯一の編集）

## 主要な実装判断

- Domain Designの12コンポーネントを1モジュール1コンポーネントとして対応させた（`cli`, `org_discovery`, `credentials`, `identity`, `scanner`, `aggregator`, `output`, `selector`, `planner`, `confirmation`, `execution`, `audit`）。
- 安全パス（Identity検証失敗系、対話式マルチセレクト初期状態、dry-run既定・削除直前二重検証、監査ログ書き込み失敗時の中断）はテスト先行（Red→Green）で実装し、それ以外は各層実装後にtest-after方式でテストした。
- `AuditLogger`の書き込み失敗を決定的にテストするため、書き込み先を`SyncWrite`トレイトで抽象化した（公開API `AuditLogger::open`/`append`のシグネチャは維持。security-design.mdのfsync付きJSON Lines追記という設計意図は保持）。
- AWS SDK境界（Organizations/STS/CloudWatch Logs）は各コンポーネントのトレイト境界を切り、テストでは手書きモックを注入した。

## テストカバレッジ・サマリ

- `cargo test --lib`: 91 passed / 0 failed
- `cargo test --test scan_select_execute --test audit_log_format`: 4 passed / 0 failed
- `cargo llvm-cov --lib --summary-only`: 全体行カバレッジ 97.09%（80%floor達成）
  - `identity.rs`: 99.22%, `selector.rs`: 100.00%, `execution.rs`: 93.66%, `audit.rs`: 90.77%, `aggregator.rs`/`confirmation.rs`/`credentials.rs`/`planner.rs`: 100.00%
- `cargo fmt --check`: 合格
- `cargo clippy --all-targets -- -D warnings`: 合格（警告0件）

## 計画からの逸脱

1. **`cargo audit` / `cargo deny check` は本ステージでは未実行。** ツールがこのセッションにインストールされておらず、CIゲートとしての実行手順のみ`README.md`に記載した。CI Pipelineステージで実際の実行・ゲート化を行う必要がある。
2. **`AuditLogger`の書き込み先抽象化。** 当初設計（`File`直書き）から、書き込み失敗（write失敗・fsync失敗・mutex poison）を決定的にテストするため`SyncWrite`トレイトによる抽象化に変更した。公開APIとfsync付きJSON Linesという設計意図自体は変更していない。

## 未達成の品質目標（正直な報告）

team.mdが要求する「100%パスカバレッジ」を厳密には満たせていない箇所がある:
- `audit.rs`（90.77%）: `serde_json::to_string`失敗分岐は`AuditEntry`が単純な型のみで構成されるため実質的に到達不能であり、意図的にテストしていない。
- `execution.rs`（93.66%）/`identity.rs`（99.22%）: `async_trait`マクロ展開由来の境界コードや、一部テストで使用しないモックの未使用アームが数行未到達。

実行時ロジック自体（dry-run既定・削除直前の二重Identity検証・監査ログ失敗時の操作中断・対話式マルチセレクトの初期状態全OFF）は、いずれも境界値を含めて明示的にRed→Greenテスト済みである。

## Assumptions & Open Questions

None.
