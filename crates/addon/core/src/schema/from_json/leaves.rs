//! `FromJson` for core's leaf types, each telling what the typed door tells for the same input:
//! identifiers, format versions, the Molang values, ranges, filters and event triggers, and the
//! kept-member types. A leaf read through its facet proxy is refused there as a wrong type at
//! its member, whatever the proxy's reason; these say the same.

use std::sync::Arc;

use super::FromJson;
use crate::diagnostic::Code;
use crate::diagnostic::codes as core_codes;
use crate::document::{Extra, UnknownMembers};
use crate::json::Value;
use crate::json::kit::{Expected, Node, Read, Written};
use crate::molang::{Molang, MolangList, MolangSource};
use crate::version::FormatVersion;
use crate::versioning::RangeMembers;

/// The value `node` holds converted by `convert`, or the wrong type its member is told when the
/// conversion refuses it: the read of a leaf behind `#[facet(proxy = json::Value)]` and a
/// `TryFrom<json::Value>`, as the door reads it. The value is cloned for the conversion; a
/// leaf that converts from a reference reads by [`from_value_ref`].
///
/// ```
/// use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
/// use bedrock_addon_core::json::kit::{Node, Read};
/// use bedrock_addon_core::json::{Value, from_str_jsonc};
/// use bedrock_addon_core::schema::from_value;
/// use bedrock_addon_core::versioning::Reading;
///
/// let json: Value = from_str_jsonc(r#""north""#).unwrap();
/// let mut found = Diagnostics::default();
/// let mut read = Read::new(Reading::DOCUMENT, &mut found);
/// let node = Node::new(&json, JsonPath::of_keys(["facing"]));
/// let even = |value: Value| value.as_number().and_then(|n| n.to_u64()).filter(|n| n % 2 == 0).ok_or(());
/// assert_eq!(from_value(&node, &mut read, even), None);
/// assert_eq!(found.iter().next().unwrap().code.to_string(), "wrong JSON type");
/// ```
pub fn from_value<T, E>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    convert: impl FnOnce(Value) -> Result<T, E>,
) -> Option<T> {
    from_value_ref(node, read, |value| convert(value.clone()))
}

/// [`from_value`] by a conversion from a reference: nothing is cloned to read.
pub fn from_value_ref<T, E>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    convert: impl FnOnce(&Value) -> Result<T, E>,
) -> Option<T> {
    if read.stopped() {
        return None;
    }
    let value = convert(node.value()).ok();
    if value.is_none() {
        read.wrong_type(node);
    }
    value
}

/// `FromJson` for each type named, by its `TryFrom<json::Value>` ([`from_value`]), or with
/// `ref` by its `TryFrom<&json::Value>` ([`from_value_ref`]): the read of a leaf behind
/// `#[facet(opaque, proxy = json::Value)]`, a refused value a wrong type at its member, as the
/// door reads it. A generic type is named as `impl<T: Bound> Type<T>`.
///
/// A bridge, like [`from_json_via_facet!`](crate::from_json_via_facet): the leaf's own
/// conversion reads the whole value by the shape's rule, so the slice's `Reading` does not
/// reach inside it (its scalars, its `null`s), a refusal names no member inside it and a family
/// cannot word it. Each use carries a `// bridge:` comment; a leaf leaves it for a `FromJson`
/// over the kit when its family's pins allow the Reading in (`rg 'from_json_via_value'`).
///
/// ```
/// use bedrock_addon_core::json::Value;
///
/// pub struct Even(u64);
///
/// impl TryFrom<&Value> for Even {
///     type Error = ();
///     fn try_from(value: &Value) -> Result<Self, ()> {
///         value.as_number().and_then(|n| n.to_u64()).filter(|n| n % 2 == 0).map(Even).ok_or(())
///     }
/// }
///
/// // bridge: the example's leaf reads by its conversion
/// bedrock_addon_core::from_json_via_value!(ref Even);
/// ```
#[macro_export]
macro_rules! from_json_via_value {
    (ref $(impl<$($param:ident $(: $bound:path)?),+> $ty:ty),+ $(,)?) => {
        $(
            impl<$($param $(: $bound)?),+> $crate::schema::FromJson for $ty {
                fn from_json(
                    node: &$crate::json::kit::Node<'_>,
                    read: &mut $crate::json::kit::Read<'_>,
                ) -> ::core::option::Option<Self> {
                    $crate::schema::from_value_ref(node, read, |value| <Self as ::core::convert::TryFrom<&$crate::json::Value>>::try_from(value))
                }
            }
        )+
    };
    (ref $($ty:ty),+ $(,)?) => {
        $(
            impl $crate::schema::FromJson for $ty {
                fn from_json(
                    node: &$crate::json::kit::Node<'_>,
                    read: &mut $crate::json::kit::Read<'_>,
                ) -> ::core::option::Option<Self> {
                    $crate::schema::from_value_ref(node, read, |value| <Self as ::core::convert::TryFrom<&$crate::json::Value>>::try_from(value))
                }
            }
        )+
    };
    ($(impl<$($param:ident $(: $bound:path)?),+> $ty:ty),+ $(,)?) => {
        $(
            impl<$($param $(: $bound)?),+> $crate::schema::FromJson for $ty {
                fn from_json(
                    node: &$crate::json::kit::Node<'_>,
                    read: &mut $crate::json::kit::Read<'_>,
                ) -> ::core::option::Option<Self> {
                    $crate::schema::from_value(node, read, <Self as ::core::convert::TryFrom<$crate::json::Value>>::try_from)
                }
            }
        )+
    };
    ($($ty:ty),+ $(,)?) => {
        $(
            impl $crate::schema::FromJson for $ty {
                fn from_json(
                    node: &$crate::json::kit::Node<'_>,
                    read: &mut $crate::json::kit::Read<'_>,
                ) -> ::core::option::Option<Self> {
                    $crate::schema::from_value(node, read, <Self as ::core::convert::TryFrom<$crate::json::Value>>::try_from)
                }
            }
        )+
    };
}

// bridge: the shared leaves read by their own conversion (the game's reader of the shape)
crate::from_json_via_value!(
    ref crate::shared::Filter,
    crate::shared::FilterTest,
    crate::shared::FilterGroup,
    crate::shared::FilterSubject,
    crate::shared::FilterScalar,
    crate::shared::EventTrigger,
);
/// A number, a list of two or an object of `min`/`max` (or `range_min`/`range_max`), its ends
/// read by the range's own integer and float rules; another value is a wrong type. Under a
/// reading that says so ([`Reading::range_members`](crate::versioning::Reading::range_members))
/// each member of the object form the range does not read, and each element of the list form
/// past its two ends, is told: told, not kept (a range has no `extra`).
impl<T: crate::shared::RangeEnd> FromJson for crate::shared::Range<T> {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        // bridge: a range reads its ends by its own integer and float rules
        let range = from_value_ref(node, read, |value| Self::try_from(value))?;
        if read.reading().range_members != RangeMembers::Silent {
            if let Some(object) = node.value().as_object() {
                let unread = crate::shared::unread_range_members(object);
                if !unread.is_empty()
                    && let Some(members) = read.object(node)
                {
                    for (key, member) in members.iter() {
                        if unread.contains(&key) {
                            read.unread_range_member(&member);
                        }
                    }
                }
            } else if node.value().is_array()
                && let Some(elements) = read.array(node)
            {
                for element in elements.iter().skip(2) {
                    read.unread_range_member(&element);
                }
            }
        }
        Some(range)
    }
}

/// `true` or `false` and nothing else, whatever the reading's scalars say: a number, a string
/// is a wrong type.
impl FromJson for crate::json::StrictBool {
    const EXPECTED: Option<Expected> = Some(Expected::Bool);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        read.bool(node).map(Self)
    }
}

/// A string, kept as written; any other value is a wrong type.
impl<K: crate::id::Kind> FromJson for crate::id::Id<K> {
    const EXPECTED: Option<Expected> = Some(Expected::String);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if read.stopped() {
            return None;
        }
        let Some(text) = node.value().as_string() else {
            read.wrong_type(node);
            return None;
        };
        Some(Self::parse(Arc::<str>::from(text.as_str())))
    }
}

/// A version string (`"1.21.60"`, missing numbers 0, a pre-release tag dropped), as
/// `TryFrom<String>` reads it; a number is read by its text as facet's string proxy reads it
/// (`1` is 1.0.0, `0.5` is 0.5.0), anything else, and a text that names no version, a wrong
/// type.
impl FromJson for FormatVersion {
    const EXPECTED: Option<Expected> = Some(Expected::String);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if read.stopped() {
            return None;
        }
        let value = node.value();
        let text = if let Some(text) = value.as_string() {
            Some(text.as_str().to_owned())
        } else {
            value.as_number().map(|number| {
                if number.is_float() {
                    format!("{}", number.to_f64_lossy())
                } else if let Some(u) = number.to_u64() {
                    u.to_string()
                } else if let Some(i) = number.to_i64() {
                    i.to_string()
                } else {
                    format!("{}", number.to_f64_lossy())
                }
            })
        };
        let version = text.and_then(|text| Self::try_from(text).ok());
        if version.is_none() {
            read.wrong_type(node);
        }
        version
    }
}

/// A number as the `f32` it is (a wrong type where that is not finite), a bool, or an
/// expression: a string, or the object form `{"expression", "version"}`. `null` and an array
/// are a wrong type.
impl FromJson for Molang {
    #[allow(clippy::cast_possible_truncation)] // the nearest f32 is the reading
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if read.stopped() {
            return None;
        }
        let value = node.value();
        if let Some(number) = value.as_number() {
            let x = number.to_f64_lossy() as f32;
            if x.is_finite() {
                return Some(Molang::Const(x));
            }
        } else if let Some(b) = value.as_bool() {
            return Some(Molang::Bool(b));
        } else if value.is_string() || value.is_object() {
            return MolangSource::from_json(node, read).map(Molang::Expr);
        }
        read.wrong_type(node);
        None
    }
}

/// An expression: a string, or the object form `{"expression": string, "version": integer of
/// `i16`}` (another member ignored); a missing member is a missing required field at the
/// object, a member of another type a wrong type. Anything else is a wrong type.
impl FromJson for MolangSource {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if read.stopped() {
            return None;
        }
        let value = node.value();
        if let Some(text) = value.as_string() {
            return Some(MolangSource::string_without_context(text.as_str()));
        }
        let Some(object) = value.as_object() else {
            read.wrong_type(node);
            return None;
        };
        let (Some(expression), Some(version)) = (object.get("expression"), object.get("version"))
        else {
            read.refuse(node, Code::from(&core_codes::MISSING_REQUIRED_FIELD));
            return None;
        };
        let version = version
            .as_number()
            .filter(|number| !number.is_float())
            .and_then(facet_value::VNumber::to_i64)
            .and_then(|version| i16::try_from(version).ok());
        if let (Some(expression), Some(version)) = (expression.as_string(), version) {
            return Some(MolangSource::object(expression.as_str(), version));
        }
        read.wrong_type(node);
        None
    }
}

/// One value (a number as its finite `f32`, a bool, a string) or an array of them, at any
/// depth; an object, a `null` element and a number not finite as an `f32` are a wrong type at
/// the member, as the door's list proxy refuses the whole.
impl FromJson for MolangList {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        from_value_ref(node, read, list)
    }
}

/// The list a value is ([`MolangList`]'s `FromJson`).
#[allow(clippy::cast_possible_truncation)] // the nearest f32 is the reading
fn list(value: &Value) -> Result<MolangList, ()> {
    if let Some(number) = value.as_number() {
        let x = number.to_f64_lossy() as f32;
        return x
            .is_finite()
            .then_some(MolangList::One(Molang::Const(x)))
            .ok_or(());
    }
    if let Some(b) = value.as_bool() {
        return Ok(MolangList::One(Molang::Bool(b)));
    }
    if let Some(text) = value.as_string() {
        return Ok(MolangList::One(Molang::Expr(
            MolangSource::string_without_context(text.as_str()),
        )));
    }
    match value.as_array() {
        Some(array) => array
            .iter()
            .map(list)
            .collect::<Result<_, _>>()
            .map(MolangList::Many),
        None => Err(()),
    }
}

/// The mark of a struct that keeps its unknown members holds nothing: always read.
impl FromJson for Extra {
    fn from_json(_: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        (!read.stopped()).then(Extra::default)
    }
}

// The members a struct keeps beside its fields hold no bound and no Molang.
crate::impl_schema_leaf!(UnknownMembers);
crate::impl_no_molang!(UnknownMembers);

/// An object's members, as written, in file order.
impl FromJson for UnknownMembers {
    const EXPECTED: Option<Expected> = Some(Expected::Object);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if read.stopped() {
            return None;
        }
        let members = read.object(node)?;
        Some(
            members
                .iter()
                .map(|(key, member)| (key.to_owned(), member.value().clone()))
                .collect(),
        )
    }
}

/// The value as `T` reads it, kept as the author wrote it (left out of equality).
impl<T: FromJson> FromJson for Written<T> {
    const EXPECTED: Option<Expected> = T::EXPECTED;

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        T::from_json(node, read).map(|value| Written(Some(value)))
    }
}
