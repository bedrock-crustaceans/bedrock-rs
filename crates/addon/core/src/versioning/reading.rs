//! How a slice reads its text: [`Reading`], chosen per `(family, struct, source)` by
//! [`Family::reading`](super::Family::reading). The part a family fixes once, because it runs
//! before a struct is selected, is in `Family::ENVELOPE` ([`Envelope`](super::Envelope)).
//!
//! A [`MemberFailure`] is acted on by the scalar pre-pass (`DefaultAndTell` defaults a refused
//! scalar's member and tells it, [`Reading::scalar_rules`]; every other failure fails the read
//! there) and by a derived `Schema::check` (`schema::SchemaCx`).

use super::{Family, StructIndex};
use crate::diagnostic::{Class, Severity};
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
    /// An error: the document is rejected.
    Error,
    /// A warning: the document is kept.
    Warn,
}

impl UnknownMask {
    /// The severity of an unknown member or component under this mask.
    pub const fn severity(self) -> Severity {
        match self {
            Self::Error => Severity::Error,
            Self::Warn => Severity::Warning,
        }
    }
}

/// How a string member reads a value of another JSON type ([`Reading::strings`]), beside the
/// number and bool profiles of [`Scalars`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Strings {
    /// As the slice's [`Scalars`] read a string: a string, or what the profile converts.
    ByScalars,
    /// jsoncpp's `asString("")`: a bool is `"true"` or `"false"`, a number, `null`, an array or
    /// an object the empty string, each told as a wrong type (graded as a failed member is)
    /// with the member keeping the text: the fixed-path readers of the menu texts and the
    /// music and sound definitions.
    AsString,
}

/// How a member that is one of several JSON forms (a leaf of a shared type: `scatter_chance`,
/// a scatter coordinate) is read ([`Reading::choices`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Choices {
    /// The form it is is read member by member, by the slice's own rules for each: a struct form
    /// is a struct whose members are read and whose unknown ones are the slice's (kept where
    /// it keeps them), a number beyond its float is refused where it stands. A feature rule's.
    ByMember,
    /// The game's conversion of the whole member: a struct form with a member it does not take
    /// is no form of it, a wrong type at the member, and a number beyond its float is read as
    /// the largest finite float of its sign and told. The scatter feature's.
    Whole,
}

/// Whether, and how, the members of a range that the range does not read are told
/// ([`Reading::range_members`]): the game binds `min` and `max` (or `range_min` and
/// `range_max`) of the object form and the first two elements of the list form, and nothing
/// else.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RangeMembers {
    /// None is told.
    Silent,
    /// Each is an unknown member of the word `report` (told at the unknown severity, not kept),
    /// in the pass of the other unknown members: the entity's.
    Unrecognized,
    /// Each is a `MEMBER_NOT_KEPT` at the unknown severity, in the last pass, after every other
    /// finding of the read: the item's.
    NotKept,
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
    /// How a string member reads a value of another JSON type. Not in the reading tables:
    /// `ByScalars` in every profile, and no versioned slice reads otherwise.
    pub strings: Strings,
    /// How a member that is one of several JSON forms is read. Not in the reading tables:
    /// `ByMember` in every profile, and only a slice of a family that shares such a leaf with
    /// another says `Whole`.
    pub choices: Choices,
    /// Whether, and how, the members of a range that the range does not read are told. Not in
    /// the reading tables: `Silent` in every profile, and a slice of the item and entity
    /// families says otherwise.
    pub range_members: RangeMembers,
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
        strings: Strings::ByScalars,
        choices: Choices::ByMember,
        range_members: RangeMembers::Silent,
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
        strings: Strings::ByScalars,
        choices: Choices::ByMember,
        range_members: RangeMembers::Silent,
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
        strings: Strings::ByScalars,
        choices: Choices::ByMember,
        range_members: RangeMembers::Silent,
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

    /// The reading with string members reading another JSON type as `strings`.
    #[must_use]
    pub const fn with_strings(self, strings: Strings) -> Self {
        Self { strings, ..self }
    }

    /// The reading with the members that are one of several forms read as `choices`.
    #[must_use]
    pub const fn with_choices(self, choices: Choices) -> Self {
        Self { choices, ..self }
    }

    /// The reading telling the members of a range that the range does not read, as
    /// `range_members` says.
    #[must_use]
    pub const fn with_range_members(self, range_members: RangeMembers) -> Self {
        Self {
            range_members,
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

    /// The severity a finding of `class` has under this reading: an unknown member or
    /// component ([`Class::Unknown`]) follows the slice's [`unknown`](Self::unknown) mask, and
    /// every other class is its [`default_severity`](Class::default_severity). (Not
    /// `schema::grading::grade`, which says what a member that fails to read does.)
    pub const fn grade(self, class: Class) -> Severity {
        match class {
            Class::Unknown => self.unknown.severity(),
            _ => class.default_severity(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_unknown_mask_gives_the_severity_of_an_unknown_member() {
        assert_eq!(UnknownMask::Error.severity(), Severity::Error);
        assert_eq!(UnknownMask::Warn.severity(), Severity::Warning);
    }

    #[test]
    fn a_profile_reads_its_choice_members_by_member_and_a_slice_may_say_whole() {
        for reading in [Reading::DOCUMENT, Reading::NULL_ABSENT, Reading::OWN] {
            assert_eq!(reading.choices, Choices::ByMember);
            assert_eq!(reading.with_choices(Choices::Whole).choices, Choices::Whole);
        }
    }

    #[test]
    fn a_profile_tells_no_unread_range_member_and_a_slice_may_say_so() {
        for reading in [Reading::DOCUMENT, Reading::NULL_ABSENT, Reading::OWN] {
            assert_eq!(reading.range_members, RangeMembers::Silent);
            let told = reading.with_range_members(RangeMembers::NotKept);
            assert_eq!(told.range_members, RangeMembers::NotKept);
        }
    }

    #[test]
    fn a_reading_grades_only_an_unknown_member_by_its_mask() {
        let classes = [
            Class::Error,
            Class::KeptError,
            Class::Unknown,
            Class::Warning,
            Class::Lint,
        ];
        let graded = |reading: Reading| classes.map(|class| reading.grade(class));
        assert_eq!(
            graded(Reading::DOCUMENT),
            [
                Severity::Error,
                Severity::Error,
                Severity::Error,
                Severity::Warning,
                Severity::Info,
            ]
        );
        assert_eq!(
            graded(Reading::DOCUMENT.with_unknown(UnknownMask::Warn)),
            [
                Severity::Error,
                Severity::Error,
                Severity::Warning,
                Severity::Warning,
                Severity::Info,
            ]
        );
        assert_eq!(graded(Reading::OWN), graded(Reading::NULL_ABSENT));
        assert_eq!(
            graded(Reading::OWN.with_unknown(UnknownMask::Error)),
            graded(Reading::DOCUMENT)
        );
    }
}
