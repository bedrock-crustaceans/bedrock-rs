//! `projectile`: how the entity flies when shot, and what happens when it hits.
//!
//! The component is typed from 1.26.50 with `spread_fire` and without
//! `owner_launch_immunity_ticks`, and has the sub-structures `on_hit`, `impact_damage`,
//! `anchor` and `should_bounce`; types and defaults follow the game's 1.26.50
//! `minecraft_projectile` schema. Below 1.26.50 the key is `Dynamic`, read by the game's untyped
//! projectile reader, whose camelCase aliases the lenient conversion maps.

use crate::codes;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{DiagnosticCode, JsonPath, Severity};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::shared::{EventTrigger, Filter, FloatRange};
use bedrock_addon_core::versioning::ConvertMode;
use facet::Facet;
use facet_value::VObject;
use indexmap::IndexMap;

use super::parse::{ComponentCheck, Typed};

/// `minecraft:projectile`: the entity is a projectile.
#[allow(
    clippy::struct_excessive_bools,
    reason = "the game's flags, one member each"
)]
#[doc(alias = "minecraft:projectile")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Projectile {
    /// `anchor`: where on the shooter the projectile starts; default `origin`.
    #[facet(default = ProjectileAnchor::Origin)]
    pub anchor: ProjectileAnchor,
    /// `angle_offset`: the angle added to the shot's pitch; default 0.
    #[facet(default = 0.0_f32)]
    pub angle_offset: f32,
    /// `catch_fire`: the projectile sets what it hits on fire; default `false`.
    #[facet(default = false)]
    pub catch_fire: bool,
    /// `crit_particle_on_hurt`: critical-hit particles on hurt; default `false`.
    #[facet(default = false)]
    pub crit_particle_on_hurt: bool,
    /// `destroy_on_hurt`: the projectile is destroyed when hurt; default `false`.
    #[facet(default = false)]
    pub destroy_on_hurt: bool,
    /// `gravity`: the downward acceleration; default 0.05.
    #[facet(default = 0.05_f32)]
    pub gravity: f32,
    /// `hit_ground_sound`: the sound when it hits a block; default none (empty).
    #[facet(default = String::new())]
    pub hit_ground_sound: String,
    /// `hit_nearest_passenger`: it hits the nearest passenger of a vehicle; default `false`.
    #[facet(default = false)]
    pub hit_nearest_passenger: bool,
    /// `hit_sound`: the sound when it hits an entity; default none (empty).
    #[facet(default = String::new())]
    pub hit_sound: String,
    /// `hit_water`: it collides with water; default `false`.
    #[facet(default = false)]
    pub hit_water: bool,
    /// `homing`: it homes in on its target; default `false`.
    #[facet(default = false)]
    pub homing: bool,
    /// `ignored_entities`: entity identifiers it passes through.
    #[facet(default)]
    pub ignored_entities: Vec<String>,
    /// `inertia`: the velocity kept per tick in air; default 0.99.
    #[facet(default = 0.99_f32)]
    pub inertia: f32,
    /// `is_dangerous`: entities try to dodge it; default `false`.
    #[facet(default = false)]
    pub is_dangerous: bool,
    /// `isolated_physics`: its physics ignore other projectiles; default `true`.
    #[facet(default = true)]
    pub isolated_physics: bool,
    /// `lightning`: lightning strikes where it hits during a thunderstorm; default `false`.
    #[facet(default = false)]
    pub lightning: bool,
    /// `liquid_inertia`: the velocity kept per tick in liquids; default 0.6.
    #[facet(default = 0.6_f32)]
    pub liquid_inertia: f32,
    /// `multiple_targets`: it can hit several targets; default `false`.
    #[facet(default = false)]
    pub multiple_targets: bool,
    /// `offset`: the launch position relative to the anchor; default `[0, 0, 0]`.
    #[facet(default = ProjectileOffset::default())]
    pub offset: ProjectileOffset,
    /// `on_fire_time`: seconds a target burns; default 5.
    #[facet(default = 5.0_f32)]
    pub on_fire_time: f32,
    /// `on_hit`: what happens when it hits.
    pub on_hit: Option<OnHit>,
    /// `particle`: vanilla's legacy particle name shown on hit; default `iconcrack`.
    #[facet(default = "iconcrack".to_owned())]
    pub particle: String,
    /// `potion_effect`: the potion id applied on hit; −1 (the default) is none.
    #[facet(default = -1_i32)]
    pub potion_effect: i32,
    /// `power`: the launch speed; default 1.3.
    #[facet(default = 1.3_f32)]
    pub power: f32,
    /// `reflect_immunity`: seconds it cannot be reflected; default 0.
    #[facet(default = 0.0_f32)]
    pub reflect_immunity: f32,
    /// `reflect_on_hurt`: it is reflected when hurt; default `false`.
    #[facet(default = false)]
    pub reflect_on_hurt: bool,
    /// `shoot_sound`: the sound when shot; default none (empty).
    #[facet(default = String::new())]
    pub shoot_sound: String,
    /// `shoot_target`: it is aimed at the shooter's target; default `true`.
    #[facet(default = true)]
    pub shoot_target: bool,
    /// `should_bounce`: when it bounces off what it hits; default `no`.
    #[facet(default = ShouldBounce::No)]
    pub should_bounce: ShouldBounce,
    /// `splash_range`: the radius of a splash effect; default 4.
    #[facet(default = 4.0_f32)]
    pub splash_range: f32,
    /// `spread_fire`: a burning projectile sets the entity it hits on fire; default `true`.
    #[facet(default = true)]
    pub spread_fire: bool,
    /// `stop_on_hurt`: it stops when hurt; default `false`.
    #[facet(default = false)]
    pub stop_on_hurt: bool,
    /// `uncertainty_base`: the base spread of the shot; default 0.
    #[facet(default = 0.0_f32)]
    pub uncertainty_base: f32,
    /// `uncertainty_multiplier`: the spread added per difficulty level; default 0.
    #[facet(default = 0.0_f32)]
    pub uncertainty_multiplier: f32,
}

/// Where on the shooter a [`Projectile`] starts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[repr(u8)]
#[non_exhaustive]
pub enum ProjectileAnchor {
    /// The shooter's position (JSON `origin`).
    #[default]
    #[facet(rename = "origin")]
    Origin,
    /// The shooter's eyes (JSON `eye_height`).
    #[facet(rename = "eye_height")]
    EyeHeight,
    /// The middle of the shooter (JSON `middle`).
    #[facet(rename = "middle")]
    Middle,
}

/// When a [`Projectile`] bounces off what it hits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[repr(u8)]
#[non_exhaustive]
pub enum ShouldBounce {
    /// Never (JSON `no`).
    #[default]
    #[facet(rename = "no")]
    No,
    /// When the target is invulnerable (JSON `if_invulnerable`).
    #[facet(rename = "if_invulnerable")]
    IfInvulnerable,
    /// When it dealt no damage (JSON `if_no_damage_dealt`).
    #[facet(rename = "if_no_damage_dealt")]
    IfNoDamageDealt,
}

/// The launch offset of a [`Projectile`]: `[x, y, z]` or `{"x", "y", "z"}`, kept in the form
/// read.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum ProjectileOffset {
    /// `[x, y, z]`.
    Array(Vec<f32>),
    /// `{"x", "y", "z"}`.
    Object(Xyz),
}

impl Default for ProjectileOffset {
    fn default() -> Self {
        Self::Array(vec![0.0; 3])
    }
}

/// The object form of a [`ProjectileOffset`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Xyz {
    /// `x`: default 0.
    #[facet(default = 0.0_f32)]
    pub x: f32,
    /// `y`: default 0.
    #[facet(default = 0.0_f32)]
    pub y: f32,
    /// `z`: default 0.
    #[facet(default = 0.0_f32)]
    pub z: f32,
}

/// `on_hit` of a [`Projectile`]: the subcomponents that run when it hits.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct OnHit {
    /// `arrow_effect`: the arrow's potion effects apply.
    pub arrow_effect: Option<ArrowEffect>,
    /// `catch_fire`: the target catches fire.
    pub catch_fire: Option<CatchFire>,
    /// `definition_event`: an event fires.
    pub definition_event: Option<DefinitionEvent>,
    /// `douse_fire`: a burning target is extinguished.
    pub douse_fire: Option<OnHitFlag>,
    /// `freeze_on_hit`: water around the hit freezes.
    pub freeze_on_hit: Option<FreezeOnHit>,
    /// `grant_xp`: experience drops.
    pub grant_xp: Option<GrantXp>,
    /// `hurt_owner`: the shooter is hurt.
    pub hurt_owner: Option<HurtOwner>,
    /// `ignite`: the hit block or entity is set alight.
    pub ignite: Option<OnHitFlag>,
    /// `impact_damage`: the target takes damage.
    pub impact_damage: Option<ImpactDamage>,
    /// `mob_effect`: effects are applied to the target.
    pub mob_effect: Option<MobEffect>,
    /// `particle_on_hit`: particles show.
    pub particle_on_hit: Option<ParticleOnHit>,
    /// `remove_on_hit`: the projectile is removed.
    pub remove_on_hit: Option<OnHitFlag>,
    /// `spawn_aoe_cloud`: an area effect cloud spawns.
    pub spawn_aoe_cloud: Option<SpawnAoeCloud>,
    /// `spawn_chance`: entities may spawn.
    pub spawn_chance: Option<SpawnChance>,
    /// `stick_in_ground`: the projectile sticks in the ground.
    pub stick_in_ground: Option<StickInGround>,
    /// `teleport_owner`: the shooter teleports to the hit.
    pub teleport_owner: Option<OnHitFlag>,
    /// `thrown_potion_effect`: a thrown potion's effect applies.
    pub thrown_potion_effect: Option<ThrownPotionEffect>,
    /// `wind_burst_on_hit`: a wind burst happens.
    pub wind_burst_on_hit: Option<OnHitFlag>,
}

/// An `on_hit` subcomponent without members (`douse_fire`, `ignite`, `remove_on_hit`,
/// `teleport_owner`, `wind_burst_on_hit`): present means on.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct OnHitFlag {}

/// `on_hit.arrow_effect`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ArrowEffect {
    /// `apply_effect_to_blocking_targets`: the effects apply to a blocking target; default
    /// `true`.
    #[facet(default = true)]
    pub apply_effect_to_blocking_targets: bool,
}

/// `on_hit.catch_fire`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct CatchFire {
    /// `fire_affected_by_griefing`: the fire follows the `mobGriefing` rule; default `false`.
    #[facet(default = false)]
    pub fire_affected_by_griefing: bool,
    /// `on_fire_time`: seconds the target burns; default 0.
    #[facet(default = 0.0_f32)]
    pub on_fire_time: f32,
}

/// `on_hit.definition_event`.
#[allow(
    clippy::struct_excessive_bools,
    reason = "the game's flags, one member each"
)]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DefinitionEvent {
    /// `affect_projectile`: the event fires on the projectile; default `false`.
    #[facet(default = false)]
    pub affect_projectile: bool,
    /// `affect_shooter`: the event fires on the shooter; default `false`.
    #[facet(default = false)]
    pub affect_shooter: bool,
    /// `affect_splash_area`: the event fires on entities in the splash area; default `false`.
    #[facet(default = false)]
    pub affect_splash_area: bool,
    /// `affect_target`: the event fires on the target; default `false`.
    #[facet(default = false)]
    pub affect_target: bool,
    /// `event_trigger`: the event.
    #[facet(schema::foreign_event)]
    pub event_trigger: Option<EventTrigger>,
    /// `splash_area`: the splash area's radius; default 0.
    #[facet(default = 0.0_f32)]
    pub splash_area: f32,
}

/// `on_hit.freeze_on_hit`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct FreezeOnHit {
    /// `shape`: the frozen area's shape; default `cube`.
    #[facet(default = FreezeShape::Cube)]
    pub shape: FreezeShape,
    /// `size`: the half-extent of the frozen area, in blocks; default 1.
    #[facet(default = 1.0_f32)]
    pub size: f32,
    /// `snap_to_block`: the area is aligned to the block grid; default `false`.
    #[facet(default = false)]
    pub snap_to_block: bool,
}

/// The shape of a [`FreezeOnHit`] area.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[repr(u8)]
#[non_exhaustive]
pub enum FreezeShape {
    /// A cube (JSON `cube`).
    #[default]
    #[facet(rename = "cube")]
    Cube,
    /// A sphere (JSON `sphere`).
    #[facet(rename = "sphere")]
    Sphere,
}

/// `on_hit.grant_xp`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct GrantXp {
    /// `maxXP`: the most experience; default 0.
    #[facet(rename = "maxXP", default = 0_i32)]
    pub max_xp: i32,
    /// `minXP`: the least experience; default 0.
    #[facet(rename = "minXP", default = 0_i32)]
    pub min_xp: i32,
}

/// `on_hit.hurt_owner`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct HurtOwner {
    /// `ignite`: the shooter is set on fire; default `false`.
    #[facet(default = false)]
    pub ignite: bool,
    /// `knockback`: the shooter is knocked back; default `false`.
    #[facet(default = false)]
    pub knockback: bool,
    /// `owner_damage`: the damage the shooter takes; default 0.
    #[facet(default = 0.0_f32)]
    pub owner_damage: f32,
}

/// `on_hit.impact_damage`.
#[allow(
    clippy::struct_excessive_bools,
    reason = "the game's flags, one member each"
)]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ImpactDamage {
    /// `apply_knockback_to_blocking_targets`: a blocking target is knocked back too; default
    /// `false`.
    #[facet(default = false)]
    pub apply_knockback_to_blocking_targets: bool,
    /// `catch_fire`: the target catches fire; default `false`.
    #[facet(default = false)]
    pub catch_fire: bool,
    /// `ceil_pre_critical_damage`: the damage is rounded up before critical damage is added;
    /// default `false`.
    #[facet(default = false)]
    pub ceil_pre_critical_damage: bool,
    /// `channeling`: lightning strikes the target during a thunderstorm; default `false`.
    #[facet(default = false)]
    pub channeling: bool,
    /// `damage`: the damage range; default `{min: 0, max: 0}`.
    #[facet(default = FloatRange::default())]
    pub damage: FloatRange,
    /// `destroy_on_hit`: the projectile is destroyed on hit; default `false`.
    #[facet(default = false)]
    pub destroy_on_hit: bool,
    /// `destroy_on_hit_requires_damage`: only when damage was dealt; default `true`.
    #[facet(default = true)]
    pub destroy_on_hit_requires_damage: bool,
    /// `difficulty_randomization`: how difficulty randomises the damage; default `none`.
    #[facet(default = DifficultyRandomization::None)]
    pub difficulty_randomization: DifficultyRandomization,
    /// `filter`: the entity identifier the damage applies to; default all (empty).
    #[facet(default = String::new())]
    pub filter: String,
    /// `knockback`: the target is knocked back; default `false`.
    #[facet(default = false)]
    pub knockback: bool,
    /// `max_critical_damage`: the largest critical bonus; default `i32::MAX`.
    #[facet(default = i32::MAX)]
    pub max_critical_damage: i32,
    /// `min_critical_damage`: the smallest critical bonus; default 0.
    #[facet(default = 0_i32)]
    pub min_critical_damage: i32,
    /// `power_multiplier`: the damage added per unit of speed; default 0.
    #[facet(default = 0.0_f32)]
    pub power_multiplier: f32,
    /// `set_last_hurt_requires_damage`: the target remembers its attacker only when damaged;
    /// default `true`.
    #[facet(default = true)]
    pub set_last_hurt_requires_damage: bool,
}

/// How [`ImpactDamage`] randomises its damage by difficulty.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[repr(u8)]
#[non_exhaustive]
pub enum DifficultyRandomization {
    /// No randomisation (JSON `none`).
    #[default]
    #[facet(rename = "none")]
    None,
    /// A random amount is added (JSON `additive`).
    #[facet(rename = "additive")]
    Additive,
    /// The damage is multiplied by a random factor (JSON `multiplicative`).
    #[facet(rename = "multiplicative")]
    Multiplicative,
}

/// `on_hit.mob_effect`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MobEffect {
    /// `effects`: the effects.
    #[facet(default)]
    pub effects: Vec<MobEffectInstance>,
}

/// One effect of [`MobEffect`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MobEffectInstance {
    /// `amplifier`: the effect level; default 1.
    #[facet(default = 1_i32)]
    pub amplifier: i32,
    /// `durationeasy`: the duration on easy.
    pub durationeasy: Option<EffectDuration>,
    /// `durationhard`: the duration on hard.
    pub durationhard: Option<EffectDuration>,
    /// `durationnormal`: the duration on normal.
    pub durationnormal: Option<EffectDuration>,
    /// `effect`: the effect name; default none (empty).
    #[facet(default = String::new())]
    pub effect: String,
}

/// An effect duration: ticks (an integer, at least 0) or a name (`infinite`). JSON: an integer
/// or a string; any other JSON type is rejected.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum EffectDuration {
    /// A number of ticks.
    Ticks(i32),
    /// A name.
    Named(String),
}

/// `on_hit.particle_on_hit`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ParticleOnHit {
    /// `num_particles`: how many particles; default 1.
    #[facet(default = 1_i32)]
    pub num_particles: i32,
    /// `on_entity_hit`: particles when an entity is hit; default `false`.
    #[facet(default = false)]
    pub on_entity_hit: bool,
    /// `on_other_hit`: particles when anything else is hit; default `false`.
    #[facet(default = false)]
    pub on_other_hit: bool,
    /// `particle_item_name`: item names for item particles, each with its condition.
    #[facet(default)]
    pub particle_item_name: IndexMap<String, Filter>,
    /// `particle_type`: vanilla's legacy particle name; default `none`.
    #[facet(default = "none".to_owned())]
    pub particle_type: String,
}

/// `on_hit.spawn_aoe_cloud`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SpawnAoeCloud {
    /// `affect_owner`: the cloud affects the shooter; default `true`.
    #[facet(default = true)]
    pub affect_owner: bool,
    /// `duration`: the cloud's duration; default 0.
    #[facet(default = EffectDuration::Ticks(0_i32))]
    pub duration: EffectDuration,
    /// `particle`: vanilla's legacy particle name; default `mobspellambient`.
    #[facet(default = "mobspellambient".to_owned())]
    pub particle: String,
    /// `potion`: the potion id; −1 (the default) is none.
    #[facet(default = -1_i32)]
    pub potion: i32,
    /// `radius`: the cloud's radius; default 0.
    #[facet(default = 0.0_f32)]
    pub radius: f32,
    /// `radius_on_use`: the radius change per use; default −1.
    #[facet(default = -1.0_f32)]
    pub radius_on_use: f32,
    /// `reapplication_delay`: ticks before an entity is affected again; default 0.
    #[facet(default = 0_i32)]
    pub reapplication_delay: i32,
}

/// `on_hit.spawn_chance`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct SpawnChance {
    /// `first_spawn_chance`: the chance of the first spawn, 0 to 1; default 0.
    #[facet(default = 0.0_f32, schema::min = 0.0, schema::max = 1.0)]
    pub first_spawn_chance: f32,
    /// `first_spawn_count`: entities of the first spawn; default 0.
    #[facet(default = 0_i32)]
    pub first_spawn_count: i32,
    /// `on_spawn`: events fired on the spawned entities.
    #[facet(default, schema::foreign_event)]
    pub on_spawn: Vec<EventTrigger>,
    /// `second_spawn_chance`: the chance of the second spawn, 0 to 1; default 0.
    #[facet(default = 0.0_f32, schema::min = 0.0, schema::max = 1.0)]
    pub second_spawn_chance: f32,
    /// `second_spawn_count`: entities of the second spawn; default 0.
    #[facet(default = 0_i32)]
    pub second_spawn_count: i32,
    /// `spawn_baby`: the spawned entities are babies; default `false`.
    #[facet(default = false)]
    pub spawn_baby: bool,
    /// `spawn_definition`: the entity identifier spawned; default none (empty).
    #[facet(default = String::new())]
    pub spawn_definition: String,
}

/// `on_hit.stick_in_ground`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct StickInGround {
    /// `shake_time`: seconds the projectile shakes; default 0.
    #[facet(default = 0.0_f32)]
    pub shake_time: f32,
}

/// `on_hit.thrown_potion_effect`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ThrownPotionEffect {
    /// `effect`: the potion id; −1 (the default) is none.
    #[facet(default = -1_i32)]
    pub effect: i32,
}

/// The legacy reader's member aliases and the member each stands for.
// The aliases of the game's untyped projectile reader; the aliases with no member in the
// 1.26.50 struct are left unknown.
const ALIASES: [(&str, &str); 20] = [
    ("reflectOnHurt", "reflect_on_hurt"),
    ("destroyOnHurt", "destroy_on_hurt"),
    ("stopOnHurt", "stop_on_hurt"),
    ("angleoffset", "angle_offset"),
    ("uncertaintyBase", "uncertainty_base"),
    ("uncertaintyMultiplier", "uncertainty_multiplier"),
    ("critParticleOnHurt", "crit_particle_on_hurt"),
    ("multipletargets", "multiple_targets"),
    ("onHit", "on_hit"),
    ("onFireTime", "on_fire_time"),
    ("catchFire", "catch_fire"),
    ("potionEffect", "potion_effect"),
    ("splashRange", "splash_range"),
    ("isdangerous", "is_dangerous"),
    ("shoottarget", "shoot_target"),
    ("shootSound", "shoot_sound"),
    ("hitSound", "hit_sound"),
    ("hitGroundSound", "hit_ground_sound"),
    ("shouldbounce", "should_bounce"),
    ("hitWater", "hit_water"),
];

/// `impact_damage`'s legacy aliases.
const IMPACT_DAMAGE_ALIASES: [(&str, &str); 5] = [
    ("catchFire", "catch_fire"),
    ("destroyonhit", "destroy_on_hit"),
    ("maxcriticaldamage", "max_critical_damage"),
    ("mincriticaldamage", "min_critical_damage"),
    ("powermultiplier", "power_multiplier"),
];

/// The camel-case spellings the other legacy on-hit readers take, by subcomponent, and the
/// member each stands for.
// Each legacy reader tries the snake-case name and then the camel-case one; `grant_xp`'s
// `minXP` / `maxXP` are the typed names.
const ON_HIT_ALIASES: [(&str, &[(&str, &str)]); 4] = [
    (
        "catch_fire",
        &[
            ("onFireTime", "on_fire_time"),
            ("fireAffectedByGriefing", "fire_affected_by_griefing"),
        ],
    ),
    ("hurt_owner", &[("ownerDamage", "owner_damage")]),
    (
        "spawn_chance",
        &[
            ("firstSpawnChance", "first_spawn_chance"),
            ("secondSpawnChance", "second_spawn_chance"),
            ("firstSpawnCount", "first_spawn_count"),
            ("secondSpawnCount", "second_spawn_count"),
            ("spawnDefinition", "spawn_definition"),
            ("spawnBaby", "spawn_baby"),
        ],
    ),
    (
        "definition_event",
        &[
            ("affectShooter", "affect_shooter"),
            ("affectProjectile", "affect_projectile"),
            ("affectTarget", "affect_target"),
            ("affectSplashArea", "affect_splash_area"),
            ("splashArea", "splash_area"),
            ("eventTrigger", "event_trigger"),
        ],
    ),
];

/// Renames the legacy reader's aliases of a `projectile` and of its `on_hit.impact_damage`
/// to the members they stand for, where the member is absent; `moved` is told each rename
/// (paths below the component, the outer one first). Returns whether one was renamed.
///
/// The lenient read does it, and before it the step that converts members of a projectile
/// (1.26.30 → 1.26.40), so that a value under an alias is converted as the member is.
pub(crate) fn name_aliases(json: &mut VObject, moved: &mut dyn FnMut(JsonPath, JsonPath)) -> bool {
    let mut changed = map_aliases(json, &ALIASES, &JsonPath::new(), moved);
    if let Some(damage) = impact_damage(json) {
        let at = JsonPath::new().with(&["on_hit", "impact_damage"]);
        changed |= map_aliases(damage, &IMPACT_DAMAGE_ALIASES, &at, moved);
    }
    if let Some(on_hit) = json.get_mut("on_hit").and_then(Value::as_object_mut) {
        for (key, aliases) in ON_HIT_ALIASES {
            if let Some(subcomponent) = on_hit.get_mut(key).and_then(Value::as_object_mut) {
                changed |= map_aliases(
                    subcomponent,
                    aliases,
                    &JsonPath::new().with(&["on_hit", key]),
                    moved,
                );
            }
        }
    }
    changed
}

/// Renames each alias of `aliases` (members of the object at `at`) whose member is absent,
/// with the move told; returns whether one was renamed.
fn map_aliases(
    json: &mut VObject,
    aliases: &[(&str, &str)],
    at: &JsonPath,
    moved: &mut dyn FnMut(JsonPath, JsonPath),
) -> bool {
    let mut changed = false;
    for (alias, member) in aliases {
        if !json.contains_key(member)
            && let Some(value) = json.remove(alias)
        {
            json.insert(*member, value);
            moved(at.child(alias), at.child(member));
            changed = true;
        }
    }
    changed
}

/// A string member outside `allowed`: vanilla's warning (`'<value>' is not a valid '<member>'
/// value.`) and `fallback` instead. Returns whether the value was replaced.
fn enum_spelling(
    json: &mut VObject,
    member: &'static str,
    allowed: &[&str],
    fallback: &str,
    at: &JsonPath,
    cx: &mut ComponentCheck<'_>,
) -> bool {
    let Some(text) = json.get(member).and_then(Value::as_string) else {
        return false;
    };
    if allowed.contains(&text.as_str()) {
        return false;
    }
    let value = text.as_str().to_owned();
    // The game logs a level-3 line for `should_bounce` and for `difficulty_randomization` and
    // takes the default: it logs this at the error level and keeps the document, a kept error.
    cx.kept(
        codes::NOT_A_VALID_VALUE
            .with([value.as_str(), member])
            .into(),
        at,
    );
    json.insert(member, Value::from(fallback));
    true
}

impl Typed for Projectile {
    fn prepare(json: &mut VObject, cx: &mut ComponentCheck<'_>) -> bool {
        let mut changed = false;
        if matches!(cx.mode(), ConvertMode::Lenient) {
            changed |= name_aliases(json, &mut |from, to| cx.moved(from, to));
            // an integer `anchor`, which the game reads as a name from 1.26.30.
            if let Some(number) = json.get("anchor").and_then(Value::as_number) {
                let name = match number.to_i64() {
                    Some(0) => Some("origin"),
                    Some(1) => Some("eye_height"),
                    Some(2) => Some("middle"),
                    _ => None,
                };
                if name.is_none() {
                    // The game writes no content-log line for another integer, so the crate's own
                    // information stands here.
                    cx.push(
                        DiagnosticCode::from(&core_codes::INVALID_VALUE),
                        Severity::Info,
                        &JsonPath::new().child("anchor"),
                    );
                }
                json.insert("anchor", Value::from(name.unwrap_or("origin")));
                changed = true;
            }
            changed |= legacy_on_hit(json, cx);
        }
        // warnings in both modes, the default used.
        changed |= enum_spelling(
            json,
            "should_bounce",
            &["no", "if_invulnerable", "if_no_damage_dealt"],
            "no",
            &JsonPath::new().child("should_bounce"),
            cx,
        );
        if let Some(damage) = impact_damage(json) {
            changed |= enum_spelling(
                damage,
                "difficulty_randomization",
                &["none", "additive", "multiplicative"],
                "none",
                &JsonPath::new().with(&["on_hit", "impact_damage", "difficulty_randomization"]),
                cx,
            );
        }
        changed
    }

    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        if let ProjectileOffset::Array(list) = &self.offset
            && list.len() != 3
        {
            let at = JsonPath::new().child("offset");
            cx.invalid(&at, &mut self.offset, ProjectileOffset::default());
        }
        if let Some(on_hit) = &mut self.on_hit {
            on_hit.check(&JsonPath::new().child("on_hit"), cx);
        }
    }
}

/// The lenient conversion's legacy `on_hit` shape: a single effect at `mob_effect`'s root
/// becomes the one element of `effects`, and the legacy `spawn_chance` chances are read as the
/// game's reader reads them ([`legacy_spawn_chances`]). Returns whether anything changed.
// The crate's own reading, taken from the game's packs: the 1.26.0 `shulker_bullet` file
// against its 1.26.50 copy.
fn legacy_on_hit(json: &mut VObject, cx: &mut ComponentCheck<'_>) -> bool {
    let Some(on_hit) = json.get_mut("on_hit").and_then(Value::as_object_mut) else {
        return false;
    };
    let mut changed = false;
    if let Some(spawn) = on_hit
        .get_mut("spawn_chance")
        .and_then(Value::as_object_mut)
    {
        changed |= legacy_spawn_chances(
            spawn,
            &JsonPath::new().with(&["on_hit", "spawn_chance"]),
            cx,
        );
    }
    // The game's mob-effect reader takes `effects` as an array or a single object.
    if let Some(effect) = on_hit.get_mut("mob_effect").and_then(Value::as_object_mut)
        && effect.get("effects").is_some_and(Value::is_object)
    {
        changed |= super::value::one_element(
            effect,
            "effects",
            &JsonPath::new().with(&["on_hit", "mob_effect"]),
            cx,
        );
    }
    if let Some(effect) = on_hit.get_mut("mob_effect").and_then(Value::as_object_mut)
        && !effect.contains_key("effects")
    {
        let effect_at = JsonPath::new().with(&["on_hit", "mob_effect"]);
        let element = effect_at.child("effects").index(0);
        let mut single = VObject::new();
        let mut moves = Vec::new();
        for key in [
            "effect",
            "amplifier",
            "durationeasy",
            "durationnormal",
            "durationhard",
        ] {
            if let Some(value) = effect.remove(key) {
                single.insert(key, value);
                moves.push((effect_at.child(key), element.child(key)));
            }
        }
        if !single.is_empty() {
            // The single effect is `mob_effect` itself in the file: what is told of
            // `effects/0` is told there.
            cx.moved(effect_at, element);
            for (from, to) in moves {
                cx.moved(from, to);
            }
            let mut effects = facet_value::VArray::new();
            effects.push(Value::from(single));
            effect.insert("effects", Value::from(effects));
            changed = true;
        }
    }
    changed
}

/// The legacy reader of `spawn_chance`: for each group, `*_spawn_percent_chance` (or its camel
/// spelling) is read as a float with the default −1 and divided by 100; when that is negative,
/// `*_spawn_chance` (or its camel spelling) is read as an integer N with the default 100 and the
/// chance is 1/N. The typed 1.26.50 struct binds the chance alone (0 to 1), so the conversion
/// writes it and drops the percent members. The reader takes a member by name and then reads it
/// with jsoncpp's `asFloat(default)` or `asInt(default)`, so each value converts as jsoncpp
/// converts it ([`legacy_as_float`], [`legacy_as_int`]): a string percent is 0 % and a string N is
/// 0. N = 0 makes 1/0, +inf in the game, written as 1 (always). A member the reader takes as
/// another number than the one written is told where it is written with an information
/// `ValueRewritten` ([`spawn_chance_rewritten`]; the game writes no content-log line for this).
/// Returns whether anything changed.
fn legacy_spawn_chances(spawn: &mut VObject, at: &JsonPath, cx: &mut ComponentCheck<'_>) -> bool {
    let mut changed = false;
    for (percent, percent_camel, chance, chance_camel) in [
        (
            "first_spawn_percent_chance",
            "firstSpawnPercentChance",
            "first_spawn_chance",
            "firstSpawnChance",
        ),
        (
            "second_spawn_percent_chance",
            "secondSpawnPercentChance",
            "second_spawn_chance",
            "secondSpawnChance",
        ),
    ] {
        // The snake-case name when the object has it, else the camel-case one, read once.
        // The value read and the name it is written under.
        let read = |spawn: &mut VObject, snake: &'static str, camel: &'static str| {
            let (snake_value, camel_value) = (spawn.remove(snake), spawn.remove(camel));
            match (snake_value, camel_value) {
                (Some(value), _) => Some((value, snake)),
                (None, Some(value)) => Some((value, camel)),
                (None, None) => None,
            }
        };
        let mut tell = |read: Option<&(Value, &'static str)>, integer: bool| {
            if let Some((value, name)) = read
                && spawn_chance_rewritten(value, integer)
            {
                cx.push(
                    DiagnosticCode::from(&core_codes::VALUE_REWRITTEN),
                    Severity::Info,
                    &at.child(name),
                );
            }
        };
        let percent_read = read(spawn, percent, percent_camel);
        tell(percent_read.as_ref(), false);
        let percent_value = legacy_as_float(percent_read.as_ref().map(|(value, _)| value), -1.0);
        let percent_value = percent_value / 100.0;
        let value = if percent_value >= 0.0 {
            spawn.remove(chance_camel);
            percent_value
        } else {
            let chance_read = read(spawn, chance, chance_camel);
            tell(chance_read.as_ref(), true);
            let n = legacy_as_int(chance_read.as_ref().map(|(value, _)| value), 100);
            #[allow(clippy::cast_precision_loss, reason = "the game converts with scvtf")]
            let one_in = 1.0 / n as f32;
            if one_in.is_finite() { one_in } else { 1.0 }
        };
        let value = f64::from(value);
        spawn.insert(chance, Value::from(value));
        changed = true;
    }
    changed
}

/// Whether the legacy spawn-chance reader takes `value` as another number than the one written: any
/// value of another type than a number (`null` aside, rule N1's), and for the one-in-N member
/// (`integer`) a number that is no integer from 1 to `i32::MAX` (a real is truncated, a wider one
/// saturates or keeps its low 32 bits, and N below 1 is no one-in-N).
fn spawn_chance_rewritten(value: &Value, integer: bool) -> bool {
    if value.is_null() {
        return false;
    }
    let Some(number) = value.as_number() else {
        return true;
    };
    integer
        && !(number.is_integer()
            && number
                .to_i64()
                .is_some_and(|n| (1..=i64::from(i32::MAX)).contains(&n)))
}

/// jsoncpp's `asInt(default)` of a member: absent or `null` is the default; an
/// integer keeps its low 32 bits; a real is truncated toward zero and saturates; a
/// bool is 0 or 1; a string, array or object is 0.
#[allow(
    clippy::cast_possible_truncation,
    reason = "the low 32 bits and a saturating truncation, as the game converts"
)]
fn legacy_as_int(value: Option<&Value>, default: i32) -> i32 {
    let Some(value) = value.filter(|value| !value.is_null()) else {
        return default;
    };
    if let Some(flag) = value.as_bool() {
        return i32::from(flag);
    }
    match value.as_number() {
        Some(number) if !number.is_float() => number
            .to_i64()
            .or_else(|| number.to_u64().map(u64::cast_signed))
            .map_or(0, |n| n as i32),
        Some(number) => number.to_f64_lossy() as i32,
        None => 0,
    }
}

/// jsoncpp's `asFloat(default)` of a member: absent or `null` is the default; a
/// number is itself as a float; a bool is 0 or 1; a string, array or object is 0.
#[allow(
    clippy::cast_possible_truncation,
    reason = "the game holds the value as f32"
)]
fn legacy_as_float(value: Option<&Value>, default: f32) -> f32 {
    let Some(value) = value.filter(|value| !value.is_null()) else {
        return default;
    };
    if let Some(flag) = value.as_bool() {
        return f32::from(u8::from(flag));
    }
    value
        .as_number()
        .map_or(0.0, |number| number.to_f64_lossy() as f32)
}

/// The `on_hit.impact_damage` object of a projectile's JSON, if there is one.
fn impact_damage(json: &mut VObject) -> Option<&mut VObject> {
    json.get_mut("on_hit")?
        .as_object_mut()?
        .get_mut("impact_damage")?
        .as_object_mut()
}

impl OnHit {
    /// The rule checks of the subcomponents, under `at`.
    fn check(&mut self, at: &JsonPath, cx: &mut ComponentCheck<'_>) {
        if let Some(event) = &self.definition_event
            && let Some(trigger) = &event.event_trigger
        {
            let mut here = at.child("definition_event");
            here.push("event_trigger");
            cx.extend(trigger.check(&here, cx.unknown()));
        }
        if let Some(damage) = &mut self.impact_damage {
            let mut here = at.child("impact_damage");
            here.push("damage");
            cx.range(&here, &mut damage.damage);
        }
        if let Some(effects) = &mut self.mob_effect {
            let list = at.child("mob_effect").child("effects");
            for (i, effect) in effects.effects.iter_mut().enumerate() {
                let here = list.index(i);
                for (key, duration) in [
                    ("durationeasy", &mut effect.durationeasy),
                    ("durationhard", &mut effect.durationhard),
                    ("durationnormal", &mut effect.durationnormal),
                ] {
                    if let Some(EffectDuration::Ticks(ticks)) = duration {
                        cx.at_least(&here.child(key), ticks, 0, 0);
                    }
                }
            }
        }
        if let Some(particles) = &self.particle_on_hit {
            let names = at.child("particle_on_hit").child("particle_item_name");
            for (name, filter) in &particles.particle_item_name {
                cx.extend(filter.check(&names.child(name), cx.unknown()));
            }
        }
        if let Some(cloud) = &mut self.spawn_aoe_cloud
            && let EffectDuration::Ticks(ticks) = &mut cloud.duration
        {
            let here = at.child("spawn_aoe_cloud").child("duration");
            cx.at_least(&here, ticks, 0, 0);
        }
        if let Some(spawn) = &mut self.spawn_chance {
            // The game's 1.26.50 schema bounds both chances to 0 ..= 1.
            let here = at.child("spawn_chance");
            cx.bounds(here.clone(), spawn);
            let triggers = here.child("on_spawn");
            for (i, trigger) in spawn.on_spawn.iter().enumerate() {
                cx.extend(trigger.check(&triggers.index(i), cx.unknown()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Default` is the value read from an empty object, whose members all take their defaults.
    fn default_matches_empty_object<T>()
    where
        T: Default + PartialEq + std::fmt::Debug + for<'a> Facet<'a>,
    {
        let parsed = bedrock_addon_core::json::from_str::<T>("{}").expect("an empty object parses");
        assert_eq!(T::default(), parsed, "{}", std::any::type_name::<T>());
    }

    #[test]
    fn on_hit_subcomponents_default_to_the_empty_object() {
        default_matches_empty_object::<ArrowEffect>();
        default_matches_empty_object::<CatchFire>();
        default_matches_empty_object::<DefinitionEvent>();
        default_matches_empty_object::<FreezeOnHit>();
        default_matches_empty_object::<GrantXp>();
        default_matches_empty_object::<HurtOwner>();
        default_matches_empty_object::<ImpactDamage>();
        default_matches_empty_object::<MobEffectInstance>();
        default_matches_empty_object::<ParticleOnHit>();
        default_matches_empty_object::<SpawnAoeCloud>();
        default_matches_empty_object::<SpawnChance>();
        default_matches_empty_object::<StickInGround>();
        default_matches_empty_object::<ThrownPotionEffect>();
        default_matches_empty_object::<Xyz>();
    }
}

bedrock_addon_core::impl_schema_leaf!(ProjectileOffset);
bedrock_addon_core::impl_no_molang!(ProjectileOffset);
bedrock_addon_core::impl_schema_leaf!(EffectDuration);
bedrock_addon_core::impl_no_molang!(EffectDuration);
