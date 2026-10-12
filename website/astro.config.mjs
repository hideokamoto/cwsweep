// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

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
