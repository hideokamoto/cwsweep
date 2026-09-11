**Collaborator:** aidlc-developer-agent

## Contribution

開発者視点（実装者として実際にこのコードを書き、モジュール分割・エラー型・CI設定を担う立場）から、
ドラフトのモジュール構造・エラーハンドリング・ファイル構成・コードスタイル観点を評価する。

### 命名規則（naming conventions）

- ドラフトは Rust 慣習（snake_case / CamelCase）に従うとしており妥当。追加提案として、本ツール固有の
  ドメイン語彙（`account`, `log_group`, `identity_check`, `dry_run` など）は型名・関数名で一貫させ、
  略語（`lg` など）を避けることを明文化したい。特に「スキャン時点の Identity 検証」と「削除直前の
  二重 Identity 検証」は別関数として命名を分離すべき（例: `verify_identity_pre_scan` /
  `verify_identity_pre_mutate` のように役割が名前から読み取れるようにする）。同一関数の使い回しは
  二重検証という構造的要件を曖昧にするリスクがある。

### レイヤー境界（module structure）

スコープ文書の依存関係（M1→M2→M3→M4→M5→M6）はそのままモジュール境界の候補になる。具体的に以下の
crate内モジュール分割を提案する:

- `identity`（AssumeRole・`sts:get-caller-identity` 検証。読み取り専用）
- `discovery`（`organizations:list-accounts` 列挙、`describe-log-groups` の全ページ集計）
- `ui`（対話式マルチセレクト、確認画面、table/json 出力）
- `actions`（`delete-log-group` / `put-retention-policy` の実行。**この module だけが破壊的AWS呼び出しを持つ**）
- `audit`（監査ログ出力。無効化オプションなし）

重要なのは、**「読み取り・集計系」と「書き込み・削除系」を crate 内で物理的に別モジュールに分離し、
`actions` モジュールの公開関数はすべて `--execute` フラグの真偽値と二重目の Identity 検証結果を
引数として要求するシグネチャにする**こと（型レベルで dry-run 抜けを防ぐ）。例えば
`fn delete_log_group(verified: VerifiedIdentity, target: LogGroupRef) -> Result<...>` のように、
「検証済みである」ことを表す型（newtype）を経由しないと `actions` の関数を呼べない設計にすれば、
検証をスキップした削除呼び出しはコンパイルが通らなくなる。これは Testing Posture ドラフトが提案する
「100% パスカバレッジ」の実装コストを下げる効果もある。

### エラーハンドリング（Result/Error 型、Identity 検証失敗パス）

- クレート共通の `enum CwsweepError`（`thiserror` 採用を推奨）を定義し、少なくとも以下のバリアントを
  区別する: `IdentityMismatch { expected: AccountId, actual: AccountId }` / `AssumeRoleFailed` /
  `PaginationIncomplete` / `AwsApiError`（SDK エラーのラップ）/ `AuditWriteFailed`。
  `IdentityMismatch` は「警告して続行」ではなく即座にそのアカウント処理を打ち切るという Forbidden 節の
  要件を型で表現するため、`Result` の `Err` として扱い、呼び出し側でアカウント単位のループを
  `continue`（他アカウントは処理継続）ではなく該当アカウントの結果に失敗を記録して次アカウントへ進む
  設計とする（1アカウントの不一致が全体を止めるのは過剰、しかし握りつぶしは禁止）。
- `AuditWriteFailed` は無効化オプションを持たない監査ログの書き込み失敗であり、これをどう扱うか
  （fail-fast で削除自体を中止するか、削除は成功として記録失敗のみ別途警告するか）はスコープ文書に
  明記がない。安全側に倒すなら「監査ログ書き込みに失敗したら、それ以降の削除アクションを打ち切る」
  べきだと考える。この扱いを Forbidden/Mandated に追記するかは要インタビュー事項として追加したい。
- ページネーション未完了（`PaginationIncomplete`）はエラーとして扱い、部分的な集計結果で対話式選択に
  進ませない（構造的事故の再発防止という PRD の目的に直結する）。

### ファイル構成

- 単一バイナリクレート（`src/main.rs` + 上記5モジュールを `src/*.rs` または `src/*/mod.rs`）で十分。
  単一開発者・v1スコープの規模感からワークスペース分割（複数crate化）は過剰と考える。
- 結合テストは `tests/` 配下に配置し、AWS SDK呼び出しは `aws-sdk-*` の `mock` 機能または
  トレイトによる抽象化（`OrganizationsClient` トレイトなど）でスタブ化する。実 AWS 呼び出しを伴う
  統合確認は自動テストと分離し、Deployment ドラフトが提案する「実 Organization 環境での動作確認
  チェックリスト」の一部として手動運用する。

### コードスタイル（rustfmt/clippy、unsafe方針）

- `cargo fmt --check` / `cargo clippy -- -D warnings` の CI 必須化はAGREE。ただし単一開発者プロジェクトで
  最初から `-D warnings` を全警告に適用すると、AWS SDK の生成コードに起因する誤検知や、開発初期の
  試行錯誤を阻害する可能性がある。実務上は `clippy::all` を `-D` にしつつ `clippy::pedantic` は
  `-W`（警告止まり）から始め、Bolt が進むにつれて段階的に締める運用を提案する。
- `unsafe` 原則禁止（`#![forbid(unsafe_code)]` をcrateルートに宣言）に賛成する。AWS SDK呼び出しと
  対話式CLIのみで完結するv1スコープにおいて `unsafe` を要する場面は想定されない。

## Positions

AGREE: トランクベース開発・短命フィーチャーブランチ・`main` へのマージ方針
AGREE: 単一開発者体制のためPRレビューは自己レビュー＋CI greenを必須条件とする
AGREE: Bolt ブランチの squash-merge、1 Bolt = 1 コミット方針
AGREE: Walking Skeleton をスコープファイルの `skeleton: on/off` 宣言に従わせる方針
AGREE: M6（`--execute` 実装）到達 Bolt には毎回ゲートを推奨する提案
AGREE: Testing Posture の test-after・層ごとの実装直後テスト作成という Ordering
AGREE: Identity検証失敗系・`--execute`未指定時の抑止・マルチセレクト初期状態への100%パスカバレッジ＋境界値テストの追加要件提案
OBJECT: `clippy -- -D warnings` を初回から全警告に一律適用する方針には条件付きで反対する。`clippy::all` は `-D` とし、`clippy::pedantic` 相当は段階的に締める運用（まずは `-W` から開始し、Bolt進行に応じて `-D` へ引き上げる）を代案として提案する。全警告一律 `-D` はAWS SDK生成コード起因の誤検知や単一開発者の初期イテレーション速度を不必要に阻害するリスクがある。
AGREE: `unsafe` コードは原則禁止(`#![forbid(unsafe_code)]`)とする方針
AGREE: タグ駆動リリース(GitHub Releasesへのクロスプラットフォームバイナリアップロード)方式
OBJECT: 監査ログ書き込み失敗時の扱い（削除処理自体をfail-fastで打ち切るか、削除は成功として記録失敗のみ警告するか）がドラフトに未記載である点は看過できない。「無効化オプションを設けない」という Mandated 要件だけでは書き込み失敗時の挙動が未定義のままであり、実装者としてはこの分岐を Mandated/Forbidden に明文化するか、少なくともインタビュー確認事項として追加することを求める。
