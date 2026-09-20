# unit-of-work-dependency.md — Unit依存関係（260917-cli-subcommands）

本ファイルはUnit間のトポロジー（何が何に依存できるか）のみを記述する。Bolt実施順序の
経済的判断（value-first/risk-first/walking-skeleton-first）はDelivery Planning（2.9）の
スコープであり、ここでは決定しない。

## 依存DAG（プローズ）

- **audit-reader (U1)** は他のいずれのUnitにも依存しない（`depends_on: []`）。domain-design/
  components.mdの`AuditReader`が`depends_on: []`であることをそのまま反映する。
- **cli-foundation (U2)** は audit-reader (U1) に依存する。`CliApp`の`run_audit`ハンドラが
  `AuditReader`を呼び出すという、domain-design/components.mdの`CliApp.depends_on`→
  `AuditReader`エッジをそのまま反映する。
- **release-docs (U3)** は cli-foundation (U2) と audit-reader (U1) の双方に依存する。
  移行ガイド（旧フラグ→新サブコマンド対応表）とバージョン番号が、両Unitの最終的な
  インターフェース確定を前提とするため。

## 統合点（Unit間のAPI・共有データ・イベント）

Unit間にネットワークAPIや非同期イベントは存在しない（単一Cargoパッケージ内の再構成のため）。
統合点は同一クレート内の型シグネチャである:

| 統合点 | 提供Unit | 消費Unit | 形状 |
|---|---|---|---|
| `AuditRead`相当の読み取り専用トレイト・`AuditEntry`型 | audit-reader (U1) | cli-foundation (U2) | 同期呼び出し（クレート内関数呼び出し） |
| `Commands::Audit`バリアントの確定仕様 | cli-foundation (U2) | release-docs (U3) | ドキュメント記述の入力 |
| `run_scan`/`run_clean`のフラグ仕様（旧フラグとの対応） | cli-foundation (U2) | release-docs (U3) | 移行ガイドの入力 |
| `AuditEntry`一覧のtable/JSON整形仕様 | audit-reader (U1) + cli-foundation (U2, OutputFormatter) | release-docs (U3) | ドキュメント記述の入力 |

## 並行開発の余地

DAG上、audit-reader (U1) はいずれのUnitにも依存されて開始を待つ必要がなく、cli-foundation
(U2) の着手と並行して実施可能な複数の妥当なトポロジカル順序が存在する:

- 順序案1: audit-reader → cli-foundation → release-docs（厳密な順次）
- 順序案2: audit-reader と cli-foundation の一部（scan/clean部分、audit統合を除く）を並行実施し、
  cli-foundationのaudit統合完了後にrelease-docsへ進む

いずれの順序を採るかはDelivery Planning（2.9）が決定する。本Unitは単一開発者体制
（team.md Way of Working）であることを踏まえた経済的判断の対象であり、本ステージでは
決定しない。

## 機械可読エッジブロック

```yaml
units:
  - name: audit-reader
    kind: library
    depends_on: []
  - name: cli-foundation
    kind: service
    depends_on: [audit-reader]
  - name: release-docs
    kind: packaging
    depends_on: [cli-foundation, audit-reader]
```
