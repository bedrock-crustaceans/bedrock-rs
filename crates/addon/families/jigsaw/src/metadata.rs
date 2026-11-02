//! Jigsaw structure metadata: the binary `worldgen/metadata/*.dat` cache
//! (`minecraft:jigsaw_structure_metadata`), and its typed model.

use indexmap::IndexMap;

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{StrictInt, StrictString};

/// A jigsaw structure metadata file (`worldgen/metadata/*.dat`): a precomputed cache of the
/// size and jigsaw blocks of each structure template a pool element places.
///
/// The file is binary: a length-prefixed version string, then the registry in vanilla's binary
/// encoding. The version is checked as vanilla checks it; the body is kept as bytes and not
/// decoded, because vanilla computes the metadata of any element the cache lacks (the game
/// itself decodes the body and merges it by key, which this crate does not do). [`MetadataRegistry`] is the typed model of
/// the body. Written back byte for byte by [`to_bytes`](Self::to_bytes); the file has no JSON
/// form.
#[derive(Clone, PartialEq, Eq)]
#[doc(alias = "minecraft:jigsaw_structure_metadata")]
#[doc(alias = "JigsawStructureMetadataFile")]
pub struct JigsawMetadata {
    /// The leading version string, as written (`1.21.90`).
    pub format_version: String,
    /// The bytes after the version string: the registry, not decoded.
    pub body: Vec<u8>,
}

/// The version and the body's length: vanilla's bodies run to a megabyte.
impl std::fmt::Debug for JigsawMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JigsawMetadata")
            .field("format_version", &self.format_version)
            .field("body_len", &self.body.len())
            .finish()
    }
}

/// The typed body of a metadata file at 1.21.80 and later, the payload of
/// `minecraft:jigsaw_structure_metadata`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct MetadataRegistry {
    /// The metadata of each structure element, by the element's 64-bit metadata key written in
    /// decimal (JSON `metadata`).
    pub metadata: IndexMap<String, StructureMetadata>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The metadata of one generated structure template.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct StructureMetadata {
    /// The template's bounding size (JSON `size`).
    pub size: [StrictInt<i32>; 3],
    /// The jigsaw blocks in the template (JSON `markers`).
    pub markers: Vec<JigsawBlockMetadata>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One jigsaw block of a template.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
pub struct JigsawBlockMetadata {
    /// The name another jigsaw block's `target` refers to (JSON `name`).
    pub name: StrictString,
    /// The template pool to generate from (JSON `pool`).
    pub pool: StrictString,
    /// The name of the jigsaw block to attach to (JSON `target`).
    pub target: StrictString,
    /// The block's serialisation id before generation (JSON `block`).
    pub block: StrictString,
    /// The serialisation id the jigsaw block turns into (JSON `final_block`).
    pub final_block: StrictString,
    /// The joint made with `target` (JSON `joint_type`).
    pub joint_type: JointType,
    /// The block's position (JSON `pos`).
    pub pos: [StrictInt<i32>; 3],
    /// The priority in target selection (JSON `selection_priority`).
    pub selection_priority: StrictInt<i32>,
    /// The priority in placement (JSON `placement_priority`).
    pub placement_priority: StrictInt<i32>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The joint a jigsaw block makes with its target (`joint_type`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum JointType {
    /// JSON `rollable`.
    Rollable,
    /// JSON `aligned`.
    Aligned,
}
