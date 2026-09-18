<!-- INVARIANT: examples are single-line HTML comments so a fresh template parses to total=0 (MEMORY_EMPTY). Do NOT un-comment or split across lines. t100 guards this. -->
> This file is kept up to date automatically while the stage runs. Add observations at the review step, not by editing here directly.

## Interpretations
<!-- example: 2026-05-29T10:14:32Z — chose REST over GraphQL; the consuming team only needs CRUD, revisit if subscriptions land -->
- 2026-09-18T21:26:14Z — 別セッションからの引き継ぎ: redo-jump後の新試行でaudit-readerのNFR設計成果物は内容を変えずに再利用（Keep）し、要約確認を人間に取り直してから助言レビューを再ディスパッチした。

## Deviations
<!-- example: 2026-05-29T10:14:32Z — skipped the optional caching layer the stage prose suggested; the dataset is small enough that it adds risk -->

## Tradeoffs
<!-- example: 2026-05-29T10:14:32Z — picked TDD over BDD this run; the team is unit-first and the domain is well-understood -->

## Open questions
<!-- example: 2026-05-29T10:14:32Z — confirm the retention window with compliance before the next stage hardens the schema -->
2026-09-18T02:28:31Z — advisory reviewer (audit-reader, R-01, Major): the proposed text-based static regression test for NFR2.1/NFR2.2 has false-negative gaps (aliased imports, type aliases, generic/trait-object indirection) and doesn't actually prove "compile-time unreachable" as NFR2.1 demands. Recommend a stronger check (AST-based via `syn`, or a dependency-graph tool) at Code Generation, or explicit documentation of the residual risk.
2026-09-18T02:28:31Z — advisory reviewer (audit-reader, R-02, Major): `AuditReadError::Io(std::io::Error)` as designed can't derive `Clone`/`PartialEq`/`Eq` like every sibling error type in src/error.rs does (std::io::Error implements none of those). Code Generation should resolve this — e.g. store the io::Error's message as a String, matching the rest of the crate's error-type convention.
2026-09-18T02:28:31Z — advisory reviewer (audit-reader, R-03, Minor): the static test's region-extraction marker convention for carving `AuditReader` code out of the shared src/audit.rs file was left unspecified in security-design.md; a missing/misplaced marker would silently produce an always-passing test. Code Generation should define this concretely.
2026-09-18T02:28:31Z — advisory reviewer (audit-reader, R-04, Minor): security-design.md's pseudocode doesn't explicitly show rules.md BR2.1 (missing file → empty outcome, not an error) as its own branch, only implying it via the NFR3.3 discussion.
