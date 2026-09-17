# Discovered Rules — 最終版（260917-cli-subcommands）

> `260910-cwsweep-cli` で affirm 済みの `project.md` の `## Mandated` /
> `## Forbidden` を UNCHANGED のまま継承する。本 intent（CLI のサブコマンド化）は
> これらのハードな制約に抵触する変更を含まないため、既存項目はすべてそのまま維持する。
> 本 intent の人間インタビュー（Q4）で唯一新規に project.md への昇格が確定した
> 1 件の Forbidden のみを末尾に追記する。

## Mandated

- ALWAYS: いずれかのメンバーアカウントに対する AWS API 呼び出し（スキャン・削除・retention変更の
  いずれも含む）の直前に `sts:get-caller-identity` を実行し、想定アカウントIDと一致することを検証する。
- ALWAYS: `delete-log-group` または `put-retention-policy` を実行する直前に、スキャン時点とは独立した
  二重目の Identity 検証を再実行する。
- ALWAYS: `cloudwatch-logs:describe-log-groups` はページネーションを最後まで辿り、全ページ取得後に
  集計する。1ページのみで集計を確定する実装を許容しない。
- ALWAYS: 管理アカウント自身に対しては `AssumeRole` を行わず、現在の認証情報をそのまま使用する。
  メンバーアカウントに対してのみ `AssumeRole`（デフォルトロール名 `OrganizationAccountAccessRole`、
  引数で上書き可能）を行う。
- ALWAYS: 削除・retention変更を伴うすべての操作について、対象アカウントID・リージョン・ログループ名・
  実行時刻・成功/失敗を監査ログに出力する。無効化オプションは設けない。
- ALWAYS: 削除・retention変更の実行前に、対象アカウントID・リージョン・ログループ名一覧・合計バイト数を
  再掲した確認画面を提示する。
- ALWAYS: 監査ログへの書き込みに失敗した場合、実行中の削除・retention変更操作自体を中断し、
  エラーとして扱う。ログが残せない操作は実行しない（警告のみで続行することは許容しない）。
- ALWAYS: 依存クレートの脆弱性スキャン（`cargo audit`）を CI で実行し、既知脆弱性のある依存があれば
  マージをブロックする。あわせて `cargo deny check`（advisories / licenses / bans / sources）を
  CI 必須ゲートとし、`Cargo.lock` をコミットして `--locked` フラグでビルド・テストする。

（本 intent での確認結果：サブコマンド化後も、上記の Identity 二重検証・監査ログ必須・dry-run既定・
確認画面再掲の各制約は、実行系操作を担う `clean` サブコマンドの経路に読み替えて維持される。これは
既存 Mandated の適用対象をサブコマンド構造に読み替えるものであり、内容自体の変更ではない。新規
昇格ではないため、上記リストへの追記は行わない。）

## Forbidden

- NEVER: `--execute` フラグが明示的に渡されていない限り、`delete-log-group` または
  `put-retention-policy` を呼び出さない（dry-run をデフォルト動作とする）。
- NEVER: 対話式マルチセレクトの初期状態を「全選択（all-selected）」にしない。初期状態は常に
  全チェックOFFとする。
- NEVER: `sts:get-caller-identity` によるアカウントID不一致を検知した際に、警告のみで処理を
  続行しない。不一致が判明した時点でそのアカウントに対する処理を即座に失敗させる。
- NEVER: CloudWatch Logs 以外のリソース種別（EC2、S3、EBS 等）への操作を v1 スコープに含めない。
- NEVER: 削除条件のルールベース自動判定（人手の選択を経ない自動削除）を実装しない。
- NEVER: `AssumeRole` で取得した一時クレデンシャル（`AccessKeyId` / `SecretAccessKey` /
  `SessionToken`）を、監査ログ・標準出力・エラーメッセージ・パニックメッセージのいずれにも出力しない。
  クレデンシャル構造体全体を `{:?}` でダンプするような実装を禁止する。
- NEVER: `unsafe` コードブロックを使用しない（クレートルートで `#![forbid(unsafe_code)]` を宣言する）。
- NEVER: `unwrap()` / `expect()` / `panic!` を本番コードパス（AWS API呼び出し・Identity検証・
  削除実行のパス）で使用しない。エラーは常に `Result` で呼び出し元に伝播させる
  （テストコードは対象外）。
- NEVER: `audit`（監査ログ閲覧）サブコマンドのハンドラに、`delete-log-group` / `put-retention-policy`
  を実行しうる型（`ExecutionEngine`、および書き込み系の `AuditWrite` 等）への依存を一切持たせない。
  `run_audit` ハンドラへ注入する依存は読み取り専用の型（例: `AuditRead`）のみに限定し、削除・
  retention変更操作や監査ログの書き込み・改変を行う経路にコンパイル時点で到達不能な構造とする。
  コードレビューのみによる担保は許容しない（型／依存性注入レベルでの構造的強制を必須とする）。
  （本 intent の人間インタビュー Q4 で project.md への昇格を確定。開発・DevSecOps 両エージェントの
  推奨に基づく。）

## 本 intent でのスコープ外・見送り事項

- サブコマンド化後、`scan` サブコマンド（読み取り専用）の経路から破壊的操作（`delete-log-group` /
  `put-retention-policy`）を呼び出せない構造にする、という論点についても検討したが、`scan` は
  そもそも選択・削除フローを持たないサブコマンド（`clean` のみが破壊的操作を担う）であるため、
  `audit` と同様の明示的な Forbidden 昇格は不要と判断した。既存の「`--execute` なしでは呼ばない」
  Forbidden、および `clean`/`scan`/`audit` という型（コマンドの選択）による分離自体が、この安全性を
  実質的に担保する。
- `config` サブコマンドは本 intent のスコープ外であり、本 intent の作業で追加しない（`intent` の
  スコープ定義で既に除外されている事項の確認であり、project.md へのハード制約としての追加は
  不要と判断した：スコープ管理は project.md の `## Scope Overrides` 相当であり、破壊的操作の
  安全性に関わるハード制約ではないため）。
