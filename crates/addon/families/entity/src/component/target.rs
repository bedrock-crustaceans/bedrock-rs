//! `wither_target_highest_damage`, and the mob descriptors (`entity_types`) and goal control
//! flags it shares with the targeting goals.
//!
//! `wither_target_highest_damage` is typed from 1.26.30 and derived from the goal of the same
//! name; every name of the target goal is deprecated and `entity_types` is bound again. Types and
//! defaults follow the game's 1.26.30 schema and the 1.21.120 mob descriptor.

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{StrictBool, StrictFloat, StrictInt};
use bedrock_addon_core::shared::Filter;
use facet::Facet;
use facet_value::VObject;

use super::parse::{Check, Typed, drop_deprecated, index, join, path};

/// `minecraft:wither_target_highest_damage`: the wither targets the entity that dealt it the
/// most damage.
#[doc(alias = "minecraft:wither_target_highest_damage")]
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct WitherTargetHighestDamage {
    /// `priority`: the goal priority, lower runs first; at least 0, default 0.
    #[facet(default = StrictInt(0_i32))]
    pub priority: StrictInt<i32>,
    /// `control_flags`: the controls the goal takes while it runs.
    #[facet(default)]
    pub control_flags: Vec<ControlFlag>,
    /// `entity_types`: the entities it may target.
    #[facet(default = EntityTypes::default())]
    pub entity_types: EntityTypes,
    /// Members the component does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// The target goal's member names `wither_target_highest_damage` inherits and deprecates.
const DEPRECATED_TARGET_GOAL_NAMES: [&str; 6] = [
    "attack_owner",
    "must_reach",
    "must_see",
    "must_see_forget_duration",
    "persist_time",
    "within_radius",
];

impl Typed for WitherTargetHighestDamage {
    fn prepare(json: &mut VObject, cx: &mut Check<'_>) -> bool {
        drop_deprecated(json, &DEPRECATED_TARGET_GOAL_NAMES, cx)
    }

    fn check(&mut self, cx: &mut Check<'_>) {
        cx.at_least(&path(&["priority"]), &mut self.priority.0, 0, 0);
        self.entity_types.check(&path(&["entity_types"]), cx);
    }
}

/// A goal's control flag: what the goal takes control of while it runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[repr(u8)]
#[non_exhaustive]
pub enum ControlFlag {
    /// Movement (JSON `move`).
    #[facet(rename = "move")]
    Move,
    /// Looking (JSON `look`).
    #[facet(rename = "look")]
    Look,
    /// Jumping (JSON `jump`).
    #[facet(rename = "jump")]
    Jump,
}

/// The `entity_types` of a targeting goal: one mob descriptor or an array of them.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
#[allow(
    clippy::large_enum_variant,
    reason = "unboxed so the unknown-member walk keeps the object form's members"
)]
pub enum EntityTypes {
    /// An array of descriptors.
    Many(Vec<MobDescriptor>),
    /// One descriptor, written as an object.
    One(MobDescriptor),
}

impl Default for EntityTypes {
    fn default() -> Self {
        Self::Many(Vec::new())
    }
}

impl EntityTypes {
    /// The descriptors, in order.
    #[must_use]
    pub fn as_slice(&self) -> &[MobDescriptor] {
        match self {
            Self::Many(list) => list,
            Self::One(one) => std::slice::from_ref(one),
        }
    }

    /// The findings of every descriptor's filters, under `at`.
    pub(crate) fn check(&self, at: &bedrock_addon_core::diagnostic::JsonPath, cx: &mut Check<'_>) {
        match self {
            Self::Many(list) => {
                for (i, one) in list.iter().enumerate() {
                    one.check(&index(at, i), cx);
                }
            }
            Self::One(one) => one.check(at, cx),
        }
    }
}

/// Which entities a goal applies to, and how (the game's mob descriptor).
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MobDescriptor {
    /// `check_if_outnumbered`: only when the entity is outnumbered; default `false`.
    #[facet(default = StrictBool(false))]
    pub check_if_outnumbered: StrictBool,
    /// `cooldown`: seconds before the goal may pick this type again; default 0.
    #[facet(default = StrictInt(0_i32))]
    pub cooldown: StrictInt<i32>,
    /// `filters`: which entities match; absent means all.
    #[facet(default)]
    pub filters: Option<Filter>,
    /// `max_dist`: the largest distance to a matching entity; default 16.
    #[facet(default = StrictFloat(16.0_f32))]
    pub max_dist: StrictFloat<f32>,
    /// `max_flee`: the distance fled from a matching entity; default 10.
    #[facet(default = StrictFloat(10.0_f32))]
    pub max_flee: StrictFloat<f32>,
    /// `max_height`: the largest height difference; −1 (the default) is no limit.
    #[facet(default = StrictFloat(-1.0_f32))]
    pub max_height: StrictFloat<f32>,
    /// `must_see`: the entity must be seen; absent means the goal's own setting.
    #[facet(default)]
    pub must_see: Option<StrictBool>,
    /// `must_see_forget_duration`: seconds an unseen entity is still targeted; default 3.
    #[facet(default = StrictInt(3_i32))]
    pub must_see_forget_duration: StrictInt<i32>,
    /// `priority`: the order among the descriptors; default 0.
    #[facet(default = StrictInt(0_i32))]
    pub priority: StrictInt<i32>,
    /// `reevaluate_description`: the descriptor is checked again while the goal runs; default
    /// `false`.
    #[facet(default = StrictBool(false))]
    pub reevaluate_description: StrictBool,
    /// `sprint_speed_multiplier`: the speed multiplier while sprinting; default 1.
    #[facet(default = StrictFloat(1.0_f32))]
    pub sprint_speed_multiplier: StrictFloat<f32>,
    /// `walk_speed_multiplier`: the speed multiplier while walking; default 1.
    #[facet(default = StrictFloat(1.0_f32))]
    pub walk_speed_multiplier: StrictFloat<f32>,
    /// Members the descriptor does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Default for MobDescriptor {
    fn default() -> Self {
        Self {
            check_if_outnumbered: StrictBool(false),
            cooldown: StrictInt(0),
            filters: None,
            max_dist: StrictFloat(16.0),
            max_flee: StrictFloat(10.0),
            max_height: StrictFloat(-1.0),
            must_see: None,
            must_see_forget_duration: StrictInt(3),
            priority: StrictInt(0),
            reevaluate_description: StrictBool(false),
            sprint_speed_multiplier: StrictFloat(1.0),
            walk_speed_multiplier: StrictFloat(1.0),
            extra: Extra::default(),
        }
    }
}

impl MobDescriptor {
    /// The findings of the descriptor's filters, under `at`.
    fn check(&self, at: &bedrock_addon_core::diagnostic::JsonPath, cx: &mut Check<'_>) {
        if let Some(filters) = &self.filters {
            cx.extend(filters.check(&join(at, "filters")));
        }
    }
}
