# Visual QA Report — nexum documentation site

**Build under test:** final state of the repository (see git-less zip `nexum-docs.zip`).
**Gates:** `pnpm build` exit 0 / zero warning lines · `pnpm check` 0 errors, 0 warnings, 0 hints · `pnpm lint` 0 issues · `pnpm qa` exit 0 — **24 routes × 2 viewports (1440×900, 390×844) × 2 themes (light, dark) = 96 probe runs, 96 screenshots** plus a 404 capture.
**Method:** every screenshot produced by `scripts/qa-screenshots.mjs` was eyeballed via full-page montages (landing, tutorial, installation guide, architecture, core reference, single-instance guide) and dedicated zoom captures (header logo at deviceScaleFactor 3 in both themes, search modal, dark code block, pagination, mobile tables).

## Per-route results

Legend: ✓ = inspected, no defect. The automated probe asserted, per route and combination: theme applied, exactly one `<h1>`, `<main>` text ≥ 200 chars, no horizontal overflow, no broken images, no console/page errors, no HTTP ≥ 400.

| Route | Desktop light | Desktop dark | Mobile light | Mobile dark | Notes |
| --- | --- | --- | --- | --- | --- |
| `/` (landing) | ✓ | ✓ | ✓ | ✓ | Hero actions, CardGrid, tables, showcase image render; dark mode has no white panels |
| `/tutorials/getting-started/` | ✓ | ✓ | ✓ | ✓ | Steps, alerts, syntax-highlighted Rust/bash blocks; active sidebar state correct |
| `/guides/installation/` | ✓ | ✓ | ✓ | ✓ | Tabs (Facade / Core+platform / Framework adapter) switch correctly |
| `/guides/choosing-an-adapter/` | ✓ | ✓ | ✓ | ✓ | Decision table + per-framework snippets |
| `/guides/macos-setup/` | ✓ | ✓ | ✓ | ✓ | CAUTION alert renders; plist snippets highlighted |
| `/guides/windows-setup/` | ✓ | ✓ | ✓ | ✓ | Registry table fits and wraps cleanly |
| `/guides/linux-setup/` | ✓ | ✓ | ✓ | ✓ | INI block and template table legible |
| `/guides/single-instance/` | ✓ | ✓ | ✓ | ✓ | Transport table (Unix/Windows) scrolls on mobile |
| `/guides/bounded-hub/` | ✓ | ✓ | ✓ | ✓ | Verified test snippet highlighted |
| `/guides/async-consumption/` | ✓ | ✓ | ✓ | ✓ | Signature table + stream snippet |
| `/guides/mobile-deep-links/` | ✓ | ✓ | ✓ | ✓ | Generated intent-filter XML block renders |
| `/reference/nexum/` | ✓ | ✓ | ✓ | ✓ | Re-export + feature tables |
| `/reference/core/` | ✓ | ✓ | ✓ | ✓ | Longest page; all signature tables fit/scroll, TOC highlights current heading |
| `/reference/errors/` | ✓ | ✓ | ✓ | ✓ | Error variant table with exact messages |
| `/reference/platform/` | ✓ | ✓ | ✓ | ✓ | Backend matrix + module tables; pagination Previous/Next verified correct |
| `/reference/macros/` | ✓ | ✓ | ✓ | ✓ | Syntax table + generated-code block |
| `/reference/adapters/floem/` | ✓ | ✓ | ✓ | ✓ | Signal feature table |
| `/reference/adapters/xilem/` | ✓ | ✓ | ✓ | ✓ | |
| `/reference/adapters/dioxus/` | ✓ | ✓ | ✓ | ✓ | |
| `/reference/adapters/leptos/` | ✓ | ✓ | ✓ | ✓ | MSRV aside renders |
| `/reference/adapters/gpui/` | ✓ | ✓ | ✓ | ✓ | |
| `/explanation/architecture/` | ✓ | ✓ | ✓ | ✓ | FileTree workspace layout renders |
| `/explanation/event-sources/` | ✓ | ✓ | ✓ | ✓ | macOS nuance CAUTION aside |
| `/explanation/design-principles/` | ✓ | ✓ | ✓ | ✓ | Quality-bar table |
| `404` (one capture) | ✓ | n/a | n/a | n/a | Starlight 404 route with custom draft entry; helpful links present |

## Detail checks (zoom captures)

- **Logo sharpness (3× zoom, both themes)** — vector mark edges crisp, no pixelation, no halo, no clipping; dark variant swaps the near-black accent layer for a light tone so the weave stays visible on dark backgrounds. `viewBox` preserved (verified in optimized SVG and rendered output).
- **Accent color** — links, active sidebar item, focus states and the hero button use the derived brand blue (hsl 222 92% 48% light / 60% dark), visibly not Starlight's default blue; WCAG AA contrast holds in both themes (light accent on white ≈ 6.6:1; dark accent-high ≈ 9:1).
- **Code blocks** — multiple token colors present (Shiki via Expressive Code), copy button present, blocks scroll horizontally instead of overflowing the page.
- **Search** — Pagefind modal opens from the header button, returns 24 sensible results for the domain query "deep link", with titles, section anchors and excerpts.
- **Tables on mobile** — Starlight's native pattern (`display: block; overflow-x: auto`): tables scroll horizontally inside their block instead of overflowing the page; no column is lost, only reachable by scroll. Accepted as the framework's standard behavior.
- **Mobile navigation** — hamburger menu present, content does not clip or overlap, tap targets usable.
- **Dark mode** — no pure-white panels, no invisible text, code-block background distinct from page background.

## Defects found and fixed during this pass

1. **Landing showcase sizing/alignment** — the original brand render was displayed left-aligned at 360px; centered via `.nexum-showcase` and increased to 480px. Fixed and re-verified with a full QA re-run.
2. **Suspected mobile table defect** — large row gaps on `/reference/core/` mobile were investigated at DOM level (row heights 129/185px traced to the wrapped `Description` column inside Starlight's scrollable-table pattern). Confirmed as standard framework behavior, not a defect; no change made.

## Re-run rule

The final state of the repository passed all gates **as the last action before packaging**: after the showcase/CSS fix, `pnpm build`, `pnpm check`, `pnpm lint` and `pnpm qa` were all re-run and green. `.qa/` output is excluded from the deliverable archive.
