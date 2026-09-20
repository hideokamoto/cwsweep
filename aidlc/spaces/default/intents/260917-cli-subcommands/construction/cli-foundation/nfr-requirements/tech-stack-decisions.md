# tech-stack-decisions.md — Unit: cli-foundation

すべて既存依存の範囲内で実装し、新規クレートは追加しない
（`codekb/cwsweep/technology-stack.md`、`Cargo.lock` 固定）。

| 領域 | 選択 | 根拠 |
|---|---|---|
| 引数解析 | `clap` 4（derive、`Subcommand` derive を追加利用） | 既存依存。`#[command(subcommand)]` で FR1.1 の未指定エラー、バリアント別 `#[arg]` で FR2.2 / FR3.2 の所属制御を宣言的に実現できる |
| 出力整形 | `comfy_table` + `serde_json`（既存 `OutputFormatter` を拡張） | Contract 2「既存 table と同じ書式ルール」を満たす最短経路 |
| 監査ログ読み取り | audit-reader Unit の `AuditRead` / `AuditReader`（`src/audit.rs`） | Contract 1。本Unitは消費のみ |
| 非同期ランタイム | `tokio`（既存） | `run_scan` / `run_clean` は既存 `scan_all` / `execute` の async 経路を再利用 |
| 対話 UI | `inquire`（既存、main 側アダプタ） | `clean` のみ。lib 側は `MultiSelectPrompt` / `ConfirmPrompt` trait 経由で非依存 |
| TTY 判定 | `std::io::IsTerminal`（既存利用） | main で判定し `run_clean` へ真偽値として渡す（lib のテスト容易性 NFR1） |
| エラー型 | `thiserror`（既存）＋ `Box<dyn Error>` 境界は main のみ | 現行方針維持 |
| テスト | 既存モックアダプタ（`scan_all` テスト群）、`clap::Parser::try_parse_from` | NFR1 / NFR4 の lib 単体テスト |

## 明示的に採用しないもの

- 旧フラグ互換のための `clap` alias / hidden arg（NFR4 / FR6.1 違反）。
- `config` サブコマンド（本intent対象外）。
