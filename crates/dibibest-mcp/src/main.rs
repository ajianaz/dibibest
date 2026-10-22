//! Minimal MCP (Model Context Protocol) stdio server for dibibest.
//!
//! Implements the JSON-RPC 2.0 surface AI agents need: initialize,
//! tools/list, tools/call. No external MCP SDK dependency — the protocol
//! surface used here is small and stability matters more than features.
//! Roadmap (R5): resources/list for schema previews, official conformance run.

use dibibest_core::{detect_format, FormatId, Registry};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

fn main() -> std::io::Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let registry = Registry::with_defaults();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let Ok(req) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let id = req.get("id").cloned().unwrap_or(Value::Null);
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let resp = handle(method, req.get("params"), &registry);
        let mut out = stdout.lock();
        serde_json::to_writer(
            &mut out,
            &json!({ "jsonrpc": "2.0", "id": id, "result": resp }),
        )?;
        out.write_all(b"\n")?;
        out.flush()?;
    }
    Ok(())
}

fn handle(method: &str, params: Option<&Value>, registry: &Registry) -> Value {
    match method {
        "initialize" => json!({
            "protocolVersion": "2025-06-18",
            "capabilities": { "tools": {} },
            "serverInfo": {
                "name": "dibibest",
                "title": "dibibest — bidirectional schema conversion",
                "version": env!("CARGO_PKG_VERSION"),
            }
        }),
        "tools/list" => json!({
            "tools": [{
                "name": "convert_schema",
                "title": "Convert schema bolak-balik",
                "description": "Convert a database schema between formats (dbml, sql, prisma). \
                                ERD-JSON is on the roadmap. Returns the converted \
                                output plus a fidelity report of what survived.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "source": { "type": "string", "description": "Schema source text (DBML, SQL DDL or Prisma schema)" },
                        "from": { "type": "string", "description": "Source format id; auto-detected when omitted", "enum": ["dbml", "sql", "prisma"] },
                        "to": { "type": "string", "description": "Target format id", "enum": ["dbml", "sql", "prisma"] }
                    },
                    "required": ["source", "to"]
                }
            },
            {
                "name": "list_formats",
                "title": "List supported formats",
                "description": "List format ids the server can convert between.",
                "inputSchema": { "type": "object", "properties": {} }
            }]
        }),
        "tools/call" => {
            let p = params.cloned().unwrap_or(Value::Null);
            let name = p.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let args = p.get("arguments").cloned().unwrap_or(json!({}));
            tool_call(name, &args, registry)
        }
        other => json!({ "error": format!("unsupported method: {other}") }),
    }
}

fn tool_call(name: &str, args: &Value, registry: &Registry) -> Value {
    match name {
        "list_formats" => {
            json!({ "content": [{ "type": "text", "text": registry.ids().join("\n") }] })
        }
        "convert_schema" => {
            let src = args.get("source").and_then(|s| s.as_str()).unwrap_or("");
            let to = args.get("to").and_then(|s| s.as_str()).unwrap_or("");
            let to_id = FormatId::new(to);
            let result = match args.get("from").and_then(|s| s.as_str()) {
                Some(from) => registry.convert_explicit(src, &FormatId::new(from), &to_id),
                None => match detect_format(src) {
                    Some(from) => registry.convert_explicit(src, &FormatId::new(from), &to_id),
                    None => Err(dibibest_core::CoreError::UnknownFormat(
                        "could not detect source format; pass `from`".into(),
                    )),
                },
            };
            match result {
                Ok(c) => {
                    let mut resp = json!({
                        "content": [{
                            "type": "text",
                            "text": format!("{}\n\n-- fidelity --\ntables {}/{}  columns {}/{}  refs {}/{}  indexes {}/{}  round-trip: {}",
                                c.output,
                                c.fidelity.tables.kept, c.fidelity.tables.total,
                                c.fidelity.columns.kept, c.fidelity.columns.total,
                                c.fidelity.refs.kept, c.fidelity.refs.total,
                                c.fidelity.indexes.kept, c.fidelity.indexes.total,
                                if c.fidelity.idempotent { "lossless" } else { "NOT idempotent" }),
                        }],
                        "isError": false
                    });
                    if let Some(obj) = resp.as_object_mut() {
                        obj.insert(
                            "structuredContent".into(),
                            serde_json::to_value(&c).unwrap_or(Value::Null),
                        );
                    }
                    resp
                }
                Err(e) => json!({
                    "content": [{ "type": "text", "text": format!("error: {e}") }],
                    "isError": true
                }),
            }
        }
        other => {
            json!({ "content": [{ "type": "text", "text": format!("unknown tool: {other}") }], "isError": true })
        }
    }
}
