//! What the three sound documents share when the client merges their copies as JSON across the
//! stack: one pack's copy, the reading of it, the checks on the copies, and the rejection of a
//! file read on its own that is no JSON. Each family's loader walks its one file
//! (`Walk::exact_files`) with [`parse_copy`] as its parse.

use bedrock_addon_core::load::{LoadContext, WalkFile};

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, JsonPathSegment, Severity, VanillaText,
};
use bedrock_addon_core::document::{Origin, RawLayer, Rejected, unversioned};
use bedrock_addon_core::json::{
    EncodeError, MAX_JSON_DEPTH, Value, WriteStyle, check_depth, write_json,
};

use crate::jsoncpp::ReadError;

/// Vanilla's text for a lowest copy that is no JSON: the merge is given up.
const BASE_UNPARSABLE: &str = "Could not parse file at the top of the stack: {}.";

/// One pack's copy of a sound document: its JSON, or why it is none.
#[derive(Clone, Debug)]
pub struct SoundCopy {
    /// Where the copy came from.
    pub origin: Origin,
    /// The JSON as read, or why it is none.
    pub json: Result<Value, ReadError>,
    /// What reading the copy found, in its pack's file: the warnings of the numbers no
    /// double holds, each read as the largest finite one.
    pub found: Diagnostics,
    /// The pack's Molang version, which the `sounds.json` transform stamps.
    pub molang_version: i16,
}

/// Reads `bytes` as JSON the way the merge does: the last of two equal members stands.
///
/// # Errors
///
/// A [`ReadError`] for nesting past the limit, bytes that are no UTF-8, or text that is no JSON.
pub fn read_json(bytes: &[u8]) -> Result<Value, ReadError> {
    // The BOM is blanked, not cut, so every offset is an offset into the file as written.
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    // Depth before UTF-8, as the merge reads: a text both too deep and not UTF-8 is
    // `TooDeep` (`json::Prepared` would say `NotUtf8`).
    check_depth(bytes, MAX_JSON_DEPTH)?;
    let text = std::str::from_utf8(bytes)?;
    Ok(bedrock_addon_core::json::from_str_jsonc::<Value>(
        &bedrock_addon_core::json::last_wins(text),
    )?)
}

/// Moves `found` into `into`, each finding at `origin`'s pack and file.
pub fn located(found: Diagnostics, origin: &Origin, into: &mut Diagnostics) {
    // Pushed one by one, as the merge always has: the places of the moves `found` held are
    // not carried over (`Diagnostics::append` would carry them).
    into.extend(found.located(origin));
}

/// The copies that are JSON, lowest first, after the checks of the game's merge: a
/// lowest copy that is no JSON gives the merge up (an error with vanilla's text), any other such
/// copy is skipped (an information).
pub fn readable(
    path: &str,
    copies: Vec<SoundCopy>,
    diagnostics: &mut Diagnostics,
) -> Vec<(Origin, Value, i16)> {
    let mut out = Vec::new();
    for (index, copy) in copies.into_iter().enumerate() {
        diagnostics.extend(copy.found);
        match copy.json {
            Ok(json) => out.push((copy.origin, json, copy.molang_version)),
            Err(error) => {
                let mut found = Diagnostics::default();
                let (text, severity) = if index == 0 {
                    (
                        Some(VanillaText::Formatted {
                            template: BASE_UNPARSABLE,
                            args: vec![path.to_owned()],
                        }),
                        Severity::Error,
                    )
                } else {
                    (None, Severity::Info)
                };
                found.push(
                    Diagnostic::error(DiagnosticCode::MalformedInput {
                        message: error.to_string(),
                        text,
                    })
                    .severity(severity),
                );
                located(found, &copy.origin, diagnostics);
                if index == 0 {
                    return Vec::new();
                }
            }
        }
    }
    out
}

/// The layers of a merge, top first.
pub fn layers(copies: &[(Origin, Value, i16)]) -> Vec<RawLayer> {
    copies
        .iter()
        .rev()
        .map(|(origin, json, _)| RawLayer::new(origin.clone(), json.clone()))
        .collect()
}

/// The member name at segment `at` of a finding's path; `None` when the path is shorter or the
/// segment is an array index.
pub fn path_member(path: &JsonPath, at: usize) -> Option<String> {
    path.segments().nth(at).and_then(|segment| match segment {
        JsonPathSegment::Key(key) => Some(key.to_owned()),
        JsonPathSegment::Index(_) => None,
    })
}

/// The rejection of a file that is no JSON.
pub fn not_json(origin: Origin, error: &ReadError) -> Rejected {
    let mut found = Diagnostics::default();
    found.push(Diagnostic::error(DiagnosticCode::MalformedInput {
        message: error.to_string(),
        text: None,
    }));
    let mut diagnostics = Diagnostics::default();
    located(found, &origin, &mut diagnostics);
    Rejected::new(origin, diagnostics)
}

/// Writes a document's JSON.
///
/// # Errors
///
/// `EncodeError::NotRepresentable` when `json` holds a non-finite number, or nests deeper than the
/// readers of the crate follow.
pub fn write_document(
    json: &Value,
    out: &mut Vec<u8>,
    style: WriteStyle,
) -> Result<(), EncodeError> {
    write_json(json, out, style)
}

/// One pack's copy of the sound document a walked file is: its JSON, or why it is none, and
/// what reading found.
pub fn parse_copy(file: &WalkFile, cx: &LoadContext) -> SoundCopy {
    let origin = unversioned::origin_of(file);
    // The copy is read with the numbers no double holds clamped, so neither its
    // layer nor the merged document keeps an infinity; the warning names this pack's
    // file.
    let finite = bedrock_addon_core::document::FiniteText::of(&file.bytes);
    let json = read_json(finite.bytes());
    let mut found = Diagnostics::default();
    finite.settle(&mut found, Some(&origin));
    SoundCopy {
        origin,
        json,
        found,
        molang_version: cx.molang_version.0,
    }
}
