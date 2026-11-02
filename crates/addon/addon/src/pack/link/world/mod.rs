//! The link rules of world generation, the small behaviour-pack families and cameras.
//!
//! One file per pair of modules, named for the module that holds the reference. Where a
//! family has a checker of its own (the jigsaw documents, the POI types, the behaviour trees,
//! the aim-assist documents), the rule hands it the lookups of the pass and tells what it
//! finds; a target whose module is compiled out counts as resolved, so nothing is told of it.

#[cfg(feature = "aim_assist")]
mod aim_assist;
#[cfg(all(feature = "aim_assist", feature = "block"))]
mod aim_assist_block;
#[cfg(all(feature = "aim_assist", feature = "entity"))]
mod aim_assist_entity;
#[cfg(all(feature = "aim_assist", feature = "item"))]
mod aim_assist_item;
#[cfg(feature = "behavior_tree")]
mod behavior_tree;
#[cfg(feature = "biome")]
mod biome;
#[cfg(all(feature = "biome", feature = "block"))]
mod biome_block;
#[cfg(all(feature = "dimension", feature = "biome"))]
mod dimension_biome;
#[cfg(feature = "feature")]
mod feature;
#[cfg(all(feature = "feature", feature = "block"))]
mod feature_block;
#[cfg(all(feature = "feature_rules", feature = "feature"))]
mod feature_rules;
#[cfg(all(feature = "feature", feature = "structure"))]
mod feature_structure;
#[cfg(feature = "jigsaw")]
mod jigsaw;
#[cfg(all(feature = "jigsaw", feature = "block"))]
mod jigsaw_block;
#[cfg(all(feature = "jigsaw", feature = "loot"))]
mod jigsaw_loot;
#[cfg(all(feature = "jigsaw", feature = "structure"))]
mod jigsaw_structure;
#[cfg(all(feature = "poi", feature = "block"))]
mod poi_block;
#[cfg(all(feature = "structure", feature = "block"))]
mod structure_block;

#[allow(
    unused_variables,
    reason = "without a content module the group has no rule"
)]
pub(super) fn run(cx: &super::LinkCx<'_>, out: &mut super::Findings) {
    #[cfg(feature = "feature")]
    feature::run(cx, out);
    #[cfg(all(feature = "feature_rules", feature = "feature"))]
    feature_rules::run(cx, out);
    #[cfg(feature = "jigsaw")]
    jigsaw::run(cx, out);
    #[cfg(feature = "biome")]
    biome::run(cx, out);
    #[cfg(all(feature = "dimension", feature = "biome"))]
    dimension_biome::run(cx, out);
    #[cfg(all(feature = "structure", feature = "block"))]
    structure_block::run(cx, out);
    #[cfg(all(feature = "poi", feature = "block"))]
    poi_block::run(cx, out);
    #[cfg(feature = "behavior_tree")]
    behavior_tree::run(cx, out);
    #[cfg(feature = "aim_assist")]
    aim_assist::run(cx, out);
}

#[cfg(test)]
mod tests;
