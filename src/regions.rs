//! `--regions` の解決ロジック。
//!
//! - `--regions all`（大文字小文字を区別しない）: 商用パーティション（`aws`）の全リージョンを
//!   `ec2:describe-regions` で列挙して対象にする。TTYでは対象数を表示して確認を取る。
//! - `--regions` 未指定: 標準入力がTTYなら全リージョン列挙結果から対話式に選択する。
//!   非TTY（CI/パイプ）では明確なエラーで終了する。
//!
//! AWS呼び出しと対話UIはトレイト経由で注入し、単体テストでは実プロンプト・実AWS接続なしに
//! 分岐を検証できるようにする。

use async_trait::async_trait;
use thiserror::Error;

/// `--regions` 引数の解釈結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegionSpec {
    /// 明示的にリージョンが指定された。
    Explicit(Vec<String>),
    /// `all` が指定された（他の値と混在していても `all` を優先する）。
    All,
    /// `--regions` が指定されなかった。
    Unspecified,
}

pub fn region_spec(regions: &[String]) -> RegionSpec {
    if regions.is_empty() {
        return RegionSpec::Unspecified;
    }
    if regions.iter().any(|r| r.eq_ignore_ascii_case("all")) {
        return RegionSpec::All;
    }
    RegionSpec::Explicit(regions.to_vec())
}

/// v1スコープは商用パーティション（`aws`）のみ。GovCloud（`us-gov-*`）・中国（`cn-*`）・
/// その他の分離パーティション（`us-iso*`, `eu-isoe-*`）は列挙対象から除外する。
pub fn is_commercial_region(name: &str) -> bool {
    !(name.starts_with("us-gov-")
        || name.starts_with("cn-")
        || name.starts_with("us-iso")
        || name.starts_with("eu-isoe-"))
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("describe-regions call failed: {message}")]
pub struct ListRegionsError {
    pub message: String,
}

/// `ec2:describe-regions` を抽象化するトレイト。実装は商用パーティションのリージョン名を
/// ソート済みで返す。
#[async_trait]
pub trait ListRegionsOperations: Send + Sync {
    async fn list_commercial_regions(&self) -> Result<Vec<String>, ListRegionsError>;
}

/// リージョン解決に伴う対話UIを抽象化するトレイト。
pub trait RegionPrompt {
    /// `all` 指定時の最終確認。`false` を返すと処理を中止する。
    fn confirm_all(&self, regions: &[String]) -> bool;
    /// 未指定時の対話式マルチセレクト。選択されたリージョン名を返す。
    fn select(&self, regions: &[String]) -> Result<Vec<String>, String>;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RegionResolveError {
    #[error(
        "--regions が指定されていません。CI/パイプ等の非対話環境では `--regions <region>[,<region>...]` または `--regions all` を明示してください。"
    )]
    MissingInNonInteractive,
    #[error("describe-regions で列挙されたリージョンが0件でした")]
    NoRegionsListed,
    #[error("リージョンが1件も選択されなかったため、処理を中止します")]
    NothingSelected,
    #[error("全リージョンを対象とする実行が確認されなかったため、処理を中止します")]
    AllNotConfirmed,
    #[error(transparent)]
    List(#[from] ListRegionsError),
    #[error("リージョン選択に失敗しました: {0}")]
    Prompt(String),
}

/// `RegionSpec` を実際のスキャン対象リージョン一覧へ解決する。
///
/// `Explicit` はAWSにも対話UIにも触れずそのまま返す。`Unspecified` かつ非TTYは、
/// `describe-regions` を呼ぶ前にエラーで終了する。
pub async fn resolve_regions(
    spec: RegionSpec,
    lister: &dyn ListRegionsOperations,
    stdin_is_tty: bool,
    prompt: &dyn RegionPrompt,
) -> Result<Vec<String>, RegionResolveError> {
    match spec {
        RegionSpec::Explicit(regions) => Ok(regions),
        RegionSpec::All => {
            let regions = listed_regions(lister).await?;
            eprintln!(
                "--regions all が指定されたため、商用リージョン {} 件を対象にします: {}",
                regions.len(),
                regions.join(", ")
            );
            if stdin_is_tty && !prompt.confirm_all(&regions) {
                return Err(RegionResolveError::AllNotConfirmed);
            }
            Ok(regions)
        }
        RegionSpec::Unspecified => {
            if !stdin_is_tty {
                return Err(RegionResolveError::MissingInNonInteractive);
            }
            let regions = listed_regions(lister).await?;
            let selected = prompt
                .select(&regions)
                .map_err(RegionResolveError::Prompt)?;
            if selected.is_empty() {
                return Err(RegionResolveError::NothingSelected);
            }
            Ok(selected)
        }
    }
}

async fn listed_regions(
    lister: &dyn ListRegionsOperations,
) -> Result<Vec<String>, RegionResolveError> {
    let mut regions: Vec<String> = lister
        .list_commercial_regions()
        .await?
        .into_iter()
        .filter(|r| is_commercial_region(r))
        .collect();
    regions.sort();
    regions.dedup();
    if regions.is_empty() {
        return Err(RegionResolveError::NoRegionsListed);
    }
    Ok(regions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    struct StubLister {
        regions: Vec<String>,
        calls: AtomicUsize,
    }

    impl StubLister {
        fn new(regions: &[&str]) -> Self {
            Self {
                regions: strings(regions),
                calls: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl ListRegionsOperations for StubLister {
        async fn list_commercial_regions(&self) -> Result<Vec<String>, ListRegionsError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.regions.clone())
        }
    }

    struct StubPrompt {
        confirm: bool,
        selection: Vec<String>,
        confirm_calls: AtomicUsize,
        select_calls: AtomicUsize,
        offered: Mutex<Vec<String>>,
    }

    impl StubPrompt {
        fn new(confirm: bool, selection: &[&str]) -> Self {
            Self {
                confirm,
                selection: strings(selection),
                confirm_calls: AtomicUsize::new(0),
                select_calls: AtomicUsize::new(0),
                offered: Mutex::new(Vec::new()),
            }
        }
    }

    impl RegionPrompt for StubPrompt {
        fn confirm_all(&self, _regions: &[String]) -> bool {
            self.confirm_calls.fetch_add(1, Ordering::SeqCst);
            self.confirm
        }
        fn select(&self, regions: &[String]) -> Result<Vec<String>, String> {
            self.select_calls.fetch_add(1, Ordering::SeqCst);
            *self.offered.lock().unwrap() = regions.to_vec();
            Ok(self.selection.clone())
        }
    }

    #[test]
    fn region_spec_classifies_empty_all_and_explicit() {
        assert_eq!(region_spec(&[]), RegionSpec::Unspecified);
        assert_eq!(region_spec(&strings(&["all"])), RegionSpec::All);
        assert_eq!(region_spec(&strings(&["ALL"])), RegionSpec::All);
        assert_eq!(
            region_spec(&strings(&["us-east-1", "All"])),
            RegionSpec::All
        );
        assert_eq!(
            region_spec(&strings(&["us-east-1", "ap-northeast-1"])),
            RegionSpec::Explicit(strings(&["us-east-1", "ap-northeast-1"]))
        );
    }

    #[test]
    fn non_commercial_partitions_are_excluded() {
        assert!(is_commercial_region("us-east-1"));
        assert!(is_commercial_region("ap-northeast-1"));
        assert!(is_commercial_region("il-central-1"));
        assert!(!is_commercial_region("us-gov-west-1"));
        assert!(!is_commercial_region("cn-north-1"));
        assert!(!is_commercial_region("us-iso-east-1"));
        assert!(!is_commercial_region("us-isob-east-1"));
        assert!(!is_commercial_region("eu-isoe-west-1"));
    }

    #[tokio::test]
    async fn explicit_regions_bypass_listing_and_prompts() {
        let lister = StubLister::new(&["us-east-1"]);
        let prompt = StubPrompt::new(false, &[]);
        let resolved = resolve_regions(
            RegionSpec::Explicit(strings(&["ap-northeast-1"])),
            &lister,
            false,
            &prompt,
        )
        .await
        .unwrap();
        assert_eq!(resolved, strings(&["ap-northeast-1"]));
        assert_eq!(lister.calls.load(Ordering::SeqCst), 0);
        assert_eq!(prompt.confirm_calls.load(Ordering::SeqCst), 0);
        assert_eq!(prompt.select_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn unspecified_regions_in_non_tty_fail_without_calling_aws() {
        let lister = StubLister::new(&["us-east-1"]);
        let prompt = StubPrompt::new(true, &["us-east-1"]);
        let err = resolve_regions(RegionSpec::Unspecified, &lister, false, &prompt)
            .await
            .unwrap_err();
        assert_eq!(err, RegionResolveError::MissingInNonInteractive);
        assert!(err.to_string().contains("--regions"));
        assert_eq!(lister.calls.load(Ordering::SeqCst), 0);
        assert_eq!(prompt.select_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn unspecified_regions_in_tty_offer_sorted_commercial_regions_for_selection() {
        let lister = StubLister::new(&["us-west-2", "us-gov-west-1", "us-east-1", "cn-north-1"]);
        let prompt = StubPrompt::new(true, &["us-east-1"]);
        let resolved = resolve_regions(RegionSpec::Unspecified, &lister, true, &prompt)
            .await
            .unwrap();
        assert_eq!(resolved, strings(&["us-east-1"]));
        assert_eq!(
            *prompt.offered.lock().unwrap(),
            strings(&["us-east-1", "us-west-2"])
        );
        assert_eq!(prompt.confirm_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn unspecified_regions_with_empty_selection_is_an_error() {
        let lister = StubLister::new(&["us-east-1"]);
        let prompt = StubPrompt::new(true, &[]);
        let err = resolve_regions(RegionSpec::Unspecified, &lister, true, &prompt)
            .await
            .unwrap_err();
        assert_eq!(err, RegionResolveError::NothingSelected);
    }

    #[tokio::test]
    async fn all_in_tty_requires_confirmation() {
        let lister = StubLister::new(&["us-west-2", "us-east-1"]);
        let declined = StubPrompt::new(false, &[]);
        let err = resolve_regions(RegionSpec::All, &lister, true, &declined)
            .await
            .unwrap_err();
        assert_eq!(err, RegionResolveError::AllNotConfirmed);

        let accepted = StubPrompt::new(true, &[]);
        let resolved = resolve_regions(RegionSpec::All, &lister, true, &accepted)
            .await
            .unwrap();
        assert_eq!(resolved, strings(&["us-east-1", "us-west-2"]));
        assert_eq!(accepted.select_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn all_in_non_tty_skips_confirmation_and_filters_non_commercial() {
        let lister = StubLister::new(&["us-gov-east-1", "eu-west-1", "us-east-1"]);
        let prompt = StubPrompt::new(false, &[]);
        let resolved = resolve_regions(RegionSpec::All, &lister, false, &prompt)
            .await
            .unwrap();
        assert_eq!(resolved, strings(&["eu-west-1", "us-east-1"]));
        assert_eq!(prompt.confirm_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn empty_listing_is_an_error() {
        let lister = StubLister::new(&[]);
        let prompt = StubPrompt::new(true, &[]);
        let err = resolve_regions(RegionSpec::All, &lister, false, &prompt)
            .await
            .unwrap_err();
        assert_eq!(err, RegionResolveError::NoRegionsListed);
    }
}
