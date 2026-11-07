//! The JSON operations upgrade steps are made of, for a family whose older eras hold their
//! payload as JSON (entity, block): the members of an object behind one trait, the
//! game's comparisons and renames, the fragments a step inserts as built values, the jsoncpp
//! value readers, and the findings of a step.
//!
//! Two member renames live here because the game's fixers rename differently and each family
//! follows its own ([`move_member`] for entity's, [`rename_member_in_place`] for block's): they
//! differ on a `null` member and on where the renamed member stands.

use facet_value::{VArray, VObject};
use indexmap::IndexMap;

use crate::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath, Phase, Severity};
use crate::json::Value;

/// The members of a JSON object: an ordered map of a payload (`IndexMap`) or an object of the
/// JSON (`VObject`).
pub trait Members {
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

    /// The member `key` as an object, mutably.
    fn object(&mut self, key: &str) -> Option<&mut VObject> {
        self.get_mut(key)?.as_object_mut()
    }

    /// Inserts `key` when absent; returns whether it did. A `null` member is an absent
    /// member: it is taken out first, so the member stands where the one of a document without
    /// it stands.
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

/// Whether two JSON values are the same value: numbers by what they spell (`1` is `1.0`).
#[allow(clippy::float_cmp, reason = "the same number, not one near it")]
pub fn same(a: &Value, b: &Value) -> bool {
    match (number(a), number(b)) {
        (Some(a), Some(b)) => a == b,
        _ => a == b,
    }
}

/// A number of a JSON value.
pub fn number(value: &Value) -> Option<f64> {
    value.as_number().map(facet_value::VNumber::to_f64_lossy)
}

/// A string of a JSON value.
pub fn string(value: &Value) -> Option<&str> {
    value.as_string().map(facet_value::VString::as_str)
}

/// An object's members as an ordered map, in order.
pub fn members(object: &VObject) -> IndexMap<String, Value> {
    object
        .iter()
        .map(|(k, v)| (k.as_str().to_owned(), v.clone()))
        .collect()
}

/// An ordered map as a JSON object, in order.
pub fn to_object(map: &IndexMap<String, Value>) -> Value {
    let mut object = VObject::new();
    for (k, v) in map {
        object.insert(k.as_str(), v.clone());
    }
    object.into()
}

/// Renames `old` to `new` in `map`, keeping its position, the value mapped by `convert`. A
/// `new` the map already has goes first (the renamed member is the one kept): the caller tells
/// of it.
pub fn rename_key(
    map: &mut IndexMap<String, Value>,
    old: &str,
    new: &str,
    convert: &mut impl FnMut(Value) -> Value,
) {
    if !map.contains_key(old) {
        return;
    }
    map.shift_remove(new);
    let Some((index, _, value)) = map.shift_remove_full(old) else {
        return;
    };
    let value = convert(value);
    map.shift_insert(index.min(map.len()), new.to_owned(), value);
}

/// What a member rename did, when there was a member to rename.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Renamed {
    /// The object had its own `new` with another value (not `null`), which the rename replaced:
    /// the caller tells of it (`DroppedField`) before it records the move.
    pub replaced: bool,
}

/// Sets `key` of `object` to `value`, whatever the object has there (in place when present,
/// appended otherwise); `true` when the object's own member held another value (not `null`),
/// which the caller tells of.
pub fn overwrite(object: &mut VObject, key: &str, value: Value) -> bool {
    let replaced = object
        .get(key)
        .is_some_and(|had| !had.is_null() && !same(had, &value));
    object.insert(key, value);
    replaced
}

/// Entity's member rename: the member `old` is taken out whatever it holds (a `null` too) and
/// [`overwrite`]s `new`, which keeps `new`'s position when the object has one and is appended
/// otherwise. `None` when there is no `old`.
pub fn move_member(object: &mut VObject, old: &str, new: &str) -> Option<Renamed> {
    let value = object.remove(old)?;
    let replaced = overwrite(object, new, value);
    Some(Renamed { replaced })
}

/// Block's member rename: a `null` `old` is an absent member and goes, with nothing renamed; any
/// other `old` becomes `new` at `old`'s position ([`rename_key`]), the object's own `new` going
/// first. `None` when nothing was renamed.
pub fn rename_member_in_place(
    map: &mut IndexMap<String, Value>,
    old: &str,
    new: &str,
) -> Option<Renamed> {
    if map.get(old).is_some_and(Value::is_null) {
        map.shift_remove(old);
    }
    let value = map.get(old)?;
    let replaced = map
        .get(new)
        .is_some_and(|had| !had.is_null() && !same(had, value));
    map.shift_remove(new);
    rename_key(map, old, new, &mut |v| v);
    Some(Renamed { replaced })
}

/// A member that is present and not an array becomes a one-element array.
pub fn single_to_array(object: &mut VObject, key: &str) {
    if let Some(member) = object.get_mut(key)
        && !member.is_array()
    {
        let mut list = VArray::new();
        list.push(member.clone());
        *member = list.into();
    }
}

/// A JSON number from an `f32` constant, widened as vanilla widens it.
pub fn f32_value(value: f32) -> Value {
    Value::from(f64::from(value))
}

/// A JSON object of `members`, in order: a fragment a step inserts, built as a value (there is
/// no text to parse, so nothing can fail).
pub fn object<const N: usize>(members: [(&str, Value); N]) -> Value {
    let mut object = VObject::new();
    for (key, value) in members {
        object.insert(key, value);
    }
    object.into()
}

/// The empty JSON object `{}`.
pub fn empty_object() -> Value {
    VObject::new().into()
}

/// A JSON array of `items`, in order.
pub fn array<const N: usize>(items: [Value; N]) -> Value {
    items.into_iter().collect::<VArray>().into()
}

/// A JSON array of strings, in order.
pub fn strings(items: &[&str]) -> Value {
    items
        .iter()
        .map(|item| Value::from(*item))
        .collect::<VArray>()
        .into()
}

/// An information of a step at `at` (the path in the file): the game discards the value
/// without a content-log line, so the finding is the crate's own.
pub fn note(d: &mut Diagnostics, code: Code, at: JsonPath) {
    tell(d, code, Severity::Info, at);
}

/// A finding of a step at `at`, of `severity`.
pub fn tell(d: &mut Diagnostics, code: Code, severity: Severity, at: JsonPath) {
    d.push(
        Diagnostic::error(code)
            .at(at)
            .phase(Phase::Upgrade)
            .severity(severity),
    );
}

/// A kept error of a step at `at`: the game logs an error and loads the document.
pub fn kept(d: &mut Diagnostics, code: Code, at: JsonPath) {
    d.push_kept_error(Diagnostic::error(code).at(at).phase(Phase::Upgrade));
}

/// The value readers of jsoncpp (`asInt`, `asFloat`, `asDouble`) as the game's fixers read a
/// member, with what is to tell of a value they read as another.
pub mod jsoncpp {
    use crate::diagnostic::codes as core_codes;
    use crate::diagnostic::{Code, Severity};
    use crate::json::Value;

    /// What is to tell of a value read as another: the code, and INFO where the game reads it
    /// so, a warning where the value is the crate's own.
    pub type Misread = (Code, Severity);

    /// What `asInt` makes of a member, and what is to tell of it: a value that is no number is
    /// 0 (`InvalidType`); a number with a fraction is truncated and one beyond an int saturates
    /// (`InvalidValue`). `null` is 0 with nothing to tell (the callers take a `null` member as
    /// an absent one).
    pub fn int_of(value: &Value) -> (i32, Option<Misread>) {
        let Some(number) = value.as_number() else {
            let code = (!value.is_null())
                .then_some((Code::from(&core_codes::INVALID_TYPE), Severity::Info));
            return (0, code);
        };
        let number = number.to_f64_lossy();
        #[allow(clippy::cast_possible_truncation, reason = "vanilla reads an int")]
        let int = number as i32;
        #[allow(clippy::float_cmp, reason = "exactly the int, not a number near it")]
        let exact = f64::from(int) == number;
        // The game truncates a fraction and says nothing; a number beyond an int reaches
        // jsoncpp's `asInt` assert at most (no content-log line): both are informations of the
        // crate's own.
        (
            int,
            (!exact).then_some((Code::from(&core_codes::INVALID_VALUE), Severity::Info)),
        )
    }

    /// What `asFloat` / `asDouble` makes of a member: its number, or 0 for a value that is no
    /// number (`InvalidType`, INFO; nothing to tell of a `null`).
    pub fn number_of(value: &Value) -> (f64, Option<Misread>) {
        match value.as_number() {
            Some(number) => (number.to_f64_lossy(), None),
            None => (
                0.0,
                (!value.is_null())
                    .then_some((Code::from(&core_codes::INVALID_TYPE), Severity::Info)),
            ),
        }
    }

    /// What `asFloat` makes of a member, as [`number_of`]; a number beyond a float is the
    /// largest finite float of its sign (`InvalidValue`, an information: the bound is the
    /// crate's own, jsoncpp's `asFloat` gives the infinity without a line, like [`int_of`] for
    /// a number beyond an int), so no arithmetic of a step is infinite.
    pub fn float_of(value: &Value) -> (f32, Option<Misread>) {
        let (number, code) = number_of(value);
        #[allow(clippy::cast_possible_truncation, reason = "vanilla's f32")]
        let float = number as f32;
        if float.is_infinite() {
            return (
                f32::MAX.copysign(float),
                Some((Code::from(&core_codes::INVALID_VALUE), Severity::Info)),
            );
        }
        (float, code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::codes as core_codes;
    use crate::json::{from_str, to_json};

    fn json(text: &str) -> Value {
        from_str::<Value>(text).expect("a fragment of the test")
    }

    fn map(text: &str) -> IndexMap<String, Value> {
        members(json(text).as_object().expect("an object"))
    }

    fn keys(object: &impl Members) -> Vec<String> {
        object.keys()
    }

    /// The same value and the same text: `VNumber`'s `==` takes `1` for `1.0`, the text does not.
    fn assert_same_fragment(built: &Value, text: &str) {
        let parsed = json(text);
        assert_eq!(built, &parsed);
        assert_eq!(to_json(built).ok(), to_json(&parsed).ok(), "{text}");
    }

    #[test]
    fn the_builders_build_what_the_text_spells() {
        assert_same_fragment(&empty_object(), "{}");
        assert_same_fragment(
            &object([
                ("b", Value::from(1)),
                ("a", Value::from(0.5)),
                ("c", array([Value::from("x"), Value::from(true)])),
            ]),
            r#"{"b": 1, "a": 0.5, "c": ["x", true]}"#,
        );
        assert_same_fragment(&strings(&["fire", "magma"]), r#"["fire", "magma"]"#);
        assert_same_fragment(&array([Value::from(-1), Value::from(1)]), "[-1, 1]");
    }

    #[test]
    fn an_int_and_a_whole_float_are_the_same_value() {
        assert!(same(&json("1"), &json("1.0")));
        assert!(!same(&json("1"), &json("\"1\"")));
        assert!(same(&json("[1]"), &json("[1]")));
    }

    #[test]
    fn members_round_trip_in_order() {
        let text = r#"{"z": 1, "a": {"b": 2}, "m": null}"#;
        let map = map(text);
        assert_eq!(keys(&map), ["z", "a", "m"]);
        assert_eq!(to_json(&to_object(&map)).ok(), to_json(&json(text)).ok());
    }

    #[test]
    fn rename_keeps_old_s_position_and_replaces_new() {
        let mut object = map(r#"{"a": 1, "old": 2, "new": 3, "z": 4}"#);
        Members::rename(&mut object, "old", "new");
        assert_eq!(keys(&object), ["a", "new", "z"]);
        assert_eq!(object["new"], json("2"));
        let mut object = json(r#"{"a": 1, "old": 2, "new": 3, "z": 4}"#);
        Members::rename(object.as_object_mut().expect("an object"), "old", "new");
        assert_eq!(
            to_json(&object).ok(),
            to_json(&json(r#"{"a": 1, "new": 2, "z": 4}"#)).ok()
        );
    }

    #[test]
    fn insert_absent_replaces_a_null_at_the_end() {
        let mut object = map(r#"{"k": null, "a": 1}"#);
        assert!(object.insert_absent("k", json("{}")));
        assert_eq!(keys(&object), ["a", "k"]);
        assert!(!object.insert_absent("a", json("2")));
        assert_eq!(object["a"], json("1"));
    }

    #[test]
    fn rename_key_converts_in_place() {
        let mut object = map(r#"{"new": 0, "a": 1, "old": 2}"#);
        rename_key(&mut object, "old", "new", &mut |v| object_of("seconds", v));
        assert_eq!(keys(&object), ["a", "new"]);
        assert_eq!(object["new"], json(r#"{"seconds": 2}"#));
    }

    fn object_of(key: &str, value: Value) -> Value {
        let mut object = VObject::new();
        object.insert(key, value);
        object.into()
    }

    // H1: the two renames differ, each as its family's fixer does; pinned, not unified.

    /// Entity's rename: a `null` old is moved as `null` and overwrites `new`.
    #[test]
    fn move_member_moves_a_null_old() {
        let mut object = json(r#"{"old": null, "new": 3}"#);
        let object = object.as_object_mut().expect("an object");
        assert_eq!(
            move_member(object, "old", "new"),
            Some(Renamed { replaced: true })
        );
        assert_eq!(
            to_json(&Value::from(object.clone())).ok(),
            to_json(&json(r#"{"new": null}"#)).ok()
        );
    }

    /// Entity's rename: the renamed member stands where `new` stood, or at the end.
    #[test]
    fn move_member_puts_new_at_new_s_place_or_the_end() {
        let mut object = json(r#"{"old": 1, "a": 2}"#);
        let object = object.as_object_mut().expect("an object");
        assert_eq!(
            move_member(object, "old", "new"),
            Some(Renamed { replaced: false })
        );
        assert_eq!(keys(object), ["a", "new"]);
        let mut object = json(r#"{"new": 1, "a": 2, "old": 1.0}"#);
        let object = object.as_object_mut().expect("an object");
        assert_eq!(
            move_member(object, "old", "new"),
            Some(Renamed { replaced: false }),
            "the same number"
        );
        assert_eq!(keys(object), ["new", "a"]);
        let mut object = json(r#"{"a": 2}"#);
        assert_eq!(
            move_member(object.as_object_mut().expect("an object"), "old", "new"),
            None
        );
    }

    /// Block's rename: a `null` old is an absent member: dropped, nothing renamed or replaced.
    #[test]
    fn rename_member_in_place_drops_a_null_old() {
        let mut object = map(r#"{"old": null, "new": 3}"#);
        assert_eq!(rename_member_in_place(&mut object, "old", "new"), None);
        assert_eq!(keys(&object), ["new"]);
        assert_eq!(object["new"], json("3"));
    }

    /// Block's rename: the renamed member stands where `old` stood.
    #[test]
    fn rename_member_in_place_keeps_old_s_place() {
        let mut object = map(r#"{"new": 3, "old": 1, "a": 2}"#);
        assert_eq!(
            rename_member_in_place(&mut object, "old", "new"),
            Some(Renamed { replaced: true })
        );
        assert_eq!(keys(&object), ["new", "a"]);
        assert_eq!(object["new"], json("1"));
        let mut object = map(r#"{"old": 1, "a": 2}"#);
        assert_eq!(
            rename_member_in_place(&mut object, "old", "new"),
            Some(Renamed { replaced: false })
        );
        assert_eq!(keys(&object), ["new", "a"]);
    }

    #[test]
    fn overwrite_tells_only_another_value() {
        let mut object = json(r#"{"k": null, "n": 1}"#);
        let object = object.as_object_mut().expect("an object");
        assert!(!overwrite(object, "k", json("2")), "a null is no value");
        assert!(!overwrite(object, "n", json("1.0")), "the same number");
        assert!(overwrite(object, "n", json("2")));
        assert!(!overwrite(object, "new", json("2")));
        assert_eq!(keys(object), ["k", "n", "new"]);
    }

    #[test]
    fn single_to_array_wraps_a_non_array() {
        let mut object = json(r#"{"a": 1, "b": [1], "c": null}"#);
        let members = object.as_object_mut().expect("an object");
        for key in ["a", "b", "c", "d"] {
            single_to_array(members, key);
        }
        assert_eq!(
            to_json(&object).ok(),
            to_json(&json(r#"{"a": [1], "b": [1], "c": [null]}"#)).ok()
        );
    }

    #[test]
    fn the_jsoncpp_readers_read_as_jsoncpp() {
        use jsoncpp::{float_of, int_of, number_of};
        let invalid_type = Some((Code::from(&core_codes::INVALID_TYPE), Severity::Info));
        let invalid_value = Some((Code::from(&core_codes::INVALID_VALUE), Severity::Info));
        assert_eq!(int_of(&json("3")), (3, None));
        assert_eq!(int_of(&json("3.7")), (3, invalid_value.clone()));
        assert_eq!(int_of(&json("1e12")), (i32::MAX, invalid_value.clone()));
        assert_eq!(int_of(&json("\"3\"")), (0, invalid_type.clone()));
        assert_eq!(int_of(&Value::NULL), (0, None));
        assert_eq!(number_of(&json("2.5")), (2.5, None));
        assert_eq!(number_of(&json("true")), (0.0, invalid_type));
        assert_eq!(float_of(&json("-1e300")), (-f32::MAX, invalid_value));
        assert_eq!(f32_value(0.13), json(&f64::from(0.13_f32).to_string()));
    }

    #[test]
    fn the_findings_are_upgrade_findings_at_their_severity() {
        let mut d = Diagnostics::default();
        let at = JsonPath::new().child("minecraft:block");
        note(&mut d, Code::from(&core_codes::DROPPED_FIELD), at.clone());
        tell(
            &mut d,
            Code::from(&core_codes::INVALID_TYPE),
            Severity::Warning,
            at.clone(),
        );
        kept(&mut d, Code::from(&core_codes::INVALID_VALUE), at.clone());
        let expected = [
            Diagnostic::of(&core_codes::DROPPED_FIELD)
                .at(at.clone())
                .phase(Phase::Upgrade),
            Diagnostic::of(&core_codes::INVALID_TYPE)
                .severity(Severity::Warning)
                .at(at.clone())
                .phase(Phase::Upgrade),
        ];
        let found: Vec<_> = d.iter().cloned().collect();
        assert_eq!(found[..2], expected);
        assert_eq!(found[2].severity, Severity::Error);
        assert!(found[2].kept);
    }
}
