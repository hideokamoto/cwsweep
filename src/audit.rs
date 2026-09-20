//! `AuditLogger` コンポーネント（安全パス・TDD: Step 15）。
//!
//! project.md Mandated: 削除・retention変更を伴うすべての操作について、対象アカウントID・
//! リージョン・ログループ名・実行時刻・成功/失敗を監査ログに出力する。無効化オプションは設けない。
//! project.md Mandated: 監査ログへの書き込みに失敗した場合、実行中の操作自体を中断し、エラーとして扱う。

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::error::AuditWriteError;
use crate::planner::ActionKind;

/// 監査ログの1行が「実行意図」を表すのか「結果」を表すのかを区別する。
///
/// CodeRabbit指摘#8: 削除成功後に監査ログ追記が失敗すると、不可逆操作の記録が
/// どこにも残らない問題への対応。`ExecutionEngine`はAPI呼び出しの「前」に
/// `Intent`エントリを追記し（追記が失敗した場合はAPIを呼び出さずに中断する）、
/// API呼び出しの「後」に`Result`エントリを追記する（成功/失敗いずれも記録する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventKind {
    Intent,
    Result,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEntry {
    pub run_id: String,
    pub timestamp: String,
    pub account_id: String,
    pub region: String,
    pub log_group_name: String,
    pub action_kind: ActionKind,
    /// このエントリが実行意図(`Intent`)か結果(`Result`)かを示す。
    pub event: AuditEventKind,
    /// `event == Intent`の場合は結果未確定のため`false`のプレースホルダ値
    /// （`event`を見ずにこのフィールド単体を成功/失敗判定に使ってはならない）。
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

/// 監査ログの書き込み先。ファイルは最初の`append`で初めて作成する（スキャンのみ・
/// 選択なしで終了した実行が空ファイルを残さないようにするため）。
enum AuditSink {
    Pending(PathBuf),
    Open(Box<dyn SyncWrite>),
}

/// JSON Linesを1エントリずつ`fsync`付きで追記する監査ログ書き込みコンポーネント。
/// 無効化するオプションは意図的に存在しない。
pub struct AuditLogger {
    file: Mutex<AuditSink>,
}

impl AuditWrite for AuditLogger {
    fn append(&self, entry: &AuditEntry) -> Result<(), AuditWriteError> {
        AuditLogger::append(self, entry)
    }
}

impl AuditLogger {
    /// 出力先を検証し、ロガーを作成する。ファイル自体はまだ作成せず、最初の`append`で
    /// 作成する。親ディレクトリが存在しない等、明らかに書き込めない場合はここで失敗する。
    pub fn open(path: &Path) -> Result<Self, AuditWriteError> {
        let parent = match path.parent() {
            Some(p) if !p.as_os_str().is_empty() => p,
            _ => Path::new("."),
        };
        if !parent.is_dir() {
            return Err(AuditWriteError {
                message: format!(
                    "failed to open audit log file {}: parent directory {} does not exist",
                    path.display(),
                    parent.display()
                ),
            });
        }
        if path.exists() && !path.is_file() {
            return Err(AuditWriteError {
                message: format!(
                    "failed to open audit log file {}: not a regular file",
                    path.display()
                ),
            });
        }
        Ok(Self {
            file: Mutex::new(AuditSink::Pending(path.to_path_buf())),
        })
    }

    #[cfg(test)]
    fn from_writer(writer: Box<dyn SyncWrite>) -> Self {
        Self {
            file: Mutex::new(AuditSink::Open(writer)),
        }
    }

    fn open_file(path: &Path) -> Result<Box<dyn SyncWrite>, AuditWriteError> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| AuditWriteError {
                message: format!("failed to open audit log file {}: {e}", path.display()),
            })?;
        Ok(Box::new(file))
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

        let mut sink = self.file.lock().map_err(|_| AuditWriteError {
            message: "audit log file mutex poisoned".to_string(),
        })?;

        if let AuditSink::Pending(path) = &*sink {
            *sink = AuditSink::Open(Self::open_file(path)?);
        }
        let AuditSink::Open(file) = &mut *sink else {
            return Err(AuditWriteError {
                message: "audit log sink is not open".to_string(),
            });
        };

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

// `audit-reader` Unit（読み取り専用）。以下のマーカーに囲まれた型・トレイト・
// 実装は、本ファイル冒頭で定義した書き込み側（`AuditWrite`/`AuditLogger`）
// とは型レベルで分離された独立した領域である（domain-design Q2: 同一ファイル・
// 型レベル分離方針）。
//
// project.md Forbidden: `audit`（監査ログ閲覧）サブコマンドのハンドラに、
// `delete-log-group`/`put-retention-policy`を実行しうる型（`ExecutionEngine`、
// および書き込み系の`AuditWrite`等）への依存を一切持たせない。マーカーで
// 囲んだ領域の下のコードは`ExecutionEngine`・`AuditWrite`・`AuditLogger`への
// `use`文・型参照を一切含まない（この宣言自体は識別子を挙げて説明するため
// 意図的にマーカーの外側に置く。構造的検証は本モジュール末尾の
// `audit_reader_module_has_no_execution_or_write_dependency` を参照）。
// === AuditRead (read-only) region begin ===

/// 監査ログ（JSON Lines）読み取り専用ビューにおける1行分のエントリ
/// (entities.md `AuditEntry`)。
///
/// 本ファイル冒頭で定義される書き込み側のエントリ型とはフィールド構成が
/// 同一（書き込み側が実際に出力する9フィールド構成をそのまま踏襲する）
/// だが、型としては意図的に分離する。読み取り側の実装が書き込み側の型を
/// `use`する経路自体を作らないための設計上の境界であり
/// （security-design.md NFR2.1/NFR2.2）、型を共有すると、その型を経由した
/// 間接的な依存関係が生まれうる。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditReadEntry {
    pub run_id: String,
    pub timestamp: String,
    pub account_id: String,
    pub region: String,
    pub log_group_name: String,
    pub action_kind: ActionKind,
    /// このエントリが実行意図(`Intent`)か結果(`Result`)かを示す。`audit`
    /// サブコマンドはこの値で絞り込みを行わず、両方をそのまま表示する
    /// (rules.md BR5.1)。
    pub event: AuditEventKind,
    /// `event == Intent`の場合は結果未確定のプレースホルダ値であり、
    /// 失敗を意味しない（呼び出し元は`event`と併せて解釈する）。
    pub success: bool,
    pub error_message: Option<String>,
}

/// パース不能、または必須フィールドを1つ以上欠くために `AuditReadEntry` と
/// して解釈できなかった監査ログの1行 (rules.md BR1.1/BR1.2)。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkippedLine {
    /// 監査ログファイル内の行番号（1始まり）。
    pub line_number: usize,
    /// スキップ理由（JSONパース失敗／必須フィールド欠落）を表す人間可読な文。
    pub reason: String,
}

/// `AuditRead::entries()` 1回分の読み取り結果全体 (entities.md
/// `AuditReadOutcome`)。対象ファイルが存在しない場合も、`entries`・
/// `skipped_lines`とも空の正常なインスタンスとして返る (rules.md BR2.1)。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditReadOutcome {
    /// 監査ログファイルに記録された順序をそのまま保持する (rules.md BR3.2)。
    pub entries: Vec<AuditReadEntry>,
    /// ファイル中の出現順を保持する (rules.md BR3.2)。
    pub skipped_lines: Vec<SkippedLine>,
}

/// `AuditRead::entries()` の失敗系。
///
/// `std::io::Error`は`Clone`/`Eq`を導出できないため、`src/error.rs`の他の
/// エラー型と同様に直接ラップせず、メッセージ文字列を保持する`Io(String)`
/// として表現する（advisory reviewのR-02指摘への対応）。
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AuditReadError {
    /// ファイル不在以外のI/Oエラー（権限不足、読み取り失敗等）
    /// (rules.md BR2.2)。ファイル不在自体はエラーではない (BR2.1)。
    #[error("failed to read audit log file: {0}")]
    Io(String),
}

/// 監査ログ（JSON Lines）読み取り専用境界の抽象化 (rules.md BR4.2)。
///
/// 実装は削除・retention変更を実行しうる型、および監査ログ書き込み系の型
/// （いずれも本ファイル冒頭で定義される）への依存を一切持たない。このシグ
/// ネチャ自体が`&self`のみを要求し、書き込み・削除系の値への参照を要求し
/// ないことが、この非依存性の主たる保証である（多層防御の一部としての
/// 静的テストは本モジュール末尾を参照）。
pub trait AuditRead {
    fn entries(&self) -> Result<AuditReadOutcome, AuditReadError>;
}

/// 監査ログファイルを行単位でストリーミング読み取りする `AuditRead` 実装。
///
/// 読み取り専用でファイルを開き（`File::open`、書き込み・作成フラグ
/// `OpenOptions::write`/`create`/`append`は一切使用しない）、いかなる
/// 書き込みも行わない (rules.md BR4.1、NFR3.3)。
pub struct AuditReader {
    path: PathBuf,
}

impl AuditReader {
    /// 読み取り対象の監査ログファイルパスを指定してリーダーを構築する。
    /// この時点ではファイルへアクセスしない（`entries()`呼び出し時に初めて
    /// 開く）。
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl AuditRead for AuditReader {
    fn entries(&self) -> Result<AuditReadOutcome, AuditReadError> {
        let file = match File::open(&self.path) {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // BR2.1: ファイル不在はエラーとせず、空の正常結果として扱う。
                return Ok(AuditReadOutcome {
                    entries: Vec::new(),
                    skipped_lines: Vec::new(),
                });
            }
            Err(e) => {
                return Err(AuditReadError::Io(format!(
                    "failed to open audit log file {}: {e}",
                    self.path.display()
                )));
            }
        };

        let reader = BufReader::new(file);
        let mut outcome = AuditReadOutcome {
            entries: Vec::new(),
            skipped_lines: Vec::new(),
        };

        for (index, line_result) in reader.lines().enumerate() {
            let line_number = index + 1;
            // ファイル不在以外のI/Oエラー（読み取り自体の失敗）は行単位の
            // スキップ処理には進まず、直ちに呼び出し元へ伝播させる (BR2.2)。
            let line = line_result.map_err(|e| {
                AuditReadError::Io(format!(
                    "failed to read audit log file {} at line {}: {e}",
                    self.path.display(),
                    line_number
                ))
            })?;

            match serde_json::from_str::<AuditReadEntry>(&line) {
                Ok(entry) => outcome.entries.push(entry),
                Err(e) => {
                    // BR1.1/BR1.2: パース失敗または必須フィールド欠落は
                    // SkippedLineとして記録し、警告を出力した上で処理を継続
                    // する（処理全体は中断しない）。
                    let reason = e.to_string();
                    eprintln!("warning: skipped malformed audit log line {line_number}: {reason}");
                    outcome.skipped_lines.push(SkippedLine {
                        line_number,
                        reason,
                    });
                }
            }
        }

        Ok(outcome)
    }
}

// === AuditRead (read-only) region end ===

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
            event: AuditEventKind::Result,
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
    fn open_does_not_create_file_until_first_append() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");

        let logger = AuditLogger::open(&path).unwrap();
        assert!(!path.exists());

        logger.append(&entry()).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn open_fails_when_path_is_a_directory() {
        let dir = tempfile::tempdir().unwrap();

        let result = AuditLogger::open(dir.path());

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

    // --- CodeRabbit指摘#8: intent/result二段階記録のためのevent種別のテスト ---

    #[test]
    fn intent_event_serializes_with_lowercase_snake_case_tag() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");
        let logger = AuditLogger::open(&path).unwrap();
        let mut intent = entry();
        intent.event = AuditEventKind::Intent;
        intent.success = false;
        intent.error_message = None;

        logger.append(&intent).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let value: serde_json::Value = serde_json::from_str(contents.trim()).unwrap();
        assert_eq!(value["event"], "intent");
    }

    #[test]
    fn result_event_serializes_with_lowercase_snake_case_tag() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");
        let logger = AuditLogger::open(&path).unwrap();

        logger.append(&entry()).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let value: serde_json::Value = serde_json::from_str(contents.trim()).unwrap();
        assert_eq!(value["event"], "result");
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

/// `audit-reader` Unit（読み取り専用）のテスト。既存の`mod tests`
/// （`AuditLogger`書き込み側）とは意図的に別モジュールとする
/// （code-generation-plan.md Step 1）。
#[cfg(test)]
mod audit_reader_tests {
    use super::*;

    fn write_lines(path: &Path, lines: &[&str]) {
        let mut file = File::create(path).expect("failed to create fixture audit log file");
        for line in lines {
            writeln!(file, "{line}").expect("failed to write fixture audit log line");
        }
    }

    fn sample_entry() -> AuditReadEntry {
        AuditReadEntry {
            run_id: "a1b2c3d4-0000-0000-0000-000000000000".to_string(),
            timestamp: "2026-09-10T12:00:00Z".to_string(),
            account_id: "111111111111".to_string(),
            region: "us-east-1".to_string(),
            log_group_name: "/aws/lambda/foo".to_string(),
            action_kind: ActionKind::Delete,
            event: AuditEventKind::Result,
            success: true,
            error_message: None,
        }
    }

    // --- Step 2: データモデル層 (entities.md準拠) ---

    #[test]
    fn audit_read_entry_roundtrips_through_json() {
        let entry = sample_entry();

        let json = serde_json::to_string(&entry).expect("serialize should succeed");
        let parsed: AuditReadEntry =
            serde_json::from_str(&json).expect("deserialize should succeed");

        assert_eq!(parsed, entry);
    }

    #[test]
    fn audit_read_entry_deserialization_fails_when_required_field_missing() {
        // `success`フィールド（必須8フィールドの1つ）を欠いたJSON。
        let json = r#"{
            "run_id": "r1",
            "timestamp": "2026-09-10T12:00:00Z",
            "account_id": "111111111111",
            "region": "us-east-1",
            "log_group_name": "/aws/lambda/foo",
            "action_kind": "Delete",
            "event": "result"
        }"#;

        let result: Result<AuditReadEntry, _> = serde_json::from_str(json);

        assert!(result.is_err());
    }

    // --- Step 3: ビジネスロジック層 (rules.md準拠) ---

    #[test]
    fn entries_returns_empty_outcome_when_file_does_not_exist() {
        // BR2.1
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let path = dir.path().join("does-not-exist.jsonl");
        let reader = AuditReader::new(path);

        let outcome = reader.entries().expect("missing file must not be an error");

        assert!(outcome.entries.is_empty());
        assert!(outcome.skipped_lines.is_empty());
    }

    #[test]
    fn entries_preserves_order_across_multiple_valid_lines() {
        // BR3.2
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let path = dir.path().join("audit.jsonl");
        let mut first = sample_entry();
        first.log_group_name = "/a".to_string();
        let mut second = sample_entry();
        second.log_group_name = "/b".to_string();
        let mut third = sample_entry();
        third.log_group_name = "/c".to_string();
        write_lines(
            &path,
            &[
                &serde_json::to_string(&first).expect("serialize should succeed"),
                &serde_json::to_string(&second).expect("serialize should succeed"),
                &serde_json::to_string(&third).expect("serialize should succeed"),
            ],
        );
        let reader = AuditReader::new(path);

        let outcome = reader.entries().expect("read should succeed");

        assert_eq!(outcome.entries.len(), 3);
        assert_eq!(outcome.entries[0].log_group_name, "/a");
        assert_eq!(outcome.entries[1].log_group_name, "/b");
        assert_eq!(outcome.entries[2].log_group_name, "/c");
        assert!(outcome.skipped_lines.is_empty());
    }

    #[test]
    fn entries_skips_malformed_line_and_continues_with_warning() {
        // team.md Q7 / NFR3.1: (a)不正行はスキップされ警告出力、
        // (b)不正行の前後の正常行は引き続きentriesに含まれる。
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let path = dir.path().join("audit.jsonl");
        let mut before = sample_entry();
        before.log_group_name = "/before".to_string();
        let mut after = sample_entry();
        after.log_group_name = "/after".to_string();
        write_lines(
            &path,
            &[
                &serde_json::to_string(&before).expect("serialize should succeed"),
                "{ this is not valid json",
                &serde_json::to_string(&after).expect("serialize should succeed"),
            ],
        );
        let reader = AuditReader::new(path);

        let outcome = reader
            .entries()
            .expect("read should succeed despite bad line");

        // (a) 不正行はSkippedLineとして記録される（警告は標準エラーへ出力
        // 済みであり、ここではskipped_linesへの記録を検証する）。
        assert_eq!(outcome.skipped_lines.len(), 1);
        assert_eq!(outcome.skipped_lines[0].line_number, 2);
        // (b) 不正行の前後の正常なエントリは引き続きentriesに含まれる。
        assert_eq!(outcome.entries.len(), 2);
        assert_eq!(outcome.entries[0].log_group_name, "/before");
        assert_eq!(outcome.entries[1].log_group_name, "/after");
    }

    #[test]
    fn entries_returns_io_error_when_path_is_not_a_regular_file() {
        // BR2.2: パスがディレクトリの場合、ファイル不在ではないI/Oエラー
        // として扱われる（読み取り時にEISDIR相当のエラーが発生する）。
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let reader = AuditReader::new(dir.path().to_path_buf());

        let result = reader.entries();

        assert!(matches!(result, Err(AuditReadError::Io(_))));
    }

    #[test]
    fn entries_never_creates_or_writes_the_target_file() {
        // BR4.1/NFR3.3: ファイル不在時に新規ファイルを作成しない
        // （読み取り専用であり、いかなる書き込みも行わない）。
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let path = dir.path().join("does-not-exist.jsonl");
        let reader = AuditReader::new(path.clone());

        let outcome = reader.entries().expect("missing file must not be an error");

        assert!(outcome.entries.is_empty());
        assert!(
            !path.exists(),
            "AuditReader must not create the audit log file as a side effect"
        );
    }

    // --- Step 4: 構造的分離の回帰テスト (NFR2.1/NFR2.2、security-design.md準拠) ---

    /// `AuditReader`/`AuditRead`関連コードのソーステキストに`ExecutionEngine`・
    /// `AuditWrite`・`AuditLogger`の識別子が一切出現しないことを検証する
    /// テキストベースの静的回帰テスト。
    ///
    /// advisory review（nfr-design R-01）の指摘どおり、この検査はエイリアス
    /// や間接参照までは検出できない**多層防御の一部**であり、単独で
    /// 「コンパイル時点で到達不能」を証明するものではない。主たる保証は
    /// `AuditRead::entries(&self)`のシグネチャ自体が`ExecutionEngine`/
    /// `AuditWrite`への参照を要求しない設計であること、および型を共有しない
    /// 設計であることにある。
    #[test]
    fn audit_reader_module_has_no_execution_or_write_dependency() {
        let source_path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/audit.rs");
        let source = std::fs::read_to_string(source_path).expect("src/audit.rs must be readable");

        const BEGIN_MARKER: &str = "// === AuditRead (read-only) region begin ===";
        const END_MARKER: &str = "// === AuditRead (read-only) region end ===";
        let start = source
            .find(BEGIN_MARKER)
            .expect("AuditRead region begin marker not found in src/audit.rs");
        let end = source
            .find(END_MARKER)
            .expect("AuditRead region end marker not found in src/audit.rs");
        assert!(start < end, "AuditRead region markers are out of order");
        let region = &source[start..end];

        for forbidden in ["ExecutionEngine", "AuditWrite", "AuditLogger"] {
            assert!(
                !region.contains(forbidden),
                "AuditRead read-only region must not reference forbidden identifier: {forbidden}"
            );
        }
    }
}
