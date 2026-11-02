//! The typed read of one entity component: the `parse` of every typed row of
//! [`EntityComponent`]'s table.
//!
//! A document at or above a component's typed version is read strictly: a member of the wrong
//! type or outside its range is an error and the component is dropped. A component converted
//! from an older document's JSON (`ConvertMode::Lenient`, the lenient conversion) keeps going: a
//! bad member is a warning and takes its default, and only a value that is not an object drops
//! the component. Unknown members are kept in the `extra` of their level with an
//! `UnrecognizedField` warning in both modes.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, JsonPathSegment, Location, Phase, Severity,
    Span, Told, VanillaText, code_for_deserialize_error_kind,
};
use bedrock_addon_core::json::{JsonSlice, SliceError, Value, parse_at, to_json};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::versioning::ConvertMode;
use facet::Facet;
use facet_value::{VArray, VObject};

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
    fn prepare(_json: &mut VObject, _cx: &mut Check<'_>) -> bool {
        false
    }

    /// The rule checks on the read value (ranges, enum spellings, the shared types' own
    /// findings), reported through `cx`.
    fn check(&mut self, _cx: &mut Check<'_>) {}

    /// Whether the lenient conversion keeps a member, or a list element, whose nested member
    /// it cannot read, that nested member taken out (so it takes its default) and told
    /// (`InvalidValue` warning where it fails): the legacy readers of the components 1.26.60
    /// types first read member by member, at every level. Otherwise the member or the
    /// element goes whole.
    const KEEP_NESTED: bool = false;
}

/// Where a component's rule checks report: the mode decides what a broken rule does.
pub(crate) struct Check<'d> {
    mode: ConvertMode,
    #[cfg_attr(
        not(feature = "molang"),
        allow(dead_code, reason = "only the lenient Molang compile reads it")
    )]
    load: &'d LoadContext,
    out: &'d mut Diagnostics,
    failed: bool,
}

impl<'d> Check<'d> {
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
    /// ([`Check::invalid`]).
    pub(crate) fn broken(&mut self, at: &JsonPath, text: Option<VanillaText>) {
        let mut unit = ();
        self.invalid(at, &mut unit, (), text);
    }

    /// The schema bound `min ..= max` of a number at `at` ([`Check::invalid`] outside it).
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

    /// The schema minimum of a number at `at` ([`Check::invalid`] below it).
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

    /// A finding that does not depend on the mode (a vanilla warning; the value stays).
    pub(crate) fn push(&mut self, code: DiagnosticCode, severity: Severity, at: &JsonPath) {
        self.out.push(Diagnostic::new(
            code,
            severity,
            Phase::Parse,
            Location::new(None, None, at.clone(), None),
        ));
    }

    /// A kept error at `at`: a level-3 line of the game after which the entity loads: the game logs
    /// this at the error level and keeps the document, a kept error.
    pub(crate) fn kept(&mut self, code: DiagnosticCode, at: &JsonPath) {
        self.out.push_kept_error(Diagnostic::new(
            code,
            Severity::Error,
            Phase::Parse,
            Location::new(None, None, at.clone(), None),
        ));
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
                        let mut here = at.clone();
                        here.push(end);
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
pub(crate) fn drop_deprecated(json: &mut VObject, names: &[&str], cx: &mut Check<'_>) -> bool {
    let mut changed = false;
    for name in names {
        if json.remove(name).is_some() {
            changed = true;
            cx.out
                .push(Diagnostic::deprecated_member(name, path(&[name])));
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
    cx: &mut Check<'_>,
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
            cx.moved(join(at, alias), join(at, member));
            changed = true;
        }
    }
    changed
}

/// `keys` as a path relative to the component.
pub(crate) fn path(keys: &[&str]) -> JsonPath {
    let mut at = JsonPath::new();
    for key in keys {
        at.push(key);
    }
    at
}

/// `base` followed by `key`.
pub(crate) fn join(base: &JsonPath, key: &str) -> JsonPath {
    let mut at = base.clone();
    at.push(key);
    at
}

/// `base` followed by the element `index`.
pub(crate) fn index(base: &JsonPath, index: usize) -> JsonPath {
    let mut at = base.clone();
    at.push_index(index);
    at
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
    before: impl FnOnce(&mut VObject, &mut Check<'_>) -> bool,
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
    before: impl FnOnce(&mut VObject, &mut Check<'_>) -> bool,
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
        d.push(finding(
            DiagnosticCode::InvalidType { text: None },
            Severity::Info,
            &JsonPath::new(),
        ));
    }
    let Some(object) = shorthand.or_else(|| json.as_ref().and_then(Value::as_object).cloned())
    else {
        if matches!(mode, ConvertMode::Lenient) {
            // a value that is not an object is dropped with `InvalidValue`, an information of the
            // crate's own (the game writes no content-log line for this): the legacy readers take
            // the members through const `Json::Value` access with defaults, and no line of the game
            // says it.
            d.push(finding(
                DiagnosticCode::InvalidValue { text: None },
                Severity::Info,
                &JsonPath::new(),
            ));
            return None;
        }
        // Not an object (the typed read, strict): the typed read names the problem (a syntax
        // error, a wrong type).
        match parse_at::<T>(slice) {
            Err(error) => d.push(slice_finding(&error, slice, true, Severity::Error)),
            Ok(_) => d.push(finding(
                DiagnosticCode::InvalidType { text: None },
                Severity::Error,
                &JsonPath::new(),
            )),
        }
        return None;
    };
    // The game reads an entity through JsonCpp: a member written as `null` is absent and
    // keeps its default, at every version (a `null` is an absent member).
    let (mut object, _) = without_null_members::<T>(object, d);
    let mut prepare = Check::new(mode, cx, d);
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
    let mut value = match (
        bedrock_addon_core::json::parse_at_null_absent::<T>(at),
        mode,
    ) {
        (Ok(value), _) => value,
        (Err(error), ConvertMode::Strict) => {
            d.push(slice_finding(&error, at, !changed, Severity::Error));
            return None;
        }
        (Err(_), ConvertMode::Lenient) => isolate::<T>(&mut object, d)?,
    };
    let source = Value::from(object);
    let (unknown, misread) = crate::read::keep_unknown(&mut value, &source);
    for (path, _) in unknown {
        // The game logs an unknown member at level 2 (a warning) for the legacy slice; the
        // strict slice from 1.21.89 makes it an error.
        d.push(finding(
            DiagnosticCode::UnrecognizedField { text: None },
            Severity::Warning,
            &path,
        ));
    }
    // A value of another JSON type: the typed read fails it and the entity is
    // refused; the legacy readers' `as*`
    // convert it in silence, the crate's own.
    let misread_severity = match mode {
        ConvertMode::Strict => Severity::Error,
        ConvertMode::Lenient => Severity::Info,
    };
    for path in misread {
        d.push(finding(
            DiagnosticCode::InvalidType { text: None },
            misread_severity,
            &path,
        ));
    }
    let mut check = Check::new(mode, cx, d);
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

/// The lenient conversion's member isolation: each member read alone, a failing one reported
/// (`InvalidValue` warning) and removed so its field takes its default, then the rest read
/// together.
fn isolate<T: Typed>(object: &mut VObject, d: &mut Diagnostics) -> Option<T> {
    let keys: Vec<String> = object.keys().map(|k| k.as_str().to_owned()).collect();
    for key in keys {
        let Some(member) = object.get(&key) else {
            continue;
        };
        let mut alone = VObject::new();
        alone.insert(key.as_str(), member.clone());
        let text = written::<T>(&alone, ConvertMode::Lenient, d)?;
        if let Err(error) = parse_at::<T>(JsonSlice {
            text: &text,
            offset: 0,
        }) {
            if isolate_elements::<T>(object, &key, d) {
                continue;
            }
            if T::KEEP_NESTED
                && let Some((kept, removed)) = strip_nested::<T>(&alone)
            {
                for at in removed {
                    d.push(finding(
                        DiagnosticCode::InvalidValue { text: None },
                        // The lenient isolation's notes are the crate's own (the game writes no
                        // content-log line for this): the legacy readers keep what they read in silence.
                        Severity::Info,
                        &at,
                    ));
                }
                if let Some(member) = kept.get(&key) {
                    object.insert(key.as_str(), member.clone());
                }
                continue;
            }
            let at = if error.path.is_empty() {
                path(&[&key])
            } else {
                error.path
            };
            let nested = at.len() > 1;
            d.push(finding(
                DiagnosticCode::InvalidValue { text: None },
                // The crate's own, as above.
                Severity::Info,
                &at,
            ));
            if nested {
                // The member goes whole for the value inside it: told at the member too, so
                // what else it holds is known to be dropped (an information, the warning is
                // the one above).
                d.push(finding(
                    DiagnosticCode::DroppedField { text: None },
                    Severity::Info,
                    &path(&[&key]),
                ));
            }
            object.remove(&key);
        }
    }
    let text = written::<T>(object, ConvertMode::Lenient, d)?;
    match parse_at::<T>(JsonSlice {
        text: &text,
        offset: 0,
    }) {
        Ok(value) => Some(value),
        Err(error) => {
            d.push(slice_finding(
                &error,
                JsonSlice {
                    text: &text,
                    offset: 0,
                },
                false,
                // The crate's own, as above.
                Severity::Info,
            ));
            None
        }
    }
}

/// [`isolate`] for a member that is a list: each element read alone, a failing one
/// reported (`InvalidValue` warning where it fails, and `DroppedField` INFO at the element
/// when the failure is inside it) and left out, the others kept. `false`, with nothing told
/// or changed, when the member is no list, no element fails alone, or the elements left do
/// not read together: the member then goes whole, as any other.
fn isolate_elements<T: Typed>(object: &mut VObject, key: &str, d: &mut Diagnostics) -> bool {
    let Some(list) = object.get(key).and_then(Value::as_array) else {
        return false;
    };
    let reads = |elements: VArray| {
        let mut alone = VObject::new();
        alone.insert(key, elements);
        let text = json_text::<T>(&alone, ConvertMode::Lenient)?;
        Some(
            parse_at::<T>(JsonSlice {
                text: &text,
                offset: 0,
            })
            .map(|_| ()),
        )
    };
    let mut kept = VArray::new();
    let mut told = Vec::new();
    let mut moves = Vec::new();
    for (place, element) in list.iter().enumerate() {
        let mut one = VArray::new();
        one.push(element.clone());
        let at_element = index(&path(&[key]), place);
        match reads(one) {
            None => return false,
            Some(Ok(())) => {
                if kept.len() != place {
                    moves.push((at_element, index(&path(&[key]), kept.len())));
                }
                kept.push(element.clone());
            }
            Some(Err(_)) if T::KEEP_NESTED && salvage::<T>(key, element).is_some() => {
                let Some((salvaged, removed)) = salvage::<T>(key, element) else {
                    continue;
                };
                told.extend(removed.iter().map(|at| element_finding(key, place, at)));
                if kept.len() != place {
                    moves.push((at_element, index(&path(&[key]), kept.len())));
                }
                kept.push(salvaged);
            }
            Some(Err(error)) => {
                // The failure's path names the element as the first of its list of one.
                // Below the element it is followed as far as the file has a value: a
                // failure named deeper (the element `0` of an empty list) is of the value
                // the file has there.
                let mut at = at_element.clone();
                let mut inside = false;
                let mut node = Some(element);
                for segment in error.path.segments().skip(2) {
                    node = match segment {
                        JsonPathSegment::Key(name) => node
                            .and_then(Value::as_object)
                            .and_then(|object| object.get(name)),
                        JsonPathSegment::Index(number) => node
                            .and_then(Value::as_array)
                            .and_then(|array| array.get(number)),
                    };
                    if node.is_none() {
                        break;
                    }
                    inside = true;
                    match segment {
                        JsonPathSegment::Key(name) => at.push(name),
                        JsonPathSegment::Index(number) => at.push_index(number),
                    }
                }
                told.push(finding(
                    DiagnosticCode::InvalidValue { text: None },
                    // The crate's own, as above.
                    Severity::Info,
                    &at,
                ));
                if inside {
                    told.push(finding(
                        DiagnosticCode::DroppedField { text: None },
                        Severity::Info,
                        &at_element,
                    ));
                }
            }
        }
    }
    if told.is_empty() || !matches!(reads(kept.clone()), Some(Ok(()))) {
        return false;
    }
    for finding in told {
        d.push(finding);
    }
    for (from, to) in moves {
        d.moved(from, to);
    }
    object.insert(key, kept);
    true
}

/// The `InvalidValue` warning at `at`, a path that names the element `place` of the list `key`
/// as the first of its list of one, at the element the file has.
fn element_finding(key: &str, place: usize, at: &JsonPath) -> Diagnostic {
    let mut here = path(&[key]);
    here.push_index(place);
    for segment in at.segments().skip(2) {
        match segment {
            JsonPathSegment::Key(name) => here.push(name),
            JsonPathSegment::Index(number) => here.push_index(number),
        }
    }
    finding(
        DiagnosticCode::InvalidValue { text: None },
        // The crate's own, as above.
        Severity::Info,
        &here,
    )
}

/// [`strip_nested`] of the element `element` of the list `key`, read as the one element of its
/// list: the element with the nested members taken out, and their paths (below the list of
/// one), or `None` when it cannot be read so.
fn salvage<T: Typed>(key: &str, element: &Value) -> Option<(Value, Vec<JsonPath>)> {
    let mut one = VArray::new();
    one.push(element.clone());
    let mut alone = VObject::new();
    alone.insert(key, one);
    let (kept, removed) = strip_nested::<T>(&alone)?;
    let element = kept.get(key)?.as_array()?.get(0)?.clone();
    Some((element, removed))
}

/// Takes out of `alone` (one member of a component), one by one, each nested member at which a
/// lone read fails, until the read succeeds: the object left and the paths taken out (below the
/// component). `None` when a failure is not at a member of an object below the top level (the
/// member itself is of the wrong shape), or after 64 tries.
fn strip_nested<T: Typed>(alone: &VObject) -> Option<(VObject, Vec<JsonPath>)> {
    let mut object = alone.clone();
    let mut removed = Vec::new();
    for _ in 0..64 {
        let text = json_text::<T>(&object, ConvertMode::Lenient)?;
        let Err(error) = parse_at::<T>(JsonSlice {
            text: &text,
            offset: 0,
        }) else {
            return (!removed.is_empty()).then_some((object, removed));
        };
        let segments: Vec<JsonPathSegment<'_>> = error.path.segments().collect();
        let (JsonPathSegment::Key(name), parents) = segments.split_last()? else {
            return None;
        };
        if parents.is_empty() {
            return None;
        }
        let mut node: Option<&mut Value> = None;
        for (i, segment) in parents.iter().enumerate() {
            node = match (i, segment, node) {
                (0, JsonPathSegment::Key(key), _) => object.get_mut(key),
                (_, JsonPathSegment::Key(key), Some(value)) => {
                    value.as_object_mut().and_then(|o| o.get_mut(key))
                }
                (_, JsonPathSegment::Index(n), Some(value)) => {
                    value.as_array_mut().and_then(|a| a.get_mut(*n))
                }
                _ => None,
            };
        }
        node?.as_object_mut()?.remove(name)?;
        removed.push(error.path.clone());
    }
    None
}

/// `object` as JSON text: in the lenient conversion as facet writes a `Value` (an integral real
/// becomes an integer, as in vanilla's conversion of a `Dynamic` value), else with the number
/// tags kept.
///
/// A real negative zero keeps its sign in the lenient text (`-0.0`) wherever `T` reads the
/// text so: a float member then holds the zero of the file. Where `T` does not (the zero is
/// at an integer member, which takes the integer `0` the plain conversion writes), the plain
/// text is the one.
fn json_text<T: Typed>(object: &VObject, mode: ConvertMode) -> Option<String> {
    let value = Value::from(object.clone());
    match mode {
        ConvertMode::Strict => to_json(&value).ok(),
        ConvertMode::Lenient => bedrock_addon_support::signed_zero::text_for::<T>(&value),
    }
}

/// [`json_text`] for a read that gives the component up when its JSON does not write (it
/// nests past `MAX_JSON_DEPTH`, which no file read through the door does): `InvalidValue`
/// at the component, an information in the lenient conversion, which drops it, and an error
/// in a strict read.
fn written<T: Typed>(object: &VObject, mode: ConvertMode, d: &mut Diagnostics) -> Option<String> {
    let text = json_text::<T>(object, mode);
    if text.is_none() {
        // The lenient conversion's drop is the crate's own: an information (the game writes no
        // content-log line for this).
        let severity = match mode {
            ConvertMode::Strict => Severity::Error,
            ConvertMode::Lenient => Severity::Info,
        };
        d.push(finding(
            DiagnosticCode::InvalidValue { text: None },
            severity,
            &JsonPath::new(),
        ));
    }
    text
}

/// A finding at `at` relative to the component, without a span.
pub(crate) fn finding(code: DiagnosticCode, severity: Severity, at: &JsonPath) -> Diagnostic {
    Diagnostic::new(
        code,
        severity,
        Phase::Parse,
        Location::new(None, None, at.clone(), None),
    )
}

/// The finding of a failed typed read; the span only when the text is the file's.
fn slice_finding(
    error: &SliceError,
    slice: JsonSlice<'_>,
    as_written: bool,
    severity: Severity,
) -> Diagnostic {
    let blank = slice.text.trim().is_empty();
    let code = code_for_deserialize_error_kind(&error.error.kind, !error.path.is_empty(), blank);
    let span: Option<Span> = as_written.then_some(error.span);
    Diagnostic::new(
        code,
        severity,
        Phase::Parse,
        Location::new(None, None, error.path.clone(), span),
    )
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
