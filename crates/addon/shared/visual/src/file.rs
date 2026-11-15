//! A file of a client entity or an attachable, from its bytes to a document: vanilla's admit
//! step (the text, the root, the version gate), the chain, the placing of the findings, and
//! the rules that decide which definition of an identifier wins in a stack.

use std::borrow::Cow;

use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, Phase, Severity, builders,
};
use bedrock_addon_core::document::{Document, EngineVersionDecision, Origin, Rejected};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, PrepareError, Prepared, Unrepresentable, Value, WriteStyle, write_json,
};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::{FormatVersionInput, SemVersion};
use bedrock_addon_core::versioning::{
    Identity, Loaded, Parsed, VersionStruct, finish, parse_as_written,
};

use crate::codes as visual_codes;
use crate::family::Visual;
use crate::read::Named;
use crate::rules::FILE;
use crate::{MergeStep, ShadowReason, merge_step};

/// The floor: every explicit version below 1.8.0 is rejected by vanilla's own schema.
const FLOOR: SemVersion = SemVersion::new(1, 8, 0);

/// The minimum that message names: the `v1_10` schema's.
const SCHEMA_MINIMUM: &str = "1.10.0";

/// The origin of a file of `pack`.
pub fn origin(pack: PackRef, side: PackSide, path: &PackPath) -> Origin {
    Origin::file(pack, side, path.clone())
}

/// A rejection with one finding.
pub fn reject(origin: &Origin, found: Diagnostic) -> Rejected {
    let diagnostics: Diagnostics = [found].into_iter().collect();
    Rejected::new(origin.clone(), place(diagnostics, origin))
}

/// Gives every finding the document's pack and file, and vanilla's texts the file's path.
pub fn place(mut found: Diagnostics, origin: &Origin) -> Diagnostics {
    for finding in found.iter_mut() {
        // The held item scale names the file, which the rule did not know.
        if let Some([file, scale]) = finding.code.args_of(&visual_codes::HELD_ITEM_SCALE_INVALID)
            && file == FILE
        {
            finding.code =
                visual_codes::HELD_ITEM_SCALE_INVALID.with([origin.path.as_str(), scale.as_str()]);
        }
    }
    // In place: the list keeps where the members of the upgraded document are in the file.
    found.located(origin)
}

/// The file's text when vanilla's loader would go on to the schema: UTF-8 within the nesting
/// limit, a root that is a non-empty object, and a `format_version` that is absent or within
/// 1.8.0 and the engine version. The rejections carry the loader's own texts, in its order.
/// `root` is the family's peek at the same bytes.
///
/// # Errors
///
/// Returns [`Rejected`] with the loader's line when the text does not go on to the schema.
pub fn admit<'b>(
    bytes: &'b [u8],
    root: Option<&Value>,
    origin: &Origin,
    cx: &LoadContext,
) -> Result<Cow<'b, str>, Rejected> {
    let malformed = |message: &str| reject(origin, Diagnostic::of(builders::syntax(message)));
    // `new` never refuses an empty text: that is the chain's to report.
    let text = Prepared::new(bytes)
        .map_err(|error| match error {
            PrepareError::NotUtf8(_) => malformed("the file is not UTF-8"),
            _ => malformed("the JSON nests too deeply"),
        })?
        .into_text();
    // A text that is no JSON is the chain's to report (a parse error, or an empty document).
    let Some(root) = root else {
        return Ok(text);
    };
    // A root that is null, no object or empty has nothing to parse.
    let Some(root) = root.as_object().filter(|root| !root.is_empty()) else {
        return Err(reject(
            origin,
            Diagnostic::of(&visual_codes::NOTHING_TO_PARSE).detail(DiagnosticDetail::Syntax(
                visual_codes::NOTHING_TO_PARSE.template().to_owned(),
            )),
        ));
    };
    if let Some(version) = root.get("format_version")
        && let Some(code) = version_gate(version, origin.path.as_str(), cx)
    {
        return Err(reject(
            origin,
            Diagnostic::error(code).at(JsonPath::of_keys(["format_version"])),
        ));
    }
    Ok(text)
}

/// `loadActorResourceDefinitionFormatVersion` and the schema's minimum, in vanilla's order: a
/// value that is no version (or the any-version, or the `"beta"` literal) is unknown; a
/// version above the engine exceeds the game version; below 1.0.0 it is unknown again; from
/// 1.0.0 up to 1.8.0 it is below the schema's minimum.
fn version_gate(version: &Value, path: &str, cx: &LoadContext) -> Option<Code> {
    // A value the game cannot read as a version, or that is the `beta` literal, is unknown.
    let unknown = || visual_codes::UNKNOWN_VERSION.with([path]);
    let literal = version.as_string().map(facet_value::VString::as_str);
    let Some(read) =
        SemVersion::from_json(version).filter(|read| !read.is_any() && literal != Some("beta"))
    else {
        return Some(unknown());
    };
    let written = literal.map_or_else(|| read.to_string(), str::to_owned);
    let engine = cx.engine_version;
    if read > SemVersion::new(engine.major, engine.minor, engine.patch) {
        return Some(visual_codes::ACTOR_ABOVE_ENGINE.with([
            written,
            format!("{}.{}.{}", engine.major, engine.minor, engine.patch),
        ]));
    }
    if read < SemVersion::new(1, 0, 0) {
        return Some(unknown());
    }
    if read < FLOOR {
        return Some(bedrock_addon_core::diagnostic::cereal::version_below_min(
            &written,
            SCHEMA_MINIMUM,
        ));
    }
    None
}

/// What loading a file gave: the identifier it names, in lower case, and the document or its
/// rejection.
pub type FileLoad<V> = (
    Option<<V as Visual>::Id>,
    Result<Document<<V as Visual>::Latest>, Rejected>,
);

/// Loads one file: the identifier it names, in lower case (when the payload has one), and the
/// document or its rejection. `root` is the family's peek at `bytes`.
pub fn parse_file<V: Visual>(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    root: Option<&Value>,
    cx: &LoadContext,
) -> FileLoad<V> {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let origin = origin(pack, side, path);
    let text = match admit(bytes, root, &origin, cx) {
        Ok(text) => text,
        Err(rejected) => return (None, Err(rejected)),
    };
    let parsed = parse_as_written::<V::Family>(JsonSlice::new(&text), cx);
    let identifier = parsed
        .identity
        .identifier
        .as_deref()
        .map(|id| V::Id::named(id.to_ascii_lowercase()));
    let outcome = finish(parsed, cx);
    let diagnostics = place(outcome.diagnostics, &origin);
    let result = match outcome.value {
        Some(Loaded::Stable(latest)) => {
            Ok(Document::new(latest, origin, outcome.version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    };
    (identifier, result)
}

/// Parses one file without upgrading it: the struct its `format_version` selects, after the
/// admit step. `root` is the family's peek at `bytes`.
///
/// # Errors
///
/// Returns [`Rejected`] when the file cannot be read into a value at all, such as bytes that
/// are not JSON or a document the family refuses; it carries every finding about the file.
pub fn parse_written<V: Visual>(
    path: &PackPath,
    bytes: &[u8],
    root: Option<&Value>,
    cx: &LoadContext,
) -> Result<Document<V::History>, Rejected> {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let origin = origin(PackRef::app_data(), PackSide::Resource, path);
    let text = admit(bytes, root, &origin, cx)?;
    let parsed = parse_as_written::<V::Family>(JsonSlice::new(&text), cx);
    let diagnostics = place(parsed.diagnostics, &origin);
    match parsed.value {
        Some(value) => Ok(Document::new(value, origin, parsed.version, diagnostics)),
        None => Err(Rejected::new(origin, diagnostics)),
    }
}

/// Runs the remaining upgrade steps of a document [`parse_written`] gave: rebuilds the parse
/// stage from the document and finishes it.
///
/// # Errors
///
/// Returns [`Rejected`] when an upgrade step or a check refuses the document; it carries every
/// finding about the file.
pub fn upgrade_document<V: Visual>(
    document: Document<V::History>,
    cx: &LoadContext,
) -> Result<Document<V::Latest>, Rejected> {
    let (value, origin, version, mut diagnostics) = document.into_parsed_parts();
    // The steps start from no finding, with the unknown members to move and take.
    let mut rows = Diagnostics::default();
    let unknown = diagnostics.take_unknown();
    if !unknown.is_empty() {
        *rows.unknown_mut() = unknown;
    }
    let identity = Identity {
        identifier: Some(V::identifier(&value).to_owned()),
        runtime_identifier: None,
    };
    let written = FormatVersionInput {
        version: Some(version.effective),
        beta_literal: false,
        prerelease: false,
    };
    let parsed =
        Parsed::<V::Family>::from_value(Some(value), version, written, JsonPath::new(), rows)
            .with_identity(identity);
    let outcome = finish(parsed, cx);
    let mut found = diagnostics;
    found.extend_with_unknown(place(outcome.diagnostics, &origin));
    match outcome.value {
        Some(Loaded::Stable(latest)) => Ok(Document::new(latest, origin, outcome.version, found)),
        _ => Err(Rejected::new(origin, found)),
    }
}

/// Where this loader refuses `text`, the text of a written document, read as the engine's own
/// content without a Molang compile; `None` when it loads. `root` is the family's peek at
/// `text`; `path` stands for the file.
pub fn refused_at<V: Visual>(
    text: &[u8],
    root: Option<&Value>,
    path: &'static str,
) -> Option<JsonPath> {
    let cx = LoadContext::detached();
    let path = PackPath::from_static(path);
    let outcome = parse_file::<V>(
        PackRef::app_data(),
        PackSide::Resource,
        &path,
        text,
        root,
        &cx,
    );
    let rejected = outcome.1.err()?;
    let at = rejected
        .diagnostics
        .iter()
        .find(|found| found.severity == Severity::Error)
        .map(|found| found.location.json_path.clone());
    Some(at.unwrap_or_default())
}

/// A value built in code may hold what no file can: an empty identifier, a slot name outside its
/// pattern, a version above the engine's. The document is written only when this loader reads it
/// back, else refused with the path of what it would not read. Whether its Molang compiles is
/// not asked: that depends on the pack. `refused_at` is [`refused_at`] of the family's peek.
///
/// # Errors
///
/// Returns [`EncodeError`] when the document has no JSON form or the loader would refuse it.
pub fn refuse_write<V: Visual>(
    latest: &V::Latest,
    refused_at: impl FnOnce(&[u8]) -> Option<JsonPath>,
) -> Result<(), EncodeError> {
    let mut text = Vec::new();
    write_json(
        &VersionStruct::write(latest)?,
        &mut text,
        WriteStyle::default(),
    )?;
    match refused_at(&text) {
        Some(at) => Err(EncodeError::NotRepresentable {
            at,
            what: Unrepresentable::Unsupported,
        }),
        None => Ok(()),
    }
}

/// What the shared identifier map does with `incoming` offered for an identifier whose
/// definition is `existing`, in a stack whose highest pack `min_engine_version` is `full_stack`:
/// merged, replacing it, or ignored with the crate's own information (vanilla is silent).
pub fn decide<V: Visual>(
    full_stack: &SemVersion,
    existing: Option<&Document<V::Latest>>,
    incoming: &Document<V::Latest>,
    d: &mut Diagnostics,
) -> EngineVersionDecision {
    let existing = existing.map(|existing| V::min_engine_version(existing.value()));
    match merge_step(
        full_stack,
        &V::min_engine_version(incoming.value()),
        existing.as_ref(),
    ) {
        MergeStep::Merge => EngineVersionDecision::Merge,
        MergeStep::Ignore(reason) => {
            // The reason is in the finding's text.
            let code = match reason {
                ShadowReason::OlderMinEngineVersion => V::SHADOWED.older,
                ShadowReason::MinEngineVersionAboveStack => V::SHADOWED.above_stack,
                _ => V::SHADOWED.other,
            };
            d.push(
                Diagnostic::of(code)
                    .detail(DiagnosticDetail::Shadow(reason))
                    .phase(Phase::Validate)
                    .in_file(incoming.origin()),
            );
            EngineVersionDecision::Ignore
        }
        // `Create`, `Replace`, and any step a newer core adds.
        _ => EngineVersionDecision::Replace,
    }
}
