//! Every typed goal: its key, the format version vanilla first types it at, and the
//! round trip of its type, from the goal rows and the hand-listed goals of the crate.

use bedrock_addon_core::document::Component;
use bedrock_addon_core::schema::GoalVisitor;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_family_entity::component::EntityComponent;
use bedrock_addon_family_entity::goal;
use facet::Facet;

use super::{Goal, round_trip};

/// `(key, first typed, round trip)` for every `minecraft:behavior.*` goal typed by 1.26.50,
/// sorted by key: each goal's row (or its hand-listed `goals!` line) gives its key, its typed
/// version and its type.
pub fn goals() -> Vec<Goal> {
    /// The goals a round trip can read strictly.
    struct Collect(Vec<Goal>);

    impl GoalVisitor<EntityComponent> for Collect {
        fn goal<T>(&mut self, key: &'static str, typed: FormatVersion)
        where
            T: Component<Set = EntityComponent> + for<'a> Facet<'a> + PartialEq + std::fmt::Debug,
        {
            // The round trip reads the written goal strictly as of 1.26.50, the newest version
            // a goal's typed read has rules for: the goals typed after it and the Brain's
            // memory goals and sensors stay out.
            if key.starts_with("minecraft:behavior.") && typed <= FormatVersion::new(1, 26, 50) {
                self.0.push((
                    key,
                    (typed.major, typed.minor, typed.patch),
                    round_trip::<T>,
                ));
            }
        }
    }

    let mut out = Collect(Vec::new());
    goal::visit_goals(&mut out);
    out.0.sort_by_key(|(key, ..)| *key);
    out.0
}
