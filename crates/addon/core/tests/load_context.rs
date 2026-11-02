//! The per-pack load context (including a detached one) and the load requirements
//! components and members are gated on.

use bedrock_addon_core::load::{
    Experiment, Experiments, LoadContext, LoadOptions, LoadRequirement,
};
use bedrock_addon_core::molang::{ExperimentMask, MolangVersion, RawVersion};
use bedrock_addon_core::pack_identity::PackType;
use bedrock_addon_core::version::{BaseGameVersion, ENGINE_VERSION, MinEngineVersion, SemVersion};
use uuid::Uuid;

/// The game's hard-coded allowlist of packs that satisfy the legacy creator features requirement.
const LEGACY_CREATOR_FEATURES_PACKS: [&str; 10] = [
    "da09e399-6f5e-4287-a013-4cf13de32166",
    "168d15a6-78cd-4133-ad85-ecc24a0ad6db",
    "955c2b94-306a-48df-bfc6-22f90d951d25",
    "6d63491a-ea68-446f-bf1a-4185375db7d7",
    "29a9003d-4e59-4a1c-87c9-dea2f8988623",
    "27f9ef69-0dc9-48ba-a857-f9b8c1d0df5b",
    "6c4a4939-9f5a-45ad-bf93-013ed60e47c6",
    "b4a0f58d-fccb-4198-86f5-b44a4fb162cd",
    "81e5d014-0f03-4cd0-aef0-ba6914f6ac2f",
    "185f05da-1408-4070-b3cf-75a0baa17d8f",
];

fn engine_semver() -> SemVersion {
    SemVersion::new(
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch,
    )
}

#[test]
fn detached_context_values() {
    let cx = LoadContext::detached();
    assert_eq!(cx.engine_version, ENGINE_VERSION);
    let engine = engine_semver();
    assert_eq!(
        cx.min_engine_version.as_ref().map(|v| &v.version),
        Some(&engine)
    );
    // The Molang version of the engine: 1.26.50 is past the last threshold (1.21.100 → 13).
    assert_eq!(cx.molang_version, RawVersion::from(MolangVersion::LATEST));
    assert!(cx.experiments.is_empty());
    assert_eq!(cx.molang_experiments, ExperimentMask::empty());
    assert!(!cx.is_base_game_pack);
    assert!(!cx.is_trusted_pack);
    assert!(!cx.beta_apis);
    assert!(!cx.upcoming_creator_features);
    assert_eq!(cx.pack_uuid, Uuid::nil());
    assert_eq!(cx.pack_type, PackType::Invalid);
    assert_eq!(cx.world_base_game_version, BaseGameVersion::new(engine));
    assert!(cx.client_render_identifiers.is_none());
}

#[test]
fn a_pack_context_takes_the_world_options_and_the_pack_identity() {
    let uuid = Uuid::from_u128(0x1234);
    let options = LoadOptions {
        experiments: [Experiment::Poi].into_iter().collect(),
        upcoming_creator_features: true,
        base_game_version: BaseGameVersion::new(SemVersion::new(1, 21, 40)),
        ..LoadOptions::default()
    };
    let cx = LoadContext::for_pack(
        &options,
        uuid,
        PackType::Behavior,
        Some(MinEngineVersion::new(1, 20, 10)),
        true,
    );
    assert_eq!(cx.pack_uuid, uuid);
    assert_eq!(cx.pack_type, PackType::Behavior);
    assert!(cx.is_base_game_pack);
    assert!(cx.experiments.contains(Experiment::Poi));
    assert!(cx.upcoming_creator_features);
    // The toggle and its experiment are one setting: either sets both.
    assert!(cx.experiments.contains(Experiment::UpcomingCreatorFeatures));
    assert!(!cx.beta_apis);
    assert_eq!(
        cx.world_base_game_version,
        BaseGameVersion::new(SemVersion::new(1, 21, 40))
    );
    // 1.20.10 maps to Molang version 9.
    assert_eq!(cx.molang_version, RawVersion::from(MolangVersion::V9));
    assert_eq!(cx.molang_experiments, cx.experiments.molang_mask());
    assert_eq!(cx.engine_version, ENGINE_VERSION);
}

#[test]
fn the_beta_apis_experiment_turns_on_the_toggle() {
    let options = LoadOptions {
        experiments: [Experiment::BetaApis].into_iter().collect(),
        ..LoadOptions::default()
    };
    let cx = LoadContext::for_pack(&options, Uuid::nil(), PackType::Resource, None, false);
    assert!(cx.beta_apis);
    let options = LoadOptions {
        beta_apis: true,
        ..LoadOptions::default()
    };
    let cx = LoadContext::for_pack(&options, Uuid::nil(), PackType::Resource, None, false);
    assert!(cx.experiments.contains(Experiment::BetaApis));
}

#[test]
fn a_pack_without_min_engine_version_has_the_invalid_molang_version() {
    let cx = LoadContext::for_pack(
        &LoadOptions::default(),
        Uuid::nil(),
        PackType::Resource,
        None,
        false,
    );
    assert_eq!(cx.min_engine_version, None);
    assert_eq!(cx.molang_version, RawVersion::from(MolangVersion::Invalid));
}

#[test]
fn an_experiment_requirement_needs_its_experiment() {
    let requirement = LoadRequirement::Experiment(Experiment::Poi);
    let mut cx = LoadContext::detached();
    assert!(!requirement.satisfied(&cx));
    cx.experiments.insert(Experiment::Poi);
    assert!(requirement.satisfied(&cx));
}

#[test]
fn the_toggles_satisfy_their_experiment_requirements() {
    let mut cx = LoadContext::detached();
    cx.beta_apis = true;
    assert!(LoadRequirement::Experiment(Experiment::BetaApis).satisfied(&cx));
    assert!(!LoadRequirement::Experiment(Experiment::UpcomingCreatorFeatures).satisfied(&cx));
    cx.upcoming_creator_features = true;
    assert!(LoadRequirement::Experiment(Experiment::UpcomingCreatorFeatures).satisfied(&cx));
}

#[test]
fn the_base_game_requirement_needs_a_base_game_pack() {
    let mut cx = LoadContext::detached();
    assert!(!LoadRequirement::BaseGamePack.satisfied(&cx));
    cx.is_base_game_pack = true;
    assert!(LoadRequirement::BaseGamePack.satisfied(&cx));
}

#[test]
fn the_beta_features_requirement_needs_beta_apis_or_a_base_game_pack() {
    let mut cx = LoadContext::detached();
    assert!(!LoadRequirement::BetaFeatures.satisfied(&cx));
    cx.beta_apis = true;
    assert!(LoadRequirement::BetaFeatures.satisfied(&cx));
    let mut cx = LoadContext::detached();
    cx.is_base_game_pack = true;
    assert!(LoadRequirement::BetaFeatures.satisfied(&cx));
}

#[test]
fn legacy_creator_features_needs_one_of_the_ten_packs() {
    let mut cx = LoadContext::detached();
    for text in LEGACY_CREATOR_FEATURES_PACKS {
        cx.pack_uuid = Uuid::parse_str(text).unwrap();
        assert!(
            LoadRequirement::LegacyCreatorFeatures.satisfied(&cx),
            "{text}"
        );
    }
    // No base-game or beta shortcut.
    cx.pack_uuid = Uuid::from_u128(1);
    cx.is_base_game_pack = true;
    cx.beta_apis = true;
    assert!(!LoadRequirement::LegacyCreatorFeatures.satisfied(&cx));
}

#[test]
fn requirements_are_const_constructible() {
    const GATES: &[LoadRequirement] = &[
        LoadRequirement::Experiment(Experiment::VoxelShapes),
        LoadRequirement::BaseGamePack,
    ];
    assert_eq!(GATES.len(), 2);
}

#[test]
fn experiments_on_the_options_are_shared_with_the_context() {
    let experiments: Experiments = [Experiment::VoxelShapes].into_iter().collect();
    let options = LoadOptions {
        experiments: experiments.clone(),
        ..LoadOptions::default()
    };
    let cx = LoadContext::for_pack(&options, Uuid::nil(), PackType::Behavior, None, false);
    assert_eq!(cx.experiments, experiments);
}
