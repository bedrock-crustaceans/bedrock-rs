// The entity's rows of the legacy probe: the steps of its chain and one test per band of
// versions. Included by the entity's test binaries (`writable_legacy_entity*.rs`): a test
// binary runs under a limit of CPU seconds, and the entity's texts take more than one.

use bedrock_addon::families::entity::history;

/// The steps of the entity's chain, each run alone.
fn runs() -> Vec<StepRun> {
    steps![LoadContext::detached;
        history::v1_8_0::Entity,
        history::v1_10_0::Entity,
        history::v1_11_0::Entity,
        history::v1_12_0::Entity,
        history::v1_13_0::Entity,
        history::v1_15_0::Entity,
        history::v1_16_0::Entity,
        history::v1_16_100::Entity,
        history::v1_16_200::Entity,
        history::v1_17_10::Entity,
        history::v1_17_20::Entity,
        history::v1_18_0::Entity,
        history::v1_18_10::Entity,
        history::v1_18_20::Entity,
        history::v1_18_30::Entity,
        history::v1_19_0::Entity,
        history::v1_19_50::Entity,
        history::v1_19_60::Entity,
        history::v1_20_0::Entity,
        history::v1_20_60::Entity,
        history::v1_20_70::Entity,
        history::v1_20_80::Entity,
        history::v1_21_0::Entity,
        history::v1_21_10::Entity,
        history::v1_21_20::Entity,
        history::v1_21_40::Entity,
        history::v1_21_60::Entity,
        history::v1_21_70::Entity,
        history::v1_21_80::Entity,
        history::v1_21_89::Entity,
        history::v1_21_90::Entity,
        history::v1_21_100::Entity,
        history::v1_21_110::Entity,
        history::v1_21_120::Entity,
        history::v1_21_130::Entity,
        history::v1_26_0::Entity,
        history::v1_26_10::Entity,
        history::v1_26_20::Entity,
        history::v1_26_30::Entity,
        history::v1_26_40::Entity,
    ]
}

/// One test of the entity's documents of the versions from the first up to and without the
/// second.
macro_rules! entity_band {
    ($test:ident, $from:expr, $below:expr) => {
        legacy_families! {
            $test = "entity" in ($from, $below):
                ["entity"], ["minecraft:entity"], [
                ("a pack", bedrock_addon::families::entity::Entity, "entities/x.json", LoadContext::detached()),
                ("the base game", bedrock_addon::families::entity::Entity, "entities/x.json", base_game()),
            ], runs();
        }
    };
}
