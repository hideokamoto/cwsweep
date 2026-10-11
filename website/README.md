# cwsweep website

https://cwsweep.hidetaka.dev のソース（Astro Starlight）。

```bash
npm ci
npm run dev     # http://localhost:4321
npm run build   # dist/ に出力
```

ページは `src/content/docs/` 配下の Markdown。`main` への push（`website/` 配下の変更）で
`.github/workflows/deploy-website.yml` が GitHub Pages へデプロイする。
