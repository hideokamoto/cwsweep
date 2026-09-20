# Domain Design — 質問

このintentは既存brownfieldの再構成であり、`architecture.md`の「サブコマンド化に向けた
設計上の要点」で方向性は示されているが、コンポーネント境界の具体的な決定はまだされていない。

## Q1. `CliApp`の分割方針

現行の`CliApp`は`scan_all`→`render_output`→`select`→`plan`→`confirm`→`execute`の
オーケストレーションを一手に担う唯一のハブである。サブコマンド化にあたり、この
オーケストレーション責務をどう分割しますか?

```question
prompt: "サブコマンド化にあたり、CliAppの分割方針はどれにしますか?"
header: "CliApp分割"
multiSelect: false
options:
  - label: "A. CliAppは維持しつつメソッドをサブコマンド別に整理"
    description: "CliApp自体は1つのまま、run_scan/run_clean/run_auditの3メソッドを持たせ、各メソッドが必要なコンポーネントのみを呼ぶ。既存のscan_all等の内部メソッドはそのまま再利用する"
  - label: "B. サブコマンドごとに専用の薄いオーケストレータ構造体を新設"
    description: "ScanApp/CleanApp/AuditAppのような専用構造体を新設し、CliAppは廃止またはコンストラクタのみに縮小する"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. CliApp維持しメソッド整理（推奨どおり確定）

## Q2. `AuditRead`(新設読み取り専用コンポーネント)の配置

`audit`サブコマンドのために、既存の追記専用`AuditLogger`/`AuditWrite`とは独立した
読み取り専用コンポーネントを新設する(project.md Forbiddenで確定済み: 削除・retention
変更系の型への依存を一切持たない)。

```question
prompt: "新設する読み取り専用の監査ログコンポーネント(AuditRead)は、どこに配置しますか?"
header: "AuditRead配置"
multiSelect: false
options:
  - label: "A. 既存src/audit.rsに追加する(同ファイル内で書き込み専用/読み取り専用を型で分離)"
    description: "同じファイル内にAuditReadトレイト・実装を追加。依存関係の型レベル分離は保ちつつファイルは共有する"
  - label: "B. 新規モジュール(例: src/audit_read.rs)として完全分離する"
    description: "ファイルレベルでも書き込み系(audit.rs)と読み取り系(audit_read.rs)を分離し、依存の非対称性を構造的にも明確にする"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. 既存src/audit.rsに追加する

## Q3. `AuditEntry`エンティティの導入

`audit`サブコマンドは監査ログ(JSON Lines)の各行をパースして表示する。この「1行分の
監査エントリ」をどう扱いますか?

```question
prompt: "audit サブコマンドが読み取る監査ログの1行分は、AuditEntryのような新規エンティティとしてAuditRead側で所有しますか、それとも既存のAuditLogger側で定義済みの型を再利用しますか?"
header: "AuditEntryの所有"
multiSelect: false
options:
  - label: "A. AuditRead側で新規にAuditEntryエンティティを所有する"
    description: "書き込み時のシリアライズ型と、読み取り時のデシリアライズ・表示用の型を分離する（read/writeで型を分離する設計と整合）"
  - label: "B. 既存のAuditLogger側の型を読み取り時にも再利用する"
    description: "型を1つに統一し、読み取り専用APIは既存型を返すだけにする"
  - label: "C. 自由記述で伝える"
    description: "別の方針を指定する"
  - label: "X. Other (please specify)"
    description: "その他"
```

[Answer]: A. AuditRead側で新規にAuditEntryエンティティを所有する

## Consolidated Summary Confirmation

- CliApp: 維持し、run_scan/run_clean/run_auditの3ハンドラメソッドを追加(ADR-001)
- AuditReader: 新設コンポーネント。既存src/audit.rsに同居させつつ型レベルで完全分離、depends_on: []。AuditEntryエンティティを新規所有(ADR-002)
- OutputFormatter: 責務を拡張し、AuditReader由来のAuditEntry一覧のtable/JSON出力にも対応(ADR-003)
- 上記以外の10コンポーネントは変更なし

Does this all look correct before I generate the artifact?

[Answer]: Looks correct
