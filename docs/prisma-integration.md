# Prisma integration (issue #8)

## Decision

**Hand-written `.prisma` parser + emitter in `dibibest-core` (`src/prisma.rs`), no new
dependencies.** `prisma-engines` is not used, not even behind a feature flag.

## Investigation

| Option | Finding |
|---|---|
| (a) Small standalone crate | `cargo search` (22 Oct 2026): crates.io `psl` is the *Public Suffix List* crate, unrelated to Prisma. No first-party Prisma schema crate is published. The only hit, `saola-psl-core 0.1.0`, is a third-party extraction by a single author for another ORM — an unvetted, pre-1.0 dependency for the product's core promise. |
| (b) prisma-engines crates (`psl`, `schema-ast`, `parser-database`, …) as git/path deps | Those crates are workspace members that depend on each other by path and are **not published to crates.io**, so they would have to be git dependencies pinned to a commit of a monorepo whose workspace also carries the query/schema engines and DB drivers. Not verified here by building it: the weight/MSRV/lockfile cost was judged from the repo layout, not measured. Even in the best case the parser yields Prisma's validated, dialect-aware DML — a second schema model that we would still have to map to our IR in both directions — and `psl` validates against a connector (it rejects schemas the target database cannot hold), which is the opposite of a pure, WASM-safe codec. |
| (c) Hand-write | The subset needed for schema conversion (models, fields, attributes, enums, relations, comments) is small and regular. ~1k lines (rustfmt-expanded), zero deps, WASM-safe, and it plugs into the same `Codec` seam as DBML/SQL. |

(a) and (b) fail non-negotiable #4 (core stays pure / WASM-safe) or add an unvetted
dependency; (c) keeps the registry contract and costs one file.

### License notes

No Prisma code is vendored or copied; the grammar was implemented from the public
Prisma schema reference. `prisma-engines` is Apache-2.0, so a future switch (or a
test-only oracle, e.g. running `prisma validate` in CI) stays license-compatible. If
we later add a `prisma` feature wrapping `psl`, it must be optional and off by default.

## Mapping

* model ⇄ table (`@@map` keeps names that are not Prisma identifiers, e.g. `core.tenants`),
  field ⇄ column (`@map`), enum ⇄ enum, `/// doc` ⇄ notes.
* Types: DBML/SQL type → Prisma scalar + `@db.*` (`varchar(255)` → `String @db.VarChar(255)`);
  anything unmapped (`geography(Point, 4326)`, arrays of unmapped types) → `Unsupported("…")`.
  Reverse maps back; bare scalars use canonical names (`Int` → `integer`,
  `String` → `varchar`, `DateTime` → `timestamp`, `Json` → `jsonb`), so the *first* trip can
  re-spell a type (`int` → `integer`) and every later trip is stable.
* Defaults: `now()`/`CURRENT_TIMESTAMP` → `now()`, literals stay literals, plain strings are
  quoted, everything else → `dbgenerated("…")`. `serial`/`bigserial` → `autoincrement()`.
* Relations: IR `Ref` ⇄ the FK-side `@relation(fields, references)` field plus the generated
  back-reference field. Cardinality is derived from uniqueness of the FK columns (unique/pk →
  1-1, else 1-n); `<` refs are re-oriented to the FK side. A relation name is emitted only
  when a ref has a user name or the table pair is ambiguous/self-referencing.
* Composite pk → `@@id`, `@@unique`/`@@index` with `map:` carrying the index name.

## Fidelity (what the report shows)

* **Expression / partial indexes are dropped.** `Caps::expr_indexes = false` for Prisma
  (true for DBML and SQL); the registry reports
  `N expression/partial index(es) dropped … : table.index, …`, and the `indexes` counts show
  the loss. DBML stays the lossless home.
* Not representable → preserved as `// unknown:` comments and re-read on parse: unrecognised
  settings, `Ref` settings such as `delete: cascade` (not mapped to `onDelete`), the DBML
  `Project` block (`// project:`), composite-`pk` index markers.
* Not parsed: `@@fulltext`-style block attributes (kept as `// unknown:`), implicit
  many-to-many (list-on-both-sides) relations, `type`/`view` blocks, `datasource`/`generator`
  contents. One-line blocks (`model A { id Int }`) are rejected with a clear error.
* Emitted schemas are not validated against Prisma (e.g. a model without `@id`, or
  `references` to a non-unique column, would be rejected by `prisma validate`).
* `timestamp`/`timestamptz` precision (Prisma defaults to `timestamp(3)`) is not modelled.

## Conformance

`tests/prisma_roundtrip.rs`: Prisma → IR → Prisma lossless and stable; DBML → Prisma →
DBML reaches a fixed point (second pass byte-identical to the first) on all 12 corpus files,
with tables/columns/refs/enums preserved; expression-index drop report.
