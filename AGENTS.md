# AGENTS.md — dibibest

Guidance for AI coding agents working in this repository. Humans: most of this
applies to you too.

## Project

dibibest converts database schemas bidirectionally (DBML ⇄ SQL today; Prisma and
ERD-JSON on the roadmap) and reports conversion fidelity. The whole product
promise is round-trip losslessness — treat fidelity regressions as release
blockers, not bugs.

## Layout

```
crates/dibibest-core   # IR + format codecs + registry (pure, no I/O — WASM-safe)
crates/dibibest-cli    # CLI (stdin/file → stdout; fidelity to stderr)
crates/dibibest-mcp    # MCP stdio server (initialize/tools.list/tools.call)
docs/                  # ROADMAP, ARCHITECTURE
```

## Non-negotiables

1. **Every conversion carries a fidelity report.** A codec change without a
   round-trip test update is incomplete.
2. **Round-trip conformance:** `parse(emit(parse(src))) == parse(src)` must hold
   for every format's own format. Enforced in
   `crates/dibibest-core/tests/roundtrip.rs`.
3. **Registry is the only seam.** Formats never import each other; adding a
   format = one new `impl Codec` + one `register()` call + tests. If your change
   edits another codec, stop.
4. **Core stays pure** (no I/O, no network, no clock) so it compiles to WASM.
5. `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings &&
   cargo test --workspace` must pass before any push.
6. **Commit identity (owner standard 2026-10-16):** all commits use
   `ajianaz <ajianaz@users.noreply.github.com>` — never a personal email, and
   **no AI attribution** (no `Co-Authored-By: …Claude…` trailers; Claude Code
   runs with `includeCoAuthoredBy=false`). Squash merges inherit the PR author's
   GitHub privacy setting — keep "Keep my email addresses private" ON.

## Branching & PRs (CodeCora standard)

- Default branch: `develop`. PRs target `develop`; `main` only receives
  `develop` (or `chore/release-*`) via PR — enforced by `source-branch-check`.
- Branch names: `feat/`, `fix/`, `docs/`, `chore/`, `refactor/`, `test/`, `ci/`.
- Conventional Commits for PR titles: `feat(core): …`.
- PR description must contain `## What`, `## Why`, `## Testing`.
- CI runs fmt/clippy/test + Cora AI review; address review findings before merge
  (CI green ≠ review-clean — read the Cora comment; "Review could not complete"
  is a blocker, re-run until a real verdict). Self-PR verdicts are posted as
  structured comments (GitHub forbids self-approval).
- Dependency bots (dependabot/renovate/github-actions) have their
  PRs triaged by maintainers; breaking bumps (e.g. sqlparser major) require
  a hand-migration PR that supersedes the bot's.

## Language

Git/GitHub artifacts (commits, PRs, issues, code comments) in English.
