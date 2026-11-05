//! `wither_target_highest_damage`, and the mob descriptors (`entity_types`) and goal control
//! flags it shares with the targeting goals.
//!
//! `wither_target_highest_damage` is typed from 1.26.30 and derived from the goal of the same
//! name; every name of the target goal is deprecated and `entity_types` is bound again. Types and
//! defaults follow the game's 1.26.30 schema and the 1.21.120 mob descriptor.

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::shared::Filter;
use facet::Facet;
use facet_value::VObject;

use super::parse::{ComponentCheck, Typed, drop_deprecated};

/// `minecraft:wither_target_highest_damage`: the wither targets the entity that dealt it the
/// most damage.
#[doc(alias = "minecraft:wither_target_highest_damage")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct WitherTargetHighestDamage {
    /// `priority`: the goal priority, lower runs first; at least 0, default 0.
    #[facet(default = 0_i32, schema::min = 0)]
    pub priority: i32,
    /// `control_flags`: the controls the goal takes while it runs.
    #[facet(default)]
    pub control_flags: Vec<ControlFlag>,
    /// `entity_types`: the entities it may target.
    #[facet(default = EntityTypes::default())]
    pub entity_types: EntityTypes,
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
    fn prepare(json: &mut VObject, cx: &mut ComponentCheck<'_>) -> bool {
        drop_deprecated(json, &DEPRECATED_TARGET_GOAL_NAMES, cx)
    }

    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        cx.bounds(JsonPath::new(), self);
        self.entity_types
            .check(&JsonPath::new().child("entity_types"), cx);
    }
}

/// A goal's control flag: what the goal takes control of while it runs.
///
/// The names of [`goal::ControlFlag`](crate::goal::ControlFlag), but read by facet's enum: a
/// strict read tells an unknown name as `InvalidEnumValue` at the list (or at the element
/// before it), where the goal's tells `InvalidType` at the element; `tests/entity/descriptors.rs`
/// pins both.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet, Schema)]
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

/// The `entity_types` of a targeting goal: one [`MobDescriptor`] or an array of them.
pub type EntityTypes = crate::EntityTypesOf<MobDescriptor>;

impl EntityTypes {
    /// The findings of every descriptor's filters, under `at`.
    pub(crate) fn check(
        &self,
        at: &bedrock_addon_core::diagnostic::JsonPath,
        cx: &mut ComponentCheck<'_>,
    ) {
        match self {
            Self::Many(list) => {
                for (i, one) in list.iter().enumerate() {
                    one.check(&at.index(i), cx);
                }
            }
            Self::One(one) => one.check(at, cx),
        }
    }
}

/// Which entities a goal applies to, and how (the game's mob descriptor).
///
/// [`goal::MobDescriptor`](crate::goal::MobDescriptor) is the same descriptor read differently,
/// kept apart until a capture settles it: here the members are plain with their defaults, which
/// are written back, and `must_see_forget_duration` is an `i32` (2.5 is refused, and a lenient
/// read drops the whole descriptor); the goal's is an `f32`, the type the game's schema gives
/// the goal member. No capture has the member inside a descriptor (`zombie_nautilus.json` has
/// it as a `nearest_attackable_target` member, `17`). `tests/entity/descriptors.rs` pins both.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MobDescriptor {
    /// `check_if_outnumbered`: only when the entity is outnumbered; default `false`.
    #[facet(default = false)]
    pub check_if_outnumbered: bool,
    /// `cooldown`: seconds before the goal may pick this type again; default 0.
    #[facet(default = 0_i32)]
    pub cooldown: i32,
    /// `filters`: which entities match; absent means all.
    pub filters: Option<Filter>,
    /// `max_dist`: the largest distance to a matching entity; default 16.
    #[facet(default = 16.0_f32)]
    pub max_dist: f32,
    /// `max_flee`: the distance fled from a matching entity; default 10.
    #[facet(default = 10.0_f32)]
    pub max_flee: f32,
    /// `max_height`: the largest height difference; −1 (the default) is no limit.
    #[facet(default = -1.0_f32)]
    pub max_height: f32,
    /// `must_see`: the entity must be seen; absent means the goal's own setting.
    pub must_see: Option<bool>,
    /// `must_see_forget_duration`: seconds an unseen entity is still targeted; default 3.
    #[facet(default = 3_i32)]
    pub must_see_forget_duration: i32,
    /// `priority`: the order among the descriptors; default 0.
    #[facet(default = 0_i32)]
    pub priority: i32,
    /// `reevaluate_description`: the descriptor is checked again while the goal runs; default
    /// `false`.
    #[facet(default = false)]
    pub reevaluate_description: bool,
    /// `sprint_speed_multiplier`: the speed multiplier while sprinting; default 1.
    #[facet(default = 1.0_f32)]
    pub sprint_speed_multiplier: f32,
    /// `walk_speed_multiplier`: the speed multiplier while walking; default 1.
    #[facet(default = 1.0_f32)]
    pub walk_speed_multiplier: f32,
}

impl MobDescriptor {
    /// The findings of the descriptor's filters, under `at`.
    fn check(&self, at: &bedrock_addon_core::diagnostic::JsonPath, cx: &mut ComponentCheck<'_>) {
        if let Some(filters) = &self.filters {
            cx.extend(filters.check(&at.child("filters")));
        }
    }
}

bedrock_addon_core::impl_schema_leaf!(EntityTypes);
bedrock_addon_core::impl_no_molang!(EntityTypes);
