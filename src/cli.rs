//! `CliApp` コンポーネント。
//!
//! CLI引数解析（`scan` / `clean` / `audit` / `retention set` サブコマンド）と、サブコマンドごとの
//! ハンドラ（`run_scan` / `run_clean` / `run_audit` / `run_set_retention`）を担う。
//! `--regions`は明示リージョン・`all`（商用全リージョン）・未指定（TTYのみ対話式選択、
//! 非TTYはエラー）を受け付ける。解決ロジックは`crate::regions`を参照。
//! 旧フラグ方式（`--scan-only`／トップレベル`--execute`）への互換エイリアスは提供しない（FR6.1）。

use std::io::Write;
use std::sync::Arc;

use clap::{Args, Parser, Subcommand};

use crate::aggregator::ScanAggregator;
use crate::audit::AuditRead;
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

/// cwsweep CLI引数。サブコマンド未指定はパースエラー（BR1.1）。
#[derive(Parser, Debug, Clone)]
#[command(
    name = "cwsweep",
    version,
    about = "AWS Organization全体のCloudWatch Logs棚卸し・削除CLI"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// ロググループを棚卸しして表示する（読み取り専用。監査ログは書かない）。
    Scan(ScanArgs),
    /// スキャン→選択→確認→（--execute時のみ）実行。監査ログを必ず記録する。
    Clean(CleanArgs),
    /// 監査ログ（JSON Lines）を読み取り専用で表示する。
    Audit(AuditArgs),
    /// retention（保持期間）に関する操作。削除は一切行わない。
    Retention(RetentionArgs),
}

#[derive(Args, Debug, Clone)]
pub struct RetentionArgs {
    #[command(subcommand)]
    pub command: RetentionCommands,
}

/// `cwsweep retention <sub>`。将来 `get` / `clear` 等を追加する拡張点。
#[derive(Subcommand, Debug, Clone)]
pub enum RetentionCommands {
    /// スキャンで見つかった全ロググループに一律のretention日数を設定する
    /// （--execute時のみ実行。削除は行わない。監査ログを必ず記録する）。
    Set(SetRetentionArgs),
}

#[derive(Args, Debug, Clone)]
pub struct SetRetentionArgs {
    /// 対象リージョン（カンマ区切りまたは複数回指定）。`all` で商用全リージョン。
    /// 未指定かつ標準入力がTTYの場合は対話式に選択する（非TTYではエラー）。
    #[arg(long, num_args = 1.., value_delimiter = ',')]
    pub regions: Vec<String>,

    /// メンバーアカウントへAssumeRoleする際のロール名。
    #[arg(long, default_value = "OrganizationAccountAccessRole")]
    pub role_name: String,

    /// 設定するretention日数（CloudWatch Logsが許容する離散値のみ）。
    #[arg(long)]
    pub days: i32,

    /// このフラグを明示的に指定しない限り、put-retention-policy は呼び出されない（dry-run既定）。
    #[arg(long, default_value_t = false)]
    pub execute: bool,

    /// 監査ログ（JSON Lines）の出力先パス。無効化するオプションは存在しない。
    #[arg(long, default_value = "cwsweep-audit.jsonl")]
    pub audit_log_path: std::path::PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct ScanArgs {
    /// 対象リージョン（カンマ区切りまたは複数回指定）。`all` で商用全リージョン。
    /// 未指定かつ標準入力がTTYの場合は対話式に選択する（非TTYではエラー）。
    #[arg(long, num_args = 1.., value_delimiter = ',')]
    pub regions: Vec<String>,

    /// メンバーアカウントへAssumeRoleする際のロール名。
    #[arg(long, default_value = "OrganizationAccountAccessRole")]
    pub role_name: String,

    /// 出力フォーマット（table | json）。
    #[arg(long, value_enum, default_value = "table")]
    pub output: OutputFormat,
}

#[derive(Args, Debug, Clone)]
pub struct CleanArgs {
    /// 対象リージョン（カンマ区切りまたは複数回指定）。`all` で商用全リージョン。
    /// 未指定かつ標準入力がTTYの場合は対話式に選択する（非TTYではエラー）。
    #[arg(long, num_args = 1.., value_delimiter = ',')]
    pub regions: Vec<String>,

    /// メンバーアカウントへAssumeRoleする際のロール名。
    #[arg(long, default_value = "OrganizationAccountAccessRole")]
    pub role_name: String,

    /// このフラグを明示的に指定しない限り、delete-log-group / put-retention-policy は
    /// 一切呼び出されない（dry-run既定）。
    #[arg(long, default_value_t = false)]
    pub execute: bool,

    /// 監査ログ（JSON Lines）の出力先パス。無効化するオプションは存在しない
    /// （project.md Mandated）。
    #[arg(long, default_value = "cwsweep-audit.jsonl")]
    pub audit_log_path: std::path::PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct AuditArgs {
    /// 読み取る監査ログ（JSON Lines）のパス。
    #[arg(long, default_value = "cwsweep-audit.jsonl")]
    pub audit_log_path: std::path::PathBuf,

    /// 出力フォーマット（table | json）。
    #[arg(long, value_enum, default_value = "table")]
    pub output: OutputFormat,
}

impl Cli {
    /// プロセスのコマンドライン引数をパースする（`clap::Parser::parse`相当）。
    pub fn parse_args() -> Self {
        let mut cli = <Self as Parser>::parse();
        cli.dedup_regions();
        cli
    }

    pub fn parse_from_args<I, T>(args: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let mut cli = Self::try_parse_from(args)?;
        cli.dedup_regions();
        Ok(cli)
    }

    fn dedup_regions(&mut self) {
        let regions = match &mut self.command {
            Commands::Scan(args) => &mut args.regions,
            Commands::Clean(args) => &mut args.regions,
            Commands::Audit(_) => return,
            Commands::Retention(RetentionArgs {
                command: RetentionCommands::Set(args),
            }) => &mut args.regions,
        };
        let mut seen = std::collections::HashSet::new();
        regions.retain(|r| seen.insert(r.clone()));
    }
}

/// ハンドラが返す終了方針。`main`はこれをプロセス終了コードへ写像するだけで、
/// 判定ロジックは持たない（BR6.1）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExitDisposition {
    Success,
    /// R-03: 対象アカウント×リージョンが1件以上あり、その全件が失敗した。
    ScanFullyFailed {
        attempted: usize,
    },
    Error(String),
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

/// 読み取り専用のスキャン依存（`scan`サブコマンドの全依存、`clean`の前半）。
/// 監査ログ書き込み・破壊的API型を一切持たない（BR2.2 / NFR2.4）。
pub struct ScanApp {
    pub credential_provider: Arc<CredentialProvider>,
    pub identity: Arc<dyn IdentityCheck>,
    pub logs_client: Arc<dyn DescribeLogGroupsOperations>,
}

/// `clean`パイプライン全体を統括するオーケストレータ。
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
    fn scan_app(&self) -> ScanApp {
        ScanApp {
            credential_provider: self.credential_provider.clone(),
            identity: self.identity.clone(),
            logs_client: self.logs_client.clone(),
        }
    }

    /// アカウント一覧×リージョン一覧をスキャンし、`ScanAggregator`へ集約する。
    pub async fn scan_all(
        &self,
        accounts: &[AccountInfo],
        regions: &[String],
    ) -> (ScanAggregator, Vec<AccountRegionOutcome>) {
        self.scan_app().scan_all(accounts, regions).await
    }
}

impl ScanApp {
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

    /// FR2: スキャン → 警告 → 全滅判定 → 出力で終了する。選択・確認・実行へ進まず、
    /// 監査ログにも書き込まない（BR2.1 / BR2.2）。
    pub async fn run_scan(
        &self,
        accounts: &[AccountInfo],
        regions: &[String],
        output: OutputFormat,
        out: &mut dyn Write,
    ) -> ExitDisposition {
        let (aggregator, outcomes) = self.scan_all(accounts, regions).await;
        if let Some(disposition) = report_scan_outcomes(&outcomes) {
            return disposition;
        }
        if writeln!(out, "{}", CliApp::render_output(&aggregator, output)).is_err() {
            return ExitDisposition::Error("failed to write output".to_string());
        }
        if aggregator.is_empty() && output == OutputFormat::Table {
            let _ = writeln!(out, "ロググループは見つかりませんでした。");
        }
        ExitDisposition::Success
    }
}

/// 失敗したアカウント×リージョンを警告ログへ出力し、全滅なら終了方針を返す（BR2.3）。
fn report_scan_outcomes(outcomes: &[AccountRegionOutcome]) -> Option<ExitDisposition> {
    for outcome in outcomes {
        if let AccountRegionOutcome::Failed {
            account_id,
            region,
            error,
        } = outcome
        {
            tracing::warn!(account_id, region, %error, "scan failed for account/region");
        }
    }
    if CliApp::scan_fully_failed(outcomes) {
        return Some(ExitDisposition::ScanFullyFailed {
            attempted: outcomes.len(),
        });
    }
    None
}

/// `clean`の対話部分（実装は`main`側のアダプタ、テストではスタブ）。
pub struct CleanInteraction<'a, S: MultiSelectPrompt, C: ConfirmPrompt> {
    pub selector: &'a InteractiveSelector<S>,
    pub presenter: &'a ConfirmationPresenter<C>,
    pub choose_action: &'a dyn Fn() -> Result<ActionKind, String>,
}

/// `retention set`の実行オプション。
#[derive(Debug, Clone, Copy)]
pub struct SetRetentionOptions {
    pub days: i32,
    pub execute: bool,
    pub stdin_is_tty: bool,
}

/// FR4: 監査ログを読み取り専用で全件表示する。依存は`AuditRead`と出力先のみで、
/// 破壊的操作系・監査ログ書き込み系の型には到達しない（BR4.1 / BR4.2）。
pub fn run_audit(
    reader: &dyn AuditRead,
    output: OutputFormat,
    out: &mut dyn Write,
) -> ExitDisposition {
    let outcome = match reader.entries() {
        Ok(outcome) => outcome,
        Err(e) => return ExitDisposition::Error(format!("監査ログの読み取りに失敗しました: {e}")),
    };
    match writeln!(
        out,
        "{}",
        OutputFormatter::format_audit(&outcome.entries, output)
    ) {
        Ok(()) => ExitDisposition::Success,
        Err(_) => ExitDisposition::Error("failed to write output".to_string()),
    }
}

/// 実行結果1件を人間可読の1行にする。dry-runは失敗ではなくスキップとして表現する。
pub fn format_outcome_line(outcome: &ExecutionOutcome) -> String {
    let target = format!(
        "{}/{}/{}",
        outcome.action.account_id, outcome.action.region, outcome.action.log_group_name
    );
    if outcome.dry_run {
        return format!("{target}: skipped (dry-run: --execute not supplied)");
    }
    match (&outcome.success, &outcome.error_message) {
        (true, _) => format!("{target}: success"),
        (false, Some(msg)) => format!("{target}: failed: {msg}"),
        (false, None) => format!("{target}: failed"),
    }
}

impl CliApp {
    /// FR3: スキャン → 表示 → TTY判定 → 選択 → プラン → 確認 → 実行。
    /// `execute == false`ならAPIは呼ばれない（BR3.2）。非TTYでは警告して正常終了する（BR3.4）。
    pub async fn run_clean<S: MultiSelectPrompt, C: ConfirmPrompt>(
        &self,
        accounts: &[AccountInfo],
        regions: &[String],
        execute: bool,
        stdin_is_tty: bool,
        interaction: CleanInteraction<'_, S, C>,
        out: &mut dyn Write,
    ) -> ExitDisposition {
        let (aggregator, outcomes) = self.scan_all(accounts, regions).await;
        if let Some(disposition) = report_scan_outcomes(&outcomes) {
            return disposition;
        }
        let _ = writeln!(
            out,
            "{}",
            CliApp::render_output(&aggregator, OutputFormat::Table)
        );
        if aggregator.is_empty() {
            let _ = writeln!(out, "削除対象のロググループはありません。");
            return ExitDisposition::Success;
        }
        if !stdin_is_tty {
            tracing::warn!(
                "stdin is not a TTY; cannot proceed to interactive selection. Use `cwsweep scan` for non-interactive inventory."
            );
            return ExitDisposition::Success;
        }

        let selected = match CliApp::select(&aggregator, interaction.selector) {
            Ok(selected) => selected,
            Err(e) => return ExitDisposition::Error(e.to_string()),
        };
        if selected.is_empty() {
            let _ = writeln!(out, "選択されたロググループはありません。終了します。");
            return ExitDisposition::Success;
        }

        let action_kind = match (interaction.choose_action)() {
            Ok(kind) => kind,
            Err(e) => return ExitDisposition::Error(e),
        };
        let planned = match CliApp::plan(&selected, action_kind) {
            Ok(planned) => planned,
            Err(e) => return ExitDisposition::Error(e.to_string()),
        };
        let total_bytes: i64 = selected.iter().map(|r| r.stored_bytes).sum();

        let Some(confirmed_actions) = CliApp::confirm(interaction.presenter, &planned, total_bytes)
        else {
            let _ = writeln!(out, "確認が得られなかったため、処理を中止します。");
            return ExitDisposition::Success;
        };

        match self.execute(&confirmed_actions, execute).await {
            Ok(results) => {
                for outcome in &results {
                    let _ = writeln!(out, "{}", format_outcome_line(outcome));
                }
                ExitDisposition::Success
            }
            Err(e) => ExitDisposition::Error(e.to_string()),
        }
    }

    /// `retention set`: スキャン → 表示 → 全件に`SetRetention { days }`を計画 → 確認（TTY時）→ 実行。
    /// 削除アクションは一切構築しない。`execute == false`ならAPIは呼ばれない。
    /// 非TTYでは確認プロンプトを省略し、`--execute`があればそのまま適用する。
    pub async fn run_set_retention<C: ConfirmPrompt>(
        &self,
        accounts: &[AccountInfo],
        regions: &[String],
        options: SetRetentionOptions,
        presenter: &ConfirmationPresenter<C>,
        out: &mut dyn Write,
    ) -> ExitDisposition {
        let SetRetentionOptions {
            days,
            execute,
            stdin_is_tty,
        } = options;
        let action_kind = ActionKind::SetRetention { days };
        if let Err(e) = CliApp::plan(&[], action_kind) {
            return ExitDisposition::Error(e.to_string());
        }

        let (aggregator, outcomes) = self.scan_all(accounts, regions).await;
        if let Some(disposition) = report_scan_outcomes(&outcomes) {
            return disposition;
        }
        let _ = writeln!(
            out,
            "{}",
            CliApp::render_output(&aggregator, OutputFormat::Table)
        );
        if aggregator.is_empty() {
            let _ = writeln!(out, "retention設定対象のロググループはありません。");
            return ExitDisposition::Success;
        }

        let targets: Vec<crate::aggregator::LogGroupRecord> = aggregator
            .sorted_by_size_desc()
            .into_iter()
            .cloned()
            .collect();
        let planned = match CliApp::plan(&targets, action_kind) {
            Ok(planned) => planned,
            Err(e) => return ExitDisposition::Error(e.to_string()),
        };
        let total_bytes: i64 = targets.iter().map(|r| r.stored_bytes).sum();

        let confirmed_actions = if stdin_is_tty {
            match CliApp::confirm(presenter, &planned, total_bytes) {
                Some(actions) => actions,
                None => {
                    let _ = writeln!(out, "確認が得られなかったため、処理を中止します。");
                    return ExitDisposition::Success;
                }
            }
        } else {
            planned
                .into_iter()
                .map(|a| PlannedAction {
                    confirmed: true,
                    ..a
                })
                .collect()
        };

        match self.execute(&confirmed_actions, execute).await {
            Ok(results) => {
                for outcome in &results {
                    let _ = writeln!(out, "{}", format_outcome_line(outcome));
                }
                ExitDisposition::Success
            }
            Err(e) => ExitDisposition::Error(e.to_string()),
        }
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
            .sorted_by_size_desc()
            .into_iter()
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
    ) -> Result<Vec<PlannedAction>, crate::error::InvalidRetentionDaysError> {
        ActionPlanner::plan(selected, action_kind)
    }

    pub fn confirm<P: ConfirmPrompt>(
        presenter: &ConfirmationPresenter<P>,
        actions: &[PlannedAction],
        total_bytes: i64,
    ) -> Option<Vec<PlannedAction>> {
        presenter.confirm(actions, total_bytes)
    }

    /// R-05 (NFR4.2): 1アカウントの`AssumeRole`失敗は当該アカウント分のアクションのみを
    /// 失敗として扱い、資格情報を取得できた他アカウントの処理は継続する
    /// （バルクヘッド分離。`scan_all`と同じ方針をexecuteフェーズにも適用する）。
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
        // 失敗した場合も`?`で即座に中断せず、そのアカウントIDに対する結果として保持する
        // （他アカウントの資格情報解決・実行は継続する）。
        let mut resolved: std::collections::HashMap<
            String,
            Result<crate::credentials::AccountCredentials, crate::error::AssumeRoleError>,
        > = std::collections::HashMap::new();
        for action in confirmed_actions {
            if !resolved.contains_key(&action.account_id) {
                let result = provider.credentials_for(&action.account_id).await;
                resolved.insert(action.account_id.clone(), result);
            }
        }

        // 資格情報を取得できたアクションのみ`ExecutionEngine`へ渡す。取得できなかった
        // アクションは、ここで直接失敗した`ExecutionOutcome`として記録する
        // （元の`confirmed_actions`の順序を保ったまま結果を合成する）。
        let mut ordered: Vec<Option<ExecutionOutcome>> = vec![None; confirmed_actions.len()];
        let mut runnable_actions: Vec<PlannedAction> = Vec::new();
        let mut runnable_indices: Vec<usize> = Vec::new();

        for (idx, action) in confirmed_actions.iter().enumerate() {
            match resolved.get(&action.account_id) {
                Some(Ok(_)) => {
                    runnable_actions.push(action.clone());
                    runnable_indices.push(idx);
                }
                Some(Err(e)) => {
                    ordered[idx] = Some(ExecutionOutcome {
                        action: action.clone(),
                        success: false,
                        error_message: Some(format!(
                            "credentials unavailable for account {}: {e}",
                            action.account_id
                        )),
                        dry_run: false,
                    });
                }
                None => {
                    // 構造的に到達不能: `resolved`は上のループで全`account_id`について
                    // 必ずエントリを持つ。万一到達した場合でも、当該アクションを失敗として
                    // 記録するのみでpanicはしない。
                    ordered[idx] = Some(ExecutionOutcome {
                        action: action.clone(),
                        success: false,
                        error_message: Some(format!(
                            "credentials unavailable for account {}: not resolved",
                            action.account_id
                        )),
                        dry_run: false,
                    });
                }
            }
        }

        let engine_outcomes = engine
            .execute_plan(
                &runnable_actions,
                |account_id| {
                    resolved
                        .get(account_id)
                        .and_then(|r| r.as_ref().ok())
                        .cloned()
                },
                execute_flag,
            )
            .await?;

        for (idx, outcome) in runnable_indices.into_iter().zip(engine_outcomes) {
            ordered[idx] = Some(outcome);
        }

        Ok(ordered.into_iter().flatten().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan_args(cli: Cli) -> ScanArgs {
        match cli.command {
            Commands::Scan(args) => args,
            other => panic!("expected scan, got {other:?}"),
        }
    }

    fn clean_args(cli: Cli) -> CleanArgs {
        match cli.command {
            Commands::Clean(args) => args,
            other => panic!("expected clean, got {other:?}"),
        }
    }

    fn audit_args(cli: Cli) -> AuditArgs {
        match cli.command {
            Commands::Audit(args) => args,
            other => panic!("expected audit, got {other:?}"),
        }
    }

    // --- FR1.1 / FR6.1 / NFR4: サブコマンド必須・旧フラグ廃止（互換シムなし） ---

    #[test]
    fn missing_subcommand_is_a_parse_error() {
        assert!(Cli::parse_from_args(["cwsweep"]).is_err());
    }

    #[test]
    fn legacy_flat_flags_without_subcommand_are_rejected() {
        assert!(Cli::parse_from_args(["cwsweep", "--regions", "us-east-1"]).is_err());
        assert!(Cli::parse_from_args(["cwsweep", "--regions", "us-east-1", "--execute"]).is_err());
        assert!(
            Cli::parse_from_args(["cwsweep", "--regions", "us-east-1", "--scan-only"]).is_err()
        );
    }

    #[test]
    fn scan_only_flag_no_longer_exists_on_any_subcommand() {
        assert!(
            Cli::parse_from_args(["cwsweep", "scan", "--regions", "us-east-1", "--scan-only"])
                .is_err()
        );
        assert!(Cli::parse_from_args([
            "cwsweep",
            "clean",
            "--regions",
            "us-east-1",
            "--scan-only"
        ])
        .is_err());
    }

    // --- FR2: scan ---

    #[test]
    fn scan_regions_is_optional_and_defaults_to_empty() {
        let args = scan_args(Cli::parse_from_args(["cwsweep", "scan"]).unwrap());
        assert!(args.regions.is_empty());
        assert_eq!(
            crate::regions::region_spec(&args.regions),
            crate::regions::RegionSpec::Unspecified
        );
    }

    #[test]
    fn scan_regions_all_is_case_insensitive() {
        for value in ["all", "ALL", "All"] {
            let args =
                scan_args(Cli::parse_from_args(["cwsweep", "scan", "--regions", value]).unwrap());
            assert_eq!(
                crate::regions::region_spec(&args.regions),
                crate::regions::RegionSpec::All
            );
        }
    }

    #[test]
    fn scan_regions_flag_without_value_is_a_parse_error() {
        assert!(Cli::parse_from_args(["cwsweep", "scan", "--regions"]).is_err());
    }

    #[test]
    fn scan_accepts_comma_separated_regions_and_dedupes_preserving_order() {
        let args = scan_args(
            Cli::parse_from_args([
                "cwsweep",
                "scan",
                "--regions",
                "us-west-2,us-east-1,us-west-2",
                "--regions",
                "us-east-1",
            ])
            .unwrap(),
        );
        assert_eq!(args.regions, vec!["us-west-2", "us-east-1"]);
    }

    #[test]
    fn scan_defaults_role_name_and_table_output() {
        let args =
            scan_args(Cli::parse_from_args(["cwsweep", "scan", "--regions", "us-east-1"]).unwrap());
        assert_eq!(args.role_name, "OrganizationAccountAccessRole");
        assert_eq!(args.output, OutputFormat::Table);
    }

    #[test]
    fn scan_accepts_json_output_and_custom_role_name() {
        let args = scan_args(
            Cli::parse_from_args([
                "cwsweep",
                "scan",
                "--regions",
                "us-east-1",
                "--output",
                "json",
                "--role-name",
                "CustomRole",
            ])
            .unwrap(),
        );
        assert_eq!(args.output, OutputFormat::Json);
        assert_eq!(args.role_name, "CustomRole");
    }

    #[test]
    fn scan_rejects_execute_and_audit_log_path() {
        assert!(
            Cli::parse_from_args(["cwsweep", "scan", "--regions", "us-east-1", "--execute"])
                .is_err()
        );
        assert!(Cli::parse_from_args([
            "cwsweep",
            "scan",
            "--regions",
            "us-east-1",
            "--audit-log-path",
            "x.jsonl",
        ])
        .is_err());
    }

    // --- FR3: clean ---

    #[test]
    fn clean_regions_is_optional_and_dedupes() {
        let args = clean_args(Cli::parse_from_args(["cwsweep", "clean"]).unwrap());
        assert!(args.regions.is_empty());
        let args =
            clean_args(Cli::parse_from_args(["cwsweep", "clean", "--regions", "a,b,a"]).unwrap());
        assert_eq!(args.regions, vec!["a", "b"]);
    }

    #[test]
    fn clean_execute_defaults_to_false_and_is_true_only_when_supplied() {
        let args = clean_args(
            Cli::parse_from_args(["cwsweep", "clean", "--regions", "us-east-1"]).unwrap(),
        );
        assert!(!args.execute);
        let args = clean_args(
            Cli::parse_from_args(["cwsweep", "clean", "--regions", "us-east-1", "--execute"])
                .unwrap(),
        );
        assert!(args.execute);
    }

    #[test]
    fn clean_audit_log_path_defaults_to_cwsweep_audit_jsonl_and_can_be_overridden() {
        let args = clean_args(
            Cli::parse_from_args(["cwsweep", "clean", "--regions", "us-east-1"]).unwrap(),
        );
        assert_eq!(
            args.audit_log_path,
            std::path::PathBuf::from("cwsweep-audit.jsonl")
        );
        let args = clean_args(
            Cli::parse_from_args([
                "cwsweep",
                "clean",
                "--regions",
                "us-east-1",
                "--audit-log-path",
                "/var/log/cwsweep/audit.jsonl",
            ])
            .unwrap(),
        );
        assert_eq!(
            args.audit_log_path,
            std::path::PathBuf::from("/var/log/cwsweep/audit.jsonl")
        );
    }

    #[test]
    fn clean_rejects_output_flag() {
        assert!(Cli::parse_from_args([
            "cwsweep",
            "clean",
            "--regions",
            "us-east-1",
            "--output",
            "json"
        ])
        .is_err());
    }

    // --- FR4: audit ---

    #[test]
    fn audit_defaults_and_overrides() {
        let args = audit_args(Cli::parse_from_args(["cwsweep", "audit"]).unwrap());
        assert_eq!(
            args.audit_log_path,
            std::path::PathBuf::from("cwsweep-audit.jsonl")
        );
        assert_eq!(args.output, OutputFormat::Table);
        let args = audit_args(
            Cli::parse_from_args([
                "cwsweep",
                "audit",
                "--audit-log-path",
                "x.jsonl",
                "--output",
                "json",
            ])
            .unwrap(),
        );
        assert_eq!(args.audit_log_path, std::path::PathBuf::from("x.jsonl"));
        assert_eq!(args.output, OutputFormat::Json);
    }

    #[test]
    fn audit_rejects_regions_and_execute() {
        assert!(Cli::parse_from_args(["cwsweep", "audit", "--regions", "us-east-1"]).is_err());
        assert!(Cli::parse_from_args(["cwsweep", "audit", "--execute"]).is_err());
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
            session_token: Some(SecretString::from("token".to_string())),
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
        retentions: std::sync::atomic::AtomicUsize,
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
            self.retentions
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
                retentions: std::sync::atomic::AtomicUsize::new(0),
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
                retentions: std::sync::atomic::AtomicUsize::new(0),
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
    fn select_presents_items_in_stored_bytes_desc_order_matching_table() {
        let mut aggregator = ScanAggregator::new();
        aggregator.add_all(
            [("/small", 1), ("/large", 300), ("/medium", 20)]
                .into_iter()
                .map(|(name, bytes)| crate::aggregator::LogGroupRecord {
                    account_id: MANAGEMENT_ACCOUNT_ID.to_string(),
                    region: REGION.to_string(),
                    log_group_name: name.to_string(),
                    stored_bytes: bytes,
                    retention_in_days: None,
                })
                .collect(),
        );
        let selector = InteractiveSelector::new(FixedSelectAll);

        let selected = CliApp::select(&aggregator, &selector).unwrap();

        let names: Vec<&str> = selected.iter().map(|r| r.log_group_name.as_str()).collect();
        assert_eq!(names, vec!["/large", "/medium", "/small"]);
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

        let planned = CliApp::plan(&records, ActionKind::Delete).unwrap();

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
            retentions: std::sync::atomic::AtomicUsize::new(0),
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
            retentions: std::sync::atomic::AtomicUsize::new(0),
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

    // --- CodeRabbit指摘#1: executeフェーズでのAssumeRole失敗バルクヘッド分離のテスト ---

    struct StubAssumeRoleFailsForAccount(&'static str);
    #[async_trait]
    impl AssumeRoleOperations for StubAssumeRoleFailsForAccount {
        async fn assume_role(
            &self,
            account_id: &str,
            role_arn: &str,
            _session_name: &str,
        ) -> Result<AccountCredentials, AssumeRoleError> {
            if account_id == self.0 {
                Err(AssumeRoleError {
                    account_id: account_id.to_string(),
                    role_name: role_arn.to_string(),
                    message: "access denied".to_string(),
                })
            } else {
                Ok(test_creds(account_id))
            }
        }
    }

    const OTHER_MEMBER_ACCOUNT_ID: &str = "333333333333";

    #[tokio::test]
    async fn execute_continues_for_other_accounts_when_one_account_assume_role_fails() {
        let dir = tempfile::tempdir().unwrap();
        let api = Arc::new(RecordingApi {
            deletes: std::sync::atomic::AtomicUsize::new(0),
            retentions: std::sync::atomic::AtomicUsize::new(0),
        });
        let audit_logger: Arc<dyn AuditWrite> =
            Arc::new(AuditLogger::open(&dir.path().join("audit.jsonl")).unwrap());
        let credential_provider = Arc::new(CredentialProvider::with_default_role_name(
            MANAGEMENT_ACCOUNT_ID,
            Arc::new(StubAssumeRoleFailsForAccount(MEMBER_ACCOUNT_ID)),
            test_creds(MANAGEMENT_ACCOUNT_ID),
        ));
        let app = CliApp {
            credential_provider,
            identity: Arc::new(StubIdentityOk),
            logs_client: Arc::new(StubDescribeLogGroups),
            api_client: api.clone(),
            audit_logger,
            run_id: "test-run".to_string(),
        };

        let confirmed = vec![
            PlannedAction {
                account_id: MEMBER_ACCOUNT_ID.to_string(),
                region: REGION.to_string(),
                log_group_name: "/a".to_string(),
                action_kind: ActionKind::Delete,
                confirmed: true,
            },
            PlannedAction {
                account_id: OTHER_MEMBER_ACCOUNT_ID.to_string(),
                region: REGION.to_string(),
                log_group_name: "/b".to_string(),
                action_kind: ActionKind::Delete,
                confirmed: true,
            },
        ];

        let outcomes = app.execute(&confirmed, true).await.unwrap();

        assert_eq!(outcomes.len(), 2);
        // MEMBER_ACCOUNT_ID分はAssumeRole失敗で失敗として記録されるが、処理は中断しない。
        assert!(!outcomes[0].success);
        assert!(outcomes[0]
            .error_message
            .as_deref()
            .unwrap_or_default()
            .contains("credentials unavailable"));
        // OTHER_MEMBER_ACCOUNT_ID分は資格情報が取得できたため、正常に実行され成功する。
        assert!(outcomes[1].success);
        assert_eq!(api.deletes.load(std::sync::atomic::Ordering::SeqCst), 1);
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

    // --- ハンドラ層（run_scan / run_clean / run_audit）のテスト ---

    struct StubIdentityFailAll;
    #[async_trait]
    impl IdentityCheck for StubIdentityFailAll {
        async fn verify(&self, _c: &AccountCredentials, e: &str) -> Result<(), IdentityError> {
            Err(IdentityMismatchError {
                expected: e.to_string(),
                actual: None,
            }
            .into())
        }
    }

    fn build_scan_app(identity: Arc<dyn IdentityCheck>) -> ScanApp {
        ScanApp {
            credential_provider: Arc::new(CredentialProvider::with_default_role_name(
                MANAGEMENT_ACCOUNT_ID,
                Arc::new(StubAssumeRole),
                test_creds(MANAGEMENT_ACCOUNT_ID),
            )),
            identity,
            logs_client: Arc::new(StubDescribeLogGroups),
        }
    }

    #[tokio::test]
    async fn run_scan_json_output_is_a_single_json_document() {
        let app = build_scan_app(Arc::new(StubIdentityOk));
        let mut out = Vec::new();

        let disposition = app
            .run_scan(
                &accounts(),
                &[REGION.to_string()],
                OutputFormat::Json,
                &mut out,
            )
            .await;

        assert_eq!(disposition, ExitDisposition::Success);
        let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(parsed["log_group_count"], 2);
    }

    #[tokio::test]
    async fn run_scan_returns_fully_failed_when_every_account_region_fails() {
        let app = build_scan_app(Arc::new(StubIdentityFailAll));
        let mut out = Vec::new();

        let disposition = app
            .run_scan(
                &accounts(),
                &[REGION.to_string()],
                OutputFormat::Table,
                &mut out,
            )
            .await;

        assert_eq!(
            disposition,
            ExitDisposition::ScanFullyFailed { attempted: 2 }
        );
        assert!(out.is_empty());
    }

    #[tokio::test]
    async fn run_scan_with_no_targets_succeeds_and_prints_table_notice() {
        let app = build_scan_app(Arc::new(StubIdentityOk));
        let mut out = Vec::new();

        let disposition = app
            .run_scan(&[], &[REGION.to_string()], OutputFormat::Table, &mut out)
            .await;

        assert_eq!(disposition, ExitDisposition::Success);
        assert!(String::from_utf8(out)
            .unwrap()
            .contains("ロググループは見つかりませんでした"));
    }

    fn recording_api() -> Arc<RecordingApi> {
        Arc::new(RecordingApi {
            deletes: std::sync::atomic::AtomicUsize::new(0),
            retentions: std::sync::atomic::AtomicUsize::new(0),
        })
    }

    fn deletes(api: &RecordingApi) -> usize {
        api.deletes.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn retentions(api: &RecordingApi) -> usize {
        api.retentions.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn choose_delete() -> Result<ActionKind, String> {
        Ok(ActionKind::Delete)
    }

    #[tokio::test]
    async fn run_clean_without_tty_prints_scan_and_never_reaches_execution() {
        let dir = tempfile::tempdir().unwrap();
        let api = recording_api();
        let app = build_app(Arc::new(StubIdentityOk), api.clone(), &dir);
        let selector = InteractiveSelector::new(FixedSelectAll);
        let presenter = ConfirmationPresenter::new(AlwaysConfirmPrompt);
        let mut out = Vec::new();

        let disposition = app
            .run_clean(
                &accounts(),
                &[REGION.to_string()],
                true,
                false,
                CleanInteraction {
                    selector: &selector,
                    presenter: &presenter,
                    choose_action: &choose_delete,
                },
                &mut out,
            )
            .await;

        assert_eq!(disposition, ExitDisposition::Success);
        assert_eq!(deletes(&api), 0);
        assert!(String::from_utf8(out).unwrap().contains(MEMBER_ACCOUNT_ID));
    }

    #[tokio::test]
    async fn run_clean_dry_run_default_never_calls_delete_api() {
        let dir = tempfile::tempdir().unwrap();
        let api = recording_api();
        let app = build_app(Arc::new(StubIdentityOk), api.clone(), &dir);
        let selector = InteractiveSelector::new(FixedSelectAll);
        let presenter = ConfirmationPresenter::new(AlwaysConfirmPrompt);
        let mut out = Vec::new();

        let disposition = app
            .run_clean(
                &accounts(),
                &[REGION.to_string()],
                false,
                true,
                CleanInteraction {
                    selector: &selector,
                    presenter: &presenter,
                    choose_action: &choose_delete,
                },
                &mut out,
            )
            .await;

        assert_eq!(disposition, ExitDisposition::Success);
        assert_eq!(deletes(&api), 0);
        assert!(String::from_utf8(out).unwrap().contains("skipped (dry-run"));
    }

    #[tokio::test]
    async fn run_clean_with_execute_and_confirmation_calls_delete_api() {
        let dir = tempfile::tempdir().unwrap();
        let api = recording_api();
        let app = build_app(Arc::new(StubIdentityOk), api.clone(), &dir);
        let selector = InteractiveSelector::new(FixedSelectAll);
        let presenter = ConfirmationPresenter::new(AlwaysConfirmPrompt);
        let mut out = Vec::new();

        let disposition = app
            .run_clean(
                &accounts(),
                &[REGION.to_string()],
                true,
                true,
                CleanInteraction {
                    selector: &selector,
                    presenter: &presenter,
                    choose_action: &choose_delete,
                },
                &mut out,
            )
            .await;

        assert_eq!(disposition, ExitDisposition::Success);
        assert_eq!(deletes(&api), 2);
        assert!(String::from_utf8(out).unwrap().contains(": success"));
    }

    struct NeverConfirmPrompt;
    impl ConfirmPrompt for NeverConfirmPrompt {
        fn confirm(&self, _summary: &ConfirmationSummary) -> bool {
            false
        }
    }

    #[tokio::test]
    async fn run_clean_aborts_without_execution_when_confirmation_is_declined() {
        let dir = tempfile::tempdir().unwrap();
        let api = recording_api();
        let app = build_app(Arc::new(StubIdentityOk), api.clone(), &dir);
        let selector = InteractiveSelector::new(FixedSelectAll);
        let presenter = ConfirmationPresenter::new(NeverConfirmPrompt);
        let mut out = Vec::new();

        let disposition = app
            .run_clean(
                &accounts(),
                &[REGION.to_string()],
                true,
                true,
                CleanInteraction {
                    selector: &selector,
                    presenter: &presenter,
                    choose_action: &choose_delete,
                },
                &mut out,
            )
            .await;

        assert_eq!(disposition, ExitDisposition::Success);
        assert_eq!(deletes(&api), 0);
        assert!(String::from_utf8(out).unwrap().contains("処理を中止します"));
    }

    // --- retention set ---

    fn set_retention_args(cli: Cli) -> SetRetentionArgs {
        match cli.command {
            Commands::Retention(RetentionArgs {
                command: RetentionCommands::Set(args),
            }) => args,
            other => panic!("expected retention set, got {other:?}"),
        }
    }

    #[test]
    fn retention_set_parses_days_and_defaults() {
        let args = set_retention_args(
            Cli::parse_from_args([
                "cwsweep",
                "retention",
                "set",
                "--regions",
                "us-east-1,us-west-2,us-east-1",
                "--days",
                "30",
            ])
            .unwrap(),
        );
        assert_eq!(args.regions, vec!["us-east-1", "us-west-2"]);
        assert_eq!(args.days, 30);
        assert!(!args.execute);
        assert_eq!(args.role_name, "OrganizationAccountAccessRole");
        assert_eq!(
            args.audit_log_path,
            std::path::PathBuf::from("cwsweep-audit.jsonl")
        );
    }

    #[test]
    fn retention_set_accepts_execute_and_overrides() {
        let args = set_retention_args(
            Cli::parse_from_args([
                "cwsweep",
                "retention",
                "set",
                "--days",
                "7",
                "--execute",
                "--role-name",
                "CustomRole",
                "--audit-log-path",
                "x.jsonl",
            ])
            .unwrap(),
        );
        assert!(args.regions.is_empty());
        assert!(args.execute);
        assert_eq!(args.role_name, "CustomRole");
        assert_eq!(args.audit_log_path, std::path::PathBuf::from("x.jsonl"));
    }

    #[test]
    fn retention_set_requires_days_and_a_nested_subcommand() {
        assert!(
            Cli::parse_from_args(["cwsweep", "retention", "set", "--regions", "us-east-1"])
                .is_err()
        );
        assert!(Cli::parse_from_args(["cwsweep", "retention"]).is_err());
        assert!(Cli::parse_from_args(["cwsweep", "retention", "set", "--days", "abc"]).is_err());
    }

    #[tokio::test]
    async fn run_set_retention_dry_run_default_never_calls_any_api() {
        let dir = tempfile::tempdir().unwrap();
        let api = recording_api();
        let app = build_app(Arc::new(StubIdentityOk), api.clone(), &dir);
        let presenter = ConfirmationPresenter::new(AlwaysConfirmPrompt);
        let mut out = Vec::new();

        let disposition = app
            .run_set_retention(
                &accounts(),
                &[REGION.to_string()],
                SetRetentionOptions {
                    days: 30,
                    execute: false,
                    stdin_is_tty: true,
                },
                &presenter,
                &mut out,
            )
            .await;

        assert_eq!(disposition, ExitDisposition::Success);
        assert_eq!(retentions(&api), 0);
        assert_eq!(deletes(&api), 0);
        assert!(String::from_utf8(out).unwrap().contains("skipped (dry-run"));
    }

    #[tokio::test]
    async fn run_set_retention_with_execute_calls_put_retention_for_every_group_and_never_deletes()
    {
        let dir = tempfile::tempdir().unwrap();
        let api = recording_api();
        let app = build_app(Arc::new(StubIdentityOk), api.clone(), &dir);
        let presenter = ConfirmationPresenter::new(AlwaysConfirmPrompt);
        let mut out = Vec::new();

        let disposition = app
            .run_set_retention(
                &accounts(),
                &[REGION.to_string()],
                SetRetentionOptions {
                    days: 30,
                    execute: true,
                    stdin_is_tty: true,
                },
                &presenter,
                &mut out,
            )
            .await;

        assert_eq!(disposition, ExitDisposition::Success);
        assert_eq!(retentions(&api), 2);
        assert_eq!(deletes(&api), 0);
        assert!(String::from_utf8(out).unwrap().contains(": success"));
    }

    #[tokio::test]
    async fn run_set_retention_without_tty_applies_when_execute_is_supplied() {
        let dir = tempfile::tempdir().unwrap();
        let api = recording_api();
        let app = build_app(Arc::new(StubIdentityOk), api.clone(), &dir);
        let presenter = ConfirmationPresenter::new(NeverConfirmPrompt);
        let mut out = Vec::new();

        let disposition = app
            .run_set_retention(
                &accounts(),
                &[REGION.to_string()],
                SetRetentionOptions {
                    days: 30,
                    execute: true,
                    stdin_is_tty: false,
                },
                &presenter,
                &mut out,
            )
            .await;

        assert_eq!(disposition, ExitDisposition::Success);
        assert_eq!(retentions(&api), 2);
        assert_eq!(deletes(&api), 0);
    }

    #[tokio::test]
    async fn run_set_retention_aborts_when_tty_confirmation_is_declined() {
        let dir = tempfile::tempdir().unwrap();
        let api = recording_api();
        let app = build_app(Arc::new(StubIdentityOk), api.clone(), &dir);
        let presenter = ConfirmationPresenter::new(NeverConfirmPrompt);
        let mut out = Vec::new();

        let disposition = app
            .run_set_retention(
                &accounts(),
                &[REGION.to_string()],
                SetRetentionOptions {
                    days: 30,
                    execute: true,
                    stdin_is_tty: true,
                },
                &presenter,
                &mut out,
            )
            .await;

        assert_eq!(disposition, ExitDisposition::Success);
        assert_eq!(retentions(&api), 0);
        assert!(String::from_utf8(out).unwrap().contains("処理を中止します"));
    }

    #[tokio::test]
    async fn run_set_retention_rejects_invalid_days_before_scanning() {
        let dir = tempfile::tempdir().unwrap();
        let api = recording_api();
        let app = build_app(Arc::new(StubIdentityOk), api.clone(), &dir);
        let presenter = ConfirmationPresenter::new(AlwaysConfirmPrompt);
        let mut out = Vec::new();

        let disposition = app
            .run_set_retention(
                &accounts(),
                &[REGION.to_string()],
                SetRetentionOptions {
                    days: 13,
                    execute: true,
                    stdin_is_tty: true,
                },
                &presenter,
                &mut out,
            )
            .await;

        assert!(matches!(disposition, ExitDisposition::Error(_)));
        assert_eq!(retentions(&api), 0);
        assert!(out.is_empty());
    }

    // --- run_audit ---

    use crate::audit::AuditReader;

    #[test]
    fn run_audit_with_missing_file_succeeds_with_empty_table() {
        let dir = tempfile::tempdir().unwrap();
        let reader = AuditReader::new(dir.path().join("missing.jsonl"));
        let mut out = Vec::new();

        let disposition = run_audit(&reader, OutputFormat::Table, &mut out);

        assert_eq!(disposition, ExitDisposition::Success);
        assert!(String::from_utf8(out)
            .unwrap()
            .contains("Total: 0 audit entry(ies)"));
        assert!(!dir.path().join("missing.jsonl").exists());
    }

    #[test]
    fn run_audit_json_output_lists_every_recorded_entry() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");
        let logger = AuditLogger::open(&path).unwrap();
        let entry = |event: crate::audit::AuditEventKind, success: bool| crate::audit::AuditEntry {
            run_id: "run-1".to_string(),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            account_id: MANAGEMENT_ACCOUNT_ID.to_string(),
            region: REGION.to_string(),
            log_group_name: "/a".to_string(),
            action_kind: ActionKind::Delete,
            event,
            success,
            error_message: None,
        };
        logger
            .append(&entry(crate::audit::AuditEventKind::Intent, false))
            .unwrap();
        logger
            .append(&entry(crate::audit::AuditEventKind::Result, true))
            .unwrap();
        let reader = AuditReader::new(path);
        let mut out = Vec::new();

        let disposition = run_audit(&reader, OutputFormat::Json, &mut out);

        assert_eq!(disposition, ExitDisposition::Success);
        let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 2);
        assert_eq!(parsed[0]["run_id"], "run-1");
    }

    #[test]
    fn run_audit_reports_error_when_path_is_a_directory() {
        let dir = tempfile::tempdir().unwrap();
        let reader = AuditReader::new(dir.path().to_path_buf());
        let mut out = Vec::new();

        let disposition = run_audit(&reader, OutputFormat::Table, &mut out);

        assert!(matches!(disposition, ExitDisposition::Error(_)));
    }

    // --- format_outcome_line ---

    fn outcome(success: bool, error_message: Option<&str>, dry_run: bool) -> ExecutionOutcome {
        ExecutionOutcome {
            action: PlannedAction {
                account_id: MANAGEMENT_ACCOUNT_ID.to_string(),
                region: REGION.to_string(),
                log_group_name: "/a".to_string(),
                action_kind: ActionKind::Delete,
                confirmed: true,
            },
            success,
            error_message: error_message.map(str::to_string),
            dry_run,
        }
    }

    #[test]
    fn outcome_line_marks_dry_run_as_skipped_not_failed() {
        let line = format_outcome_line(&outcome(
            false,
            Some("dry-run: --execute not supplied"),
            true,
        ));
        assert_eq!(
            line,
            "111111111111/us-east-1//a: skipped (dry-run: --execute not supplied)"
        );
    }

    #[test]
    fn outcome_line_reports_success_and_failure_distinctly() {
        assert_eq!(
            format_outcome_line(&outcome(true, None, false)),
            "111111111111/us-east-1//a: success"
        );
        assert_eq!(
            format_outcome_line(&outcome(false, Some("boom"), false)),
            "111111111111/us-east-1//a: failed: boom"
        );
    }
}
