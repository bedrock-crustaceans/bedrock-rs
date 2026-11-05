//! The dimension document as written in older format versions, and the step that upgrades it
//! to [`Dimension`].
//!
//! The chain is `v1_18_0 → v1_26_50`: `v1_18_0::Dimension` parses `[1.18.0, 1.26.50)` (vanilla
//! reads 1.18.0 to 1.21.60 with its legacy slice and the rest with its cereal slice, the same
//! struct both times) and its `Upgrade` impl turns the bounds into a height. Below 1.18.0 a
//! document is not loaded. Dimensions have no beta struct; feature rules have one struct and no
//! history.

pub mod v1_18_0;

use super::Dimension;
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::version::FormatVersionInput;
use bedrock_addon_core::versioning::{Identity, Loaded, Parsed, finish};

/// The dimension family of the version-chain machinery (hidden: only the loaders name it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DimensionFamily;

bedrock_addon_core::version_chain! {
    family = DimensionFamily;
    /// A dimension document parsed by the struct its `format_version` selects, not upgraded.
    #[derive(Clone, Debug, PartialEq)]
    pub enum Any {
        /// Format 1.18.0 up to 1.26.50.
        V1_18_0(v1_18_0::Dimension),
        /// Format 1.26.50 and later: the newest document.
        V1_26_50(Dimension),
    }
    molang = none;
}

/// Runs the remaining upgrade step of a document
/// [`Dimension::parse_as_written`](bedrock_addon_core::document::JsonDocument::parse_as_written)
/// gave: the same result [`Dimension::parse`](bedrock_addon_core::document::Content::parse)
/// gives for its file.
/// It works over the document because the document holds the origin.
///
/// # Errors
///
/// Returns [`Rejected`](bedrock_addon_core::document::Rejected) when the upgrade step, or the
/// checks that follow it, find an error that stops the document from loading.
pub fn upgrade(document: Document<Any>, cx: &LoadContext) -> Result<Document<Dimension>, Rejected> {
    upgrade_document(document, cx)
}

/// `history::upgrade`: rebuilds the parse stage from the document and finishes it.
pub(crate) fn upgrade_document(
    document: Document<Any>,
    cx: &LoadContext,
) -> Result<Document<Dimension>, Rejected> {
    let origin = document.origin().clone();
    let version = document.version().clone();
    let diagnostics = document.diagnostics().clone();
    let value = document.into_value();
    let identifier = match &value {
        Any::V1_18_0(d) => d.description.identifier.as_str().to_owned(),
        Any::V1_26_50(d) => d.description.identifier.as_str().to_owned(),
    };
    let written = FormatVersionInput {
        version: Some(version.effective),
        beta_literal: false,
        prerelease: false,
    };
    let parsed = Parsed::<DimensionFamily>::from_value(
        Some(value),
        version,
        written,
        JsonPath::new(),
        diagnostics,
    )
    .with_identity(Identity {
        identifier: Some(identifier),
        runtime_identifier: None,
    });
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    diagnostics.locate(&origin);
    match outcome.value {
        Some(Loaded::Stable(dimension)) => Ok(Document::new(
            dimension,
            origin,
            outcome.version,
            diagnostics,
        )),
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}
