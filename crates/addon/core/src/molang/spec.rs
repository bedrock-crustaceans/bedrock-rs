//! A Molang field's static spec: how it is read, where its version comes from, how
//! it compiles, and what the family does when it fails.

use super::{MolangVersionSource, ReaderKind};
use crate::diagnostic::Severity;

/// One Molang field of a family's document: its reader and version source (pass 1), and its
/// query admission, operations, empty-text rule, failure outcome and error severity (pass 2).
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MolangFieldSpec {
    /// Which JSON shapes the field takes.
    pub reader: ReaderKind,
    /// Where a string form's version comes from.
    pub version: MolangVersionSource,
    /// Which queries the expression may call.
    pub admission: Admission,
    /// Which operations the expression may use (vanilla's side-effect restriction).
    pub ops: OpsRule,
    /// What an empty text does.
    pub empty: EmptyRule,
    /// What the family does with a rejected expression.
    pub on_failure: MolangFailure,
    /// The severity of a rejecting message: `Error`, or `Warning` where the family says so
    /// (animation controller `on_entry` / `on_exit`).
    pub error_severity: Severity,
    /// A rejecting message at `Error` is a kept error (`Diagnostic::kept`): the game logs
    /// the parser's line at content-log error level and goes on with the document (the field
    /// kept, replaced, or its entry dropped). `false` leaves the error rejecting.
    pub error_kept: bool,
}

impl MolangFieldSpec {
    /// The usual cereal Molang field: strict versioned, the pack's version,
    /// the `default` query set, every operation, an empty text no expression, a rejected
    /// expression kept and logged, errors at `Error`, not kept. Families write
    /// `MolangFieldSpec { reader: …, ..MolangFieldSpec::DEFAULT }`.
    pub const DEFAULT: Self = Self {
        reader: ReaderKind::StrictVersioned,
        version: MolangVersionSource::Pack,
        admission: Admission::Default,
        ops: OpsRule::All,
        empty: EmptyRule::NoExpression,
        on_failure: MolangFailure::KeepLogged,
        error_severity: Severity::Error,
        error_kept: false,
    };
}

impl Default for MolangFieldSpec {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Which queries an expression may call.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Admission {
    /// The `default` query set.
    Default,
    /// The `tags` set: block and item descriptor `tags`.
    Tags,
    /// The `world_gen` set: features, feature rules, biomes.
    WorldGen,
    /// Only these queries, by full name (`"query.block_state"`): entity property defaults, legacy
    /// block permutation conditions and `bone_visibility`.
    Only(&'static [&'static str]),
}

/// The queries an entity property default may call.
#[doc(hidden)]
pub const PROPERTY_DEFAULT_QUERIES: &[&str] = &["query.had_component_group"];

/// The queries a legacy block permutation condition or `bone_visibility` may call.
#[doc(hidden)]
pub const BLOCK_STATE_QUERIES: &[&str] = &["query.block_state"];

/// Which operations an expression may use (vanilla's side-effect restriction).
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OpsRule {
    /// Every operation.
    All,
    /// No assignment (`math.random` stays allowed): entity property defaults.
    NoAssignments,
    /// No assignment and nothing random: block permutation conditions and `bone_visibility`.
    NoAssignmentsOrRandom,
}

/// What an empty text does.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EmptyRule {
    /// It is compiled, and rejected from version 4 (`No tokens found in expression`): the
    /// loaders that parse a string themselves.
    Compile,
    /// An empty text is no expression, not compiled and not logged (vanilla's cereal
    /// reader clears the node): every cereal Molang field. An object
    /// form at version −1 still compiles, at −1.
    NoExpression,
    /// An empty text, or an object form at version −1, is no expression: cereal block files,
    /// whose transformer also empties a node stored at −1.
    BlockNoExpression,
}

/// What the family does with a field whose expression is rejected.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MolangFailure {
    /// The whole document is dropped.
    RejectDocument,
    /// The entry holding the field is dropped (one render controller, a named script, a property
    /// group).
    DropEntry,
    /// The value is replaced by a constant (a controller transition by 0).
    ReplaceWith(f32),
    /// The field keeps its text and is logged.
    KeepLogged,
}

/// What the compile pass found for one field.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MolangOutcome {
    /// The expression compiled, or is a constant, or uses render-controller arrays or resources.
    Ok,
    /// No expression under the field's `EmptyRule`: nothing compiled, nothing logged.
    NoExpression,
    /// The expression was rejected; the family applies the field's failure rule.
    Failed(MolangFailure),
}
