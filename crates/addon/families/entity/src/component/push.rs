//! How the entity pushes and is pushed: `apply_knockback_rules` and `pushable_by_entity`.
//! Types and defaults follow the game's published knockback and push-preset schemas. The
//! older shapes are in `older.rs`.

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::shared::Filter;
use facet::Facet;

use super::parse::{ComponentCheck, Typed};

/// `minecraft:apply_knockback_rules`: the knockback the entity deals, by preset; the first
/// preset whose filter passes applies.
#[doc(alias = "minecraft:apply_knockback_rules")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ApplyKnockbackRules {
    /// `presets`: the presets, in order; at least one.
    #[facet(default)]
    pub presets: Vec<KnockbackPreset>,
}

impl ApplyKnockbackRules {
    /// The rules with one preset.
    pub fn with_preset(preset: KnockbackPreset) -> Self {
        Self {
            presets: vec![preset],
        }
    }
}

impl Typed for ApplyKnockbackRules {
    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        let at = JsonPath::new().child("presets");
        if self.presets.is_empty() {
            // The game accepts an empty list; the crate's own check refuses it.
            cx.broken(&at);
        }
        for (i, preset) in self.presets.iter_mut().enumerate() {
            preset.check(&at.index(i), cx);
        }
    }
}

/// One preset of [`ApplyKnockbackRules`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct KnockbackPreset {
    /// `filter`: which targets the preset applies to; absent means all.
    pub filter: Option<Filter>,
    /// `horizontal_power`: the horizontal strength; default 1.
    #[facet(default = 1.0_f32)]
    pub horizontal_power: f32,
    /// `vertical_power`: the vertical strength; default 0.4.
    #[facet(default = 0.4_f32)]
    pub vertical_power: f32,
    /// `vertical_velocity_cap`: the largest vertical velocity it gives; default 0.4.
    #[facet(default = 0.4_f32)]
    pub vertical_velocity_cap: f32,
    /// `check_if_target_is_immersed_in_water`: only targets in water; default `false`.
    #[facet(default = false)]
    pub check_if_target_is_immersed_in_water: bool,
    /// `extra_knockback_approach`: how knockback beyond the default is applied; default
    /// `reapply_default`.
    #[facet(default = ExtraKnockbackApproach::ReapplyDefault)]
    pub extra_knockback_approach: ExtraKnockbackApproach,
    /// `knockback_mode`: which direction the knockback takes; default `relative_horizontal`.
    #[facet(default = KnockbackMode::RelativeHorizontal)]
    pub knockback_mode: KnockbackMode,
    /// `scale_with_damage`: the knockback grows with the damage dealt; default `false`.
    #[facet(default = false)]
    pub scale_with_damage: bool,
    /// `slowdown_scale`: how much the attacker slows down, 0 to 1; default 0.5.
    #[facet(default = 0.5_f32)]
    pub slowdown_scale: f32,
}

impl KnockbackPreset {
    /// A preset for every target with these powers, the 1.26.30 members at their defaults.
    pub fn new(
        horizontal_power: f32,
        vertical_power: f32,
        vertical_velocity_cap: f32,
        check_if_target_is_immersed_in_water: bool,
    ) -> Self {
        Self {
            filter: None,
            horizontal_power,
            vertical_power,
            vertical_velocity_cap,
            check_if_target_is_immersed_in_water,
            extra_knockback_approach: ExtraKnockbackApproach::ReapplyDefault,
            knockback_mode: KnockbackMode::RelativeHorizontal,
            scale_with_damage: false,
            slowdown_scale: 0.5,
        }
    }

    /// The preset's rule checks, under `at`.
    fn check(
        &mut self,
        at: &bedrock_addon_core::diagnostic::JsonPath,
        cx: &mut ComponentCheck<'_>,
    ) {
        if let Some(filter) = &self.filter {
            cx.extend(filter.check(&at.child("filter"), cx.unknown()));
        }
        // The game's schema: `slowdown_scale` minimum 0, maximum 1.
        cx.within(
            &at.child("slowdown_scale"),
            &mut self.slowdown_scale,
            (0.0, 1.0),
            0.5,
        );
    }
}

/// How a [`KnockbackPreset`] applies knockback beyond the default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[repr(u8)]
#[non_exhaustive]
pub enum ExtraKnockbackApproach {
    /// The default knockback is applied again (JSON `reapply_default`).
    #[default]
    #[facet(rename = "reapply_default")]
    ReapplyDefault,
    /// The reduced knockback is multiplied (JSON `multiply_reduced`).
    #[facet(rename = "multiply_reduced")]
    MultiplyReduced,
}

/// The direction of a [`KnockbackPreset`]'s knockback.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[repr(u8)]
#[non_exhaustive]
pub enum KnockbackMode {
    /// Away from the attacker, horizontally (JSON `relative_horizontal`).
    #[default]
    #[facet(rename = "relative_horizontal")]
    RelativeHorizontal,
    /// Along the hit's direction (JSON `hit_direction`).
    #[facet(rename = "hit_direction")]
    HitDirection,
}

/// `minecraft:pushable_by_entity`: other entities push this one, by preset; without presets
/// the built-in push behaviour applies.
#[doc(alias = "minecraft:pushable_by_entity")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct PushableByEntity {
    /// `presets`: the presets, in order; the first whose filters pass applies.
    #[facet(default)]
    pub presets: Vec<PushPreset>,
    /// `use_default_with_overlap_check` (not in the published schema): the built-in behaviour
    /// checks for overlap; absent means `true`. The 1.26.40 upgrade sets it to `false` for a
    /// component without presets.
    pub use_default_with_overlap_check: Option<bool>,
}

impl PushableByEntity {
    /// Whether the built-in behaviour checks for overlap (`use_default_with_overlap_check`,
    /// default `true`).
    pub fn uses_default_with_overlap_check(&self) -> bool {
        self.use_default_with_overlap_check.is_none_or(|b| b)
    }
}

impl Typed for PushableByEntity {
    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        let at = JsonPath::new().child("presets");
        for (i, preset) in self.presets.iter_mut().enumerate() {
            preset.check(&at.index(i), cx);
        }
    }
}

/// One preset of [`PushableByEntity`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct PushPreset {
    /// `filters`: which pushing entities the preset applies to; absent means all.
    pub filters: Option<Filter>,
    /// `push_mode`: how the entity moves when pushed; default `default`.
    #[facet(default = PushMode::Default)]
    pub push_mode: PushMode,
    /// `strength_multiplier`: the push strength; default 0.05.
    #[facet(default = 0.05_f32)]
    pub strength_multiplier: f32,
    /// `min_distance`: the distance below which the push is strongest; at least 0, default
    /// 0.01.
    #[facet(default = 0.01_f32)]
    pub min_distance: f32,
    /// `max_distance`: the distance beyond which nothing pushes; at least 0, default the
    /// largest `f32`.
    #[facet(default = f32::MAX)]
    pub max_distance: f32,
    /// `push_scale_self`: how much of the push this entity takes; default 1.
    #[facet(default = 1.0_f32)]
    pub push_scale_self: f32,
    /// `push_scale_other`: how much of the push the other entity takes; default 1.
    #[facet(default = 1.0_f32)]
    pub push_scale_other: f32,
    /// `kick_speed_scale`: the kick speed scale (`ball`); default 2.
    #[facet(default = 2.0_f32)]
    pub kick_speed_scale: f32,
    /// `min_kick_speed`: the lowest kick speed (`ball`); default 0.
    #[facet(default = 0.0_f32)]
    pub min_kick_speed: f32,
    /// `max_kick_speed`: the highest kick speed (`ball`); default 0.5.
    #[facet(default = 0.5_f32)]
    pub max_kick_speed: f32,
    /// `vertical_kick_multiplier`: the vertical part of a kick (`ball`); default 0.3.
    #[facet(default = 0.3_f32)]
    pub vertical_kick_multiplier: f32,
    /// `play_sound`: a sound plays when pushed; default `false`.
    #[facet(default = false)]
    pub play_sound: bool,
    /// `play_sound_cooldown_in_seconds`: the least time between sounds; at least 0, default
    /// 0.2.
    #[facet(default = 0.2_f32)]
    pub play_sound_cooldown_in_seconds: f32,
    /// `play_sound_impulse_threshold`: the least impulse that plays the sound; at least 0,
    /// default 0.2.
    #[facet(default = 0.2_f32)]
    pub play_sound_impulse_threshold: f32,
    /// `require_collision_overlap`: the push needs the collision boxes to overlap; default
    /// `true`.
    #[facet(default = true)]
    pub require_collision_overlap: bool,
}

impl PushPreset {
    /// The preset's rule checks, under `at`.
    fn check(
        &mut self,
        at: &bedrock_addon_core::diagnostic::JsonPath,
        cx: &mut ComponentCheck<'_>,
    ) {
        if let Some(filters) = &self.filters {
            cx.extend(filters.check(&at.child("filters"), cx.unknown()));
        }
        // The game's schema minimums.
        cx.at_least(&at.child("min_distance"), &mut self.min_distance, 0.0, 0.01);
        cx.at_least(
            &at.child("max_distance"),
            &mut self.max_distance,
            0.0,
            f32::MAX,
        );
        cx.at_least(
            &at.child("play_sound_cooldown_in_seconds"),
            &mut self.play_sound_cooldown_in_seconds,
            0.0,
            0.2,
        );
        cx.at_least(
            &at.child("play_sound_impulse_threshold"),
            &mut self.play_sound_impulse_threshold,
            0.0,
            0.2,
        );
    }
}

/// How an entity moves when a [`PushPreset`] pushes it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[repr(u8)]
#[non_exhaustive]
pub enum PushMode {
    /// Not pushed (JSON `none`).
    #[facet(rename = "none")]
    None,
    /// The default push (JSON `default`).
    #[default]
    #[facet(rename = "default")]
    Default,
    /// The boat's push from before 1.26.30 (JSON `legacy_boat`).
    #[facet(rename = "legacy_boat")]
    LegacyBoat,
    /// The minecart's push from before 1.26.30 (JSON `legacy_minecart`).
    #[facet(rename = "legacy_minecart")]
    LegacyMinecart,
    /// Kicked like a ball (JSON `ball`).
    #[facet(rename = "ball")]
    Ball,
}
