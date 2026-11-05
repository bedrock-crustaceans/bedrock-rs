//! The components of an entity: one closed enum, [`EntityComponent`], over every key vanilla
//! binds in `components` and in each component group, AI goals included (their types live in
//! [`goal`]).
//!
//! Each typed variant holds its component type; the keys vanilla still parses at run time
//! from the JSON are rows of the [`Dynamic`](EntityComponent::Dynamic) variant, kept as
//! written. A key no row names is [`Unknown`](EntityComponent::Unknown): kept for write-back
//! and reported as `UnrecognizedComponent`.
//!
//! The row list is fixed here (406 keys plus one pattern key, as `ComponentMeta` values); the
//! component types live in this module and the goal types in [`goal`].

// The derive's table of 423 rows is one constant array.
#![allow(clippy::large_stack_arrays)]

mod marker;
pub use marker::*;
mod health;
pub use health::*;
mod value;
pub use value::*;
mod lifetime;
pub use lifetime::*;
mod trigger;
pub use trigger::*;
mod trade;
pub use trade::*;
mod target;
pub use target::*;
mod push;
pub use push::*;
mod movement;
pub use movement::*;
mod molang;
pub mod older;
pub use molang::*;
mod interact;
pub use interact::*;
mod projectile;
pub(crate) use projectile::name_aliases as name_projectile_aliases;
pub use projectile::*;
mod lenient;
pub(crate) mod parse;
#[doc(hidden)]
pub mod upgrade;

use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::CustomComponent;
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::{LoadContext, LoadRequirement};
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_derive::Components;

use super::Entity;
use super::goal;
use parse::parse_typed;

/// The requirement of a component vanilla binds only for beta documents before its release
/// (`bounciness`, `air_drag_modifier` below 1.26.30): the Beta APIs toggle or a base-game pack.
// No `LoadRequirement` names a beta document, so the beta-features rule stands in for it (a
// beta document's version is above the release anyway).
const BETA_DOCUMENT: LoadRequirement = LoadRequirement::BetaFeatures;

/// Keeps a component's JSON as written: the parse of every dynamic key, and of every typed key
/// until its type parses it. The map stores the entry under its key, untyped.
#[allow(clippy::unnecessary_wraps, reason = "the `ParseFn` signature")]
fn keep_json(
    _: JsonSlice<'_>,
    _: ConvertMode,
    _: &LoadContext,
    _: &mut Diagnostics,
) -> Option<EntityComponent> {
    Some(EntityComponent::Dynamic(Value::NULL))
}

/// One entry of an entity's `components` (or of a component group): every component and AI
/// goal vanilla binds, by its JSON key.
#[derive(Clone, Debug, PartialEq, Components)]
#[component(family = Entity, custom = Custom, unknown = Unknown, dynamic = Dynamic, write, typed_table = crate::game_schema::typed_since, versioned = crate::goal::parse_versioned, triggers)]
#[non_exhaustive]
#[allow(
    clippy::large_enum_variant,
    reason = "one variant per component, sized as written; boxing would lose the unknown members `keep_unknown_members` keeps"
)]
pub enum EntityComponent {
    /// `minecraft:addrider`.
    #[component(name = "minecraft:addrider", typed = "1.21.130", released = "1.8.0", parse = parse_typed::<Addrider>)]
    Addrider(Addrider),
    /// `minecraft:admire_item`.
    #[component(name = "minecraft:admire_item", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<AdmireItem>)]
    AdmireItem(AdmireItem),
    /// `minecraft:anger_level`.
    #[component(name = "minecraft:anger_level", typed = "1.26.50", released = "1.8.0", molang, parse = parse_typed::<AngerLevel>)]
    AngerLevel(AngerLevel),
    /// `minecraft:annotation.open_door`.
    #[component(name = "minecraft:annotation.open_door", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<AnnotationOpenDoor>)]
    AnnotationOpenDoor(AnnotationOpenDoor),
    /// `minecraft:apply_knockback_rules`.
    #[component(name = "minecraft:apply_knockback_rules", typed = "1.8.0", parse = parse_typed::<ApplyKnockbackRules>)]
    ApplyKnockbackRules(ApplyKnockbackRules),
    /// `minecraft:barter`.
    #[component(name = "minecraft:barter", typed = "1.26.50", released = "1.8.0", parse = parse_typed::<Barter>)]
    Barter(Barter),
    /// `minecraft:behavior.admire_item`.
    #[component(name = "minecraft:behavior.admire_item", typed = "1.26.20", released = "1.8.0", molang, parse = goal::parse::<goal::AdmireItem>)]
    BehaviorAdmireItem(goal::AdmireItem),
    /// `minecraft:behavior.aquatic_charge_attack`.
    #[component(name = "minecraft:behavior.aquatic_charge_attack", typed = "1.21.120", goal, released = "1.21.120", parse = goal::parse::<goal::AquaticChargeAttack>)]
    BehaviorAquaticChargeAttack(goal::AquaticChargeAttack),
    /// `minecraft:behavior.avoid_block`.
    #[component(name = "minecraft:behavior.avoid_block", typed = "1.21.120", released = "1.8.0", molang, parse = goal::parse::<goal::AvoidBlock>)]
    BehaviorAvoidBlock(goal::AvoidBlock),
    /// `minecraft:behavior.avoid_mob_type`.
    #[component(name = "minecraft:behavior.avoid_mob_type", typed = "1.21.120", released = "1.8.0", molang, parse = goal::parse::<goal::AvoidMobType>)]
    BehaviorAvoidMobType(goal::AvoidMobType),
    /// `minecraft:behavior.barter`.
    #[component(name = "minecraft:behavior.barter", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::Barter>)]
    BehaviorBarter(goal::Barter),
    /// `minecraft:behavior.beg`.
    #[component(name = "minecraft:behavior.beg", typed = "1.26.40", goal, released = "1.8.0", molang, parse = goal::parse::<goal::Beg>)]
    BehaviorBeg(goal::Beg),
    /// `minecraft:behavior.break_door`.
    #[component(name = "minecraft:behavior.break_door", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::BreakDoor>)]
    BehaviorBreakDoor(goal::BreakDoor),
    /// `minecraft:behavior.breed`.
    #[component(name = "minecraft:behavior.breed", typed = "1.21.90", goal, released = "1.8.0", parse = goal::parse::<goal::Breed>)]
    BehaviorBreed(goal::Breed),
    /// `minecraft:behavior.celebrate`.
    #[component(name = "minecraft:behavior.celebrate", typed = "1.26.10", released = "1.8.0", molang, parse = goal::parse::<goal::Celebrate>)]
    BehaviorCelebrate(goal::Celebrate),
    /// `minecraft:behavior.celebrate_survive`.
    #[component(name = "minecraft:behavior.celebrate_survive", typed = "1.26.10", released = "1.8.0", molang, parse = goal::parse::<goal::CelebrateSurvive>)]
    BehaviorCelebrateSurvive(goal::CelebrateSurvive),
    /// `minecraft:behavior.charge_attack`.
    #[component(name = "minecraft:behavior.charge_attack", typed = "1.21.100", goal, released = "1.8.0", parse = goal::parse::<goal::ChargeAttack>)]
    BehaviorChargeAttack(goal::ChargeAttack),
    /// `minecraft:behavior.charge_held_item`.
    #[component(name = "minecraft:behavior.charge_held_item", typed = "1.26.40", goal, released = "1.8.0", molang, parse = goal::parse::<goal::ChargeHeldItem>)]
    BehaviorChargeHeldItem(goal::ChargeHeldItem),
    /// `minecraft:behavior.circle_around_anchor`.
    #[component(name = "minecraft:behavior.circle_around_anchor", typed = "1.21.100", goal, released = "1.8.0", parse = goal::parse::<goal::CircleAroundAnchor>)]
    BehaviorCircleAroundAnchor(goal::CircleAroundAnchor),
    /// `minecraft:behavior.controlled_by_player`.
    #[component(name = "minecraft:behavior.controlled_by_player", typed = "1.21.100", goal, released = "1.8.0", parse = goal::parse::<goal::ControlledByPlayer>)]
    BehaviorControlledByPlayer(goal::ControlledByPlayer),
    /// `minecraft:behavior.croak`.
    #[component(name = "minecraft:behavior.croak", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::Croak>)]
    BehaviorCroak(goal::Croak),
    /// `minecraft:behavior.defend_trusted_target`.
    #[component(name = "minecraft:behavior.defend_trusted_target", typed = "1.26.30", released = "1.8.0", molang, parse = goal::parse::<goal::DefendTrustedTarget>)]
    BehaviorDefendTrustedTarget(goal::DefendTrustedTarget),
    /// `minecraft:behavior.defend_village_target`.
    #[component(name = "minecraft:behavior.defend_village_target", typed = "1.26.0", goal, released = "1.8.0", parse = goal::parse::<goal::DefendVillageTarget>)]
    BehaviorDefendVillageTarget(goal::DefendVillageTarget),
    /// `minecraft:behavior.delayed_attack`.
    #[component(name = "minecraft:behavior.delayed_attack", typed = "1.26.0", released = "1.8.0", molang, parse = goal::parse::<goal::DelayedAttack>)]
    BehaviorDelayedAttack(goal::DelayedAttack),
    /// `minecraft:behavior.dig`.
    #[component(name = "minecraft:behavior.dig", typed = "1.21.120", released = "1.8.0", molang, parse = goal::parse::<goal::Dig>)]
    BehaviorDig(goal::Dig),
    /// `minecraft:behavior.door_interact`.
    #[component(name = "minecraft:behavior.door_interact", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::DoorInteract>)]
    BehaviorDoorInteract(goal::DoorInteract),
    /// `minecraft:behavior.dragonchargeplayer`.
    #[component(name = "minecraft:behavior.dragonchargeplayer", typed = "1.26.0", goal, released = "1.8.0", parse = goal::parse::<goal::DragonChargePlayer>)]
    BehaviorDragonChargePlayer(goal::DragonChargePlayer),
    /// `minecraft:behavior.dragondeath`.
    #[component(name = "minecraft:behavior.dragondeath", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::DragonDeath>)]
    BehaviorDragonDeath(goal::DragonDeath),
    /// `minecraft:behavior.dragonflaming`.
    #[component(name = "minecraft:behavior.dragonflaming", typed = "1.21.100", goal, released = "1.8.0", parse = goal::parse::<goal::DragonFlaming>)]
    BehaviorDragonFlaming(goal::DragonFlaming),
    /// `minecraft:behavior.dragonholdingpattern`.
    #[component(name = "minecraft:behavior.dragonholdingpattern", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::DragonHoldingPattern>)]
    BehaviorDragonHoldingPattern(goal::DragonHoldingPattern),
    /// `minecraft:behavior.dragonlanding`.
    #[component(name = "minecraft:behavior.dragonlanding", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::DragonLanding>)]
    BehaviorDragonLanding(goal::DragonLanding),
    /// `minecraft:behavior.dragonscanning`.
    #[component(name = "minecraft:behavior.dragonscanning", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::DragonScanning>)]
    BehaviorDragonScanning(goal::DragonScanning),
    /// `minecraft:behavior.dragonstrafeplayer`.
    #[component(name = "minecraft:behavior.dragonstrafeplayer", typed = "1.26.0", goal, released = "1.8.0", parse = goal::parse::<goal::DragonStrafePlayer>)]
    BehaviorDragonStrafePlayer(goal::DragonStrafePlayer),
    /// `minecraft:behavior.dragontakeoff`.
    #[component(name = "minecraft:behavior.dragontakeoff", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::DragonTakeoff>)]
    BehaviorDragonTakeoff(goal::DragonTakeoff),
    /// `minecraft:behavior.drink_milk`.
    #[component(name = "minecraft:behavior.drink_milk", typed = "1.21.120", goal, released = "1.8.0", parse = goal::parse::<goal::DrinkMilk>)]
    BehaviorDrinkMilk(goal::DrinkMilk),
    /// `minecraft:behavior.drink_potion`.
    #[component(name = "minecraft:behavior.drink_potion", typed = "1.26.40", goal, released = "1.8.0", parse = goal::parse::<goal::DrinkPotion>)]
    BehaviorDrinkPotion(goal::DrinkPotion),
    /// `minecraft:behavior.drop_item_for`.
    #[component(name = "minecraft:behavior.drop_item_for", typed = "1.26.10", released = "1.8.0", molang, parse = goal::parse::<goal::DropItemFor>)]
    BehaviorDropItemFor(goal::DropItemFor),
    /// `minecraft:behavior.eat_block`.
    #[component(name = "minecraft:behavior.eat_block", typed = "1.26.40", released = "1.8.0", molang, parse = goal::parse::<goal::EatBlock>)]
    BehaviorEatBlock(goal::EatBlock),
    /// `minecraft:behavior.eat_carried_item`.
    #[component(name = "minecraft:behavior.eat_carried_item", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::EatCarriedItem>)]
    BehaviorEatCarriedItem(goal::EatCarriedItem),
    /// `minecraft:behavior.eat_mob`.
    #[component(name = "minecraft:behavior.eat_mob", typed = "1.21.100", goal, released = "1.8.0", parse = goal::parse::<goal::EatMob>)]
    BehaviorEatMob(goal::EatMob),
    /// `minecraft:behavior.emerge`.
    #[component(name = "minecraft:behavior.emerge", typed = "1.26.30", released = "1.8.0", molang, parse = goal::parse::<goal::Emerge>)]
    BehaviorEmerge(goal::Emerge),
    /// `minecraft:behavior.equip_item`.
    #[component(name = "minecraft:behavior.equip_item", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::EquipItem>)]
    BehaviorEquipItem(goal::EquipItem),
    /// `minecraft:behavior.explore_outskirts`.
    #[component(name = "minecraft:behavior.explore_outskirts", typed = "1.21.130", goal, released = "1.8.0", parse = goal::parse::<goal::ExploreOutskirts>)]
    BehaviorExploreOutskirts(goal::ExploreOutskirts),
    /// `minecraft:behavior.fertilize_farm_block`.
    #[component(name = "minecraft:behavior.fertilize_farm_block", typed = "1.26.10", goal, released = "1.8.0", parse = goal::parse::<goal::FertilizeFarmBlock>)]
    BehaviorFertilizeFarmBlock(goal::FertilizeFarmBlock),
    /// `minecraft:behavior.find_cover`.
    #[component(name = "minecraft:behavior.find_cover", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::FindCover>)]
    BehaviorFindCover(goal::FindCover),
    /// `minecraft:behavior.find_mount`.
    #[component(name = "minecraft:behavior.find_mount", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::FindMount>)]
    BehaviorFindMount(goal::FindMount),
    /// `minecraft:behavior.find_underwater_treasure`.
    #[component(name = "minecraft:behavior.find_underwater_treasure", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::FindUnderwaterTreasure>)]
    BehaviorFindUnderwaterTreasure(goal::FindUnderwaterTreasure),
    /// `minecraft:behavior.fire_at_target`.
    #[component(name = "minecraft:behavior.fire_at_target", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::FireAtTarget>)]
    BehaviorFireAtTarget(goal::FireAtTarget),
    /// `minecraft:behavior.flee_sun`.
    #[component(name = "minecraft:behavior.flee_sun", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::FleeSun>)]
    BehaviorFleeSun(goal::FleeSun),
    /// `minecraft:behavior.float`.
    #[component(name = "minecraft:behavior.float", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::Float>)]
    BehaviorFloat(goal::Float),
    /// `minecraft:behavior.float_tempt` (typed at 1.26.60).
    #[component(name = "minecraft:behavior.float_tempt", typed = "1.26.60", released = "1.8.0", molang, parse = goal::parse::<goal::FloatTempt>)]
    BehaviorFloatTempt(goal::FloatTempt),
    /// `minecraft:behavior.float_wander`.
    #[component(name = "minecraft:behavior.float_wander", typed = "1.26.40", goal, released = "1.8.0", parse = goal::parse::<goal::FloatWander>)]
    BehaviorFloatWander(goal::FloatWander),
    /// `minecraft:behavior.follow_caravan`.
    #[component(name = "minecraft:behavior.follow_caravan", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::FollowCaravan>)]
    BehaviorFollowCaravan(goal::FollowCaravan),
    /// `minecraft:behavior.follow_mob`.
    #[component(name = "minecraft:behavior.follow_mob", typed = "1.26.50", goal, released = "1.8.0", parse = goal::parse::<goal::FollowMob>)]
    BehaviorFollowMob(goal::FollowMob),
    /// `minecraft:behavior.follow_owner`.
    #[component(name = "minecraft:behavior.follow_owner", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::FollowOwner>)]
    BehaviorFollowOwner(goal::FollowOwner),
    /// `minecraft:behavior.follow_parent`.
    #[component(name = "minecraft:behavior.follow_parent", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::FollowParent>)]
    BehaviorFollowParent(goal::FollowParent),
    /// `minecraft:behavior.follow_target_leader`.
    #[component(name = "minecraft:behavior.follow_target_leader", typed = "1.26.30", goal, released = "1.26.20", supersedes = "minecraft:behavior.follow_target_captain", parse = goal::parse::<goal::FollowTargetLeader>)]
    BehaviorFollowTargetLeader(goal::FollowTargetLeader),
    /// `minecraft:behavior.go_and_give_items_to_noteblock`.
    #[component(name = "minecraft:behavior.go_and_give_items_to_noteblock", typed = "1.26.20", released = "1.8.0", molang, parse = goal::parse::<goal::GoAndGiveItemsToNoteblock>)]
    BehaviorGoAndGiveItemsToNoteblock(goal::GoAndGiveItemsToNoteblock),
    /// `minecraft:behavior.go_and_give_items_to_owner`.
    #[component(name = "minecraft:behavior.go_and_give_items_to_owner", typed = "1.26.20", released = "1.8.0", molang, parse = goal::parse::<goal::GoAndGiveItemsToOwner>)]
    BehaviorGoAndGiveItemsToOwner(goal::GoAndGiveItemsToOwner),
    /// `minecraft:behavior.go_home`.
    #[component(name = "minecraft:behavior.go_home", typed = "1.26.20", released = "1.8.0", molang, parse = goal::parse::<goal::GoHome>)]
    BehaviorGoHome(goal::GoHome),
    /// `minecraft:behavior.guardian_attack`.
    #[component(name = "minecraft:behavior.guardian_attack", typed = "1.26.0", goal, released = "1.8.0", parse = goal::parse::<goal::GuardianAttack>)]
    BehaviorGuardianAttack(goal::GuardianAttack),
    /// `minecraft:behavior.harvest_farm_block`.
    #[component(name = "minecraft:behavior.harvest_farm_block", typed = "1.26.10", goal, released = "1.8.0", parse = goal::parse::<goal::HarvestFarmBlock>)]
    BehaviorHarvestFarmBlock(goal::HarvestFarmBlock),
    /// `minecraft:behavior.hide`.
    #[component(name = "minecraft:behavior.hide", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::Hide>)]
    BehaviorHide(goal::Hide),
    /// `minecraft:behavior.hold_ground`.
    #[component(name = "minecraft:behavior.hold_ground", typed = "1.26.20", released = "1.8.0", molang, parse = goal::parse::<goal::HoldGround>)]
    BehaviorHoldGround(goal::HoldGround),
    /// `minecraft:behavior.hover`.
    #[component(name = "minecraft:behavior.hover", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::Hover>)]
    BehaviorHover(goal::Hover),
    /// `minecraft:behavior.hurt_by_target`.
    #[component(name = "minecraft:behavior.hurt_by_target", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::HurtByTarget>)]
    BehaviorHurtByTarget(goal::HurtByTarget),
    /// `minecraft:behavior.inspect_bookshelf`.
    #[component(name = "minecraft:behavior.inspect_bookshelf", typed = "1.26.10", goal, released = "1.8.0", parse = goal::parse::<goal::InspectBookshelf>)]
    BehaviorInspectBookshelf(goal::InspectBookshelf),
    /// `minecraft:behavior.investigate_suspicious_location`.
    #[component(name = "minecraft:behavior.investigate_suspicious_location", typed = "1.21.130", goal, released = "1.8.0", parse = goal::parse::<goal::InvestigateSuspiciousLocation>)]
    BehaviorInvestigateSuspiciousLocation(goal::InvestigateSuspiciousLocation),
    /// `minecraft:behavior.jump_around_target`.
    #[component(name = "minecraft:behavior.jump_around_target", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::JumpAroundTarget>)]
    BehaviorJumpAroundTarget(goal::JumpAroundTarget),
    /// `minecraft:behavior.jump_to_block`.
    #[component(name = "minecraft:behavior.jump_to_block", typed = "1.26.20", goal, released = "1.8.0", molang, parse = goal::parse::<goal::JumpToBlock>)]
    BehaviorJumpToBlock(goal::JumpToBlock),
    /// `minecraft:behavior.knockback_roar`.
    #[component(name = "minecraft:behavior.knockback_roar", typed = "1.26.40", released = "1.8.0", molang, parse = goal::parse::<goal::KnockbackRoar>)]
    BehaviorKnockbackRoar(goal::KnockbackRoar),
    /// `minecraft:behavior.lay_down`.
    #[component(name = "minecraft:behavior.lay_down", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::LayDown>)]
    BehaviorLayDown(goal::LayDown),
    /// `minecraft:behavior.lay_egg`.
    #[component(name = "minecraft:behavior.lay_egg", typed = "1.26.10", released = "1.8.0", molang, parse = goal::parse::<goal::LayEgg>)]
    BehaviorLayEgg(goal::LayEgg),
    /// `minecraft:behavior.leap_at_target`.
    #[component(name = "minecraft:behavior.leap_at_target", typed = "1.26.40", goal, released = "1.8.0", parse = goal::parse::<goal::LeapAtTarget>)]
    BehaviorLeapAtTarget(goal::LeapAtTarget),
    /// `minecraft:behavior.look_at_entity`.
    #[component(name = "minecraft:behavior.look_at_entity", typed = "1.26.0", goal, released = "1.8.0", parse = goal::parse::<goal::LookAtEntity>)]
    BehaviorLookAtEntity(goal::LookAtEntity),
    /// `minecraft:behavior.look_at_player`.
    #[component(name = "minecraft:behavior.look_at_player", typed = "1.26.0", goal, released = "1.8.0", parse = goal::parse::<goal::LookAtPlayer>)]
    BehaviorLookAtPlayer(goal::LookAtPlayer),
    /// `minecraft:behavior.look_at_target`.
    #[component(name = "minecraft:behavior.look_at_target", typed = "1.26.0", goal, released = "1.8.0", parse = goal::parse::<goal::LookAtTarget>)]
    BehaviorLookAtTarget(goal::LookAtTarget),
    /// `minecraft:behavior.look_at_trading_player`.
    #[component(name = "minecraft:behavior.look_at_trading_player", typed = "1.26.0", goal, released = "1.8.0", parse = goal::parse::<goal::LookAtTradingPlayer>)]
    BehaviorLookAtTradingPlayer(goal::LookAtTradingPlayer),
    /// `minecraft:behavior.make_love`.
    #[component(name = "minecraft:behavior.make_love", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::MakeLove>)]
    BehaviorMakeLove(goal::MakeLove),
    /// `minecraft:behavior.melee_attack`.
    #[component(name = "minecraft:behavior.melee_attack", typed = "1.26.0", released = "1.8.0", molang, parse = goal::parse::<goal::MeleeAttack>)]
    BehaviorMeleeAttack(goal::MeleeAttack),
    /// `minecraft:behavior.melee_box_attack`.
    #[component(name = "minecraft:behavior.melee_box_attack", typed = "1.26.0", released = "1.8.0", molang, parse = goal::parse::<goal::MeleeBoxAttack>)]
    BehaviorMeleeBoxAttack(goal::MeleeBoxAttack),
    /// `minecraft:behavior.mingle`.
    #[component(name = "minecraft:behavior.mingle", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::Mingle>)]
    BehaviorMingle(goal::Mingle),
    /// `minecraft:behavior.mount_pathing`.
    #[component(name = "minecraft:behavior.mount_pathing", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::MountPathing>)]
    BehaviorMountPathing(goal::MountPathing),
    /// `minecraft:behavior.move_around_target`.
    #[component(name = "minecraft:behavior.move_around_target", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::MoveAroundTarget>)]
    BehaviorMoveAroundTarget(goal::MoveAroundTarget),
    /// `minecraft:behavior.move_indoors`.
    #[component(name = "minecraft:behavior.move_indoors", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::MoveIndoors>)]
    BehaviorMoveIndoors(goal::MoveIndoors),
    /// `minecraft:behavior.move_outdoors`.
    #[component(name = "minecraft:behavior.move_outdoors", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::MoveOutdoors>)]
    BehaviorMoveOutdoors(goal::MoveOutdoors),
    /// `minecraft:behavior.move_through_village`.
    #[component(name = "minecraft:behavior.move_through_village", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::MoveThroughVillage>)]
    BehaviorMoveThroughVillage(goal::MoveThroughVillage),
    /// `minecraft:behavior.move_to_block`.
    #[component(name = "minecraft:behavior.move_to_block", typed = "1.26.40", released = "1.8.0", molang, parse = goal::parse::<goal::MoveToBlock>)]
    BehaviorMoveToBlock(goal::MoveToBlock),
    /// `minecraft:behavior.move_to_land`.
    #[component(name = "minecraft:behavior.move_to_land", typed = "1.26.10", goal, released = "1.8.0", parse = goal::parse::<goal::MoveToLand>)]
    BehaviorMoveToLand(goal::MoveToLand),
    /// `minecraft:behavior.move_to_lava`.
    #[component(name = "minecraft:behavior.move_to_lava", typed = "1.26.10", goal, released = "1.8.0", parse = goal::parse::<goal::MoveToLava>)]
    BehaviorMoveToLava(goal::MoveToLava),
    /// `minecraft:behavior.move_to_liquid`.
    #[component(name = "minecraft:behavior.move_to_liquid", typed = "1.26.10", goal, released = "1.8.0", parse = goal::parse::<goal::MoveToLiquid>)]
    BehaviorMoveToLiquid(goal::MoveToLiquid),
    /// `minecraft:behavior.move_to_poi`.
    #[component(name = "minecraft:behavior.move_to_poi", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::MoveToPoi>)]
    BehaviorMoveToPoi(goal::MoveToPoi),
    /// `minecraft:behavior.move_to_random_block`.
    #[component(name = "minecraft:behavior.move_to_random_block", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::MoveToRandomBlock>)]
    BehaviorMoveToRandomBlock(goal::MoveToRandomBlock),
    /// `minecraft:behavior.move_to_village`.
    #[component(name = "minecraft:behavior.move_to_village", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::MoveToVillage>)]
    BehaviorMoveToVillage(goal::MoveToVillage),
    /// `minecraft:behavior.move_to_water`.
    #[component(name = "minecraft:behavior.move_to_water", typed = "1.26.10", goal, released = "1.8.0", parse = goal::parse::<goal::MoveToWater>)]
    BehaviorMoveToWater(goal::MoveToWater),
    /// `minecraft:behavior.move_towards_dwelling_restriction`.
    #[component(name = "minecraft:behavior.move_towards_dwelling_restriction", typed = "1.26.0", goal, released = "1.16.100", parse = goal::parse::<goal::MoveTowardsDwellingRestriction>)]
    BehaviorMoveTowardsDwellingRestriction(goal::MoveTowardsDwellingRestriction),
    /// `minecraft:behavior.move_towards_home_restriction`.
    #[component(name = "minecraft:behavior.move_towards_home_restriction", typed = "1.26.0", goal, released = "1.16.100", parse = goal::parse::<goal::MoveTowardsHomeRestriction>)]
    BehaviorMoveTowardsHomeRestriction(goal::MoveTowardsHomeRestriction),
    /// `minecraft:behavior.move_towards_target`.
    #[component(name = "minecraft:behavior.move_towards_target", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::MoveTowardsTarget>)]
    BehaviorMoveTowardsTarget(goal::MoveTowardsTarget),
    /// `minecraft:behavior.nap`.
    #[component(name = "minecraft:behavior.nap", typed = "1.26.40", goal, released = "1.8.0", parse = goal::parse::<goal::Nap>)]
    BehaviorNap(goal::Nap),
    /// `minecraft:behavior.nearest_attackable_target`.
    #[component(name = "minecraft:behavior.nearest_attackable_target", typed = "1.26.0", goal, released = "1.8.0", parse = goal::parse::<goal::NearestAttackableTarget>)]
    BehaviorNearestAttackableTarget(goal::NearestAttackableTarget),
    /// `minecraft:behavior.nearest_prioritized_attackable_target`.
    #[component(name = "minecraft:behavior.nearest_prioritized_attackable_target", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::NearestPrioritizedAttackableTarget>)]
    BehaviorNearestPrioritizedAttackableTarget(goal::NearestPrioritizedAttackableTarget),
    /// `minecraft:behavior.ocelot_sit_on_block`.
    #[component(name = "minecraft:behavior.ocelot_sit_on_block", typed = "1.26.10", goal, released = "1.8.0", parse = goal::parse::<goal::OcelotSitOnBlock>)]
    BehaviorOcelotSitOnBlock(goal::OcelotSitOnBlock),
    /// `minecraft:behavior.ocelotattack`.
    #[component(name = "minecraft:behavior.ocelotattack", typed = "1.26.10", goal, released = "1.8.0", parse = goal::parse::<goal::Ocelotattack>)]
    BehaviorOcelotattack(goal::Ocelotattack),
    /// `minecraft:behavior.offer_flower`.
    #[component(name = "minecraft:behavior.offer_flower", typed = "1.26.10", goal, released = "1.8.0", parse = goal::parse::<goal::OfferFlower>)]
    BehaviorOfferFlower(goal::OfferFlower),
    /// `minecraft:behavior.open_door`.
    #[component(name = "minecraft:behavior.open_door", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::OpenDoor>)]
    BehaviorOpenDoor(goal::OpenDoor),
    /// `minecraft:behavior.owner_hurt_by_target`.
    #[component(name = "minecraft:behavior.owner_hurt_by_target", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::OwnerHurtByTarget>)]
    BehaviorOwnerHurtByTarget(goal::OwnerHurtByTarget),
    /// `minecraft:behavior.owner_hurt_target`.
    #[component(name = "minecraft:behavior.owner_hurt_target", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::OwnerHurtTarget>)]
    BehaviorOwnerHurtTarget(goal::OwnerHurtTarget),
    /// `minecraft:behavior.panic` (typed at 1.26.60).
    #[component(name = "minecraft:behavior.panic", typed = "1.26.60", released = "1.8.0", parse = goal::parse::<goal::Panic>)]
    BehaviorPanic(goal::Panic),
    /// `minecraft:behavior.pet_sleep_with_owner`.
    #[component(name = "minecraft:behavior.pet_sleep_with_owner", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::PetSleepWithOwner>)]
    BehaviorPetSleepWithOwner(goal::PetSleepWithOwner),
    /// `minecraft:behavior.pickup_items`.
    #[component(name = "minecraft:behavior.pickup_items", typed = "1.26.20", released = "1.8.0", molang, parse = goal::parse::<goal::PickupItems>)]
    BehaviorPickupItems(goal::PickupItems),
    /// `minecraft:behavior.place_block`.
    #[component(name = "minecraft:behavior.place_block", typed = "1.26.30", released = "1.21.100", molang, parse = goal::parse::<goal::PlaceBlock>)]
    BehaviorPlaceBlock(goal::PlaceBlock),
    /// `minecraft:behavior.play`.
    #[component(name = "minecraft:behavior.play", typed = "1.26.10", goal, released = "1.8.0", parse = goal::parse::<goal::Play>)]
    BehaviorPlay(goal::Play),
    /// `minecraft:behavior.play_dead`.
    #[component(name = "minecraft:behavior.play_dead", typed = "1.26.10", goal, released = "1.8.0", parse = goal::parse::<goal::PlayDead>)]
    BehaviorPlayDead(goal::PlayDead),
    /// `minecraft:behavior.player_ride_tamed`.
    #[component(name = "minecraft:behavior.player_ride_tamed", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::PlayerRideTamed>)]
    BehaviorPlayerRideTamed(goal::PlayerRideTamed),
    /// `minecraft:behavior.raid_garden`.
    #[component(name = "minecraft:behavior.raid_garden", typed = "1.26.10", goal, released = "1.8.0", parse = goal::parse::<goal::RaidGarden>)]
    BehaviorRaidGarden(goal::RaidGarden),
    /// `minecraft:behavior.ram_attack`.
    #[component(name = "minecraft:behavior.ram_attack", typed = "1.26.10", released = "1.8.0", molang, parse = goal::parse::<goal::RamAttack>)]
    BehaviorRamAttack(goal::RamAttack),
    /// `minecraft:behavior.random_breach`.
    #[component(name = "minecraft:behavior.random_breach", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::RandomBreach>)]
    BehaviorRandomBreach(goal::RandomBreach),
    /// `minecraft:behavior.random_fly`.
    #[component(name = "minecraft:behavior.random_fly", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::RandomFly>)]
    BehaviorRandomFly(goal::RandomFly),
    /// `minecraft:behavior.random_hover`.
    #[component(name = "minecraft:behavior.random_hover", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::RandomHover>)]
    BehaviorRandomHover(goal::RandomHover),
    /// `minecraft:behavior.random_look_around`.
    #[component(name = "minecraft:behavior.random_look_around", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::RandomLookAround>)]
    BehaviorRandomLookAround(goal::RandomLookAround),
    /// `minecraft:behavior.random_look_around_and_sit`.
    #[component(name = "minecraft:behavior.random_look_around_and_sit", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::RandomLookAroundAndSit>)]
    BehaviorRandomLookAroundAndSit(goal::RandomLookAroundAndSit),
    /// `minecraft:behavior.random_search_and_dig`.
    #[component(name = "minecraft:behavior.random_search_and_dig", typed = "1.26.10", released = "1.8.0", molang, parse = goal::parse::<goal::RandomSearchAndDig>)]
    BehaviorRandomSearchAndDig(goal::RandomSearchAndDig),
    /// `minecraft:behavior.random_sitting`.
    #[component(name = "minecraft:behavior.random_sitting", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::RandomSitting>)]
    BehaviorRandomSitting(goal::RandomSitting),
    /// `minecraft:behavior.random_stroll`.
    #[component(name = "minecraft:behavior.random_stroll", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::RandomStroll>)]
    BehaviorRandomStroll(goal::RandomStroll),
    /// `minecraft:behavior.random_swim`.
    #[component(name = "minecraft:behavior.random_swim", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::RandomSwim>)]
    BehaviorRandomSwim(goal::RandomSwim),
    /// `minecraft:behavior.ranged_attack`.
    #[component(name = "minecraft:behavior.ranged_attack", typed = "1.26.40", goal, released = "1.8.0", parse = goal::parse::<goal::RangedAttack>)]
    BehaviorRangedAttack(goal::RangedAttack),
    /// `minecraft:behavior.receive_love`.
    #[component(name = "minecraft:behavior.receive_love", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::ReceiveLove>)]
    BehaviorReceiveLove(goal::ReceiveLove),
    /// `minecraft:behavior.restrict_open_door`.
    #[component(name = "minecraft:behavior.restrict_open_door", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::RestrictOpenDoor>)]
    BehaviorRestrictOpenDoor(goal::RestrictOpenDoor),
    /// `minecraft:behavior.restrict_sun`.
    #[component(name = "minecraft:behavior.restrict_sun", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::RestrictSun>)]
    BehaviorRestrictSun(goal::RestrictSun),
    /// `minecraft:behavior.rise_to_liquid_level`.
    #[component(name = "minecraft:behavior.rise_to_liquid_level", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::RiseToLiquidLevel>)]
    BehaviorRiseToLiquidLevel(goal::RiseToLiquidLevel),
    /// `minecraft:behavior.roar`.
    #[component(name = "minecraft:behavior.roar", typed = "1.21.110", goal, released = "1.8.0", parse = goal::parse::<goal::Roar>)]
    BehaviorRoar(goal::Roar),
    /// `minecraft:behavior.roll`.
    #[component(name = "minecraft:behavior.roll", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::Roll>)]
    BehaviorRoll(goal::Roll),
    /// `minecraft:behavior.run_around_like_crazy`.
    #[component(name = "minecraft:behavior.run_around_like_crazy", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::RunAroundLikeCrazy>)]
    BehaviorRunAroundLikeCrazy(goal::RunAroundLikeCrazy),
    /// `minecraft:behavior.scared`.
    #[component(name = "minecraft:behavior.scared", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::Scared>)]
    BehaviorScared(goal::Scared),
    /// `minecraft:behavior.send_event` (typed at 1.26.60).
    #[component(name = "minecraft:behavior.send_event", typed = "1.26.60", goal, released = "1.8.0", parse = goal::parse::<goal::SendEvent>)]
    BehaviorSendEvent(goal::SendEvent),
    /// `minecraft:behavior.share_items`.
    #[component(name = "minecraft:behavior.share_items", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::ShareItems>)]
    BehaviorShareItems(goal::ShareItems),
    /// `minecraft:behavior.silverfish_merge_with_stone`.
    #[component(name = "minecraft:behavior.silverfish_merge_with_stone", typed = "1.26.40", goal, released = "1.8.0", parse = goal::parse::<goal::SilverfishMergeWithStone>)]
    BehaviorSilverfishMergeWithStone(goal::SilverfishMergeWithStone),
    /// `minecraft:behavior.silverfish_wake_up_friends`.
    #[component(name = "minecraft:behavior.silverfish_wake_up_friends", typed = "1.26.40", goal, released = "1.8.0", parse = goal::parse::<goal::SilverfishWakeUpFriends>)]
    BehaviorSilverfishWakeUpFriends(goal::SilverfishWakeUpFriends),
    /// `minecraft:behavior.skeleton_horse_trap`.
    #[component(name = "minecraft:behavior.skeleton_horse_trap", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::SkeletonHorseTrap>)]
    BehaviorSkeletonHorseTrap(goal::SkeletonHorseTrap),
    /// `minecraft:behavior.sleep`.
    #[component(name = "minecraft:behavior.sleep", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::Sleep>)]
    BehaviorSleep(goal::Sleep),
    /// `minecraft:behavior.slime_attack`.
    #[component(name = "minecraft:behavior.slime_attack", typed = "1.21.110", goal, released = "1.8.0", parse = goal::parse::<goal::SlimeAttack>)]
    BehaviorSlimeAttack(goal::SlimeAttack),
    /// `minecraft:behavior.slime_float`.
    #[component(name = "minecraft:behavior.slime_float", typed = "1.21.110", goal, released = "1.8.0", parse = goal::parse::<goal::SlimeFloat>)]
    BehaviorSlimeFloat(goal::SlimeFloat),
    /// `minecraft:behavior.slime_keep_on_jumping`.
    #[component(name = "minecraft:behavior.slime_keep_on_jumping", typed = "1.21.110", goal, released = "1.8.0", parse = goal::parse::<goal::SlimeKeepOnJumping>)]
    BehaviorSlimeKeepOnJumping(goal::SlimeKeepOnJumping),
    /// `minecraft:behavior.slime_random_direction`.
    #[component(name = "minecraft:behavior.slime_random_direction", typed = "1.21.110", goal, released = "1.8.0", parse = goal::parse::<goal::SlimeRandomDirection>)]
    BehaviorSlimeRandomDirection(goal::SlimeRandomDirection),
    /// `minecraft:behavior.snacking`.
    #[component(name = "minecraft:behavior.snacking", typed = "1.26.40", goal, released = "1.8.0", molang, parse = goal::parse::<goal::Snacking>)]
    BehaviorSnacking(goal::Snacking),
    /// `minecraft:behavior.sneeze`.
    #[component(name = "minecraft:behavior.sneeze", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::Sneeze>)]
    BehaviorSneeze(goal::Sneeze),
    /// `minecraft:behavior.sniff`.
    #[component(name = "minecraft:behavior.sniff", typed = "1.21.110", goal, released = "1.8.0", parse = goal::parse::<goal::Sniff>)]
    BehaviorSniff(goal::Sniff),
    /// `minecraft:behavior.sonic_boom`.
    #[component(name = "minecraft:behavior.sonic_boom", typed = "1.21.110", goal, released = "1.8.0", parse = goal::parse::<goal::SonicBoom>)]
    BehaviorSonicBoom(goal::SonicBoom),
    /// `minecraft:behavior.squid_dive`.
    #[component(name = "minecraft:behavior.squid_dive", typed = "1.26.40", goal, released = "1.8.0", parse = goal::parse::<goal::SquidDive>)]
    BehaviorSquidDive(goal::SquidDive),
    /// `minecraft:behavior.squid_flee`.
    #[component(name = "minecraft:behavior.squid_flee", typed = "1.26.40", goal, released = "1.8.0", parse = goal::parse::<goal::SquidFlee>)]
    BehaviorSquidFlee(goal::SquidFlee),
    /// `minecraft:behavior.squid_idle`.
    #[component(name = "minecraft:behavior.squid_idle", typed = "1.26.40", goal, released = "1.8.0", parse = goal::parse::<goal::SquidIdle>)]
    BehaviorSquidIdle(goal::SquidIdle),
    /// `minecraft:behavior.squid_move_away_from_ground`.
    #[component(name = "minecraft:behavior.squid_move_away_from_ground", typed = "1.26.40", goal, released = "1.8.0", parse = goal::parse::<goal::SquidMoveAwayFromGround>)]
    BehaviorSquidMoveAwayFromGround(goal::SquidMoveAwayFromGround),
    /// `minecraft:behavior.squid_out_of_water`.
    #[component(name = "minecraft:behavior.squid_out_of_water", typed = "1.26.40", goal, released = "1.8.0", parse = goal::parse::<goal::SquidOutOfWater>)]
    BehaviorSquidOutOfWater(goal::SquidOutOfWater),
    /// `minecraft:behavior.stalk_and_pounce_on_target`.
    #[component(name = "minecraft:behavior.stalk_and_pounce_on_target", typed = "1.26.40", goal, released = "1.8.0", parse = goal::parse::<goal::StalkAndPounceOnTarget>)]
    BehaviorStalkAndPounceOnTarget(goal::StalkAndPounceOnTarget),
    /// `minecraft:behavior.stay_near_noteblock`.
    #[component(name = "minecraft:behavior.stay_near_noteblock", typed = "1.21.110", goal, released = "1.8.0", parse = goal::parse::<goal::StayNearNoteblock>)]
    BehaviorStayNearNoteblock(goal::StayNearNoteblock),
    /// `minecraft:behavior.stay_while_sitting`.
    #[component(name = "minecraft:behavior.stay_while_sitting", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::StayWhileSitting>)]
    BehaviorStayWhileSitting(goal::StayWhileSitting),
    /// `minecraft:behavior.stomp_attack`.
    #[component(name = "minecraft:behavior.stomp_attack", typed = "1.26.0", released = "1.8.0", molang, parse = goal::parse::<goal::StompAttack>)]
    BehaviorStompAttack(goal::StompAttack),
    /// `minecraft:behavior.stomp_turtle_egg`.
    #[component(name = "minecraft:behavior.stomp_turtle_egg", typed = "1.26.10", goal, released = "1.8.0", parse = goal::parse::<goal::StompTurtleEgg>)]
    BehaviorStompTurtleEgg(goal::StompTurtleEgg),
    /// `minecraft:behavior.stroll_towards_village`.
    #[component(name = "minecraft:behavior.stroll_towards_village", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::StrollTowardsVillage>)]
    BehaviorStrollTowardsVillage(goal::StrollTowardsVillage),
    /// `minecraft:behavior.summon_entity`.
    #[component(name = "minecraft:behavior.summon_entity", typed = "1.26.40", goal, released = "1.8.0", parse = goal::parse::<goal::SummonEntity>)]
    BehaviorSummonEntity(goal::SummonEntity),
    /// `minecraft:behavior.swell`.
    #[component(name = "minecraft:behavior.swell", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::Swell>)]
    BehaviorSwell(goal::Swell),
    /// `minecraft:behavior.swim_idle`.
    #[component(name = "minecraft:behavior.swim_idle", typed = "1.21.110", goal, released = "1.8.0", parse = goal::parse::<goal::SwimIdle>)]
    BehaviorSwimIdle(goal::SwimIdle),
    /// `minecraft:behavior.swim_up_for_breath`.
    #[component(name = "minecraft:behavior.swim_up_for_breath", typed = "1.21.110", goal, released = "1.8.0", parse = goal::parse::<goal::SwimUpForBreath>)]
    BehaviorSwimUpForBreath(goal::SwimUpForBreath),
    /// `minecraft:behavior.swim_wander`.
    #[component(name = "minecraft:behavior.swim_wander", typed = "1.21.110", goal, released = "1.8.0", parse = goal::parse::<goal::SwimWander>)]
    BehaviorSwimWander(goal::SwimWander),
    /// `minecraft:behavior.swim_with_entity`.
    #[component(name = "minecraft:behavior.swim_with_entity", typed = "1.26.10", goal, released = "1.8.0", parse = goal::parse::<goal::SwimWithEntity>)]
    BehaviorSwimWithEntity(goal::SwimWithEntity),
    /// `minecraft:behavior.swoop_attack`.
    #[component(name = "minecraft:behavior.swoop_attack", typed = "1.21.110", goal, released = "1.8.0", parse = goal::parse::<goal::SwoopAttack>)]
    BehaviorSwoopAttack(goal::SwoopAttack),
    /// `minecraft:behavior.take_block`.
    #[component(name = "minecraft:behavior.take_block", typed = "1.26.30", released = "1.21.100", molang, parse = goal::parse::<goal::TakeBlock>)]
    BehaviorTakeBlock(goal::TakeBlock),
    /// `minecraft:behavior.take_flower`.
    #[component(name = "minecraft:behavior.take_flower", typed = "1.26.10", released = "1.8.0", molang, parse = goal::parse::<goal::TakeFlower>)]
    BehaviorTakeFlower(goal::TakeFlower),
    /// `minecraft:behavior.target_when_pushed`.
    #[component(name = "minecraft:behavior.target_when_pushed", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::TargetWhenPushed>)]
    BehaviorTargetWhenPushed(goal::TargetWhenPushed),
    /// `minecraft:behavior.teleport_to_owner`.
    #[component(name = "minecraft:behavior.teleport_to_owner", typed = "1.26.10", goal, released = "1.8.0", parse = goal::parse::<goal::TeleportToOwner>)]
    BehaviorTeleportToOwner(goal::TeleportToOwner),
    /// `minecraft:behavior.tempt` (typed at 1.26.60).
    #[component(name = "minecraft:behavior.tempt", typed = "1.26.60", released = "1.8.0", molang, parse = goal::parse::<goal::Tempt>)]
    BehaviorTempt(goal::Tempt),
    /// `minecraft:behavior.timer_flag_1`.
    #[component(name = "minecraft:behavior.timer_flag_1", typed = "1.26.0", released = "1.8.0", molang, parse = goal::parse::<goal::TimerFlag1>)]
    BehaviorTimerFlag1(goal::TimerFlag1),
    /// `minecraft:behavior.timer_flag_2`.
    #[component(name = "minecraft:behavior.timer_flag_2", typed = "1.26.0", released = "1.8.0", molang, parse = goal::parse::<goal::TimerFlag2>)]
    BehaviorTimerFlag2(goal::TimerFlag2),
    /// `minecraft:behavior.timer_flag_3`.
    #[component(name = "minecraft:behavior.timer_flag_3", typed = "1.26.0", released = "1.8.0", molang, parse = goal::parse::<goal::TimerFlag3>)]
    BehaviorTimerFlag3(goal::TimerFlag3),
    /// `minecraft:behavior.trade_interest`.
    #[component(name = "minecraft:behavior.trade_interest", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::TradeInterest>)]
    BehaviorTradeInterest(goal::TradeInterest),
    /// `minecraft:behavior.trade_with_player`.
    #[component(name = "minecraft:behavior.trade_with_player", typed = "1.26.10", goal, released = "1.8.0", parse = goal::parse::<goal::TradeWithPlayer>)]
    BehaviorTradeWithPlayer(goal::TradeWithPlayer),
    /// `minecraft:behavior.transport_items`.
    #[component(name = "minecraft:behavior.transport_items", typed = "1.21.100", goal, released = "1.21.100", molang, parse = goal::parse::<goal::TransportItems>)]
    BehaviorTransportItems(goal::TransportItems),
    /// `minecraft:behavior.use_kinetic_weapon`.
    #[component(name = "minecraft:behavior.use_kinetic_weapon", typed = "1.21.120", released = "1.21.120", molang, parse = goal::parse::<goal::UseKineticWeapon>)]
    BehaviorUseKineticWeapon(goal::UseKineticWeapon),
    /// `minecraft:behavior.wither_random_attack_pos_goal`.
    #[component(name = "minecraft:behavior.wither_random_attack_pos_goal", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::WitherRandomAttackPosGoal>)]
    BehaviorWitherRandomAttackPosGoal(goal::WitherRandomAttackPosGoal),
    /// `minecraft:behavior.wither_target_highest_damage`.
    #[component(name = "minecraft:behavior.wither_target_highest_damage", typed = "1.26.30", goal, released = "1.8.0", parse = goal::parse::<goal::WitherTargetHighestDamage>)]
    BehaviorWitherTargetHighestDamage(goal::WitherTargetHighestDamage),
    /// `minecraft:behavior.work`.
    #[component(name = "minecraft:behavior.work", typed = "1.26.10", released = "1.8.0", molang, parse = goal::parse::<goal::Work>)]
    BehaviorWork(goal::Work),
    /// `minecraft:behavior.work_composter`.
    #[component(name = "minecraft:behavior.work_composter", typed = "1.26.10", released = "1.8.0", molang, parse = goal::parse::<goal::WorkComposter>)]
    BehaviorWorkComposter(goal::WorkComposter),
    /// `minecraft:block_climber`.
    #[component(name = "minecraft:block_climber", typed = "1.26.20", released = "1.17.10", supersedes = "minecraft:scaffolding_climber", parse = parse_typed::<BlockClimber>)]
    BlockClimber(BlockClimber),
    /// `minecraft:block_movement_slowdown_immunity`.
    #[component(name = "minecraft:block_movement_slowdown_immunity", typed = "1.26.50", released = "1.26.50", molang, parse = parse_typed::<BlockMovementSlowdownImmunity>)]
    BlockMovementSlowdownImmunity(BlockMovementSlowdownImmunity),
    /// `minecraft:body_rotation_always_follows_head`.
    #[component(name = "minecraft:body_rotation_always_follows_head", typed = "1.21.120", released = "1.8.0", parse = parse_typed::<BodyRotationAlwaysFollowsHead>)]
    BodyRotationAlwaysFollowsHead(BodyRotationAlwaysFollowsHead),
    /// `minecraft:body_rotation_blocked`.
    #[component(name = "minecraft:body_rotation_blocked", typed = "1.21.120", released = "1.8.0", parse = parse_typed::<BodyRotationBlocked>)]
    BodyRotationBlocked(BodyRotationBlocked),
    /// `minecraft:boss`.
    #[component(name = "minecraft:boss", typed = "1.26.50", released = "1.8.0", parse = parse_typed::<Boss>)]
    Boss(Boss),
    /// `minecraft:bucketable`.
    #[component(name = "minecraft:bucketable", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<Bucketable>)]
    Bucketable(Bucketable),
    /// `minecraft:burns_in_daylight`.
    #[component(name = "minecraft:burns_in_daylight", typed = "1.21.130", released = "1.8.0", parse = parse_typed::<BurnsInDaylight>)]
    BurnsInDaylight(BurnsInDaylight),
    /// `minecraft:can_climb`.
    #[component(name = "minecraft:can_climb", typed = "1.21.120", released = "1.8.0", parse = parse_typed::<CanClimb>)]
    CanClimb(CanClimb),
    /// `minecraft:can_fly`.
    #[component(name = "minecraft:can_fly", typed = "1.21.120", released = "1.8.0", parse = parse_typed::<CanFly>)]
    CanFly(CanFly),
    /// `minecraft:can_join_raid`.
    #[component(name = "minecraft:can_join_raid", typed = "1.26.20", released = "1.8.0", parse = parse_typed::<CanJoinRaid>)]
    CanJoinRaid(CanJoinRaid),
    /// `minecraft:can_power_jump`.
    #[component(name = "minecraft:can_power_jump", typed = "1.21.120", released = "1.8.0", parse = parse_typed::<CanPowerJump>)]
    CanPowerJump(CanPowerJump),
    /// `minecraft:can_stand_on_powder_snow`.
    #[component(name = "minecraft:can_stand_on_powder_snow", typed = "1.26.50", released = "1.26.50", parse = parse_typed::<CanStandOnPowderSnow>)]
    CanStandOnPowderSnow(CanStandOnPowderSnow),
    /// `minecraft:cannot_be_attacked`.
    #[component(name = "minecraft:cannot_be_attacked", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<CannotBeAttacked>)]
    CannotBeAttacked(CannotBeAttacked),
    /// `minecraft:collision_box`.
    #[component(name = "minecraft:collision_box", typed = "1.26.50", released = "1.8.0", parse = parse_typed::<CollisionBox>)]
    CollisionBox(CollisionBox),
    /// `minecraft:damage_over_time`.
    #[component(name = "minecraft:damage_over_time", typed = "1.26.50", released = "1.8.0", parse = parse_typed::<DamageOverTime>)]
    DamageOverTime(DamageOverTime),
    /// `minecraft:dimension_bound`.
    #[component(name = "minecraft:dimension_bound", typed = "1.26.20", released = "1.21.40", parse = parse_typed::<DimensionBound>)]
    DimensionBound(DimensionBound),
    /// `minecraft:exhaustion_values`.
    #[component(name = "minecraft:exhaustion_values", typed = "1.26.50", released = "1.18.30", parse = parse_typed::<ExhaustionValues>)]
    ExhaustionValues(ExhaustionValues),
    /// `minecraft:experience_reward`.
    #[component(name = "minecraft:experience_reward", typed = "1.21.110", released = "1.8.0", molang, parse = parse_typed::<ExperienceReward>)]
    ExperienceReward(ExperienceReward),
    /// `minecraft:fire_immune`.
    #[component(name = "minecraft:fire_immune", typed = "1.21.120", released = "1.8.0", parse = parse_typed::<FireImmune>)]
    FireImmune(FireImmune),
    /// `minecraft:floats_in_liquid`.
    #[component(name = "minecraft:floats_in_liquid", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<FloatsInLiquid>)]
    FloatsInLiquid(FloatsInLiquid),
    /// `minecraft:flocking`.
    #[component(name = "minecraft:flocking", typed = "1.26.50", released = "1.8.0", parse = parse_typed::<Flocking>)]
    Flocking(Flocking),
    /// `minecraft:freezing_immune`.
    #[component(name = "minecraft:freezing_immune", typed = "1.26.50", released = "1.26.50", parse = parse_typed::<FreezingImmune>)]
    FreezingImmune(FreezingImmune),
    /// `minecraft:freezing_vulnerable`.
    #[component(name = "minecraft:freezing_vulnerable", typed = "1.26.50", released = "1.26.50", parse = parse_typed::<FreezingVulnerable>)]
    FreezingVulnerable(FreezingVulnerable),
    /// `minecraft:game_event_movement_tracking`.
    #[component(name = "minecraft:game_event_movement_tracking", typed = "1.26.50", released = "1.8.0", parse = parse_typed::<GameEventMovementTracking>)]
    GameEventMovementTracking(GameEventMovementTracking),
    /// `minecraft:grows_crop`.
    #[component(name = "minecraft:grows_crop", typed = "1.26.50", released = "1.8.0", parse = parse_typed::<GrowsCrop>)]
    GrowsCrop(GrowsCrop),
    /// `minecraft:hide`.
    #[component(name = "minecraft:hide", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<Hide>)]
    Hide(Hide),
    /// `minecraft:home`.
    #[component(name = "minecraft:home", typed = "1.26.50", released = "1.8.0", parse = parse_typed::<Home>)]
    Home(Home),
    /// `minecraft:ignore_cannot_be_attacked`.
    #[component(name = "minecraft:ignore_cannot_be_attacked", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<IgnoreCannotBeAttacked>)]
    IgnoreCannotBeAttacked(IgnoreCannotBeAttacked),
    /// `minecraft:input_ground_controlled`.
    #[component(name = "minecraft:input_ground_controlled", typed = "1.21.120", released = "1.8.0", parse = parse_typed::<InputGroundControlled>)]
    InputGroundControlled(InputGroundControlled),
    /// `minecraft:insomnia`.
    #[component(name = "minecraft:insomnia", typed = "1.26.50", released = "1.8.0", parse = parse_typed::<Insomnia>)]
    Insomnia(Insomnia),
    /// `minecraft:instant_despawn`.
    #[component(name = "minecraft:instant_despawn", typed = "1.26.50", released = "1.8.0", parse = parse_typed::<InstantDespawn>)]
    InstantDespawn(InstantDespawn),
    /// `minecraft:interact`.
    #[component(name = "minecraft:interact", typed = "1.26.50", released = "1.8.0", parse = parse_typed::<Interact>)]
    Interact(Interact),
    /// `minecraft:is_baby`.
    #[component(name = "minecraft:is_baby", typed = "1.21.120", released = "1.8.0", parse = parse_typed::<IsBaby>)]
    IsBaby(IsBaby),
    /// `minecraft:is_charged`.
    #[component(name = "minecraft:is_charged", typed = "1.21.120", released = "1.8.0", parse = parse_typed::<IsCharged>)]
    IsCharged(IsCharged),
    /// `minecraft:is_chested`.
    #[component(name = "minecraft:is_chested", typed = "1.21.120", released = "1.8.0", parse = parse_typed::<IsChested>)]
    IsChested(IsChested),
    /// `minecraft:is_hidden_when_invisible`.
    #[component(name = "minecraft:is_hidden_when_invisible", typed = "1.21.120", released = "1.8.0", parse = parse_typed::<IsHiddenWhenInvisible>)]
    IsHiddenWhenInvisible(IsHiddenWhenInvisible),
    /// `minecraft:is_ignited`.
    #[component(name = "minecraft:is_ignited", typed = "1.21.120", released = "1.8.0", parse = parse_typed::<IsIgnited>)]
    IsIgnited(IsIgnited),
    /// `minecraft:is_illager_captain`.
    #[component(name = "minecraft:is_illager_captain", typed = "1.21.120", released = "1.8.0", parse = parse_typed::<IsIllagerCaptain>)]
    IsIllagerCaptain(IsIllagerCaptain),
    /// `minecraft:is_pregnant`.
    #[component(name = "minecraft:is_pregnant", typed = "1.21.120", released = "1.8.0", parse = parse_typed::<IsPregnant>)]
    IsPregnant(IsPregnant),
    /// `minecraft:is_saddled`.
    #[component(name = "minecraft:is_saddled", typed = "1.21.120", released = "1.8.0", parse = parse_typed::<IsSaddled>)]
    IsSaddled(IsSaddled),
    /// `minecraft:is_shaking`.
    #[component(name = "minecraft:is_shaking", typed = "1.21.120", released = "1.8.0", parse = parse_typed::<IsShaking>)]
    IsShaking(IsShaking),
    /// `minecraft:is_sheared`.
    #[component(name = "minecraft:is_sheared", typed = "1.21.120", released = "1.8.0", parse = parse_typed::<IsSheared>)]
    IsSheared(IsSheared),
    /// `minecraft:is_stackable`.
    #[component(name = "minecraft:is_stackable", typed = "1.21.120", released = "1.8.0", parse = parse_typed::<IsStackable>)]
    IsStackable(IsStackable),
    /// `minecraft:is_stunned`.
    #[component(name = "minecraft:is_stunned", typed = "1.21.120", released = "1.8.0", parse = parse_typed::<IsStunned>)]
    IsStunned(IsStunned),
    /// `minecraft:is_tamed`.
    #[component(name = "minecraft:is_tamed", typed = "1.21.120", released = "1.8.0", parse = parse_typed::<IsTamed>)]
    IsTamed(IsTamed),
    /// `minecraft:item_hopper`.
    #[component(name = "minecraft:item_hopper", typed = "1.26.20", released = "1.8.0", parse = parse_typed::<ItemHopper>)]
    ItemHopper(ItemHopper),
    /// `minecraft:jump.dynamic`.
    #[component(name = "minecraft:jump.dynamic", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<JumpDynamic>)]
    JumpDynamic(JumpDynamic),
    /// `minecraft:jump.static`.
    #[component(name = "minecraft:jump.static", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<JumpStatic>)]
    JumpStatic(JumpStatic),
    /// `minecraft:managed_wandering_trader`.
    #[component(name = "minecraft:managed_wandering_trader", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<ManagedWanderingTrader>)]
    ManagedWanderingTrader(ManagedWanderingTrader),
    /// `minecraft:memory_behavior.follow_target`.
    #[component(name = "minecraft:memory_behavior.follow_target", typed = "1.26.20", goal, released = "1.8.0", parse = goal::parse::<goal::MemoryFollowTarget>)]
    MemoryBehaviorFollowTarget(goal::MemoryFollowTarget),
    /// `minecraft:memory_behavior.jump_to_block` (a `.<name>` family).
    #[component(name = "minecraft:memory_behavior.jump_to_block", typed = "1.26.50", released = "1.26.50", molang, parse = goal::parse::<goal::MemoryJumpToBlock>)]
    MemoryBehaviorJumpToBlock(goal::MemoryJumpToBlock),
    /// `minecraft:memory_behavior.look_at_entity`.
    #[component(name = "minecraft:memory_behavior.look_at_entity", typed = "1.26.10", goal, released = "1.8.0", parse = goal::parse::<goal::MemoryLookAtEntity>)]
    MemoryBehaviorLookAtEntity(goal::MemoryLookAtEntity),
    /// `minecraft:memory_behavior.move_to_position`.
    #[component(name = "minecraft:memory_behavior.move_to_position", typed = "1.26.20", released = "1.26.20", molang, parse = goal::parse::<goal::MemoryMoveToPosition>)]
    MemoryBehaviorMoveToPosition(goal::MemoryMoveToPosition),
    /// `minecraft:memory_behavior.prepare_ram_attack` (a `.<name>` family).
    #[component(name = "minecraft:memory_behavior.prepare_ram_attack", typed = "1.26.50", released = "1.26.50", molang, parse = goal::parse::<goal::MemoryPrepareRamAttack>)]
    MemoryBehaviorPrepareRamAttack(goal::MemoryPrepareRamAttack),
    /// `minecraft:memory_behavior.ram_attack` (a `.<name>` family).
    #[component(name = "minecraft:memory_behavior.ram_attack", typed = "1.26.50", released = "1.26.50", molang, parse = goal::parse::<goal::MemoryRamAttack>)]
    MemoryBehaviorRamAttack(goal::MemoryRamAttack),
    /// `minecraft:memory_behavior.wait`.
    #[component(name = "minecraft:memory_behavior.wait", typed = "1.26.0", goal, released = "1.8.0", parse = goal::parse::<goal::MemoryWait>)]
    MemoryBehaviorWait(goal::MemoryWait),
    /// `minecraft:memory_sensor.find_nearest_block`.
    #[component(name = "minecraft:memory_sensor.find_nearest_block", typed = "1.26.30", released = "1.8.0", molang, parse = goal::parse::<goal::MemorySensorFindNearestBlock>)]
    MemorySensorFindNearestBlock(goal::MemorySensorFindNearestBlock),
    /// `minecraft:memory_sensor.find_nearest_entity`.
    #[component(name = "minecraft:memory_sensor.find_nearest_entity", typed = "1.26.20", released = "1.8.0", parse = goal::parse::<goal::MemorySensorFindNearestEntity>)]
    MemorySensorFindNearestEntity(goal::MemorySensorFindNearestEntity),
    /// `minecraft:memory_sensor.find_nearest_poi`.
    #[component(name = "minecraft:memory_sensor.find_nearest_poi", typed = "1.26.20", released = "1.8.0", parse = goal::parse::<goal::MemorySensorFindNearestPoi>)]
    MemorySensorFindNearestPoi(goal::MemorySensorFindNearestPoi),
    /// `minecraft:memory_sensor.find_ram_charge_position`.
    #[component(name = "minecraft:memory_sensor.find_ram_charge_position", typed = "1.26.50", released = "1.26.50", parse = goal::parse::<goal::MemorySensorFindRamChargePosition>)]
    MemorySensorFindRamChargePosition(goal::MemorySensorFindRamChargePosition),
    /// `minecraft:memory_sensor.game_event`.
    #[component(name = "minecraft:memory_sensor.game_event", typed = "1.26.20", released = "1.26.20", parse = goal::parse::<goal::MemorySensorGameEvent>)]
    MemorySensorGameEvent(goal::MemorySensorGameEvent),
    /// `minecraft:memory_sensor.in_range_of_entity`.
    #[component(name = "minecraft:memory_sensor.in_range_of_entity", typed = "1.26.0", released = "1.8.0", parse = goal::parse::<goal::MemorySensorInRangeOfEntity>)]
    MemorySensorInRangeOfEntity(goal::MemorySensorInRangeOfEntity),
    /// `minecraft:not_pickable_from_inside`.
    #[component(name = "minecraft:not_pickable_from_inside", typed = "1.26.40", released = "1.26.40", parse = parse_typed::<NotPickableFromInside>)]
    NotPickableFromInside(NotPickableFromInside),
    /// `minecraft:on_death`.
    #[component(name = "minecraft:on_death", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<OnDeath>)]
    OnDeath(OnDeath),
    /// `minecraft:on_equipment_changed`.
    #[component(name = "minecraft:on_equipment_changed", typed = "1.26.20", released = "1.26.20", triggers = ["slots[].on_equip", "slots[].on_unequip"], parse = parse_typed::<OnEquipmentChanged>)]
    OnEquipmentChanged(OnEquipmentChanged),
    /// `minecraft:on_friendly_anger`.
    #[component(name = "minecraft:on_friendly_anger", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<OnFriendlyAnger>)]
    OnFriendlyAnger(OnFriendlyAnger),
    /// `minecraft:on_hurt`.
    #[component(name = "minecraft:on_hurt", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<OnHurt>)]
    OnHurt(OnHurt),
    /// `minecraft:on_hurt_by_player`.
    #[component(name = "minecraft:on_hurt_by_player", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<OnHurtByPlayer>)]
    OnHurtByPlayer(OnHurtByPlayer),
    /// `minecraft:on_ignite`.
    #[component(name = "minecraft:on_ignite", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<OnIgnite>)]
    OnIgnite(OnIgnite),
    /// `minecraft:on_start_landing`.
    #[component(name = "minecraft:on_start_landing", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<OnStartLanding>)]
    OnStartLanding(OnStartLanding),
    /// `minecraft:on_start_takeoff`.
    #[component(name = "minecraft:on_start_takeoff", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<OnStartTakeoff>)]
    OnStartTakeoff(OnStartTakeoff),
    /// `minecraft:on_target_acquired`.
    #[component(name = "minecraft:on_target_acquired", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<OnTargetAcquired>)]
    OnTargetAcquired(OnTargetAcquired),
    /// `minecraft:on_target_escape`.
    #[component(name = "minecraft:on_target_escape", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<OnTargetEscape>)]
    OnTargetEscape(OnTargetEscape),
    /// `minecraft:on_wake_with_owner`.
    #[component(name = "minecraft:on_wake_with_owner", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<OnWakeWithOwner>)]
    OnWakeWithOwner(OnWakeWithOwner),
    /// `minecraft:out_of_control`.
    #[component(name = "minecraft:out_of_control", typed = "1.26.20", released = "1.8.0", parse = parse_typed::<OutOfControl>)]
    OutOfControl(OutOfControl),
    /// `minecraft:persistent`.
    #[component(name = "minecraft:persistent", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<Persistent>)]
    Persistent(Persistent),
    /// `minecraft:physics`.
    #[component(name = "minecraft:physics", typed = "1.26.50", released = "1.8.0", parse = parse_typed::<Physics>)]
    Physics(Physics),
    /// `minecraft:preferred_path`.
    #[component(name = "minecraft:preferred_path", typed = "1.26.50", released = "1.8.0", molang, parse = parse_typed::<PreferredPath>)]
    PreferredPath(PreferredPath),
    /// `minecraft:projectile`.
    #[component(name = "minecraft:projectile", typed = "1.26.50", released = "1.8.0", fields = [{ name = "on_hit.impact_damage.apply_knockback_to_blocking_targets", since_engine = "1.21.120" }, { name = "on_hit.impact_damage.ceil_pre_critical_damage", since_engine = "1.26.40" }, { name = "on_hit.impact_damage.difficulty_randomization", since_engine = "1.26.40" }], parse = parse_typed::<Projectile>)]
    Projectile(Projectile),
    /// `minecraft:pushable_by_entity`.
    #[component(name = "minecraft:pushable_by_entity", typed = "1.26.20", parse = parse_typed::<PushableByEntity>)]
    PushableByEntity(PushableByEntity),
    /// `minecraft:renders_when_invisible`.
    #[component(name = "minecraft:renders_when_invisible", typed = "1.21.120", released = "1.21.60", parse = parse_typed::<RendersWhenInvisible>)]
    RendersWhenInvisible(RendersWhenInvisible),
    /// `minecraft:rotation_axis_aligned`.
    #[component(name = "minecraft:rotation_axis_aligned", typed = "1.21.120", released = "1.21.100", supersedes = "minecraft:body_rotation_axis_aligned", parse = parse_typed::<RotationAxisAligned>)]
    RotationAxisAligned(RotationAxisAligned),
    /// `minecraft:rotation_locked_to_vehicle`.
    #[component(name = "minecraft:rotation_locked_to_vehicle", typed = "1.21.120", released = "1.21.120", parse = parse_typed::<RotationLockedToVehicle>)]
    RotationLockedToVehicle(RotationLockedToVehicle),
    /// `minecraft:spawn_on_death`.
    #[component(name = "minecraft:spawn_on_death", typed = "1.26.20", released = "1.26.20", parse = parse_typed::<SpawnOnDeath>)]
    SpawnOnDeath(SpawnOnDeath),
    /// `minecraft:suspect_tracking`.
    #[component(name = "minecraft:suspect_tracking", typed = "1.26.20", released = "1.8.0", parse = parse_typed::<SuspectTracking>)]
    SuspectTracking(SuspectTracking),
    /// `minecraft:tamemount`.
    #[component(name = "minecraft:tamemount", typed = "1.26.50", released = "1.8.0", molang, parse = parse_typed::<Tamemount>)]
    Tamemount(Tamemount),
    /// `minecraft:tick_world`.
    #[component(name = "minecraft:tick_world", typed = "1.21.100", released = "1.8.0", parse = parse_typed::<TickWorld>)]
    TickWorld(TickWorld),
    /// `minecraft:trade_resupply`.
    #[component(name = "minecraft:trade_resupply", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<TradeResupply>)]
    TradeResupply(TradeResupply),
    /// `minecraft:trade_table`.
    #[component(name = "minecraft:trade_table", typed = "1.26.50", released = "1.8.0", parse = parse_typed::<TradeTable>)]
    TradeTable(TradeTable),
    /// `minecraft:transient`.
    #[component(name = "minecraft:transient", typed = "1.26.20", released = "1.8.0", parse = parse_typed::<Transient>)]
    Transient(Transient),
    /// `minecraft:trust`.
    #[component(name = "minecraft:trust", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<Trust>)]
    Trust(Trust),
    /// `minecraft:uses_legacy_ambient_sounds`.
    #[component(name = "minecraft:uses_legacy_ambient_sounds", typed = "1.26.50", released = "1.26.50", parse = parse_typed::<UsesLegacyAmbientSounds>)]
    UsesLegacyAmbientSounds(UsesLegacyAmbientSounds),
    /// `minecraft:uses_legacy_friction`.
    #[component(name = "minecraft:uses_legacy_friction", typed = "1.26.20", released = "1.26.20", parse = parse_typed::<UsesLegacyFriction>)]
    UsesLegacyFriction(UsesLegacyFriction),
    /// `minecraft:uses_uniform_air_drag`.
    #[component(name = "minecraft:uses_uniform_air_drag", typed = "1.26.20", released = "1.26.20", parse = parse_typed::<UsesUniformAirDrag>)]
    UsesUniformAirDrag(UsesUniformAirDrag),
    /// `minecraft:vibration_damper`.
    #[component(name = "minecraft:vibration_damper", typed = "1.26.20", released = "1.8.0", parse = parse_typed::<VibrationDamper>)]
    VibrationDamper(VibrationDamper),
    /// `minecraft:vibration_listener`.
    #[component(name = "minecraft:vibration_listener", typed = "1.26.20", released = "1.8.0", parse = parse_typed::<VibrationListener>)]
    VibrationListener(VibrationListener),
    /// `minecraft:wants_jockey`.
    #[component(name = "minecraft:wants_jockey", typed = "1.26.40", released = "1.8.0", parse = parse_typed::<WantsJockey>)]
    WantsJockey(WantsJockey),
    /// `minecraft:wither_target_highest_damage`.
    #[component(name = "minecraft:wither_target_highest_damage", typed = "1.26.30", released = "1.8.0", parse = parse_typed::<WitherTargetHighestDamage>)]
    WitherTargetHighestDamage(WitherTargetHighestDamage),
    // The components 1.26.60 types first (27 component keys; first typed in
    // `history/typed.rs`).
    /// `minecraft:buoyant`.
    #[component(name = "minecraft:buoyant", typed = "1.26.60", released = "1.8.0", molang, parse = parse_typed::<Buoyant>)]
    Buoyant(Buoyant),
    /// `minecraft:dash_action`.
    #[component(name = "minecraft:dash_action", typed = "1.26.60", released = "1.21.110", supersedes = "minecraft:dash", parse = parse_typed::<DashAction>)]
    DashAction(DashAction),
    /// `minecraft:movement.amphibious`.
    #[component(name = "minecraft:movement.amphibious", typed = "1.26.60", released = "1.8.0", parse = parse_typed::<MovementAmphibious>)]
    MovementAmphibious(MovementAmphibious),
    /// `minecraft:movement.basic`.
    #[component(name = "minecraft:movement.basic", typed = "1.26.60", released = "1.8.0", parse = parse_typed::<MovementBasic>)]
    MovementBasic(MovementBasic),
    /// `minecraft:movement.fly`.
    #[component(name = "minecraft:movement.fly", typed = "1.26.60", released = "1.8.0", parse = parse_typed::<MovementFly>)]
    MovementFly(MovementFly),
    /// `minecraft:movement.generic`.
    #[component(name = "minecraft:movement.generic", typed = "1.26.60", released = "1.8.0", parse = parse_typed::<MovementGeneric>)]
    MovementGeneric(MovementGeneric),
    /// `minecraft:movement.glide`.
    #[component(name = "minecraft:movement.glide", typed = "1.26.60", released = "1.8.0", parse = parse_typed::<MovementGlide>)]
    MovementGlide(MovementGlide),
    /// `minecraft:movement.hover`.
    #[component(name = "minecraft:movement.hover", typed = "1.26.60", released = "1.8.0", parse = parse_typed::<MovementHover>)]
    MovementHover(MovementHover),
    /// `minecraft:movement.jump`.
    #[component(name = "minecraft:movement.jump", typed = "1.26.60", released = "1.8.0", parse = parse_typed::<MovementJump>)]
    MovementJump(MovementJump),
    /// `minecraft:movement.skip`.
    #[component(name = "minecraft:movement.skip", typed = "1.26.60", released = "1.8.0", parse = parse_typed::<MovementSkip>)]
    MovementSkip(MovementSkip),
    /// `minecraft:movement.sway`.
    #[component(name = "minecraft:movement.sway", typed = "1.26.60", released = "1.8.0", parse = parse_typed::<MovementSway>)]
    MovementSway(MovementSway),
    /// `minecraft:rail_movement`.
    #[component(name = "minecraft:rail_movement", typed = "1.26.60", released = "1.8.0", parse = parse_typed::<RailMovement>)]
    RailMovement(RailMovement),
    /// `minecraft:attack`.
    #[component(name = "minecraft:attack", typed = "1.26.60", released = "1.8.0", parse = parse_typed::<Attack>)]
    Attack(Attack),
    /// `minecraft:healable`.
    #[component(name = "minecraft:healable", typed = "1.26.60", released = "1.8.0", molang, parse = parse_typed::<Healable>)]
    Healable(Healable),
    /// `minecraft:spell_effects`.
    #[component(name = "minecraft:spell_effects", typed = "1.26.60", released = "1.8.0", parse = parse_typed::<SpellEffects>)]
    SpellEffects(SpellEffects),
    /// `minecraft:strength`.
    #[component(name = "minecraft:strength", typed = "1.26.60", released = "1.8.0", parse = parse_typed::<Strength>)]
    Strength(Strength),
    /// `minecraft:teleport`.
    #[component(name = "minecraft:teleport", typed = "1.26.60", released = "1.8.0", parse = parse_typed::<Teleport>)]
    Teleport(Teleport),
    /// `minecraft:timer`.
    #[component(name = "minecraft:timer", typed = "1.26.60", released = "1.8.0", parse = parse_typed::<Timer>)]
    Timer(Timer),
    /// `minecraft:trail`.
    #[component(name = "minecraft:trail", typed = "1.26.60", released = "1.8.0", molang, parse = parse_typed::<Trail>)]
    Trail(Trail),
    /// `minecraft:transformation`.
    #[component(name = "minecraft:transformation", typed = "1.26.60", released = "1.8.0", parse = parse_typed::<Transformation>)]
    Transformation(Transformation),
    /// `minecraft:annotation.break_door`.
    #[component(name = "minecraft:annotation.break_door", typed = "1.26.60", released = "1.8.0", parse = parse_typed::<AnnotationBreakDoor>)]
    AnnotationBreakDoor(AnnotationBreakDoor),
    /// `minecraft:economy_trade_table`.
    #[component(name = "minecraft:economy_trade_table", typed = "1.26.60", released = "1.8.0", parse = parse_typed::<EconomyTradeTable>)]
    EconomyTradeTable(EconomyTradeTable),
    /// `minecraft:inventory`.
    #[component(name = "minecraft:inventory", typed = "1.26.60", released = "1.8.0", parse = parse_typed::<Inventory>)]
    Inventory(Inventory),
    /// `minecraft:pushable_by_block`.
    #[component(name = "minecraft:pushable_by_block", typed = "1.26.60", released = "1.26.10", supersedes = "minecraft:pushable", parse = parse_typed::<PushableByBlock>)]
    PushableByBlock(PushableByBlock),
    /// `minecraft:rideable`.
    #[component(name = "minecraft:rideable", typed = "1.26.60", released = "1.8.0", molang, triggers = ["on_rider_enter_event", "on_rider_exit_event"], parse = parse_typed::<Rideable>)]
    Rideable(Rideable),
    /// `minecraft:scale_by_age`.
    #[component(name = "minecraft:scale_by_age", typed = "1.26.60", released = "1.8.0", parse = parse_typed::<ScaleByAge>)]
    ScaleByAge(ScaleByAge),
    /// `minecraft:shareables`.
    #[component(name = "minecraft:shareables", typed = "1.26.60", released = "1.8.0", molang, parse = parse_typed::<Shareables>)]
    Shareables(Shareables),
    /// A custom component (`namespace:name`). Entities have none in vanilla; the variant
    /// completes the shared component model.
    Custom(CustomComponent),
    /// A key no row names, kept as written for write-back (`UnrecognizedComponent`).
    Unknown(Value),
    /// A key vanilla parses at run time from the JSON, kept as written.
    /// Each attribute below is one row of the table.
    #[component(name = "minecraft:absorption", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:ageable", released = "1.8.0", triggers = ["grow_up", "on_pause_growth", "on_reset_growth"], parse = keep_json)]
    #[component(name = "minecraft:air_drag_modifier", released = "1.26.30", requires = [BETA_DOCUMENT], parse = keep_json)]
    #[component(name = "minecraft:ambient_sound_interval", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:angry", released = "1.8.0", triggers = ["calm_event"], parse = keep_json)]
    #[component(name = "minecraft:area_attack", released = "1.8.0", fields = [{ name = "use_self_as_damage_source", since_engine = "1.26.30" }], parse = keep_json)]
    #[component(name = "minecraft:attack_cooldown", released = "1.8.0", triggers = ["attack_cooldown_complete_event"], parse = keep_json)]
    #[component(name = "minecraft:attack_damage", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:balloon", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:balloonable", released = "1.8.0", triggers = ["on_balloon", "on_unballoon"], parse = keep_json)]
    #[component(name = "minecraft:behavior.enderman_leave_block", released = "1.8.0", deprecated_since = "1.21.100", parse = keep_json)]
    #[component(name = "minecraft:behavior.enderman_take_block", released = "1.8.0", deprecated_since = "1.21.100", parse = keep_json)]
    #[component(name = "minecraft:behavior.vex_copy_owner_target", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:behavior.vex_random_move", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:block_sensor", released = "1.8.0", triggers = ["on_break[].on_block_broken"], parse = keep_json)]
    #[component(name = "minecraft:boostable", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:bounciness", released = "1.26.30", requires = [BETA_DOCUMENT], parse = keep_json)]
    #[component(name = "minecraft:break_blocks", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:breathable", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:breedable", released = "1.8.0", triggers = ["breeds_with.*"], parse = keep_json)]
    #[component(name = "minecraft:bribeable", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:celebrate_hunt", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:color", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:color2", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:combat_regeneration", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:conditional_bandwidth_optimization", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:custom_hit_test", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:damage_sensor", released = "1.8.0", triggers = ["triggers[].on_damage"], parse = keep_json)]
    #[component(name = "minecraft:default_look_angle", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:despawn", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:drying_out_timer", released = "1.8.0", triggers = ["dried_out_event", "recover_after_dried_out_event", "stopped_drying_out_event"], parse = keep_json)]
    #[component(name = "minecraft:dweller", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:entity_armor_equipment_slot_mapping", released = "1.21.10", parse = keep_json)]
    #[component(name = "minecraft:entity_sensor", released = "1.8.0", triggers = ["event", "subsensors[].event"], parse = keep_json)]
    #[component(name = "minecraft:environment_sensor", released = "1.8.0", triggers = ["triggers[]"], parse = keep_json)]
    #[component(name = "minecraft:equip_item", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:equipment", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:equippable", released = "1.8.0", triggers = ["slots[].on_equip", "slots[].on_unequip"], parse = keep_json)]
    #[component(name = "minecraft:explode", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:flying_speed", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:follow_range", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:free_camera_controlled", released = "1.21.110", supersedes = "minecraft:input_air_controlled", parse = keep_json)]
    #[component(name = "minecraft:friction_modifier", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:genetics", released = "1.8.0", triggers = ["genes[].genetic_variants[].birth_event"], parse = keep_json)]
    #[component(name = "minecraft:giveable", released = "1.8.0", triggers = ["triggers[].on_give"], parse = keep_json)]
    #[component(name = "minecraft:ground_offset", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:group_size", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:health", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:heartbeat", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:horse.jump_strength", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:hurt_on_condition", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:inside_block_notifier", released = "1.8.0", triggers = ["block_list[].entered_block_event", "block_list[].exited_block_event"], parse = keep_json)]
    #[component(name = "minecraft:is_collidable", released = "1.21.70", parse = keep_json)]
    #[component(name = "minecraft:is_dyeable", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:item_controllable", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:knockback_resistance", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:lava_movement", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:leashable", released = "1.8.0", fields = [{ name = "unleash_on_removal", since_engine = "1.26.30" }], triggers = ["on_leash", "on_unleash"], parse = keep_json)]
    #[component(name = "minecraft:leashable_to", released = "1.8.0", fields = [{ name = "unleash_on_removal", since_engine = "1.26.30" }], parse = keep_json)]
    #[component(name = "minecraft:looked_at", released = "1.21.40", supersedes = "minecraft:lookat", triggers = ["looked_at_event", "not_looked_at_event"], parse = keep_json)]
    #[component(name = "minecraft:loot", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:luck", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:mark_variant", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:memory_behavior.flee_threat", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:memory_sensor.find_nearest_block_from", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:memory_sensor.find_nearest_entity_from", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:memory_sensor.in_range_of_block", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:mob_effect", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:mob_effect_immunity", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:movement", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:movement.dolphin", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:movement_sound_distance_offset", released = "1.21.60", parse = keep_json)]
    #[component(name = "minecraft:nameable", released = "1.8.0", triggers = ["default_trigger", "name_actions[].on_named"], parse = keep_json)]
    #[component(name = "minecraft:navigation.climb", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:navigation.float", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:navigation.fly", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:navigation.generic", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:navigation.hover", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:navigation.swim", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:navigation.walk", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:npc", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:offspring", released = "1.26.0", parse = keep_json)]
    #[component(name = "minecraft:peek", released = "1.8.0", triggers = ["on_close", "on_open", "on_target_open"], parse = keep_json)]
    #[component(name = "minecraft:player.exhaustion", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:player.experience", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:player.hunger", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:player.level", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:player.saturation", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:push_through", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:raid_trigger", released = "1.8.0", triggers = ["triggered_event"], parse = keep_json)]
    #[component(name = "minecraft:rail_sensor", released = "1.8.0", triggers = ["on_activate", "on_deactivate"], parse = keep_json)]
    #[component(name = "minecraft:ravager_blocked", released = "1.8.0", triggers = ["reaction_choices[].value"], parse = keep_json)]
    #[component(name = "minecraft:reflect_projectiles", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:scale", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:scheduler", released = "1.8.0", triggers = ["scheduled_events[]"], parse = keep_json)]
    #[component(name = "minecraft:shooter", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:sittable", released = "1.8.0", triggers = ["sit_event", "stand_event"], parse = keep_json)]
    #[component(name = "minecraft:skin_id", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:sound_volume", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:spawn_egg_interaction", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:spawn_entity", released = "1.8.0", triggers = ["entities[].spawn_item_event"], parse = keep_json)]
    #[component(name = "minecraft:tameable", released = "1.8.0", triggers = ["tame_event"], parse = keep_json)]
    #[component(name = "minecraft:target_nearby_sensor", released = "1.8.0", triggers = ["on_inside_range", "on_outside_range", "on_vision_lost_inside_range"], parse = keep_json)]
    #[component(name = "minecraft:trusting", released = "1.8.0", triggers = ["trust_event"], parse = keep_json)]
    #[component(name = "minecraft:type_family", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:underwater_mount_breathing", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:underwater_movement", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:variable_max_auto_step", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:variant", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:vertical_movement_action", released = "1.21.110", parse = keep_json)]
    #[component(name = "minecraft:walk_animation_speed", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:water_movement", released = "1.8.0", parse = keep_json)]
    // the legacy keys a later key supersedes, deprecated from its release
    // (the steps rename them in older documents).
    #[component(name = "minecraft:scaffolding_climber", released = "1.8.0", deprecated_since = "1.17.10", parse = keep_json)]
    #[component(name = "minecraft:foot_size", released = "1.8.0", deprecated_since = "1.16.100", parse = keep_json)]
    #[component(name = "minecraft:lookat", released = "1.8.0", deprecated_since = "1.21.40", parse = keep_json)]
    #[component(name = "minecraft:body_rotation_axis_aligned", released = "1.8.0", deprecated_since = "1.21.100", parse = keep_json)]
    #[component(name = "minecraft:input_air_controlled", released = "1.8.0", deprecated_since = "1.21.110", parse = keep_json)]
    #[component(name = "minecraft:dash", released = "1.8.0", deprecated_since = "1.21.110", parse = keep_json)]
    #[component(name = "minecraft:pushable", released = "1.8.0", deprecated_since = "1.26.10", parse = keep_json)]
    #[component(name = "minecraft:behavior.follow_target_captain", released = "1.8.0", deprecated_since = "1.26.20", parse = keep_json)]
    Dynamic(Value),
}

#[cfg(test)]
mod tests {
    use super::EntityComponent;

    /// `(row, typed)` of every row the game table has no reader for.
    const PINNED_TYPED: [(&str, Option<&str>); 199] = [
        ("minecraft:absorption", None),
        ("minecraft:air_drag_modifier", None),
        ("minecraft:annotation.open_door", Some("1.26.40")),
        ("minecraft:apply_knockback_rules", Some("1.8.0")),
        ("minecraft:attack", Some("1.26.60")),
        ("minecraft:attack_damage", None),
        ("minecraft:behavior.admire_item", Some("1.26.20")),
        ("minecraft:behavior.aquatic_charge_attack", Some("1.21.120")),
        ("minecraft:behavior.barter", Some("1.26.20")),
        ("minecraft:behavior.beg", Some("1.26.40")),
        ("minecraft:behavior.break_door", Some("1.26.30")),
        ("minecraft:behavior.breed", Some("1.21.90")),
        ("minecraft:behavior.defend_trusted_target", Some("1.26.30")),
        ("minecraft:behavior.door_interact", Some("1.26.30")),
        ("minecraft:behavior.dragondeath", Some("1.26.30")),
        ("minecraft:behavior.dragonholdingpattern", Some("1.26.30")),
        ("minecraft:behavior.dragonlanding", Some("1.26.30")),
        ("minecraft:behavior.dragonscanning", Some("1.26.30")),
        ("minecraft:behavior.dragontakeoff", Some("1.26.30")),
        ("minecraft:behavior.drink_potion", Some("1.26.40")),
        ("minecraft:behavior.eat_carried_item", Some("1.26.20")),
        ("minecraft:behavior.enderman_leave_block", None),
        ("minecraft:behavior.enderman_take_block", None),
        ("minecraft:behavior.find_cover", Some("1.26.20")),
        ("minecraft:behavior.find_mount", Some("1.26.20")),
        (
            "minecraft:behavior.find_underwater_treasure",
            Some("1.26.20"),
        ),
        ("minecraft:behavior.flee_sun", Some("1.26.20")),
        ("minecraft:behavior.float", Some("1.26.20")),
        ("minecraft:behavior.float_tempt", Some("1.26.60")),
        ("minecraft:behavior.float_wander", Some("1.26.40")),
        ("minecraft:behavior.follow_caravan", Some("1.26.30")),
        ("minecraft:behavior.follow_mob", Some("1.26.50")),
        ("minecraft:behavior.follow_owner", Some("1.26.20")),
        ("minecraft:behavior.follow_parent", Some("1.26.20")),
        ("minecraft:behavior.follow_target_captain", None),
        ("minecraft:behavior.follow_target_leader", Some("1.26.30")),
        ("minecraft:behavior.hide", Some("1.26.20")),
        ("minecraft:behavior.hold_ground", Some("1.26.20")),
        ("minecraft:behavior.hover", Some("1.26.20")),
        ("minecraft:behavior.hurt_by_target", Some("1.26.30")),
        ("minecraft:behavior.inspect_bookshelf", Some("1.26.10")),
        ("minecraft:behavior.lay_down", Some("1.26.20")),
        ("minecraft:behavior.lay_egg", Some("1.26.10")),
        ("minecraft:behavior.leap_at_target", Some("1.26.40")),
        ("minecraft:behavior.look_at_entity", Some("1.26.0")),
        ("minecraft:behavior.look_at_player", Some("1.26.0")),
        ("minecraft:behavior.look_at_target", Some("1.26.0")),
        ("minecraft:behavior.look_at_trading_player", Some("1.26.0")),
        ("minecraft:behavior.make_love", Some("1.26.30")),
        ("minecraft:behavior.mingle", Some("1.26.20")),
        ("minecraft:behavior.mount_pathing", Some("1.26.20")),
        ("minecraft:behavior.move_through_village", Some("1.26.20")),
        ("minecraft:behavior.move_to_block", Some("1.26.40")),
        ("minecraft:behavior.move_to_land", Some("1.26.10")),
        ("minecraft:behavior.move_to_lava", Some("1.26.10")),
        ("minecraft:behavior.move_to_liquid", Some("1.26.10")),
        ("minecraft:behavior.move_to_poi", Some("1.26.20")),
        ("minecraft:behavior.move_to_random_block", Some("1.26.20")),
        ("minecraft:behavior.move_to_village", Some("1.26.20")),
        ("minecraft:behavior.move_to_water", Some("1.26.10")),
        ("minecraft:behavior.move_towards_target", Some("1.26.20")),
        ("minecraft:behavior.nap", Some("1.26.40")),
        (
            "minecraft:behavior.nearest_prioritized_attackable_target",
            Some("1.26.30"),
        ),
        ("minecraft:behavior.ocelot_sit_on_block", Some("1.26.10")),
        ("minecraft:behavior.open_door", Some("1.26.30")),
        ("minecraft:behavior.owner_hurt_by_target", Some("1.26.30")),
        ("minecraft:behavior.owner_hurt_target", Some("1.26.30")),
        ("minecraft:behavior.panic", Some("1.26.60")),
        ("minecraft:behavior.pet_sleep_with_owner", Some("1.26.20")),
        ("minecraft:behavior.pickup_items", Some("1.26.20")),
        ("minecraft:behavior.player_ride_tamed", Some("1.26.20")),
        ("minecraft:behavior.raid_garden", Some("1.26.10")),
        ("minecraft:behavior.random_breach", Some("1.26.30")),
        ("minecraft:behavior.random_fly", Some("1.26.30")),
        ("minecraft:behavior.random_hover", Some("1.26.20")),
        ("minecraft:behavior.random_look_around", Some("1.26.30")),
        (
            "minecraft:behavior.random_look_around_and_sit",
            Some("1.26.30"),
        ),
        ("minecraft:behavior.random_sitting", Some("1.26.20")),
        ("minecraft:behavior.random_stroll", Some("1.26.20")),
        ("minecraft:behavior.random_swim", Some("1.26.30")),
        ("minecraft:behavior.receive_love", Some("1.26.30")),
        ("minecraft:behavior.restrict_open_door", Some("1.26.30")),
        ("minecraft:behavior.restrict_sun", Some("1.26.30")),
        ("minecraft:behavior.rise_to_liquid_level", Some("1.26.30")),
        ("minecraft:behavior.roll", Some("1.26.30")),
        ("minecraft:behavior.run_around_like_crazy", Some("1.26.30")),
        ("minecraft:behavior.scared", Some("1.26.30")),
        ("minecraft:behavior.send_event", Some("1.26.60")),
        ("minecraft:behavior.share_items", Some("1.26.30")),
        (
            "minecraft:behavior.silverfish_merge_with_stone",
            Some("1.26.40"),
        ),
        (
            "minecraft:behavior.silverfish_wake_up_friends",
            Some("1.26.40"),
        ),
        ("minecraft:behavior.skeleton_horse_trap", Some("1.26.30")),
        ("minecraft:behavior.sleep", Some("1.26.20")),
        ("minecraft:behavior.snacking", Some("1.26.40")),
        ("minecraft:behavior.sneeze", Some("1.26.30")),
        ("minecraft:behavior.squid_dive", Some("1.26.40")),
        ("minecraft:behavior.squid_flee", Some("1.26.40")),
        ("minecraft:behavior.squid_idle", Some("1.26.40")),
        (
            "minecraft:behavior.squid_move_away_from_ground",
            Some("1.26.40"),
        ),
        ("minecraft:behavior.squid_out_of_water", Some("1.26.40")),
        (
            "minecraft:behavior.stalk_and_pounce_on_target",
            Some("1.26.40"),
        ),
        ("minecraft:behavior.stay_while_sitting", Some("1.26.30")),
        ("minecraft:behavior.stomp_turtle_egg", Some("1.26.10")),
        ("minecraft:behavior.stroll_towards_village", Some("1.26.20")),
        ("minecraft:behavior.summon_entity", Some("1.26.40")),
        ("minecraft:behavior.swell", Some("1.26.30")),
        ("minecraft:behavior.target_when_pushed", Some("1.26.30")),
        ("minecraft:behavior.tempt", Some("1.26.60")),
        ("minecraft:behavior.trade_interest", Some("1.26.30")),
        ("minecraft:behavior.transport_items", Some("1.21.100")),
        ("minecraft:behavior.use_kinetic_weapon", Some("1.21.120")),
        ("minecraft:behavior.vex_copy_owner_target", None),
        ("minecraft:behavior.vex_random_move", None),
        (
            "minecraft:behavior.wither_random_attack_pos_goal",
            Some("1.26.30"),
        ),
        (
            "minecraft:behavior.wither_target_highest_damage",
            Some("1.26.30"),
        ),
        (
            "minecraft:block_movement_slowdown_immunity",
            Some("1.26.50"),
        ),
        ("minecraft:body_rotation_axis_aligned", None),
        ("minecraft:bounciness", None),
        ("minecraft:break_blocks", None),
        ("minecraft:bucketable", Some("1.26.40")),
        ("minecraft:can_stand_on_powder_snow", Some("1.26.50")),
        ("minecraft:dash", None),
        ("minecraft:dweller", None),
        ("minecraft:follow_range", None),
        ("minecraft:foot_size", None),
        ("minecraft:freezing_immune", Some("1.26.50")),
        ("minecraft:freezing_vulnerable", Some("1.26.50")),
        ("minecraft:friction_modifier", None),
        ("minecraft:health", None),
        ("minecraft:hide", Some("1.26.40")),
        ("minecraft:horse.jump_strength", None),
        ("minecraft:input_air_controlled", None),
        ("minecraft:jump.dynamic", Some("1.26.40")),
        ("minecraft:jump.static", Some("1.26.40")),
        ("minecraft:knockback_resistance", None),
        ("minecraft:lava_movement", None),
        ("minecraft:lookat", None),
        ("minecraft:luck", None),
        ("minecraft:managed_wandering_trader", Some("1.26.40")),
        ("minecraft:memory_behavior.flee_threat", None),
        ("minecraft:memory_behavior.follow_target", Some("1.26.20")),
        ("minecraft:memory_behavior.jump_to_block", Some("1.26.50")),
        ("minecraft:memory_behavior.look_at_entity", Some("1.26.10")),
        (
            "minecraft:memory_behavior.move_to_position",
            Some("1.26.20"),
        ),
        (
            "minecraft:memory_behavior.prepare_ram_attack",
            Some("1.26.50"),
        ),
        ("minecraft:memory_behavior.ram_attack", Some("1.26.50")),
        ("minecraft:memory_behavior.wait", Some("1.26.0")),
        (
            "minecraft:memory_sensor.find_nearest_block",
            Some("1.26.30"),
        ),
        ("minecraft:memory_sensor.find_nearest_block_from", None),
        (
            "minecraft:memory_sensor.find_nearest_entity",
            Some("1.26.20"),
        ),
        ("minecraft:memory_sensor.find_nearest_entity_from", None),
        ("minecraft:memory_sensor.find_nearest_poi", Some("1.26.20")),
        (
            "minecraft:memory_sensor.find_ram_charge_position",
            Some("1.26.50"),
        ),
        ("minecraft:memory_sensor.game_event", Some("1.26.20")),
        ("minecraft:memory_sensor.in_range_of_block", None),
        ("minecraft:memory_sensor.in_range_of_entity", Some("1.26.0")),
        ("minecraft:movement", None),
        ("minecraft:movement.amphibious", Some("1.26.60")),
        ("minecraft:movement.basic", Some("1.26.60")),
        ("minecraft:movement.dolphin", None),
        ("minecraft:movement.fly", Some("1.26.60")),
        ("minecraft:movement.generic", Some("1.26.60")),
        ("minecraft:movement.glide", Some("1.26.60")),
        ("minecraft:movement.hover", Some("1.26.60")),
        ("minecraft:movement.jump", Some("1.26.60")),
        ("minecraft:movement.skip", Some("1.26.60")),
        ("minecraft:movement.sway", Some("1.26.60")),
        ("minecraft:navigation.climb", None),
        ("minecraft:navigation.float", None),
        ("minecraft:navigation.fly", None),
        ("minecraft:navigation.generic", None),
        ("minecraft:navigation.hover", None),
        ("minecraft:navigation.swim", None),
        ("minecraft:navigation.walk", None),
        ("minecraft:not_pickable_from_inside", Some("1.26.40")),
        ("minecraft:on_equipment_changed", Some("1.26.20")),
        ("minecraft:persistent", Some("1.26.40")),
        ("minecraft:player.exhaustion", None),
        ("minecraft:player.experience", None),
        ("minecraft:player.hunger", None),
        ("minecraft:player.level", None),
        ("minecraft:player.saturation", None),
        ("minecraft:preferred_path", Some("1.26.50")),
        ("minecraft:projectile", Some("1.26.50")),
        ("minecraft:pushable", None),
        ("minecraft:ravager_blocked", None),
        ("minecraft:scaffolding_climber", None),
        ("minecraft:spawn_on_death", Some("1.26.20")),
        ("minecraft:spell_effects", Some("1.26.60")),
        ("minecraft:tick_world", Some("1.21.100")),
        ("minecraft:trade_resupply", Some("1.26.40")),
        ("minecraft:trail", Some("1.26.60")),
        ("minecraft:transformation", Some("1.26.60")),
        ("minecraft:trust", Some("1.26.40")),
        ("minecraft:underwater_movement", None),
        ("minecraft:uses_legacy_ambient_sounds", Some("1.26.50")),
        ("minecraft:uses_uniform_air_drag", Some("1.26.20")),
        ("minecraft:water_movement", None),
        ("minecraft:wither_target_highest_damage", Some("1.26.30")),
    ];

    /// The typed version of every row the game table has no reader for, pinned (the
    /// `typed_table` test holds the others to the game table): `None` where the row is never
    /// read by a typed struct.
    #[test]
    fn the_rows_the_game_table_lacks_keep_their_pinned_typed_version() {
        let rows: Vec<(&str, Option<String>)> = EntityComponent::SCHEMA_ROWS
            .iter()
            .filter(|row| crate::game_schema::typed_since(row.name).is_none())
            .map(|row| (row.name, row.typed.map(|version| version.to_string())))
            .collect();
        let pinned: Vec<(&str, Option<String>)> = PINNED_TYPED
            .iter()
            .map(|(name, typed)| (*name, typed.map(str::to_owned)))
            .collect();
        assert_eq!(rows, pinned);
    }
}
