# Security Design

## 認証情報の保持（NFR2.1対応）

`AccountCredentials`の`secret_access_key`/`session_token`は`secrecy::SecretString`型でラップする。`Debug`導出は使わず、手動実装で`[REDACTED]`にマスクする。

```rust
// 疑似コード（設計レベル）
pub struct AccountCredentials {
    pub access_key_id: String,
    pub secret_access_key: secrecy::SecretString, // 機微
    pub session_token: secrecy::SecretString,     // 機微
}
// Debugは手動実装し、機微フィールドは "[REDACTED]" にマスクする
```

## Identity検証（NFR2.3, NFR2.5対応）

`IdentityVerifier`は副作用のない検証関数とし、失敗時はエラー型で呼び出し元へ伝播させる。呼び出し元（LogGroupScanner起動前、ExecutionEngine実行直前の二重目）で`?`により即座に処理を打ち切る。

```rust
// 疑似コード
pub async fn verify_identity(creds: &AccountCredentials, expected_account_id: &str) -> Result<(), IdentityMismatchError> {
    let actual = sts_client(creds).get_caller_identity().send().await?;
    if actual.account() != Some(expected_account_id) {
        return Err(IdentityMismatchError { expected: expected_account_id.into(), actual: actual.account().map(String::from) });
    }
    Ok(())
}
```

## dry-run既定（NFR2.4対応）

`ExecutionEngine`は`execute: bool`フラグを受け取り、`false`（既定）の場合は`delete-log-group`/`put-retention-policy`のAPI呼び出しコード自体を通過しない分岐にする（フラグ判定漏れで誤って実行される余地を減らすため、dry-run分岐をAPI呼び出し関数の外側・エントリポイントに近い位置に置く）。Domain Design ADR-003（計画・確認・実行の3層分離）の上に、この1フラグ判定を最終防衛線として重ねる。

## 入力値検証・リージョン明示指定（NFR3.2対応）

CLI引数（アカウントID形式、リージョン名、retention日数の範囲）は`clap`のderive `value_parser`で構文的に検証する。特に`--regions`は必須引数とし、既定値・全リージョン自動列挙のフォールバックを持たせない（`OrgDiscovery`はアカウント列挙のみを担い、対象リージョンの決定はCliAppの引数解析責務とする）。これにより誤って無関係なリージョンをスキャンする事故を構造的に防ぐ。

## シークレット管理

環境変数・設定ファイルによる認証情報の直接指定はサポートしない。AWS標準の認証チェーン（`aws-config`）とAssumeRoleのみを経路とする。

## 監査ログの完全性（NFR4.3対応）

`AuditLogger`はJSON Linesを1エントリずつ`fsync`付きで追記する。書き込み失敗（`std::io::Error`）は`ExecutionEngine`へ`Result`で伝播し、当該操作を中断させる。

```rust
// 疑似コード
pub fn append(&mut self, entry: &AuditEntry) -> Result<(), AuditWriteError> {
    let line = serde_json::to_string(entry)?;
    self.file.write_all(line.as_bytes())?;
    self.file.write_all(b"\n")?;
    self.file.sync_data()?; // fsync
    Ok(())
}
```

## Assumptions & Open Questions

None.
