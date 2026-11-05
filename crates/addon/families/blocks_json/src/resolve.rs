//! Reading one `blocks.json` file as the client does (the loader entry point): the effective
//! version, then every member in bytewise key order, each resolved to the blocks it applies to.

use facet_value::VObject;
use indexmap::IndexMap;

use super::entry::{self, ReadEntry};
use super::legacy::{self, LegacyAlias};
use super::texts::{
    BASE_GAME_BLOCKSHAPE, NOT_JSON, OLD_NAME, VERSION_ABOVE_CEILING, VERSION_INVALID,
    VERSION_NOT_GIVEN, formatted,
};
use super::version::{self, BLOCKS_JSON_MAX_FORMAT_VERSION, Effective, Log};
use super::{BlocksJson, ClientBlock};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, Severity,
};
use bedrock_addon_core::id::BlockId;
use bedrock_addon_core::json::{
    JsonSlice, MAX_JSON_DEPTH, Value, check_depth, last_wins, strip_bom, trailing_content_finding,
};

/// The namespace of every vanilla block, and the only one a vanilla block can have.
const MINECRAFT: &str = "minecraft";

/// One block an entry applies to.
#[derive(Clone, Debug)]
pub(super) struct Applied {
    /// The block.
    pub(super) id: BlockId,
    /// The key the entry is written under.
    pub(super) key: String,
    /// The fields the entry sets (`blockshape` is set by the stack, from `requested_shape`).
    pub(super) block: ClientBlock,
    /// The shape the entry asks for.
    pub(super) requested_shape: Option<(String, i16)>,
    /// The findings of this application.
    pub(super) diagnostics: Diagnostics,
}

/// A file as read.
#[derive(Clone, Debug)]
pub(super) struct FileRead {
    /// The file's members; `None` when the client skips the file (it is not a JSON object).
    pub(super) file: Option<BlocksJson>,
    /// The version the client reads the file as.
    pub(super) effective: Effective,
    /// The findings about the file and about the keys that apply to no block.
    pub(super) diagnostics: Diagnostics,
    /// Every application, in the order the client makes them.
    pub(super) applied: Vec<Applied>,
}

/// Reads the bytes of a `blocks.json` of a pack that is (`base_game`) or is not the base game's.
pub(super) fn read(bytes: &[u8], base_game: bool) -> FileRead {
    let mut diagnostics = Diagnostics::default();
    let root = match parse(bytes) {
        Ok(root) => root,
        Err(reason) => {
            let message = reason.to_string();
            // The client skips the file without a word, so the finding is an information of
            // this crate's own; the file is still refused.
            diagnostics.push(
                Diagnostic::info(DiagnosticCode::MalformedInput {
                    text: Some(formatted(NOT_JSON, &[&message])),
                    message,
                })
                .phase(Phase::Validate),
            );
            return FileRead {
                file: None,
                effective: Effective::release(version::BLOCKS_JSON_DEFAULT_VERSION),
                diagnostics,
                applied: Vec::new(),
            };
        }
    };
    // The client keeps the last value of a repeated key; each earlier one is reported.
    if let Ok(text) = std::str::from_utf8(strip_bom(bytes)) {
        // Content after the root object: the client stops reading at the object's end, and this
        // is the reader of the file's root, so it reports the ignored content. A leading byte
        // order mark is part of the file, so the offset counts it.
        let offset = bytes.len() - text.len();
        diagnostics.extend(trailing_content_finding(JsonSlice { text, offset }));
        for (key, count) in duplicate_keys(text) {
            for _ in 1..count {
                diagnostics.push(
                    Diagnostic::info(DiagnosticCode::DuplicateIdentifier { text: None })
                        .at(path(&key))
                        .phase(Phase::Validate),
                );
            }
        }
    }
    let mut file = BlocksJson {
        format_version: None,
        format_version_index: 0,
        entries: IndexMap::new(),
    };
    for (key, value) in &root {
        // A number beyond a double is held as the largest finite one.
        let mut value = value.clone();
        for clamped in bedrock_addon_core::json::clamp_non_finite(&mut value, None) {
            let mut at = path(key.as_str());
            at.append(&clamped.path);
            diagnostics.push(
                Diagnostic::info(DiagnosticCode::ClampedValue { text: None })
                    .at(at)
                    .phase(Phase::Validate),
            );
        }
        if key.as_str() == "format_version" {
            // Its place among the entries is kept for the writer, so the file writes back with
            // the version where its author put it.
            file.format_version_index = file.entries.len();
            file.format_version = Some(value);
        } else {
            file.entries.insert(key.as_str().to_owned(), value);
        }
    }
    let outcome = version::effective(file.format_version.as_ref());
    let fell_back = version_log(outcome.log.as_ref(), &mut diagnostics);
    // The client walks the members in `strcmp` order of their keys, because its JSON object is
    // a map ordered by key.
    let mut keys: Vec<&String> = file.entries.keys().collect();
    keys.sort_unstable_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    let mut applied = Vec::new();
    for key in keys {
        let cx = KeyCx {
            key,
            version: &outcome.effective,
            fell_back,
            base_game,
        };
        resolve(&cx, &file.entries[key], &mut diagnostics, &mut applied);
    }
    // The readers above work on parsed values, which have no offsets: each finding gets the
    // span of the value its path names from the text.
    let stripped = strip_bom(bytes);
    if let Ok(text) = std::str::from_utf8(stripped) {
        let spans = Spans::of(JsonSlice {
            text,
            offset: bytes.len() - stripped.len(),
        });
        spans.fill(&mut diagnostics);
        for application in &mut applied {
            spans.fill(&mut application.diagnostics);
        }
    }
    FileRead {
        file: Some(file),
        effective: outcome.effective,
        diagnostics,
        applied,
    }
}

/// The members of a file's root by key, split once, so that the span of any finding's path
/// is found below its member without reading the whole file again per finding.
struct Spans<'a> {
    root: JsonSlice<'a>,
    members: IndexMap<String, facet_json::RawJson<'a>>,
}

impl<'a> Spans<'a> {
    fn of(root: JsonSlice<'a>) -> Self {
        // A key written twice keeps its last value, the one the client reads.
        let members = bedrock_addon_core::json::parse_at(root).unwrap_or_default();
        Self { root, members }
    }

    /// Gives every finding without a span the span of the value its path names; a path
    /// that names nothing in the text (a member that is absent, the file itself) stays
    /// without one.
    fn fill(&self, diagnostics: &mut Diagnostics) {
        for mut finding in std::mem::take(diagnostics) {
            if finding.location.span.is_none() {
                finding.location.span = self.at(&finding.location.json_path);
            }
            diagnostics.push(finding);
        }
    }

    fn at(&self, path: &JsonPath) -> Option<bedrock_addon_core::diagnostic::Span> {
        let mut segments = path.segments();
        let bedrock_addon_core::diagnostic::JsonPathSegment::Key(key) = segments.next()? else {
            return None;
        };
        let member = self.root.nested(self.members.get(key)?)?;
        let mut below = JsonPath::new();
        for segment in segments {
            match segment {
                bedrock_addon_core::diagnostic::JsonPathSegment::Key(key) => below.push(key),
                bedrock_addon_core::diagnostic::JsonPathSegment::Index(index) => {
                    below.push_index(index);
                }
            }
        }
        bedrock_addon_core::json::span_at(member, &below)
    }
}

// A number that is no finite double (`1e999`, which jsoncpp reads as an infinity) is held as
// the largest finite double of its sign, with a `ClampedValue` information at its path: an
// infinity has no JSON text, so a file that held one could not be written back. The
// information is the record that the file as written differs from the file as read. The walk is
// core's (`json::clamp_non_finite`).

/// Why a file is no JSON object. The text of each is the reason in the finding.
#[derive(Debug, thiserror::Error)]
enum NotAnObject {
    /// The text nests deeper than the reader follows.
    #[error("nested too deeply")]
    TooDeep,
    /// The bytes are not UTF-8.
    #[error("not UTF-8")]
    NotUtf8,
    /// The text is not JSON: the reader's own error.
    #[error("{0}")]
    Syntax(Box<facet_json::DeserializeError>),
    /// The text is JSON and its root is another value than an object.
    #[error("the root is not an object")]
    Root,
}

/// The root object of a file, with the last value of every repeated key.
fn parse(bytes: &[u8]) -> Result<VObject, NotAnObject> {
    let bytes = strip_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH).map_err(|_| NotAnObject::TooDeep)?;
    let text = std::str::from_utf8(bytes).map_err(|_| NotAnObject::NotUtf8)?;
    let json = bedrock_addon_core::json::from_str_jsonc::<Value>(&last_wins(text))
        .map_err(|error| NotAnObject::Syntax(Box::new(error)))?;
    match json.as_object() {
        Some(object) => Ok(object.clone()),
        None => Err(NotAnObject::Root),
    }
}

/// The findings of the version: vanilla's warning, and an information for a partial version.
/// Returns whether the client fell back from what the file says, which is what makes a split of
/// a legacy name worth reporting.
fn version_log(log: Option<&Log>, d: &mut Diagnostics) -> bool {
    let at = path("format_version");
    match log {
        None => return false,
        Some(Log::NotGiven) => d.push(
            Diagnostic::warn(DiagnosticCode::MissingFormatVersion {
                text: Some(formatted(VERSION_NOT_GIVEN, &[])),
            })
            .phase(Phase::Validate),
        ),
        Some(Log::Invalid { partial }) => {
            d.push(
                Diagnostic::warn(DiagnosticCode::MalformedFormatVersion {
                    text: Some(formatted(VERSION_INVALID, &[])),
                })
                .at(at.clone())
                .phase(Phase::Validate),
            );
            if partial.is_some() {
                // The client says it uses 1.1.0 and reads the partial version.
                d.push(
                    Diagnostic::info(DiagnosticCode::UnusualFormatVersion)
                        .at(at)
                        .phase(Phase::Validate),
                );
            }
        }
        Some(Log::AboveCeiling(written)) => d.push(
            Diagnostic::warn(DiagnosticCode::UnsupportedVersion {
                text: Some(formatted(
                    VERSION_ABOVE_CEILING,
                    &[written, &BLOCKS_JSON_MAX_FORMAT_VERSION.to_string()],
                )),
            })
            .at(at)
            .phase(Phase::Validate),
        ),
    }
    true
}

/// What resolving one key needs besides the key's value.
struct KeyCx<'a> {
    /// The key as written.
    key: &'a str,
    /// The file's effective version.
    version: &'a Effective,
    /// The client fell back from the file's own version (the missing, invalid or too-new
    /// version warning was logged).
    fell_back: bool,
    /// The file is a base-game pack's.
    base_game: bool,
}

/// The path of the root member `key`.
fn path(key: &str) -> JsonPath {
    JsonPath::new().child(key)
}

/// Resolves the member `key: value`, in the order the client tests it: a member that is not an
/// object, a base-game entry with a `blockshape`, the mushroom blocks, `chalkboard`, then a
/// legacy name or an ordinary block name.
fn resolve(cx: &KeyCx<'_>, value: &Value, d: &mut Diagnostics, applied: &mut Vec<Applied>) {
    let key = cx.key;
    let at = path(key);
    // A member that is not an object is skipped.
    let Some(object) = value.as_object() else {
        d.push(
            Diagnostic::info(DiagnosticCode::InvalidType { text: None })
                .at(at)
                .phase(Phase::Validate),
        );
        return;
    };
    entry::unread_members(object, key, d);
    // Before the key is looked at: a base-game pack's entry with a `blockshape` member
    // (presence is the test, so `null` counts) is skipped whole.
    if cx.base_game && object.get("blockshape").is_some() {
        // The game logs this as an error, skips the entry and loads the file: a kept error.
        d.push_kept_error(
            Diagnostic::error(DiagnosticCode::InvalidValue {
                text: Some(formatted(BASE_GAME_BLOCKSHAPE, &[key])),
            })
            .at(at)
            .phase(Phase::Validate),
        );
        return;
    }
    // The alias lookup is on `minecraft:` and the lower-cased key, so it is blind to a key that
    // carries a namespace.
    let lower = key.to_lowercase();
    // A mushroom block's entry also goes to the stem.
    if matches!(
        lower.as_str(),
        "brown_mushroom_block" | "red_mushroom_block"
    ) {
        d.push(
            Diagnostic::info(DiagnosticCode::ShadowedDefinition { text: None })
                .at(at)
                .phase(Phase::Validate),
        );
        apply(cx, object, &lower, true, applied);
        apply(cx, object, "mushroom_stem", false, applied);
        return;
    }
    // `chalkboard` is ignored without a word.
    if lower == "chalkboard" {
        d.push(
            Diagnostic::info(DiagnosticCode::IgnoredMember { text: None })
                .at(at)
                .phase(Phase::Validate),
        );
        return;
    }
    match legacy::alias(&lower) {
        // At or above its gate a legacy name is an error, unless it is still the name of its own
        // block.
        Some(alias) if alias.is_flattened_at(cx.version) => {
            if alias.is_still_a_block() {
                apply(cx, object, &lower, true, applied);
            } else {
                // The game logs this as an error, skips the entry and loads the file: a kept
                // error, except in a base-game pack, where it is only a debug line.
                let finding = old_name(alias, cx.base_game, at);
                if finding.severity == Severity::Error {
                    d.push_kept_error(finding);
                } else {
                    d.push(finding);
                }
            }
        }
        // Below its gate the entry is split onto the post-split blocks.
        Some(alias) => split(cx, alias, object, d, applied),
        // An ordinary name.
        None => ordinary(cx, object, &lower, d, applied),
    }
}

/// The old-name error. The client logs it for its own base-game files too; there it is noise,
/// so it is kept at debug level.
fn old_name(alias: &LegacyAlias, base_game: bool, at: JsonPath) -> Diagnostic {
    let mut names = String::new();
    for (_, name) in alias.post_split {
        names.push_str("\n - minecraft:");
        names.push_str(name);
    }
    let severity = if base_game {
        Severity::Debug
    } else {
        Severity::Error
    };
    Diagnostic::error(DiagnosticCode::DeprecatedMember {
        text: Some(formatted(
            OLD_NAME,
            &[&format!("{MINECRAFT}:{}", alias.name), &names],
        )),
    })
    .severity(severity)
    .at(at)
    .phase(Phase::Validate)
}

/// Splits a legacy name: a copy of the entry for every post-split block, its texture names given
/// the block's variant.
fn split(
    cx: &KeyCx<'_>,
    alias: &LegacyAlias,
    object: &VObject,
    d: &mut Diagnostics,
    applied: &mut Vec<Applied>,
) {
    let at = path(cx.key);
    if alias.removed {
        // `lava_cauldron` is dropped without a word.
        d.push(
            Diagnostic::info(DiagnosticCode::IgnoredMember { text: None })
                .at(at)
                .phase(Phase::Validate),
        );
        return;
    }
    if cx.fell_back {
        // The author asked for another version and got the split.
        d.push(
            Diagnostic::info(DiagnosticCode::DeprecatedMember { text: None })
                .at(at)
                .phase(Phase::Validate),
        );
    }
    for (index, (position, name)) in alias.post_split.iter().enumerate() {
        let variant = alias.start_variant + position;
        let mut copy = object.clone();
        for member in ["textures", "carried_textures"] {
            if let Some(rewritten) = object.get(member).and_then(|v| with_variant(v, variant)) {
                copy.insert(member, rewritten);
            }
        }
        apply(cx, &copy, name, index == 0, applied);
    }
}

/// A texture member with `:<variant>` after every texture name: a string, or the string
/// members of an object. `None` for a value the client leaves alone.
fn with_variant(value: &Value, variant: u32) -> Option<Value> {
    if let Some(name) = value.as_string() {
        return Some(Value::from(format!("{}:{variant}", name.as_str()).as_str()));
    }
    let object = value.as_object()?;
    let mut rewritten = VObject::new();
    for (face, name) in object {
        match name.as_string() {
            Some(name) => rewritten.insert(
                face.as_str(),
                Value::from(format!("{}:{variant}", name.as_str()).as_str()),
            ),
            None => rewritten.insert(face.as_str(), name.clone()),
        };
    }
    Some(rewritten.into())
}

/// An ordinary name: the key names a block itself. A bare key is a vanilla block's name,
/// lower-cased; a key with a namespace is taken as written, except a namespaced legacy name,
/// which the registry resolves to the legacy name's block of data value 0. The registry looks
/// the name up as given, then lower-cased with `minecraft:` when it has no namespace. Whether
/// the block exists is the link stage's question.
fn ordinary(
    cx: &KeyCx<'_>,
    object: &VObject,
    lower: &str,
    d: &mut Diagnostics,
    applied: &mut Vec<Applied>,
) {
    let Some((namespace, name)) = cx.key.split_once(':') else {
        apply(cx, object, lower, true, applied);
        return;
    };
    let aliased = namespace
        .eq_ignore_ascii_case(MINECRAFT)
        .then(|| legacy::alias(&name.to_lowercase()))
        .flatten()
        .filter(|alias| !alias.removed && !alias.is_still_a_block());
    if let Some(alias) = aliased {
        d.push(
            Diagnostic::info(DiagnosticCode::IdentifierNamespaced)
                .at(path(cx.key))
                .phase(Phase::Validate),
        );
        apply(cx, object, alias.aux0, true, applied);
        return;
    }
    // A vanilla name is lower case; the registry lower-cases a name it does not find as written.
    let vanilla = namespace.eq_ignore_ascii_case(MINECRAFT);
    let id = if vanilla {
        BlockId::parse(cx.key.to_lowercase())
    } else {
        BlockId::parse(cx.key)
    };
    push(cx, object, id, vanilla, true, applied);
}

/// Applies `object` to the vanilla block `name` (without namespace).
fn apply(cx: &KeyCx<'_>, object: &VObject, name: &str, first: bool, applied: &mut Vec<Applied>) {
    let id = BlockId::parse(format!("{MINECRAFT}:{name}"));
    push(cx, object, id, true, first, applied);
}

/// Reads `object` for the block `id`. The crate's own findings about the entry are raised once
/// per key (`first`); vanilla's are raised for every block, as the client logs them.
fn push(
    cx: &KeyCx<'_>,
    object: &VObject,
    id: BlockId,
    vanilla: bool,
    first: bool,
    applied: &mut Vec<Applied>,
) {
    let mut diagnostics = Diagnostics::default();
    let ReadEntry {
        block,
        requested_shape,
    } = entry::read(object, cx.key, id.as_str(), vanilla, &mut diagnostics);
    if !first {
        let found = std::mem::take(&mut diagnostics);
        for diagnostic in found {
            if has_vanilla_text(&diagnostic.code) {
                diagnostics.push(diagnostic);
            }
        }
    }
    applied.push(Applied {
        id,
        key: cx.key.to_owned(),
        block,
        requested_shape,
        diagnostics,
    });
}

/// Whether a finding of [`entry::read`] is one the client logs.
fn has_vanilla_text(code: &DiagnosticCode) -> bool {
    matches!(
        code,
        DiagnosticCode::InvalidType { text: Some(_) }
            | DiagnosticCode::InvalidValue { text: Some(_) }
            | DiagnosticCode::IgnoredMember { text: Some(_) }
    )
}

/// The keys the root object of `text` holds more than once, each with the number of times, in
/// the order of their first occurrence. Keys compare by their value, as the client's reader
/// does; text that is not well-formed JSON gives what the scan saw before it stopped. The scan
/// ends where the root value does, and a root that is an array or a string has no keys. The
/// client's reader keeps the last value of a repeated key, so the parsed value holds each key
/// once and the count has to come from this scan of the text.
pub(super) fn duplicate_keys(text: &str) -> Vec<(String, usize)> {
    let bytes = text.as_bytes();
    let mut counts: IndexMap<String, usize> = IndexMap::new();
    let mut depth = 0usize;
    let mut expects_key = false;
    let mut at = 0;
    while at < bytes.len() {
        match bytes[at] {
            // A root that is no object has no keys: the commas of a root array stand before
            // elements, and what follows a root string or array is not the file's.
            b'"' | b'[' if depth == 0 => break,
            b'"' => {
                let start = at;
                at += 1;
                while at < bytes.len() && bytes[at] != b'"' {
                    at += if bytes[at] == b'\\' { 2 } else { 1 };
                }
                at = (at + 1).min(bytes.len());
                if depth == 1 && expects_key {
                    expects_key = false;
                    let key = text.get(start..at).and_then(|literal| {
                        bedrock_addon_core::json::from_str::<String>(literal).ok()
                    });
                    if let Some(key) = key {
                        *counts.entry(key).or_insert(0) += 1;
                    }
                }
                continue;
            }
            b'/' if bytes.get(at + 1) == Some(&b'/') => {
                while at < bytes.len() && bytes[at] != b'\n' {
                    at += 1;
                }
                continue;
            }
            b'/' if bytes.get(at + 1) == Some(&b'*') => {
                at += 2;
                while at < bytes.len() && !(bytes[at] == b'*' && bytes.get(at + 1) == Some(&b'/')) {
                    at += 1;
                }
                at = (at + 2).min(bytes.len());
                continue;
            }
            b'{' => {
                depth += 1;
                expects_key = depth == 1;
            }
            b'[' => depth += 1,
            b'}' | b']' => {
                depth = depth.saturating_sub(1);
                // The root has ended: the client's reader stops here, and what follows is
                // content after the root, not members of the file.
                if depth == 0 {
                    break;
                }
            }
            b',' if depth == 1 => expects_key = true,
            _ => {}
        }
        at += 1;
    }
    counts.into_iter().filter(|(_, count)| *count > 1).collect()
}

#[cfg(test)]
mod tests {
    use super::{NotAnObject, parse};

    /// Each way a file is no JSON object is a reason of its own, whatever its words.
    #[test]
    fn a_file_that_is_no_json_object_is_refused_for_its_own_reason() {
        let deep = "[".repeat(bedrock_addon_core::json::MAX_JSON_DEPTH + 1);
        assert!(matches!(parse(deep.as_bytes()), Err(NotAnObject::TooDeep)));
        assert!(matches!(
            parse(b"{\"a\": \"\xff\"}"),
            Err(NotAnObject::NotUtf8)
        ));
        for syntax in ["", "{\"a\": }", r#"{"obsidian": {"textures": "b""#] {
            assert!(
                matches!(parse(syntax.as_bytes()), Err(NotAnObject::Syntax(_))),
                "{syntax:?}"
            );
        }
        for root in ["[]", "5", r#""blocks""#] {
            assert!(
                matches!(parse(root.as_bytes()), Err(NotAnObject::Root)),
                "{root:?}"
            );
        }
        assert!(parse(br#"{"obsidian": {}}"#).is_ok());
    }
}
