# Test Results — cwsweep

## ビルド状況

`cargo build --locked`: **成功**。警告0件。

## テスト結果（Loop-back 1後の再実行）

| 種別 | 総数 | Pass | Fail | Skip |
|---|---|---|---|---|
| Unit（`cargo test --lib`） | 99 | 99 | 0 | 0 |
| 統合（`cargo test --test scan_select_execute --test audit_log_format`） | 4 | 4 | 0 | 0 |
| Doc-tests | 0 | 0 | 0 | 0 |
| **合計** | **103** | **103** | **0** | **0** |

失敗テストなし。

## 静的解析・フォーマット

- `cargo fmt --check`: 差分なし（合格）
- `cargo clippy --all-targets -- -D warnings`: 警告0件（合格）
- `grep forbid(unsafe_code)`: `src/main.rs:7`, `src/lib.rs:7`に確認
- `grep deny(clippy::unwrap_used...)`: `src/main.rs:8`, `src/lib.rs:10`に確認

## カバレッジレポート（`cargo llvm-cov --lib --summary-only`、Loop-back 1後）

全体行カバレッジ **97.47%**（execution.rs追加テストにより97.19%→97.47%に改善）。モジュール別は`build-and-test-summary.md`参照。

## Target Verification Matrix

| Target ID | Source | Expected | Actual | Evidence | Owning Stage | Verdict |
|---|---|---|---|---|---|---|
| Testing Contract: 80%行カバレッジfloor | code-generation-plan.md 埋め込みTesting Contract | 全モジュール80%以上 | 全13モジュールとも80%以上（最低90.77%、全体97.19%） | `cargo llvm-cov --lib --summary-only`出力（本ファイル上部） | build-and-test | Met |
| Testing Contract: 破壊的操作・安全パス100%floor | code-generation-plan.md 埋め込みTesting Contract | execution.rs/audit.rs/identity.rs/selector.rs 100% | selector.rs 100%達成。Loop-back 1でexecution.rs 93.66%→95.68%（LCOV実行回数0行は0件に到達）、identity.rs 99.22%→99.26%（同0件）に改善。audit.rs 90.77%は変化なし（LCOV実行回数0行17件、構造的に到達不能と検証済み）。残るcargo llvm-cov summary上の未達分（execution.rs/identity.rs）はassert!/matches!マクロ展開由来の測定上のアーティファクトとreview記録で検証済み | `cargo llvm-cov --lib --summary-only`/`--lcov`出力、`.aidlc-reviews/code-generation/stage/30639fec9d25f391/1.review.md`（R-01, 再検証・Accepted risk） | code-generation（Loop-back 1で再検証済み、人間がAccepted riskとして再承認） | Not Met（Accepted risk — Loop-back 1で追加検証を実施済み。execution.rs/identity.rsはLCOV基準で実質到達済み、audit.rsの残存17行は構造的に到達不能と確認の上、人間が明示的に受容） |
| NFR1.1 | performance-requirements.md | 標準規模で実用上2分以内（設計目標、ハードSLA外） | 自動ベンチマーク未実施。要件文書自体が初期リリースでの自動ベンチマークを必須としないと明記 | performance-requirements.md「Benchmarks」節 | 対象外（設計判断） | N/A |
| NFR1.2 | performance-requirements.md | ページネーションは逐次呼び出し | `scanner::tests::multi_page_scan_walks_every_page_before_returning`で全ページ逐次取得を検証 | `cargo test --lib scanner::` | build-and-test | Met |
| NFR1.3 | performance-requirements.md | 数百件規模をメモリ上に全件保持 | `ScanAggregator`は`Vec<LogGroupRecord>`で全件保持する設計。単体テストで複数件の集計を確認 | `cargo test --lib aggregator::` | build-and-test | Met |
| NFR1.4 | performance-requirements.md | aws-config標準リトライ（最大3回）、使い切ったら当該アカウントのみ失敗 | `Cargo.toml`で`aws-config`既定のRetryMode::Standardを利用（追加設定不要）。個別の自動テストは実施せず、コード上の設定確認に留まる | `Cargo.toml`依存定義、コードレビュー | build-and-test | Met |
| NFR2.1 | security-requirements.md | クレデンシャル非ログ出力（`secrecy`マスキング） | `credentials::tests`で`Debug`出力に実値が含まれず`[REDACTED]`のみであることを検証 | `cargo test --lib credentials::` | build-and-test | Met |
| NFR2.2 | security-requirements.md | 認証・認可をAWS IAMに完全委譲 | 本ツール独自の認証機構は実装していない（設計確認） | components.md, コードレビュー | build-and-test | Met（設計確認） |
| NFR2.3 | security-requirements.md | Identity検証必須、不一致時は即時失敗 | `identity::tests`で不一致ID・境界値（リージョン跨ぎ等）を含め検証 | `cargo test --lib identity::` | build-and-test | Met |
| NFR2.4 | security-requirements.md | 書き込みAPIは`--execute`明示時のみ | `execution::tests`と`tests/scan_select_execute.rs`のdry-runシナリオで検証 | `cargo test --lib execution::`, `cargo test --test scan_select_execute` | build-and-test | Met |
| NFR2.5 | security-requirements.md | 削除直前の二重Identity検証 | `execution::tests`で削除直前の独立した再検証呼び出しを検証 | `cargo test --lib execution::` | build-and-test | Met |
| NFR2.6 | security-requirements.md | `cargo audit`をCI必須ゲート | 本stageでは`cargo-audit`未インストールのため未実行 | — | ci-pipeline | Unverified（次stageで所有・実行） |
| NFR2.7 | security-requirements.md | `cargo deny check`をCI必須ゲート | 本stageでは`cargo-deny`未インストールのため未実行 | — | ci-pipeline | Unverified（次stageで所有・実行） |
| NFR2.8 | security-requirements.md | `#![forbid(unsafe_code)]` | `src/main.rs:7`, `src/lib.rs:7`に確認 | `grep forbid(unsafe_code) src/main.rs src/lib.rs` | build-and-test | Met |
| NFR2.9 | security-requirements.md | マルチセレクト初期状態は全チェックOFF | `selector::tests::initial_selected_indices_is_empty_for_many_items`等、100%カバレッジで検証 | `cargo test --lib selector::` | build-and-test | Met |
| NFR3.1 | scalability-requirements.md | Organization内全アクティブアカウント対応 | `org_discovery::tests`でACTIVEアカウントのみの列挙を検証 | `cargo test --lib org_discovery::` | build-and-test | Met |
| NFR3.2 | scalability-requirements.md | `--regions`明示必須、自動列挙なし | `cli::tests`で`--regions`必須のバリデーションを検証 | `cargo test --lib cli::` | build-and-test | Met |
| NFR3.3 | scalability-requirements.md | 数百件規模をメモリ上に全件保持 | NFR1.3と同一根拠 | `cargo test --lib aggregator::` | build-and-test | Met |
| NFR3.4 | scalability-requirements.md | v1は逐次実行（将来の並行化余地を残す設計） | 実装は逐次実行のみ。並行化は将来の拡張であり本v1の検証対象外 | コードレビュー（scanner.rsの逐次awaitチェーン） | 対象外（設計判断） | N/A |
| NFR4.1 | reliability-requirements.md | 可用性SLA/SLOは設定しない | 常駐サービスでないため対象外（要件自体がN/A） | reliability-requirements.md | 対象外（設計判断） | N/A |
| NFR4.2 | reliability-requirements.md | 部分障害時、当該アカウントのみ失敗として継続 | `scanner::tests::pagination_error_mid_stream_discards_partial_results_and_fails`等で検証 | `cargo test --lib scanner::` | build-and-test | Met |
| NFR4.3 | reliability-requirements.md | 監査ログ書き込み失敗時は操作を中断 | `audit_log_write_failure_is_reported_as_error_not_silently_ignored`で検証 | `cargo test --test audit_log_format` | build-and-test | Met |
| NFR4.4 | reliability-requirements.md | バックアップ/リカバリは対象外 | 要件自体が対象外と明記 | reliability-requirements.md | 対象外（設計判断） | N/A |
| NFR5.1 | observability-requirements.md | JSON Linesの監査ログ必須出力 | `audit_log_is_valid_json_lines_with_all_mandated_fields`で必須フィールドを検証 | `cargo test --test audit_log_format` | build-and-test | Met |
| NFR5.2 | observability-requirements.md | 標準エラー出力への進捗・エラー表示 | `tracing`/`tracing-subscriber`による実装をコードレビューで確認（自動テストなし、人間可読出力のため） | コードレビュー（main.rs, cli.rs） | build-and-test | Met（設計確認） |
| NFR5.3 | observability-requirements.md | 外部監視統合は対象外 | 要件自体が対象外と明記 | observability-requirements.md | 対象外（設計判断） | N/A |
| NFR5.4 | observability-requirements.md | ダッシュボードは対象外 | 要件自体が対象外と明記 | observability-requirements.md | 対象外（設計判断） | N/A |

## 失敗テスト詳細

なし（全件pass）。

## Loop-Back Log

### Loop-back 1 — 2026-09-10T23:45:35Z（解決済み）

修正・再レビュー・人間の再承認（Accepted risk）まで完了。execution.rs/identity.rsはLCOV基準で実行回数0行がゼロに到達、audit.rsの残存17行は構造的到達不能と検証された上で受容された。

- **Diagnosis**: Target Verification Matrixの「Testing Contract: 破壊的操作・安全パス100%floor」がNot Met（R-01, Code Generationのadvisoryレビューで既に指摘済み）。`execution.rs`(93.66%)/`audit.rs`(90.77%)/`identity.rs`(99.22%)に到達困難な未カバー分岐（`async_trait`マクロ展開由来の境界コード、`serde_json::to_string`失敗分岐、一部テストで未使用のモックアーム等）が残存。
- **Root-cause stage**: code-generation（生成されたテストコードのカバレッジ範囲の問題であり、Build and Testのビルド/テスト設定自体に起因しない）。
- **Planned fix**: 到達困難な分岐を狙った追加ユニットテストを`execution.rs`/`audit.rs`/`identity.rs`に追加し、100%パスカバレッジに近づける。
- **Estimated impact**: 効果 — 小〜中（既存の91件のテスト基盤に数件〜十数件追加する程度）。金銭コスト — なし。リスク — 低（新規テストの追加のみで既存の振る舞い・公開APIは変更しない）。

## Assumptions & Open Questions

None.
