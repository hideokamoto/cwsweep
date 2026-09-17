<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs. Add observations at the review step, not by editing here directly.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->
- 2026-09-17T00:00:00Z — advisory reviューは1回のみのため、Request Changes後の再レビューは通常のiteration+1では受理されず、redo jump（/aidlc --stage units-generation）で本ステージをクリーンに再入場する必要があった。既存4成果物はKeepで再利用し、Consolidated Summary Confirmationを再確認した上でレビューを再ディスパッチした。再レビューはREADY判定（R-01/R-02解消、R-03は非ブロッキングMinor）。

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
- 2026-09-17T00:00:00Z — アーキテクチャレビュアー(advisory)がREADY判定を返したが、R-01(Major)として、unit-of-work.mdの「Coverage Verification」がNFR1-NFR5全件のUnit割当を主張しているにもかかわらず、NFR4（後方互換シム混入防止の固定化テスト）がどのUnitの「カバーする要件」にも明記されていない齟齬を指摘。R-02(Minor)としてFR4.3のtraceability.json上のtarget表記（U1のみ）がunit-of-work.md/story-mapのクロスカッティング記述と完全には一致しない点も指摘。いずれも承認ゲートで人間に判断を委ねる。
