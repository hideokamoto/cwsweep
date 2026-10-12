# cwsweep website

https://cwsweep.hidetaka.dev のソース（Astro Starlight）。

```bash
npm ci
npm run dev     # http://localhost:4321
npm run build   # dist/ に出力
```

ページは `src/content/docs/` 配下の Markdown。英語（既定ロケール、`/`）は `src/content/docs/` 直下、
日本語（`/ja/`）は `src/content/docs/ja/` 配下に同じパス構成で置く。ページを追加・変更するときは両言語を揃えること
（片方しかないページは Starlight が既定言語の内容でフォールバック表示する）。
`main` への push（`website/` 配下の変更）で
`.github/workflows/deploy-website.yml` が GitHub Pages へデプロイする。

`/llms.txt` 系は `starlight-llms-txt` が生成する。`/skill.md` と `/.well-known/agent-skills/` は、リポジトリ直下の
`skills/` を `src/lib/skills.ts` がビルド時に読み込んで配信する（コピーは持たない。`website/` をカレントに
してビルドすること）。`skills/` の変更でもデプロイされるよう、ワークフローの `paths` に `skills/**` を含めている。
