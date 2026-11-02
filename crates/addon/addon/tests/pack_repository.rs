//! `PackRepository`: registration and `stacks`, the 05 composition rules through the public
//! API. Packs are opened with `Pack::from_files` from small in-memory manifests.

#[path = "pack/common.rs"]
mod common;

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::load::{Experiment, LoadOptions};
use bedrock_addon::pack::{
    PackIdVersion, PackOrigin, PackRepository, PackSelection, PackStack, Side, StackSet,
    VANILLA_EXPERIMENTAL_PACKS, WorldPacks,
};
use bedrock_addon::version::{BaseGameVersion, SemVersion};
use uuid::Uuid;

use common::{Spec, bp, rp};

fn select(spec: &Spec) -> PackSelection {
    let id = spec.id();
    PackSelection {
        pack_id: id.uuid,
        version: id.version,
        subpack: None,
    }
}

fn world(behavior: &[&Spec], resource: &[&Spec]) -> WorldPacks {
    WorldPacks {
        behavior: behavior.iter().map(|spec| select(spec)).collect(),
        resource: resource.iter().map(|spec| select(spec)).collect(),
    }
}

fn ids<S: Side>(stack: &PackStack<S>) -> Vec<PackIdVersion> {
    stack.packs().map(|pack| pack.id().clone()).collect()
}

/// Selection findings that name a place or a pack of the world's lists: all informations; the base-game layer's missing vanilla packs carry no place and are
/// left out.
fn warnings(set: &StackSet) -> Vec<DiagnosticCode> {
    set.selection()
        .iter()
        .filter(|diagnostic| {
            let missing_dependency =
                matches!(diagnostic.code, DiagnosticCode::MissingDependency { .. });
            let pack_not_found_at_no_path =
                matches!(diagnostic.code, DiagnosticCode::PackNotFound { .. })
                    && diagnostic.location.path.is_none();
            !missing_dependency && !pack_not_found_at_no_path
        })
        .inspect(|diagnostic| assert_eq!(diagnostic.severity, Severity::Info, "{diagnostic:?}"))
        .map(|diagnostic| diagnostic.code.clone())
        .collect()
}

#[test]
fn an_empty_repository_stacks_nothing_and_says_what_it_missed() {
    let ghost = bp(7);
    let set = PackRepository::new().stacks(&world(&[&ghost], &[]), LoadOptions::default());
    assert_eq!(set.behavior.packs().len(), 0);
    assert_eq!(set.resource.packs().len(), 0);
    let found = warnings(&set);
    assert!(matches!(&found[..], [DiagnosticCode::PackNotFound { pack }] if *pack == ghost.id()));
    // The base-game layer: both script libraries and `vanilla`, as info, and the ghost's own
    // entry.
    let info = set
        .selection()
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Info)
        .count();
    assert_eq!(info, 4);
}

#[test]
fn the_test_packs_manifests_are_valid() {
    let r = rp(10).subpack("low", "Low", 0).subpack("high", "High", 4);
    let a = bp(1).depends_on(&r).depends_on(&bp(2));
    let slice = bundled(VANILLA, [1, 20, 0]);
    for spec in [&a, &r, &slice] {
        let manifest = spec.parsed_manifest();
        assert_eq!(manifest.identity(), spec.id());
        assert_eq!(manifest.identity().pack_type, spec.id().pack_type);
    }
    let tiers: Vec<u8> = r
        .parsed_manifest()
        .subpacks
        .iter()
        .map(|subpack| subpack.memory_tier.number())
        .collect();
    assert_eq!(tiers, [0, 4]);
    assert_eq!(a.parsed_manifest().dependencies.len(), 2);
}

#[test]
fn an_empty_repository_has_no_packs_reports_or_invalid_packs() {
    let repository = PackRepository::new();
    assert_eq!(repository.packs().count(), 0);
    assert!(repository.invalid().is_empty());
    assert!(repository.source_reports().is_empty());
    assert!(repository.get(&bp(1).id()).is_none());
}

#[test]
fn the_first_registration_of_an_identity_and_origin_wins() {
    let first = bp(1).file("first.txt", b"1");
    let second = bp(1).file("second.txt", b"2");
    let mut repository = PackRepository::new();
    repository.add_pack(first.open(), PackOrigin::World);
    repository.add_pack(second.open(), PackOrigin::World);
    assert_eq!(repository.packs().count(), 1);
    let kept = repository.get(&first.id()).expect("registered");
    assert!(kept.paths().any(|path| path.as_str() == "first.txt"));
    let set = repository.stacks(&world(&[], &[]), LoadOptions::default());
    assert!(matches!(
        &warnings(&set)[..],
        [DiagnosticCode::DuplicateIdentifier { .. }]
    ));
}

#[test]
fn the_same_identity_from_another_origin_registers_too_and_the_first_is_found() {
    let world_copy = bp(1).file("world.txt", b"w");
    let global_copy = bp(1).file("global.txt", b"g");
    let mut repository = PackRepository::new();
    repository.add_pack(world_copy.open(), PackOrigin::World);
    repository.add_pack(global_copy.open(), PackOrigin::User);
    assert_eq!(repository.packs().count(), 2);
    let found = repository.get(&world_copy.id()).expect("registered");
    assert!(found.paths().any(|path| path.as_str() == "world.txt"));
    let set = repository.stacks(&world(&[&world_copy], &[]), LoadOptions::default());
    assert_eq!(ids(&set.behavior), [world_copy.id()]);
    assert!(warnings(&set).is_empty());
}

#[test]
fn a_missing_dependency_hides_the_pack_with_an_error() {
    let missing = bp(9);
    let a = bp(1).depends_on(&missing);
    let b = bp(2);
    let mut repository = PackRepository::new();
    repository.add_pack(a.open(), PackOrigin::World);
    repository.add_pack(b.open(), PackOrigin::World);
    let set = repository.stacks(&world(&[&a, &b], &[]), LoadOptions::default());
    assert_eq!(ids(&set.behavior), [b.id()]);
    let errors: Vec<_> = set.selection().at_least(Severity::Error).collect();
    assert!(matches!(
        &errors[..],
        [d] if matches!(&d.code, DiagnosticCode::MissingDependency { pack, dependency }
            if *pack == a.id() && *dependency == missing.id())
    ));
}

#[test]
fn dependencies_follow_their_dependent() {
    let c = bp(3);
    let b = bp(2).depends_on(&c);
    let a = bp(1).depends_on(&b);
    let mut repository = PackRepository::new();
    for spec in [&c, &b, &a] {
        repository.add_pack(spec.open(), PackOrigin::World);
    }
    let set = repository.stacks(&world(&[&a], &[]), LoadOptions::default());
    assert_eq!(ids(&set.behavior), [a.id(), b.id(), c.id()]);
}

#[test]
fn a_behavior_packs_resource_pack_is_auto_added_with_a_warning() {
    let r = rp(10);
    let q = rp(11);
    let a = bp(1).depends_on(&r);
    let mut repository = PackRepository::new();
    for spec in [&a, &r, &q] {
        repository.add_pack(spec.open(), PackOrigin::World);
    }
    let set = repository.stacks(&world(&[&a], &[&q]), LoadOptions::default());
    assert_eq!(ids(&set.resource), [q.id(), r.id()]);
    let found = warnings(&set);
    assert!(matches!(
        &found[..],
        [DiagnosticCode::DependencyAutoAdded { pack, dependency }]
            if *pack == a.id() && *dependency == r.id()
    ));
}

/// An app-bundled behaviour pack at `uuid` and `version`.
fn bundled(uuid: Uuid, version: [u16; 3]) -> Spec {
    Spec {
        name: format!("bundled {uuid} {version:?}"),
        uuid,
        ..bp(0)
    }
    .version(version)
}

const VANILLA: Uuid = Uuid::from_u128(0xfe9f8597_5454_481a_8730_8d070a8e2e58);
const SERVER_LIBRARY: Uuid = Uuid::from_u128(0x37ca4282_69e9_4402_b40b_0b07de579f02);
const SERVER_UI_LIBRARY: Uuid = Uuid::from_u128(0xf53f490f_9d47_45ca_86de_0946b6d1c366);

#[test]
fn the_base_game_layer_is_experiments_libraries_slices_newest_first_then_vanilla() {
    let world_pack = bp(1);
    let (shapes_uuid, _, _) = VANILLA_EXPERIMENTAL_PACKS
        .iter()
        .find(|(_, _, experiment)| *experiment == Experiment::VoxelShapes)
        .expect("a row");
    let shapes = bundled(*shapes_uuid, [0, 0, 1]);
    let library = bundled(SERVER_LIBRARY, [0, 0, 1]);
    let ui_library = bundled(SERVER_UI_LIBRARY, [0, 0, 1]);
    let vanilla = bundled(VANILLA, [0, 0, 1]);
    let (s1_19, s1_20, s1_21) = (
        bundled(VANILLA, [1, 19, 0]),
        bundled(VANILLA, [1, 20, 0]),
        bundled(VANILLA, [1, 21, 0]),
    );
    let mut repository = PackRepository::new();
    repository.add_pack(world_pack.open(), PackOrigin::World);
    for spec in [
        &vanilla,
        &s1_19,
        &s1_21,
        &s1_20,
        &ui_library,
        &library,
        &shapes,
    ] {
        repository.add_pack(spec.open(), PackOrigin::Package);
    }
    let options = LoadOptions {
        experiments: [Experiment::VoxelShapes].into_iter().collect(),
        base_game_version: BaseGameVersion::new(SemVersion::new(1, 20, 10)),
        ..LoadOptions::default()
    };
    let set = repository.stacks(&world(&[&world_pack], &[]), options);
    assert_eq!(
        ids(&set.behavior),
        [
            world_pack.id(),
            shapes.id(),
            library.id(),
            ui_library.id(),
            s1_20.id(),
            s1_19.id(),
            vanilla.id(),
        ]
    );
    assert!(set.selection().is_empty(), "{:?}", set.selection());
}

#[test]
fn the_world_chooses_a_subpack_by_folder_name() {
    let r = rp(10)
        .subpack("low", "Low", 0)
        .subpack("high", "High", 4)
        .file("textures/a.png", b"parent")
        .file("subpacks/low/textures/a.png", b"low");
    let mut repository = PackRepository::new();
    repository.add_pack(r.open(), PackOrigin::World);
    let mut chosen = world(&[], &[&r]);
    chosen.resource[0].subpack = Some("low".to_owned());
    let set = repository.stacks(&chosen, LoadOptions::default());
    assert_eq!(ids(&set.resource), [r.id()]);
    assert!(warnings(&set).is_empty());
    let unknown = {
        let mut unknown = world(&[], &[&r]);
        unknown.resource[0].subpack = Some("medium".to_owned());
        repository.stacks(&unknown, LoadOptions::default())
    };
    assert!(matches!(
        &warnings(&unknown)[..],
        [DiagnosticCode::InvalidValue { .. }]
    ));
}
