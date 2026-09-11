//! `AuditLogger` コンポーネント（安全パス・TDD: Step 15）。
//!
//! project.md Mandated: 削除・retention変更を伴うすべての操作について、対象アカウントID・
//! リージョン・ログループ名・実行時刻・成功/失敗を監査ログに出力する。無効化オプションは設けない。
//! project.md Mandated: 監査ログへの書き込みに失敗した場合、実行中の操作自体を中断し、エラーとして扱う。

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::error::AuditWriteError;
use crate::planner::ActionKind;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEntry {
    pub run_id: String,
    pub timestamp: String,
    pub account_id: String,
    pub region: String,
    pub log_group_name: String,
    pub action_kind: ActionKind,
    pub success: bool,
    pub error_message: Option<String>,
}

/// 監査ログ書き込み境界の抽象化。`ExecutionEngine`はこのトレイト経由でのみ監査ログへ
/// アクセスし、テストでは書き込み失敗を模擬したフェイクを注入できる。
pub trait AuditWrite: Send + Sync {
    fn append(&self, entry: &AuditEntry) -> Result<(), AuditWriteError>;
}

/// 実際の書き込み先（本番では`File`）を抽象化するトレイト。テストでは書き込み・fsyncの
/// 失敗を決定的に再現するフェイクを注入できる。
trait SyncWrite: Write + Send {
    fn sync(&mut self) -> io::Result<()>;
}

impl SyncWrite for File {
    fn sync(&mut self) -> io::Result<()> {
        self.sync_data()
    }
}

/// JSON Linesを1エントリずつ`fsync`付きで追記する監査ログ書き込みコンポーネント。
/// 無効化するオプションは意図的に存在しない。
pub struct AuditLogger {
    file: Mutex<Box<dyn SyncWrite>>,
}

impl AuditWrite for AuditLogger {
    fn append(&self, entry: &AuditEntry) -> Result<(), AuditWriteError> {
        AuditLogger::append(self, entry)
    }
}

impl AuditLogger {
    pub fn open(path: &std::path::Path) -> Result<Self, AuditWriteError> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| AuditWriteError {
                message: format!("failed to open audit log file {}: {e}", path.display()),
            })?;
        Ok(Self::from_writer(Box::new(file)))
    }

    fn from_writer(writer: Box<dyn SyncWrite>) -> Self {
        Self {
            file: Mutex::new(writer),
        }
    }

    /// 1エントリを追記し、`fsync`（`sync_data`）まで完了させる。
    ///
    /// 書き込み・fsyncのいずれかが失敗した場合は`Err`を返す。呼び出し元
    /// （`ExecutionEngine`）はこれを`?`で伝播させ、当該操作を中断しなければならない。
    pub fn append(&self, entry: &AuditEntry) -> Result<(), AuditWriteError> {
        // R-01: このエラー分岐は構造的に到達不能である。`AuditEntry`のフィールドは
        // `String` / `bool` / `Option<String>` / `ActionKind`（フィールドなしバリアントを
        // 持つ列挙型）のみで構成され、非UTF-8マップキーや`NaN`/`Infinity`浮動小数点など
        // `serde_json::to_string`が失敗しうる要素を一切含まない。そのため呼び出し元の
        // フィールド構成が変わらない限り`Err`は発生せず、決定的に失敗させるテストは
        // 書けない（`AuditEntry`自体を不正な状態にする手段が公開APIに存在しない）。
        let line = serde_json::to_string(entry).map_err(|e| AuditWriteError {
            message: format!("failed to serialize audit entry: {e}"),
        })?;

        let mut file = self.file.lock().map_err(|_| AuditWriteError {
            message: "audit log file mutex poisoned".to_string(),
        })?;

        file.write_all(line.as_bytes())
            .map_err(|e| AuditWriteError {
                message: format!("failed to write audit log line: {e}"),
            })?;
        file.write_all(b"\n").map_err(|e| AuditWriteError {
            message: format!("failed to write audit log newline: {e}"),
        })?;
        file.sync().map_err(|e| AuditWriteError {
            message: format!("failed to fsync audit log: {e}"),
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn entry() -> AuditEntry {
        AuditEntry {
            run_id: "a1b2c3d4-0000-0000-0000-000000000000".to_string(),
            timestamp: "2026-09-10T12:00:00Z".to_string(),
            account_id: "111111111111".to_string(),
            region: "us-east-1".to_string(),
            log_group_name: "/aws/lambda/foo".to_string(),
            action_kind: ActionKind::Delete,
            success: true,
            error_message: None,
        }
    }

    // --- Step 15 (TDD, Red→Green): 書き込み失敗時は操作を中断する期待仕様 ---

    #[test]
    fn append_succeeds_and_writes_one_json_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");
        let logger = AuditLogger::open(&path).unwrap();

        let result = logger.append(&entry());

        assert!(result.is_ok());
        let mut contents = String::new();
        File::open(&path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        assert_eq!(contents.lines().count(), 1);
        let parsed: AuditEntry = serde_json::from_str(contents.lines().next().unwrap()).unwrap();
        assert_eq!(parsed, entry());
    }

    #[test]
    fn append_writes_required_fields_account_region_log_group_timestamp_success() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");
        let logger = AuditLogger::open(&path).unwrap();

        logger.append(&entry()).unwrap();

        let mut contents = String::new();
        File::open(&path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(contents.trim()).unwrap();
        for field in [
            "account_id",
            "region",
            "log_group_name",
            "timestamp",
            "success",
            "run_id",
        ] {
            assert!(value.get(field).is_some(), "missing required field {field}");
        }
    }

    #[test]
    fn append_to_unwritable_path_fails_with_audit_write_error() {
        // 書き込み不能なパス（存在しないディレクトリ配下）を与えて失敗をシミュレートする。
        let bogus_path = std::path::Path::new("/nonexistent-dir-for-cwsweep-tests/audit.jsonl");

        let result = AuditLogger::open(bogus_path);

        assert!(result.is_err());
    }

    #[test]
    fn multiple_appends_each_produce_one_line_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");
        let logger = AuditLogger::open(&path).unwrap();

        let mut first = entry();
        first.log_group_name = "/a".to_string();
        let mut second = entry();
        second.log_group_name = "/b".to_string();

        logger.append(&first).unwrap();
        logger.append(&second).unwrap();

        let mut contents = String::new();
        File::open(&path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        let lines: Vec<&str> = contents.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\"/a\""));
        assert!(lines[1].contains("\"/b\""));
    }

    #[test]
    fn failed_entry_includes_error_message_field() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");
        let logger = AuditLogger::open(&path).unwrap();

        let mut failure = entry();
        failure.success = false;
        failure.error_message = Some("delete-log-group denied".to_string());
        logger.append(&failure).unwrap();

        let mut contents = String::new();
        File::open(&path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(contents.trim()).unwrap();
        assert_eq!(value["success"], false);
        assert_eq!(value["error_message"], "delete-log-group denied");
    }

    // R-01: 以下のフェイク書き込み先群における`flush()`実装は、`std::io::Write`トレイトの
    // 必須メソッドを満たすためだけに存在し、構造的に到達不能である。`AuditLogger::append`は
    // `write_all`と`SyncWrite::sync`のみを呼び出し、`flush()`を一切呼ばないため
    // （明示的な`fsync`で永続化を保証する設計であり、バッファのフラッシュは不要）、
    // どのテストでもこの分岐には到達しない。同様に`FailingWriteAllWriter`/
    // `FailingOnSecondWriteWriter`の`sync()`実装も、それぞれ`write`が`?`で先にエラーを
    // 返して処理が中断するため到達不能であり、これは「1回の失敗だけを決定的に再現する」
    // という各フェイクの設計上の意図そのものである。到達可能にするための改変
    // （＝`sync`を先に失敗させる等）は、フェイクの目的自体を損なうため行わない。

    /// 常に`write_all`が失敗するフェイク書き込み先。
    struct FailingWriteAllWriter;
    impl Write for FailingWriteAllWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("simulated write failure"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    impl SyncWrite for FailingWriteAllWriter {
        fn sync(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    /// 書き込みには成功するが`fsync`が失敗するフェイク書き込み先。
    struct FailingSyncWriter;
    impl Write for FailingSyncWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    impl SyncWrite for FailingSyncWriter {
        fn sync(&mut self) -> io::Result<()> {
            Err(io::Error::other("simulated fsync failure"))
        }
    }

    #[test]
    fn append_fails_when_underlying_write_fails() {
        let logger = AuditLogger::from_writer(Box::new(FailingWriteAllWriter));

        let result = logger.append(&entry());

        assert!(result.is_err());
    }

    /// 最初の`write_all`（本文）は成功するが、2回目（改行）が失敗するフェイク。
    struct FailingOnSecondWriteWriter {
        calls: std::sync::atomic::AtomicUsize,
    }
    impl Write for FailingOnSecondWriteWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
                Ok(buf.len())
            } else {
                Err(io::Error::other("simulated newline write failure"))
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    impl SyncWrite for FailingOnSecondWriteWriter {
        fn sync(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn append_fails_when_newline_write_fails_after_body_write_succeeds() {
        let logger = AuditLogger::from_writer(Box::new(FailingOnSecondWriteWriter {
            calls: std::sync::atomic::AtomicUsize::new(0),
        }));

        let result = logger.append(&entry());

        assert!(result.is_err());
    }

    #[test]
    fn append_fails_when_fsync_fails_even_though_write_succeeded() {
        let logger = AuditLogger::from_writer(Box::new(FailingSyncWriter));

        let result = logger.append(&entry());

        assert!(result.is_err());
    }

    #[test]
    fn append_fails_with_clear_error_when_underlying_mutex_is_poisoned() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");
        let logger = std::sync::Arc::new(AuditLogger::open(&path).unwrap());

        let poisoning_logger = logger.clone();
        let _ = std::thread::spawn(move || {
            let _guard = poisoning_logger.file.lock().unwrap();
            panic!("intentionally poison the audit log mutex");
        })
        .join();

        let result = logger.append(&entry());

        assert!(result.is_err());
    }
}
