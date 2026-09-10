//! `CliApp` コンポーネント。
//!
//! CLI引数解析と、他コンポーネントの呼び出し順序制御（パイプライン統括）を担う。
//! security-design.md: `--regions`は必須引数とし、既定値・全リージョン自動列挙の
//! フォールバックを持たせない。

use std::sync::Arc;

use clap::Parser;

use crate::aggregator::ScanAggregator;
use crate::confirmation::{ConfirmPrompt, ConfirmationPresenter};
use crate::credentials::CredentialProvider;
use crate::error::ScanError;
use crate::execution::{ActionApiOperations, ExecutionEngine, ExecutionOutcome};
use crate::identity::IdentityCheck;
use crate::org_discovery::AccountInfo;
use crate::output::{OutputFormat, OutputFormatter};
use crate::planner::{ActionKind, ActionPlanner, PlannedAction};
use crate::scanner::{DescribeLogGroupsOperations, LogGroupScanner};
use crate::selector::{InteractiveSelector, MultiSelectPrompt, SelectableItem};

/// cwsweep CLI引数。
#[derive(Parser, Debug, Clone)]
#[command(
    name = "cwsweep",
    version,
    about = "AWS Organization全体のCloudWatch Logs棚卸し・削除CLI"
)]
pub struct Cli {
    /// 対象リージョン（必須・複数指定可、カンマ区切りまたは複数回指定）。
    /// 自動列挙フォールバックは存在しない。
    #[arg(long, required = true, num_args = 1.., value_delimiter = ',')]
    pub regions: Vec<String>,

    /// メンバーアカウントへAssumeRoleする際のロール名。
    #[arg(long, default_value = "OrganizationAccountAccessRole")]
    pub role_name: String,

    /// 出力フォーマット（table | json）。
    #[arg(long, value_enum, default_value = "table")]
    pub output: OutputFormat,

    /// このフラグを明示的に指定しない限り、delete-log-group / put-retention-policy は
    /// 一切呼び出されない（dry-run既定）。
    #[arg(long, default_value_t = false)]
    pub execute: bool,

    /// 監査ログ（JSON Lines）の出力先パス。既定値は後方互換のため現行のカレント
    /// ディレクトリ直下`cwsweep-audit.jsonl`を維持する。無効化するオプションは
    /// 存在しない（project.md Mandated: 監査ログ出力に無効化オプションを設けない）。
    #[arg(long, default_value = "cwsweep-audit.jsonl")]
    pub audit_log_path: std::path::PathBuf,
}

impl Cli {
    pub fn parse_from_args<I, T>(args: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        Self::try_parse_from(args)
    }
}

/// 1アカウント×1リージョンのスキャン結果（成功/失敗を独立に保持する。reliability-design.md
/// のバルクヘッド分離）。
#[derive(Debug)]
pub enum AccountRegionOutcome {
    Success {
        account_id: String,
        region: String,
    },
    Failed {
        account_id: String,
        region: String,
        error: ScanError,
    },
}

/// パイプライン全体を統括するオーケストレータ。
///
/// 各コンポーネントはトレイト経由で注入されるため、単体テストでは実AWS接続なしに
/// 呼び出し順序・エラー時の継続動作を検証できる。
pub struct CliApp {
    pub credential_provider: Arc<CredentialProvider>,
    pub identity: Arc<dyn IdentityCheck>,
    pub logs_client: Arc<dyn DescribeLogGroupsOperations>,
    pub api_client: Arc<dyn ActionApiOperations>,
    pub audit_logger: Arc<dyn crate::audit::AuditWrite>,
    pub run_id: String,
}

impl CliApp {
    /// アカウント一覧×リージョン一覧をスキャンし、`ScanAggregator`へ集約する。
    /// 1アカウント×1リージョンの失敗は他の処理を止めない（バルクヘッド分離）。
    pub async fn scan_all(
        &self,
        accounts: &[AccountInfo],
        regions: &[String],
    ) -> (ScanAggregator, Vec<AccountRegionOutcome>) {
        let scanner = LogGroupScanner::new(self.logs_client.clone(), self.identity.clone());
        let mut aggregator = ScanAggregator::new();
        let mut outcomes = Vec::new();

        for account in accounts {
            for region in regions {
                let creds = match self
                    .credential_provider
                    .credentials_for(&account.account_id)
                    .await
                {
                    Ok(c) => c,
                    Err(e) => {
                        outcomes.push(AccountRegionOutcome::Failed {
                            account_id: account.account_id.clone(),
                            region: region.clone(),
                            error: ScanError::AssumeRole(e),
                        });
                        continue;
                    }
                };

                match scanner
                    .scan_account_region(&creds, &account.account_id, region)
                    .await
                {
                    Ok(records) => {
                        aggregator.add_all(records);
                        outcomes.push(AccountRegionOutcome::Success {
                            account_id: account.account_id.clone(),
                            region: region.clone(),
                        });
                    }
                    Err(error) => {
                        outcomes.push(AccountRegionOutcome::Failed {
                            account_id: account.account_id.clone(),
                            region: region.clone(),
                            error,
                        });
                    }
                }
            }
        }

        (aggregator, outcomes)
    }

    pub fn render_output(aggregator: &ScanAggregator, format: OutputFormat) -> String {
        OutputFormatter::format(aggregator, format)
    }

    /// R-03: 終了コード判定方針。
    ///
    /// 対象となったアカウント×リージョンの組み合わせが1件以上あり、かつ
    /// そのすべてが失敗した場合にのみ`true`（＝全滅）を返す。1件でも成功が
    /// あれば`false`（正常終了扱い）。対象自体が0件（accounts/regionsが空）の
    /// 場合も`false`とする——これはスキャン失敗ではないため。
    ///
    /// CI/自動化からの呼び出しで「スキャン全滅」と「本当に削除対象0件」を
    /// 終了コードで区別できるようにするための判定である。一部成功・一部失敗の
    /// 場合は0（成功）終了とし、失敗したアカウント/リージョンは`tracing::warn!`
    /// によるログ出力で個別に把握する。
    pub fn scan_fully_failed(outcomes: &[AccountRegionOutcome]) -> bool {
        !outcomes.is_empty()
            && outcomes
                .iter()
                .all(|o| matches!(o, AccountRegionOutcome::Failed { .. }))
    }

    pub fn select<P: MultiSelectPrompt>(
        aggregator: &ScanAggregator,
        selector: &InteractiveSelector<P>,
    ) -> Result<Vec<crate::aggregator::LogGroupRecord>, crate::selector::SelectorError> {
        let items: Vec<SelectableItem> = aggregator
            .records()
            .iter()
            .map(|r| SelectableItem {
                label: format!("{}/{}/{}", r.account_id, r.region, r.log_group_name),
                record: r.clone(),
            })
            .collect();
        selector.select(&items)
    }

    pub fn plan(
        selected: &[crate::aggregator::LogGroupRecord],
        action_kind: ActionKind,
    ) -> Vec<PlannedAction> {
        ActionPlanner::plan(selected, action_kind)
    }

    pub fn confirm<P: ConfirmPrompt>(
        presenter: &ConfirmationPresenter<P>,
        actions: &[PlannedAction],
        total_bytes: i64,
    ) -> Option<Vec<PlannedAction>> {
        presenter.confirm(actions, total_bytes)
    }

    pub async fn execute(
        &self,
        confirmed_actions: &[PlannedAction],
        execute_flag: bool,
    ) -> Result<Vec<ExecutionOutcome>, crate::error::ExecutionError> {
        let engine = ExecutionEngine::new(
            self.api_client.clone(),
            self.identity.clone(),
            self.audit_logger.clone(),
            self.run_id.clone(),
        );
        let provider = self.credential_provider.clone();
        // credentials_by_accountはクロージャとして渡す必要があるため、事前に解決しておく。
        let mut resolved = std::collections::HashMap::new();
        for action in confirmed_actions {
            if let std::collections::hash_map::Entry::Vacant(entry) =
                resolved.entry(action.account_id.clone())
            {
                let creds = provider.credentials_for(&action.account_id).await?;
                entry.insert(creds);
            }
        }
        engine
            .execute_plan(
                confirmed_actions,
                |account_id| resolved.get(account_id).cloned(),
                execute_flag,
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regions_flag_is_required_and_missing_it_fails_parsing() {
        let result = Cli::parse_from_args(["cwsweep"]);
        assert!(result.is_err());
    }

    #[test]
    fn regions_flag_accepts_comma_separated_values() {
        let cli = Cli::parse_from_args(["cwsweep", "--regions", "us-east-1,us-west-2"]).unwrap();
        assert_eq!(cli.regions, vec!["us-east-1", "us-west-2"]);
    }

    #[test]
    fn execute_flag_defaults_to_false_when_not_supplied() {
        let cli = Cli::parse_from_args(["cwsweep", "--regions", "us-east-1"]).unwrap();
        assert!(!cli.execute);
    }

    #[test]
    fn execute_flag_is_true_only_when_explicitly_supplied() {
        let cli = Cli::parse_from_args(["cwsweep", "--regions", "us-east-1", "--execute"]).unwrap();
        assert!(cli.execute);
    }

    #[test]
    fn role_name_defaults_to_organization_account_access_role() {
        let cli = Cli::parse_from_args(["cwsweep", "--regions", "us-east-1"]).unwrap();
        assert_eq!(cli.role_name, "OrganizationAccountAccessRole");
    }

    #[test]
    fn role_name_can_be_overridden() {
        let cli = Cli::parse_from_args([
            "cwsweep",
            "--regions",
            "us-east-1",
            "--role-name",
            "CustomRole",
        ])
        .unwrap();
        assert_eq!(cli.role_name, "CustomRole");
    }

    #[test]
    fn output_format_defaults_to_table() {
        let cli = Cli::parse_from_args(["cwsweep", "--regions", "us-east-1"]).unwrap();
        assert_eq!(cli.output, OutputFormat::Table);
    }

    // --- CliApp オーケストレーション層のテスト ---

    use crate::audit::{AuditLogger, AuditWrite};
    use crate::confirmation::ConfirmationSummary;
    use crate::credentials::{AccountCredentials, AssumeRoleOperations};
    use crate::error::{AssumeRoleError, IdentityError, IdentityMismatchError};
    use crate::org_discovery::AccountStatus;
    use crate::scanner::{LogGroupPage, RawLogGroup};
    use crate::selector::SelectorError;
    use async_trait::async_trait;
    use secrecy::SecretString;

    const MANAGEMENT_ACCOUNT_ID: &str = "111111111111";
    const MEMBER_ACCOUNT_ID: &str = "222222222222";
    const REGION: &str = "us-east-1";

    fn test_creds(account_id: &str) -> AccountCredentials {
        AccountCredentials {
            account_id: account_id.to_string(),
            access_key_id: "AKIAFIXTURE".to_string(),
            secret_access_key: SecretString::from("secret".to_string()),
            session_token: SecretString::from("token".to_string()),
            expiration: None,
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
            Ok(test_creds(account_id))
        }
    }

    struct StubIdentityOk;
    #[async_trait]
    impl IdentityCheck for StubIdentityOk {
        async fn verify(&self, _c: &AccountCredentials, _e: &str) -> Result<(), IdentityError> {
            Ok(())
        }
    }

    struct StubIdentityFailForMember;
    #[async_trait]
    impl IdentityCheck for StubIdentityFailForMember {
        async fn verify(
            &self,
            creds: &AccountCredentials,
            expected: &str,
        ) -> Result<(), IdentityError> {
            if creds.account_id == MEMBER_ACCOUNT_ID {
                Err(IdentityMismatchError {
                    expected: expected.to_string(),
                    actual: Some("999999999999".to_string()),
                }
                .into())
            } else {
                Ok(())
            }
        }
    }

    struct StubDescribeLogGroups;
    #[async_trait]
    impl DescribeLogGroupsOperations for StubDescribeLogGroups {
        async fn describe_log_groups_page(
            &self,
            creds: &AccountCredentials,
            _region: &str,
            _next_token: Option<String>,
        ) -> Result<LogGroupPage, String> {
            Ok(LogGroupPage {
                log_groups: vec![RawLogGroup {
                    name: format!("/aws/lambda/{}", creds.account_id),
                    stored_bytes: 42,
                    retention_in_days: None,
                }],
                next_token: None,
            })
        }
    }

    struct RecordingApi {
        deletes: std::sync::atomic::AtomicUsize,
    }
    #[async_trait]
    impl ActionApiOperations for RecordingApi {
        async fn delete_log_group(
            &self,
            _c: &AccountCredentials,
            _r: &str,
            _l: &str,
        ) -> Result<(), String> {
            self.deletes
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
        async fn put_retention_policy(
            &self,
            _c: &AccountCredentials,
            _r: &str,
            _l: &str,
            _d: i32,
        ) -> Result<(), String> {
            Ok(())
        }
    }

    fn accounts() -> Vec<AccountInfo> {
        vec![
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
        ]
    }

    fn build_app(
        identity: Arc<dyn IdentityCheck>,
        api_client: Arc<dyn ActionApiOperations>,
        dir: &tempfile::TempDir,
    ) -> CliApp {
        let credential_provider = Arc::new(CredentialProvider::with_default_role_name(
            MANAGEMENT_ACCOUNT_ID,
            Arc::new(StubAssumeRole),
            test_creds(MANAGEMENT_ACCOUNT_ID),
        ));
        let audit_logger: Arc<dyn AuditWrite> =
            Arc::new(AuditLogger::open(&dir.path().join("audit.jsonl")).unwrap());
        CliApp {
            credential_provider,
            identity,
            logs_client: Arc::new(StubDescribeLogGroups),
            api_client,
            audit_logger,
            run_id: "test-run".to_string(),
        }
    }

    #[tokio::test]
    async fn scan_all_aggregates_records_across_accounts_and_regions() {
        let dir = tempfile::tempdir().unwrap();
        let app = build_app(
            Arc::new(StubIdentityOk),
            Arc::new(RecordingApi {
                deletes: std::sync::atomic::AtomicUsize::new(0),
            }),
            &dir,
        );

        let (aggregator, outcomes) = app.scan_all(&accounts(), &[REGION.to_string()]).await;

        assert_eq!(aggregator.len(), 2);
        assert!(outcomes
            .iter()
            .all(|o| matches!(o, AccountRegionOutcome::Success { .. })));
    }

    #[tokio::test]
    async fn scan_all_continues_past_a_single_account_failure_bulkhead() {
        let dir = tempfile::tempdir().unwrap();
        let app = build_app(
            Arc::new(StubIdentityFailForMember),
            Arc::new(RecordingApi {
                deletes: std::sync::atomic::AtomicUsize::new(0),
            }),
            &dir,
        );

        let (aggregator, outcomes) = app.scan_all(&accounts(), &[REGION.to_string()]).await;

        // 管理アカウント分は成功、メンバーアカウント分は失敗しても処理全体は止まらない。
        assert_eq!(aggregator.len(), 1);
        let failed = outcomes
            .iter()
            .filter(|o| matches!(o, AccountRegionOutcome::Failed { .. }))
            .count();
        assert_eq!(failed, 1);
    }

    #[test]
    fn render_output_delegates_to_output_formatter() {
        let mut aggregator = ScanAggregator::new();
        aggregator.add_all(vec![crate::aggregator::LogGroupRecord {
            account_id: MANAGEMENT_ACCOUNT_ID.to_string(),
            region: REGION.to_string(),
            log_group_name: "/a".to_string(),
            stored_bytes: 10,
            retention_in_days: None,
        }]);

        let out = CliApp::render_output(&aggregator, OutputFormat::Json);

        assert!(out.contains(MANAGEMENT_ACCOUNT_ID));
    }

    struct FixedSelectAll;
    impl MultiSelectPrompt for FixedSelectAll {
        fn prompt(
            &self,
            items: &[SelectableItem],
            _defaults: &[usize],
        ) -> Result<Vec<usize>, SelectorError> {
            Ok((0..items.len()).collect())
        }
    }

    #[test]
    fn select_returns_every_record_when_prompt_picks_all() {
        let mut aggregator = ScanAggregator::new();
        aggregator.add_all(vec![crate::aggregator::LogGroupRecord {
            account_id: MANAGEMENT_ACCOUNT_ID.to_string(),
            region: REGION.to_string(),
            log_group_name: "/a".to_string(),
            stored_bytes: 10,
            retention_in_days: None,
        }]);
        let selector = InteractiveSelector::new(FixedSelectAll);

        let selected = CliApp::select(&aggregator, &selector).unwrap();

        assert_eq!(selected.len(), 1);
    }

    #[test]
    fn plan_delegates_to_action_planner() {
        let records = vec![crate::aggregator::LogGroupRecord {
            account_id: MANAGEMENT_ACCOUNT_ID.to_string(),
            region: REGION.to_string(),
            log_group_name: "/a".to_string(),
            stored_bytes: 10,
            retention_in_days: None,
        }];

        let planned = CliApp::plan(&records, ActionKind::Delete);

        assert_eq!(planned.len(), 1);
    }

    struct AlwaysConfirmPrompt;
    impl ConfirmPrompt for AlwaysConfirmPrompt {
        fn confirm(&self, _summary: &ConfirmationSummary) -> bool {
            true
        }
    }

    #[test]
    fn confirm_delegates_to_confirmation_presenter() {
        let planned = vec![PlannedAction {
            account_id: MANAGEMENT_ACCOUNT_ID.to_string(),
            region: REGION.to_string(),
            log_group_name: "/a".to_string(),
            action_kind: ActionKind::Delete,
            confirmed: false,
        }];
        let presenter = ConfirmationPresenter::new(AlwaysConfirmPrompt);

        let confirmed = CliApp::confirm(&presenter, &planned, 10).unwrap();

        assert!(confirmed[0].confirmed);
    }

    #[tokio::test]
    async fn execute_resolves_credentials_and_delegates_to_execution_engine() {
        let dir = tempfile::tempdir().unwrap();
        let api = Arc::new(RecordingApi {
            deletes: std::sync::atomic::AtomicUsize::new(0),
        });
        let app = build_app(Arc::new(StubIdentityOk), api.clone(), &dir);
        let confirmed = vec![PlannedAction {
            account_id: MEMBER_ACCOUNT_ID.to_string(),
            region: REGION.to_string(),
            log_group_name: "/a".to_string(),
            action_kind: ActionKind::Delete,
            confirmed: true,
        }];

        let outcomes = app.execute(&confirmed, true).await.unwrap();

        assert!(outcomes[0].success);
        assert_eq!(api.deletes.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn execute_dry_run_default_never_calls_delete_api() {
        let dir = tempfile::tempdir().unwrap();
        let api = Arc::new(RecordingApi {
            deletes: std::sync::atomic::AtomicUsize::new(0),
        });
        let app = build_app(Arc::new(StubIdentityOk), api.clone(), &dir);
        let confirmed = vec![PlannedAction {
            account_id: MEMBER_ACCOUNT_ID.to_string(),
            region: REGION.to_string(),
            log_group_name: "/a".to_string(),
            action_kind: ActionKind::Delete,
            confirmed: true,
        }];

        let outcomes = app.execute(&confirmed, false).await.unwrap();

        assert!(!outcomes[0].success);
        assert_eq!(api.deletes.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    #[test]
    fn output_format_json_can_be_selected() {
        let cli = Cli::parse_from_args(["cwsweep", "--regions", "us-east-1", "--output", "json"])
            .unwrap();
        assert_eq!(cli.output, OutputFormat::Json);
    }

    #[test]
    fn audit_log_path_defaults_to_cwsweep_audit_jsonl_for_backward_compat() {
        let cli = Cli::parse_from_args(["cwsweep", "--regions", "us-east-1"]).unwrap();
        assert_eq!(
            cli.audit_log_path,
            std::path::PathBuf::from("cwsweep-audit.jsonl")
        );
    }

    #[test]
    fn audit_log_path_can_be_overridden() {
        let cli = Cli::parse_from_args([
            "cwsweep",
            "--regions",
            "us-east-1",
            "--audit-log-path",
            "/var/log/cwsweep/audit.jsonl",
        ])
        .unwrap();
        assert_eq!(
            cli.audit_log_path,
            std::path::PathBuf::from("/var/log/cwsweep/audit.jsonl")
        );
    }

    // --- R-03: scan_fully_failed（終了コード判定）のテスト ---

    fn failed_outcome(account_id: &str, region: &str) -> AccountRegionOutcome {
        AccountRegionOutcome::Failed {
            account_id: account_id.to_string(),
            region: region.to_string(),
            error: ScanError::AssumeRole(AssumeRoleError {
                account_id: account_id.to_string(),
                role_name: "role".to_string(),
                message: "boom".to_string(),
            }),
        }
    }

    fn success_outcome(account_id: &str, region: &str) -> AccountRegionOutcome {
        AccountRegionOutcome::Success {
            account_id: account_id.to_string(),
            region: region.to_string(),
        }
    }

    #[test]
    fn scan_fully_failed_is_true_when_every_outcome_failed() {
        let outcomes = vec![
            failed_outcome(MANAGEMENT_ACCOUNT_ID, REGION),
            failed_outcome(MEMBER_ACCOUNT_ID, REGION),
        ];
        assert!(CliApp::scan_fully_failed(&outcomes));
    }

    #[test]
    fn scan_fully_failed_is_false_when_at_least_one_outcome_succeeded() {
        let outcomes = vec![
            failed_outcome(MANAGEMENT_ACCOUNT_ID, REGION),
            success_outcome(MEMBER_ACCOUNT_ID, REGION),
        ];
        assert!(!CliApp::scan_fully_failed(&outcomes));
    }

    #[test]
    fn scan_fully_failed_is_false_when_outcomes_are_empty() {
        // accounts/regionsの組み合わせが0件の場合は「スキャン失敗」ではないため false。
        let outcomes: Vec<AccountRegionOutcome> = Vec::new();
        assert!(!CliApp::scan_fully_failed(&outcomes));
    }

    #[test]
    fn scan_fully_failed_is_false_when_all_outcomes_succeeded() {
        let outcomes = vec![
            success_outcome(MANAGEMENT_ACCOUNT_ID, REGION),
            success_outcome(MEMBER_ACCOUNT_ID, REGION),
        ];
        assert!(!CliApp::scan_fully_failed(&outcomes));
    }
}
