---
title: Agent Skill と llms.txt
description: cwsweep の Agent Skill のインストール方法と、このサイトが公開している llms.txt・スキルの URL。
sidebar:
  order: 6
---

cwsweep には、コーディングエージェントが CLI を安全に使うための [Agent Skill](https://agentskills.io/specification) が付属します。必ず先に `scan` を実行する、dry-run を既定として扱う、`--execute` での本実行は利用者の承認後に利用者の端末で行う、といった手順を伝えます。

## インストール

```bash
npx skills add hideokamoto/cwsweep --skill cwsweep
```

`--skill cwsweep` は必ず付けてください。このリポジトリには、無関係な他のスキル（`.claude/` と `.cursor/` の AI-DLC ワークフロー）も含まれており、付けないとそれらも選択肢に出ます。

スキルの本体は [`skills/cwsweep/SKILL.md`](https://github.com/hideokamoto/cwsweep/blob/main/skills/cwsweep/SKILL.md) です。CLI を変更するときは同じ変更でスキルも更新します。フラグ・既定値・retention の許容値がコードとずれると、テストが失敗します。

## このサイトが公開しているもの

| URL | 内容 |
|---|---|
| [`/llms.txt`](/llms.txt) | [llms.txt](https://llmstxt.org/) の入口。全ページから `<link rel="describedby" href="/llms.txt">` で参照されます。 |
| [`/llms-full.txt`](/llms-full.txt)、[`/llms-small.txt`](/llms-small.txt) | ドキュメント全体のプレーンテキスト（完全版と抜粋版）。英語のページから生成されます。 |
| [`/skill.md`](/skill.md) | スキルファイル。 |
| [`/.well-known/agent-skills/cwsweep/SKILL.md`](/.well-known/agent-skills/cwsweep/SKILL.md) | 同じファイル（ディスカバリ用のパス）。 |
| [`/.well-known/agent-skills/index.json`](/.well-known/agent-skills/index.json) | スキルの `name`・`description`・`url` と、ファイルの `sha256` ダイジェストを持つディスカバリ用インデックス。 |

スキルのファイルはサイトのビルド時に `skills/` から読み込みます。サイト側にコピーは持ちません。ディスカバリ用インデックスは [Agent Skills Discovery RFC](https://github.com/cloudflare/agent-skills-discovery-rfc) に従っています。この RFC は現時点では草案（v0.2.0）です。
