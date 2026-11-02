//! The catalogue as read at `[1.21.60, 1.26.50)`: vanilla's first slice
//! (`v1_21_60::CraftingCatalogDocument`), read where an unknown member is only a warning, and its
//! step to the newest struct.
//!
//! The 1.26.50 struct binds the same member (`categories`, of the same element type
//! `v1_21_60::CraftingCatalogCategory`); the step is a plain copy of that member, which
//! renames, defaults, drops and logs nothing.

use super::load::{CatalogFamily, upgrade_document};
use super::parse::parse_catalog;
use super::{CraftingCatalog, V1_21_60};
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};

/// A catalogue document read by the 1.21.60 slice: the newest struct's members, read where an
/// unknown member is only a warning (and kept).
#[derive(Clone, Debug, PartialEq)]
pub struct CatalogV1_21_60(pub CraftingCatalog);

bedrock_addon_core::version_chain! {
    family = CatalogFamily;
    /// A catalogue document parsed by the struct its `format_version` selects, not upgraded.
    #[derive(Clone, Debug, PartialEq)]
    pub enum CatalogAny {
        /// Format 1.21.60 up to 1.26.50, the unstrict slice.
        V1_21_60(CatalogV1_21_60),
        /// Format 1.26.50 up to the engine, the strict slice.
        V1_26_50(CraftingCatalog),
    }
    molang = none;
}

impl CatalogAny {
    /// Runs the rest of the load on a document [`CraftingCatalog::parse_as_written`] gave: the
    /// same result [`Content::parse`](bedrock_addon_core::document::Content::parse) gives for its file.
    pub fn upgrade(
        document: Document<Self>,
        cx: &LoadContext,
    ) -> Result<Document<CraftingCatalog>, Rejected> {
        upgrade_document(document, cx)
    }
}

impl Versioned for CatalogV1_21_60 {
    const VERSION: FormatVersion = V1_21_60;
}

impl VersionStruct for CatalogV1_21_60 {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        parse_catalog(payload, cx, d).map(Self)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        Ok(self.0.payload())
    }
}

/// The plain copy: `categories` and what the reader kept.
impl Upgrade for CatalogV1_21_60 {
    type Next = CraftingCatalog;

    fn upgrade(self, _: &mut UpgradeCx<'_>) -> CraftingCatalog {
        self.0
    }
}
