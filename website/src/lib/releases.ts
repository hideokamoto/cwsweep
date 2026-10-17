// GitHub Releases の一覧を、ビルド時に GitHub REST API から取得する。
// リリースノートの単一の情報源は GitHub Release（semantic-release がコミットメッセージから生成する）で、
// サイト側にコピーは持たない。新しいリリースは .github/workflows/deploy-website.yml の
// `release: published` で再ビルドして反映する。

const REPOSITORY = 'hideokamoto/cwsweep';
const PER_PAGE = 100;

export interface ReleaseAsset {
	name: string;
	url: string;
	size: number;
}

export interface Release {
	tag: string;
	name: string;
	url: string;
	publishedAt: string;
	prerelease: boolean;
	/** GitHub Release の本文（Markdown）。 */
	body: string;
	assets: ReleaseAsset[];
}

interface ApiRelease {
	tag_name: string;
	name: string | null;
	html_url: string;
	published_at: string | null;
	draft: boolean;
	prerelease: boolean;
	body: string | null;
	assets: { name: string; browser_download_url: string; size: number }[];
}

/**
 * 公開済み（draft 以外）のリリースを新しい順に返す。
 *
 * API の失敗（レート制限を含む）はビルドを失敗させる。空のリリース一覧を黙って配信しないため。
 * 環境変数 GITHUB_TOKEN があれば認証付きで呼ぶ（未認証は 60 リクエスト/時の制限がある）。
 */
export async function fetchReleases(): Promise<Release[]> {
	const headers: Record<string, string> = {
		Accept: 'application/vnd.github+json',
		'X-GitHub-Api-Version': '2022-11-28',
	};
	const token = process.env.GITHUB_TOKEN;
	if (token) headers.Authorization = `Bearer ${token}`;

	const releases: Release[] = [];
	for (let page = 1; ; page++) {
		const url = `https://api.github.com/repos/${REPOSITORY}/releases?per_page=${PER_PAGE}&page=${page}`;
		const res = await fetch(url, { headers });
		if (!res.ok) {
			throw new Error(`GitHub Releases API failed: ${res.status} ${res.statusText} (${url})`);
		}
		const items = (await res.json()) as ApiRelease[];
		for (const item of items) {
			if (item.draft || !item.published_at) continue;
			releases.push({
				tag: item.tag_name,
				name: item.name || item.tag_name,
				url: item.html_url,
				publishedAt: item.published_at,
				prerelease: item.prerelease,
				body: item.body ?? '',
				assets: item.assets.map((a) => ({ name: a.name, url: a.browser_download_url, size: a.size })),
			});
		}
		if (items.length < PER_PAGE) break;
	}
	return releases.sort((a, b) => b.publishedAt.localeCompare(a.publishedAt));
}
