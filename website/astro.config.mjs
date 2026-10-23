// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import starlightLlmsTxt from 'starlight-llms-txt';

// Google アナリティクス（GA4）。本番ドメインで開いたときだけ gtag.js を読み込み、
// `astro dev` や `astro preview`（localhost）からは送信しない。
// 送信内容と停止方法は src/content/docs/privacy.md と ja/privacy.md で公表している。
const GA_MEASUREMENT_ID = 'G-T6E24WDMJK';
const PRODUCTION_HOST = 'cwsweep.hidetaka.dev';
const analyticsScript = `if (location.hostname === '${PRODUCTION_HOST}') {
	window.dataLayer = window.dataLayer || [];
	window.gtag = function () { window.dataLayer.push(arguments); };
	window.gtag('js', new Date());
	window.gtag('config', '${GA_MEASUREMENT_ID}');
	const script = document.createElement('script');
	script.async = true;
	script.src = 'https://www.googletagmanager.com/gtag/js?id=${GA_MEASUREMENT_ID}';
	document.head.appendChild(script);
}`;

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
			routeMiddleware: './src/routeData.ts',
			// llms.txt v2: 全ページから /llms.txt を rel="describedby" で示す。
			// https://llmstxt.org/changes.html
			head: [
				{ tag: 'link', attrs: { rel: 'describedby', href: '/llms.txt' } },
				{ tag: 'script', content: analyticsScript },
			],
			components: {
				Footer: './src/components/Footer.astro',
			},
			plugins: [
				starlightLlmsTxt({
					projectName: 'cwsweep',
					// プライバシーポリシーは CLI の使い方ではないため、要約版（llms-small.txt）から外す。
					// starlight-llms-txt の exclude は llms-small.txt にだけ効き、llms-full.txt には全ページが入る。
					exclude: ['privacy', 'ja/privacy'],
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
