//! `Manifest::parse`: the text, the reader the `format_version` selects, and the document.

use crate::diagnostic::{
    DiagnosticCode as Code, JsonPath, Span, VanillaText, code_for_deserialize_error_kind,
};
use crate::document::{Document, Origin, Rejected, VersionInfo};
use crate::json::{MAX_JSON_DEPTH, Value, check_depth, strip_bom};
use crate::pack_identity::{PackPath, PackRef, PackSide, PackType};
use crate::version::{FormatVersion, FormatVersionInput, PickedVersion};

use super::capability::CapabilityRegistry;
use super::read::Sink;
use super::{Manifest, ManifestFormat, comprehensive, legacy};

/// The version from which `format_version` selects the comprehensive reader.
const COMPREHENSIVE: FormatVersion = FormatVersion::new(3, 0, 0);

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
    let file = &*crate::json::blank_bom(file);
    let bytes = strip_bom(file);
    if check_depth(bytes, MAX_JSON_DEPTH).is_err() {
        sink.error(
            Code::MalformedInput {
                message: "JSON nesting deeper than the limit".to_owned(),
                text: None,
            },
            &JsonPath::new(),
        );
        return None;
    }
    // The entry point reads JSON with comments; a syntax error is `MalformedInput` at its
    // source position, and nothing is read. Bytes that are not UTF-8 are the parser's to
    // report.
    let parsed = match std::str::from_utf8(bytes) {
        Ok(text) => crate::json::from_str_jsonc(&crate::json::last_wins(text)),
        Err(_) => crate::json::from_slice_jsonc(bytes),
    };
    let root: Value = match parsed {
        Ok(root) => root,
        Err(error) => {
            let blank = bytes.iter().all(u8::is_ascii_whitespace);
            let code = code_for_deserialize_error_kind(&error.kind, false, blank);
            sink.error_in_text(code, file_span(file, bytes, error.span));
            return None;
        }
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
                Code::MalformedFormatVersion { text: None },
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
                        Code::UnsupportedVersion {
                            text: Some(VanillaText::NoParserForVersion { version: shown }),
                        },
                        &JsonPath::of_keys(["format_version"]),
                    );
                    return None;
                }
            }
        }
    };
    if let Some(routed) = routed {
        comprehensive::read(&root, capabilities, sink).map(|manifest| (manifest, Some(routed)))
    } else {
        legacy::read(&root, capabilities, sink).map(|manifest| (manifest, None))
    }
}

/// A span of the BOM-stripped `text` as a range of `file`; the whole file when there is none.
fn file_span(file: &[u8], text: &[u8], span: Option<facet::Span>) -> Span {
    let to_u32 = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    let bom = to_u32(file.len() - text.len());
    span.map_or(
        Span {
            offset: 0,
            len: to_u32(file.len()),
        },
        |span| Span {
            offset: bom.saturating_add(span.offset),
            len: span.len,
        },
    )
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
