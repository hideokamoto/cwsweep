import assert from 'node:assert/strict';
import { access, readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

const distDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../dist');

const locales = {
	en: {
		image: 'https://cwsweep.hidetaka.dev/og/og-en.png',
		alt: 'cwsweep — Clean up CloudWatch Logs across your whole AWS Organization',
	},
	ja: {
		image: 'https://cwsweep.hidetaka.dev/og/og-ja.png',
		alt: 'cwsweep — AWS Organization 全体の CloudWatch Logs をまとめてお掃除',
	},
};

const pages = [
	{
		name: 'English home',
		path: 'index.html',
		locale: 'en',
		video: {
			source: '/videos/cwsweep-intro-en.mp4',
			poster: '/videos/cwsweep-intro-en-poster.jpg',
		},
	},
	{
		name: 'English getting started',
		path: 'getting-started/index.html',
		locale: 'en',
		video: {
			source: '/videos/cwsweep-howto-en.mp4',
			poster: '/videos/cwsweep-howto-en-poster.jpg',
		},
	},
	{ name: 'English guide', path: 'guides/scan/index.html', locale: 'en' },
	{
		name: 'Japanese home',
		path: 'ja/index.html',
		locale: 'ja',
		video: {
			source: '/videos/cwsweep-intro-ja.mp4',
			poster: '/videos/cwsweep-intro-ja-poster.jpg',
		},
	},
	{
		name: 'Japanese getting started',
		path: 'ja/getting-started/index.html',
		locale: 'ja',
		video: {
			source: '/videos/cwsweep-howto-ja.mp4',
			poster: '/videos/cwsweep-howto-ja-poster.jpg',
		},
	},
	{ name: 'Japanese guide', path: 'ja/guides/scan/index.html', locale: 'ja' },
];

async function readPage(relativePath) {
	return readFile(path.join(distDir, relativePath), 'utf8');
}

function parseAttributes(tag) {
	return Object.fromEntries(
		[...tag.matchAll(/([^\s=<>/]+)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))/g)].map(
			([, name, doubleQuoted, singleQuoted, unquoted]) => [
				name,
				doubleQuoted ?? singleQuoted ?? unquoted,
			],
		),
	);
}

function metaTags(html) {
	return [...html.matchAll(/<meta\b[^>]*>/gi)].map(([tag]) => parseAttributes(tag));
}

function assertMeta(html, attribute, name, content, pageName) {
	const matching = metaTags(html).filter((meta) => meta[attribute] === name);
	assert.equal(matching.length, 1, `${pageName} must have exactly one ${name} meta tag`);
	assert.equal(matching[0].content, content, `${pageName} ${name} content`);
}

for (const page of pages) {
	test(`${page.name} has the correct social metadata`, async () => {
		const html = await readPage(page.path);
		const { image, alt } = locales[page.locale];

		assertMeta(html, 'property', 'og:image', image, page.name);
		assertMeta(html, 'property', 'og:image:width', '1200', page.name);
		assertMeta(html, 'property', 'og:image:height', '630', page.name);
		assertMeta(html, 'property', 'og:image:alt', alt, page.name);
		assertMeta(html, 'name', 'twitter:card', 'summary_large_image', page.name);
		assertMeta(html, 'name', 'twitter:image', image, page.name);
	});

	test(`${page.name} references existing public media`, async () => {
		const html = await readPage(page.path);
		const referencedPaths = [
			...html.matchAll(/(?:https?:\/\/[^/"'\s]+)?(\/(?:videos|og)\/[^"'?#\s<]+)/g),
		]
			.map(([, url]) => url)
			.filter((url, index, all) => all.indexOf(url) === index);

		for (const url of referencedPaths) {
			await access(path.join(distDir, url.slice(1)));
		}
	});

	if (page.video) {
		test(`${page.name} embeds its expected video and poster`, async () => {
			const html = await readPage(page.path);
			const videoElements = [...html.matchAll(/<video\b[^>]*>[\s\S]*?<\/video>/gi)];

			assert.equal(videoElements.length, 1, `${page.name} must contain one video`);
			const [videoHtml] = videoElements[0];
			const openingTag = videoHtml.match(/^<video\b[^>]*>/i)[0];
			const videoAttributes = parseAttributes(openingTag);
			const sourceTags = [...videoHtml.matchAll(/<source\b[^>]*>/gi)];

			assert.equal(videoAttributes.preload, 'none');
			assert.equal(videoAttributes.poster, page.video.poster);
			assert.match(openingTag, /\bcontrols(?:\s|>|=)/);
			assert.match(openingTag, /\bplaysinline(?:\s|>|=)/);
			assert.equal(sourceTags.length, 1);
			assert.equal(parseAttributes(sourceTags[0][0]).src, page.video.source);
			assert.equal(parseAttributes(sourceTags[0][0]).type, 'video/mp4');
			assert.match(videoHtml, new RegExp(`<a\\b[^>]*href="${page.video.source}"`));
		});
	}
}
