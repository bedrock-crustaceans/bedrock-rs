//! `FromJson` for the scalars, `json::Value` and the containers.

use std::collections::{BTreeMap, HashMap};
use std::hash::BuildHasher;

use indexmap::IndexMap;

use super::FromJson;
use crate::json::Declared;
use crate::json::Value;
use crate::json::kit::{Expected, Node, Read, Scalar};
use crate::versioning::{MemberFailure, NullHandling};

macro_rules! integers {
    ($($ty:ty => $bits:literal, $signed:literal;)+) => {
        $(
            /// An integer token of the member's range, as the reading's profile reads it
            /// ([`Read::scalar`]).
            impl FromJson for $ty {
                const EXPECTED: Option<Expected> = Some(Expected::Number);

                fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
                    let scalar = read.scalar(node, Declared::Int { bits: $bits, signed: $signed })?;
                    let value = scalar.integer().and_then(|value| <$ty>::try_from(value).ok());
                    if value.is_none() {
                        read.wrong_type(node);
                    }
                    value
                }
            }
        )+
    };
}

integers! {
    i8 => 8, true;
    i16 => 16, true;
    i32 => 32, true;
    i64 => 64, true;
    isize => 64, true;
    u8 => 8, false;
    u16 => 16, false;
    u32 => 32, false;
    u64 => 64, false;
    usize => 64, false;
}

/// A number, as the reading's profile reads it; beyond the width, the infinity it is (and an
/// information: the versioned load clamps the number and reads again).
impl FromJson for f32 {
    const EXPECTED: Option<Expected> = Some(Expected::Number);

    #[allow(clippy::cast_possible_truncation)] // the nearest f32 is the reading
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let scalar = read.scalar(node, Declared::Float { bits: 32 })?;
        scalar.real().map(|x| x as f32)
    }
}

/// A number, as the reading's profile reads it.
impl FromJson for f64 {
    const EXPECTED: Option<Expected> = Some(Expected::Number);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let scalar = read.scalar(node, Declared::Float { bits: 64 })?;
        scalar.real()
    }
}

/// `true` or `false`, as the reading's profile reads it.
impl FromJson for bool {
    const EXPECTED: Option<Expected> = Some(Expected::Bool);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if let Scalar::Bool(b) = read.scalar(node, Declared::Bool)? {
            return Some(b);
        }
        read.wrong_type(node);
        None
    }
}

/// A string, as the reading's profile reads it.
impl FromJson for String {
    const EXPECTED: Option<Expected> = Some(Expected::String);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if let Scalar::Text(text) = read.scalar(node, Declared::Text)? {
            return Some(text.into_owned());
        }
        read.wrong_type(node);
        None
    }
}

/// A string of one character.
impl FromJson for char {
    const EXPECTED: Option<Expected> = Some(Expected::String);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let text = String::from_json(node, read)?;
        let mut chars = text.chars();
        if let (Some(c), None) = (chars.next(), chars.next()) {
            return Some(c);
        }
        read.wrong_type(node);
        None
    }
}

/// Any value, as written; a `null` too.
impl FromJson for Value {
    const HOLDS_NULL: bool = true;

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if read.stopped() {
            return None;
        }
        let mut value = node.value().clone();
        if read.clamps_kept() {
            let mut clamped = crate::diagnostic::Diagnostics::default();
            crate::json::clamp_non_finite_warned(
                &mut value,
                node.text(),
                node.path(),
                &mut clamped,
            );
            for finding in &clamped {
                read.push(finding.clone());
            }
        }
        Some(value)
    }
}

/// The value `T` reads, or `None` for a `null` by the reading's null rule:
/// [`NullHandling::Rejected`] a wrong type (in [`Pass::Null`](crate::json::kit::Pass::Null) under [`Passes::AfterRead`](crate::json::kit::Passes::AfterRead)), which fails
/// it; [`NullHandling::Own`] and [`NullHandling::Absent`] `None` (under `Absent` a struct reads a
/// member written `null` as absent before it gets here). A type that holds a `null`
/// (`json::Value`) reads it as `None` under every rule.
impl<T: FromJson> FromJson for Option<T> {
    const EXPECTED: Option<Expected> = T::EXPECTED;

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if node.value().is_null() && read.reading().null != NullHandling::AsValue {
            return null::<T>(node, read).then_some(None);
        }
        T::from_json(node, read).map(Some)
    }
}

/// Whether an `Option<T>` reads a `null` as `None` (the `Option` impl's rule); `false` when the
/// reading refuses it, which is told.
pub(super) fn null<T: FromJson>(node: &Node<'_>, read: &mut Read<'_>) -> bool {
    null_of(T::EXPECTED, T::HOLDS_NULL, node, read)
}

/// [`null`] for a type that takes `expected` and holds a `null` when `holds_null`.
pub(super) fn null_of(
    expected: Option<Expected>,
    holds_null: bool,
    node: &Node<'_>,
    read: &mut Read<'_>,
) -> bool {
    if read.stopped() {
        return false;
    }
    if holds_null {
        return true;
    }
    match read.reading().null {
        NullHandling::Rejected => {
            if read.telling().defers() {
                read.null_error(node);
            } else {
                match expected {
                    Some(expected) => read.wrong_kind(node, expected),
                    None => read.wrong_type(node),
                }
            }
            false
        }
        _ => true,
    }
}

/// What a list does with an element that fails to read ([`read_list`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ElementFailure {
    /// As the read says: the element is left out under [`Failures::Every`] when the reading
    /// drops a failed element or its owner (`DropElement`, `DropOwner`); otherwise the list
    /// fails.
    ///
    /// [`Failures::Every`]: crate::json::kit::Failures::Every
    ByReading,
    /// The element is left out and the rest kept, whatever the reading (`schema::elements =
    /// "drop"`): a game reader that skips an element it cannot read.
    Drop,
    /// The list fails (every element is still read and told).
    FailList,
}

impl ElementFailure {
    /// Whether a failed element of the list at `node` is left out of it under `read`.
    fn drops(self, read: &Read<'_>, node: &Node<'_>) -> bool {
        match self {
            Self::Drop => true,
            Self::FailList => false,
            Self::ByReading => {
                (!read.telling().first_alone()
                    && matches!(
                        read.reading().on_member_failure,
                        MemberFailure::DropElement | MemberFailure::DropOwner
                    ))
                    || read.defaults_drop(node, true)
            }
        }
    }
}

/// The elements of the array `node`, each read by `element`, a failed one as the reading
/// says ([`ElementFailure::ByReading`]): what `Vec<T>`'s read is, for a list a family reads
/// with a function of its own (`schema::read_with = names`, where `names` is
/// `read_each(node, read, name)`). A value of another JSON type than an array is a wrong type.
///
/// ```
/// use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, codes};
/// use bedrock_addon_core::json::kit::{Node, Read};
/// use bedrock_addon_core::json::{Value, from_str_jsonc};
/// use bedrock_addon_core::schema::read_each;
/// use bedrock_addon_core::versioning::Reading;
///
/// /// A name with a namespace.
/// fn name(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
///     let text = read.string(node)?;
///     if !text.contains(':') {
///         read.refuse(node, (&codes::INVALID_VALUE).into());
///         return None;
///     }
///     Some(text.to_owned())
/// }
///
/// let json: Value = from_str_jsonc(r#"["a:b", "c", 1]"#).unwrap();
/// let mut found = Diagnostics::default();
/// let mut read = Read::new(Reading::DOCUMENT, &mut found);
/// assert_eq!(read_each(&Node::new(&json, JsonPath::of_keys(["names"])), &mut read, name), None);
/// let told: Vec<String> = found.iter().map(|f| format!("{} {}", f.location.json_path, f.code)).collect();
/// assert_eq!(told, ["names/1 invalid value", "names/2 wrong JSON type"]);
/// ```
pub fn read_each<T>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    element: impl FnMut(&Node<'_>, &mut Read<'_>) -> Option<T>,
) -> Option<Vec<T>> {
    read_list(node, read, ElementFailure::ByReading, element)
}

/// [`read_each`] with a failed element doing `failure`. A failed element is told and the read
/// goes on to the next one; one left out leaves the document's table of kept unknown members
/// renumbered to the elements kept, and fails nothing (the read is as failed as it was before
/// the element).
pub fn read_list<T>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    failure: ElementFailure,
    mut element: impl FnMut(&Node<'_>, &mut Read<'_>) -> Option<T>,
) -> Option<Vec<T>> {
    if read.stopped() {
        return None;
    }
    let elements = read.array(node)?;
    let mut out = Vec::with_capacity(elements.len());
    // The indices of the elements kept, once one is left out.
    let mut kept: Option<Vec<usize>> = None;
    let mut failed = false;
    let drops = failure.drops(read, node);
    for (index, at) in elements.iter().enumerate() {
        if read.stopped() {
            return None;
        }
        let was_failed = read.failed();
        let mark = read.mark();
        match element(&at, read) {
            Some(value) => {
                out.push(value);
                if let Some(kept) = &mut kept {
                    kept.push(index);
                }
            }
            None if drops || read.left_out_since(mark) => {
                kept.get_or_insert_with(|| (0..index).collect());
                read.set_failed(was_failed);
            }
            None => failed = true,
        }
    }
    if failed {
        return None;
    }
    if let Some(kept) = kept {
        read.findings_mut().left_out(node.path(), &kept);
    }
    Some(out)
}

/// The elements of `node` as jsoncpp iterates a value (`schema::iterated`): an array's elements,
/// an object's member values in the bytewise order of their names (its objects are a
/// `std::map`), nothing for any other value. A value that is no array is told as a wrong type
/// (graded as a failed member is) and read so: the member is there, its list the values
/// iterated. Each element is read by `element`, a failed one doing `failure`.
pub fn read_iterated<T>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    failure: ElementFailure,
    mut element: impl FnMut(&Node<'_>, &mut Read<'_>) -> Option<T>,
) -> Option<Vec<T>> {
    if read.stopped() {
        return None;
    }
    if node.value().is_array() {
        return read_list(node, read, failure, element);
    }
    read.wrong_kind(node, Expected::Array);
    if read.stopped() {
        return None;
    }
    if !node.value().is_object() {
        return Some(Vec::new());
    }
    let members = read.object(node)?;
    let mut values: Vec<(&str, Node<'_>)> = members.iter().collect();
    values.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let drops = failure.drops(read, node);
    let mut out = Vec::with_capacity(values.len());
    let mut failed = false;
    for (_, at) in values {
        if read.stopped() {
            return None;
        }
        let was_failed = read.failed();
        match element(&at, read) {
            Some(value) => out.push(value),
            None if drops => read.set_failed(was_failed),
            None => failed = true,
        }
    }
    (!failed).then_some(out)
}

/// An array, each element read as `T` ([`read_each`]): a failed element fails the list (every
/// element is still read and told), or under `DropElement` and `DropOwner` is left out.
impl<T: FromJson> FromJson for Vec<T> {
    const EXPECTED: Option<Expected> = Some(Expected::Array);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        read_each(node, read, T::from_json)
    }
}

/// An array of exactly `N` elements, each read as `T`; another length is a wrong type. An
/// unknown member inside an element is told and not kept, as the door's walk cannot reach
/// into a fixed array to keep it.
impl<T: FromJson, const N: usize> FromJson for [T; N] {
    const EXPECTED: Option<Expected> = Some(Expected::Array);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if read.stopped() {
            return None;
        }
        let elements = read.array(node)?;
        if elements.len() != N {
            read.wrong_type(node);
            return None;
        }
        let mut out = Vec::with_capacity(N);
        let mut failed = false;
        for element in elements.iter() {
            match read.unkeepable(|read| T::from_json(&element, read)) {
                Some(value) => out.push(value),
                None => failed = true,
            }
        }
        if failed {
            return None;
        }
        out.try_into().ok()
    }
}

/// The boxed value (a recursive type holds itself in a `Box`). An unknown member inside is
/// told and not kept, as the door's walk cannot reach it.
impl<T: FromJson> FromJson for Box<T> {
    const EXPECTED: Option<Expected> = T::EXPECTED;
    const HOLDS_NULL: bool = T::HOLDS_NULL;

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        read.unkeepable(|read| T::from_json(node, read))
            .map(Box::new)
    }
}

/// A key of a map read from JSON, from the member's name as written: `String` and `Id<K>`
/// here; a family's own key type (a number, a version, a name with a rule) implements it
/// beside the type. A name that is no key is a wrong type at the member's value.
///
/// ```
/// use bedrock_addon_core::schema::MapKey;
///
/// /// A key that is a whole number.
/// #[derive(Debug, PartialEq, Eq, Hash)]
/// struct Slot(u8);
///
/// impl MapKey for Slot {
///     fn from_key(name: &str) -> Option<Self> {
///         name.parse().ok().map(Slot)
///     }
/// }
///
/// assert_eq!(Slot::from_key("3"), Some(Slot(3)));
/// assert_eq!(Slot::from_key("x"), None);
/// ```
pub trait MapKey: Sized {
    /// The key a member named `name` is, or `None` (a wrong type at the member's value).
    fn from_key(name: &str) -> Option<Self>;
}

impl MapKey for String {
    fn from_key(name: &str) -> Option<Self> {
        Some(name.to_owned())
    }
}

impl<K: crate::id::Kind> MapKey for crate::id::Id<K> {
    fn from_key(name: &str) -> Option<Self> {
        Some(Self::parse(name))
    }
}

/// The entries of an object, each value read as `V` at its member's path, in file order. A
/// failed value fails the map (every value is still read and told); a value read as absent (a
/// scalar its profile refuses, under [`Passes::AfterRead`](crate::json::kit::Passes::AfterRead) with `DefaultAndTell`: the door's pre-pass
/// blanks it) is left out and the rest kept. An unknown member inside a value is told and not
/// kept, as the door's walk cannot reach it, unless the field that holds the map says
/// `schema::unknown = "keep"`: its values' unknown members are then kept in the document's table
/// at the value's path, `map/<key>`, and written back there.
fn map_entries<K: MapKey, V>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    mut entry: impl FnMut(&Node<'_>, &mut Read<'_>) -> Option<V>,
    mut put: impl FnMut(K, V),
) -> Option<()> {
    if read.stopped() {
        return None;
    }
    let members = read.object(node)?;
    let keeps = read.field_unknown() == Some(crate::schema::UnknownRule::Keep);
    let drops = read.defaults_drop(node, false);
    let mut failed = false;
    for (key, member) in read.ordered(&members) {
        if read.stopped() {
            return None;
        }
        let Some(key) = K::from_key(key) else {
            read.wrong_type(&member);
            failed = true;
            continue;
        };
        let mark = read.mark();
        let value = if keeps {
            read.in_map_value(true, |read| entry(&member, read))
        } else {
            read.unkeepable(|read| read.in_map_value(true, |read| entry(&member, read)))
        };
        match value {
            Some(value) => put(key, value),
            None if read.take_defaulted() || drops || read.left_out_since(mark) => {}
            None => failed = true,
        }
    }
    (!failed).then_some(())
}

/// The entries of the object `node`, each value read by `value` at its member's path, keyed
/// by its member's name ([`MapKey`]): what a map's read is, for a map a family reads with a
/// function of its own. A failed value fails the map (every value is still read and told); a
/// value of another JSON type than an object is a wrong type.
///
/// ```
/// use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
/// use bedrock_addon_core::json::kit::{Node, Read};
/// use bedrock_addon_core::json::{Value, from_str_jsonc};
/// use bedrock_addon_core::schema::{FromJson, read_entries};
/// use bedrock_addon_core::versioning::Reading;
///
/// let json: Value = from_str_jsonc(r#"{"a": 1, "b": 2}"#).unwrap();
/// let mut found = Diagnostics::default();
/// let mut read = Read::new(Reading::DOCUMENT, &mut found);
/// let doubled = read_entries::<String, u8>(&Node::new(&json, JsonPath::new()), &mut read, |node, read| {
///     u8::from_json(node, read).map(|n| n * 2)
/// });
/// assert_eq!(doubled.unwrap()["b"], 4);
/// ```
pub fn read_entries<K: MapKey + std::hash::Hash + Eq, V>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    value: impl FnMut(&Node<'_>, &mut Read<'_>) -> Option<V>,
) -> Option<IndexMap<K, V>> {
    let mut out = IndexMap::new();
    map_entries(node, read, value, |key, value| {
        out.insert(key, value);
    })?;
    Some(out)
}

/// An object, each member a key and its value read as `V`.
impl<K: MapKey + std::hash::Hash + Eq, V: FromJson, S: BuildHasher + Default> FromJson
    for IndexMap<K, V, S>
{
    const EXPECTED: Option<Expected> = Some(Expected::Object);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let mut out = IndexMap::with_hasher(S::default());
        map_entries(node, read, V::from_json, |key, value| {
            out.insert(key, value);
        })?;
        Some(out)
    }
}

/// An object, each member a key and its value read as `V`.
impl<K: MapKey + std::hash::Hash + Eq, V: FromJson, S: BuildHasher + Default> FromJson
    for HashMap<K, V, S>
{
    const EXPECTED: Option<Expected> = Some(Expected::Object);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let mut out = HashMap::with_hasher(S::default());
        map_entries(node, read, V::from_json, |key, value| {
            out.insert(key, value);
        })?;
        Some(out)
    }
}

/// An object, each member a key and its value read as `V`.
impl<K: MapKey + Ord, V: FromJson> FromJson for BTreeMap<K, V> {
    const EXPECTED: Option<Expected> = Some(Expected::Object);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let mut out = BTreeMap::new();
        map_entries(node, read, V::from_json, |key, value| {
            out.insert(key, value);
        })?;
        Some(out)
    }
}

macro_rules! tuples {
    ($($len:literal => ($($name:ident),+);)+) => {
        $(
            /// An array of exactly as many elements as the tuple, each read as its type; another
            /// length is a wrong type.
            impl<$($name: FromJson),+> FromJson for ($($name,)+) {
                const EXPECTED: Option<Expected> = Some(Expected::Array);

                #[allow(non_snake_case)]
                fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
                    if read.stopped() {
                        return None;
                    }
                    let elements = read.array(node)?;
                    if elements.len() != $len {
                        read.wrong_type(node);
                        return None;
                    }
                    let mut at = 0;
                    $(
                        let $name = elements.element(at).and_then(|element| $name::from_json(&element, read));
                        at += 1;
                    )+
                    let _ = at;
                    Some(($($name?,)+))
                }
            }
        )+
    };
}

tuples! {
    1 => (A);
    2 => (A, B);
    3 => (A, B, C);
    4 => (A, B, C, D);
}
