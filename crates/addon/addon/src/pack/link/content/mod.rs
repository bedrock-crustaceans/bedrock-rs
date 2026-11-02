//! The link rules of the behaviour-pack content: block, item, loot, trade, recipe, catalogue and
//! trim references.
//!
//! One file per pair of modules, the first being the module that holds the reference; a rule
//! is one function per row, called from [`run`] in the order of the rows. The
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

/// The rule files of the group, in the order their rules run: each is one module with one
/// `run`, compiled when the modules it relates are.
macro_rules! rules {
    ($(#[cfg($cfg:meta)] $name:ident;)*) => {
        $(
            #[cfg($cfg)]
            mod $name;
        )*

        pub(super) fn run(cx: &super::LinkCx<'_>, out: &mut super::Findings) {
            $(
                #[cfg($cfg)]
                $name::run(cx, out);
            )*
            let _ = (cx, out);
        }
    };
}

rules! {
    #[cfg(all(feature = "block", feature = "geometry"))] block_geometry;
    #[cfg(feature = "block")] block;
    #[cfg(all(feature = "block", feature = "voxel_shape"))] block_voxel_shape;
    #[cfg(all(feature = "block", feature = "texture_atlas"))] block_texture;
    #[cfg(all(feature = "block", feature = "loot"))] block_loot;
    #[cfg(all(feature = "block", feature = "lang", feature = "languages"))] block_text;
    #[cfg(all(feature = "block", feature = "item"))] block_item;
    #[cfg(feature = "item")] item;
    #[cfg(all(feature = "item", feature = "entity"))] item_entity;
    #[cfg(all(feature = "item", feature = "block"))] item_block;
    #[cfg(all(feature = "item", feature = "sound_definitions", feature = "sounds_json"))] item_sound;
    #[cfg(all(feature = "crafting_catalog", feature = "lang", feature = "languages"))] item_text;
    #[cfg(all(feature = "loot", feature = "item"))] loot_item;
    #[cfg(feature = "loot")] loot;
    #[cfg(all(feature = "loot", feature = "entity"))] loot_entity;
    #[cfg(all(feature = "trade", feature = "item"))] trade_item;
    #[cfg(all(feature = "recipe", feature = "item"))] recipe_item;
    #[cfg(all(feature = "recipe", feature = "block"))] recipe_block;
}

#[cfg(test)]
mod tests;
