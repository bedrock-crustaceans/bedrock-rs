//! Format 1.18.0: the struct and the step to 1.18.10.

super::legacy_struct!(1, 18, 0);

use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

use super::ops::{Members, Told, group, json, path, same, section_a};

/// 1.18.0 → 1.18.10 (gate 1.18.10).
impl Upgrade for Entity {
    type Next = super::v1_18_10::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        if section_a(&payload, cx) && cx.actor_type == ActorType::Ocelot {
            let mut told = Told::default();
            ocelot(&mut payload, &mut told);
            told.report(cx);
        }
        super::v1_18_10::Entity(payload)
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
        let constant = json(
            r#"{"entities": {"min_wait_time": 0, "max_wait_time": 0, "num_to_spawn": 2, "single_use": true,
                "spawn_entity": "minecraft:ocelot", "spawn_event": "minecraft:entity_born", "spawn_method": "born",
                "spawn_sound": "", "filters": [{"test": "random_chance", "value": 7}]}}"#,
        );
        // A `spawn_entity` the file's group has is replaced, as the fixer replaces it, and
        // noted when it held something else.
        if group
            .get("minecraft:spawn_entity")
            .is_some_and(|had| !had.is_null() && !same(had, &constant))
        {
            told.dropped(path(&[
                "component_groups",
                "minecraft:wild_child_ocelot_spawn",
                "minecraft:spawn_entity",
            ]));
        }
        group.set("minecraft:spawn_entity", constant);
    }
}
