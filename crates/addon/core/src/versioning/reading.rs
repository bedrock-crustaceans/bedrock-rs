//! How a slice reads its text: [`Reading`], chosen per `(family, struct, source)` by
//! [`Family::reading`](super::Family::reading). The part a family fixes once, because it runs
//! before a struct is selected, is in `Family::ENVELOPE` ([`Envelope`](super::Envelope)).
//!
//! A [`MemberFailure`] is acted on by the scalar pre-pass (`DefaultAndTell` defaults a refused
//! scalar's member and tells it, [`Reading::scalar_rules`]; every other failure fails the read
//! there) and by a derived `Schema::check` (`schema::SchemaCx`).

use super::{ErrorMask, Family, StructIndex};
use crate::json::NullRule;
pub use crate::json::{BeyondWidth, Scalars};
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
/// [`OWN`](Self::OWN)) and at most a tweak or two with the `with_*` builders.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Reading {
    /// How a `null` member is read.
    pub null: NullHandling,
    /// How a scalar token of another kind is read.
    pub scalars: Scalars,
    /// What a number beyond the width of its float member does.
    pub beyond_width: BeyondWidth,
    /// What an unknown member or component of the payload is.
    pub unknown: UnknownMask,
    /// Whether the unknown members of the slice's structs are kept in the document's table
    /// and written back, where a struct does not say otherwise (`schema::unknown`, or the
    /// `extra: Extra` mark). `false` in every profile: only a struct that says so keeps them.
    pub keeps_unknown: bool,
    /// Whether a component that fails to read rejects the document (today's
    /// `LoadContext::strict_components`).
    pub components_fatal: bool,
    /// What a member that fails to read does.
    pub on_member_failure: MemberFailure,
}

impl Reading {
    /// The document input's cereal slices: `null` is a type error, an unknown member is an
    /// error and so is a component that fails to read. Scalars are read as a plain member reads
    /// them ([`Scalars::ConvertAndTell`], a number beyond a float clamped and told), which is
    /// how every plain member of these slices is read today; a slice whose members are read by
    /// the strict rule says [`Scalars::Strict`].
    pub const DOCUMENT: Self = Self {
        null: NullHandling::Rejected,
        scalars: Scalars::ConvertAndTell,
        beyond_width: BeyondWidth::ClampAndTell,
        unknown: UnknownMask::Error,
        keeps_unknown: false,
        components_fatal: true,
        on_member_failure: MemberFailure::Refuse,
    };
    /// The `JsonCpp` families (entity, features, the legacy item): a `null` member is
    /// absent, an unknown member a warning.
    pub const NULL_ABSENT: Self = Self {
        null: NullHandling::Absent,
        scalars: Scalars::ConvertAndTell,
        beyond_width: BeyondWidth::ClampAndTell,
        unknown: UnknownMask::Warn,
        keeps_unknown: false,
        components_fatal: false,
        on_member_failure: MemberFailure::Refuse,
    };
    /// A slice whose reader is the family's own (`FirstRun`, a hand-written legacy slice):
    /// no null rule, an unknown member a warning.
    pub const OWN: Self = Self {
        null: NullHandling::Own,
        scalars: Scalars::ConvertAndTell,
        beyond_width: BeyondWidth::ClampAndTell,
        unknown: UnknownMask::Warn,
        keeps_unknown: false,
        components_fatal: false,
        on_member_failure: MemberFailure::Refuse,
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

    /// The reading with a number beyond its float's width doing `beyond_width`.
    #[must_use]
    pub const fn with_beyond_width(self, beyond_width: BeyondWidth) -> Self {
        Self {
            beyond_width,
            ..self
        }
    }

    /// The reading with unknown members `unknown`.
    #[must_use]
    pub const fn with_unknown(self, unknown: UnknownMask) -> Self {
        Self { unknown, ..self }
    }

    /// The reading keeping the unknown members of its structs or not, where a struct does not
    /// say otherwise.
    #[must_use]
    pub const fn with_keeps_unknown(self, keeps_unknown: bool) -> Self {
        Self {
            keeps_unknown,
            ..self
        }
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

    /// The scalar rules the context carries for this reading: its profile, its rule for a
    /// number beyond its float, and whether a refused member defaults and tells
    /// ([`MemberFailure::DefaultAndTell`]; every other failure fails the read there).
    pub const fn scalar_rules(self) -> crate::json::ScalarRules {
        crate::json::ScalarRules::PLAIN
            .with_scalars(self.scalars)
            .with_beyond_width(self.beyond_width)
            .with_default_and_tell(matches!(
                self.on_member_failure,
                MemberFailure::DefaultAndTell
            ))
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
