//! `ScanAggregator` コンポーネント。
//!
//! `LogGroupScanner` から渡された確定済みレコードのみを保持・集計する。
//! ページ単位の途中経過は一切保持しない（`LogGroupScanner` が全ページ確定後にのみ渡すため）。

use serde::{Deserialize, Serialize};

/// スキャンで確定した1ロググループのレコード。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogGroupRecord {
    pub account_id: String,
    pub region: String,
    pub log_group_name: String,
    pub stored_bytes: i64,
    pub retention_in_days: Option<i32>,
}

/// 全アカウント×全リージョンのスキャン結果を保持・集約するモデル。
#[derive(Debug, Default)]
pub struct ScanAggregator {
    records: Vec<LogGroupRecord>,
}

impl ScanAggregator {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// 完了済みの（全ページ取得済みの）レコード群を追加する。
    pub fn add_all(&mut self, records: Vec<LogGroupRecord>) {
        self.records.extend(records);
    }

    pub fn records(&self) -> &[LogGroupRecord] {
        &self.records
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// 全レコードの合計バイト数。
    pub fn total_bytes(&self) -> i64 {
        self.records.iter().map(|r| r.stored_bytes).sum()
    }

    /// サイズ（`stored_bytes`）降順にソートしたビューを返す。
    pub fn sorted_by_size_desc(&self) -> Vec<&LogGroupRecord> {
        let mut refs: Vec<&LogGroupRecord> = self.records.iter().collect();
        refs.sort_by(|a, b| b.stored_bytes.cmp(&a.stored_bytes));
        refs
    }

    /// 指定アカウント×リージョンに絞ったレコードを返す。
    pub fn filter_by_account_region(&self, account_id: &str, region: &str) -> Vec<&LogGroupRecord> {
        self.records
            .iter()
            .filter(|r| r.account_id == account_id && r.region == region)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(account_id: &str, region: &str, name: &str, bytes: i64) -> LogGroupRecord {
        LogGroupRecord {
            account_id: account_id.to_string(),
            region: region.to_string(),
            log_group_name: name.to_string(),
            stored_bytes: bytes,
            retention_in_days: None,
        }
    }

    #[test]
    fn new_aggregator_is_empty() {
        let agg = ScanAggregator::new();
        assert!(agg.is_empty());
        assert_eq!(agg.total_bytes(), 0);
    }

    #[test]
    fn add_all_accumulates_records_across_multiple_calls() {
        let mut agg = ScanAggregator::new();
        agg.add_all(vec![record("111111111111", "us-east-1", "/a", 100)]);
        agg.add_all(vec![record("222222222222", "us-west-2", "/b", 200)]);

        assert_eq!(agg.len(), 2);
    }

    #[test]
    fn total_bytes_sums_all_records() {
        let mut agg = ScanAggregator::new();
        agg.add_all(vec![
            record("111111111111", "us-east-1", "/a", 100),
            record("111111111111", "us-east-1", "/b", 250),
        ]);

        assert_eq!(agg.total_bytes(), 350);
    }

    #[test]
    fn sorted_by_size_desc_orders_largest_first() {
        let mut agg = ScanAggregator::new();
        agg.add_all(vec![
            record("111111111111", "us-east-1", "/small", 10),
            record("111111111111", "us-east-1", "/large", 1000),
            record("111111111111", "us-east-1", "/mid", 100),
        ]);

        let sorted = agg.sorted_by_size_desc();
        let names: Vec<&str> = sorted.iter().map(|r| r.log_group_name.as_str()).collect();
        assert_eq!(names, vec!["/large", "/mid", "/small"]);
    }

    #[test]
    fn filter_by_account_region_returns_only_matching_records() {
        let mut agg = ScanAggregator::new();
        agg.add_all(vec![
            record("111111111111", "us-east-1", "/a", 10),
            record("222222222222", "us-east-1", "/b", 20),
            record("111111111111", "us-west-2", "/c", 30),
        ]);

        let filtered = agg.filter_by_account_region("111111111111", "us-east-1");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].log_group_name, "/a");
    }

    #[test]
    fn filter_by_account_region_returns_empty_when_no_match() {
        let mut agg = ScanAggregator::new();
        agg.add_all(vec![record("111111111111", "us-east-1", "/a", 10)]);

        let filtered = agg.filter_by_account_region("999999999999", "eu-west-1");
        assert!(filtered.is_empty());
    }
}
