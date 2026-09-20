# Code Generation — Unit: cli-foundation — 質問

## Q1. ScanDeps の実現方法（nfr-design レビュー R-01）

[Answer]: 新規 `ScanApp` 構造体（credential_provider / identity / logs_client）として実装し、`CliApp::scan_all` は `ScanApp` へ委譲する。既存 `CliApp` の公開フィールドとテストは変更しない。

## Q2. ハンドラの標準出力

[Answer]: `&mut dyn std::io::Write` を注入し、テストで `Vec<u8>` に捕捉する（`--output json` が単一 JSON 文書であることを検証可能にする）。
