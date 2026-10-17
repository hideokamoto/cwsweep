import { defineCollection, z } from 'astro:content';
import { docsLoader } from '@astrojs/starlight/loaders';
import { docsSchema } from '@astrojs/starlight/schema';
import rehypeParse from 'rehype-parse';
import rehypeSanitize from 'rehype-sanitize';
import rehypeStringify from 'rehype-stringify';
import { unified } from 'unified';
import { fetchReleases } from './lib/releases';

// リリース本文はコミットメッセージ由来で、サイトの作者以外が書いた文字列を含みうる。
// Astro の Markdown 処理は生の HTML を通すため、描画後の HTML を rehype-sanitize の
// 既定スキーマ（GitHub と同等の許可リスト）で無害化してから載せる。
const sanitizer = unified().use(rehypeParse, { fragment: true }).use(rehypeSanitize).use(rehypeStringify);
const sanitizeHtml = async (html: string) => String(await sanitizer.process(html));

export const collections = {
	docs: defineCollection({ loader: docsLoader(), schema: docsSchema() }),
	// GitHub Releases（ビルド時に取得）。本文の Markdown は Astro の Markdown 処理で HTML にする。
	releases: defineCollection({
		loader: {
			name: 'github-releases',
			load: async ({ store, renderMarkdown }) => {
				store.clear();
				for (const release of await fetchReleases()) {
					const { body, ...data } = release;
					const rendered = await renderMarkdown(body);
					store.set({ id: release.tag, data, rendered: { ...rendered, html: await sanitizeHtml(rendered.html) } });
				}
			},
		},
		schema: z.object({
			tag: z.string(),
			name: z.string(),
			url: z.string().url(),
			publishedAt: z.string(),
			prerelease: z.boolean(),
			assets: z.array(z.object({ name: z.string(), url: z.string().url(), size: z.number() })),
		}),
	}),
};
