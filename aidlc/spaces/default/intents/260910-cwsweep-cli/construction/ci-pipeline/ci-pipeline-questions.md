# CI Pipeline — Questions

## Q1: CIツール

本リポジトリはGitHub（`hideokamoto/cwsweep`）でホストされており、team.md/project.mdはCI必須ゲート（`cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo audit`, `cargo deny check`, テストスイート）をCI必須ゲートとして規定していますが、具体的なCIツール（GitHub Actions等）は明示的に確定していません。

[Answer]: CircleCI

## Q2: ブランチ戦略

team.mdの「Way of Working」で、トランクベース開発（短命なフィーチャーブランチ → `main`へsquash-merge）が既に確定しています。CIはPull Requestと`main`へのプッシュの両方でトリガーする想定でよいですか？

[Answer]: PR + mainプッシュ両方

## Q3: マージ前の必須品質ゲート

team.md/project.mdより、以下が既に確定しています:
- `cargo fmt --check`
- `cargo clippy -- -D warnings`（初期Boltは重大警告のみエラー化、段階的に厳格化。本リリース時点では既に`-D warnings`全面適用済みのためこのまま採用）
- `cargo test --locked`（実AWSアカウントに接触するテストは`#[ignore]`で分離、通常CI実行では自動実行しない）
- `cargo audit`（NFR2.6）
- `cargo deny check`（advisories/licenses/bans/sources、NFR2.7）
- `Cargo.lock`をコミット済み、`--locked`でビルド・テスト

これらすべてをCI必須ゲートとしてよいですか？

[Answer]: はい、すべて必須とする

## Q4: アーティファクトリポジトリ

team.mdの「Deployment」で、バージョンタグ（`v0.1.0`等）push駆動のクロスプラットフォームリリースビルド（Linux/macOS、x86_64/arm64）をGitHub Releasesに添付する方式が既に確定しています。ECR/CodeArtifact/S3等の追加アーティファクトリポジトリは不要という理解でよいですか？

[Answer]: はい、GitHub Releasesのみ

## Assumptions & Open Questions

None.

## Consolidated Summary Confirmation

- CIツール: CircleCI
- トリガー: PR + mainプッシュ両方
- 必須品質ゲート: fmt/clippy/test/coverage(80%)/audit/denyすべてブロッキング
- アーティファクトリポジトリ: GitHub Releasesのみ

- Looks correct
- Request changes

[Answer]: Looks correct
