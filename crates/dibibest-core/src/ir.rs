//! The intermediate representation. Every format converts IR ↔ text; the IR is
//! the only thing codecs talk to, so adding a format never touches another one.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SchemaIR {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tables: Vec<Table>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refs: Vec<Ref>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enums: Vec<EnumDef>,
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    #[serde(default)]
    pub columns: Vec<Column>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub indexes: Vec<IndexDef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub ty: String,
    #[serde(default)]
    pub pk: bool,
    #[serde(default)]
    pub not_null: bool,
    #[serde(default)]
    pub unique: bool,
    #[serde(default)]
    pub autoincrement: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexDef {
    pub name: Option<String>,
    pub columns: Vec<String>,
    #[serde(default)]
    pub unique: bool,
    /// DBML expression indexes (e.g. partial indexes) — kept verbatim when a
    /// target format cannot represent them, so round-trips stay lossless.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expr: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ref {
    pub from_table: String,
    pub from_cols: Vec<String>,
    pub to_table: String,
    pub to_cols: Vec<String>,
    pub kind: RefKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RefKind {
    ManyToOne,
    OneToOne,
    OneToMany,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EnumDef {
    pub name: String,
    #[serde(default)]
    pub values: Vec<String>,
}

impl SchemaIR {
    pub fn table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.name == name)
    }
}
