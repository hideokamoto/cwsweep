# Observability Design

## 構造化ログ

`tracing`クレートを使用する。各アカウント×リージョンの処理を`tracing::Span`で囲み、進捗を標準エラー出力（`tracing-subscriber`のfmt layer）に人間可読形式で出力する。

## 相関ID

実行1回ごとに`run_id`（UUID v4）を発行し、`AuditEntry`の各行に付与する。これにより1回の実行内の全操作を後から相関できる（NFR5.1のJSON Lines監査ログと連携）。

```json
{"run_id": "a1b2c3d4-...", "timestamp": "2026-09-10T12:00:00Z", "account_id": "123456789012", "region": "us-east-1", "log_group_name": "/aws/lambda/foo", "action_kind": "delete", "success": true}
```

## SLI/SLO

定義しない（NFR5.3/5.4の通り対象外。常駐サービスではないため）。

## アラート・ダッシュボード

対象外。

## Assumptions & Open Questions

None.
