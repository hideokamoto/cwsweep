# code-structure.md — cwsweep

## パッケージ構成

Cargo 単一パッケージ `cwsweep`（`Cargo.toml`）に、バイナリクレートとライブラリ
クレートを同居させる構成。

```
cwsweep/
├── Cargo.toml           # [[bin]] cwsweep (src/main.rs) + [lib] cwsweep (src/lib.rs)
├── Cargo.lock           # コミット済み、CIは --locked
├── rust-toolchain.toml  # channel = "1.98.1", rustfmt/clippy固定
├── deny.toml            # cargo deny (advisories/licenses/bans)
├── README.md            # 使い方・スコープ制約・CI向け終了コード契約
├── .circleci/config.yml # fmt/clippy/test/coverage/audit/deny/release workflows
├── src/
│   ├── main.rs          # バイナリエントリポイント。実AWS SDKアダプタの配線
│   ├── lib.rs           # ライブラリクレートルート（12 pub mod 宣言）
│   ├── cli.rs           # CliApp: clap Cli構造体 + オーケストレーション
│   ├── scanner.rs        # LogGroupScanner
│   ├── aggregator.rs     # ScanAggregator
│   ├── selector.rs       # InteractiveSelector
│   ├── planner.rs        # ActionPlanner
│   ├── confirmation.rs   # ConfirmationPresenter
│   ├── execution.rs      # ExecutionEngine
│   ├── audit.rs          # AuditLogger
│   ├── credentials.rs    # CredentialProvider
│   ├── identity.rs       # IdentityVerifier
│   ├── org_discovery.rs  # OrgDiscovery
│   ├── output.rs         # OutputFormatter
│   └── error.rs          # 共通エラー型 (thiserror)
└── tests/
    ├── scan_select_execute.rs   # 統合テスト: スキャン→選択→実行の一連フロー
    └── audit_log_format.rs     # 統合テスト: 監査ログのフォーマット検証
```

## モジュール分類（1モジュール1コンポーネント）

Domain Design の 12 コンポーネントと 1:1 対応する。各ファイルの責務は
`component-inventory.md` を参照（重複記載を避けるためここでは一覧のみ）:

| ファイル | コンポーネント |
|---|---|
| `src/cli.rs` | `CliApp` |
| `src/scanner.rs` | `LogGroupScanner` |
| `src/aggregator.rs` | `ScanAggregator` |
| `src/selector.rs` | `InteractiveSelector` |
| `src/planner.rs` | `ActionPlanner` |
| `src/confirmation.rs` | `ConfirmationPresenter` |
| `src/execution.rs` | `ExecutionEngine` |
| `src/credentials.rs` | `CredentialProvider` |
| `src/identity.rs` | `IdentityVerifier` |
| `src/org_discovery.rs` | `OrgDiscovery` |
| `src/audit.rs` | `AuditLogger` |
| `src/output.rs` | `OutputFormatter` |

`src/error.rs` は全モジュール共通のエラー型（`thiserror`）を提供する非コンポーネント
的な横断モジュール。`src/main.rs` はコンポーネントではなく、実 AWS SDK アダプタ
（`StsAssumeRoleAdapter`、`StsCallerIdentityAdapter`、
`OrganizationsListAccountsAdapter`、`CloudWatchLogsAdapter`、
`InquireMultiSelectPrompt`、`InquireConfirmPrompt` 等）を実装し配線する合成ルート。

## テストの配置パターン

- 全 12 ソースモジュールが同一ファイル内に `#[cfg(test)] mod tests` を持つ
  （ユニットテスト、各コンポーネント境界 trait を手書きスタブ/フェイクで直接実装）。
- `tests/` 配下に統合テストが2ファイル（`scan_select_execute.rs`＝一連の業務
  フロー横断シナリオ、`audit_log_format.rs`＝監査ログの出力内容専用検証）。

## コードパターン

- **ports-and-adapters**: 各コンポーネントの外部依存は `async_trait` の trait
  として定義され（例: `DescribeLogGroupsOperations`、`AuditWrite`）、本番実装は
  `main.rs` のアダプタが持つ。テストはこの trait をスタブ実装で差し替える。
- **エラー伝播**: `thiserror` ベースの共通エラー型を `error.rs` に集約し、
  `Result` で呼び出し元へ伝播。`unwrap`/`expect`/`panic!` は本番コードパスから
  `#![deny(...)]` により構造的に排除（`lib.rs`・`main.rs` 冒頭、テストコードのみ除外）。
- **クレデンシャルマスキング**: `secrecy` により一時クレデンシャルの `Debug`
  実装を `[REDACTED]` 化。
- **監査の二段階記録**: `AuditLogger::append` は `Intent`（実行意図）と
  `Result`（実行結果）を分けて記録する構造。

## エントリポイント／フロー分岐（本 intent の直接対象）

- `src/cli.rs` `struct Cli`: `--regions`（必須）、`--role-name`
  （既定 `OrganizationAccountAccessRole`）、`--output`（`table`|`json`）、
  `--execute`、`--scan-only`（`conflicts_with = "execute"`）、`--audit-log-path`。
- `src/main.rs` `main()`: `cli.scan_only` 判定 →
  `!std::io::stdin().is_terminal()` による非TTYフォールバック（scan-only相当）→
  対話式選択・確認・実行、という条件分岐がフラット構造の核。本 intent の
  サブコマンド化ではこの2箇所の再配置が焦点になる。
