//! The typed read of one entity component: the `parse` of every typed row of
//! [`EntityComponent`]'s table.
//!
//! Both reads are the reader's (`FromJson`, member by member). A document at or above a
//! component's typed version is read strictly: the first member of the wrong type is an error
//! and the component is dropped, as the typed door read it; a value outside its range is an
//! error of the component's check. A component converted from an older document's JSON
//! (`ConvertMode::Lenient`, the lenient conversion) keeps going: a bad member is an information
//! and takes its default ([`Recovery`]), and only a value that is not an object drops the
//! component. Unknown members are kept among the document's unknown members, by the path of
//! their level, with an `UnrecognizedField` at the slice's grade in both modes.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Class, Code, Diagnostic, Diagnostics, JsonPath, JsonPathSegment, Severity, Told,
};
use bedrock_addon_core::json::kit::{Node, Passes, Profile, Read, Recovery, Texts};
use bedrock_addon_core::json::parse::{Options, Parsed, parse_slice};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema::{FromJson, Schema, SchemaCx, read_document};
use bedrock_addon_core::versioning::{ConvertMode, MemberFailure, RangeMembers};
use facet::Facet;
use facet_value::VObject;

use super::EntityComponent;

/// A typed entity component: its JSON fix-ups before the typed read and its rule checks after
/// it. Both default to nothing.
pub(crate) trait Typed: for<'a> Facet<'a> + FromJson {
    /// The object a value that is not an object stands for, if the reader accepts that form
    /// (vanilla's legacy parsers take some scalars as "all defaults"); `None` rejects it.
    fn shorthand(_json: &Value, _mode: ConvertMode) -> Option<VObject> {
        None
    }

    /// Rewrites the component's JSON object before the typed read: an alias or a value
    /// spelling the reader maps. Returns whether anything changed.
    fn prepare(_json: &mut VObject, _cx: &mut ComponentCheck<'_>) -> bool {
        false
    }

    /// The rule checks on the read value (ranges, enum spellings, the shared types' own
    /// findings), reported through `cx`.
    fn check(&mut self, _cx: &mut ComponentCheck<'_>) {}

    /// What the lenient conversion gives up of a value that does not read, as the component's
    /// legacy reader does: the member of the component, or the element of its list, whole
    /// ([`Recovery::TopLevel`]), or, for the legacy readers of the components 1.26.60 types
    /// first, which read member by member at every level, the innermost value
    /// ([`Recovery::Value`]).
    const RECOVERY: Recovery = Recovery::TopLevel;
}

/// Where a component's rule checks report: the mode decides what a broken rule does.
pub(crate) struct ComponentCheck<'d> {
    mode: ConvertMode,
    load: &'d LoadContext,
    out: &'d mut Diagnostics,
    failed: bool,
}

impl<'d> ComponentCheck<'d> {
    /// A check reporting to `out` in `mode`, for a file loaded in `load`.
    pub(crate) fn new(mode: ConvertMode, load: &'d LoadContext, out: &'d mut Diagnostics) -> Self {
        Self {
            mode,
            load,
            out,
            failed: false,
        }
    }

    /// The load context of the file.
    #[cfg(feature = "molang")]
    pub(crate) fn load(&self) -> &LoadContext {
        self.load
    }

    /// The severity of an unknown member of a shared value the component holds: the grade of
    /// the reading the file is loaded under.
    pub(crate) fn unknown(&self) -> Severity {
        self.load.grade(Class::Unknown)
    }

    /// The mode of the read.
    pub(crate) fn mode(&self) -> ConvertMode {
        self.mode
    }

    /// A broken rule at `at`: an error that drops the component in a strict read; in the lenient
    /// conversion an information of the crate's own, as the legacy readers check no bound (the game
    /// writes no content-log line for this), and `field` takes `default`.
    pub(crate) fn invalid<T>(&mut self, at: &JsonPath, field: &mut T, default: T) {
        match self.mode {
            ConvertMode::Strict => {
                self.failed = true;
                self.push((&core_codes::INVALID_VALUE).into(), Severity::Error, at);
            }
            ConvertMode::Lenient => {
                *field = default;
                // The legacy readers check no bound: the typed struct's rule is the crate's own, an
                // information. Every caller passes no text (the game writes no content-log line for
                // this).
                self.push((&core_codes::INVALID_VALUE).into(), Severity::Info, at);
            }
        }
    }

    /// A member the typed read refuses, with the game's text where the generic read has none:
    /// `code` as an error that drops the component (the strict read only; the caller checks the
    /// mode).
    pub(crate) fn refuse(&mut self, code: Code, at: &JsonPath) {
        self.failed = true;
        self.push(code, Severity::Error, at);
    }

    /// A broken rule at `at` with no field default to fall back to: an error that drops the
    /// component in a strict read, an information of the crate's own in the lenient conversion
    /// ([`ComponentCheck::invalid`]).
    pub(crate) fn broken(&mut self, at: &JsonPath) {
        let mut unit = ();
        self.invalid(at, &mut unit, ());
    }

    /// The schema bound `min ..= max` of a number at `at` ([`ComponentCheck::invalid`] outside it).
    pub(crate) fn within<T: PartialOrd + Copy>(
        &mut self,
        at: &JsonPath,
        field: &mut T,
        (min, max): (T, T),
        default: T,
    ) {
        if !(min <= *field && *field <= max) {
            self.invalid(at, field, default);
        }
    }

    /// The schema minimum of a number at `at` ([`ComponentCheck::invalid`] below it).
    pub(crate) fn at_least<T: PartialOrd + Copy>(
        &mut self,
        at: &JsonPath,
        field: &mut T,
        min: T,
        default: T,
    ) {
        if *field < min {
            self.invalid(at, field, default);
        }
    }

    /// The bounds `value`'s `Schema` states (`schema::min`, `max`, `above`, `below`), at `at`,
    /// graded as [`ComponentCheck::invalid`] grades a broken rule: an error that drops the component in
    /// a strict read; in the lenient conversion an information, and the member takes its
    /// default.
    pub(crate) fn bounds<T: Schema>(&mut self, at: JsonPath, value: &mut T) {
        let failure = match self.mode {
            ConvertMode::Strict => MemberFailure::Refuse,
            ConvertMode::Lenient => MemberFailure::DefaultAndTell,
        };
        let reading = crate::READING.with_on_member_failure(failure);
        let mut cx = SchemaCx::new(reading, at, self.out);
        value.check(&mut cx);
        self.failed |= cx.failed();
    }

    /// A finding that does not depend on the mode (a vanilla warning; the value stays).
    pub(crate) fn push(&mut self, code: Code, severity: Severity, at: &JsonPath) {
        self.out
            .push(Diagnostic::error(code).at(at.clone()).severity(severity));
    }

    /// A kept error at `at`: a level-3 line of the game after which the entity loads: the game logs
    /// this at the error level and keeps the document, a kept error.
    pub(crate) fn kept(&mut self, code: Code, at: &JsonPath) {
        self.out
            .push_kept_error(Diagnostic::error(code).at(at.clone()));
    }

    /// A cereal range member at `at`: at the typed version (the strict read) the range
    /// binds only `min` / `max` (the game's range schema), so a `{range_min, range_max}` range is
    /// an unknown member at each end, an error that drops the component; the lenient conversion
    /// reads the legacy spelling and the typed value is written as `{min, max}`. Then the
    /// range's own findings.
    pub(crate) fn range<T: bedrock_addon_core::shared::RangeEnd>(
        &mut self,
        at: &JsonPath,
        range: &mut bedrock_addon_core::shared::Range<T>,
    ) {
        use bedrock_addon_core::shared::RangeForm;
        if range.form == RangeForm::RangeMinMax {
            if self.mode == ConvertMode::Strict {
                for (end, missing) in [
                    ("range_min", range.missing_min),
                    ("range_max", range.missing_max),
                ] {
                    if !missing {
                        let here = at.child(end);
                        self.refuse(Code::from(&core_codes::UNRECOGNIZED_FIELD), &here);
                    }
                }
                return;
            }
            range.form = RangeForm::MinMax;
        }
        self.out.extend(range.check(at));
    }

    /// The findings of a shared type's own `check`, as they are.
    pub(crate) fn extend(&mut self, findings: Diagnostics) {
        self.out.extend(findings);
    }

    /// The member at `from` is now at `to` (paths below the component): what is told of it
    /// from here on is told at `from`, the member the file has. Of two moves to
    /// nested places, the inner one is recorded last.
    pub(crate) fn moved(&mut self, from: JsonPath, to: JsonPath) {
        self.out.moved(from, to);
    }

    /// The list at `list` (a path below the component) now holds, in order, the elements
    /// it had at the indices `kept`: a check took the others out. What is told of an
    /// element from here on is told at the index the file has.
    pub(crate) fn left_out(&mut self, list: &JsonPath, kept: &[usize]) {
        self.out.left_out(list, kept);
    }
}

/// Removes the deprecated member names `names` from `json`, each with a `DeprecatedMember`
/// error (vanilla's cereal text, an error at every level: the document is rejected): the value is
/// not used. Returns whether one was there.
pub(crate) fn drop_deprecated(
    json: &mut VObject,
    names: &[&str],
    cx: &mut ComponentCheck<'_>,
) -> bool {
    let mut changed = false;
    for name in names {
        if json.remove(name).is_some() {
            changed = true;
            cx.out.push(Diagnostic::deprecated_member(
                name,
                JsonPath::new().child(name),
            ));
        }
    }
    changed
}

/// The legacy schema reader's other spellings (`a|b`, the camel-case names): below the typed
/// version each is renamed to the member it spells where that member is absent, the move recorded
/// so that what is told of it is told where the file has it. Returns whether anything moved.
pub(crate) fn legacy_spellings(
    json: &mut VObject,
    at: &JsonPath,
    spellings: &[(&str, &str)],
    cx: &mut ComponentCheck<'_>,
) -> bool {
    if !matches!(cx.mode(), ConvertMode::Lenient) {
        return false;
    }
    let mut changed = false;
    for (alias, member) in spellings {
        if !json.contains_key(member)
            && let Some(value) = json.remove(alias)
        {
            json.insert(*member, value);
            cx.moved(at.child(alias), at.child(member));
            changed = true;
        }
    }
    changed
}

/// Reads component `T` from its slice (the `parse` of `T`'s row): strictly, or with the lenient
/// conversion's member isolation; then keeps unknown members and runs `T`'s checks.
pub(crate) fn parse_typed<T: Typed + Into<EntityComponent>>(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<EntityComponent> {
    read::<T>(slice, mode, cx, d).map(Into::into)
}

/// [`parse_typed`] without the conversion into the enum: the typed value.
pub(crate) fn read<T: Typed>(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<T> {
    read_prepared::<T>(slice, mode, cx, d, |_, _| false)
}

/// [`read`] with a fix-up of an older version's shape before `T`'s own (`before`, which
/// returns whether it changed anything).
pub(crate) fn read_prepared<T: Typed>(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
    before: impl FnOnce(&mut VObject, &mut ComponentCheck<'_>) -> bool,
) -> Option<T> {
    let mut found = Diagnostics::default();
    let value = read_shaped::<T>(slice, mode, cx, &mut found, before);
    // The fix-ups wrap a legacy single value into a one-element array (`one_or_many`), and
    // the typed read names a member of it through the element `0` the file does not have:
    // a finding is at the member as written.
    let wrapped = |finding: &Diagnostic| {
        finding
            .location
            .json_path
            .segments()
            .any(|segment| segment == JsonPathSegment::Index(0))
    };
    let written = found
        .iter()
        .any(wrapped)
        .then(|| bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text).ok())
        .flatten();
    let kept = found.take_unknown();
    for told in found.into_told() {
        match told {
            Told::Finding(mut finding) => {
                if let Some(written) = &written {
                    finding.location.json_path = as_written(written, &finding.location.json_path);
                }
                d.push(*finding);
            }
            // An element the lenient read moved: what is told of it from here on
            // is told at the index the file has.
            Told::Moved { from, to } => d.moved(from, to),
        }
    }
    if value.is_some() {
        d.unknown_mut().absorb(&JsonPath::new(), kept);
    }
    value
}

/// `path` without the element `0` of a value that is no array in `written`: the member the
/// path names in the text as written. A path that leaves `written` (a member a fix-up
/// renamed or added) is kept from there on.
fn as_written(written: &Value, path: &JsonPath) -> JsonPath {
    let mut out = JsonPath::new();
    let mut node = Some(written);
    for segment in path.segments() {
        match segment {
            JsonPathSegment::Index(0) if node.is_some_and(|node| !node.is_array()) => {}
            JsonPathSegment::Index(index) => {
                out.push_index(index);
                node = node.and_then(Value::as_array).and_then(|a| a.get(index));
            }
            JsonPathSegment::Key(key) => {
                out.push(key);
                node = node.and_then(Value::as_object).and_then(|o| o.get(key));
            }
        }
    }
    out
}

/// The object without its `null` members (the game reads them as absent), the absent ones told,
/// and whether there were any.
fn without_null_members<T: Typed>(object: VObject, d: &mut Diagnostics) -> (VObject, bool) {
    let mut value = Value::from(object);
    let absent = bedrock_addon_core::json::remove_null_members(T::SHAPE, &mut value);
    bedrock_addon_core::json::tell_null_members(&absent, &JsonPath::new(), d);
    (
        value.as_object().cloned().unwrap_or_default(),
        !absent.is_empty(),
    )
}

/// [`read_prepared`], with the findings at the paths of the shape the fix-ups made.
fn read_shaped<T: Typed>(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
    before: impl FnOnce(&mut VObject, &mut ComponentCheck<'_>) -> bool,
) -> Option<T> {
    let Ok(mut parsed) = parse_slice(slice, Options::default()) else {
        // Not reached: the payload's parse has read the component's text.
        d.push(Diagnostic::of(&core_codes::INVALID_VALUE));
        return None;
    };
    not_null(Some(&parsed.value), d)?;
    let shorthand = (!parsed.value.is_object())
        .then(|| T::shorthand(&parsed.value, mode))
        .flatten();
    if shorthand.is_some()
        && parsed
            .value
            .as_string()
            .is_some_and(|text| !text.as_str().is_empty())
    {
        // The legacy parser reads any string as the component of defaults; vanilla's own
        // files write `""`. A string that says something is read the same, and told
        // (an information: the game does it without a word).
        d.push(Diagnostic::of(&core_codes::INVALID_TYPE).severity(Severity::Info));
    }
    let shorthand_read = shorthand.is_some();
    let Some(object) = shorthand.or_else(|| parsed.value.as_object().cloned()) else {
        if matches!(mode, ConvertMode::Lenient) {
            // a value that is not an object is dropped with `InvalidValue`, an information of the
            // crate's own (the game writes no content-log line for this): the legacy readers take
            // the members through const `Json::Value` access with defaults, and no line of the game
            // says it.
            d.push(Diagnostic::of(&core_codes::INVALID_VALUE).severity(Severity::Info));
            return None;
        }
        // Not an object (the typed read, strict): the reader names the problem.
        return read_component::<T>(&parsed, true, mode, cx, d).and(None);
    };
    // The game reads an entity through JsonCpp: a member written as `null` is absent and
    // keeps its default, at every version (a `null` is an absent member).
    let (mut object, _) = without_null_members::<T>(object, d);
    let mut prepare = ComponentCheck::new(mode, cx, d);
    let older = before(&mut object, &mut prepare);
    let changed = T::prepare(&mut object, &mut prepare) || older;
    if prepare.failed {
        return None;
    }
    // A fix-up may have given a member its shape (a single object wrapped into a list): the
    // `null` members it now reaches are absent too, and marked (a path through the
    // wrapped element `0` is placed at the member as written by `read_prepared`).
    if older || changed {
        let (prepared, _) = without_null_members::<T>(object, d);
        object = prepared;
    }
    let mut value = Value::from(object);
    let lenient = matches!(mode, ConvertMode::Lenient);
    if lenient {
        // The lenient conversion reads the JSON as vanilla writes it back (the int/real tag
        // lost).
        written_back(&mut value);
    }
    // What the fix-ups or the conversion rewrote is no text of the file: read without one.
    let as_written = !(older || changed || lenient || shorthand_read);
    parsed.value = value;
    let mut value = read_component::<T>(&parsed, as_written, mode, cx, d)?;
    let mut check = ComponentCheck::new(mode, cx, d);
    value.check(&mut check);
    (!check.failed).then_some(value)
}

/// Reads the component `T` from `parsed` (as the fix-ups left it, its text when `as_written`,
/// so every finding carries its span) by [`read_converted`], in the load `cx`, its unknown
/// members at the grade of the slice the document is read under.
fn read_component<T: Typed>(
    parsed: &Parsed<'_>,
    as_written: bool,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<T> {
    let strict = matches!(mode, ConvertMode::Strict);
    let grade = cx.grade(Class::Unknown);
    read_converted(parsed, as_written, strict, T::RECOVERY, grade, Some(cx), d)
}

/// Reads `T` from `parsed`, from its text when `as_written`: strictly, the first failure an
/// error that drops it, as the typed door read; in the lenient conversion every value that does
/// not read takes its default and is told (`recovery`: what a failed value gives up), as the
/// crate's own information. The entity's scalar and null rules at every slice, whatever the
/// context says, its structs keeping their unknown members ([`TYPED_READING`](crate::TYPED_READING));
/// the unknown members at `grade`; the rows read in the load `cx` where there is one.
pub(crate) fn read_converted<T: FromJson>(
    parsed: &Parsed<'_>,
    as_written: bool,
    strict: bool,
    recovery: Recovery,
    grade: Severity,
    cx: Option<&LoadContext>,
    d: &mut Diagnostics,
) -> Option<T> {
    let (failure, profile) = if strict {
        (MemberFailure::Refuse, Profile::FIRST_FAILURE)
    } else {
        (
            MemberFailure::DefaultAndTell,
            Profile::EVERY_FAILURE
                .with_passes(Passes::AfterRead)
                .with_recovery(recovery),
        )
    };
    let reading = crate::TYPED_READING
        .with_on_member_failure(failure)
        .with_range_members(RangeMembers::Unrecognized);
    let mut read = Read::new(reading, d)
        .unknown_severity(grade)
        .profile(profile)
        .texts(LENIENT_TEXTS);
    if let Some(cx) = cx {
        read = read.load(cx);
    }
    let node = if as_written {
        Node::parsed(parsed, JsonPath::new())
    } else {
        Node::new(&parsed.value, JsonPath::new())
    };
    read_document::<T>(&node, &mut read)
}

/// The words of the lenient conversion: a value it gives up is the crate's own
/// `InvalidValue` (the game writes no content-log line for it).
const LENIENT_TEXTS: Texts = Texts {
    defaulted: Some(&core_codes::INVALID_VALUE),
    ..Texts::CODES
};

/// `value` as vanilla writes a `Dynamic` value back: an integral real (other than a negative
/// zero, which keeps its sign) becomes an integer.
pub(crate) fn written_back(value: &mut Value) {
    if let Some(object) = value.as_object_mut() {
        for (_, member) in object.iter_mut() {
            written_back(member);
        }
    } else if let Some(array) = value.as_array_mut() {
        for element in array.iter_mut() {
            written_back(element);
        }
    } else if let Some(number) = value.as_number()
        && !number.is_integer()
    {
        let real = number.to_f64_lossy();
        #[allow(clippy::cast_possible_truncation, reason = "an integral real in range")]
        if real.fract() == 0.0
            && real.is_finite()
            && !(real == 0.0 && real.is_sign_negative())
            && real.abs() < 9.0e15
        {
            *value = Value::from(real as i64);
        }
    }
}

/// `None` when the component's value is `null`, which the game reads as an absent component: the
/// game's member-reading sites report a member written as `null` as not found.
/// The place carries an information, since the write-back leaves the component out.
fn not_null(json: Option<&Value>, d: &mut Diagnostics) -> Option<()> {
    if json.is_some_and(Value::is_null) {
        bedrock_addon_core::json::tell_null_members(&[JsonPath::new()], &JsonPath::new(), d);
        return None;
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use bedrock_addon_core::diagnostic::Span;

    use super::*;
    use crate::component::Attack;

    /// The finding of reading `text` as an `attack` component strictly.
    fn refused(text: &str) -> Diagnostic {
        let mut d = Diagnostics::default();
        let value = read::<Attack>(
            JsonSlice::new(text),
            ConvertMode::Strict,
            &LoadContext::detached(),
            &mut d,
        );
        assert!(value.is_none(), "a string other than `infinite`");
        d.iter().next().expect("the refusal").clone()
    }

    /// The span of the first `needle` in `text`.
    fn span_of(text: &str, needle: &str) -> Option<Span> {
        let offset = text.find(needle)?;
        Some(Span {
            offset: u32::try_from(offset).ok()?,
            len: u32::try_from(needle.len()).ok()?,
        })
    }

    #[test]
    fn a_refused_member_is_spanned_at_its_value() {
        let text = r#"{"effect_duration": "soon"}"#;
        let found = refused(text);
        assert!(found.is(&core_codes::INVALID_TYPE));
        assert_eq!(found.location.span, span_of(text, r#""soon""#));
    }

    #[test]
    fn a_refused_member_keeps_its_value_span_past_a_comment_before_it() {
        for text in [
            r#"{"effect_duration": /* a "quoted": comment */ "soon"}"#,
            "{\"effect_duration\": // a line comment\n \"soon\"}",
            r#"{"effect_duration" /* before the colon */ : "soon"}"#,
            r#"{"damage": {"min": 1, "max": 2}, "effect_name": "a // \" /*", "effect_duration": "soon"}"#,
        ] {
            let found = refused(text);
            assert_eq!(found.location.span, span_of(text, r#""soon""#), "{text}");
        }
    }
}
