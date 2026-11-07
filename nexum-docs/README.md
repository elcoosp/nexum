# nexum-docs — documentation site for [nexum](https://github.com/elcoosp/nexum)

Production-grade Astro **Starlight** documentation for the nexum Rust workspace (framework-agnostic deep linking), organized after the Diátaxis framework.

## Stack

- Astro 7.4.7 · @astrojs/starlight 0.42.10 · TypeScript 6.0.3 · pnpm
- Pagefind search (built-in), Shiki syntax highlighting via Expressive Code
- Plugins: starlight-links-validator, starlight-image-zoom, starlight-heading-badges

## Commands

```bash
pnpm install
pnpm dev       # dev server
pnpm build     # production build + llms.txt / llms-full.txt generation
pnpm check     # astro check (must be 0 errors / 0 warnings)
pnpm lint      # markdownlint over src/content/docs
pnpm preview   # serve dist/ on :4321
pnpm qa        # playwright screenshots — every route × 2 viewports × 2 themes
```

## Layout

- `src/content/docs/` — Diátaxis content: `tutorials/`, `guides/`, `reference/`, `explanation/` (frontmatter `type` field per page)
- `src/assets/` — traced SVG logo variants (light/dark) + the original brand render for the landing showcase
- `src/styles/custom.css` — brand accent derived from the logo (electric blue hsl 222 92% 48%)
- `scripts/` — QA screenshot suite (`qa-screenshots.mjs`), llms.txt generator, detail captures
- `docs/` — `AGENTS.md` (maintenance conventions), `qa-report.md` (per-route visual QA), `decisions/` (ADRs)

## Deploy note

`site` in `astro.config.mjs` is the single deploy-specific setting (required by Starlight 0.42's built-in sitemap). Update it — and the constant in `scripts/generate-llms.mjs` — when the docs move to their final origin, then re-run the full gate suite (`build` → `check` → `lint` → `preview` → `qa`).
