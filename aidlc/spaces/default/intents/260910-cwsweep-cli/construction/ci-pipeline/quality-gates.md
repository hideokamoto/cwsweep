# Quality Gates — cwsweep

マージ前に必須で通過しなければならないゲート（team.md/project.md Mandated要件、Q3の人間確認通りすべてブロッキング）:

| ゲート | コマンド | 失敗時の挙動 | 出典 |
|---|---|---|---|
| フォーマット | `cargo fmt --check` | ワークフロー失敗、マージ不可 | team.md Code Style |
| Lint | `cargo clippy --all-targets --locked -- -D warnings` | ワークフロー失敗、マージ不可 | team.md Code Style |
| ビルド | `cargo build --locked` | ワークフロー失敗、マージ不可 | team.md Code Style（`--locked`必須） |
| テスト | `cargo test --locked` | ワークフロー失敗、マージ不可 | team.md Testing Posture |
| カバレッジ | `cargo llvm-cov --lib --locked --fail-under-lines 80 --summary-only` | 80%未満でワークフロー失敗 | org.md Testing Posture（`mvp`等スコープの80%floor） |
| 依存脆弱性スキャン | `cargo audit` | 既知脆弱性検出時にワークフロー失敗 | project.md Mandated（NFR2.6） |
| サプライチェーンチェック | `cargo deny check` | advisories/licenses/bans/sources違反でワークフロー失敗 | project.md Mandated（NFR2.7） |

## CI実行から除外される項目

- **破壊的操作パス・安全パスの100%パスカバレッジ**（team.md Testing Posture）: `cargo llvm-cov --fail-under-lines`はモジュール単位の個別floorをCLIオプションで直接指定できないため、本CI設定では全体80%floorのみを機械的ゲート化している。破壊的操作パス・安全パス（execution.rs/audit.rs/identity.rs）の100%目標は、Build and Testステージの`test-results.md`のTarget Verification Matrixで人間がレビューする運用とする（Code Generation承認ゲートでR-01がAccepted riskとして記録済み）。将来的に`cargo llvm-cov`のper-fileしきい値機能または専用スクリプトでの自動チェックへ拡張する余地がある。
- **実AWSアカウントに接触するテスト**: `#[ignore]`属性で分離されており、通常の`cargo test`実行では自動実行しない（team.mdの合意事項）。専用ジョブでの実行は本v1のCI設定には含めない（実AWSアカウントの認証情報をCI環境に持ち込む運用判断が必要なため、スコープ外）。

## リリースゲート（タグ駆動）

team.mdより、M6（`--execute`による実削除・実retention変更）を含むリリースタグには、テスト用サンドボックスAWSアカウントでの動作確認済みチェックリストへのサインオフを要求する、とされている。本v1実装は`--execute`機能を含むため、この要件が適用される。CircleCIの自動化されたゲートとしては実装していない（人間のサインオフを要する手動プロセスのため）。リリースタグ作成前の運用チェックリストとして、`README.md`または別途のリリース手順書に記載することを推奨する（本stageの成果物には含めていない — 運用ドキュメント整備は本ステージの範囲外）。

## Assumptions & Open Questions

None.
