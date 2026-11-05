//! One file of a family as a [`Document`]: the bytes through [`Prepared`], the pipeline, the
//! family's close and the file's origin on every finding. The generic body of the per-family
//! `parse_file` copies (`voxel_shape`, `block_culling`, `dimension`, `feature_rules`,
//! `crafting_catalog`, `server_sound_definitions`, `particle`, support's `chain_file`).

use super::{Family, Loaded, finish_file, parse_as_written, parse_file_as_written};
use crate::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics};
use crate::document::{Document, Origin, Rejected};
use crate::json::Prepared;
use crate::load::LoadContext;
use crate::pack_identity::{PackPath, PackRef, PackSide};

/// The finding of an unreadable file: not UTF-8, or nested past the limit (one text for both,
/// so the order `Prepared` checks them in does not show).
fn malformed() -> Diagnostic {
    Diagnostic::error(DiagnosticCode::MalformedInput {
        message: "not UTF-8 JSON within the nesting limit".to_owned(),
        text: None,
    })
}

/// The rejection of a file at `origin` that is no text: the malformed finding, then the
/// family's close of a failed file, every finding in the file.
fn unreadable<F: Family>(origin: Origin) -> Rejected {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(malformed());
    F::close(&mut diagnostics, None, &origin.path, true);
    Rejected::located(origin, diagnostics)
}

/// Loads the file at `origin` of family `F`, upgraded to the newest struct and validated, and
/// closed once ([`Family::close`], told whether the file failed). Gives the identifier the
/// payload names (`Family::identity`), when it is a string, beside the document or its
/// rejection, so that a registry can name the identifier a failed file claimed.
///
/// A document of the beta struct is a rejection here: a family with a beta struct routes it
/// itself.
pub fn parse_document<F: Family>(
    origin: Origin,
    bytes: &[u8],
    cx: &LoadContext,
) -> (Option<String>, Result<Document<F::Latest>, Rejected>) {
    // The BOM is blanked, not cut, so every span is an offset into the file as written.
    let Ok(prepared) = Prepared::new(bytes) else {
        return (None, Err(unreadable::<F>(origin)));
    };
    let document = prepared.slice();
    let parsed = parse_as_written::<F>(document, cx);
    let identifier = parsed.identity.identifier.clone();
    let outcome = finish_file(parsed, Some(document), &origin.path, cx);
    let mut diagnostics = outcome.diagnostics;
    diagnostics.locate(&origin);
    let document = match outcome.value {
        Some(Loaded::Stable(value)) => {
            Ok(Document::new(value, origin, outcome.version, diagnostics))
        }
        Some(Loaded::Beta(_)) | None => Err(Rejected::new(origin, diagnostics)),
    };
    (identifier, document)
}

/// The struct the file's `format_version` selects, not upgraded, for the file `path` read on
/// its own (the app-data handle, on `side`): the parse stage of a family's document. The file
/// is closed once: a parse with no value as failed, a parse with one as not failed (the end of
/// the as-written path, which no `finish_file` follows).
///
/// # Errors
///
/// `Rejected` when the text is not readable JSON or the file does not parse into a value; it
/// carries the located findings.
pub fn parse_document_as_written<F: Family>(
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<F::Any>, Rejected> {
    let origin = Origin::file(PackRef::app_data(), side, path.clone());
    // The BOM is blanked, not cut, so every span is an offset into the file as written.
    let Ok(prepared) = Prepared::new(bytes) else {
        return Err(unreadable::<F>(origin));
    };
    let document = prepared.slice();
    let mut parsed = parse_file_as_written::<F>(document, path, cx);
    if parsed.value.is_some() {
        F::close(&mut parsed.diagnostics, Some(document), path, false);
    }
    let mut diagnostics = parsed.diagnostics;
    diagnostics.locate(&origin);
    match parsed.value {
        Some(value) => Ok(Document::new(value, origin, parsed.version, diagnostics)),
        None => Err(Rejected::new(origin, diagnostics)),
    }
}
