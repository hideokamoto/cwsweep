---
name: cwsweep-orgwide-logs-cleanup
depth: Standard
keywords: []
description: "Org-wide logs cleanup for cwsweep - composed custom scope"
skeleton: off
review_cap: advisory
change_control: strict
---

# cwsweep-orgwide-logs-cleanup scope

Composed custom scope (13 EXECUTE / 20 SKIP / 10 gates) for an org-wide logs
cleanup effort. Not inferable — `keywords: []` — since this scope was
produced by the adaptive-workflows composer for this specific task rather
than affirmed as a reusable team default; select it explicitly with
`--scope cwsweep-orgwide-logs-cleanup`.

Change Control defaults to strict: an input that changes after a human
approved or confirmed something reopens that approval rather than being
recorded and continuing.

## Membership

No keyword triggers (composed scopes are not inferable). Executes:
workspace-scaffold, workspace-detection, state-init, intent-capture,
scope-definition, approval-handoff, practices-discovery, domain-design,
nfr-requirements, nfr-design, code-generation, build-and-test, ci-pipeline.
Skips: market-research, feasibility, team-formation, rough-mockups,
reverse-engineering, requirements-analysis, user-stories, refined-mockups,
units-generation, contract-design, delivery-planning, functional-design,
infrastructure-design, deployment-pipeline, environment-provisioning,
deployment-execution, observability-setup, incident-response,
performance-validation, feedback-optimization.
