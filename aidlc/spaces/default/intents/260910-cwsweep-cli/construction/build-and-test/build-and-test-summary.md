# Build and Test Summary — cwsweep

## ビルド状況・前提条件

- Rust stable + `rustfmt`/`clippy`コンポーネントがインストール済みであること。
- `cargo build --locked`が成功し、`Cargo.lock`と`Cargo.toml`の整合性が取れていること。
- 実AWSアカウントへの接触は本stageでは一切行わない（全テストはモック/トレイト差し替えで完結）。

## 生成したテスト種別一覧

- `build-instructions.md`（必須）
- `integration-test-instructions.md`（Standard戦略の必須項目。スキャン→選択→削除の統合シナリオ、監査ログフォーマット検証）
- `security-test-instructions.md`（Standard戦略では必須ではないが、破壊的操作・クレデンシャル取り扱いを伴うプロジェクト特性を踏まえたソフトガイドライン適用）
- `performance-test-instructions.md`は生成していない: NFR1.1（スキャン完了時間）はperformance-requirements.mdの「Benchmarks」節で「初期リリースでは自動ベンチマークは必須要件としない」と明記されており、統合シナリオテストでの動作確認で足りると判断した。

## Unit毎のカバレッジ期待値

zero-Unit（stage-level）のため単一の集計。`cargo llvm-cov --lib --summary-only`実測:

| モジュール | 行カバレッジ | 80%floor | 100%floor対象（破壊的操作/安全パス） |
|---|---|---|---|
| aggregator.rs | 100.00% | 達成 | — |
| audit.rs | 90.77% | 達成 | 対象・未達（Accepted risk、下記参照） |
| cli.rs | 97.46% | 達成 | — |
| confirmation.rs | 100.00% | 達成 | — |
| credentials.rs | 100.00% | 達成 | — |
| error.rs | 95.24% | 達成 | — |
| execution.rs | 93.66% | 達成 | 対象・未達（Accepted risk、下記参照） |
| identity.rs | 99.22% | 達成 | 対象・未達（Accepted risk、下記参照） |
| org_discovery.rs | 100.00% | 達成 | — |
| output.rs | 98.91% | 達成 | — |
| planner.rs | 100.00% | 達成 | — |
| scanner.rs | 99.38% | 達成 | — |
| selector.rs | 100.00% | 達成（NFR2.9初期状態全OFFの安全パス、100%達成） | 対象・達成 |
| **TOTAL** | **97.19%** | **達成** | — |

**破壊的操作・安全パス100%floor未達（execution.rs 93.66%, audit.rs 90.77%, identity.rs 99.22%）は新規欠陥ではない。** Code Generationステージのadvisoryレビュー指摘R-01として既に記録され、人間が明示的に「今回は未対応のまま残す（Accepted risk）」と判断済み（`aidlc/spaces/default/intents/260910-cwsweep-cli/.aidlc-reviews/code-generation/stage/df820a4cc21bd395/1.review.md`、`.../b7cf606c56d6783d/1.review.md`）。本stageではこの既知ギャップをTarget Verification Matrixに転記するのみで、追加の是正は行わない。

## Target Verification Matrix

`test-results.md`に最終版を記載（本ファイルでは要約のみ）。全対象目標は`Met`、破壊的操作パスの100%カバレッジ1件のみ既知の`Accepted risk`（Not Metとして正直に記録、後述の理由により本stageの合格判定を妨げない）。詳細は`test-results.md`参照。

## Readiness Assessment

- **Build-ready**: Yes（`cargo build --locked`成功）
- **Test-ready**: Yes（97 unit + 4 integration すべてpass、`cargo fmt --check`/`cargo clippy --all-targets -- -D warnings`合格）
- **Deployment-ready**: Partial — `cargo audit`/`cargo deny check`はCI Pipelineステージの所有物として未実行（ツール未インストール）。team.md/project.mdのMandated要件によりCI必須ゲートとして次stageで導入する。

## 既知の制限・積み残し事項

1. **R-01（Accepted risk）**: 破壊的操作パス・安全パスの100%カバレッジがexecution.rs/audit.rs/identity.rsで未達（到達不能分岐のみ）。人間が明示的に受容済み。
2. **`cargo audit`/`cargo deny check`未実行**: CI Pipelineステージで導入・実行する（NFR2.6, NFR2.7）。
3. **NFR1.1（スキャン完了時間の実測）**: 実Org環境での実測はまだ行われていない。performance-requirements.mdの方針通り、初期リリースでは統合シナリオテスト（モック）による動作確認で足りるとされているため、本stageでは`N/A`（対象外の設計判断）として扱う。

## Assumptions & Open Questions

None.
