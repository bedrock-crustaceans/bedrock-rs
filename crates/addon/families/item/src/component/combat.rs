//! Attacking and shooting: `cooldown`, `damage`, `kinetic_weapon`, `piercing_weapon`,
//! `projectile`, `shooter`, `swing_duration`, `swing_sounds`, `throwable`.

use bedrock_addon_core::id::{EntityId, SoundEventName};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::shared::{FloatRange, ItemDescriptor};
use facet::Facet;

use super::value::value_component;
use super::visit_item_descriptor;
use bedrock_addon_core::document::Extra;

/// `minecraft:cooldown`: the item cannot be used again for `duration` seconds; items sharing a
/// `category` share the cooldown.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Cooldown {
    /// The cooldown group (JSON `category`, required).
    pub category: String,
    /// The cooldown in seconds (JSON `duration`, required).
    pub duration: f32,
    /// Whether using or attacking starts it (JSON `type`; default `use`; engine 1.21.130).
    #[facet(default, rename = "type")]
    pub kind: CooldownType,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Cooldown {
    /// A `use` cooldown of `duration` seconds in `category`.
    pub fn new(category: impl Into<String>, duration: f32) -> Self {
        Self {
            category: category.into(),
            duration,
            kind: CooldownType::Use,
            extra: Extra::default(),
        }
    }
}

/// What starts a [`Cooldown`] (JSON `use` or `attack`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet)]
#[repr(u8)]
#[non_exhaustive]
pub enum CooldownType {
    /// Using the item (JSON `use`).
    #[default]
    #[facet(rename = "use")]
    Use,
    /// Attacking with the item (JSON `attack`).
    #[facet(rename = "attack")]
    Attack,
}

value_component! {
    /// `minecraft:damage`: the extra attack damage, 0–32767 (JSON `{"value": n}` or `n`).
    Damage(i32);
}

value_component! {
    /// `minecraft:swing_duration`: the swing animation's length in seconds, at least 0 (engine
    /// 1.21.120).
    SwingDuration(f32);
}

/// `minecraft:kinetic_weapon`: a charged attack that deals damage, knockback or dismounts by
/// the attacker's speed (engine 1.21.130). No member is required.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct KineticWeapon {
    /// Ticks before the effects apply (JSON `delay`).
    #[facet(default)]
    pub delay: Option<i32>,
    /// The reach in blocks (JSON `reach`).
    #[facet(default)]
    pub reach: Option<FloatRange>,
    /// The reach in creative mode (JSON `creative_reach`).
    #[facet(default)]
    pub creative_reach: Option<FloatRange>,
    /// The extra hit-box margin (JSON `hitbox_margin`).
    #[facet(default)]
    pub hitbox_margin: Option<f32>,
    /// The speed-to-damage factor (JSON `damage_multiplier`).
    #[facet(default)]
    pub damage_multiplier: Option<f32>,
    /// The flat damage added (JSON `damage_modifier`).
    #[facet(default)]
    pub damage_modifier: Option<f32>,
    /// When damage applies (JSON `damage_conditions`).
    #[facet(default)]
    pub damage_conditions: Option<KineticCondition>,
    /// When knockback applies (JSON `knockback_conditions`).
    #[facet(default)]
    pub knockback_conditions: Option<KineticCondition>,
    /// When the target is dismounted (JSON `dismount_conditions`).
    #[facet(default)]
    pub dismount_conditions: Option<KineticCondition>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One effect condition of a [`KineticWeapon`].
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct KineticCondition {
    /// How long the effect lasts, in ticks (JSON `max_duration`); 0 is an error.
    #[facet(default)]
    pub max_duration: Option<i32>,
    /// The attacker's minimum speed, at least 0 (JSON `min_speed`).
    #[facet(default)]
    pub min_speed: Option<f32>,
    /// The minimum speed relative to the target, at least 0 (JSON `min_relative_speed`);
    /// exclusive with `min_speed`.
    #[facet(default)]
    pub min_relative_speed: Option<f32>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:piercing_weapon`: an attack that hits every entity along its reach (engine
/// 1.21.130).
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct PiercingWeapon {
    /// The reach in blocks (JSON `reach`).
    #[facet(default)]
    pub reach: Option<FloatRange>,
    /// The reach in creative mode (JSON `creative_reach`).
    #[facet(default)]
    pub creative_reach: Option<FloatRange>,
    /// The extra hit-box margin (JSON `hitbox_margin`).
    #[facet(default)]
    pub hitbox_margin: Option<f32>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:projectile`: the item can be shot as `projectile_entity`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Projectile {
    /// The power above which the shot is critical (JSON `minimum_critical_power`).
    #[facet(default)]
    pub minimum_critical_power: Option<f32>,
    /// The entity shot (JSON `projectile_entity`, required).
    pub projectile_entity: EntityId,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:shooter`: the item shoots one of its `ammunition` items.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Shooter {
    /// The ammunition, in order of preference (JSON `ammunition`).
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub ammunition: Vec<ShooterAmmunition>,
    /// The longest draw in seconds (JSON `max_draw_duration`).
    #[facet(default)]
    pub max_draw_duration: Option<f32>,
    /// The item charges while drawn (JSON `charge_on_draw`).
    #[facet(default)]
    pub charge_on_draw: Option<bool>,
    /// The power grows with the draw (JSON `scale_power_by_draw_duration`).
    #[facet(default)]
    pub scale_power_by_draw_duration: Option<bool>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One ammunition entry of a [`Shooter`].
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct ShooterAmmunition {
    /// The ammunition item (JSON `item`, required).
    pub item: ItemDescriptor,
    /// Taken from the off hand (JSON `use_offhand`).
    #[facet(default)]
    pub use_offhand: Option<bool>,
    /// Searched for in the inventory (JSON `search_inventory`).
    #[facet(default)]
    pub search_inventory: Option<bool>,
    /// Needed in creative mode (JSON `use_in_creative`).
    #[facet(default)]
    pub use_in_creative: Option<bool>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl MolangFields for Shooter {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.enter("ammunition".into());
        for (i, ammunition) in self.ammunition.iter_mut().enumerate() {
            visitor.enter(i.into());
            visit_item_descriptor(&mut ammunition.item, visitor);
            visitor.leave();
        }
        visitor.leave();
    }
}

/// `minecraft:swing_sounds`: the sounds of a swing (engine 1.21.130). At least one member must
/// be set.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct SwingSounds {
    /// A swing that hits nothing (JSON `attack_miss`).
    #[facet(default)]
    pub attack_miss: Option<SoundEventName>,
    /// A swing that hits (JSON `attack_hit`).
    #[facet(default)]
    pub attack_hit: Option<SoundEventName>,
    /// A critical hit (JSON `attack_critical_hit`).
    #[facet(default)]
    pub attack_critical_hit: Option<SoundEventName>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:throwable`: the item can be thrown.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Throwable {
    /// The swing animation plays (JSON `do_swing_animation`).
    #[facet(default)]
    pub do_swing_animation: Option<bool>,
    /// The launch power scale (JSON `launch_power_scale`).
    #[facet(default)]
    pub launch_power_scale: Option<f32>,
    /// The longest draw in seconds (JSON `max_draw_duration`).
    #[facet(default)]
    pub max_draw_duration: Option<f32>,
    /// The greatest launch power (JSON `max_launch_power`).
    #[facet(default)]
    pub max_launch_power: Option<f32>,
    /// The shortest draw in seconds (JSON `min_draw_duration`).
    #[facet(default)]
    pub min_draw_duration: Option<f32>,
    /// The power grows with the draw (JSON `scale_power_by_draw_duration`).
    #[facet(default)]
    pub scale_power_by_draw_duration: Option<bool>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}
