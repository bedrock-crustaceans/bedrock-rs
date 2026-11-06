//! Format 1.19.50: the struct and the step to 1.19.60.

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

use super::Payload;
use super::fragments;
use super::ops::{Members, Told, components, section_a, section_b, sections};
use super::v1_19_0::append_all_of;

/// An entity document as written in format 1.19.50 (up to the next struct's version): its
/// payload as written.
pub type Entity = Legacy<Payload, 1, 19, 50>;

/// 1.19.50 → 1.19.60 (gate 1.19.60).
impl LegacyStep<1, 19, 50> for Payload {
    type Next = super::v1_19_60::Entity;

    fn step(this: Entity, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = this.0;
        if section_a(&payload, cx) && cx.actor_type == ActorType::Witch {
            // The 1.19.60 witch fragment is one constant. The crate's own
            // information, since the fixer does it: a component of the document that the
            // fragment replaces by another value is told.
            let mut told = Told::default();
            let components = components(&mut payload);
            for (key, fragment) in [
                ("minecraft:shooter", fragments::witch_shooter()),
                (
                    "minecraft:behavior.drink_potion",
                    fragments::witch_drink_potion(),
                ),
            ] {
                if components
                    .get(key)
                    .is_some_and(|had| !had.is_null() && *had != fragment)
                {
                    told.dropped(JsonPath::new().with(&["components", key]));
                }
                components.set(key, fragment);
            }
            told.report(cx);
        }
        if section_b(&payload, cx) {
            let mut told = Told::default();
            for section in sections(&mut payload) {
                if let Some(goal) = section
                    .members
                    .object("minecraft:behavior.trade_with_player")
                {
                    append_all_of(
                        goal,
                        vec![
                            fragments::not_in_water(),
                            fragments::on_ground_or_sleeping(),
                        ],
                        &section.at.child("minecraft:behavior.trade_with_player"),
                        &mut told,
                    );
                }
            }
            told.report(cx);
        }
        Legacy(payload)
    }
}
