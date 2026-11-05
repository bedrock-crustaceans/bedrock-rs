//! Molang in pack content: the molang types fields hold, re-exported once so there is exactly
//! one `MolangVersion` across this crate and the Molang crate; [`Molang`], the name the families
//! use for a Molang-valued field; [`MolangList`] for value-or-array fields; and, with the `molang`
//! feature, `MolangCheck`, the load-time compile check.
//!
//! A loaded document's expressions always carry their version: a plain string takes the version
//! its field's vanilla loader gives it (the pack's `min_engine_version`, the file's
//! `format_version`, or a constant), an object form `{"expression", "version"}` keeps its own. So
//! `molang::compile::compile_source` compiles any of them without knowing where it came from.
//! With the `molang` feature, loading compiles every expression once and reports
//! `InvalidMolang` / `MolangLint` with the compiler's diagnostic attached; without it, a
//! document holding expressions gets one `MolangNotChecked` info.
//!
//! The rest is internal to the crate family (hidden from the documentation, outside `SemVer`):
//! - `read`, `read_list`, `json_scalar`: a field read from an untyped `Value` by a hand-written
//!   parser; `write`, `write_list`: written back in the form it was read;
//! - `MolangFieldSpec` and its parts: each field's reader, version source, query admission,
//!   operations, empty-text rule, failure rule and error severity;
//! - `MolangFields`: the walk over a version struct's fields; `pass_one` (reader check and
//!   version, both builds), `not_checked`, and with `molang` `check_fields` / `MolangCheck::check`
//!   (pass 2, over molang's `compile::check`), `MolangCheck::check_source_with` and `ReportRules`
//!   (a family's own pass 2) and `molang_diagnostic` (the mapping of one compiler diagnostic);
//! - `MolangContext`, `pack_molang_version`, `format_molang_version`: where versions come from.

#[cfg(feature = "molang")]
mod check;
mod fields;
mod list;
mod read;
mod spec;
mod version;
mod write;

#[cfg(feature = "molang")]
#[cfg_attr(docsrs, doc(cfg(feature = "molang")))]
pub use check::{MolangCheck, RenderNote, ReportRules};

/// The exact crate, for callers that need more than the list below.
pub use molang;
pub use molang::catalog::{QueryAdmission, QueryAllowList, QueryCatalog, QuerySetMask, Side};
pub use molang::hash::HashedStr;
pub use molang::json::{
    JsonScalar, JsonType, MolangSource, MolangValueRepr, OtherJson, ReadField, ReaderKind,
    ReprError, SourceForm,
};
pub use molang::ops::OpSet;
pub use molang::version::{
    EngineVersion, Experiment, ExperimentMask, MolangVersion, RawVersion, VERSION_THRESHOLDS,
};
#[cfg(feature = "molang")]
#[cfg_attr(docsrs, doc(cfg(feature = "molang")))]
pub use molang::{
    cache::CompileCache,
    compile::{
        CheckDecision, CheckOptions, Checked, ClientOnlyNotes, CompileFailure, CompileOptions,
        CompileOutcome, Compiled, Deviations, OnFailure, VersionFrom, check, compile_source,
    },
    diag::{DiagCode, Diagnostic as MolangDiagnostic, LanguageMessage, Severity as MolangSeverity},
};

/// The name the families use for a Molang-valued field (vanilla
/// reads it as a string, a short or a float): `Const(f32) | Bool(bool) | Expr(MolangSource)`.
pub type Molang = molang::json::MolangValueRepr;

/// A value-or-array Molang field (vanilla reads it as an expression node from a JSON value): one
/// value, or an array whose elements are read the same way, recursively. The field is an
/// `Option<MolangList>`, where JSON `null` is `None`, "no expression".
///
/// An object is never a value here, not even the `{"expression", "version"}` form: the load
/// rejects it with `InvalidType`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(proxy = list::ListProxy)]
#[repr(u8)]
pub enum MolangList {
    /// A number, bool or string.
    One(Molang),
    /// An array of values.
    Many(Vec<MolangList>),
}

// Internal to the crate family: the readers, the field specs and the two passes.
#[cfg(feature = "molang")]
#[doc(hidden)]
pub use check::is_pack_setting_lint;
#[cfg(feature = "molang")]
#[doc(hidden)]
pub use check::{FieldOutcome, check_fields, molang_diagnostic};
#[doc(hidden)]
pub use fields::{
    MolangField, MolangFieldMut, MolangFields, MolangPass, MolangSlot, MolangVisitor, Step,
    count_sources, not_checked, pass_one, visit_fields, visit_fields_under,
};
#[doc(hidden)]
pub use read::{
    beyond_f32, beyond_f32_warning, json_scalar, read, read_finite, read_list, settle, settle_list,
    settle_source,
};
#[doc(hidden)]
pub use spec::{
    Admission, BLOCK_STATE_QUERIES, EmptyRule, MolangFailure, MolangFieldSpec, MolangOutcome,
    OpsRule, PROPERTY_DEFAULT_QUERIES,
};
#[doc(hidden)]
pub use version::{MolangContext, MolangVersionSource, format_molang_version, pack_molang_version};
#[doc(hidden)]
pub(crate) use write::ConstToken;
pub use write::{write, write_list};
