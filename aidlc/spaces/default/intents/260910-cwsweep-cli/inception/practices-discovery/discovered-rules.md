# 発見されたルール（ドラフト）

> **ステータス: ドラフト（未確定）** — インタビュー実施前の草案。ここに挙げるのは、
> PRD/スコープ文書（`scope-document.md`）で既に構造的要件として決定済みの事項のみ。
> チームの合意形成が必要な事項（テスト手法の選択など）はここには含めない。

## Mandated

- ALWAYS: いずれかのメンバーアカウントに対する AWS API 呼び出し（スキャン・削除・retention変更のいずれも含む）の直前に `sts:get-caller-identity` を実行し、想定アカウントIDと一致することを検証する。
- ALWAYS: `delete-log-group` または `put-retention-policy` を実行する直前に、スキャン時点とは独立した二重目の Identity 検証を再実行する。
- ALWAYS: `cloudwatch-logs:describe-log-groups` はページネーションを最後まで辿り、全ページ取得後に集計する。1ページのみで集計を確定する実装を許容しない。
- ALWAYS: 管理アカウント自身に対しては `AssumeRole` を行わず、現在の認証情報をそのまま使用する。メンバーアカウントに対してのみ `AssumeRole`（デフォルトロール名 `OrganizationAccountAccessRole`、引数で上書き可能）を行う。
- ALWAYS: 削除・retention変更を伴うすべての操作について、対象アカウントID・リージョン・ログループ名・実行時刻・成功/失敗を監査ログに出力する。無効化オプションは設けない。
- ALWAYS: 削除・retention変更の実行前に、対象アカウントID・リージョン・ログループ名一覧・合計バイト数を再掲した確認画面を提示する。

## Forbidden

- NEVER: `--execute` フラグが明示的に渡されていない限り、`delete-log-group` または `put-retention-policy` を呼び出さない（dry-run をデフォルト動作とする）。
- NEVER: 対話式マルチセレクトの初期状態を「全選択（all-selected）」にしない。初期状態は常に全チェックOFFとする。
- NEVER: `sts:get-caller-identity` によるアカウントID不一致を検知した際に、警告のみで処理を続行しない。不一致が判明した時点でそのアカウントに対する処理を即座に失敗させる。
- NEVER: CloudWatch Logs 以外のリソース種別（EC2、S3、EBS 等）への操作をv1スコープに含めない。
- NEVER: 削除条件のルールベース自動判定（人手の選択を経ない自動削除）を実装しない。

## 未決事項（インタビューで確認予定 — Mandated/Forbidden には未反映）

- テスト手法（TDD / test-after）そのものの選択
- リリース配布チャネル・バージョニング規則の詳細
- PR起票の要否、`clippy` 警告レベルの厳格さ、`unsafe` 使用可否
