# rules.md — Unit: audit-reader

`audit-reader` Unit（`AuditReader` / `AuditRead`）が遵守するビジネスルール。
番号（BRグループ）は `entities.md` の `AuditEntry` / `SkippedLine` /
`AuditReadOutcome` に対応する。

```yaml
rules:
  - id: BR1.1
    statement: >
      監査ログ（JSON Lines）の読み取り中に、JSONとしてパースできない行、
      または必須フィールドを1つ以上欠く行に遭遇した場合、その行を
      SkippedLineとして記録し、標準エラーへ警告を出力した上で、残りの
      正常な行の読み取りを継続する。処理全体を中断しない。
    category: validation
    applies_to: AuditReadOutcome
    trigger: "監査ログの1行を読み取るとき"
    logic: >
      IF その行がJSONとしてパースできない OR BR1.2の必須フィールドが
      1つ以上欠けている
      THEN その行をSkippedLine（line_number, reason）として記録し、
           警告を標準エラーへ出力し、次の行の読み取りへ進む
    violation_behaviour: "（このルール自体は防御的スキップ処理であり、違反＝スキップ漏れは構造的回帰テストで検知する。NFR3参照）"
    source: "FR4.4"

  - id: BR1.2
    statement: >
      1行が有効なAuditEntryとして扱われるのは、run_id・timestamp・
      account_id・region・log_group_name・action_kind・event・successの
      8フィールドすべてが存在する場合に限る（error_messageのみ任意）。
    category: validation
    applies_to: AuditEntry
    trigger: "監査ログの1行をAuditEntryとして解釈しようとするとき"
    logic: >
      IF 上記8フィールドのいずれかが欠落している
      THEN その行は無効であり、BR1.1に従いSkippedLineとして扱う
    violation_behaviour: "無効な行はAuditEntryのリストに含めない"
    source: "AuditReadContract (contract-summary.md Contract 1)"

  - id: BR2.1
    statement: >
      監査ログファイルが指定パスに存在しない場合、エラーとせず、
      entries・skipped_linesとも空のAuditReadOutcomeを返して正常終了する。
    category: constraint
    applies_to: AuditReadOutcome
    trigger: "AuditRead::entries()の呼び出し開始時"
    logic: "IF ファイルが存在しない THEN 空のAuditReadOutcomeを返す（Errを返さない）"
    violation_behaviour: "ファイル不在を実行時エラー（AuditReadError）として扱うことは許容しない"
    source: "FR4.6"

  - id: BR2.2
    statement: >
      監査ログファイルが存在するにもかかわらず開けない、または読み取れ
      ない場合（権限不足等、ファイル不在以外のI/Oエラー）は、行単位の
      スキップ処理は行わず、AuditReadError::Ioとして呼び出し元へ
      伝播させる。
    category: constraint
    applies_to: AuditReadOutcome
    trigger: "ファイルのオープンまたは読み取り自体が失敗したとき"
    logic: "IF ファイル不在以外のI/Oエラー THEN AuditReadError::Ioを返す"
    violation_behaviour: "unwrap()/expect()/panic!でのクラッシュは許容しない（project.md Forbidden）"
    source: "AuditReadContract (contract-summary.md Contract 1)"

  - id: BR3.1
    statement: >
      初回実装のaudit-readerは、アカウントID・リージョン・成功/失敗等
      による絞り込み機能を一切持たない。ファイル全体をそのまま列挙する。
    category: constraint
    applies_to: AuditReadOutcome
    trigger: "AuditRead::entries()の呼び出し全体"
    logic: "常にファイル中の全行を対象とし、フィルタ条件を受け付けない"
    violation_behaviour: "将来のフィルタ追加は本intentのスコープ外（requirements.md Out of Scope）"
    source: "FR4.2"

  - id: BR3.2
    statement: >
      entriesおよびskipped_linesは、監査ログファイル中の出現順（記録順）
      を保持したまま返される。並べ替え・再グルーピングは行わない。
    category: constraint
    applies_to: AuditReadOutcome
    trigger: "AuditReadOutcomeを構築するとき"
    logic: "ファイルの行番号順にAuditEntry / SkippedLineを追加していく"
    violation_behaviour: "順序を変更する実装は、AuditEntryの識別子仕様（記録順序で一意性を担保）と矛盾する"
    source: "components.md AuditEntry identifier note"

  - id: BR4.1
    statement: >
      AuditReaderは監査ログファイルへの書き込み・作成を一切行わない。
      ファイル不在時も新規ファイルを作成しない。
    category: constraint
    applies_to: AuditReadOutcome
    trigger: "AuditRead::entries()の呼び出し全体"
    logic: "読み取り専用の操作のみを行い、いかなる書き込みAPIも呼び出さない"
    violation_behaviour: "書き込みを行う実装はaudit-readerの責務外であり、project.md Forbiddenに抵触する"
    source: "project.md Forbidden（audit経路の構造的分離）"

  - id: BR4.2
    statement: >
      AuditRead trait の実装（および実装が依存する型グラフ）は、
      delete-log-group／put-retention-policyを実行しうる型
      （ExecutionEngine）、および監査ログ書き込み系の型（AuditWrite/
      AuditLogger）への依存を一切持たない。この非依存性はコンパイル時点
      で到達不能な構造として強制される。
    category: authorization
    applies_to: AuditReadOutcome
    trigger: "AuditRead実装のモジュール／依存グラフ設計時"
    logic: >
      IF AuditReadの実装または依存先の型がExecutionEngine/AuditWriteに
      到達可能 THEN 設計として不成立（Code Generation/Build and Test段階
      の構造的回帰テストで検知する）
    violation_behaviour: "コードレビューのみによる担保は許容しない（型／依存性注入レベルでの構造的強制が必須）"
    source: "FR4.5 / project.md Forbidden"

  - id: BR5.1
    statement: >
      auditコマンドはevent（intent/result）による絞り込みを行わず、
      両方の種別のAuditEntryを時系列（記録順）にすべて表示する。
    category: policy
    applies_to: AuditEntry
    trigger: "auditコマンドの表示処理全体"
    logic: "AuditReadOutcome.entriesの全件を、event種別で除外せずそのまま表示対象とする"
    violation_behaviour: "result行のみへの絞り込みは、本ステージで確認済みの方針（Q1）と矛盾する"
    source: "FR4.1（本ステージ確認事項 Q1）"

  - id: BR5.2
    statement: >
      対応するresult行が存在しないintent行（未完了操作）を、他のAuditEntry
      と異なる特別な表示（pending/incomplete等の付加ステータス）にしない。
      event列の値のみで識別できるものとする。
    category: policy
    applies_to: AuditEntry
    trigger: "auditコマンドの表示処理全体"
    logic: "IF あるintent行に対応するresult行がない THEN 追加のステータス付与を行わず、他の行と同じ形式で表示する"
    violation_behaviour: "追加のハイライトロジックの実装は本ステージの確認事項（Q2）と矛盾する"
    source: "本ステージ確認事項 Q2（functional-design-questions.md）"
```

## 人間可読サマリー

| ID | 概要 | カテゴリ | 出典 |
|---|---|---|---|
| BR1.1 | 不正行はスキップ+警告し、処理を継続する | validation | FR4.4 |
| BR1.2 | 8フィールド全て揃った行のみ有効なAuditEntry | validation | Contract 1 |
| BR2.1 | ファイル不在は空の正常結果として扱う | constraint | FR4.6 |
| BR2.2 | ファイル不在以外のI/OエラーはAuditReadError::Io | constraint | Contract 1 |
| BR3.1 | 絞り込み機能は持たない（初回実装） | constraint | FR4.2 |
| BR3.2 | 記録順を保持する | constraint | components.md |
| BR4.1 | 書き込み・ファイル作成を一切行わない | constraint | project.md Forbidden |
| BR4.2 | ExecutionEngine/AuditWriteへの依存を型レベルで排除 | authorization | FR4.5 / project.md Forbidden |
| BR5.1 | intent/result両方を全件表示 | policy | FR4.1（Q1確認） |
| BR5.2 | 未完了操作を特別扱いしない | policy | Q2確認 |
