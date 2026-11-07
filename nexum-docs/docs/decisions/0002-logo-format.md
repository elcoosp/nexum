# ADR 0002 — Logo format: trace the raster render, keep the original as hero art

**Status:** accepted (2026-11)

## Context

The nexum repository ships exactly one logo asset: `docs/logo.png`, a 1024×1024 RGBA render of the "N" mark — an electric-blue ribbon interwoven with a near-black stroke, with the glow baked as semi-transparent pixels and an otherwise fully transparent canvas (84 % of pixels have alpha 0). No vector original exists anywhere in the repository. This is a **logo-quality risk**: a raster source cannot scale losslessly and would pixelate in close-up use if treated carelessly. Never upscale the PNG.

Per the branding pipeline, the options were: (a) trace to SVG, or (b) ship @2x/@3x raster variants. Never upscale the PNG.

## Decision

1. **Trace the mark to a flat SVG** (`src/assets/logo-light.svg`). Pipeline: threshold the two glyph color layers from the render (blue `#0a4be8`, dark `#070a12`), build the union silhouette, fill interior holes, classify glyph pixels per layer, trace with marching squares (scikit-image) and simplify. The SVG keeps `viewBox="0 0 1024 1024"`.
2. **Optimize with SVGO** (`svgo.config.js`, multipass). SVGO 4 no longer includes `removeViewBox` in `preset-default` and preserves `viewBox` by default — the config documents this; the optimized outputs were verified to still carry the viewBox.
3. **Dark-mode variant** (`src/assets/logo-dark.svg`): identical geometry, near-black accent layer recolored to a light tone (`#c7cfdd`) so the weave stays visible on dark backgrounds.
4. **Favicon** (`public/favicon.svg`): silhouette-only mark in brand blue — legible on both browser tab themes.
5. **Original render preserved** as `src/assets/hero-logo.png` and used as the landing-page showcase (centered), where the glow artwork shows at full quality on both themes. The glow and shading effects are intentionally absent from the flat header mark.

The `experimental.svgOptimizer` Astro flag was **not** enabled: the docs config avoids experimental Astro flags by guardrail, and every committed SVG is already SVGO-optimized by step 2. The flag is a build-time safety net, not a replacement for optimized committed assets.

## Consequences

- The header logo is vector: sharp at any zoom, no pixelation in 2× QA.
- The flat trace intentionally simplifies the artwork; the hero showcase keeps the original render visible.
- If upstream publishes a true vector original, replace the traced files and re-run the QA pass.
