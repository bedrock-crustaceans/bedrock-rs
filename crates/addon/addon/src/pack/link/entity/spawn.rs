//! The conditions of the stack's spawn rules (1.8.0 shape) and raid spawn groups, with the
//! place of each: what the spawn rules walk.
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::document::Origin;

use super::super::LinkCx;
use crate::families::spawn_rules::{Condition, SpawnRules};

/// The payload key of a spawn-rules document.
pub(super) const RULES: &str = "minecraft:spawn_rules";
/// The payload key of a spawn-groups document.
#[cfg(feature = "spawn_groups")]
const GROUPS: &str = "minecraft:spawn_groups";

/// One condition of a document.
#[derive(Debug)]
pub(super) struct Conditions<'a> {
    /// The document.
    pub(super) origin: &'a Origin,
    /// The condition's path among the conditions the document holds.
    pub(super) at: JsonPath,
    /// The document's findings: they give the place in the file of a member the document
    /// holds ([`Conditions::place`]).
    pub(super) places: &'a Diagnostics,
    /// The condition.
    pub(super) condition: &'a Condition,
}

impl Conditions<'_> {
    /// The path the member at `held` (a path below [`Conditions::at`], counted as the
    /// document holds its lists) has in the file: the reader of the 1.8.0 shape leaves out a
    /// condition or a permutation that is no object and a block descriptor it refuses.
    pub(super) fn place(&self, held: &JsonPath) -> JsonPath {
        self.places.place_of(held)
    }
}

/// Every condition of the standard spawn rules, then of the spawn groups, in load order.
pub(super) fn each_condition<'a>(cx: &LinkCx<'a>) -> impl Iterator<Item = Conditions<'a>> {
    let standard =
        cx.behavior()
            .spawn_rules()
            .into_iter()
            .filter_map(|(_, document)| match document.value() {
                SpawnRules::Standard(rules) => Some((
                    document.origin(),
                    document.diagnostics(),
                    RULES,
                    rules.conditions.as_deref(),
                )),
                SpawnRules::Simplified(_) => None,
            });
    // The raid spawn groups are the `spawn_groups` family's.
    #[cfg(feature = "spawn_groups")]
    let groups = cx
        .behavior()
        .spawn_groups()
        .into_iter()
        .map(|(_, document)| {
            (
                document.origin(),
                document.diagnostics(),
                GROUPS,
                document.value().conditions.as_deref(),
            )
        });
    #[cfg(not(feature = "spawn_groups"))]
    let groups = std::iter::empty::<(&Origin, &Diagnostics, &str, Option<&[Condition]>)>();
    standard
        .chain(groups)
        .flat_map(|(origin, places, payload, conditions)| {
            conditions
                .into_iter()
                .flatten()
                .enumerate()
                .map(move |(index, condition)| {
                    let mut at = JsonPath::new();
                    at.push(payload);
                    at.push("conditions");
                    at.push_index(index);
                    Conditions {
                        origin,
                        at,
                        places,
                        condition,
                    }
                })
        })
}
