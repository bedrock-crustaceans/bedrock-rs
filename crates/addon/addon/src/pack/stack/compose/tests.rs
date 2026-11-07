//! The 05 composition rules over manifest facts.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::load::{Experiment, Experiments};
use bedrock_addon_core::version::{BaseGameVersion, SemVersion};
use uuid::Uuid;

use super::*;
use bedrock_addon_core::diagnostic::DiagnosticDetail;

/// The packs a stack finding names, from its detail.
fn packs(finding: &Diagnostic) -> Option<(&PackIdVersion, Option<&PackIdVersion>)> {
    match finding.detail.as_deref()? {
        DiagnosticDetail::Packs { pack, dependency } => Some((pack, dependency.as_ref())),
        _ => None,
    }
}
use crate::pack::SubpackFacts;

/// A pack of `pack_type` with the UUID `n`, version 1.0.0, one module of its type.
fn pack(n: u128, pack_type: PackType) -> PackFacts {
    PackFacts {
        id: PackIdVersion::new(Uuid::from_u128(n), SemVersion::new(1, 0, 0), pack_type),
        name: format!("pack {n}").into(),
        description: None,
        capabilities: Vec::new(),
        module_types: vec![pack_type],
        dependencies: Vec::new(),
        subpacks: Vec::new(),
        min_engine_version: None,
    }
}

fn bp(n: u128) -> PackFacts {
    pack(n, PackType::Behavior)
}

fn rp(n: u128) -> PackFacts {
    pack(n, PackType::Resource)
}

/// An app-bundled behaviour pack at `uuid` and `version`.
fn bundled(uuid: Uuid, version: SemVersion) -> PackFacts {
    PackFacts {
        id: PackIdVersion::new(uuid, version, PackType::Behavior),
        ..bp(0)
    }
}

fn depends(mut facts: PackFacts, on: &[&PackFacts]) -> PackFacts {
    facts
        .dependencies
        .extend(on.iter().map(|dependency| dependency.id.clone()));
    facts
}

fn with_subpacks(mut facts: PackFacts, subpacks: &[(&str, &str, u8)]) -> PackFacts {
    facts.subpacks = subpacks
        .iter()
        .map(|(folder_name, name, memory_tier)| SubpackFacts {
            folder_name: (*folder_name).into(),
            name: (*name).into(),
            memory_tier: *memory_tier,
        })
        .collect();
    facts
}

fn select(facts: &PackFacts) -> PackSelection {
    PackSelection {
        pack_id: facts.id.uuid,
        version: facts.id.version.clone(),
        subpack: None,
    }
}

fn select_subpack(facts: &PackFacts, subpack: &str) -> PackSelection {
    PackSelection {
        subpack: Some(subpack.to_owned()),
        ..select(facts)
    }
}

fn world(behavior: Vec<PackSelection>, resource: Vec<PackSelection>) -> WorldPacks {
    WorldPacks { behavior, resource }
}

/// Composes with every pack registered at `origin` `Dev`.
fn run(packs: &[&PackFacts], world: &WorldPacks, options: &LoadOptions) -> Composition {
    let candidates: Vec<Candidate<'_>> = packs
        .iter()
        .map(|facts| Candidate {
            facts,
            origin: PackOrigin::Dev,
        })
        .collect();
    compose(&candidates, world, options)
}

fn run_with_origins(
    packs: &[(&PackFacts, PackOrigin)],
    world: &WorldPacks,
    options: &LoadOptions,
) -> Composition {
    let candidates: Vec<Candidate<'_>> = packs
        .iter()
        .map(|(facts, origin)| Candidate {
            facts,
            origin: *origin,
        })
        .collect();
    compose(&candidates, world, options)
}

/// The candidate indices of a stack, top first.
fn order(stack: &[Planned]) -> Vec<usize> {
    stack.iter().map(|planned| planned.candidate).collect()
}

/// The findings the world's lists and the composition raise, base-game layer's missing vanilla
/// packs (which carry no place) left out.
fn placed_findings(composition: &Composition) -> Vec<&Diagnostic> {
    composition
        .findings
        .iter()
        .filter(|diagnostic| {
            !(diagnostic.is(&core_codes::PACK_NOT_FOUND) && diagnostic.location.path.is_none())
        })
        .collect()
}

/// [`placed_findings`] without the missing dependencies: the composition's own findings, all
/// informations, since the game logs none of them.
fn level_findings(composition: &Composition) -> Vec<&Diagnostic> {
    placed_findings(composition)
        .into_iter()
        .filter(|diagnostic| !diagnostic.is(&core_codes::MISSING_DEPENDENCY))
        .inspect(|diagnostic| assert_eq!(diagnostic.severity, Severity::Info, "{diagnostic:?}"))
        .collect()
}

/// [`placed_findings`]' codes.
fn above_info(composition: &Composition) -> Vec<&Code> {
    placed_findings(composition)
        .into_iter()
        // A dependency the education layer hides silently is an information.
        .filter(|diagnostic| {
            !(diagnostic.is(&core_codes::MISSING_DEPENDENCY)
                && diagnostic.severity == Severity::Info)
        })
        .map(|diagnostic| &diagnostic.code)
        .collect()
}

#[test]
fn a_world_list_is_kept_top_first() {
    let (a, b, c) = (bp(1), bp(2), bp(3));
    let composition = run(
        &[&a, &b, &c],
        &world(vec![select(&c), select(&a), select(&b)], vec![]),
        &LoadOptions::default(),
    );
    assert_eq!(order(&composition.behavior), [2, 0, 1]);
    assert!(above_info(&composition).is_empty());
}

#[test]
fn dependencies_follow_their_dependent_depth_first() {
    let (leaf, sibling) = (bp(3), bp(4));
    let middle = depends(bp(2), &[&leaf]);
    let top = depends(bp(1), &[&middle, &sibling]);
    let other = bp(5);
    let composition = run(
        &[&top, &middle, &leaf, &sibling, &other],
        &world(vec![select(&top), select(&other)], vec![]),
        &LoadOptions::default(),
    );
    assert_eq!(order(&composition.behavior), [0, 1, 2, 3, 4]);
    let dependent: Vec<bool> = composition
        .behavior
        .iter()
        .map(|planned| planned.is_dependent)
        .collect();
    assert_eq!(dependent, [false, true, true, true, false]);
}

#[test]
fn a_dependency_cycle_adds_each_pack_once() {
    let a_id = bp(1);
    let b = depends(bp(2), &[&a_id]);
    let a = depends(bp(1), &[&b]);
    let composition = run(
        &[&a, &b],
        &world(vec![select(&a)], vec![]),
        &LoadOptions::default(),
    );
    assert_eq!(order(&composition.behavior), [0, 1]);
}

#[test]
fn a_missing_dependency_hides_the_pack_and_is_an_error() {
    let missing = bp(9);
    let a = depends(bp(1), &[&missing]);
    let b = bp(2);
    let composition = run(
        &[&a, &b],
        &world(vec![select(&a), select(&b)], vec![]),
        &LoadOptions::default(),
    );
    assert_eq!(order(&composition.behavior), [0, 1]);
    assert!(composition.behavior[0].hidden);
    assert!(!composition.behavior[1].hidden);
    let errors: Vec<&Diagnostic> = composition.findings.at_least(Severity::Error).collect();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].is(&core_codes::MISSING_DEPENDENCY));
    assert_eq!(packs(errors[0]), Some((&a.id, Some(&missing.id))));
}

#[test]
fn a_behavior_packs_resource_dependency_is_auto_added_after_the_listed_resource_packs() {
    let r = rp(10);
    let q = rp(11);
    let a = depends(bp(1), &[&r]);
    let composition = run(
        &[&a, &r, &q],
        &world(vec![select(&a)], vec![select(&q)]),
        &LoadOptions::default(),
    );
    assert_eq!(order(&composition.behavior), [0]);
    assert!(!composition.behavior[0].hidden);
    assert_eq!(order(&composition.resource), [2, 1]);
    assert!(composition.resource[1].is_dependent);
    let found = above_info(&composition);
    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0],
        &core_codes::DEPENDENCY_AUTO_ADDED.with([r.id.to_string(), a.id.to_string()])
    );
    let added = composition
        .findings
        .iter()
        .find(|diagnostic| diagnostic.is(&core_codes::DEPENDENCY_AUTO_ADDED))
        .and_then(packs);
    assert_eq!(added, Some((&a.id, Some(&r.id))));
    let warning = composition
        .findings
        .iter()
        .find(|diagnostic| diagnostic.is(&core_codes::DEPENDENCY_AUTO_ADDED));
    assert_eq!(
        warning.map(|diagnostic| diagnostic.severity),
        Some(Severity::Info)
    );
}

#[test]
fn a_resource_dependency_the_world_already_lists_adds_nothing() {
    let r = rp(10);
    let a = depends(bp(1), &[&r]);
    let composition = run(
        &[&a, &r],
        &world(vec![select(&a)], vec![select(&r)]),
        &LoadOptions::default(),
    );
    assert_eq!(order(&composition.resource), [1]);
    assert!(above_info(&composition).is_empty());
}

#[test]
fn an_absent_resource_dependency_hides_the_behavior_pack() {
    let r = rp(10);
    let a = depends(bp(1), &[&r]);
    let composition = run(
        &[&a],
        &world(vec![select(&a)], vec![]),
        &LoadOptions::default(),
    );
    assert!(composition.behavior[0].hidden);
    assert!(composition.resource.is_empty());
    assert_eq!(
        above_info(&composition)
            .iter()
            .map(|code| code.is(&core_codes::MISSING_DEPENDENCY))
            .collect::<Vec<_>>(),
        [true]
    );
}

#[test]
fn a_resource_packs_behavior_dependency_is_satisfied_by_registration() {
    let b = bp(1);
    let r = depends(rp(10), &[&b]);
    let composition = run(
        &[&b, &r],
        &world(vec![], vec![select(&r)]),
        &LoadOptions::default(),
    );
    assert_eq!(order(&composition.resource), [1]);
    assert!(!composition.resource[0].hidden);
    assert!(composition.behavior.is_empty());
}

#[test]
fn an_unregistered_world_entry_is_skipped_with_an_information() {
    let (a, ghost) = (bp(1), bp(7));
    let composition = run(
        &[&a],
        &world(vec![select(&ghost), select(&a)], vec![]),
        &LoadOptions::default(),
    );
    assert_eq!(order(&composition.behavior), [0]);
    let found = level_findings(&composition);
    assert_eq!(found.len(), 1);
    assert!(found[0].is(&core_codes::PACK_NOT_FOUND));
    assert_eq!(packs(found[0]), Some((&ghost.id, None)));
    assert_eq!(
        found[0].location.path.as_ref().map(PackPath::as_str),
        Some(WorldPacks::BEHAVIOR_FILE)
    );
    assert_eq!(found[0].location.json_path.to_string(), "0");
}

#[test]
fn a_repeated_world_entry_keeps_the_first_with_an_information() {
    let (a, b) = (bp(1), bp(2));
    let composition = run(
        &[&a, &b],
        &world(vec![select(&a), select(&b), select(&a)], vec![]),
        &LoadOptions::default(),
    );
    assert_eq!(order(&composition.behavior), [0, 1]);
    let found = level_findings(&composition);
    assert_eq!(found.len(), 1);
    assert!(found[0].is(&core_codes::DUPLICATE_IDENTIFIER));
    assert_eq!(found[0].location.json_path.to_string(), "2");
}

#[test]
fn listing_a_dependency_as_well_keeps_the_first_with_an_information() {
    let b = bp(2);
    let a = depends(bp(1), &[&b]);
    let composition = run(
        &[&a, &b],
        &world(vec![select(&a), select(&b)], vec![]),
        &LoadOptions::default(),
    );
    assert_eq!(order(&composition.behavior), [0, 1]);
    assert!(composition.behavior[1].is_dependent);
    let found = level_findings(&composition);
    assert!(matches!(
        found[..],
        [d] if d.is(&core_codes::DUPLICATE_IDENTIFIER)
    ));
    assert_eq!(found[0].location.json_path.to_string(), "1");
}

#[test]
fn a_base_game_pack_already_pulled_in_as_a_dependency_is_a_duplicate() {
    let library = in_package(SERVER_LIBRARY);
    let a = depends(bp(1), &[&library]);
    let composition = run_with_origins(
        &[(&a, PackOrigin::World), (&library, PackOrigin::Package)],
        &world(vec![select(&a)], vec![]),
        &LoadOptions::default(),
    );
    assert_eq!(order(&composition.behavior), [0, 1]);
    assert_eq!(
        above_info(&composition)
            .iter()
            .map(|code| code.is(&core_codes::DUPLICATE_IDENTIFIER))
            .collect::<Vec<_>>(),
        [true]
    );
}

#[test]
fn a_hidden_behavior_pack_adds_no_resource_pack() {
    let r = rp(10);
    let ghost = bp(9);
    let a = depends(bp(1), &[&r, &ghost]);
    let composition = run(
        &[&a, &r],
        &world(vec![select(&a)], vec![]),
        &LoadOptions::default(),
    );
    assert!(composition.behavior[0].hidden);
    assert!(composition.resource.is_empty());
    assert_eq!(
        above_info(&composition)
            .iter()
            .map(|code| code.is(&core_codes::MISSING_DEPENDENCY))
            .collect::<Vec<_>>(),
        [true]
    );
}

#[test]
fn a_pack_of_the_other_side_is_skipped_from_a_world_list() {
    let r = rp(10);
    let composition = run(
        &[&r],
        &world(vec![select(&r)], vec![]),
        &LoadOptions::default(),
    );
    assert!(composition.behavior.is_empty());
    let found = level_findings(&composition);
    assert!(matches!(found[..], [d] if d.is(&core_codes::INVALID_VALUE)));
    assert_eq!(found[0].location.json_path.to_string(), "0/pack_id");
}

#[test]
fn a_pack_with_both_modules_is_accepted_on_either_list() {
    let mut both = rp(5);
    both.module_types = vec![PackType::Behavior, PackType::Resource];
    let composition = run(
        &[&both],
        &world(vec![select(&both)], vec![select(&both)]),
        &LoadOptions::default(),
    );
    assert_eq!(order(&composition.behavior), [0]);
    assert_eq!(order(&composition.resource), [0]);
    assert!(above_info(&composition).is_empty());
}

#[test]
fn the_default_subpack_is_the_highest_tier_under_the_ceiling_ties_to_the_later() {
    let facts = with_subpacks(
        rp(1),
        &[("a", "A", 1), ("b", "B", 3), ("c", "C", 3), ("d", "D", 2)],
    );
    assert_eq!(default_subpack(&facts, None), Some(2));
    assert_eq!(default_subpack(&facts, Some(5)), Some(2));
    assert_eq!(default_subpack(&facts, Some(2)), Some(3));
    assert_eq!(default_subpack(&facts, Some(1)), Some(0));
    assert_eq!(default_subpack(&facts, Some(0)), None);
    assert_eq!(default_subpack(&rp(2), None), None);
}

#[test]
fn a_world_entry_activates_its_subpack_by_folder_name_or_name() {
    let r = with_subpacks(rp(1), &[("low_res", "Low", 0), ("high_res", "High", 4)]);
    let s = with_subpacks(rp(2), &[("low_res", "Low", 0), ("high_res", "High", 4)]);
    let t = with_subpacks(rp(3), &[("low_res", "Low", 0), ("high_res", "High", 4)]);
    let composition = run(
        &[&r, &s, &t],
        &world(
            vec![],
            vec![
                select_subpack(&r, "low_res"),
                select_subpack(&s, "Low"),
                select(&t),
            ],
        ),
        &LoadOptions::default(),
    );
    let subpacks: Vec<Option<usize>> = composition
        .resource
        .iter()
        .map(|planned| planned.subpack)
        .collect();
    assert_eq!(subpacks, [Some(0), Some(0), Some(1)]);
    assert!(above_info(&composition).is_empty());
}

#[test]
fn an_unknown_subpack_falls_back_to_the_default_with_an_information() {
    let r = with_subpacks(rp(1), &[("low_res", "Low", 0), ("high_res", "High", 4)]);
    let composition = run(
        &[&r],
        &world(vec![], vec![select_subpack(&r, "medium")]),
        &LoadOptions::default(),
    );
    assert_eq!(composition.resource[0].subpack, Some(1));
    let found = level_findings(&composition);
    assert!(matches!(found[..], [d] if d.is(&core_codes::INVALID_VALUE)));
    assert_eq!(
        found[0].location.path.as_ref().map(PackPath::as_str),
        Some(WorldPacks::RESOURCE_FILE)
    );
    assert_eq!(found[0].location.json_path.to_string(), "0/subpack");
}

fn in_package(uuid: Uuid) -> PackFacts {
    bundled(uuid, IN_PACKAGE_VERSION)
}

fn slice(major: u16, minor: u16, patch: u16) -> PackFacts {
    bundled(VANILLA_BEHAVIOR, SemVersion::new(major, minor, patch))
}

fn experimental(experiment: Experiment) -> PackFacts {
    let (uuid, _, _) = VANILLA_EXPERIMENTAL_PACKS
        .iter()
        .find(|(_, pack_type, e)| *pack_type == PackType::Behavior && *e == experiment)
        .expect("a behaviour pack for the experiment");
    in_package(*uuid)
}

fn options_with(experiments: &[Experiment], base_game_version: BaseGameVersion) -> LoadOptions {
    LoadOptions {
        experiments: experiments.iter().copied().collect::<Experiments>(),
        base_game_version,
        ..LoadOptions::default()
    }
}

#[test]
fn the_base_game_layer_sits_below_the_world_in_vanillas_order() {
    let world_pack = bp(1);
    let cameras = experimental(Experiment::ExperimentalCreatorCameras);
    let shapes = experimental(Experiment::VoxelShapes);
    let library = in_package(SERVER_LIBRARY);
    let ui_library = in_package(SERVER_UI_LIBRARY);
    let (s1_19, s1_20_50, s1_21) = (slice(1, 19, 0), slice(1, 20, 50), slice(1, 21, 0));
    let vanilla = in_package(VANILLA_BEHAVIOR);
    let packs = [
        &vanilla,
        &s1_21,
        &s1_19,
        &s1_20_50,
        &ui_library,
        &library,
        &shapes,
        &cameras,
    ];
    let mut registered: Vec<(&PackFacts, PackOrigin)> = vec![(&world_pack, PackOrigin::World)];
    registered.extend(packs.iter().map(|facts| (*facts, PackOrigin::Package)));
    let composition = run_with_origins(
        &registered,
        &world(vec![select(&world_pack)], vec![]),
        // Shapes before cameras in the experiment list: the order is the builder's, not the
        // world's.
        &options_with(
            &[
                Experiment::VoxelShapes,
                Experiment::ExperimentalCreatorCameras,
            ],
            BaseGameVersion::new(SemVersion::new(1, 20, 60)),
        ),
    );
    // world, cameras, shapes, server_library, server_ui_library, 1.20.50, 1.19.0, vanilla
    assert_eq!(order(&composition.behavior), [0, 8, 7, 6, 5, 4, 3, 1]);
    assert!(
        composition.findings.is_empty(),
        "{:?}",
        composition.findings
    );
}

// The Drop 4 behaviour pack is pushed right after the POI pack; the UUIDs are those of the
// shipped manifests (`experimental_y_2026_drop_4`).
#[test]
fn the_y_2026_drop_4_experiment_adds_its_shipped_pack_after_the_poi_pack() {
    let behaviour_uuid = Uuid::from_u128(0x463a758b_b207_41d2_88ed_50b4bc034e03);
    let resource_uuid = Uuid::from_u128(0xf3d96f1e_fec7_4c4c_8c8a_f63d7ab39218);
    let listed = |pack_type: PackType| {
        VANILLA_EXPERIMENTAL_PACKS
            .iter()
            .find(|(_, t, e)| *t == pack_type && *e == Experiment::Y2026Drop4)
            .map(|(uuid, _, _)| *uuid)
    };
    assert_eq!(
        listed(PackType::Behavior),
        Some(behaviour_uuid),
        "the Drop 4 behaviour pack"
    );
    assert_eq!(
        listed(PackType::Resource),
        Some(resource_uuid),
        "the Drop 4 resource pack"
    );
    let poi = experimental(Experiment::Poi);
    let drop_4 = in_package(behaviour_uuid);
    let registered = [(&drop_4, PackOrigin::Package), (&poi, PackOrigin::Package)];
    let composition = run_with_origins(
        &registered,
        &world(vec![], vec![]),
        &options_with(
            &[Experiment::Y2026Drop4, Experiment::Poi],
            BaseGameVersion::ANY,
        ),
    );
    assert_eq!(order(&composition.behavior), [1, 0]);
}

#[test]
fn every_slice_applies_to_an_any_or_unset_base_game_version() {
    let (s1_19, s1_21) = (slice(1, 19, 0), slice(1, 21, 0));
    let registered = [(&s1_19, PackOrigin::Package), (&s1_21, PackOrigin::Package)];
    for version in [BaseGameVersion::ANY, BaseGameVersion::EMPTY] {
        let composition = run_with_origins(
            &registered,
            &world(vec![], vec![]),
            &options_with(&[], version),
        );
        assert_eq!(order(&composition.behavior), [1, 0]);
    }
}

// An unparsable world value selects every slice, as `*` does.
#[test]
fn an_unparsable_world_base_game_version_selects_every_slice() {
    let (s1_19, s1_21) = (slice(1, 19, 0), slice(1, 21, 0));
    let registered = [(&s1_19, PackOrigin::Package), (&s1_21, PackOrigin::Package)];
    let unparsable = BaseGameVersion {
        version: SemVersion::parse("1.twenty"),
        never_compatible: false,
    };
    assert_eq!(unparsable.version, None, "the world value does not parse");
    assert_eq!(slice_selector(&unparsable), BaseGameVersion::ANY);
    let composition = run_with_origins(
        &registered,
        &world(vec![], vec![]),
        &options_with(&[], unparsable),
    );
    assert_eq!(order(&composition.behavior), [1, 0]);
}

// no version, which is 05's "invalid": every slice, where this test required none.
#[test]
fn a_never_compatible_world_base_game_version_selects_every_slice() {
    let (s1_19, s1_21) = (slice(1, 19, 0), slice(1, 21, 0));
    let registered = [(&s1_19, PackOrigin::Package), (&s1_21, PackOrigin::Package)];
    assert_eq!(
        slice_selector(&BaseGameVersion::INCOMPATIBLE),
        BaseGameVersion::ANY
    );
    let composition = run_with_origins(
        &registered,
        &world(vec![], vec![]),
        &options_with(&[], BaseGameVersion::INCOMPATIBLE),
    );
    assert_eq!(order(&composition.behavior), [1, 0]);
}

#[test]
fn a_slice_must_be_app_bundled() {
    let foreign = slice(1, 19, 0);
    let composition = run_with_origins(
        &[(&foreign, PackOrigin::World)],
        &world(vec![], vec![]),
        &options_with(&[], BaseGameVersion::ANY),
    );
    assert!(composition.behavior.is_empty());
}

#[test]
fn an_experimental_pack_needs_its_experiment() {
    let poi = experimental(Experiment::Poi);
    let off = run_with_origins(
        &[(&poi, PackOrigin::Package)],
        &world(vec![], vec![]),
        &LoadOptions::default(),
    );
    assert!(off.behavior.is_empty());
    let on = run_with_origins(
        &[(&poi, PackOrigin::Package)],
        &world(vec![], vec![]),
        &options_with(&[Experiment::Poi], BaseGameVersion::EMPTY),
    );
    assert_eq!(order(&on.behavior), [0]);
}

#[test]
fn a_missing_base_game_pack_is_skipped_with_an_info() {
    let composition = run(&[], &world(vec![], vec![]), &LoadOptions::default());
    assert!(composition.behavior.is_empty());
    let missing: Vec<&PackIdVersion> = composition
        .findings
        .iter()
        .map(|diagnostic| {
            assert_eq!(diagnostic.severity, Severity::Info);
            assert!(diagnostic.is(&core_codes::PACK_NOT_FOUND), "{diagnostic:?}");
            match packs(diagnostic) {
                Some((pack, None)) => pack,
                other => panic!("unexpected {other:?}"),
            }
        })
        .collect();
    let uuids: Vec<Uuid> = missing.iter().map(|id| id.uuid).collect();
    assert_eq!(uuids, [SERVER_LIBRARY, SERVER_UI_LIBRARY, VANILLA_BEHAVIOR]);
}

#[test]
fn the_vanilla_behavior_packs_resource_dependency_is_dropped() {
    let vanilla_rp = PackFacts {
        id: PackIdVersion::new(
            Uuid::from_u128(0x0575c61f_a5da_4b7f_9961_ffda2908861e),
            IN_PACKAGE_VERSION,
            PackType::Resource,
        ),
        ..rp(0)
    };
    let vanilla = depends(in_package(VANILLA_BEHAVIOR), &[&vanilla_rp]);
    let with_rp = run_with_origins(
        &[
            (&vanilla, PackOrigin::Package),
            (&vanilla_rp, PackOrigin::Package),
        ],
        &world(vec![], vec![]),
        &LoadOptions::default(),
    );
    assert_eq!(order(&with_rp.behavior), [0]);
    assert!(with_rp.resource.is_empty());
    assert!(above_info(&with_rp).is_empty());
    let without_rp = run_with_origins(
        &[(&vanilla, PackOrigin::Package)],
        &world(vec![], vec![]),
        &LoadOptions::default(),
    );
    assert!(!without_rp.behavior[0].hidden);
    assert!(above_info(&without_rp).is_empty());
}

#[test]
fn a_world_listed_vanilla_pack_stays_where_the_world_put_it() {
    let vanilla = in_package(VANILLA_BEHAVIOR);
    let a = bp(1);
    let composition = run_with_origins(
        &[(&vanilla, PackOrigin::Package), (&a, PackOrigin::World)],
        &world(vec![select(&vanilla), select(&a)], vec![]),
        &LoadOptions::default(),
    );
    assert_eq!(order(&composition.behavior), [0, 1]);
    assert_eq!(
        above_info(&composition)
            .iter()
            .map(|code| code.is(&core_codes::DUPLICATE_IDENTIFIER))
            .collect::<Vec<_>>(),
        [true]
    );
}

#[test]
fn the_first_registered_pack_of_an_identity_is_the_one_selected() {
    let world_copy = bp(1);
    let global_copy = bp(1);
    let composition = run_with_origins(
        &[
            (&world_copy, PackOrigin::World),
            (&global_copy, PackOrigin::User),
        ],
        &world(vec![select(&world_copy)], vec![]),
        &LoadOptions::default(),
    );
    assert_eq!(order(&composition.behavior), [0]);
}

/// A pack at `uuid` and `version` with one module of `pack_type`, depending on `on`.
fn versioned(n: u128, version: SemVersion, pack_type: PackType, on: &[PackIdVersion]) -> PackFacts {
    PackFacts {
        id: PackIdVersion::new(Uuid::from_u128(n), version, pack_type),
        dependencies: on.to_vec(),
        ..pack(n, pack_type)
    }
}

// The manifests of the game's `chemistry_<version>` packs. The
// resource slice's header is `[1, 20, 50]` and it depends on the behaviour slice's UUID at
// `[1, 0, 0]`; the behaviour slice's header is `[1, 20, 50]` too and it depends on the
// resource slice at `[1, 0, 0]`. A dependency is looked up by UUID and version, so neither
// finds the other: both are hidden, whatever the world lists. The unversioned `chemistry`
// pair has `[1, 0, 0]` on both sides and is found.
#[test]
fn a_dependency_at_another_version_than_the_packs_header_hides_the_pack() {
    let one = SemVersion::new(1, 0, 0);
    let slice = SemVersion::new(1, 20, 50);
    let at = |n: u128, pack_type| PackIdVersion::new(Uuid::from_u128(n), one.clone(), pack_type);
    let r = versioned(
        1,
        slice.clone(),
        PackType::Resource,
        &[at(2, PackType::Behavior)],
    );
    let b = versioned(2, slice, PackType::Behavior, &[at(1, PackType::Resource)]);
    let composition = run(
        &[&b, &r],
        &world(vec![select(&b)], vec![select(&r)]),
        &LoadOptions::default(),
    );
    assert!(composition.behavior.iter().all(|planned| planned.hidden));
    assert!(composition.resource.iter().all(|planned| planned.hidden));
    let missing: Vec<(Uuid, Uuid, SemVersion)> = composition
        .findings
        .iter()
        .filter(|d| d.is(&core_codes::MISSING_DEPENDENCY))
        .map(|d| {
            assert_eq!(d.severity, Severity::Error);
            let Some((pack, Some(dependency))) = packs(d) else {
                panic!("no packs: {d:?}");
            };
            (pack.uuid, dependency.uuid, dependency.version.clone())
        })
        .collect();
    assert_eq!(
        missing,
        [
            (Uuid::from_u128(2), Uuid::from_u128(1), one.clone()),
            (Uuid::from_u128(1), Uuid::from_u128(2), one.clone()),
        ]
    );
    // The unversioned pair: each header is the version the other asks for.
    let r = versioned(
        1,
        one.clone(),
        PackType::Resource,
        &[at(2, PackType::Behavior)],
    );
    let b = versioned(
        2,
        one.clone(),
        PackType::Behavior,
        &[at(1, PackType::Resource)],
    );
    let composition = run(
        &[&b, &r],
        &world(vec![select(&b)], vec![select(&r)]),
        &LoadOptions::default(),
    );
    assert!(composition.behavior.iter().all(|planned| !planned.hidden));
    assert!(composition.resource.iter().all(|planned| !planned.hidden));
    assert!(above_info(&composition).is_empty());
}

// The game does not serve a persona piece pack as a resource pack: it is skipped and reported as
// `PersonaPiecePack`.
#[test]
fn a_persona_piece_pack_on_the_resource_list_is_skipped_with_its_own_information() {
    let piece = pack(7, PackType::PersonaPiece);
    let composition = run(
        &[&piece],
        &world(vec![], vec![select(&piece)]),
        &LoadOptions::default(),
    );
    assert!(composition.resource.is_empty());
    let found = level_findings(&composition);
    assert!(
        matches!(found[..], [d] if d.is(&core_codes::PERSONA_PIECE_PACK)
            && d.severity == Severity::Info),
        "{found:?}"
    );
    assert_eq!(found[0].location.json_path.to_string(), "0/pack_id");
    // On the behaviour list it is a pack of another kind, as any other.
    let composition = run(
        &[&piece],
        &world(vec![select(&piece)], vec![]),
        &LoadOptions::default(),
    );
    let found = level_findings(&composition);
    assert!(
        matches!(found[..], [d] if d.is(&core_codes::INVALID_VALUE)),
        "{found:?}"
    );
}

// ---- the education layer ------------------------------------------------------------

/// A chemistry pack as the game ships it: app-bundled, `capabilities: ["chemistry"]`, header
/// `version`, depending on the other half at `on`.
fn chemistry(n: u128, version: SemVersion, pack_type: PackType, on: &[PackIdVersion]) -> PackFacts {
    PackFacts {
        capabilities: vec!["chemistry".into()],
        ..versioned(n, version, pack_type, on)
    }
}

/// The shipped pattern (`behavior_packs/chemistry`, `chemistry_1.20.50`, `chemistry_1.21.30` and their
/// resource halves): the unversioned pair depends on each other at 1.0.0; each slice depends on
/// the other half at 1.0.0 while that half's header is the slice's version.
fn chemistry_packs() -> Vec<PackFacts> {
    let one = SemVersion::new(1, 0, 0);
    let at = |n: u128, pack_type| PackIdVersion::new(Uuid::from_u128(n), one.clone(), pack_type);
    vec![
        chemistry(
            10,
            one.clone(),
            PackType::Behavior,
            &[at(20, PackType::Resource)],
        ),
        chemistry(
            11,
            SemVersion::new(1, 20, 50),
            PackType::Behavior,
            &[at(21, PackType::Resource)],
        ),
        chemistry(
            12,
            SemVersion::new(1, 21, 30),
            PackType::Behavior,
            &[at(22, PackType::Resource)],
        ),
        chemistry(
            20,
            one.clone(),
            PackType::Resource,
            &[at(10, PackType::Behavior)],
        ),
        chemistry(
            21,
            SemVersion::new(1, 20, 50),
            PackType::Resource,
            &[at(11, PackType::Behavior)],
        ),
        chemistry(
            22,
            SemVersion::new(1, 21, 30),
            PackType::Resource,
            &[at(12, PackType::Behavior)],
        ),
    ]
}

fn education(base_game_version: BaseGameVersion) -> LoadOptions {
    LoadOptions {
        education: true,
        base_game_version,
        ..LoadOptions::default()
    }
}

/// In an education world the game appends the chemistry behaviour packs to the world's stack. It
/// walks the chemistry category from its end, skips a pack whose header version is above the
/// world's base-game version (any or invalid: none skipped) or that is no behaviour pack, and
/// adds each to the stack. A dependency is looked up by exact `(uuid, version)`; on a miss the
/// dependent is marked in error without a log line, so the slices are on the stack, hidden,
/// silently.
#[test]
fn an_education_world_appends_the_chemistry_behaviour_packs_and_hides_the_slices_silently() {
    let packs = chemistry_packs();
    let world_pack = bp(1);
    let mut registered: Vec<(&PackFacts, PackOrigin)> = vec![(&world_pack, PackOrigin::World)];
    registered.extend(packs.iter().map(|facts| (facts, PackOrigin::Package)));
    let composition = run_with_origins(
        &registered,
        &world(vec![select(&world_pack)], vec![]),
        &education(BaseGameVersion::new(SemVersion::new(1, 21, 0))),
    );
    // world, then from the category's end: chemistry_1.20.50 (1.21.30 is above the world),
    // chemistry with its resource half split off.
    assert_eq!(order(&composition.behavior), [0, 2, 1]);
    let hidden: Vec<bool> = composition.behavior.iter().map(|p| p.hidden).collect();
    assert_eq!(hidden, [false, true, false]);
    // The unversioned pair's resource half goes to the resource stack, silently.
    assert_eq!(order(&composition.resource), [4]);
    assert!(
        above_info(&composition).is_empty(),
        "{:?}",
        composition.findings
    );
    let missing: Vec<_> = composition
        .findings
        .iter()
        .filter(|d| d.is(&core_codes::MISSING_DEPENDENCY))
        .map(|d| d.severity)
        .collect();
    assert_eq!(missing, [Severity::Info]);
}

#[test]
fn an_education_world_of_any_version_takes_every_chemistry_behaviour_pack() {
    let packs = chemistry_packs();
    let registered: Vec<(&PackFacts, PackOrigin)> = packs
        .iter()
        .map(|facts| (facts, PackOrigin::Package))
        .collect();
    let composition = run_with_origins(
        &registered,
        &world(vec![], vec![]),
        &education(BaseGameVersion::ANY),
    );
    assert_eq!(order(&composition.behavior), [2, 1, 0]);
    assert!(
        above_info(&composition).is_empty(),
        "{:?}",
        composition.findings
    );
}

/// Without education no chemistry pack is on a stack and nothing is said.
#[test]
fn a_world_without_education_leaves_the_chemistry_packs_off_the_stacks_silently() {
    let packs = chemistry_packs();
    let registered: Vec<(&PackFacts, PackOrigin)> = packs
        .iter()
        .map(|facts| (facts, PackOrigin::Package))
        .collect();
    let composition = run_with_origins(
        &registered,
        &world(vec![], vec![]),
        &LoadOptions {
            base_game_version: BaseGameVersion::ANY,
            ..LoadOptions::default()
        },
    );
    assert!(composition.behavior.is_empty() && composition.resource.is_empty());
    assert!(
        above_info(&composition).is_empty(),
        "{:?}",
        composition.findings
    );
}

/// A chemistry-capable pack of a world or a developer folder is no category member: only the
/// app-bundled packs are (the game's category is the in-package path's).
#[test]
fn an_education_world_takes_only_the_app_bundled_chemistry_packs() {
    let packs = chemistry_packs();
    let registered: Vec<(&PackFacts, PackOrigin)> =
        packs.iter().map(|facts| (facts, PackOrigin::Dev)).collect();
    let composition = run_with_origins(
        &registered,
        &world(vec![], vec![]),
        &education(BaseGameVersion::ANY),
    );
    assert!(
        composition.behavior.is_empty(),
        "{:?}",
        composition.behavior
    );
}

/// A world that lists an app-bundled pack whose dependency is unmet (the chemistry slices of an
/// every-pack world): the pack is hidden, as the game marks it in error, and the finding is an
/// information, because the game logs nothing and no activation dialog reaches an app-bundled
/// pack. A third-party pack keeps the error.
#[test]
fn a_world_listed_app_bundled_pack_with_an_unmet_dependency_is_hidden_with_an_information() {
    let one = SemVersion::new(1, 0, 0);
    let slice = chemistry(
        11,
        SemVersion::new(1, 20, 50),
        PackType::Behavior,
        &[PackIdVersion::new(
            Uuid::from_u128(21),
            one,
            PackType::Resource,
        )],
    );
    for (origin, severity) in [
        (PackOrigin::Package, Severity::Info),
        (PackOrigin::World, Severity::Error),
    ] {
        let composition = run_with_origins(
            &[(&slice, origin)],
            &world(vec![select(&slice)], vec![]),
            &LoadOptions::default(),
        );
        assert!(composition.behavior[0].hidden, "{origin:?}");
        let found: Vec<Severity> = composition
            .findings
            .iter()
            .filter(|d| d.is(&core_codes::MISSING_DEPENDENCY))
            .map(|d| d.severity)
            .collect();
        assert_eq!(found, [severity], "{origin:?}");
    }
}
