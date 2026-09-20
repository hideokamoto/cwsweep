# rules.md — Unit: cli-foundation

`cli-foundation` Unit（`Cli` / `Commands` / `CliApp::run_scan` /
`run_clean` / `run_audit`）が遵守するビジネスルール。BRグループは
BR1=引数解析、BR2=scan、BR3=clean、BR4=audit、BR5=安全機構維持、
BR6=構造要件に対応する。

```yaml
rules:
  - id: BR1.1
    statement: >
      サブコマンド（scan / clean / audit）を指定しない呼び出しは、後方互換
      フォールバックなしにパースエラーとして扱い、使用方法メッセージを表示して
      非ゼロ終了する。
    category: validation
    applies_to: Cli
    trigger: "コマンドライン引数を解析するとき"
    logic: >
      IF 先頭の位置引数が scan / clean / audit のいずれでもない（未指定を含む）
      THEN 使用方法メッセージを標準エラーへ出力し、コマンドライン誤用の終了コードで終了する
    violation_behaviour: "旧フラグ形式（`cwsweep --regions ...`）が動作してしまうことはFR6.1違反"
    source: "FR1.1, FR6.1"

  - id: BR1.2
    statement: >
      各オプションは所属するサブコマンドでのみ受理される。scan は --output を持ち
      --execute / --audit-log-path を持たない。clean は --execute /
      --audit-log-path を持ち --output を持たない。audit は --audit-log-path /
      --output のみを持つ。
    category: validation
    applies_to: Commands
    trigger: "サブコマンドのオプションを解析するとき"
    logic: >
      IF そのサブコマンドに所属しないオプションが指定された
      THEN パースエラー（未知の引数）として扱い BR1.1 と同じ形で終了する
    violation_behaviour: "受理してはならないオプションを黙って無視することは禁止"
    source: "FR2.2, FR3.2, FR4.1"

  - id: BR1.3
    statement: >
      --regions は scan / clean で必須・1件以上・カンマ区切りまたは複数回指定可であり、
      重複は初出の位置を保って除去する。全リージョン自動列挙などのフォールバックは
      持たない。
    category: validation
    applies_to: Commands
    trigger: "scan / clean の --regions を解析するとき"
    logic: >
      IF --regions が未指定 THEN パースエラー
      ELSE 重複を初出順で除去した一覧を保持する
    violation_behaviour: "既定リージョンを暗黙に補うことは禁止"
    source: "FR2.1, FR3.1（既存 security-design 継承）"

  - id: BR2.1
    statement: >
      run_scan はスキャン結果の表示のみを行い、対話式選択・アクションプラン生成・
      確認・実行のいずれにも進まない。
    category: constraint
    applies_to: run_scan
    trigger: "scan サブコマンドが実行されたとき"
    logic: >
      スキャン → 集約 → 出力 で必ず終了する。select / plan / confirm / execute
      への遷移経路を持たない
    violation_behaviour: "scan 経路から delete-log-group / put-retention-policy に到達可能なら設計違反"
    source: "FR2.1"

  - id: BR2.2
    statement: >
      run_scan は監査ログへ一切書き込まない。監査ログ書き込み型（AuditWrite）を
      依存として要求しない。
    category: constraint
    applies_to: run_scan
    trigger: "scan サブコマンドが実行されたとき"
    logic: >
      run_scan の依存集合は CredentialProvider / IdentityCheck /
      DescribeLogGroups / OutputFormatter に限られ、AuditWrite / ActionApi を含まない
    violation_behaviour: "scan 実行後に監査ログファイルが生成・変更されていれば違反"
    source: "FR2.2"

  - id: BR2.3
    statement: >
      run_scan の終了方針は R-03 を維持する: 対象アカウント×リージョンが1件以上あり
      その全件が失敗した場合のみ非ゼロ終了。1件でも成功があれば正常終了し、失敗分は
      警告ログで個別に報告する。対象0件は正常終了。
    category: policy
    applies_to: ExitDisposition
    trigger: "run_scan / run_clean のスキャンフェーズが完了したとき"
    logic: >
      IF outcomes が空でない AND outcomes の全件が Failed
      THEN ExitDisposition::ScanFullyFailed
      ELSE 続行
    violation_behaviour: "一部失敗を非ゼロ終了にするのは CI 契約違反"
    source: "FR2.4"

  - id: BR2.4
    statement: >
      run_scan でロググループが0件のとき、table 出力では棚卸し文脈の0件メッセージを
      標準出力へ表示し、json 出力では構造化出力のみ（空の records）を返し追加文言を
      混ぜない。
    category: policy
    applies_to: run_scan
    trigger: "スキャン結果の集約件数が0のとき"
    logic: >
      IF output == json THEN 構造化出力のみ
      ELSE 構造化出力（空table）に続けて0件メッセージを出力する
    violation_behaviour: "json 出力に非JSON文言が混入すると機械可読性を失う"
    source: "functional-design-questions.md Q2"

  - id: BR3.1
    statement: >
      run_clean は既存のデフォルトフロー（スキャン → 対話式マルチセレクト（初期状態は
      全チェックOFF）→ アクションプラン生成 → 確認画面 → 実行）をそのまま担う。
    category: policy
    applies_to: run_clean
    trigger: "clean サブコマンドが実行されたとき"
    logic: >
      scan_all → （0件なら終了）→ select → （0件選択なら終了）→ plan → confirm →
      （確認否なら終了）→ execute の順で進む
    violation_behaviour: "順序の入れ替え・確認ステップの省略は禁止"
    source: "FR3.1"

  - id: BR3.2
    statement: >
      --execute が明示的に指定されない限り、delete-log-group / put-retention-policy
      は一切呼び出されない（dry-run既定）。
    category: authorization
    applies_to: run_clean
    trigger: "確認済みアクションを実行するとき"
    logic: >
      IF execute == false THEN 各アクションは dry-run としてスキップ記録され API は
      呼ばれない
    violation_behaviour: "--execute なしで API 呼び出しが発生したら重大違反"
    source: "FR3.3"

  - id: BR3.3
    statement: >
      run_clean は監査ログを --audit-log-path（既定 cwsweep-audit.jsonl）へ必ず記録する。
      無効化オプションは存在しない。
    category: policy
    applies_to: run_clean
    trigger: "clean サブコマンドが実行されたとき"
    logic: >
      監査ログ出力先は起動時に必ずオープンされ、オープン失敗は clean 全体の失敗とする
    violation_behaviour: "監査ログなしで実行フェーズへ進むことは禁止"
    source: "FR3.5"

  - id: BR3.4
    statement: >
      標準入力がTTYでない環境で clean が呼ばれた場合、スキャン結果を表示した上で
      対話式選択に進めない旨と scan の利用案内を警告し、正常終了する。破壊的操作へは
      進まない。
    category: policy
    applies_to: run_clean
    trigger: "スキャン完了後、対話式選択へ進む直前"
    logic: >
      IF stdin が TTY でない THEN 警告を出力して ExitDisposition::Success
    violation_behaviour: "非TTYで確認プロンプトを迂回して実行へ進むことは禁止"
    source: "functional-design-questions.md Q1"

  - id: BR4.1
    statement: >
      run_audit は AuditRead（読み取り専用契約）から AuditReadOutcome を受け取り、
      entries を絞り込みなしで OutputFormatter へ渡して表示する。
    category: policy
    applies_to: run_audit
    trigger: "audit サブコマンドが実行されたとき"
    logic: >
      entries() → 全件を --output に応じて整形 → 標準出力
    violation_behaviour: "event 種別や成功/失敗での暗黙フィルタは禁止"
    source: "FR4.1, FR4.2, FR4.3"

  - id: BR4.2
    statement: >
      run_audit の依存は AuditRead と OutputFormatter のみとし、ExecutionEngine /
      AuditWrite / CredentialProvider / IdentityCheck / AWS クライアント類を一切
      要求しない。AWS への通信を行わない。
    category: constraint
    applies_to: run_audit
    trigger: "run_audit の依存を構成するとき"
    logic: >
      run_audit の入力は AuditRead 実装と OutputFormat の2つに限定する
    violation_behaviour: "run_audit から delete / put-retention / 監査ログ書き込みへ到達可能なら重大違反"
    source: "FR4.5, FR4.6"

  - id: BR4.3
    statement: >
      audit の table 出力は run_id を除く8列（timestamp / event / account_id /
      region / log_group_name / action_kind / success / error_message）、json 出力は
      AuditReadEntry の全9フィールドを配列でシリアライズする。intent 行の success は
      table では未確定表記（-）とする。
    category: policy
    applies_to: OutputFormatter
    trigger: "AuditReadOutcome.entries を整形するとき"
    logic: >
      IF output == table THEN 8列テーブル（intent 行の success は "-"）
      ELSE entries をそのまま JSON 配列へ
    violation_behaviour: "json で情報を欠落させることは禁止"
    source: "FR4.3, Contract 2, functional-design-questions.md Q3"

  - id: BR4.4
    statement: >
      run_audit は skipped_lines の警告を再出力しない（AuditReader が読み取り中に
      標準エラーへ出力済み）。読み取りが AuditReadError で失敗した場合はエラー終了する。
    category: policy
    applies_to: run_audit
    trigger: "entries() の結果を受け取ったとき"
    logic: >
      IF Err(AuditReadError) THEN ExitDisposition::Error
      ELSE 表示して Success（skipped_lines は再出力しない）
    violation_behaviour: "警告の二重出力は禁止"
    source: "FR4.4, Contract 1"

  - id: BR5.1
    statement: >
      scan / clean のいずれにおいても、メンバーアカウントへの AWS API 呼び出し直前の
      Identity 検証（sts:get-caller-identity）、実行直前の二重目 Identity 検証、
      describe-log-groups のページネーション完走、管理アカウントへの AssumeRole 非実施
      という既存機構を変更しない。
    category: constraint
    applies_to: run_scan, run_clean
    trigger: "スキャン・実行フェーズで AWS API を呼び出すとき"
    logic: >
      既存の LogGroupScanner / ExecutionEngine / CredentialProvider の呼び出し
      順序・依存注入をそのまま再利用する
    violation_behaviour: "サブコマンド化に伴う検証の省略は禁止"
    source: "FR5.1, FR5.2, FR5.3, FR5.4"

  - id: BR6.1
    statement: >
      サブコマンド判定・ディスパッチ・各ハンドラのロジックは lib クレート側に置き、
      main は実 AWS SDK アダプタの実体化とハンドラ呼び出しのみを行う。
    category: constraint
    applies_to: CliApp
    trigger: "実装配置を決定するとき"
    logic: >
      main には分岐ロジック（サブコマンド判定・終了コード判定・0件判定）を置かない
    violation_behaviour: "lib カバレッジ計測対象外にロジックが漏れることは NFR1 違反"
    source: "FR1.2, NFR1"

  - id: BR6.2
    statement: >
      旧フラグ（--scan-only、トップレベル --execute / --regions）に対する後方互換
      エイリアス・シムを実装せず、廃止状態をテストで固定化する。
    category: constraint
    applies_to: Cli
    trigger: "引数解析仕様を変更するとき"
    logic: >
      旧形式の呼び出しは BR1.1 のパースエラーになることを回帰テストで確認する
    violation_behaviour: "隠しエイリアスの追加は NFR4 違反"
    source: "FR6.1, NFR4"
```

## ルールサマリー

| ID | 概要 |
|---|---|
| BR1.1 | サブコマンド未指定はパースエラー、後方互換なし |
| BR1.2 | オプションは所属サブコマンドでのみ受理 |
| BR1.3 | --regions 必須・重複除去・フォールバックなし |
| BR2.1 | scan は表示のみ、選択/確認/実行へ進まない |
| BR2.2 | scan は監査ログへ書き込まない |
| BR2.3 | R-03 終了コード方針を維持 |
| BR2.4 | scan 0件時のtable/json出力方針 |
| BR3.1 | clean は既存デフォルトフローを担う |
| BR3.2 | --execute なしでは API を呼ばない（dry-run既定） |
| BR3.3 | clean は監査ログを必ず記録、無効化不可 |
| BR3.4 | 非TTYの clean は警告して正常終了 |
| BR4.1 | audit は全件を絞り込みなしで表示 |
| BR4.2 | run_audit の依存は AuditRead と OutputFormatter のみ |
| BR4.3 | audit の table 8列 / json 全9フィールド |
| BR4.4 | 警告は再出力しない、読み取り失敗はエラー終了 |
| BR5.1 | 既存 Identity 二重検証・ページネーション・AssumeRole 方針を維持 |
| BR6.1 | ロジックは lib 側、main は配線のみ |
| BR6.2 | 旧フラグの後方互換シムを実装しない |
