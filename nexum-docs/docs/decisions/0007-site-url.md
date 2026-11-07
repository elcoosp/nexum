# ADR 0007 — Site URL: required by Starlight 0.42's built-in sitemap

**Status:** accepted (2026-11)

## Context

Starlight 0.42 registers a built-in `@astrojs/sitemap` integration automatically (when no other sitemap integration exists). That integration emits a build warning — and skips sitemap generation — unless `site` is set in the Astro config. Since the zero-warning build gate is a release blocker, `site` is no longer optional for this Starlight version, notwithstanding the general guidance to avoid `site` unless the deployment requires it.

## Decision

- `site: 'https://elcoosp.github.io/nexum'` — the GitHub Pages project-site origin, chosen as the most common zero-infrastructure hosting path for an OSS repository of this shape. It is the **single deploy-specific setting** in this codebase.
- `base` is deliberately **unset** (root deployment). Root-relative internal links (`/reference/core/`) then work identically in Starlight markdown, `<LinkCard>` component hrefs, and `starlight-links-validator` validation. Deploying under a subpath later requires adding `base` *and* re-running the full QA pass, because component-level hrefs must then be authored base-prefixed.

When the documentation moves to its final origin, update `site` in `astro.config.mjs`, keep the constant in `scripts/generate-llms.mjs` in sync, and re-run the full QA pass.

## Consequences

- Sitemap generation works warning-free.
- `scripts/generate-llms.mjs` emits absolute URLs based on the same origin.
