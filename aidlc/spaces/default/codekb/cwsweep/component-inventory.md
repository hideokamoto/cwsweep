# component-inventory.md — cwsweep

12 コンポーネント（`src/lib.rs` の `pub mod` 宣言に対応、1モジュール1コンポーネント）
＋ 横断的な `error` モジュール ＋ 合成ルートである `main.rs`（コンポーネントではない）。

## CliApp（`src/cli.rs`）

- **責務**: CLI 引数解析（`Cli` 構造体、clap）とパイプライン全体のオーケストレーション。
  `scan_all` → `render_output` → `scan_fully_failed` → `select` → `plan` →
  `confirm` → `execute` の7メソッドが唯一の入口群。
- **依存**: `aggregator`, `confirmation`, `credentials`, `error`, `execution`,
  `identity`, `org_discovery`, `output`, `planner`, `scanner`, `selector`
  （ほぼ全モジュールに依存する唯一のハブ）。
- **本 intent での位置づけ**: サブコマンド化の直接対象（`Cli` 構造体自体と
  オーケストレーション呼び出し順序）。

## LogGroupScanner（`src/scanner.rs`）

- **責務**: `scan_account_region` — `DescribeLogGroupsOperations` trait 経由で
  CloudWatch Logs `describe-log-groups` をページ完了まで辿る。
- **依存**: `aggregator`（`LogGroupRecord` 生成）, `credentials`, `error`, `identity`。

## ScanAggregator（`src/aggregator.rs`）

- **責務**: スキャン結果の集約・集計。`add_all`, `records`, `is_empty`, `len`,
  `total_bytes`, `sorted_by_size_desc`, `filter_by_account_region`。
- **依存**: なし（葉ノード）。`output`, `selector`, `planner` から参照される
  単一の真実源。

## InteractiveSelector（`src/selector.rs`）

- **責務**: `select` — `MultiSelectPrompt` trait 経由の対話式マルチセレクト。
  初期選択は常に空配列（project.md Forbidden: 全選択初期状態の禁止）。
- **依存**: `aggregator`。

## ActionPlanner（`src/planner.rs`）

- **責務**: `plan` — 選択済みレコードから `ActionKind::Delete` /
  `ActionKind::SetRetention{days}` の `PlannedAction` を構築。retention 許容値検証込み。
- **依存**: `aggregator`（`LogGroupRecord` から構築）, `error`。

## ConfirmationPresenter（`src/confirmation.rs`）

- **責務**: `build_summary`, `confirm` — `ConfirmPrompt` trait 経由で、
  対象アカウントID・リージョン・ログループ名一覧・合計バイト数を再掲した
  確認画面を提示（project.md Mandated）。
- **依存**: `planner`。

## ExecutionEngine（`src/execution.rs`）

- **責務**: `execute_plan` — `ActionApiOperations` trait 経由で
  `delete-log-group` / `put-retention-policy` を実行。実行直前に独立した
  二重目 Identity 検証を行う（project.md Mandated）。
- **依存**: `audit`, `credentials`, `error`, `identity`, `planner`。

## CredentialProvider（`src/credentials.rs`）

- **責務**: `credentials_for` — `AssumeRoleOperations` trait 経由でメンバー
  アカウントの一時クレデンシャルを取得。管理アカウント自身へは AssumeRole せず
  現行クレデンシャルを使用（project.md Mandated の非対称設計）。
- **依存**: `error`。

## IdentityVerifier（`src/identity.rs`）

- **責務**: `verify_identity` / `IdentityCheck::verify` — `CallerIdentityOperations`
  trait 経由で STS `get-caller-identity` を実行し、期待アカウントIDとの一致を検証。
  不一致時は当該アカウント処理を即座に失敗させる（project.md Mandated/Forbidden）。
- **依存**: `credentials`, `error`。

## OrgDiscovery（`src/org_discovery.rs`）

- **責務**: `list_active_accounts` — `ListAccountsOperations` trait 経由で
  Organizations `list-accounts` をページネーション込みで実行し、`Status=ACTIVE`
  でフィルタする。
- **依存**: `error`。

## AuditLogger（`src/audit.rs`）

- **責務**: `append` — `AuditWrite` trait 経由で JSON Lines・fsync 付き監査ログを
  `Intent`/`Result` の二段階で記録。無効化オプションを設けない
  （project.md Mandated）。書き込み失敗時は当該操作を中断させる契約
  （`execution` 側の責務と連動）。
- **依存**: `error`, `planner`（`ActionKind` を監査エントリに埋め込む）。
- **既知の空白点**: 追記専用 API のみで読み取り API が存在しない。
  `audit` 閲覧サブコマンド新設時の検討事項（`code-quality-assessment.md` 参照）。

## OutputFormatter（`src/output.rs`）

- **責務**: `format` — table（`comfy-table`）/ JSON の出力切り替え。
- **依存**: `aggregator`。

## error（`src/error.rs`）

- **責務**: 共通エラー型（`thiserror`）。全モジュールから利用される横断的
  コンポーネントで、他モジュールには依存しない葉ノード。コンポーネント一覧としては
  非業務ロジック的な基盤モジュールとして扱う。

## main.rs（合成ルート、非コンポーネント）

- **責務**: `#[tokio::main]` エントリポイント。実 AWS SDK アダプタ
  （`StsAssumeRoleAdapter`, `StsCallerIdentityAdapter`,
  `OrganizationsListAccountsAdapter`, `CloudWatchLogsAdapter`,
  `InquireMultiSelectPrompt`, `InquireConfirmPrompt` 等）を実装し、
  `CliApp` に配線する。`action_kind_from_prompt`（delete/set-retention選択の
  対話プロンプト）もここに存在するが `inquire` への直接依存のため未テスト
  （`code-quality-assessment.md` 参照）。
