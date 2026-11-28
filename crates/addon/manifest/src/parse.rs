//! `Manifest::parse`: the text, the reader the `format_version` selects, and the document.

use bedrock_addon_core::diagnostic::JsonPathSegment;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, JsonPath, Phase, Severity, Span};
use bedrock_addon_core::document::{Document, Origin, Rejected, VersionInfo};
use bedrock_addon_core::json::kit::{Dialect, Dialects, Node, Profile, Texts, WrongAt};
use bedrock_addon_core::json::parse::{self, Options};
use bedrock_addon_core::json::{MAX_JSON_DEPTH, Value, Winner, check_depth, strip_bom};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide, PackType};
use bedrock_addon_core::schema::{FromJson, read_document};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, PickedVersion};
use bedrock_addon_core::versioning::{MemberFailure, NullHandling, Reading};

use super::capability::CapabilityRegistry;
use super::read::{REGISTRY, Sink};
use super::{Manifest, ManifestFormat, comprehensive, legacy};

/// The version from which `format_version` selects the comprehensive reader.
const COMPREHENSIVE: FormatVersion = FormatVersion::new(3, 0, 0);

/// The game's readers of a manifest by the format they read from, each read strictly, every
/// failure an error and the member left out (the document is refused, every failure told), each
/// member in the order the reader asks for it:
/// - the legacy parser (formats 0 to 2, over a jsoncpp tree): a member written `null` is
///   absent, unknown members are kept for write-back in silence (each struct says `silent`), and
///   the codes are the pack report's (`WrongTypeProperty`, `MissingProperty`);
/// - the comprehensive document (format 3): `null` is a value of the wrong type, its unknown
///   members are refused before an object's members, and the codes are the schema's.
const DIALECTS: Dialects = Dialects(&[
    (
        FormatVersion::new(0, 0, 0),
        Dialect {
            reading: Reading::OWN
                .strict()
                .with_null(NullHandling::AbsentSilently)
                .with_on_member_failure(MemberFailure::DefaultAndRefuse),
            profile: Profile::DECLARED_ORDER,
            texts: Texts {
                wrong_at: Some(WrongAt(wrong_type_property)),
                missing_code: Some(&core_codes::MISSING_PROPERTY),
                ..Texts::CODES
            },
        },
    ),
    (
        COMPREHENSIVE,
        Dialect {
            reading: Reading::DOCUMENT
                .strict()
                .with_on_member_failure(MemberFailure::DefaultAndRefuse),
            profile: Profile::DECLARED_ORDER,
            texts: Texts::CODES,
        },
    ),
]);

/// The legacy parser's code of a member of the wrong JSON type.
fn wrong_type_property(_: &JsonPath) -> Code {
    Code::from(&core_codes::WRONG_TYPE_PROPERTY)
}

pub(super) fn parse(
    path: &PackPath,
    bytes: &[u8],
    capabilities: &CapabilityRegistry,
) -> Result<Document<Manifest>, Rejected> {
    let mut sink = Sink::new(path);
    let read = read(bytes, capabilities, &mut sink);
    let diagnostics = sink.diagnostics;
    match read {
        Some((manifest, routed)) if !diagnostics.has_errors() => {
            let origin = origin(path, Some(&manifest));
            let version = version_info(&manifest, routed);
            Ok(Document::new(manifest, origin, version, diagnostics))
        }
        read => Err(Rejected::new(
            origin(path, read.as_ref().map(|(manifest, _)| manifest)),
            diagnostics,
        )),
    }
}

/// The manifest and, for the comprehensive format, the version that routed it.
fn read(
    bytes: &[u8],
    capabilities: &CapabilityRegistry,
    sink: &mut Sink<'_>,
) -> Option<(Manifest, Option<FormatVersion>)> {
    let file = bytes;
    let file = &*bedrock_addon_core::json::blank_bom(file);
    let bytes = strip_bom(file);
    if check_depth(bytes, MAX_JSON_DEPTH).is_err() {
        sink.error(
            bedrock_addon_core::diagnostic::builders::syntax("JSON nesting deeper than the limit"),
            &JsonPath::new(),
        );
        return None;
    }
    // The entry point reads JSON with comments; a syntax error is `MalformedInput` at its
    // source position, and nothing is read. Bytes that are not UTF-8 are the parser's to
    // report.
    // A member written twice is read from its last copy, at that copy's place.
    let parsed = match std::str::from_utf8(bytes) {
        Ok(text) => parse::parse(text, Options::winner(Winner::Last)).map(Text::Spanned),
        Err(_) => parse::parse_bytes(bytes, Options::default()).map(Text::Value),
    };
    let parsed = match parsed {
        Ok(parsed) => parsed,
        Err(fault) => {
            let blank = bytes.iter().all(u8::is_ascii_whitespace);
            sink.error_in_text(fault.code(blank), file_span(file, bytes, fault.span));
            return None;
        }
    };
    let root = match &parsed {
        Text::Spanned(parsed) => Node::parsed(parsed, JsonPath::new()),
        Text::Value(value) => Node::new(value, JsonPath::new()),
    };
    // The game reads the manifest through a document input, which takes the version from
    // the first textual `"format_version"` (a number is read as its text). A value that gives
    // no version is "Malformed format version" (code 0x10) and no slice runs; an absent one
    // is the caller's 0.0.0, the legacy slice. The legacy slice spans [0.0.0, 3.0.0), the
    // comprehensive one exactly 3.0.0.
    let picked = std::str::from_utf8(bytes).map_or(PickedVersion::Absent, FormatVersionInput::pick);
    let routed = match picked {
        PickedVersion::Absent => None,
        PickedVersion::Malformed => {
            sink.error(
                Code::from(&core_codes::MALFORMED_FORMAT_VERSION),
                &JsonPath::of_keys(["format_version"]),
            );
            return None;
        }
        PickedVersion::Version { input, written } => {
            // The comprehensive slice is [3.0.0, 3.0.0]: the loader ends the sequence at the
            // larger of 3.0.0 and the engine version, and there is no beta slice; anything
            // later has no parser.
            match input.version {
                Some(version) if version < COMPREHENSIVE => None,
                Some(version) if version == COMPREHENSIVE && input.prerelease => None,
                Some(version) if version == COMPREHENSIVE => Some(version),
                _ => {
                    let shown = match input.version {
                        Some(version) if !input.prerelease => version.to_string(),
                        _ if input.beta_literal => "beta".to_owned(),
                        _ => written,
                    };
                    sink.error(
                        core_codes::NO_PARSER_FOR_VERSION.with([shown.clone()]),
                        &JsonPath::of_keys(["format_version"]),
                    );
                    return None;
                }
            }
        }
    };
    let bom = file.len() - bytes.len();
    let read = Reader {
        root: &root,
        bom,
        capabilities,
    };
    if let Some(routed) = routed {
        let document = read.document::<comprehensive::Document>(routed, sink);
        return document
            .filter(|_| !sink.has_errors())
            .map(|document| (comprehensive::manifest(document), Some(routed)));
    }
    legacy(read, sink)
}

/// Reads a legacy manifest: the format its `format_version` reads as (a format-0 one told
/// deprecated and, once read, upgraded), then the document of that format.
fn legacy(read: Reader<'_, '_>, sink: &mut Sink<'_>) -> Option<(Manifest, Option<FormatVersion>)> {
    let root = read.root;
    let format = legacy::format_version(root.value(), sink)?;
    // The legacy parser asks a jsoncpp value of any type for its members, and one that is no
    // object has none.
    let empty = Value::from(facet_value::VObject::new());
    let no_members = Node::new(&empty, JsonPath::new());
    let read = if root.value().as_object().is_some() {
        read
    } else {
        Reader {
            root: &no_members,
            ..read
        }
    };
    let at = JsonPath::of_keys(["format_version"]);
    let version = FormatVersion::new(u16::from(format), 0, 0);
    if format > 0 {
        return read
            .document::<legacy::Document12>(version, sink)
            .and_then(|document| legacy::manifest(document, format))
            .map(|manifest| (manifest, None));
    }
    // The game logs "Pack Manifest version 0 is deprecated, please include format_version 2
    // in your manifest.json." for a format-0 manifest at content-log level 2, area 36: a
    // warning.
    sink.push(
        core_codes::DEPRECATED_MEMBER_WORDED.with(["Pack Manifest version 0 is deprecated, please include format_version 2 in your manifest.json."]),
        Severity::Warning,
        Phase::Parse,
        &at,
    );
    let manifest = read
        .document::<legacy::Document0>(version, sink)
        .and_then(legacy::manifest_v0)?;
    // The game upgrades a format-0 manifest to format 1, and so it is held and written:
    // `pack_id` as `uuid`, `packs_version` as `version`, the header's modules and metadata at
    // the root. The written text is another text, which the load says at the root (the crate's
    // own information; the game upgrades in silence).
    sink.push(
        Code::from(&core_codes::UNUSUAL_FORMAT_VERSION),
        Severity::Info,
        Phase::Upgrade,
        &JsonPath::new(),
    );
    Some((manifest, None))
}

/// The text read: with the spans of its parts, or, for bytes that are not UTF-8, the value
/// alone.
enum Text<'t> {
    Spanned(parse::Parsed<'t>),
    Value(Value),
}

/// A read of the document at `root`: the file has a BOM of `bom` bytes; the manifest's
/// capabilities are checked against `capabilities`.
struct Reader<'r, 'v> {
    root: &'r Node<'v>,
    bom: usize,
    capabilities: &'r CapabilityRegistry,
}

impl Reader<'_, '_> {
    /// Reads the document as `T` in the dialect of `format`, the capability registry in scope,
    /// its findings at the file's bytes; `None` when it is not read.
    fn document<T: FromJson>(&self, format: FormatVersion, sink: &mut Sink<'_>) -> Option<T> {
        let told = sink.diagnostics.len();
        let mut read = DIALECTS.read(format, &mut sink.diagnostics)?;
        let document = read.scoped(REGISTRY, self.capabilities.clone(), |read| {
            read_document::<T>(self.root, read)
        });
        drop(read);
        let path = sink.path().clone();
        let bom = u32::try_from(self.bom).unwrap_or(u32::MAX);
        for finding in sink.diagnostics.iter_mut().skip(told) {
            finding.location.json_path = joined(&finding.location.json_path);
            finding.location.path = Some(path.clone());
            finding.location.span = finding.location.span.map(|span| Span {
                offset: span.offset.saturating_add(bom),
                len: span.len,
            });
        }
        document
    }
}

/// `path` in the segments the manifest readers have always given a member: a key that reads as
/// an unsigned number (`"0"`, `"+5"`) is an array index, and the empty key adds no segment.
// The two edges are the readers' paths as they were when built from `/`-joined text, pinned in
// `tests/characterize_manifest.rs`; they show only for a member of the file so named.
fn joined(path: &JsonPath) -> JsonPath {
    path.segments()
        .fold(JsonPath::new(), |joined, segment| match segment {
            JsonPathSegment::Key("") => joined,
            JsonPathSegment::Key(key) => match key.parse::<usize>() {
                Ok(index) => joined.index(index),
                Err(_) => joined.child(key),
            },
            JsonPathSegment::Index(index) => joined.index(index),
        })
}

/// A span of the BOM-stripped `text` as a range of `file`; the whole file when there is none.
fn file_span(file: &[u8], text: &[u8], span: Span) -> Span {
    let bom = u32::try_from(file.len() - text.len()).unwrap_or(u32::MAX);
    Span {
        offset: bom.saturating_add(span.offset),
        len: span.len,
    }
}

/// Where the manifest came from: the file, and a pack handle of the identity it declares (the
/// app-data handle when it declares none). The loader that places the pack on a stack gives it
/// its own handle.
fn origin(path: &PackPath, manifest: Option<&Manifest>) -> Origin {
    let pack = manifest.map_or_else(PackRef::app_data, |manifest| {
        PackRef::new(0, manifest.identity(), manifest.header.display_name())
    });
    let side = match manifest.map(Manifest::pack_type) {
        Some(PackType::Behavior) => PackSide::Behavior,
        _ => PackSide::Resource,
    };
    Origin::new(pack, side, path.clone(), None, Vec::new())
}

/// The format as written and the model it was read into: a legacy manifest is upgraded to the
/// comprehensive model.
fn version_info(manifest: &Manifest, routed: Option<FormatVersion>) -> VersionInfo {
    let number = |format: ManifestFormat| FormatVersion::new(u16::from(format.number()), 0, 0);
    let written = routed.unwrap_or_else(|| number(manifest.original_format));
    VersionInfo::new(
        Some(written),
        false,
        written,
        number(manifest.format),
        manifest.original_format != ManifestFormat::V3,
    )
}
