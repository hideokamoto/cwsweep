# Reliability Design

## リトライポリシー（NFR1.4対応）

すべてのAWS API呼び出しは`aws-config`の`RetryConfig::standard().with_max_attempts(3)`でラップする。指数バックオフはSDKデフォルトに従う。

## バルクヘッド分離（NFR4.2対応）

1アカウントの処理は独立した`Result<AccountScanOutcome, AccountError>`として扱う。`CliApp`はアカウント単位にループし、個々のエラーを収集して継続する。

```rust
// 疑似コード
let mut outcomes = Vec::new();
for account in accounts {
    match scan_account(&account).await {
        Ok(records) => outcomes.push(AccountOutcome::Success(records)),
        Err(e) => outcomes.push(AccountOutcome::Failed { account_id: account.id, error: e }),
    }
}
```

## サーキットブレーカー

導入しない。Organization横断スキャンは都度実行のバッチ処理であり、常時トラフィックに対する保護は不要と判断。

## ヘルスチェック・フェイルオーバー

対象外（常駐サービスではないため。NFR4.1参照）。

## バックアップ戦略

対象外（NFR4.4参照。削除されたCloudWatch Logsデータの復元は本ツールの責務ではない）。

## Assumptions & Open Questions

None.
