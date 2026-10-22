//! DBML codec (dbdiagram.io dialect) — built-in parser + emitter.
//!
//! Scaffold subset (v0.1): Project, Table (columns + settings, indexes incl.
//! expressions, notes), Ref (m2o/o2o/1m), Enum. Corpus-driven conformance
//! against the full DBML spec is milestone R1 (see docs/ROADMAP.md).

use crate::ir::*;
use crate::{Caps, Codec, CoreError};
use std::fmt::Write as _;

pub struct DbmlCodec;

const FMT: &str = "dbml";

impl Codec for DbmlCodec {
    fn id(&self) -> &str {
        "dbml"
    }

    fn caps(&self) -> Caps {
        Caps {
            indexes: true,
            enums: true,
            defaults: true,
            notes: true,
            dialects: &[],
        }
    }

    fn parse(&self, src: &str) -> Result<SchemaIR, CoreError> {
        parse_dbml(src)
    }

    fn emit(&self, ir: &SchemaIR) -> Result<String, CoreError> {
        emit_dbml(ir)
    }
}

// ── Parse ───────────────────────────────────────────────────────────────────

/// Split one-line blocks (`Table a { id integer [pk] }`) into three virtual
/// lines so the line-based state machine handles agent-style one-liners.
fn expand_one_line_blocks(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    for raw in src.lines() {
        let t = raw.trim();
        let opener = t.starts_with("Table ") || t.starts_with("Enum ") || t.starts_with("Project ");
        if opener && t.ends_with('}') && t.contains('{') {
            let brace = t.find('{').unwrap();
            let header = t[..=brace].trim_end();
            let body = t[brace + 1..t.len() - 1].trim();
            out.push(header.to_string());
            if !body.is_empty() {
                out.push(body.to_string());
            }
            out.push("}".to_string());
        } else {
            out.push(raw.to_string());
        }
    }
    out
}

fn parse_dbml(src: &str) -> Result<SchemaIR, CoreError> {
    let mut ir = SchemaIR::default();
    let mut block: Option<Block> = None;

    for (lineno, raw) in expand_one_line_blocks(src).iter().enumerate() {
        let line = raw.trim();
        let fail = |m: &str| CoreError::Parse {
            format: FMT,
            message: format!("line {}: {}", lineno + 1, m),
        };

        let mut closed = false;
        match (&mut block, line) {
            // indexes sub-block closer must not close the whole table
            (Some(Block::Table(ts)), "}") if ts.index_state => {
                ts.index_state = false;
            }
            // ── inside a block ──
            (Some(b), "}") => {
                match b {
                    Block::Table(t) => ir.tables.push(std::mem::take(&mut t.table)),
                    Block::Enum(e) => ir.enums.push(std::mem::take(e)),
                    Block::Project(name) => ir.project = Some(name.clone()),
                }
                closed = true;
            }
            (Some(Block::Table(ts)), l) if !l.is_empty() && !l.starts_with("//") => {
                let l = strip_comment(l);
                if l.is_empty() {
                    continue;
                }
                if l.starts_with("indexes") {
                    ts.index_state = true;
                    continue;
                }
                if ts.index_state {
                    // `(...)` with settings inline — parse_index_line applies both.
                    let idx = parse_index_line(l).ok_or_else(|| fail("bad index line"))?;
                    ts.table.indexes.push(idx);
                    continue;
                }
                ts.table
                    .columns
                    .push(parse_column_line(l).ok_or_else(|| fail("bad column line"))?);
            }
            (Some(Block::Enum(e)), l) if !l.is_empty() => {
                let v = strip_comment(l);
                if !v.is_empty() {
                    e.values.push(v.trim_matches(',').to_string());
                }
            }
            (Some(_), _) => {}

            // ── outside any block ──
            (None, l) if l.is_empty() || l.starts_with("//") => {}
            (None, l) if l.starts_with("Project ") => {
                let name = l
                    .strip_prefix("Project ")
                    .and_then(|s| s.split('{').next())
                    .unwrap_or("")
                    .trim()
                    .to_string();
                block = Some(Block::Project(name));
            }
            (None, l) if l.starts_with("Table ") => {
                let name = l
                    .strip_prefix("Table ")
                    .and_then(|s| s.split_whitespace().next())
                    .ok_or_else(|| fail("Table without name"))?
                    .to_string();
                block = Some(Block::Table(TableState {
                    table: Table {
                        name,
                        ..Table::default()
                    },
                    index_state: false,
                }));
            }
            (None, l) if l.starts_with("Enum ") => {
                let name = l
                    .strip_prefix("Enum ")
                    .and_then(|s| s.split('{').next())
                    .unwrap_or("")
                    .trim()
                    .to_string();
                block = Some(Block::Enum(EnumDef {
                    name,
                    values: Vec::new(),
                }));
            }
            (None, l) if l.starts_with("Ref") => {
                ir.refs
                    .push(parse_ref_line(l).ok_or_else(|| fail("bad Ref line"))?);
            }
            (None, _) => return Err(fail("unrecognized top-level statement")),
        }
        if closed {
            block = None;
        }
    }
    if block.is_some() {
        return Err(CoreError::Parse {
            format: FMT,
            message: "unexpected end of input: block not closed".into(),
        });
    }
    Ok(ir)
}

enum Block {
    Project(String),
    Table(TableState),
    Enum(EnumDef),
}

struct TableState {
    table: Table,
    index_state: bool,
}

fn strip_comment(l: &str) -> &str {
    match l.find("//") {
        Some(i) if !l[..i].ends_with(':') => l[..i].trim(),
        _ => l,
    }
}

fn parse_column_line(l: &str) -> Option<Column> {
    let mut parts = l.splitn(2, char::is_whitespace);
    let name = parts.next()?.trim().to_string();
    let rest = parts.next()?.trim();
    let (ty, settings) = match rest.find('[') {
        Some(i) => {
            let ty = rest[..i].trim().to_string();
            let inner = rest[i..].strip_prefix('[')?.strip_suffix(']')?;
            (ty, inner)
        }
        None => (rest.to_string(), ""),
    };
    let mut c = Column {
        name,
        ty,
        ..Default::default()
    };
    for s in split_settings(settings) {
        match s.as_str() {
            "pk" | "primary key" => c.pk = true,
            "not null" => c.not_null = true,
            "unique" => c.unique = true,
            "increment" | "autoincrement" => c.autoincrement = true,
            _ if s.starts_with("default:") => {
                let v = s["default:".len()..].trim();
                c.default = Some(v.trim_matches(|q| q == '\'' || q == '`').to_string());
            }
            _ if s.starts_with("note:") => {
                c.note = Some(s["note:".len()..].trim().trim_matches('\'').to_string());
            }
            _ => {} // unknown settings ignored on parse, recorded on round-trip diff
        }
    }
    if c.pk {
        c.not_null = true; // DBML semantics: pk implies not null
    }
    Some(c)
}

/// Parse `(...) [settings]` — the `(...)` may be a column list or an
/// expression (which can itself contain parens/quotes); settings may follow
/// on the same line.
fn parse_index_line(l: &str) -> Option<IndexDef> {
    let open = l.find('(')?;
    let mut depth = 0usize;
    let mut close = None;
    let mut in_quote = false;
    for (i, ch) in l.char_indices().skip(open) {
        match ch {
            '\'' => in_quote = !in_quote,
            '(' if !in_quote => depth += 1,
            ')' if !in_quote => {
                depth -= 1;
                if depth == 0 {
                    close = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = close?;
    let inner = &l[open + 1..close];
    let mut idx = if inner.contains(',') {
        IndexDef {
            name: None,
            columns: inner.split(',').map(|c| c.trim().to_string()).collect(),
            unique: false,
            expr: None,
        }
    } else {
        IndexDef {
            name: None,
            columns: Vec::new(),
            unique: false,
            expr: Some(inner.to_string()),
        }
    };
    let rest = l[close + 1..].trim();
    if let Some(settings) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        apply_index_settings(&mut idx, settings);
    }
    Some(idx)
}

fn apply_index_settings(idx: &mut IndexDef, settings: &str) {
    for s in split_settings(settings) {
        if s == "unique" {
            idx.unique = true;
        } else if let Some(n) = s.strip_prefix("name:") {
            idx.name = Some(n.trim().trim_matches('\'').to_string());
        }
    }
}

fn parse_ref_line(l: &str) -> Option<Ref> {
    let body = l.strip_prefix("Ref:")?.trim();
    // forms: a.b > c.d | a.b - c.d | a.b < c.d   (m2o | o2o | 1m)
    let (op, kind) = match body.find('>') {
        Some(i) => (i, RefKind::ManyToOne),
        None => match body.find('-') {
            Some(i) => (i, RefKind::OneToOne),
            None => body.find('<').map(|i| (i, RefKind::OneToMany))?,
        },
    };
    let lhs = body[..op].trim();
    let rhs = body[op + 1..].trim();
    let (ft, fc) = lhs.split_once('.')?;
    let (tt, tc) = rhs.split_once('.')?;
    Some(Ref {
        from_table: ft.trim().to_string(),
        from_cols: vec![fc.trim().to_string()],
        to_table: tt.trim().to_string(),
        to_cols: vec![tc.trim().to_string()],
        kind,
        name: None,
    })
}

/// Split DBML settings on commas that are not inside single quotes.
fn split_settings(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    for ch in s.chars() {
        match ch {
            '\'' => {
                in_quote = !in_quote;
                cur.push(ch);
            }
            ',' if !in_quote => {
                out.push(cur.trim().to_string());
                cur = String::new();
            }
            _ => cur.push(ch),
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

// ── Emit ────────────────────────────────────────────────────────────────────

fn emit_dbml(ir: &SchemaIR) -> Result<String, CoreError> {
    let mut out = String::new();
    if let Some(p) = &ir.project {
        let _ = writeln!(out, "Project {} {{\n}}\n", quote_ident(p));
    }
    for e in &ir.enums {
        let _ = writeln!(out, "Enum {} {{", quote_ident(&e.name));
        for v in &e.values {
            let _ = writeln!(out, "  {}", v);
        }
        let _ = writeln!(out, "}}\n");
    }
    for t in &ir.tables {
        let _ = writeln!(out, "Table {} {{", quote_ident(&t.name));
        for c in &t.columns {
            let _ = write!(out, "  {} {}", quote_ident(&c.name), c.ty);
            let mut settings: Vec<String> = Vec::new();
            if c.pk {
                settings.push("pk".into());
            }
            if c.not_null && !c.pk {
                settings.push("not null".into());
            }
            if c.unique {
                settings.push("unique".into());
            }
            if c.autoincrement {
                settings.push("increment".into());
            }
            if let Some(d) = &c.default {
                settings.push(format!("default: `{}`", d));
            }
            if let Some(n) = &c.note {
                settings.push(format!("note: '{}'", n.replace('\'', "''")));
            }
            if !settings.is_empty() {
                let _ = write!(out, " [{}]", settings.join(", "));
            }
            let _ = writeln!(out);
        }
        if !t.indexes.is_empty() {
            let _ = writeln!(out, "\n  indexes {{");
            for idx in &t.indexes {
                let cols = if idx.columns.is_empty() {
                    format!("({})", idx.expr.as_deref().unwrap_or(""))
                } else {
                    format!("({})", idx.columns.join(", "))
                };
                let mut settings: Vec<String> = Vec::new();
                if idx.unique {
                    settings.push("unique".into());
                }
                if let Some(n) = &idx.name {
                    settings.push(format!("name: '{}'", n));
                }
                if settings.is_empty() {
                    let _ = writeln!(out, "    {}", cols);
                } else {
                    let _ = writeln!(out, "    {} [{}]", cols, settings.join(", "));
                }
            }
            let _ = writeln!(out, "  }}");
        }
        if let Some(n) = &t.note {
            let _ = writeln!(out, "\n  note: '{}'", n.replace('\'', "''"));
        }
        let _ = writeln!(out, "}}\n");
    }
    let by_rel: Vec<&Ref> = ir.refs.iter().collect();
    if !by_rel.is_empty() {
        for r in by_rel {
            let op = match r.kind {
                RefKind::ManyToOne => ">",
                RefKind::OneToOne => "-",
                RefKind::OneToMany => "<",
            };
            let _ = writeln!(
                out,
                "Ref: {}.{} {} {}.{}",
                quote_ident(&r.from_table),
                r.from_cols.join(", "),
                op,
                quote_ident(&r.to_table),
                r.to_cols.join(", ")
            );
        }
    }
    Ok(out)
}

fn quote_ident(s: &str) -> String {
    if s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && !s.parse::<f64>().is_ok() {
        s.to_string()
    } else {
        format!("\"{}\"", s)
    }
}
