//! How a slice reads its text: [`Reading`], chosen per `(family, struct, source)` by
//! [`Family::reading`](super::Family::reading). The part a family fixes once, because it runs
//! before a struct is selected, is in `Family::ENVELOPE` ([`Envelope`](super::Envelope)).
//!
//! Nothing acts on [`Scalars::Coercing`] or on a [`MemberFailure`] other than `Refuse` yet.

use super::{ErrorMask, Family, StructIndex};
use crate::json::NullRule;
use crate::load::DocumentSource;

/// The reading the pipeline gives the slice `index` of `F` from `source`: `F::reading`, with an
/// MCB source read under no null rule (the binary reader's), whatever the family says. Reached
/// from outside the crate as [`testing::slice_reading`](super::testing::slice_reading).
#[must_use]
pub fn slice_reading<F: Family>(index: StructIndex, source: DocumentSource) -> Reading {
    let reading = F::reading(index, source);
    match source {
        DocumentSource::Mcb => reading.with_null(NullHandling::Own),
        DocumentSource::Json => reading,
    }
}

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
}
