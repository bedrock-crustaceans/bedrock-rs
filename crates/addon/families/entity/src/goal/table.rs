//! Every goal key and its parse by the document's format version, sorted by key.

use super::check::{VersionedParseFn, parse_versioned};

/// The goals still listed by hand, each `key => Type`: [`VERSIONED`], and (for the test that
/// holds each goal's `Goal::TYPED` to its row) `TYPED`, from the one list.
macro_rules! goals {
    ($($key:literal => $goal:ident,)*) => {
        /// `(key, parse)` for every goal, Brain memory goal and memory sensor.
        pub(crate) const VERSIONED: &[(&str, VersionedParseFn)] =
            &[$(($key, parse_versioned::<super::$goal>)),*];

        /// `(key, Goal::TYPED)` of each goal of [`VERSIONED`].
        #[cfg(test)]
        pub(crate) const TYPED: &[(&str, bedrock_addon_core::version::FormatVersion)] =
            &[$(($key, <super::$goal as super::check::Goal>::TYPED)),*];
    };
}

goals! {
    "minecraft:behavior.admire_item" => AdmireItem,
    "minecraft:behavior.avoid_block" => AvoidBlock,
    "minecraft:behavior.avoid_mob_type" => AvoidMobType,
    "minecraft:behavior.celebrate" => Celebrate,
    "minecraft:behavior.celebrate_survive" => CelebrateSurvive,
    "minecraft:behavior.defend_trusted_target" => DefendTrustedTarget,
    "minecraft:behavior.delayed_attack" => DelayedAttack,
    "minecraft:behavior.dig" => Dig,
    "minecraft:behavior.drop_item_for" => DropItemFor,
    "minecraft:behavior.eat_block" => EatBlock,
    "minecraft:behavior.emerge" => Emerge,
    // Typed from 1.26.60.
    "minecraft:behavior.float_tempt" => FloatTempt,
    "minecraft:behavior.go_and_give_items_to_noteblock" => GoAndGiveItemsToNoteblock,
    "minecraft:behavior.go_and_give_items_to_owner" => GoAndGiveItemsToOwner,
    "minecraft:behavior.go_home" => GoHome,
    "minecraft:behavior.hold_ground" => HoldGround,
    "minecraft:behavior.knockback_roar" => KnockbackRoar,
    "minecraft:behavior.lay_egg" => LayEgg,
    "minecraft:behavior.melee_attack" => MeleeAttack,
    "minecraft:behavior.melee_box_attack" => MeleeBoxAttack,
    "minecraft:behavior.move_to_block" => MoveToBlock,
    // Typed from 1.26.60.
    "minecraft:behavior.panic" => Panic,
    "minecraft:behavior.pickup_items" => PickupItems,
    "minecraft:behavior.place_block" => PlaceBlock,
    "minecraft:behavior.ram_attack" => RamAttack,
    "minecraft:behavior.random_search_and_dig" => RandomSearchAndDig,
    "minecraft:behavior.stomp_attack" => StompAttack,
    "minecraft:behavior.take_block" => TakeBlock,
    "minecraft:behavior.take_flower" => TakeFlower,
    // Typed from 1.26.60.
    "minecraft:behavior.tempt" => Tempt,
    "minecraft:behavior.timer_flag_1" => TimerFlag1,
    "minecraft:behavior.timer_flag_2" => TimerFlag2,
    "minecraft:behavior.timer_flag_3" => TimerFlag3,
    "minecraft:behavior.use_kinetic_weapon" => UseKineticWeapon,
    "minecraft:behavior.work" => Work,
    "minecraft:behavior.work_composter" => WorkComposter,
    // Typed from 1.26.50.
    "minecraft:memory_behavior.jump_to_block" => MemoryJumpToBlock,
    "minecraft:memory_behavior.move_to_position" => MemoryMoveToPosition,
    // Typed from 1.26.50.
    "minecraft:memory_behavior.prepare_ram_attack" => MemoryPrepareRamAttack,
    // Typed from 1.26.50.
    "minecraft:memory_behavior.ram_attack" => MemoryRamAttack,
    "minecraft:memory_sensor.find_nearest_block" => MemorySensorFindNearestBlock,
    "minecraft:memory_sensor.find_nearest_entity" => MemorySensorFindNearestEntity,
    "minecraft:memory_sensor.find_nearest_poi" => MemorySensorFindNearestPoi,
    // Typed from 1.26.50.
    "minecraft:memory_sensor.find_ram_charge_position" => MemorySensorFindRamChargePosition,
    "minecraft:memory_sensor.game_event" => MemorySensorGameEvent,
    "minecraft:memory_sensor.in_range_of_entity" => MemorySensorInRangeOfEntity,
}

#[cfg(test)]
mod tests {
    use crate::component::EntityComponent;

    /// Each goal listed by hand states the typed version its row does: `Goal::TYPED` is what the
    /// parse splits on, `typed_since` what the component table says.
    #[test]
    fn each_hand_listed_goals_typed_version_is_its_rows() {
        assert_eq!(super::TYPED.len(), super::VERSIONED.len());
        for (key, typed) in super::TYPED {
            assert_eq!(
                EntityComponent::typed_since(key),
                Some(*typed),
                "{key}: Goal::TYPED against its row"
            );
        }
    }
}
