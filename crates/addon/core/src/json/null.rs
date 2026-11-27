//! A member written as `null`, as the game's reader of a family reads it.
//!
//! facet reads a JSON `null` for an `Option` as `None`, the value an absent member gets, and
//! refuses it for a strict wrapper. The game decides by the reader its loader feeds
//! (the rules below):
//!
//! - **Rule N1, `JsonCpp` input** (entity, features, the item legacy
//!   component chain): the reader reports a member whose value is `null` as not found. A member
//!   written as `null` is an absent member: it keeps its default, and a
//!   required one is `missing required field`. [`NullRule::Absent`].
//! - **Rule N2, the Minecraft Document reader** (every other Cereal family: block, the modern
//!   item, biome, dimension, camera, trade, …): the strict reader finds the member, and its
//!   type reads `null` and fails: a type error that rejects the document. Only an untyped value
//!   (the game's dynamic value) holds a `null`. [`NullRule::Rejected`].
//!
//! # What the readers do with it
//!
//! The reader takes the rule as the slice's `Reading::null` (`NullHandling`): under N2 a `null`
//! where a value is read is a wrong type; under N1 the member is absent, told by
//! [`tell_null_members`] (an information: the write-back writes the default where the file
//! has `null`). A reader that fixes a value up before its typed read takes the `null` members
//! out first with [`remove_null_members`] (the struct members [`null_members_of_shape`] finds:
//! it follows the struct shapes, not the inside of a proxied shape or an untyped value; a
//! flattened struct's members are the struct's own).

use facet::{Def, Facet, Field, Shape, StructKind, Type, UserType, Variant};

use super::Value;
use crate::diagnostic::codes as core_codes;
use crate::diagnostic::{
    Code, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, JsonPathSegment, Location, Phase,
    Severity,
};
use crate::json::arms::{Lookup, find_field, struct_fields, untagged_arm};

/// How a family's reader takes a member written as `null`; the family's input kind decides
/// (module documentation).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum NullRule {
    /// `JsonCpp` input (rule N1): the member is absent.
    Absent,
    /// The Minecraft Document reader (rule N2): the member is present and its type rejects `null`.
    Rejected,
}

/// The note of the information [`tell_null_members`] leaves at a member written as `null` that
/// rule N1 read as absent.
pub const NULL_MEMBER_READ_AS_ABSENT: &str = "the member written as `null` is read as absent; written back as its default, or left out for an optional member";

/// Rule N1's mark at each member in `found` (paths below `at`), the members a read took as
/// absent because they were written as `null` ([`remove_null_members`], [`null_members_of_shape`]): an
/// information `ValueRewritten` with the note [`NULL_MEMBER_READ_AS_ABSENT`]. The game says
/// nothing there; the crate writes the member's default back where the file has `null`, and a
/// place the write-back changes carries a finding, of the lowest severity, a lint.
pub fn tell_null_members(found: &[JsonPath], at: &JsonPath, d: &mut Diagnostics) {
    for path in found {
        let mut full = at.clone();
        full.append(path);
        let mut finding = Diagnostic::new(
            Code::from(&core_codes::VALUE_REWRITTEN),
            Severity::Info,
            Phase::Parse,
            Location::new(None, None, full, None),
        );
        finding.detail = Some(Box::new(DiagnosticDetail::Note(NULL_MEMBER_READ_AS_ABSENT)));
        d.push(finding);
    }
}

/// Removes from `json` the members [`null_members_of_shape`] finds, and returns their paths.
pub fn remove_null_members(shape: &'static Shape, json: &mut Value) -> Vec<JsonPath> {
    let found = null_members_of_shape(shape, json);
    remove_members(json, &found);
    found
}

/// Every struct member of `json`, read as `shape`, written as `null` (rule N1's absent
/// members), in document order: the members of each struct the shape reads an object as,
/// through `Option`s, pointers, transparent newtypes, flattened structs (whose members are the
/// struct's own), list elements, map values and enum variants (the arm an untagged enum takes,
/// the variant a tag or a single key names). Map entries, list elements and the inside of an
/// untyped value or a proxied shape are not members.
///
/// Runs on a stack the depth of `json` can take; a value nested past `MAX_JSON_DEPTH` has none.
pub fn null_members_of_shape(shape: &'static Shape, json: &Value) -> Vec<JsonPath> {
    let Ok(levels) = super::write::levels_of(json) else {
        return Vec::new();
    };
    super::stack::with_stack_for(levels, || {
        let mut found = Vec::new();
        nulls_in(shape, json, &mut JsonPath::new(), &mut found);
        found
    })
}

/// The null members of `json` read as `shape`, at `path`.
fn nulls_in(shape: &'static Shape, json: &Value, path: &mut JsonPath, found: &mut Vec<JsonPath>) {
    if shape.effective_proxy(None).is_some() || shape.id == Value::SHAPE.id {
        return;
    }
    let element = match shape.def {
        Def::Option(option) => {
            if !json.is_null() {
                nulls_in(option.t(), json, path, found);
            }
            return;
        }
        Def::Pointer(pointer) => {
            if let Some(pointee) = pointer.pointee() {
                nulls_in(pointee, json, path, found);
            }
            return;
        }
        Def::Map(map) => {
            for (key, item) in json.as_object().into_iter().flatten() {
                path.push(key.as_str());
                nulls_in(map.v(), item, path, found);
                path.pop();
            }
            return;
        }
        Def::List(list) => Some(list.t()),
        Def::Array(array) => Some(array.t()),
        Def::Slice(slice) => Some(slice.t()),
        Def::Set(set) => Some(set.t()),
        _ => None,
    };
    if let Some(element) = element {
        for (index, item) in json.as_array().into_iter().flatten().enumerate() {
            path.push_index(index);
            nulls_in(element, item, path, found);
            path.pop();
        }
        return;
    }
    if shape.scalar_type().is_some() {
        return;
    }
    match shape.ty {
        Type::User(UserType::Struct(st)) if shape.is_transparent() && st.fields.len() == 1 => {
            nulls_in(st.fields[0].shape(), json, path, found);
        }
        Type::User(UserType::Struct(st)) => nulls_of_members(st.fields, json, path, found),
        Type::User(UserType::Enum(en)) => nulls_of_enum(shape, en.variants, json, path, found),
        _ => {}
    }
}

/// The null members of an object read as the named `fields`.
fn nulls_of_members(
    fields: &'static [Field],
    json: &Value,
    path: &mut JsonPath,
    found: &mut Vec<JsonPath>,
) {
    for (key, item) in json.as_object().into_iter().flatten() {
        path.push(key.as_str());
        null_member(fields, key.as_str(), item, path, found);
        path.pop();
    }
}

/// The member `key: item` of an object read as `fields`: a field's (null itself, and the null
/// members below it), or a flattened struct's, at the same path.
fn null_member(
    fields: &'static [Field],
    key: &str,
    item: &Value,
    path: &mut JsonPath,
    found: &mut Vec<JsonPath>,
) {
    match find_field(fields, key) {
        Lookup::Field(field) => {
            if item.is_null() {
                found.push(path.clone());
            }
            if field.proxy().is_none() {
                nulls_in(field.shape(), item, path, found);
            }
        }
        Lookup::Flattened(flattened) => {
            if let Some(inner) = struct_fields(flattened.shape()) {
                null_member(inner, key, item, path, found);
            }
        }
        Lookup::Unknown => {}
    }
}

/// The null members of an enum's payload: the arm an untagged enum reads `json` as, the
/// variant an internal tag names (its fields beside the tag), or the one key of an external tag.
fn nulls_of_enum(
    shape: &'static Shape,
    variants: &'static [Variant],
    json: &Value,
    path: &mut JsonPath,
    found: &mut Vec<JsonPath>,
) {
    let by_name = |name: &str| variants.iter().find(|v| v.effective_name() == name);
    if shape.is_untagged() {
        if let Some(index) = untagged_arm(variants, json) {
            nulls_of_variant(&variants[index], json, path, found);
        }
        return;
    }
    let Some(object) = json.as_object() else {
        return;
    };
    if let Some(tag) = shape.get_tag_attr() {
        let Some(variant) = object
            .get(tag)
            .and_then(Value::as_string)
            .and_then(|name| by_name(name.as_str()))
        else {
            return;
        };
        let mut rest = object.clone();
        rest.remove(tag);
        nulls_of_variant(variant, &Value::from(rest), path, found);
        return;
    }
    let mut members = object.iter();
    if let (Some((name, payload)), None) = (members.next(), members.next())
        && let Some(variant) = by_name(name.as_str())
    {
        path.push(name.as_str());
        nulls_of_variant(variant, payload, path, found);
        path.pop();
    }
}

/// A variant's payload: a struct variant's members, or a newtype variant's one field.
fn nulls_of_variant(
    variant: &'static Variant,
    json: &Value,
    path: &mut JsonPath,
    found: &mut Vec<JsonPath>,
) {
    let fields = variant.data.fields;
    match variant.data.kind {
        StructKind::Struct => nulls_of_members(fields, json, path, found),
        StructKind::TupleStruct | StructKind::Tuple if fields.len() == 1 => {
            nulls_in(fields[0].shape(), json, path, found);
        }
        _ => {}
    }
}

/// Removes from `json` the struct members at `found` ([`member_paths`]).
pub(crate) fn remove_members(json: &mut Value, found: &[JsonPath]) {
    for path in found {
        let Some(key) = path.last_key() else {
            continue;
        };
        let mut node = Some(&mut *json);
        for step in path.segments().take(path.len() - 1) {
            node = node.and_then(|value| match step {
                JsonPathSegment::Key(key) => value.as_object_mut()?.get_mut(key),
                JsonPathSegment::Index(index) => value.as_array_mut()?.get_mut(index),
            });
        }
        if let Some(object) = node.and_then(Value::as_object_mut) {
            object.remove(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    #[derive(Debug, Default, PartialEq, Facet)]
    struct Inner {
        #[facet(default)]
        size: Option<u8>,
    }

    /// A component of a `JsonCpp` family in small: bare strict members with defaults, and a
    /// required one.
    #[derive(Debug, PartialEq, Facet)]
    struct Bare {
        #[facet(default = 3.0_f32)]
        days: f32,
        #[facet(default)]
        flag: bool,
        #[facet(default)]
        inner: Option<BareInner>,
        #[facet(default)]
        any: Value,
    }

    #[derive(Debug, PartialEq, Facet)]
    struct BareInner {
        required: i32,
        #[facet(default = 7)]
        count: i32,
    }

    fn json(text: &str) -> Value {
        crate::json::from_str_jsonc(text).unwrap()
    }

    fn paths(found: &[JsonPath]) -> Vec<String> {
        found.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn null_members_are_struct_members_only() {
        let text = r#"{"days": null, "inner": {"required": 1, "count": null}, "any": {"k": null}}"#;
        assert_eq!(
            paths(&null_members_of_shape(Bare::SHAPE, &json(text))),
            ["days", "inner/count"]
        );
        let mut value = json(text);
        let removed = remove_null_members(Bare::SHAPE, &mut value);
        assert_eq!(paths(&removed), ["days", "inner/count"]);
        assert_eq!(
            value,
            json(r#"{"inner": {"required": 1}, "any": {"k": null}}"#)
        );
    }

    #[derive(Debug, Facet)]
    struct Shaped {
        #[facet(default)]
        a: Option<u8>,
        #[facet(default, rename = "b_written", alias = "b_alias")]
        b: Option<Inner>,
        #[facet(flatten)]
        flat: Flat,
        #[facet(default)]
        wrapped: Option<Wrapped>,
        #[facet(default)]
        tagged: Option<Tagged>,
        #[facet(default)]
        external: Option<External>,
        #[facet(default)]
        either: Vec<Either>,
        #[facet(default)]
        boxed: Option<Box<Inner>>,
        #[facet(default)]
        map: IndexMap<String, Vec<Inner>>,
        #[facet(skip)]
        skipped: Option<u8>,
    }

    #[derive(Debug, Default, Facet)]
    struct Flat {
        #[facet(default)]
        f: Option<u8>,
    }

    #[derive(Debug, Facet)]
    #[facet(transparent)]
    struct Wrapped(Inner);

    #[derive(Debug, Facet)]
    #[repr(u8)]
    #[facet(tag = "type")]
    #[allow(dead_code)]
    enum Tagged {
        One { x: Option<u8> },
        Two { y: Option<u8> },
    }

    #[derive(Debug, Facet)]
    #[repr(u8)]
    #[allow(dead_code)]
    enum External {
        Named { x: Option<u8> },
        Newtype(Inner),
    }

    #[derive(Debug, Facet)]
    #[repr(u8)]
    #[facet(untagged)]
    #[allow(dead_code)]
    enum Either {
        Left { l: Option<u8> },
        Right { r: Option<u8>, more: Option<u8> },
        Many(Vec<Inner>),
    }

    /// The shape's own finder finds the null members of every kind of shape, in the order of the
    /// text (checked against the old walk while it was there).
    #[test]
    fn the_null_members_of_a_shape_are_found_through_every_kind_of_shape() {
        let cases: &[(&str, &[&str])] = &[
            ("{}", &[]),
            ("null", &[]),
            (
                r#"{"a": null, "b_written": null, "b_alias": {"size": null}}"#,
                &["a", "b_written", "b_alias/size"],
            ),
            (r#"{"f": null, "skipped": null, "unknown": null}"#, &["f"]),
            (
                r#"{"wrapped": {"size": null}, "boxed": {"size": null}}"#,
                &["wrapped/size", "boxed/size"],
            ),
            (
                r#"{"tagged": {"type": "Two", "y": null, "x": null}}"#,
                &["tagged/y"],
            ),
            (r#"{"tagged": {"type": "None", "y": null}}"#, &[]),
            (
                r#"{"external": {"Named": {"x": null}}}"#,
                &["external/Named/x"],
            ),
            (
                r#"{"external": {"Newtype": {"size": null}}}"#,
                &["external/Newtype/size"],
            ),
            (
                r#"{"external": {"Named": {"x": null}, "Newtype": {}}}"#,
                &[],
            ),
            (
                r#"{"either": [{"l": null}, {"r": 1, "more": null}, [{"size": null}], 3]}"#,
                &["either/0/l", "either/1/more", "either/2/0/size"],
            ),
            (
                r#"{"map": {"k": [{"size": null}, null], "j": null}}"#,
                &["map/k/0/size"],
            ),
            (
                r#"{"a": 1, "b_written": [null], "wrapped": null, "map": []}"#,
                &["wrapped"],
            ),
        ];
        for (text, expected) in cases {
            let value = json(text);
            let found = null_members_of_shape(Shaped::SHAPE, &value);
            assert_eq!(paths(&found), *expected, "{text}");
        }
    }
}
