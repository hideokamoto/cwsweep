//! `ActionPlanner` コンポーネント。選択されたログループ群からの実行計画構築。
//!
//! project.md Forbidden: 削除条件のルールベース自動判定（人手の選択を経ない自動削除）は実装しない。
//! ここでは選択済みレコード＋人手が選んだアクション種別のみを受け取り、計画を組み立てる。

use serde::{Deserialize, Serialize};

use crate::aggregator::LogGroupRecord;
use crate::error::InvalidRetentionDaysError;

/// CloudWatch Logsの`put-retention-policy`が受け付けるretentionInDaysの許容値一覧。
/// これ以外の値は`ResourceNotFoundException`ではなく`InvalidParameterException`で
/// API呼び出し自体が拒否されるため、計画作成の時点で検証する。
pub const ALLOWED_RETENTION_DAYS: &[i32] = &[
    1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1096, 1827, 2192, 2557, 2922,
    3288, 3653,
];

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
    ///
    /// `action_kind`が`SetRetention`の場合、`days`がCloudWatch Logsの許容する離散値
    /// （[`ALLOWED_RETENTION_DAYS`]）のいずれかであることを検証する。含まれない場合は
    /// 計画を作成せず`Err`を返す。
    pub fn plan(
        selected: &[LogGroupRecord],
        action_kind: ActionKind,
    ) -> Result<Vec<PlannedAction>, InvalidRetentionDaysError> {
        if let ActionKind::SetRetention { days } = action_kind {
            if !ALLOWED_RETENTION_DAYS.contains(&days) {
                return Err(InvalidRetentionDaysError { days });
            }
        }
        Ok(selected
            .iter()
            .map(|r| PlannedAction::from_record(r, action_kind))
            .collect())
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

        let planned = ActionPlanner::plan(&selected, ActionKind::Delete).unwrap();

        assert_eq!(planned.len(), 2);
        assert!(planned.iter().all(|p| p.action_kind == ActionKind::Delete));
    }

    #[test]
    fn plan_with_set_retention_carries_the_requested_days() {
        let selected = vec![record("/a")];

        let planned =
            ActionPlanner::plan(&selected, ActionKind::SetRetention { days: 14 }).unwrap();

        assert_eq!(
            planned[0].action_kind,
            ActionKind::SetRetention { days: 14 }
        );
    }

    #[test]
    fn planned_actions_start_unconfirmed() {
        let selected = vec![record("/a")];

        let planned = ActionPlanner::plan(&selected, ActionKind::Delete).unwrap();

        assert!(!planned[0].confirmed);
    }

    #[test]
    fn empty_selection_produces_empty_plan() {
        let planned = ActionPlanner::plan(&[], ActionKind::Delete).unwrap();
        assert!(planned.is_empty());
    }

    // --- CodeRabbit指摘#5: retentionInDaysの許容値検証 ---

    #[test]
    fn plan_with_set_retention_accepts_every_allowed_value() {
        let selected = vec![record("/a")];

        for &days in ALLOWED_RETENTION_DAYS {
            let planned =
                ActionPlanner::plan(&selected, ActionKind::SetRetention { days }).unwrap();
            assert_eq!(planned[0].action_kind, ActionKind::SetRetention { days });
        }
    }

    #[test]
    fn plan_with_set_retention_rejects_a_disallowed_value() {
        let selected = vec![record("/a")];

        let result = ActionPlanner::plan(&selected, ActionKind::SetRetention { days: 13 });

        let err = result.unwrap_err();
        assert_eq!(err.days, 13);
    }

    #[test]
    fn plan_with_set_retention_rejects_negative_and_zero_boundary_values() {
        let selected = vec![record("/a")];

        assert!(ActionPlanner::plan(&selected, ActionKind::SetRetention { days: 0 }).is_err());
        assert!(ActionPlanner::plan(&selected, ActionKind::SetRetention { days: -1 }).is_err());
    }

    #[test]
    fn plan_with_set_retention_rejects_value_beyond_the_maximum_allowed() {
        let selected = vec![record("/a")];

        let result = ActionPlanner::plan(&selected, ActionKind::SetRetention { days: 3654 });

        assert!(result.is_err());
    }

    #[test]
    fn planned_action_preserves_account_region_and_log_group_name() {
        let selected = vec![record("/aws/lambda/foo")];

        let planned = ActionPlanner::plan(&selected, ActionKind::Delete).unwrap();

        assert_eq!(planned[0].account_id, "111111111111");
        assert_eq!(planned[0].region, "us-east-1");
        assert_eq!(planned[0].log_group_name, "/aws/lambda/foo");
    }
}
