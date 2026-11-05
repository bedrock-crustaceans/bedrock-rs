//! One file of a family as a [`Document`]: the bytes through [`Prepared`], the pipeline, the
//! family's close and the file's origin on every finding.

use std::fmt;

use super::{Family, Loaded, finish, parse_as_written, parse_file_as_written};
use crate::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, VanillaText};
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

/// What reading one file of family `F` gives ([`parse_document`],
/// [`JsonDocument::read`](crate::document::JsonDocument::read)): the newest stable struct, the
/// beta struct, or the rejection.
///
/// The stable document and the rejection are closed ([`Family::close`], once). The beta document
/// is not closed yet ([`OpenBeta`]): whether it failed is the taker's to say, a registry keeping
/// it ([`OpenBeta::keep`]) and the stable type's own `Content::parse` refusing it
/// ([`OpenBeta::refuse`]).
pub enum DocumentRead<F: Family> {
    /// The file upgraded to the newest stable struct and validated.
    Stable(Document<F::Latest>),
    /// A beta document (`"format_version": "beta"`) the load allowed, with the identifier the
    /// payload names (`Family::identity`), when it is a string.
    Beta(Option<String>, OpenBeta<F>),
    /// The rejection, with the identifier the payload names when the file got as far as its
    /// envelope and version (`None` for a file that is no text or whose version is unreadable).
    Rejected(Option<String>, Rejected),
}

impl<F: Family> DocumentRead<F> {
    /// The stable document or the rejection: a beta document is refused
    /// ([`OpenBeta::refuse`]), as a file kind with no beta struct of its own refuses it.
    ///
    /// # Errors
    ///
    /// `Rejected` for a rejected file or a beta one; it carries the located findings.
    pub fn stable(self) -> Result<Document<F::Latest>, Rejected> {
        match self {
            Self::Stable(document) => Ok(document),
            Self::Beta(_, beta) => Err(beta.refuse()),
            Self::Rejected(_, rejected) => Err(rejected),
        }
    }
}

/// Prints the variant and the document's origin, without a `Debug` bound on the structs.
impl<F: Family> fmt::Debug for DocumentRead<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stable(document) => f.debug_tuple("Stable").field(document.origin()).finish(),
            Self::Beta(id, beta) => f
                .debug_tuple("Beta")
                .field(id)
                .field(beta.document.origin())
                .finish(),
            Self::Rejected(id, rejected) => f
                .debug_tuple("Rejected")
                .field(id)
                .field(&rejected.origin)
                .finish(),
        }
    }
}

/// A beta document read and not yet closed: [`keep`](Self::keep) closes it as loaded,
/// [`refuse`](Self::refuse) and [`reject`](Self::reject) as failed. Either closes it once; the file's text is gone by then,
/// so the close is told no text (`document: None`).
pub struct OpenBeta<F: Family> {
    document: Document<F::Beta>,
}

impl<F: Family> OpenBeta<F> {
    /// A beta document a family read its own way (a `JsonDocument::read` that does not wrap
    /// [`parse_document`]), its findings located and not yet closed: the taker closes it, as
    /// it closes one [`parse_document`] gave.
    pub fn new(document: Document<F::Beta>) -> Self {
        Self { document }
    }

    /// The document, as read.
    pub fn document(&self) -> &Document<F::Beta> {
        &self.document
    }

    /// Adds a finding before the close (a family's `read` wrapping the default), put in the
    /// document's file.
    pub fn push(&mut self, finding: Diagnostic) {
        let origin = self.document.origin().clone();
        let diagnostics = self.document.diagnostics_mut();
        diagnostics.push(finding);
        diagnostics.locate(&origin);
    }

    /// The document kept (a registry's beta contribution), closed as loaded.
    pub fn keep(mut self) -> Document<F::Beta> {
        self.close(false);
        self.document
    }

    /// The document refused, as a file kind with no beta struct refuses it: "no parser
    /// available" for the version `beta` ([`beta_refused`]), then the close as failed.
    pub fn refuse(mut self) -> Rejected {
        self.document.diagnostics_mut().push(beta_refused());
        self.reject()
    }

    /// The document rejected for a reason the taker told first ([`push`](Self::push)), closed
    /// as failed.
    pub fn reject(mut self) -> Rejected {
        self.close(true);
        let (_, origin, _, diagnostics, _) = self.document.into_parts();
        Rejected::located(origin, diagnostics)
    }

    fn close(&mut self, failed: bool) {
        let origin = self.document.origin().clone();
        let diagnostics = self.document.diagnostics_mut();
        F::close(diagnostics, None, &origin.path, failed);
        diagnostics.locate(&origin);
    }
}

/// The finding of a beta document read by a file kind with no beta struct: the game's
/// versioned loader has no parser for the version `beta` (`VanillaText::NoParserForVersion`).
pub fn beta_refused() -> Diagnostic {
    Diagnostic::error(DiagnosticCode::UnsupportedVersion {
        text: Some(VanillaText::NoParserForVersion {
            version: "beta".to_owned(),
        }),
    })
}

/// Loads the file at `origin` of family `F`, upgraded to the newest struct and validated: a
/// stable document or a rejection closed once ([`Family::close`], told whether the file
/// failed), or a beta document left open for its taker to close ([`DocumentRead`]). Every
/// finding is put in the file's pack and path.
pub fn parse_document<F: Family>(
    origin: Origin,
    bytes: &[u8],
    cx: &LoadContext,
) -> DocumentRead<F> {
    // The BOM is blanked, not cut, so every span is an offset into the file as written.
    let Ok(prepared) = Prepared::new(bytes) else {
        return DocumentRead::Rejected(None, unreadable::<F>(origin));
    };
    let document = prepared.slice();
    let parsed = parse_as_written::<F>(document, cx);
    let identifier = parsed.identity.identifier.clone();
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    match outcome.value {
        Some(Loaded::Stable(value)) => {
            F::close(&mut diagnostics, Some(document), &origin.path, false);
            diagnostics.locate(&origin);
            DocumentRead::Stable(Document::new(value, origin, outcome.version, diagnostics))
        }
        Some(Loaded::Beta(value)) => {
            diagnostics.locate(&origin);
            DocumentRead::Beta(
                identifier,
                OpenBeta {
                    document: Document::new(value, origin, outcome.version, diagnostics),
                },
            )
        }
        None => {
            F::close(&mut diagnostics, Some(document), &origin.path, true);
            DocumentRead::Rejected(identifier, Rejected::located(origin, diagnostics))
        }
    }
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
