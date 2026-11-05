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

/// The presets and the `const fn` builder: a spec is a preset and the fields that differ from
/// it (`MolangFieldSpec::KEPT.reader(ReaderKind::SchemaValidated)`).
impl MolangFieldSpec {
    /// [`DEFAULT`](Self::DEFAULT) with a rejecting message kept: the game logs the parser's
    /// line at the error level and loads the document with the field. The base of most cereal
    /// specs (28 of the 46 that are not aliases).
    pub const KEPT: Self = Self::DEFAULT.kept();

    /// [`DEFAULT`](Self::DEFAULT) with a rejected expression dropping the document, the error
    /// not kept: a schema-typed member whose failure fails the file (11 of the 46).
    pub const REJECTING: Self = Self::DEFAULT.on_failure(MolangFailure::RejectDocument);

    /// The same spec with another reader.
    #[must_use]
    pub const fn reader(mut self, reader: ReaderKind) -> Self {
        self.reader = reader;
        self
    }

    /// The same spec with another version source.
    #[must_use]
    pub const fn version(mut self, version: MolangVersionSource) -> Self {
        self.version = version;
        self
    }

    /// The same spec with another query admission.
    #[must_use]
    pub const fn admission(mut self, admission: Admission) -> Self {
        self.admission = admission;
        self
    }

    /// The same spec admitting the `tags` query set.
    #[must_use]
    pub const fn tags(self) -> Self {
        self.admission(Admission::Tags)
    }

    /// The same spec admitting the `world_gen` query set.
    #[must_use]
    pub const fn world_gen(self) -> Self {
        self.admission(Admission::WorldGen)
    }

    /// The same spec admitting only `queries`.
    #[must_use]
    pub const fn only(self, queries: &'static [&'static str]) -> Self {
        self.admission(Admission::Only(queries))
    }

    /// The same spec with another operations rule.
    #[must_use]
    pub const fn ops(mut self, ops: OpsRule) -> Self {
        self.ops = ops;
        self
    }

    /// The same spec with another empty-text rule.
    #[must_use]
    pub const fn empty(mut self, empty: EmptyRule) -> Self {
        self.empty = empty;
        self
    }

    /// The same spec compiling an empty text ([`EmptyRule::Compile`]).
    #[must_use]
    pub const fn compile_empty(self) -> Self {
        self.empty(EmptyRule::Compile)
    }

    /// The same spec with another failure outcome.
    #[must_use]
    pub const fn on_failure(mut self, on_failure: MolangFailure) -> Self {
        self.on_failure = on_failure;
        self
    }

    /// The same spec with a rejecting message at another severity.
    #[must_use]
    pub const fn error_severity(mut self, error_severity: Severity) -> Self {
        self.error_severity = error_severity;
        self
    }

    /// The same spec with a rejecting message kept ([`error_kept`](Self::error_kept)).
    #[must_use]
    pub const fn kept(mut self) -> Self {
        self.error_kept = true;
        self
    }

    /// The same spec with a rejecting message not kept.
    #[must_use]
    pub const fn not_kept(mut self) -> Self {
        self.error_kept = false;
        self
    }
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
