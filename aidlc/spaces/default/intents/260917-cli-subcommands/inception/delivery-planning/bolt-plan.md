# bolt-plan.md — Bolt計画（260917-cli-subcommands）

**Bolt**とは、Construction段階で実行する1回のビルドパスのことで、1つ以上のUnit of Work
（実装の作業単位）を束ね、完了条件・何を証明するかの仮説・担当を持つ。本intentは
DAG（audit-reader → cli-foundation → release-docsの一方向チェーン）のトポロジカル順を
そのまま採用し、3つのBoltに1 Unitずつを割り当てる（unit-of-work-dependency.md、
delivery-planning-questions.md Q1/Q3参照）。

## Bolt 1: audit-reader

- **含むUnit**: U1 audit-reader（`u1-audit-reader`）
- **ウォーキングスケルトン**: いいえ（本プロジェクトのスコープファイルは`skeleton: off`を
  宣言しており、アーキテクチャ全層を貫く薄い一枚岩を最初に作る運用は採らない）
- **完了条件（Definition of Done）**:
  - `AuditReader`（読み取り専用パーサ）・`AuditEntry`エンティティの実装
  - 監査ログのJSON Lines行単位ストリーミング読み取り
  - 不正フォーマット行のスキップ+標準エラーへの警告出力、正常行の継続表示（FR4.4）
  - 対象ファイル不在時の空扱い正常終了（FR4.6）
  - `AuditReader`の依存注入グラフに削除・retention変更・監査ログ書き込み系の型
    （`ExecutionEngine`/`AuditWrite`）が一切含まれないことを検証する構造的回帰テスト（NFR2）
  - 少なくとも1行の不正フォーマット行を含む監査ログフィクスチャによるテスト（NFR3）
  - 既存テストスイートが引き続きグリーン
- **確認したいこと（Confidence Hypothesis）**: project.mdのForbidden制約
  （`run_audit`ハンドラの依存注入グラフに削除・retention変更・監査ログ書き込み系の型を
  一切含めない、コンパイル時点で到達不能な構造とする）を、型設計と構造的回帰テストの
  組み合わせで実際に満たせるか。ここで実装パターンが確立すれば、Bolt 2の`run_audit`
  ハンドラ実装はそれをそのまま利用できる。
- **想定デモ**: `cargo test`で新設の構造的回帰テスト・不正行スキップテストがパスする様子、
  および`AuditReader`単体を呼び出す最小限のサンプルコードでの動作確認。

## Bolt 2: cli-foundation

- **含むUnit**: U2 cli-foundation（`u2-cli-foundation`）
- **ウォーキングスケルトン**: いいえ
- **完了条件（Definition of Done）**:
  - `Cli`/`Commands`（`scan`/`clean`/`audit`の3バリアント）への再構成
  - `run_scan`・`run_clean`・`run_audit`の3ハンドラメソッド実装（`run_`接頭辞、
    practices-discovery Q5）
  - `run_audit`がBolt 1の`AuditReader`を呼び出し、`OutputFormatter`経由で表示
  - 旧フラグ形式（`--scan-only`/`--execute`単体）の廃止、サブコマンド未指定時のclap標準
    エラー表示（FR6.1）
  - サブコマンド判定・ディスパッチロジックの`lib`側実装（`cargo llvm-cov --lib`計測対象、
    NFR1）
  - 新規CLI層統合テスト（`Cli::try_parse_from`によるパース検証、サブコマンド未指定時の
    振る舞いを含む）
  - 既存の`IdentityVerifier`二重検証・`AuditLogger`必須記録・dry-run既定の維持を検証する
    既存テストスイートが引き続きグリーン
- **確認したいこと（Confidence Hypothesis）**: 既存12コンポーネントの呼び出し順序を
  サブコマンド別に再編しても、既存の安全機構（Identity二重検証・dry-run既定・監査ログ
  必須記録）が壊れないか。壊れていないことは既存テストスイートのグリーン維持で証明する。
- **想定デモ**: `cwsweep scan`・`cwsweep clean`（`--execute`なし、dry-run）・`cwsweep audit`の
  それぞれを実行し、意図した振る舞い（scanは選択に進まない、cleanはdry-run既定、auditは
  Bolt 1のAuditReaderを経由して表示）を確認する。

**このBoltは`--execute`実装（削除・retention変更の実行）を含むため、team.mdのWalking
Skeleton方針により、Bolt完了時に人間の明示承認ゲートを必須とする**（自律進行モードを
選択した場合でも例外として維持）。

## Bolt 3: release-docs

- **含むUnit**: U3 release-docs（`u3-release-docs`）
- **ウォーキングスケルトン**: いいえ
- **完了条件（Definition of Done）**:
  - README.mdへの新コマンド体系（`scan`/`clean`/`audit`）の説明追加
  - CHANGELOGへの破壊的変更エントリ追加（旧フラグ→新サブコマンドの移行ガイド・対応表を含む）
  - `Cargo.toml`のバージョンを`0.1.0`から`0.2.0`へ更新
- **確認したいこと（Confidence Hypothesis）**: Bolt 1・Bolt 2で確定した実際の振る舞い
  （フラグ名・エラーメッセージ・出力形式）を、利用者が迷わず移行できる形で文書化できるか。
- **想定デモ**: README/CHANGELOGの差分レビュー、および`Cargo.toml`のバージョン番号確認。

## Bolt順序サマリー

| 順序 | Bolt | Unit | 複雑度 | ゲート要否 |
|---|---|---|---|---|
| 1 | audit-reader | U1 | M | 通常（自律 or 都度、team.md Ladder Promptに従う） |
| 2 | cli-foundation | U2 | M | **必須**（`--execute`実装のため毎回ゲート） |
| 3 | release-docs | U3 | S | 通常 |

DAG上、Bolt 1（audit-reader）はBolt 2（cli-foundation）の着手を待たずに独立して進行
可能である（unit-of-work-dependency.mdの「並行開発の余地」参照）が、本intentは単一
開発者体制・AI単独セッションでの逐次実施（delivery-planning-questions.md Q4）を
採用するため、実際の実施順序は上記の1→2→3を厳密に守る。
