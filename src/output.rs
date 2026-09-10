//! `OutputFormatter` コンポーネント。table/json出力切り替え。

use comfy_table::{presets::UTF8_FULL, Table};
use serde::Serialize;

use crate::aggregator::{LogGroupRecord, ScanAggregator};

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
}
