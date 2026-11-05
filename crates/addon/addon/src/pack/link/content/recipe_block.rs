//! The crafting stations a recipe is offered at.

use std::collections::HashSet;

use bedrock_addon_core::diagnostic::Severity;

use super::super::{At, Findings, LinkCx, unresolved};
use super::block::sections;
use crate::families::block::component::CraftingTable;
use crate::families::recipe::{Recipe, Tags};

/// A crafting tag: what a recipe's `tags` and a block's `crafting_table.crafting_tags` name.
const CRAFTING_TAG: &str = "crafting tag";

/// Each of a recipe's `tags` is a station of the game or a crafting tag a block of
/// the stack offers (`minecraft:crafting_table.crafting_tags`), compared as written. The game
/// compares no tag when it reads a recipe; one no station offers is never shown: a lint.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    if !cx.lints() {
        return;
    }
    let recipes = cx.behavior().recipes();
    if recipes.is_empty() {
        return;
    }
    let mut offered: HashSet<&str> = HashSet::new();
    for (_, block) in cx.behavior().blocks() {
        for (_, components) in sections(block) {
            if let Some(table) = components.get::<CraftingTable>() {
                offered.extend(table.crafting_tags.iter().map(String::as_str));
            }
        }
    }
    for (_, document) in recipes {
        let (payload, tags) = tags_of(document.value());
        let Some(tags) = tags else {
            continue;
        };
        for (index, tag) in tags.as_slice().iter().enumerate() {
            // The stations are the engine's: they stand with any host, which can add to them.
            let known = offered.contains(tag.as_str())
                || super::super::aliases::is_crafting_station_tag(tag.as_str())
                || cx
                    .built_ins()
                    .is_some_and(|host| host.contains(CRAFTING_TAG, tag));
            if known {
                continue;
            }
            let path = match tags {
                Tags::One(_) => at![payload, "tags"],
                // At the index the file has the tag at: the reader leaves out an entry
                // that is no string.
                Tags::Many(_) => document
                    .diagnostics()
                    .place_of(&at![payload, "tags", index]),
            };
            let place = At::new(document.origin(), path, Severity::Info);
            out.push(unresolved(place, CRAFTING_TAG, tag, None));
        }
    }
}

/// The payload key of a recipe and its `tags`.
fn tags_of(recipe: &Recipe) -> (&'static str, Option<&Tags>) {
    let tags = match recipe {
        Recipe::Shaped(recipe) => recipe.tags.as_ref(),
        Recipe::Shapeless(recipe) => recipe.tags.as_ref(),
        Recipe::Furnace(recipe) => recipe.tags.as_ref(),
        Recipe::BrewingMix(recipe) => recipe.tags.as_ref(),
        Recipe::BrewingContainer(recipe) => recipe.tags.as_ref(),
        Recipe::MaterialReduction(recipe) => recipe.tags.as_ref(),
        Recipe::SmithingTransform(recipe) => recipe.tags.as_ref(),
        Recipe::SmithingTrim(recipe) => recipe.tags.as_ref(),
        // `Recipe` is non-exhaustive across the crate boundary; a type added later has no tags here.
        _ => None,
    };
    (recipe.kind().payload_key(), tags)
}
