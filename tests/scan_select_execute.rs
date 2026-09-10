//! 統合シナリオテスト: スキャン → 選択 → 削除（dry-run）の一連の流れを
//! 複数モジュールを通して検証する。モックAWS境界のみを使用し、実AWSアカウントには
//! 一切接触しない。

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use secrecy::SecretString;

use cwsweep::aggregator::ScanAggregator;
use cwsweep::audit::AuditLogger;
use cwsweep::confirmation::{ConfirmPrompt, ConfirmationPresenter, ConfirmationSummary};
use cwsweep::credentials::{AccountCredentials, AssumeRoleOperations, CredentialProvider};
use cwsweep::error::{AssumeRoleError, CallerIdentityCallError};
use cwsweep::execution::{ActionApiOperations, ExecutionEngine};
use cwsweep::identity::{CallerIdentityOperations, IdentityCheck, IdentityVerifier};
use cwsweep::org_discovery::{AccountInfo, AccountStatus, ListAccountsOperations, OrgDiscovery};
use cwsweep::output::{OutputFormat, OutputFormatter};
use cwsweep::planner::{ActionKind, ActionPlanner};
use cwsweep::scanner::{DescribeLogGroupsOperations, LogGroupPage, LogGroupScanner, RawLogGroup};
use cwsweep::selector::{InteractiveSelector, MultiSelectPrompt, SelectableItem, SelectorError};

const MANAGEMENT_ACCOUNT_ID: &str = "111111111111";
const MEMBER_ACCOUNT_ID: &str = "222222222222";
const REGION: &str = "us-east-1";

fn creds(account_id: &str) -> AccountCredentials {
    AccountCredentials {
        account_id: account_id.to_string(),
        access_key_id: "AKIAFIXTURE".to_string(),
        secret_access_key: SecretString::from("secret".to_string()),
        session_token: SecretString::from("token".to_string()),
        expiration: None,
    }
}

struct StubListAccounts;
#[async_trait]
impl ListAccountsOperations for StubListAccounts {
    async fn list_all_accounts(
        &self,
    ) -> Result<Vec<AccountInfo>, cwsweep::error::OrgDiscoveryError> {
        Ok(vec![
            AccountInfo {
                account_id: MANAGEMENT_ACCOUNT_ID.to_string(),
                account_name: "management".to_string(),
                status: AccountStatus::Active,
            },
            AccountInfo {
                account_id: MEMBER_ACCOUNT_ID.to_string(),
                account_name: "member".to_string(),
                status: AccountStatus::Active,
            },
        ])
    }
}

struct StubAssumeRole;
#[async_trait]
impl AssumeRoleOperations for StubAssumeRole {
    async fn assume_role(
        &self,
        account_id: &str,
        _role_arn: &str,
        _session_name: &str,
    ) -> Result<AccountCredentials, AssumeRoleError> {
        Ok(creds(account_id))
    }
}

struct StubCallerIdentity;
#[async_trait]
impl CallerIdentityOperations for StubCallerIdentity {
    async fn get_caller_identity(
        &self,
        creds: &AccountCredentials,
    ) -> Result<String, CallerIdentityCallError> {
        Ok(creds.account_id.clone())
    }
}

struct StubDescribeLogGroups;
#[async_trait]
impl DescribeLogGroupsOperations for StubDescribeLogGroups {
    async fn describe_log_groups_page(
        &self,
        creds: &AccountCredentials,
        _region: &str,
        next_token: Option<String>,
    ) -> Result<LogGroupPage, String> {
        // アカウントごとに2ページ返す小さなフィクスチャ。
        match next_token {
            None => Ok(LogGroupPage {
                log_groups: vec![RawLogGroup {
                    name: format!("/aws/lambda/{}-page1", creds.account_id),
                    stored_bytes: 1_000_000,
                    retention_in_days: Some(30),
                }],
                next_token: Some("page-2".to_string()),
            }),
            Some(_) => Ok(LogGroupPage {
                log_groups: vec![RawLogGroup {
                    name: format!("/aws/lambda/{}-page2", creds.account_id),
                    stored_bytes: 500_000,
                    retention_in_days: None,
                }],
                next_token: None,
            }),
        }
    }
}

struct RecordingApiClient {
    delete_calls: AtomicUsize,
}
#[async_trait]
impl ActionApiOperations for RecordingApiClient {
    async fn delete_log_group(
        &self,
        _creds: &AccountCredentials,
        _region: &str,
        _log_group_name: &str,
    ) -> Result<(), String> {
        self.delete_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    async fn put_retention_policy(
        &self,
        _creds: &AccountCredentials,
        _region: &str,
        _log_group_name: &str,
        _days: i32,
    ) -> Result<(), String> {
        Ok(())
    }
}

struct FixedMultiSelect {
    label_to_pick: String,
}
impl MultiSelectPrompt for FixedMultiSelect {
    fn prompt(
        &self,
        items: &[SelectableItem],
        default_selected_indices: &[usize],
    ) -> Result<Vec<usize>, SelectorError> {
        // 初期状態が全チェックOFFであることを統合レベルでも確認する。
        assert!(default_selected_indices.is_empty());
        Ok(items
            .iter()
            .enumerate()
            .filter(|(_, i)| i.label == self.label_to_pick)
            .map(|(idx, _)| idx)
            .collect())
    }
}

struct AlwaysConfirm;
impl ConfirmPrompt for AlwaysConfirm {
    fn confirm(&self, _summary: &ConfirmationSummary) -> bool {
        true
    }
}

#[tokio::test]
async fn full_pipeline_scan_select_confirm_dry_run_never_calls_delete_api() {
    // --- スキャン ---
    let org_discovery = OrgDiscovery::new(StubListAccounts);
    let accounts = org_discovery.list_active_accounts().await.unwrap();
    assert_eq!(accounts.len(), 2);

    let credential_provider = CredentialProvider::with_default_role_name(
        MANAGEMENT_ACCOUNT_ID,
        Arc::new(StubAssumeRole),
        creds(MANAGEMENT_ACCOUNT_ID),
    );
    let identity: Arc<dyn IdentityCheck> = Arc::new(IdentityVerifier::new(StubCallerIdentity));
    let scanner = LogGroupScanner::new(Arc::new(StubDescribeLogGroups), identity.clone());

    let mut aggregator = ScanAggregator::new();
    for account in &accounts {
        let creds = credential_provider
            .credentials_for(&account.account_id)
            .await
            .unwrap();
        let records = scanner
            .scan_account_region(&creds, &account.account_id, REGION)
            .await
            .unwrap();
        aggregator.add_all(records);
    }

    // 2アカウント × 2ページ = 4レコード。全ページ確定後の集計であることを確認。
    assert_eq!(aggregator.len(), 4);

    let table = OutputFormatter::format(&aggregator, OutputFormat::Table);
    assert!(table.contains(MANAGEMENT_ACCOUNT_ID));
    assert!(table.contains(MEMBER_ACCOUNT_ID));

    // --- 選択（初期状態は全チェックOFF、1件だけ明示的に選択する） ---
    let target_label =
        format!("{MANAGEMENT_ACCOUNT_ID}/{REGION}//aws/lambda/{MANAGEMENT_ACCOUNT_ID}-page1");
    let selector = InteractiveSelector::new(FixedMultiSelect {
        label_to_pick: target_label.clone(),
    });
    let items: Vec<SelectableItem> = aggregator
        .records()
        .iter()
        .map(|r| SelectableItem {
            label: format!("{}/{}/{}", r.account_id, r.region, r.log_group_name),
            record: r.clone(),
        })
        .collect();
    let selected = selector.select(&items).unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(
        selected[0].log_group_name,
        format!("/aws/lambda/{MANAGEMENT_ACCOUNT_ID}-page1")
    );

    // --- 計画 ---
    let planned = ActionPlanner::plan(&selected, ActionKind::Delete);
    assert_eq!(planned.len(), 1);

    // --- 確認 ---
    let presenter = ConfirmationPresenter::new(AlwaysConfirm);
    let total_bytes: i64 = selected.iter().map(|r| r.stored_bytes).sum();
    let confirmed = presenter.confirm(&planned, total_bytes).unwrap();
    assert!(confirmed[0].confirmed);

    // --- 実行（dry-run既定: --executeを渡さない） ---
    let dir = tempfile::tempdir().unwrap();
    let audit_logger: Arc<dyn cwsweep::audit::AuditWrite> =
        Arc::new(AuditLogger::open(&dir.path().join("audit.jsonl")).unwrap());
    let api_client = Arc::new(RecordingApiClient {
        delete_calls: AtomicUsize::new(0),
    });
    let engine = ExecutionEngine::new(
        api_client.clone(),
        identity,
        audit_logger,
        "run-integration-1",
    );

    let outcomes = engine
        .execute_plan(&confirmed, |id| Some(creds(id)), false)
        .await
        .unwrap();

    assert_eq!(api_client.delete_calls.load(Ordering::SeqCst), 0);
    assert!(outcomes.iter().all(|o| !o.success));
}

#[tokio::test]
async fn full_pipeline_with_execute_true_calls_delete_and_writes_audit_log() {
    let org_discovery = OrgDiscovery::new(StubListAccounts);
    let accounts = org_discovery.list_active_accounts().await.unwrap();

    let credential_provider = CredentialProvider::with_default_role_name(
        MANAGEMENT_ACCOUNT_ID,
        Arc::new(StubAssumeRole),
        creds(MANAGEMENT_ACCOUNT_ID),
    );
    let identity: Arc<dyn IdentityCheck> = Arc::new(IdentityVerifier::new(StubCallerIdentity));
    let scanner = LogGroupScanner::new(Arc::new(StubDescribeLogGroups), identity.clone());

    let mut aggregator = ScanAggregator::new();
    for account in &accounts {
        let creds = credential_provider
            .credentials_for(&account.account_id)
            .await
            .unwrap();
        let records = scanner
            .scan_account_region(&creds, &account.account_id, REGION)
            .await
            .unwrap();
        aggregator.add_all(records);
    }

    let target_label =
        format!("{MEMBER_ACCOUNT_ID}/{REGION}//aws/lambda/{MEMBER_ACCOUNT_ID}-page1");
    let selector = InteractiveSelector::new(FixedMultiSelect {
        label_to_pick: target_label,
    });
    let items: Vec<SelectableItem> = aggregator
        .records()
        .iter()
        .map(|r| SelectableItem {
            label: format!("{}/{}/{}", r.account_id, r.region, r.log_group_name),
            record: r.clone(),
        })
        .collect();
    let selected = selector.select(&items).unwrap();

    let planned = ActionPlanner::plan(&selected, ActionKind::Delete);
    let presenter = ConfirmationPresenter::new(AlwaysConfirm);
    let total_bytes: i64 = selected.iter().map(|r| r.stored_bytes).sum();
    let confirmed = presenter.confirm(&planned, total_bytes).unwrap();

    let dir = tempfile::tempdir().unwrap();
    let audit_path = dir.path().join("audit.jsonl");
    let audit_logger: Arc<dyn cwsweep::audit::AuditWrite> =
        Arc::new(AuditLogger::open(&audit_path).unwrap());
    let api_client = Arc::new(RecordingApiClient {
        delete_calls: AtomicUsize::new(0),
    });
    let engine = ExecutionEngine::new(
        api_client.clone(),
        identity,
        audit_logger,
        "run-integration-2",
    );

    let outcomes = engine
        .execute_plan(&confirmed, |id| Some(creds(id)), true)
        .await
        .unwrap();

    assert_eq!(api_client.delete_calls.load(Ordering::SeqCst), 1);
    assert!(outcomes[0].success);

    let contents = std::fs::read_to_string(&audit_path).unwrap();
    assert_eq!(contents.lines().count(), 1);
    assert!(contents.contains(MEMBER_ACCOUNT_ID));
}
