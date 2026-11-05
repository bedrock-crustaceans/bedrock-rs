//! The JSON operations the steps are made of: reading and rewriting components in `components`
//! and in every component group, the way vanilla's upgrade and rename steps
//! visit them, and the findings of a removal.

use bedrock_addon_core::diagnostic::{
    self, Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, Severity, VanillaText,
};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::{Family, UpgradeCx};
use facet_value::{VArray, VObject};
use indexmap::IndexMap;

use super::Payload;
use crate::Entity;
use crate::game_schema;

/// The members of a JSON object: `components` (an `IndexMap`) or a group (a `VObject`).
pub(crate) trait Members {
    /// The member `key`.
    fn get(&self, key: &str) -> Option<&Value>;
    /// The member `key`, mutably.
    fn get_mut(&mut self, key: &str) -> Option<&mut Value>;
    /// Sets `key`: in place when present, appended otherwise.
    fn set(&mut self, key: &str, value: Value);
    /// Removes `key`, keeping the order of the others.
    fn take(&mut self, key: &str) -> Option<Value>;
    /// The keys, in order.
    fn keys(&self) -> Vec<String>;

    /// Whether `key` is present.
    fn has(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// Renames `old` to `new` at `old`'s position; an existing `new` is replaced.
    fn rename(&mut self, old: &str, new: &str) {
        let Some(value) = self.get(old).cloned() else {
            return;
        };
        let keys = self.keys();
        let mut entries: Vec<(String, Value)> = Vec::with_capacity(keys.len());
        for key in keys {
            if key == new {
                continue;
            }
            let member = if key == old {
                (new.to_owned(), value.clone())
            } else {
                let member = self.get(&key).cloned().unwrap_or(Value::NULL);
                (key, member)
            };
            entries.push(member);
        }
        for key in self.keys() {
            self.take(&key);
        }
        for (key, value) in entries {
            self.set(&key, value);
        }
    }

    /// The component `key` as an object, mutably.
    fn object(&mut self, key: &str) -> Option<&mut VObject> {
        self.get_mut(key)?.as_object_mut()
    }

    /// Inserts `key` when absent; returns whether it did. A `null` member is an absent
    /// member: it is taken out first, so the member stands where the
    /// one of a document without it stands.
    fn insert_absent(&mut self, key: &str, value: Value) -> bool {
        if self.get(key).is_some_and(|member| !member.is_null()) {
            return false;
        }
        self.take(key);
        self.set(key, value);
        true
    }
}

impl Members for IndexMap<String, Value> {
    fn get(&self, key: &str) -> Option<&Value> {
        IndexMap::get(self, key)
    }
    fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        IndexMap::get_mut(self, key)
    }
    fn set(&mut self, key: &str, value: Value) {
        self.insert(key.to_owned(), value);
    }
    fn take(&mut self, key: &str) -> Option<Value> {
        self.shift_remove(key)
    }
    fn keys(&self) -> Vec<String> {
        IndexMap::keys(self).cloned().collect()
    }
}

impl Members for VObject {
    fn get(&self, key: &str) -> Option<&Value> {
        VObject::get(self, key)
    }
    fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        VObject::get_mut(self, key)
    }
    fn set(&mut self, key: &str, value: Value) {
        self.insert(key, value);
    }
    fn take(&mut self, key: &str) -> Option<Value> {
        self.remove(key)
    }
    fn keys(&self) -> Vec<String> {
        VObject::keys(self).map(|k| k.as_str().to_owned()).collect()
    }
}

/// One components section: `components` or a group.
pub(crate) struct PayloadSection<'a> {
    /// Its path below the payload (`components`, `component_groups/<name>`).
    pub at: JsonPath,
    /// Its members.
    pub members: &'a mut dyn Members,
}

/// `components` (when present) and every group that is an object, in order.
pub(crate) fn sections(payload: &mut Payload) -> Vec<PayloadSection<'_>> {
    let mut out = Vec::new();
    if let Some(components) = &mut payload.components {
        out.push(PayloadSection {
            at: JsonPath::new().child("components"),
            members: components,
        });
    }
    if let Some(groups) = &mut payload.component_groups {
        for (name, group) in groups {
            if let Some(object) = group.as_object_mut() {
                out.push(PayloadSection {
                    at: JsonPath::new().with(&["component_groups", name]),
                    members: object,
                });
            }
        }
    }
    out
}

/// The conversions of the step to `to` whose legacy reader is one of the game's schema-driven
/// readers: each component the game types first at `to` is validated as that reader validates it,
/// before the step's own ops (its typed-shape ops act on the converted component). What the game
/// logs is told, and the component becomes what the reader parses: `{}` when the validation failed
/// (the component keeps its defaults), the component without what the lenient mode took out. Strict
/// from 1.15.0 ([`game_schema::STRICT_SINCE`]); a component written as `null` is left to rule N1.
pub(crate) fn schema_conversions(payload: &mut Payload, to: FormatVersion, cx: &mut UpgradeCx<'_>) {
    let strict = cx.original_version >= game_schema::STRICT_SINCE;
    let mut found = Diagnostics::default();
    for section in sections(payload) {
        for key in section.members.keys() {
            let Some(owner) = game_schema::component(&key).filter(|owner| {
                owner
                    .typed_since
                    .is_some_and(|(a, b, c)| FormatVersion::new(a, b, c) == to)
            }) else {
                continue;
            };
            let Some(value) = section.members.get_mut(&key).filter(|v| !v.is_null()) else {
                continue;
            };
            let mut outcome = game_schema::validate(owner.members, value, strict);
            outcome.check_numbers(owner.name, value);
            let at = JsonPath::new()
                .child(<Entity as Family>::PAYLOAD_KEY)
                .join(&section.at)
                .child(&key);
            found.extend(outcome.findings(&at, Phase::Upgrade));
            outcome.tell_left_out(&at, &mut found);
            if let Some(replacement) = outcome.replacement() {
                *value = replacement;
            }
        }
        // Each goal the game types first at `to` is read by its legacy reader here; the `min` /
        // `max` of its ranges are not read.
        for key in section.members.keys() {
            if crate::goal::versioned(&key).is_none() || super::first_typed(&key) != Some(to) {
                continue;
            }
            let Some(goal) = section.members.get_mut(&key).and_then(Value::as_object_mut) else {
                continue;
            };
            let at = JsonPath::new()
                .child(<Entity as Family>::PAYLOAD_KEY)
                .join(&section.at)
                .child(&key);
            for (below, code) in crate::goal::legacy_ranges(&key, goal) {
                let path = at.join(&below);
                found.push(Diagnostic::info(code).at(path).phase(Phase::Upgrade));
            }
        }
    }
    for told in found.into_told() {
        match told {
            diagnostic::Told::Finding(finding) => cx.diagnostics.push(*finding),
            diagnostic::Told::Moved { from, to } => cx.diagnostics.moved(from, to),
        }
    }
}

/// `components`, created when absent.
pub(crate) fn components(payload: &mut Payload) -> &mut IndexMap<String, Value> {
    payload.components.get_or_insert_with(IndexMap::new)
}

/// The group `name` as an object, when it exists.
pub(crate) fn group<'a>(payload: &'a mut Payload, name: &str) -> Option<&'a mut VObject> {
    payload
        .component_groups
        .as_mut()?
        .get_mut(name)?
        .as_object_mut()
}

/// What a removal removed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Removed {
    /// A whole component (`DroppedComponent`).
    Component,
    /// A member of one (`DroppedField`).
    Field,
}

/// The finding of a removal at `at` (below the payload), with the crate's text: the game drops
/// the member silently (it writes no content-log line), so the crate reports an information.
pub(crate) fn removed(
    cx: &mut UpgradeCx<'_>,
    what: Removed,
    at: &JsonPath,
    member: &str,
    version: FormatVersion,
) {
    let text = Some(VanillaText::RemovedByUpgrade {
        member: member.to_owned(),
        version,
    });
    let code = match what {
        Removed::Component => DiagnosticCode::DroppedComponent { text },
        Removed::Field => DiagnosticCode::DroppedField { text },
    };
    note(cx, code, at);
}

/// An information of a step at `at` (below the payload): the game writes no content-log line for
/// it, so the crate reports it as an information. Its callers tell a value jsoncpp's `as*` read in
/// silence (`InvalidType`, the misread member of `v1_26_0` / `v1_26_30`).
pub(crate) fn finding(cx: &mut UpgradeCx<'_>, code: DiagnosticCode, at: &JsonPath) {
    finding_of(cx, code, Severity::Info, at);
}

/// A kept error of a step at `at` (below the payload): a level-3 line of the game after which
/// the entity loads: a kept error.
pub(crate) fn kept(cx: &mut UpgradeCx<'_>, code: DiagnosticCode, at: &JsonPath) {
    let path = JsonPath::new()
        .child(<Entity as Family>::PAYLOAD_KEY)
        .join(at);
    cx.diagnostics
        .push_kept_error(Diagnostic::error(code).at(path).phase(Phase::Upgrade));
}

/// A note of a step at `at` (below the payload): INFO, a value the game itself discards at
/// this version.
pub(crate) fn note(cx: &mut UpgradeCx<'_>, code: DiagnosticCode, at: &JsonPath) {
    finding_of(cx, code, Severity::Info, at);
}

/// A finding of a step at `at` (below the payload).
fn finding_of(cx: &mut UpgradeCx<'_>, code: DiagnosticCode, severity: Severity, at: &JsonPath) {
    let path = JsonPath::new()
        .child(<Entity as Family>::PAYLOAD_KEY)
        .join(at);
    cx.diagnostics.push(
        Diagnostic::error(code)
            .at(path)
            .phase(Phase::Upgrade)
            .severity(severity),
    );
}

/// What a step has to tell, gathered while the payload is borrowed and reported after, in the order
/// the step made them: the findings and the moves. A finding names the member by the path it has
/// when it is told; a move recorded before it is undone for it, so that it sits at the member as
/// the file has it.
#[derive(Default)]
pub(crate) struct Told(Vec<Event>);

/// One thing a step tells.
enum Event {
    /// A finding at a path below the payload.
    Finding(DiagnosticCode, Severity, JsonPath),
    /// The member at the first path is now at the second.
    Moved(JsonPath, JsonPath),
}

impl Told {
    /// A value of another JSON shape than the step converts, read as the game reads it
    /// (`InvalidType`, INFO: the game discards the value without a word).
    pub(crate) fn wrong_type(&mut self, at: JsonPath) {
        self.note(DiagnosticCode::InvalidType { text: None }, at);
    }

    /// A member the step drops, or replaces by what it makes of another, as the game's
    /// fixer does (`DroppedField`, INFO).
    pub(crate) fn dropped(&mut self, at: JsonPath) {
        self.note(DiagnosticCode::DroppedField { text: None }, at);
    }

    /// A member of an object form the step does not read (`UnrecognizedField`, INFO: the
    /// game does not read it either).
    pub(crate) fn unknown(&mut self, at: JsonPath) {
        self.note(DiagnosticCode::UnrecognizedField { text: None }, at);
    }

    /// A value the game itself discards at this version: INFO.
    pub(crate) fn note(&mut self, code: DiagnosticCode, at: JsonPath) {
        self.0.push(Event::Finding(code, Severity::Info, at));
    }

    /// A value this library loses or changes where the game would keep it: an information
    /// of the crate's own (the game writes no content-log line for a deviation of the crate's).
    pub(crate) fn deviation(&mut self, code: DiagnosticCode, at: JsonPath) {
        self.0.push(Event::Finding(code, Severity::Info, at));
    }

    /// The member at `from` is now at `to` (paths below the payload): what is told of it
    /// from here on is told at `from`.
    pub(crate) fn moved(&mut self, from: JsonPath, to: JsonPath) {
        self.0.push(Event::Moved(from, to));
    }

    /// The finding of `read`, if it has one, at `at`.
    pub(crate) fn read<T>(
        &mut self,
        read: (T, Option<Misread>),
        at: impl FnOnce() -> JsonPath,
    ) -> T {
        if let Some((code, severity)) = read.1 {
            self.0.push(Event::Finding(code, severity, at()));
        }
        read.0
    }

    /// Reports the findings and records the moves, in their order.
    pub(crate) fn report(self, cx: &mut UpgradeCx<'_>) {
        let rooted = |at: &JsonPath| {
            JsonPath::new()
                .child(<Entity as Family>::PAYLOAD_KEY)
                .join(at)
        };
        for event in self.0 {
            match event {
                Event::Finding(code, severity, at) => finding_of(cx, code, severity, &at),
                Event::Moved(from, to) => cx.diagnostics.moved(rooted(&from), rooted(&to)),
            }
        }
    }
}

/// What is to tell of a value read as another: the code, and INFO where the game reads it
/// so, a warning where the value is the crate's own.
pub(crate) type Misread = (DiagnosticCode, Severity);

/// Vanilla's component rename in one section (at `section`): `old` becomes `new` at `old`'s
/// position. A
/// `new` the file has with another value is replaced, as the game replaces it, and noted;
/// the move is recorded.
pub(crate) fn rename(
    members: &mut dyn Members,
    section: &JsonPath,
    old: &str,
    new: &str,
    told: &mut Told,
) {
    let Some(value) = members.get(old) else {
        return;
    };
    let to = section.child(new);
    if members
        .get(new)
        .is_some_and(|had| !had.is_null() && !same(had, value))
    {
        told.dropped(to.clone());
    }
    members.rename(old, new);
    told.moved(section.child(old), to);
}

/// Moves the member `old` of `object` (at `at`) to `new`, whatever the object has there: a
/// `new` of the file with another value is noted as replaced, and the move recorded.
/// Returns whether there was an `old`.
pub(crate) fn rename_member(
    object: &mut VObject,
    old: &str,
    new: &str,
    at: &JsonPath,
    told: &mut Told,
) -> bool {
    let Some(value) = object.remove(old) else {
        return false;
    };
    overwrite(object, new, value, at, told);
    told.moved(at.child(old), at.child(new));
    true
}

/// What vanilla's `asInt` makes of a member, and what is to tell of it: a value that is no
/// number is 0 (`InvalidType`); a number with a fraction is truncated and one beyond an int
/// saturates (`InvalidValue`). `null` is 0 with nothing to tell (the callers take a `null`
/// member as an absent one).
pub(crate) fn int_of(value: &Value) -> (i32, Option<Misread>) {
    let Some(number) = value.as_number() else {
        let code = (!value.is_null())
            .then_some((DiagnosticCode::InvalidType { text: None }, Severity::Info));
        return (0, code);
    };
    let number = number.to_f64_lossy();
    #[allow(clippy::cast_possible_truncation, reason = "vanilla reads an int")]
    let int = number as i32;
    #[allow(clippy::float_cmp, reason = "exactly the int, not a number near it")]
    let exact = f64::from(int) == number;
    // The game truncates a fraction and says nothing; a number beyond an int reaches jsoncpp's
    // `asInt` assert at most (no content-log line): both are informations of the crate's own.
    (
        int,
        (!exact).then_some((DiagnosticCode::InvalidValue { text: None }, Severity::Info)),
    )
}

/// What vanilla's `asFloat` / `asDouble` makes of a member: its number, or 0 for a value that
/// is no number (`InvalidType`, INFO; nothing to tell of a `null`).
pub(crate) fn number_of(value: &Value) -> (f64, Option<Misread>) {
    match value.as_number() {
        Some(number) => (number.to_f64_lossy(), None),
        None => (
            0.0,
            (!value.is_null())
                .then_some((DiagnosticCode::InvalidType { text: None }, Severity::Info)),
        ),
    }
}

/// What vanilla's `asFloat` makes of a member, as [`number_of`]; a number beyond a float is the
/// largest finite float of its sign (`InvalidValue`, an information: the bound is the crate's own,
/// jsoncpp's `asFloat` gives the infinity without a line, like `int_of` for a number beyond an
/// int), so no arithmetic of a step is infinite.
pub(crate) fn float_of(value: &Value) -> (f32, Option<Misread>) {
    let (number, code) = number_of(value);
    #[allow(clippy::cast_possible_truncation, reason = "vanilla's f32")]
    let float = number as f32;
    if float.is_infinite() {
        return (
            f32::MAX.copysign(float),
            Some((DiagnosticCode::InvalidValue { text: None }, Severity::Info)),
        );
    }
    (float, code)
}

/// The member `name` of `object` (at `at`) as one half of a range a step builds, taken out:
/// its number. The crate's own note: a value that is no number is no half (`InvalidType`), so the
/// other half is kept; a number beyond a float is the largest finite float of its sign
/// (`InvalidValue`); both warnings at the member. A `null` member is an absent one.
pub(crate) fn half(
    object: &mut VObject,
    name: &str,
    at: &JsonPath,
    told: &mut Told,
) -> Option<Value> {
    let value = object.remove(name).filter(|value| !value.is_null())?;
    let Some(number) = value.as_number() else {
        told.deviation(DiagnosticCode::InvalidType { text: None }, at.child(name));
        return None;
    };
    #[allow(clippy::cast_possible_truncation, reason = "the range's f32")]
    let float = number.to_f64_lossy() as f32;
    if float.is_infinite() {
        told.deviation(DiagnosticCode::InvalidValue { text: None }, at.child(name));
        return Some(f32_value(f32::MAX.copysign(float)));
    }
    Some(value)
}

/// Whether two JSON values are the same value: numbers by what they spell (`1` is `1.0`).
#[allow(clippy::float_cmp, reason = "the same number, not one near it")]
pub(crate) fn same(a: &Value, b: &Value) -> bool {
    match (a.as_number(), b.as_number()) {
        (Some(a), Some(b)) => a.to_f64_lossy() == b.to_f64_lossy(),
        _ => a == b,
    }
}

/// Sets `key` of `object` to `value`, as the step does whatever the document has there; a
/// member of the document that held another value is told as dropped at `at` (the object's
/// path), a `null` one not.
pub(crate) fn overwrite(
    object: &mut VObject,
    key: &str,
    value: Value,
    at: &JsonPath,
    told: &mut Told,
) {
    if object
        .get(key)
        .is_some_and(|had| !had.is_null() && !same(had, &value))
    {
        told.dropped(at.child(key));
    }
    object.insert(key, value);
}

/// A constant fragment written as JSON text in the step.
pub(crate) fn json(text: &str) -> Value {
    // The fragments are literals of this module; a parse failure is a bug the step tests catch.
    bedrock_addon_core::json::from_str::<Value>(text).unwrap_or(Value::NULL)
}

/// Whether the entity's type is one of `types`.
pub(crate) fn is(cx: &UpgradeCx<'_>, types: &[ActorType]) -> bool {
    types.contains(&cx.actor_type)
}

/// The type id's category bits: `id & mask == mask`.
pub(crate) fn has_bits(cx: &UpgradeCx<'_>, mask: u32) -> bool {
    cx.actor_type.id() & mask == mask
}

/// A JSON number from an `f32` constant, widened as vanilla widens it.
pub(crate) fn f32_value(value: f32) -> Value {
    Value::from(f64::from(value))
}

/// The version at or below which a payload without `components` gets no fixer op at all.
const UNTOUCHED_AT_OR_BELOW: FormatVersion = known::V1_8_0;

/// The game's early exit: a document at or below 1.8.0 whose
/// `minecraft:entity.components` is absent or `null` skips both fixer sections.
fn untouched(payload: &Payload, cx: &UpgradeCx<'_>) -> bool {
    cx.original_version <= UNTOUCHED_AT_OR_BELOW && payload.components.is_none()
}

/// Whether the step's section-A ops run: not with `do_not_upgrade`, not at the engine version
/// (`UpgradeCx::skip_section_a`), not for an untouched document.
pub(crate) fn section_a(payload: &Payload, cx: &UpgradeCx<'_>) -> bool {
    !cx.skip_section_a && !untouched(payload, cx)
}

/// Whether the step's section-B ops run: always, except for an untouched document.
pub(crate) fn section_b(payload: &Payload, cx: &UpgradeCx<'_>) -> bool {
    !untouched(payload, cx)
}

/// [`single_to_array`] of the member `key` of `object` (at `at`), with the move recorded:
/// what is told of the one element is told of the member the file has.
pub(crate) fn single_to_array_at(object: &mut VObject, key: &str, at: &JsonPath, told: &mut Told) {
    if object.get(key).is_some_and(|member| !member.is_array()) {
        let member = at.child(key);
        told.moved(member.clone(), member.index(0));
    }
    single_to_array(object, key);
}

/// A member that is present and not an array becomes a
/// one-element array.
pub(crate) fn single_to_array(object: &mut VObject, key: &str) {
    if let Some(member) = object.get_mut(key)
        && !member.is_array()
    {
        let mut list = VArray::new();
        list.push(member.clone());
        *member = list.into();
    }
}
