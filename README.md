# cwsweep

## AI-DLC Workflows v2

This repository ships [AI-DLC Workflows v2](https://github.com/awslabs/aidlc-workflows)
(`awslabs/aidlc-workflows`, v2.8.2), configured for both **Claude Code**
(including Claude Code on the web) and **Cursor** (including its cloud/
background agent). Run `/aidlc` in either harness to start or resume a
workflow.

- `.claude/` — Claude Code harness projection.
- `.cursor/` — Cursor harness projection (same tree serves the Cursor IDE,
  the Cursor CLI, and Cursor's cloud/background agent).
- `aidlc/` — shared workspace (method files, workflow state, audit trail);
  identical across both harnesses.
- `AGENTS.md` — ambient project instructions Cursor auto-reads.

Both harness trees were generated unmodified from upstream's own build
pipeline, with one deliberate change: the AWS Bedrock forcing that upstream's
`.claude/settings.json` ships by default has been removed, so the framework
authenticates through the normal Claude Code/Anthropic account path instead —
required for it to run on Claude Code on the web and other non-AWS-
credentialed runners. Nothing else was touched. Full details, exactly what
was changed and how it was verified, and the third-party license: see
[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).

## License

This repository's own code is licensed under the GNU GPLv3 — see
[`LICENSE`](LICENSE). The vendored AI-DLC Workflows tree
(`.claude/`, `.cursor/`, `aidlc/`, `AGENTS.md`, `.mcp.json`) is
third-party software under the MIT-0 license; see
[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) and
[`licenses/aidlc-workflows-LICENSE.txt`](licenses/aidlc-workflows-LICENSE.txt).
