//! `FromJson` for core's leaf types, each telling what facet-json's typed read told for the
//! same input: identifiers, format versions, the Molang values, ranges, filters and event
//! triggers, and the kept-member types. A leaf read through its facet proxy is refused there as
//! a wrong type at its member, whatever the proxy's reason; these say the same.

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

/// The read of a member that the struct's own rule reads (`schema::after_read`): the member is
/// one of the struct's, so it is no unknown one, and the derived read takes the default of its
/// field and tells nothing.
///
/// ```
/// use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
/// use bedrock_addon_core::json::kit::{Node, Read};
/// use bedrock_addon_core::json::{Value, from_str_jsonc};
/// use bedrock_addon_core::schema::read_later;
/// use bedrock_addon_core::versioning::Reading;
///
/// let json: Value = from_str_jsonc("[1, 2]").unwrap();
/// let mut found = Diagnostics::default();
/// let mut read = Read::new(Reading::DOCUMENT, &mut found);
/// let node = Node::new(&json, JsonPath::of_keys(["sounds"]));
/// assert_eq!(read_later::<Vec<u8>>(&node, &mut read), Some(Vec::new()));
/// assert!(found.iter().next().is_none());
/// ```
pub fn read_later<T: Default>(_: &Node<'_>, _: &mut Read<'_>) -> Option<T> {
    Some(T::default())
}

/// The value `node` holds converted by `convert`, or the wrong type its member is told when the
/// conversion refuses it: the read of a leaf behind `#[facet(proxy = json::Value)]` and a
/// `TryFrom<json::Value>`, as facet-json read it. The value is cloned for the conversion; a
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

/// A number, a list of two or an object of `min`/`max` (or `range_min`/`range_max`), its ends
/// read by the range's own integer and float rules; another value is a wrong type. Under a
/// reading that says so ([`Reading::range_members`](crate::versioning::Reading::range_members))
/// each member of the object form the range does not read, and each element of the list form
/// past its two ends, is told: told, not kept (a range has no `extra`).
impl<T: crate::shared::RangeEnd> FromJson for crate::shared::Range<T> {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        // The ends are read as the game's range reader reads them (an integer end is an
        // integer token, a float end any finite number), not by the slice's scalar rule, and
        // a refusal is one wrong type at the range, so the conversion is the read.
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
/// A family's own read of Molang, in place of core's [`Molang`] leaf for every `Molang` and
/// `[Molang; N]` member a read meets ([`Read::molang_leaf`]): a reader that takes Molang its own
/// way (particle: the legacy parser takes a bool and a string at the pack's version and keeps a
/// value it cannot read as the member's default, `Read::member_molang`; the 1.20.80 reader
/// takes the versioned object form).
#[derive(Clone, Copy, Debug)]
pub struct MolangLeaf {
    /// Reads one Molang value; tells what it refuses and returns `None`.
    pub one: fn(&Node<'_>, &mut Read<'_>) -> Option<Molang>,
    /// Reads an array of `len` Molang values (a `[Molang; len]` member): exactly `len` of them,
    /// or `None` with what it refuses told.
    pub array: fn(&Node<'_>, &mut Read<'_>, usize) -> Option<Vec<Molang>>,
}

impl FromJson for Molang {
    #[allow(clippy::cast_possible_truncation)] // the nearest f32 is the reading
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if read.stopped() {
            return None;
        }
        if let Some(leaf) = read.reads_molang_by() {
            return (leaf.one)(node, read);
        }
        if read.telling().objects == crate::json::kit::Objects::Tree {
            return expression_node(node, read);
        }
        let value = node.value();
        if let Some(number) = value.as_number() {
            let x = number.to_f64_lossy() as f32;
            if x.is_finite() {
                return Some(Molang::Const(x));
            }
            if read.clamps_molang() {
                // The largest finite float of the sign, told at the token.
                let finding = crate::molang::beyond_f32_warning(node.path().clone(), node.span());
                read.push(finding);
                return Some(Molang::Const(f32::MAX.copysign(x)));
            }
        } else if let Some(b) = value.as_bool() {
            return Some(Molang::Bool(b));
        } else if value.is_string() || value.is_object() {
            return MolangSource::from_json(node, read).map(Molang::Expr);
        }
        read.wrong_type(node);
        None
    }

    /// By the read's Molang leaf where it has one: the leaf's array of `N`.
    fn array_from_json<const N: usize>(node: &Node<'_>, read: &mut Read<'_>) -> Option<[Self; N]> {
        let Some(leaf) = read.reads_molang_by() else {
            return super::impls::array_by_element(node, read);
        };
        if read.stopped() {
            return None;
        }
        (leaf.array)(node, read, N).and_then(|values| values.try_into().ok())
    }
}

/// The game's schema node tree's `ExpressionNode` (a tree's read of a [`Molang`]): a number (one
/// beyond `f32` the largest finite float of its sign, with an `INVALID_VALUE` information at it),
/// a bool or a string, whose context version the Molang passes set. Anything else is a wrong type
/// accepting `"ExpressionNode"`.
fn expression_node(node: &Node<'_>, read: &mut Read<'_>) -> Option<Molang> {
    use crate::molang::{JsonScalar, RawVersion, ReadField, ReaderKind};
    let value = node.value();
    let scalar = if let Some(number) = value.as_number() {
        JsonScalar::Number(number.to_f64_lossy())
    } else if let Some(b) = value.as_bool() {
        JsonScalar::Bool(b)
    } else if let Some(text) = value.as_string() {
        JsonScalar::String(Arc::from(text.as_str()))
    } else {
        read.wrong_accepting(node, "ExpressionNode");
        return None;
    };
    let Ok((ReadField::Value(value), clamped)) =
        crate::molang::read_finite(scalar, ReaderKind::SchemaValidated, RawVersion(-1))
    else {
        read.wrong_accepting(node, "ExpressionNode");
        return None;
    };
    if clamped {
        read.push(crate::molang::beyond_f32_warning(
            node.path().clone(),
            node.span(),
        ));
    }
    Some(value)
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
/// the member, as the old list proxy refused the whole.
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
