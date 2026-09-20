# api-documentation.md — cwsweep

`cwsweep` は Web API を持たない CLI ツールである。ここでの「API」は
(1) 外部に公開された CLI フラグ面（clap）、(2) 内部コンポーネント境界（trait
経由でDI可能な内部API）、(3) 実際に呼び出す AWS 外部 API、の3層を指す。

## 1. 外部 API — CLI フラグ面（`src/cli.rs` `struct Cli`, clap）

現状はサブコマンドを持たない単一フラットコマンド（本 intent の再構成対象）。

| フラグ | 必須/既定 | 説明 |
|---|---|---|
| `--regions <REGIONS>` | 必須、複数指定可（カンマ区切り、重複除去） | スキャン対象リージョン。全リージョン自動列挙のフォールバックなし |
| `--role-name <ROLE_NAME>` | 既定 `OrganizationAccountAccessRole` | メンバーアカウントへの AssumeRole 先ロール名 |
| `--output <table\|json>` | 既定 `table` | 出力フォーマット |
| `--execute` | 既定 `false` | 実際に削除・retention変更を実行（明示しない限り絶対に実行しない） |
| `--scan-only` | 既定 `false`、`--execute` と `conflicts_with` | スキャン結果を表示して終了（選択・確認・実行に進まない） |
| `--audit-log-path <PATH>` | 既定 `cwsweep-audit.jsonl` | 監査ログ出力先。無効化オプションは存在しない（project.md Mandated） |
| `--version` / `--help` | clap 自動生成 | — |

**非TTYフォールバック**: 標準入力が TTY でない場合（パイプ・CI等）、フラグ無しでも
`--scan-only` 相当の挙動になる（`src/main.rs` `!std::io::stdin().is_terminal()`）。

**終了コード契約（CI/自動化向け、README.md記載）**: 対象 account×region の
組み合わせが1件以上あり、かつ全件でスキャンが失敗した場合のみ非ゼロ終了。
1件でも成功していれば失敗分は警告ログのみで終了コード0。

## 2. 内部コンポーネント境界（trait 経由でDI可能な内部API）

各コンポーネントの公開メソッドと、それが依存する外部境界 trait。実装詳細・
依存関係グラフは `architecture.md`（Component Relationships）と
`component-inventory.md` を参照。

| コンポーネント | 公開メソッド | 外部境界 trait |
|---|---|---|
| `CliApp`（`cli.rs`） | `scan_all`, `render_output`, `scan_fully_failed`, `select`, `plan`, `confirm`, `execute` | — （オーケストレーション、下位コンポーネントを合成） |
| `LogGroupScanner`（`scanner.rs`） | `scan_account_region` | `DescribeLogGroupsOperations` |
| `ScanAggregator`（`aggregator.rs`） | `add_all`, `records`, `is_empty`, `len`, `total_bytes`, `sorted_by_size_desc`, `filter_by_account_region` | — |
| `InteractiveSelector`（`selector.rs`） | `select` | `MultiSelectPrompt` |
| `ActionPlanner`（`planner.rs`） | `plan` | — |
| `ConfirmationPresenter`（`confirmation.rs`） | `build_summary`, `confirm` | `ConfirmPrompt` |
| `ExecutionEngine`（`execution.rs`） | `execute_plan` | `ActionApiOperations` |
| `CredentialProvider`（`credentials.rs`） | `credentials_for` | `AssumeRoleOperations` |
| `IdentityVerifier`（`identity.rs`） | `verify_identity` / `IdentityCheck::verify` | `CallerIdentityOperations` |
| `OrgDiscovery`（`org_discovery.rs`） | `list_active_accounts` | `ListAccountsOperations` |
| `AuditLogger`（`audit.rs`） | `append` | `AuditWrite` |
| `OutputFormatter`（`output.rs`） | `format` | — |

## 3. 外部 AWS API（`src/main.rs` のアダプタ経由で呼び出す実サービス呼び出し）

| サービス | API | アダプタ | 備考 |
|---|---|---|---|
| STS | `AssumeRole` | `StsAssumeRoleAdapter` | メンバーアカウントのみ。管理アカウント自身へは呼ばない |
| STS | `GetCallerIdentity` | `StsCallerIdentityAdapter` | スキャン時＋実行直前の二重検証。リージョン `us-east-1` 固定（グローバルサービス） |
| Organizations | `ListAccounts` | `OrganizationsListAccountsAdapter` | ページネーション完全対応、`Status=ACTIVE` フィルタ |
| CloudWatch Logs | `DescribeLogGroups` | `CloudWatchLogsAdapter` | ページネーション完走後に集計（1ページのみでの確定は不可） |
| CloudWatch Logs | `DeleteLogGroup` | `CloudWatchLogsAdapter` | `--execute` 時のみ |
| CloudWatch Logs | `PutRetentionPolicy` | `CloudWatchLogsAdapter` | `--execute` 時のみ |

## 契約・失敗時の挙動

- Identity 不一致（`sts:get-caller-identity` の結果が期待アカウントIDと不一致）は
  警告に留めず、当該アカウントの処理を即座に失敗させる。
- 監査ログ（`AuditLogger::append`）書き込み失敗時は、実行中の削除・retention変更
  操作自体を中断する（警告のみで続行しない）。
- `DescribeLogGroups` は全ページ取得後に集計を確定する（1ページのみでの確定禁止）。

## 本 intent との関係

現状の「1コマンド＋相互排他フラグ」面が `scan` / `clean`（／将来的な `audit` 閲覧）
サブコマンドへ再構成される想定。内部コンポーネント境界（上記2.の trait 群）は
サブコマンド化の影響を受けない設計上の安定面であり、変更の中心は
外部 API（1.の `Cli` 構造体）と `CliApp` によるオーケストレーション呼び出し順序。
