#![allow(clippy::pedantic, missing_docs)]
//! Experiments by save name and the set that keeps unknown names.

use bedrock_addon_core::load::{Experiment, Experiments, SaveName};
use bedrock_addon_core::molang::{Experiment as MolangExperiment, ExperimentMask};

/// Every row: the crate's variant and its save name (`None` for the one without), in table
/// order, which is the discriminant order.
const TABLE: &[(Experiment, Option<&str>)] = &[
    (Experiment::TestExperiment, Some("test_experiment")),
    (Experiment::NextUpdate, Some("next_update")),
    (Experiment::DisabledExperiment, None),
    (Experiment::ExperimentalText, Some("experimental_text")),
    (Experiment::AllowSeedChange, Some("allow_seed_change")),
    (
        Experiment::UpcomingCreatorFeatures,
        Some("upcoming_creator_features"),
    ),
    (Experiment::BetaApis, Some("gametest")),
    (
        Experiment::DataDrivenVanillaBlocksAndItems,
        Some("data_driven_vanilla_blocks_and_items"),
    ),
    (
        Experiment::DisableDataDrivenVanillaBlocksAndItems,
        Some("disable_data_driven_vanilla_blocks_and_items"),
    ),
    (
        Experiment::ExperimentalCreatorCameras,
        Some("experimental_creator_cameras"),
    ),
    (Experiment::MinecraftExplorer, Some("minecraft_explorer")),
    (
        Experiment::DeferredTechnicalPreview,
        Some("deferred_technical_preview"),
    ),
    (
        Experiment::VillagerTradesRebalance,
        Some("villager_trades_rebalance"),
    ),
    (
        Experiment::VanillaBlockGeometry,
        Some("vanilla_block_geometry"),
    ),
    (
        Experiment::SimplifiedSpawnRules,
        Some("simplified_spawn_rules"),
    ),
    (Experiment::VoxelShapes, Some("voxel_shapes")),
    (Experiment::Poi, Some("poi")),
    (Experiment::BrainPrototype, Some("brain_prototype")),
    (Experiment::FurnaceRecipeBook, Some("furnace_recipe_book")),
    (Experiment::CustomProjectiles, Some("custom_projectiles")),
    (Experiment::Y2026Drop2, Some("y_2026_drop_2")),
    (Experiment::Lwe, Some("lwe")),
    (Experiment::GameplayUi, Some("gameplay_ui")),
    (
        Experiment::GameEventSystemRefactor,
        Some("game_event_system_refactor"),
    ),
    (Experiment::OffhandParity, Some("offhand_parity")),
    (Experiment::Y2026Drop3, Some("y_2026_drop_3")),
    (Experiment::Mao, Some("mao")),
    (Experiment::CreatorWorldClocks, Some("creator_world_clocks")),
    // The newest experiment.
    (Experiment::Y2026Drop4, Some("y_2026_drop_4")),
];

/// The deprecated save names: the 1.26.20 list plus the three that later releases added.
const DEPRECATED: [&str; 23] = [
    "caves_and_cliffs",
    "wild_update",
    "spectator_mode",
    "vanilla_experiments",
    "bundles",
    "data_driven_items",
    "data_driven_biomes",
    "scripting",
    "experimental_molang_features",
    "next_major_update",
    "sniffer",
    "updateAnnouncedLive2023",
    "armadillo",
    "jigsaw_structures",
    "camera_aim_assist",
    "y_2025_drop_1",
    "y_2025_drop_2",
    "locator_bar",
    "y_2025_drop_3",
    "y_2026_drop_1",
    "furnace_recipe_book",
    "custom_projectiles",
    "y_2026_drop_2",
];

#[test]
fn every_row_of_the_table_has_its_save_name() {
    assert_eq!(
        TABLE.len(),
        29,
        "the table has 28 older experiments and y_2026_drop_4"
    );
    for (experiment, save_name) in TABLE {
        assert_eq!(experiment.save_name(), *save_name, "{experiment:?}");
        if let Some(name) = save_name {
            assert_eq!(
                Experiment::from_save_name(name),
                Some(*experiment),
                "{name}"
            );
        }
    }
}

#[test]
fn all_lists_the_table_in_discriminant_order() {
    let all: Vec<Experiment> = TABLE.iter().map(|(experiment, _)| *experiment).collect();
    assert_eq!(Experiment::ALL, all.as_slice());
    for (index, experiment) in Experiment::ALL.iter().enumerate() {
        assert_eq!(usize::from(experiment.bit()), index, "{experiment:?}");
    }
}

#[test]
fn beta_apis_saves_as_gametest_not_beta_apis() {
    assert_eq!(Experiment::BetaApis.save_name(), Some("gametest"));
    assert_eq!(Experiment::from_save_name("beta_apis"), None);
    assert_eq!(Experiment::from_save_name("BetaApis"), None);
}

#[test]
fn save_names_are_case_sensitive() {
    assert_eq!(Experiment::from_save_name("Poi"), None);
    assert_eq!(Experiment::from_save_name("GAMETEST"), None);
}

#[test]
fn every_deprecated_name_is_listed() {
    assert_eq!(Experiment::DEPRECATED_NAMES, DEPRECATED);
}

#[test]
fn a_known_save_name_sets_its_bit() {
    let mut experiments = Experiments::new();
    assert_eq!(
        experiments.insert_save_name("gametest"),
        SaveName::Known(Experiment::BetaApis)
    );
    assert!(experiments.contains(Experiment::BetaApis));
    assert!(experiments.unknown().is_empty());
}

#[test]
fn a_deprecated_name_is_kept_as_unknown_without_a_lint() {
    let mut experiments = Experiments::new();
    assert_eq!(
        experiments.insert_save_name("caves_and_cliffs"),
        SaveName::Deprecated
    );
    assert_eq!(experiments.unknown(), ["caves_and_cliffs"]);
    assert!(experiments.iter().next().is_none());
}

#[test]
fn a_deprecated_name_goes_to_unknown_though_1_26_20_had_the_experiment() {
    // `furnace_recipe_book`, `custom_projectiles` and `y_2026_drop_2` were 1.26.20 toggles
    // and are deprecated names from 1.26.45 on; a deprecated name goes to `unknown`. The table
    // still names their variants.
    let mut experiments = Experiments::new();
    for name in ["furnace_recipe_book", "custom_projectiles", "y_2026_drop_2"] {
        assert_eq!(
            experiments.insert_save_name(name),
            SaveName::Deprecated,
            "{name}"
        );
    }
    assert!(experiments.iter().next().is_none());
    assert_eq!(
        experiments.unknown(),
        ["furnace_recipe_book", "custom_projectiles", "y_2026_drop_2"]
    );
    assert_eq!(
        Experiment::from_save_name("furnace_recipe_book"),
        Some(Experiment::FurnaceRecipeBook)
    );
}

#[test]
fn an_unknown_name_is_kept_once() {
    let mut experiments = Experiments::new();
    assert_eq!(
        experiments.insert_save_name("y_2027_drop_1"),
        SaveName::Unknown
    );
    assert_eq!(
        experiments.insert_save_name("y_2027_drop_1"),
        SaveName::Unknown
    );
    assert_eq!(experiments.unknown(), ["y_2027_drop_1"]);
}

#[test]
fn insert_remove_and_iterate_in_discriminant_order() {
    let mut experiments: Experiments = [Experiment::Poi, Experiment::NextUpdate]
        .into_iter()
        .collect();
    assert!(!experiments.insert(Experiment::Poi), "already present");
    assert!(experiments.insert(Experiment::CreatorWorldClocks));
    assert_eq!(
        experiments.iter().collect::<Vec<_>>(),
        [
            Experiment::NextUpdate,
            Experiment::Poi,
            Experiment::CreatorWorldClocks
        ]
    );
    assert!(experiments.remove(Experiment::NextUpdate));
    assert!(!experiments.remove(Experiment::NextUpdate));
    assert!(!experiments.contains(Experiment::NextUpdate));
    assert!(!experiments.is_empty());
    assert!(Experiments::default().is_empty());
}

#[test]
fn the_molang_mask_numbers_experiments_by_our_discriminant() {
    let experiments: Experiments = [Experiment::NextUpdate, Experiment::Lwe]
        .into_iter()
        .collect();
    let expected: ExperimentMask = [1u8, 21]
        .into_iter()
        .filter_map(MolangExperiment::new)
        .collect();
    assert_eq!(experiments.molang_mask(), expected);
    assert_eq!(experiments.molang_mask().bits(), (1 << 1) | (1 << 21));
}

/// The game knows `y_2026_drop_4`, the experiment of the vanilla pack
/// `experimental_y_2026_drop_4`; a world that saves it turns it on.
#[test]
fn the_y_2026_save_name_is_an_experiment() {
    let mut experiments = Experiments::new();
    let SaveName::Known(experiment) = experiments.insert_save_name("y_2026_drop_4") else {
        panic!("y_2026_drop_4 is no experiment's save name");
    };
    assert_eq!(experiment.save_name(), Some("y_2026_drop_4"));
    assert!(experiments.contains(experiment));
    assert!(experiments.unknown().is_empty());
}
