//! `contents.json`: the table of a pack's files, and for a marketplace pack the key of each
//! encrypted asset.

use std::collections::BTreeMap;

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::kit::{Node, Profile, Read};
use bedrock_addon_core::json::{
    MAX_JSON_DEPTH, Scalars, TooDeep, Value, check_depth, last_wins, strip_bom,
};
use bedrock_addon_core::schema::{Schema, read_document};
use bedrock_addon_core::versioning::Reading;
use facet::Facet;

use super::{ContentKey, PackPath};

/// `contents.json` as read: `{ "version": 1, "content": [ { "path", "key", "mode" } ] }`.
/// Unknown members are ignored; nothing writes the file back.
///
/// Every member is optional: the `contents.json` of an unencrypted pack has no `version` and no
/// entry has a `key`.
#[derive(Clone, Debug, PartialEq, Eq, Facet, Schema)]
pub struct ContentsJson {
    /// `version`: 1 in a current file; the game rewrites any other value (and a missing one);
    /// the crate reports it.
    pub version: Option<u32>,
    /// `content`: one entry per file.
    #[facet(default)]
    pub content: Vec<ContentsEntry>,
}

/// One entry of `contents.json`'s `content` array.
#[derive(Clone, Debug, PartialEq, Eq, Facet, Schema)]
pub struct ContentsEntry {
    /// `path`: the file, relative to the pack root.
    pub path: String,
    /// `key`: the asset's own key string (encrypted packs only).
    pub key: Option<String>,
    /// `mode`: `"cfb"` (the default) or `"ctr"`.
    // The game's newer per-asset cipher mode: absent or `"cfb"` is CFB8, `"ctr"` is counter mode.
    pub mode: Option<String>,
}

/// Why `contents.json` text could not be read.
#[derive(Debug, thiserror::Error)]
pub enum ContentsError {
    /// Nested deeper than [`MAX_JSON_DEPTH`].
    #[error("contents.json is nested too deeply")]
    TooDeep(#[source] TooDeep),
    /// Not a contents table: a syntax or type error.
    #[error("contents.json is not a contents table")]
    Json(#[source] Box<ContentsJsonError>),
}

/// What is wrong with the text of `contents.json`.
#[derive(Debug, thiserror::Error)]
pub enum ContentsJsonError {
    /// The text is not JSON.
    #[error(transparent)]
    Syntax(#[from] facet_json::DeserializeError),
    /// The JSON is not a contents table: the first member that does not read, where it is.
    #[error("the value at `{path}` is not what a contents table holds")]
    Shape {
        /// Where in the document.
        path: JsonPath,
    },
}

impl ContentsJson {
    /// Reads `contents.json` text like any pack JSON: BOM stripped, depth checked, comments
    /// allowed, the last copy of a repeated member standing.
    pub fn parse(bytes: &[u8]) -> Result<ContentsJson, ContentsError> {
        let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
        let bytes = strip_bom(bytes);
        check_depth(bytes, MAX_JSON_DEPTH).map_err(ContentsError::TooDeep)?;
        // Of a member written twice the last one stands, as in the game's own JSON readers; the
        // earlier copies are blanked first. Bytes that are not UTF-8 go to the byte reader as
        // they are.
        let parsed: Result<Value, _> = match std::str::from_utf8(bytes) {
            Ok(text) => bedrock_addon_core::json::from_str_jsonc(&last_wins(text)),
            Err(_) => bedrock_addon_core::json::from_slice_jsonc(bytes),
        };
        let json = parsed.map_err(|error| ContentsError::Json(Box::new(error.into())))?;
        let mut told = Diagnostics::default();
        let mut read = Read::new(Reading::OWN.with_scalars(Scalars::Strict), &mut told)
            .profile(Profile::FIRST_FAILURE);
        read_document(&Node::new(&json, JsonPath::new()), &mut read).ok_or_else(|| {
            let path = told
                .iter()
                .find(|finding| finding.severity == Severity::Error)
                .map(|finding| finding.location.json_path.clone())
                .unwrap_or_default();
            ContentsError::Json(Box::new(ContentsJsonError::Shape { path }))
        })
    }

    /// `version` is 1.
    pub fn is_current(&self) -> bool {
        self.version == Some(1)
    }
}

/// How one asset is decrypted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetCipher {
    /// AES-256-CFB8 with the asset's key (mode `"cfb"` or absent).
    Cfb8(ContentKey),
    /// AES-256-CTR with the asset's key (mode `"ctr"`).
    Ctr(ContentKey),
    /// A key shorter than 32 bytes or an unknown mode: the asset cannot be decrypted.
    Unusable,
}

impl AssetCipher {
    /// The cipher of an entry with `key` and `mode`: the key's first 32 bytes, CFB8 unless the
    /// mode is `"ctr"`.
    fn from_entry(key: &str, mode: Option<&str>) -> AssetCipher {
        let Some(key) = ContentKey::from_slice(key.as_bytes()) else {
            return AssetCipher::Unusable;
        };
        match mode {
            None | Some("cfb") => AssetCipher::Cfb8(key),
            Some("ctr") => AssetCipher::Ctr(key),
            Some(_) => AssetCipher::Unusable,
        }
    }
}

/// The encrypted assets of a pack by path: the entries of `contents.json` that carry a key.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ContentsTable {
    assets: BTreeMap<PackPath, AssetCipher>,
}

impl ContentsTable {
    /// The table of `contents`. An entry without a key is a plain file and is left out, as is
    /// one whose path breaks the pack-path rules (no read can name it).
    pub fn from_json(contents: &ContentsJson) -> ContentsTable {
        let assets = contents
            .content
            .iter()
            .filter_map(|entry| {
                let key = entry.key.as_deref()?;
                let path = PackPath::new(&entry.path).ok()?;
                Some((path, AssetCipher::from_entry(key, entry.mode.as_deref())))
            })
            .collect();
        ContentsTable { assets }
    }

    /// How the asset at `path` is decrypted; `None` for a plain file.
    pub fn get(&self, path: &PackPath) -> Option<&AssetCipher> {
        self.assets.get(path)
    }

    /// How many encrypted assets the table lists.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.assets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Of a member written twice the last one stands, as in the game's own JSON readers. A second `content` array replaces the first; it is not added to it.
    #[test]
    fn of_a_member_written_twice_the_last_stands() {
        let text = br#"{
            "version": 7,
            "content": [ { "path": "a.json", "key": "k" }, { "path": "b.json" } ],
            "version": 1,
            "content": [ { "path": "c.json", "path": "d.json" } ]
        }"#;
        let contents = ContentsJson::parse(text).unwrap();
        assert!(contents.is_current());
        let paths: Vec<&str> = contents
            .content
            .iter()
            .map(|entry| entry.path.as_str())
            .collect();
        assert_eq!(paths, ["d.json"]);
        assert_eq!(ContentsTable::from_json(&contents).len(), 0);
    }

    #[test]
    fn contents_errors_display_lowercase_and_carry_their_cause() {
        use std::error::Error;
        let error = ContentsJson::parse(b"{ \"content\": 5 }").unwrap_err();
        assert!(error.source().is_some(), "the parse error is the source");
        let text = error.to_string();
        assert!(
            text.starts_with("contents.json") && !text.ends_with('.'),
            "{text}"
        );
    }

    #[test]
    fn a_file_without_a_repeated_member_reads_as_before() {
        let text = br#"{ "version": 1, "content": [ { "path": "a.json" } ] }"#;
        let contents = ContentsJson::parse(text).unwrap();
        assert_eq!(contents.content.len(), 1);
        assert!(matches!(
            ContentsJson::parse(b"{ \"content\": 5 }"),
            Err(ContentsError::Json(_))
        ));
        assert!(matches!(
            ContentsJson::parse(b"\xff\xfe"),
            Err(ContentsError::Json(_))
        ));
    }
}
