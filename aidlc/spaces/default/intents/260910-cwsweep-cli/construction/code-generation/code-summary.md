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

## Build-and-Testループバックへの対応（R-01, Step 19）

Build and Testステージの完了判定でNot MetとなったR-01（破壊的操作パス・安全パス100%パスカバレッジ未達成）について、人間が明示的に「Retry with fix」を選択し、code-generationへループバックした。

- `src/execution.rs`: LCOV基準（実行回数0の行）で未到達だった4行を解消するテストを2件追加（`audit_log_write_failure_aborts_the_operation_for_set_retention`、`execute_true_with_failing_retention_api_call_returns_api_error_after_audit_write`）。既存の`AlwaysFailingApiClient`をテスト間で共有する形にリファクタリングし、`delete_log_group`/`put_retention_policy`双方のアームを確実に実行させた。LCOV基準の実行回数0行は0件になった（`cargo llvm-cov --lib --lcov`で確認）。
- `src/identity.rs`: LCOV基準の実行回数0行は元々0件（`llvm-cov`summary上の「missed」は`assert!`/`matches!`マクロ展開由来の失敗分岐カウンタであり、テストが成功する限り構造的に到達不能）。到達不能である理由をコード内コメントで明記した（コード変更なし）。
- `src/audit.rs`: LCOV基準で実行回数0の行が17行残る。内訳は(1) `serde_json::to_string(entry)`の失敗分岐（`AuditEntry`が`String`/`bool`/`Option<String>`/フィールドなし列挙型のみで構成されシリアライズ失敗要素を含まないため構造的に到達不能）、(2) テスト用フェイク書き込み先の`flush()`実装（`AuditLogger::append`が`flush()`を呼ばない設計のため到達不能）、(3) 2種のフェイクの`sync()`（`write`が先にエラーを返す設計上、到達不能）。いずれも到達不能である理由をコード内コメントで明記した（コード変更なし。無理な失敗誘発テストによる数値の偽装は行っていない）。

修正後の`cargo llvm-cov --lib --summary-only`実測: `execution.rs` 95.68%（LCOV実行回数0行: 0件）、`identity.rs` 99.26%（LCOV実行回数0行: 0件）、`audit.rs` 90.77%（変化なし。到達不能箇所のみ残存）。`cargo test`は99 unit（既存97+新規2）+ 4 integration すべてpass。`cargo fmt --check`/`cargo clippy --all-targets -- -D warnings`合格。

**引き続き残る既知のギャップ**: `cargo llvm-cov`のregion/summary集計上は3モジュールとも100%に到達していない。ただし実質的な指標（LCOV基準の行実行回数）で見ると、`execution.rs`/`identity.rs`は既に実行回数0の行がゼロであり、残る差分はマクロ展開由来の測定上のアーティファクトである。`audit.rs`の17行は構造的に到達不能なコードパスであり、テストで到達させることは設計上不可能（無理に到達可能な形に書き換えることは、フェイクの「1回の失敗だけを決定的に再現する」という設計意図やproduction側のI/O設計を損なう）。この残存ギャップは`test-results.md`のTarget Verification Matrixで正直に記録する。

## レビュー指摘への対応（Request Changes → Revision 1）

advisoryアーキテクチャレビュー（Iteration 1, Verdict: READY, Major 1件/Minor 3件）を受け、人間の明示的な選択によりR-02/R-03/R-04を修正した（R-01は今回の対応対象から明示的に除外され、未解決のまま残る）。

- **R-02（STSクライアントのリージョン固定）**: コード変更は行わず、v1はAWS標準パーティション（商用リージョン）のみを対象とする設計判断をREADME.mdと`sts_client_for`のコード内コメントに明記した。
- **R-03（全滅時の終了コード）**: `CliApp::scan_fully_failed`を追加し、対象1件以上かつ全件失敗の場合のみ非ゼロ終了コードを返すようにした（部分成功時・対象0件時は0終了）。ユニットテスト4件追加。
- **R-04（監査ログ出力先固定）**: `--audit-log-path`引数を追加（既定値は後方互換の`cwsweep-audit.jsonl`）。無効化オプションは追加していない（project.md Mandated要件を維持）。ユニットテスト2件追加。
- **R-01（破壊的操作パス100%カバレッジ未達成）**: 人間が明示的に「未解決のまま残す」と選択したため、今回は対応していない。

修正後のテスト結果: `cargo test` 97 unit（既存91+新規6）+ 4 integration すべてpass。`cargo fmt --check`/`cargo clippy --all-targets -- -D warnings`: 合格。

## 未達成の品質目標（正直な報告）

team.mdが要求する「100%パスカバレッジ」を厳密には満たせていない箇所がある:
- `audit.rs`（90.77%）: `serde_json::to_string`失敗分岐は`AuditEntry`が単純な型のみで構成されるため実質的に到達不能であり、意図的にテストしていない。
- `execution.rs`（93.66%）/`identity.rs`（99.22%）: `async_trait`マクロ展開由来の境界コードや、一部テストで使用しないモックの未使用アームが数行未到達。

実行時ロジック自体（dry-run既定・削除直前の二重Identity検証・監査ログ失敗時の操作中断・対話式マルチセレクトの初期状態全OFF）は、いずれも境界値を含めて明示的にRed→Greenテスト済みである。

## Assumptions & Open Questions

None.
