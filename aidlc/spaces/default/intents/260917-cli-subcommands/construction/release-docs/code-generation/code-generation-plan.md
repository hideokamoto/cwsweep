# Code Generation Plan — Unit: release-docs

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

| ファイル | 変更 |
|---|---|
| `README.md` | 「使い方」を scan / clean / audit の小節に再構成、「安全機構」「v0.1 からの移行」節を追加 |
| `CHANGELOG.md` | 新規作成（Keep a Changelog 形式、`[0.2.0]` に破壊的変更・Added・移行ガイド） |
| `Cargo.toml` / `Cargo.lock` | `version = "0.2.0"` と lock 同期 |

## Steps

1. README 再構成（D-SEC-1）: 安全な形のコマンド例を先頭に置き、既存の安全機構の箇条書きをサブコマンド名付きで維持。
2. 移行ガイド（D-SEC-2）: README と CHANGELOG に同一の対応表。
3. バージョン更新（D-SEC-3）: `Cargo.toml` → 0.2.0、`cargo update -p cwsweep --offline` で lock 同期。
4. 検証: `cargo build --locked`、README の旧フラグ例が移行表以外に残っていないこと。

## Story-to-Code Traceability

`traceability.json` 参照。
