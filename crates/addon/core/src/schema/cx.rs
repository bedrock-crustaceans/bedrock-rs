//! [`SchemaCx`]: where a derived `Schema::check` reports a failed bound, graded by the slice's
//! [`Reading`] rather than by the struct.

use super::grading::{Grading, grade};
use super::sides::{Found, Sides};
use super::walk::{End, Number};
use crate::diagnostic::cereal::NumericRange;
use crate::diagnostic::{
    Class, Code, CodeDef, Diagnostic, Diagnostics, JsonPath, Phase, Severity, codes,
};
use crate::load::LoadContext;
use crate::version::FormatVersion;
use crate::versioning::{FieldMeta, Reading, ValidateCx, check_fields_since_engine};

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
/// A failed member is graded by [`grade`]: `Refuse` records an `Error` and mark the owner failed (the value stays, [`Verdict::Keep`]); `DefaultAndTell`
/// records an `Info` and resets the member ([`Verdict::Reset`]).
///
/// A field's bounds are one range ([`range`](Self::range)), and a value outside it is one
/// `InvalidValue` finding, with no text unless the family opts into the game's range text
/// ([`with_range_text`](Self::with_range_text)). These are the typed struct's bounds only: the
/// legacy per-row numeric templates of entity's goals stay on their generated table.
#[derive(Debug)]
pub struct SchemaCx<'d> {
    grading: Grading,
    unknown: Severity,
    range_text: bool,
    one_sided_text: Option<bool>,
    tree: Option<TreeWording>,
    phase: Phase,
    leaf_findings: bool,
    load: Option<Loaded<'d>>,
    path: JsonPath,
    out: &'d mut Diagnostics,
    failed: bool,
}

/// How a game that reads with the older schema tree words a broken bound: `Value '%s' outside
/// valid range [%s, %s]` (core's `OUTSIDE_RANGE`), stating the value and both limits as the
/// type writes them, and its own line for each end of a range (`'range_min' with value '%s'
/// outside valid range ..`), which the family names; an array too short or too long is
/// `ARRAY_TOO_SMALL` or `ARRAY_TOO_LARGE`.
#[derive(Clone, Copy, Debug)]
pub struct TreeWording {
    /// The line of a range whose lower end is outside, with the end, the lower and the upper
    /// limit as arguments.
    pub range_min: &'static CodeDef<3>,
    /// The line of a range whose upper end is outside.
    pub range_max: &'static CodeDef<3>,
}

/// What a field says of its broken bounds (`schema::low_code`, `schema::high_code`,
/// `schema::kept`): the code a value below the low bound or past the high one is told under, in
/// place of the context's own wording, and whether the game logs the line and keeps the value.
// hidden: emitted by the schema derive
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default)]
pub struct BoundRules {
    /// The code of a value outside the low bound (`min`, `above`), made when it is told.
    pub low: Option<fn() -> Code>,
    /// The code of a value outside the high bound (`max`, `below`), made when it is told.
    pub high: Option<fn() -> Code>,
    /// The game logs the line and goes on with the value ([`SchemaCx::kept`]): nothing fails or
    /// resets.
    pub kept: bool,
}

/// What a check run by the validate stage knows about the pack and the file.
#[derive(Clone, Copy, Debug)]
struct Loaded<'d> {
    load: &'d LoadContext,
    original_version: FormatVersion,
}

impl<'d> SchemaCx<'d> {
    /// A context for the value at `at`, read under `reading`, reporting to `out`.
    pub fn new(reading: Reading, at: JsonPath, out: &'d mut Diagnostics) -> Self {
        Self {
            grading: grade(reading.on_member_failure),
            unknown: reading.grade(Class::Unknown),
            range_text: false,
            one_sided_text: None,
            tree: None,
            phase: Phase::Parse,
            leaf_findings: false,
            load: None,
            path: at,
            out,
            failed: false,
        }
    }

    /// The context of a family's validate stage ([`validating`](Self::validating) for what a
    /// `ValidateCx` holds): the payload's path, the slice's reading, the findings told in
    /// [`Phase::Validate`].
    pub fn for_validate(cx: &'d mut ValidateCx<'_>) -> Self {
        Self::validating(
            cx.reading,
            cx.at.clone(),
            cx.load,
            cx.original_version,
            &mut *cx.diagnostics,
        )
    }

    /// A context for the value at `at` that runs after the read, for the pack of `load` and a
    /// file that wrote `original_version`: its findings are told in [`Phase::Validate`], and
    /// it knows the engine ([`engine_version`](Self::engine_version)), which what a
    /// `schema::since_engine` member is checked against.
    pub fn validating(
        reading: Reading,
        at: JsonPath,
        load: &'d LoadContext,
        original_version: FormatVersion,
        out: &'d mut Diagnostics,
    ) -> Self {
        let mut this = Self::new(reading, at, out);
        this.phase = Phase::Validate;
        this.load = Some(Loaded {
            load,
            original_version,
        });
        this
    }

    /// The context with the leaf types that carry findings of their own (a block descriptor
    /// that is no block, a range whose ends were swapped) telling them where the walk meets
    /// them, at their member's path ([`leaf_findings`](Self::leaf_findings)). Off until a
    /// family whose game logs those lines turns it on: another family that holds such a
    /// leaf tells none.
    #[must_use]
    pub fn with_leaf_findings(mut self, on: bool) -> Self {
        self.leaf_findings = on;
        self
    }

    /// The context with a bound's finding worded with the game's range text
    /// ([`NumericRange::text`], `the input is outside of the accepted range [0, 0.9]`) where
    /// `on`, as block tells it; without, the finding has no text, as entity and item tell it.
    #[must_use]
    pub fn with_range_text(mut self, on: bool) -> Self {
        self.range_text = on;
        self
    }

    /// The context with the finding of a one-sided bound (a lower bound alone, or an upper one)
    /// worded with the game's text where `on`, or with none where not, whatever
    /// [`with_range_text`](Self::with_range_text) says of a range with both sides: for a game
    /// that words only a range with both ends.
    #[must_use]
    pub fn with_one_sided_range_text(mut self, on: bool) -> Self {
        self.one_sided_text = Some(on);
        self
    }

    /// The context with a broken bound and a list of the wrong length worded as the game's
    /// older schema tree words them ([`TreeWording`]), whatever the range text says.
    #[must_use]
    pub fn with_tree_wording(mut self, wording: TreeWording) -> Self {
        self.tree = Some(wording);
        self
    }

    /// The path of the member the walk is at.
    pub fn path(&self) -> &JsonPath {
        &self.path
    }

    /// Whether a leaf type tells the findings of its own at its member
    /// ([`with_leaf_findings`](Self::with_leaf_findings)).
    pub fn leaf_findings(&self) -> bool {
        self.leaf_findings
    }

    /// The severity of an unknown member under the slice's reading
    /// ([`Reading::grade`] of [`Class::Unknown`]): what a leaf that tells its unknown members
    /// grades them by.
    pub fn unknown_severity(&self) -> Severity {
        self.unknown
    }

    /// The engine version of the pack, when the check runs in the validate stage
    /// ([`for_validate`](Self::for_validate)).
    pub fn engine_version(&self) -> Option<FormatVersion> {
        self.load.map(|loaded| loaded.load.engine_version)
    }

    /// The `format_version` the file wrote (the effective one when it wrote none), when the
    /// check runs in the validate stage.
    pub fn original_version(&self) -> Option<FormatVersion> {
        self.load.map(|loaded| loaded.original_version)
    }

    /// Tells `finding` as the walk meets it: a leaf's own finding, located by the leaf at
    /// [`path`](Self::path). Not graded, and it does not fail the owner by itself.
    pub fn push(&mut self, finding: Diagnostic) {
        self.out.push(finding);
    }

    /// [`push`](Self::push) for each of `findings`.
    pub fn extend(&mut self, findings: Diagnostics) {
        self.out.extend(findings);
    }

    /// An error with `code` at the member `key` of the one the walk is at (`""`: that one
    /// itself), whatever the slice's rule says of a failed member: the game's line after which
    /// it drops the document.
    pub fn error(&mut self, key: &str, code: Code) {
        let finding = self.told(key, code);
        self.failed = true;
        self.out.push(finding);
    }

    /// A kept error with `code` at the member `key` (`""`: the one the walk is at): the game
    /// logs the line and goes on with the value, so the owner does not fail and the member keeps
    /// what it holds.
    pub fn kept(&mut self, key: &str, code: Code) {
        let finding = self.told(key, code);
        self.out.push_kept_error(finding);
    }

    fn told(&self, key: &str, code: Code) -> Diagnostic {
        let at = if key.is_empty() {
            self.path.clone()
        } else {
            self.path.child(key)
        };
        Diagnostic::error(code).at(at).phase(self.phase)
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
        let code = self.range_code(range);
        self.broken(code, resettable)
    }

    fn range_code(&self, range: NumericRange) -> Code {
        let one_sided = range.min.is_none() || range.max.is_none();
        let text = match self.one_sided_text {
            Some(on) if one_sided => on,
            _ => self.range_text,
        };
        if text {
            range.code()
        } else {
            Code::from(&codes::INVALID_VALUE)
        }
    }

    /// The member at the current path has a number outside its field's bounds: [`range`](Self::range)
    /// for a broken bound that states which number it is and the texts of the value and the
    /// limits, which the older schema tree's wording ([`with_tree_wording`](Self::with_tree_wording))
    /// needs.
    pub fn found(&mut self, found: Found, resettable: bool) -> Verdict {
        let code = self.found_code(found);
        self.broken(code, resettable)
    }

    /// The code [`found`](Self::found) tells a broken bound under.
    fn found_code(&self, found: Found) -> Code {
        let Some(tree) = self.tree else {
            return self.range_code(found.range);
        };
        let line = match found.end {
            End::Only => &codes::OUTSIDE_RANGE,
            End::Min => tree.range_min,
            End::Max => tree.range_max,
        };
        line.with([found.value, found.low, found.high])
    }

    /// [`found`](Self::found) for a field that states what its broken bounds are told as
    /// (`rules`): the code of the `side` the value broke where the field names one, else the
    /// context's own; a kept bound is logged and nothing fails.
    pub fn found_by(
        &mut self,
        found: Found,
        side: super::Side,
        rules: &BoundRules,
        resettable: bool,
    ) -> Verdict {
        let own = match side {
            super::Side::Low => rules.low,
            super::Side::High => rules.high,
        };
        let code = own.map_or_else(|| self.found_code(found), |code| code());
        if rules.kept {
            self.kept("", code);
            return Verdict::Keep;
        }
        self.broken(code, resettable)
    }

    /// A number a rule holds is outside `[min, max]` (a side left `None` is no limit): the
    /// hand-written form of `schema::min` and `schema::max` for a rule that bounds a number only
    /// under a condition (an engine), told at the member `key` (`""`: the one the walk is at)
    /// as a derived bound is worded and graded, failing the owner. Whether it was outside.
    pub fn within<T: Number>(
        &mut self,
        key: &str,
        value: T,
        min: Option<T>,
        max: Option<T>,
    ) -> bool {
        let sides = Sides {
            low: min,
            low_exclusive: false,
            high: max,
            high_exclusive: false,
        };
        if sides.broken(value).is_none() {
            return false;
        }
        if !key.is_empty() {
            self.enter(key);
        }
        self.found(sides.found(End::Only, value), false);
        if !key.is_empty() {
            self.leave();
        }
        true
    }

    /// The member at the current path has `len` elements, fewer than `min` or more than `max`
    /// (`schema::min_len`, `schema::max_len`): one finding, graded as a broken bound is, worded as
    /// the older schema tree words an array of the wrong size where
    /// [`with_tree_wording`](Self::with_tree_wording) is on, and without text otherwise.
    pub fn length(
        &mut self,
        len: usize,
        min: Option<usize>,
        max: Option<usize>,
        resettable: bool,
    ) -> Verdict {
        let code = match (self.tree, min, max) {
            (Some(_), Some(min), _) if len < min => {
                codes::ARRAY_TOO_SMALL.with([len.to_string(), min.to_string()])
            }
            (Some(_), _, Some(max)) => {
                codes::ARRAY_TOO_LARGE.with([len.to_string(), max.to_string()])
            }
            _ => Code::from(&codes::INVALID_VALUE),
        };
        self.broken(code, resettable)
    }

    fn broken(&mut self, code: Code, resettable: bool) -> Verdict {
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
            Code::from(&codes::INVALID_VALUE)
        };
        if resettable {
            self.fail(code)
        } else {
            self.report(code, true);
            Verdict::Keep
        }
    }

    fn fail(&mut self, code: Code) -> Verdict {
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
    fn report(&mut self, code: Code, fails_owner: bool) {
        self.failed |= fails_owner;
        self.out.push(
            Diagnostic::error(code)
                .at(self.path.clone())
                .phase(self.phase)
                .severity(self.grading.severity),
        );
    }

    /// The member at the current path is present, and the engine of the pack is older than
    /// `since` (`schema::since_engine`): the game's schema of that engine has no such member,
    /// so the document fails with "member '%s' is not valid here". On an engine from `since`
    /// on, a pack whose `min_engine_version` is older than `since` gets the information
    /// `FieldSinceEngine`. Nothing outside the validate stage ([`for_validate`](Self::for_validate)),
    /// which alone knows the engine.
    pub fn since_engine(&mut self, since: FormatVersion, present: bool) {
        let Some(Loaded { load, .. }) = self.load else {
            return;
        };
        if !present {
            return;
        }
        if load.engine_version < since {
            let key = match self.path.segments().last() {
                Some(crate::diagnostic::JsonPathSegment::Key(key)) => key.to_owned(),
                Some(crate::diagnostic::JsonPathSegment::Index(index)) => index.to_string(),
                None => String::new(),
            };
            self.failed = true;
            self.out.push(
                Diagnostic::of(codes::CHILD_NOT_VALID.with([key]))
                    .severity(Severity::Error)
                    .at(self.path.clone()),
            );
            return;
        }
        let field = FieldMeta {
            name: "",
            since_engine: since,
        };
        check_fields_since_engine(&[field], |_| true, &self.path, load, self.out);
    }
}
