//! cwsweep: AWS Organization全体のCloudWatch Logs棚卸し・削除CLI ライブラリクレート。
//!
//! Domain Design (`components.md`) の12コンポーネントを1モジュール1コンポーネントとして
//! 対応させる。本番コードパスでは `unwrap()` / `expect()` / `panic!` を使用しない
//! （テストモジュールは対象外とする）。

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod aggregator;
pub mod audit;
pub mod cli;
pub mod confirmation;
pub mod credentials;
pub mod error;
pub mod execution;
pub mod identity;
pub mod org_discovery;
pub mod output;
pub mod planner;
pub mod scanner;
pub mod selector;
