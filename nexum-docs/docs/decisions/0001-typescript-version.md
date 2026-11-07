# ADR 0001 — Pin TypeScript to 6.x for astro check

**Status:** accepted (2026-11)

## Context

As of Astro 7.4, `astro check` does not support TypeScript 7. Running it against a TS 7 toolchain fails with:

> astro check does not currently support TypeScript 7.0

An experimental workaround exists (`tsc` + `@astrojs/ts-content-mapper`), but it is not the stable default and is not appropriate for a docs site that must pass a strict zero-error, zero-warning gate.

## Decision

Pin `typescript` to the latest 6.x release — **6.0.3** — as a dev dependency. The pin is explicit in `package.json` so any install (including fresh pnpm setups for this deliverable) resolves a compatible toolchain.

## Consequences

- `pnpm check` runs on a supported TypeScript and must report 0 errors / 0 warnings.
- Upgrading to TypeScript 7 becomes possible only when Astro ships support, or if the experimental path is explicitly verified. Until then, Dependabot-style bumps of `typescript` beyond 6.x must be rejected.
