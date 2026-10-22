# Architecture

The hybrid-registry design (full rationale: design-twice comparison, 22 Oct 2026 —
3 interface variants evaluated on depth/leverage/locality; recommendation: registry
inside, two-function API outside).

```
                 ┌── Codec: DbmlCodec        (parse/emit DBML)
  SchemaIR ◄────►├── Codec: SqlCodec         (parse/emit SQL DDL via sqlparser-rs)
    (hub)        ├── Codec: PrismaCodec      (R4 — prisma-engines is Rust)
                 └── Codec: ErdJsonCodec     (R3 — canvas contract)

  Registry (owns codecs)  ── public API ──►  convert(src, to)
                                          convert_explicit(src, from, to)
                                          → Conversion { output, Fidelity }
```

## Decisions

1. **IR hub, not pairwise converters.** N formats = 2N codecs, not N² paths.
   Adding a format never touches another one (kill-test: Drizzle = 1 file +
   1 register line, zero edits elsewhere).
2. **Fidelity is measured, not claimed.** `convert_explicit` re-parses its own
   output and diffs counts against the source IR (`tables/columns/refs/indexes
   kept/total`), plus an idempotency check in the source format
   (`parse(emit(parse(src))) == parse(src)`). Both ship in `Conversion` and are
   tested in CI.
3. **Core is pure.** No I/O in `dibibest-core` — it compiles to WASM (R3 canvas)
   and links into the CLI and MCP server unchanged.
4. **Codec trait = the seam.** Two real adapters exist per format slot today
   (production codec + conformance fixtures); community codecs plug in without
   core changes later.
5. **MCP is a thin layer** over the same core; no second conversion
   implementation exists anywhere in the repo.

6. **ERD-JSON is the canvas contract.** The `erdjson` codec emits deterministic
   render data (nodes/edges/meta, see `docs/erdjson-schema.md`); layout runs as a
   separate step (`erdjson::apply_layout`) so snapshots stay stable.

## Known scaffold limitations (tracked)

- DBML subset (Project/Table/Ref/Enum/indexes+expressions/settings). Unknown
  settings are ignored on parse today — corpus conformance (R1) will switch this
  to preserve-as-comment so round-trips stay lossless on unknowns.
- SQL emit is PG-flavored; dialect selection arrives with R2.
- ObjectName quoting is minimal; identifier edge cases belong to the R1 corpus.
