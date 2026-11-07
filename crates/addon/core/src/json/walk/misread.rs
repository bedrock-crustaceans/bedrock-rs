//! The scalars a typed member read as another value than the document writes.

use crate::diagnostic::codes as core_codes;
use crate::diagnostic::{Code, Diagnostic, JsonPath, Location, Phase, Severity};

/// A scalar of a document that the typed member it belongs to read as another value than the
/// document writes: facet coerces scalars and reads a float beyond its width as an infinity
/// without a word (an integer that does not fit its member it
/// refuses). Members behind a strict wrapper or another proxy judge their value
/// themselves and are not looked at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Misread {
    /// The scalar's path, from the root of the value walked.
    pub path: JsonPath,
    /// What was misread.
    pub kind: Misreading,
}

/// The text of the finding of a float member that read a number beyond its width
/// ([`Misread::finding`]): by it the versioned load knows the numbers to clamp.
pub const BEYOND_FLOAT_MEMBER: &str = core_codes::BEYOND_FLOAT_MEMBER.template();

impl Misread {
    /// The finding that tells the misreading, at `at` followed by the scalar's path. For a
    /// number beyond a float an `InvalidValue` information with the text
    /// [`BEYOND_FLOAT_MEMBER`]: the versioned load then clamps the number in the text and
    /// reads again, and its finding of the clamped number stands in the place of this one
    /// (the crate's own: cereal's `JsonCpp`
    /// reader and its non-strict `RapidJSON` reader narrow the double to an infinity without a
    /// line, while its strict `RapidJSON` reader refuses it; which slice reads strictly is not
    /// established per family).
    /// For another JSON type an `InvalidType` error that rejects the document: both cereal
    /// readers fail a value of another type with "invalid numeric value" at level 3 in both
    /// log masks, and the load fails.
    #[must_use]
    pub fn finding(&self, at: &JsonPath) -> Diagnostic {
        let mut path = at.clone();
        path.append(&self.path);
        let code = match self.kind {
            Misreading::BeyondFloat => Code::from(&core_codes::BEYOND_FLOAT_MEMBER),
            Misreading::WrongType => Code::from(&core_codes::INVALID_TYPE),
            Misreading::Defaulted => Code::from(&core_codes::INVALID_VALUE),
        };
        let severity = match self.kind {
            Misreading::BeyondFloat | Misreading::Defaulted => Severity::Info,
            Misreading::WrongType => Severity::Error,
        };
        Diagnostic::new(
            code,
            severity,
            Phase::Parse,
            Location::new(None, None, path, None),
        )
    }
}

/// How a scalar was misread.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Misreading {
    /// A number whose nearest float of the member's width is an infinity: the member holds
    /// the infinity, which no writer spells.
    BeyondFloat,
    /// A scalar of another JSON type than the member's: a string where a number or a boolean
    /// belongs, a number or a boolean where a string belongs, `null` where a value belongs.
    /// The member holds what facet made of it.
    WrongType,
    /// A scalar its member refuses, read as the member left out under a reading that defaults
    /// a failed member and tells it (`MemberFailure::DefaultAndTell`): an information at the
    /// scalar, the crate's own.
    Defaulted,
}
