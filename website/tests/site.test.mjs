import assert from 'node:assert/strict';
import { access, readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import vm from 'node:vm';

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

const GA_MEASUREMENT_ID = 'G-T6E24WDMJK';

function inlineScripts(html) {
	return [...html.matchAll(/<script\b([^>]*)>([\s\S]*?)<\/script>/gi)].map(([, attrs, body]) => ({
		attrs: parseAttributes(`<script ${attrs}>`),
		body,
	}));
}

for (const page of pages) {
	test(`${page.name} loads Google Analytics only on the production host`, async () => {
		const html = await readPage(page.path);
		const analytics = inlineScripts(html).filter(({ body }) => body.includes(GA_MEASUREMENT_ID));

		assert.equal(analytics.length, 1, `${page.name} must have exactly one analytics script`);
		assert.match(analytics[0].body, /location\.hostname === 'cwsweep\.hidetaka\.dev'/);
		// gtag.js は本番ホストの判定を通ったあとに動的に読み込む。静的な <script src> では
		// localhost でも送信されてしまうため、置かれていないことを確かめる。
		assert.doesNotMatch(html, /<script\b[^>]*src="https:\/\/www\.googletagmanager\.com\//);
	});

	test(`${page.name} links to the privacy policy in its own locale`, async () => {
		const html = await readPage(page.path);
		const href = page.locale === 'ja' ? '/ja/privacy/' : '/privacy/';
		const links = [...html.matchAll(/<a\b[^>]*data-privacy-link[^>]*>/gi)].map(([tag]) => parseAttributes(tag));

		assert.equal(links.length, 1, `${page.name} must have one privacy link`);
		assert.equal(links[0].href, href);
		await access(path.join(distDir, href.slice(1), 'index.html'));
	});
}

// 本番ホストでの実行結果を、ビルド済みのスクリプトをそのまま動かして確かめる
async function runAnalytics(href, referrer) {
	const html = await readPage('guides/scan/index.html');
	const [{ body }] = inlineScripts(html).filter((script) => script.body.includes(GA_MEASUREMENT_ID));
	const appended = [];
	const window = {};
	const context = {
		window,
		location: new URL(href),
		document: {
			referrer,
			createElement: () => ({}),
			head: { appendChild: (element) => appended.push(element) },
		},
		URL,
		Date,
	};
	vm.runInNewContext(body, context);
	const config = (window.dataLayer ?? []).map((args) => Array.from(args)).find(([command]) => command === 'config');
	return { config, appended };
}

test('analytics strips identifying query values and limits the referrer', async () => {
	const { config, appended } = await runAnalytics(
		'https://cwsweep.hidetaka.dev/guides/scan/?name=Hanako%20Yamada&utm_source=newsletter#top',
		'https://example.com/private?token=abc',
	);

	assert.equal(config[1], GA_MEASUREMENT_ID);
	assert.equal(config[2].page_location, 'https://cwsweep.hidetaka.dev/guides/scan/?utm_source=newsletter');
	assert.equal(config[2].page_referrer, 'https://example.com/');
	assert.equal(appended.length, 1);
	assert.equal(appended[0].src, `https://www.googletagmanager.com/gtag/js?id=${GA_MEASUREMENT_ID}`);
});

test('analytics keeps the path of a same-site referrer but not its query', async () => {
	const { config } = await runAnalytics(
		'https://cwsweep.hidetaka.dev/guides/clean/',
		'https://cwsweep.hidetaka.dev/guides/scan/?q=secret',
	);

	assert.equal(config[2].page_location, 'https://cwsweep.hidetaka.dev/guides/clean/');
	assert.equal(config[2].page_referrer, 'https://cwsweep.hidetaka.dev/guides/scan/');
});

test('analytics does nothing outside the production host', async () => {
	const { config, appended } = await runAnalytics('http://localhost:4321/guides/scan/', '');

	assert.equal(config, undefined);
	assert.equal(appended.length, 0);
});

test('Japanese privacy policy discloses the recipient, purpose, and opt-out', async () => {
	const html = await readPage('ja/privacy/index.html');

	assert.match(html, /Google LLC/);
	assert.match(html, /https:\/\/business\.safety\.google\/intl\/ja\/tba-jp\//);
	assert.match(html, /https:\/\/policies\.google\.com\/technologies\/partner-sites/);
	assert.match(html, /https:\/\/tools\.google\.com\/dlpage\/gaoptout/);
});

test('robots.txt points to the generated sitemap', async () => {
	const robots = await readFile(path.join(distDir, 'robots.txt'), 'utf8');
	const sitemap = robots.match(/^Sitemap: https:\/\/cwsweep\.hidetaka\.dev\/(\S+)$/m);

	assert.ok(sitemap, 'robots.txt must declare the sitemap');
	await access(path.join(distDir, sitemap[1]));
});

test('the abridged llms-small.txt leaves out the privacy policy', async () => {
	for (const file of ['llms.txt', 'llms-small.txt']) {
		const text = await readFile(path.join(distDir, file), 'utf8');
		assert.doesNotMatch(text, /privacy/i, `${file} must not include the privacy policy`);
	}
});
