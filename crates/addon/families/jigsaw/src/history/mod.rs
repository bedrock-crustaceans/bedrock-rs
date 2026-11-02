//! The jigsaw documents as written in older format versions, and their chains.
//!
//! The four JSON documents (`minecraft:jigsaw`, `minecraft:template_pool`,
//! `minecraft:structure_set`, `minecraft:processor_list`) have one struct each, at 1.21.20, read
//! from 1.21.20 up to the engine version; there is no older shape and no upgrade step. The
//! metadata registry has two: `v1_21_50` for `[1.21.50, 1.21.80)`, whose step drops
//! `use_legacy_rotation`, and [`super::MetadataRegistry`] from 1.21.80.

pub mod v1_21_50;

use super::{JigsawStructure, MetadataRegistry, ProcessorList, StructureSet, TemplatePool};

/// The `minecraft:jigsaw` family of the version-chain machinery (hidden: only the loaders
/// name it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct StructureFamily;

/// The `minecraft:template_pool` family.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct PoolFamily;

/// The `minecraft:structure_set` family.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct StructureSetFamily;

/// The `minecraft:processor_list` family.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct ProcessorListFamily;

/// The `minecraft:jigsaw_structure_metadata` family (the typed model of the `.dat` body).
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct MetadataFamily;

bedrock_addon_core::version_chain! {
    family = StructureFamily;
    /// A `minecraft:jigsaw` document parsed by the struct its `format_version` selects.
    #[derive(Clone, Debug, PartialEq)]
    pub enum StructureAny {
        /// Format 1.21.20 and later: the newest document.
        V1_21_20(JigsawStructure),
    }
    molang = none;
}

bedrock_addon_core::version_chain! {
    family = PoolFamily;
    /// A `minecraft:template_pool` document parsed by the struct its `format_version` selects.
    #[derive(Clone, Debug, PartialEq)]
    pub enum PoolAny {
        /// Format 1.21.20 and later: the newest document.
        V1_21_20(TemplatePool),
    }
    molang = none;
}

bedrock_addon_core::version_chain! {
    family = StructureSetFamily;
    /// A `minecraft:structure_set` document parsed by the struct its `format_version` selects.
    #[derive(Clone, Debug, PartialEq)]
    pub enum StructureSetAny {
        /// Format 1.21.20 and later: the newest document.
        V1_21_20(StructureSet),
    }
    molang = none;
}

bedrock_addon_core::version_chain! {
    family = ProcessorListFamily;
    /// A `minecraft:processor_list` document parsed by the struct its `format_version` selects.
    #[derive(Clone, Debug, PartialEq)]
    pub enum ProcessorListAny {
        /// Format 1.21.20 and later: the newest document.
        V1_21_20(ProcessorList),
    }
    molang = none;
}

bedrock_addon_core::version_chain! {
    family = MetadataFamily;
    /// A metadata registry parsed by the struct its version selects.
    #[derive(Clone, Debug, PartialEq)]
    pub enum MetadataAny {
        /// `[1.21.50, 1.21.80)`.
        V1_21_50(v1_21_50::MetadataRegistry),
        /// 1.21.80 and later: the newest registry.
        V1_21_80(MetadataRegistry),
    }
    molang = none;
}
