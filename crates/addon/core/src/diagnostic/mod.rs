//! Findings as data: one typed `Diagnostic` per thing a vanilla content log would print, never a
//! log line. Tests match on `code` variants, never on text.

mod builder;
pub mod builders;
pub mod cereal;
mod code;
mod def;
mod deprecated;
mod detail;
mod event;
mod facet;
mod list;
mod location;
mod moves;
mod path;
mod report;
mod severity;
pub mod testing;
// hidden: API not yet reviewed
#[doc(hidden)]
pub mod texts;
pub mod validator;

pub use code::{DiagnosticCode, NotFoundHint, VanillaText};
pub use def::{Class, Code, CodeDef, Kind};
pub use deprecated::ENUM_VALUE_DEPRECATED;
pub use event::EVENT_BOTH_RANDOMIZE_AND_SEQUENCE;
// hidden: reader plumbing: maps the facet error kinds to codes for the crate family's readers
#[doc(hidden)]
pub use facet::code_for_deserialize_error_kind;

pub use detail::{DiagnosticDetail, ReferenceHint, ShadowReason};
pub use list::{Diagnostics, Told};
pub use location::{Location, Span};
pub use path::{JsonPath, JsonPathSegment};
pub use report::PackReport;
pub use severity::{Phase, Severity};

use std::fmt;

/// One finding: a typed code, a severity, where, and in which phase.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Diagnostic {
    /// What happened, with typed payload. `Display` renders vanilla's text.
    pub code: DiagnosticCode,
    /// Chosen by the emitting rule; the same code can be an error in one family and a warning
    /// in another.
    pub severity: Severity,
    /// The loading phase that raised it.
    pub phase: Phase,
    /// Where.
    pub location: Location,
    /// Molang compiler detail and similar attachments.
    pub detail: Option<Box<DiagnosticDetail>>,
    /// A kept error: an error-level finding after which the game keeps the document. It counts
    /// as an error wherever findings are counted, and rejects nothing:
    /// [`Diagnostics::has_errors`] and the loads' "any error rejects" checks pass it by. Set
    /// by [`Diagnostic::of`] for a code of [`Class::KeptError`], by
    /// [`Diagnostics::push_kept_error`], and by the doc-hidden `Diagnostic::into_kept` that it
    /// uses, for a compile pass that marks a finding before it is pushed;
    /// [`Diagnostic::severity`] below error level clears it.
    pub kept: bool,
}

impl Diagnostic {
    /// An error-level finding that rejects the document it is about: an error that is not
    /// [`kept`](Self::kept). A loader's "any error rejects" check asks this.
    pub fn rejects(&self) -> bool {
        self.severity == Severity::Error && !self.kept
    }

    /// This finding as a kept error: at error level, rejecting nothing. A legacy code is made
    /// [`kept`](Self::kept) here: [`Diagnostics::push_kept_error`] goes through it, and so
    /// does a compile pass that marks a Molang finding kept before it is pushed; a specific
    /// code is kept by its [`Class::KeptError`] in [`of`](Self::of). Both halves must hold for
    /// the finding: it is an error in the game's content log, and the game's loader goes on
    /// with the document.
    #[must_use]
    pub fn into_kept(mut self) -> Self {
        self.severity = Severity::Error;
        self.kept = true;
        self
    }

    /// The general kind of the finding: a specific code's own, or the kind a legacy code
    /// belongs to. For the question `d.kind() == Kind::InvalidValue`, whichever rule raised it.
    pub fn kind(&self) -> Kind {
        def::kind_of(&self.code)
    }

    /// Whether the finding is a code of `def`, whatever its arguments: the specific question.
    /// A legacy code is no code of any definition.
    pub fn is<const N: usize>(&self, def: &CodeDef<N>) -> bool {
        matches!(&self.code, DiagnosticCode::Specific { code } if code.is(def))
    }

    /// The game's sentence for the finding, with its arguments; `None` when the game prints
    /// none and the finding is the toolkit's own. A legacy code gives the game text it carries
    /// in its `text` field.
    pub fn vanilla(&self) -> Option<String> {
        match &self.code {
            DiagnosticCode::Specific { code } => code.vanilla(),
            legacy => legacy.game_text().map(ToString::to_string),
        }
    }

    /// A finding without detail, from all four parts: the long form of the constructors.
    ///
    /// It does not grade by class (the `severity` is taken as given) and the finding is not
    /// `kept`. Prefer [`Diagnostic::of`], [`Diagnostic::error`], [`Diagnostic::warn`] and
    /// [`Diagnostic::info`] with the builder where the severity follows from the code. The
    /// struct is `#[non_exhaustive]`, so this is the only way to build one outside the crate.
    pub fn new(code: DiagnosticCode, severity: Severity, phase: Phase, location: Location) -> Self {
        Self {
            code,
            severity,
            phase,
            location,
            detail: None,
            kept: false,
        }
    }
}

/// Vanilla's content-log shape:
/// `[warning] <pack name> | blocks/ruby.json | minecraft:block/components/minecraft:friction | <message>`.
///
/// A part of the location that is not known (no pack, no file, the document root) is left out
/// with its separator. The message is the code's text followed by the detail: a Molang
/// compiler message after ` | ` (vanilla's `<expression> | <message>` scope chain), a
/// reference as `: <kind> '<id>'` with its hint.
impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] ", self.severity)?;
        let Location {
            pack,
            path,
            json_path,
            span: _,
        } = &self.location;
        if let Some(pack) = pack {
            write!(f, "{} | ", pack.name())?;
        }
        if let Some(path) = path {
            write!(f, "{path} | ")?;
        }
        if !json_path.is_empty() {
            write!(f, "{json_path} | ")?;
        }
        write!(f, "{}", self.code)?;
        match self.detail.as_deref() {
            None => Ok(()),
            Some(detail) => write!(f, "{detail}"),
        }
    }
}
