import { defineRouteMiddleware } from '@astrojs/starlight/route-data';

export const onRequest = defineRouteMiddleware((context) => {
	const route = context.locals.starlightRoute;
	const imagePath = route.locale === 'ja' ? '/og/og-ja.png' : '/og/og-en.png';
	const site = context.site;

	if (!site) {
		throw new Error('Astro site must be configured to build social image URLs.');
	}

	const imageUrl = new URL(imagePath, site).href;
	const metadata: typeof route.head = [
		{ tag: 'meta', attrs: { property: 'og:image', content: imageUrl } },
		{ tag: 'meta', attrs: { property: 'og:image:width', content: '1200' } },
		{ tag: 'meta', attrs: { property: 'og:image:height', content: '630' } },
		{
			tag: 'meta',
			attrs: {
				property: 'og:image:alt',
				content:
					route.locale === 'ja'
						? 'cwsweep — AWS Organization 全体の CloudWatch Logs をまとめてお掃除'
						: 'cwsweep — Clean up CloudWatch Logs across your whole AWS Organization',
			},
		},
		{ tag: 'meta', attrs: { name: 'twitter:card', content: 'summary_large_image' } },
		{ tag: 'meta', attrs: { name: 'twitter:image', content: imageUrl } },
	];
	const metadataKeys = new Set([
		'property:og:image',
		'property:og:image:width',
		'property:og:image:height',
		'property:og:image:alt',
		'name:twitter:card',
		'name:twitter:image',
	]);

	for (let index = route.head.length - 1; index >= 0; index--) {
		const item = route.head[index];
		if (item.tag !== 'meta') continue;
		const key = item.attrs?.property
			? `property:${item.attrs.property}`
			: item.attrs?.name
				? `name:${item.attrs.name}`
				: undefined;
		if (key && metadataKeys.has(key)) route.head.splice(index, 1);
	}

	route.head.push(...metadata);
});
