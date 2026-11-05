//! [`SchemaCx`]: where a derived `Schema::check` reports a failed bound, graded by the slice's
//! [`Reading`] rather than by the struct.

use super::grading::{Grading, grade};
use crate::diagnostic::cereal::NumericRange;
use crate::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath};
use crate::versioning::Reading;

/// What a derived check does with the member after [`SchemaCx`] recorded its failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Verdict {
    /// The value stays; the owner has failed ([`SchemaCx::failed`]) and the slice's rule drops
    /// what it drops.
    Keep,
    /// The member takes its default (`MemberFailure::DefaultAndTell`): a plain field its facet
    /// `default`, an `Option` `None` (a `schema::fallback` is what absence stands for, never
    /// written into the data), and the side of a cross-field bound the other member.
    Reset,
}

/// The context a derived `Schema::check` reports to: the JSON path it walks, the slice's rule
/// for a member that fails, and the findings.
///
/// A failed member is graded by [`grade`]: `Refuse`, `DropOwner` and `DropElement` record an
/// `Error` and mark the owner failed (the value stays, [`Verdict::Keep`]); `DefaultAndTell`
/// records an `Info` and resets the member ([`Verdict::Reset`]).
///
/// A field's bounds are one range ([`range`](Self::range)), and a value outside it is one
/// `InvalidValue` finding, with no text unless the family opts into the game's range text
/// ([`with_range_text`](Self::with_range_text)). These are the typed struct's bounds only: the
/// legacy per-row numeric templates of entity's goals stay on their generated table.
#[derive(Debug)]
pub struct SchemaCx<'d> {
    grading: Grading,
    range_text: bool,
    path: JsonPath,
    out: &'d mut Diagnostics,
    failed: bool,
}

impl<'d> SchemaCx<'d> {
    /// A context for the value at `at`, read under `reading`, reporting to `out`.
    pub fn new(reading: Reading, at: JsonPath, out: &'d mut Diagnostics) -> Self {
        Self {
            grading: grade(reading.on_member_failure),
            range_text: false,
            path: at,
            out,
            failed: false,
        }
    }

    /// The context with a bound's finding worded with the game's range text
    /// ([`NumericRange::text`], `the input is outside of the accepted range [0, 0.9]`) where
    /// `on`, as block tells it; without, the finding has no text, as entity and item tell it.
    #[must_use]
    pub fn with_range_text(mut self, on: bool) -> Self {
        self.range_text = on;
        self
    }

    /// The path of the member the walk is at.
    pub fn path(&self) -> &JsonPath {
        &self.path
    }

    /// Whether a member failed under a rule that fails its owner.
    pub fn failed(&self) -> bool {
        self.failed
    }

    /// Steps into the member `key`; [`leave`](Self::leave) steps out.
    pub fn enter(&mut self, key: &str) {
        self.path.push(key);
    }

    /// Steps into the element `index`.
    pub fn enter_index(&mut self, index: usize) {
        self.path.push_index(index);
    }

    /// Steps out of the last [`enter`](Self::enter) or [`enter_index`](Self::enter_index).
    pub fn leave(&mut self) {
        self.path.pop();
    }

    /// The member at the current path is outside `range`, its field's bounds: one finding,
    /// whichever side it broke. `resettable` says whether the member has a value to take
    /// (a default, a fallback, `None`, the other member); a required field has none, so under
    /// `DefaultAndTell` it fails its owner at that rule's severity instead of being accepted.
    pub fn range(&mut self, range: NumericRange, resettable: bool) -> Verdict {
        let text = self.range_text.then(|| range.text());
        let code = DiagnosticCode::InvalidValue { text };
        if resettable {
            self.fail(code)
        } else {
            self.report(code, true);
            Verdict::Keep
        }
    }

    /// The member at the current path, `value`, is no multiple of its field's `step`
    /// (`schema::multiple_of`): one finding, graded as a broken bound is
    /// ([`range`](Self::range)), and worded with the game's text (`40 is not multiple of 16`)
    /// where the range text is on.
    pub fn multiple(&mut self, value: f64, step: f64, resettable: bool) -> Verdict {
        let code = if self.range_text {
            crate::diagnostic::cereal::not_multiple_of(value, step)
        } else {
            DiagnosticCode::InvalidValue { text: None }
        };
        if resettable {
            self.fail(code)
        } else {
            self.report(code, true);
            Verdict::Keep
        }
    }

    fn fail(&mut self, code: DiagnosticCode) -> Verdict {
        let Grading {
            fails_owner,
            resets,
            ..
        } = self.grading;
        self.report(code, fails_owner);
        if resets {
            Verdict::Reset
        } else {
            Verdict::Keep
        }
    }

    /// Records `code` at the current path at the rule's severity, failing the owner where
    /// `fails_owner`.
    fn report(&mut self, code: DiagnosticCode, fails_owner: bool) {
        self.failed |= fails_owner;
        self.out.push(
            Diagnostic::error(code)
                .at(self.path.clone())
                .severity(self.grading.severity),
        );
    }
}
