//! Unknown members: the keys of a document no field of its typed struct takes.
//!
//! facet skips an unknown key silently. This walk compares the struct shapes with a `Value`
//! read from the same text: field names come from `effective_name()` (and `alias`), fields
//! with `should_skip_deserializing()` take no key, a flattened struct's fields are the
//! struct's own, and every other key is reported with its JSON path (`UnrecognizedField`). `keep_unknown_members` also keeps each one of a struct that keeps
//! its unknown members in a document's `Unknown` table, by the path of its object.
//!
//! Which struct keeps them ([`keeps`](keep::keeps)): one marked with `#[facet(skip)] extra: Extra`, or whose
//! `schema::unknown` is `keep`; and, under a reading that keeps unknown members
//! (`Reading::keeps_unknown`), every struct whose `schema::unknown` is not `report`, `reject` or
//! `ignore`. The word is the struct's own (`#[facet(schema::unknown = ..)]` on the struct, or
//! on a variant for its fields), or the field's that holds it, which comes first.

use facet::{Def, Facet, Field, Peek, Shape, Type, UserType};

use super::Value;
use super::extra::Extra;
use super::scalar::{Declared, ScalarRules, Verdict};
use super::slice::element_of;
use crate::diagnostic::JsonPath;
use crate::document::Unknown as Table;
use crate::schema::UnknownRule;

mod enums;
mod keep;
mod misread;
mod scalars;
mod untagged;

use enums::walk_enum;
pub(crate) use keep::door_unknown_members;
pub use keep::{
    keep_members, keep_members_and_range_members, keep_unknown_members, keep_unknown_members_with,
};
pub use misread::{BEYOND_FLOAT_MEMBER, Misread, Misreading};
use scalars::{judge_scalar, misreading};

/// Every member of `json` that no field of `shape` takes, at any depth, with its path, in
/// document order. Proxied types (strict wrappers, Molang fields) and untyped `Value`s take
/// anything and are not walked; a member of the wrong JSON type is the typed parse's to reject
/// and is not walked either. An untagged enum is walked through the arm facet would read the
/// value as: for an object, the first object arm left alone by a key in document order, else
/// the first object arm; for an array, the first list arm. With the typed value at hand, prefer
/// [`keep_unknown_members`], which takes the arm the value actually has.
///
/// The walk needs no stack of the caller that grows with `json`: a megabyte on the calling
/// thread is enough at any depth to `MAX_JSON_DEPTH` (`MAX_JSON_DEPTH` names what is outside
/// that rule). A value nested deeper is no text a reader of the crate reads: nothing of it is
/// walked, here and in the functions below.
pub fn unknown_members(shape: &'static Shape, json: &Value) -> Vec<(JsonPath, Value)> {
    Walk::new(shape, json)
        .run()
        .unknowns
        .into_iter()
        .map(|unknown| (unknown.path, unknown.value))
        .collect()
}

/// Whether `json` nests past `MAX_JSON_DEPTH`, which only a value built by hand can: the walks
/// of this module report nothing of such a value, so an empty answer of theirs says "no
/// unknown member" only where this is `false`.
#[must_use]
pub fn too_deep_to_walk(json: &Value) -> bool {
    super::write::levels_of(json).is_err()
}

/// One unknown member.
#[derive(Debug)]
pub(crate) struct Unknown {
    /// The member's path, its key last.
    pub(crate) path: JsonPath,
    /// Its key.
    key: Option<String>,
    /// Its value.
    pub(crate) value: Value,
    /// How to reach the struct that owns it from the root value, when every step is one facet's
    /// reflection can take mutably.
    route: Option<Vec<Step>>,
    /// Whether the struct that owns it is marked with an `extra` field.
    has_extra: bool,
    /// The struct's `schema::unknown` word, the field's that holds it before its own.
    rule: Option<UnknownRule>,
}

/// One mutable step from a value to a value inside it.
#[derive(Clone, Copy, Debug)]
enum Step {
    /// A struct field, by index.
    Field(usize),
    /// A field of an enum variant: the variant's index, then the field's. Taken only when that
    /// variant is the active one.
    VariantField(usize, usize),
    /// The value inside `Some`.
    Some,
    /// A list element, by index.
    Index(usize),
}

/// The walk's position: the JSON path and the mutable route (`None` once a step cannot be
/// taken mutably).
struct Cursor {
    path: JsonPath,
    route: Option<Vec<Step>>,
    /// The scalars read as another value, so far.
    misread: Vec<Misread>,
    /// Whether the members a range in its object form does not read are unknown members.
    ranges: bool,
    /// The places, so far, where the shape reads a value and the JSON has `null` (`json::null`,
    /// rule N2).
    nulls: Vec<JsonPath>,
    /// The struct members, so far, written as `null` (`json::null`, rule N1).
    null_members: Vec<JsonPath>,
    /// The slice's scalar rules, when the walk is the scalar pre-pass: each scalar is judged
    /// by them ([`ScalarCall`]) instead of being looked at for a misreading.
    rules: Option<ScalarRules>,
    /// The scalars judged, so far, that are not read as written.
    scalars: Vec<ScalarCall>,
    /// Whether the walk without rules looks for misread scalars ([`Cursor::misread`]).
    tell_misread: bool,
    /// The `schema::unknown` of the field being walked, for the object it holds (taken by the
    /// first object below it, through `Option`, pointers, list elements and map values).
    field_rule: Option<UnknownRule>,
}

/// What the scalar pre-pass decided for one scalar of the text: every verdict but
/// [`Verdict::Keep`], in document order.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScalarCall {
    /// The scalar's path from the root of the value walked.
    pub(crate) path: JsonPath,
    /// What its member reads it as.
    pub(crate) verdict: Verdict,
    /// The scalar's declared type; `None` for a scalar no arm of its untagged enum takes
    /// (`arm`), or a `null` given for a struct.
    pub(crate) declared: Option<Declared>,
    /// Whether the scalar is a value of an untagged enum none of whose scalar arms takes it:
    /// refused there, before any arm is read.
    pub(crate) arm: bool,
    /// The length of the array the text has where a scalar belongs (under every profile but
    /// [`Scalars::ConvertAndTell`]): facet refuses it at its `[`, a strict wrapper at its last
    /// element (`json::scalar::parse_with_rules` says it there).
    pub(crate) list: Option<usize>,
}

impl Cursor {
    fn step(&mut self, step: Option<Step>) {
        match (&mut self.route, step) {
            (Some(route), Some(step)) => route.push(step),
            (route, _) => *route = None,
        }
    }
}

/// The typed value at the walk's position, when the walk has one.
type Typed<'m, 'f> = Option<Peek<'m, 'f>>;

/// One walk of `json` as `shape`, and what it looks for besides the unknown members: the
/// options of [`Walk::run`]. Every option is off at [`Walk::new`]: no typed value, no range
/// members, no scalar rules, no misread scalars.
pub(crate) struct Walk<'a, 'm, 'f> {
    shape: &'static Shape,
    json: &'a Value,
    /// The value deserialised from `json`, whose enum variants then decide the arms.
    typed: Typed<'m, 'f>,
    /// Whether the members a range in its object form does not read are unknown members.
    ranges: bool,
    /// The slice's scalar rules, for the scalar pre-pass ([`WalkReport::scalars`]).
    rules: Option<ScalarRules>,
    /// Whether the walk without rules looks for misread scalars ([`WalkReport::misread`]).
    misreads: bool,
}

impl<'a, 'm, 'f> Walk<'a, 'm, 'f> {
    /// The walk of `json` as `shape`, every option off.
    pub(crate) fn new(shape: &'static Shape, json: &'a Value) -> Self {
        Self {
            shape,
            json,
            typed: None,
            ranges: false,
            rules: None,
            misreads: false,
        }
    }

    /// The walk of `json` as the shape of `target`, the value deserialised from it, with
    /// `target` beside it ([`Walk::typed`]).
    pub(crate) fn of<T: Facet<'f> + ?Sized>(target: &'m T, json: &'a Value) -> Self {
        Self::new(T::SHAPE, json).typed(Some(Peek::new(target)))
    }

    /// The walk with `typed`, the value deserialised from `json`, beside it.
    pub(crate) fn typed(mut self, typed: Typed<'m, 'f>) -> Self {
        self.typed = typed;
        self
    }

    /// The walk reporting, when `ranges`, the members a range does not read.
    pub(crate) fn ranges(mut self, ranges: bool) -> Self {
        self.ranges = ranges;
        self
    }

    /// The walk looking, when `misreads`, for the scalars read as another value; a walk with
    /// scalar rules judges them instead ([`Walk::scalars`]).
    pub(crate) fn misreads(mut self, misreads: bool) -> Self {
        self.misreads = misreads;
        self
    }

    /// The scalar pre-pass's walk: every scalar judged under `rules`
    /// ([`WalkReport::scalars`]), an untagged enum's scalar arm chosen as the rules choose it.
    /// Untyped: it runs before the typed read.
    pub(crate) fn scalars(mut self, rules: ScalarRules) -> Self {
        self.rules = Some(rules);
        self
    }

    /// Walks.
    pub(crate) fn run(self) -> WalkReport {
        // The walk calls itself for each level of `json`: it runs with the stack that depth
        // can take, as facet's read of the typed value did, and not on what its caller
        // happens to have left. A value nested past the limit is no text a reader of
        // the crate reads and is not walked.
        let Ok(levels) = super::write::levels_of(self.json) else {
            return WalkReport::default();
        };
        super::stack::with_stack_for(levels, || self.run_on_this_stack())
    }

    /// [`Walk::run`] on the stack of its caller: the walk calls itself for each level of
    /// `json`.
    fn run_on_this_stack(self) -> WalkReport {
        let mut out = Vec::new();
        let mut cursor = Cursor {
            path: JsonPath::new(),
            route: Some(Vec::new()),
            misread: Vec::new(),
            ranges: self.ranges,
            nulls: Vec::new(),
            null_members: Vec::new(),
            rules: self.rules,
            scalars: Vec::new(),
            tell_misread: self.misreads,
            field_rule: None,
        };
        walk_value(self.shape, self.json, self.typed, &mut cursor, &mut out);
        WalkReport {
            unknowns: out,
            misread: cursor.misread,
            nulls: cursor.nulls,
            null_members: cursor.null_members,
            scalars: cursor.scalars,
        }
    }
}

/// What one walk of a value found.
#[derive(Default)]
pub(crate) struct WalkReport {
    /// The unknown members.
    pub(crate) unknowns: Vec<Unknown>,
    /// The scalars read as another value, for a walk that looks for them (`keep_members`).
    pub(crate) misread: Vec<Misread>,
    /// Every place where the shape reads a value (an `Option` among them) and the JSON has
    /// `null`; opaque places (`Value`, `RawJson`, proxies) and a unit are left out.
    pub(crate) nulls: Vec<JsonPath>,
    /// Every struct member written as `null`, whatever its type.
    pub(crate) null_members: Vec<JsonPath>,
    /// The scalars the pre-pass does not read as written ([`Walk::scalars`]); empty for a
    /// walk without rules.
    pub(crate) scalars: Vec<ScalarCall>,
}

fn walk_value(
    shape: &'static Shape,
    json: &Value,
    typed: Typed<'_, '_>,
    cursor: &mut Cursor,
    out: &mut Vec<Unknown>,
) {
    // A `RawJson` is parsed on its own, like a proxy or a `Value`.
    if shape.effective_proxy(None).is_some()
        || shape.id == Value::SHAPE.id
        || shape.id == <facet_json::RawJson<'static> as Facet<'static>>::SHAPE.id
    {
        walk_range(shape, json, cursor, out);
        return;
    }
    if json.is_null() && !takes_null(shape) {
        cursor.nulls.push(cursor.path.clone());
    }
    match shape.def {
        Def::Option(option) => {
            if !json.is_null() {
                let inner = typed
                    .and_then(|t| t.into_option().ok())
                    .and_then(facet::PeekOption::value);
                with_step(cursor, Some(Step::Some), |cursor| {
                    walk_value(option.t(), json, inner, cursor, out);
                });
            }
            return;
        }
        Def::Pointer(pointer) => {
            if let Some(pointee) = pointer.pointee() {
                let inner = typed
                    .and_then(|t| t.into_pointer().ok())
                    .and_then(|p| p.borrow_inner());
                with_step(cursor, None, |cursor| {
                    walk_value(pointee, json, inner, cursor, out);
                });
            }
            return;
        }
        Def::List(_) | Def::Array(_) | Def::Slice(_) | Def::Set(_) => {
            let (Some(array), Some(element)) = (json.as_array(), element_or_set(shape)) else {
                return;
            };
            let list = typed.and_then(|t| t.into_list_like().ok());
            let mutable = matches!(shape.def, Def::List(_));
            for (index, item) in array.iter().enumerate() {
                cursor.path.push_index(index);
                let typed_item = list.and_then(|l| l.get(index));
                with_step(cursor, mutable.then_some(Step::Index(index)), |cursor| {
                    walk_value(element, item, typed_item, cursor, out);
                });
                cursor.path.pop();
            }
            return;
        }
        Def::Map(map) => {
            walk_map(map.v(), json, typed, cursor, out);
            return;
        }
        _ => {}
    }
    if let Some(ty) = shape.scalar_type() {
        if let Some(rules) = cursor.rules {
            judge_scalar(ty, json, rules, cursor);
        } else if cursor.tell_misread
            && let Some(kind) = misreading(ty, json)
        {
            cursor.misread.push(Misread {
                path: cursor.path.clone(),
                kind,
            });
        }
        return;
    }
    match shape.ty {
        // A `null` the typed parse took for a struct is a struct of defaults: another value
        // than the document writes (an `Option` above reads it as none).
        Type::User(UserType::Struct(_)) if json.is_null() => {
            if cursor.rules.is_some() {
                cursor.scalars.push(ScalarCall {
                    path: cursor.path.clone(),
                    verdict: Verdict::Converted,
                    declared: None,
                    arm: false,
                    list: None,
                });
            } else if cursor.tell_misread {
                cursor.misread.push(Misread {
                    path: cursor.path.clone(),
                    kind: Misreading::WrongType,
                });
            }
        }
        Type::User(UserType::Struct(st)) => walk_struct(shape, st.fields, json, typed, cursor, out),
        Type::User(UserType::Enum(en)) => walk_enum(shape, en.variants, json, typed, cursor, out),
        _ => {}
    }
}

/// A struct of the named `fields` (or a transparent newtype, walked as its one field), whose
/// own `schema::unknown` is in `shape`'s attributes.
fn walk_struct(
    shape: &'static Shape,
    fields: &'static [Field],
    json: &Value,
    typed: Typed<'_, '_>,
    cursor: &mut Cursor,
    out: &mut Vec<Unknown>,
) {
    let typed_fields = typed.and_then(|t| t.into_struct().ok());
    let typed_field = |index: usize| typed_fields.and_then(|f| f.field(index).ok());
    if shape.is_transparent() && fields.len() == 1 {
        with_step(cursor, Some(Step::Field(0)), |cursor| {
            walk_value(fields[0].shape(), json, typed_field(0), cursor, out);
        });
    } else {
        let own = rule_of(shape.attributes.iter());
        walk_object(fields, own, &Step::Field, &typed_field, json, cursor, out);
    }
}

/// Whether a value of `shape` holds a JSON `null` as what it is: a unit, or an `Option` of an
/// untyped value (vanilla's dynamic value holds a `null`).
fn takes_null(shape: &'static Shape) -> bool {
    match shape.def {
        Def::Option(option) => option.t().id == Value::SHAPE.id,
        _ => shape.scalar_type() == Some(facet::ScalarType::Unit),
    }
}

/// The entries of a JSON object read as a map whose values have the shape `value`.
///
/// A function of its own, so that what it holds is not in the frame of [`walk_value`],
/// which calls itself for each level of a value and runs on the stack of its caller.
// The typed value of each key is found once for the map: looked for from the first entry
// at every key, a map of n entries cost n².
#[inline(never)]
fn walk_map(
    value: &'static Shape,
    json: &Value,
    typed: Typed<'_, '_>,
    cursor: &mut Cursor,
    out: &mut Vec<Unknown>,
) {
    let Some(object) = json.as_object() else {
        return;
    };
    let mut typed_items = std::collections::HashMap::new();
    if let Some(typed_map) = typed.and_then(|t| t.into_map().ok()) {
        for (typed_key, typed_item) in typed_map.iter() {
            if let Some(typed_key) = typed_key.as_str() {
                typed_items.entry(typed_key).or_insert(typed_item);
            }
        }
    }
    for (key, item) in object {
        cursor.path.push(key.as_str());
        let typed_item = typed_items.get(key.as_str()).copied();
        with_step(cursor, None, |cursor| {
            walk_value(value, item, typed_item, cursor, out);
        });
        cursor.path.pop();
    }
}

/// The members of a range given as an object that the range does not read, and the elements
/// of one given as a list behind its two ends, as unknown members without a route and with the
/// word `report` (a range has no `extra`: reported, not kept), when the walk was asked for them.
fn walk_range(shape: &'static Shape, json: &Value, cursor: &mut Cursor, out: &mut Vec<Unknown>) {
    if !cursor.ranges || !crate::shared::is_range(shape) {
        return;
    }
    // The elements of a list behind the two ends (an integer range reads the first two).
    if let Some(list) = json.as_array() {
        for (index, element) in list.iter().enumerate().skip(2) {
            cursor.path.push_index(index);
            out.push(Unknown {
                path: cursor.path.clone(),
                key: None,
                value: element.clone(),
                route: None,
                has_extra: false,
                rule: Some(UnknownRule::Report),
            });
            cursor.path.pop();
        }
        return;
    }
    let Some(object) = json.as_object() else {
        return;
    };
    for key in crate::shared::unread_range_members(object) {
        cursor.path.push(key);
        out.push(Unknown {
            path: cursor.path.clone(),
            key: None,
            value: object.get(key).cloned().unwrap_or(Value::NULL),
            route: None,
            has_extra: false,
            rule: Some(UnknownRule::Report),
        });
        cursor.path.pop();
    }
}

/// Runs `f` one route step deeper.
fn with_step(cursor: &mut Cursor, step: Option<Step>, f: impl FnOnce(&mut Cursor)) {
    let saved = cursor.route.clone();
    cursor.step(step);
    f(cursor);
    cursor.route = saved;
}

/// The element shape of a list, array, slice or set.
fn element_or_set(shape: &'static Shape) -> Option<&'static Shape> {
    match shape.def {
        Def::Set(set) => Some(set.t()),
        _ => element_of(shape),
    }
}

/// The members of a JSON object read as the named `fields` (of a struct, or of a struct
/// variant, whose `schema::unknown` is `own`): known keys are walked, the others reported with
/// the struct's mark and word, which the keep step reads ([`keeps`](keep::keeps)). `to_step` and
/// `typed_field` give the route step and the typed value of the field at an index.
fn walk_object<'m, 'f>(
    fields: &'static [Field],
    own: Option<UnknownRule>,
    to_step: &dyn Fn(usize) -> Step,
    typed_field: &dyn Fn(usize) -> Typed<'m, 'f>,
    json: &Value,
    cursor: &mut Cursor,
    out: &mut Vec<Unknown>,
) {
    let Some(object) = json.as_object() else {
        return;
    };
    let held = cursor.field_rule.take();
    let owner = (fields.iter().any(is_extra), held.or(own));
    for (key, item) in object {
        let key = key.as_str();
        cursor.path.push(key);
        walk_member(
            fields,
            to_step,
            typed_field,
            (key, item),
            owner,
            cursor,
            out,
        );
        cursor.path.pop();
    }
    // A list or a map below the field holds more objects the field's word is for.
    cursor.field_rule = held;
}

/// The member `key` (whose value is `item`, at the cursor's path) of an object read as the
/// named `fields`: [`walk_object`]'s step for one key. A key a flattened struct field takes is
/// walked as that struct's member, one route step deeper and at the same JSON path: a
/// flattened field adds no JSON level. `owner` is the object's mark and word, the flattened
/// struct's members being the object's own.
fn walk_member<'m, 'f>(
    fields: &'static [Field],
    to_step: &dyn Fn(usize) -> Step,
    typed_field: &dyn Fn(usize) -> Typed<'m, 'f>,
    (key, item): (&str, &Value),
    owner: (bool, Option<UnknownRule>),
    cursor: &mut Cursor,
    out: &mut Vec<Unknown>,
) {
    match find_field(fields, key) {
        Lookup::Field(index, field) => {
            if item.is_null() {
                cursor.null_members.push(cursor.path.clone());
            }
            if field.proxy().is_none() {
                with_step(cursor, Some(to_step(index)), |cursor| {
                    cursor.field_rule = rule_of(field.attributes.iter());
                    walk_value(field.shape(), item, typed_field(index), cursor, out);
                    cursor.field_rule = None;
                });
            }
        }
        Lookup::Flattened(index, flattened) => {
            // A flattened map takes every key as an entry it holds as written.
            let Some(inner) = struct_fields(flattened.shape()) else {
                return;
            };
            let typed_inner = typed_field(index).and_then(|t| t.into_struct().ok());
            let typed_inner = |inner: usize| typed_inner.and_then(|f| f.field(inner).ok());
            with_step(cursor, Some(to_step(index)), |cursor| {
                walk_member(
                    inner,
                    &Step::Field,
                    &typed_inner,
                    (key, item),
                    owner,
                    cursor,
                    out,
                );
            });
        }
        Lookup::Unknown => out.push(Unknown {
            path: cursor.path.clone(),
            key: Some(key.to_owned()),
            value: item.clone(),
            route: cursor.route.clone(),
            has_extra: owner.0,
            rule: owner.1,
        }),
    }
}

/// The `schema::unknown` word among `attributes` (a shape's, a variant's or a field's).
fn rule_of<'a>(mut attributes: impl Iterator<Item = &'a facet::Attr>) -> Option<UnknownRule> {
    attributes
        .find(|attr| attr.ns() == Some("schema") && attr.key() == "unknown")
        .and_then(|attr| attr.get_as::<&'static str>())
        .and_then(|word| UnknownRule::from_word(word))
}

/// Where a JSON key goes among a struct's fields.
enum Lookup {
    /// To the field at this index.
    Field(usize, &'static Field),
    /// Into the flattened field at this index: a struct one of whose fields takes the key, or
    /// a map.
    Flattened(usize, &'static Field),
    /// Nowhere.
    Unknown,
}

fn find_field(fields: &'static [Field], key: &str) -> Lookup {
    for (index, field) in fields.iter().enumerate() {
        if field.should_skip_deserializing() {
            continue;
        }
        if field.is_flattened() {
            let takes = match field.shape().def {
                Def::Map(_) => true,
                _ => struct_fields(field.shape())
                    .is_some_and(|inner| !matches!(find_field(inner, key), Lookup::Unknown)),
            };
            if takes {
                return Lookup::Flattened(index, field);
            }
            continue;
        }
        if field.effective_name() == key || field.alias == Some(key) {
            return Lookup::Field(index, field);
        }
    }
    Lookup::Unknown
}

/// The named fields of a struct shape.
fn struct_fields(shape: &'static Shape) -> Option<&'static [Field]> {
    match shape.ty {
        Type::User(UserType::Struct(st)) => Some(st.fields),
        _ => None,
    }
}

/// `#[facet(skip)] extra: Extra`.
fn is_extra(field: &Field) -> bool {
    field.should_skip_deserializing() && field.shape().id == Extra::SHAPE.id
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::*;

    #[derive(Debug, Facet)]
    struct Inner {
        #[facet(default)]
        v: u8,
        #[facet(skip)]
        extra: Extra,
    }

    #[derive(Debug, Facet)]
    struct Outer {
        #[facet(default)]
        list: Vec<Inner>,
        #[facet(default)]
        map: IndexMap<String, Inner>,
        #[facet(default)]
        boxed: Option<Box<Inner>>,
        #[facet(default)]
        tagged: Option<Tagged>,
    }

    #[derive(Debug, Facet)]
    #[repr(u8)]
    #[allow(
        dead_code,
        reason = "only the shape is walked; no test reads a variant's fields"
    )]
    enum Tagged {
        First { v: u8 },
        Second(Inner),
    }

    /// The paths of the unknown members of `text` read as `Outer`, joined by `/`.
    fn paths(text: &str) -> Vec<String> {
        let json: Value = crate::json::from_str(text).unwrap();
        Walk::new(Outer::SHAPE, &json)
            .run()
            .unknowns
            .iter()
            .map(|u| u.path.to_string())
            .collect()
    }

    /// The typed value of a map entry is found once for the map, not looked for from the
    /// map's first entry at every entry.
    #[test]
    fn the_walk_of_a_typed_map_of_n_entries_costs_n() {
        crate::test_cost::assert_linear_at(
            "core unknown members: a typed map of n entries",
            15_000,
            |n| {
                let entries: Vec<String> = (0..n)
                    .map(|i| format!(r#""k_{i}": {{"v": 1, "zz": 2}}"#))
                    .collect();
                let text = format!(r#"{{"map": {{{}}}}}"#, entries.join(","));
                let typed: Outer = crate::json::from_str(&text).unwrap();
                let json: Value = crate::json::from_str(&text).unwrap();
                (typed, json, n)
            },
            |(typed, json, n)| {
                let unknown = Walk::of(typed, json).run().unknowns;
                assert_eq!(unknown.len(), *n);
                unknown
            },
        );
    }

    #[test]
    fn paths_name_list_indices_and_map_keys() {
        assert_eq!(
            paths(r#"{ "list": [{}, { "a": 1 }], "map": { "k": { "b": 2 } }, "c": 3 }"#),
            ["list/1/a", "map/k/b", "c"]
        );
    }

    #[test]
    fn paths_go_through_boxes_and_enum_variants() {
        assert_eq!(
            paths(r#"{ "boxed": { "a": 1 }, "tagged": { "Second": { "b": 2 } } }"#),
            ["boxed/a", "tagged/Second/b"]
        );
        assert_eq!(
            paths(r#"{ "tagged": { "First": { "c": 3 } } }"#),
            ["tagged/First/c"]
        );
    }

    #[test]
    fn only_structs_with_an_extra_field_reachable_mutably_get_a_route() {
        let json: Value = crate::json::from_str(
            r#"{ "list": [{ "a": 1 }], "map": { "k": { "b": 2 } }, "boxed": { "c": 3 }, "d": 4 }"#,
        )
        .unwrap();
        let routes: Vec<bool> = Walk::new(Outer::SHAPE, &json)
            .run()
            .unknowns
            .iter()
            .map(|u| u.route.is_some() && super::keep::keeps(u.has_extra, u.rule, false))
            .collect();
        // list element: kept; map value, box: unreachable; `Outer` has no `extra`.
        assert_eq!(routes, [true, false, false, false]);
    }

    #[derive(Debug, Facet)]
    struct Scalars {
        #[facet(default)]
        float: f32,
        #[facet(default)]
        double: f64,
        #[facet(default)]
        whole: i32,
        #[facet(default)]
        small: u8,
        #[facet(default)]
        name: String,
        #[facet(default)]
        flag: bool,
        #[facet(default)]
        maybe: Option<f32>,
        #[facet(default)]
        list: Vec<f32>,
        #[facet(default)]
        map: IndexMap<String, Box<Scalars>>,
        #[facet(default)]
        strict: Option<crate::json::StrictInt<i32>>,
    }

    /// The scalars of `text` that `Scalars` reads as another value, as `path: kind`.
    fn misread(text: &str) -> Vec<String> {
        let json: Value = crate::json::from_str(text).unwrap();
        let typed: Scalars = crate::json::from_str(text).unwrap();
        let (_, misread) = keep_members(&typed, &json, &mut Table::new());
        misread
            .iter()
            .map(|m| format!("{}: {:?}", m.path, m.kind))
            .collect()
    }

    #[test]
    fn a_scalar_read_as_the_value_it_spells_is_no_misreading() {
        let none: [&str; 0] = [];
        assert_eq!(
            misread(
                r#"{"float": 1, "double": 1e39, "whole": -7, "small": 255, "name": "7",
                    "flag": true, "maybe": null, "list": [0.5, 3.4028235e38],
                    "map": {"a": {"whole": 2.0, "float": 4294967296}}, "strict": 1}"#
            ),
            none
        );
    }

    #[test]
    fn a_number_beyond_a_float_member_is_a_misreading_at_its_path() {
        assert_eq!(
            misread(
                r#"{"float": 1e39, "double": 1e39, "maybe": -1e39, "list": [0, 1e300],
                    "map": {"a": {"float": 3.5e38}}}"#
            ),
            [
                "float: BeyondFloat",
                "maybe: BeyondFloat",
                "list/1: BeyondFloat",
                "map/a/float: BeyondFloat"
            ]
        );
    }

    #[test]
    fn a_number_an_integer_member_does_not_hold_is_refused_by_the_typed_parse() {
        // Characterization: facet neither saturates nor truncates an integer member, so
        // there is no such misreading to tell.
        for text in [
            r#"{"whole": 0.5}"#,
            r#"{"whole": 1e39}"#,
            r#"{"whole": 4294967296}"#,
            r#"{"whole": -2147483649}"#,
            r#"{"small": 256}"#,
            r#"{"small": -1}"#,
            r#"{"small": 1.5}"#,
            r#"{"map": {"a": {"whole": 4294967296.0}}}"#,
        ] {
            assert!(crate::json::from_str::<Scalars>(text).is_err(), "{text}");
        }
    }

    #[test]
    fn a_scalar_of_another_json_type_is_a_misreading() {
        assert_eq!(
            misread(r#"{"float": "7", "whole": "7", "name": 7, "flag": "true"}"#),
            [
                "float: WrongType",
                "whole: WrongType",
                "name: WrongType",
                "flag: WrongType"
            ]
        );
    }

    #[derive(Debug, Default, Facet)]
    struct Leaf {
        #[facet(default)]
        whole: i32,
    }

    #[derive(Debug, Facet)]
    struct Nulls {
        #[facet(default)]
        list: Vec<Leaf>,
        #[facet(default)]
        one: Leaf,
        #[facet(default)]
        maybe: Option<Leaf>,
    }

    #[test]
    fn a_null_read_as_an_empty_struct_is_a_misreading() {
        // facet reads a `null` as a struct of defaults; an `Option` reads it as none, which
        // is what it spells.
        let misread = |text: &str| -> Vec<String> {
            let json: Value = crate::json::from_str(text).unwrap();
            let typed: Nulls = crate::json::from_str(text).unwrap();
            let (_, misread) = keep_members(&typed, &json, &mut Table::new());
            misread
                .iter()
                .map(|m| format!("{}: {:?}", m.path, m.kind))
                .collect()
        };
        let none: [&str; 0] = [];
        assert_eq!(misread(r#"{"maybe": null, "one": {}, "list": [{}]}"#), none);
        assert_eq!(
            misread(r#"{"list": [{"whole": 1}, null], "one": null}"#),
            ["list/1: WrongType", "one: WrongType"]
        );
    }

    #[derive(Debug, Default, Facet)]
    struct Ranged {
        #[facet(default)]
        interval: Option<crate::shared::FloatRange>,
        #[facet(default)]
        counts: Vec<crate::shared::IntRange>,
        #[facet(skip)]
        extra: Extra,
    }

    #[test]
    fn a_member_a_range_does_not_read_is_unknown_when_asked() {
        let text = r#"{"interval": {"a": 1, "min": 2}, "counts": [{"range_min": 1, "max": 3}, 4, [1, 2]]}"#;
        let json: Value = crate::json::from_str(text).unwrap();
        let typed: Ranged = crate::json::from_str(text).unwrap();
        assert!(
            keep_members(&typed, &json, &mut Table::new()).0.is_empty(),
            "not asked"
        );
        let mut kept = Table::new();
        let paths: Vec<String> = keep_members_and_range_members(&typed, &json, &mut kept)
            .0
            .iter()
            .map(|(path, _)| path.to_string())
            .collect();
        assert_eq!(paths, ["interval/a", "counts/0/max"]);
        assert!(kept.is_empty(), "a range has no extra: not kept");
    }

    #[test]
    fn an_element_a_range_given_as_a_list_does_not_read_is_unknown_when_asked() {
        // An integer range takes the first two numbers of a list; a
        // third went without a word.
        let text = r#"{"interval": [1, 2], "counts": [[1, 2], [3, 4, 5, 6]]}"#;
        let json: Value = crate::json::from_str(text).unwrap();
        let typed: Ranged = crate::json::from_str(text).unwrap();
        assert!(
            keep_members(&typed, &json, &mut Table::new()).0.is_empty(),
            "not asked"
        );
        let paths: Vec<String> = keep_members_and_range_members(&typed, &json, &mut Table::new())
            .0
            .iter()
            .map(|(path, _)| path.to_string())
            .collect();
        assert_eq!(paths, ["counts/1/2", "counts/1/3"]);
    }

    #[test]
    fn a_strict_member_is_its_own_to_judge() {
        // The proxy refuses `"1"`; what it takes is not looked at here.
        let none: [&str; 0] = [];
        assert_eq!(misread(r#"{"strict": 5}"#), none);
    }

    /// Every entry point of the walk over one fixture: a struct with an `extra`, a list, a
    /// map, a `Box`, an `Option`, a range object, an untagged enum, the four words and a
    /// struct that says nothing. A characterization: what each returns today, the oracle
    /// of the walk's split into the walk and the keep step.
    mod entry_points {
        use facet::Facet;
        use indexmap::IndexMap;

        use super::super::{
            Extra, Peek, Table, Value, Walk, door_unknown_members, keep_members,
            keep_members_and_range_members, keep_unknown_members, keep_unknown_members_with,
            unknown_members,
        };
        use crate::diagnostic::JsonPath;
        use crate::json::ScalarRules;
        use crate::schema;

        #[derive(Debug, Default, Facet)]
        struct Marked {
            #[facet(default)]
            v: u8,
            #[facet(skip)]
            extra: Extra,
        }

        #[derive(Debug, Default, Facet)]
        #[facet(schema::unknown = "reject")]
        struct Rejects {
            #[facet(default)]
            v: u8,
        }

        #[derive(Debug, Default, Facet)]
        #[facet(schema::unknown = "ignore")]
        struct Ignores {
            #[facet(default)]
            v: u8,
        }

        #[derive(Debug, Default, Facet)]
        #[facet(schema::unknown = "report")]
        struct Reports {
            #[facet(default)]
            v: u8,
        }

        #[derive(Debug, Default, Facet)]
        struct Silent {
            #[facet(default)]
            v: u8,
        }

        #[derive(Debug, Facet)]
        #[facet(untagged)]
        #[repr(u8)]
        #[allow(dead_code, reason = "read, never looked at")]
        enum Arm {
            Number(f32),
            Object {
                #[facet(default)]
                w: u8,
            },
        }

        #[derive(Debug, Facet)]
        struct Root {
            #[facet(default)]
            f: f32,
            #[facet(default)]
            big: f32,
            #[facet(default)]
            list: Vec<Marked>,
            #[facet(default)]
            map: IndexMap<String, Marked>,
            #[facet(default)]
            boxed: Option<Box<Marked>>,
            #[facet(default)]
            maybe: Option<Marked>,
            #[facet(default)]
            one: Marked,
            #[facet(default)]
            nothing: Option<Marked>,
            #[facet(default)]
            range: Option<crate::shared::FloatRange>,
            #[facet(default)]
            arms: Vec<Arm>,
            #[facet(default)]
            rejects: Option<Rejects>,
            #[facet(default)]
            ignores: Option<Ignores>,
            #[facet(default)]
            reports: Option<Reports>,
            #[facet(default)]
            silent: Option<Silent>,
            #[facet(default, schema::unknown = "keep")]
            field_keeps: Option<Silent>,
        }

        const TEXT: &str = r#"{
            "f": "1.5", "big": 1e39,
            "list": [{"v": 1, "a": 1}], "map": {"k": {"b": 2}},
            "boxed": {"c": 3}, "maybe": {"d": 4}, "one": null, "nothing": null,
            "range": {"min": 1, "max": 2, "e": 5}, "arms": [2, {"w": 1, "g": 6}],
            "rejects": {"h": 7}, "ignores": {"i": 8}, "reports": {"j": 9},
            "silent": {"k": 10}, "field_keeps": {"l": 11}, "z": 12
        }"#;

        fn fixture() -> (Root, Value) {
            let typed: Root = crate::json::from_str(TEXT).unwrap();
            let json: Value = crate::json::from_str(TEXT).unwrap();
            (typed, json)
        }

        fn members(found: &[(JsonPath, Value)]) -> Vec<String> {
            found
                .iter()
                .map(|(path, value)| {
                    format!("{path}={}", crate::json::to_json(value).unwrap().trim_end())
                })
                .collect()
        }

        fn table(kept: &Table) -> Vec<String> {
            kept.iter()
                .map(|(at, key, value)| {
                    format!(
                        "{}={}",
                        at.child(key),
                        crate::json::to_json(value).unwrap().trim_end()
                    )
                })
                .collect()
        }

        const ALL: [&str; 11] = [
            "list/0/a=1",
            "map/k/b=2",
            "boxed/c=3",
            "maybe/d=4",
            "arms/1/g=6",
            "rejects/h=7",
            "ignores/i=8",
            "reports/j=9",
            "silent/k=10",
            "field_keeps/l=11",
            "z=12",
        ];

        #[test]
        fn unknown_members_reports_every_member_none_kept() {
            let (_, json) = fixture();
            assert_eq!(members(&unknown_members(Root::SHAPE, &json)), ALL);
        }

        #[test]
        fn keep_unknown_members_keeps_the_marked_and_the_keep_word_reachable() {
            let (typed, json) = fixture();
            let mut kept = Table::new();
            assert_eq!(
                members(&keep_unknown_members(&typed, &json, &mut kept)),
                ALL
            );
            assert_eq!(
                table(&kept),
                ["field_keeps/l=11", "list/0/a=1", "maybe/d=4"]
            );
        }

        #[test]
        fn keep_unknown_members_with_a_keeping_reading_keeps_the_silent_too() {
            let (typed, json) = fixture();
            let mut kept = Table::new();
            assert_eq!(
                members(&keep_unknown_members_with(&typed, &json, true, &mut kept)),
                ALL
            );
            assert_eq!(
                table(&kept),
                [
                    "z=12",
                    "arms/1/g=6",
                    "field_keeps/l=11",
                    "list/0/a=1",
                    "maybe/d=4",
                    "silent/k=10"
                ]
            );
            let mut kept = Table::new();
            assert_eq!(
                members(&keep_unknown_members_with(&typed, &json, false, &mut kept)),
                ALL
            );
            assert_eq!(
                table(&kept),
                ["field_keeps/l=11", "list/0/a=1", "maybe/d=4"]
            );
        }

        #[test]
        fn door_unknown_members_tells_each_member_s_disposition() {
            let (typed, json) = fixture();
            let told = |keeps_unknown: bool| -> Vec<String> {
                door_unknown_members(&typed, &json, keeps_unknown, &mut Table::new())
                    .iter()
                    .map(|(path, rule)| format!("{path} {rule:?}"))
                    .collect()
            };
            assert_eq!(
                told(false),
                [
                    "list/0/a Keep",
                    "map/k/b Report",
                    "boxed/c Report",
                    "maybe/d Keep",
                    "arms/1/g Report",
                    "rejects/h Reject",
                    "ignores/i Ignore",
                    "reports/j Report",
                    "silent/k Report",
                    "field_keeps/l Keep",
                    "z Report",
                ]
            );
            assert_eq!(
                told(true),
                [
                    "list/0/a Keep",
                    "map/k/b Report",
                    "boxed/c Report",
                    "maybe/d Keep",
                    "arms/1/g Keep",
                    "rejects/h Reject",
                    "ignores/i Ignore",
                    "reports/j Report",
                    "silent/k Keep",
                    "field_keeps/l Keep",
                    "z Keep",
                ]
            );
        }

        #[test]
        fn keep_members_adds_the_misread_scalars() {
            let (typed, json) = fixture();
            let mut kept = Table::new();
            let (found, misread) = keep_members(&typed, &json, &mut kept);
            assert_eq!(members(&found), ALL);
            let misread: Vec<String> = misread
                .iter()
                .map(|m| format!("{}: {:?}", m.path, m.kind))
                .collect();
            assert_eq!(
                misread,
                ["f: WrongType", "big: BeyondFloat", "one: WrongType"]
            );
            assert_eq!(
                table(&kept),
                ["field_keeps/l=11", "list/0/a=1", "maybe/d=4"]
            );
        }

        #[test]
        fn keep_members_and_range_members_adds_what_a_range_does_not_read() {
            let (typed, json) = fixture();
            let mut kept = Table::new();
            let (found, misread) = keep_members_and_range_members(&typed, &json, &mut kept);
            let mut all = ALL.to_vec();
            all.insert(4, "range/e=5");
            assert_eq!(members(&found), all);
            assert_eq!(misread.len(), 3);
            assert_eq!(
                table(&kept),
                ["field_keeps/l=11", "list/0/a=1", "maybe/d=4"]
            );
        }

        #[test]
        fn a_walk_finds_the_nulls_and_the_null_members() {
            let (typed, json) = fixture();
            for typed in [None, Some(Peek::new(&typed))] {
                for ranges in [false, true] {
                    let walked = Walk::new(Root::SHAPE, &json)
                        .typed(typed)
                        .ranges(ranges)
                        .run();
                    let nulls: Vec<String> = walked.nulls.iter().map(ToString::to_string).collect();
                    assert_eq!(nulls, ["one", "nothing"]);
                    let null_members: Vec<String> = walked
                        .null_members
                        .iter()
                        .map(ToString::to_string)
                        .collect();
                    assert_eq!(null_members, ["one", "nothing"]);
                    assert_eq!(walked.unknowns.len(), 11 + usize::from(ranges));
                    assert!(walked.misread.is_empty());
                    assert!(walked.scalars.is_empty());
                }
            }
        }

        #[test]
        fn the_scalar_walk_judges_the_scalars_alone() {
            let (_, json) = fixture();
            let walked = Walk::new(Root::SHAPE, &json)
                .scalars(ScalarRules::PLAIN)
                .run();
            let scalars: Vec<String> = walked
                .scalars
                .iter()
                .map(|call| {
                    format!(
                        "{} {:?} {:?} {} {:?}",
                        call.path, call.verdict, call.declared, call.arm, call.list
                    )
                })
                .collect();
            assert_eq!(
                scalars,
                [
                    "f Converted Some(Float { bits: 32 }) false None",
                    "big BeyondFloat Some(Float { bits: 32 }) false None",
                    "one Converted None false None",
                ]
            );
            assert_eq!(walked.unknowns.len(), 11);
            assert!(walked.misread.is_empty());
        }
    }

    /// The stack of the walk: each case in a child process (a stack that overflows ends the
    /// process), on a thread of a stack the case chooses.
    #[allow(
        clippy::disallowed_methods,
        clippy::disallowed_types,
        reason = "the tests run each case in a child process, on a thread of a stack they choose"
    )]
    mod stack {
        use std::process::{Command, Stdio};

        use facet::Facet;
        use indexmap::IndexMap;

        use super::super::{Peek, Table, Value, Walk, keep_unknown_members};
        use crate::json::MAX_JSON_DEPTH;
        use crate::json::stack::to_stay;

        /// An untagged enum that holds itself in a box and in a list: the costliest shape a
        /// level of the walk.
        #[derive(Facet)]
        #[facet(untagged)]
        #[repr(u8)]
        #[allow(dead_code, reason = "read, never looked at")]
        enum Either {
            Many(Vec<Either>),
            One {
                #[facet(default)]
                a: Option<Box<Either>>,
            },
        }

        /// A struct that holds itself in a box.
        #[derive(Facet)]
        struct Chain {
            #[facet(default)]
            a: Option<Box<Chain>>,
        }

        /// A struct that holds itself in a list.
        #[derive(Facet)]
        struct Tree {
            #[facet(default)]
            a: Option<Vec<Tree>>,
        }

        /// A struct that holds itself in a map of options.
        #[derive(Facet)]
        struct Named {
            #[facet(default)]
            a: Option<IndexMap<String, Option<Named>>>,
        }

        /// A struct that holds itself in a map of maps.
        #[derive(Facet)]
        struct Mapped {
            #[facet(default)]
            a: Option<IndexMap<String, IndexMap<String, Option<Mapped>>>>,
        }

        /// `open` `levels` times around `bottom`, closed.
        fn nest(levels: usize, open: fn(usize) -> &'static str, bottom: &str) -> String {
            let mut text = String::new();
            for level in 0..levels {
                text.push_str(open(level));
            }
            text.push_str(bottom);
            for level in (0..levels).rev() {
                text.push(if open(level).starts_with('[') {
                    ']'
                } else {
                    '}'
                });
            }
            text
        }

        fn objects(_: usize) -> &'static str {
            "{\"a\":"
        }

        fn mixed(level: usize) -> &'static str {
            if level.is_multiple_of(2) {
                "{\"a\":"
            } else {
                "["
            }
        }

        /// The bottom of a text of [`mixed`] levels: an object ends in `null`, a list in
        /// nothing.
        fn mixed_bottom(levels: usize) -> &'static str {
            if levels.is_multiple_of(2) { "" } else { "null" }
        }

        /// How a case walks: through the entry point, which gives the walk its stack, or
        /// the walk itself on the thread's own.
        #[derive(Clone, Copy)]
        enum How {
            Entry,
            OnThisStack,
        }

        /// `T` and a `Value` read from `text` through the door, and walked; whether the
        /// walk found no unknown member, which the texts have none of.
        fn walks<T: for<'f> Facet<'f>>(text: &str, how: How) -> bool {
            let typed: T = crate::json::from_str(text).expect("the typed value");
            let json: Value = crate::json::from_str(text).expect("the value");
            let found = match how {
                How::Entry => keep_unknown_members(&typed, &json, &mut Table::new()).len(),
                How::OnThisStack => Walk::new(T::SHAPE, &json)
                    .typed(Some(Peek::new(&typed)))
                    .ranges(true)
                    .misreads(true)
                    .run_on_this_stack()
                    .unknowns
                    .len(),
            };
            // What nests is dropped by the process ending, not by a call for each level.
            std::mem::forget((typed, json));
            found == 0
        }

        /// A shape by name, and its walk of a text `levels` deep.
        type Shaped = (&'static str, fn(usize, How) -> bool);

        const SHAPES: &[Shaped] = &[
            (
                "an untagged enum that holds itself in a box",
                |levels, how| walks::<Either>(&nest(levels, objects, "null"), how),
            ),
            (
                "an untagged enum that holds itself in a list",
                |levels, how| walks::<Either>(&nest(levels, mixed, mixed_bottom(levels)), how),
            ),
            ("a struct that holds itself in a box", |levels, how| {
                walks::<Chain>(&nest(levels, objects, "null"), how)
            }),
            ("a struct that holds itself in a list", |levels, how| {
                walks::<Tree>(&nest(levels, mixed, mixed_bottom(levels)), how)
            }),
            (
                "a struct that holds itself in a map of options",
                |levels, how| walks::<Named>(&nest(levels, objects, "null"), how),
            ),
            (
                "a struct that holds itself in a map of maps",
                |levels, how| {
                    let open = |level: usize| {
                        if level.is_multiple_of(3) {
                            "{\"a\":"
                        } else {
                            "{\"k\":"
                        }
                    };
                    // An empty map ends the text where a `null` would be a map's value.
                    let text = if levels % 3 == 2 {
                        nest(levels - 1, open, "{}")
                    } else {
                        nest(levels, open, "null")
                    };
                    walks::<Mapped>(&text, how)
                },
            ),
        ];

        /// The child of the tests below: `<shape> <levels> <stack in bytes> <0: through the
        /// entry point, 1: on the thread's own stack>`. Without the variable it is no child.
        #[test]
        #[ignore = "no test of its own: the child process of the tests below, which run it with --ignored"]
        fn child() {
            let Ok(case) = std::env::var("JSON_WALK_STACK") else {
                return;
            };
            let parts: Vec<usize> = case.split(' ').map(|part| part.parse().unwrap()).collect();
            let (shape, levels, stack) = (parts[0], parts[1], parts[2]);
            let how = if parts[3] == 0 {
                How::Entry
            } else {
                How::OnThisStack
            };
            let (_, run) = SHAPES[shape];
            let ended = std::thread::Builder::new()
                .stack_size(stack)
                .spawn(move || run(levels, how))
                .unwrap()
                .join()
                .unwrap();
            assert!(
                ended,
                "JSON_WALK_STACK: the walk found a member the text has not"
            );
        }

        /// How the child of one case ended: well, or with what it said.
        fn in_a_child(shape: usize, levels: usize, stack: usize, how: How) -> Result<(), String> {
            let output = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "json::walk::tests::stack::child",
                    "--ignored",
                    "--test-threads=1",
                    "--nocapture",
                ])
                .env(
                    "JSON_WALK_STACK",
                    format!("{shape} {levels} {stack} {}", how as usize),
                )
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .output()
                .unwrap();
            if output.status.success() {
                return Ok(());
            }
            let said = String::from_utf8_lossy(&output.stderr);
            let why = said
                .lines()
                .find(|line| line.contains("overflow") || line.contains("JSON_WALK_STACK"))
                .unwrap_or("no message");
            Err(format!("{} ({why})", output.status))
        }

        /// Every case of `stack_of` levels, through `how`; the ones that did not end well.
        fn failures(how: How, stack_of: fn(usize) -> usize) -> Vec<String> {
            let mut failures = Vec::new();
            for (shape, (name, _)) in SHAPES.iter().enumerate() {
                for levels in [1, 16, MAX_JSON_DEPTH] {
                    if let Err(ended) = in_a_child(shape, levels, stack_of(levels), how) {
                        failures.push(format!("{name}, {levels} levels: {ended}"));
                    }
                }
            }
            failures
        }

        /// The walk does not run on what its caller happens to have left: through an entry
        /// point it ends well on a thread of a quarter of a megabyte, at one level, at the
        /// depth of an ordinary file and at the limit, whatever the shape.
        #[test]
        fn the_walk_of_a_value_nested_to_the_limit_ends_well_on_a_small_stack() {
            let failures = failures(How::Entry, |_| 256 * 1024);
            assert!(failures.is_empty(), "{}", failures.join("\n"));
        }

        /// What a thread must have left for the walk to stay on it is more than twice what
        /// the walk takes, in the build at hand: each shape ends well on half of it, the
        /// walk run on the thread's own stack.
        #[test]
        fn a_walk_that_stays_on_the_thread_s_stack_has_twice_what_it_takes() {
            let failures = failures(How::OnThisStack, |levels| to_stay(levels) / 2);
            assert!(failures.is_empty(), "{}", failures.join("\n"));
        }
    }
}
