# Code Generation Plan — Unit: audit-reader

## Testing Contract

```json
{
  "version": 1,
  "methodology": "custom",
  "source": "team",
  "ordering": "基本方針は各層（Identity検証層、ページネーション集計層、対話式UI層、アクション実行層）",
  "scope": "classic",
  "test_strategy": "standard",
  "project_type": "brownfield",
  "applicable_notes": [
    {
      "layer": "org",
      "text": "We treat tests as a first-class deliverable in every Bolt. The specific\nmethodology (TDD, BDD, ATDD, or classic test-after) is affirmed at\npractices-discovery and recorded in `team.md` under this heading with explicit\n`Methodology` and `Ordering` fields; Code Generation resolves those fields\nindependently from coverage, tooling, and scope notes.\n\nWhen no posture has been affirmed, our default per scope is:\n- **Methodology**: test-after\n- **Ordering**: implement each applicable testable layer, then write and run\n  that layer's tests.\n- `mvp`, `enterprise`, `feature`, `infra`, `classic` add an 80% line-coverage\n  floor and CI execution before merge.\n- `bugfix`, `security-patch` add a targeted regression for the specific\n  bug/vulnerability and require the existing suite to remain green.\n- `express` uses the Minimal strategy: requirement-driven unit tests (one per\n  requirement, with a happy-path floor per component); existing tests remain\n  green.\n- `poc`, `refactor`, `workshop` add no extra new-test floor and require the\n  existing suite to remain green.\n\nThe active `Test Strategy` still applies in every scope and determines test\nvolume/types. Scope floors are additive; they never reduce or replace the\nselected strategy.\n\nBuild and Test verifies defined coverage floors and affirmed quality targets;\nthey may not be weakened to make a step pass.\n\nAffirm a stricter posture in `team.md` if the team commits to one."
    },
    {
      "layer": "team",
      "text": "- **Methodology**: custom\n- **Ordering**: 基本方針は各層（Identity検証層、ページネーション集計層、対話式UI層、アクション実行層）\n  を実装した直後にその層の単体テストを作成・実行する test-after 方式とするが、破壊的操作に直結する\n  安全パス（Identity検証・削除直前の二重確認・監査ログ出力・dry-run既定の挙動）だけは、実装に先立って\n  期待仕様をテストとして書いてから実装する TDD 的な進め方とする。\n- カバレッジ基準:\n  - 通常コードは行カバレッジ 80% 以上を floor とする。\n  - 破壊的操作（`delete-log-group` / `put-retention-policy`）に関わるコードパス、および\n    Identity検証失敗系・`--execute` 未指定時の抑止・対話式マルチセレクトの初期状態は、\n    80% floor とは別に **100% パスカバレッジ＋境界値テスト**（不一致ID、リージョン跨ぎ、\n    ページネーション途中エラー等）を要求する。\n  - カバレッジ計測には `cargo llvm-cov` を使用する。\n- テストの種類:\n  - AWS SDK 境界（AssumeRole・各種 API 呼び出し）はモックを用いたユニットテストに加え、\n    境界を跨ぐ統合的なシナリオテスト（スキャン → 選択 → 削除の一連の流れを複数モジュールを\n    通して検証するテスト）を別途用意する。\n  - 監査ログの出力内容（フォーマット・必須フィールド：対象アカウントID・リージョン・\n    ログループ名・実行時刻・成功/失敗）自体を検証する専用テストを用意する。監査ログ書き込み\n    失敗時に該当操作が中断されることを検証するテストも含める。\n  - dry-run 既定・全選択禁止などのデフォルト挙動は、フラグを一切指定しない呼び出しを\n    固定テストケースとして必須化し、将来の意図しないデフォルト値反転（退行）を検出できるようにする。\n- CI では実 AWS アカウントに接触するテストとモックで完結するテストを明示的に分離し\n  （例: `#[ignore]` + 専用ジョブ）、通常の CI 実行では前者を自動実行しない。\n- 既存スイートは常にグリーンを維持する。\n- 【本 intent での確定事項（Q3・カバレッジ計測スコープ）】CI の `coverage` ジョブは\n  `cargo llvm-cov --lib`（バイナリクレート `src/main.rs` を計測対象外とする）で構成されている。\n  この計測範囲を今回は変更せず、代わりに新しい `Commands`（`scan`/`clean`/`audit`）への\n  ディスパッチロジックそのものを `lib` 側（`cli.rs` 等、`cargo llvm-cov --lib` の対象範囲）に\n  実装し、`main.rs` は薄い呼び出し（アダプタ配線）のみに留める、という方針を人間が確定した。\n  これにより、新設のディスパッチ分岐が 80%/100% カバレッジ floor の計測対象から漏れることを防ぐ。\n  この制約は Code Style セクションにも構造上の要件として明記する。\n- 【本 intent での確定事項（Q4・`audit` の構造的読み取り専用）】`audit` サブコマンドのハンドラは、\n  `delete-log-group` / `put-retention-policy` を実行しうる型（`ExecutionEngine`・書き込み系\n  `AuditWrite` 等）への依存を一切持たない構造とすることを、`project.md` の `## Forbidden`\n  （破ったら即アウトのハード制約）として明文化することを人間が確定した（`discovered-rules.md`\n  参照）。このため、`audit` ハンドラの依存注入グラフに削除・retention変更・監査ログ書き込み系の\n  型が含まれないことを検証する構造的な回帰テスト（コンパイル成立を前提とした型/モジュール境界の\n  テスト、またはアーキテクチャレベルの統合テスト）を追加テスト観点とする。具体的な検証手段\n  （静的解析／依存方向の統合テストのいずれで担保するか）は Code Generation / Build and Test 段階で\n  設計する。\n- 【本 intent での確定事項（Q7・`audit` の異常系挙動）】監査ログ（JSON Lines）の読み取り中に\n  不正フォーマット・欠落フィールドを持つ壊れた行に遭遇した場合、`audit` サブコマンドは\n  **その行をスキップし警告を出力した上で、残りの正常な行の表示を継続する**（処理全体を中断\n  しない）ことを人間が確定した。これに伴い、少なくとも1行の不正フォーマット行を含む監査ログ\n  フィクスチャを用いて、(a) 不正行がスキップされ警告が出力されること、(b) 不正行の前後にある\n  正常なエントリが引き続き表示されることの両方を検証するテストを必須の追加テスト観点とする。\n- 【品質担当エージェントの指摘による訂正】既存の統合テスト `tests/scan_select_execute.rs` は\n  `cwsweep::cli::Cli` や `clap` のパース処理には一切触れない、`OrgDiscovery`・\n  `CredentialProvider`・`LogGroupScanner`・`InteractiveSelector`・`ActionPlanner`・\n  `ConfirmationPresenter`・`ExecutionEngine` を直接手組みで配線して検証する CLI 非依存の\n  ドメイン層パイプラインテストである。したがって「サブコマンド化に伴い `tests/scan_select_execute.rs`\n  の更新が必要になる」という記述は不正確であり撤回する。真のギャップは、`Cli`/`Commands` の\n  パースと `main()` のサブコマンドディスパッチそのものを検証する統合テストが現状存在しないことで\n  あり、本 intent では**新規の CLI 層統合テスト（`Cli`/`Commands` のパース検証、および\n  サブコマンド一切指定なしの場合の振る舞いを含む）を新設する**ことを追加テスト観点として明記する。\n  パース検証は `Cli::try_parse_from` を使ったインプロセスの単体テスト（`cli.rs` 内\n  `#[cfg(test)]`）を基本とし、`--lib` カバレッジにも算入される形にすることを推奨する。"
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
    "runner_step": "Verify the existing test runner/configuration and record the exact unit-scoped command.",
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
      "Verify the existing test runner/configuration and record the exact unit-scoped command.",
      "Custom ordering - 基本方針は各層（Identity検証層、ページネーション集計層、対話式UI層、アクション実行層）",
      "Implementation and tests - preserve that exact ordering; do not convert it to layer-local TDD.",
      "Environment/build configuration.",
      "Documentation and traceability."
    ]
  },
  "input_sha256": "sha256:ddab97d7bdea85eb537c0db5e92a62973d07d0d7775600c9633d1e401b7a67e7",
  "contract_sha256": "sha256:8f258c414ec8e4a1b8c6c5f852b7a7db0e7d7ef63a30c315c11fac2be247ddc8"
}
```

**適用メソドロジー**: `custom`（test-after。ただしteam.mdが名指しする
「Identity検証・削除直前の二重確認・監査ログ出力・dry-run既定の挙動」の
安全パスはTDD）。`audit-reader`はこの4つのいずれにも該当しない
読み取り専用コンポーネントであるため、全レイヤーtest-after方式を適用
する。

## 実装対象ファイル

既存の`src/audit.rs`に、書き込み側（`AuditWrite`/`AuditLogger`）とは
型レベルで分離した新規パブリックアイテムを追加する（domain-design
Q2で確定した「同一ファイル・型レベル分離」方針）。新規ファイルは
作成しない。

## Steps

- [ ] Step 1: テストランナー確認 — 既存の`cargo test`セットアップを確認し、
  本Unit専用のテストモジュール名を決定する（`#[cfg(test)] mod
  audit_reader_tests`、既存の`mod tests`とは別モジュール）。本Unit
  スコープの実行コマンドを確定する: `cargo test --lib
  audit_reader_tests:: --locked`。このコマンドを`unit-test-instructions.md`
  に記録し、実装前に空のモジュールに対して実行して動作することを確認する。

- [ ] Step 2: データモデル層（entities.md準拠） — `src/audit.rs`に以下を
  追加する:
  - `AuditReadEntry`（`Serialize, Deserialize, Debug, Clone, PartialEq`導出）:
    `run_id`・`timestamp`・`account_id`・`region`・`log_group_name`・
    `action_kind: ActionKind`（既存の`planner::ActionKind`を再利用）・
    `event: AuditEventKind`（既存の`audit::AuditEventKind`を再利用）・
    `success: bool`・`error_message: Option<String>`
  - `SkippedLine { line_number: usize, reason: String }`
  - `AuditReadOutcome { entries: Vec<AuditReadEntry>, skipped_lines: Vec<SkippedLine> }`
  - `AuditReadError`（`thiserror`導出、`Clone, PartialEq, Eq`を既存の
    `src/error.rs`の他のエラー型と同様に導出できるよう、`std::io::Error`を
    直接ラップせず`Io(String)`としてメッセージ文字列を保持する —
    advisory reviewのR-02指摘に対応した設計）
  - **テスト（Step 2の直後、test-after）**: `AuditReadEntry`の
    JSON往復シリアライズ/デシリアライズ、必須フィールド欠落時の
    デシリアライズ失敗（`serde_json::from_str`が`Err`を返すこと）

- [ ] Step 3: ビジネスロジック層（rules.md準拠） — `AuditRead`トレイト
  （`fn entries(&self) -> Result<AuditReadOutcome, AuditReadError>`）と
  `AuditReader { path: PathBuf }`を追加し、`impl AuditRead for
  AuditReader`を実装する:
  - ファイル不在 → 空の`AuditReadOutcome`を返す（BR2.1）
  - ファイル不在以外のI/Oエラー → `AuditReadError::Io`（BR2.2）
  - 行単位ストリーミング読み取り（`BufReader::lines()`）、
    `serde_json::from_str::<AuditReadEntry>`失敗時は`SkippedLine`へ
    記録し`eprintln!`で警告、処理継続（BR1.1/BR1.2）
  - `entries`/`skipped_lines`は出現順を保持（BR3.2）
  - ファイルは読み取り専用でオープンし、書き込み・作成フラグを
    一切使用しない（BR4.1/NFR3.3）
  - **テスト（Step 3の直後、test-after）**: 正常系（複数行が正しい順序で
    `entries`に入る）、ファイル不在（空の`AuditReadOutcome`、`Ok`）、
    不正フォーマット行が混在するフィクスチャ（`(a)`不正行がスキップされ
    警告が出ること、`(b)`不正行の前後の正常行が引き続き表示されること
    — team-practices.md Q7・NFR3.1の要求どおり）、ファイル不在以外の
    I/Oエラー（例: パスがディレクトリの場合）で`AuditReadError::Io`が
    返ること

- [ ] Step 4: 構造的分離の回帰テスト（NFR2.1/NFR2.2、security-design.md
  準拠） — `AuditReader`/`AuditRead`関連コードのソーステキストに
  `ExecutionEngine`・`AuditWrite`・`AuditLogger`の識別子が一切出現しない
  ことを検証するテキストベースの静的テストを追加する。advisory review
  （nfr-design R-01）の指摘どおり、この検査はエイリアスや間接参照までは
  検出できない**多層防御の一部**であり、単独で「コンパイル時点で
  到達不能」を証明するものではないことをコメントとテスト名で明記する
  （主たる保証は`AuditRead::entries(&self)`のシグネチャ自体が
  `ExecutionEngine`/`AuditWrite`への参照を要求しない設計であること、
  および型を共有しない設計であること）。

- [ ] Step 5: 環境・ビルド設定 — 新規クレート追加なし
  （tech-stack-decisions.md準拠）。`Cargo.toml`の変更は不要。

- [ ] Step 6: ドキュメント・トレーサビリティ — `code-summary.md`・
  `traceability.json`を作成し、advisory reviewの残存指摘（NFR3.2の
  カバレッジ強制ギャップ、NFR2.1/2.2の静的テストの限界）を
  「デビエーション」として明記する。

## Story-to-Code Traceability

（本intentはUser Storiesをスキップしたため、requirements.mdのFR/NFR IDを
用いる。unit-of-work-story-map.mdのFR→Unitマッピングに準拠。）

| Plan Step | 対応するFR/NFR/BR |
|---|---|
| Step 2 | FR4.1, FR4.2, BR1.2 |
| Step 3 | FR4.1, FR4.2, FR4.4, FR4.6, BR1.1, BR2.1, BR2.2, BR3.1, BR3.2, BR4.1, NFR3.1, NFR3.3 |
| Step 4 | FR4.5, NFR2.1, NFR2.2 |
| Step 6 | NFR3.2（ギャップの明記） |
