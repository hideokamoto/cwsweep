# Code Generation Plan — cwsweep (zero-Unit / stage-level)

対象: Units Generationがスキップされているため、本ステージはCLI全体を単一イテレーションで実装する（Bolt/ウォーキングスケルトン/per-Unitレシートは適用しない）。

## 参照した設計成果物

- Domain Design: `aidlc/spaces/default/intents/260910-cwsweep-cli/inception/domain-design/components.md`（12コンポーネント）, `decisions.md`（ADR-001〜004）
- NFR Requirements: `aidlc/spaces/default/intents/260910-cwsweep-cli/construction/nfr-requirements/`配下 各`*-requirements.md`, `tech-stack-decisions.md`
- NFR Design: `aidlc/spaces/default/intents/260910-cwsweep-cli/construction/nfr-design/`配下 `security-design.md`, `performance-design.md`, `scalability-design.md`, `reliability-design.md`, `observability-design.md`, `logical-components.md`
- Ideation: `aidlc/spaces/default/intents/260910-cwsweep-cli/ideation/scope-definition/scope-document.md`（M1-M5がMVPスコープ、M6=`--execute`実装は本コード生成に含めるが、完了承認ゲート自体をteam.mdのM6ゲートとして扱う）, `intent-backlog.md`（Proto-Unit ID）
- Team/Project Memory: `aidlc/spaces/default/memory/team.md`, `project.md`（Forbidden/Mandated条項、テスト方針）

## Testing Contract

```json
{
  "version": 1,
  "methodology": "custom",
  "source": "team",
  "ordering": "基本方針は各層（Identity検証層、ページネーション集計層、対話式UI層、アクション実行層）",
  "scope": "cwsweep-orgwide-logs-cleanup",
  "test_strategy": "standard",
  "project_type": "greenfield",
  "applicable_notes": [
    {
      "layer": "org",
      "text": "We treat tests as a first-class deliverable in every Bolt. The specific\nmethodology (TDD, BDD, ATDD, or classic test-after) is affirmed at\npractices-discovery and recorded in `team.md` under this heading with explicit\n`Methodology` and `Ordering` fields; Code Generation resolves those fields\nindependently from coverage, tooling, and scope notes.\n\nWhen no posture has been affirmed, our default per scope is:\n- **Methodology**: test-after\n- **Ordering**: implement each applicable testable layer, then write and run\n  that layer's tests.\n- `mvp`, `enterprise`, `feature`, `infra`, `classic` add an 80% line-coverage\n  floor and CI execution before merge.\n- `bugfix`, `security-patch` add a targeted regression for the specific\n  bug/vulnerability and require the existing suite to remain green.\n- `express` uses the Minimal strategy: requirement-driven unit tests (one per\n  requirement, with a happy-path floor per component); existing tests remain\n  green.\n- `poc`, `refactor`, `workshop` add no extra new-test floor and require the\n  existing suite to remain green.\n\nThe active `Test Strategy` still applies in every scope and determines test\nvolume/types. Scope floors are additive; they never reduce or replace the\nselected strategy.\n\nBuild and Test verifies defined coverage floors and affirmed quality targets;\nthey may not be weakened to make a step pass.\n\nAffirm a stricter posture in `team.md` if the team commits to one."
    },
    {
      "layer": "team",
      "text": "- **Methodology**: custom\n- **Ordering**: 基本方針は各層（Identity検証層、ページネーション集計層、対話式UI層、アクション実行層）\n  を実装した直後にその層の単体テストを作成・実行する test-after 方式とするが、破壊的操作に直結する\n  安全パス（Identity検証・削除直前の二重確認・監査ログ出力・dry-run既定の挙動）だけは、実装に先立って\n  期待仕様をテストとして書いてから実装する TDD 的な進め方とする。\n- カバレッジ基準:\n  - 通常コードは行カバレッジ 80% 以上を floor とする。\n  - 破壊的操作（`delete-log-group` / `put-retention-policy`）に関わるコードパス、および\n    Identity検証失敗系・`--execute` 未指定時の抑止・対話式マルチセレクトの初期状態は、\n    80% floor とは別に **100% パスカバレッジ＋境界値テスト**（不一致ID、リージョン跨ぎ、\n    ページネーション途中エラー等）を要求する。\n  - カバレッジ計測には `cargo llvm-cov` を使用する。\n- テストの種類:\n  - AWS SDK 境界（AssumeRole・各種 API 呼び出し）はモックを用いたユニットテストに加え、\n    境界を跨ぐ統合的なシナリオテスト（スキャン → 選択 → 削除の一連の流れを複数モジュールを\n    通して検証するテスト）を別途用意する。\n  - 監査ログの出力内容（フォーマット・必須フィールド：対象アカウントID・リージョン・\n    ログループ名・実行時刻・成功/失敗）自体を検証する専用テストを用意する。監査ログ書き込み\n    失敗時に該当操作が中断されることを検証するテストも含める。\n  - dry-run 既定・全選択禁止などのデフォルト挙動は、フラグを一切指定しない呼び出しを\n    固定テストケースとして必須化し、将来の意図しないデフォルト値反転（退行）を検出できるようにする。\n- CI では実 AWS アカウントに接触するテストとモックで完結するテストを明示的に分離し\n  （例: `#[ignore]` + 専用ジョブ）、通常の CI 実行では前者を自動実行しない。\n- 既存スイートは常にグリーンを維持する。"
    }
  ],
  "obligations": {
    "strategy": "standard",
    "strategy_volume": [
      "Five to eight tests per component.",
      "Unit tests plus integration tests for key boundaries.",
      "Add E2E, performance, or security tests when requirements demand them."
    ],
    "scope_floor": [
      "Keep the existing test suite green.",
      "This scope adds no extra new-test floor beyond the selected test strategy."
    ],
    "combination_rule": "Apply every selected-strategy obligation and every scope-floor obligation; neither replaces the other, and a targeted scope regression may add the narrowest necessary test type beyond the strategy default."
  },
  "plan_profile": {
    "methodology": "custom",
    "runner_step": "Bootstrap the minimal test runner/configuration and record the exact unit-scoped command.",
    "runner_ready_before_first_test": true,
    "testable_layers": [
      "Data model / database behavior",
      "Repository / data access",
      "Business logic",
      "API / endpoint",
      "Frontend behavior"
    ],
    "steps": [
      "Project structure and production configuration skeleton.",
      "Bootstrap the minimal test runner/configuration and record the exact unit-scoped command.",
      "Custom ordering - 基本方針は各層（Identity検証層、ページネーション集計層、対話式UI層、アクション実行層）",
      "Implementation and tests - preserve that exact ordering; do not convert it to layer-local TDD.",
      "Environment/build configuration.",
      "Documentation and traceability."
    ]
  },
  "input_sha256": "sha256:67cfab2348f4f84069f1d24abf7c2dc10e1e2b8a04de7173a35db9122c05a90b",
  "contract_sha256": "sha256:f511f45111afb4cb95463a57fe1267d38ec7c5219e1f3a753d50475438f8ee51"
}
```

## Story-to-Code トレーサビリティ

| Plan Step | 実装対象コンポーネント | 対応 Proto-Unit / FR / NFR |
|---|---|---|
| Step 3-4 | CliApp, OrgDiscovery | PU-01, PU-02（アカウント列挙、`--regions`必須） |
| Step 5-6 | CredentialProvider, IdentityVerifier | PU-03, PU-04, NFR2.1, NFR2.3, NFR2.5 |
| Step 7-8 | LogGroupScanner, ScanAggregator | PU-05, PU-06, NFR1.2, NFR1.3, NFR3.3 |
| Step 9-10 | OutputFormatter, InteractiveSelector | PU-07, PU-08, NFR2.9 |
| Step 11-13 | ActionPlanner, ConfirmationPresenter | PU-09, PU-10 |
| Step 14-16 | ExecutionEngine, AuditLogger | PU-11, NFR2.4, NFR4.2, NFR4.3 |
| Step 17 | 統合シナリオテスト | 全PU横断 |
| Step 18 | CI分離・ドキュメント・traceability | NFR2.6, NFR2.7 |

## Plan Steps

### Step 1: プロジェクト構造・本番設定スケルトン

- [x] `Cargo.toml` を新規作成する。依存: `aws-config`, `aws-sdk-organizations`, `aws-sdk-sts`, `aws-sdk-cloudwatchlogs`, `tokio`（`rt-multi-thread`, `macros`）, `clap`（`derive`）, `inquire`, `serde`, `serde_json`, `comfy-table`, `thiserror`, `secrecy`, `tracing`, `tracing-subscriber`, `uuid`（v4）。開発依存: `cargo-llvm-cov`（CIで利用、Cargo.tomlには不要）, テスト用モッククレート（`aws-smithy-mocks-experimental` または手書きトレイト差し替え）。
- [x] クレートルート `src/main.rs` に `#![forbid(unsafe_code)]` を宣言する。
- [x] 本番コードパスに `#[deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` を適用する（テストモジュールは対象外）。
- [x] `src/lib.rs` を作成し、下記モジュール構成の空スケルトンを用意する: `cli`, `org_discovery`, `credentials`, `identity`, `scanner`, `aggregator`, `output`, `selector`, `planner`, `confirmation`, `execution`, `audit`, `error`。
- [x] Domain Designの12コンポーネントを1モジュール1コンポーネントとして対応させる（`CliApp`→`cli`, `OrgDiscovery`→`org_discovery`, `CredentialProvider`→`credentials`, `IdentityVerifier`→`identity`, `LogGroupScanner`→`scanner`, `ScanAggregator`→`aggregator`, `OutputFormatter`→`output`, `InteractiveSelector`→`selector`, `ActionPlanner`→`planner`, `ConfirmationPresenter`→`confirmation`, `ExecutionEngine`→`execution`, `AuditLogger`→`audit`）。

### Step 2: テストランナー/設定のブートストラップ（最初のテストの前に完了させる）

- [x] `cargo test` がユニットテストの既定ランナーであることを確認し、`tests/`（統合テスト）ディレクトリを作成する。
- [x] `cargo llvm-cov` のローカル実行手順を `unit-test-instructions.md` に記録する。
- [x] スコープ内テストを実行できる最小コマンド（例: `cargo test --lib`）が通ることを確認してから最初のRed/Green/テストファーストステップへ進む。

### Step 3〜16: Custom ordering（各層をtest-afterで実装するが、安全パスはTDD）

> Testing Contractの`ordering`を厳密に保持する。安全パス（Identity検証・削除直前の二重確認・監査ログ出力・dry-run既定）はStep内で「テスト先行（Red→Green）」の小節を明示し、それ以外の通常ロジックは「実装→その層のテスト作成・実行」の順（test-after）で進める。

#### Identity検証層

- [x] Step 3: `error` モジュールに `thiserror` ベースのエラー型（`IdentityMismatchError`, `AssumeRoleError`, `AuditWriteError`, `PaginationError` 等）を定義する。
- [x] Step 4: `credentials` モジュールに `AccountCredentials`（`secret_access_key`/`session_token`を`secrecy::SecretString`でラップし、`Debug`を手動実装して`[REDACTED]`マスク）と、管理アカウントは現在の認証情報をそのまま使用しメンバーアカウントのみ`AssumeRole`（既定ロール名`OrganizationAccountAccessRole`、引数で上書き可）する `CredentialProvider` を実装する。
- [x] Step 5（テスト先行・TDD）: `identity` モジュールの `IdentityVerifier::verify_identity` について、まず「アカウントID不一致時に即座に`IdentityMismatchError`を返し処理を継続しない」「削除直前の二重目の検証が独立して呼び出される」の期待仕様をテストとして書き（Red）、最小実装でGreenにする。境界値: 不一致ID、リージョン跨ぎでの検証、`sts:get-caller-identity`呼び出し失敗時の伝播を含める。
- [x] Step 6: `credentials`/`identity`モジュールの通常系（正常な認証情報解決、AssumeRoleの成功パス）を実装後にユニットテストを作成・実行する（test-after）。

#### ページネーション集計層

- [x] Step 7: `scanner` モジュールに `LogGroupScanner` を実装し、`describe-log-groups`のページネーションを最後まで辿ってから集計に渡す（1ページのみでの確定を許容しない）。リージョンは`CliApp`の引数解析結果（`--regions`必須、自動列挙フォールバックなし）を受け取る。
- [x] Step 8: `aggregator` モジュールに `ScanAggregator`（`LogGroupRecord`エンティティ、`Vec<LogGroupRecord>`保持、複数アカウント×リージョンの集計）を実装する。両モジュールの実装後にその層のユニットテストを作成・実行する（test-after）。境界値としてページネーション途中エラーのケースを含める。

#### 対話式UI層

- [x] Step 9: `output` モジュールに `OutputFormatter`（`comfy-table`によるテーブル表示、合計バイト数集計）を実装する。
- [x] Step 10（テスト先行・TDD、初期状態の抑止仕様のみ）: `selector` モジュールの `InteractiveSelector`（`inquire`のマルチセレクト）について、まず「初期状態は常に全チェックOFF（all-selectedにしない）」という期待仕様をテストとして書き（Red）、最小実装でGreenにする。それ以外の選択操作ロジックは実装後にtest-afterでテストする。

#### アクション実行層

- [x] Step 11: `planner` モジュールに `ActionPlanner`（`PlannedAction`エンティティ、選択結果からの計画生成）を実装する。
- [x] Step 12: `confirmation` モジュールに `ConfirmationPresenter`（対象アカウントID・リージョン・ログループ名一覧・合計バイト数を再掲する確認画面。`PlannedAction`を直接ミューテートしない）を実装する。
- [x] Step 13: Planner/Confirmationの実装後にユニットテストを作成・実行する（test-after）。
- [x] Step 14（テスト先行・TDD）: `execution` モジュールの `ExecutionEngine` について、まず「`--execute`フラグが明示的に渡されていない限り`delete-log-group`/`put-retention-policy`のAPI呼び出しコード自体を通過しない」という dry-run 既定の期待仕様と、「削除・retention変更の直前に独立した二重目のIdentity検証を再実行する」という期待仕様をテストとして書き（Red）、最小実装でGreenにする。フラグ未指定の固定テストケースを含め、将来のデフォルト値反転を検出できるようにする。
- [x] Step 15（テスト先行・TDD）: `audit` モジュールの `AuditLogger::append`（JSON Linesを1エントリずつ`fsync`付きで追記、対象アカウントID・リージョン・ログループ名・実行時刻・成功/失敗・`run_id`相関を含む）について、まず「書き込み失敗（`std::io::Error`）時に`ExecutionEngine`へ`Result`で伝播し当該操作を中断する」という期待仕様をテストとして書き（Red）、最小実装でGreenにする。無効化オプションを設けないことも固定テストケース化する。
- [x] Step 16: `ExecutionEngine`/`AuditLogger`の通常系（成功パスでの実行・監査ログ出力）を実装後にユニットテストを作成・実行する（test-after）。100%パスカバレッジ＋境界値テスト（不一致ID、リージョン跨ぎ、ページネーション途中エラー等）は破壊的操作パスと安全パス全体に適用する。

### Step 17: 統合シナリオテスト

- [x] `tests/scan_select_execute.rs` に、モックAWS境界を用いた「スキャン → 選択 → 削除（dry-run）」の一連の流れを複数モジュールを通して検証する統合テストを作成する。
- [x] `tests/audit_log_format.rs` に監査ログの出力内容（フォーマット・必須フィールド）を検証する専用テストを作成する。
- [x] 実AWSアカウントに接触するテスト（存在する場合）は`#[ignore]`でCIの通常実行から分離する。

### Step 18: 環境/ビルド設定・ドキュメント・トレーサビリティ

- [x] `.cargo/config.toml`（必要な場合）とワークスペース設定を整える。
- [x] `rustfmt.toml`（既定設定）、`clippy`実行手順、`cargo audit`/`cargo deny`の実行手順を`README.md`（または`docs/`）に記載する。
- [x] `README.md`にCLI使用方法（`--regions`必須、`--execute`既定OFF等）を記載する。
- [x] `code-summary.md`, `source-manifest.json`, `traceability.json`を作成する（Step 5, 6の成果物）。

### Step 19: Build-and-Test loop-back — 破壊的操作パス100%カバレッジの追加テスト（R-01対応）

- [x] Build and Testステージで検出されたNot Met項目（R-01: `execution.rs`/`audit.rs`/`identity.rs`の破壊的操作・安全パス100%パスカバレッジ未達成）に対し、到達困難な分岐（`async_trait`マクロ展開由来の境界コード、`serde_json::to_string`失敗分岐、未使用テストアーム等）を狙った追加ユニットテストを作成する。
- [x] 追加テスト後、`cargo llvm-cov --lib --summary-only`で該当3モジュールが100%（または到達不能であることが構造的に説明できる場合はその理由をコード内コメントで明記）に近づいたことを確認する。
- [x] 既存の97件のユニットテスト+4件の統合テストを壊さないこと。

## Assumptions & Open Questions

None.
