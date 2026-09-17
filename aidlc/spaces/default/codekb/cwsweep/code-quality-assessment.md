# code-quality-assessment.md — cwsweep

## テストカバレッジ

- **テストディレクトリ**: `tests/`（統合テスト2ファイル: `scan_select_execute.rs`,
  `audit_log_format.rs`）。加えて全12ソースモジュールがファイル内に
  `#[cfg(test)] mod tests` を持つ（ユニットテスト）。
- **テストフレームワーク**: 標準の `#[test]` / `#[tokio::test]`（`tokio` の
  `macros` 機能経由）。モックは全て手書きスタブ/フェイク（外部モックライブラリ
  不使用、各コンポーネント境界 trait を直接実装）。
- **カバレッジ設定**: 存在する。CI の `coverage` ジョブが
  `cargo llvm-cov --lib --locked --fail-under-lines 80 --summary-only` を実行
  （`--lib` のみ、統合テストはカバレッジ計測対象外）。README.md にもローカル
  実行コマンドの記載あり（`cargo llvm-cov --lib --summary-only`、要
  `rustup component add llvm-tools-preview`）。
- 実 AWS アカウントに接触するテストは本リポジトリには存在しない
  （全てモック境界のみ）。`#[ignore]` 分離は将来の実アカウント接触テスト追加時の
  方針であり、現状のテストコード自体は全てモック完結。

## コード品質指標

- **Linting**: `clippy`（`.circleci/config.yml` の `clippy` ジョブで
  `cargo clippy --all-targets --locked -- -D warnings`）。フォーマットは
  `fmt` ジョブで `cargo fmt --check`。`src/main.rs`・`src/lib.rs` 双方に
  `#![forbid(unsafe_code)]` と
  `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`
  （`lib.rs` 側は `#[cfg_attr(not(test), deny(...))]` でテストコードのみ除外）。
- **CI/CD**: `.circleci/config.yml`（CircleCI 2.1）。ジョブ: `fmt`, `clippy`,
  `test`, `coverage`, `audit`（`cargo audit`、週次のみ）,
  `deny`（`cargo deny check`、通常は `bans licenses sources`、週次に
  `advisories` 追加）, `release-linux` / `release-macos`（タグ駆動、
  `v\d+\.\d+\.\d+` パターンでクロスプラットフォームバイナリビルド）,
  `publish-github-release`。ワークフロー: `build-and-test`（push/PR時）,
  `weekly-security`（スケジュール実行時のみ）, `release`（タグpush時のみ）。
- **ドキュメント**: README.md（日本語、使い方・スコープ制約・開発コマンド・
  AI-DLC説明を包括）。各ソースファイル冒頭にモジュールdocコメント（`//!`）で
  コンポーネントの責務と Mandated/Forbidden 規則の参照が付記されている。
  関数レベルのdocコメントも要所（特に安全パス関連: identity検証・監査ログ・
  実行エンジン）に充実。

## 技術的負債シグナル

1. **CLI のフラット構造（本 intent の直接対象）**: `--scan-only`/`--execute`
   の相互排他フラグのみで、本 intent が目指すサブコマンド化（`scan`/`clean`/
   `audit`）は未着手。`Cli` 構造体・`main.rs` のフロー分岐（`cli.scan_only` →
   `stdin().is_terminal()` 判定 → 選択/確認/実行）が今回の再構成の直接の対象。
2. **`action_kind_from_prompt`（`main.rs`）が未テスト**: delete/set-retention
   選択（`inquire::Select`/`inquire::CustomType`）は `inquire` への直接依存の
   ためユニットテスト対象外。既存の設計判断としては妥当だが、将来のサブコマンド化
   でCLI引数から直接 `ActionKind` を指定する経路を設ける場合はテスト可能な形に
   切り出す余地がある。
3. **監査ログの読み取り API 不在**: `audit-log-path` に無効化オプションを設けない
   設計（project.md Mandated）のため、`audit` サブコマンドを新設する場合は
   既存の `AuditLogger`/`AuditWrite` をそのまま読み取り専用ビューアとして
   再利用できるか（現状は追記専用APIのみで読み取りAPIが存在しない）が設計上の
   空白点。

それ以外に深刻な技術的負債の兆候（TODO残置、デッドコード、重複ロジック等）は
確認されなかった。エラーハンドリングは `thiserror` ベースで一貫し、
`unwrap`/`expect`/`panic` は本番コードパスから `#![deny(...)]` で構造的に
排除されている。

## セキュリティ・運用上の留意点

- 一時クレデンシャル（`AccessKeyId`/`SecretAccessKey`/`SessionToken`）は
  `secrecy` によりマスキングされ、ログ・標準出力・エラーメッセージに出力されない
  設計（project.md Forbidden）。
- `unsafe` コードはクレートルートで `#![forbid(unsafe_code)]` により全面禁止。
- 破壊的操作（`delete-log-group`/`put-retention-policy`）は dry-run 既定＋
  二重 Identity 検証＋監査ログ必須という多層防御。

## 本 intent への含意

上記1（CLI フラット構造）が唯一の設計変更対象であり、2・3は将来の関連
サブコマンド（特に `audit` 閲覧）を検討する際の既知の空白点として
`architecture.md`「改善の余地」および `component-inventory.md`「AuditLogger」に
併記済み。回帰リスクは README.md 記載の終了コード契約とdry-run既定の維持。
