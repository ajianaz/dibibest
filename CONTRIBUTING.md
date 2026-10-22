# Contributing to dibibest

Thanks for your interest in contributing! 🎉

## Getting started

```bash
git clone https://github.com/ajianaz/dibibest.git
cd dibibest
cargo test --workspace        # everything must stay green
```

## Ground rules (enforced by CI + review)

1. **Round-trip fidelity is the product.** If your change touches a codec, it
   must update `crates/dibibest-core/tests/roundtrip.rs` — the idempotency
   invariant (`parse(emit(parse(x))) == parse(x)`) may not regress.
2. **The registry is the only seam.** New format = one new `impl Codec` file +
   registration. Formats never import each other.
3. **Core stays pure** — no I/O, no network, no clock in `dibibest-core`.
4. Branch from `develop`, PR back into `develop`. Branch names:
   `feat/…`, `fix/…`, `docs/…`, `chore/…`, `test/…`, `ci/…`.
5. PR titles follow Conventional Commits (`feat(core): …`). PR bodies need
   `## What`, `## Why`, `## Testing`.
6. Before pushing:
   ```bash
   cargo fmt --all
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```

## CLA

This project uses a Contributor License Agreement. The `cla-check` CI job
verifies your signature against the org's signatures file on your first PR —
you'll get a link to sign if it's missing. Contributions are made under the
project's MIT license.

## No compensation

dibibest is free open-source; there is no payment or bounty for contributions.

## AI agents

`AGENTS.md` carries the repo conventions for AI coding agents (same rules as
above, plus registry/seam discipline details).
