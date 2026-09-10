//! `ActionPlanner` コンポーネント。選択されたログループ群からの実行計画構築。
//!
//! project.md Forbidden: 削除条件のルールベース自動判定（人手の選択を経ない自動削除）は実装しない。
//! ここでは選択済みレコード＋人手が選んだアクション種別のみを受け取り、計画を組み立てる。

use serde::{Deserialize, Serialize};

use crate::aggregator::LogGroupRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionKind {
    Delete,
    SetRetention { days: i32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlannedAction {
    pub account_id: String,
    pub region: String,
    pub log_group_name: String,
    pub action_kind: ActionKind,
    pub confirmed: bool,
}

impl PlannedAction {
    fn from_record(record: &LogGroupRecord, action_kind: ActionKind) -> Self {
        Self {
            account_id: record.account_id.clone(),
            region: record.region.clone(),
            log_group_name: record.log_group_name.clone(),
            action_kind,
            confirmed: false,
        }
    }
}

pub struct ActionPlanner;

impl ActionPlanner {
    /// 選択済みレコード群に対し、単一のアクション種別（人手が選んだもの）を適用した
    /// 実行計画一覧を構築する。`confirmed` は常に `false` で初期化される
    /// （確認は `ConfirmationPresenter` の責務）。
    pub fn plan(selected: &[LogGroupRecord], action_kind: ActionKind) -> Vec<PlannedAction> {
        selected
            .iter()
            .map(|r| PlannedAction::from_record(r, action_kind))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(name: &str) -> LogGroupRecord {
        LogGroupRecord {
            account_id: "111111111111".to_string(),
            region: "us-east-1".to_string(),
            log_group_name: name.to_string(),
            stored_bytes: 10,
            retention_in_days: None,
        }
    }

    #[test]
    fn plan_with_delete_action_produces_one_planned_action_per_record() {
        let selected = vec![record("/a"), record("/b")];

        let planned = ActionPlanner::plan(&selected, ActionKind::Delete);

        assert_eq!(planned.len(), 2);
        assert!(planned.iter().all(|p| p.action_kind == ActionKind::Delete));
    }

    #[test]
    fn plan_with_set_retention_carries_the_requested_days() {
        let selected = vec![record("/a")];

        let planned = ActionPlanner::plan(&selected, ActionKind::SetRetention { days: 14 });

        assert_eq!(
            planned[0].action_kind,
            ActionKind::SetRetention { days: 14 }
        );
    }

    #[test]
    fn planned_actions_start_unconfirmed() {
        let selected = vec![record("/a")];

        let planned = ActionPlanner::plan(&selected, ActionKind::Delete);

        assert!(!planned[0].confirmed);
    }

    #[test]
    fn empty_selection_produces_empty_plan() {
        let planned = ActionPlanner::plan(&[], ActionKind::Delete);
        assert!(planned.is_empty());
    }

    #[test]
    fn planned_action_preserves_account_region_and_log_group_name() {
        let selected = vec![record("/aws/lambda/foo")];

        let planned = ActionPlanner::plan(&selected, ActionKind::Delete);

        assert_eq!(planned[0].account_id, "111111111111");
        assert_eq!(planned[0].region, "us-east-1");
        assert_eq!(planned[0].log_group_name, "/aws/lambda/foo");
    }
}
