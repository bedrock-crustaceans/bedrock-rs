//! `FromJson` for the scalars, `json::Value` and the containers.

use std::collections::{BTreeMap, HashMap};
use std::hash::BuildHasher;

use indexmap::IndexMap;

use super::FromJson;
use crate::json::Declared;
use crate::json::Value;
use crate::json::kit::{Expected, Node, Order, Pass, Read, Scalar};
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
        (!read.stopped()).then(|| node.value().clone())
    }
}

/// The value `T` reads, or `None` for a `null` by the reading's null rule:
/// [`NullHandling::Rejected`] a wrong type (in [`Pass::Null`] under [`Order::Door`]), which fails
/// it; [`NullHandling::Own`] and [`NullHandling::Absent`] `None` (under `Absent` a struct reads a
/// member written `null` as absent before it gets here). A type that holds a `null`
/// (`json::Value`) reads it as `None` under every rule.
impl<T: FromJson> FromJson for Option<T> {
    const EXPECTED: Option<Expected> = T::EXPECTED;

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if node.value().is_null() {
            return null::<T>(node, read).then_some(None);
        }
        T::from_json(node, read).map(Some)
    }
}

/// Whether an `Option<T>` reads a `null` as `None` (the `Option` impl's rule); `false` when the
/// reading refuses it, which is told.
pub(super) fn null<T: FromJson>(node: &Node<'_>, read: &mut Read<'_>) -> bool {
    if read.stopped() {
        return false;
    }
    if T::HOLDS_NULL {
        return true;
    }
    match read.reading().null {
        NullHandling::Rejected => {
            match read.telling() {
                Order::Door => {
                    let mut finding =
                        crate::diagnostic::Diagnostic::of(&crate::diagnostic::codes::INVALID_TYPE)
                            .at(node.path().clone());
                    finding.location.span = node.span();
                    read.defer(Pass::Null, finding);
                }
                _ => match T::EXPECTED {
                    Some(expected) => read.wrong_kind(node, expected),
                    None => read.wrong_type(node),
                },
            }
            false
        }
        _ => true,
    }
}

/// Whether a failed element is dropped from its list (and the rest kept) rather than failing
/// the list: as read (not in [`Order::Door`]), when the reading drops a failed element or its
/// owner.
fn drops_elements(read: &Read<'_>) -> bool {
    read.telling() != Order::Door
        && matches!(
            read.reading().on_member_failure,
            MemberFailure::DropElement | MemberFailure::DropOwner
        )
}

/// An array, each element read as `T`. A failed element fails the list (every element is
/// still read and told), or under `DropElement` and `DropOwner` is left out, the document's
/// table of kept unknown members renumbered to the elements kept.
impl<T: FromJson> FromJson for Vec<T> {
    const EXPECTED: Option<Expected> = Some(Expected::Array);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if read.stopped() {
            return None;
        }
        let elements = read.array(node)?;
        let mut out = Vec::with_capacity(elements.len());
        let mut kept = Vec::with_capacity(elements.len());
        let mut failed = false;
        let drops = drops_elements(read);
        for (index, element) in elements.iter().enumerate() {
            if read.stopped() {
                return None;
            }
            match T::from_json(&element, read) {
                Some(value) => {
                    out.push(value);
                    kept.push(index);
                }
                None if drops => {}
                None => failed = true,
            }
        }
        if failed {
            return None;
        }
        if kept.len() < elements.len() {
            read.findings_mut().left_out(node.path(), &kept);
        }
        Some(out)
    }
}

/// An array of exactly `N` elements, each read as `T`; another length is a wrong type.
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
            match T::from_json(&element, read) {
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

/// A key of a map read from JSON: the member name, as written.
// hidden: the map impls' bound; a key type of a family implements it beside the type
#[doc(hidden)]
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
/// failed value fails the map (every value is still read and told). An unknown member inside a
/// value is told and not kept, as the door's walk cannot reach it.
fn map_entries<K: MapKey, V: FromJson>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    mut put: impl FnMut(K, V),
) -> Option<()> {
    if read.stopped() {
        return None;
    }
    let members = read.object(node)?;
    let mut failed = false;
    for (key, member) in members.iter() {
        if read.stopped() {
            return None;
        }
        let Some(key) = K::from_key(key) else {
            read.wrong_type(&member);
            failed = true;
            continue;
        };
        match read.unkeepable(|read| V::from_json(&member, read)) {
            Some(value) => put(key, value),
            None => failed = true,
        }
    }
    (!failed).then_some(())
}

/// An object, each member a key and its value read as `V`.
impl<K: MapKey + std::hash::Hash + Eq, V: FromJson, S: BuildHasher + Default> FromJson
    for IndexMap<K, V, S>
{
    const EXPECTED: Option<Expected> = Some(Expected::Object);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let mut out = IndexMap::with_hasher(S::default());
        map_entries(node, read, |key, value| {
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
        map_entries(node, read, |key, value| {
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
        map_entries(node, read, |key, value| {
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
