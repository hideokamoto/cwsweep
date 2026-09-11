//! `InteractiveSelector` コンポーネント（安全パス・TDD: Step 10）。
//!
//! project.md Forbidden: 対話式マルチセレクトの初期状態を「全選択」にしない。
//! 初期状態は常に全チェックOFFとする。

use crate::aggregator::LogGroupRecord;

#[derive(Debug, Clone, PartialEq)]
pub struct SelectableItem {
    pub label: String,
    pub record: LogGroupRecord,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectorError(pub String);

impl std::fmt::Display for SelectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "selector error: {}", self.0)
    }
}
impl std::error::Error for SelectorError {}

/// 実際の対話式マルチセレクトUI（`inquire`等）を抽象化するトレイト。
/// テストでは手書きスタブを注入し、実プロンプトを起動しない。
pub trait MultiSelectPrompt: Send + Sync {
    /// `default_selected_indices` は初期状態でチェック済みにする項目のインデックス。
    /// InteractiveSelectorは常に空配列を渡す。
    fn prompt(
        &self,
        items: &[SelectableItem],
        default_selected_indices: &[usize],
    ) -> Result<Vec<usize>, SelectorError>;
}

pub struct InteractiveSelector<P: MultiSelectPrompt> {
    prompt: P,
}

impl<P: MultiSelectPrompt> InteractiveSelector<P> {
    pub fn new(prompt: P) -> Self {
        Self { prompt }
    }

    /// 初期状態でチェック済みにするインデックス一覧。
    ///
    /// 項目数に関わらず常に空配列を返す（全選択をデフォルトにしない構造的強制）。
    /// 「全選択」操作自体はユーザーが明示的に呼び出す別操作として提供してよいが、
    /// この初期値には一切影響しない。
    pub fn initial_selected_indices(&self, _items_len: usize) -> Vec<usize> {
        Vec::new()
    }

    pub fn select(&self, items: &[SelectableItem]) -> Result<Vec<LogGroupRecord>, SelectorError> {
        let defaults = self.initial_selected_indices(items.len());
        let chosen_indices = self.prompt.prompt(items, &defaults)?;
        Ok(chosen_indices
            .into_iter()
            .filter_map(|i| items.get(i).map(|it| it.record.clone()))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn item(name: &str) -> SelectableItem {
        SelectableItem {
            label: name.to_string(),
            record: LogGroupRecord {
                account_id: "111111111111".to_string(),
                region: "us-east-1".to_string(),
                log_group_name: name.to_string(),
                stored_bytes: 10,
                retention_in_days: None,
            },
        }
    }

    struct RecordingPrompt {
        seen_defaults: Mutex<Option<Vec<usize>>>,
        returns: Vec<usize>,
    }

    impl MultiSelectPrompt for RecordingPrompt {
        fn prompt(
            &self,
            _items: &[SelectableItem],
            default_selected_indices: &[usize],
        ) -> Result<Vec<usize>, SelectorError> {
            *self.seen_defaults.lock().unwrap() = Some(default_selected_indices.to_vec());
            Ok(self.returns.clone())
        }
    }

    // --- Step 10 (TDD, Red→Green): 初期状態が常に全チェックOFFであることの期待仕様 ---

    #[test]
    fn initial_selected_indices_is_empty_for_zero_items() {
        let selector = InteractiveSelector::new(RecordingPrompt {
            seen_defaults: Mutex::new(None),
            returns: vec![],
        });
        assert_eq!(selector.initial_selected_indices(0), Vec::<usize>::new());
    }

    #[test]
    fn initial_selected_indices_is_empty_for_many_items() {
        let selector = InteractiveSelector::new(RecordingPrompt {
            seen_defaults: Mutex::new(None),
            returns: vec![],
        });
        // 項目数がいくつであっても、初期状態は決して「全選択」にならない。
        assert_eq!(selector.initial_selected_indices(1), Vec::<usize>::new());
        assert_eq!(selector.initial_selected_indices(50), Vec::<usize>::new());
    }

    #[test]
    fn select_passes_empty_defaults_to_underlying_prompt() {
        let prompt = RecordingPrompt {
            seen_defaults: Mutex::new(None),
            returns: vec![],
        };
        let items = vec![item("/a"), item("/b"), item("/c")];
        let selector = InteractiveSelector::new(prompt);

        let _ = selector.select(&items).unwrap();

        let seen = selector.prompt.seen_defaults.lock().unwrap().clone();
        assert_eq!(seen, Some(Vec::<usize>::new()));
    }

    #[test]
    fn select_with_no_user_choice_returns_empty_vec() {
        let selector = InteractiveSelector::new(RecordingPrompt {
            seen_defaults: Mutex::new(None),
            returns: vec![],
        });
        let items = vec![item("/a"), item("/b")];

        let selected = selector.select(&items).unwrap();

        assert!(selected.is_empty());
    }

    #[test]
    fn select_returns_records_matching_chosen_indices() {
        let selector = InteractiveSelector::new(RecordingPrompt {
            seen_defaults: Mutex::new(None),
            returns: vec![1],
        });
        let items = vec![item("/a"), item("/b"), item("/c")];

        let selected = selector.select(&items).unwrap();

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].log_group_name, "/b");
    }

    #[test]
    fn selector_error_display_includes_underlying_message() {
        let err = SelectorError("user cancelled".to_string());
        assert_eq!(err.to_string(), "selector error: user cancelled");
    }

    #[test]
    fn underlying_prompt_error_propagates() {
        struct FailingPrompt;
        impl MultiSelectPrompt for FailingPrompt {
            fn prompt(
                &self,
                _items: &[SelectableItem],
                _defaults: &[usize],
            ) -> Result<Vec<usize>, SelectorError> {
                Err(SelectorError("user cancelled".to_string()))
            }
        }
        let selector = InteractiveSelector::new(FailingPrompt);
        let items = vec![item("/a")];

        let result = selector.select(&items);

        assert!(result.is_err());
    }
}
