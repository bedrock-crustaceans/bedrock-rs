//! Reading a catalogue payload: the document's types read member by member by core's reader
//! (`versioning::read_payload_with`), every finding with the span of its value. Vanilla's cereal
//! reader fails the file on a wrong type, a missing required member, an unknown category or a
//! constraint, and tells every unknown member as an `UnrecognizedField` graded by the slice:
//! the strict 1.26.50 slice's error that skips the file, a warning in the 1.20.0 slice
//! (`Family::reading`). The read tells every failure, as read; a wrong type names what was
//! expected and what was found (the catalogue's [`Texts`]).
//!
//! What is the family's own: the names a category takes (`CategoryName`'s read), an item as a
//! bare string or `{"name"}` (`CatalogItem`'s), and the namespace constraint of a group name
//! (`schema::read_with = group_name`) and of an item name.

use super::codes;
use super::{CatalogItem, CategoryName, CraftingCatalog};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostics, builders};
use bedrock_addon_core::id::ItemId;
use bedrock_addon_core::json::kit::{Expected, Node, Order, Read, Texts, WrongKind};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::schema::{FromJson, UnknownRule};
use bedrock_addon_core::versioning::{ParseCx, read_payload_with};

/// A value's JSON type as the catalogue's messages name it.
fn token(value: &Value) -> &'static str {
    if value.is_object() {
        "an object"
    } else if value.is_array() {
        "an array"
    } else if value.is_string() {
        "a string"
    } else {
        "a number, a boolean or null"
    }
}

/// A value of another JSON type than `expected`: vanilla's `MalformedInput` naming both, which
/// fails the file.
fn wrong_kind(expected: Expected, found: &Value) -> Code {
    let expected = match expected {
        Expected::Object => "an object",
        Expected::Array => "an array",
        Expected::String => "a string",
        Expected::Bool => "a boolean",
        _ => "a number",
    };
    builders::syntax(format!("expected {expected}, found {}", token(found)))
}

/// How the catalogue words the reader's findings: a wrong type names both types; the rest in
/// the codes' own words.
const TEXTS: Texts = Texts {
    wrong_kind: Some(WrongKind(wrong_kind)),
    ..Texts::CODES
};

/// The pattern an item name must match; a mismatch reads "expected string to match the
/// \"<pattern>\" regex".
const ITEM_NAME_REGEX: &str = "^(?:.)+:(?:.)+$";

/// The code of a group name that fails its namespace constraint.
fn group_name_code() -> Code {
    Code::from(&codes::GROUP_NAME)
}

/// The code of an item name that does not match its pattern.
fn item_name_code() -> Code {
    bedrock_addon_core::diagnostic::cereal::string_mismatch(ITEM_NAME_REGEX)
}

/// Whether `text` matches `^(?:.)+:(?:.)+$`: a `:` with at least one character on each side,
/// and no line terminator anywhere (`.` matches `:` too, so `::x` and `a:b:c:d` pass). The
/// pattern has no letter, so the group name's `icase` flag and the item name's case-sensitive
/// match accept the same strings.
fn is_namespaced(text: &str) -> bool {
    let bytes = text.as_bytes();
    !bytes.iter().any(|b| matches!(b, b'\n' | b'\r'))
        && bytes
            .iter()
            .enumerate()
            .any(|(index, b)| *b == b':' && index > 0 && index + 1 < bytes.len())
}

/// The string `node` holds, which must be `namespace:name`; a violation is `code`, vanilla's
/// message of the constraint, and fails the member.
fn constrained(read: &mut Read<'_>, node: &Node<'_>, code: Code) -> Option<String> {
    let value = read.string(node)?;
    if is_namespaced(value) {
        return Some(value.to_owned());
    }
    read.refuse(node, code);
    None
}

/// A group's `name` (`schema::read_with` of `GroupIdentifier::name`): a namespaced string.
pub(crate) fn group_name(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    constrained(read, node, group_name_code())
}

/// `category_name`: one of the four names, compared with case; an unknown name is an
/// `InvalidValue` that stays fatal under either slice.
impl FromJson for CategoryName {
    const EXPECTED: Option<Expected> = Some(Expected::String);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let name = read.string(node)?;
        let category = CategoryName::from_name(name);
        if category.is_none() {
            read.refuse(node, Code::from(&core_codes::INVALID_VALUE));
        }
        category
    }
}

/// An item: the bare string, or `{ "name": "..." }`, whose other members are unknown (told
/// before the name is read, never kept). The game reads the bare string as if it were the
/// object's `name` member; either is a namespaced name.
impl FromJson for CatalogItem {
    const EXPECTED: Option<Expected> = Some(Expected::String);

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if !node.value().is_object() {
            return constrained(read, node, item_name_code())
                .map(|name| Self::new(ItemId::parse(name)));
        }
        let members = read.object(node)?;
        for (key, member) in members.iter() {
            if key != "name" {
                read.unknown_by_rule(&member, UnknownRule::Report);
            }
        }
        let name = read.required(&members, "name")?;
        constrained(read, &name, item_name_code()).map(|name| Self::new(ItemId::parse(name)))
    }
}

// Leaves of the derived check and Molang walk: neither holds a bound or Molang.
bedrock_addon_core::impl_schema_leaf!(CategoryName, CatalogItem);
bedrock_addon_core::impl_no_molang!(CategoryName, CatalogItem);

/// Reads the payload `payload` of a catalogue document; `None` when anything in it fails the
/// file (the reasons are in `d`).
pub(super) fn parse_catalog(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<CraftingCatalog> {
    read_payload_with(payload, cx, d, |read| {
        read.order(Order::UnknownFirst).texts(TEXTS)
    })
}
