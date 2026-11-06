//! Feature documents as written in older format versions, and the steps that upgrade them to
//! [`Feature`].
//!
//! Three feature types changed their schema: `scatter_feature` at 1.21.10 (the scatter
//! parameters moved into `distribution`), `single_block_feature` at 1.21.40 (weighted
//! `places_block`, `randomize_rotation`, `may_not_attach_to`, `diagonal`) and
//! `snap_to_surface_feature` at 1.26.50 (`vertical_search_range` renamed `search_range`).
//! [`v1_13_0`] has their older structs, each with its `Upgrade` impl. Every other type has one
//! schema from 1.13.0 (`multi_block_feature` from 1.26.40), so its documents parse as the
//! newest struct and run no step. Below its floor a document is not loaded. Features have no
//! beta struct.

pub mod v1_13_0;

use super::Feature;
use super::upgrade_document;
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::id::FeatureId;
use bedrock_addon_core::json::EncodeError;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{VersionStruct, Versioned};

/// A feature document parsed by the struct its `format_version` selects, not upgraded.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
#[allow(
    clippy::large_enum_variant,
    reason = "the variants are the types callers match on; a document is built once and stored"
)]
pub enum FeatureHistory {
    /// A `minecraft:scatter_feature` below format 1.21.10.
    ScatterV1_13_0(v1_13_0::ScatterFeature),
    /// A `minecraft:single_block_feature` below format 1.21.40.
    SingleBlockV1_13_0(v1_13_0::SingleBlockFeature),
    /// A `minecraft:snap_to_surface_feature` below format 1.26.50.
    SnapToSurfaceV1_13_0(v1_13_0::SnapToSurfaceFeature),
    /// A document its type's newest struct parsed.
    Latest(Feature),
}

impl FeatureHistory {
    /// The payload key of the document's feature type.
    pub fn type_key(&self) -> &'static str {
        match self {
            Self::ScatterV1_13_0(_) => "minecraft:scatter_feature",
            Self::SingleBlockV1_13_0(_) => "minecraft:single_block_feature",
            Self::SnapToSurfaceV1_13_0(_) => "minecraft:snap_to_surface_feature",
            Self::Latest(feature) => feature.type_key(),
        }
    }

    /// The feature's identifier (`description.identifier`).
    pub fn identifier(&self) -> &FeatureId {
        match self {
            Self::ScatterV1_13_0(value) => &value.description.identifier,
            Self::SingleBlockV1_13_0(value) => &value.description.identifier,
            Self::SnapToSurfaceV1_13_0(value) => &value.description.identifier,
            Self::Latest(feature) => feature.identifier(),
        }
    }

    /// The version of the struct that parsed the document.
    pub fn parsed_version(&self) -> FormatVersion {
        match self {
            Self::ScatterV1_13_0(_)
            | Self::SingleBlockV1_13_0(_)
            | Self::SnapToSurfaceV1_13_0(_) => v1_13_0::VERSION,
            Self::Latest(feature) => feature.dispatch(LatestVersion),
        }
    }

    /// Writes the document in its struct's own version (a round trip without the upgrade).
    ///
    /// # Errors
    ///
    /// Returns an [`EncodeError`] when a value cannot be written as JSON.
    pub fn to_json(&self) -> Result<String, EncodeError> {
        let payload = match self {
            Self::ScatterV1_13_0(value) => value.write()?,
            Self::SingleBlockV1_13_0(value) => value.write()?,
            Self::SnapToSurfaceV1_13_0(value) => value.write()?,
            Self::Latest(feature) => return feature.to_json(),
        };
        let mut root = facet_value::VObject::new();
        root.insert(
            "format_version",
            bedrock_addon_core::json::Value::from(v1_13_0::VERSION.to_string()),
        );
        root.insert(self.type_key(), payload);
        bedrock_addon_core::json::to_json(&bedrock_addon_core::json::Value::from(root))
    }
}

/// The version of a newest struct.
struct LatestVersion;

impl super::FeatureFn<FormatVersion> for LatestVersion {
    fn call<T: super::FeatureType>(self, _: &T) -> FormatVersion {
        <T as Versioned>::VERSION
    }
}

/// Runs the remaining upgrade steps of a document [`Feature::parse_as_written`] gave: the same
/// result [`Feature::parse`](bedrock_addon_core::document::Content::parse) gives for its file.
///
/// # Errors
///
/// Returns [`Rejected`] when an upgrade step or a check refuses the document; it carries every
/// finding about the file.
pub fn upgrade(
    document: Document<FeatureHistory>,
    cx: &LoadContext,
) -> Result<Document<Feature>, Rejected> {
    upgrade_document(document, cx)
}
