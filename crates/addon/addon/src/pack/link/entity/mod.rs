//! The link rules of the entity group: entity and spawn references.
//!
//! The rules over an entity's components read the members the entity family names as
//! references of each kind of target (`Entity::references`, gathered once per pass in
//! [`walk`]) and judge each name. The findings of one rule are in document order; within one
//! component, in the order of the family's table for the kind.

rules! {
    mod spawn_rules needs ["spawn_rules", any["block", "entity"]];
    mod spawn_rules_block needs ["spawn_rules", "block"];
    mod spawn_rules_entity needs ["spawn_rules", "entity"];

    with entities = walk::Entities::new(cx) needs ["entity"] {
        mod entity_animation needs ["animation", "animation_controller"];
        mod entity_block needs ["block"];
        mod entity_entity needs [];
        mod entity_item needs ["item"];
        mod entity_loot needs ["loot"];
        mod entity_particle needs ["particle"];
        mod entity_poi needs ["poi"];
        mod entity_trade needs ["trade"];

        /// The rules over the references of the stack's entities, gathered once.
        run references =
            entity_animation::run,
            entity_block::run,
            entity_loot::run,
            entity_trade::run,
            entity_entity::run,
            entity_item::run,
            entity_particle::run,
            entity_poi::run;
    }

    /// The rules over the conditions of spawn rules and spawn groups.
    run spawns() = spawn_rules_block::run, spawn_rules_entity::run;
}

#[cfg(all(test, feature = "entity"))]
mod tests;

pub(super) fn run(cx: &super::LinkCx<'_>, out: &mut super::Findings) {
    references(cx, out);
    spawns(cx, out);
}

/// Whether the paths [`unread`] gives are folded to lower case.
#[cfg(all(feature = "entity", any(feature = "loot", feature = "trade")))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fold {
    /// The paths as the load has them.
    #[cfg(feature = "trade")]
    Keep,
    /// The paths in lower case.
    #[cfg(feature = "loot")]
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
            matches!(
                finding.code.id(),
                "core/unsupported_file_format" | "core/file_not_loaded"
            )
        })
        .filter_map(|finding| finding.location.path.as_ref())
        .filter(|path| {
            path.as_str()
                .strip_prefix(directory)
                .is_some_and(|rest| rest.starts_with('/'))
        })
        .map(|path| match fold {
            #[cfg(feature = "loot")]
            Fold::Lower => path.as_str().to_lowercase(),
            #[cfg(feature = "trade")]
            Fold::Keep => path.as_str().to_owned(),
        })
        .collect()
}
