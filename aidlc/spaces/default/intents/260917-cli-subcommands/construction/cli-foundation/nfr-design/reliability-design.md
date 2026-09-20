# reliability-design.md — Unit: cli-foundation

- **D-REL-1（NFR5.10）**: 終了方針は `ExitDisposition` enum（`Success` /
  `ScanFullyFailed { count }` / `Error(message)`）として lib 側で決定し、main は
  `Success → Ok(())`、それ以外 → `Err(message)` へ機械的に写像する。判定ロジック
  （`scan_fully_failed`）は既存関数を再利用。
- **D-REL-2（NFR5.11）**: 引数誤用は `Cli::parse_args` で `clap::Error::exit()` に
  よりプロセス終了。AWS 設定ロードより前に位置するためネットワーク到達なし。
- **D-REL-3（NFR5.12）**: main の `Clean` 分岐で監査ログ open を最初に実行し、
  `?` で即時伝播（フェイルクローズ）。
- **D-REL-4（NFR5.13）**: `run_audit` は `reader.entries()` の `Err(AuditReadError)` を
  `ExitDisposition::Error` へ変換。ファイル不在は audit-reader 側で `Ok(空)` のため
  自然に正常終了。
- **D-REL-5（NFR5.14）**: `stdin_is_tty: bool` を `run_clean` の引数として main から
  注入する。lib テストで `false` を渡し、選択・確認・実行に進まず `Success` を返す
  ことを検証可能にする（nfr-requirements レビュー R-01 への対応）。
- **D-REL-6**: 資格情報解決失敗のアクション単位分離は既存 `CliApp::execute` を再利用。
