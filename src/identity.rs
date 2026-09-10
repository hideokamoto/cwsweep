//! `IdentityVerifier` コンポーネント（安全パス・TDD: Step 5）。
//!
//! project.md Mandated: いずれかのメンバーアカウントに対するAWS API呼び出しの直前に
//! `sts:get-caller-identity` を実行し、想定アカウントIDと一致することを検証する。
//! 不一致の場合は警告のみで続行せず、そのアカウントの処理を即座に失敗させる。
//! 削除・retention変更の実行直前には、スキャン時点とは独立した二重目の検証を再実行する。

use async_trait::async_trait;

use crate::credentials::AccountCredentials;
use crate::error::{CallerIdentityCallError, IdentityError, IdentityMismatchError};

/// `sts:get-caller-identity` 境界の抽象化。テストでは手書きモックを注入する。
#[async_trait]
pub trait CallerIdentityOperations: Send + Sync {
    /// 実際に呼び出し先が認識しているアカウントIDを返す。
    async fn get_caller_identity(
        &self,
        creds: &AccountCredentials,
    ) -> Result<String, CallerIdentityCallError>;
}

/// Identity検証を行うコンポーネント。副作用のない検証関数として実装し、
/// 呼び出し元（LogGroupScanner起動前、ExecutionEngine実行直前の二重目）から
/// 独立して何度でも呼び出せる。
pub struct IdentityVerifier<C: CallerIdentityOperations> {
    client: C,
}

impl<C: CallerIdentityOperations> IdentityVerifier<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    /// `expected_account_id` と実際のアカウントIDが一致するか検証する。
    ///
    /// 不一致・呼び出し失敗のいずれの場合も `Err` を返し、呼び出し元は `?` で
    /// 即座に処理を打ち切ることができる（警告のみでの継続を構造的に禁止する）。
    pub async fn verify_identity(
        &self,
        creds: &AccountCredentials,
        expected_account_id: &str,
    ) -> Result<(), IdentityError> {
        let actual = self.client.get_caller_identity(creds).await?;
        if actual != expected_account_id {
            return Err(IdentityMismatchError {
                expected: expected_account_id.to_string(),
                actual: Some(actual),
            }
            .into());
        }
        Ok(())
    }
}

/// `LogGroupScanner` / `ExecutionEngine` から二重目の検証を含めて独立に呼び出せるよう、
/// 具象型に依存しない形でIdentity検証を抽象化するトレイト。
#[async_trait]
pub trait IdentityCheck: Send + Sync {
    async fn verify(
        &self,
        creds: &AccountCredentials,
        expected_account_id: &str,
    ) -> Result<(), IdentityError>;
}

#[async_trait]
impl<C: CallerIdentityOperations> IdentityCheck for IdentityVerifier<C> {
    async fn verify(
        &self,
        creds: &AccountCredentials,
        expected_account_id: &str,
    ) -> Result<(), IdentityError> {
        self.verify_identity(creds, expected_account_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::SecretString;

    const EXPECTED_ACCOUNT_ID: &str = "111111111111";
    const OTHER_ACCOUNT_ID: &str = "222222222222";

    fn creds() -> AccountCredentials {
        AccountCredentials {
            account_id: EXPECTED_ACCOUNT_ID.to_string(),
            access_key_id: "AKIAFIXTURE".to_string(),
            secret_access_key: SecretString::from("fixture-secret".to_string()),
            session_token: SecretString::from("fixture-token".to_string()),
            expiration: None,
        }
    }

    struct StubCallerIdentity {
        returns: String,
    }

    #[async_trait]
    impl CallerIdentityOperations for StubCallerIdentity {
        async fn get_caller_identity(
            &self,
            _creds: &AccountCredentials,
        ) -> Result<String, CallerIdentityCallError> {
            Ok(self.returns.clone())
        }
    }

    struct FailingCallerIdentity;

    #[async_trait]
    impl CallerIdentityOperations for FailingCallerIdentity {
        async fn get_caller_identity(
            &self,
            _creds: &AccountCredentials,
        ) -> Result<String, CallerIdentityCallError> {
            Err(CallerIdentityCallError {
                message: "network error".to_string(),
            })
        }
    }

    // --- Step 5 (TDD, Red→Green): 期待仕様を先にテストとして書く ---

    #[tokio::test]
    async fn matching_account_id_passes_verification() {
        let verifier = IdentityVerifier::new(StubCallerIdentity {
            returns: EXPECTED_ACCOUNT_ID.to_string(),
        });

        let result = verifier
            .verify_identity(&creds(), EXPECTED_ACCOUNT_ID)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn mismatched_account_id_fails_immediately_without_continuing() {
        let verifier = IdentityVerifier::new(StubCallerIdentity {
            returns: OTHER_ACCOUNT_ID.to_string(),
        });

        let result = verifier
            .verify_identity(&creds(), EXPECTED_ACCOUNT_ID)
            .await;

        let err = result.expect_err("mismatch must fail, never continue with only a warning");
        match err {
            IdentityError::Mismatch(m) => {
                assert_eq!(m.expected, EXPECTED_ACCOUNT_ID);
                assert_eq!(m.actual.as_deref(), Some(OTHER_ACCOUNT_ID));
            }
            other => panic!("expected Mismatch, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn caller_identity_call_failure_propagates_as_error() {
        let verifier = IdentityVerifier::new(FailingCallerIdentity);

        let result = verifier
            .verify_identity(&creds(), EXPECTED_ACCOUNT_ID)
            .await;

        assert!(matches!(result, Err(IdentityError::CallFailed(_))));
    }

    #[tokio::test]
    async fn verification_can_be_re_run_independently_a_second_time() {
        // スキャン時点の検証と、削除直前の二重目の検証が独立して呼び出されることを模擬する。
        let verifier = IdentityVerifier::new(StubCallerIdentity {
            returns: EXPECTED_ACCOUNT_ID.to_string(),
        });

        let first = verifier
            .verify_identity(&creds(), EXPECTED_ACCOUNT_ID)
            .await;
        let second = verifier
            .verify_identity(&creds(), EXPECTED_ACCOUNT_ID)
            .await;

        assert!(first.is_ok());
        assert!(second.is_ok());
    }

    #[tokio::test]
    async fn second_independent_verification_can_fail_even_if_first_succeeded() {
        // 境界値: リージョン跨ぎ等で状態が変わり、二重目の検証だけが不一致になるケース。
        // ここでは「同じベリファイアインスタンスへの2回目の呼び出しが独立して失敗しうる」ことを
        // 別クライアント（状態が変化した想定）で表現する。
        let verifier = IdentityVerifier::new(StubCallerIdentity {
            returns: OTHER_ACCOUNT_ID.to_string(),
        });

        let result = verifier
            .verify_identity(&creds(), EXPECTED_ACCOUNT_ID)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn identity_check_trait_object_delegates_to_verify_identity() {
        // scanner/executionはIdentityVerifierを`Arc<dyn IdentityCheck>`として利用するため、
        // トレイト経由の呼び出し（verify）でもverify_identityと同じ結果になることを確認する。
        let verifier: std::sync::Arc<dyn IdentityCheck> =
            std::sync::Arc::new(IdentityVerifier::new(StubCallerIdentity {
                returns: EXPECTED_ACCOUNT_ID.to_string(),
            }));

        let ok_result = verifier.verify(&creds(), EXPECTED_ACCOUNT_ID).await;
        assert!(ok_result.is_ok());

        let mismatch_verifier: std::sync::Arc<dyn IdentityCheck> =
            std::sync::Arc::new(IdentityVerifier::new(StubCallerIdentity {
                returns: OTHER_ACCOUNT_ID.to_string(),
            }));
        let mismatch_result = mismatch_verifier
            .verify(&creds(), EXPECTED_ACCOUNT_ID)
            .await;
        assert!(mismatch_result.is_err());
    }

    #[tokio::test]
    async fn empty_expected_account_id_is_treated_as_mismatch_boundary() {
        let verifier = IdentityVerifier::new(StubCallerIdentity {
            returns: EXPECTED_ACCOUNT_ID.to_string(),
        });

        let result = verifier.verify_identity(&creds(), "").await;

        assert!(result.is_err());
    }
}
