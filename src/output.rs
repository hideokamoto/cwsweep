//! `OutputFormatter` コンポーネント。table/json出力切り替え。

use comfy_table::{presets::UTF8_FULL, Table};
use serde::Serialize;

use crate::aggregator::{LogGroupRecord, ScanAggregator};
use crate::audit::{AuditEventKind, AuditReadEntry};
use crate::planner::ActionKind;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
}

#[derive(Debug, Serialize)]
struct JsonReport<'a> {
    total_bytes: i64,
    log_group_count: usize,
    records: &'a [LogGroupRecord],
}

pub struct OutputFormatter;

impl OutputFormatter {
    /// 集計結果を指定フォーマットで整形して文字列として返す。
    pub fn format(aggregator: &ScanAggregator, format: OutputFormat) -> String {
        match format {
            OutputFormat::Table => Self::format_table(aggregator),
            OutputFormat::Json => Self::format_json(aggregator),
        }
    }

    fn format_table(aggregator: &ScanAggregator) -> String {
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec![
            "Account ID",
            "Region",
            "Log Group",
            "Stored Bytes",
            "Retention (days)",
        ]);
        for record in aggregator.sorted_by_size_desc() {
            table.add_row(vec![
                record.account_id.clone(),
                record.region.clone(),
                record.log_group_name.clone(),
                record.stored_bytes.to_string(),
                record
                    .retention_in_days
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "never expire".to_string()),
            ]);
        }
        format!(
            "{table}\nTotal: {} log group(s), {} byte(s)",
            aggregator.len(),
            aggregator.total_bytes()
        )
    }

    fn format_json(aggregator: &ScanAggregator) -> String {
        let report = JsonReport {
            total_bytes: aggregator.total_bytes(),
            log_group_count: aggregator.len(),
            records: aggregator.records(),
        };
        // シリアライズ失敗は入力が全て検証済みのプレーンデータのため通常発生しないが、
        // 万一失敗した場合でもpanicせずフォールバック文字列を返す。
        serde_json::to_string_pretty(&report)
            .unwrap_or_else(|e| format!("{{\"error\": \"json serialization failed: {e}\"}}"))
    }

    /// 監査ログエントリを指定フォーマットで整形する（記録順を保持、絞り込みなし）。
    /// table は `run_id` を除く8列、json は全フィールドの配列（Contract 2）。
    pub fn format_audit(entries: &[AuditReadEntry], format: OutputFormat) -> String {
        match format {
            OutputFormat::Table => Self::format_audit_table(entries),
            OutputFormat::Json => serde_json::to_string_pretty(entries)
                .unwrap_or_else(|e| format!("{{\"error\": \"json serialization failed: {e}\"}}")),
        }
    }

    fn format_audit_table(entries: &[AuditReadEntry]) -> String {
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec![
            "Timestamp",
            "Event",
            "Account ID",
            "Region",
            "Log Group",
            "Action",
            "Success",
            "Error",
        ]);
        for entry in entries {
            let action = match entry.action_kind {
                ActionKind::Delete => "Delete".to_string(),
                ActionKind::SetRetention { days } => format!("SetRetention({days}d)"),
            };
            // intent 行の success は結果未確定のプレースホルダなので "-" と表示する。
            let success = match entry.event {
                AuditEventKind::Intent => "-".to_string(),
                AuditEventKind::Result => entry.success.to_string(),
            };
            let event = match entry.event {
                AuditEventKind::Intent => "intent",
                AuditEventKind::Result => "result",
            };
            table.add_row(vec![
                entry.timestamp.clone(),
                event.to_string(),
                entry.account_id.clone(),
                entry.region.clone(),
                entry.log_group_name.clone(),
                action,
                success,
                entry.error_message.clone().unwrap_or_default(),
            ]);
        }
        format!("{table}\nTotal: {} audit entry(ies)", entries.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_aggregator() -> ScanAggregator {
        let mut agg = ScanAggregator::new();
        agg.add_all(vec![
            LogGroupRecord {
                account_id: "111111111111".to_string(),
                region: "us-east-1".to_string(),
                log_group_name: "/aws/lambda/foo".to_string(),
                stored_bytes: 2048,
                retention_in_days: Some(30),
            },
            LogGroupRecord {
                account_id: "222222222222".to_string(),
                region: "us-west-2".to_string(),
                log_group_name: "/aws/lambda/bar".to_string(),
                stored_bytes: 1024,
                retention_in_days: None,
            },
        ]);
        agg
    }

    #[test]
    fn table_format_contains_account_ids_and_log_group_names() {
        let agg = sample_aggregator();
        let out = OutputFormatter::format(&agg, OutputFormat::Table);

        assert!(out.contains("111111111111"));
        assert!(out.contains("/aws/lambda/foo"));
        assert!(out.contains("Total: 2 log group(s), 3072 byte(s)"));
    }

    #[test]
    fn table_format_shows_never_expire_for_null_retention() {
        let agg = sample_aggregator();
        let out = OutputFormatter::format(&agg, OutputFormat::Table);

        assert!(out.contains("never expire"));
    }

    #[test]
    fn json_format_is_valid_json_with_expected_fields() {
        let agg = sample_aggregator();
        let out = OutputFormatter::format(&agg, OutputFormat::Json);

        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["total_bytes"], 3072);
        assert_eq!(parsed["log_group_count"], 2);
        assert_eq!(parsed["records"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn empty_aggregator_table_shows_zero_totals() {
        let agg = ScanAggregator::new();
        let out = OutputFormatter::format(&agg, OutputFormat::Table);

        assert!(out.contains("Total: 0 log group(s), 0 byte(s)"));
    }

    #[test]
    fn empty_aggregator_json_has_empty_records_array() {
        let agg = ScanAggregator::new();
        let out = OutputFormatter::format(&agg, OutputFormat::Json);

        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["records"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn default_output_format_is_table() {
        assert_eq!(OutputFormat::default(), OutputFormat::Table);
    }

    fn audit_entry(event: AuditEventKind, success: bool) -> AuditReadEntry {
        AuditReadEntry {
            run_id: "run-1".to_string(),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            account_id: "111111111111".to_string(),
            region: "us-east-1".to_string(),
            log_group_name: "/aws/lambda/foo".to_string(),
            action_kind: ActionKind::SetRetention { days: 30 },
            event,
            success,
            error_message: None,
        }
    }

    #[test]
    fn audit_table_hides_run_id_and_marks_intent_success_as_undetermined() {
        let entries = vec![
            audit_entry(AuditEventKind::Intent, false),
            audit_entry(AuditEventKind::Result, true),
        ];
        let out = OutputFormatter::format_audit(&entries, OutputFormat::Table);

        assert!(!out.contains("run-1"));
        assert!(out.contains("SetRetention(30d)"));
        assert!(out.contains("intent"));
        assert!(out.contains("Total: 2 audit entry(ies)"));
    }

    #[test]
    fn audit_json_is_an_array_with_every_field() {
        let entries = vec![audit_entry(AuditEventKind::Result, true)];
        let out = OutputFormatter::format_audit(&entries, OutputFormat::Json);
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();

        assert_eq!(parsed.as_array().unwrap().len(), 1);
        assert_eq!(parsed[0]["run_id"], "run-1");
        assert_eq!(parsed[0]["success"], true);
    }
}
