//! Format 1.18.0: the struct and the step to 1.18.10.

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

use super::Payload;
use super::fragments;
use super::ops::{Members, Told, group, same, section_a};

/// An entity document as written in format 1.18.0 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 18, 0>;

/// 1.18.0 → 1.18.10 (gate 1.18.10).
impl LegacyStep<1, 18, 0> for Payload {
    type Next = super::v1_18_10::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        if section_a(&payload, cx) && cx.actor_type == ActorType::Ocelot {
            let mut told = Told::default();
            ocelot(&mut payload, &mut told);
            told.report(cx);
        }
        Legacy(payload)
    }
}

/// A, vanilla `ocelot`: a recursive rewrite of `minecraft:entity_spawned`; only
/// when it returns true, the group `minecraft:wild_child_ocelot_spawn` gets the constant
/// `spawn_entity` (its `entities` an object). The rewrite reports true when the event
/// exists.
fn ocelot(payload: &mut Payload, told: &mut Told) {
    let rewritten = payload
        .events
        .as_ref()
        .is_some_and(|events| events.contains_key("minecraft:entity_spawned"));
    if !rewritten {
        return;
    }
    if let Some(group) = group(payload, "minecraft:wild_child_ocelot_spawn") {
        let constant = fragments::ocelot_spawn();
        // A `spawn_entity` the file's group has is replaced, as the fixer replaces it, and
        // noted when it held something else.
        if group
            .get("minecraft:spawn_entity")
            .is_some_and(|had| !had.is_null() && !same(had, &constant))
        {
            told.dropped(JsonPath::new().with(&[
                "component_groups",
                "minecraft:wild_child_ocelot_spawn",
                "minecraft:spawn_entity",
            ]));
        }
        group.set("minecraft:spawn_entity", constant);
    }
}
