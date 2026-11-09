//! The link rules of the behaviour-pack content: block, item, loot, trade, recipe, catalogue and
//! trim references.
//!
//! One file per pair of modules, the first being the module that holds the reference, listed
//! once with `rules!` and its needs; a rule is one function per row, and [`run`] calls them in
//! the order of the rows. The
//! rows that are not link rows (checked at load, at run time or not at all) have no rule
//! here: their tests are in `tests/link/content`.

#[allow(
    unused_imports,
    reason = "which rules exist depends on the enabled features"
)]
use bedrock_addon_core::diagnostic::JsonPath;

/// One step of a JSON path: a member name or an array index.
#[allow(
    dead_code,
    reason = "which rules exist depends on the enabled features"
)]
#[derive(Debug, Clone, Copy)]
pub(super) enum Step<'s> {
    /// A member.
    Key(&'s str),
    /// An element.
    Index(usize),
}

impl<'s> From<&'s str> for Step<'s> {
    fn from(key: &'s str) -> Self {
        Self::Key(key)
    }
}

impl<'s> From<&'s String> for Step<'s> {
    fn from(key: &'s String) -> Self {
        Self::Key(key)
    }
}

impl From<usize> for Step<'_> {
    fn from(index: usize) -> Self {
        Self::Index(index)
    }
}

/// `base` continued by `steps`.
#[allow(
    dead_code,
    reason = "which rules exist depends on the enabled features"
)]
pub(super) fn below(base: &JsonPath, steps: &[Step<'_>]) -> JsonPath {
    let mut path = base.clone();
    for step in steps {
        match step {
            Step::Key(key) => path.push(key),
            Step::Index(index) => path.push_index(*index),
        }
    }
    path
}

/// The path of `steps` from the root of a document, `at!["pools", 0, "entries"]`, or below
/// a path, `at![&base; "name"]`. In scope in every rule file of the group, which are declared
/// below it.
#[allow(
    unused_macros,
    reason = "which rules exist depends on the enabled features"
)]
macro_rules! at {
    ($($step:expr),* $(,)?) => {
        $crate::pack::link::content::below(
            &bedrock_addon_core::diagnostic::JsonPath::new(),
            &[$($crate::pack::link::content::Step::from($step)),*],
        )
    };
    ($base:expr; $($step:expr),* $(,)?) => {
        $crate::pack::link::content::below(
            $base,
            &[$($crate::pack::link::content::Step::from($step)),*],
        )
    };
}

rules! {
    mod block_geometry needs ["block", "geometry"];
    mod block needs ["block"];
    mod block_voxel_shape needs ["block", "voxel_shape"];
    mod block_texture needs ["block", "texture_atlas"];
    mod block_loot needs ["block", "loot"];
    mod block_text needs ["block", "lang", "languages"];
    mod block_item needs ["block", "item"];
    mod item needs ["item"];
    mod item_entity needs ["item", "entity"];
    mod item_block needs ["item", "block"];
    mod item_sound needs ["item", "sound_definitions", "sounds_json"];
    mod crafting_catalog_text needs ["crafting_catalog", "lang", "languages"];
    mod loot_item needs ["loot", "item"];
    mod loot needs ["loot"];
    mod loot_entity needs ["loot", "entity"];
    mod trade_item needs ["trade", "item"];
    mod recipe_item needs ["recipe", "item"];
    mod recipe_block needs ["recipe", "block"];

    run pub(super) run() =
        block_geometry::run,
        block::run,
        block_voxel_shape::run,
        block_texture::run,
        block_loot::run,
        block_text::run,
        block_item::run,
        item::run,
        item_entity::run,
        item_block::run,
        item_sound::run,
        crafting_catalog_text::run,
        loot_item::run,
        loot::run,
        loot_entity::run,
        trade_item::run,
        recipe_item::run,
        recipe_block::run;
}

#[cfg(test)]
mod tests;
