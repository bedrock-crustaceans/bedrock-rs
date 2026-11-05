//! The metadata registry at `[1.21.50, 1.21.80)`,
//! and its step to 1.21.80.

use indexmap::IndexMap;

pub use super::super::JigsawBlockMetadata;

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, JsonPath, Phase};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{StrictBool, StrictInt};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx, Versioned};

/// The metadata registry at 1.21.50 (JSON `minecraft:jigsaw_structure_metadata`).
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct MetadataRegistry {
    /// The metadata of each structure element, by its metadata key in decimal (JSON
    /// `metadata`).
    pub metadata: IndexMap<String, StructureMetadata>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The metadata of one generated structure template at 1.21.50.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct StructureMetadata {
    /// The template's bounding size (JSON `size`).
    pub size: [StrictInt<i32>; 3],
    /// Unused; it selected the legacy size rotation (JSON `use_legacy_rotation`,
    /// default `false`). Removed at 1.21.80.
    #[facet(default)]
    pub use_legacy_rotation: Option<StrictBool>,
    /// The jigsaw blocks in the template (JSON `markers`).
    pub markers: Vec<JigsawBlockMetadata>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
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
                let mut at = JsonPath::new();
                at.push("minecraft:jigsaw_structure_metadata");
                at.push("metadata");
                at.push(&key);
                at.push("use_legacy_rotation");
                cx.diagnostics.push(
                    Diagnostic::info(DiagnosticCode::DroppedField { text: None })
                        .at(at)
                        .phase(Phase::Upgrade),
                );
            }
            metadata.insert(
                key,
                super::super::StructureMetadata {
                    size: entry.size,
                    markers: entry.markers,
                    extra: entry.extra,
                },
            );
        }
        super::super::MetadataRegistry {
            metadata,
            extra: self.extra,
        }
    }
}
