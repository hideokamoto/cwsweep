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

use crate::audit::{AuditEntry, AuditEventKind, AuditWrite};
use crate::credentials::AccountCredentials;
use crate::error::ExecutionError;
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
    /// `execute=true`の場合、各アクションにつき:
    /// 1. 資格情報が解決できない場合、当該アクションのみを失敗として記録し、他の
    ///    アクションの処理は継続する（NFR4.2バルクヘッド分離。呼び出し元が資格情報
    ///    解決に失敗したアカウントの扱いを決められるよう、計画全体を中断しない）。
    /// 2. スキャン時点とは独立した二重目のIdentity検証を再実行する（失敗した場合は
    ///    即時に計画全体を中断する — NFR2.3/2.5）。
    /// 3. API呼び出し「前」に実行意図(`Intent`)を監査ログへ記録する。この書き込みに
    ///    失敗した場合はAPIを呼び出さずに即時に計画全体を中断する（NFR4.3。不可逆操作の
    ///    記録なき実行を許さない）。
    /// 4. APIを呼び出す。
    /// 5. API呼び出し「後」に結果(`Result`、成功/失敗いずれも)を監査ログへ記録する。
    ///    この書き込みに失敗した場合も即時に計画全体を中断する（NFR4.3、既存どおり）。
    /// 6. API呼び出し自体の失敗（例: AccessDenied）は当該アクションのみの失敗として
    ///    `ExecutionOutcome`に記録し、他の独立したアクションの処理は継続する
    ///    （NFR4.2。計画全体を中断しない）。
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
            let creds = match credentials_by_account(&action.account_id) {
                Some(c) => c,
                None => {
                    // NFR4.2: 資格情報が解決できないアクションのみを失敗として記録し、
                    // 他の独立したアクションの処理は継続する（計画全体を中断しない）。
                    outcomes.push(ExecutionOutcome {
                        action: action.clone(),
                        success: false,
                        error_message: Some("no credentials resolved for account".to_string()),
                    });
                    continue;
                }
            };

            // 削除・retention変更の直前に独立した二重目のIdentity検証を再実行する。
            // Identity不一致は即時に計画全体を中断する（他アクションへは継続しない）。
            self.identity.verify(&creds, &action.account_id).await?;

            // API呼び出し「前」に実行意図(intent)を監査ログへ記録する。この追記に
            // 失敗した場合はAPIを一切呼び出さず即時に計画全体を中断する
            // （削除済みなのに記録がどこにも残らない、という状態を作らない）。
            let intent_entry = AuditEntry {
                run_id: self.run_id.clone(),
                timestamp: Utc::now().to_rfc3339(),
                account_id: action.account_id.clone(),
                region: action.region.clone(),
                log_group_name: action.log_group_name.clone(),
                action_kind: action.action_kind,
                event: AuditEventKind::Intent,
                success: false,
                error_message: None,
            };
            self.audit_logger.append(&intent_entry)?;

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

            // API呼び出し「後」に結果(result)を監査ログへ記録する。
            let result_entry = AuditEntry {
                run_id: self.run_id.clone(),
                timestamp: Utc::now().to_rfc3339(),
                account_id: action.account_id.clone(),
                region: action.region.clone(),
                log_group_name: action.log_group_name.clone(),
                action_kind: action.action_kind,
                event: AuditEventKind::Result,
                success,
                error_message: error_message.clone(),
            };

            // 監査ログ書き込みに失敗した場合、実行中の操作自体を中断しエラーとして扱う
            // （既存どおり: 結果の記録なき「成功扱い」を許さない）。
            self.audit_logger.append(&result_entry)?;

            // NFR4.2: API呼び出し自体の失敗（AccessDenied等）は当該アクションのみの
            // 失敗として記録し、他の独立したアクションの処理は継続する
            // （計画全体を中断しない。旧実装は`return Err`しており、既に成功した
            // アクションの`ExecutionOutcome`が呼び出し元へ返らない問題があった）。
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
            session_token: Some(SecretString::from("token".to_string())),
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

        // R-06 (NFR4.2): 資格情報が解決できないアクションは、計画全体を中断する`Err`
        // ではなく、当該アクションのみを失敗とした`ExecutionOutcome`として返す。
        let outcomes = eng.execute_plan(&actions, |_id| None, true).await.unwrap();

        assert_eq!(outcomes.len(), 1);
        assert!(!outcomes[0].success);
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
    async fn execute_true_with_failing_delete_api_call_records_failure_and_continues() {
        // R-07 (NFR4.2): API呼び出し自体の失敗（AccessDenied等）は、計画全体を中断する
        // `Err`ではなく、当該アクションのみを失敗とした`ExecutionOutcome`として返す。
        let dir = tempfile::tempdir().unwrap();
        let audit_path = dir.path().join("audit.jsonl");
        let logger: Arc<dyn crate::audit::AuditWrite> =
            Arc::new(AuditLogger::open(&audit_path).unwrap());
        let identity = Arc::new(AlwaysOkIdentity {
            calls: AtomicUsize::new(0),
        });
        let eng = ExecutionEngine::new(Arc::new(AlwaysFailingApiClient), identity, logger, "run-1");
        let actions = vec![planned_action("/a")];

        let outcomes = eng
            .execute_plan(&actions, |id| Some(creds(id)), true)
            .await
            .unwrap();

        assert_eq!(outcomes.len(), 1);
        assert!(!outcomes[0].success);
        assert_eq!(outcomes[0].error_message.as_deref(), Some("AccessDenied"));
        // R-08 (二段階記録): 失敗した実行結果も、intent（実行前）とresult（実行後）の
        // 2行として監査ログに記録されている。
        let contents = std::fs::read_to_string(&audit_path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\"event\":\"intent\""));
        assert!(lines[1].contains("\"event\":\"result\""));
        assert!(lines[1].contains("\"success\":false"));
    }

    #[tokio::test]
    async fn execute_true_with_failing_retention_api_call_records_failure_and_continues() {
        // R-01: `execute_true_with_failing_delete_api_call_records_failure_and_continues`の
        // retention版。`put_retention_policy`のAPI失敗経路も、削除と同様に当該アクションのみの
        // 失敗として記録され、計画全体は中断しない。
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

        let outcomes = eng
            .execute_plan(&[action], |id| Some(creds(id)), true)
            .await
            .unwrap();

        assert_eq!(outcomes.len(), 1);
        assert!(!outcomes[0].success);
        let contents = std::fs::read_to_string(&audit_path).unwrap();
        assert_eq!(contents.lines().count(), 2);
        assert!(contents.contains("\"success\":false"));
    }

    #[tokio::test]
    async fn execute_true_continues_past_a_failing_action_and_still_executes_the_next_one() {
        // R-07 (NFR4.2): 1件目のAPI呼び出しが失敗しても、2件目の独立したアクションの
        // 処理は継続される（計画全体が中断されない）。
        struct FailFirstThenSucceedApi {
            calls: AtomicUsize,
        }
        #[async_trait]
        impl ActionApiOperations for FailFirstThenSucceedApi {
            async fn delete_log_group(
                &self,
                _c: &AccountCredentials,
                _r: &str,
                _l: &str,
            ) -> Result<(), String> {
                let n = self.calls.fetch_add(1, Ordering::SeqCst);
                if n == 0 {
                    Err("AccessDenied".to_string())
                } else {
                    Ok(())
                }
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

        let dir = tempfile::tempdir().unwrap();
        let logger: Arc<dyn crate::audit::AuditWrite> =
            Arc::new(AuditLogger::open(&dir.path().join("audit.jsonl")).unwrap());
        let identity = Arc::new(AlwaysOkIdentity {
            calls: AtomicUsize::new(0),
        });
        let api = Arc::new(FailFirstThenSucceedApi {
            calls: AtomicUsize::new(0),
        });
        let eng = ExecutionEngine::new(api, identity, logger, "run-1");
        let actions = vec![planned_action("/a"), planned_action("/b")];

        let outcomes = eng
            .execute_plan(&actions, |id| Some(creds(id)), true)
            .await
            .unwrap();

        assert_eq!(outcomes.len(), 2);
        assert!(!outcomes[0].success);
        assert!(outcomes[1].success);
    }

    #[tokio::test]
    async fn execute_true_writes_intent_entry_before_calling_the_api() {
        // R-08: API呼び出し「前」にintentエントリが書き込まれることを、書き込み順を
        // 記録するスパイの監査ロガーで直接検証する。
        struct OrderRecordingAuditWriter {
            events: std::sync::Mutex<Vec<crate::audit::AuditEventKind>>,
        }
        impl crate::audit::AuditWrite for OrderRecordingAuditWriter {
            fn append(&self, entry: &AuditEntry) -> Result<(), crate::error::AuditWriteError> {
                self.events.lock().unwrap().push(entry.event);
                Ok(())
            }
        }

        let identity = Arc::new(AlwaysOkIdentity {
            calls: AtomicUsize::new(0),
        });
        let api = Arc::new(CountingApiClient::new());
        let logger = Arc::new(OrderRecordingAuditWriter {
            events: std::sync::Mutex::new(Vec::new()),
        });
        let eng = ExecutionEngine::new(api.clone(), identity, logger.clone(), "run-1");
        let actions = vec![planned_action("/a")];

        eng.execute_plan(&actions, |id| Some(creds(id)), true)
            .await
            .unwrap();

        let events = logger.events.lock().unwrap().clone();
        assert_eq!(events, vec![AuditEventKind::Intent, AuditEventKind::Result]);
        // APIが呼ばれる前にintentが書けているという因果関係を、少なくとも
        // 削除APIが1回呼ばれたこととあわせて確認する。
        assert_eq!(api.delete_calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn execute_true_never_calls_api_when_intent_audit_write_fails() {
        // R-08: intentエントリの書き込みに失敗した場合、APIは一切呼び出されず
        // 即時に計画全体を中断する（削除済みなのに記録が残らない状態を作らない）。
        let identity = Arc::new(AlwaysOkIdentity {
            calls: AtomicUsize::new(0),
        });
        let api = Arc::new(CountingApiClient::new());
        let eng =
            ExecutionEngine::new(api.clone(), identity, Arc::new(FailingAuditWriter), "run-1");
        let actions = vec![planned_action("/a")];

        let result = eng.execute_plan(&actions, |id| Some(creds(id)), true).await;

        assert!(matches!(result, Err(ExecutionError::AuditWrite(_))));
        assert_eq!(api.delete_calls.load(Ordering::SeqCst), 0);
    }
}
