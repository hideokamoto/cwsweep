// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import starlightLlmsTxt from 'starlight-llms-txt';

// GitHub Pages + カスタムドメイン（public/CNAME）で配信するため base は設定しない。
export default defineConfig({
	site: 'https://cwsweep.hidetaka.dev',
	integrations: [
		starlight({
			title: 'cwsweep',
			description: 'A Rust CLI that inventories CloudWatch Logs across an entire AWS Organization and safely deletes or changes their retention',
			// 英語をルート（/）に置き、日本語は /ja/ 配下で配信する。
			defaultLocale: 'root',
			locales: {
				root: { label: 'English', lang: 'en' },
				ja: { label: '日本語', lang: 'ja' },
			},
			// llms.txt v2: 全ページから /llms.txt を rel="describedby" で示す。
			// https://llmstxt.org/changes.html
			head: [
				{ tag: 'link', attrs: { rel: 'describedby', href: '/llms.txt' } },
			],
			plugins: [
				starlightLlmsTxt({
					projectName: 'cwsweep',
					// 本文は英語（既定ロケール）のページから生成される。
					details: [
						'cwsweep is a CLI. Run `cwsweep --help` (every help ends with the docs URL) or read the pages below. Dry-run is the default; nothing is deleted or changed unless `--execute` is passed.',
						'',
						'An Agent Skill (https://agentskills.io/specification) that teaches an agent how to use cwsweep safely is published from the same repository:',
						'',
						'- Skill file: https://cwsweep.hidetaka.dev/skill.md',
						'- Skill discovery index (Agent Skills Discovery RFC, draft): https://cwsweep.hidetaka.dev/.well-known/agent-skills/index.json',
						'- Install: `npx skills add hideokamoto/cwsweep --skill cwsweep`',
					].join('\n'),
				}),
			],
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/hideokamoto/cwsweep' },
			],
			editLink: {
				baseUrl: 'https://github.com/hideokamoto/cwsweep/edit/main/website/',
			},
			sidebar: [
				{
					label: 'Introduction',
					translations: { ja: 'はじめに' },
					items: ['getting-started'],
				},
				{
					label: 'Guides',
					translations: { ja: 'ガイド' },
					items: [{ autogenerate: { directory: 'guides' } }],
				},
				{
					label: 'Reference',
					translations: { ja: 'リファレンス' },
					items: [{ autogenerate: { directory: 'reference' } }],
				},
			],
		}),
	],
});
