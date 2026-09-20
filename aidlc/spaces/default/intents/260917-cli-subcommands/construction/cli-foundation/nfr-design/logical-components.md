# logical-components.md — Unit: cli-foundation

| 論理コンポーネント | 配置 | 責務 | 障害ドメイン / 影響範囲 |
|---|---|---|---|
| `Cli` / `Commands`（clap） | lib `src/cli.rs` | 引数解析、サブコマンド・所属オプションの宣言 | 誤用はプロセス起動時に終了。AWS 未到達 |
| `run_scan` | lib `src/cli.rs` | スキャン → 全滅判定 → 出力 | 読み取り専用。監査ログ・破壊的 API に非到達 |
| `run_clean` | lib `src/cli.rs` | スキャン → TTY 判定 → 選択 → プラン → 確認 → 実行 | 破壊的操作は `--execute` かつ確認承認時のみ。監査ログ必須 |
| `run_audit` | lib `src/cli.rs` | `AuditRead` → `OutputFormatter` | ローカルファイル読み取りのみ。AWS 非到達 |
| `ExitDisposition` | lib `src/cli.rs` | 終了方針の表現 | — |
| `OutputFormatter::format_audit` | lib `src/output.rs` | audit の table / json 整形 | — |
| `AuditRead` / `AuditReader` | lib `src/audit.rs`（audit-reader Unit 所有） | 監査ログ読み取り | — |
| main 配線 | bin `src/main.rs` | 実 SDK アダプタ実体化、監査ログ open、TTY 判定、`match Commands` によるハンドラ呼び出し、終了コード写像 | 分岐ロジックを持たない |

## 分離戦略

- 破壊的操作系（`ExecutionEngine` / `ActionApiOperations` / `AuditWrite`）に到達できる
  論理コンポーネントは `run_clean` のみ。`run_scan` / `run_audit` はシグネチャ上
  それらを受け取らない（D-SEC-1）。
- 共有リソースは監査ログファイル（`clean` が書き、`audit` が読む）のみ。同時実行の
  排他は本intentの範囲外（既存 `AuditLogger` の追記方式を維持）。
