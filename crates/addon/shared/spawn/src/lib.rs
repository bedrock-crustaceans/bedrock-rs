//! What the `spawn_rules` and `spawn_groups` families share: the 1.8.0 condition and its filters,
//! the spawn rules a condition gives, the readers of the hand-written parsers, the texts of the
//! loaders and the steps of loading one spawn file. A family composes these; the crate depends on
//! `bedrock_addon_core` and `bedrock_addon_shared_shapes` only.
//!
//! The two families stay two crates over this one. The game reads both files with the one hand
//! parser here ([`legacy::read`], switched by [`legacy::Flavor`] and [`condition::Profile`]), but
//! everything around that parser differs: spawn rules are `spawn_rules/**/*.json` in every
//! behaviour pack and spawn groups only the app data's flat `definitions/spawn_groups/*.json`;
//! they have their own identifiers and registries (`SpawnRulesId`, `SpawnGroupId`); a spawn rules
//! file without a version is read at the floor with an information while a spawn group without
//! one is rejected; spawn rules have a second, experiment-gated simplified shape that groups never
//! have; and the registration rule, the validate findings, the closing texts and the
//! unknown-member findings are each the game's own per file. A merge would delete one manifest
//! and one re-export list, no shared body.
//!
//! The Molang check of the conditions' `tags` ([`validate::molang`]) is on only with this
//! crate's `molang` feature, which each family forwards from its own.

pub mod condition;
pub mod legacy;
pub mod load;
pub mod read;
pub mod rules;
pub mod texts;
pub mod validate;

#[cfg(test)]
mod helper_pins;
#[cfg(test)]
mod tests;

pub use condition::{
    AboveBlockFilter, Condition, DelayFilter, DensityLimit, DifficultyFilter, Herd, MobEventFilter,
    OneOrMany, Permutation, RangeFilter, VillageFilter, Weight, WorldAgeFilter,
};
pub use rules::{Difficulty, HerdInfo, MobSpawnRules, PopulationControl};

#[cfg(test)]
mod type_tests {
    use std::fmt::Debug;

    fn assert_debug<T: Debug>() {}
    fn assert_hashable<T: Eq + std::hash::Hash>() {}

    #[test]
    fn public_types_implement_debug() {
        assert_debug::<crate::load::FileLoad<()>>();
        assert_debug::<crate::legacy::Read>();
        assert_debug::<crate::condition::ReadCx<'static>>();
    }

    #[test]
    fn data_types_are_comparable_and_hashable() {
        assert_hashable::<crate::condition::Profile>();
        assert_hashable::<crate::condition::OneOrMany<String>>();
    }
}
