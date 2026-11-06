//! Value types several gated content modules use, defined once and always built: ranges,
//! filters and event triggers. The descriptors and menu categories live in
//! `bedrock_addon_shared_shapes`; `bedrock_addon::shared` re-exports both as one public path.
//!
//! Each type reads every JSON form vanilla accepts for it and remembers the form, so writing
//! reproduces the input; unknown members of an object form are kept in the type's `extra`.
//! The findings vanilla's readers log for a value (a swapped range, an unknown filter test)
//! come from the type's `check`, located under the path the using family passes; the family
//! sets the pack and file.

mod event;
mod filter;
mod range;
// hidden: reader plumbing of the crate family (the game's jsoncpp contract)
#[doc(hidden)]
pub mod read;

pub use event::EventTrigger;
pub use filter::{Filter, FilterGroup, FilterScalar, FilterSubject, FilterTest};
pub use range::{FloatRange, IntRange, Range, RangeEnd, RangeForm};
pub(crate) use range::{is_range, unread_range_members};
pub use read::{ShapeError, ShapeErrorKind};

use crate::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use crate::document::UnknownMembers;

/// An `UnrecognizedField` warning for each member of `extra`, at `at/<key>`: a shared type the
/// game reads with cereal (a filter, an event trigger, a block descriptor's proxy), whose
/// unknown member is logged at level 2 by the unstrict reader; a `Strict` slice makes it an
/// error.
// hidden: reader plumbing of the crate family (the game's jsoncpp contract)
#[doc(hidden)]
pub fn unrecognized(extra: &UnknownMembers, at: &JsonPath, out: &mut Diagnostics) {
    unrecognized_except(extra, at, &[], Severity::Warning, out);
}

/// [`unrecognized`] for the members of `extra` other than `skip` (members another finding
/// already reports), at `severity` (an information for a type the game reads from a
/// plain JSON value, which ignores the rest in silence).
// hidden: reader plumbing of the crate family (the game's jsoncpp contract)
#[doc(hidden)]
pub fn unrecognized_except(
    extra: &UnknownMembers,
    at: &JsonPath,
    skip: &[&str],
    severity: Severity,
    out: &mut Diagnostics,
) {
    for (key, _) in extra.iter() {
        if !skip.contains(&key) {
            let mut path = at.clone();
            path.push(key);
            out.push(finding(
                DiagnosticCode::UnrecognizedField { text: None },
                severity,
                &path,
            ));
        }
    }
}

/// Seals the traits of this module.
mod sealed {
    pub trait Sealed {}
}

/// A finding of a reader rule at `at`: vanilla logs these while it parses.
// hidden: reader plumbing of the crate family (the game's jsoncpp contract)
#[doc(hidden)]
pub fn finding(code: DiagnosticCode, severity: Severity, at: &JsonPath) -> Diagnostic {
    Diagnostic::new(
        code,
        severity,
        Phase::Parse,
        Location::new(None, None, at.clone(), None),
    )
}
