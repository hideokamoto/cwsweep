# entities.md — Unit: audit-reader

`audit-reader` Unit（`AuditReader` コンポーネント）が扱うエンティティ定義。
本Unitはこのエンティティを**読み取り専用**で列挙するのみであり、生成・更新・
削除は一切行わない（`project.md` Forbidden: `audit` ハンドラは削除・
retention変更・監査ログ書き込み系の型に到達不能でなければならない）。

`AuditEntry` の実際のフィールド構成は、既存の書き込み側 `AuditLogger`
（`src/audit.rs`）が出力するJSON Linesのスキーマをそのまま踏襲する。
Inception段階（`contract-summary.md` Contract 1）で仮置きされた簡略モデル
（`action` / `result` の自由文字列）は、実装済みの実際のスキーマとは異なる
ことが本ステージで判明したため、実際のスキーマに合わせて確定する
（本ステージ Q1/Q2 の確認事項を参照）。

```yaml
entities:
  - name: AuditEntry
    description: >
      監査ログ（JSON Lines）の1行を表す読み取り専用エンティティ。1つの
      削除・retention変更操作につき、API呼び出し「前」の意図(intent)行と
      「後」の結果(result)行の2行が対応する場合がある（対応するresult行が
      存在しないintent行＝未完了操作もありうる）。
    attributes:
      - name: run_id
        type: string
        required: true
        unique: false
        references: null
        allowed_values: null
        default: null
        constraints: "実行単位を識別する文字列。1回の cwsweep clean 実行内で発生した intent/result のペアリングに用いる（一意性の強制はしない）"
      - name: timestamp
        type: string (ISO 8601)
        required: true
        unique: false
        references: null
        allowed_values: null
        default: null
        constraints: "UTC ISO 8601形式のタイムスタンプ文字列。フォーマット検証は行わず、存在確認のみ行う"
      - name: account_id
        type: string
        required: true
        unique: false
        references: null
        allowed_values: null
        default: null
        constraints: "対象AWSアカウントID"
      - name: region
        type: string
        required: true
        unique: false
        references: null
        allowed_values: null
        default: null
        constraints: "対象AWSリージョン"
      - name: log_group_name
        type: string
        required: true
        unique: false
        references: null
        allowed_values: null
        default: null
        constraints: "対象CloudWatch Logsロググループ名"
      - name: action_kind
        type: enum
        required: true
        unique: false
        references: null
        allowed_values: ["Delete", "SetRetention"]
        default: null
        constraints: "SetRetentionは追加の`days`（整数）を伴う。既存の`ActionKind`（`src/planner.rs`）と同一の構造を持つ"
      - name: event
        type: enum
        required: true
        unique: false
        references: null
        allowed_values: ["intent", "result"]
        default: null
        constraints: "intent = API呼び出し前の実行意図記録。result = API呼び出し後の結果記録（成功・失敗いずれも）。本Unitはこの値による絞り込みを行わず、両方をそのまま列挙する（本ステージ確認事項 Q1）"
      - name: success
        type: boolean
        required: true
        unique: false
        references: null
        allowed_values: null
        default: null
        constraints: "event=resultの場合のみ確定した成功/失敗を表す。event=intentの場合は結果未確定のプレースホルダ値`false`であり、失敗を意味しない（呼び出し元は`event`と併せて解釈する）"
      - name: error_message
        type: string
        required: false
        unique: false
        references: null
        allowed_values: null
        default: null
        constraints: "success=falseの場合に付与されうる任意のエラーメッセージ。クレデンシャル等の機微情報を含まない（project.md Forbidden）"
    entity_constraints:
      - "対応するresult行が存在しないintent行（未完了操作）を特別扱いしない。event列の値のみで区別できるものとし、追加のステータスは導入しない（本ステージ確認事項 Q2）"
      - "全フィールド（error_messageを除く）が揃っていない、またはJSONとしてパースできない行は、このエンティティのインスタンスとして扱わず SkippedLine として別扱いする（rules.md BR1.1/BR1.2参照）"
    relationships: []

  - name: SkippedLine
    description: >
      不正フォーマット・欠落フィールドによりAuditEntryとして解釈できな
      かった監査ログの1行を表す。表示対象ではなく、警告出力と診断のため
      の記録である。
    attributes:
      - name: line_number
        type: integer
        required: true
        unique: false
        references: null
        allowed_values: null
        default: null
        constraints: "監査ログファイル内の行番号（1始まり）"
      - name: reason
        type: string
        required: true
        unique: false
        references: null
        allowed_values: null
        default: null
        constraints: "スキップ理由（JSONパース失敗／必須フィールド欠落、のいずれか）を人間可読な文で表す"
    entity_constraints: []
    relationships: []

  - name: AuditReadOutcome
    description: >
      監査ログ読み取り1回分の結果全体を表す集約。AuditEntryの列挙と
      SkippedLineの列挙の両方を保持する。対象ファイルが存在しない場合は
      entries・skipped_linesとも空のインスタンスとして正常に生成される
      （rules.md BR2.1）。
    attributes:
      - name: entries
        type: "list of AuditEntry"
        required: true
        unique: false
        references: AuditEntry
        allowed_values: null
        default: "[]"
        constraints: "監査ログファイルに記録された順序をそのまま保持する（rules.md BR3.2）"
      - name: skipped_lines
        type: "list of SkippedLine"
        required: true
        unique: false
        references: SkippedLine
        allowed_values: null
        default: "[]"
        constraints: "ファイル中の出現順を保持する"
    entity_constraints: []
    relationships:
      - target: AuditEntry
        cardinality: "1..N or 0"
        direction: "AuditReadOutcome has many AuditEntry"
      - target: SkippedLine
        cardinality: "0..N"
        direction: "AuditReadOutcome has many SkippedLine"
```

## 人間可読サマリー

`AuditEntry` は既存の `AuditLogger` が実際に書き込む9フィールド構成
（`run_id` / `timestamp` / `account_id` / `region` / `log_group_name` /
`action_kind` / `event` / `success` / `error_message`）をそのまま読み取る。
1操作につき最大2件の `AuditEntry`（`intent` と `result`）が対応しうるが、
本Unitはこれらを対にせず、記録順のフラットな一覧として扱う。パース不能・
欠落フィールドを持つ行は `AuditEntry` にはならず `SkippedLine` として
別集計され、処理全体は継続する。`AuditReadOutcome` はこの2種類の列挙を
束ねる読み取り結果全体を表し、対象ファイル不在時も正常な（空の）
インスタンスとして返る。
