//! The app-bundled packs the base-game layer is built from:
//! the experimental packs by UUID, the script libraries, the `vanilla` behaviour pack and its
//! versioned slices.

use bedrock_addon_core::load::Experiment;
use bedrock_addon_core::pack_identity::PackType;
use bedrock_addon_core::version::SemVersion;
use uuid::Uuid;

/// The vanilla experimental packs: header UUID, pack type and the experiment that adds the pack
/// to the base-game stack (the experimental packs of the released game).
///
/// The behaviour packs are listed in the order the server's base-game stack pushes them; such a
/// pack is added only while its experiment is on, above the script libraries, the `vanilla`
/// slices and `vanilla`. A third-party pack with one of these UUIDs is an ordinary pack.
// The UUIDs are the ones in the packs' own manifests, lower-cased. The behaviour packs follow the
// order in which the game's server pushes them onto its base-game stack. The `Y2026Drop3`
// behaviour pack is last: it is pushed after the voxel shapes pack, and no experiment between
// the two can be switched on.
pub static VANILLA_EXPERIMENTAL_PACKS: &[(Uuid, PackType, Experiment)] = &[
    (
        Uuid::from_u128(0x47db1a44_55b4_4d4e_aa0f_d5de6206a0fd),
        PackType::Behavior,
        Experiment::NextUpdate,
    ),
    (
        Uuid::from_u128(0x05112080_727b_4ed4_98d4_b099fc75f2be),
        PackType::Behavior,
        Experiment::ExperimentalCreatorCameras,
    ),
    (
        Uuid::from_u128(0x73a34f28_7c43_4b33_baaf_c9bcdcb0909d),
        PackType::Behavior,
        Experiment::MinecraftExplorer,
    ),
    (
        Uuid::from_u128(0x0127bd67_dc5b_45c2_8d5b_1d65876d5679),
        PackType::Behavior,
        Experiment::VillagerTradesRebalance,
    ),
    (
        Uuid::from_u128(0xe3ab7251_987f_4367_bd92_02c66d77cdf5),
        PackType::Behavior,
        Experiment::SimplifiedSpawnRules,
    ),
    (
        Uuid::from_u128(0x0ffb286d_6ee3_4243_a640_b46898344a8e),
        PackType::Behavior,
        Experiment::VoxelShapes,
    ),
    (
        Uuid::from_u128(0x32dd7cc5_e6fb_46f3_ace9_15aa250989a6),
        PackType::Behavior,
        Experiment::Poi,
    ),
    // The `Y2026Drop4` behaviour pack is pushed right after the POI pack; its UUID is the one in
    // its shipped manifest.
    (
        Uuid::from_u128(0x463a758b_b207_41d2_88ed_50b4bc034e03),
        PackType::Behavior,
        Experiment::Y2026Drop4,
    ),
    (
        Uuid::from_u128(0xc5b9eaf5_41cc_4a37_838b_3f6ef6fdf7e3),
        PackType::Behavior,
        Experiment::BrainPrototype,
    ),
    (
        Uuid::from_u128(0xd3eb1863_37f7_4a41_af0f_a52d4a1968f4),
        PackType::Behavior,
        Experiment::Y2026Drop2,
    ),
    (
        Uuid::from_u128(0x56c148f6_3933_4531_81a0_6e93021c3fd9),
        PackType::Behavior,
        Experiment::Lwe,
    ),
    (
        Uuid::from_u128(0x19f86644_3c8e_426e_a28a_cc9847cf60c4),
        PackType::Behavior,
        Experiment::Y2026Drop3,
    ),
    (
        Uuid::from_u128(0x5b99922d_319c_49dd_99e6_015ae77f7a8d),
        PackType::Resource,
        Experiment::ExperimentalText,
    ),
    (
        Uuid::from_u128(0x3704747e_b27f_4cfd_98d6_4265be2c769c),
        PackType::Resource,
        Experiment::MinecraftExplorer,
    ),
    (
        Uuid::from_u128(0xe440ef15_118e_4ca4_a794_18cd786367a4),
        PackType::Resource,
        Experiment::DeferredTechnicalPreview,
    ),
    (
        Uuid::from_u128(0x4c29cbb6_f763_4787_b4f8_abab5e74a050),
        PackType::Resource,
        Experiment::VanillaBlockGeometry,
    ),
    (
        Uuid::from_u128(0x8a5c56fb_3850_456a_85d5_812f8de9d320),
        PackType::Resource,
        Experiment::Poi,
    ),
    // The `Y2026Drop4` resource pack comes right after the POI pack, with the UUID of its
    // shipped manifest.
    (
        Uuid::from_u128(0xf3d96f1e_fec7_4c4c_8c8a_f63d7ab39218),
        PackType::Resource,
        Experiment::Y2026Drop4,
    ),
    (
        Uuid::from_u128(0x40d23612_15ef_471d_9607_0ab5ec97bf5c),
        PackType::Resource,
        Experiment::BrainPrototype,
    ),
    (
        Uuid::from_u128(0xdd26f013_a1c5_46b1_9905_94016ac6d1e9),
        PackType::Resource,
        Experiment::FurnaceRecipeBook,
    ),
    (
        Uuid::from_u128(0xa33d0195_3037_4a57_b82b_541a8846b2c5),
        PackType::Resource,
        Experiment::Y2026Drop2,
    ),
    (
        Uuid::from_u128(0xbfafc2fc_55fa_4da0_9cb1_3ee6435d1879),
        PackType::Resource,
        Experiment::Lwe,
    ),
    (
        Uuid::from_u128(0x6df10650_43ae_4b61_be5c_b924a35d3414),
        PackType::Resource,
        Experiment::Y2026Drop3,
    ),
];

/// The version every app-bundled pack of the base-game layer is looked up at.
pub(crate) const IN_PACKAGE_VERSION: SemVersion = SemVersion::new(0, 0, 1);

/// `server_library` (`@minecraft/server` TS library), pushed unconditionally.
pub(crate) const SERVER_LIBRARY: Uuid = Uuid::from_u128(0x37ca4282_69e9_4402_b40b_0b07de579f02);

/// `server_ui_library`, pushed unconditionally.
pub(crate) const SERVER_UI_LIBRARY: Uuid = Uuid::from_u128(0xf53f490f_9d47_45ca_86de_0946b6d1c366);

/// The `vanilla` behaviour pack; its versioned slices (`vanilla_1.20.50`, …) share this UUID
/// with the slice version as their header version.
pub(crate) const VANILLA_BEHAVIOR: Uuid = Uuid::from_u128(0xfe9f8597_5454_481a_8730_8d070a8e2e58);
