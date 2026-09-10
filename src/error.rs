//! 共通エラー型。
//!
//! Domain Design / NFR Design (security-design.md, reliability-design.md) に基づき、
//! すべてのエラーは `thiserror` ベースで定義し、`unwrap()`/`expect()`/`panic!` を用いず
//! 呼び出し元へ `Result` として伝播させる。

use thiserror::Error;

/// アカウントID不一致エラー (NFR2.3, NFR2.5)。
///
/// `sts:get-caller-identity` が返す実アカウントIDが期待値と一致しない場合に
/// 即座に返され、当該アカウントの処理を継続させない。
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("account identity mismatch: expected {expected}, actual {actual:?}")]
pub struct IdentityMismatchError {
    pub expected: String,
    pub actual: Option<String>,
}

/// `sts:get-caller-identity` 呼び出し自体の失敗。
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("get-caller-identity call failed: {message}")]
pub struct CallerIdentityCallError {
    pub message: String,
}

/// Identity検証全体のエラー。呼び出し失敗と不一致を区別する。
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum IdentityError {
    #[error(transparent)]
    Mismatch(#[from] IdentityMismatchError),
    #[error(transparent)]
    CallFailed(#[from] CallerIdentityCallError),
}

/// `sts:AssumeRole` 呼び出しの失敗。
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("assume-role failed for account {account_id} (role: {role_name}): {message}")]
pub struct AssumeRoleError {
    pub account_id: String,
    pub role_name: String,
    pub message: String,
}

/// 監査ログ書き込みの失敗 (NFR4.3)。
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("audit log write failed: {message}")]
pub struct AuditWriteError {
    pub message: String,
}

/// ページネーション途中でのエラー。
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("pagination failed while listing log groups for account {account_id} region {region} (page {page_index}): {message}")]
pub struct PaginationError {
    pub account_id: String,
    pub region: String,
    pub page_index: usize,
    pub message: String,
}

/// Organizationsアカウント列挙の失敗。
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("organizations list-accounts failed: {message}")]
pub struct OrgDiscoveryError {
    pub message: String,
}

/// スキャン層全体のエラー。
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ScanError {
    #[error(transparent)]
    Identity(#[from] IdentityError),
    #[error(transparent)]
    Pagination(#[from] PaginationError),
    #[error(transparent)]
    AssumeRole(#[from] AssumeRoleError),
}

/// アクション実行（delete-log-group / put-retention-policy）の失敗。
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("execution failed for {account_id}/{region}/{log_group_name}: {message}")]
pub struct ExecutionApiError {
    pub account_id: String,
    pub region: String,
    pub log_group_name: String,
    pub message: String,
}

/// `ExecutionEngine` 全体のエラー。
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    #[error(transparent)]
    Identity(#[from] IdentityError),
    #[error(transparent)]
    Api(#[from] ExecutionApiError),
    #[error(transparent)]
    AuditWrite(#[from] AuditWriteError),
    #[error(transparent)]
    AssumeRole(#[from] AssumeRoleError),
    /// `--execute` が指定されていないのに実行APIへ到達しようとした場合の防御的エラー。
    /// 通常はdry-run分岐で到達しないが、構造上の保険として用意する。
    #[error("refusing to execute: --execute flag was not supplied (dry-run is the default)")]
    ExecuteFlagNotSet,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_mismatch_error_message_includes_both_ids() {
        let err = IdentityMismatchError {
            expected: "111111111111".into(),
            actual: Some("222222222222".into()),
        };
        let msg = err.to_string();
        assert!(msg.contains("111111111111"));
        assert!(msg.contains("222222222222"));
    }

    #[test]
    fn identity_error_from_mismatch_variant() {
        let mismatch = IdentityMismatchError {
            expected: "111111111111".into(),
            actual: None,
        };
        let err: IdentityError = mismatch.clone().into();
        match err {
            IdentityError::Mismatch(m) => assert_eq!(m, mismatch),
            _ => panic!("expected Mismatch variant"),
        }
    }

    #[test]
    fn scan_error_from_pagination_error() {
        let pag = PaginationError {
            account_id: "111111111111".into(),
            region: "us-east-1".into(),
            page_index: 2,
            message: "boom".into(),
        };
        let err: ScanError = pag.into();
        assert!(matches!(err, ScanError::Pagination(_)));
    }

    #[test]
    fn execution_error_execute_flag_not_set_message() {
        let err = ExecutionError::ExecuteFlagNotSet;
        assert!(err.to_string().contains("--execute"));
    }

    #[test]
    fn audit_write_error_roundtrip_into_execution_error() {
        let audit_err = AuditWriteError {
            message: "disk full".into(),
        };
        let err: ExecutionError = audit_err.clone().into();
        match err {
            ExecutionError::AuditWrite(a) => assert_eq!(a, audit_err),
            _ => panic!("expected AuditWrite variant"),
        }
    }
}
