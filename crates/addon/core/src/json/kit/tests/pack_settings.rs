//! The saved pack settings reader in the kit's terms: `addon/src/pack/settings/read.rs`
//! (`Reader::document`, `entry`, `values`, `value`, `pack_id`, `last_used`, `float`, with its
//! `object`, `members`, `required`) under the 1.26.40 grammar, each level a slice of the file.
//!
//! The reader is the strict document's in every finding, so the kit's wording, severities and
//! spans are its own: a wrong type spans the value, an unknown member its value, a missing
//! member the object. Its two informations (a string that is no UUID, a number past a float)
//! are the family's, through [`Read::push`].

use super::*;
use crate::diagnostic::codes as core_codes;

/// An information with `code` at `node`, spanning it.
fn information(read: &mut Read<'_>, node: &Node<'_>) {
    let mut finding = Diagnostic::of(&core_codes::INVALID_VALUE)
        .severity(Severity::Info)
        .at(node.path().clone());
    finding.location.span = node.span();
    read.push(finding);
}

/// `Reader::document`: `{"settings": [...]}`.
fn document(read: &mut Read<'_>, payload: &Node<'_>) -> Option<Vec<String>> {
    let members = read.object(payload)?;
    read.unknown(&members, &["settings"], Disposition::Keep);
    let settings = read.required(&members, "settings")?;
    let entries = read.array(&settings)?;
    let mut entries_read = Vec::with_capacity(entries.len());
    let mut failed = false;
    for entry in entries.iter() {
        match self::entry(read, &entry) {
            Some(entry) => entries_read.push(entry),
            None => failed = true,
        }
    }
    (!failed).then_some(entries_read)
}

/// `Reader::entry`: `pack_id` and `values` required, `last_used` optional.
fn entry(read: &mut Read<'_>, node: &Node<'_>) -> Option<String> {
    let members = read.object(node)?;
    read.unknown(
        &members,
        &["pack_id", "last_used", "values"],
        Disposition::Keep,
    );
    let pack_id = read.required(&members, "pack_id");
    let values = read.required(&members, "values");
    let pack_id = self::pack_id(read, &pack_id?);
    let last_used = match read.opt(&members, "last_used") {
        Some(node) => Some(self::last_used(read, &node)?),
        None => None,
    };
    let values = self::values(read, &values?);
    Some(format!("{:?} {last_used:?} {:?}", pack_id?, values?))
}

/// `Reader::pack_id`: a string; one that is no UUID is the nil UUID, with an information.
fn pack_id(read: &mut Read<'_>, node: &Node<'_>) -> Option<uuid::Uuid> {
    let text = read.string(node)?;
    let id = matches!(text.len(), 32 | 36)
        .then(|| uuid::Uuid::parse_str(text).ok())
        .flatten();
    if id.is_none() {
        information(read, node);
    }
    Some(id.unwrap_or(uuid::Uuid::nil()))
}

/// `Reader::last_used`: an unsigned 64-bit integer.
fn last_used(read: &mut Read<'_>, node: &Node<'_>) -> Option<u64> {
    let value = node
        .value()
        .as_number()
        .filter(|number| number.is_integer())
        .and_then(facet_value::VNumber::to_u64);
    if value.is_none() {
        read.wrong_type(node);
    }
    value
}

/// `Reader::values`: an object of setting values.
fn values(read: &mut Read<'_>, node: &Node<'_>) -> Option<Vec<String>> {
    let members = read.object(node)?;
    let mut values = Vec::with_capacity(members.len());
    let mut failed = false;
    for (_, member) in members.iter() {
        match value(read, &member) {
            Some(value) => values.push(value),
            None => failed = true,
        }
    }
    (!failed).then_some(values)
}

/// `Reader::value`: a number (an `f32`), a bool, a string, or an array of strings.
fn value(read: &mut Read<'_>, node: &Node<'_>) -> Option<String> {
    let json = node.value();
    let value = if let Some(number) = json.as_number() {
        let number = number.to_f64_lossy();
        #[allow(clippy::cast_possible_truncation, reason = "the game reads a float")]
        let float = number as f32;
        if !float.is_finite() {
            information(read, node);
        }
        Some(format!("{float}"))
    } else if let Some(flag) = json.as_bool() {
        Some(flag.to_string())
    } else if let Some(text) = json.as_string() {
        Some(text.as_str().to_owned())
    } else if let Some(items) = json.as_array() {
        items
            .iter()
            .map(|item| item.as_string().map(|text| text.as_str().to_owned()))
            .collect::<Option<Vec<_>>>()
            .map(|list| list.join(","))
    } else {
        None
    };
    if value.is_none() {
        read.wrong_type(node);
    }
    value
}

#[test]
fn pack_settings_reader_reads_as_the_kit() {
    assert_reads_as(FIXTURES, EXPECTED, |text, found| {
        let json = super::value(text);
        let payload = Node::with_text(
            &json,
            at(&["minecraft:pack_settings"]),
            JsonSlice::new(text),
        );
        let mut read = Read::new(Reading::DOCUMENT, found);
        document(&mut read, &payload).is_some()
    });
}

/// Settings payloads after `addon/tests/pack_settings/members.rs` (`ps09`, `ps10`, `d6`) and
/// the reader's own types.
const FIXTURES: &[&str] = &[
    r#"{"settings": []}"#,
    r#"{"settings": [], "comment": "x"}"#,
    r#"{"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "values": {}, "extra": 1}]}"#,
    r#"{"settings": [{"pack_id": 5, "values": {}}]}"#,
    r#"{"settings": [{"pack_id": "BadPackID", "values": {}}]}"#,
    r"{}",
    r#"{"settings": [{"values": {}}]}"#,
    r#"{"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915"}]}"#,
    r#"{"settings": {}}"#,
    r"[]",
    r#"{"settings": [5, {"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "values": []}]}"#,
    r#"{"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "last_used": 12, "values": {"a": 1.5, "b": true, "c": "x", "d": ["p", "q"], "e": null, "f": {}, "g": 1e39, "h": [1]}}]}"#,
    r#"{"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "last_used": -1, "values": {}}]}"#,
    r#"{"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "last_used": 1.5, "values": {}}]}"#,
    r#"{"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "last_used": "3", "values": {}}]}"#,
    r#"{"settings": [{"pack_id": null, "values": null, "zz": 0}]}"#,
];

/// What `read::read` tells on each of `FIXTURES`, and whether it reads it.
const EXPECTED: &[(bool, &[&str])] = &[
    // {"settings": []}
    (true, &[]),
    // {"settings": [], "comment": "x"}
    (
        true,
        &[r"Error|minecraft:pack_settings/comment|core/unrecognized_field|Some((28, 3))|false"],
    ),
    // {"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "values": {}, "extra": 1}]}
    (
        true,
        &[
            r"Error|minecraft:pack_settings/settings/0/extra|core/unrecognized_field|Some((89, 1))|false",
        ],
    ),
    // {"settings": [{"pack_id": 5, "values": {}}]}
    (
        false,
        &[
            r"Error|minecraft:pack_settings/settings/0/pack_id|core/invalid_type|Some((26, 1))|false",
        ],
    ),
    // {"settings": [{"pack_id": "BadPackID", "values": {}}]}
    (
        true,
        &[
            r"Info|minecraft:pack_settings/settings/0/pack_id|core/invalid_value|Some((26, 11))|false",
        ],
    ),
    // {}
    (
        false,
        &[r"Error|minecraft:pack_settings/settings|core/missing_required_field|Some((0, 2))|false"],
    ),
    // {"settings": [{"values": {}}]}
    (
        false,
        &[
            r"Error|minecraft:pack_settings/settings/0/pack_id|core/missing_required_field|Some((14, 14))|false",
        ],
    ),
    // {"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915"}]}
    (
        false,
        &[
            r"Error|minecraft:pack_settings/settings/0/values|core/missing_required_field|Some((14, 51))|false",
        ],
    ),
    // {"settings": {}}
    (
        false,
        &[r"Error|minecraft:pack_settings/settings|core/invalid_type|Some((13, 2))|false"],
    ),
    // []
    (
        false,
        &[r"Error|minecraft:pack_settings|core/invalid_type|Some((0, 2))|false"],
    ),
    // {"settings": [5, {"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "values": []}]}
    (
        false,
        &[
            r"Error|minecraft:pack_settings/settings/0|core/invalid_type|Some((14, 1))|false",
            r"Error|minecraft:pack_settings/settings/1/values|core/invalid_type|Some((79, 2))|false",
        ],
    ),
    // {"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "last_used": 12, "values": {"a": 1
    (
        false,
        &[
            r"Error|minecraft:pack_settings/settings/0/values/e|core/invalid_type|Some((147, 4))|false",
            r"Error|minecraft:pack_settings/settings/0/values/f|core/invalid_type|Some((158, 2))|false",
            r"Info|minecraft:pack_settings/settings/0/values/g|core/invalid_value|Some((167, 4))|false",
            r"Error|minecraft:pack_settings/settings/0/values/h|core/invalid_type|Some((178, 3))|false",
        ],
    ),
    // {"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "last_used": -1, "values": {}}]}
    (
        false,
        &[
            r"Error|minecraft:pack_settings/settings/0/last_used|core/invalid_type|Some((79, 2))|false",
        ],
    ),
    // {"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "last_used": 1.5, "values": {}}]}
    (
        false,
        &[
            r"Error|minecraft:pack_settings/settings/0/last_used|core/invalid_type|Some((79, 3))|false",
        ],
    ),
    // {"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "last_used": "3", "values": {}}]}
    (
        false,
        &[
            r"Error|minecraft:pack_settings/settings/0/last_used|core/invalid_type|Some((79, 3))|false",
        ],
    ),
    // {"settings": [{"pack_id": null, "values": null, "zz": 0}]}
    (
        false,
        &[
            r"Error|minecraft:pack_settings/settings/0/zz|core/unrecognized_field|Some((54, 1))|false",
            r"Error|minecraft:pack_settings/settings/0/pack_id|core/invalid_type|Some((26, 4))|false",
            r"Error|minecraft:pack_settings/settings/0/values|core/invalid_type|Some((42, 4))|false",
        ],
    ),
];
