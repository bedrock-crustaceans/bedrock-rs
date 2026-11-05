//! The parse of one AI goal from its slice (the `parse` of every goal row of the component
//! table), strict or lenient, and the member checks every goal runs while it is read.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{
    JsonSlice, SliceError, StrictFloat, StrictInt, StrictIntType, Value, parse_at,
    parse_at_null_absent, remove_null_members, tell_null_members,
};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::shared::{
    BlockDescriptor, EventTrigger, Filter, ItemDescriptor, Range, RangeEnd, RangeForm,
};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::ConvertMode;
use facet::Facet;
use facet_value::{VArray, VObject};

use super::common::{EntityTypes, SoundEvent, check_entity_types};
use crate::component::EntityComponent;

/// Vanilla's message for a deprecated member: `member "<name>" is deprecated`.
macro_rules! deprecated_text {
    ($name:literal) => {
        concat!("member \"", $name, "\" is deprecated")
    };
}
pub(crate) use deprecated_text;

/// The newest format version: what a strict parse without a document version reads as.
pub(crate) const NEWEST: FormatVersion = known::V1_26_50;
/// 1.26.0: `aquatic_charge_attack` renames `charge_speed_multiplier`.
pub(crate) const V1_26_0: FormatVersion = known::V1_26_0;
/// 1.26.30: `pickup_items` gains `stop_if_holding_item`.
pub(crate) const V1_26_30: FormatVersion = known::V1_26_30;
/// 1.26.40: the melee goals bind `on_kill` on its own.
pub(crate) const V1_26_40: FormatVersion = known::V1_26_40;
/// 1.26.50: `ranged_attack` gains `attack_range` and `in_range_movement_mode`.
pub(crate) const V1_26_50: FormatVersion = NEWEST;

/// One deprecated inherited member: its name, vanilla's message and the version from which the
/// goal's struct deprecates it.
pub(crate) type Deprecated = (&'static str, &'static str, FormatVersion);

/// What every goal type tells the shared parse: its typed version, its deprecated and
/// version-gated members, the checks of its members, and its conversions of older shapes.
pub(crate) trait Goal: for<'a> Facet<'a> + Into<EntityComponent> + Default + Sized {
    /// The format version from which vanilla reads the goal with its typed struct: a document
    /// below it is converted leniently.
    const TYPED: FormatVersion;

    /// Inherited member names this goal's struct deprecates: a document at or above the
    /// version that writes one gets a `DeprecatedMember` warning and the value is not used.
    // The game logs a deprecated name as an error and ignores its value.
    const DEPRECATED: &'static [Deprecated] = &[];

    /// Members of the goal's older shape: the lenient parse converts them, and a document at or
    /// above the typed version that still writes one is an error that drops the goal.
    // Documents at the typed version or above must use the new shape; older ones are
    // converted.
    const REJECTED: &'static [&'static str] = &[];

    /// Members that exist only from a version: a document below it that writes one gets
    /// `UnrecognizedField` and the value is not used ([`Goal::forget`]).
    const GATED: &'static [(&'static str, FormatVersion)] = &[];

    /// The content-log texts the legacy goal parser writes when a
    /// top-level member of the goal does not read, and their level: the lenient conversion
    /// tells a member it gives up with this text and severity instead of its own.
    const LENIENT_TEXTS: &'static [(&'static str, &'static str, Severity)] = &[];

    /// The goal's unknown members.
    fn extra_mut(&mut self) -> &mut Extra;

    /// The lenient read gave up the [`Goal::LENIENT_TEXTS`] member `name`: what the legacy
    /// reader does after that failure. Returns the members it set back to their defaults that
    /// the goal was written with, each told as dropped. Nothing by default.
    fn legacy_member_failed(&mut self, _name: &str) -> Vec<&'static str> {
        Vec::new()
    }

    /// Checks every member: ranges, constraints and the shared types' own findings.
    fn check(&mut self, c: &mut Check<'_>);

    /// A deprecated member was written with `value`; the goal may carry it over to its
    /// replacement.
    fn deprecated(&mut self, name: &str, value: &Value) {
        let _ = (name, value);
    }

    /// Clears the gated member `name` (one of [`Goal::GATED`]).
    fn forget(&mut self, name: &str) {
        let _ = name;
    }

    /// The typed → typed conversions of a document read strictly at `from`, below a later
    /// redeclaration of the goal: members of the older struct are taken from `extra`.
    fn upgrade(&mut self, from: FormatVersion, d: &mut Diagnostics) {
        let _ = (from, d);
    }

    /// Rewrites an older document's JSON into this struct's shape before the lenient parse.
    fn lenient_shape(json: &mut VObject, d: &mut Diagnostics) {
        let _ = (json, d);
    }

    /// Rewrites the parsed value in the lenient path.
    fn lenient_value(&mut self) {}
}

/// How a goal is read: strictly, by a document at the given version (at or above the goal's
/// typed version), or leniently, converted from a document below it.
#[derive(Clone, Copy, Debug)]
enum Mode {
    Strict(FormatVersion),
    Lenient,
}

/// Parses one goal: the `parse` of a goal row.
///
/// `Strict` reads the goal as a document at the newest format version does; `Lenient` as the
/// conversion of a document below the goal's typed version. See [`read`].
pub(crate) fn parse<T: Goal>(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    _: &LoadContext,
    d: &mut Diagnostics,
) -> Option<EntityComponent> {
    let mode = match mode {
        ConvertMode::Strict => Mode::Strict(NEWEST),
        ConvertMode::Lenient => Mode::Lenient,
    };
    read::<T>(slice, mode, d).map(Into::into)
}

/// A parse of a goal by the document's format version: what the entity document's component
/// map calls for a goal key (see [`versioned`](super::versioned)).
pub(crate) type VersionedParseFn =
    fn(JsonSlice<'_>, FormatVersion, &LoadContext, &mut Diagnostics) -> Option<EntityComponent>;

/// Parses one goal of a document at `version`: leniently below the goal's typed version,
/// strictly at or above it, with the typed → typed conversions and member gates of the versions
/// between `version` and the newest.
pub(crate) fn parse_versioned<T: Goal>(
    slice: JsonSlice<'_>,
    version: FormatVersion,
    _: &LoadContext,
    d: &mut Diagnostics,
) -> Option<EntityComponent> {
    let mode = if version < T::TYPED {
        Mode::Lenient
    } else {
        Mode::Strict(version)
    };
    read::<T>(slice, mode, d).map(Into::into)
}

/// Reads a goal.
///
/// Strict (a document at or above the goal's typed version): a member of the wrong type is an
/// error and drops the goal, and so is a value outside its member's bounds. Lenient (an older
/// document converted in a step): such a member is an `InvalidValue` warning and takes its
/// default; the goal is dropped only when its value is not an object. Unknown members are kept
/// and reported as `UnrecognizedField` warnings either way.
fn read<T: Goal>(slice: JsonSlice<'_>, mode: Mode, d: &mut Diagnostics) -> Option<T> {
    let (mut value, json, version) = match mode {
        Mode::Strict(version) => {
            // The game reads an entity through jsoncpp, where a member written as `null` is
            // absent; the write-back writes nothing there, so each such place carries an
            // information.
            let value = match parse_at_null_absent::<T>(slice) {
                Ok(value) => value,
                Err(error) => {
                    d.push(slice_error(&error, Severity::Error));
                    return None;
                }
            };
            let mut json = bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text)
                .unwrap_or(Value::NULL);
            let absent = remove_null_members(T::SHAPE, &mut json);
            tell_null_members(&absent, &JsonPath::new(), d);
            (value, json, Some(version))
        }
        Mode::Lenient => {
            let (value, json) = lenient::<T>(slice, d)?;
            (value, json, None)
        }
    };
    let (unknown, misread) = crate::read::keep_unknown(&mut value, &json);
    for path in misread {
        d.push(
            // The game writes no content-log line for this, so the crate reports it as an
            // information: facet's coercion, which the readers' `as*` make in silence.
            Diagnostic::info(DiagnosticCode::InvalidType { text: None }).at(path),
        );
    }
    if let Some(version) = version {
        for (name, since) in T::GATED {
            if version < *since && json.as_object().is_some_and(|o| o.get(name).is_some()) {
                value.forget(name);
                d.push(
                    // The game logs an unrecognised member of a legacy-format goal as a
                    // warning; from 1.21.89 it refuses the document.
                    Diagnostic::warn(DiagnosticCode::UnrecognizedField { text: None })
                        .at(JsonPath::new().child(name)),
                );
            }
        }
        value.upgrade(version, d);
    }
    let mut rejected = false;
    for (path, member) in unknown {
        // The member's own key, as the file has it, when the path is a member of the goal.
        let top = path.len() == 1;
        let name = path
            .last_key()
            .filter(|_| top)
            .unwrap_or_default()
            .to_owned();
        if top && value.extra_mut().get(&name).is_none() {
            // Taken by a conversion.
            continue;
        }
        let deprecated = top
            .then(|| {
                T::DEPRECATED
                    .iter()
                    .find(|(n, _, since)| *n == name && version.is_none_or(|v| v >= *since))
            })
            .flatten();
        if top && version.is_some() && T::REJECTED.contains(&name.as_str()) {
            rejected = true;
            d.push(Diagnostic::error(DiagnosticCode::InvalidValue { text: None }).at(path));
        } else if let Some((name, text, _)) = deprecated {
            value.extra_mut().remove(name);
            // The game logs a deprecated name as an error that rejects the document in a
            // typed read; the lenient conversion of an older document, whose struct may still
            // bind the name, is only told.
            let severity = if version.is_some() {
                Severity::Error
            } else {
                // The lenient conversion: the legacy reader knows no deprecation and writes no
                // content-log line, so the crate reports an information.
                Severity::Info
            };
            d.push(
                Diagnostic::error(DiagnosticCode::DeprecatedMember {
                    text: Some(VanillaText::Fixed(text)),
                })
                .at(path)
                .severity(severity),
            );
            value.deprecated(name, &member);
        } else {
            d.push(
                // The game logs an unrecognised member of a legacy-format goal as a warning;
                // from 1.21.89 it refuses the document.
                Diagnostic::warn(DiagnosticCode::UnrecognizedField { text: None }).at(path),
            );
        }
    }
    if version.is_none() {
        value.lenient_value();
    }
    let mut check = Check::new(version.is_some(), d);
    value.check(&mut check);
    if check.failed || rejected {
        return None;
    }
    Some(value)
}

/// The severity of a broken rule in the lenient read: an information, with or without the
/// game's text: the game writes no content-log line for these, because the texts are
/// cereal's constraints, which the legacy readers do not check. A line the game logs whatever the reader is told with
/// [`Check::kept`].
fn lenient_severity(text: Option<&VanillaText>) -> Severity {
    let _ = text;
    Severity::Info
}

/// The lenient parse: the value is read from its JSON again, each round giving up the innermost
/// member (or array element) a failure names, which then takes its default. A failure that
/// names no member gives the goal's defaults. Returns the value and the JSON it was read from.
/// A [`Goal::LENIENT_TEXTS`] member the read gives up is passed to
/// [`Goal::legacy_member_failed`].
fn lenient<T: Goal>(slice: JsonSlice<'_>, d: &mut Diagnostics) -> Option<(T, Value)> {
    let mut failed: Vec<&'static str> = Vec::new();
    let json = bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text).ok();
    let Some(object) = json.as_ref().and_then(Value::as_object).cloned() else {
        // A goal whose value is not an object is dropped.
        d.push(
            // The game writes no content-log line for this (the legacy reader takes
            // defaults), so the crate reports it as an information.
            Diagnostic::info(DiagnosticCode::InvalidValue { text: None }),
        );
        return None;
    };
    // A `null` member is absent, as in the strict read.
    let mut without_nulls = Value::from(object);
    let absent = remove_null_members(T::SHAPE, &mut without_nulls);
    tell_null_members(&absent, &JsonPath::new(), d);
    let mut object = without_nulls.as_object().cloned().unwrap_or_default();
    for (name, _) in T::GATED {
        if object.remove(name).is_some() {
            d.push(
                // The lenient conversion: the legacy reader reads only what it knows, in
                // silence, so the crate reports an information.
                Diagnostic::info(DiagnosticCode::UnrecognizedField { text: None })
                    .at(JsonPath::new().child(name)),
            );
        }
    }
    T::lenient_shape(&mut object, d);
    let mut json = Value::from(object);
    // The elements each list still has, by the indices the goal was written with: a round
    // that takes an element out moves the ones behind it up.
    let mut lists = Lists::default();
    // Each round removes one value of the JSON, so the loop ends.
    loop {
        // (A real negative zero keeps its sign where the goal reads the text with it.)
        let text = lenient_text::<T>(&json, d)?;
        match parse_at::<T>(JsonSlice::new(&text)) {
            Ok(mut value) => {
                lists.record(d);
                for name in failed {
                    // Members the legacy reader never reaches: the game is silent, the crate
                    // tells each one written as dropped.
                    for unread in value.legacy_member_failed(name) {
                        d.push(
                            Diagnostic::info(DiagnosticCode::DroppedField { text: None })
                                .at(JsonPath::new().child(unread)),
                        );
                    }
                }
                return Some((value, json));
            }
            Err(error) => {
                // The innermost value the failure names that the JSON HAS: a failure named
                // below it (the element `0` of an empty list) is the member's.
                let mut steps = path_steps(&error.path);
                while !steps.is_empty() && !has_at(&json, &steps) {
                    steps.pop();
                }
                // Told where the goal was written with the value, not where the rounds
                // before this one have left it; with the legacy reader's text where it has
                // one for the member.
                let at = lists.as_written(&json, &steps);
                let legacy = (steps.len() == 1)
                    .then(|| T::LENIENT_TEXTS.iter().find(|(name, ..)| *name == steps[0]))
                    .flatten();
                if let Some((name, ..)) = legacy {
                    failed.push(name);
                }
                d.push(
                    // Without a row of the game's text the game writes no line, so the crate
                    // reports an information; a row with a text carries its own level.
                    Diagnostic::error(DiagnosticCode::InvalidValue {
                        text: legacy.map(|(_, text, _)| VanillaText::Fixed(text)),
                    })
                    .at(at)
                    .severity(legacy.map_or(Severity::Info, |(.., severity)| *severity)),
                );
                if steps.is_empty() || !lists.remove(&mut json, &steps) {
                    return Some((T::default(), Value::from(VObject::new())));
                }
            }
        }
    }
}

/// The legacy readers of every goal read a range object through `range_min` / `range_max`
/// only, whether the goal is read by a schema reader or by the legacy goal parser, so the
/// `min` / `max` of a member of the goal as the legacy reader meets it (an object that has
/// either) are not read. They are taken out and
/// their paths (below the goal) returned, for an information each; a range left empty goes, the
/// member keeping its default. `range_*` stay. Run on the goal as the step to its typed version
/// starts, before that step's own conversions (which write the typed `{min, max}`).
pub(crate) fn legacy_ranges(key: &str, goal: &mut VObject) -> Vec<(JsonPath, DiagnosticCode)> {
    let mut dropped = Vec::new();
    // A string `look_time` of the look goals is read by the game's integer range parser, which
    // `parse_range_string` mirrors.
    if PARSE_RANGE_LOOK_TIME.contains(&key)
        && let Some(text) = goal.get("look_time").and_then(Value::as_string)
    {
        let (min, max) = parse_range_string(text.as_str()).unwrap_or((2, 4));
        let mut range = VObject::new();
        range.insert("min", Value::from(i64::from(min)));
        range.insert("max", Value::from(i64::from(max)));
        goal.insert("look_time", Value::from(range));
        dropped.push((
            JsonPath::new().child("look_time"),
            DiagnosticCode::ValueRewritten { text: None },
        ));
    }
    let names: Vec<String> = goal.keys().map(|key| key.as_str().to_owned()).collect();
    for name in names {
        // The look goals' `look_time` is read by the game's integer range parser, which reads
        // `min` and `max` and never `range_*`.
        let unread: [&str; 2] = if name == "look_time" && PARSE_RANGE_LOOK_TIME.contains(&key) {
            ["range_min", "range_max"]
        } else {
            ["min", "max"]
        };
        let Some(range) = goal.get_mut(&name).and_then(Value::as_object_mut) else {
            continue;
        };
        if !unread.iter().any(|end| range.contains_key(end)) {
            continue;
        }
        for end in unread {
            if range.remove(end).is_some() {
                let mut at = JsonPath::new().child(&name);
                at.push(end);
                dropped.push((at, DiagnosticCode::DroppedField { text: None }));
            }
        }
        if range.is_empty() {
            goal.remove(&name);
        }
    }
    dropped
}

/// The game's integer range parser for a string `look_time`. With a `[` and a `]` after it,
/// the text from after the `[` (its length is `]`'s offset minus one bytes, at most to the end
/// of the string) is split at `,` without the empty parts, and the first part, a space and the
/// second part (when there are two) are read through a string stream as `min`, `max = min`,
/// `max`, swapped when `min > max`. Without a `[` the whole string is read for `min` and
/// `max = min`. A `[` without a `]` fails (`None`; the goal then takes (2, 4)). An inner text
/// of no part (`"[]"`) indexes an empty list in the game, whose result is undefined; it is
/// read here as no number, (0, 0).
fn parse_range_string(text: &str) -> Option<(i32, i32)> {
    // The game works on the bytes of its strings; so does this, so the game's substring
    // length (which can end inside a UTF-8 character) never splits a `&str`.
    let bytes = text.as_bytes();
    let mut min = 0_i32;
    let Some(open) = bytes.iter().position(|b| *b == b'[') else {
        let mut stream = IntStream::new(bytes);
        stream.read(&mut min);
        return Some((min, min));
    };
    let close = bytes[open..].iter().position(|b| *b == b']')? + open;
    let start = open + 1;
    let count = (bytes.len() - start).min(close.saturating_sub(1));
    let inner = &bytes[start..start + count];
    let parts: Vec<&[u8]> = inner
        .split(|b| *b == b',')
        .filter(|part| !part.is_empty())
        .collect();
    let line: Vec<u8> = match parts.as_slice() {
        [] => Vec::new(),
        [one] => one.to_vec(),
        [first, second, ..] => [*first, b" ", *second].concat(),
    };
    let mut stream = IntStream::new(&line);
    stream.read(&mut min);
    let mut max = min;
    stream.read(&mut max);
    if min > max {
        std::mem::swap(&mut min, &mut max);
    }
    Some((min, max))
}

/// Reading an int from `text` as a stream does: leading white space
/// skipped; a failed sentry (the stream not good, or at its end) leaves the value; no digits
/// stores 0 and fails; digits beyond the type store its bound and fail; reaching the end sets
/// eof, so the next read leaves its value.
struct IntStream<'a> {
    bytes: &'a [u8],
    at: usize,
    good: bool,
}

impl<'a> IntStream<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            at: 0,
            good: true,
        }
    }

    fn read(&mut self, out: &mut i32) {
        if !self.good {
            return;
        }
        while self.at < self.bytes.len() && self.bytes[self.at].is_ascii_whitespace() {
            self.at += 1;
        }
        if self.at == self.bytes.len() {
            self.good = false;
            return;
        }
        let start = self.at;
        if matches!(self.bytes[self.at], b'+' | b'-') {
            self.at += 1;
        }
        let digits = self.at;
        while self.at < self.bytes.len() && self.bytes[self.at].is_ascii_digit() {
            self.at += 1;
        }
        if self.at == digits {
            *out = 0;
            self.good = false;
            return;
        }
        let text = std::str::from_utf8(&self.bytes[start..self.at]).unwrap_or("0");
        if let Ok(value) = text.parse::<i32>() {
            *out = value;
        } else {
            *out = if text.starts_with('-') {
                i32::MIN
            } else {
                i32::MAX
            };
            self.good = false;
        }
        if self.at == self.bytes.len() {
            self.good = false;
        }
    }
}

/// The goals whose `look_time` the legacy goal parser reads with the game's integer range
/// parser (a string such as `"[1,3]"`, or a `{min, max}` object).
const PARSE_RANGE_LOOK_TIME: [&str; 7] = [
    "minecraft:behavior.beg",
    "minecraft:behavior.look_at_entity",
    "minecraft:behavior.look_at_player",
    "minecraft:behavior.look_at_target",
    "minecraft:behavior.look_at_trading_player",
    "minecraft:behavior.random_look_around",
    "minecraft:behavior.random_look_around_and_sit",
];

/// Where the elements of the lists of a goal's JSON were before the lenient parse took
/// elements out: for each list that lost one, by its path as the goal was written, the
/// indices (as written) of the elements it still has.
#[derive(Default)]
struct Lists(Vec<(JsonPath, Vec<usize>)>);

impl Lists {
    /// The indices as written of the elements the list written at `list` still has.
    fn of(&self, list: &JsonPath) -> Option<&[usize]> {
        self.0
            .iter()
            .find(|(at, _)| at == list)
            .map(|(_, kept)| kept.as_slice())
    }

    /// The path the value at `steps` of `json` (as the rounds so far have left it) had in
    /// the goal as written.
    fn as_written(&self, json: &Value, steps: &[String]) -> JsonPath {
        let mut at = JsonPath::new();
        let mut node = Some(json);
        for step in steps {
            let index = node
                .and_then(Value::as_array)
                .and_then(|_| step.parse::<usize>().ok());
            if let Some(index) = index {
                let written = self
                    .of(&at)
                    .and_then(|kept| kept.get(index).copied())
                    .unwrap_or(index);
                node = node.and_then(Value::as_array).and_then(|a| a.get(index));
                at.push_index(written);
            } else {
                node = node.and_then(Value::as_object).and_then(|o| o.get(step));
                at.push(step);
            }
        }
        at
    }

    /// [`remove_at`], keeping count of an element taken out of a list.
    fn remove(&mut self, json: &mut Value, steps: &[String]) -> bool {
        if let Some((last, parents)) = steps.split_last() {
            let list = parents.iter().try_fold(&*json, |at, step| {
                if let Some(object) = at.as_object() {
                    object.get(step)
                } else {
                    at.as_array()?.get(step.parse::<usize>().ok()?)
                }
            });
            if let (Some(length), Ok(index)) = (
                list.and_then(Value::as_array).map(VArray::len),
                last.parse::<usize>(),
            ) {
                let written = self.as_written(json, parents);
                if self.of(&written).is_none() {
                    self.0.push((written.clone(), (0..length).collect()));
                }
                if let Some((_, kept)) = self.0.iter_mut().find(|(at, _)| *at == written)
                    && index < kept.len()
                {
                    kept.remove(index);
                }
            }
        }
        remove_at(json, steps)
    }

    /// Records where the elements the lists still have were written: what is told of the
    /// goal from here on is at the index the file has. An inner list before the list it
    /// is in.
    fn record(mut self, d: &mut Diagnostics) {
        self.0.sort_by_key(|(at, _)| std::cmp::Reverse(at.len()));
        for (list, kept) in &self.0 {
            d.left_out(list, kept);
        }
    }
}

/// `json` as the text the lenient parse reads, as facet writes a `Value`. A JSON that does
/// not write (it nests past `MAX_JSON_DEPTH`, which no file read through the door does) is
/// no goal: `InvalidValue`, a warning at the goal, which is dropped.
fn lenient_text<T: Goal>(json: &Value, d: &mut Diagnostics) -> Option<String> {
    let text = bedrock_addon_support::signed_zero::text_for::<T>(json);
    if text.is_none() {
        d.push(
            // The game writes no content-log line for this, so the crate reports an
            // information.
            Diagnostic::info(DiagnosticCode::InvalidValue { text: None }),
        );
    }
    text
}

/// The steps of a path as written (`a/0/b`).
fn path_steps(path: &JsonPath) -> Vec<String> {
    path.segments()
        .map(|segment| match segment {
            bedrock_addon_core::diagnostic::JsonPathSegment::Key(key) => key.to_owned(),
            bedrock_addon_core::diagnostic::JsonPathSegment::Index(index) => index.to_string(),
        })
        .collect()
}

/// Whether `json` has a value at `steps`.
fn has_at(json: &Value, steps: &[String]) -> bool {
    steps
        .iter()
        .try_fold(json, |at, step| {
            if let Some(object) = at.as_object() {
                object.get(step)
            } else {
                at.as_array()?.get(step.parse::<usize>().ok()?)
            }
        })
        .is_some()
}

/// Removes the value at `steps` below `json`: a member of an object or an element of an array.
fn remove_at(json: &mut Value, steps: &[String]) -> bool {
    let Some((last, parents)) = steps.split_last() else {
        return false;
    };
    let mut at = json;
    for step in parents {
        let next = if at.as_object().is_some() {
            at.as_object_mut().and_then(|o| o.get_mut(step))
        } else {
            let index = step.parse::<usize>().ok();
            at.as_array_mut()
                .and_then(|a| index.and_then(|i| a.get_mut(i)))
        };
        let Some(next) = next else { return false };
        at = next;
    }
    if let Some(object) = at.as_object_mut() {
        object.remove(last).is_some()
    } else if let (Some(array), Ok(index)) = (at.as_array_mut(), last.parse::<usize>()) {
        array.remove(index).is_some()
    } else {
        false
    }
}

/// The finding of a failed strict parse, at the failing member.
fn slice_error(error: &SliceError, severity: Severity) -> Diagnostic {
    Diagnostic::error(error.code_with_path(error.has_path()))
        .at(error.path.clone())
        .span(error.span)
        .severity(severity)
}

/// The number `value` holds, as an `f32`; `None` for any other JSON value or a number with no
/// finite `f32`.
pub(crate) fn float_of(value: &Value) -> Option<f32> {
    #[allow(clippy::cast_possible_truncation)] // the nearest f32 is the reading
    let x = value.as_number()?.to_f64_lossy() as f32;
    x.is_finite().then_some(x)
}

/// The bounds a numeric member's written value must keep: the schema's `minimum` / `maximum` /
/// `exclusiveMinimum`, as the game's numeric constraints apply them.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Bound<T> {
    /// `≥ min`.
    Min(T),
    /// `≤ max`.
    Max(T),
    /// `min ≤ x ≤ max`.
    MinMax(T, T),
    /// `> min`.
    Above(T),
}

impl<T: PartialOrd + Copy> Bound<T> {
    /// `x` keeps the bound.
    fn holds(self, x: T) -> bool {
        match self {
            Self::Min(min) => x >= min,
            Self::Max(max) => x <= max,
            Self::MinMax(min, max) => x >= min && x <= max,
            Self::Above(min) => x > min,
        }
    }
}

/// The member checks of one goal: where the findings go, and whether a violation drops the goal
/// (strict) or resets the member to its default (lenient).
pub(crate) struct Check<'d> {
    strict: bool,
    d: &'d mut Diagnostics,
    failed: bool,
    at: JsonPath,
}

impl<'d> Check<'d> {
    /// A check of a goal at the root of its slice.
    pub(crate) fn new(strict: bool, d: &'d mut Diagnostics) -> Self {
        Self {
            strict,
            d,
            failed: false,
            at: JsonPath::new(),
        }
    }

    /// The path of member `key`.
    pub(crate) fn path(&self, key: &str) -> JsonPath {
        self.at.child(key)
    }

    /// Adds findings made elsewhere (a shared type's `check`).
    pub(crate) fn extend(&mut self, found: Diagnostics) {
        self.d.extend(found);
    }

    /// A value of member `key` the game logs at level 3 at the goal's set-up, whatever the
    /// reader, and replaces by its default: the game logs this at the error level and keeps the document: a kept error.
    pub(crate) fn kept(&mut self, key: &str, text: VanillaText) {
        let at = self.path(key);
        self.d.push_kept_error(
            Diagnostic::error(DiagnosticCode::InvalidValue { text: Some(text) }).at(at),
        );
    }

    /// Reports a value of member `key` that vanilla rejects: an error that drops the goal in
    /// the strict parse; in the lenient one (the caller then resets the member) an
    /// information: the legacy readers check no bound, so the typed struct's bound is the
    /// crate's own and the game writes no line for it.
    pub(crate) fn reject(&mut self, key: &str, text: Option<VanillaText>) {
        let severity = if self.strict {
            self.failed = true;
            Severity::Error
        } else {
            lenient_severity(text.as_ref())
        };
        let at = self.path(key);
        self.d.push(
            Diagnostic::error(DiagnosticCode::InvalidValue { text })
                .at(at)
                .severity(severity),
        );
    }

    /// [`Check::reject`] at `at`, a path below the goal.
    fn reject_at(&mut self, at: JsonPath, text: Option<VanillaText>) {
        let severity = if self.strict {
            self.failed = true;
            Severity::Error
        } else {
            lenient_severity(text.as_ref())
        };
        self.d.push(
            Diagnostic::error(DiagnosticCode::InvalidValue { text })
                .at(at)
                .severity(severity),
        );
    }

    /// Checks a float member with the game's numeric constraint of a lower bound `min`: a value
    /// below refuses the member with the constraint's text (`the input must be a number >= …`,
    /// the game's range text).
    pub(crate) fn float_at_least(
        &mut self,
        key: &str,
        slot: &mut Option<StrictFloat<f32>>,
        min: f64,
    ) {
        let range = bedrock_addon_core::diagnostic::cereal::NumericRange::at_least(min);
        if let Some(StrictFloat(x)) = *slot
            && !range.contains(f64::from(x))
        {
            self.reject(key, Some(range.text()));
            *slot = None;
        }
    }

    /// Checks a member that is optionally a number or a float range, with the game's
    /// constraint of a number at least 0 or a range whose ends
    /// are each at least 0: the number form below 0, an end below 0 (told at the end as the
    /// file writes it) or a max below its min ("max value cannot be lower than min value")
    /// refuses the member with the constraint's text.
    pub(crate) fn seconds(
        &mut self,
        key: &str,
        slot: &mut Option<bedrock_addon_core::shared::FloatRange>,
    ) {
        let Some(range) = *slot else {
            return;
        };
        if self.typed_range_spelling(key, &range) {
            *slot = None;
            return;
        }
        let at_least = bedrock_addon_core::diagnostic::cereal::NumericRange::at_least(0.0);
        let (lo, hi) = if range.swapped {
            (range.max, range.min)
        } else {
            (range.min, range.max)
        };
        let mut refused = false;
        if range.form == RangeForm::Scalar {
            if lo < 0.0 {
                self.reject(key, Some(at_least.text()));
                refused = true;
            }
        } else {
            let names = match range.form {
                RangeForm::Array => ["0", "1"],
                RangeForm::RangeMinMax => ["range_min", "range_max"],
                _ => ["min", "max"],
            };
            for (end, missing, name) in [
                (lo, range.missing_min, names[0]),
                (hi, range.missing_max, names[1]),
            ] {
                if !missing && end < 0.0 {
                    let mut at = self.path(key);
                    at.push(name);
                    self.reject_at(at, Some(at_least.text()));
                    refused = true;
                }
            }
            // An end the object leaves out holds the type's default 0 in the stored range,
            // which the game validates.
            if !refused && hi < lo {
                self.reject(
                    key,
                    Some(VanillaText::Fixed(
                        "max value cannot be lower than min value",
                    )),
                );
                refused = true;
            }
        }
        if refused {
            *slot = None;
        }
    }

    /// A member the goal's newest struct binds and then deprecates, written as `text` says: in
    /// the strict parse the game logs an error that refuses the document; in the lenient
    /// conversion the older reader reads it, and the newest
    /// document cannot hold it, so the write-back leaves it out, told (`MemberNotKept`, INFO).
    pub(crate) fn deprecated_member(&mut self, key: &str, text: &'static str) {
        let at = self.path(key);
        let (code, severity) = if self.strict {
            self.failed = true;
            (
                DiagnosticCode::DeprecatedMember {
                    text: Some(VanillaText::Fixed(text)),
                },
                Severity::Error,
            )
        } else {
            (DiagnosticCode::MemberNotKept { text: None }, Severity::Info)
        };
        self.d
            .push(Diagnostic::error(code).at(at).severity(severity));
    }

    /// The goal is read by the lenient conversion of an older document.
    pub(crate) fn lenient(&self) -> bool {
        !self.strict
    }

    /// A content-log error of the legacy goal parser at member `key`, after which the goal is
    /// kept (the game ignores the parser's result): the game logs this at the error level and
    /// keeps the document: a kept error.
    pub(crate) fn legacy_error(&mut self, key: &str, text: &'static str) {
        let at = self.path(key);
        self.legacy_error_at(at, text);
    }

    /// [`Check::legacy_error`] at `at` (a path below the goal, an element of a list).
    pub(crate) fn legacy_error_at(&mut self, at: JsonPath, text: &'static str) {
        self.d.push_kept_error(
            Diagnostic::error(DiagnosticCode::InvalidValue {
                text: Some(VanillaText::Fixed(text)),
            })
            .at(at),
        );
    }

    /// A content-log warning of the legacy goal parser at `at` (a path below the goal, an
    /// element of a list).
    pub(crate) fn legacy_warning_at(&mut self, at: JsonPath, text: &'static str) {
        self.d.push(
            // The legacy goal parser logs these at the warning level.
            Diagnostic::warn(DiagnosticCode::InvalidValue {
                text: Some(VanillaText::Fixed(text)),
            })
            .at(at),
        );
    }

    /// Checks the goal base: `priority` ≥ 0.
    // The game states the constraint as "`priority` minimum 0".
    pub(crate) fn base(&mut self, priority: &mut Option<StrictInt<i32>>) {
        self.int("priority", priority, Bound::Min(0));
    }

    /// Checks a float member against its bound.
    pub(crate) fn float(
        &mut self,
        key: &str,
        slot: &mut Option<StrictFloat<f32>>,
        bound: Bound<f32>,
    ) {
        if let Some(StrictFloat(x)) = *slot
            && !bound.holds(x)
        {
            self.reject(key, None);
            *slot = None;
        }
    }

    /// Checks an integer member against its bound.
    pub(crate) fn int<T: StrictIntType + PartialOrd>(
        &mut self,
        key: &str,
        slot: &mut Option<StrictInt<T>>,
        bound: Bound<T>,
    ) {
        if let Some(StrictInt(x)) = *slot
            && !bound.holds(x)
        {
            self.reject(key, None);
            *slot = None;
        }
    }

    /// Reports a range member's findings (a swapped range).
    pub(crate) fn range<T: RangeEnd>(&mut self, key: &str, range: &mut Option<Range<T>>) {
        if let Some(range) = range {
            if self.typed_range_spelling(key, range) {
                return;
            }
            // The lenient conversion reads the legacy `range_min` / `range_max` spelling; the
            // typed goal it makes is written in the cereal one, `{min, max}`.
            if range.form == RangeForm::RangeMinMax {
                range.form = RangeForm::MinMax;
            }
            let found = range.check(&self.path(key));
            self.d.extend(found);
        }
    }

    /// In the strict read (the goal's typed version) a range written `{range_min, range_max}`:
    /// the cereal `FloatRange` / `IntRange` bind only `min` and `max` and read no other
    /// spelling, so each is an unknown member of the range, an error that refuses the goal.
    /// Returns whether it told so. The legacy readers' `range_min` / `range_max` spelling is
    /// the lenient conversion's, read there.
    fn typed_range_spelling<T: RangeEnd>(&mut self, key: &str, range: &Range<T>) -> bool {
        if !self.strict || range.form != RangeForm::RangeMinMax {
            return false;
        }
        self.failed = true;
        for (end, missing) in [
            ("range_min", range.missing_min),
            ("range_max", range.missing_max),
        ] {
            if !missing {
                let mut at = self.path(key);
                at.push(end);
                self.d.push(
                    Diagnostic::error(DiagnosticCode::UnrecognizedField { text: None }).at(at),
                );
            }
        }
        true
    }

    /// Checks a range member that takes only the `{min, max}` object form (and a number):
    /// an array is rejected in the strict parse and becomes the object form in the lenient one.
    // `beg.look_time` and
    // `float_wander.float_duration` "accept only `{min, max}` from 1.26.40".
    pub(crate) fn object_range<T: RangeEnd>(&mut self, key: &str, range: &mut Option<Range<T>>) {
        if let Some(r) = range
            && r.form == RangeForm::Array
        {
            if self.strict {
                self.reject(key, None);
            } else {
                r.form = RangeForm::MinMax;
            }
        }
        self.range(key, range);
    }

    /// Reports a filter member's findings.
    pub(crate) fn filter(&mut self, key: &str, filter: Option<&Filter>) {
        if let Some(filter) = filter {
            let found = filter.check(&self.path(key));
            self.d.extend(found);
        }
    }

    /// Reports a trigger member's findings.
    pub(crate) fn trigger(&mut self, key: &str, trigger: Option<&EventTrigger>) {
        if let Some(trigger) = trigger {
            let found = trigger.check(&self.path(key));
            self.d.extend(found);
        }
    }

    /// Reports the findings of each trigger of a list member.
    pub(crate) fn triggers(&mut self, key: &str, triggers: Option<&[EventTrigger]>) {
        for (i, trigger) in triggers.into_iter().flatten().enumerate() {
            let mut at = self.path(key);
            at.push_index(i);
            let found = trigger.check(&at);
            self.d.extend(found);
        }
    }

    /// Reports the findings of each item descriptor of a list member.
    pub(crate) fn items(&mut self, key: &str, items: Option<&[ItemDescriptor]>) {
        for (i, item) in items.into_iter().flatten().enumerate() {
            let mut at = self.path(key);
            at.push_index(i);
            let found = item.check(&at);
            self.d.extend(found);
        }
    }

    /// Reports the members a vector's object form does not define (kept in its `extra`).
    pub(crate) fn vector(&mut self, key: &str, extra: Option<&Extra>) {
        for (name, _) in extra.into_iter().flat_map(Extra::iter) {
            let mut at = self.path(key);
            at.push(name);
            self.d.push(
                // The game logs an unrecognised member of a legacy-format goal as a warning;
                // from 1.21.89 it refuses the document.
                Diagnostic::warn(DiagnosticCode::UnrecognizedField { text: None }).at(at),
            );
        }
    }

    /// Reports a sound event name that is not one of vanilla's: an information (the name is
    /// kept).
    pub(crate) fn sound(&mut self, key: &str, sound: Option<&SoundEvent>) {
        if sound.is_some_and(|s| !s.is_known()) {
            let at = self.path(key);
            self.d.push(
                // The game keeps a sound name it does not know without a line, so the crate
                // reports an information.
                Diagnostic::info(DiagnosticCode::UnknownSound { text: None }).at(at),
            );
        }
    }

    /// Reports the findings of a block descriptor member.
    pub(crate) fn block(&mut self, key: &str, block: Option<&BlockDescriptor>) {
        if let Some(block) = block {
            let found = block.check(&self.path(key));
            self.d.extend(found);
        }
    }

    /// Reports a required member that is absent: an error that drops the goal in the strict
    /// parse, a warning in the lenient one.
    pub(crate) fn required(&mut self, key: &str, present: bool) {
        if present {
            return;
        }
        let severity = if self.strict {
            self.failed = true;
            Severity::Error
        } else {
            // The lenient conversion takes the default without a line, so the crate reports an
            // information.
            Severity::Info
        };
        let at = self.path(key);
        self.d.push(
            Diagnostic::error(DiagnosticCode::MissingRequiredField { text: None })
                .at(at)
                .severity(severity),
        );
    }

    /// Reports the findings of each block descriptor of a list member.
    pub(crate) fn blocks(&mut self, key: &str, blocks: Option<&[BlockDescriptor]>) {
        for (i, block) in blocks.into_iter().flatten().enumerate() {
            let mut at = self.path(key);
            at.push_index(i);
            let found = block.check(&at);
            self.d.extend(found);
        }
    }

    /// Reports the findings of an `entity_types` member.
    pub(crate) fn entity_types(&mut self, key: &str, types: Option<&EntityTypes>) {
        check_entity_types(self, key, types);
    }

    /// Runs `check` on each element of a list member, with the path of the element.
    pub(crate) fn each<T>(
        &mut self,
        key: &str,
        list: &mut Option<Vec<T>>,
        mut check: impl FnMut(&mut T, &mut Check<'_>),
    ) {
        for (i, element) in list.iter_mut().flatten().enumerate() {
            self.at.push(key);
            self.at.push_index(i);
            check(element, self);
            self.at.pop();
            self.at.pop();
        }
    }
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

    /// a goal whose JSON does not write (it nests past the
    /// limit; no pack reaches it today, the file is refused first) went without a word.
    #[test]
    fn a_goal_whose_json_does_not_write_is_told_at_the_goal() {
        let mut d = Diagnostics::default();
        let text = lenient_text::<crate::goal::attack::AquaticChargeAttack>(
            &Value::from(nested(MAX_JSON_DEPTH)),
            &mut d,
        );
        assert!(text.is_some_and(|text| text.starts_with(r#"{"zz":[["#)));
        assert!(d.is_empty(), "{d:#?}");
        let text = lenient_text::<crate::goal::attack::AquaticChargeAttack>(
            &Value::from(nested(MAX_JSON_DEPTH + 1)),
            &mut d,
        );
        assert_eq!(text, None);
        assert_eq!(told(&d), ["InvalidValue Info "]);
    }
}
