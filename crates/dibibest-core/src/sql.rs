//! SQL DDL codec via sqlparser-rs (GenericDialect parse; PG-flavored emit).
//! Scaffold subset: CREATE TABLE (columns, PK/NOT NULL/UNIQUE/DEFAULT, inline
//! + table-level FK), CREATE [UNIQUE] INDEX. CHECK/COMMENT/etc = R2 corpus work.

use crate::ir::*;
use crate::{Caps, Codec, CoreError};
use sqlparser::ast::{
    ColumnOption, DataType, Expr, ObjectName, Statement, TableConstraint, TimezoneInfo,
};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use std::fmt::Write as _;

pub struct SqlCodec;

const FMT: &str = "sql";

impl Codec for SqlCodec {
    fn id(&self) -> &str {
        "sql"
    }

    fn caps(&self) -> Caps {
        Caps {
            indexes: true,
            expr_indexes: true,
            enums: false, // PG ENUM TYPES: R2
            defaults: true,
            notes: false, // COMMENT ON: R2
            dialects: &["postgresql"],
        }
    }

    fn parse(&self, src: &str) -> Result<SchemaIR, CoreError> {
        let dialect = GenericDialect {};
        let stmts = Parser::parse_sql(&dialect, src).map_err(|e| CoreError::Parse {
            format: FMT,
            message: e.to_string(),
        })?;
        let mut ir = SchemaIR::default();
        for stmt in stmts {
            match stmt {
                Statement::CreateTable(ct) => {
                    let name = object_name_to_string(&ct.name);
                    let mut t = Table {
                        name,
                        ..Table::default()
                    };
                    let mut pending_refs: Vec<Ref> = Vec::new();
                    for cd in &ct.columns {
                        let mut c = Column {
                            name: cd.name.to_string(),
                            ty: render_type(&cd.data_type),
                            ..Column::default()
                        };
                        for opt in &cd.options {
                            match &opt.option {
                                ColumnOption::PrimaryKey(_) => {
                                    c.pk = true;
                                    c.not_null = true;
                                }
                                ColumnOption::Unique(_) => c.unique = true,
                                ColumnOption::NotNull => c.not_null = true,
                                ColumnOption::Default(expr) => {
                                    c.default = Some(expr.to_string());
                                }
                                ColumnOption::ForeignKey(fk) => pending_refs.push(Ref {
                                    from_table: t.name.clone(),
                                    from_cols: vec![c.name.clone()],
                                    to_table: object_name_to_string(&fk.foreign_table),
                                    to_cols: fk.referred_columns.iter().map(ident_str).collect(),
                                    kind: RefKind::ManyToOne,
                                    name: None,
                                }),
                                _ => {}
                            }
                        }
                        t.columns.push(c);
                    }
                    for cons in &ct.constraints {
                        match cons {
                            TableConstraint::PrimaryKey(pk) => {
                                for ic in &pk.columns {
                                    if let Expr::Identifier(id) = &ic.column.expr {
                                        if let Some(col) =
                                            t.columns.iter_mut().find(|c| c.name == id.value)
                                        {
                                            col.pk = true;
                                            col.not_null = true;
                                        }
                                    }
                                }
                            }
                            TableConstraint::ForeignKey(fk) => pending_refs.push(Ref {
                                from_table: t.name.clone(),
                                from_cols: fk.columns.iter().map(ident_str).collect(),
                                to_table: object_name_to_string(&fk.foreign_table),
                                to_cols: fk.referred_columns.iter().map(ident_str).collect(),
                                kind: RefKind::ManyToOne,
                                name: None,
                            }),
                            _ => {}
                        }
                    }
                    ir.tables.push(t);
                    ir.refs.extend(pending_refs);
                }
                Statement::CreateIndex(ci) => {
                    let table = object_name_to_string(&ci.table_name);
                    let cols: Vec<String> = ci.columns.iter().map(index_col_str).collect();
                    if let Some(t) = ir.tables.iter_mut().find(|t| t.name == table) {
                        t.indexes.push(IndexDef {
                            name: ci.name.as_ref().map(object_name_to_string),
                            columns: cols,
                            unique: ci.unique,
                            // partial-index predicate preserved verbatim (round-trips
                            // as a DBML expression index)
                            expr: ci.predicate.as_ref().map(|p| p.to_string()),
                        });
                    }
                }
                _ => {}
            }
        }
        Ok(ir)
    }

    fn emit(&self, ir: &SchemaIR) -> Result<String, CoreError> {
        let mut out = String::new();
        for t in &ir.tables {
            let mut lines: Vec<String> = Vec::new();
            for c in &t.columns {
                let mut s = format!("  {} {}", c.name, c.ty);
                if c.pk {
                    s.push_str(" PRIMARY KEY");
                } else if c.not_null {
                    s.push_str(" NOT NULL");
                }
                if c.unique && !c.pk {
                    s.push_str(" UNIQUE");
                }
                if let Some(d) = &c.default {
                    let _ = write!(s, " DEFAULT {}", d);
                }
                lines.push(s);
            }
            let pk_cols: Vec<String> = t
                .columns
                .iter()
                .filter(|c| c.pk)
                .map(|c| c.name.clone())
                .collect();
            if pk_cols.len() > 1 {
                lines.push(format!("  PRIMARY KEY ({})", pk_cols.join(", ")));
            }
            for r in ir.refs.iter().filter(|r| r.from_table == t.name) {
                lines.push(format!(
                    "  FOREIGN KEY ({}) REFERENCES {}({})",
                    r.from_cols.join(", "),
                    r.to_table,
                    r.to_cols.join(", ")
                ));
            }
            let _ = writeln!(out, "CREATE TABLE {} (\n{}\n);", t.name, lines.join(",\n"));
            for idx in &t.indexes {
                let target = if idx.columns.is_empty() {
                    idx.expr.clone().unwrap_or_default()
                } else {
                    idx.columns.join(", ")
                };
                let kw = if idx.unique {
                    "CREATE UNIQUE INDEX"
                } else {
                    "CREATE INDEX"
                };
                let name = idx.name.clone().unwrap_or_else(|| {
                    format!(
                        "idx_{}_{}",
                        t.name,
                        idx.columns.first().map_or("expr", String::as_str)
                    )
                });
                let _ = writeln!(out, "{} {} ON {} ({});", kw, name, t.name, target);
            }
            let _ = writeln!(out);
        }
        Ok(out)
    }
}

fn ident_str(i: &sqlparser::ast::Ident) -> String {
    i.value.clone()
}

fn part_str(p: &sqlparser::ast::ObjectNamePart) -> String {
    p.as_ident().map(ident_str).unwrap_or_else(|| p.to_string())
}

fn object_name_to_string(o: &ObjectName) -> String {
    o.0.last().map(part_str).unwrap_or_else(|| o.to_string())
}

fn index_col_str(ic: &sqlparser::ast::IndexColumn) -> String {
    match &ic.column.expr {
        Expr::Identifier(id) => id.value.clone(),
        other => other.to_string(),
    }
}

fn render_type(dt: &DataType) -> String {
    match dt {
        DataType::Integer(_) | DataType::Int(_) => "INTEGER".into(),
        DataType::BigInt(_) => "BIGINT".into(),
        DataType::Text | DataType::String(_) => "TEXT".into(),
        DataType::Varchar(n) => match n {
            Some(l) => format!("VARCHAR({})", l),
            None => "VARCHAR".into(),
        },
        DataType::Boolean => "BOOLEAN".into(),
        DataType::Timestamp(_, tz) => match tz {
            TimezoneInfo::Tz => "TIMESTAMPTZ".into(),
            _ => "TIMESTAMP".into(),
        },
        DataType::Real | DataType::Float(_) => "REAL".into(),
        DataType::Double(_) => "DOUBLE PRECISION".into(),
        other => other.to_string().to_uppercase(),
    }
}
