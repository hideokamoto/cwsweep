//! `ExecutionEngine` コンポーネント（安全パス・TDD: Step 14）。
//!
//! project.md Forbidden: `--execute`フラグが明示的に渡されていない限り、`delete-log-group`
//! または`put-retention-policy`を呼び出さない（dry-runをデフォルト動作とする）。
//! project.md Mandated: `delete-log-group`/`put-retention-policy`を実行する直前に、
//! スキャン時点とは独立した二重目のIdentity検証を再実行する。
//! project.md Mandated: 監査ログへの書き込みに失敗した場合、実行中の操作自体を中断する。

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;

use crate::audit::{AuditEntry, AuditWrite};
use crate::credentials::AccountCredentials;
use crate::error::{ExecutionApiError, ExecutionError};
use crate::identity::IdentityCheck;
use crate::planner::{ActionKind, PlannedAction};

/// `delete-log-group` / `put-retention-policy` 境界の抽象化。
#[async_trait]
pub trait ActionApiOperations: Send + Sync {
    async fn delete_log_group(
        &self,
        creds: &AccountCredentials,
        region: &str,
        log_group_name: &str,
    ) -> Result<(), String>;

    async fn put_retention_policy(
        &self,
        creds: &AccountCredentials,
        region: &str,
        log_group_name: &str,
        days: i32,
    ) -> Result<(), String>;
}

/// 1件の実行結果。
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionOutcome {
    pub action: PlannedAction,
    pub success: bool,
    pub error_message: Option<String>,
}

/// dry-run既定、`--execute`時のみ書き込みAPIを呼び出す実行エンジン。
pub struct ExecutionEngine {
    api_client: Arc<dyn ActionApiOperations>,
    identity: Arc<dyn IdentityCheck>,
    audit_logger: Arc<dyn AuditWrite>,
    run_id: String,
}

impl ExecutionEngine {
    pub fn new(
        api_client: Arc<dyn ActionApiOperations>,
        identity: Arc<dyn IdentityCheck>,
        audit_logger: Arc<dyn AuditWrite>,
        run_id: impl Into<String>,
    ) -> Self {
        Self {
            api_client,
            identity,
            audit_logger,
            run_id: run_id.into(),
        }
    }

    /// `execute=false`（既定）の場合は、確認済みの計画一覧をそのまま「実行予定」として
    /// 返すのみで、`delete-log-group`/`put-retention-policy`のAPI呼び出しコード自体を
    /// 一切通過しない。
    ///
    /// `execute=true`の場合のみ、各アクションにつき:
    /// 1. スキャン時点とは独立した二重目のIdentity検証を再実行する。
    /// 2. 検証通過後にAPIを呼び出す。
    /// 3. 結果を監査ログへ記録する（記録に失敗した場合は当該操作を中断しエラーとして扱う）。
    pub async fn execute_plan(
        &self,
        confirmed_actions: &[PlannedAction],
        credentials_by_account: impl Fn(&str) -> Option<AccountCredentials>,
        execute: bool,
    ) -> Result<Vec<ExecutionOutcome>, ExecutionError> {
        if !execute {
            // dry-run: 何も呼び出さず、実行予定の一覧を「未実行」として返す。
            return Ok(confirmed_actions
                .iter()
                .cloned()
                .map(|action| ExecutionOutcome {
                    action,
                    success: false,
                    error_message: Some("dry-run: --execute not supplied".to_string()),
                })
                .collect());
        }

        let mut outcomes = Vec::with_capacity(confirmed_actions.len());
        for action in confirmed_actions {
            let creds =
                credentials_by_account(&action.account_id).ok_or_else(|| ExecutionApiError {
                    account_id: action.account_id.clone(),
                    region: action.region.clone(),
                    log_group_name: action.log_group_name.clone(),
                    message: "no credentials resolved for account".to_string(),
                })?;

            // 削除・retention変更の直前に独立した二重目のIdentity検証を再実行する。
            self.identity.verify(&creds, &action.account_id).await?;

            let api_result = match action.action_kind {
                ActionKind::Delete => {
                    self.api_client
                        .delete_log_group(&creds, &action.region, &action.log_group_name)
                        .await
                }
                ActionKind::SetRetention { days } => {
                    self.api_client
                        .put_retention_policy(&creds, &action.region, &action.log_group_name, days)
                        .await
                }
            };

            let (success, error_message) = match &api_result {
                Ok(()) => (true, None),
                Err(msg) => (false, Some(msg.clone())),
            };

            let entry = AuditEntry {
                run_id: self.run_id.clone(),
                timestamp: Utc::now().to_rfc3339(),
                account_id: action.account_id.clone(),
                region: action.region.clone(),
                log_group_name: action.log_group_name.clone(),
                action_kind: action.action_kind,
                success,
                error_message: error_message.clone(),
            };

            // 監査ログ書き込みに失敗した場合、実行中の操作自体を中断しエラーとして扱う。
            self.audit_logger.append(&entry)?;

            if let Err(msg) = api_result {
                return Err(ExecutionApiError {
                    account_id: action.account_id.clone(),
                    region: action.region.clone(),
                    log_group_name: action.log_group_name.clone(),
                    message: msg,
                }
                .into());
            }

            outcomes.push(ExecutionOutcome {
                action: action.clone(),
                success,
                error_message,
            });
        }

        Ok(outcomes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditLogger;
    use crate::error::IdentityError;
    use secrecy::SecretString;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const ACCOUNT_ID: &str = "111111111111";
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

    fn planned_action(name: &str) -> PlannedAction {
        PlannedAction {
            account_id: ACCOUNT_ID.to_string(),
            region: REGION.to_string(),
            log_group_name: name.to_string(),
            action_kind: ActionKind::Delete,
            confirmed: true,
        }
    }

    struct CountingApiClient {
        delete_calls: AtomicUsize,
        retention_calls: AtomicUsize,
    }
    impl CountingApiClient {
        fn new() -> Self {
            Self {
                delete_calls: AtomicUsize::new(0),
                retention_calls: AtomicUsize::new(0),
            }
        }
    }
    #[async_trait]
    impl ActionApiOperations for CountingApiClient {
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
            self.retention_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    struct AlwaysOkIdentity {
        calls: AtomicUsize,
    }
    #[async_trait]
    impl IdentityCheck for AlwaysOkIdentity {
        async fn verify(&self, _c: &AccountCredentials, _e: &str) -> Result<(), IdentityError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
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

    fn engine(
        api: Arc<dyn ActionApiOperations>,
        identity: Arc<dyn IdentityCheck>,
        dir: &tempfile::TempDir,
    ) -> ExecutionEngine {
        let logger: Arc<dyn crate::audit::AuditWrite> =
            Arc::new(AuditLogger::open(&dir.path().join("audit.jsonl")).unwrap());
        ExecutionEngine::new(api, identity, logger, "run-1")
    }

    // --- Step 14 (TDD, Red→Green): dry-run既定・二重Identity検証の期待仕様 ---

    #[tokio::test]
    async fn dry_run_default_never_calls_delete_or_retention_api() {
        let dir = tempfile::tempdir().unwrap();
        let api = Arc::new(CountingApiClient::new());
        let identity = Arc::new(AlwaysOkIdentity {
            calls: AtomicUsize::new(0),
        });
        let eng = engine(api.clone(), identity.clone(), &dir);
        let actions = vec![planned_action("/a"), planned_action("/b")];

        // execute引数を一切指定しない固定テストケース（フラグ未指定＝false相当）。
        let execute_flag_not_supplied = false;
        let outcomes = eng
            .execute_plan(&actions, |id| Some(creds(id)), execute_flag_not_supplied)
            .await
            .unwrap();

        assert_eq!(api.delete_calls.load(Ordering::SeqCst), 0);
        assert_eq!(api.retention_calls.load(Ordering::SeqCst), 0);
        assert_eq!(identity.calls.load(Ordering::SeqCst), 0);
        assert_eq!(outcomes.len(), 2);
        assert!(outcomes.iter().all(|o| !o.success));
    }

    #[tokio::test]
    async fn execute_true_calls_delete_api_and_records_success() {
        let dir = tempfile::tempdir().unwrap();
        let api = Arc::new(CountingApiClient::new());
        let identity = Arc::new(AlwaysOkIdentity {
            calls: AtomicUsize::new(0),
        });
        let eng = engine(api.clone(), identity.clone(), &dir);
        let actions = vec![planned_action("/a")];

        let outcomes = eng
            .execute_plan(&actions, |id| Some(creds(id)), true)
            .await
            .unwrap();

        assert_eq!(api.delete_calls.load(Ordering::SeqCst), 1);
        assert!(outcomes[0].success);
    }

    #[tokio::test]
    async fn execute_true_runs_independent_second_identity_check_before_each_action() {
        let dir = tempfile::tempdir().unwrap();
        let api = Arc::new(CountingApiClient::new());
        let identity = Arc::new(AlwaysOkIdentity {
            calls: AtomicUsize::new(0),
        });
        let eng = engine(api, identity.clone(), &dir);
        let actions = vec![planned_action("/a"), planned_action("/b")];

        eng.execute_plan(&actions, |id| Some(creds(id)), true)
            .await
            .unwrap();

        // アクション数と同じ回数だけ、削除直前の二重目の検証が独立して呼ばれる。
        assert_eq!(identity.calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn execute_true_with_identity_mismatch_fails_and_never_calls_api() {
        let dir = tempfile::tempdir().unwrap();
        let api = Arc::new(CountingApiClient::new());
        let eng = engine(api.clone(), Arc::new(AlwaysFailIdentity), &dir);
        let actions = vec![planned_action("/a")];

        let result = eng.execute_plan(&actions, |id| Some(creds(id)), true).await;

        assert!(matches!(result, Err(ExecutionError::Identity(_))));
        assert_eq!(api.delete_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn execute_true_with_set_retention_calls_retention_api_not_delete() {
        let dir = tempfile::tempdir().unwrap();
        let api = Arc::new(CountingApiClient::new());
        let identity = Arc::new(AlwaysOkIdentity {
            calls: AtomicUsize::new(0),
        });
        let eng = engine(api.clone(), identity, &dir);
        let mut action = planned_action("/a");
        action.action_kind = ActionKind::SetRetention { days: 30 };

        eng.execute_plan(&[action], |id| Some(creds(id)), true)
            .await
            .unwrap();

        assert_eq!(api.retention_calls.load(Ordering::SeqCst), 1);
        assert_eq!(api.delete_calls.load(Ordering::SeqCst), 0);
    }

    /// 監査ログ書き込みが常に失敗するフェイク（`AuditLogger`本体の失敗経路は`audit.rs`側で
    /// 別途検証済みであり、ここでは`ExecutionEngine`が失敗を受けて操作を中断することのみを
    /// 決定的に検証する）。
    struct FailingAuditWriter;
    impl crate::audit::AuditWrite for FailingAuditWriter {
        fn append(&self, _entry: &AuditEntry) -> Result<(), crate::error::AuditWriteError> {
            Err(crate::error::AuditWriteError {
                message: "simulated disk full".to_string(),
            })
        }
    }

    struct FailingApiClientNeverCalled;
    #[async_trait]
    impl ActionApiOperations for FailingApiClientNeverCalled {
        async fn delete_log_group(
            &self,
            _c: &AccountCredentials,
            _r: &str,
            _l: &str,
        ) -> Result<(), String> {
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

    #[tokio::test]
    async fn audit_log_write_failure_aborts_the_operation() {
        let identity = Arc::new(AlwaysOkIdentity {
            calls: AtomicUsize::new(0),
        });
        let eng = ExecutionEngine::new(
            Arc::new(FailingApiClientNeverCalled),
            identity,
            Arc::new(FailingAuditWriter),
            "run-1",
        );
        let actions = vec![planned_action("/a")];

        let result = eng.execute_plan(&actions, |id| Some(creds(id)), true).await;

        assert!(matches!(result, Err(ExecutionError::AuditWrite(_))));
    }

    #[tokio::test]
    async fn audit_log_write_failure_aborts_the_operation_for_set_retention() {
        // R-01: 監査ログ書き込み失敗による中断は、`ActionKind::Delete`だけでなく
        // `ActionKind::SetRetention`の実行パス（`put_retention_policy`呼び出し）でも
        // 同様に成立することを決定的に検証する。
        let identity = Arc::new(AlwaysOkIdentity {
            calls: AtomicUsize::new(0),
        });
        let eng = ExecutionEngine::new(
            Arc::new(FailingApiClientNeverCalled),
            identity,
            Arc::new(FailingAuditWriter),
            "run-1",
        );
        let mut action = planned_action("/a");
        action.action_kind = ActionKind::SetRetention { days: 30 };

        let result = eng
            .execute_plan(&[action], |id| Some(creds(id)), true)
            .await;

        assert!(matches!(result, Err(ExecutionError::AuditWrite(_))));
    }

    #[tokio::test]
    async fn audit_log_write_failure_means_api_result_success_is_never_reported() {
        // APIは成功するがaudit書き込みが失敗するケースでも、呼び出し元へは
        // 成功として報告されず、必ずエラーとして扱われることを確認する（境界値）。
        let identity = Arc::new(AlwaysOkIdentity {
            calls: AtomicUsize::new(0),
        });
        let eng = ExecutionEngine::new(
            Arc::new(FailingApiClientNeverCalled),
            identity,
            Arc::new(FailingAuditWriter),
            "run-1",
        );
        let actions = vec![planned_action("/a")];

        let result = eng.execute_plan(&actions, |id| Some(creds(id)), true).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn execute_true_with_no_resolved_credentials_fails_without_calling_identity_or_api() {
        // 境界値: credentials_by_accountが該当アカウントの資格情報を持たない場合
        // （例えば呼び出し元の事前解決ロジックに不整合があった場合）でも、panicせずに
        // エラーとして扱う。
        let dir = tempfile::tempdir().unwrap();
        let api = Arc::new(CountingApiClient::new());
        let identity = Arc::new(AlwaysOkIdentity {
            calls: AtomicUsize::new(0),
        });
        let eng = engine(api.clone(), identity.clone(), &dir);
        let actions = vec![planned_action("/a")];

        let result = eng.execute_plan(&actions, |_id| None, true).await;

        assert!(matches!(result, Err(ExecutionError::Api(_))));
        assert_eq!(identity.calls.load(Ordering::SeqCst), 0);
        assert_eq!(api.delete_calls.load(Ordering::SeqCst), 0);
    }

    /// `delete-log-group`と`put-retention-policy`の両方が常に`AccessDenied`で失敗する
    /// フェイクAPIクライアント。削除・retention変更の両アクション種別で共有し、両方の
    /// 実装が確実に実行される（未使用のモックアームを残さない）。
    struct AlwaysFailingApiClient;
    #[async_trait]
    impl ActionApiOperations for AlwaysFailingApiClient {
        async fn delete_log_group(
            &self,
            _c: &AccountCredentials,
            _r: &str,
            _l: &str,
        ) -> Result<(), String> {
            Err("AccessDenied".to_string())
        }
        async fn put_retention_policy(
            &self,
            _c: &AccountCredentials,
            _r: &str,
            _l: &str,
            _d: i32,
        ) -> Result<(), String> {
            Err("AccessDenied".to_string())
        }
    }

    #[tokio::test]
    async fn execute_true_with_failing_delete_api_call_returns_api_error_after_audit_write() {
        let dir = tempfile::tempdir().unwrap();
        let audit_path = dir.path().join("audit.jsonl");
        let logger: Arc<dyn crate::audit::AuditWrite> =
            Arc::new(AuditLogger::open(&audit_path).unwrap());
        let identity = Arc::new(AlwaysOkIdentity {
            calls: AtomicUsize::new(0),
        });
        let eng = ExecutionEngine::new(Arc::new(AlwaysFailingApiClient), identity, logger, "run-1");
        let actions = vec![planned_action("/a")];

        let result = eng.execute_plan(&actions, |id| Some(creds(id)), true).await;

        assert!(matches!(result, Err(ExecutionError::Api(_))));
        // 失敗した実行結果も監査ログには記録されている（success=falseで1行）。
        let contents = std::fs::read_to_string(&audit_path).unwrap();
        assert_eq!(contents.lines().count(), 1);
        assert!(contents.contains("\"success\":false"));
    }

    #[tokio::test]
    async fn execute_true_with_failing_retention_api_call_returns_api_error_after_audit_write() {
        // R-01: `execute_true_with_failing_delete_api_call_returns_api_error_after_audit_write`の
        // retention版。`put_retention_policy`のAPI失敗経路も、削除と同様にAPIエラーとして
        // 呼び出し元へ伝播し、失敗結果が監査ログに記録されることを検証する。
        let dir = tempfile::tempdir().unwrap();
        let audit_path = dir.path().join("audit.jsonl");
        let logger: Arc<dyn crate::audit::AuditWrite> =
            Arc::new(AuditLogger::open(&audit_path).unwrap());
        let identity = Arc::new(AlwaysOkIdentity {
            calls: AtomicUsize::new(0),
        });
        let eng = ExecutionEngine::new(Arc::new(AlwaysFailingApiClient), identity, logger, "run-1");
        let mut action = planned_action("/a");
        action.action_kind = ActionKind::SetRetention { days: 30 };

        let result = eng
            .execute_plan(&[action], |id| Some(creds(id)), true)
            .await;

        assert!(matches!(result, Err(ExecutionError::Api(_))));
        let contents = std::fs::read_to_string(&audit_path).unwrap();
        assert_eq!(contents.lines().count(), 1);
        assert!(contents.contains("\"success\":false"));
    }
}
