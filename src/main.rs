//! cwsweep バイナリエントリポイント。
//!
//! 実AWS SDKクライアントを構築し、`cwsweep`ライブラリのコンポーネントを配線する。
//! project.md Mandated: 管理アカウント自身へはAssumeRoleを行わず現在の認証情報をそのまま使用し、
//! メンバーアカウントに対してのみAssumeRoleを行う。

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::io::IsTerminal;
use std::sync::Arc;

use async_trait::async_trait;
use aws_config::retry::RetryConfig;
use aws_config::BehaviorVersion;
use secrecy::{ExposeSecret, SecretString};
use uuid::Uuid;

use aws_sdk_sts::config::ProvideCredentials;
use cwsweep::audit::{AuditLogger, AuditReader, AuditWrite};
use cwsweep::cli::{run_audit, CleanInteraction, Cli, CliApp, Commands, ExitDisposition, ScanApp};
use cwsweep::confirmation::{ConfirmPrompt, ConfirmationPresenter, ConfirmationSummary};
use cwsweep::credentials::{AccountCredentials, AssumeRoleOperations, CredentialProvider};
use cwsweep::error::{AssumeRoleError, CallerIdentityCallError, OrgDiscoveryError};
use cwsweep::execution::ActionApiOperations;
use cwsweep::identity::{CallerIdentityOperations, IdentityCheck, IdentityVerifier};
use cwsweep::org_discovery::{AccountInfo, AccountStatus, ListAccountsOperations, OrgDiscovery};
use cwsweep::planner::ActionKind;
use cwsweep::regions::{
    region_spec, resolve_regions, ListRegionsError, ListRegionsOperations, RegionPrompt,
    RegionResolveError, RegionSpec,
};
use cwsweep::scanner::{DescribeLogGroupsOperations, LogGroupPage, RawLogGroup};
use cwsweep::selector::{InteractiveSelector, MultiSelectPrompt, SelectableItem, SelectorError};

const RETRY_MAX_ATTEMPTS: u32 = 3;

fn to_sdk_credentials(creds: &AccountCredentials) -> aws_sdk_sts::config::Credentials {
    // `session_token`が`None`（長期IAMクレデンシャル等、セッショントークンを伴わない場合）は
    // `None`のままAWS SDKへ渡す。`Some(String::new())`（空文字列）とは意味が異なるため、
    // ここで`unwrap_or_default`のような変換を行ってはならない。
    aws_sdk_sts::config::Credentials::new(
        creds.access_key_id.clone(),
        creds.secret_access_key.expose_secret().to_string(),
        creds
            .session_token
            .as_ref()
            .map(|t| t.expose_secret().to_string()),
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

/// グローバルサービス（STS / Organizations）のエンドポイントリージョン。
/// `AWS_REGION`等の環境設定に依存せず常にこのリージョンへ接続する。
const GLOBAL_SERVICE_REGION: &str = "us-east-1";

/// R-02: STSはグローバルサービスであり、`us-east-1`をエンドポイントリージョンとして
/// 固定してもAWS標準パーティション（`aws`、商用リージョン）内であれば`--regions`に
/// 何を指定しても正しく動作する。v1は商用パーティションのみをスコープとしており
/// （README.md「スコープ制約」参照）、AWS GovCloudや中国リージョン等の非商用
/// パーティションは対象外である。
fn sts_client_for(creds: &AccountCredentials) -> aws_sdk_sts::Client {
    let config = aws_sdk_sts::Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(aws_sdk_sts::config::Region::new(GLOBAL_SERVICE_REGION))
        .credentials_provider(to_sdk_credentials(creds))
        .retry_config(RetryConfig::standard().with_max_attempts(RETRY_MAX_ATTEMPTS))
        .build();
    aws_sdk_sts::Client::from_conf(config)
}

/// AWS SDKエラーをソースチェーン込みの1行文字列にする。`to_string()`では
/// "service error" / "dispatch failure" 等の最上位分類しか得られず原因が分からない。
fn sdk_error_message(e: &(dyn std::error::Error + 'static)) -> String {
    let mut parts = vec![e.to_string()];
    let mut source = e.source();
    while let Some(s) = source {
        let text = s.to_string();
        if parts.last() != Some(&text) {
            parts.push(text);
        }
        source = s.source();
    }
    parts.join(": ")
}

/// OrganizationsもSTSと同様のグローバルサービスであり、`--regions`や環境の既定
/// リージョンとは独立に`us-east-1`へ固定する。
fn organizations_client_for(creds: &AccountCredentials) -> aws_sdk_organizations::Client {
    let config = aws_sdk_organizations::Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(aws_sdk_organizations::config::Region::new(
            GLOBAL_SERVICE_REGION,
        ))
        .credentials_provider(to_sdk_credentials(creds))
        .retry_config(RetryConfig::standard().with_max_attempts(RETRY_MAX_ATTEMPTS))
        .build();
    aws_sdk_organizations::Client::from_conf(config)
}

/// `ec2:describe-regions` はどのリージョンのエンドポイントでも全リージョンを返すため、
/// STS / Organizations と同じく`GLOBAL_SERVICE_REGION`へ固定する。
fn ec2_client_for(creds: &AccountCredentials) -> aws_sdk_ec2::Client {
    let config = aws_sdk_ec2::Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(aws_sdk_ec2::config::Region::new(GLOBAL_SERVICE_REGION))
        .credentials_provider(to_sdk_credentials(creds))
        .retry_config(RetryConfig::standard().with_max_attempts(RETRY_MAX_ATTEMPTS))
        .build();
    aws_sdk_ec2::Client::from_conf(config)
}

/// `ec2:describe-regions` の実AWS SDK実装。オプトインリージョン（未有効化含む）も
/// 列挙する。商用パーティションへの絞り込みは`cwsweep::regions`側で行う。
struct Ec2DescribeRegionsAdapter {
    client: aws_sdk_ec2::Client,
}

#[async_trait]
impl ListRegionsOperations for Ec2DescribeRegionsAdapter {
    async fn list_commercial_regions(&self) -> Result<Vec<String>, ListRegionsError> {
        let output = self
            .client
            .describe_regions()
            .all_regions(true)
            .send()
            .await
            .map_err(|e| ListRegionsError {
                message: sdk_error_message(&e),
            })?;
        Ok(output
            .regions()
            .iter()
            .filter_map(|r| r.region_name().map(str::to_string))
            .collect())
    }
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
                message: sdk_error_message(&e),
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
            session_token: Some(SecretString::from(creds.session_token)),
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
                    message: sdk_error_message(&e),
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
            let output = request.send().await.map_err(|e| {
                let not_in_organization = e
                    .as_service_error()
                    .map(|se| se.is_aws_organizations_not_in_use_exception())
                    .unwrap_or(false);
                OrgDiscoveryError {
                    message: sdk_error_message(&e),
                    not_in_organization,
                }
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
        let output = request.send().await.map_err(|e| sdk_error_message(&e))?;

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
            .map_err(|e| sdk_error_message(&e))
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
            .map_err(|e| sdk_error_message(&e))
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
        // CodeRabbit指摘#4: `--regions`に同一リージョンを複数回指定できる
        // （`ScanAggregator::add_all`は重複除去しない）ため、同一ラベルの項目が
        // 複数件存在しうる。`prompt()`（ラベル文字列のみを返す）で選択を復元すると
        // ラベル文字列一致になり、同じラベルを持つ項目が全て選択扱いになってしまう。
        // `raw_prompt()`は選択された項目それぞれの元のインデックス(`ListOption::index`)を
        // 返すため、ラベルの重複に関わらず選択されたインデックスのみを正しく復元できる。
        let selected =
            inquire::MultiSelect::new("削除・retention変更の対象を選択してください:", options)
                .with_default(default_selected_indices)
                .raw_prompt()
                .map_err(|e| SelectorError(e.to_string()))?;
        Ok(selected.into_iter().map(|opt| opt.index).collect())
    }
}

/// `inquire`ベースのリージョン選択・`all`確認UI実装。
struct InquireRegionPrompt;

impl RegionPrompt for InquireRegionPrompt {
    fn confirm_all(&self, regions: &[String]) -> bool {
        inquire::Confirm::new(&format!(
            "商用リージョン {} 件をすべて対象にします。続行しますか？",
            regions.len()
        ))
        .with_default(false)
        .prompt()
        .unwrap_or(false)
    }

    fn select(&self, regions: &[String]) -> Result<Vec<String>, String> {
        // 初期状態は全チェックOFF（ロググループ選択と同じ安全不変条件）。
        inquire::MultiSelect::new(
            "--regions が未指定です。スキャン対象のリージョンを選択してください:",
            regions.to_vec(),
        )
        .prompt()
        .map_err(|e| e.to_string())
    }
}

/// `inquire`ベースの実最終確認プロンプト実装。
struct InquireConfirmPrompt {
    execute: bool,
}

fn execution_mode_label(execute: bool) -> &'static str {
    if execute {
        "EXECUTE（実際にAWS APIを呼び出します）"
    } else {
        "DRY-RUN（--execute未指定のため、AWS APIは呼び出しません）"
    }
}

fn confirm_question(execute: bool) -> &'static str {
    if execute {
        "上記の内容で実行してよろしいですか？"
    } else {
        "上記の内容でdry-runを続行しますか？（実際の変更は行われません）"
    }
}

impl ConfirmPrompt for InquireConfirmPrompt {
    fn confirm(&self, summary: &ConfirmationSummary) -> bool {
        println!("実行モード: {}", execution_mode_label(self.execute));
        println!("対象アカウントID: {:?}", summary.account_ids);
        println!("対象リージョン: {:?}", summary.regions);
        println!("対象ログループ: {:?}", summary.log_group_names);
        println!("合計バイト数: {}", summary.total_bytes);
        // CodeRabbit指摘#3: アカウント/リージョン/ログループ名の独立配列だけでは、
        // 異なるアカウント/リージョンに同名のロググループがある場合に対応関係が
        // 失われる。`targets`は1件ごとの対応関係（＋アクション種別）を保持しており、
        // ここで対象一覧として1行ずつ再掲する。
        println!("--- 実行対象一覧（対応関係付き） ---");
        for target in &summary.targets {
            let action_label = match target.action_kind {
                cwsweep::planner::ActionKind::Delete => "delete".to_string(),
                cwsweep::planner::ActionKind::SetRetention { days } => {
                    format!("set-retention({days}日)")
                }
            };
            println!(
                "  - account={} region={} log_group={} action={}",
                target.account_id, target.region, target.log_group_name, action_label
            );
        }
        inquire::Confirm::new(confirm_question(self.execute))
            .with_default(false)
            .prompt()
            .unwrap_or(false)
    }
}

/// Organizationのメンバーではないアカウントに対する単一アカウントモードの
/// フォールバック対象を組み立てる。呼び出し元アカウント自身のみを対象とし、
/// `AccountStatus::Active`として扱う（AssumeRoleは行わず、`CredentialProvider`が
/// 管理アカウント自身の現行クレデンシャルをそのまま用いる）。
fn single_account_fallback(management_account_id: &str) -> Vec<AccountInfo> {
    vec![AccountInfo {
        account_id: management_account_id.to_string(),
        account_name: String::new(),
        status: AccountStatus::Active,
    }]
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

/// scan / clean 共通の AWS 配線結果。
struct AwsWiring {
    credential_provider: Arc<CredentialProvider>,
    identity: Arc<dyn IdentityCheck>,
    accounts: Vec<AccountInfo>,
    regions: Vec<String>,
}

/// 管理アカウントのクレデンシャル解決 → Identity 取得 → リージョン解決 → Organization アカウント列挙。
/// `audit` サブコマンドからは呼ばれない（AWS へ一切アクセスしない）。
async fn wire_aws(
    role_name: &str,
    regions: &[String],
) -> Result<AwsWiring, Box<dyn std::error::Error>> {
    let stdin_is_tty = std::io::stdin().is_terminal();
    let spec = region_spec(regions);
    // 非TTYで`--regions`未指定の場合は、AWSへ一切アクセスする前に失敗させる。
    if spec == RegionSpec::Unspecified && !stdin_is_tty {
        return Err(RegionResolveError::MissingInNonInteractive
            .to_string()
            .into());
    }
    // `base_config`はクレデンシャル解決のみに用いる。各サービスクライアントは
    // `--regions`（CloudWatch Logs）または`GLOBAL_SERVICE_REGION`（STS / Organizations）
    // を明示するため、ここでの既定リージョン解決（IMDS問い合わせ等）は不要。
    let base_config = aws_config::defaults(BehaviorVersion::latest())
        .region(aws_config::Region::new(GLOBAL_SERVICE_REGION))
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
        session_token: raw_creds
            .session_token()
            .map(|t| SecretString::from(t.to_string())),
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

    // リージョン解決（`all`の列挙・未指定時の対話式選択）は管理アカウントの
    // 資格情報で行い、アカウント列挙より前に確定させる。
    let regions = resolve_regions(
        spec,
        &Ec2DescribeRegionsAdapter {
            client: ec2_client_for(&management_credentials),
        },
        stdin_is_tty,
        &InquireRegionPrompt,
    )
    .await
    .map_err(|e| e.to_string())?;

    let sts_client = sts_client_for(&management_credentials);
    let organizations_client = organizations_client_for(&management_credentials);
    let assume_role_client: Arc<dyn AssumeRoleOperations> =
        Arc::new(StsAssumeRoleAdapter { client: sts_client });
    let credential_provider = Arc::new(CredentialProvider::new(
        management_account_id.clone(),
        role_name.to_string(),
        assume_role_client,
        management_credentials,
    ));

    let identity: Arc<dyn IdentityCheck> =
        Arc::new(IdentityVerifier::new(StsCallerIdentityAdapter));

    let org_discovery = OrgDiscovery::new(OrganizationsListAccountsAdapter {
        client: organizations_client,
    });
    // R-05: 呼び出し元アカウントがAWS Organizationのメンバーではない場合
    // （`AwsOrganizationsNotInUseException`）、Organization横断の前提が成立しないだけで
    // 単独アカウントに対する棚卸し・削除自体は成立するため、呼び出し元アカウント単体を
    // 対象とする単一アカウントモードにフォールバックする。それ以外の失敗
    // （権限不足等）は従来どおり致命的エラーとして扱う。
    let accounts = match org_discovery.list_active_accounts().await {
        Ok(accounts) => accounts,
        Err(err) if err.not_in_organization => {
            eprintln!(
                "このアカウント（{management_account_id}）はAWS Organizationのメンバーではないため、単一アカウントモードで実行します。"
            );
            single_account_fallback(&management_account_id)
        }
        Err(err) => return Err(err.into()),
    };

    Ok(AwsWiring {
        credential_provider,
        identity,
        accounts,
        regions,
    })
}

fn exit_with(disposition: ExitDisposition) -> Result<(), Box<dyn std::error::Error>> {
    match disposition {
        ExitDisposition::Success => Ok(()),
        ExitDisposition::ScanFullyFailed { attempted } => Err(format!(
            "全{attempted}件のアカウント×リージョンの組み合わせでスキャンに失敗しました。詳細は上記の警告ログを確認してください。"
        )
        .into()),
        ExitDisposition::Error(message) => Err(message.into()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(std::io::stderr().is_terminal())
        .init();

    let cli = Cli::parse_args();
    let mut stdout = std::io::stdout();

    match cli.command {
        Commands::Scan(args) => {
            let wiring = wire_aws(&args.role_name, &args.regions).await?;
            let app = ScanApp {
                credential_provider: wiring.credential_provider,
                identity: wiring.identity,
                logs_client: Arc::new(CloudWatchLogsAdapter),
            };
            let disposition = app
                .run_scan(&wiring.accounts, &wiring.regions, args.output, &mut stdout)
                .await;
            exit_with(disposition)
        }
        Commands::Clean(args) => {
            // 監査ログ出力先は最初にオープンする（フェイルクローズ）。無効化オプションは
            // project.md Mandatedにより設けない。
            let audit_logger: Arc<dyn AuditWrite> =
                Arc::new(AuditLogger::open(&args.audit_log_path)?);
            let wiring = wire_aws(&args.role_name, &args.regions).await?;
            let app = CliApp {
                credential_provider: wiring.credential_provider,
                identity: wiring.identity,
                logs_client: Arc::new(CloudWatchLogsAdapter),
                api_client: Arc::new(CloudWatchLogsAdapter),
                audit_logger,
                run_id: Uuid::new_v4().to_string(),
            };
            let selector = InteractiveSelector::new(InquireMultiSelectPrompt);
            let presenter = ConfirmationPresenter::new(InquireConfirmPrompt {
                execute: args.execute,
            });
            let interaction = CleanInteraction {
                selector: &selector,
                presenter: &presenter,
                choose_action: &action_kind_from_prompt,
            };
            let disposition = app
                .run_clean(
                    &wiring.accounts,
                    &wiring.regions,
                    args.execute,
                    std::io::stdin().is_terminal(),
                    interaction,
                    &mut stdout,
                )
                .await;
            exit_with(disposition)
        }
        Commands::Audit(args) => {
            let reader = AuditReader::new(args.audit_log_path);
            exit_with(run_audit(&reader, args.output, &mut stdout))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirmation_texts_distinguish_dry_run_from_execute() {
        assert!(execution_mode_label(false).contains("DRY-RUN"));
        assert!(execution_mode_label(true).contains("EXECUTE"));
        assert!(confirm_question(false).contains("dry-run"));
        assert!(!confirm_question(true).contains("dry-run"));
    }

    fn creds_with_session_token(session_token: Option<&str>) -> AccountCredentials {
        AccountCredentials {
            account_id: "111111111111".to_string(),
            access_key_id: "AKIAFIXTURE".to_string(),
            secret_access_key: SecretString::from("secret".to_string()),
            session_token: session_token.map(|t| SecretString::from(t.to_string())),
            expiration: None,
        }
    }

    // CodeRabbit指摘#2: `session_token`が`None`の場合、AWS SDKには`Some("")`ではなく
    // `None`のまま渡す必要がある（`None`と`Some("")`はAWS SDK上で意味が異なり、長期IAM
    // クレデンシャルでの認証が`Some("")`だと失敗しうる）。

    #[test]
    fn to_sdk_credentials_passes_none_session_token_through_as_none() {
        let creds = creds_with_session_token(None);

        let sdk_creds = to_sdk_credentials(&creds);

        assert_eq!(sdk_creds.session_token(), None);
    }

    #[test]
    fn to_sdk_credentials_passes_some_session_token_through_as_some() {
        let creds = creds_with_session_token(Some("a-session-token"));

        let sdk_creds = to_sdk_credentials(&creds);

        assert_eq!(sdk_creds.session_token(), Some("a-session-token"));
    }

    // 単一アカウントモードのフォールバック（AwsOrganizationsNotInUseExceptionを検知した際に
    // Organization APIを介さず呼び出し元アカウント自身のみを対象とする）のテスト。

    #[test]
    fn single_account_fallback_targets_only_the_calling_account() {
        let accounts = single_account_fallback("111111111111");

        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account_id, "111111111111");
        assert_eq!(accounts[0].status, AccountStatus::Active);
    }
}
