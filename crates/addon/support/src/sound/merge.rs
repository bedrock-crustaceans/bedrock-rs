//! What the three sound documents share when the client merges their copies as JSON across the
//! stack: one pack's copy, the reading of it, the checks on the copies, and the documents of
//! the files read on their own.

use bedrock_addon_core::load::{AppDataPlace, ListDepth, LoadContext, Walk, WalkFile, WalkOrder};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, JsonPathSegment, Location, Phase, Severity,
    VanillaText,
};
use bedrock_addon_core::document::{Origin, RawLayer, Rejected, VersionInfo};
use bedrock_addon_core::json::{
    EncodeError, MAX_JSON_DEPTH, Value, WriteStyle, check_depth, strip_bom, write_json,
};
use bedrock_addon_core::version::ENGINE_VERSION;

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
    let bytes = strip_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH)?;
    let text = std::str::from_utf8(bytes)?;
    Ok(bedrock_addon_core::json::from_str_jsonc::<Value>(
        &bedrock_addon_core::json::last_wins(text),
    )?)
}

/// Moves `found` into `into`, each finding at `origin`'s pack and file.
pub fn located(found: Diagnostics, origin: &Origin, into: &mut Diagnostics) {
    for mut finding in found {
        finding.location.pack = Some(origin.pack.clone());
        finding.location.path = Some(origin.path.clone());
        into.push(finding);
    }
}

/// A finding of the merge itself.
#[must_use]
pub fn merge_finding(code: DiagnosticCode, severity: Severity, path: JsonPath) -> Diagnostic {
    Diagnostic::new(
        code,
        severity,
        Phase::Parse,
        Location::new(None, None, path, None),
    )
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
                found.push(merge_finding(
                    DiagnosticCode::MalformedInput {
                        message: error.to_string(),
                        text,
                    },
                    severity,
                    JsonPath::new(),
                ));
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
#[must_use]
pub fn layers(copies: &[(Origin, Value, i16)]) -> Vec<RawLayer> {
    copies
        .iter()
        .rev()
        .map(|(origin, json, _)| RawLayer::new(origin.clone(), json.clone()))
        .collect()
}

/// The member name at segment `at` of a finding's path; `None` when the path is shorter or the
/// segment is an array index.
#[must_use]
pub fn path_member(path: &JsonPath, at: usize) -> Option<String> {
    path.segments().nth(at).and_then(|segment| match segment {
        JsonPathSegment::Key(key) => Some(key.to_owned()),
        JsonPathSegment::Index(_) => None,
    })
}

/// The origin of a file parsed outside a stack.
#[must_use]
pub fn detached_origin(path: &PackPath) -> Origin {
    Origin::new(
        PackRef::app_data(),
        PackSide::Resource,
        path.clone(),
        None,
        Vec::new(),
    )
}

/// The rejection of a file that is no JSON.
#[must_use]
pub fn not_json(origin: Origin, error: &ReadError) -> Rejected {
    let mut found = Diagnostics::default();
    found.push(merge_finding(
        DiagnosticCode::MalformedInput {
            message: error.to_string(),
            text: None,
        },
        Severity::Error,
        JsonPath::new(),
    ));
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

/// The text of a document's JSON.
///
/// # Errors
///
/// `EncodeError::NotRepresentable` when `json` holds a non-finite number, or nests deeper than the
/// readers of the crate follow.
pub fn document_text(json: &Value) -> Result<String, EncodeError> {
    let mut out = Vec::new();
    write_document(json, &mut out, WriteStyle::default())?;
    Ok(String::from_utf8(out).unwrap_or_default())
}

/// The version information of a file no loader reads a version of.
#[must_use]
pub fn unversioned() -> VersionInfo {
    VersionInfo::new(None, false, ENGINE_VERSION, ENGINE_VERSION, false)
}

/// One pack's copy of the sound document a walked file is: its JSON, or why it is none, and
/// what reading found.
pub fn parse_copy(file: &WalkFile, cx: &LoadContext) -> SoundCopy {
    let origin = Origin::new(
        file.pack.clone(),
        file.side,
        file.path.clone(),
        None,
        Vec::new(),
    );
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

/// The walk of one document: the file `accept` takes, in every pack, lowest pack first, as
/// vanilla collects the copies of a merged file.
pub const fn walk(accept: fn(&PackPath) -> bool) -> Walk {
    Walk {
        directories: &[""],
        depth: ListDepth::Recursive,
        accept,
        order: WalkOrder::BottomUp,
        directory_major: false,
        app_data: &[],
        app_data_place: AppDataPlace::After,
        in_pack: None,
        shadowed: None,
        read_error: None,
    }
}
