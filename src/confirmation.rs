//! `ConfirmationPresenter` コンポーネント。実行前確認画面。
//!
//! Domain Design: `ConfirmationPresenter`は`ActionPlanner`所有の`PlannedAction`を
//! 直接ミューテートせず、確認結果を反映した確認済みコピーを生成して`ExecutionEngine`へ渡す。
//! 元の`PlannedAction`（`ActionPlanner`所有）は`confirmed`未設定のまま不変とする。

use crate::planner::{ActionKind, PlannedAction};

/// 確認対象1件分。`account_id`/`region`/`log_group_name`/`action_kind`の対応関係を
/// 崩さずに保持する（CodeRabbit指摘#3: `ConfirmationSummary`が`account_ids`/
/// `regions`/`log_group_names`を独立配列として保持していると、異なるアカウント/
/// リージョンに同名のロググループがある場合にどのレコードがどの対象かを一意に
/// 識別できない）。
#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmationTarget {
    pub account_id: String,
    pub region: String,
    pub log_group_name: String,
    pub action_kind: ActionKind,
}

pub struct ConfirmationSummary {
    /// 確認対象ごとの一覧。`account_id`/`region`/`log_group_name`/`action_kind`の
    /// 対応関係を保ったまま、`actions`と同じ順序・同じ件数で保持する
    /// （重複除去やソートは行わない）。
    pub targets: Vec<ConfirmationTarget>,
    /// 表示用に重複除去したアカウントID一覧（対応関係の特定には`targets`を使うこと）。
    pub account_ids: Vec<String>,
    /// 表示用に重複除去したリージョン一覧（対応関係の特定には`targets`を使うこと）。
    pub regions: Vec<String>,
    /// 表示用のログループ名一覧（`actions`と同じ順序。対応関係の特定には`targets`を使うこと）。
    pub log_group_names: Vec<String>,
    pub total_bytes: i64,
}

/// 最終確認の取得を抽象化するトレイト。テストでは固定応答のスタブを注入する。
pub trait ConfirmPrompt: Send + Sync {
    fn confirm(&self, summary: &ConfirmationSummary) -> bool;
}

pub struct ConfirmationPresenter<P: ConfirmPrompt> {
    prompt: P,
}

impl<P: ConfirmPrompt> ConfirmationPresenter<P> {
    pub fn new(prompt: P) -> Self {
        Self { prompt }
    }

    /// 対象アカウントID・リージョン・ログループ名・アクション種別一覧・合計バイト数を
    /// 再掲したサマリを構築する。`targets`は`actions`との対応関係（どのレコードが
    /// どのアカウント×リージョン×ログループ×アクション種別か）を1件も失わない。
    pub fn build_summary(actions: &[PlannedAction], total_bytes: i64) -> ConfirmationSummary {
        let targets: Vec<ConfirmationTarget> = actions
            .iter()
            .map(|a| ConfirmationTarget {
                account_id: a.account_id.clone(),
                region: a.region.clone(),
                log_group_name: a.log_group_name.clone(),
                action_kind: a.action_kind,
            })
            .collect();

        let mut account_ids: Vec<String> = actions.iter().map(|a| a.account_id.clone()).collect();
        account_ids.sort();
        account_ids.dedup();

        let mut regions: Vec<String> = actions.iter().map(|a| a.region.clone()).collect();
        regions.sort();
        regions.dedup();

        let log_group_names: Vec<String> =
            actions.iter().map(|a| a.log_group_name.clone()).collect();

        ConfirmationSummary {
            targets,
            account_ids,
            regions,
            log_group_names,
            total_bytes,
        }
    }

    /// 確認を取り、確認済みなら元の計画から`confirmed=true`の新しいコピーを返す。
    /// 確認が得られなかった場合は`None`を返す（`ExecutionEngine`は一切呼ばれない）。
    ///
    /// 引数の`actions`（`ActionPlanner`所有）は不変のまま。戻り値は独立したコピー。
    pub fn confirm(
        &self,
        actions: &[PlannedAction],
        total_bytes: i64,
    ) -> Option<Vec<PlannedAction>> {
        let summary = Self::build_summary(actions, total_bytes);
        if self.prompt.confirm(&summary) {
            Some(
                actions
                    .iter()
                    .cloned()
                    .map(|mut a| {
                        a.confirmed = true;
                        a
                    })
                    .collect(),
            )
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::ActionKind;

    fn action(name: &str) -> PlannedAction {
        PlannedAction {
            account_id: "111111111111".to_string(),
            region: "us-east-1".to_string(),
            log_group_name: name.to_string(),
            action_kind: ActionKind::Delete,
            confirmed: false,
        }
    }

    struct FixedPrompt(bool);
    impl ConfirmPrompt for FixedPrompt {
        fn confirm(&self, _summary: &ConfirmationSummary) -> bool {
            self.0
        }
    }

    #[test]
    fn build_summary_deduplicates_account_ids_and_regions() {
        let actions = vec![action("/a"), action("/b")];

        let summary = ConfirmationPresenter::<FixedPrompt>::build_summary(&actions, 300);

        assert_eq!(summary.account_ids, vec!["111111111111"]);
        assert_eq!(summary.regions, vec!["us-east-1"]);
        assert_eq!(summary.log_group_names, vec!["/a", "/b"]);
        assert_eq!(summary.total_bytes, 300);
    }

    fn action_in(account_id: &str, region: &str, name: &str, kind: ActionKind) -> PlannedAction {
        PlannedAction {
            account_id: account_id.to_string(),
            region: region.to_string(),
            log_group_name: name.to_string(),
            action_kind: kind,
            confirmed: false,
        }
    }

    // --- CodeRabbit指摘#3: 対象の対応関係（アカウント/リージョン/ログループ/アクション種別）---

    #[test]
    fn build_summary_targets_preserve_one_to_one_correspondence_with_actions() {
        let actions = vec![
            action_in("111111111111", "us-east-1", "/a", ActionKind::Delete),
            action_in(
                "222222222222",
                "us-west-2",
                "/a",
                ActionKind::SetRetention { days: 30 },
            ),
        ];

        let summary = ConfirmationPresenter::<FixedPrompt>::build_summary(&actions, 100);

        assert_eq!(summary.targets.len(), 2);
        assert_eq!(summary.targets[0].account_id, "111111111111");
        assert_eq!(summary.targets[0].region, "us-east-1");
        assert_eq!(summary.targets[0].log_group_name, "/a");
        assert_eq!(summary.targets[0].action_kind, ActionKind::Delete);
        assert_eq!(summary.targets[1].account_id, "222222222222");
        assert_eq!(summary.targets[1].region, "us-west-2");
        assert_eq!(summary.targets[1].log_group_name, "/a");
        assert_eq!(
            summary.targets[1].action_kind,
            ActionKind::SetRetention { days: 30 }
        );
    }

    #[test]
    fn build_summary_targets_distinguish_same_log_group_name_in_different_accounts_and_regions() {
        // 異なるアカウント/リージョンに同名のロググループがある場合でも、
        // `targets`経由であればどのレコードがどの対象かを一意に識別できる。
        let actions = vec![
            action_in(
                "111111111111",
                "us-east-1",
                "/aws/lambda/shared",
                ActionKind::Delete,
            ),
            action_in(
                "222222222222",
                "us-east-1",
                "/aws/lambda/shared",
                ActionKind::Delete,
            ),
            action_in(
                "111111111111",
                "us-west-2",
                "/aws/lambda/shared",
                ActionKind::Delete,
            ),
        ];

        let summary = ConfirmationPresenter::<FixedPrompt>::build_summary(&actions, 0);

        let distinct: std::collections::HashSet<(String, String, String)> = summary
            .targets
            .iter()
            .map(|t| {
                (
                    t.account_id.clone(),
                    t.region.clone(),
                    t.log_group_name.clone(),
                )
            })
            .collect();
        assert_eq!(
            distinct.len(),
            3,
            "3件はすべて別々の対象として区別できるべき"
        );
    }

    #[test]
    fn confirm_true_returns_confirmed_copies_without_mutating_original() {
        let original = vec![action("/a")];
        let presenter = ConfirmationPresenter::new(FixedPrompt(true));

        let confirmed = presenter.confirm(&original, 100).unwrap();

        assert!(confirmed[0].confirmed);
        // 元のオリジナルは不変（confirmed未設定のまま）。
        assert!(!original[0].confirmed);
    }

    #[test]
    fn confirm_false_returns_none_and_execution_engine_must_not_proceed() {
        let original = vec![action("/a")];
        let presenter = ConfirmationPresenter::new(FixedPrompt(false));

        let result = presenter.confirm(&original, 100);

        assert!(result.is_none());
    }

    #[test]
    fn confirm_with_empty_plan_still_calls_prompt_and_returns_empty_confirmed_vec() {
        let presenter = ConfirmationPresenter::new(FixedPrompt(true));

        let confirmed = presenter.confirm(&[], 0).unwrap();

        assert!(confirmed.is_empty());
    }

    #[test]
    fn confirmed_copy_preserves_action_kind_and_target_identifiers() {
        let original = vec![action("/aws/lambda/foo")];
        let presenter = ConfirmationPresenter::new(FixedPrompt(true));

        let confirmed = presenter.confirm(&original, 50).unwrap();

        assert_eq!(confirmed[0].log_group_name, "/aws/lambda/foo");
        assert_eq!(confirmed[0].action_kind, ActionKind::Delete);
    }
}
