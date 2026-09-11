//! `LogGroupScanner` コンポーネント。
//!
//! `describe-log-groups` のページネーションを最後まで辿ってから集計に渡す構造的な強制。
//! 1ページのみで確定する実装を書けないよう、`scan_account_region` はループが完全に
//! 完了した後にのみ `Vec<LogGroupRecord>` を返す（呼び出し元へページ単位の途中結果を渡す
//! 経路自体が存在しない）。呼び出し前に `IdentityCheck` による検証を経る。

use std::sync::Arc;

use async_trait::async_trait;

use crate::aggregator::LogGroupRecord;
use crate::credentials::AccountCredentials;
use crate::error::{PaginationError, ScanError};
use crate::identity::IdentityCheck;

/// `describe-log-groups` の1ページ分。
#[derive(Debug, Clone, PartialEq)]
pub struct LogGroupPage {
    pub log_groups: Vec<RawLogGroup>,
    pub next_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawLogGroup {
    pub name: String,
    pub stored_bytes: i64,
    pub retention_in_days: Option<i32>,
}

/// CloudWatch Logsの`describe-log-groups`境界の抽象化。
#[async_trait]
pub trait DescribeLogGroupsOperations: Send + Sync {
    async fn describe_log_groups_page(
        &self,
        creds: &AccountCredentials,
        region: &str,
        next_token: Option<String>,
    ) -> Result<LogGroupPage, String>;
}

pub struct LogGroupScanner {
    logs_client: Arc<dyn DescribeLogGroupsOperations>,
    identity: Arc<dyn IdentityCheck>,
}

impl LogGroupScanner {
    pub fn new(
        logs_client: Arc<dyn DescribeLogGroupsOperations>,
        identity: Arc<dyn IdentityCheck>,
    ) -> Self {
        Self {
            logs_client,
            identity,
        }
    }

    /// 単一アカウント×単一リージョンをスキャンする。
    ///
    /// 全ページ取得が完了して初めて `Ok` を返す。途中のページでエラーが発生した場合は
    /// それまでに取得した部分結果を破棄し `Err` を返す（部分確定の集計を許さない）。
    pub async fn scan_account_region(
        &self,
        creds: &AccountCredentials,
        account_id: &str,
        region: &str,
    ) -> Result<Vec<LogGroupRecord>, ScanError> {
        let mut all_records = Vec::new();
        let mut next_token: Option<String> = None;
        let mut page_index = 0usize;

        loop {
            // NFR2.3: 各AWS API呼び出し（各ページの`describe-log-groups`呼び出し）の
            // 直前に毎回`sts:get-caller-identity`を実行する。ページネーション開始前の
            // 1回だけでは、途中でクレデンシャルの実効アカウントが変わった場合を検知できない。
            self.identity.verify(creds, account_id).await?;

            let page = self
                .logs_client
                .describe_log_groups_page(creds, region, next_token.clone())
                .await
                .map_err(|message| PaginationError {
                    account_id: account_id.to_string(),
                    region: region.to_string(),
                    page_index,
                    message,
                })?;

            for raw in page.log_groups {
                all_records.push(LogGroupRecord {
                    account_id: account_id.to_string(),
                    region: region.to_string(),
                    log_group_name: raw.name,
                    stored_bytes: raw.stored_bytes,
                    retention_in_days: raw.retention_in_days,
                });
            }

            page_index += 1;

            match page.next_token {
                Some(token) => next_token = Some(token),
                None => break,
            }
        }

        Ok(all_records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::IdentityError;
    use crate::identity::IdentityCheck;
    use secrecy::SecretString;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    const ACCOUNT_ID: &str = "111111111111";
    const REGION: &str = "us-east-1";

    fn creds() -> AccountCredentials {
        AccountCredentials {
            account_id: ACCOUNT_ID.to_string(),
            access_key_id: "AKIAFIXTURE".to_string(),
            secret_access_key: SecretString::from("secret".to_string()),
            session_token: Some(SecretString::from("token".to_string())),
            expiration: None,
        }
    }

    struct AlwaysOkIdentity;
    #[async_trait]
    impl IdentityCheck for AlwaysOkIdentity {
        async fn verify(&self, _c: &AccountCredentials, _e: &str) -> Result<(), IdentityError> {
            Ok(())
        }
    }

    /// 呼び出し回数を記録するIdentityスタブ。CodeRabbit指摘#6:
    /// 各ページ取得の直前に毎回Identity検証が実行されることを検証するために使う。
    struct CountingIdentity {
        calls: AtomicUsize,
    }
    #[async_trait]
    impl IdentityCheck for CountingIdentity {
        async fn verify(&self, _c: &AccountCredentials, _e: &str) -> Result<(), IdentityError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    /// 指定した呼び出し回数目（1始まり）以降で不一致エラーを返すIdentityスタブ。
    /// ページネーション途中でIdentity検証が失敗するケースを再現する。
    struct FailsFromNthCallIdentity {
        calls: AtomicUsize,
        fail_from_call: usize,
    }
    #[async_trait]
    impl IdentityCheck for FailsFromNthCallIdentity {
        async fn verify(
            &self,
            _c: &AccountCredentials,
            expected: &str,
        ) -> Result<(), IdentityError> {
            let n = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
            if n >= self.fail_from_call {
                Err(crate::error::IdentityMismatchError {
                    expected: expected.to_string(),
                    actual: Some("999999999999".to_string()),
                }
                .into())
            } else {
                Ok(())
            }
        }
    }

    struct AlwaysFailIdentity;
    #[async_trait]
    impl IdentityCheck for AlwaysFailIdentity {
        async fn verify(
            &self,
            _c: &AccountCredentials,
            expected: &str,
        ) -> Result<(), IdentityError> {
            Err(crate::error::IdentityMismatchError {
                expected: expected.to_string(),
                actual: Some("999999999999".to_string()),
            }
            .into())
        }
    }

    /// 複数ページを順に返すスタブ。
    struct MultiPageClient {
        pages: Mutex<Vec<Result<LogGroupPage, String>>>,
        calls: AtomicUsize,
    }

    #[async_trait]
    impl DescribeLogGroupsOperations for MultiPageClient {
        async fn describe_log_groups_page(
            &self,
            _creds: &AccountCredentials,
            _region: &str,
            _next_token: Option<String>,
        ) -> Result<LogGroupPage, String> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let mut pages = self.pages.lock().unwrap();
            if pages.is_empty() {
                panic!("no more pages configured");
            }
            pages.remove(0)
        }
    }

    fn page(names: &[&str], next_token: Option<&str>) -> LogGroupPage {
        LogGroupPage {
            log_groups: names
                .iter()
                .map(|n| RawLogGroup {
                    name: n.to_string(),
                    stored_bytes: 100,
                    retention_in_days: Some(30),
                })
                .collect(),
            next_token: next_token.map(String::from),
        }
    }

    #[tokio::test]
    async fn single_page_scan_returns_all_records() {
        let client = Arc::new(MultiPageClient {
            pages: Mutex::new(vec![Ok(page(&["/a", "/b"], None))]),
            calls: AtomicUsize::new(0),
        });
        let scanner = LogGroupScanner::new(client.clone(), Arc::new(AlwaysOkIdentity));

        let records = scanner
            .scan_account_region(&creds(), ACCOUNT_ID, REGION)
            .await
            .unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(client.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn multi_page_scan_walks_every_page_before_returning() {
        let client = Arc::new(MultiPageClient {
            pages: Mutex::new(vec![
                Ok(page(&["/a"], Some("token-2"))),
                Ok(page(&["/b"], Some("token-3"))),
                Ok(page(&["/c"], None)),
            ]),
            calls: AtomicUsize::new(0),
        });
        let scanner = LogGroupScanner::new(client.clone(), Arc::new(AlwaysOkIdentity));

        let records = scanner
            .scan_account_region(&creds(), ACCOUNT_ID, REGION)
            .await
            .unwrap();

        // 全3ページ分のレコードが揃っていることを確認 -- 1ページ目だけでの確定を許容しない。
        assert_eq!(records.len(), 3);
        assert_eq!(client.calls.load(Ordering::SeqCst), 3);
        let names: Vec<&str> = records.iter().map(|r| r.log_group_name.as_str()).collect();
        assert_eq!(names, vec!["/a", "/b", "/c"]);
    }

    // --- CodeRabbit指摘#6: ページネーション中のIdentity検証が各ページ直前で行われること ---

    #[tokio::test]
    async fn identity_check_runs_once_per_page_across_multiple_pages() {
        let client = Arc::new(MultiPageClient {
            pages: Mutex::new(vec![
                Ok(page(&["/a"], Some("token-2"))),
                Ok(page(&["/b"], Some("token-3"))),
                Ok(page(&["/c"], None)),
            ]),
            calls: AtomicUsize::new(0),
        });
        let identity = Arc::new(CountingIdentity {
            calls: AtomicUsize::new(0),
        });
        let scanner = LogGroupScanner::new(client.clone(), identity.clone());

        let records = scanner
            .scan_account_region(&creds(), ACCOUNT_ID, REGION)
            .await
            .unwrap();

        assert_eq!(records.len(), 3);
        // 1ページ目の取得前だけでなく、3ページ全て（各ページ取得の直前）に検証が走る。
        assert_eq!(identity.calls.load(Ordering::SeqCst), 3);
        assert_eq!(client.calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn identity_check_failure_on_a_later_page_stops_before_fetching_that_page() {
        // 境界値: 1ページ目のIdentity検証は成功するが、2ページ目取得直前の検証が
        // 不一致で失敗するケース。2ページ目のAPI呼び出し自体が行われてはならない。
        let client = Arc::new(MultiPageClient {
            pages: Mutex::new(vec![
                Ok(page(&["/a"], Some("token-2"))),
                Ok(page(&["/b"], None)),
            ]),
            calls: AtomicUsize::new(0),
        });
        let identity = Arc::new(FailsFromNthCallIdentity {
            calls: AtomicUsize::new(0),
            fail_from_call: 2,
        });
        let scanner = LogGroupScanner::new(client.clone(), identity);

        let result = scanner
            .scan_account_region(&creds(), ACCOUNT_ID, REGION)
            .await;

        assert!(matches!(result, Err(ScanError::Identity(_))));
        // 2ページ目のIdentity検証で失敗したため、2ページ目のAPI呼び出しは発生しない。
        assert_eq!(client.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn identity_check_runs_before_any_page_is_fetched() {
        let client = Arc::new(MultiPageClient {
            pages: Mutex::new(vec![Ok(page(&["/a"], None))]),
            calls: AtomicUsize::new(0),
        });
        let scanner = LogGroupScanner::new(client.clone(), Arc::new(AlwaysFailIdentity));

        let result = scanner
            .scan_account_region(&creds(), ACCOUNT_ID, REGION)
            .await;

        assert!(result.is_err());
        assert_eq!(client.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn pagination_error_mid_stream_discards_partial_results_and_fails() {
        let client = Arc::new(MultiPageClient {
            pages: Mutex::new(vec![
                Ok(page(&["/a"], Some("token-2"))),
                Err("throttled".to_string()),
            ]),
            calls: AtomicUsize::new(0),
        });
        let scanner = LogGroupScanner::new(client, Arc::new(AlwaysOkIdentity));

        let result = scanner
            .scan_account_region(&creds(), ACCOUNT_ID, REGION)
            .await;

        match result {
            Err(ScanError::Pagination(e)) => {
                assert_eq!(e.page_index, 1);
                assert_eq!(e.account_id, ACCOUNT_ID);
            }
            other => panic!("expected Pagination error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn empty_result_set_is_a_valid_completed_scan() {
        let client = Arc::new(MultiPageClient {
            pages: Mutex::new(vec![Ok(page(&[], None))]),
            calls: AtomicUsize::new(0),
        });
        let scanner = LogGroupScanner::new(client, Arc::new(AlwaysOkIdentity));

        let records = scanner
            .scan_account_region(&creds(), ACCOUNT_ID, REGION)
            .await
            .unwrap();

        assert!(records.is_empty());
    }
}
