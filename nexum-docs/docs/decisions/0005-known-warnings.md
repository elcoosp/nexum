# ADR 0005 — Known warnings: none allowlisted

**Status:** accepted (2026-11)

## Context

The release gate requires `pnpm build`, `pnpm check` and `pnpm lint` to emit zero warning lines. The only permitted exception is a warning originating in a third-party dependency that cannot be fixed, allowlisted narrowly with its exact string and a justifying comment.

## Decision

**No warnings are allowlisted.** The final build/check/lint logs contain zero `warn`/`warning` lines and zero errors. Any future warning must either be fixed at the cause or, if it provably originates in an unfixable third-party dependency, allowlisted here with:

1. the exact line,
2. the reason it cannot be fixed locally,
3. a narrow `grep -v` match for that exact string in the gate scripts (blanket suppression remains forbidden).

## Consequences

- The zero-warning assertion runs against the raw logs; today it needs no filters at all.
- This file doubles as the changelog of accepted noise — it must stay empty by default.
