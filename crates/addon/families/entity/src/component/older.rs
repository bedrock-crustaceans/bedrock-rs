//! Components as older format versions define them, where the shape differs from the newest:
//! the version chain reads an older document's component with these, then converts it with
//! `From` into the newest type.
//!
//! - [`ApplyKnockbackRulesV1_8_0`]: `apply_knockback_rules` from 1.8.0 to 1.26.20, whose
//!   presets lack the 1.26.30 members.
//! - [`PushableByEntityV1_26_20`]: `pushable_by_entity` in 1.26.20, without members.
//! - [`PushableByEntityV1_26_30`]: `pushable_by_entity` in 1.26.30, whose presets lack
//!   `require_collision_overlap`.
//!
//! A member a newer version added is unknown to the older shape: written in an older document,
//! it is kept in `extra` and reported as `UnrecognizedField`.

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{JsonSlice, StrictBool, StrictFloat};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::shared::Filter;
use bedrock_addon_core::versioning::ConvertMode;
use facet::Facet;

use super::parse::{Check, Typed, index, join, path, read};
use super::push::{
    ApplyKnockbackRules, ExtraKnockbackApproach, KnockbackMode, KnockbackPreset, PushMode,
    PushPreset, PushableByEntity,
};

/// `minecraft:apply_knockback_rules` from 1.8.0 to 1.26.20.
#[doc(alias = "minecraft:apply_knockback_rules")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct ApplyKnockbackRulesV1_8_0 {
    /// `presets`: the presets, in order.
    #[facet(default)]
    pub presets: Vec<KnockbackPresetV1_8_0>,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One preset of [`ApplyKnockbackRulesV1_8_0`].
// The shape follows the game's 1.21.80 `minecraft_apply_knockback_rules_instance.json`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct KnockbackPresetV1_8_0 {
    /// `filter`: which targets the preset applies to; absent means all.
    #[facet(default)]
    pub filter: Option<Filter>,
    /// `horizontal_power`: default 1.
    #[facet(default = StrictFloat(1.0_f32))]
    pub horizontal_power: StrictFloat<f32>,
    /// `vertical_power`: default 0.4.
    #[facet(default = StrictFloat(0.4_f32))]
    pub vertical_power: StrictFloat<f32>,
    /// `vertical_velocity_cap`: default 0.4.
    #[facet(default = StrictFloat(0.4_f32))]
    pub vertical_velocity_cap: StrictFloat<f32>,
    /// `check_if_target_is_immersed_in_water`: default `false`.
    #[facet(default = StrictBool(false))]
    pub check_if_target_is_immersed_in_water: StrictBool,
    /// Members the preset does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for KnockbackPresetV1_8_0 {
    fn default() -> Self {
        Self {
            filter: None,
            horizontal_power: StrictFloat(1.0),
            vertical_power: StrictFloat(0.4),
            vertical_velocity_cap: StrictFloat(0.4),
            check_if_target_is_immersed_in_water: StrictBool(false),
            extra: Extra::default(),
        }
    }
}

impl ApplyKnockbackRulesV1_8_0 {
    /// Reads the component from its slice in `mode` (the parse the older versions' tables
    /// use) for a file loaded in `cx`; findings relative to the component.
    #[doc(hidden)]
    pub fn read(
        slice: JsonSlice<'_>,
        mode: ConvertMode,
        cx: &LoadContext,
        d: &mut Diagnostics,
    ) -> Option<Self> {
        read::<Self>(slice, mode, cx, d)
    }
}

impl Typed for ApplyKnockbackRulesV1_8_0 {
    fn check(&mut self, cx: &mut Check<'_>) {
        // No minimum on `presets` here: vanilla's floor bind has no container constraint.
        let at = path(&["presets"]);
        for (i, preset) in self.presets.iter().enumerate() {
            if let Some(filter) = &preset.filter {
                cx.extend(filter.check(&join(&index(&at, i), "filter")));
            }
        }
    }
}

/// The 1.26.30 upgrade: every preset keeps its members and gets the new ones at the values that
/// preserve the old behaviour (`reapply_default`, `relative_horizontal`, no damage scaling,
/// the default slowdown).
// `apply_knockback_rules` has no custom upgrade hook; the struct defaults apply.
impl From<ApplyKnockbackRulesV1_8_0> for ApplyKnockbackRules {
    fn from(old: ApplyKnockbackRulesV1_8_0) -> Self {
        Self {
            presets: old
                .presets
                .into_iter()
                .map(|p| KnockbackPreset {
                    filter: p.filter,
                    horizontal_power: p.horizontal_power,
                    vertical_power: p.vertical_power,
                    vertical_velocity_cap: p.vertical_velocity_cap,
                    check_if_target_is_immersed_in_water: p.check_if_target_is_immersed_in_water,
                    extra_knockback_approach: ExtraKnockbackApproach::ReapplyDefault,
                    knockback_mode: KnockbackMode::RelativeHorizontal,
                    scale_with_damage: StrictBool(false),
                    slowdown_scale: StrictFloat(0.5),
                    extra: p.extra,
                })
                .collect(),
            extra: old.extra,
        }
    }
}

/// `minecraft:pushable_by_entity` in 1.26.20: other entities push this one; no members.
#[doc(alias = "minecraft:pushable_by_entity")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct PushableByEntityV1_26_20 {
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl PushableByEntityV1_26_20 {
    /// Reads the component from its slice in `mode`; findings relative to the component.
    #[doc(hidden)]
    pub fn read(
        slice: JsonSlice<'_>,
        mode: ConvertMode,
        cx: &LoadContext,
        d: &mut Diagnostics,
    ) -> Option<Self> {
        read::<Self>(slice, mode, cx, d)
    }
}

impl Typed for PushableByEntityV1_26_20 {}

/// The 1.26.30 upgrade: the empty component becomes `{presets: []}`, the built-in behaviour.
// The game's upgrade hook returns without writing the destination.
impl From<PushableByEntityV1_26_20> for PushableByEntityV1_26_30 {
    fn from(old: PushableByEntityV1_26_20) -> Self {
        Self {
            presets: Vec::new(),
            extra: old.extra,
        }
    }
}

/// `minecraft:pushable_by_entity` in 1.26.30.
#[doc(alias = "minecraft:pushable_by_entity")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct PushableByEntityV1_26_30 {
    /// `presets`: the presets, in order.
    #[facet(default)]
    pub presets: Vec<PushPresetV1_26_30>,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One preset of [`PushableByEntityV1_26_30`]: [`PushPreset`] without
/// `require_collision_overlap`.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct PushPresetV1_26_30 {
    /// `filters`: absent means all.
    #[facet(default)]
    pub filters: Option<Filter>,
    /// `push_mode`: default `default`.
    #[facet(default = PushMode::Default)]
    pub push_mode: PushMode,
    /// `strength_multiplier`: default 0.05.
    #[facet(default = StrictFloat(0.05_f32))]
    pub strength_multiplier: StrictFloat<f32>,
    /// `min_distance`: default 0.01.
    #[facet(default = StrictFloat(0.01_f32))]
    pub min_distance: StrictFloat<f32>,
    /// `max_distance`: default the largest `f32`.
    #[facet(default = StrictFloat(f32::MAX))]
    pub max_distance: StrictFloat<f32>,
    /// `push_scale_self`: default 1.
    #[facet(default = StrictFloat(1.0_f32))]
    pub push_scale_self: StrictFloat<f32>,
    /// `push_scale_other`: default 1.
    #[facet(default = StrictFloat(1.0_f32))]
    pub push_scale_other: StrictFloat<f32>,
    /// `kick_speed_scale`: default 2.
    #[facet(default = StrictFloat(2.0_f32))]
    pub kick_speed_scale: StrictFloat<f32>,
    /// `min_kick_speed`: default 0.
    #[facet(default = StrictFloat(0.0_f32))]
    pub min_kick_speed: StrictFloat<f32>,
    /// `max_kick_speed`: default 0.5.
    #[facet(default = StrictFloat(0.5_f32))]
    pub max_kick_speed: StrictFloat<f32>,
    /// `vertical_kick_multiplier`: default 0.3.
    #[facet(default = StrictFloat(0.3_f32))]
    pub vertical_kick_multiplier: StrictFloat<f32>,
    /// `play_sound`: default `false`.
    #[facet(default = StrictBool(false))]
    pub play_sound: StrictBool,
    /// `play_sound_cooldown_in_seconds`: default 0.2.
    #[facet(default = StrictFloat(0.2_f32))]
    pub play_sound_cooldown_in_seconds: StrictFloat<f32>,
    /// `play_sound_impulse_threshold`: default 0.2.
    #[facet(default = StrictFloat(0.2_f32))]
    pub play_sound_impulse_threshold: StrictFloat<f32>,
    /// Members the preset does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for PushPresetV1_26_30 {
    fn default() -> Self {
        let new = PushPreset::default();
        Self {
            filters: new.filters,
            push_mode: new.push_mode,
            strength_multiplier: new.strength_multiplier,
            min_distance: new.min_distance,
            max_distance: new.max_distance,
            push_scale_self: new.push_scale_self,
            push_scale_other: new.push_scale_other,
            kick_speed_scale: new.kick_speed_scale,
            min_kick_speed: new.min_kick_speed,
            max_kick_speed: new.max_kick_speed,
            vertical_kick_multiplier: new.vertical_kick_multiplier,
            play_sound: new.play_sound,
            play_sound_cooldown_in_seconds: new.play_sound_cooldown_in_seconds,
            play_sound_impulse_threshold: new.play_sound_impulse_threshold,
            extra: new.extra,
        }
    }
}

impl PushableByEntityV1_26_30 {
    /// Reads the component from its slice in `mode`; findings relative to the component.
    #[doc(hidden)]
    pub fn read(
        slice: JsonSlice<'_>,
        mode: ConvertMode,
        cx: &LoadContext,
        d: &mut Diagnostics,
    ) -> Option<Self> {
        read::<Self>(slice, mode, cx, d)
    }
}

impl Typed for PushableByEntityV1_26_30 {
    fn check(&mut self, cx: &mut Check<'_>) {
        let at = path(&["presets"]);
        for (i, preset) in self.presets.iter().enumerate() {
            if let Some(filters) = &preset.filters {
                let at: JsonPath = join(&index(&at, i), "filters");
                cx.extend(filters.check(&at));
            }
        }
    }
}

/// The 1.26.40 upgrade: every
/// carried preset gets `require_collision_overlap: false`, and a component without presets
/// gets `use_default_with_overlap_check: false`, which keep the 1.26.30 behaviour.
impl From<PushableByEntityV1_26_30> for PushableByEntity {
    fn from(old: PushableByEntityV1_26_30) -> Self {
        let use_default_with_overlap_check = old.presets.is_empty().then_some(StrictBool(false));
        Self {
            presets: old
                .presets
                .into_iter()
                .map(|p| PushPreset {
                    filters: p.filters,
                    push_mode: p.push_mode,
                    strength_multiplier: p.strength_multiplier,
                    min_distance: p.min_distance,
                    max_distance: p.max_distance,
                    push_scale_self: p.push_scale_self,
                    push_scale_other: p.push_scale_other,
                    kick_speed_scale: p.kick_speed_scale,
                    min_kick_speed: p.min_kick_speed,
                    max_kick_speed: p.max_kick_speed,
                    vertical_kick_multiplier: p.vertical_kick_multiplier,
                    play_sound: p.play_sound,
                    play_sound_cooldown_in_seconds: p.play_sound_cooldown_in_seconds,
                    play_sound_impulse_threshold: p.play_sound_impulse_threshold,
                    require_collision_overlap: StrictBool(false),
                    extra: p.extra,
                })
                .collect(),
            use_default_with_overlap_check,
            extra: old.extra,
        }
    }
}
