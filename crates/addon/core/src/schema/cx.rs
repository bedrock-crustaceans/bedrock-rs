//! [`SchemaCx`]: where a derived `Schema::check` reports a failed bound or a member the game
//! lower-cases, graded by the slice's [`Reading`] rather than by the struct.

use super::rows::BoundOp;
use crate::diagnostic::cereal::NumericRange;
use crate::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity};
use crate::versioning::{MemberFailure, Reading};

/// What a derived check does with the member after [`SchemaCx`] recorded its failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Verdict {
    /// The value stays; the owner has failed ([`SchemaCx::failed`]) and the slice's rule drops
    /// what it drops.
    Keep,
    /// The member takes its default (`MemberFailure::DefaultAndTell`): the field's facet
    /// `default`, its `schema::fallback`, the other member of a cross-field bound, or `None`.
    Reset,
}

/// The context a derived `Schema::check` reports to: the JSON path it walks, the slice's rule
/// for a member that fails, and the findings.
///
/// `Refuse`, `DropOwner` and `DropElement` record an `Error` and mark the owner failed (the
/// value stays, [`Verdict::Keep`]); `DefaultAndTell` records a `Warning` and resets the member
/// ([`Verdict::Reset`]). A bound's finding is `InvalidValue` with the game's range text
/// ([`NumericRange::text`]).
#[derive(Debug)]
pub struct SchemaCx<'d> {
    failure: MemberFailure,
    path: JsonPath,
    out: &'d mut Diagnostics,
    failed: bool,
}

impl<'d> SchemaCx<'d> {
    /// A context for the value at `at`, read under `reading`, reporting to `out`.
    pub fn new(reading: Reading, at: JsonPath, out: &'d mut Diagnostics) -> Self {
        Self {
            failure: reading.on_member_failure,
            path: at,
            out,
            failed: false,
        }
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

    /// The member at the current path broke the bound `op limit`.
    pub fn bound(&mut self, op: BoundOp, limit: f64) -> Verdict {
        let range = match op {
            BoundOp::Min => NumericRange::at_least(limit),
            BoundOp::Max => NumericRange::at_most(limit),
            BoundOp::Above => NumericRange {
                exclusive_min: true,
                ..NumericRange::at_least(limit)
            },
            BoundOp::Below => NumericRange {
                exclusive_max: true,
                ..NumericRange::at_most(limit)
            },
        };
        self.fail(range.code())
    }

    /// The member at the current path has an upper-case letter where the game lower-cases it.
    pub fn not_lowercase(&mut self) -> Verdict {
        self.fail(DiagnosticCode::InvalidValue { text: None })
    }

    fn fail(&mut self, code: DiagnosticCode) -> Verdict {
        let (severity, verdict) = if self.failure == MemberFailure::DefaultAndTell {
            (Severity::Warning, Verdict::Reset)
        } else {
            self.failed = true;
            (Severity::Error, Verdict::Keep)
        };
        self.out.push(
            Diagnostic::error(code)
                .at(self.path.clone())
                .severity(severity),
        );
        verdict
    }
}
