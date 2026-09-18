# Evidence — practices-discovery（最終版: 260917-cli-subcommands）

## 参照した既存の affirm 済み記録

- `aidlc/spaces/default/memory/team.md` — `260910-cwsweep-cli` で affirm 済みの
  `## Way of Working` / `## Walking Skeleton` / `## Testing Posture` /
  `## Deployment` / `## Code Style`（5 セクションすべて非空であることを確認済み）。
- `aidlc/spaces/default/memory/project.md` — 同 intent で affirm 済みの
  `## Mandated` / `## Forbidden`。いずれも AWS 破壊的操作の安全性（dry-run既定・
  二重Identity検証・監査ログ必須・クレデンシャル非出力・`unsafe`禁止・
  `unwrap`/`expect`/`panic!`禁止）に関するハード制約。

## 参照したブラウンフィールド逆設計エビデンス（`aidlc/spaces/default/codekb/cwsweep/`）

- `code-structure.md`: 単一 Cargo パッケージ（バイナリ `src/main.rs` + ライブラリ
  `src/lib.rs`、12 モジュール=12 コンポーネント）。`src/cli.rs`（`CliApp`）が
  `clap` の `Cli` 構造体とオーケストレーションを担い、`--regions`（必須）・
  `--role-name`・`--output`・`--execute`・`--scan-only`（`conflicts_with =
  "execute"`）・`--audit-log-path` というフラットなフラグ構成であることを確認。
  `src/main.rs` の `main()` にある `cli.scan_only` 判定 → 非TTYフォールバック →
  対話式選択・確認・実行、という条件分岐が本 intent の再配置対象であることを確認。
- `architecture.md`: 「Improvement Opportunities」節および「サブコマンド化に
  向けた設計上の要点（本 intent 固有）」節に、この intent の設計意図
  （`scan`＝スキャンのみ、`clean`＝選択・削除フロー＋`--execute`で実行/dry-run切替、
  `config`はスコープ外、`--audit-log-path`の無効化オプションなし制約は維持）が
  codekb 側に既に記述されていることを確認。また「監査ログの読み取り API 不在」が
  技術的負債シグナルとして挙げられており、新設の `audit` 閲覧サブコマンドには
  `AuditLogger`/`AuditWrite` への読み取り API 追加が必要になることを確認。
- `code-quality-assessment.md`: CI の `coverage` ジョブが
  `cargo llvm-cov --lib --locked --fail-under-lines 80 --summary-only`（`--lib`
  のみ、バイナリクレート `src/main.rs` は計測対象外）であることを、品質担当
  エージェントの指摘により確認。
- `technology-stack.md` / `dependencies.md` / `business-overview.md`: 参照したが、
  本 intent（CLI サブコマンド化）の team/project 慣行の再確認に直接影響する新規
  事実は見つからなかった（ports-and-adapters・`secrecy`によるクレデンシャル
  マスキング等は既存の team.md/project.md の記述と整合していることを確認しただけ）。

## リポジトリ状態

- 参照コミット（本 stage 開始時点）: `29e3a629ab808d3f5314ab7ed46ac1e756798e14`
- 参照コミット（本 stage 完了時点、`git rev-parse HEAD`）: `0df33b1be665ad04961720077eea9b6d0823d379`

## 支援コントリビューションからの主な指摘（反映済み）

- **aidlc-quality-agent**: CI カバレッジ計測が `--lib` のみでバイナリクレートを
  対象外とする点を指摘し、サブコマンドディスパッチロジックの配置（lib 側 vs
  coverage ジョブ拡張）を人間インタビューに追加する必要性を提起（→ Q3 として
  出題、lib 側集約で確定）。また、既存の統合テスト `tests/scan_select_execute.rs`
  は `Cli`/`clap` パースに触れないドメイン層専用テストであり、「サブコマンド化で
  更新が必要」というリード案の記述は不正確であるとの訂正提案（→
  `team-practices.md` の該当箇所を訂正し、「新規の CLI 層統合テストの新設」が
  真のギャップであると明記）。
- **aidlc-developer-agent**: `run_scan`/`run_clean`/`run_audit` という `run_`
  接頭辞命名が既存コードの動詞のみ命名慣習（`scan_all`/`execute`等）の自然な
  延長とは言い切れない点、`audit` 閲覧用の新規読み取りAPIの配置モジュールが
  未確定な点、`audit` の異常系（不正ログ行）の挙動方針が未確定な点を指摘
  （→ それぞれ Q5・Q4（関連）・Q7 として出題し確定）。
- **aidlc-devsecops-agent**: `audit` サブコマンドを構造的に読み取り専用にする
  制約について、project.md の `## Forbidden`（ハード制約）への昇格を明確に
  推奨（既存 Forbidden 群と同じ「機械的に事故を防ぐ」思想との整合性を根拠に）
  （→ Q4 として出題し、推奨どおり Forbidden 昇格で確定）。既存のクレデンシャル
  非露出・監査ログ必須制約は実装（`src/credentials.rs`/`src/audit.rs`）に
  既に反映されており、サブコマンド化後も対象読み替えのみでよいと確認。

## 人間インタビューの最終回答（7問すべて解決済み）

1. **開発フロー・デプロイ方針**: 変更なし。既存のトランクベース開発・
   squash-merge・タグ駆動リリースをそのまま継承する。
2. **`clean` サブコマンドの M6 ゲート**: 既存の「破壊的操作を実装する Bolt は
   毎回ゲート」ルールは、`--execute` 相当の実行系操作を担う `clean` サブコマンドの
   実装 Bolt にそのまま適用される（サブコマンド名の読み替えのみで、運用自体の
   変更はない）。
3. **サブコマンド分岐ロジックの配置（カバレッジ計測の抜け穴対策）**: `Commands`
   （`scan`/`clean`/`audit`）へのディスパッチロジックは `lib` 側（`cli.rs` 等、
   `cargo llvm-cov --lib` の対象範囲）に置き、`main.rs` は薄い呼び出しのみに
   留める方針とする。CI の `coverage` ジョブ自体（`--lib` スコープ）は変更しない。
4. **`audit` サブコマンドの構造的読み取り専用化の強制レベル**: `project.md` の
   `## Forbidden`（ハード制約・機械的強制対象）に昇格する。`audit` ハンドラは
   `delete-log-group` / `put-retention-policy` を実行しうる型への依存を型／
   依存性注入レベルで一切持たない構造とする。
5. **ハンドラ関数の命名規則**: `run_scan` / `run_clean` / `run_audit` という
   `run_` 接頭辞方式を採用する。既存の `CliApp` メソッド命名（接頭辞なし動詞形）
   とは異なる慣習であることを認識した上での採用。
6. **旧フラグ（`--scan-only`/`--execute`）の後方互換**: 提供しない。v1 未リリース
   （0.1.0）段階であることを理由に、明示的な破壊的変更として割り切る。
   README/CHANGELOG に移行ガイドを明記する。
7. **`audit` が不正な監査ログ行を読んだ場合の挙動**: 該当行をスキップして警告を
   出力し、残りの正常なエントリの表示を継続する（処理全体を中断しない）。
   これに伴い、不正フォーマット行を含むフィクスチャで正常行が引き続き表示される
   ことを検証するテストを追加テスト観点として要求する。
