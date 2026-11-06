//! The trigger components: an event fired on a subject when something happens to the entity
//! (`on_death`, `on_hurt`, …), and `on_equipment_changed`.
//!
//! The ten `on_*` components have `event`, `filters` and `target` (1.26.40); and
//! `on_equipment_changed` has `slots` (1.26.20).

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::schema::{self, Schema, SchemaCx, SchemaFields};
use bedrock_addon_core::shared::{Filter, FilterSubject};
use facet::Facet;

use super::parse::{ComponentCheck, Typed};
use super::value::EquipmentSlot;

/// Declares one trigger component struct per line: `event`, `filters`, `target`.
macro_rules! triggers {
    ($( $(#[$doc:meta])* $name:ident = $key:literal; )*) => {$(
        $(#[$doc])*
        #[doc(alias = $key)]
        #[derive(Clone, Debug, Default, PartialEq, Facet)]
        #[facet(schema::unknown = "keep")]
        pub struct $name {
            /// `event`: the event to fire, looked up in the definition of `target`; default
            /// none (empty).
            #[facet(default = String::new())]
            pub event: String,
            /// `filters`: the condition for firing; absent means always.
            pub filters: Option<Filter>,
            /// `target`: whose event fires; default `self`.
            #[facet(default = FilterSubject::Self_)]
            pub target: FilterSubject,
        }

        impl $name {
            /// The trigger firing `event` on `self` without a condition.
            pub fn new(event: impl Into<String>) -> Self {
                Self {
                    event: event.into(),
                    ..Self::default()
                }
            }
        }

        // The component is a trigger itself (`{event, filters, target}`): its trigger member is
        // the empty pattern, which the derive states only for an `EventTrigger` field.
        impl SchemaFields for $name {
            fn check_fields(&mut self, _: &mut SchemaCx<'_>) {}

            fn trigger_members(prefix: &str, out: &mut Vec<String>) {
                out.push(prefix.to_owned());
            }
        }

        impl Typed for $name {
            fn check(&mut self, cx: &mut ComponentCheck<'_>) {
                if let Some(filters) = &self.filters {
                    cx.extend(filters.check(&JsonPath::new().child("filters"), cx.unknown()));
                }
            }
        }
    )*};
}

triggers! {
    /// `minecraft:on_death`: fires when the entity dies (only the ender dragon in vanilla).
    OnDeath = "minecraft:on_death";
    /// `minecraft:on_friendly_anger`: fires when a nearby entity of the same type gets angry.
    OnFriendlyAnger = "minecraft:on_friendly_anger";
    /// `minecraft:on_hurt`: fires when the entity is hurt.
    OnHurt = "minecraft:on_hurt";
    /// `minecraft:on_hurt_by_player`: fires when a player hurts the entity.
    OnHurtByPlayer = "minecraft:on_hurt_by_player";
    /// `minecraft:on_ignite`: fires when the entity is set on fire.
    OnIgnite = "minecraft:on_ignite";
    /// `minecraft:on_start_landing`: fires when the entity starts landing (the ender dragon).
    OnStartLanding = "minecraft:on_start_landing";
    /// `minecraft:on_start_takeoff`: fires when the entity starts taking off (the ender dragon).
    OnStartTakeoff = "minecraft:on_start_takeoff";
    /// `minecraft:on_target_acquired`: fires when the entity finds a target.
    OnTargetAcquired = "minecraft:on_target_acquired";
    /// `minecraft:on_target_escape`: fires when the entity's target escapes.
    OnTargetEscape = "minecraft:on_target_escape";
    /// `minecraft:on_wake_with_owner`: fires when the entity's owner wakes up while the entity
    /// sleeps with them.
    OnWakeWithOwner = "minecraft:on_wake_with_owner";
}

/// `minecraft:on_equipment_changed`: events fired when an item is put into, or taken out of,
/// an equipment slot.
#[doc(alias = "minecraft:on_equipment_changed")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct OnEquipmentChanged {
    /// `slots`: one entry per watched slot.
    #[facet(default)]
    pub slots: Vec<SlotEvents>,
}

/// One watched slot of [`OnEquipmentChanged`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SlotEvents {
    /// `slot`: the slot; default `slot.weapon.mainhand`.
    #[facet(default = EquipmentSlot::WeaponMainhand)]
    pub slot: EquipmentSlot,
    /// `on_equip`: the event fired when an item goes into the slot; default none (empty).
    #[facet(default = String::new())]
    pub on_equip: String,
    /// `on_unequip`: the event fired when the item leaves the slot; default none (empty).
    #[facet(default = String::new())]
    pub on_unequip: String,
}

impl Typed for OnEquipmentChanged {}
