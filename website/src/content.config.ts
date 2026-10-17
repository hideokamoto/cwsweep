import { defineCollection, z } from 'astro:content';
import { docsLoader } from '@astrojs/starlight/loaders';
import { docsSchema } from '@astrojs/starlight/schema';
import { fetchReleases } from './lib/releases';

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
					store.set({ id: release.tag, data, rendered: await renderMarkdown(body) });
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
