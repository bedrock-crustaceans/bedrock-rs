//! The metadata registry at `[1.21.50, 1.21.80)`,
//! and its step to 1.21.80.

use indexmap::IndexMap;

pub use super::super::JigsawBlockMetadata;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, JsonPath, Phase};
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx, Versioned};

/// The metadata registry at 1.21.50 (JSON `minecraft:jigsaw_structure_metadata`).
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MetadataRegistry {
    /// The metadata of each structure element, by its metadata key in decimal (JSON
    /// `metadata`).
    pub metadata: IndexMap<String, StructureMetadata>,
}

/// The metadata of one generated structure template at 1.21.50.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct StructureMetadata {
    /// The template's bounding size (JSON `size`).
    pub size: [i32; 3],
    /// Unused; it selected the legacy size rotation (JSON `use_legacy_rotation`,
    /// default `false`). Removed at 1.21.80.
    pub use_legacy_rotation: Option<bool>,
    /// The jigsaw blocks in the template (JSON `markers`).
    pub markers: Vec<JigsawBlockMetadata>,
}

impl Versioned for MetadataRegistry {
    const VERSION: FormatVersion = FormatVersion::new(1, 21, 50);
}

/// Every member is copied except `use_legacy_rotation`, which is dropped
/// (the game drops it without a message; the crate reports a `DroppedField` information where
/// it was written).
impl Upgrade for MetadataRegistry {
    type Next = super::super::MetadataRegistry;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut metadata = IndexMap::with_capacity(self.metadata.len());
        for (key, entry) in self.metadata {
            if entry.use_legacy_rotation.is_some() {
                let at = JsonPath::new()
                    .child("minecraft:jigsaw_structure_metadata")
                    .child("metadata")
                    .child(&key)
                    .child("use_legacy_rotation");
                cx.diagnostics.push(
                    Diagnostic::of(&core_codes::DROPPED_FIELD)
                        .at(at)
                        .phase(Phase::Upgrade),
                );
            }
            metadata.insert(
                key,
                super::super::StructureMetadata {
                    size: entry.size,
                    markers: entry.markers,
                },
            );
        }
        super::super::MetadataRegistry { metadata }
    }
}
