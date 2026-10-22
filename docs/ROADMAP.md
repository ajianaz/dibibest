# Roadmap

Milestone-driven. Each milestone has explicit exit criteria; a milestone that
cannot meet them is renegotiated, not silently extended.

## M0 — Scaffold (this release)
- [x] Workspace: core / cli / mcp crates
- [x] DBML codec (subset) + SQL codec, round-trip conformance tests
- [x] Registry + fidelity-first public API
- [x] MCP stdio server (initialize, tools/list, tools/call)
- Exit: `cargo test` green incl. idempotency; CI green; Cora review clean.

## M1 — R1: DBML conformance corpus
- Build a corpus from real-world DBML (dbdiagram docs, OSS `schema.rb`-derived,
  community samples). Parser accepted only when corpus round-trips ≥95% byte-
  -identical after canonicalization (unknown settings preserved as comments).
- Exit: corpus CI job green; gaps documented in `docs/dbml-gaps.md`.

## M2 — R2: SQL dialects + Prisma
- MySQL + SQLite emit (parse already via sqlparser). PG ENUM TYPES.
- Prisma schema codec via prisma-engines components (Rust, Apache-2.0).
- Exit: DBML→Prisma→DBML idempotent on corpus; dialect matrix documented.

## M3 — R3: ERD-JSON + canvas (Svelte 5)
- ERD-JSON codec; Svelte 5 + svelte-flow canvas (read-only first), edit via
  DBML text panel; export PNG/SVG.
- Exit: paste DBML → render → export; fidelity chip reflects real report.

## M4 — R5: MCP registry + hardening
- Publish server.json to MCP registry; fuzz parse (cargo-fuzz) on both codecs;
  structured error catalog.
- Exit: MCP conformance smoke; fuzz clean 1h run; 0.2.0 release.

## Non-goals (v1)
Real-time collaboration · AI schema design (Liam territory) · migration diffing ·
desktop apps.
