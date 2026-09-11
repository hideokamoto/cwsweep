# Cross-Unit Final Coverage Gate — cwsweep

Requirements Analysisはスコープ設計によりスキップされているため、`requirements.md`は存在しない。User Storiesも同様に存在しない（Requirements Analysis前提のstoriesステージは本スコープでは実行されない）。代わりに`aidlc/spaces/default/intents/260910-cwsweep-cli/ideation/scope-definition/intent-backlog.md`のProto-Unit ID（PU-1〜PU-7、Must）を代替の網羅性チェック対象とする。

`construction/code-generation/traceability.json`（zero-Unit/stage-levelのみ、per-Unitファイルは存在しない）を読み込み、各Proto-Unit ID・NFR IDが`status: OK`でカバーされ、対象ファイルが実在することを検証した。

## Verdict: PASS

## 網羅性チェック

| ID | Status | Owning Stage/Unit | Target File | ファイル存在確認 |
|---|---|---|---|---|
| PU-1 | OK | code-generation (stage-level) | src/org_discovery.rs, src/identity.rs, src/credentials.rs | 存在確認済み |
| PU-2 | OK | code-generation (stage-level) | src/scanner.rs, src/aggregator.rs | 存在確認済み |
| PU-3 | OK | code-generation (stage-level) | src/output.rs | 存在確認済み |
| PU-4 | OK | code-generation (stage-level) | src/selector.rs | 存在確認済み |
| PU-5 | OK | code-generation (stage-level) | src/execution.rs | 存在確認済み |
| PU-6 | OK | code-generation (stage-level) | src/execution.rs, src/planner.rs, src/confirmation.rs | 存在確認済み |
| PU-7 | OK | code-generation (stage-level) | src/audit.rs | 存在確認済み |
| NFR1.2 | OK | code-generation (stage-level) | src/scanner.rs | 存在確認済み |
| NFR1.3 | OK | code-generation (stage-level) | src/aggregator.rs | 存在確認済み |
| NFR1.4 | OK | code-generation (stage-level) | Cargo.toml (aws-config既定リトライ設定を利用) | 存在確認済み |
| NFR2.1 | OK | code-generation (stage-level) | src/credentials.rs | 存在確認済み |
| NFR2.3 | OK | code-generation (stage-level) | src/identity.rs | 存在確認済み |
| NFR2.4 | OK | code-generation (stage-level) | src/execution.rs | 存在確認済み |
| NFR2.5 | OK | code-generation (stage-level) | src/execution.rs | 存在確認済み |
| NFR2.9 | OK | code-generation (stage-level) | src/selector.rs | 存在確認済み |
| NFR3.2 | OK | code-generation (stage-level) | src/cli.rs | 存在確認済み |
| NFR3.3 | OK | code-generation (stage-level) | src/aggregator.rs | 存在確認済み |
| NFR4.2 | OK | code-generation (stage-level) | src/scanner.rs | 存在確認済み |
| NFR4.3 | OK | code-generation (stage-level) | src/audit.rs | 存在確認済み |
| NFR5.1 | OK | code-generation (stage-level) | src/audit.rs | 存在確認済み |
| NFR5.2 | OK | code-generation (stage-level) | src/cli.rs | 存在確認済み |

## 未カバー要素

なし。`construction/code-generation/traceability.json`に列挙された全IDが`status: OK`で、対象ファイルはすべてワークスペースルートに実在することを確認した（N/A判定のNFR2.2/2.6-2.8/3.1/3.4/4.1/4.4/5.3/5.4は同ファイル・`test-results.md`のTarget Verification Matrixで別途説明済み）。

## Assumptions & Open Questions

None.
