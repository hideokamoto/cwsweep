//! `OrgDiscovery` コンポーネント。Organization内のアクティブアカウントを列挙する。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::OrgDiscoveryError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountInfo {
    pub account_id: String,
    pub account_name: String,
    pub status: AccountStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountStatus {
    Active,
    Suspended,
    PendingClosure,
}

/// `organizations:list-accounts` 境界の抽象化。
#[async_trait]
pub trait ListAccountsOperations: Send + Sync {
    /// ページネーションも含め、全件取得後のアカウント一覧を返す（ステータスは未フィルタ）。
    async fn list_all_accounts(&self) -> Result<Vec<AccountInfo>, OrgDiscoveryError>;
}

pub struct OrgDiscovery<L: ListAccountsOperations> {
    client: L,
}

impl<L: ListAccountsOperations> OrgDiscovery<L> {
    pub fn new(client: L) -> Self {
        Self { client }
    }

    /// `Status=ACTIVE` のアカウントのみを返す（管理アカウント自身を含む）。
    pub async fn list_active_accounts(&self) -> Result<Vec<AccountInfo>, OrgDiscoveryError> {
        let all = self.client.list_all_accounts().await?;
        Ok(all
            .into_iter()
            .filter(|a| a.status == AccountStatus::Active)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(id: &str, status: AccountStatus) -> AccountInfo {
        AccountInfo {
            account_id: id.to_string(),
            account_name: format!("account-{id}"),
            status,
        }
    }

    struct StubClient {
        accounts: Vec<AccountInfo>,
    }
    #[async_trait]
    impl ListAccountsOperations for StubClient {
        async fn list_all_accounts(&self) -> Result<Vec<AccountInfo>, OrgDiscoveryError> {
            Ok(self.accounts.clone())
        }
    }

    struct FailingClient;
    #[async_trait]
    impl ListAccountsOperations for FailingClient {
        async fn list_all_accounts(&self) -> Result<Vec<AccountInfo>, OrgDiscoveryError> {
            Err(OrgDiscoveryError {
                message: "access denied".to_string(),
            })
        }
    }

    #[tokio::test]
    async fn returns_only_active_accounts() {
        let discovery = OrgDiscovery::new(StubClient {
            accounts: vec![
                account("111111111111", AccountStatus::Active),
                account("222222222222", AccountStatus::Suspended),
                account("333333333333", AccountStatus::Active),
            ],
        });

        let active = discovery.list_active_accounts().await.unwrap();

        assert_eq!(active.len(), 2);
        assert!(active.iter().all(|a| a.status == AccountStatus::Active));
    }

    #[tokio::test]
    async fn includes_management_account_when_active() {
        let discovery = OrgDiscovery::new(StubClient {
            accounts: vec![account("111111111111", AccountStatus::Active)],
        });

        let active = discovery.list_active_accounts().await.unwrap();

        assert_eq!(active[0].account_id, "111111111111");
    }

    #[tokio::test]
    async fn excludes_pending_closure_accounts() {
        let discovery = OrgDiscovery::new(StubClient {
            accounts: vec![account("111111111111", AccountStatus::PendingClosure)],
        });

        let active = discovery.list_active_accounts().await.unwrap();

        assert!(active.is_empty());
    }

    #[tokio::test]
    async fn propagates_list_accounts_failure() {
        let discovery = OrgDiscovery::new(FailingClient);

        let result = discovery.list_active_accounts().await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn empty_organization_returns_empty_list() {
        let discovery = OrgDiscovery::new(StubClient { accounts: vec![] });

        let active = discovery.list_active_accounts().await.unwrap();

        assert!(active.is_empty());
    }
}
