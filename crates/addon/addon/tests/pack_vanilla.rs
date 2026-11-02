#![allow(clippy::pedantic, missing_docs)]
//! `pack::VANILLA_EXPERIMENTAL_PACKS` against the shipped pack folders: every row is the header
//! UUID of an experimental pack of its type in the 1.26.20 dev or the 1.26.45 retail build, or
//! (the Drop 4 rows) in the 1.26.60 install. Needs `BEDROCK_ADDON_CORPUS`;
//! skipped with a message without it; the 1.26.60 rows need `BEDROCK_ADDON_VANILLA_1_26_60`.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "pack/common.rs"]
mod common;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use bedrock_addon::json::Value;
use bedrock_addon::load::Experiment;
use bedrock_addon::pack::{PackType, VANILLA_EXPERIMENTAL_PACKS};
use uuid::Uuid;

#[test]
fn the_table_has_twelve_behavior_and_eleven_resource_packs_with_distinct_uuids() {
    let count = |pack_type| {
        VANILLA_EXPERIMENTAL_PACKS
            .iter()
            .filter(|(_, t, _)| *t == pack_type)
            .count()
    };
    // With the 1.26.60 `experimental_y_2026_drop_4` pair.
    assert_eq!(count(PackType::Behavior), 12);
    assert_eq!(count(PackType::Resource), 11);
    let mut uuids: Vec<Uuid> = VANILLA_EXPERIMENTAL_PACKS
        .iter()
        .map(|(u, _, _)| *u)
        .collect();
    uuids.sort_unstable();
    uuids.dedup();
    assert_eq!(uuids.len(), VANILLA_EXPERIMENTAL_PACKS.len());
}

/// The Drop 4 rows are the header UUIDs of the 1.26.60 `experimental_y_2026_drop_4`
/// manifests, checked on every run; the install lookup below is the second check.
#[test]
fn the_drop_4_rows_are_the_shipped_manifests_uuids() {
    let rows: Vec<(Uuid, PackType)> = VANILLA_EXPERIMENTAL_PACKS
        .iter()
        .filter(|(_, _, experiment)| *experiment == Experiment::Y2026Drop4)
        .map(|(uuid, pack_type, _)| (*uuid, *pack_type))
        .collect();
    assert_eq!(
        rows,
        [
            (
                Uuid::from_u128(0x463a758b_b207_41d2_88ed_50b4bc034e03),
                PackType::Behavior
            ),
            (
                Uuid::from_u128(0xf3d96f1e_fec7_4c4c_8c8a_f63d7ab39218),
                PackType::Resource
            ),
        ]
    );
}

#[test]
fn the_behavior_packs_are_in_the_order_the_server_pushes_them() {
    let order: Vec<Experiment> = VANILLA_EXPERIMENTAL_PACKS
        .iter()
        .filter(|(_, pack_type, _)| *pack_type == PackType::Behavior)
        .map(|(_, _, experiment)| *experiment)
        .collect();
    assert_eq!(
        order,
        [
            Experiment::NextUpdate,
            Experiment::ExperimentalCreatorCameras,
            Experiment::MinecraftExplorer,
            Experiment::VillagerTradesRebalance,
            Experiment::SimplifiedSpawnRules,
            Experiment::VoxelShapes,
            Experiment::Poi,
            // right after POI, as `initializeBehaviorStack` tests them
            Experiment::Y2026Drop4,
            Experiment::BrainPrototype,
            Experiment::Y2026Drop2,
            Experiment::Lwe,
            Experiment::Y2026Drop3,
        ]
    );
}

/// The header UUID of every manifest under `<build>/<behavior_packs|resource_packs>/*/`.
fn shipped(corpus: &Path) -> HashMap<(Uuid, PackType), PathBuf> {
    let mut found = HashMap::new();
    for build in ["gdk-dev", "gdk-retail"] {
        for (folder, pack_type) in [
            ("behavior_packs", PackType::Behavior),
            ("resource_packs", PackType::Resource),
        ] {
            let Ok(entries) = fs::read_dir(corpus.join(build).join(folder)) else {
                continue;
            };
            scan(entries, pack_type, &mut found);
        }
    }
    // The 1.26.60 install's own folders (the Drop 4 rows).
    if let Some(install) = std::env::var_os("BEDROCK_ADDON_VANILLA_1_26_60").map(PathBuf::from) {
        for (folder, pack_type) in [
            ("behavior_packs", PackType::Behavior),
            ("resource_packs", PackType::Resource),
        ] {
            if let Ok(entries) = fs::read_dir(install.join(folder)) {
                scan(entries, pack_type, &mut found);
            }
        }
    }
    found
}

/// The header UUID of each pack folder of `entries`, as `pack_type`, into `found`.
fn scan(entries: fs::ReadDir, pack_type: PackType, found: &mut HashMap<(Uuid, PackType), PathBuf>) {
    for entry in entries.flatten() {
        let manifest = entry.path().join("manifest.json");
        let Ok(text) = fs::read_to_string(&manifest) else {
            continue;
        };
        let Ok(value) = facet_json::from_str_jsonc::<Value>(&text) else {
            continue;
        };
        let header_uuid = value
            .as_object()
            .and_then(|root| root.get("header"))
            .and_then(Value::as_object)
            .and_then(|header| header.get("uuid"))
            .and_then(Value::as_string)
            .and_then(|text| Uuid::parse_str(text.as_str()).ok());
        if let Some(uuid) = header_uuid {
            found.insert((uuid, pack_type), entry.path());
        }
    }
}

#[test]
fn every_row_names_a_shipped_experimental_pack() {
    let Some(corpus) = std::env::var_os("BEDROCK_ADDON_CORPUS").map(PathBuf::from) else {
        eprintln!("skipped: BEDROCK_ADDON_CORPUS is not set");
        return;
    };
    let shipped = shipped(&corpus);
    if shipped.is_empty() {
        eprintln!(
            "skipped: no gdk-dev or gdk-retail packs under {}",
            corpus.display()
        );
        return;
    }
    // The Drop 4 pair ships only in 1.26.60: looked up there when the install is
    // given, else left out with a message.
    let install = std::env::var_os("BEDROCK_ADDON_VANILLA_1_26_60").is_some();
    if !install {
        eprintln!("skipped: the Drop 4 rows need BEDROCK_ADDON_VANILLA_1_26_60");
    }
    let missing: Vec<_> = VANILLA_EXPERIMENTAL_PACKS
        .iter()
        .filter(|(_, _, experiment)| install || *experiment != Experiment::Y2026Drop4)
        .filter(|(uuid, pack_type, _)| !shipped.contains_key(&(*uuid, *pack_type)))
        .collect();
    assert!(missing.is_empty(), "rows with no shipped pack: {missing:?}");
}

/// The `vanilla` behaviour pack and its slices (one identity, the slice's version).
const VANILLA: Uuid = Uuid::from_u128(0xfe9f8597_5454_481a_8730_8d070a8e2e58);

/// The versions of the `vanilla` slices on the behaviour stack of a world that lists nothing,
/// top first, over an install with the slices 1.19.0, 1.20.0 and 1.21.0 and `vanilla` itself.
fn slices_selected_for(world: bedrock_addon::version::BaseGameVersion) -> Vec<[u16; 3]> {
    use bedrock_addon::load::LoadOptions;
    use bedrock_addon::pack::{PackOrigin, PackRepository, WorldPacks};

    let bundled = |version: [u16; 3]| {
        common::Spec {
            name: format!("vanilla {version:?}"),
            uuid: VANILLA,
            ..common::bp(0)
        }
        .version(version)
    };
    let mut repository = PackRepository::new();
    // Registered out of order: the composer sorts the slices.
    for version in [[0, 0, 1], [1, 20, 0], [1, 19, 0], [1, 21, 0]] {
        repository.add_pack(bundled(version).open(), PackOrigin::Package);
    }
    let options = LoadOptions {
        base_game_version: world,
        ..LoadOptions::default()
    };
    let nothing = WorldPacks {
        behavior: Vec::new(),
        resource: Vec::new(),
    };
    let set = repository.stacks(&nothing, options);
    let stack: Vec<[u16; 3]> = set
        .behavior
        .packs()
        .map(|pack| {
            assert_eq!(pack.id().uuid, VANILLA);
            let version = &pack.id().version;
            [version.major, version.minor, version.patch]
        })
        .collect();
    assert_eq!(
        stack.last(),
        Some(&[0, 0, 1]),
        "`vanilla` is below the slices"
    );
    stack[..stack.len() - 1].to_vec()
}

/// 05 "Slices": "world version `*` (`isAnyVersion`) or invalid → every slice; otherwise the
/// slices whose version is ≤ the world's `base_game_version`", newest first. A world value that
/// is no version is `*`, unset, unparsable, or the never-compatible constant: vanilla's slice
/// code asks only whether the version is any or invalid, so each selects every slice.
#[test]
fn a_world_base_game_version_that_is_no_version_selects_every_slice() {
    use bedrock_addon::version::{BaseGameVersion, SemVersion};

    let every = [[1, 21, 0], [1, 20, 0], [1, 19, 0]];
    let unparsable = BaseGameVersion {
        version: SemVersion::parse("1.twenty"),
        never_compatible: false,
    };
    assert_eq!(unparsable.version, None);
    for (name, world) in [
        ("*", BaseGameVersion::ANY),
        ("unset", BaseGameVersion::EMPTY),
        ("unparsable", unparsable),
        ("never compatible", BaseGameVersion::INCOMPATIBLE),
    ] {
        assert_eq!(slices_selected_for(world), every, "{name}");
    }
}

/// Both sides of the bound: a slice at the world's version is selected, the next one is not.
#[test]
fn a_world_base_game_version_selects_the_slices_not_above_it() {
    use bedrock_addon::version::{BaseGameVersion, SemVersion};

    let at = |major, minor, patch| BaseGameVersion::new(SemVersion::new(major, minor, patch));
    assert_eq!(
        slices_selected_for(at(1, 20, 0)),
        [[1, 20, 0], [1, 19, 0]],
        "a slice at the world's version"
    );
    assert_eq!(
        slices_selected_for(at(1, 19, 99)),
        [[1, 19, 0]],
        "a slice above the world's version"
    );
    assert_eq!(slices_selected_for(at(1, 18, 0)), [[0u16; 3]; 0]);
    assert_eq!(
        slices_selected_for(at(1, 26, 0)),
        [[1, 21, 0], [1, 20, 0], [1, 19, 0]]
    );
}

/// A version that also carries the never-compatible flag is a version: vanilla's slice code
/// asks whether the version is any or invalid and reads nothing else, so the slices not above
/// it are selected, as without the flag.
#[test]
fn a_never_compatible_flag_on_a_version_does_not_change_the_slices() {
    use bedrock_addon::version::{BaseGameVersion, SemVersion};

    let flagged = |major, minor, patch| BaseGameVersion {
        version: Some(SemVersion::new(major, minor, patch)),
        never_compatible: true,
    };
    assert_eq!(
        slices_selected_for(flagged(1, 20, 0)),
        [[1, 20, 0], [1, 19, 0]]
    );
    assert_eq!(slices_selected_for(flagged(1, 19, 99)), [[1, 19, 0]]);
    assert_eq!(slices_selected_for(flagged(1, 18, 0)), [[0u16; 3]; 0]);
}
