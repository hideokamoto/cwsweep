# Performance Design

## 非同期処理・ページネーション

- `tokio`シングルランタイム上で、アカウント×リージョンの組み合わせを逐次`await`する（NFR1.2の逐次スロットリング尊重方針に従う）。
- `LogGroupScanner`は`aws-sdk-cloudwatchlogs`のページネーターAPI（`into_paginator()`）を用いて全ページ取得を完了してから`ScanAggregator`へ渡す。

```rust
// 疑似コード: ページネーション完全対応
let mut all_groups = Vec::new();
let mut pages = client.describe_log_groups().into_paginator().send();
while let Some(page) = pages.next().await {
    all_groups.extend(page?.log_groups.unwrap_or_default());
}
aggregator.ingest(account_id, region, all_groups); // 全ページ確定後にのみ渡す
```

## コネクションプーリング

独自実装はせず、`aws-config`が内部で使用するhyperクライアントのデフォルトのコネクションプーリングに委ねる。

## パフォーマンス予算

NFR1.1（標準規模2分以内、設計目標）に対応するため、アカウント×リージョンのループはCLI起動時に一度だけHTTPクライアントを初期化し、各呼び出しで再生成しない。

## Assumptions & Open Questions

None.
