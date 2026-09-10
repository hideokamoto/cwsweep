//! cwsweep バイナリエントリポイント。
//!
//! 実AWS SDKクライアントを構築し、`cwsweep`ライブラリのコンポーネントを配線する。
//! project.md Mandated: 管理アカウント自身へはAssumeRoleを行わず現在の認証情報をそのまま使用し、
//! メンバーアカウントに対してのみAssumeRoleを行う。

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use async_trait::async_trait;
use aws_config::retry::RetryConfig;
use aws_config::BehaviorVersion;
use clap::Parser;
use secrecy::{ExposeSecret, SecretString};
use uuid::Uuid;

use aws_sdk_sts::config::ProvideCredentials;
use cwsweep::aggregator::LogGroupRecord;
use cwsweep::audit::{AuditLogger, AuditWrite};
use cwsweep::cli::{Cli, CliApp};
use cwsweep::confirmation::{ConfirmPrompt, ConfirmationPresenter, ConfirmationSummary};
use cwsweep::credentials::{AccountCredentials, AssumeRoleOperations, CredentialProvider};
use cwsweep::error::{AssumeRoleError, CallerIdentityCallError, OrgDiscoveryError};
use cwsweep::execution::ActionApiOperations;
use cwsweep::identity::{CallerIdentityOperations, IdentityCheck, IdentityVerifier};
use cwsweep::org_discovery::{AccountInfo, AccountStatus, ListAccountsOperations, OrgDiscovery};
use cwsweep::planner::ActionKind;
use cwsweep::scanner::{DescribeLogGroupsOperations, LogGroupPage, RawLogGroup};
use cwsweep::selector::{InteractiveSelector, MultiSelectPrompt, SelectableItem, SelectorError};

const RETRY_MAX_ATTEMPTS: u32 = 3;

fn to_sdk_credentials(creds: &AccountCredentials) -> aws_sdk_sts::config::Credentials {
    aws_sdk_sts::config::Credentials::new(
        creds.access_key_id.clone(),
        creds.secret_access_key.expose_secret().to_string(),
        Some(creds.session_token.expose_secret().to_string()),
        None,
        "cwsweep",
    )
}

fn cwlogs_client_for(creds: &AccountCredentials, region: &str) -> aws_sdk_cloudwatchlogs::Client {
    let config = aws_sdk_cloudwatchlogs::Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(aws_sdk_cloudwatchlogs::config::Region::new(
            region.to_string(),
        ))
        .credentials_provider(to_sdk_credentials(creds))
        .retry_config(RetryConfig::standard().with_max_attempts(RETRY_MAX_ATTEMPTS))
        .build();
    aws_sdk_cloudwatchlogs::Client::from_conf(config)
}

fn sts_client_for(creds: &AccountCredentials) -> aws_sdk_sts::Client {
    let config = aws_sdk_sts::Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(aws_sdk_sts::config::Region::new("us-east-1".to_string()))
        .credentials_provider(to_sdk_credentials(creds))
        .retry_config(RetryConfig::standard().with_max_attempts(RETRY_MAX_ATTEMPTS))
        .build();
    aws_sdk_sts::Client::from_conf(config)
}

/// `sts:AssumeRole` の実AWS SDK実装。
struct StsAssumeRoleAdapter {
    client: aws_sdk_sts::Client,
}

#[async_trait]
impl AssumeRoleOperations for StsAssumeRoleAdapter {
    async fn assume_role(
        &self,
        account_id: &str,
        role_arn: &str,
        session_name: &str,
    ) -> Result<AccountCredentials, AssumeRoleError> {
        let output = self
            .client
            .assume_role()
            .role_arn(role_arn)
            .role_session_name(session_name)
            .send()
            .await
            .map_err(|e| AssumeRoleError {
                account_id: account_id.to_string(),
                role_name: role_arn.to_string(),
                message: e.to_string(),
            })?;

        let creds = output.credentials.ok_or_else(|| AssumeRoleError {
            account_id: account_id.to_string(),
            role_name: role_arn.to_string(),
            message: "assume-role response contained no credentials".to_string(),
        })?;

        Ok(AccountCredentials {
            account_id: account_id.to_string(),
            access_key_id: creds.access_key_id,
            secret_access_key: SecretString::from(creds.secret_access_key),
            session_token: SecretString::from(creds.session_token),
            expiration: Some(creds.expiration.to_string()),
        })
    }
}

/// `sts:get-caller-identity` の実AWS SDK実装。呼び出しごとにクレデンシャルに応じた
/// クライアントを構築する。
struct StsCallerIdentityAdapter;

#[async_trait]
impl CallerIdentityOperations for StsCallerIdentityAdapter {
    async fn get_caller_identity(
        &self,
        creds: &AccountCredentials,
    ) -> Result<String, CallerIdentityCallError> {
        let client = sts_client_for(creds);
        let output =
            client
                .get_caller_identity()
                .send()
                .await
                .map_err(|e| CallerIdentityCallError {
                    message: e.to_string(),
                })?;
        output.account.ok_or_else(|| CallerIdentityCallError {
            message: "get-caller-identity response contained no account id".to_string(),
        })
    }
}

/// `organizations:list-accounts` の実AWS SDK実装。全ページを辿って返す。
struct OrganizationsListAccountsAdapter {
    client: aws_sdk_organizations::Client,
}

#[async_trait]
impl ListAccountsOperations for OrganizationsListAccountsAdapter {
    async fn list_all_accounts(&self) -> Result<Vec<AccountInfo>, OrgDiscoveryError> {
        let mut accounts = Vec::new();
        let mut next_token: Option<String> = None;
        loop {
            let mut request = self.client.list_accounts();
            if let Some(token) = &next_token {
                request = request.next_token(token.clone());
            }
            let output = request.send().await.map_err(|e| OrgDiscoveryError {
                message: e.to_string(),
            })?;

            for account in output.accounts() {
                let Some(account_id) = account.id() else {
                    continue;
                };
                let status = match account.status() {
                    Some(aws_sdk_organizations::types::AccountStatus::Active) => {
                        AccountStatus::Active
                    }
                    Some(aws_sdk_organizations::types::AccountStatus::Suspended) => {
                        AccountStatus::Suspended
                    }
                    Some(aws_sdk_organizations::types::AccountStatus::PendingClosure) => {
                        AccountStatus::PendingClosure
                    }
                    _ => AccountStatus::Suspended,
                };
                accounts.push(AccountInfo {
                    account_id: account_id.to_string(),
                    account_name: account.name().unwrap_or_default().to_string(),
                    status,
                });
            }

            next_token = output.next_token().map(str::to_string);
            if next_token.is_none() {
                break;
            }
        }
        Ok(accounts)
    }
}

/// CloudWatch Logs (`describe-log-groups` / `delete-log-group` / `put-retention-policy`) の
/// 実AWS SDK実装。
struct CloudWatchLogsAdapter;

#[async_trait]
impl DescribeLogGroupsOperations for CloudWatchLogsAdapter {
    async fn describe_log_groups_page(
        &self,
        creds: &AccountCredentials,
        region: &str,
        next_token: Option<String>,
    ) -> Result<LogGroupPage, String> {
        let client = cwlogs_client_for(creds, region);
        let mut request = client.describe_log_groups();
        if let Some(token) = next_token {
            request = request.next_token(token);
        }
        let output = request.send().await.map_err(|e| e.to_string())?;

        let log_groups = output
            .log_groups()
            .iter()
            .map(|lg| RawLogGroup {
                name: lg.log_group_name().unwrap_or_default().to_string(),
                stored_bytes: lg.stored_bytes().unwrap_or(0),
                retention_in_days: lg.retention_in_days(),
            })
            .collect();

        Ok(LogGroupPage {
            log_groups,
            next_token: output.next_token().map(str::to_string),
        })
    }
}

#[async_trait]
impl ActionApiOperations for CloudWatchLogsAdapter {
    async fn delete_log_group(
        &self,
        creds: &AccountCredentials,
        region: &str,
        log_group_name: &str,
    ) -> Result<(), String> {
        let client = cwlogs_client_for(creds, region);
        client
            .delete_log_group()
            .log_group_name(log_group_name)
            .send()
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    async fn put_retention_policy(
        &self,
        creds: &AccountCredentials,
        region: &str,
        log_group_name: &str,
        days: i32,
    ) -> Result<(), String> {
        let client = cwlogs_client_for(creds, region);
        client
            .put_retention_policy()
            .log_group_name(log_group_name)
            .retention_in_days(days)
            .send()
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

/// `inquire`ベースの実対話式マルチセレクトUI実装。
struct InquireMultiSelectPrompt;

impl MultiSelectPrompt for InquireMultiSelectPrompt {
    fn prompt(
        &self,
        items: &[SelectableItem],
        default_selected_indices: &[usize],
    ) -> Result<Vec<usize>, SelectorError> {
        let options: Vec<String> = items.iter().map(|i| i.label.clone()).collect();
        let selected =
            inquire::MultiSelect::new("削除・retention変更の対象を選択してください:", options)
                .with_default(default_selected_indices)
                .prompt()
                .map_err(|e| SelectorError(e.to_string()))?;
        Ok(items
            .iter()
            .enumerate()
            .filter(|(_, item)| selected.contains(&item.label))
            .map(|(i, _)| i)
            .collect())
    }
}

/// `inquire`ベースの実最終確認プロンプト実装。
struct InquireConfirmPrompt;

impl ConfirmPrompt for InquireConfirmPrompt {
    fn confirm(&self, summary: &ConfirmationSummary) -> bool {
        println!("対象アカウントID: {:?}", summary.account_ids);
        println!("対象リージョン: {:?}", summary.regions);
        println!("対象ログループ: {:?}", summary.log_group_names);
        println!("合計バイト数: {}", summary.total_bytes);
        inquire::Confirm::new("上記の内容で実行してよろしいですか？")
            .with_default(false)
            .prompt()
            .unwrap_or(false)
    }
}

fn action_kind_from_prompt() -> Result<ActionKind, String> {
    let options = vec!["delete", "set-retention"];
    let choice = inquire::Select::new("実行するアクションを選択してください:", options)
        .prompt()
        .map_err(|e| e.to_string())?;
    match choice {
        "delete" => Ok(ActionKind::Delete),
        _ => {
            let days = inquire::CustomType::<i32>::new("retention日数を入力してください:")
                .prompt()
                .map_err(|e| e.to_string())?;
            Ok(ActionKind::SetRetention { days })
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    let run_id = Uuid::new_v4().to_string();

    let base_config = aws_config::defaults(BehaviorVersion::latest())
        .retry_config(RetryConfig::standard().with_max_attempts(RETRY_MAX_ATTEMPTS))
        .load()
        .await;

    let provider = base_config
        .credentials_provider()
        .ok_or("no credentials provider resolved from the environment")?;
    let raw_creds = provider.provide_credentials().await?;
    let management_credentials = AccountCredentials {
        account_id: String::new(),
        access_key_id: raw_creds.access_key_id().to_string(),
        secret_access_key: SecretString::from(raw_creds.secret_access_key().to_string()),
        session_token: SecretString::from(
            raw_creds.session_token().unwrap_or_default().to_string(),
        ),
        expiration: None,
    };

    let caller_identity = StsCallerIdentityAdapter;
    let management_account_id = caller_identity
        .get_caller_identity(&management_credentials)
        .await?;
    let management_credentials = AccountCredentials {
        account_id: management_account_id.clone(),
        ..management_credentials
    };

    let sts_client = aws_sdk_sts::Client::new(&base_config);
    let assume_role_client: Arc<dyn AssumeRoleOperations> =
        Arc::new(StsAssumeRoleAdapter { client: sts_client });
    let credential_provider = Arc::new(CredentialProvider::new(
        management_account_id.clone(),
        cli.role_name.clone(),
        assume_role_client,
        management_credentials,
    ));

    let identity: Arc<dyn IdentityCheck> =
        Arc::new(IdentityVerifier::new(StsCallerIdentityAdapter));

    let organizations_client = aws_sdk_organizations::Client::new(&base_config);
    let org_discovery = OrgDiscovery::new(OrganizationsListAccountsAdapter {
        client: organizations_client,
    });
    let accounts = org_discovery.list_active_accounts().await?;

    let audit_log_path = std::path::Path::new("cwsweep-audit.jsonl");
    let audit_logger: Arc<dyn AuditWrite> = Arc::new(AuditLogger::open(audit_log_path)?);

    let logs_adapter: Arc<dyn DescribeLogGroupsOperations> = Arc::new(CloudWatchLogsAdapter);
    let api_adapter: Arc<dyn ActionApiOperations> = Arc::new(CloudWatchLogsAdapter);

    let app = CliApp {
        credential_provider: credential_provider.clone(),
        identity: identity.clone(),
        logs_client: logs_adapter,
        api_client: api_adapter,
        audit_logger,
        run_id,
    };

    let (aggregator, outcomes) = app.scan_all(&accounts, &cli.regions).await;
    for outcome in &outcomes {
        if let cwsweep::cli::AccountRegionOutcome::Failed {
            account_id,
            region,
            error,
        } = outcome
        {
            tracing::warn!(account_id, region, %error, "scan failed for account/region");
        }
    }

    println!("{}", CliApp::render_output(&aggregator, cli.output));

    if aggregator.is_empty() {
        println!("削除対象のロググループはありません。");
        return Ok(());
    }

    let selector = InteractiveSelector::new(InquireMultiSelectPrompt);
    let selected: Vec<LogGroupRecord> = CliApp::select(&aggregator, &selector)?;
    if selected.is_empty() {
        println!("選択されたロググループはありません。終了します。");
        return Ok(());
    }

    let action_kind = action_kind_from_prompt()?;
    let planned = CliApp::plan(&selected, action_kind);
    let total_bytes: i64 = selected.iter().map(|r| r.stored_bytes).sum();

    let presenter = ConfirmationPresenter::new(InquireConfirmPrompt);
    let confirmed = CliApp::confirm(&presenter, &planned, total_bytes);
    let Some(confirmed_actions) = confirmed else {
        println!("確認が得られなかったため、処理を中止します。");
        return Ok(());
    };

    let outcomes = app.execute(&confirmed_actions, cli.execute).await?;
    for outcome in outcomes {
        println!(
            "{}/{}/{}: success={} {}",
            outcome.action.account_id,
            outcome.action.region,
            outcome.action.log_group_name,
            outcome.success,
            outcome.error_message.unwrap_or_default()
        );
    }

    Ok(())
}
