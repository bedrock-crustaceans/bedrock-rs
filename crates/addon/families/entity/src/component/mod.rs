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
#[component(family = Entity, custom = Custom, unknown = Unknown, dynamic = Dynamic)]
#[non_exhaustive]
#[allow(
    clippy::large_enum_variant,
    reason = "one variant per component, sized as written; boxing would lose the unknown members `keep_unknown_members` keeps"
)]
pub enum EntityComponent {
    /// `minecraft:addrider`.
    #[component(name = "minecraft:addrider", released = "1.8.0", parse = parse_typed::<Addrider>)]
    Addrider(Addrider),
    /// `minecraft:admire_item`.
    #[component(name = "minecraft:admire_item", released = "1.8.0", parse = parse_typed::<AdmireItem>)]
    AdmireItem(AdmireItem),
    /// `minecraft:anger_level`.
    #[component(name = "minecraft:anger_level", released = "1.8.0", molang, parse = parse_typed::<AngerLevel>)]
    AngerLevel(AngerLevel),
    /// `minecraft:annotation.open_door`.
    #[component(name = "minecraft:annotation.open_door", released = "1.8.0", parse = parse_typed::<AnnotationOpenDoor>)]
    AnnotationOpenDoor(AnnotationOpenDoor),
    /// `minecraft:apply_knockback_rules`.
    #[component(name = "minecraft:apply_knockback_rules", parse = parse_typed::<ApplyKnockbackRules>)]
    ApplyKnockbackRules(ApplyKnockbackRules),
    /// `minecraft:barter`.
    #[component(name = "minecraft:barter", released = "1.8.0", parse = parse_typed::<Barter>)]
    Barter(Barter),
    /// `minecraft:behavior.admire_item`.
    #[component(name = "minecraft:behavior.admire_item", released = "1.8.0", molang, parse = goal::parse::<goal::AdmireItem>)]
    BehaviorAdmireItem(goal::AdmireItem),
    /// `minecraft:behavior.aquatic_charge_attack`.
    #[component(name = "minecraft:behavior.aquatic_charge_attack", released = "1.21.120", parse = goal::parse::<goal::AquaticChargeAttack>)]
    BehaviorAquaticChargeAttack(goal::AquaticChargeAttack),
    /// `minecraft:behavior.avoid_block`.
    #[component(name = "minecraft:behavior.avoid_block", released = "1.8.0", molang, parse = goal::parse::<goal::AvoidBlock>)]
    BehaviorAvoidBlock(goal::AvoidBlock),
    /// `minecraft:behavior.avoid_mob_type`.
    #[component(name = "minecraft:behavior.avoid_mob_type", released = "1.8.0", molang, parse = goal::parse::<goal::AvoidMobType>)]
    BehaviorAvoidMobType(goal::AvoidMobType),
    /// `minecraft:behavior.barter`.
    #[component(name = "minecraft:behavior.barter", released = "1.8.0", parse = goal::parse::<goal::Barter>)]
    BehaviorBarter(goal::Barter),
    /// `minecraft:behavior.beg`.
    #[component(name = "minecraft:behavior.beg", released = "1.8.0", molang, parse = goal::parse::<goal::Beg>)]
    BehaviorBeg(goal::Beg),
    /// `minecraft:behavior.break_door`.
    #[component(name = "minecraft:behavior.break_door", released = "1.8.0", parse = goal::parse::<goal::BreakDoor>)]
    BehaviorBreakDoor(goal::BreakDoor),
    /// `minecraft:behavior.breed`.
    #[component(name = "minecraft:behavior.breed", released = "1.8.0", parse = goal::parse::<goal::Breed>)]
    BehaviorBreed(goal::Breed),
    /// `minecraft:behavior.celebrate`.
    #[component(name = "minecraft:behavior.celebrate", released = "1.8.0", molang, parse = goal::parse::<goal::Celebrate>)]
    BehaviorCelebrate(goal::Celebrate),
    /// `minecraft:behavior.celebrate_survive`.
    #[component(name = "minecraft:behavior.celebrate_survive", released = "1.8.0", molang, parse = goal::parse::<goal::CelebrateSurvive>)]
    BehaviorCelebrateSurvive(goal::CelebrateSurvive),
    /// `minecraft:behavior.charge_attack`.
    #[component(name = "minecraft:behavior.charge_attack", released = "1.8.0", parse = goal::parse::<goal::ChargeAttack>)]
    BehaviorChargeAttack(goal::ChargeAttack),
    /// `minecraft:behavior.charge_held_item`.
    #[component(name = "minecraft:behavior.charge_held_item", released = "1.8.0", molang, parse = goal::parse::<goal::ChargeHeldItem>)]
    BehaviorChargeHeldItem(goal::ChargeHeldItem),
    /// `minecraft:behavior.circle_around_anchor`.
    #[component(name = "minecraft:behavior.circle_around_anchor", released = "1.8.0", parse = goal::parse::<goal::CircleAroundAnchor>)]
    BehaviorCircleAroundAnchor(goal::CircleAroundAnchor),
    /// `minecraft:behavior.controlled_by_player`.
    #[component(name = "minecraft:behavior.controlled_by_player", released = "1.8.0", parse = goal::parse::<goal::ControlledByPlayer>)]
    BehaviorControlledByPlayer(goal::ControlledByPlayer),
    /// `minecraft:behavior.croak`.
    #[component(name = "minecraft:behavior.croak", released = "1.8.0", parse = goal::parse::<goal::Croak>)]
    BehaviorCroak(goal::Croak),
    /// `minecraft:behavior.defend_trusted_target`.
    #[component(name = "minecraft:behavior.defend_trusted_target", released = "1.8.0", molang, parse = goal::parse::<goal::DefendTrustedTarget>)]
    BehaviorDefendTrustedTarget(goal::DefendTrustedTarget),
    /// `minecraft:behavior.defend_village_target`.
    #[component(name = "minecraft:behavior.defend_village_target", released = "1.8.0", parse = goal::parse::<goal::DefendVillageTarget>)]
    BehaviorDefendVillageTarget(goal::DefendVillageTarget),
    /// `minecraft:behavior.delayed_attack`.
    #[component(name = "minecraft:behavior.delayed_attack", released = "1.8.0", molang, parse = goal::parse::<goal::DelayedAttack>)]
    BehaviorDelayedAttack(goal::DelayedAttack),
    /// `minecraft:behavior.dig`.
    #[component(name = "minecraft:behavior.dig", released = "1.8.0", molang, parse = goal::parse::<goal::Dig>)]
    BehaviorDig(goal::Dig),
    /// `minecraft:behavior.door_interact`.
    #[component(name = "minecraft:behavior.door_interact", released = "1.8.0", parse = goal::parse::<goal::DoorInteract>)]
    BehaviorDoorInteract(goal::DoorInteract),
    /// `minecraft:behavior.dragonchargeplayer`.
    #[component(name = "minecraft:behavior.dragonchargeplayer", released = "1.8.0", parse = goal::parse::<goal::DragonChargePlayer>)]
    BehaviorDragonChargePlayer(goal::DragonChargePlayer),
    /// `minecraft:behavior.dragondeath`.
    #[component(name = "minecraft:behavior.dragondeath", released = "1.8.0", parse = goal::parse::<goal::DragonDeath>)]
    BehaviorDragonDeath(goal::DragonDeath),
    /// `minecraft:behavior.dragonflaming`.
    #[component(name = "minecraft:behavior.dragonflaming", released = "1.8.0", parse = goal::parse::<goal::DragonFlaming>)]
    BehaviorDragonFlaming(goal::DragonFlaming),
    /// `minecraft:behavior.dragonholdingpattern`.
    #[component(name = "minecraft:behavior.dragonholdingpattern", released = "1.8.0", parse = goal::parse::<goal::DragonHoldingPattern>)]
    BehaviorDragonHoldingPattern(goal::DragonHoldingPattern),
    /// `minecraft:behavior.dragonlanding`.
    #[component(name = "minecraft:behavior.dragonlanding", released = "1.8.0", parse = goal::parse::<goal::DragonLanding>)]
    BehaviorDragonLanding(goal::DragonLanding),
    /// `minecraft:behavior.dragonscanning`.
    #[component(name = "minecraft:behavior.dragonscanning", released = "1.8.0", parse = goal::parse::<goal::DragonScanning>)]
    BehaviorDragonScanning(goal::DragonScanning),
    /// `minecraft:behavior.dragonstrafeplayer`.
    #[component(name = "minecraft:behavior.dragonstrafeplayer", released = "1.8.0", parse = goal::parse::<goal::DragonStrafePlayer>)]
    BehaviorDragonStrafePlayer(goal::DragonStrafePlayer),
    /// `minecraft:behavior.dragontakeoff`.
    #[component(name = "minecraft:behavior.dragontakeoff", released = "1.8.0", parse = goal::parse::<goal::DragonTakeoff>)]
    BehaviorDragonTakeoff(goal::DragonTakeoff),
    /// `minecraft:behavior.drink_milk`.
    #[component(name = "minecraft:behavior.drink_milk", released = "1.8.0", parse = goal::parse::<goal::DrinkMilk>)]
    BehaviorDrinkMilk(goal::DrinkMilk),
    /// `minecraft:behavior.drink_potion`.
    #[component(name = "minecraft:behavior.drink_potion", released = "1.8.0", parse = goal::parse::<goal::DrinkPotion>)]
    BehaviorDrinkPotion(goal::DrinkPotion),
    /// `minecraft:behavior.drop_item_for`.
    #[component(name = "minecraft:behavior.drop_item_for", released = "1.8.0", molang, parse = goal::parse::<goal::DropItemFor>)]
    BehaviorDropItemFor(goal::DropItemFor),
    /// `minecraft:behavior.eat_block`.
    #[component(name = "minecraft:behavior.eat_block", released = "1.8.0", molang, parse = goal::parse::<goal::EatBlock>)]
    BehaviorEatBlock(goal::EatBlock),
    /// `minecraft:behavior.eat_carried_item`.
    #[component(name = "minecraft:behavior.eat_carried_item", released = "1.8.0", parse = goal::parse::<goal::EatCarriedItem>)]
    BehaviorEatCarriedItem(goal::EatCarriedItem),
    /// `minecraft:behavior.eat_mob`.
    #[component(name = "minecraft:behavior.eat_mob", released = "1.8.0", parse = goal::parse::<goal::EatMob>)]
    BehaviorEatMob(goal::EatMob),
    /// `minecraft:behavior.emerge`.
    #[component(name = "minecraft:behavior.emerge", released = "1.8.0", molang, parse = goal::parse::<goal::Emerge>)]
    BehaviorEmerge(goal::Emerge),
    /// `minecraft:behavior.equip_item`.
    #[component(name = "minecraft:behavior.equip_item", released = "1.8.0", parse = goal::parse::<goal::EquipItem>)]
    BehaviorEquipItem(goal::EquipItem),
    /// `minecraft:behavior.explore_outskirts`.
    #[component(name = "minecraft:behavior.explore_outskirts", released = "1.8.0", parse = goal::parse::<goal::ExploreOutskirts>)]
    BehaviorExploreOutskirts(goal::ExploreOutskirts),
    /// `minecraft:behavior.fertilize_farm_block`.
    #[component(name = "minecraft:behavior.fertilize_farm_block", released = "1.8.0", parse = goal::parse::<goal::FertilizeFarmBlock>)]
    BehaviorFertilizeFarmBlock(goal::FertilizeFarmBlock),
    /// `minecraft:behavior.find_cover`.
    #[component(name = "minecraft:behavior.find_cover", released = "1.8.0", parse = goal::parse::<goal::FindCover>)]
    BehaviorFindCover(goal::FindCover),
    /// `minecraft:behavior.find_mount`.
    #[component(name = "minecraft:behavior.find_mount", released = "1.8.0", parse = goal::parse::<goal::FindMount>)]
    BehaviorFindMount(goal::FindMount),
    /// `minecraft:behavior.find_underwater_treasure`.
    #[component(name = "minecraft:behavior.find_underwater_treasure", released = "1.8.0", parse = goal::parse::<goal::FindUnderwaterTreasure>)]
    BehaviorFindUnderwaterTreasure(goal::FindUnderwaterTreasure),
    /// `minecraft:behavior.fire_at_target`.
    #[component(name = "minecraft:behavior.fire_at_target", released = "1.8.0", parse = goal::parse::<goal::FireAtTarget>)]
    BehaviorFireAtTarget(goal::FireAtTarget),
    /// `minecraft:behavior.flee_sun`.
    #[component(name = "minecraft:behavior.flee_sun", released = "1.8.0", parse = goal::parse::<goal::FleeSun>)]
    BehaviorFleeSun(goal::FleeSun),
    /// `minecraft:behavior.float`.
    #[component(name = "minecraft:behavior.float", released = "1.8.0", parse = goal::parse::<goal::Float>)]
    BehaviorFloat(goal::Float),
    /// `minecraft:behavior.float_tempt` (typed at 1.26.60).
    #[component(name = "minecraft:behavior.float_tempt", released = "1.8.0", molang, parse = goal::parse::<goal::FloatTempt>)]
    BehaviorFloatTempt(goal::FloatTempt),
    /// `minecraft:behavior.float_wander`.
    #[component(name = "minecraft:behavior.float_wander", released = "1.8.0", parse = goal::parse::<goal::FloatWander>)]
    BehaviorFloatWander(goal::FloatWander),
    /// `minecraft:behavior.follow_caravan`.
    #[component(name = "minecraft:behavior.follow_caravan", released = "1.8.0", parse = goal::parse::<goal::FollowCaravan>)]
    BehaviorFollowCaravan(goal::FollowCaravan),
    /// `minecraft:behavior.follow_mob`.
    #[component(name = "minecraft:behavior.follow_mob", released = "1.8.0", parse = goal::parse::<goal::FollowMob>)]
    BehaviorFollowMob(goal::FollowMob),
    /// `minecraft:behavior.follow_owner`.
    #[component(name = "minecraft:behavior.follow_owner", released = "1.8.0", parse = goal::parse::<goal::FollowOwner>)]
    BehaviorFollowOwner(goal::FollowOwner),
    /// `minecraft:behavior.follow_parent`.
    #[component(name = "minecraft:behavior.follow_parent", released = "1.8.0", parse = goal::parse::<goal::FollowParent>)]
    BehaviorFollowParent(goal::FollowParent),
    /// `minecraft:behavior.follow_target_leader`.
    #[component(name = "minecraft:behavior.follow_target_leader", released = "1.26.20", supersedes = "minecraft:behavior.follow_target_captain", parse = goal::parse::<goal::FollowTargetLeader>)]
    BehaviorFollowTargetLeader(goal::FollowTargetLeader),
    /// `minecraft:behavior.go_and_give_items_to_noteblock`.
    #[component(name = "minecraft:behavior.go_and_give_items_to_noteblock", released = "1.8.0", molang, parse = goal::parse::<goal::GoAndGiveItemsToNoteblock>)]
    BehaviorGoAndGiveItemsToNoteblock(goal::GoAndGiveItemsToNoteblock),
    /// `minecraft:behavior.go_and_give_items_to_owner`.
    #[component(name = "minecraft:behavior.go_and_give_items_to_owner", released = "1.8.0", molang, parse = goal::parse::<goal::GoAndGiveItemsToOwner>)]
    BehaviorGoAndGiveItemsToOwner(goal::GoAndGiveItemsToOwner),
    /// `minecraft:behavior.go_home`.
    #[component(name = "minecraft:behavior.go_home", released = "1.8.0", molang, parse = goal::parse::<goal::GoHome>)]
    BehaviorGoHome(goal::GoHome),
    /// `minecraft:behavior.guardian_attack`.
    #[component(name = "minecraft:behavior.guardian_attack", released = "1.8.0", parse = goal::parse::<goal::GuardianAttack>)]
    BehaviorGuardianAttack(goal::GuardianAttack),
    /// `minecraft:behavior.harvest_farm_block`.
    #[component(name = "minecraft:behavior.harvest_farm_block", released = "1.8.0", parse = goal::parse::<goal::HarvestFarmBlock>)]
    BehaviorHarvestFarmBlock(goal::HarvestFarmBlock),
    /// `minecraft:behavior.hide`.
    #[component(name = "minecraft:behavior.hide", released = "1.8.0", parse = goal::parse::<goal::Hide>)]
    BehaviorHide(goal::Hide),
    /// `minecraft:behavior.hold_ground`.
    #[component(name = "minecraft:behavior.hold_ground", released = "1.8.0", molang, parse = goal::parse::<goal::HoldGround>)]
    BehaviorHoldGround(goal::HoldGround),
    /// `minecraft:behavior.hover`.
    #[component(name = "minecraft:behavior.hover", released = "1.8.0", parse = goal::parse::<goal::Hover>)]
    BehaviorHover(goal::Hover),
    /// `minecraft:behavior.hurt_by_target`.
    #[component(name = "minecraft:behavior.hurt_by_target", released = "1.8.0", parse = goal::parse::<goal::HurtByTarget>)]
    BehaviorHurtByTarget(goal::HurtByTarget),
    /// `minecraft:behavior.inspect_bookshelf`.
    #[component(name = "minecraft:behavior.inspect_bookshelf", released = "1.8.0", parse = goal::parse::<goal::InspectBookshelf>)]
    BehaviorInspectBookshelf(goal::InspectBookshelf),
    /// `minecraft:behavior.investigate_suspicious_location`.
    #[component(name = "minecraft:behavior.investigate_suspicious_location", released = "1.8.0", parse = goal::parse::<goal::InvestigateSuspiciousLocation>)]
    BehaviorInvestigateSuspiciousLocation(goal::InvestigateSuspiciousLocation),
    /// `minecraft:behavior.jump_around_target`.
    #[component(name = "minecraft:behavior.jump_around_target", released = "1.8.0", parse = goal::parse::<goal::JumpAroundTarget>)]
    BehaviorJumpAroundTarget(goal::JumpAroundTarget),
    /// `minecraft:behavior.jump_to_block`.
    #[component(name = "minecraft:behavior.jump_to_block", released = "1.8.0", molang, parse = goal::parse::<goal::JumpToBlock>)]
    BehaviorJumpToBlock(goal::JumpToBlock),
    /// `minecraft:behavior.knockback_roar`.
    #[component(name = "minecraft:behavior.knockback_roar", released = "1.8.0", molang, parse = goal::parse::<goal::KnockbackRoar>)]
    BehaviorKnockbackRoar(goal::KnockbackRoar),
    /// `minecraft:behavior.lay_down`.
    #[component(name = "minecraft:behavior.lay_down", released = "1.8.0", parse = goal::parse::<goal::LayDown>)]
    BehaviorLayDown(goal::LayDown),
    /// `minecraft:behavior.lay_egg`.
    #[component(name = "minecraft:behavior.lay_egg", released = "1.8.0", molang, parse = goal::parse::<goal::LayEgg>)]
    BehaviorLayEgg(goal::LayEgg),
    /// `minecraft:behavior.leap_at_target`.
    #[component(name = "minecraft:behavior.leap_at_target", released = "1.8.0", parse = goal::parse::<goal::LeapAtTarget>)]
    BehaviorLeapAtTarget(goal::LeapAtTarget),
    /// `minecraft:behavior.look_at_entity`.
    #[component(name = "minecraft:behavior.look_at_entity", released = "1.8.0", parse = goal::parse::<goal::LookAtEntity>)]
    BehaviorLookAtEntity(goal::LookAtEntity),
    /// `minecraft:behavior.look_at_player`.
    #[component(name = "minecraft:behavior.look_at_player", released = "1.8.0", parse = goal::parse::<goal::LookAtPlayer>)]
    BehaviorLookAtPlayer(goal::LookAtPlayer),
    /// `minecraft:behavior.look_at_target`.
    #[component(name = "minecraft:behavior.look_at_target", released = "1.8.0", parse = goal::parse::<goal::LookAtTarget>)]
    BehaviorLookAtTarget(goal::LookAtTarget),
    /// `minecraft:behavior.look_at_trading_player`.
    #[component(name = "minecraft:behavior.look_at_trading_player", released = "1.8.0", parse = goal::parse::<goal::LookAtTradingPlayer>)]
    BehaviorLookAtTradingPlayer(goal::LookAtTradingPlayer),
    /// `minecraft:behavior.make_love`.
    #[component(name = "minecraft:behavior.make_love", released = "1.8.0", parse = goal::parse::<goal::MakeLove>)]
    BehaviorMakeLove(goal::MakeLove),
    /// `minecraft:behavior.melee_attack`.
    #[component(name = "minecraft:behavior.melee_attack", released = "1.8.0", molang, parse = goal::parse::<goal::MeleeAttack>)]
    BehaviorMeleeAttack(goal::MeleeAttack),
    /// `minecraft:behavior.melee_box_attack`.
    #[component(name = "minecraft:behavior.melee_box_attack", released = "1.8.0", molang, parse = goal::parse::<goal::MeleeBoxAttack>)]
    BehaviorMeleeBoxAttack(goal::MeleeBoxAttack),
    /// `minecraft:behavior.mingle`.
    #[component(name = "minecraft:behavior.mingle", released = "1.8.0", parse = goal::parse::<goal::Mingle>)]
    BehaviorMingle(goal::Mingle),
    /// `minecraft:behavior.mount_pathing`.
    #[component(name = "minecraft:behavior.mount_pathing", released = "1.8.0", parse = goal::parse::<goal::MountPathing>)]
    BehaviorMountPathing(goal::MountPathing),
    /// `minecraft:behavior.move_around_target`.
    #[component(name = "minecraft:behavior.move_around_target", released = "1.8.0", parse = goal::parse::<goal::MoveAroundTarget>)]
    BehaviorMoveAroundTarget(goal::MoveAroundTarget),
    /// `minecraft:behavior.move_indoors`.
    #[component(name = "minecraft:behavior.move_indoors", released = "1.8.0", parse = goal::parse::<goal::MoveIndoors>)]
    BehaviorMoveIndoors(goal::MoveIndoors),
    /// `minecraft:behavior.move_outdoors`.
    #[component(name = "minecraft:behavior.move_outdoors", released = "1.8.0", parse = goal::parse::<goal::MoveOutdoors>)]
    BehaviorMoveOutdoors(goal::MoveOutdoors),
    /// `minecraft:behavior.move_through_village`.
    #[component(name = "minecraft:behavior.move_through_village", released = "1.8.0", parse = goal::parse::<goal::MoveThroughVillage>)]
    BehaviorMoveThroughVillage(goal::MoveThroughVillage),
    /// `minecraft:behavior.move_to_block`.
    #[component(name = "minecraft:behavior.move_to_block", released = "1.8.0", molang, parse = goal::parse::<goal::MoveToBlock>)]
    BehaviorMoveToBlock(goal::MoveToBlock),
    /// `minecraft:behavior.move_to_land`.
    #[component(name = "minecraft:behavior.move_to_land", released = "1.8.0", parse = goal::parse::<goal::MoveToLand>)]
    BehaviorMoveToLand(goal::MoveToLand),
    /// `minecraft:behavior.move_to_lava`.
    #[component(name = "minecraft:behavior.move_to_lava", released = "1.8.0", parse = goal::parse::<goal::MoveToLava>)]
    BehaviorMoveToLava(goal::MoveToLava),
    /// `minecraft:behavior.move_to_liquid`.
    #[component(name = "minecraft:behavior.move_to_liquid", released = "1.8.0", parse = goal::parse::<goal::MoveToLiquid>)]
    BehaviorMoveToLiquid(goal::MoveToLiquid),
    /// `minecraft:behavior.move_to_poi`.
    #[component(name = "minecraft:behavior.move_to_poi", released = "1.8.0", parse = goal::parse::<goal::MoveToPoi>)]
    BehaviorMoveToPoi(goal::MoveToPoi),
    /// `minecraft:behavior.move_to_random_block`.
    #[component(name = "minecraft:behavior.move_to_random_block", released = "1.8.0", parse = goal::parse::<goal::MoveToRandomBlock>)]
    BehaviorMoveToRandomBlock(goal::MoveToRandomBlock),
    /// `minecraft:behavior.move_to_village`.
    #[component(name = "minecraft:behavior.move_to_village", released = "1.8.0", parse = goal::parse::<goal::MoveToVillage>)]
    BehaviorMoveToVillage(goal::MoveToVillage),
    /// `minecraft:behavior.move_to_water`.
    #[component(name = "minecraft:behavior.move_to_water", released = "1.8.0", parse = goal::parse::<goal::MoveToWater>)]
    BehaviorMoveToWater(goal::MoveToWater),
    /// `minecraft:behavior.move_towards_dwelling_restriction`.
    #[component(name = "minecraft:behavior.move_towards_dwelling_restriction", released = "1.16.100", parse = goal::parse::<goal::MoveTowardsDwellingRestriction>)]
    BehaviorMoveTowardsDwellingRestriction(goal::MoveTowardsDwellingRestriction),
    /// `minecraft:behavior.move_towards_home_restriction`.
    #[component(name = "minecraft:behavior.move_towards_home_restriction", released = "1.16.100", parse = goal::parse::<goal::MoveTowardsHomeRestriction>)]
    BehaviorMoveTowardsHomeRestriction(goal::MoveTowardsHomeRestriction),
    /// `minecraft:behavior.move_towards_target`.
    #[component(name = "minecraft:behavior.move_towards_target", released = "1.8.0", parse = goal::parse::<goal::MoveTowardsTarget>)]
    BehaviorMoveTowardsTarget(goal::MoveTowardsTarget),
    /// `minecraft:behavior.nap`.
    #[component(name = "minecraft:behavior.nap", released = "1.8.0", parse = goal::parse::<goal::Nap>)]
    BehaviorNap(goal::Nap),
    /// `minecraft:behavior.nearest_attackable_target`.
    #[component(name = "minecraft:behavior.nearest_attackable_target", released = "1.8.0", parse = goal::parse::<goal::NearestAttackableTarget>)]
    BehaviorNearestAttackableTarget(goal::NearestAttackableTarget),
    /// `minecraft:behavior.nearest_prioritized_attackable_target`.
    #[component(name = "minecraft:behavior.nearest_prioritized_attackable_target", released = "1.8.0", parse = goal::parse::<goal::NearestPrioritizedAttackableTarget>)]
    BehaviorNearestPrioritizedAttackableTarget(goal::NearestPrioritizedAttackableTarget),
    /// `minecraft:behavior.ocelot_sit_on_block`.
    #[component(name = "minecraft:behavior.ocelot_sit_on_block", released = "1.8.0", parse = goal::parse::<goal::OcelotSitOnBlock>)]
    BehaviorOcelotSitOnBlock(goal::OcelotSitOnBlock),
    /// `minecraft:behavior.ocelotattack`.
    #[component(name = "minecraft:behavior.ocelotattack", released = "1.8.0", parse = goal::parse::<goal::Ocelotattack>)]
    BehaviorOcelotattack(goal::Ocelotattack),
    /// `minecraft:behavior.offer_flower`.
    #[component(name = "minecraft:behavior.offer_flower", released = "1.8.0", parse = goal::parse::<goal::OfferFlower>)]
    BehaviorOfferFlower(goal::OfferFlower),
    /// `minecraft:behavior.open_door`.
    #[component(name = "minecraft:behavior.open_door", released = "1.8.0", parse = goal::parse::<goal::OpenDoor>)]
    BehaviorOpenDoor(goal::OpenDoor),
    /// `minecraft:behavior.owner_hurt_by_target`.
    #[component(name = "minecraft:behavior.owner_hurt_by_target", released = "1.8.0", parse = goal::parse::<goal::OwnerHurtByTarget>)]
    BehaviorOwnerHurtByTarget(goal::OwnerHurtByTarget),
    /// `minecraft:behavior.owner_hurt_target`.
    #[component(name = "minecraft:behavior.owner_hurt_target", released = "1.8.0", parse = goal::parse::<goal::OwnerHurtTarget>)]
    BehaviorOwnerHurtTarget(goal::OwnerHurtTarget),
    /// `minecraft:behavior.panic` (typed at 1.26.60).
    #[component(name = "minecraft:behavior.panic", released = "1.8.0", parse = goal::parse::<goal::Panic>)]
    BehaviorPanic(goal::Panic),
    /// `minecraft:behavior.pet_sleep_with_owner`.
    #[component(name = "minecraft:behavior.pet_sleep_with_owner", released = "1.8.0", parse = goal::parse::<goal::PetSleepWithOwner>)]
    BehaviorPetSleepWithOwner(goal::PetSleepWithOwner),
    /// `minecraft:behavior.pickup_items`.
    #[component(name = "minecraft:behavior.pickup_items", released = "1.8.0", molang, parse = goal::parse::<goal::PickupItems>)]
    BehaviorPickupItems(goal::PickupItems),
    /// `minecraft:behavior.place_block`.
    #[component(name = "minecraft:behavior.place_block", released = "1.21.100", molang, parse = goal::parse::<goal::PlaceBlock>)]
    BehaviorPlaceBlock(goal::PlaceBlock),
    /// `minecraft:behavior.play`.
    #[component(name = "minecraft:behavior.play", released = "1.8.0", parse = goal::parse::<goal::Play>)]
    BehaviorPlay(goal::Play),
    /// `minecraft:behavior.play_dead`.
    #[component(name = "minecraft:behavior.play_dead", released = "1.8.0", parse = goal::parse::<goal::PlayDead>)]
    BehaviorPlayDead(goal::PlayDead),
    /// `minecraft:behavior.player_ride_tamed`.
    #[component(name = "minecraft:behavior.player_ride_tamed", released = "1.8.0", parse = goal::parse::<goal::PlayerRideTamed>)]
    BehaviorPlayerRideTamed(goal::PlayerRideTamed),
    /// `minecraft:behavior.raid_garden`.
    #[component(name = "minecraft:behavior.raid_garden", released = "1.8.0", parse = goal::parse::<goal::RaidGarden>)]
    BehaviorRaidGarden(goal::RaidGarden),
    /// `minecraft:behavior.ram_attack`.
    #[component(name = "minecraft:behavior.ram_attack", released = "1.8.0", molang, parse = goal::parse::<goal::RamAttack>)]
    BehaviorRamAttack(goal::RamAttack),
    /// `minecraft:behavior.random_breach`.
    #[component(name = "minecraft:behavior.random_breach", released = "1.8.0", parse = goal::parse::<goal::RandomBreach>)]
    BehaviorRandomBreach(goal::RandomBreach),
    /// `minecraft:behavior.random_fly`.
    #[component(name = "minecraft:behavior.random_fly", released = "1.8.0", parse = goal::parse::<goal::RandomFly>)]
    BehaviorRandomFly(goal::RandomFly),
    /// `minecraft:behavior.random_hover`.
    #[component(name = "minecraft:behavior.random_hover", released = "1.8.0", parse = goal::parse::<goal::RandomHover>)]
    BehaviorRandomHover(goal::RandomHover),
    /// `minecraft:behavior.random_look_around`.
    #[component(name = "minecraft:behavior.random_look_around", released = "1.8.0", parse = goal::parse::<goal::RandomLookAround>)]
    BehaviorRandomLookAround(goal::RandomLookAround),
    /// `minecraft:behavior.random_look_around_and_sit`.
    #[component(name = "minecraft:behavior.random_look_around_and_sit", released = "1.8.0", parse = goal::parse::<goal::RandomLookAroundAndSit>)]
    BehaviorRandomLookAroundAndSit(goal::RandomLookAroundAndSit),
    /// `minecraft:behavior.random_search_and_dig`.
    #[component(name = "minecraft:behavior.random_search_and_dig", released = "1.8.0", molang, parse = goal::parse::<goal::RandomSearchAndDig>)]
    BehaviorRandomSearchAndDig(goal::RandomSearchAndDig),
    /// `minecraft:behavior.random_sitting`.
    #[component(name = "minecraft:behavior.random_sitting", released = "1.8.0", parse = goal::parse::<goal::RandomSitting>)]
    BehaviorRandomSitting(goal::RandomSitting),
    /// `minecraft:behavior.random_stroll`.
    #[component(name = "minecraft:behavior.random_stroll", released = "1.8.0", parse = goal::parse::<goal::RandomStroll>)]
    BehaviorRandomStroll(goal::RandomStroll),
    /// `minecraft:behavior.random_swim`.
    #[component(name = "minecraft:behavior.random_swim", released = "1.8.0", parse = goal::parse::<goal::RandomSwim>)]
    BehaviorRandomSwim(goal::RandomSwim),
    /// `minecraft:behavior.ranged_attack`.
    #[component(name = "minecraft:behavior.ranged_attack", released = "1.8.0", parse = goal::parse::<goal::RangedAttack>)]
    BehaviorRangedAttack(goal::RangedAttack),
    /// `minecraft:behavior.receive_love`.
    #[component(name = "minecraft:behavior.receive_love", released = "1.8.0", parse = goal::parse::<goal::ReceiveLove>)]
    BehaviorReceiveLove(goal::ReceiveLove),
    /// `minecraft:behavior.restrict_open_door`.
    #[component(name = "minecraft:behavior.restrict_open_door", released = "1.8.0", parse = goal::parse::<goal::RestrictOpenDoor>)]
    BehaviorRestrictOpenDoor(goal::RestrictOpenDoor),
    /// `minecraft:behavior.restrict_sun`.
    #[component(name = "minecraft:behavior.restrict_sun", released = "1.8.0", parse = goal::parse::<goal::RestrictSun>)]
    BehaviorRestrictSun(goal::RestrictSun),
    /// `minecraft:behavior.rise_to_liquid_level`.
    #[component(name = "minecraft:behavior.rise_to_liquid_level", released = "1.8.0", parse = goal::parse::<goal::RiseToLiquidLevel>)]
    BehaviorRiseToLiquidLevel(goal::RiseToLiquidLevel),
    /// `minecraft:behavior.roar`.
    #[component(name = "minecraft:behavior.roar", released = "1.8.0", parse = goal::parse::<goal::Roar>)]
    BehaviorRoar(goal::Roar),
    /// `minecraft:behavior.roll`.
    #[component(name = "minecraft:behavior.roll", released = "1.8.0", parse = goal::parse::<goal::Roll>)]
    BehaviorRoll(goal::Roll),
    /// `minecraft:behavior.run_around_like_crazy`.
    #[component(name = "minecraft:behavior.run_around_like_crazy", released = "1.8.0", parse = goal::parse::<goal::RunAroundLikeCrazy>)]
    BehaviorRunAroundLikeCrazy(goal::RunAroundLikeCrazy),
    /// `minecraft:behavior.scared`.
    #[component(name = "minecraft:behavior.scared", released = "1.8.0", parse = goal::parse::<goal::Scared>)]
    BehaviorScared(goal::Scared),
    /// `minecraft:behavior.send_event` (typed at 1.26.60).
    #[component(name = "minecraft:behavior.send_event", released = "1.8.0", parse = goal::parse::<goal::SendEvent>)]
    BehaviorSendEvent(goal::SendEvent),
    /// `minecraft:behavior.share_items`.
    #[component(name = "minecraft:behavior.share_items", released = "1.8.0", parse = goal::parse::<goal::ShareItems>)]
    BehaviorShareItems(goal::ShareItems),
    /// `minecraft:behavior.silverfish_merge_with_stone`.
    #[component(name = "minecraft:behavior.silverfish_merge_with_stone", released = "1.8.0", parse = goal::parse::<goal::SilverfishMergeWithStone>)]
    BehaviorSilverfishMergeWithStone(goal::SilverfishMergeWithStone),
    /// `minecraft:behavior.silverfish_wake_up_friends`.
    #[component(name = "minecraft:behavior.silverfish_wake_up_friends", released = "1.8.0", parse = goal::parse::<goal::SilverfishWakeUpFriends>)]
    BehaviorSilverfishWakeUpFriends(goal::SilverfishWakeUpFriends),
    /// `minecraft:behavior.skeleton_horse_trap`.
    #[component(name = "minecraft:behavior.skeleton_horse_trap", released = "1.8.0", parse = goal::parse::<goal::SkeletonHorseTrap>)]
    BehaviorSkeletonHorseTrap(goal::SkeletonHorseTrap),
    /// `minecraft:behavior.sleep`.
    #[component(name = "minecraft:behavior.sleep", released = "1.8.0", parse = goal::parse::<goal::Sleep>)]
    BehaviorSleep(goal::Sleep),
    /// `minecraft:behavior.slime_attack`.
    #[component(name = "minecraft:behavior.slime_attack", released = "1.8.0", parse = goal::parse::<goal::SlimeAttack>)]
    BehaviorSlimeAttack(goal::SlimeAttack),
    /// `minecraft:behavior.slime_float`.
    #[component(name = "minecraft:behavior.slime_float", released = "1.8.0", parse = goal::parse::<goal::SlimeFloat>)]
    BehaviorSlimeFloat(goal::SlimeFloat),
    /// `minecraft:behavior.slime_keep_on_jumping`.
    #[component(name = "minecraft:behavior.slime_keep_on_jumping", released = "1.8.0", parse = goal::parse::<goal::SlimeKeepOnJumping>)]
    BehaviorSlimeKeepOnJumping(goal::SlimeKeepOnJumping),
    /// `minecraft:behavior.slime_random_direction`.
    #[component(name = "minecraft:behavior.slime_random_direction", released = "1.8.0", parse = goal::parse::<goal::SlimeRandomDirection>)]
    BehaviorSlimeRandomDirection(goal::SlimeRandomDirection),
    /// `minecraft:behavior.snacking`.
    #[component(name = "minecraft:behavior.snacking", released = "1.8.0", molang, parse = goal::parse::<goal::Snacking>)]
    BehaviorSnacking(goal::Snacking),
    /// `minecraft:behavior.sneeze`.
    #[component(name = "minecraft:behavior.sneeze", released = "1.8.0", parse = goal::parse::<goal::Sneeze>)]
    BehaviorSneeze(goal::Sneeze),
    /// `minecraft:behavior.sniff`.
    #[component(name = "minecraft:behavior.sniff", released = "1.8.0", parse = goal::parse::<goal::Sniff>)]
    BehaviorSniff(goal::Sniff),
    /// `minecraft:behavior.sonic_boom`.
    #[component(name = "minecraft:behavior.sonic_boom", released = "1.8.0", parse = goal::parse::<goal::SonicBoom>)]
    BehaviorSonicBoom(goal::SonicBoom),
    /// `minecraft:behavior.squid_dive`.
    #[component(name = "minecraft:behavior.squid_dive", released = "1.8.0", parse = goal::parse::<goal::SquidDive>)]
    BehaviorSquidDive(goal::SquidDive),
    /// `minecraft:behavior.squid_flee`.
    #[component(name = "minecraft:behavior.squid_flee", released = "1.8.0", parse = goal::parse::<goal::SquidFlee>)]
    BehaviorSquidFlee(goal::SquidFlee),
    /// `minecraft:behavior.squid_idle`.
    #[component(name = "minecraft:behavior.squid_idle", released = "1.8.0", parse = goal::parse::<goal::SquidIdle>)]
    BehaviorSquidIdle(goal::SquidIdle),
    /// `minecraft:behavior.squid_move_away_from_ground`.
    #[component(name = "minecraft:behavior.squid_move_away_from_ground", released = "1.8.0", parse = goal::parse::<goal::SquidMoveAwayFromGround>)]
    BehaviorSquidMoveAwayFromGround(goal::SquidMoveAwayFromGround),
    /// `minecraft:behavior.squid_out_of_water`.
    #[component(name = "minecraft:behavior.squid_out_of_water", released = "1.8.0", parse = goal::parse::<goal::SquidOutOfWater>)]
    BehaviorSquidOutOfWater(goal::SquidOutOfWater),
    /// `minecraft:behavior.stalk_and_pounce_on_target`.
    #[component(name = "minecraft:behavior.stalk_and_pounce_on_target", released = "1.8.0", parse = goal::parse::<goal::StalkAndPounceOnTarget>)]
    BehaviorStalkAndPounceOnTarget(goal::StalkAndPounceOnTarget),
    /// `minecraft:behavior.stay_near_noteblock`.
    #[component(name = "minecraft:behavior.stay_near_noteblock", released = "1.8.0", parse = goal::parse::<goal::StayNearNoteblock>)]
    BehaviorStayNearNoteblock(goal::StayNearNoteblock),
    /// `minecraft:behavior.stay_while_sitting`.
    #[component(name = "minecraft:behavior.stay_while_sitting", released = "1.8.0", parse = goal::parse::<goal::StayWhileSitting>)]
    BehaviorStayWhileSitting(goal::StayWhileSitting),
    /// `minecraft:behavior.stomp_attack`.
    #[component(name = "minecraft:behavior.stomp_attack", released = "1.8.0", molang, parse = goal::parse::<goal::StompAttack>)]
    BehaviorStompAttack(goal::StompAttack),
    /// `minecraft:behavior.stomp_turtle_egg`.
    #[component(name = "minecraft:behavior.stomp_turtle_egg", released = "1.8.0", parse = goal::parse::<goal::StompTurtleEgg>)]
    BehaviorStompTurtleEgg(goal::StompTurtleEgg),
    /// `minecraft:behavior.stroll_towards_village`.
    #[component(name = "minecraft:behavior.stroll_towards_village", released = "1.8.0", parse = goal::parse::<goal::StrollTowardsVillage>)]
    BehaviorStrollTowardsVillage(goal::StrollTowardsVillage),
    /// `minecraft:behavior.summon_entity`.
    #[component(name = "minecraft:behavior.summon_entity", released = "1.8.0", parse = goal::parse::<goal::SummonEntity>)]
    BehaviorSummonEntity(goal::SummonEntity),
    /// `minecraft:behavior.swell`.
    #[component(name = "minecraft:behavior.swell", released = "1.8.0", parse = goal::parse::<goal::Swell>)]
    BehaviorSwell(goal::Swell),
    /// `minecraft:behavior.swim_idle`.
    #[component(name = "minecraft:behavior.swim_idle", released = "1.8.0", parse = goal::parse::<goal::SwimIdle>)]
    BehaviorSwimIdle(goal::SwimIdle),
    /// `minecraft:behavior.swim_up_for_breath`.
    #[component(name = "minecraft:behavior.swim_up_for_breath", released = "1.8.0", parse = goal::parse::<goal::SwimUpForBreath>)]
    BehaviorSwimUpForBreath(goal::SwimUpForBreath),
    /// `minecraft:behavior.swim_wander`.
    #[component(name = "minecraft:behavior.swim_wander", released = "1.8.0", parse = goal::parse::<goal::SwimWander>)]
    BehaviorSwimWander(goal::SwimWander),
    /// `minecraft:behavior.swim_with_entity`.
    #[component(name = "minecraft:behavior.swim_with_entity", released = "1.8.0", parse = goal::parse::<goal::SwimWithEntity>)]
    BehaviorSwimWithEntity(goal::SwimWithEntity),
    /// `minecraft:behavior.swoop_attack`.
    #[component(name = "minecraft:behavior.swoop_attack", released = "1.8.0", parse = goal::parse::<goal::SwoopAttack>)]
    BehaviorSwoopAttack(goal::SwoopAttack),
    /// `minecraft:behavior.take_block`.
    #[component(name = "minecraft:behavior.take_block", released = "1.21.100", molang, parse = goal::parse::<goal::TakeBlock>)]
    BehaviorTakeBlock(goal::TakeBlock),
    /// `minecraft:behavior.take_flower`.
    #[component(name = "minecraft:behavior.take_flower", released = "1.8.0", molang, parse = goal::parse::<goal::TakeFlower>)]
    BehaviorTakeFlower(goal::TakeFlower),
    /// `minecraft:behavior.target_when_pushed`.
    #[component(name = "minecraft:behavior.target_when_pushed", released = "1.8.0", parse = goal::parse::<goal::TargetWhenPushed>)]
    BehaviorTargetWhenPushed(goal::TargetWhenPushed),
    /// `minecraft:behavior.teleport_to_owner`.
    #[component(name = "minecraft:behavior.teleport_to_owner", released = "1.8.0", parse = goal::parse::<goal::TeleportToOwner>)]
    BehaviorTeleportToOwner(goal::TeleportToOwner),
    /// `minecraft:behavior.tempt` (typed at 1.26.60).
    #[component(name = "minecraft:behavior.tempt", released = "1.8.0", molang, parse = goal::parse::<goal::Tempt>)]
    BehaviorTempt(goal::Tempt),
    /// `minecraft:behavior.timer_flag_1`.
    #[component(name = "minecraft:behavior.timer_flag_1", released = "1.8.0", molang, parse = goal::parse::<goal::TimerFlag1>)]
    BehaviorTimerFlag1(goal::TimerFlag1),
    /// `minecraft:behavior.timer_flag_2`.
    #[component(name = "minecraft:behavior.timer_flag_2", released = "1.8.0", molang, parse = goal::parse::<goal::TimerFlag2>)]
    BehaviorTimerFlag2(goal::TimerFlag2),
    /// `minecraft:behavior.timer_flag_3`.
    #[component(name = "minecraft:behavior.timer_flag_3", released = "1.8.0", molang, parse = goal::parse::<goal::TimerFlag3>)]
    BehaviorTimerFlag3(goal::TimerFlag3),
    /// `minecraft:behavior.trade_interest`.
    #[component(name = "minecraft:behavior.trade_interest", released = "1.8.0", parse = goal::parse::<goal::TradeInterest>)]
    BehaviorTradeInterest(goal::TradeInterest),
    /// `minecraft:behavior.trade_with_player`.
    #[component(name = "minecraft:behavior.trade_with_player", released = "1.8.0", parse = goal::parse::<goal::TradeWithPlayer>)]
    BehaviorTradeWithPlayer(goal::TradeWithPlayer),
    /// `minecraft:behavior.transport_items`.
    #[component(name = "minecraft:behavior.transport_items", released = "1.21.100", molang, parse = goal::parse::<goal::TransportItems>)]
    BehaviorTransportItems(goal::TransportItems),
    /// `minecraft:behavior.use_kinetic_weapon`.
    #[component(name = "minecraft:behavior.use_kinetic_weapon", released = "1.21.120", molang, parse = goal::parse::<goal::UseKineticWeapon>)]
    BehaviorUseKineticWeapon(goal::UseKineticWeapon),
    /// `minecraft:behavior.wither_random_attack_pos_goal`.
    #[component(name = "minecraft:behavior.wither_random_attack_pos_goal", released = "1.8.0", parse = goal::parse::<goal::WitherRandomAttackPosGoal>)]
    BehaviorWitherRandomAttackPosGoal(goal::WitherRandomAttackPosGoal),
    /// `minecraft:behavior.wither_target_highest_damage`.
    #[component(name = "minecraft:behavior.wither_target_highest_damage", released = "1.8.0", parse = goal::parse::<goal::WitherTargetHighestDamage>)]
    BehaviorWitherTargetHighestDamage(goal::WitherTargetHighestDamage),
    /// `minecraft:behavior.work`.
    #[component(name = "minecraft:behavior.work", released = "1.8.0", molang, parse = goal::parse::<goal::Work>)]
    BehaviorWork(goal::Work),
    /// `minecraft:behavior.work_composter`.
    #[component(name = "minecraft:behavior.work_composter", released = "1.8.0", molang, parse = goal::parse::<goal::WorkComposter>)]
    BehaviorWorkComposter(goal::WorkComposter),
    /// `minecraft:block_climber`.
    #[component(name = "minecraft:block_climber", released = "1.17.10", supersedes = "minecraft:scaffolding_climber", parse = parse_typed::<BlockClimber>)]
    BlockClimber(BlockClimber),
    /// `minecraft:block_movement_slowdown_immunity`.
    #[component(name = "minecraft:block_movement_slowdown_immunity", released = "1.26.50", molang, parse = parse_typed::<BlockMovementSlowdownImmunity>)]
    BlockMovementSlowdownImmunity(BlockMovementSlowdownImmunity),
    /// `minecraft:body_rotation_always_follows_head`.
    #[component(name = "minecraft:body_rotation_always_follows_head", released = "1.8.0", parse = parse_typed::<BodyRotationAlwaysFollowsHead>)]
    BodyRotationAlwaysFollowsHead(BodyRotationAlwaysFollowsHead),
    /// `minecraft:body_rotation_blocked`.
    #[component(name = "minecraft:body_rotation_blocked", released = "1.8.0", parse = parse_typed::<BodyRotationBlocked>)]
    BodyRotationBlocked(BodyRotationBlocked),
    /// `minecraft:boss`.
    #[component(name = "minecraft:boss", released = "1.8.0", parse = parse_typed::<Boss>)]
    Boss(Boss),
    /// `minecraft:bucketable`.
    #[component(name = "minecraft:bucketable", released = "1.8.0", parse = parse_typed::<Bucketable>)]
    Bucketable(Bucketable),
    /// `minecraft:burns_in_daylight`.
    #[component(name = "minecraft:burns_in_daylight", released = "1.8.0", parse = parse_typed::<BurnsInDaylight>)]
    BurnsInDaylight(BurnsInDaylight),
    /// `minecraft:can_climb`.
    #[component(name = "minecraft:can_climb", released = "1.8.0", parse = parse_typed::<CanClimb>)]
    CanClimb(CanClimb),
    /// `minecraft:can_fly`.
    #[component(name = "minecraft:can_fly", released = "1.8.0", parse = parse_typed::<CanFly>)]
    CanFly(CanFly),
    /// `minecraft:can_join_raid`.
    #[component(name = "minecraft:can_join_raid", released = "1.8.0", parse = parse_typed::<CanJoinRaid>)]
    CanJoinRaid(CanJoinRaid),
    /// `minecraft:can_power_jump`.
    #[component(name = "minecraft:can_power_jump", released = "1.8.0", parse = parse_typed::<CanPowerJump>)]
    CanPowerJump(CanPowerJump),
    /// `minecraft:can_stand_on_powder_snow`.
    #[component(name = "minecraft:can_stand_on_powder_snow", released = "1.26.50", parse = parse_typed::<CanStandOnPowderSnow>)]
    CanStandOnPowderSnow(CanStandOnPowderSnow),
    /// `minecraft:cannot_be_attacked`.
    #[component(name = "minecraft:cannot_be_attacked", released = "1.8.0", parse = parse_typed::<CannotBeAttacked>)]
    CannotBeAttacked(CannotBeAttacked),
    /// `minecraft:collision_box`.
    #[component(name = "minecraft:collision_box", released = "1.8.0", parse = parse_typed::<CollisionBox>)]
    CollisionBox(CollisionBox),
    /// `minecraft:damage_over_time`.
    #[component(name = "minecraft:damage_over_time", released = "1.8.0", parse = parse_typed::<DamageOverTime>)]
    DamageOverTime(DamageOverTime),
    /// `minecraft:dimension_bound`.
    #[component(name = "minecraft:dimension_bound", released = "1.21.40", parse = parse_typed::<DimensionBound>)]
    DimensionBound(DimensionBound),
    /// `minecraft:exhaustion_values`.
    #[component(name = "minecraft:exhaustion_values", released = "1.18.30", parse = parse_typed::<ExhaustionValues>)]
    ExhaustionValues(ExhaustionValues),
    /// `minecraft:experience_reward`.
    #[component(name = "minecraft:experience_reward", released = "1.8.0", molang, parse = parse_typed::<ExperienceReward>)]
    ExperienceReward(ExperienceReward),
    /// `minecraft:fire_immune`.
    #[component(name = "minecraft:fire_immune", released = "1.8.0", parse = parse_typed::<FireImmune>)]
    FireImmune(FireImmune),
    /// `minecraft:floats_in_liquid`.
    #[component(name = "minecraft:floats_in_liquid", released = "1.8.0", parse = parse_typed::<FloatsInLiquid>)]
    FloatsInLiquid(FloatsInLiquid),
    /// `minecraft:flocking`.
    #[component(name = "minecraft:flocking", released = "1.8.0", parse = parse_typed::<Flocking>)]
    Flocking(Flocking),
    /// `minecraft:freezing_immune`.
    #[component(name = "minecraft:freezing_immune", released = "1.26.50", parse = parse_typed::<FreezingImmune>)]
    FreezingImmune(FreezingImmune),
    /// `minecraft:freezing_vulnerable`.
    #[component(name = "minecraft:freezing_vulnerable", released = "1.26.50", parse = parse_typed::<FreezingVulnerable>)]
    FreezingVulnerable(FreezingVulnerable),
    /// `minecraft:game_event_movement_tracking`.
    #[component(name = "minecraft:game_event_movement_tracking", released = "1.8.0", parse = parse_typed::<GameEventMovementTracking>)]
    GameEventMovementTracking(GameEventMovementTracking),
    /// `minecraft:grows_crop`.
    #[component(name = "minecraft:grows_crop", released = "1.8.0", parse = parse_typed::<GrowsCrop>)]
    GrowsCrop(GrowsCrop),
    /// `minecraft:hide`.
    #[component(name = "minecraft:hide", released = "1.8.0", parse = parse_typed::<Hide>)]
    Hide(Hide),
    /// `minecraft:home`.
    #[component(name = "minecraft:home", released = "1.8.0", parse = parse_typed::<Home>)]
    Home(Home),
    /// `minecraft:ignore_cannot_be_attacked`.
    #[component(name = "minecraft:ignore_cannot_be_attacked", released = "1.8.0", parse = parse_typed::<IgnoreCannotBeAttacked>)]
    IgnoreCannotBeAttacked(IgnoreCannotBeAttacked),
    /// `minecraft:input_ground_controlled`.
    #[component(name = "minecraft:input_ground_controlled", released = "1.8.0", parse = parse_typed::<InputGroundControlled>)]
    InputGroundControlled(InputGroundControlled),
    /// `minecraft:insomnia`.
    #[component(name = "minecraft:insomnia", released = "1.8.0", parse = parse_typed::<Insomnia>)]
    Insomnia(Insomnia),
    /// `minecraft:instant_despawn`.
    #[component(name = "minecraft:instant_despawn", released = "1.8.0", parse = parse_typed::<InstantDespawn>)]
    InstantDespawn(InstantDespawn),
    /// `minecraft:interact`.
    #[component(name = "minecraft:interact", released = "1.8.0", parse = parse_typed::<Interact>)]
    Interact(Interact),
    /// `minecraft:is_baby`.
    #[component(name = "minecraft:is_baby", released = "1.8.0", parse = parse_typed::<IsBaby>)]
    IsBaby(IsBaby),
    /// `minecraft:is_charged`.
    #[component(name = "minecraft:is_charged", released = "1.8.0", parse = parse_typed::<IsCharged>)]
    IsCharged(IsCharged),
    /// `minecraft:is_chested`.
    #[component(name = "minecraft:is_chested", released = "1.8.0", parse = parse_typed::<IsChested>)]
    IsChested(IsChested),
    /// `minecraft:is_hidden_when_invisible`.
    #[component(name = "minecraft:is_hidden_when_invisible", released = "1.8.0", parse = parse_typed::<IsHiddenWhenInvisible>)]
    IsHiddenWhenInvisible(IsHiddenWhenInvisible),
    /// `minecraft:is_ignited`.
    #[component(name = "minecraft:is_ignited", released = "1.8.0", parse = parse_typed::<IsIgnited>)]
    IsIgnited(IsIgnited),
    /// `minecraft:is_illager_captain`.
    #[component(name = "minecraft:is_illager_captain", released = "1.8.0", parse = parse_typed::<IsIllagerCaptain>)]
    IsIllagerCaptain(IsIllagerCaptain),
    /// `minecraft:is_pregnant`.
    #[component(name = "minecraft:is_pregnant", released = "1.8.0", parse = parse_typed::<IsPregnant>)]
    IsPregnant(IsPregnant),
    /// `minecraft:is_saddled`.
    #[component(name = "minecraft:is_saddled", released = "1.8.0", parse = parse_typed::<IsSaddled>)]
    IsSaddled(IsSaddled),
    /// `minecraft:is_shaking`.
    #[component(name = "minecraft:is_shaking", released = "1.8.0", parse = parse_typed::<IsShaking>)]
    IsShaking(IsShaking),
    /// `minecraft:is_sheared`.
    #[component(name = "minecraft:is_sheared", released = "1.8.0", parse = parse_typed::<IsSheared>)]
    IsSheared(IsSheared),
    /// `minecraft:is_stackable`.
    #[component(name = "minecraft:is_stackable", released = "1.8.0", parse = parse_typed::<IsStackable>)]
    IsStackable(IsStackable),
    /// `minecraft:is_stunned`.
    #[component(name = "minecraft:is_stunned", released = "1.8.0", parse = parse_typed::<IsStunned>)]
    IsStunned(IsStunned),
    /// `minecraft:is_tamed`.
    #[component(name = "minecraft:is_tamed", released = "1.8.0", parse = parse_typed::<IsTamed>)]
    IsTamed(IsTamed),
    /// `minecraft:item_hopper`.
    #[component(name = "minecraft:item_hopper", released = "1.8.0", parse = parse_typed::<ItemHopper>)]
    ItemHopper(ItemHopper),
    /// `minecraft:jump.dynamic`.
    #[component(name = "minecraft:jump.dynamic", released = "1.8.0", parse = parse_typed::<JumpDynamic>)]
    JumpDynamic(JumpDynamic),
    /// `minecraft:jump.static`.
    #[component(name = "minecraft:jump.static", released = "1.8.0", parse = parse_typed::<JumpStatic>)]
    JumpStatic(JumpStatic),
    /// `minecraft:managed_wandering_trader`.
    #[component(name = "minecraft:managed_wandering_trader", released = "1.8.0", parse = parse_typed::<ManagedWanderingTrader>)]
    ManagedWanderingTrader(ManagedWanderingTrader),
    /// `minecraft:memory_behavior.follow_target`.
    #[component(name = "minecraft:memory_behavior.follow_target", released = "1.8.0", parse = goal::parse::<goal::MemoryFollowTarget>)]
    MemoryBehaviorFollowTarget(goal::MemoryFollowTarget),
    /// `minecraft:memory_behavior.jump_to_block` (a `.<name>` family).
    #[component(name = "minecraft:memory_behavior.jump_to_block", released = "1.26.50", molang, parse = goal::parse::<goal::MemoryJumpToBlock>)]
    MemoryBehaviorJumpToBlock(goal::MemoryJumpToBlock),
    /// `minecraft:memory_behavior.look_at_entity`.
    #[component(name = "minecraft:memory_behavior.look_at_entity", released = "1.8.0", parse = goal::parse::<goal::MemoryLookAtEntity>)]
    MemoryBehaviorLookAtEntity(goal::MemoryLookAtEntity),
    /// `minecraft:memory_behavior.move_to_position`.
    #[component(name = "minecraft:memory_behavior.move_to_position", released = "1.26.20", molang, parse = goal::parse::<goal::MemoryMoveToPosition>)]
    MemoryBehaviorMoveToPosition(goal::MemoryMoveToPosition),
    /// `minecraft:memory_behavior.prepare_ram_attack` (a `.<name>` family).
    #[component(name = "minecraft:memory_behavior.prepare_ram_attack", released = "1.26.50", molang, parse = goal::parse::<goal::MemoryPrepareRamAttack>)]
    MemoryBehaviorPrepareRamAttack(goal::MemoryPrepareRamAttack),
    /// `minecraft:memory_behavior.ram_attack` (a `.<name>` family).
    #[component(name = "minecraft:memory_behavior.ram_attack", released = "1.26.50", molang, parse = goal::parse::<goal::MemoryRamAttack>)]
    MemoryBehaviorRamAttack(goal::MemoryRamAttack),
    /// `minecraft:memory_behavior.wait`.
    #[component(name = "minecraft:memory_behavior.wait", released = "1.8.0", parse = goal::parse::<goal::MemoryWait>)]
    MemoryBehaviorWait(goal::MemoryWait),
    /// `minecraft:memory_sensor.find_nearest_block`.
    #[component(name = "minecraft:memory_sensor.find_nearest_block", released = "1.8.0", molang, parse = goal::parse::<goal::MemorySensorFindNearestBlock>)]
    MemorySensorFindNearestBlock(goal::MemorySensorFindNearestBlock),
    /// `minecraft:memory_sensor.find_nearest_entity`.
    #[component(name = "minecraft:memory_sensor.find_nearest_entity", released = "1.8.0", parse = goal::parse::<goal::MemorySensorFindNearestEntity>)]
    MemorySensorFindNearestEntity(goal::MemorySensorFindNearestEntity),
    /// `minecraft:memory_sensor.find_nearest_poi`.
    #[component(name = "minecraft:memory_sensor.find_nearest_poi", released = "1.8.0", parse = goal::parse::<goal::MemorySensorFindNearestPoi>)]
    MemorySensorFindNearestPoi(goal::MemorySensorFindNearestPoi),
    /// `minecraft:memory_sensor.find_ram_charge_position`.
    #[component(name = "minecraft:memory_sensor.find_ram_charge_position", released = "1.26.50", parse = goal::parse::<goal::MemorySensorFindRamChargePosition>)]
    MemorySensorFindRamChargePosition(goal::MemorySensorFindRamChargePosition),
    /// `minecraft:memory_sensor.game_event`.
    #[component(name = "minecraft:memory_sensor.game_event", released = "1.26.20", parse = goal::parse::<goal::MemorySensorGameEvent>)]
    MemorySensorGameEvent(goal::MemorySensorGameEvent),
    /// `minecraft:memory_sensor.in_range_of_entity`.
    #[component(name = "minecraft:memory_sensor.in_range_of_entity", released = "1.8.0", parse = goal::parse::<goal::MemorySensorInRangeOfEntity>)]
    MemorySensorInRangeOfEntity(goal::MemorySensorInRangeOfEntity),
    /// `minecraft:not_pickable_from_inside`.
    #[component(name = "minecraft:not_pickable_from_inside", released = "1.26.40", parse = parse_typed::<NotPickableFromInside>)]
    NotPickableFromInside(NotPickableFromInside),
    /// `minecraft:on_death`.
    #[component(name = "minecraft:on_death", released = "1.8.0", parse = parse_typed::<OnDeath>)]
    OnDeath(OnDeath),
    /// `minecraft:on_equipment_changed`.
    #[component(name = "minecraft:on_equipment_changed", released = "1.26.20", parse = parse_typed::<OnEquipmentChanged>)]
    OnEquipmentChanged(OnEquipmentChanged),
    /// `minecraft:on_friendly_anger`.
    #[component(name = "minecraft:on_friendly_anger", released = "1.8.0", parse = parse_typed::<OnFriendlyAnger>)]
    OnFriendlyAnger(OnFriendlyAnger),
    /// `minecraft:on_hurt`.
    #[component(name = "minecraft:on_hurt", released = "1.8.0", parse = parse_typed::<OnHurt>)]
    OnHurt(OnHurt),
    /// `minecraft:on_hurt_by_player`.
    #[component(name = "minecraft:on_hurt_by_player", released = "1.8.0", parse = parse_typed::<OnHurtByPlayer>)]
    OnHurtByPlayer(OnHurtByPlayer),
    /// `minecraft:on_ignite`.
    #[component(name = "minecraft:on_ignite", released = "1.8.0", parse = parse_typed::<OnIgnite>)]
    OnIgnite(OnIgnite),
    /// `minecraft:on_start_landing`.
    #[component(name = "minecraft:on_start_landing", released = "1.8.0", parse = parse_typed::<OnStartLanding>)]
    OnStartLanding(OnStartLanding),
    /// `minecraft:on_start_takeoff`.
    #[component(name = "minecraft:on_start_takeoff", released = "1.8.0", parse = parse_typed::<OnStartTakeoff>)]
    OnStartTakeoff(OnStartTakeoff),
    /// `minecraft:on_target_acquired`.
    #[component(name = "minecraft:on_target_acquired", released = "1.8.0", parse = parse_typed::<OnTargetAcquired>)]
    OnTargetAcquired(OnTargetAcquired),
    /// `minecraft:on_target_escape`.
    #[component(name = "minecraft:on_target_escape", released = "1.8.0", parse = parse_typed::<OnTargetEscape>)]
    OnTargetEscape(OnTargetEscape),
    /// `minecraft:on_wake_with_owner`.
    #[component(name = "minecraft:on_wake_with_owner", released = "1.8.0", parse = parse_typed::<OnWakeWithOwner>)]
    OnWakeWithOwner(OnWakeWithOwner),
    /// `minecraft:out_of_control`.
    #[component(name = "minecraft:out_of_control", released = "1.8.0", parse = parse_typed::<OutOfControl>)]
    OutOfControl(OutOfControl),
    /// `minecraft:persistent`.
    #[component(name = "minecraft:persistent", released = "1.8.0", parse = parse_typed::<Persistent>)]
    Persistent(Persistent),
    /// `minecraft:physics`.
    #[component(name = "minecraft:physics", released = "1.8.0", parse = parse_typed::<Physics>)]
    Physics(Physics),
    /// `minecraft:preferred_path`.
    #[component(name = "minecraft:preferred_path", released = "1.8.0", molang, parse = parse_typed::<PreferredPath>)]
    PreferredPath(PreferredPath),
    /// `minecraft:projectile`.
    #[component(name = "minecraft:projectile", released = "1.8.0", fields = [{ name = "on_hit.impact_damage.apply_knockback_to_blocking_targets", since_engine = "1.21.120" }, { name = "on_hit.impact_damage.ceil_pre_critical_damage", since_engine = "1.26.40" }, { name = "on_hit.impact_damage.difficulty_randomization", since_engine = "1.26.40" }], parse = parse_typed::<Projectile>)]
    Projectile(Projectile),
    /// `minecraft:pushable_by_entity`.
    #[component(name = "minecraft:pushable_by_entity", parse = parse_typed::<PushableByEntity>)]
    PushableByEntity(PushableByEntity),
    /// `minecraft:renders_when_invisible`.
    #[component(name = "minecraft:renders_when_invisible", released = "1.21.60", parse = parse_typed::<RendersWhenInvisible>)]
    RendersWhenInvisible(RendersWhenInvisible),
    /// `minecraft:rotation_axis_aligned`.
    #[component(name = "minecraft:rotation_axis_aligned", released = "1.21.100", supersedes = "minecraft:body_rotation_axis_aligned", parse = parse_typed::<RotationAxisAligned>)]
    RotationAxisAligned(RotationAxisAligned),
    /// `minecraft:rotation_locked_to_vehicle`.
    #[component(name = "minecraft:rotation_locked_to_vehicle", released = "1.21.120", parse = parse_typed::<RotationLockedToVehicle>)]
    RotationLockedToVehicle(RotationLockedToVehicle),
    /// `minecraft:spawn_on_death`.
    #[component(name = "minecraft:spawn_on_death", released = "1.26.20", parse = parse_typed::<SpawnOnDeath>)]
    SpawnOnDeath(SpawnOnDeath),
    /// `minecraft:suspect_tracking`.
    #[component(name = "minecraft:suspect_tracking", released = "1.8.0", parse = parse_typed::<SuspectTracking>)]
    SuspectTracking(SuspectTracking),
    /// `minecraft:tamemount`.
    #[component(name = "minecraft:tamemount", released = "1.8.0", molang, parse = parse_typed::<Tamemount>)]
    Tamemount(Tamemount),
    /// `minecraft:tick_world`.
    #[component(name = "minecraft:tick_world", released = "1.8.0", parse = parse_typed::<TickWorld>)]
    TickWorld(TickWorld),
    /// `minecraft:trade_resupply`.
    #[component(name = "minecraft:trade_resupply", released = "1.8.0", parse = parse_typed::<TradeResupply>)]
    TradeResupply(TradeResupply),
    /// `minecraft:trade_table`.
    #[component(name = "minecraft:trade_table", released = "1.8.0", parse = parse_typed::<TradeTable>)]
    TradeTable(TradeTable),
    /// `minecraft:transient`.
    #[component(name = "minecraft:transient", released = "1.8.0", parse = parse_typed::<Transient>)]
    Transient(Transient),
    /// `minecraft:trust`.
    #[component(name = "minecraft:trust", released = "1.8.0", parse = parse_typed::<Trust>)]
    Trust(Trust),
    /// `minecraft:uses_legacy_ambient_sounds`.
    #[component(name = "minecraft:uses_legacy_ambient_sounds", released = "1.26.50", parse = parse_typed::<UsesLegacyAmbientSounds>)]
    UsesLegacyAmbientSounds(UsesLegacyAmbientSounds),
    /// `minecraft:uses_legacy_friction`.
    #[component(name = "minecraft:uses_legacy_friction", released = "1.26.20", parse = parse_typed::<UsesLegacyFriction>)]
    UsesLegacyFriction(UsesLegacyFriction),
    /// `minecraft:uses_uniform_air_drag`.
    #[component(name = "minecraft:uses_uniform_air_drag", released = "1.26.20", parse = parse_typed::<UsesUniformAirDrag>)]
    UsesUniformAirDrag(UsesUniformAirDrag),
    /// `minecraft:vibration_damper`.
    #[component(name = "minecraft:vibration_damper", released = "1.8.0", parse = parse_typed::<VibrationDamper>)]
    VibrationDamper(VibrationDamper),
    /// `minecraft:vibration_listener`.
    #[component(name = "minecraft:vibration_listener", released = "1.8.0", parse = parse_typed::<VibrationListener>)]
    VibrationListener(VibrationListener),
    /// `minecraft:wants_jockey`.
    #[component(name = "minecraft:wants_jockey", released = "1.8.0", parse = parse_typed::<WantsJockey>)]
    WantsJockey(WantsJockey),
    /// `minecraft:wither_target_highest_damage`.
    #[component(name = "minecraft:wither_target_highest_damage", released = "1.8.0", parse = parse_typed::<WitherTargetHighestDamage>)]
    WitherTargetHighestDamage(WitherTargetHighestDamage),
    // The components 1.26.60 types first (27 component keys; first typed in
    // `history/typed.rs`).
    /// `minecraft:buoyant`.
    #[component(name = "minecraft:buoyant", released = "1.8.0", molang, parse = parse_typed::<Buoyant>)]
    Buoyant(Buoyant),
    /// `minecraft:dash_action`.
    #[component(name = "minecraft:dash_action", released = "1.21.110", supersedes = "minecraft:dash", parse = parse_typed::<DashAction>)]
    DashAction(DashAction),
    /// `minecraft:movement.amphibious`.
    #[component(name = "minecraft:movement.amphibious", released = "1.8.0", parse = parse_typed::<MovementAmphibious>)]
    MovementAmphibious(MovementAmphibious),
    /// `minecraft:movement.basic`.
    #[component(name = "minecraft:movement.basic", released = "1.8.0", parse = parse_typed::<MovementBasic>)]
    MovementBasic(MovementBasic),
    /// `minecraft:movement.fly`.
    #[component(name = "minecraft:movement.fly", released = "1.8.0", parse = parse_typed::<MovementFly>)]
    MovementFly(MovementFly),
    /// `minecraft:movement.generic`.
    #[component(name = "minecraft:movement.generic", released = "1.8.0", parse = parse_typed::<MovementGeneric>)]
    MovementGeneric(MovementGeneric),
    /// `minecraft:movement.glide`.
    #[component(name = "minecraft:movement.glide", released = "1.8.0", parse = parse_typed::<MovementGlide>)]
    MovementGlide(MovementGlide),
    /// `minecraft:movement.hover`.
    #[component(name = "minecraft:movement.hover", released = "1.8.0", parse = parse_typed::<MovementHover>)]
    MovementHover(MovementHover),
    /// `minecraft:movement.jump`.
    #[component(name = "minecraft:movement.jump", released = "1.8.0", parse = parse_typed::<MovementJump>)]
    MovementJump(MovementJump),
    /// `minecraft:movement.skip`.
    #[component(name = "minecraft:movement.skip", released = "1.8.0", parse = parse_typed::<MovementSkip>)]
    MovementSkip(MovementSkip),
    /// `minecraft:movement.sway`.
    #[component(name = "minecraft:movement.sway", released = "1.8.0", parse = parse_typed::<MovementSway>)]
    MovementSway(MovementSway),
    /// `minecraft:rail_movement`.
    #[component(name = "minecraft:rail_movement", released = "1.8.0", parse = parse_typed::<RailMovement>)]
    RailMovement(RailMovement),
    /// `minecraft:attack`.
    #[component(name = "minecraft:attack", released = "1.8.0", parse = parse_typed::<Attack>)]
    Attack(Attack),
    /// `minecraft:healable`.
    #[component(name = "minecraft:healable", released = "1.8.0", molang, parse = parse_typed::<Healable>)]
    Healable(Healable),
    /// `minecraft:spell_effects`.
    #[component(name = "minecraft:spell_effects", released = "1.8.0", parse = parse_typed::<SpellEffects>)]
    SpellEffects(SpellEffects),
    /// `minecraft:strength`.
    #[component(name = "minecraft:strength", released = "1.8.0", parse = parse_typed::<Strength>)]
    Strength(Strength),
    /// `minecraft:teleport`.
    #[component(name = "minecraft:teleport", released = "1.8.0", parse = parse_typed::<Teleport>)]
    Teleport(Teleport),
    /// `minecraft:timer`.
    #[component(name = "minecraft:timer", released = "1.8.0", parse = parse_typed::<Timer>)]
    Timer(Timer),
    /// `minecraft:trail`.
    #[component(name = "minecraft:trail", released = "1.8.0", molang, parse = parse_typed::<Trail>)]
    Trail(Trail),
    /// `minecraft:transformation`.
    #[component(name = "minecraft:transformation", released = "1.8.0", parse = parse_typed::<Transformation>)]
    Transformation(Transformation),
    /// `minecraft:annotation.break_door`.
    #[component(name = "minecraft:annotation.break_door", released = "1.8.0", parse = parse_typed::<AnnotationBreakDoor>)]
    AnnotationBreakDoor(AnnotationBreakDoor),
    /// `minecraft:economy_trade_table`.
    #[component(name = "minecraft:economy_trade_table", released = "1.8.0", parse = parse_typed::<EconomyTradeTable>)]
    EconomyTradeTable(EconomyTradeTable),
    /// `minecraft:inventory`.
    #[component(name = "minecraft:inventory", released = "1.8.0", parse = parse_typed::<Inventory>)]
    Inventory(Inventory),
    /// `minecraft:pushable_by_block`.
    #[component(name = "minecraft:pushable_by_block", released = "1.26.10", supersedes = "minecraft:pushable", parse = parse_typed::<PushableByBlock>)]
    PushableByBlock(PushableByBlock),
    /// `minecraft:rideable`.
    #[component(name = "minecraft:rideable", released = "1.8.0", molang, parse = parse_typed::<Rideable>)]
    Rideable(Rideable),
    /// `minecraft:scale_by_age`.
    #[component(name = "minecraft:scale_by_age", released = "1.8.0", parse = parse_typed::<ScaleByAge>)]
    ScaleByAge(ScaleByAge),
    /// `minecraft:shareables`.
    #[component(name = "minecraft:shareables", released = "1.8.0", molang, parse = parse_typed::<Shareables>)]
    Shareables(Shareables),
    /// A custom component (`namespace:name`). Entities have none in vanilla; the variant
    /// completes the shared component model.
    Custom(CustomComponent),
    /// A key no row names, kept as written for write-back (`UnrecognizedComponent`).
    Unknown(Value),
    /// A key vanilla parses at run time from the JSON, kept as written.
    /// Each attribute below is one row of the table.
    #[component(name = "minecraft:absorption", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:ageable", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:air_drag_modifier", released = "1.26.30", requires = [BETA_DOCUMENT], parse = keep_json)]
    #[component(name = "minecraft:ambient_sound_interval", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:angry", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:area_attack", released = "1.8.0", fields = [{ name = "use_self_as_damage_source", since_engine = "1.26.30" }], parse = keep_json)]
    #[component(name = "minecraft:attack_cooldown", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:attack_damage", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:balloon", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:balloonable", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:behavior.enderman_leave_block", released = "1.8.0", deprecated_since = "1.21.100", parse = keep_json)]
    #[component(name = "minecraft:behavior.enderman_take_block", released = "1.8.0", deprecated_since = "1.21.100", parse = keep_json)]
    #[component(name = "minecraft:behavior.vex_copy_owner_target", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:behavior.vex_random_move", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:block_sensor", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:boostable", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:bounciness", released = "1.26.30", requires = [BETA_DOCUMENT], parse = keep_json)]
    #[component(name = "minecraft:break_blocks", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:breathable", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:breedable", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:bribeable", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:celebrate_hunt", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:color", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:color2", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:combat_regeneration", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:conditional_bandwidth_optimization", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:custom_hit_test", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:damage_sensor", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:default_look_angle", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:despawn", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:drying_out_timer", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:dweller", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:entity_armor_equipment_slot_mapping", released = "1.21.10", parse = keep_json)]
    #[component(name = "minecraft:entity_sensor", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:environment_sensor", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:equip_item", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:equipment", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:equippable", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:explode", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:flying_speed", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:follow_range", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:free_camera_controlled", released = "1.21.110", supersedes = "minecraft:input_air_controlled", parse = keep_json)]
    #[component(name = "minecraft:friction_modifier", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:genetics", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:giveable", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:ground_offset", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:group_size", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:health", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:heartbeat", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:horse.jump_strength", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:hurt_on_condition", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:inside_block_notifier", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:is_collidable", released = "1.21.70", parse = keep_json)]
    #[component(name = "minecraft:is_dyeable", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:item_controllable", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:knockback_resistance", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:lava_movement", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:leashable", released = "1.8.0", fields = [{ name = "unleash_on_removal", since_engine = "1.26.30" }], parse = keep_json)]
    #[component(name = "minecraft:leashable_to", released = "1.8.0", fields = [{ name = "unleash_on_removal", since_engine = "1.26.30" }], parse = keep_json)]
    #[component(name = "minecraft:looked_at", released = "1.21.40", supersedes = "minecraft:lookat", parse = keep_json)]
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
    #[component(name = "minecraft:nameable", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:navigation.climb", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:navigation.float", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:navigation.fly", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:navigation.generic", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:navigation.hover", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:navigation.swim", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:navigation.walk", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:npc", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:offspring", released = "1.26.0", parse = keep_json)]
    #[component(name = "minecraft:peek", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:player.exhaustion", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:player.experience", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:player.hunger", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:player.level", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:player.saturation", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:push_through", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:raid_trigger", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:rail_sensor", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:ravager_blocked", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:reflect_projectiles", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:scale", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:scheduler", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:shooter", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:sittable", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:skin_id", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:sound_volume", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:spawn_egg_interaction", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:spawn_entity", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:tameable", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:target_nearby_sensor", released = "1.8.0", parse = keep_json)]
    #[component(name = "minecraft:trusting", released = "1.8.0", parse = keep_json)]
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
