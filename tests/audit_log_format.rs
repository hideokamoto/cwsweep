//! 監査ログの出力内容（フォーマット・必須フィールド）を検証する専用統合テスト。

use cwsweep::audit::{AuditEntry, AuditEventKind, AuditLogger};
use cwsweep::planner::ActionKind;

fn sample_entry(success: bool) -> AuditEntry {
    AuditEntry {
        run_id: "11111111-1111-1111-1111-111111111111".to_string(),
        timestamp: "2026-09-10T12:00:00+00:00".to_string(),
        account_id: "333333333333".to_string(),
        region: "ap-northeast-1".to_string(),
        log_group_name: "/aws/lambda/example".to_string(),
        action_kind: ActionKind::Delete,
        event: AuditEventKind::Result,
        success,
        error_message: if success {
            None
        } else {
            Some("AccessDenied".to_string())
        },
    }
}

#[test]
fn audit_log_is_valid_json_lines_with_all_mandated_fields() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("audit.jsonl");
    let logger = AuditLogger::open(&path).unwrap();

    logger.append(&sample_entry(true)).unwrap();
    logger.append(&sample_entry(false)).unwrap();

    let contents = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(lines.len(), 2);

    for line in &lines {
        let value: serde_json::Value = serde_json::from_str(line).unwrap();
        // project.md Mandated: 対象アカウントID・リージョン・ログループ名・実行時刻・成功/失敗
        for field in [
            "account_id",
            "region",
            "log_group_name",
            "timestamp",
            "success",
            "run_id",
        ] {
            assert!(
                value.get(field).is_some(),
                "audit log line is missing mandated field {field}: {line}"
            );
        }
    }

    let success_entry: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(success_entry["success"], true);
    assert!(success_entry["error_message"].is_null());

    let failure_entry: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(failure_entry["success"], false);
    assert_eq!(failure_entry["error_message"], "AccessDenied");
}

#[test]
fn audit_log_write_failure_is_reported_as_error_not_silently_ignored() {
    // project.md Mandated: 監査ログへの書き込みに失敗した場合、実行中の操作自体を中断し、
    // エラーとして扱う（無効化オプションは設けない）。ここではAuditLogger単体で、
    // 開けないパスを与えたときにErrが返ることを確認する。
    let bogus_path = std::path::Path::new("/nonexistent-dir-for-cwsweep-integration/audit.jsonl");

    let result = AuditLogger::open(bogus_path);

    assert!(result.is_err());
}
