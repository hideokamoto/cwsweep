# Project-Level Rules

> Project-specific specialisation and corrections. Loaded after `org.md` and
> `team.md` as strict-additive guidance; contradictions with broader policy
> are rejected. Populated by practices-discovery and the self-learning loop.
>
> Use sparingly: most teams don't need a project layer. Reach for it
> only when this specific project needs stable, durable guidance beyond the
> team practice (for example, package-specific release checks or an additional
> regression suite for a legacy component).

## Way of Working

<!-- Project-specific specialisation. Example: -->
<!-- This monorepo requires package-scoped branch names and a package owner -->
<!-- review in addition to the team's normal merge policy. -->

## Walking Skeleton

<!-- Project-specific specialisation. Example: -->
<!-- The walking skeleton must exercise the legacy service adapter as well -->
<!-- as the new service boundary. -->

## Testing Posture

<!-- Project-specific specialisation. -->

## Change Control

<!-- Project-specific. Mode: strict or relaxed. Strict here holds for every intent and cannot be changed from chat. -->

## Deployment

<!-- Project-specific specialisation. -->

## Code Style

<!-- Project-specific specialisation. -->

## Tech Stack

<!-- Technology choices locked for this project. -->

## Decided

<!-- Decisions made in earlier stages that should not be re-asked. -->
<!-- Format: DECIDED: [decision] (Stage [slug], [date]) -->

## Scope Overrides

<!-- Custom scope rules for this project. -->

## Forbidden

<!-- Populated by practices-discovery affirmation gate. -->
<!-- Format: NEVER [behavior] (affirmed [date]) -->
<!-- Example: NEVER throw exceptions across service layer boundaries (affirmed 2026-05-17) -->

- NEVER: `--execute` フラグが明示的に渡されていない限り、`delete-log-group` または (affirmed 2026-09-10)

`put-retention-policy` を呼び出さない（dry-run をデフォルト動作とする）。 (affirmed 2026-09-10)

- NEVER: 対話式マルチセレクトの初期状態を「全選択（all-selected）」にしない。初期状態は常に (affirmed 2026-09-10)

全チェックOFFとする。 (affirmed 2026-09-10)

- NEVER: `sts:get-caller-identity` によるアカウントID不一致を検知した際に、警告のみで処理を (affirmed 2026-09-10)

続行しない。不一致が判明した時点でそのアカウントに対する処理を即座に失敗させる。 (affirmed 2026-09-10)

- NEVER: CloudWatch Logs 以外のリソース種別（EC2、S3、EBS 等）への操作を v1 スコープに含めない。 (affirmed 2026-09-10)

- NEVER: 削除条件のルールベース自動判定（人手の選択を経ない自動削除）を実装しない。 (affirmed 2026-09-10)

- NEVER: `AssumeRole` で取得した一時クレデンシャル（`AccessKeyId` / `SecretAccessKey` / (affirmed 2026-09-10)

`SessionToken`）を、監査ログ・標準出力・エラーメッセージ・パニックメッセージのいずれにも出力しない。 (affirmed 2026-09-10)

クレデンシャル構造体全体を `{:?}` でダンプするような実装を禁止する。 (affirmed 2026-09-10)

- NEVER: `unsafe` コードブロックを使用しない（クレートルートで `#![forbid(unsafe_code)]` を宣言する）。 (affirmed 2026-09-10)

- NEVER: `unwrap()` / `expect()` / `panic!` を本番コードパス（AWS API呼び出し・Identity検証・ (affirmed 2026-09-10)

削除実行のパス）で使用しない。エラーは常に `Result` で呼び出し元に伝播させる (affirmed 2026-09-10)

（テストコードは対象外）。 (affirmed 2026-09-10)

- NEVER: `--execute` フラグが明示的に渡されていない限り、`delete-log-group` または (affirmed 2026-09-17)

`put-retention-policy` を呼び出さない（dry-run をデフォルト動作とする）。 (affirmed 2026-09-17)

- NEVER: 対話式マルチセレクトの初期状態を「全選択（all-selected）」にしない。初期状態は常に (affirmed 2026-09-17)

全チェックOFFとする。 (affirmed 2026-09-17)

- NEVER: `sts:get-caller-identity` によるアカウントID不一致を検知した際に、警告のみで処理を (affirmed 2026-09-17)

続行しない。不一致が判明した時点でそのアカウントに対する処理を即座に失敗させる。 (affirmed 2026-09-17)

- NEVER: CloudWatch Logs 以外のリソース種別（EC2、S3、EBS 等）への操作を v1 スコープに含めない。 (affirmed 2026-09-17)

- NEVER: 削除条件のルールベース自動判定（人手の選択を経ない自動削除）を実装しない。 (affirmed 2026-09-17)

- NEVER: `AssumeRole` で取得した一時クレデンシャル（`AccessKeyId` / `SecretAccessKey` / (affirmed 2026-09-17)

`SessionToken`）を、監査ログ・標準出力・エラーメッセージ・パニックメッセージのいずれにも出力しない。 (affirmed 2026-09-17)

クレデンシャル構造体全体を `{:?}` でダンプするような実装を禁止する。 (affirmed 2026-09-17)

- NEVER: `unsafe` コードブロックを使用しない（クレートルートで `#![forbid(unsafe_code)]` を宣言する）。 (affirmed 2026-09-17)

- NEVER: `unwrap()` / `expect()` / `panic!` を本番コードパス（AWS API呼び出し・Identity検証・ (affirmed 2026-09-17)

削除実行のパス）で使用しない。エラーは常に `Result` で呼び出し元に伝播させる (affirmed 2026-09-17)

（テストコードは対象外）。 (affirmed 2026-09-17)

- NEVER: `audit`（監査ログ閲覧）サブコマンドのハンドラに、`delete-log-group` / `put-retention-policy` (affirmed 2026-09-17)

を実行しうる型（`ExecutionEngine`、および書き込み系の `AuditWrite` 等）への依存を一切持たせない。 (affirmed 2026-09-17)

`run_audit` ハンドラへ注入する依存は読み取り専用の型（例: `AuditRead`）のみに限定し、削除・ (affirmed 2026-09-17)

retention変更操作や監査ログの書き込み・改変を行う経路にコンパイル時点で到達不能な構造とする。 (affirmed 2026-09-17)

コードレビューのみによる担保は許容しない（型／依存性注入レベルでの構造的強制を必須とする）。 (affirmed 2026-09-17)

（本 intent の人間インタビュー Q4 で project.md への昇格を確定。開発・DevSecOps 両エージェントの (affirmed 2026-09-17)

推奨に基づく。） (affirmed 2026-09-17)

## Mandated

<!-- Populated by practices-discovery affirmation gate. -->
<!-- Format: ALWAYS [behavior] (affirmed [date]) -->
<!-- Example: ALWAYS use Result<T,E> for fallible operations in service layer (affirmed 2026-05-17) -->

- ALWAYS: いずれかのメンバーアカウントに対する AWS API 呼び出し（スキャン・削除・retention変更の (affirmed 2026-09-10)

いずれも含む）の直前に `sts:get-caller-identity` を実行し、想定アカウントIDと一致することを検証する。 (affirmed 2026-09-10)

- ALWAYS: `delete-log-group` または `put-retention-policy` を実行する直前に、スキャン時点とは独立した (affirmed 2026-09-10)

二重目の Identity 検証を再実行する。 (affirmed 2026-09-10)

- ALWAYS: `cloudwatch-logs:describe-log-groups` はページネーションを最後まで辿り、全ページ取得後に (affirmed 2026-09-10)

集計する。1ページのみで集計を確定する実装を許容しない。 (affirmed 2026-09-10)

- ALWAYS: 管理アカウント自身に対しては `AssumeRole` を行わず、現在の認証情報をそのまま使用する。 (affirmed 2026-09-10)

メンバーアカウントに対してのみ `AssumeRole`（デフォルトロール名 `OrganizationAccountAccessRole`、 (affirmed 2026-09-10)

引数で上書き可能）を行う。 (affirmed 2026-09-10)

- ALWAYS: 削除・retention変更を伴うすべての操作について、対象アカウントID・リージョン・ログループ名・ (affirmed 2026-09-10)

実行時刻・成功/失敗を監査ログに出力する。無効化オプションは設けない。 (affirmed 2026-09-10)

- ALWAYS: 削除・retention変更の実行前に、対象アカウントID・リージョン・ログループ名一覧・合計バイト数を (affirmed 2026-09-10)

再掲した確認画面を提示する。 (affirmed 2026-09-10)

- ALWAYS: 監査ログへの書き込みに失敗した場合、実行中の削除・retention変更操作自体を中断し、 (affirmed 2026-09-10)

エラーとして扱う。ログが残せない操作は実行しない（警告のみで続行することは許容しない）。 (affirmed 2026-09-10)

- ALWAYS: 依存クレートの脆弱性スキャン（`cargo audit`）を CI で実行し、既知脆弱性のある依存があれば (affirmed 2026-09-10)

マージをブロックする。あわせて `cargo deny check`（advisories / licenses / bans / sources）を (affirmed 2026-09-10)

CI 必須ゲートとし、`Cargo.lock` をコミットして `--locked` フラグでビルド・テストする。 (affirmed 2026-09-10)

- ALWAYS: いずれかのメンバーアカウントに対する AWS API 呼び出し（スキャン・削除・retention変更の (affirmed 2026-09-17)

いずれも含む）の直前に `sts:get-caller-identity` を実行し、想定アカウントIDと一致することを検証する。 (affirmed 2026-09-17)

- ALWAYS: `delete-log-group` または `put-retention-policy` を実行する直前に、スキャン時点とは独立した (affirmed 2026-09-17)

二重目の Identity 検証を再実行する。 (affirmed 2026-09-17)

- ALWAYS: `cloudwatch-logs:describe-log-groups` はページネーションを最後まで辿り、全ページ取得後に (affirmed 2026-09-17)

集計する。1ページのみで集計を確定する実装を許容しない。 (affirmed 2026-09-17)

- ALWAYS: 管理アカウント自身に対しては `AssumeRole` を行わず、現在の認証情報をそのまま使用する。 (affirmed 2026-09-17)

メンバーアカウントに対してのみ `AssumeRole`（デフォルトロール名 `OrganizationAccountAccessRole`、 (affirmed 2026-09-17)

引数で上書き可能）を行う。 (affirmed 2026-09-17)

- ALWAYS: 削除・retention変更を伴うすべての操作について、対象アカウントID・リージョン・ログループ名・ (affirmed 2026-09-17)

実行時刻・成功/失敗を監査ログに出力する。無効化オプションは設けない。 (affirmed 2026-09-17)

- ALWAYS: 削除・retention変更の実行前に、対象アカウントID・リージョン・ログループ名一覧・合計バイト数を (affirmed 2026-09-17)

再掲した確認画面を提示する。 (affirmed 2026-09-17)

- ALWAYS: 監査ログへの書き込みに失敗した場合、実行中の削除・retention変更操作自体を中断し、 (affirmed 2026-09-17)

エラーとして扱う。ログが残せない操作は実行しない（警告のみで続行することは許容しない）。 (affirmed 2026-09-17)

- ALWAYS: 依存クレートの脆弱性スキャン（`cargo audit`）を CI で実行し、既知脆弱性のある依存があれば (affirmed 2026-09-17)

マージをブロックする。あわせて `cargo deny check`（advisories / licenses / bans / sources）を (affirmed 2026-09-17)

CI 必須ゲートとし、`Cargo.lock` をコミットして `--locked` フラグでビルド・テストする。 (affirmed 2026-09-17)

（本 intent での確認結果：サブコマンド化後も、上記の Identity 二重検証・監査ログ必須・dry-run既定・ (affirmed 2026-09-17)

確認画面再掲の各制約は、実行系操作を担う `clean` サブコマンドの経路に読み替えて維持される。これは (affirmed 2026-09-17)

既存 Mandated の適用対象をサブコマンド構造に読み替えるものであり、内容自体の変更ではない。新規 (affirmed 2026-09-17)

昇格ではないため、上記リストへの追記は行わない。） (affirmed 2026-09-17)

## Corrections

<!-- Project-specific corrections from human feedback. -->
<!-- Format: NEVER/ALWAYS [behavior] (learned [date]) -->
- advisoryレビュー(review_class: advisory)はステージあたり1回のみ実施される。Request Changes後の修正を再レビューするには通常の `aidlc engine log review --iteration N+1` は受理されず、redo jump（`aidlc engine jump execute --target <stage> --direction redo`）でステージをクリーンに再入場し、Consolidated Summary Confirmationを再確認してから新規iteration 1としてレビューを再ディスパッチする必要がある。 (learned 2026-09-17) <!-- cid:260917-cli-subcommands:units-generation:c11d2f41fbd1d6fc790c51bbe76a33d21cde752c154de8f2fdac2bd30ee8cf42 -->
