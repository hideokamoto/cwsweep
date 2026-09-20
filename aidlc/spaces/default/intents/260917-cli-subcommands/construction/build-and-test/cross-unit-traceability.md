# Cross-Unit Final Coverage Gate — 260917-cli-subcommands

`inception/requirements-analysis/requirements.md` の全 FR / NFR を列挙し、3 Unit（audit-reader / cli-foundation / release-docs）の `code-generation/traceability.json` で `status: OK` にカバーされ、対象ファイルが実在することを検証した。User Stories stage は `user-stories-assessment.md` により未実行のため AC は対象外。NFR は requirements.md では NFR1〜NFR5 の親IDのみ定義され、Unit の traceability では NFR-Requirements で細分化した子ID（NFR2.3 等）で記録されているため、子IDが1件以上 OK であれば親IDをカバー済みとみなす。

## Verdict: PASS

## 網羅性チェック

| ID | Status | Owning Unit | Target File | ファイル存在確認 |
|---|---|---|---|---|
| FR1.1 | OK | cli-foundation | src/cli.rs | 存在確認済み |
| FR1.2 | OK | cli-foundation | src/main.rs | 存在確認済み |
| FR2.1 | OK | cli-foundation | src/cli.rs | 存在確認済み |
| FR2.2 | OK | cli-foundation | src/cli.rs | 存在確認済み |
| FR2.3 | OK | cli-foundation | src/cli.rs | 存在確認済み |
| FR2.4 | OK | cli-foundation | src/cli.rs | 存在確認済み |
| FR3.1 | OK | cli-foundation | src/cli.rs | 存在確認済み |
| FR3.2 | OK | cli-foundation | src/cli.rs | 存在確認済み |
| FR3.3 | OK | cli-foundation | src/cli.rs | 存在確認済み |
| FR3.4 | OK | cli-foundation | src/cli.rs | 存在確認済み |
| FR3.5 | OK | cli-foundation | src/cli.rs | 存在確認済み |
| FR4.1 | OK | audit-reader | src/audit.rs | 存在確認済み |
| FR4.2 | OK | audit-reader | src/audit.rs | 存在確認済み |
| FR4.3 | OK | cli-foundation | src/output.rs | 存在確認済み |
| FR4.4 | OK | audit-reader | src/audit.rs | 存在確認済み |
| FR4.5 | OK | audit-reader | src/audit.rs | 存在確認済み |
| FR4.6 | OK | audit-reader | src/audit.rs | 存在確認済み |
| FR5.1 | OK | cli-foundation | src/cli.rs | 存在確認済み |
| FR5.2 | OK | cli-foundation | src/cli.rs | 存在確認済み |
| FR5.3 | OK | cli-foundation | src/cli.rs | 存在確認済み |
| FR5.4 | OK | cli-foundation | src/cli.rs | 存在確認済み |
| FR6.1 | OK | cli-foundation, release-docs | README.md, src/cli.rs | 存在確認済み |
| FR6.2 | OK | release-docs | CHANGELOG.md | 存在確認済み |
| NFR1 | OK | cli-foundation | src/cli.rs（cli-foundation nfr-requirements/traceability.json で NFR1 → NFR5.1 / NFR5.10 / NFR5.11 へ細分化。3件とも code-generation traceability で OK） | 存在確認済み |
| FR6.3 | OK | release-docs | Cargo.toml | 存在確認済み |
| NFR2 | OK | audit-reader, cli-foundation | src/audit.rs, src/cli.rs | 存在確認済み |
| NFR3 | OK | audit-reader | src/audit.rs | 存在確認済み |
| NFR4 | OK | release-docs | README.md | 存在確認済み |
| NFR5 | OK | cli-foundation, release-docs | Cargo.toml, README.md, src/cli.rs, src/main.rs, src/output.rs | 存在確認済み |

## 未カバー要素

なし。NFR1 は nfr-requirements で NFR5.1 / NFR5.10 / NFR5.11 へ分解されて追跡されている（上表参照）。

## Assumptions & Open Questions

None.
