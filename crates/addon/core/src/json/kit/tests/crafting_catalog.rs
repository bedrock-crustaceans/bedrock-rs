//! The crafting catalogue's reader in the kit's terms: `families/crafting_catalog/src/read.rs`
//! (`Reader::catalog`, `category`, `category_name`, `group`, `group_identifier`, `item`,
//! `list`, `constrained`, with its `object`, `array`, `string`, `unknown` and `required`).
//!
//! Two of its findings are worded otherwise than the kit's, and stay the family's through
//! [`Read::push`]: a value of another JSON type is a `MalformedInput` naming both types, and a
//! missing member carries no span. Its unknown members are the kit's, told as warnings with the
//! span of their value (the strict 1.26.50 slice makes them errors); the catalogue keeps none.

use super::*;
use crate::diagnostic::cereal;
use crate::diagnostic::codes as core_codes;

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

/// `Reader::expect`: `node` holds a value of type `expected`, or the file fails.
fn expect(read: &mut Read<'_>, node: &Node<'_>, expected: &'static str) -> bool {
    let found = token(node.value());
    if found == expected {
        return true;
    }
    let mut finding = Diagnostic::of(crate::diagnostic::builders::syntax(format!(
        "expected {expected}, found {found}"
    )))
    .at(node.path().clone());
    finding.location.span = node.span();
    read.push(finding);
    false
}

fn object<'v>(read: &mut Read<'_>, node: &Node<'v>) -> Option<Members<'v>> {
    expect(read, node, "an object").then(|| read.object(node))?
}

fn array<'v>(read: &mut Read<'_>, node: &Node<'v>) -> Option<Elements<'v>> {
    expect(read, node, "an array").then(|| read.array(node))?
}

fn string<'v>(read: &mut Read<'_>, node: &Node<'v>) -> Option<&'v str> {
    expect(read, node, "a string").then(|| read.string(node))?
}

/// `Reader::required`: the member, or `MissingRequiredField` at it without a span.
fn required<'v>(read: &mut Read<'_>, members: &Members<'v>, key: &str) -> Option<Node<'v>> {
    let node = read.opt(members, key);
    if node.is_none() {
        read.push(
            Diagnostic::of(&core_codes::MISSING_REQUIRED_FIELD)
                .at(members.node().path().child(key)),
        );
    }
    node
}

/// `Reader::unknown`: the kit's, nothing kept.
fn unknown(read: &mut Read<'_>, members: &Members<'_>, known: &[&str]) {
    read.unknown(members, known, Disposition::Keep);
}

/// `is_namespaced`: a `:` with a character on each side, and no line terminator.
fn is_namespaced(text: &str) -> bool {
    let bytes = text.as_bytes();
    !bytes.iter().any(|b| matches!(b, b'\n' | b'\r'))
        && bytes
            .iter()
            .enumerate()
            .any(|(index, b)| *b == b':' && index > 0 && index + 1 < bytes.len())
}

/// `Reader::constrained`: a namespaced string, else `code`.
fn constrained(read: &mut Read<'_>, node: &Node<'_>, code: Code) -> Option<String> {
    let value = string(read, node)?;
    if is_namespaced(value) {
        return Some(value.to_owned());
    }
    let mut finding = Diagnostic::error(code).at(node.path().clone());
    finding.location.span = node.span();
    read.push(finding);
    None
}

fn group_name_code() -> Code {
    core_codes::INVALID_VALUE_WORDED
        .with(["string must be prefixed with a namespace (eg. namespace:value)"])
}

fn item_name_code() -> Code {
    cereal::string_mismatch("^(?:.)+:(?:.)+$")
}

/// `Reader::list`: a required array, each element read by `each`.
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

fn catalog(read: &mut Read<'_>, node: &Node<'_>) -> Option<Vec<(String, usize)>> {
    let members = object(read, node)?;
    unknown(read, &members, &["categories"]);
    list(read, &members, "categories", category)
}

fn category(read: &mut Read<'_>, node: &Node<'_>) -> Option<(String, usize)> {
    let members = object(read, node)?;
    unknown(read, &members, &["category_name", "groups"]);
    let name = required(read, &members, "category_name").and_then(|node| {
        let name = string(read, &node)?;
        let known = matches!(name, "construction" | "nature" | "equipment" | "items");
        if !known {
            let mut finding = Diagnostic::of(&core_codes::INVALID_VALUE).at(node.path().clone());
            finding.location.span = node.span();
            read.push(finding);
        }
        known.then(|| name.to_owned())
    });
    let groups = list(read, &members, "groups", group);
    Some((name?, groups?.len()))
}

fn group(read: &mut Read<'_>, node: &Node<'_>) -> Option<usize> {
    let members = object(read, node)?;
    unknown(read, &members, &["group_identifier", "items"]);
    let identifier = read
        .opt(&members, "group_identifier")
        .map(|node| group_identifier(read, &node));
    let items = list(read, &members, "items", item);
    if let Some(identifier) = identifier {
        identifier?;
    }
    items.map(|items| items.len())
}

fn group_identifier(read: &mut Read<'_>, node: &Node<'_>) -> Option<String> {
    let members = object(read, node)?;
    unknown(read, &members, &["name", "icon"]);
    let name = required(read, &members, "name")
        .and_then(|node| constrained(read, &node, group_name_code()));
    let icon = read.opt(&members, "icon").map(|node| item(read, &node));
    if let Some(icon) = icon {
        icon?;
    }
    name
}

fn item(read: &mut Read<'_>, node: &Node<'_>) -> Option<String> {
    if node.value().is_object() {
        let members = object(read, node)?;
        unknown(read, &members, &["name"]);
        let name = required(read, &members, "name")?;
        return constrained(read, &name, item_name_code());
    }
    constrained(read, node, item_name_code())
}

#[test]
fn crafting_catalog_s_reader_reads_as_the_kit() {
    assert_reads_as(FIXTURES, EXPECTED, |text, found| {
        let json = value(text);
        let payload = Node::with_text(
            &json,
            at(&["minecraft:crafting_items_catalog"]),
            JsonSlice::new(text),
        );
        // The raw reader warns; the strict slice makes the warnings errors.
        let mut read = Read::new(Reading::DOCUMENT.with_unknown(UnknownMask::Warn), found);
        catalog(&mut read, &payload).is_some()
    });
}

/// Catalogue payloads after `addon/tests/crafting_catalog` (`loader.rs`, `stack.rs`), the
/// family's `tests/crafting_catalog/located.rs` and the reader's own failures.
const FIXTURES: &[&str] = &[
    r#"{"categories": [{"category_name": "construction", "groups": [{"group_identifier": {"name": "custom:g"}, "items": ["custom:thing", "custom:ore"]}]}]}"#,
    r"{}",
    r#"{"categories": [], "extra": 1}"#,
    r"[]",
    r#"{"categories": {}}"#,
    r#"{"categories": [{"category_name": "nature", "groups": [], "zz": 1}, 5, {"groups": []}]}"#,
    r#"{"categories": [{"category_name": 3, "groups": [{"items": [{"name": "custom:a"}, {"nome": "x"}, "bad", 7], "group_identifier": {"icon": "custom:i", "x": 0}}]}]}"#,
    r#"{"categories": [{"category_name": "items", "groups": [{"group_identifier": {"name": "nons"}, "items": []}, {"group_identifier": "g", "items": {}}]}]}"#,
    r#"{"categories": [{"category_name": "Equipment", "groups": [{"group_identifier": {"name": "a:b", "icon": {"name": "c:d", "q": 1}}, "items": [null]}]}]}"#,
];

/// What `parse_catalog` tells on each of `FIXTURES`, and whether it reads it.
const EXPECTED: &[(bool, &[&str])] = &[
    // {"categories": [{"category_name": "construction", "groups": [{"group_identifier": {"name": "custom:g
    (true, &[]),
    // {}
    (
        false,
        &[
            r"Error|minecraft:crafting_items_catalog/categories|core/missing_required_field|None|false",
        ],
    ),
    // {"categories": [], "extra": 1}
    (
        true,
        &[
            r"Warning|minecraft:crafting_items_catalog/extra|core/unrecognized_field|Some((28, 1))|false",
        ],
    ),
    // []
    (
        false,
        &[
            r#"Error|minecraft:crafting_items_catalog|core/syntax ["expected an object, found an array"]|Some((0, 2))|false"#,
        ],
    ),
    // {"categories": {}}
    (
        false,
        &[
            r#"Error|minecraft:crafting_items_catalog/categories|core/syntax ["expected an array, found an object"]|Some((15, 2))|false"#,
        ],
    ),
    // {"categories": [{"category_name": "nature", "groups": [], "zz": 1}, 5, {"groups": []}]}
    (
        false,
        &[
            r"Warning|minecraft:crafting_items_catalog/categories/0/zz|core/unrecognized_field|Some((64, 1))|false",
            r#"Error|minecraft:crafting_items_catalog/categories/1|core/syntax ["expected an object, found a number, a boolean or null"]|Some((68, 1))|false"#,
            r"Error|minecraft:crafting_items_catalog/categories/2/category_name|core/missing_required_field|None|false",
        ],
    ),
    // {"categories": [{"category_name": 3, "groups": [{"items": [{"name": "custom:a"}, {"nome": "x"}, "bad
    (
        false,
        &[
            r#"Error|minecraft:crafting_items_catalog/categories/0/category_name|core/syntax ["expected a string, found a number, a boolean or null"]|Some((34, 1))|false"#,
            r"Warning|minecraft:crafting_items_catalog/categories/0/groups/0/group_identifier/x|core/unrecognized_field|Some((153, 1))|false",
            r"Error|minecraft:crafting_items_catalog/categories/0/groups/0/group_identifier/name|core/missing_required_field|None|false",
            r"Warning|minecraft:crafting_items_catalog/categories/0/groups/0/items/1/nome|core/unrecognized_field|Some((90, 3))|false",
            r"Error|minecraft:crafting_items_catalog/categories/0/groups/0/items/1/name|core/missing_required_field|None|false",
            r#"Error|minecraft:crafting_items_catalog/categories/0/groups/0/items/2|core/string_mismatch ["^(?:.)+:(?:.)+$"]|Some((96, 5))|false"#,
            r#"Error|minecraft:crafting_items_catalog/categories/0/groups/0/items/3|core/syntax ["expected a string, found a number, a boolean or null"]|Some((103, 1))|false"#,
        ],
    ),
    // {"categories": [{"category_name": "items", "groups": [{"group_identifier": {"name": "nons"}, "items"
    (
        false,
        &[
            r#"Error|minecraft:crafting_items_catalog/categories/0/groups/0/group_identifier/name|core/invalid_value_worded ["string must be prefixed with a namespace (eg. namespace:value)"]|Some((84, 6))|false"#,
            r#"Error|minecraft:crafting_items_catalog/categories/0/groups/1/group_identifier|core/syntax ["expected an object, found a string"]|Some((128, 3))|false"#,
            r#"Error|minecraft:crafting_items_catalog/categories/0/groups/1/items|core/syntax ["expected an array, found an object"]|Some((142, 2))|false"#,
        ],
    ),
    // {"categories": [{"category_name": "Equipment", "groups": [{"group_identifier": {"name": "a:b", "icon
    (
        false,
        &[
            r"Error|minecraft:crafting_items_catalog/categories/0/category_name|core/invalid_value|Some((34, 11))|false",
            r"Warning|minecraft:crafting_items_catalog/categories/0/groups/0/group_identifier/icon/q|core/unrecognized_field|Some((124, 1))|false",
            r#"Error|minecraft:crafting_items_catalog/categories/0/groups/0/items/0|core/syntax ["expected a string, found a number, a boolean or null"]|Some((139, 4))|false"#,
        ],
    ),
];
