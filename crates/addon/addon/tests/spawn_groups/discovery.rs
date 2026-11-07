//! `bp.spawn_groups()`: only the app data's `definitions/spawn_groups/*.json`, file-name order,
//! the later file per identifier winning,
//! and the raid waves (`sg01`, `sg02`).

use std::path::PathBuf;

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::spawn_groups::SpawnGroup;
use bedrock_addon::families::spawn_rules::Difficulty;
use bedrock_addon::id::SpawnGroupId;
use bedrock_addon::load::{AppData, LoadOptions};
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackOrigin, PackStack};
use bedrock_addon::storage::PackPath;

/// A group document for `id` with one condition of herd size `size`.
fn group(id: &str, version: &str, size: u32) -> Vec<u8> {
    format!(
        r#"{{"format_version": "{version}", "minecraft:spawn_groups": {{"description": {{"identifier": "{id}"}}, "conditions": [{{"minecraft:herd": {{"min_size": {size}, "max_size": {size}}}}}]}}}}"#
    )
    .into_bytes()
}

/// A behaviour pack with `files` besides its manifest.
fn pack(files: Vec<(&str, Vec<u8>)>) -> Pack {
    let manifest = format!(
        r#"{{"format_version": 2, "header": {{"name": "bp", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 26, 20]}}, "modules": [{{"type": "data", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
        uuid::Uuid::from_u128(1),
        uuid::Uuid::from_u128(1001),
    );
    let files = std::iter::once(("manifest.json", manifest.into_bytes()))
        .chain(files)
        .map(|(path, bytes)| (PackPath::new(path).expect("a pack path"), bytes));
    let options = OpenOptions {
        origin: PackOrigin::User,
        ..OpenOptions::default()
    };
    Pack::from_files(files, &options).expect("the pack opens")
}

/// The registry of a stack of `bp` with `app_data` (paths below the app-data root).
fn groups(bp: Pack, app_data: Vec<(&str, Vec<u8>)>) -> PackStack<Behavior> {
    let app_data = AppData::from_files(
        app_data
            .into_iter()
            .map(|(path, bytes)| (PackPath::new(path).expect("a path"), bytes)),
    );
    let options = LoadOptions {
        app_data: Some(app_data),
        ..LoadOptions::default()
    };
    PackStack::<Behavior>::of([bp], options).expect("a behaviour stack")
}

#[test]
fn sg40_non_recursive() {
    let stack = groups(
        pack(vec![]),
        vec![
            (
                "definitions/spawn_groups/sub/x.json",
                group("x:sub", "1.11.0", 1),
            ),
            (
                "definitions/spawn_groups/top.json",
                group("x:top", "1.11.0", 1),
            ),
        ],
    );
    let registry = stack.spawn_groups();
    assert!(registry.contains(&SpawnGroupId::parse("x:top")));
    assert!(!registry.contains(&SpawnGroupId::parse("x:sub")));
    assert!(registry.rejected().is_empty());
}

#[test]
fn sg41_extension() {
    let stack = groups(
        pack(vec![]),
        vec![
            (
                "definitions/spawn_groups/a.JSON",
                group("x:upper", "1.11.0", 1),
            ),
            (
                "definitions/spawn_groups/a.json.bak",
                group("x:bak", "1.11.0", 1),
            ),
            (
                "definitions/spawn_groups/a.txt",
                group("x:txt", "1.11.0", 1),
            ),
        ],
    );
    let registry = stack.spawn_groups();
    assert!(registry.is_empty(), "{registry:#?}");
    assert!(registry.rejected().is_empty());
}

#[test]
fn sg42_duplicate_identifier() {
    let stack = groups(
        pack(vec![]),
        vec![
            (
                "definitions/spawn_groups/b.json",
                group("minecraft:raid_group_1", "1.11.0", 2),
            ),
            (
                "definitions/spawn_groups/a.json",
                group("minecraft:raid_group_1", "1.11.0", 1),
            ),
        ],
    );
    let registry = stack.spawn_groups();
    let winner = registry
        .get(&SpawnGroupId::parse("minecraft:raid_group_1"))
        .unwrap();
    assert_eq!(
        winner.origin().path.as_str(),
        "definitions/spawn_groups/b.json"
    );
    assert_eq!(winner.spawn_rules()[0].herds[0].min_size, 2);
    let duplicates: Vec<_> = registry
        .diagnostics()
        .filter(|d| matches!(d.code.id(), "core/duplicate_identifier"))
        .collect();
    assert_eq!(duplicates.len(), 1);
    assert_eq!(duplicates[0].severity, Severity::Info);
    assert_eq!(
        duplicates[0].location.path.as_ref().map(PackPath::as_str),
        Some("definitions/spawn_groups/a.json")
    );
}

#[test]
fn sg42_duplicate_identifier_deviation() {
    // Vanilla replaces in unsorted directory order, silently; ours sorts by file name and warns.
    let stack = groups(
        pack(vec![]),
        vec![
            ("definitions/spawn_groups/z.json", group("x:g", "1.11.0", 9)),
            ("definitions/spawn_groups/m.json", group("x:g", "1.11.0", 5)),
        ],
    );
    let registry = stack.spawn_groups();
    let winner = registry.get(&SpawnGroupId::parse("x:g")).unwrap();
    assert_eq!(
        winner.origin().path.as_str(),
        "definitions/spawn_groups/z.json"
    );
}

#[test]
fn sg43_failed_duplicate() {
    let stack = groups(
        pack(vec![]),
        vec![
            ("definitions/spawn_groups/a.json", group("x:g", "1.11.0", 1)),
            ("definitions/spawn_groups/b.json", group("x:g", "1.7.0", 2)),
        ],
    );
    let registry = stack.spawn_groups();
    let winner = registry.get(&SpawnGroupId::parse("x:g")).unwrap();
    assert_eq!(
        winner.origin().path.as_str(),
        "definitions/spawn_groups/a.json"
    );
    assert_eq!(registry.rejected().len(), 1);
    assert!(
        registry.rejected()[0]
            .diagnostics
            .iter()
            .any(|d| matches!(d.code.id(), "core/no_parser_for_version"))
    );
}

#[test]
fn sg44_no_pack_source() {
    let stack = groups(
        pack(vec![
            (
                "spawn_groups/raid_group_1.json",
                group("x:pack", "1.11.0", 1),
            ),
            (
                "definitions/spawn_groups/raid_group_1.json",
                group("x:pack2", "1.11.0", 1),
            ),
        ]),
        vec![],
    );
    let registry = stack.spawn_groups();
    assert!(registry.is_empty());
    assert!(registry.rejected().is_empty());
}

#[test]
fn sg38_a_group_without_conditions_is_not_registered() {
    let stack = groups(
        pack(vec![]),
        vec![(
            "definitions/spawn_groups/a.json",
            br#"{"format_version": "1.11.0", "minecraft:spawn_groups": {"description": {"identifier": "x:empty"}}}"#.to_vec(),
        )],
    );
    let registry = stack.spawn_groups();
    assert!(!registry.contains(&SpawnGroupId::parse("x:empty")));
    assert!(registry.rejected().is_empty());
    assert!(
        registry
            .diagnostics()
            .any(|d| matches!(d.code.id(), "core/empty_document"))
    );
}

/// The raid groups vanilla spawns for a raid at `difficulty`: `minecraft:raid_group_<wave>`,
/// waves 0 up to 3, 5 or 7.
fn raid_waves(difficulty: Difficulty) -> Vec<SpawnGroupId> {
    let waves = match difficulty {
        Difficulty::Easy => 3,
        Difficulty::Normal => 5,
        Difficulty::Hard => 7,
        _ => 0,
    };
    (0..waves)
        .map(|wave| SpawnGroupId::parse(format!("minecraft:raid_group_{wave}")))
        .collect()
}

/// Vanilla's `raid_group_1.json` (GDK retail 1.26.45 `data/definitions/spawn_groups/`, byte-identical
/// in 1.26.20 and BDS 1.26.36), condensed to one line.
const RAID_GROUP_1: &str = r#"{"format_version": "1.11.0", "minecraft:spawn_groups": {"description": {"identifier": "minecraft:raid_group_1"}, "conditions": [{"minecraft:mob_event_filter": {"event": "minecraft:village_raid"}, "minecraft:distance_filter": {"min": 0, "max": 1024}, "minecraft:spawns_on_surface": {}, "minecraft:difficulty_filter": {"min": "easy", "max": "hard"}, "minecraft:herd": {"min_size": 3, "max_size": 3, "event": "minecraft:spawn_for_raid"}, "minecraft:permute_type": [{"weight": 100, "entity_type": "minecraft:pillager"}]}, {"minecraft:mob_event_filter": {"event": "minecraft:village_raid"}, "minecraft:distance_filter": {"min": 0, "max": 1024}, "minecraft:spawns_on_surface": {}, "minecraft:difficulty_filter": {"min": "easy", "max": "hard"}, "minecraft:herd": {"min_size": 2, "max_size": 2, "event": "minecraft:spawn_for_raid"}, "minecraft:permute_type": [{"weight": 100, "entity_type": "minecraft:vindicator"}]}]}}"#;

/// The checks of `sg01` on one vanilla group.
fn check_raid_group(group: &SpawnGroup) {
    for rules in group.spawn_rules() {
        assert_eq!(
            (rules.spawn_distance_min, rules.spawn_distance_max),
            (0, 1024)
        );
        assert_eq!(
            (rules.difficulty_min, rules.difficulty_max),
            (Difficulty::Easy, Difficulty::Hard)
        );
        assert!(rules.surface_spawner);
        assert_eq!(rules.mob_event, "minecraft:village_raid");
        assert_eq!(rules.permutations.len(), 1);
        assert_eq!(rules.permutations[0].0, 100);
        assert_eq!(rules.herds.len(), 1);
        assert_eq!(rules.herds[0].min_size, rules.herds[0].max_size);
    }
}

#[test]
fn sg01_vanilla_raid_group_1_inline() {
    let stack = groups(
        pack(vec![]),
        vec![(
            "definitions/spawn_groups/raid_group_1.json",
            RAID_GROUP_1.as_bytes().to_vec(),
        )],
    );
    let registry = stack.spawn_groups();
    let group = registry
        .get(&SpawnGroupId::parse("minecraft:raid_group_1"))
        .unwrap();
    assert!(group.diagnostics().is_empty(), "{:#?}", group.diagnostics());
    assert_eq!(group.spawn_rules().len(), 2);
    check_raid_group(group);
}

/// The seven vanilla groups from the corpus (`gdk-retail/definitions/spawn_groups`), or `None`
/// when `BEDROCK_ADDON_CORPUS` is not set.
fn vanilla_groups() -> Option<PackStack<Behavior>> {
    let Some(corpus) = std::env::var_os("BEDROCK_ADDON_CORPUS").map(PathBuf::from) else {
        eprintln!("skipped: BEDROCK_ADDON_CORPUS is not set");
        return None;
    };
    let dir = corpus.join("gdk-retail/definitions/spawn_groups");
    let mut files = Vec::new();
    for entry in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        files.push((
            format!("definitions/spawn_groups/{name}"),
            std::fs::read(&path).unwrap(),
        ));
    }
    let files: Vec<(&str, Vec<u8>)> = files.iter().map(|(p, b)| (p.as_str(), b.clone())).collect();
    Some(groups(pack(vec![]), files))
}

#[test]
fn sg01_vanilla_raid_groups() {
    let Some(stack) = vanilla_groups() else {
        return;
    };
    let registry = stack.spawn_groups();
    assert_eq!(registry.len(), 7);
    assert!(registry.rejected().is_empty());
    assert_eq!(
        registry.diagnostics().count(),
        0,
        "{:#?}",
        registry.diagnostics().collect::<Vec<_>>()
    );
    let counts: Vec<usize> = (0..7)
        .map(|g| {
            let id = SpawnGroupId::parse(format!("minecraft:raid_group_{g}"));
            let group = registry.get(&id).unwrap_or_else(|| panic!("{id:?}"));
            check_raid_group(group);
            group.spawn_rules().len()
        })
        .collect();
    assert_eq!(counts, [1, 2, 2, 2, 4, 3, 5]);
}

#[test]
fn sg02_raid_wave_ids() {
    assert_eq!(raid_waves(Difficulty::Easy).len(), 3);
    assert_eq!(raid_waves(Difficulty::Normal).len(), 5);
    assert_eq!(raid_waves(Difficulty::Hard).len(), 7);
    let Some(stack) = vanilla_groups() else {
        return;
    };
    let registry = stack.spawn_groups();
    for difficulty in [Difficulty::Easy, Difficulty::Normal, Difficulty::Hard] {
        for id in raid_waves(difficulty) {
            assert!(registry.contains(&id), "{difficulty:?}: {id:?}");
        }
    }
}
