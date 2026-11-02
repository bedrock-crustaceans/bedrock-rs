//! Every typed goal: its key, the format version vanilla first types it at, and the
//! round trip of its type. Generated from and the component table.

use bedrock_addon_family_entity::goal;

use super::{Goal, round_trip};

/// `(key, first typed, round trip)` for every `minecraft:behavior.*` goal.
pub const GOALS: &[Goal] = &[
    (
        "minecraft:behavior.admire_item",
        (1, 26, 20),
        round_trip::<goal::AdmireItem>,
    ),
    (
        "minecraft:behavior.aquatic_charge_attack",
        (1, 21, 120),
        round_trip::<goal::AquaticChargeAttack>,
    ),
    (
        "minecraft:behavior.avoid_block",
        (1, 21, 120),
        round_trip::<goal::AvoidBlock>,
    ),
    (
        "minecraft:behavior.avoid_mob_type",
        (1, 21, 120),
        round_trip::<goal::AvoidMobType>,
    ),
    (
        "minecraft:behavior.barter",
        (1, 26, 20),
        round_trip::<goal::Barter>,
    ),
    (
        "minecraft:behavior.beg",
        (1, 26, 40),
        round_trip::<goal::Beg>,
    ),
    (
        "minecraft:behavior.break_door",
        (1, 26, 30),
        round_trip::<goal::BreakDoor>,
    ),
    (
        "minecraft:behavior.breed",
        (1, 21, 90),
        round_trip::<goal::Breed>,
    ),
    (
        "minecraft:behavior.celebrate",
        (1, 26, 10),
        round_trip::<goal::Celebrate>,
    ),
    (
        "minecraft:behavior.celebrate_survive",
        (1, 26, 10),
        round_trip::<goal::CelebrateSurvive>,
    ),
    (
        "minecraft:behavior.charge_attack",
        (1, 21, 100),
        round_trip::<goal::ChargeAttack>,
    ),
    (
        "minecraft:behavior.charge_held_item",
        (1, 26, 40),
        round_trip::<goal::ChargeHeldItem>,
    ),
    (
        "minecraft:behavior.circle_around_anchor",
        (1, 21, 100),
        round_trip::<goal::CircleAroundAnchor>,
    ),
    (
        "minecraft:behavior.controlled_by_player",
        (1, 21, 100),
        round_trip::<goal::ControlledByPlayer>,
    ),
    (
        "minecraft:behavior.croak",
        (1, 26, 30),
        round_trip::<goal::Croak>,
    ),
    (
        "minecraft:behavior.defend_trusted_target",
        (1, 26, 30),
        round_trip::<goal::DefendTrustedTarget>,
    ),
    (
        "minecraft:behavior.defend_village_target",
        (1, 26, 0),
        round_trip::<goal::DefendVillageTarget>,
    ),
    (
        "minecraft:behavior.delayed_attack",
        (1, 26, 0),
        round_trip::<goal::DelayedAttack>,
    ),
    (
        "minecraft:behavior.dig",
        (1, 21, 120),
        round_trip::<goal::Dig>,
    ),
    (
        "minecraft:behavior.door_interact",
        (1, 26, 30),
        round_trip::<goal::DoorInteract>,
    ),
    (
        "minecraft:behavior.dragonchargeplayer",
        (1, 26, 0),
        round_trip::<goal::DragonChargePlayer>,
    ),
    (
        "minecraft:behavior.dragondeath",
        (1, 26, 30),
        round_trip::<goal::DragonDeath>,
    ),
    (
        "minecraft:behavior.dragonflaming",
        (1, 21, 100),
        round_trip::<goal::DragonFlaming>,
    ),
    (
        "minecraft:behavior.dragonholdingpattern",
        (1, 26, 30),
        round_trip::<goal::DragonHoldingPattern>,
    ),
    (
        "minecraft:behavior.dragonlanding",
        (1, 26, 30),
        round_trip::<goal::DragonLanding>,
    ),
    (
        "minecraft:behavior.dragonscanning",
        (1, 26, 30),
        round_trip::<goal::DragonScanning>,
    ),
    (
        "minecraft:behavior.dragonstrafeplayer",
        (1, 26, 0),
        round_trip::<goal::DragonStrafePlayer>,
    ),
    (
        "minecraft:behavior.dragontakeoff",
        (1, 26, 30),
        round_trip::<goal::DragonTakeoff>,
    ),
    (
        "minecraft:behavior.drink_milk",
        (1, 21, 120),
        round_trip::<goal::DrinkMilk>,
    ),
    (
        "minecraft:behavior.drink_potion",
        (1, 26, 40),
        round_trip::<goal::DrinkPotion>,
    ),
    (
        "minecraft:behavior.drop_item_for",
        (1, 26, 10),
        round_trip::<goal::DropItemFor>,
    ),
    (
        "minecraft:behavior.eat_block",
        (1, 26, 40),
        round_trip::<goal::EatBlock>,
    ),
    (
        "minecraft:behavior.eat_carried_item",
        (1, 26, 20),
        round_trip::<goal::EatCarriedItem>,
    ),
    (
        "minecraft:behavior.eat_mob",
        (1, 21, 100),
        round_trip::<goal::EatMob>,
    ),
    (
        "minecraft:behavior.emerge",
        (1, 26, 30),
        round_trip::<goal::Emerge>,
    ),
    (
        "minecraft:behavior.equip_item",
        (1, 26, 20),
        round_trip::<goal::EquipItem>,
    ),
    (
        "minecraft:behavior.explore_outskirts",
        (1, 21, 130),
        round_trip::<goal::ExploreOutskirts>,
    ),
    (
        "minecraft:behavior.fertilize_farm_block",
        (1, 26, 10),
        round_trip::<goal::FertilizeFarmBlock>,
    ),
    (
        "minecraft:behavior.find_cover",
        (1, 26, 20),
        round_trip::<goal::FindCover>,
    ),
    (
        "minecraft:behavior.find_mount",
        (1, 26, 20),
        round_trip::<goal::FindMount>,
    ),
    (
        "minecraft:behavior.find_underwater_treasure",
        (1, 26, 20),
        round_trip::<goal::FindUnderwaterTreasure>,
    ),
    (
        "minecraft:behavior.fire_at_target",
        (1, 26, 30),
        round_trip::<goal::FireAtTarget>,
    ),
    (
        "minecraft:behavior.flee_sun",
        (1, 26, 20),
        round_trip::<goal::FleeSun>,
    ),
    (
        "minecraft:behavior.float",
        (1, 26, 20),
        round_trip::<goal::Float>,
    ),
    (
        "minecraft:behavior.float_wander",
        (1, 26, 40),
        round_trip::<goal::FloatWander>,
    ),
    (
        "minecraft:behavior.follow_caravan",
        (1, 26, 30),
        round_trip::<goal::FollowCaravan>,
    ),
    (
        "minecraft:behavior.follow_mob",
        (1, 26, 50),
        round_trip::<goal::FollowMob>,
    ),
    (
        "minecraft:behavior.follow_owner",
        (1, 26, 20),
        round_trip::<goal::FollowOwner>,
    ),
    (
        "minecraft:behavior.follow_parent",
        (1, 26, 20),
        round_trip::<goal::FollowParent>,
    ),
    (
        "minecraft:behavior.follow_target_leader",
        (1, 26, 30),
        round_trip::<goal::FollowTargetLeader>,
    ),
    (
        "minecraft:behavior.go_and_give_items_to_noteblock",
        (1, 26, 20),
        round_trip::<goal::GoAndGiveItemsToNoteblock>,
    ),
    (
        "minecraft:behavior.go_and_give_items_to_owner",
        (1, 26, 20),
        round_trip::<goal::GoAndGiveItemsToOwner>,
    ),
    (
        "minecraft:behavior.go_home",
        (1, 26, 20),
        round_trip::<goal::GoHome>,
    ),
    (
        "minecraft:behavior.guardian_attack",
        (1, 26, 0),
        round_trip::<goal::GuardianAttack>,
    ),
    (
        "minecraft:behavior.harvest_farm_block",
        (1, 26, 10),
        round_trip::<goal::HarvestFarmBlock>,
    ),
    (
        "minecraft:behavior.hide",
        (1, 26, 20),
        round_trip::<goal::Hide>,
    ),
    (
        "minecraft:behavior.hold_ground",
        (1, 26, 20),
        round_trip::<goal::HoldGround>,
    ),
    (
        "minecraft:behavior.hover",
        (1, 26, 20),
        round_trip::<goal::Hover>,
    ),
    (
        "minecraft:behavior.hurt_by_target",
        (1, 26, 30),
        round_trip::<goal::HurtByTarget>,
    ),
    (
        "minecraft:behavior.inspect_bookshelf",
        (1, 26, 10),
        round_trip::<goal::InspectBookshelf>,
    ),
    (
        "minecraft:behavior.investigate_suspicious_location",
        (1, 21, 130),
        round_trip::<goal::InvestigateSuspiciousLocation>,
    ),
    (
        "minecraft:behavior.jump_around_target",
        (1, 26, 20),
        round_trip::<goal::JumpAroundTarget>,
    ),
    (
        "minecraft:behavior.jump_to_block",
        (1, 26, 20),
        round_trip::<goal::JumpToBlock>,
    ),
    (
        "minecraft:behavior.knockback_roar",
        (1, 26, 40),
        round_trip::<goal::KnockbackRoar>,
    ),
    (
        "minecraft:behavior.lay_down",
        (1, 26, 20),
        round_trip::<goal::LayDown>,
    ),
    (
        "minecraft:behavior.lay_egg",
        (1, 26, 10),
        round_trip::<goal::LayEgg>,
    ),
    (
        "minecraft:behavior.leap_at_target",
        (1, 26, 40),
        round_trip::<goal::LeapAtTarget>,
    ),
    (
        "minecraft:behavior.look_at_entity",
        (1, 26, 0),
        round_trip::<goal::LookAtEntity>,
    ),
    (
        "minecraft:behavior.look_at_player",
        (1, 26, 0),
        round_trip::<goal::LookAtPlayer>,
    ),
    (
        "minecraft:behavior.look_at_target",
        (1, 26, 0),
        round_trip::<goal::LookAtTarget>,
    ),
    (
        "minecraft:behavior.look_at_trading_player",
        (1, 26, 0),
        round_trip::<goal::LookAtTradingPlayer>,
    ),
    (
        "minecraft:behavior.make_love",
        (1, 26, 30),
        round_trip::<goal::MakeLove>,
    ),
    (
        "minecraft:behavior.melee_attack",
        (1, 26, 0),
        round_trip::<goal::MeleeAttack>,
    ),
    (
        "minecraft:behavior.melee_box_attack",
        (1, 26, 0),
        round_trip::<goal::MeleeBoxAttack>,
    ),
    (
        "minecraft:behavior.mingle",
        (1, 26, 20),
        round_trip::<goal::Mingle>,
    ),
    (
        "minecraft:behavior.mount_pathing",
        (1, 26, 20),
        round_trip::<goal::MountPathing>,
    ),
    (
        "minecraft:behavior.move_around_target",
        (1, 26, 20),
        round_trip::<goal::MoveAroundTarget>,
    ),
    (
        "minecraft:behavior.move_indoors",
        (1, 26, 20),
        round_trip::<goal::MoveIndoors>,
    ),
    (
        "minecraft:behavior.move_outdoors",
        (1, 26, 20),
        round_trip::<goal::MoveOutdoors>,
    ),
    (
        "minecraft:behavior.move_through_village",
        (1, 26, 20),
        round_trip::<goal::MoveThroughVillage>,
    ),
    (
        "minecraft:behavior.move_to_block",
        (1, 26, 40),
        round_trip::<goal::MoveToBlock>,
    ),
    (
        "minecraft:behavior.move_to_land",
        (1, 26, 10),
        round_trip::<goal::MoveToLand>,
    ),
    (
        "minecraft:behavior.move_to_lava",
        (1, 26, 10),
        round_trip::<goal::MoveToLava>,
    ),
    (
        "minecraft:behavior.move_to_liquid",
        (1, 26, 10),
        round_trip::<goal::MoveToLiquid>,
    ),
    (
        "minecraft:behavior.move_to_poi",
        (1, 26, 20),
        round_trip::<goal::MoveToPoi>,
    ),
    (
        "minecraft:behavior.move_to_random_block",
        (1, 26, 20),
        round_trip::<goal::MoveToRandomBlock>,
    ),
    (
        "minecraft:behavior.move_to_village",
        (1, 26, 20),
        round_trip::<goal::MoveToVillage>,
    ),
    (
        "minecraft:behavior.move_to_water",
        (1, 26, 10),
        round_trip::<goal::MoveToWater>,
    ),
    (
        "minecraft:behavior.move_towards_dwelling_restriction",
        (1, 26, 0),
        round_trip::<goal::MoveTowardsDwellingRestriction>,
    ),
    (
        "minecraft:behavior.move_towards_home_restriction",
        (1, 26, 0),
        round_trip::<goal::MoveTowardsHomeRestriction>,
    ),
    (
        "minecraft:behavior.move_towards_target",
        (1, 26, 20),
        round_trip::<goal::MoveTowardsTarget>,
    ),
    (
        "minecraft:behavior.nap",
        (1, 26, 40),
        round_trip::<goal::Nap>,
    ),
    (
        "minecraft:behavior.nearest_attackable_target",
        (1, 26, 0),
        round_trip::<goal::NearestAttackableTarget>,
    ),
    (
        "minecraft:behavior.nearest_prioritized_attackable_target",
        (1, 26, 30),
        round_trip::<goal::NearestPrioritizedAttackableTarget>,
    ),
    (
        "minecraft:behavior.ocelot_sit_on_block",
        (1, 26, 10),
        round_trip::<goal::OcelotSitOnBlock>,
    ),
    (
        "minecraft:behavior.ocelotattack",
        (1, 26, 10),
        round_trip::<goal::Ocelotattack>,
    ),
    (
        "minecraft:behavior.offer_flower",
        (1, 26, 10),
        round_trip::<goal::OfferFlower>,
    ),
    (
        "minecraft:behavior.open_door",
        (1, 26, 30),
        round_trip::<goal::OpenDoor>,
    ),
    (
        "minecraft:behavior.owner_hurt_by_target",
        (1, 26, 30),
        round_trip::<goal::OwnerHurtByTarget>,
    ),
    (
        "minecraft:behavior.owner_hurt_target",
        (1, 26, 30),
        round_trip::<goal::OwnerHurtTarget>,
    ),
    (
        "minecraft:behavior.pet_sleep_with_owner",
        (1, 26, 20),
        round_trip::<goal::PetSleepWithOwner>,
    ),
    (
        "minecraft:behavior.pickup_items",
        (1, 26, 20),
        round_trip::<goal::PickupItems>,
    ),
    (
        "minecraft:behavior.place_block",
        (1, 26, 30),
        round_trip::<goal::PlaceBlock>,
    ),
    (
        "minecraft:behavior.play",
        (1, 26, 10),
        round_trip::<goal::Play>,
    ),
    (
        "minecraft:behavior.play_dead",
        (1, 26, 10),
        round_trip::<goal::PlayDead>,
    ),
    (
        "minecraft:behavior.player_ride_tamed",
        (1, 26, 20),
        round_trip::<goal::PlayerRideTamed>,
    ),
    (
        "minecraft:behavior.raid_garden",
        (1, 26, 10),
        round_trip::<goal::RaidGarden>,
    ),
    (
        "minecraft:behavior.ram_attack",
        (1, 26, 10),
        round_trip::<goal::RamAttack>,
    ),
    (
        "minecraft:behavior.random_breach",
        (1, 26, 30),
        round_trip::<goal::RandomBreach>,
    ),
    (
        "minecraft:behavior.random_fly",
        (1, 26, 30),
        round_trip::<goal::RandomFly>,
    ),
    (
        "minecraft:behavior.random_hover",
        (1, 26, 20),
        round_trip::<goal::RandomHover>,
    ),
    (
        "minecraft:behavior.random_look_around",
        (1, 26, 30),
        round_trip::<goal::RandomLookAround>,
    ),
    (
        "minecraft:behavior.random_look_around_and_sit",
        (1, 26, 30),
        round_trip::<goal::RandomLookAroundAndSit>,
    ),
    (
        "minecraft:behavior.random_search_and_dig",
        (1, 26, 10),
        round_trip::<goal::RandomSearchAndDig>,
    ),
    (
        "minecraft:behavior.random_sitting",
        (1, 26, 20),
        round_trip::<goal::RandomSitting>,
    ),
    (
        "minecraft:behavior.random_stroll",
        (1, 26, 20),
        round_trip::<goal::RandomStroll>,
    ),
    (
        "minecraft:behavior.random_swim",
        (1, 26, 30),
        round_trip::<goal::RandomSwim>,
    ),
    (
        "minecraft:behavior.ranged_attack",
        (1, 26, 40),
        round_trip::<goal::RangedAttack>,
    ),
    (
        "minecraft:behavior.receive_love",
        (1, 26, 30),
        round_trip::<goal::ReceiveLove>,
    ),
    (
        "minecraft:behavior.restrict_open_door",
        (1, 26, 30),
        round_trip::<goal::RestrictOpenDoor>,
    ),
    (
        "minecraft:behavior.restrict_sun",
        (1, 26, 30),
        round_trip::<goal::RestrictSun>,
    ),
    (
        "minecraft:behavior.rise_to_liquid_level",
        (1, 26, 30),
        round_trip::<goal::RiseToLiquidLevel>,
    ),
    (
        "minecraft:behavior.roar",
        (1, 21, 110),
        round_trip::<goal::Roar>,
    ),
    (
        "minecraft:behavior.roll",
        (1, 26, 30),
        round_trip::<goal::Roll>,
    ),
    (
        "minecraft:behavior.run_around_like_crazy",
        (1, 26, 30),
        round_trip::<goal::RunAroundLikeCrazy>,
    ),
    (
        "minecraft:behavior.scared",
        (1, 26, 30),
        round_trip::<goal::Scared>,
    ),
    (
        "minecraft:behavior.share_items",
        (1, 26, 30),
        round_trip::<goal::ShareItems>,
    ),
    (
        "minecraft:behavior.silverfish_merge_with_stone",
        (1, 26, 40),
        round_trip::<goal::SilverfishMergeWithStone>,
    ),
    (
        "minecraft:behavior.silverfish_wake_up_friends",
        (1, 26, 40),
        round_trip::<goal::SilverfishWakeUpFriends>,
    ),
    (
        "minecraft:behavior.skeleton_horse_trap",
        (1, 26, 30),
        round_trip::<goal::SkeletonHorseTrap>,
    ),
    (
        "minecraft:behavior.sleep",
        (1, 26, 20),
        round_trip::<goal::Sleep>,
    ),
    (
        "minecraft:behavior.slime_attack",
        (1, 21, 110),
        round_trip::<goal::SlimeAttack>,
    ),
    (
        "minecraft:behavior.slime_float",
        (1, 21, 110),
        round_trip::<goal::SlimeFloat>,
    ),
    (
        "minecraft:behavior.slime_keep_on_jumping",
        (1, 21, 110),
        round_trip::<goal::SlimeKeepOnJumping>,
    ),
    (
        "minecraft:behavior.slime_random_direction",
        (1, 21, 110),
        round_trip::<goal::SlimeRandomDirection>,
    ),
    (
        "minecraft:behavior.snacking",
        (1, 26, 40),
        round_trip::<goal::Snacking>,
    ),
    (
        "minecraft:behavior.sneeze",
        (1, 26, 30),
        round_trip::<goal::Sneeze>,
    ),
    (
        "minecraft:behavior.sniff",
        (1, 21, 110),
        round_trip::<goal::Sniff>,
    ),
    (
        "minecraft:behavior.sonic_boom",
        (1, 21, 110),
        round_trip::<goal::SonicBoom>,
    ),
    (
        "minecraft:behavior.squid_dive",
        (1, 26, 40),
        round_trip::<goal::SquidDive>,
    ),
    (
        "minecraft:behavior.squid_flee",
        (1, 26, 40),
        round_trip::<goal::SquidFlee>,
    ),
    (
        "minecraft:behavior.squid_idle",
        (1, 26, 40),
        round_trip::<goal::SquidIdle>,
    ),
    (
        "minecraft:behavior.squid_move_away_from_ground",
        (1, 26, 40),
        round_trip::<goal::SquidMoveAwayFromGround>,
    ),
    (
        "minecraft:behavior.squid_out_of_water",
        (1, 26, 40),
        round_trip::<goal::SquidOutOfWater>,
    ),
    (
        "minecraft:behavior.stalk_and_pounce_on_target",
        (1, 26, 40),
        round_trip::<goal::StalkAndPounceOnTarget>,
    ),
    (
        "minecraft:behavior.stay_near_noteblock",
        (1, 21, 110),
        round_trip::<goal::StayNearNoteblock>,
    ),
    (
        "minecraft:behavior.stay_while_sitting",
        (1, 26, 30),
        round_trip::<goal::StayWhileSitting>,
    ),
    (
        "minecraft:behavior.stomp_attack",
        (1, 26, 0),
        round_trip::<goal::StompAttack>,
    ),
    (
        "minecraft:behavior.stomp_turtle_egg",
        (1, 26, 10),
        round_trip::<goal::StompTurtleEgg>,
    ),
    (
        "minecraft:behavior.stroll_towards_village",
        (1, 26, 20),
        round_trip::<goal::StrollTowardsVillage>,
    ),
    (
        "minecraft:behavior.summon_entity",
        (1, 26, 40),
        round_trip::<goal::SummonEntity>,
    ),
    (
        "minecraft:behavior.swell",
        (1, 26, 30),
        round_trip::<goal::Swell>,
    ),
    (
        "minecraft:behavior.swim_idle",
        (1, 21, 110),
        round_trip::<goal::SwimIdle>,
    ),
    (
        "minecraft:behavior.swim_up_for_breath",
        (1, 21, 110),
        round_trip::<goal::SwimUpForBreath>,
    ),
    (
        "minecraft:behavior.swim_wander",
        (1, 21, 110),
        round_trip::<goal::SwimWander>,
    ),
    (
        "minecraft:behavior.swim_with_entity",
        (1, 26, 10),
        round_trip::<goal::SwimWithEntity>,
    ),
    (
        "minecraft:behavior.swoop_attack",
        (1, 21, 110),
        round_trip::<goal::SwoopAttack>,
    ),
    (
        "minecraft:behavior.take_block",
        (1, 26, 30),
        round_trip::<goal::TakeBlock>,
    ),
    (
        "minecraft:behavior.take_flower",
        (1, 26, 10),
        round_trip::<goal::TakeFlower>,
    ),
    (
        "minecraft:behavior.target_when_pushed",
        (1, 26, 30),
        round_trip::<goal::TargetWhenPushed>,
    ),
    (
        "minecraft:behavior.teleport_to_owner",
        (1, 26, 10),
        round_trip::<goal::TeleportToOwner>,
    ),
    (
        "minecraft:behavior.timer_flag_1",
        (1, 26, 0),
        round_trip::<goal::TimerFlag1>,
    ),
    (
        "minecraft:behavior.timer_flag_2",
        (1, 26, 0),
        round_trip::<goal::TimerFlag2>,
    ),
    (
        "minecraft:behavior.timer_flag_3",
        (1, 26, 0),
        round_trip::<goal::TimerFlag3>,
    ),
    (
        "minecraft:behavior.trade_interest",
        (1, 26, 30),
        round_trip::<goal::TradeInterest>,
    ),
    (
        "minecraft:behavior.trade_with_player",
        (1, 26, 10),
        round_trip::<goal::TradeWithPlayer>,
    ),
    (
        "minecraft:behavior.transport_items",
        (1, 21, 100),
        round_trip::<goal::TransportItems>,
    ),
    (
        "minecraft:behavior.use_kinetic_weapon",
        (1, 21, 120),
        round_trip::<goal::UseKineticWeapon>,
    ),
    (
        "minecraft:behavior.wither_random_attack_pos_goal",
        (1, 26, 30),
        round_trip::<goal::WitherRandomAttackPosGoal>,
    ),
    (
        "minecraft:behavior.wither_target_highest_damage",
        (1, 26, 30),
        round_trip::<goal::WitherTargetHighestDamage>,
    ),
    (
        "minecraft:behavior.work",
        (1, 26, 10),
        round_trip::<goal::Work>,
    ),
    (
        "minecraft:behavior.work_composter",
        (1, 26, 10),
        round_trip::<goal::WorkComposter>,
    ),
];
