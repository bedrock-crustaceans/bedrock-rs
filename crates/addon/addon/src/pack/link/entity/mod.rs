//! The link rules of the entity group: entity and spawn references.
//!
//! The rules over an entity's components read them as JSON in the newest format's shape
//! ([`walk`]) and find the members that hold a reference through one table per kind of
//! target ([`table`]): a row is a component key and a member pattern. The findings of one
//! rule are in document order; within one component, in the order of the rule's table.

#[cfg(all(
    feature = "entity",
    feature = "animation",
    feature = "animation_controller"
))]
mod entity_animation;
#[cfg(all(feature = "entity", feature = "block"))]
mod entity_block;
#[cfg(feature = "entity")]
mod entity_entity;
#[cfg(all(feature = "entity", feature = "item"))]
mod entity_item;
#[cfg(all(feature = "entity", feature = "loot"))]
mod entity_loot;
#[cfg(all(feature = "entity", feature = "particle"))]
mod entity_particle;
#[cfg(all(feature = "entity", feature = "poi"))]
mod entity_poi;
#[cfg(all(feature = "entity", feature = "trade"))]
mod entity_trade;
#[cfg(all(feature = "spawn_rules", any(feature = "block", feature = "entity")))]
mod spawn;
#[cfg(all(feature = "spawn_rules", feature = "block"))]
mod spawn_block;
#[cfg(all(feature = "spawn_rules", feature = "entity"))]
mod spawn_entity;
#[cfg(feature = "entity")]
mod table;
#[cfg(all(test, feature = "entity"))]
mod tests;
#[cfg(feature = "entity")]
mod walk;

pub(super) fn run(cx: &super::LinkCx<'_>, out: &mut super::Findings) {
    #[cfg(feature = "entity")]
    {
        let entities = walk::Entities::new(cx);
        #[cfg(all(feature = "animation", feature = "animation_controller"))]
        entity_animation::run(cx, out, &entities);
        #[cfg(feature = "block")]
        entity_block::run(cx, out, &entities);
        #[cfg(feature = "loot")]
        entity_loot::run(cx, out, &entities);
        #[cfg(feature = "trade")]
        entity_trade::run(cx, out, &entities);
        entity_entity::run(cx, out, &entities);
        #[cfg(feature = "item")]
        entity_item::run(cx, out, &entities);
        #[cfg(feature = "particle")]
        entity_particle::run(cx, out, &entities);
        #[cfg(feature = "poi")]
        entity_poi::run(cx, out, &entities);
    }
    #[cfg(all(feature = "spawn_rules", feature = "block"))]
    spawn_block::run(cx, out);
    #[cfg(all(feature = "spawn_rules", feature = "entity"))]
    spawn_entity::run(cx, out);
    #[cfg(not(any(feature = "entity", all(feature = "spawn_rules", feature = "block"))))]
    let _ = (cx, out);
}

/// The name a member holds: its string, unless it is empty (an empty name names nothing: it
/// is what a member that was not written reads as).
#[cfg(feature = "entity")]
fn name(value: &bedrock_addon_core::json::Value) -> Option<&str> {
    value
        .as_string()
        .map(facet_value::VString::as_str)
        .filter(|name| !name.is_empty())
}

/// Whether the paths [`unread`] gives are folded to lower case.
#[cfg(all(feature = "entity", any(feature = "loot", feature = "trade")))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fold {
    /// The paths as the load has them.
    Keep,
    /// The paths in lower case.
    Lower,
}

/// The paths of the files below `directory` of the behaviour packs that the load could not
/// read as text: a baked binary document, which the game reads and this crate does not, and a
/// file that is not resident (what the loaded collections report as `UnsupportedFileFormat`
/// and `FileNotLoaded`). A reference to such a file is not judged; a file the load refused
/// for what it says is no definition, for the game either, and a reference to it is a miss
/// like any other. In lower case when `fold` is [`Fold::Lower`]. Called after the lookup that loads the
/// collection of `directory`.
#[cfg(all(feature = "entity", any(feature = "loot", feature = "trade")))]
fn unread(
    cx: &super::LinkCx<'_>,
    directory: &str,
    fold: Fold,
) -> std::collections::HashSet<String> {
    let mut findings = Vec::new();
    cx.behavior().collection_findings(&mut findings);
    findings
        .into_iter()
        .filter(|finding| {
            // By legacy name: the variant and the definitions that were it.
            matches!(
                finding.code.name(),
                "UnsupportedFileFormat" | "FileNotLoaded"
            )
        })
        .filter_map(|finding| finding.location.path.as_ref())
        .filter(|path| {
            path.as_str()
                .strip_prefix(directory)
                .is_some_and(|rest| rest.starts_with('/'))
        })
        .map(|path| match fold {
            Fold::Lower => path.as_str().to_lowercase(),
            Fold::Keep => path.as_str().to_owned(),
        })
        .collect()
}
