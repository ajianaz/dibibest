# dibibest

**Convert database schemas bolak-balik** — DBML (dbdiagram.io) ⇄ SQL DDL, dengan
**laporan fidelity** di setiap konversi: apa yang selamat, apa yang bocor.

dbdiagram memberimu design-first; Liam ERD memberimu reverse-first. **dibibest memberimu
keduanya dalam satu IR** — dan mengukur sendiri apakah bolak-baliknya lossless.

```bash
# DBML → SQL
cat schema.dbml | dibibest convert --to sql

# SQL → DBML (auto-detect source format)
cat schema.sql | dibibest convert --to dbml

# from your agent (MCP)
dibibest-mcp   # stdio server: initialize / tools/list / tools/call
```

```
-- fidelity --
  tables   3/3 ✓
  columns 11/11 ✓
  refs     2/2 ✓
  indexes  1/2 ⚠ DROPPED
  round-trip in source format: lossless ✓
```

## Kenapa fidelity report?

Konversi schema itu jujur atau tidak. Tool lain diam-diam membuang partial index,
enum, atau note. dibibest **selalu mengukur**: setiap konversi mem-parse ulang outputnya
sendiri dan membandingkan dengan IR sumber — angka `kept/total` per kategori, plus cek
idempotensi (`parse(emit(parse(src))) == ir`) di format sumber. "Bolak-balik" bukan
slogan; itu test yang jalan di setiap konversi dan di CI.

## Status (v0.1.0 scaffold)

| Format | Parse | Emit | Catatan |
|---|---|---|---|
| DBML (dbdiagram) | ✓ | ✓ | subset: Table/Column+settings, indexes (incl. expression), Ref, Enum, Project |
| SQL DDL | ✓ | ✓ | PG-flavored via sqlparser-rs; CREATE TABLE/INDEX + FK |
| Prisma | – | – | R4 (prisma-engines adalah Rust — jalan pintas sudah ditemukan) |
| ERD-JSON | – | – | R3 (kontrak kanvas Svelte) |
| Live-DB introspect | – | – | R5 |

Roadmap lengkap: [docs/ROADMAP.md](docs/ROADMAP.md) · Desain arsitektur:
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)

## Development

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace          # round-trip conformance ada di sini
cargo run -p dibibest-cli -- --help
```

## MCP (AI-agent access)

```json
{ "mcpServers": { "dibibest": { "command": "dibibest-mcp" } } }
```

Tools: `convert_schema` (source, from?, to) → output + fidelity; `list_formats`.
Manifest: [server.json](server.json) · published to the MCP registry at R5.

## License

MIT — see [LICENSE](LICENSE).
