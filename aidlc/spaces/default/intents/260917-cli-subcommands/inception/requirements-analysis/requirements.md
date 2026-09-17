# Requirements — CLIサブコマンド化 (260917-cli-subcommands)

## Intent Analysis

cwsweepの現行CLIは、`clap`によるフラットなフラグ構成（`--regions`必須、`--execute`と
`--scan-only`が相互排他）の単一コマンドである。破壊的操作（ログ削除・retention変更）と
読み取り専用のスキャンが同一コマンドの1フラグの差でしか区別されず、誤って`--execute`
を付け忘れる／付けてしまう事故のリスクがある(P3: 構造で防ぐ)。

本intentのゴールは、この安全性を**コマンド構造そのもの**で担保することである:

- `scan` — 読み取り専用の棚卸し。削除・retention変更APIへの経路をそもそも持たない。
- `clean` — 現行のデフォルトフロー（スキャン→対話式選択→確認→実行）を担う。
  `--execute`を明示しない限り破壊的操作は呼ばれない(dry-run既定、既存方針を継承)。
- `audit` — 新設。監査ログ(JSON Lines)の閲覧専用サブコマンド。削除・retention変更APIへの
  依存を型レベルで一切持たない構造的強制（practices-discovery Q4でproject.md Forbidden化
  済み）。

`config`サブコマンドは本intentのスコープ外。

practices-discoveryの人間インタビュー(7問)で既に確定した実装方針（lib側ディスパッチ、
`run_`接頭辞のハンドラ命名、audit読み取り専用の型レベル強制、後方互換シムなし、audit不正
ログ行のスキップ+警告継続、開発フロー・デプロイ方針は変更なし）は、この要件定義でも
そのまま前提とする。

## Functional Requirements

### FR1. サブコマンド体系への再構成

- **FR1.1**: `cwsweep`は3つのサブコマンド`scan`・`clean`・`audit`を持つclapベースの
  コマンドとして再構成される。サブコマンドを指定しない呼び出し（例: `cwsweep --regions ...`
  のみ）は、後方互換のフォールバックなしにエラーとして扱う（FR6.1参照）。
- **FR1.2**: サブコマンドの判定・ディスパッチロジックは`lib`クレート側（`src/cli.rs`等、
  `cargo llvm-cov --lib`の計測対象範囲）に実装し、`src/main.rs`は実AWS SDKアダプタの実体化
  とハンドラ呼び出しのみを行う薄い配線層として維持する（practices-discovery Q3で確定済みの
  構造要件）。

### FR2. `scan` サブコマンド

- **FR2.1**: `cwsweep scan --regions <regions> [--role-name <name>] [--output table|json]`
  の形式で、AWS Organization全体（管理アカウント＋メンバーアカウント）×指定リージョンの
  CloudWatch Logsロググループを棚卸しし、結果を表示して終了する。対話式選択・確認・実行には
  一切進まない。
- **FR2.2**: `scan`は`--audit-log-path`を受け付けない。`scan`は監査ログへの書き込みを一切
  行わない読み取り専用サブコマンドであるため。
- **FR2.3**: `scan`のハンドラ関数は`run_scan`と命名する（practices-discovery Q5）。
- **FR2.4**: `scan`の終了コード方針（R-03: 対象アカウント×リージョンの組み合わせが1件以上
  あり、その全件でスキャンが失敗した場合のみ非ゼロ終了）は、現行の`cwsweep-orgwide-logs-cleanup`
  intentで確立された挙動をそのまま維持する。

### FR3. `clean` サブコマンド

- **FR3.1**:
  `cwsweep clean --regions <regions> [--role-name <name>] [--execute] [--audit-log-path <path>]`
  の形式で、スキャン→対話式マルチセレクト（初期状態は常に全チェックOFF）→削除/retention変更
  アクションプランの生成→確認画面提示→（`--execute`指定時のみ）実行、という現行のデフォルト
  フローを担う。
- **FR3.2**: `clean`は`--output`を受け付けない。対話式選択に進む前提のサブコマンドであり、
  スキャン結果表示フォーマットを切り替える理由がないため。
- **FR3.3**: `--execute`を明示的に渡さない限り、`delete-log-group`・`put-retention-policy`は
  一切呼び出されない（dry-run既定、project.md Mandated/Forbiddenを継承）。
- **FR3.4**: `clean`のハンドラ関数は`run_clean`と命名する（practices-discovery Q5）。
- **FR3.5**: `clean`は`--audit-log-path`の既定値（後方互換のためカレントディレクトリ直下
  `cwsweep-audit.jsonl`）を維持する。無効化オプションは設けない（project.md Mandated）。

### FR4. `audit` サブコマンド（新設）

- **FR4.1**: `cwsweep audit [--audit-log-path <path>] [--output table|json]`の形式で、
  `--audit-log-path`が指すJSON Lines形式の監査ログファイル（既定`cwsweep-audit.jsonl`）を
  読み取り専用で表示する。
- **FR4.2**: 初回実装では絞り込み（アカウントID・リージョン・成功/失敗等でのフィルタ）は
  持たない。ファイル全体をそのまま表示する。将来的な絞り込み追加は本intentのスコープ外。
- **FR4.3**: `audit`は`scan`と同様に`--output table|json`を持つ。`table`が既定で、`json`は
  CLI/エージェント向けの構造化出力を提供する。
- **FR4.4**: 監査ログの読み取り中に不正フォーマット・欠落フィールドを持つ壊れた行に遭遇した
  場合、その行をスキップし警告を標準エラーへ出力した上で、残りの正常な行の表示を継続する
  （処理全体を中断しない）。
- **FR4.5**: `audit`のハンドラ関数`run_audit`は、`delete-log-group`・`put-retention-policy`
  を実行しうる型（`ExecutionEngine`、書き込み系`AuditWrite`等）への依存を一切持たない構造
  とする。読み取り専用の型（`AuditRead`相当、新設）のみを依存注入し、削除・retention変更・
  監査ログの書き込み/改変を行う経路にコンパイル時点で到達不能な構造とする（project.md
  Forbidden、practices-discovery Q4で確定済み）。
- **FR4.6**: `audit`が監査ログを一切生成しない（新規に何も書き込まない）ことを前提とする。
  監査対象ファイルが存在しない場合は、空扱い（0件表示）として正常終了する。

### FR5. 既存アクセス制御・安全機構の維持

- **FR5.1**: いずれかのメンバーアカウントに対するAWS API呼び出し（`scan`・`clean`の実行を
  含む）の直前に`sts:get-caller-identity`を実行し、想定アカウントIDと一致することを検証する
  （project.md Mandated、サブコマンド化後も維持）。
- **FR5.2**: `clean`が`delete-log-group`・`put-retention-policy`を実行する直前に、スキャン
  時点とは独立した二重目のIdentity検証を再実行する（project.md Mandated）。
- **FR5.3**: `scan`・`clean`ともに`cloudwatch-logs:describe-log-groups`はページネーションを
  最後まで辿り、全ページ取得後に集計する。
- **FR5.4**: 管理アカウント自身へは`AssumeRole`を行わず、現在の認証情報をそのまま使用する。
  メンバーアカウントに対してのみ`AssumeRole`（既定ロール名`OrganizationAccountAccessRole`、
  `--role-name`で上書き可能）を行う。

### FR6. 破壊的変更としての明示

- **FR6.1**: 旧フラグ方式（`cwsweep --regions ... --execute`／`--scan-only`）は、
  後方互換のエイリアスなしに廃止する。旧フラグ形式での呼び出しは、サブコマンド未指定として
  clapの標準エラー（使用方法メッセージ）を表示して終了する。
- **FR6.2**: README.mdおよびCHANGELOGに、この破壊的変更（旧フラグ→新サブコマンドの対応表を
  含む移行ガイド）を明示する。
- **FR6.3**: `Cargo.toml`のバージョンを`0.1.0`から`0.2.0`へ上げる。

## Non-Functional Requirements

- **NFR1（テスト容易性）**: サブコマンド判定・ディスパッチロジックは`lib`クレート側に実装し、
  `cargo llvm-cov --lib`のカバレッジ計測対象に含める。既存の80%ライン/100%パスカバレッジ
  floor（project.md/team.md Testing Posture）は、サブコマンド化後もこのディスパッチ層を含めて
  達成されなければならない。
- **NFR2（構造的安全性）**: `audit`ハンドラの依存注入グラフに、削除・retention変更・監査ログ
  書き込み系の型が一切含まれないことを、型/モジュール境界レベルの回帰テストで検証可能でなけ
  ればならない。
- **NFR3（監査ログ完全性）**: `audit`の不正行スキップ挙動（FR4.4）は、少なくとも1行の不正
  フォーマット行を含む監査ログフィクスチャに対し、(a)不正行のスキップと警告出力、
  (b)不正行前後の正常エントリの継続表示、の両方をテストで検証できなければならない。
- **NFR4（後方互換の意図的放棄）**: 旧フラグ形式の呼び出しに対するフォールバック処理を実装
  しないこと自体をテストで固定化し、将来の意図しない後方互換シムの混入を検出できるようにする。
- **NFR5（既存NFRの継続）**: クレデンシャル非露出（project.md Forbidden）、`unsafe`禁止、
  `unwrap`/`expect`/`panic!`の本番コードパス排除、`cargo audit`/`cargo deny check`のCI必須
  ゲートは、サブコマンド化後も変更なく維持する。

## Constraints

- 本プロジェクトは単一開発者体制であり、形式的なプルリクエストの複数人承認は求めない
  （CI green + squash-mergeのシンプルな運用、team.md Way of Working）。
- `clean`サブコマンドの`--execute`実装Boltは、既存のWalking Skeleton方針により毎回ゲート
  対象とする（practices-discovery Q2で確定）。
- v1未リリース（現行0.1.0→0.2.0への変更含む）段階であり、crates.ioへの公開は対象外。

## Assumptions

- `--role-name`・`--audit-log-path`等の既存フラグの意味・既定値は、サブコマンド化によって
  変わらない（フラグの所属先サブコマンドが変わるのみ）。
- `config`サブコマンドは将来的な検討対象であり、本intentのコード変更には一切含まれない。
- `AuditRead`相当の読み取り専用APIの具体的な内部設計（ストリーミング読み取りかフルロードか等）
  は、Domain Design以降の段階で決定する。

## Out of Scope

- `config`サブコマンド（設定ファイル対応）は本intentのスコープ外。
- 旧フラグ形式への後方互換エイリアス・移行シムは提供しない。
- `audit`サブコマンドの絞り込み（アカウントID・リージョン・成功/失敗等でのフィルタ）は
  初回実装では持たない。
- CloudWatch Logs以外のリソース種別（EC2、S3、EBS等）への操作は引き続きv1スコープ外。

## Open Questions

- なし（本intentのrequirements-analysisで解決すべき論点はすべて確認済み）。`AuditRead`の
  具体的なストリーミング/フルロード方式の選定は、Domain Design以降のスコープとする。
