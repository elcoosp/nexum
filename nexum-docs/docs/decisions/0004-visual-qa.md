# ADR 0004 — Visual QA: every route × 2 viewports × 2 themes, screenshots inspected

**Status:** accepted (2026-11)

## Context

A green build proves pages compile, not that they *work* or *look right*. The quality bar for this site requires rendering evidence: no broken images, no horizontal overflow, exactly one `<h1>`, non-empty `<main>`, correct theme application, and no console/page errors — plus a human eyeball pass for visual defects automation cannot judge.

## Decision

- `playwright` (dev dependency) drives `scripts/qa-screenshots.mjs`, which enumerates **every** `index.html` in `dist/` (no hand-maintained route list), and for each route × viewport (1440×900 desktop, 390×844 mobile) × theme (light, dark) captures a full-page screenshot into `.qa/screenshots/` and asserts:
  - theme actually applied (`data-theme` on `<html>`, with toggle fallback),
  - exactly one `<h1>`,
  - `<main>` text length ≥ 200,
  - no horizontal overflow,
  - no broken `<img>`,
  - no console errors, page errors, or HTTP ≥ 400 responses.
- `pnpm qa` runs against `pnpm preview` (production build), not the dev server. Exit code 0 is a release blocker.
- Screenshots are then **inspected by eye** (home page, one page per Diátaxis section, longest page; both themes and viewports) and the outcome recorded per route in `docs/qa-report.md`.
- `.qa/` is git-ignored and excluded from the deliverable archive.

## Consequences

- Any content, config, CSS, logo or theme change invalidates the QA pass — re-run build → preview → qa before shipping.
- Optional hardening (`@axe-core/playwright` critical/serious gate) is deliberately deferred; the automated core checks plus the mandatory eyeball pass cover the current risk surface.
