# CI Configuration — cwsweep

## CIツール

CircleCI（人間の選択、Q1）。設定ファイルはワークスペースルートの`.circleci/config.yml`。

`aidlc engine`のCircleCI MCPサーバー（`validate_config`）で実際にコンパイル検証済み（`valid: true`）。

## トリガー

- `build-and-test`ワークフロー: 全ブランチ・全PRで実行（CircleCIはデフォルトで全プッシュに反応するため、PR・mainプッシュの両方をカバーする — Q2の人間確認通り）。
- `release`ワークフロー: `v\d+\.\d+\.\d+`形式のタグpushのみでトリガー（`branches: ignore: /.*/`でブランチプッシュでは発火しない）。team.mdの「mainへのマージ自体はリリースをトリガーしない」方針と整合。

## ジョブ構成（build-and-testワークフロー）

| ジョブ | 内容 | ブロッキング |
|---|---|---|
| `fmt` | `cargo fmt --check` | Yes |
| `clippy` | `cargo clippy --all-targets --locked -- -D warnings` | Yes |
| `test` | `cargo build --locked` → `cargo test --locked`（`#[ignore]`の実AWS接触テストは通常実行から自動除外） | Yes |
| `coverage` | `cargo llvm-cov --lib --locked --fail-under-lines 80 --summary-only`（80%行カバレッジfloor未達で失敗） | Yes |
| `audit` | `cargo audit`（NFR2.6） | Yes |
| `deny` | `cargo deny check`（advisories/licenses/bans/sources、NFR2.7） | Yes |

全ジョブが必須ゲート（Q3の人間確認通り、advisory化なし）。

## リリースジョブ（releaseワークフロー、タグ駆動）

team.mdの「クロスプラットフォーム（Linux/macOS、x86_64/arm64）のリリースバイナリをビルドし、GitHub Releasesに添付する」方針に基づき、4ターゲットをビルドする:

| ジョブ | ターゲット | Executor |
|---|---|---|
| `release-linux-x86_64` | `x86_64-unknown-linux-gnu` | `cimg/rust:1.82.0`（Docker, medium） |
| `release-linux-arm64` | `aarch64-unknown-linux-gnu` | `cimg/rust:1.82.0`（Docker, arm.medium） |
| `release-macos-x86_64` | `x86_64-apple-darwin` | macOS（xcode 15.3.0, macos.m1.medium.gen1、クロスコンパイル） |
| `release-macos-arm64` | `aarch64-apple-darwin` | macOS（xcode 15.3.0, macos.m1.medium.gen1、ネイティブ） |

各ジョブが`artifacts/cwsweep-<tag>-<target>.tar.gz`を`persist_to_workspace`し、`publish-github-release`ジョブが4アーティファクトすべてを`attach_workspace`してGitHub Releasesへアップロードする（`requires`で全リリースビルドの完了を待つ）。

**未実装の詳細（既知の制限）**: `publish-github-release`ジョブの実際の`gh release upload`コマンドはコメントアウトのプレースホルダーである。GitHub Releasesへの実アップロードには、CircleCIプロジェクト設定でGitHub Personal Access Token（`GH_TOKEN`環境変数、ログ出力禁止）を登録する運用作業が別途必要であり、これは本stageの範囲外（インフラ・シークレット管理の運用手順）である。

## アーティファクトリポジトリ

GitHub Releasesのみ（Q4の人間確認通り）。ECR/CodeArtifact/S3は使用しない。

## キャッシュ戦略

`cargo-registry-{{ checksum "Cargo.lock" }}`をキーとしたキャッシュで`~/.cargo/registry`と`target`を再利用し、ビルド時間を短縮する。

## Assumptions & Open Questions

None.
