---
title: Agent Skill and llms.txt
description: Install the cwsweep Agent Skill, and find the llms.txt and skill endpoints published by this site.
sidebar:
  order: 6
---

cwsweep ships an [Agent Skill](https://agentskills.io/specification) that teaches a coding agent how to use the CLI safely: always `scan` first, treat dry-run as the default, and hand the `--execute` run to you after you approve it.

## Install

```bash
npx skills add hideokamoto/cwsweep --skill cwsweep
```

Keep `--skill cwsweep`. This repository also contains other, unrelated skills (the AI-DLC workflow files under `.claude/` and `.cursor/`), and without the flag they are offered too.

The skill source is [`skills/cwsweep/SKILL.md`](https://github.com/hideokamoto/cwsweep/blob/main/skills/cwsweep/SKILL.md). When the CLI changes, the skill is updated in the same change, and a test fails if its flags, defaults, or retention values drift from the code.

## What this site publishes

| URL | Content |
|---|---|
| [`/llms.txt`](/llms.txt) | An [llms.txt](https://llmstxt.org/) entry point. Every page links to it with `<link rel="describedby" href="/llms.txt">`. |
| [`/llms-full.txt`](/llms-full.txt), [`/llms-small.txt`](/llms-small.txt) | The documentation as plain text (complete and abridged). Generated from the English pages. |
| [`/skill.md`](/skill.md) | The skill file. |
| [`/.well-known/agent-skills/cwsweep/SKILL.md`](/.well-known/agent-skills/cwsweep/SKILL.md) | The same file, at the discovery path. |
| [`/.well-known/agent-skills/index.json`](/.well-known/agent-skills/index.json) | A discovery index with the skill's `name`, `description`, `url`, and a `sha256` digest of the file. |

The skill files are read from `skills/` when the site is built; the site keeps no copy. The discovery index follows the [Agent Skills Discovery RFC](https://github.com/cloudflare/agent-skills-discovery-rfc), which is still a draft (v0.2.0 at the time of writing).
