# Error catalog

Every `CoreError` maps to a **stable code** via `CoreError::code()`
(`crates/dibibest-core/src/lib.rs`). Codes never change meaning; new failure
classes get new codes. The MCP server prefixes tool errors with the code:
`error[unclosed-block]: parse error in dbml: …` (`isError: true`).

| Code | `CoreError` variant | Source | Meaning |
|---|---|---|---|
| `unbalanced-brace` | `Parse` (dbml) | dbml | Stray `}` with no open block |
| `unclosed-block` | `Parse` (dbml) | dbml | End of input inside a `Table`/`Enum`/`Project`/`Ref` block |
| `bad-column-line` | `Parse` (dbml) | dbml | Line inside a table that is not a valid column definition |
| `bad-index-line` | `Parse` (dbml) | dbml | Malformed entry inside `indexes { }` |
| `bad-ref-line` | `Parse` (dbml) | dbml | Malformed `Ref` (top-level, block, or inline `ref:`) |
| `unrecognized-statement` | `Parse` (dbml) | dbml | Unsupported construct: unknown top-level keyword, nested table block, table alias, multi-line `'''` note |
| `unknown-setting` | — (reserved) | dbml | Reserved. Unknown `[settings]` are **not** errors: they are preserved and surfaced through the fidelity report |
| `parse-error` | `Parse` (sql, prisma, erdjson) | codecs | Source rejected by the parser; the `format` field and message carry detail |
| `unknown-format` | `UnknownFormat` | registry | Requested `from`/`to` id is not registered |
| `detect-failed` | `UnknownFormat` | registry | Auto-detection found no format; pass `from` explicitly |
| `emit-error` | `Emit` | codecs | IR could not be rendered in the target format |
| `internal` | `Internal` | core | Invariant violation (e.g. serialization); a bug — please report |

Notes:

- `detect-failed` and `unknown-format` share one variant; detection is keyed on
  the message prefix `could not detect` (pinned by `error_codes_are_stable`).
- Codes are asserted in `crates/dibibest-core/src/lib.rs` (`error_codes_are_stable`)
  and the MCP path in `crates/dibibest-mcp/tests/conformance.rs`.
