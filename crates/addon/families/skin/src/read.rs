//! Reading `skins.json`: the types derive their read; what is the crate's own is the list of
//! skins (the game sizes it by an array and iterates whatever is there) and the rules the
//! game does not state, told as informations.

use super::{Skin, SkinPack, codes};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::kit::fixed_path::{iterated, raw_member, strict_reader};
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::{
    MAX_JSON_DEPTH, PrepareError, Prepared, Value, blank_bom, check_depth, last_wins,
};
use bedrock_addon_core::schema::{FromJson, UnknownRule, read_document};

/// The longest path the game's fixed buffers hold.
pub(super) const MAX_PATH: usize = 1023;
/// The most skins creator tools allow in a pack.
const MAX_SKINS: usize = 80;
/// The members of a pack that a field takes.
const PACK_KEYS: [&str; 3] = ["serialize_name", "localization_name", "skins"];

/// A finding of the crate's own, the game being silent about it.
fn note(at: JsonPath) -> Diagnostic {
    Diagnostic::of(&core_codes::INVALID_VALUE)
        .severity(Severity::Info)
        .at(at)
}

/// The rules of one skin, told before it is read: its own members tell nothing but the
/// unknown ones, which come last.
fn skin_rules(element: &Node<'_>, read: &mut Read<'_>) {
    let at = element.path();
    // A path the game's buffer cannot hold reads as empty; the game's only word is a debug
    // assert, no content-log line, so the crate reports an information.
    for name in ["texture", "cape"] {
        let path = raw_member(element.value(), name).and_then(Value::as_string);
        if path.is_some_and(|path| path.as_str().len() > MAX_PATH) {
            read.push(note(at.child(name)));
        }
    }
    // The game compares the type with nothing and logs nothing; creators mean `free` or
    // `paid`, so the crate reports an information.
    if let Some(kind) = raw_member(element.value(), "type")
        && !kind
            .as_string()
            .is_some_and(|kind| matches!(kind.as_str(), "free" | "paid"))
    {
        read.push(note(at.child("type")));
    }
}

/// The `skins` of a pack (`schema::read_with`). The game sizes the pack by an array and then
/// iterates whatever is there: an object gives its member values in key order, a scalar
/// nothing, and the crate reports an information. An entry that is no object is the game's
/// content-log line and a kept error.
#[allow(
    clippy::unnecessary_wraps,
    reason = "the shape of a `read_with` function"
)]
pub(super) fn skins(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Skin>> {
    if !node.value().is_array() {
        read.push(note(node.path().clone()));
    }
    let mut skins = Vec::new();
    for (at, entry) in iterated(node.value(), node.path()) {
        if !entry.is_object() {
            read.push(Diagnostic::of(&codes::ENTRY_NOT_OBJECT).at(at));
            continue;
        }
        let entry = Node::new(entry, at);
        skin_rules(&entry, read);
        skins.extend(Skin::from_json(&entry, read));
    }
    Some(skins)
}

/// The place in the file of each skin a read of `root` holds.
pub(super) fn places(root: &Value) -> Vec<JsonPath> {
    let at = JsonPath::new().child("skins");
    raw_member(root, "skins")
        .into_iter()
        .flat_map(|skins| iterated(skins, &at))
        .filter(|(_, entry)| entry.is_object())
        .map(|(at, _)| at)
        .collect()
}

impl SkinPack {
    /// Reads the root object of a `skins.json`, and keeps it as written when the fields would
    /// not write it back.
    pub(crate) fn from_value(root: &Value, diagnostics: &mut Diagnostics) -> Self {
        let mut read = strict_reader(diagnostics);
        let node = Node::new(root, JsonPath::new());
        let mut pack: Self = read_document(&node, &mut read).unwrap_or_default();
        // Creator tools want the two names equal; the game reads only one.
        if pack.serialize_name.is_none() || pack.serialize_name != pack.localization_name {
            read.push(
                Diagnostic::of(&core_codes::SKIN_PACK_NAME_MISMATCH)
                    .at(JsonPath::new().child("serialize_name")),
            );
        }
        // A creator rule, not the game's.
        if pack
            .skins
            .as_ref()
            .is_some_and(|skins| skins.len() > MAX_SKINS)
        {
            read.push(
                Diagnostic::of(&core_codes::SKIN_PACK_CREATOR_RULE)
                    .at(JsonPath::new().child("skins")),
            );
        }
        // The members no field takes are kept by the read and told after the rules above.
        for (key, member) in root.as_object().into_iter().flatten() {
            if !PACK_KEYS.contains(&key.as_str()) {
                let at = JsonPath::new().child(key.as_str());
                read.unknown_by_rule(&Node::new(member, at), UnknownRule::Keep);
            }
        }
        *pack.written = (pack.fields_value().ok().as_ref() != Some(root)).then(|| root.clone());
        pack
    }
}

/// A JSON file's root the way the game's reader takes it: a BOM skipped, comments accepted,
/// the last of repeated members kept.
pub(super) fn read_json(bytes: &[u8]) -> Result<Value, ReadJsonError> {
    // The nesting is checked before the encoding, as the game's reader does: a text that is
    // both too deep and not UTF-8 is too deep. `Prepared` checks the encoding first. The
    // mark is blanked first, or the scan stops at it.
    let bytes = &*blank_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH).map_err(|deep| ReadJsonError::TooDeep {
        offset: deep.offset,
    })?;
    let prepared = Prepared::new(bytes).map_err(|error| match error {
        PrepareError::NotUtf8(error) => ReadJsonError::NotUtf8(error),
        PrepareError::TooDeep(deep) => ReadJsonError::TooDeep {
            offset: deep.offset,
        },
        // `new` never refuses an empty text.
        PrepareError::Empty => ReadJsonError::NotJson(PrepareError::Empty.to_string()),
    })?;
    bedrock_addon_core::json::from_str_jsonc(&last_wins(prepared.as_str()))
        .map_err(|error| ReadJsonError::NotJson(error.kind.to_string()))
}

/// Why bytes are no JSON value; the display is the message a finding carries.
#[derive(Debug, thiserror::Error)]
pub(super) enum ReadJsonError {
    #[error("nesting too deep at byte {offset}")]
    TooDeep { offset: usize },
    #[error(transparent)]
    NotUtf8(#[from] std::str::Utf8Error),
    #[error("{0}")]
    NotJson(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_json_names_why_the_bytes_are_no_value() {
        fn assert_error<E: std::error::Error>(_: &E) {}
        assert_error(&read_json(b"{").unwrap_err());
        assert!(read_json(b"{}").is_ok());
        let deep = vec![b'['; 4096];
        let offset = check_depth(&deep, MAX_JSON_DEPTH).unwrap_err().offset;
        assert_eq!(
            read_json(&deep).unwrap_err().to_string(),
            format!("nesting too deep at byte {offset}")
        );
        assert_eq!(
            read_json(b"\"\xff\"").unwrap_err().to_string(),
            std::str::from_utf8(core::hint::black_box(b"\"\xff\"" as &[u8]))
                .unwrap_err()
                .to_string()
        );
        assert!(!read_json(b"{").unwrap_err().to_string().is_empty());
    }

    #[test]
    fn a_text_too_deep_and_not_utf8_is_too_deep() {
        let mut both = vec![b'['; 4096];
        both.push(0xff);
        assert!(matches!(
            read_json(&both),
            Err(ReadJsonError::TooDeep { .. })
        ));
    }

    /// Behind a byte-order mark too, at the offset in the file as written.
    #[test]
    fn a_text_too_deep_and_not_utf8_behind_a_bom_is_too_deep() {
        let mut both = b"\xEF\xBB\xBF".to_vec();
        both.extend(vec![b'['; 4096]);
        both.push(0xff);
        assert!(matches!(
            read_json(&both),
            Err(ReadJsonError::TooDeep { offset }) if offset == MAX_JSON_DEPTH + 3
        ));
    }

    #[test]
    fn a_byte_order_mark_is_read_past() {
        assert!(read_json(b"\xEF\xBB\xBF{}").unwrap().is_object());
        assert!(matches!(
            read_json(b"\xEF\xBB\xBF"),
            Err(ReadJsonError::NotJson(_))
        ));
    }
}
