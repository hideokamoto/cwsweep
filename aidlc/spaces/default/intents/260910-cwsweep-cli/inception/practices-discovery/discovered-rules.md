# 発見されたルール（確定版）

> **ステータス: 確定（affirmed）** — PRD/スコープ文書で構造的要件として決定済みの事項に加え、
> DevSecOps エージェントが指摘した認証情報非ログ出力・依存脆弱性スキャンのギャップ、および
> 人間インタビュー（Q7）で確定した監査ログ書き込み失敗時の挙動を統合した最終版です。

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
