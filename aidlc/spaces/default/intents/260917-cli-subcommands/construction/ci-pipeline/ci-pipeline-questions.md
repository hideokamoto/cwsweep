# CI Pipeline Questions — 260917-cli-subcommands

Brownfield intent。CI は既存の `.circleci/config.yml`（intent 260910-cwsweep-cli で人間が確定）を再利用するため、
以下の質問はワークスペース検出結果と team.md の確定事項で自動解決した（人間への新規質問なし。承認は代理判断）。

## Q1. CIツール
- 検出: CircleCI（`.circleci/config.yml`）。GitHub Actions は不使用。
- 決定: 変更なし。

## Q2. ブランチ戦略
- 検出: 全ブランチ・全PRで `build-and-test` ワークフロー、`v\d+\.\d+\.\d+` タグで `release`。
- 決定: 変更なし。`Cargo.toml` 0.2.0 に合わせ、リリース時は `v0.2.0` タグを打つ（本intentではタグを打たない）。

## Q3. マージ前の品質ゲート
- 検出: fmt / clippy / test / coverage(`--lib`, 80% floor) / audit / deny、すべてブロッキング。
- 決定: 変更なし。team.md Q3 の確定事項（ディスパッチを lib 側に置き `--lib` 計測に含める）は cli-foundation で実装済み（cli.rs 95.41%）。

## Q4. アーティファクトリポジトリ
- 検出: GitHub Releases のみ。
- 決定: 変更なし。
