# Conventional Commits + CodeCora sections. PR title is checked by CI
# (commit-conventional job); the body needs ## What / ## Why / ## Testing.

## What

<!-- description of the change -->

## Why

<!-- the problem you're solving; link issues with Closes #N when intended -->

## Testing

<!-- how you verified: cargo test, round-trip conformance, MCP smoke, … -->

## Checklist

- [ ] `cargo fmt --all` clean
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] `cargo test --workspace` green (round-trip conformance included)
- [ ] Fidelity report behavior unchanged or updated + tested
