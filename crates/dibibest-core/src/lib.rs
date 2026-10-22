//! Dibibest core IR — the hub every format converts through.
//!
//! Design (hybrid registry): format codecs implement [`Codec`]; callers use the
//! thin convenience layer ([`convert`], [`convert_explicit`]). Fidelity reporting
//! is first-class: every conversion reports what survived ([`Fidelity`]).

pub mod dbml;
pub mod ir;
pub mod registry;
pub mod sql;

pub use ir::{Column, IndexDef, Ref, RefKind, SchemaIR, Table};
pub use registry::{
    convert_explicit, default_registry, detect_format, registry_convert, Conversion, Fidelity,
    FormatId, Outcome, Registry,
};

/// Every fact a caller must know to use a codec: its id, its feature surface, and
/// the two directions it serves. This is the seam format plugins plug into.
pub trait Codec {
    /// Stable registry id, e.g. `FormatId("dbml")`.
    fn id(&self) -> &str;
    /// What this codec can honestly do — drives fidelity reporting.
    fn caps(&self) -> Caps;
    fn parse(&self, src: &str) -> Result<SchemaIR, CoreError>;
    fn emit(&self, ir: &SchemaIR) -> Result<String, CoreError>;
}

/// Honest feature surface of a codec. A capability listed as `false` means
/// structured data WILL be dropped on emit — and the fidelity report says so.
#[derive(Debug, Clone, Copy, Default)]
pub struct Caps {
    pub indexes: bool,
    pub enums: bool,
    pub defaults: bool,
    pub notes: bool,
    pub dialects: &'static [&'static str],
}

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("parse error in {format}: {message}")]
    Parse {
        format: &'static str,
        message: String,
    },
    #[error("emit error for {format}: {message}")]
    Emit {
        format: &'static str,
        message: String,
    },
    #[error("unknown format: {0}")]
    UnknownFormat(String),
    #[error("internal: {0}")]
    Internal(String),
}
