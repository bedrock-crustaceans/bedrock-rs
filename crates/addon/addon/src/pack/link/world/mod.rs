//! The link rules of world generation, the small behaviour-pack families and cameras.
//!
//! One file per pair of modules, named for the module that holds the reference. Where a
//! family has a checker of its own (the jigsaw documents, the POI types, the behaviour trees,
//! the aim-assist documents), the rule hands it the lookups of the pass and tells what it
//! finds; a target whose module is compiled out counts as resolved, so nothing is told of it.

rules! {
    mod aim_assist needs ["aim_assist"];
    mod aim_assist_block needs ["aim_assist", "block"];
    mod aim_assist_entity needs ["aim_assist", "entity"];
    mod aim_assist_item needs ["aim_assist", "item"];
    mod behavior_tree needs ["behavior_tree"];
    mod biome needs ["biome"];
    mod biome_block needs ["biome", "block"];
    mod dimension_biome needs ["dimension", "biome"];
    mod feature needs ["feature"];
    mod feature_block needs ["feature", "block"];
    mod feature_rules needs ["feature_rules", "feature"];
    mod feature_structure needs ["feature", "structure"];
    mod jigsaw needs ["jigsaw"];
    mod jigsaw_block needs ["jigsaw", "block"];
    mod jigsaw_loot needs ["jigsaw", "loot"];
    mod jigsaw_structure needs ["jigsaw", "structure"];
    mod poi_block needs ["poi", "block"];
    mod structure_block needs ["structure", "block"];

    run pub(super) run() =
        feature::run,
        feature_rules::run,
        jigsaw::run,
        biome::run,
        dimension_biome::run,
        structure_block::run,
        poi_block::run,
        behavior_tree::run,
        aim_assist::run;
}

#[cfg(test)]
mod tests;
