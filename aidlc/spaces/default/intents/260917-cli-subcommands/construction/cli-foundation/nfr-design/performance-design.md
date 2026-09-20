# performance-design.md — Unit: cli-foundation

- **D-PERF-1（NFR5.1）**: `Cli::parse_args` は `Parser::parse` 1 回＋`dedup_regions`
  のみ。ディスパッチは `match Commands` の単一分岐。I/O を伴う初期化（AWS 設定
  ロード、監査ログ open）はサブコマンド確定後に main で行う。
- **D-PERF-2（NFR5.2）**: `run_scan` / `run_clean` は既存 `scan_all` を共有ヘルパとして
  呼び出す。API 呼び出し経路・回数は変更しない。
- **D-PERF-3（NFR5.3）**: `OutputFormatter::format_audit` は `entries` を 1 回走査して
  table 行または JSON 配列を構築する。ソート・再走査は行わない（記録順を保持、
  audit-reader BR5 と整合）。
