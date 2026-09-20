# Performance Test Instructions — 260917-cli-subcommands

## 方針

Test Strategy は **Standard**。cli-foundation の `performance-requirements.md` は、サブコマンド化がスキャン経路の呼び出し回数・ページネーション挙動を変更しない（既存 `LogGroupScanner` をそのまま再利用）ことを要件とし、自動ベンチマークは要求していない。したがって本stageで負荷テストは実施せず、以下の回帰確認のみを行う。

## 実行コマンド

```bash
cargo test --locked --lib scanner::      # 全ページ逐次取得（FR5.3）の回帰
cargo test --locked --lib cli::tests::run_scan_   # run_scan がスキャンを1回だけ行い追加のAPI呼び出しを発生させない
```

## 期待値

- `describe-log-groups` 呼び出し回数がページ数と一致（既存テストで検証済み）。
- `run_scan` / `run_clean` はスキャンを重複実行しない（同一の `scan_all` を1回呼ぶ）。

## 実Org環境での計測

対象外（初期リリースでのSLAなし）。必要になった場合は `time cwsweep scan --regions <R> --output json > /dev/null` を計測手順とする。
