# ADR 0003 — Syntax highlighting: Shiki via Expressive Code (Starlight default)

**Status:** accepted (2026-11)

## Context

Starlight renders code blocks through Expressive Code, whose supported highlighter is **Shiki** (VS Code TextMate grammars, 180+ languages, build-time execution, zero client-side JavaScript, dual light/dark themes, transformer support). A lightweight alternative (e.g. Rangi, ~13 kB, regex tokenizer) exists for chat UIs and small apps, but is explicitly *not* positioned for documentation sites: it lacks TextMate grammars, transformers, Twoslash, and any Expressive Code integration.

## Decision

Ship with the Starlight default — **Shiki through Expressive Code**. No custom highlighter plugin is introduced and no `expressiveCode` override is configured: the default themes already integrate with the light/dark toggle, and the project needs no language whitelist (Rust, TOML, bash, ini, xml, text are all covered by the default grammar set).

## Consequences

- Code blocks get token-accurate highlighting, line highlighting, word wrap and the built-in copy button with zero configuration.
- Every fenced block in the content declares a language tag (enforced by review and the QA eyeball pass).
- Revisit only if a future requirement needs a non-default theme or bundled-language tuning — then add a narrow `expressiveCode` config block.
