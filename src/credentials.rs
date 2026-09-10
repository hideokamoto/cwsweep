//! `CredentialProvider` コンポーネント。
//!
//! 管理アカウント自身へは AssumeRole を行わず現在の認証情報をそのまま使用し、
//! メンバーアカウントに対してのみ `sts:AssumeRole` を行う (project.md Mandated)。
//! 一時クレデンシャルは `secrecy::SecretString` でラップし、`Debug` はマスクする。

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use secrecy::SecretString;

use crate::error::AssumeRoleError;

/// アカウントごとの一時（または管理アカウントの場合は現行の）クレデンシャル。
///
/// `secret_access_key` / `session_token` は `secrecy::SecretString` でラップし、
/// `Debug` は手動実装で `[REDACTED]` にマスクする（project.md Forbidden: クレデンシャル構造体の
/// `{:?}` 丸ごとダンプ禁止）。
#[derive(Clone)]
pub struct AccountCredentials {
    pub account_id: String,
    pub access_key_id: String,
    pub secret_access_key: SecretString,
    pub session_token: SecretString,
    pub expiration: Option<String>,
}

impl fmt::Debug for AccountCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccountCredentials")
            .field("account_id", &self.account_id)
            .field("access_key_id", &self.access_key_id)
            .field("secret_access_key", &"[REDACTED]")
            .field("session_token", &"[REDACTED]")
            .field("expiration", &self.expiration)
            .finish()
    }
}

/// `sts:AssumeRole` 境界の抽象化。テストでは手書きモックを注入する。
#[async_trait]
pub trait AssumeRoleOperations: Send + Sync {
    async fn assume_role(
        &self,
        account_id: &str,
        role_arn: &str,
        session_name: &str,
    ) -> Result<AccountCredentials, AssumeRoleError>;
}

/// アカウントごとの明示的クレデンシャルを構築するコンポーネント。
///
/// 環境変数由来の暗黙のクレデンシャルチェーンには依存しない
/// （呼び出し元が明示的に `management_credentials` を渡す設計とする）。
pub struct CredentialProvider {
    management_account_id: String,
    role_name: String,
    assume_role_client: Arc<dyn AssumeRoleOperations>,
    management_credentials: AccountCredentials,
}

impl CredentialProvider {
    pub fn new(
        management_account_id: impl Into<String>,
        role_name: impl Into<String>,
        assume_role_client: Arc<dyn AssumeRoleOperations>,
        management_credentials: AccountCredentials,
    ) -> Self {
        Self {
            management_account_id: management_account_id.into(),
            role_name: role_name.into(),
            assume_role_client,
            management_credentials,
        }
    }

    /// 既定のロール名 (`OrganizationAccountAccessRole`) を用いてプロバイダを構築する。
    pub fn with_default_role_name(
        management_account_id: impl Into<String>,
        assume_role_client: Arc<dyn AssumeRoleOperations>,
        management_credentials: AccountCredentials,
    ) -> Self {
        Self::new(
            management_account_id,
            "OrganizationAccountAccessRole",
            assume_role_client,
            management_credentials,
        )
    }

    pub fn role_name(&self) -> &str {
        &self.role_name
    }

    /// 指定アカウント用のクレデンシャルを取得する。
    ///
    /// - `account_id` が管理アカウント自身と一致する場合: 現在の認証情報をそのまま返す（AssumeRoleしない）。
    /// - それ以外（メンバーアカウント）: `sts:AssumeRole` を呼び出す。
    pub async fn credentials_for(
        &self,
        account_id: &str,
    ) -> Result<AccountCredentials, AssumeRoleError> {
        if account_id == self.management_account_id {
            return Ok(self.management_credentials.clone());
        }
        let role_arn = format!("arn:aws:iam::{account_id}:role/{}", self.role_name);
        self.assume_role_client
            .assume_role(account_id, &role_arn, "cwsweep")
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const MANAGEMENT_ACCOUNT_ID: &str = "111111111111";
    const MEMBER_ACCOUNT_ID: &str = "222222222222";

    fn fixture_credentials(account_id: &str) -> AccountCredentials {
        AccountCredentials {
            account_id: account_id.to_string(),
            access_key_id: format!("AKIA{account_id}"),
            secret_access_key: SecretString::from(format!("secret-for-{account_id}")),
            session_token: SecretString::from(format!("token-for-{account_id}")),
            expiration: None,
        }
    }

    struct RecordingAssumeRoleClient {
        calls: AtomicUsize,
    }

    #[async_trait]
    impl AssumeRoleOperations for RecordingAssumeRoleClient {
        async fn assume_role(
            &self,
            account_id: &str,
            _role_arn: &str,
            _session_name: &str,
        ) -> Result<AccountCredentials, AssumeRoleError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(fixture_credentials(account_id))
        }
    }

    struct FailingAssumeRoleClient;

    #[async_trait]
    impl AssumeRoleOperations for FailingAssumeRoleClient {
        async fn assume_role(
            &self,
            account_id: &str,
            role_arn: &str,
            _session_name: &str,
        ) -> Result<AccountCredentials, AssumeRoleError> {
            Err(AssumeRoleError {
                account_id: account_id.to_string(),
                role_name: role_arn.to_string(),
                message: "access denied".to_string(),
            })
        }
    }

    fn provider(client: Arc<dyn AssumeRoleOperations>) -> CredentialProvider {
        CredentialProvider::with_default_role_name(
            MANAGEMENT_ACCOUNT_ID,
            client,
            fixture_credentials(MANAGEMENT_ACCOUNT_ID),
        )
    }

    #[tokio::test]
    async fn management_account_uses_current_credentials_without_assume_role() {
        let client = Arc::new(RecordingAssumeRoleClient {
            calls: AtomicUsize::new(0),
        });
        let p = provider(client.clone());

        let creds = p.credentials_for(MANAGEMENT_ACCOUNT_ID).await.unwrap();

        assert_eq!(creds.account_id, MANAGEMENT_ACCOUNT_ID);
        assert_eq!(client.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn member_account_triggers_assume_role_exactly_once() {
        let client = Arc::new(RecordingAssumeRoleClient {
            calls: AtomicUsize::new(0),
        });
        let p = provider(client.clone());

        let creds = p.credentials_for(MEMBER_ACCOUNT_ID).await.unwrap();

        assert_eq!(creds.account_id, MEMBER_ACCOUNT_ID);
        assert_eq!(client.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn assume_role_failure_propagates_as_error() {
        let p = provider(Arc::new(FailingAssumeRoleClient));

        let err = p.credentials_for(MEMBER_ACCOUNT_ID).await.unwrap_err();

        assert_eq!(err.account_id, MEMBER_ACCOUNT_ID);
        assert!(err.role_name.contains("OrganizationAccountAccessRole"));
    }

    #[test]
    fn default_role_name_is_organization_account_access_role() {
        let client: Arc<dyn AssumeRoleOperations> = Arc::new(RecordingAssumeRoleClient {
            calls: AtomicUsize::new(0),
        });
        let p = provider(client);
        assert_eq!(p.role_name(), "OrganizationAccountAccessRole");
    }

    #[test]
    fn custom_role_name_overrides_default() {
        let client: Arc<dyn AssumeRoleOperations> = Arc::new(RecordingAssumeRoleClient {
            calls: AtomicUsize::new(0),
        });
        let p = CredentialProvider::new(
            MANAGEMENT_ACCOUNT_ID,
            "CustomRole",
            client,
            fixture_credentials(MANAGEMENT_ACCOUNT_ID),
        );
        assert_eq!(p.role_name(), "CustomRole");
    }

    #[test]
    fn debug_output_masks_secret_fields_and_never_leaks_raw_value() {
        let creds = fixture_credentials(MEMBER_ACCOUNT_ID);
        let secret_value = creds.secret_access_key.expose_secret().to_string();

        let debug_output = format!("{creds:?}");

        assert!(debug_output.contains("[REDACTED]"));
        assert!(!debug_output.contains(&secret_value));
    }
}
