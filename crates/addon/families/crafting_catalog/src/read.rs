//! Reading a catalogue payload on the reader kit, each level a node with its text so every
//! finding keeps its span: vanilla's cereal reader fails the file on a wrong type, a missing
//! required member, an unknown category or a constraint, and tells every unknown member as an
//! `UnrecognizedField` graded by the slice: the strict 1.26.50 slice's error that skips the
//! file, a warning in the 1.20.0 slice (`Family::reading`).

use indexmap::IndexMap;

use super::{CatalogItem, Category, CategoryName, CraftingCatalog, Group, GroupIdentifier};
use bedrock_addon_core::diagnostic::{Class, Diagnostic, DiagnosticCode, Diagnostics, VanillaText};
use bedrock_addon_core::id::ItemId;
use bedrock_addon_core::json::kit::{Disposition, Elements, Members, Node, Read};
use bedrock_addon_core::json::{JsonSlice, Value, from_str_jsonc, parse_at};
use bedrock_addon_core::versioning::{ParseCx, Reading, UnknownMask};
use facet_json::RawJson;

/// How the raw reader reads: the strict document, its unknown members told at the slice's grade
/// (`parse_catalog`).
const READING: Reading = Reading::DOCUMENT.with_unknown(UnknownMask::Warn);

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

/// Vanilla's text for a group name that fails its namespace constraint.
const GROUP_NAME_TEXT: &str = "string must be prefixed with a namespace (eg. namespace:value)";

/// The pattern an item name must match; a mismatch reads "expected string to match the
/// \"<pattern>\" regex".
const ITEM_NAME_REGEX: &str = "^(?:.)+:(?:.)+$";

/// The code of a group name that fails its namespace constraint.
fn group_name_code() -> DiagnosticCode {
    DiagnosticCode::InvalidValue {
        text: Some(VanillaText::Fixed(GROUP_NAME_TEXT)),
    }
}

/// The code of an item name that does not match its pattern.
fn item_name_code() -> DiagnosticCode {
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

/// An error with `code` at `node`, spanning it.
fn error(read: &mut Read<'_>, code: DiagnosticCode, node: &Node<'_>) {
    let mut finding = Diagnostic::error(code).at(node.path().clone());
    finding.location.span = node.span();
    read.push(finding);
}

/// Requires `node` to hold a value of type `expected`: another JSON type is vanilla's
/// `MalformedInput` naming both, and fails the file.
fn expect(read: &mut Read<'_>, node: &Node<'_>, expected: &'static str) -> bool {
    let found = token(node.value());
    if found == expected {
        return true;
    }
    error(
        read,
        DiagnosticCode::MalformedInput {
            message: format!("expected {expected}, found {found}"),
            text: None,
        },
        node,
    );
    false
}

/// The members of the object `node`.
fn object<'v>(read: &mut Read<'_>, node: &Node<'v>) -> Option<Members<'v>> {
    expect(read, node, "an object").then(|| read.object(node))?
}

/// The elements of the array `node`.
fn array<'v>(read: &mut Read<'_>, node: &Node<'v>) -> Option<Elements<'v>> {
    expect(read, node, "an array").then(|| read.array(node))?
}

/// The string `node` holds.
fn string<'v>(read: &mut Read<'_>, node: &Node<'v>) -> Option<&'v str> {
    expect(read, node, "a string").then(|| read.string(node))?
}

/// The string `node` holds, which must be `namespace:name`; a violation is `code`, vanilla's
/// message of the constraint.
fn constrained(read: &mut Read<'_>, node: &Node<'_>, code: DiagnosticCode) -> Option<String> {
    let value = string(read, node)?;
    if is_namespaced(value) {
        return Some(value.to_owned());
    }
    error(read, code, node);
    None
}

/// The members of an object level other than `known`: each an `UnrecognizedField` warning,
/// never kept: the game's reader skips the member it does not bind and stores nothing of it.
/// The slice decides whether the warning stays one (below 1.26.50 the game logs it at the
/// content log's warning level and goes on) or becomes an error (from 1.26.50 on).
fn unknown(read: &mut Read<'_>, members: &Members<'_>, known: &[&str]) {
    read.unknown(members, known, Disposition::Keep);
}

/// The required member `key` of `members`, or the finding of its absence (vanilla's carries no
/// span).
fn required<'v>(read: &mut Read<'_>, members: &Members<'v>, key: &str) -> Option<Node<'v>> {
    let node = read.opt(members, key);
    if node.is_none() {
        read.push(
            Diagnostic::error(DiagnosticCode::MissingRequiredField { text: None })
                .at(members.node().path().child(key)),
        );
    }
    node
}

/// `categories`, `groups` or `items`: a required array member, each element read by `each`.
/// `None` when the member or any element failed.
fn list<'v, T>(
    read: &mut Read<'_>,
    members: &Members<'v>,
    key: &str,
    each: impl Fn(&mut Read<'_>, &Node<'v>) -> Option<T>,
) -> Option<Vec<T>> {
    let node = required(read, members, key)?;
    let elements = array(read, &node)?;
    let read: Vec<Option<T>> = elements
        .iter()
        .map(|element| each(read, &element))
        .collect();
    read.into_iter().collect()
}

/// The payload: `{ "categories": [...] }`.
fn catalog(read: &mut Read<'_>, node: &Node<'_>) -> Option<CraftingCatalog> {
    let members = object(read, node)?;
    unknown(read, &members, &["categories"]);
    let categories = list(read, &members, "categories", category)?;
    Some(CraftingCatalog { categories })
}

/// One `categories` entry: `{ "category_name": "...", "groups": [...] }`.
fn category(read: &mut Read<'_>, node: &Node<'_>) -> Option<Category> {
    let members = object(read, node)?;
    unknown(read, &members, &["category_name", "groups"]);
    let category_name = category_name(read, &members);
    let groups = list(read, &members, "groups", group);
    Some(Category {
        category_name: category_name?,
        groups: groups?,
    })
}

/// `category_name`: one of the four names, compared with case.
// An unknown category name stays fatal under either slice.
fn category_name(read: &mut Read<'_>, members: &Members<'_>) -> Option<CategoryName> {
    let node = required(read, members, "category_name")?;
    let name = string(read, &node)?;
    let category = CategoryName::from_name(name);
    if category.is_none() {
        error(read, DiagnosticCode::InvalidValue { text: None }, &node);
    }
    category
}

/// One `groups` entry: `{ "group_identifier"?: {...}, "items": [...] }`.
fn group(read: &mut Read<'_>, node: &Node<'_>) -> Option<Group> {
    let members = object(read, node)?;
    unknown(read, &members, &["group_identifier", "items"]);
    let group_identifier = read
        .opt(&members, "group_identifier")
        .map(|node| self::group_identifier(read, &node));
    let items = list(read, &members, "items", item);
    Some(Group {
        group_identifier: match group_identifier {
            Some(identifier) => Some(identifier?),
            None => None,
        },
        items: items?,
    })
}

/// `group_identifier`: `{ "name": "...", "icon"?: item }`.
fn group_identifier(read: &mut Read<'_>, node: &Node<'_>) -> Option<GroupIdentifier> {
    let members = object(read, node)?;
    unknown(read, &members, &["name", "icon"]);
    let name = required(read, &members, "name")
        .and_then(|node| constrained(read, &node, group_name_code()));
    let icon = read.opt(&members, "icon").map(|node| item(read, &node));
    Some(GroupIdentifier {
        name: name?,
        icon: match icon {
            Some(icon) => Some(icon?),
            None => None,
        },
    })
}

/// An item: the bare string, or `{ "name": "..." }`.
// The game reads the bare string as if it were the object's `name` member.
fn item(read: &mut Read<'_>, node: &Node<'_>) -> Option<CatalogItem> {
    let name = if node.value().is_object() {
        let members = object(read, node)?;
        unknown(read, &members, &["name"]);
        let name = required(read, &members, "name")?;
        constrained(read, &name, item_name_code())?
    } else {
        constrained(read, node, item_name_code())?
    };
    Some(CatalogItem {
        name: ItemId::parse(name),
    })
}

/// Reads the payload `payload` of a catalogue document; `None` when anything in it fails the
/// file (the reasons are in `d`).
pub(super) fn parse_catalog(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<CraftingCatalog> {
    let Ok(json) = from_str_jsonc::<Value>(payload.text) else {
        // Not reached for a payload the door has read; the typed read words the failure.
        if let Err(error) = parse_at::<IndexMap<String, RawJson<'_>>>(payload) {
            let at_member = !cx.at.is_empty() || error.has_path();
            d.push(error.into_diagnostic(&cx.at, |_, error| error.code_with_path(at_member)));
        }
        return None;
    };
    let node = Node::with_text(&json, cx.at.clone(), payload);
    let unknown = cx.load.grade(Class::Unknown);
    catalog(&mut Read::new(READING, d).unknown_severity(unknown), &node)
}
