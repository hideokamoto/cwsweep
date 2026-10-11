// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// GitHub Pages + カスタムドメイン（public/CNAME）で配信するため base は設定しない。
export default defineConfig({
	site: 'https://cwsweep.hidetaka.dev',
	integrations: [
		starlight({
			title: 'cwsweep',
			description: 'AWS Organization全体のCloudWatch Logsを棚卸しし、安全に削除・retention変更するRust製CLI',
			defaultLocale: 'root',
			locales: {
				root: { label: '日本語', lang: 'ja' },
			},
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/hideokamoto/cwsweep' },
			],
			editLink: {
				baseUrl: 'https://github.com/hideokamoto/cwsweep/edit/main/website/',
			},
			sidebar: [
				{ label: 'はじめに', items: ['getting-started'] },
				{ label: 'ガイド', items: [{ autogenerate: { directory: 'guides' } }] },
				{ label: 'リファレンス', items: [{ autogenerate: { directory: 'reference' } }] },
			],
		}),
	],
});
