//! The frame of the link pass through the public API: the entry point on any world, with any
//! set of features. The six rule groups are empty here; what the frame does with rules is
//! tested beside it (`src/pack/link/tests.rs`).

use bedrock_addon::diagnostic::{Phase, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{LinkOptions, PackStack, StackSet};

use super::common::{
    behavior_pack, hosted, in_file, link, link_with, resource_pack, resource_pack_of, stacks,
    stacks_of, unloaded_pack, unresolved,
};

/// A custom block: the one content module the narrowest feature set of this binary has.
const BLOCK: &str = r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:lamp"}, "components": {}}}"#;

/// A block file the load rejects.
const BROKEN: &str = r#"{"format_version": "1.26.20", "minecraft:block": 5}"#;

#[test]
fn a_world_without_packs_links_to_nothing() {
    let set = StackSet {
        behavior: PackStack::of([], LoadOptions::default()).expect("no pack"),
        resource: PackStack::of([], LoadOptions::default()).expect("no pack"),
    };
    assert!(set.link().is_empty());
    assert!(
        set.link_with(&LinkOptions {
            lints: false,
            ..Default::default()
        })
        .is_empty()
    );
}

#[test]
fn a_world_with_one_side_only_links() {
    let behavior_only = stacks_of(
        vec![behavior_pack(1, &[("blocks/lamp.json", BLOCK)])],
        Vec::new(),
        LoadOptions::default(),
    );
    assert_eq!(behavior_only.resource.packs().len(), 0);
    let _ = behavior_only.link();
    let resource_only = stacks_of(
        Vec::new(),
        vec![resource_pack(2, &[("blocks.json", "{}")])],
        LoadOptions::default(),
    );
    assert_eq!(resource_only.behavior.packs().len(), 0);
    let _ = resource_only.link();
}

/// The pass never fails: files the load rejects, files of no family and an empty file are a
/// world like any other.
#[test]
fn a_world_of_broken_files_links_without_a_panic() {
    let found = link(
        &[
            ("blocks/lamp.json", BLOCK),
            ("blocks/broken.json", BROKEN),
            ("blocks/empty.json", ""),
            ("items/not_json.json", "{"),
            ("entities/array.json", "[]"),
            ("loot_tables/x.json", "null"),
        ],
        &[
            ("blocks.json", "5"),
            ("textures/terrain_texture.json", "{"),
            ("sounds/sound_definitions.json", "[]"),
            ("nowhere/file.txt", "text"),
        ],
    );
    assert!(found.iter().all(|finding| finding.phase == Phase::Link));
}

#[test]
fn link_is_link_with_the_default_options() {
    assert_eq!(
        LinkOptions::default(),
        LinkOptions {
            lints: true,
            ..Default::default()
        }
    );
    let set = stacks(&[("blocks/lamp.json", BLOCK)], &[], LoadOptions::default());
    assert_eq!(set.link(), set.link_with(&LinkOptions::default()));
    assert_eq!(
        format!("{:?}", set.link()),
        format!(
            "{:?}",
            set.link_with(&LinkOptions {
                lints: true,
                ..Default::default()
            })
        )
    );
}

/// Without the lints the pass tells no information, and nothing else is left out.
#[test]
fn without_the_lints_the_pass_tells_exactly_the_findings_above_an_information() {
    let files = [("blocks/lamp.json", BLOCK), ("blocks/broken.json", BROKEN)];
    let all = link(&files, &[]);
    let without = link_with(
        &files,
        &[],
        LoadOptions::default(),
        LinkOptions {
            lints: false,
            ..Default::default()
        },
    );
    let kept: Vec<_> = all
        .iter()
        .filter(|finding| finding.severity > Severity::Info)
        .collect();
    assert_eq!(kept, without.iter().collect::<Vec<_>>());
    assert!(without.iter().all(|f| f.severity > Severity::Info));
}

/// The pass reads the stacks and changes nothing: a collection it read is what the load left.
#[test]
fn the_pass_changes_no_collection() {
    let set = stacks(
        &[("blocks/lamp.json", BLOCK), ("blocks/broken.json", BROKEN)],
        &[],
        LoadOptions::default(),
    );
    let before = format!("{:?}", set.load_all().iter().collect::<Vec<_>>());
    let stacks_before = format!("{set:?}");
    let _ = set.link();
    let _ = set.link_with(&LinkOptions {
        lints: false,
        ..Default::default()
    });
    assert_eq!(format!("{set:?}"), stacks_before);
    assert_eq!(
        format!("{:?}", set.load_all().iter().collect::<Vec<_>>()),
        before
    );
}

/// The same packs give the same findings, twice on one world and on a world whose packs were
/// registered in the other order.
#[test]
fn the_findings_are_the_same_twice_and_for_either_registration_order() {
    let packs = || {
        (
            vec![
                behavior_pack(1, &[("blocks/lamp.json", BLOCK)]),
                behavior_pack(3, &[("blocks/lamp.json", BLOCK), ("blocks/b.json", BROKEN)]),
            ],
            vec![resource_pack(2, &[("blocks.json", "{}")])],
        )
    };
    let world = |reversed: bool| {
        use bedrock_addon::pack::{PackOrigin, PackRepository, PackSelection, WorldPacks};
        let (behavior, resource) = packs();
        let select = |packs: &[bedrock_addon::pack::Pack]| -> Vec<PackSelection> {
            packs
                .iter()
                .map(|pack| PackSelection {
                    pack_id: pack.id().uuid,
                    version: pack.id().version.clone(),
                    subpack: None,
                })
                .collect()
        };
        let selection = WorldPacks {
            behavior: select(&behavior),
            resource: select(&resource),
        };
        let mut all: Vec<_> = behavior.into_iter().chain(resource).collect();
        if reversed {
            all.reverse();
        }
        let mut repository = PackRepository::new();
        for pack in all {
            repository.add_pack(pack, PackOrigin::User);
        }
        repository.stacks(&selection, LoadOptions::default())
    };
    let (forward, backward) = (world(false), world(true));
    let text = |set: &StackSet| format!("{:?}", set.link());
    let first = text(&forward);
    assert_eq!(first, text(&forward));
    assert_eq!(first, text(&backward));
}

/// A host is part of the load options the stacks were built with: the pass takes it from
/// there, and a world with one links like a world without.
#[test]
fn a_world_with_a_host_links() {
    let found = link_with(
        &[("blocks/lamp.json", BLOCK)],
        &[],
        hosted(&[("block", "minecraft:stone")]),
        LinkOptions::default(),
    );
    assert_eq!(unresolved(&found), []);
}

/// A pack helper for bytes: the file is in the pack as given and resident.
#[test]
fn a_pack_of_binary_files_holds_their_bytes() {
    use bedrock_addon::storage::PackPath;
    const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0xff, 0x00];
    let pack = resource_pack_of(2, &[("textures/blocks/lamp.png", PNG)]);
    let path = PackPath::from_static("textures/blocks/lamp.png");
    assert!(pack.is_resident(&path));
    let set = stacks_of(Vec::new(), vec![pack], LoadOptions::default());
    assert_eq!(set.resource.packs().len(), 1);
    assert!(set.link().is_empty());
}

/// A pack opened from a folder and not loaded: its files are listed and not resident, the
/// load says so of each file a collection walks, and the pass links such a world.
#[test]
fn an_unloaded_pack_lists_its_files_without_their_bytes() {
    use bedrock_addon::storage::PackPath;
    let pack = unloaded_pack(1, "data", &[("blocks/lamp.json", BLOCK.as_bytes())]);
    assert!(pack.is_resident(&PackPath::from_static("manifest.json")));
    let lamp = PackPath::from_static("blocks/lamp.json");
    assert!(pack.paths().any(|path| *path == lamp));
    assert!(!pack.is_resident(&lamp));
    let set = stacks_of(vec![pack], Vec::new(), LoadOptions::default());
    let report = set.load_all();
    let loaded: Vec<&str> = report
        .iter()
        .filter(|finding| {
            finding
                .location
                .path
                .as_ref()
                .is_some_and(|path| *path == lamp)
        })
        .map(|finding| finding.code.name())
        .collect();
    // The file is a block: without the block collection nothing asks for it at the load.
    if cfg!(feature = "block") {
        assert_eq!(loaded, ["FileNotLoaded"]);
    } else {
        assert_eq!(loaded, [""; 0]);
    }
    // No rule asks for bytes yet: the pass tells nothing of the file.
    assert_eq!(in_file(&set.link(), "blocks/lamp.json").len(), 0);
}
