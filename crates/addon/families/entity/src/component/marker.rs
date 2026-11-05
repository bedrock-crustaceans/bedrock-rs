//! The marker components: no members, written `{}`. A member written into one is kept among
//! the document's unknown members and reported as `UnrecognizedField`.
//!
//! There are 49 markers: 23 first typed in 1.21.120, 11 in 1.26.20, 11 in 1.26.40 and 4 in
//! 1.26.50. The game binds each of them without members.

use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

use super::parse::Typed;

/// Declares one memberless component struct per line.
macro_rules! markers {
    ($( $(#[$doc:meta])* $name:ident = $key:literal; )*) => {$(
        $(#[$doc])*
        #[doc(alias = $key)]
        #[derive(Clone, Debug, PartialEq, Facet, Schema)]
        #[facet(schema::unknown = "keep")]
        pub struct $name {}

        impl Typed for $name {}
    )*};
}

markers! {
    /// `minecraft:body_rotation_always_follows_head`: the body turns with the head at once.
    BodyRotationAlwaysFollowsHead = "minecraft:body_rotation_always_follows_head";
    /// `minecraft:body_rotation_blocked`: the body never rotates on its own.
    BodyRotationBlocked = "minecraft:body_rotation_blocked";
    /// `minecraft:can_climb`: the entity climbs ladders.
    CanClimb = "minecraft:can_climb";
    /// `minecraft:can_fly`: the entity can fly; pathfinding does not stay on the ground.
    CanFly = "minecraft:can_fly";
    /// `minecraft:can_power_jump`: the entity charges its jump while ridden.
    CanPowerJump = "minecraft:can_power_jump";
    /// `minecraft:fire_immune`: fire and lava do not hurt the entity.
    FireImmune = "minecraft:fire_immune";
    /// `minecraft:input_ground_controlled`: a rider steers the entity on the ground with the
    /// movement keys.
    InputGroundControlled = "minecraft:input_ground_controlled";
    /// `minecraft:is_baby`: the entity is a baby (for filters and Molang).
    IsBaby = "minecraft:is_baby";
    /// `minecraft:is_charged`: the entity is charged (a powered creeper).
    IsCharged = "minecraft:is_charged";
    /// `minecraft:is_chested`: the entity carries a chest.
    IsChested = "minecraft:is_chested";
    /// `minecraft:is_hidden_when_invisible`: monsters ignore the entity while it is invisible.
    IsHiddenWhenInvisible = "minecraft:is_hidden_when_invisible";
    /// `minecraft:is_ignited`: the entity is ignited (a primed creeper).
    IsIgnited = "minecraft:is_ignited";
    /// `minecraft:is_illager_captain`: the entity is an illager captain.
    IsIllagerCaptain = "minecraft:is_illager_captain";
    /// `minecraft:is_pregnant`: the entity is pregnant.
    IsPregnant = "minecraft:is_pregnant";
    /// `minecraft:is_saddled`: the entity wears a saddle.
    IsSaddled = "minecraft:is_saddled";
    /// `minecraft:is_shaking`: the entity is shaking (a wet wolf).
    IsShaking = "minecraft:is_shaking";
    /// `minecraft:is_sheared`: the entity is sheared.
    IsSheared = "minecraft:is_sheared";
    /// `minecraft:is_stackable`: other entities can stand on the entity.
    IsStackable = "minecraft:is_stackable";
    /// `minecraft:is_stunned`: the entity is stunned.
    IsStunned = "minecraft:is_stunned";
    /// `minecraft:is_tamed`: the entity is tamed.
    IsTamed = "minecraft:is_tamed";
    /// `minecraft:renders_when_invisible`: the entity is still drawn while invisible.
    RendersWhenInvisible = "minecraft:renders_when_invisible";
    /// `minecraft:rotation_axis_aligned`: the entity's rotation snaps to the world axes.
    RotationAxisAligned = "minecraft:rotation_axis_aligned";
    /// `minecraft:rotation_locked_to_vehicle`: the entity's rotation follows its vehicle.
    RotationLockedToVehicle = "minecraft:rotation_locked_to_vehicle";
    /// `minecraft:block_climber`: the entity climbs scaffolding.
    BlockClimber = "minecraft:block_climber";
    /// `minecraft:can_join_raid`: the entity can join an existing raid.
    CanJoinRaid = "minecraft:can_join_raid";
    /// `minecraft:dimension_bound`: the entity does not change dimension through portals.
    DimensionBound = "minecraft:dimension_bound";
    /// `minecraft:item_hopper`: the entity is an item hopper.
    ItemHopper = "minecraft:item_hopper";
    /// `minecraft:out_of_control`: the entity is out of control (a boat in a bubble column).
    OutOfControl = "minecraft:out_of_control";
    /// `minecraft:suspect_tracking`: the entity remembers suspicious locations.
    SuspectTracking = "minecraft:suspect_tracking";
    /// `minecraft:transient`: the entity is not saved with the world.
    Transient = "minecraft:transient";
    /// `minecraft:uses_legacy_friction`: the entity keeps the friction rules from before 1.26.20.
    UsesLegacyFriction = "minecraft:uses_legacy_friction";
    /// `minecraft:uses_uniform_air_drag`: the entity's air drag is the same on every axis.
    UsesUniformAirDrag = "minecraft:uses_uniform_air_drag";
    /// `minecraft:vibration_damper`: vibrations the entity emits are ignored.
    VibrationDamper = "minecraft:vibration_damper";
    /// `minecraft:vibration_listener`: the entity listens to vibrations.
    VibrationListener = "minecraft:vibration_listener";
    /// `minecraft:annotation.open_door`: the entity opens doors, as its navigation allows.
    AnnotationOpenDoor = "minecraft:annotation.open_door";
    /// `minecraft:bucketable`: the entity can be caught in a bucket.
    Bucketable = "minecraft:bucketable";
    /// `minecraft:cannot_be_attacked`: other entities cannot attack the entity.
    CannotBeAttacked = "minecraft:cannot_be_attacked";
    /// `minecraft:floats_in_liquid`: the entity floats in liquids.
    FloatsInLiquid = "minecraft:floats_in_liquid";
    /// `minecraft:hide`: the entity hides at its owned point of interest, or the closest one.
    Hide = "minecraft:hide";
    /// `minecraft:managed_wandering_trader`: part of the wandering trader's behaviour.
    ManagedWanderingTrader = "minecraft:managed_wandering_trader";
    /// `minecraft:not_pickable_from_inside`: the entity cannot be picked (targeted) from inside
    /// it.
    NotPickableFromInside = "minecraft:not_pickable_from_inside";
    /// `minecraft:persistent`: the entity never despawns.
    Persistent = "minecraft:persistent";
    /// `minecraft:trade_resupply`: the entity resupplies its trades.
    TradeResupply = "minecraft:trade_resupply";
    /// `minecraft:trust`: the entity trusts several players.
    Trust = "minecraft:trust";
    /// `minecraft:wants_jockey`: the entity wants to become a jockey.
    WantsJockey = "minecraft:wants_jockey";
    /// `minecraft:can_stand_on_powder_snow`: the entity stands on powder snow without sinking.
    CanStandOnPowderSnow = "minecraft:can_stand_on_powder_snow";
    /// `minecraft:freezing_immune`: freezing does not affect the entity.
    FreezingImmune = "minecraft:freezing_immune";
    /// `minecraft:freezing_vulnerable`: the entity takes more damage from freezing.
    FreezingVulnerable = "minecraft:freezing_vulnerable";
    /// `minecraft:uses_legacy_ambient_sounds`: the entity picks its ambient sound by state (in
    /// the air, in water, as a baby).
    UsesLegacyAmbientSounds = "minecraft:uses_legacy_ambient_sounds";
    // `pushable_by_block` is typed memberless at 1.26.60.
    /// `minecraft:pushable_by_block`: blocks such as pistons and shulker boxes push the entity.
    PushableByBlock = "minecraft:pushable_by_block";
}
