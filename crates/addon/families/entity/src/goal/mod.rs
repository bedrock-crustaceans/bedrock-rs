//! The AI goals of an entity (`minecraft:behavior.*`), the Brain memory goals
//! (`minecraft:memory_behavior.*`) and the Brain memory sensors (`minecraft:memory_sensor.*`):
//! one type per key, each a variant of [`EntityComponent`](super::component::EntityComponent),
//! since goals and components share one map in the JSON.
//!
//! Every member is optional and kept as written (`None` when absent); each goal has a method per
//! scalar member that gives the value vanilla uses, the written one or the member's default.
//! Scalars take only their own JSON type: a string where a number belongs is a typed parse error,
//! as in vanilla's cereal schemas.
//!
//! A goal in a document at or above its typed version is read strictly: a member of the wrong
//! type or outside its bounds is an error and drops the goal. A goal converted from an older
//! document is read leniently: such a member is a warning and takes its default.

mod attack;
mod check;
mod common;
#[doc(hidden)]
pub mod convert;
mod dragon;
mod item;
mod life;
mod look;
mod memory;
mod molang;
mod movement;
mod table;
mod target;
pub use attack::*;
pub use common::*;
pub use dragon::*;
pub use item::*;
pub use life::*;
pub use look::*;
pub use memory::*;
pub use movement::*;
pub use target::*;

/// The versioned goal parse belongs to the entity document, so it is crate-private: outside the
/// crate a goal is read through the document, or through its component row.
///
/// ```compile_fail,E0603
/// let _ = bedrock_addon_family_entity::goal::versioned("minecraft:behavior.float");
/// ```
/// ```compile_fail,E0603
/// let _: Option<bedrock_addon_family_entity::goal::VersionedParseFn> = None;
/// ```
#[cfg(doctest)]
struct VersionedIsTheDocumentsOwn;

pub(crate) use check::VersionedParseFn;
pub(crate) use check::legacy_ranges;
pub(crate) use check::parse;
pub(crate) use check::parse_versioned;
#[allow(unused_imports, reason = "the component map's key routing uses them")]
pub(crate) use memory::{GenericGoal, MOVE_TO_POSITION, generic_goal, suffix_missing};

/// The parse of the goal, Brain memory goal or memory sensor `key` by the format version of the
/// document it is in, for the entity document's component map: below the goal's typed version
/// the lenient conversion, at or above it the strict parse with the typed → typed conversions
/// and member gates of the versions after the document's. A suffixed
/// `minecraft:memory_behavior.move_to_position.<name>` key is that goal's. `None` for any other
/// key.
pub(crate) fn versioned(key: &str) -> Option<VersionedParseFn> {
    let key = match generic_goal(key) {
        GenericGoal::Named { base, .. } => base,
        _ => key,
    };
    // The goal rows' own table first (`goal` on the row), then the goals still listed by hand.
    [
        crate::component::EntityComponent::VERSIONED,
        table::VERSIONED,
    ]
    .into_iter()
    .find_map(|list| {
        let i = list.binary_search_by(|(name, _)| (*name).cmp(key)).ok()?;
        Some(list[i].1)
    })
}

/// Hands `visitor` every goal, Brain memory goal and memory sensor `versioned` knows, each once:
/// its key, the format version from which it is read typed, and its type. The goal rows' own
/// (`goal` on the row) and the goals still listed by hand; for a test that needs each goal as a
/// type.
// hidden: a test-side walk over the goals; API not yet reviewed
#[doc(hidden)]
pub fn visit_goals(
    visitor: &mut impl bedrock_addon_core::schema::GoalVisitor<crate::component::EntityComponent>,
) {
    crate::component::EntityComponent::visit_goals(visitor);
    table::visit(visitor);
}

#[cfg(test)]
mod tests {
    use super::{table, versioned, visit_goals};
    use crate::component::EntityComponent;
    use bedrock_addon_core::document::Component;
    use bedrock_addon_core::schema::GoalVisitor;
    use bedrock_addon_core::version::FormatVersion;

    /// The keys a visit is handed, in order.
    struct Keys(Vec<&'static str>);

    impl GoalVisitor<EntityComponent> for Keys {
        fn goal<T>(&mut self, key: &'static str, _: FormatVersion)
        where
            T: Component<Set = EntityComponent>
                + for<'a> facet::Facet<'a>
                + PartialEq
                + std::fmt::Debug,
        {
            self.0.push(key);
        }
    }

    /// `visit_goals` hands every key `versioned` finds in its two tables, each once.
    #[test]
    fn visit_goals_hands_every_goal_once() {
        let mut visited = Keys(Vec::new());
        visit_goals(&mut visited);
        let mut visited = visited.0;
        visited.sort_unstable();
        let mut listed: Vec<&str> = EntityComponent::VERSIONED
            .iter()
            .chain(table::VERSIONED)
            .map(|(key, _)| *key)
            .collect();
        listed.sort_unstable();
        listed.dedup();
        assert_eq!(visited, listed, "each goal once");
    }

    /// The keys `versioned` knows: every goal row, the Brain memory goals and sensors, and the
    /// suffixed generic memory goal; not a component or a dynamic goal key.
    #[test]
    fn versioned_knows_every_goal_and_the_suffixed_memory_goal() {
        assert!(versioned("minecraft:behavior.float").is_some());
        assert!(versioned("minecraft:memory_sensor.game_event").is_some());
        assert!(versioned("minecraft:memory_behavior.move_to_position.gold").is_some());
        assert!(versioned("minecraft:health").is_none());
        assert!(
            versioned("minecraft:behavior.vex_random_move").is_none(),
            "a dynamic key"
        );
        let listed = || table::VERSIONED.iter().chain(EntityComponent::VERSIONED);
        for (key, _) in listed() {
            assert!(
                versioned(key).is_some(),
                "{key} is found (the table is sorted)"
            );
            assert!(EntityComponent::TABLE.get(key).is_some(), "{key} has a row");
        }
        let goals = listed()
            .filter(|(key, _)| key.starts_with("minecraft:behavior."))
            .count();
        assert_eq!(
            goals, 176,
            "every goal of the table, and the four typed at 1.26.60"
        );
    }
}
