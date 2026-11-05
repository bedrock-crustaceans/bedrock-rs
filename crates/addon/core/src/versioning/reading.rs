//! How a slice reads its text: [`Reading`], chosen per `(family, struct, source)` by
//! [`Family::reading`](super::Family::reading), and [`TextReading`], the part a family fixes
//! once because it runs before a struct is selected.
//!
//! `load` reads both: the defaults are built from the old knobs (`INPUT`, `null_rule`,
//! `error_mask`, `CLAMPS_BEYOND_DOUBLE`, `DUPLICATES_KEPT_IN`), which stay the way a family
//! states its values for now, so every family's reading equals what it did before. Nothing
//! acts on [`Scalars::Coercing`] or on a [`MemberFailure`] other than `Refuse` yet.

use super::{ErrorMask, InputKind, VersionInput};
use crate::json::NullRule;
use crate::load::DocumentSource;

/// How a member written as `null` is read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum NullHandling {
    /// A `null` where a value is read is a type error ([`NullRule::Rejected`]).
    Rejected,
    /// The member is absent ([`NullRule::Absent`]).
    Absent,
    /// The framework applies no rule: the family's hand-written reader takes `null` as it
    /// does (a `FirstRun` family, a legacy slice, an MCB source).
    Own,
}

impl NullHandling {
    /// The null rule the context carries into `parse_payload` and `parse_component`.
    pub const fn rule(self) -> Option<NullRule> {
        match self {
            Self::Rejected => Some(NullRule::Rejected),
            Self::Absent => Some(NullRule::Absent),
            Self::Own => None,
        }
    }

    /// The handling of a context's null rule.
    pub const fn from_rule(rule: Option<NullRule>) -> Self {
        match rule {
            Some(NullRule::Rejected) => Self::Rejected,
            Some(NullRule::Absent) => Self::Absent,
            None => Self::Own,
        }
    }
}

/// How a scalar token is read against the field's declared type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Scalars {
    /// A token of another kind is a type error.
    Strict,
    /// The jsoncpp value readers' coercions (a number read as a bool and back, a string read
    /// as a number). Declared, not acted on: which slices coerce, and by which of the two
    /// `as_int` rules, is settled against the game first.
    Coercing,
}

/// What an unknown member (code `0x4`) or component (code `0x400`) of the payload is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum UnknownMask {
    /// An error: the document is rejected ([`ErrorMask::Strict`]).
    Error,
    /// A warning: the document is kept ([`ErrorMask::Unstrict`]).
    Warn,
}

impl UnknownMask {
    /// The mask of an [`ErrorMask`].
    pub const fn from_error_mask(mask: ErrorMask) -> Self {
        match mask {
            ErrorMask::Strict => Self::Error,
            ErrorMask::Unstrict => Self::Warn,
        }
    }

    /// The [`ErrorMask`] of the mask.
    pub const fn error_mask(self) -> ErrorMask {
        match self {
            Self::Error => ErrorMask::Strict,
            Self::Warn => ErrorMask::Unstrict,
        }
    }
}

/// What a member that fails to read does to what holds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MemberFailure {
    /// One error, and the document is rejected. Every slice the framework reads today.
    Refuse,
    /// The object holding the member is dropped, the rest of the document kept.
    DropOwner,
    /// The member takes its default and the reader tells.
    DefaultAndTell,
    /// The failing array element is dropped, the rest of the array kept.
    DropElement,
}

/// How one slice of a family reads its payload: the axes that vary between slices, each on
/// its own.
///
/// Built from a profile ([`DOCUMENT`](Self::DOCUMENT), [`NULL_ABSENT`](Self::NULL_ABSENT),
/// [`OWN`](Self::OWN), [`COERCING`](Self::COERCING)) and at most a tweak or two with the
/// `with_*` builders.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Reading {
    /// How a `null` member is read.
    pub null: NullHandling,
    /// How a scalar token of another kind is read.
    pub scalars: Scalars,
    /// What an unknown member or component of the payload is.
    pub unknown: UnknownMask,
    /// Whether a component that fails to read rejects the document (today's
    /// `LoadContext::strict_components`).
    pub components_fatal: bool,
    /// What a member that fails to read does.
    pub on_member_failure: MemberFailure,
}

impl Reading {
    /// The document input's cereal slices: `null` is a type error, scalars are strict, an
    /// unknown member is an error and so is a component that fails to read.
    pub const DOCUMENT: Self = Self {
        null: NullHandling::Rejected,
        scalars: Scalars::Strict,
        unknown: UnknownMask::Error,
        components_fatal: true,
        on_member_failure: MemberFailure::Refuse,
    };
    /// The `JsonCpp` families (entity, features, the legacy item): a `null` member is
    /// absent, an unknown member a warning.
    pub const NULL_ABSENT: Self = Self {
        null: NullHandling::Absent,
        scalars: Scalars::Strict,
        unknown: UnknownMask::Warn,
        components_fatal: false,
        on_member_failure: MemberFailure::Refuse,
    };
    /// A slice whose reader is the family's own (`FirstRun`, a hand-written legacy slice):
    /// no null rule, an unknown member a warning.
    pub const OWN: Self = Self {
        null: NullHandling::Own,
        scalars: Scalars::Strict,
        unknown: UnknownMask::Warn,
        components_fatal: false,
        on_member_failure: MemberFailure::Refuse,
    };
    /// The jsoncpp value readers: [`OWN`](Self::OWN) with coercing scalars. No family uses
    /// it yet and the framework does not act on [`Scalars::Coercing`].
    pub const COERCING: Self = Self {
        scalars: Scalars::Coercing,
        ..Self::OWN
    };

    /// The reading with `null` read as `null`.
    #[must_use]
    pub const fn with_null(self, null: NullHandling) -> Self {
        Self { null, ..self }
    }

    /// The reading with scalars read as `scalars`.
    #[must_use]
    pub const fn with_scalars(self, scalars: Scalars) -> Self {
        Self { scalars, ..self }
    }

    /// The reading with unknown members `unknown`.
    #[must_use]
    pub const fn with_unknown(self, unknown: UnknownMask) -> Self {
        Self { unknown, ..self }
    }

    /// The reading with a failed component fatal or not.
    #[must_use]
    pub const fn with_components_fatal(self, components_fatal: bool) -> Self {
        Self {
            components_fatal,
            ..self
        }
    }

    /// The reading with a failed member doing `failure`.
    #[must_use]
    pub const fn with_on_member_failure(self, failure: MemberFailure) -> Self {
        Self {
            on_member_failure: failure,
            ..self
        }
    }

    /// The null rule the context carries for this reading.
    pub const fn null_rule(self) -> Option<NullRule> {
        self.null.rule()
    }

    /// The error mask of this reading.
    pub const fn error_mask(self) -> ErrorMask {
        self.unknown.error_mask()
    }

    /// The reading the old knobs give a slice: the null rule of `index` (none for an MCB
    /// source, which the binary reader reads), its error mask, and components fatal where
    /// the slice rejects `null`, as `load` derives them today.
    pub const fn from_knobs(
        null_rule: Option<NullRule>,
        error_mask: ErrorMask,
        source: DocumentSource,
    ) -> Self {
        let null = match source {
            DocumentSource::Mcb => NullHandling::Own,
            _ => NullHandling::from_rule(null_rule),
        };
        Self {
            null,
            scalars: Scalars::Strict,
            unknown: UnknownMask::from_error_mask(error_mask),
            // The derivation ignores the source, as `load` does.
            components_fatal: matches!(null_rule, Some(NullRule::Rejected)),
            on_member_failure: MemberFailure::Refuse,
        }
    }
}

/// The reading a family fixes for every file, because it runs on the text before a struct is
/// selected: where the version is read from (and so whether the JSON size check applies),
/// the clamp of numbers beyond a double and the members whose duplicates are kept. The
/// duplicates live here and not on [`Reading`]: the pipeline keeps or blanks a duplicate
/// before any slice is chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TextReading {
    /// Where the version is read from; [`VersionInput::Document`] also sizes a JSON text.
    pub version_input: VersionInput,
    /// Whether a number no double holds is read as the largest finite double of its sign
    /// (`Family::CLAMPS_BEYOND_DOUBLE`).
    pub clamps_beyond_double: bool,
    /// Members whose object value keeps the members written twice in it
    /// (`Family::DUPLICATES_KEPT_IN`).
    pub duplicates_kept_in: &'static [&'static str],
}

impl TextReading {
    /// The text reading of a family with the old knobs `input`, `clamps_beyond_double` and
    /// `duplicates_kept_in`.
    pub const fn from_knobs(
        input: InputKind,
        clamps_beyond_double: bool,
        duplicates_kept_in: &'static [&'static str],
    ) -> Self {
        Self {
            version_input: input.version_input(),
            clamps_beyond_double,
            duplicates_kept_in,
        }
    }

    /// Whether a text from `source` gets the JSON size check: a document input read from
    /// JSON (an MCB source is a binary input).
    pub const fn sizes_json(self, source: DocumentSource) -> bool {
        matches!(self.version_input, VersionInput::Document)
            && matches!(source, DocumentSource::Json)
    }
}
