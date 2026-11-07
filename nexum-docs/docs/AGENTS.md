# AGENTS.md — nexum documentation site

Working conventions for humans and AI agents maintaining this Astro Starlight documentation site for the [nexum](https://github.com/elcoosp/nexum) Rust workspace (framework-agnostic deep linking).

## Stack

- Astro 7.4.7 + @astrojs/starlight 0.42.10 (version floor — do not downgrade).
- TypeScript **6.x** (6.0.3) — `astro check` does not support TypeScript 7; see `docs/decisions/0001-typescript-version.md`.
- pnpm is the package manager of record. No npm/yarn lockfiles.
- Pagefind search (Starlight default, enabled), Shiki syntax highlighting via Expressive Code (default — do not introduce another highlighter).
- Plugins: `starlight-links-validator` (fails the build on broken internal links), `starlight-image-zoom`, `starlight-heading-badges`. Omitted plugins and reasons: `docs/decisions/0006-omitted-plugins.md`.

## Commands

```bash
pnpm install       # install deps
pnpm dev           # dev server
pnpm build         # astro build + llms.txt generation (must be warning-free)
pnpm check         # astro check — 0 errors, 0 warnings required
pnpm lint          # markdownlint-cli2 over src/content/docs
pnpm preview       # serve dist/ on :4321
pnpm qa            # playwright screenshots, every route × 2 viewports × 2 themes
```

## Information architecture (Diátaxis)

One page type per page; do not mix:

| Type | Folder | Frontmatter `type` |
| --- | --- | --- |
| Tutorial | `src/content/docs/tutorials/` | `tutorial` |
| How-to guide | `src/content/docs/guides/` | `how-to` |
| Reference | `src/content/docs/reference/` | `reference` |
| Explanation | `src/content/docs/explanation/` | `explanation` |

- Every page carries `title`, `description`, `type` frontmatter (`type` is a custom schema field defined in `src/content.config.ts`; the landing page uses `overview`).
- The sidebar in `astro.config.mjs` is explicit — add new pages there in the correct group.
- Landing page (`index.mdx`) is the Diátaxis hub: hero + `CardGrid` pathfinding.

## Writing rules

- Second person, present tense, active voice; imperative for instructions.
- Never write "simply", "just", "easy", "obviously".
- Every page: one-sentence summary in `description`, prerequisites when relevant, **Next steps** via `<LinkCard>`.
- All code fences declare a language (`rust`, `toml`, `bash`, `ini`, `xml`, `text`).
- Snippets must be verifiable against the nexum sources — never invent flags, options or behaviors. Source of truth: the workspace inventory in this repository's maintainer notes and the crates' doc comments.
- MDX gotchas: files using Starlight components (`Card`, `Tabs`, `LinkCard`, `Aside`, `FileTree`, …) must use the `.mdx` extension; escape `<` and `{` in prose with code spans.

## Brand

- Accent derived from the logo's electric blue `#0a4be8` (hsl 222 92% 48%); see `src/styles/custom.css` and `docs/decisions/0002-logo-format.md`.
- Logo: `src/assets/logo-light.svg` / `logo-dark.svg` (traced from the original `docs/logo.png` render, SVGO-optimized with viewBox preserved). The original PNG render lives in `src/assets/hero-logo.png` for the landing showcase.
- Dark mode must stay fully functional — never remove the theme toggle.

## Quality gates (release blockers)

1. `pnpm build` exits 0 with **zero warning lines**.
2. `pnpm check` reports **0 errors, 0 warnings**.
3. `pnpm lint` is clean.
4. `pnpm qa` exits 0 across every built route, both viewports, both themes.
5. Eyeball pass recorded in `docs/qa-report.md` (per route × light/dark × desktop/mobile).
6. Re-run all gates after any change to config, CSS, logo, theme or content, immediately before packaging.

`.qa/` is scratch output and must never be committed or shipped.
