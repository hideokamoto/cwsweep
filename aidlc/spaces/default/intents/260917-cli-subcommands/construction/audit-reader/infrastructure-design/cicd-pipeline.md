# cicd-pipeline.md — Unit: audit-reader

`audit-reader`は既存の`cwsweep`単一Cargoパッケージ内の`lib`クレート
モジュールであり、独立したデプロイ成果物を持たない。したがって、新規の
CircleCIジョブ・新規パイプラインステージは不要であり、既存パイプライン
（`technology-stack.md`参照）をそのまま再利用する。

## 既存パイプラインでのカバー状況

| ステージ | 既存ジョブ | audit-readerの扱い |
|---|---|---|
| フォーマット検証 | `fmt` | `src/audit.rs`内の新規コードも対象（クレート全体を走査） |
| 静的解析 | `clippy` | 同上。`#[deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`が本番コードパスへ適用される（team.md Code Style） |
| テスト実行 | `test` | 単体テスト（不正行スキップ・境界値・読み取り専用性の検証等）・NFR2構造的回帰テストをこのジョブ内で実行する。新規テストファイル追加のみで、ジョブ定義の変更は不要 |
| カバレッジ計測 | `coverage`（`cargo llvm-cov --lib --fail-under-lines 80`） | `audit-reader`のコードは`lib`クレート側に実装するため、既存の80%行カバレッジ floor 計測対象にそのまま含まれる |
| 依存脆弱性スキャン | `audit`（週次） | 新規クレートを追加しないため、スキャン対象・頻度に変更なし |
| ライセンス/サプライチェーン | `deny` | 同上 |
| リリース | `release-linux` / `release-macos` / `publish-github-release`（タグ駆動） | `audit-reader`は既存バイナリの一部としてビルドされるのみで、リリース手順・タグ運用に変更はない |

## デプロイ戦略・ロールバック・環境昇格

該当なし。`audit-reader`は独立したデプロイ対象ではないため、
Blue-Green/Canary/Rolling等のデプロイ戦略、環境昇格（dev/staging/prod）、
ロールバック手順は本Unit固有には存在しない。既存の「タグ駆動リリース、
`main`へのマージ自体はリリースをトリガーしない」という方針
（team.md Deployment）がそのまま適用される。

## シークレット管理

該当なし。`audit-reader`はクレデンシャルを一切扱わないため
（`CredentialProvider`の管轄外）、CI/CDにおける新規シークレット・
環境変数の追加は不要。

## 既知のギャップ（Build and Testステージへの申し送り）

team.mdが要求する「破壊的操作関連コードパス等での100%パスカバレッジ＋
境界値テスト」という基準は、`AuditReader`の異常系分岐（不正行スキップ・
ファイル不在・I/Oエラー）にも適用される（`security-requirements.md`
NFR3.2）。しかし既存の`coverage`ジョブは`cargo llvm-cov --lib
--fail-under-lines 80`という**単一のグローバル閾値**のみを機械的に
強制しており、特定モジュールに対する100%floorを自動検証する仕組みを
持たない。この検証手段（例: モジュール限定のカバレッジレポートを
PRレビュー時に手動確認する、または`cargo llvm-cov`のパス/ファイル
フィルタ機能を使った専用チェックを追加する）の具体化は、本ステージの
スコープ外としてBuild and Testステージ（3.6）で設計する。
