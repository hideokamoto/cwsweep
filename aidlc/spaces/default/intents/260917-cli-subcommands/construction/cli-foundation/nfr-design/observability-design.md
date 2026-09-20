# observability-design.md — Unit: cli-foundation

- **D-OBS-1（NFR5.15）**: ハンドラは人間向け文言を標準出力へ書く際 `output == Table`
  を条件とする。`Json` では `OutputFormatter` の戻り値のみを標準出力に書く。
  `tracing` サブスクライバは既存どおり標準エラー。
- **D-OBS-2（NFR5.16）**: `scan_all` の `Failed` 結果ごとの `tracing::warn!`
  （`account_id`, `region`, `%error`）を `run_scan` / `run_clean` 共通ヘルパに置く。
- **D-OBS-3（NFR5.17）**: `AuditLogger` の書式・intent/result 2 行記録は不変。
  `audit` サブコマンドは読み取りのみ。
- **D-OBS-4（NFR5.18）**: 非 TTY の `clean` は `tracing::warn!` で「対話式選択に進めない。
  棚卸しのみなら `scan` を使用」と出力。監査ログ不正行の警告は audit-reader が
  読み取り時に出力済みのため `run_audit` は再出力しない。
