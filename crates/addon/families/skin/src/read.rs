//! Reading `skins.json`.

use super::{Skin, SkinPack, codes};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::kit::{Disposition, Members, Node, Read, Written};
use bedrock_addon_core::json::{
    MAX_JSON_DEPTH, PrepareError, Prepared, Value, blank_bom, check_depth, last_wins,
};
use bedrock_addon_core::versioning::{MemberFailure, Reading};

/// The longest path the game's fixed buffers hold.
pub(super) const MAX_PATH: usize = 1023;
/// The most skins creator tools allow in a pack.
const MAX_SKINS: usize = 80;

const PACK_KEYS: [&str; 3] = ["serialize_name", "localization_name", "skins"];
const STRING_KEYS: [&str; 5] = ["localization_name", "geometry", "texture", "type", "cape"];
const PATCH_KEYS: [&str; 4] = [
    "animations",
    "enable_attachables",
    "held_item_ignores_lighting",
    "hide_armor",
];

/// A finding of the crate's own, the game being silent about it.
fn note(at: JsonPath) -> Diagnostic {
    Diagnostic::of(&core_codes::INVALID_VALUE)
        .severity(Severity::Info)
        .at(at)
}

/// The members of one object: a string member as its typed reading, a member of another
/// type kept as written, every member not read kept and reported.
struct Fields<'v> {
    members: Option<Members<'v>>,
    extra: UnknownMembers,
}

impl<'v> Fields<'v> {
    /// The members of `value` at `at`; none, and nothing told, when it is no object.
    fn new(read: &mut Read<'_>, value: &'v Value, at: JsonPath) -> Self {
        let node = Node::new(value, at);
        Self {
            members: value.is_object().then(|| read.object(&node)).flatten(),
            extra: UnknownMembers::default(),
        }
    }

    fn raw(&self, name: &str) -> Option<&'v Value> {
        Some(self.members.as_ref()?.get(name)?.value())
    }

    /// The string member `name`; a member of another type goes to `extra` as written.
    fn string(&mut self, name: &str) -> Option<String> {
        let member = self.raw(name)?;
        if let Some(text) = member.as_string() {
            Some(text.as_str().to_owned())
        } else {
            self.extra.insert(name.to_owned(), member.clone());
            None
        }
    }

    /// `extra`, with every member that is none of `known` added and reported.
    fn finish(mut self, known: &[&str], read: &mut Read<'_>) -> UnknownMembers {
        if let Some(members) = &self.members {
            let unknown = read.unknown(members, known, Disposition::Keep);
            self.extra.extend(
                unknown
                    .into_iter()
                    .map(|(key, value)| (key.to_owned(), value.clone())),
            );
        }
        self.extra
    }
}

/// The read of a `skins.json`: the game reads it without a content-log line for an unknown
/// member, so the kit tells one as an information.
fn reader(findings: &mut Diagnostics) -> Read<'_> {
    Read::new(
        Reading::OWN.with_on_member_failure(MemberFailure::DefaultAndTell),
        findings,
    )
    .unknown_severity(Severity::Info)
}

impl Skin {
    fn read(element: &Value, at: &JsonPath, read: &mut Read<'_>) -> Self {
        let mut fields = Fields::new(read, element, at.clone());
        let mut skin = Self {
            localization_name: fields.string("localization_name"),
            geometry: fields.string("geometry"),
            texture: fields.string("texture"),
            type_: fields.string("type"),
            cape: fields.string("cape"),
            animations: fields.raw("animations").cloned(),
            enable_attachables: fields.raw("enable_attachables").cloned(),
            held_item_ignores_lighting: fields.raw("held_item_ignores_lighting").cloned(),
            hide_armor: fields.raw("hide_armor").cloned(),
            extra: UnknownMembers::default(),
        };
        // A path the game's buffer cannot hold reads as empty; the game's only word is a debug
        // assert, no content-log line, so the crate reports an information.
        for (name, path) in [("texture", &skin.texture), ("cape", &skin.cape)] {
            if path.as_ref().is_some_and(|path| path.len() > MAX_PATH) {
                read.push(note(at.child(name)));
            }
        }
        // The game compares the type with nothing and logs nothing; creators mean `free` or
        // `paid`, so the crate reports an information.
        if fields.raw("type").is_some() && !matches!(skin.type_.as_deref(), Some("free" | "paid")) {
            read.push(note(at.child("type")));
        }
        let known: Vec<&str> = STRING_KEYS.into_iter().chain(PATCH_KEYS).collect();
        skin.extra = fields.finish(&known, read);
        skin
    }
}

impl SkinPack {
    /// Reads the root object of a `skins.json`, and keeps it as written when the fields would
    /// not write it back.
    pub(crate) fn from_value(root: &Value, diagnostics: &mut Diagnostics) -> Self {
        Self::from_value_placed(root, diagnostics).0
    }

    /// [`from_value`](Self::from_value), with the place in the file of each skin held.
    pub(super) fn from_value_placed(
        root: &Value,
        diagnostics: &mut Diagnostics,
    ) -> (Self, Vec<JsonPath>) {
        let (mut pack, places) = Self::read(root, diagnostics);
        if pack.fields_value() != *root {
            *pack.written = Some(root.clone());
        }
        (pack, places)
    }

    /// The game's reading of the root object, with nothing kept beside it, and the place in
    /// the file of each skin held.
    fn read(root: &Value, diagnostics: &mut Diagnostics) -> (Self, Vec<JsonPath>) {
        let at = JsonPath::new();
        let read = &mut reader(diagnostics);
        let mut fields = Fields::new(read, root, at.clone());
        let serialize_name = fields.string("serialize_name");
        let localization_name = fields.string("localization_name");
        let skins_at = at.child("skins");
        let mut places = Vec::new();
        let skins = fields.raw("skins").map(|member| {
            // The game sizes the pack by an array and then iterates whatever is there: an
            // object gives its member values in key order, a scalar nothing.
            let mut entries: Vec<(JsonPath, &Value)> = Vec::new();
            if let Some(elements) = member.as_array() {
                for (i, element) in elements.iter().enumerate() {
                    let at = skins_at.index(i);
                    entries.push((at, element));
                }
            } else {
                // The game iterates the members without a line, so the crate reports an
                // information.
                read.push(note(skins_at.clone()));
                let mut members: Vec<(&str, &Value)> = member
                    .as_object()
                    .into_iter()
                    .flatten()
                    .map(|(name, value)| (name.as_str(), value))
                    .collect();
                members.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
                entries.extend(
                    members
                        .into_iter()
                        .map(|(name, value)| (skins_at.child(name), value)),
                );
            }
            let mut skins = Vec::with_capacity(entries.len());
            for (at, element) in entries {
                if element.is_object() {
                    skins.push(Skin::read(element, &at, read));
                    places.push(at);
                } else {
                    // The game logs the line at the content log's error level and goes on to the
                    // next entry: a kept error.
                    read.push(Diagnostic::of(&codes::ENTRY_NOT_OBJECT).at(at));
                }
            }
            skins
        });
        // Creator tools want the two names equal; the game reads only one.
        if serialize_name.is_none() || serialize_name != localization_name {
            read.push(
                Diagnostic::of(&core_codes::SKIN_PACK_NAME_MISMATCH).at(at.child("serialize_name")),
            );
        }
        // A creator rule, not the game's.
        if skins.as_ref().is_some_and(|skins| skins.len() > MAX_SKINS) {
            read.push(Diagnostic::of(&core_codes::SKIN_PACK_CREATOR_RULE).at(skins_at));
        }
        let pack = Self {
            serialize_name,
            localization_name,
            skins,
            extra: fields.finish(&PACK_KEYS, read),
            written: Written::default(),
        };
        (pack, places)
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
    bedrock_addon_core::json::from_str_jsonc::<Value>(&last_wins(prepared.as_str()))
        .map_err(|error| ReadJsonError::NotJson(error.to_string()))
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
