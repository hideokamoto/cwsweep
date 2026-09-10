# Third-Party Notices

## AI-DLC Workflows (`.claude/`, `.cursor/`, `aidlc/`, `AGENTS.md`)

This project vendors the **AI-DLC Workflows v2** framework:

- Upstream: <https://github.com/awslabs/aidlc-workflows>
- Version installed: `2.8.2` (commit `a8f2dc6198c011343e7674bc82534c0adaf0b548`)
- License: **MIT No Attribution (MIT-0)**, Copyright Amazon.com, Inc. or its
  affiliates. Full text preserved at
  [`licenses/aidlc-workflows-LICENSE.txt`](licenses/aidlc-workflows-LICENSE.txt).
- Scope of the vendored tree: `.claude/`, `.cursor/`, `aidlc/`, `AGENTS.md`,
  and `.mcp.json`.

MIT-0 grants unrestricted use, modification, and redistribution with no
attribution requirement. This project is licensed under the GNU GPLv3 (see
[`LICENSE`](LICENSE)); combining MIT-0-licensed files into a GPLv3 project is
permitted because MIT-0 imposes no terms that conflict with the GPLv3. This
notice is provided for transparency, not because MIT-0 requires it.

### How this install was produced

The vendored files were generated **from the unmodified upstream `core/` and
`harness/{claude,cursor}/` sources**, using the project's own official build
pipeline (`bun scripts/package.ts claude` and `bun scripts/package.ts
cursor`, i.e. the same generator the upstream release process runs), then
copied into this repository exactly as generated. No file under `core/` or
`harness/` in the upstream source tree was hand-edited to produce this
install; nothing in the generated output was hand-edited except the one
deviation below.

### Deviation from upstream: Bedrock forcing removed

Upstream's generated `.claude/settings.json` unconditionally sets:

```json
"CLAUDE_CODE_USE_BEDROCK": "1",
"AWS_REGION": "us-east-1",
"ANTHROPIC_DEFAULT_FABLE_MODEL": "global.anthropic.claude-fable-5[1m]",
"ANTHROPIC_DEFAULT_OPUS_MODEL": "global.anthropic.claude-opus-4-8[1m]",
"ANTHROPIC_DEFAULT_SONNET_MODEL": "global.anthropic.claude-sonnet-4-6[1m]",
"ANTHROPIC_DEFAULT_HAIKU_MODEL": "global.anthropic.claude-haiku-4-5-20251001-v1:0"
```

This forces every Claude Code session that loads this project's settings onto
AWS Bedrock, which requires Bedrock model access and AWS credentials on the
default SDK credential chain. **Claude Code on the web** and other
non-AWS-credentialed runners cannot supply that, so these six keys were
removed from this repo's `.claude/settings.json` `env` block. Nothing else in
that file, and no file under `.claude/skills/`, `.claude/tools/`,
`.claude/hooks/`, `.claude/agents/`, `.claude/sensors/`, or
`.claude/knowledge/` (the framework's methodology/engine — its "core"), was
touched. With the six keys removed, Claude Code authenticates through the
caller's normal Anthropic account instead — the path both Claude Code on the
web and any other non-Bedrock runner already use.

The `.cursor/` projection needed no equivalent change: upstream never emits
Bedrock-forcing settings for the Cursor harness (confirmed by diffing the
freshly generated `.cursor/` tree against `.claude/`) — Cursor's CLI/cloud
agent routes model selection through Cursor's own backend regardless of this
repository's settings, so it and Cursor's cloud/background agent run
unmodified.

To opt back into Bedrock on a given machine, set the same six keys in your
own gitignored `.claude/settings.local.json` (copy
`.claude/settings.local.json.example`) — see the comment placed there, and
upstream's `docs/guide/01-getting-started.md` § "AWS Bedrock Setup" (not
vendored into this repo; read it from the upstream repository linked above).

### Verifying "no other change"

```sh
git clone https://github.com/awslabs/aidlc-workflows
cd aidlc-workflows && git checkout a8f2dc6198c011343e7674bc82534c0adaf0b548
bun install
bun scripts/package.ts claude
bun scripts/package.ts cursor
diff -r dist-release/claude/.claude <path-to-this-repo>/.claude   # only the env block in settings.json differs
diff -r dist-release/cursor/.cursor <path-to-this-repo>/.cursor   # no differences
```
