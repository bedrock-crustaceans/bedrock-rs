//! Jigsaw structures (`Jigsaw`, `JigsawStructure`, `TemplatePool`, `StructureSet`,
//! `ProcessorList`, `JigsawMetadata`): the four JSON documents below `worldgen/` of a behaviour
//! pack (`minecraft:jigsaw`, `minecraft:template_pool`, `minecraft:structure_set`,
//! `minecraft:processor_list`, each one struct at format 1.21.20) and the binary metadata cache
//! `worldgen/metadata/*.dat`, loaded together as the `Jigsaw` group.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;
#[doc(hidden)]
pub use link::{Targets, link};
pub use metadata::{
    JigsawBlockMetadata, JigsawMetadata, JointType, MetadataRegistry, StructureMetadata,
};
pub use pool::{PoolDescription, PoolElement, Projection, TemplatePool, TemplatePoolElement};
pub use processor::{
    Axis, BlockEntityModifier, BlockPredicate, CappedDelegate, IntProvider, IntProviderType,
    PositionPredicate, Processor, ProcessorList, ProcessorListDescription, ProcessorRule,
};
pub use set::{
    PlacementType, RandomSpreadPlacement, SpreadType, StructureSet, StructureSetDescription,
    StructureSetEntry,
};
pub use structure::{
    DimensionPadding, GenerationStep, GroupedPoolAlias, HeightProvider, HeightmapProjection,
    JigsawStructure, LiquidSettings, MaxDistanceFromCenter, PoolAlias, StructureDescription,
    TerrainAdaptation, VerticalAnchor, Weighted,
};

mod link;
mod load;
mod metadata;
mod new;
mod parse;
mod pool;
mod processor;
mod set;
mod structure;
mod validate;

#[cfg(test)]
mod tests;

#[doc(hidden)]
pub use load::load_group;

use bedrock_addon_core::document::{Files, Registry};

/// The jigsaw documents of the behaviour packs (`bp.jigsaw()`): five collections loaded and
/// used together.
///
/// Each JSON family is keyed by its `description.identifier`; the highest-priority pack's
/// definition wins, and inside one pack the first file (vanilla drops a later duplicate
/// silently; the crate reports it as `ShadowedDefinition`). The families are independent: one
/// identifier may name a structure, a pool, a set and a processor list. The metadata files are
/// keyed by path, the top pack's copy winning.
#[derive(Debug)]
#[non_exhaustive]
pub struct Jigsaw {
    /// The jigsaw structures (`worldgen/structures/**/*.json`).
    pub structures: Registry<JigsawStructure>,
    /// The template pools (`worldgen/template_pools/**/*.json`).
    pub pools: Registry<TemplatePool>,
    /// The structure sets (`worldgen/structure_sets/**/*.json`).
    pub structure_sets: Registry<StructureSet>,
    /// The processor lists (`worldgen/processors/**/*.json`).
    pub processors: Registry<ProcessorList>,
    /// The metadata caches (`worldgen/metadata/*.dat`).
    pub metadata: Files<JigsawMetadata>,
}

impl Jigsaw {
    /// Every finding of the five collections, structures first.
    pub fn diagnostics(
        &self,
    ) -> impl Iterator<Item = &bedrock_addon_core::diagnostic::Diagnostic> + '_ {
        self.structures
            .diagnostics()
            .chain(self.pools.diagnostics())
            .chain(self.structure_sets.diagnostics())
            .chain(self.processors.diagnostics())
            .chain(self.metadata.diagnostics())
    }
}
