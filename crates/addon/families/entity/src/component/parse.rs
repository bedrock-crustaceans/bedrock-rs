//! The typed read of one entity component: the `parse` of every typed row of
//! [`EntityComponent`]'s table.
//!
//! A document at or above a component's typed version is read strictly: a member of the wrong
//! type or outside its range is an error and the component is dropped. A component converted
//! from an older document's JSON (`ConvertMode::Lenient`, the lenient conversion) keeps going: a
//! bad member is a warning and takes its default, and only a value that is not an object drops
//! the component. Unknown members are kept among the document's unknown members, by the path
//! of their level, with an `UnrecognizedField` warning in both modes.

use bedrock_addon_core::diagnostic::{
    Class, Diagnostic, DiagnosticCode, Diagnostics, JsonPath, JsonPathSegment, Severity, Told,
    VanillaText,
};
use bedrock_addon_core::json::{JsonSlice, NullRule, Value, to_json};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema::{Schema, SchemaCx};
use bedrock_addon_core::versioning::{ConvertMode, MemberFailure};
use facet::Facet;
use facet_value::VObject;

use super::EntityComponent;

/// A typed entity component: its JSON fix-ups before the typed read and its rule checks after
/// it. Both default to nothing.
pub(crate) trait Typed: for<'a> Facet<'a> {
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

    /// Whether the lenient conversion keeps a member, or a list element, whose nested member
    /// it cannot read, that nested member taken out (so it takes its default) and told
    /// (`InvalidValue` warning where it fails): the legacy readers of the components 1.26.60
    /// types first read member by member, at every level. Otherwise the member or the
    /// element goes whole.
    const KEEP_NESTED: bool = false;
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
    pub(crate) fn invalid<T>(
        &mut self,
        at: &JsonPath,
        field: &mut T,
        default: T,
        text: Option<VanillaText>,
    ) {
        match self.mode {
            ConvertMode::Strict => {
                self.failed = true;
                self.push(DiagnosticCode::InvalidValue { text }, Severity::Error, at);
            }
            ConvertMode::Lenient => {
                *field = default;
                // The legacy readers check no bound: the typed struct's rule is the crate's own, an
                // information. Every caller passes no text (the game writes no content-log line for
                // this).
                self.push(DiagnosticCode::InvalidValue { text }, Severity::Info, at);
            }
        }
    }

    /// A member the typed read refuses, with the game's text where the generic read has none:
    /// `code` as an error that drops the component (the strict read only; the caller checks the
    /// mode).
    pub(crate) fn refuse(&mut self, code: DiagnosticCode, at: &JsonPath) {
        self.failed = true;
        self.push(code, Severity::Error, at);
    }

    /// A broken rule at `at` with no field default to fall back to: an error that drops the
    /// component in a strict read, an information of the crate's own in the lenient conversion
    /// ([`ComponentCheck::invalid`]).
    pub(crate) fn broken(&mut self, at: &JsonPath, text: Option<VanillaText>) {
        let mut unit = ();
        self.invalid(at, &mut unit, (), text);
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
            self.invalid(at, field, default, None);
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
            self.invalid(at, field, default, None);
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
    pub(crate) fn push(&mut self, code: DiagnosticCode, severity: Severity, at: &JsonPath) {
        self.out
            .push(Diagnostic::error(code).at(at.clone()).severity(severity));
    }

    /// A kept error at `at`: a level-3 line of the game after which the entity loads: the game logs
    /// this at the error level and keeps the document, a kept error.
    pub(crate) fn kept(&mut self, code: DiagnosticCode, at: &JsonPath) {
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
                        self.refuse(DiagnosticCode::UnrecognizedField { text: None }, &here);
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
    let json = bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text).ok();
    not_null(json.as_ref(), d)?;
    let shorthand = json
        .as_ref()
        .filter(|json| !json.is_object())
        .and_then(|json| T::shorthand(json, mode));
    let as_written = shorthand.is_none();
    if shorthand.is_some()
        && json
            .as_ref()
            .and_then(Value::as_string)
            .is_some_and(|text| !text.as_str().is_empty())
    {
        // The legacy parser reads any string as the component of defaults; vanilla's own
        // files write `""`. A string that says something is read the same, and told
        // (an information: the game does it without a word).
        d.push(Diagnostic::info(DiagnosticCode::InvalidType { text: None }));
    }
    let Some(object) = shorthand.or_else(|| json.as_ref().and_then(Value::as_object).cloned())
    else {
        if matches!(mode, ConvertMode::Lenient) {
            // a value that is not an object is dropped with `InvalidValue`, an information of the
            // crate's own (the game writes no content-log line for this): the legacy readers take
            // the members through const `Json::Value` access with defaults, and no line of the game
            // says it.
            d.push(Diagnostic::info(DiagnosticCode::InvalidValue {
                text: None,
            }));
            return None;
        }
        // Not an object (the typed read, strict): the typed read names the problem (a syntax
        // error, a wrong type).
        match crate::typed::read::<T>(slice, None) {
            Err(error) => d.push(crate::typed::finding(&error, Some(slice), Severity::Error)),
            Ok(_) => d.push(Diagnostic::error(DiagnosticCode::InvalidType {
                text: None,
            })),
        }
        return None;
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
    let changed = if older || changed {
        let (prepared, any) = without_null_members::<T>(object, d);
        object = prepared;
        changed || any
    } else {
        changed
    };
    // The lenient conversion reads the JSON as vanilla writes it back (the int/real tag lost).
    let changed = changed || !as_written || matches!(mode, ConvertMode::Lenient);
    let text = if changed {
        written::<T>(&object, mode, d)?
    } else {
        slice.text.to_owned()
    };
    // Spans hold only for the text as written.
    let at = JsonSlice {
        text: &text,
        offset: slice.offset,
    };
    let mut value = match (crate::typed::read::<T>(at, Some(NullRule::Absent)), mode) {
        (Ok(value), _) => value,
        (Err(error), ConvertMode::Strict) => {
            d.push(crate::typed::finding(
                &error,
                (!changed).then_some(at),
                Severity::Error,
            ));
            return None;
        }
        (Err(_), ConvertMode::Lenient) => super::lenient::isolate::<T>(&mut object, d)?,
    };
    let source = Value::from(object);
    let mut kept = bedrock_addon_core::document::Unknown::new();
    let (unknown, misread) = crate::read::keep_unknown(&value, &source, &mut kept);
    // Below the component (its caller places them).
    d.unknown_mut().absorb(&JsonPath::new(), kept);
    for (path, _) in unknown {
        // The game logs an unknown member at level 2 (a warning) for the legacy slice; the
        // strict slice from 1.21.89 makes it an error: the slice's grade.
        d.push(
            Diagnostic::warn(DiagnosticCode::UnrecognizedField { text: None })
                .severity(cx.grade(Class::Unknown))
                .at(path.clone()),
        );
    }
    // A value of another JSON type: the typed read fails it and the entity is
    // refused; the legacy readers' `as*`
    // convert it in silence, the crate's own.
    let misread_severity = match mode {
        ConvertMode::Strict => Severity::Error,
        ConvertMode::Lenient => Severity::Info,
    };
    for path in misread {
        d.push(
            Diagnostic::error(DiagnosticCode::InvalidType { text: None })
                .at(path.clone())
                .severity(misread_severity),
        );
    }
    let mut check = ComponentCheck::new(mode, cx, d);
    value.check(&mut check);
    (!check.failed).then_some(value)
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

/// `object` as JSON text: in the lenient conversion as facet writes a `Value` (an integral real
/// becomes an integer, as in vanilla's conversion of a `Dynamic` value), else with the number
/// tags kept.
///
/// A real negative zero keeps its sign in the lenient text (`-0.0`) wherever `T` reads the
/// text so: a float member then holds the zero of the file. Where `T` does not (the zero is
/// at an integer member, which takes the integer `0` the plain conversion writes), the plain
/// text is the one.
pub(super) fn json_text<T: Typed>(object: &VObject, mode: ConvertMode) -> Option<String> {
    let value = Value::from(object.clone());
    match mode {
        ConvertMode::Strict => to_json(&value).ok(),
        ConvertMode::Lenient => bedrock_addon_core::json::signed_zero::text_for::<T>(&value),
    }
}

/// [`json_text`] for a read that gives the component up when its JSON does not write (it
/// nests past `MAX_JSON_DEPTH`, which no file read through the door does): `InvalidValue`
/// at the component, an information in the lenient conversion, which drops it, and an error
/// in a strict read.
pub(super) fn written<T: Typed>(
    object: &VObject,
    mode: ConvertMode,
    d: &mut Diagnostics,
) -> Option<String> {
    let text = json_text::<T>(object, mode);
    if text.is_none() {
        // The lenient conversion's drop is the crate's own: an information (the game writes no
        // content-log line for this).
        let severity = match mode {
            ConvertMode::Strict => Severity::Error,
            ConvertMode::Lenient => Severity::Info,
        };
        d.push(Diagnostic::error(DiagnosticCode::InvalidValue { text: None }).severity(severity));
    }
    text
}

#[cfg(test)]
mod tests {
    use bedrock_addon_core::json::MAX_JSON_DEPTH;

    use super::*;

    /// An object whose member `zz` is arrays in arrays: `levels` levels in all.
    fn nested(levels: usize) -> VObject {
        let mut value = Value::from(0);
        for _ in 1..levels {
            let mut list = facet_value::VArray::new();
            list.push(value);
            value = list.into();
        }
        let mut object = VObject::new();
        object.insert("zz", value);
        object
    }

    /// The findings as `<code> <severity> <path>`.
    fn told(d: &Diagnostics) -> Vec<String> {
        d.iter()
            .map(|f| {
                format!(
                    "{} {:?} {}",
                    f.code.name(),
                    f.severity,
                    f.location.json_path
                )
            })
            .collect()
    }

    /// a component whose JSON does not write (it nests past
    /// the limit; no pack reaches it today, the file is refused first) went without a word.
    /// The lenient conversion drops it with an information of the crate's own, a strict read
    /// refuses it.
    #[test]
    fn a_component_whose_json_does_not_write_is_told_at_the_component() {
        for (mode, severity) in [
            (ConvertMode::Lenient, "Info"),
            (ConvertMode::Strict, "Error"),
        ] {
            let mut d = Diagnostics::default();
            let text = written::<crate::component::older::ApplyKnockbackRulesV1_8_0>(
                &nested(MAX_JSON_DEPTH),
                mode,
                &mut d,
            );
            assert!(text.is_some_and(|text| text.contains("\"zz\"")), "{mode:?}");
            assert!(d.is_empty(), "{mode:?}: {d:#?}");
            let text = written::<crate::component::older::ApplyKnockbackRulesV1_8_0>(
                &nested(MAX_JSON_DEPTH + 1),
                mode,
                &mut d,
            );
            assert_eq!(text, None, "{mode:?}");
            assert_eq!(told(&d), [format!("InvalidValue {severity} ")], "{mode:?}");
        }
    }
}
