//! The game's link pass, `LinkedAssetValidator`: the three test suites
//! of 1.26.60 over their packs (`LinkedAssetValidatorFunctionalTests`,
//! copied to `tests/fixtures/link/linked_asset_validator`). `…MissingReferences…` expects each
//! of 76 texts, `<breadcrumb>:  Missing referenced asset <name>` or `…:  Invalid asset path
//! <name>` (two spaces), in the content log; `…ValidReferences…` (the referencing packs with
//! the packs that define every asset) and `…VanillaReferences…` (references to the game's own
//! content) expect none.

use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::Diagnostics;
use bedrock_addon::load::{Experiment, LoadOptions};
use bedrock_addon::pack::{LinkOptions, OpenOptions, Pack};
use bedrock_addon::storage::PackPath;

use super::common::{hosted, stacks_of};

/// The fixture folder.
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/link/linked_asset_validator")
}

/// Every file below `dir`, by its path relative to `dir`.
fn files_below(dir: &Path, base: &Path, out: &mut Vec<(String, Vec<u8>)>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|entry| entry.expect("an entry").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            files_below(&path, base, out);
        } else {
            let relative = path
                .strip_prefix(base)
                .expect("below the base")
                .to_string_lossy()
                .replace('\\', "/");
            out.push((relative, std::fs::read(&path).expect("readable")));
        }
    }
}

/// The pack in the fixture folder `name`.
fn pack(name: &str) -> Pack {
    pack_without(name, &[])
}

/// The pack in the fixture folder `name` without the files `left_out`.
fn pack_without(name: &str, left_out: &[&str]) -> Pack {
    let dir = root().join(name);
    let mut files = Vec::new();
    files_below(&dir, &dir, &mut files);
    let files: Vec<(PackPath, Vec<u8>)> = files
        .into_iter()
        .filter(|(path, _)| !left_out.contains(&path.as_str()))
        .filter_map(|(path, bytes)| PackPath::new(&path).ok().map(|path| (path, bytes)))
        .collect();
    Pack::from_files(files, &OpenOptions::default()).expect("the pack opens")
}

/// The link findings of a world of the behaviour packs `bp` and the resource packs `rp`
/// (top first) with `load`.
fn linked(bp: &[&str], rp: &[&str], load: LoadOptions) -> Diagnostics {
    linked_packs(
        bp.iter().map(|name| pack(name)).collect(),
        rp.iter().map(|name| pack(name)).collect(),
        load,
    )
}

/// [`linked`] over packs already opened.
fn linked_packs(bp: Vec<Pack>, rp: Vec<Pack>, load: LoadOptions) -> Diagnostics {
    let set = stacks_of(bp, rp, load);
    // The camera loader and the catalogue's merge tell their references with the stack's
    // load (the game's validator runs after the load and sees them all): their findings join
    // the pass's.
    let mut found = set.link_with(&LinkOptions::default());
    for finding in set
        .behavior
        .cameras()
        .diagnostics()
        .chain(set.behavior.crafting_catalog().diagnostics())
    {
        found.push(finding.clone());
    }
    found
}

/// The oracle's 76 resolved texts, read from the harvest (`## RESOLVED expected messages`).
fn expected() -> Vec<String> {
    let path = root().join("missing_references.txt");
    let text = std::fs::read_to_string(&path).expect("the oracle");
    let resolved = text
        .split_once("## RESOLVED expected messages")
        .expect("the resolved section")
        .1;
    resolved
        .lines()
        .filter(|line| line.starts_with("0x"))
        .map(|line| line.split_once(": ").expect("VA: text").1.to_owned())
        .collect()
}

/// Whether the finding text `found` carries the oracle text `expected` (the game's check is
/// a substring match; an argument the harvest could not resolve, `?`, matches any name).
fn carries(found: &str, expected: &str) -> bool {
    match expected.strip_suffix(" ?") {
        Some(prefix) => found
            .find(prefix)
            .is_some_and(|at| !found[at + prefix.len()..].trim().is_empty()),
        None => found.contains(expected),
    }
}

/// Whether a finding text is one of the validator's.
fn validator_text(found: &str) -> bool {
    found.contains(":  Missing referenced asset ") || found.contains(":  Invalid asset path ")
}

/// A host that knows none of the referenced names: every view is whole, so the atlas keys
/// are judged too (the game's own atlases are loaded). The suites turn experiments on; the
/// 1.21.60 spawn rules of the packs need Simplified Spawn Rules.
fn game() -> LoadOptions {
    let mut options = hosted(&[]);
    options.experiments.insert(Experiment::SimplifiedSpawnRules);
    options
}

#[test]
fn every_expected_text_of_the_missing_references_suite_is_one_finding() {
    const COPY: &str = "spawn_rules/legacy_version_spawn_rules_with_references copy.json";

    let found = linked(&["UsesReferencesBP"], &["UsesReferencesRP"], game());
    let texts: Vec<String> = found.iter().map(|f| f.code.to_string()).collect();
    let expected = expected();
    assert_eq!(expected.len(), 76);
    // Each expected text is a finding (the game's check is "the log contains it"; a text the
    // packs hold in several files is one finding in each).
    let missing: Vec<&String> = expected
        .iter()
        .filter(|e| !texts.iter().any(|t| carries(t, e)))
        .collect();
    // The two spawn rules files of the pack define one identifier: the crate links the
    // effective definition (the legacy copy, read last); the game's validator holds the
    // references of every file it loaded. The shadowed file's text is one finding in a world
    // without the copy.
    let alone = linked_packs(
        vec![pack_without("UsesReferencesBP", &[COPY])],
        vec![pack("UsesReferencesRP")],
        game(),
    );
    let alone: Vec<String> = alone.iter().map(|f| f.code.to_string()).collect();
    let (shadowed, missing): (Vec<&String>, Vec<&String>) = missing
        .into_iter()
        .partition(|e| alone.iter().any(|t| carries(t, e)));
    assert_eq!(
        shadowed,
        [
            "-> conditions[0] -> permute_types[0] -> entity_type:  Missing referenced asset test:entity"
        ]
    );
    assert!(
        missing.is_empty(),
        "{} of 76 texts not found: {missing:#?}\nvalidator texts found: {:#?}",
        missing.len(),
        texts
            .iter()
            .filter(|t| validator_text(t))
            .collect::<Vec<_>>()
    );
    // Each carries the game's severity, a content-log error, and a code of a
    // miss.
    let wrong_grade: Vec<String> = found
        .iter()
        .filter(|f| validator_text(&f.code.to_string()))
        // The item icon (R32) is judged by the view of the item atlas, which the game checks on
        // a client alone (`LinkedAssetValidator(bool isClient)`); its grading is the atlas
        // rules'.
        .filter(|f| {
            !f.code
                .to_string()
                .contains("-> components -> minecraft:icon:")
        })
        .filter(|f| {
            f.severity != bedrock_addon::diagnostic::Severity::Error
                || f.code.kind() != bedrock_addon::diagnostic::Kind::UnresolvedReference
        })
        .map(|f| format!("{:?} {} {}", f.severity, f.code.id(), f.code))
        .collect();
    assert_eq!(wrong_grade, Vec::<String>::new());
    // And one finding per place: no file and member carries the same line twice.
    let mut places: Vec<(String, String, String)> = found
        .iter()
        .filter(|f| validator_text(&f.code.to_string()))
        .map(|f| {
            (
                f.location
                    .path
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                f.location.json_path.to_string(),
                f.code.to_string(),
            )
        })
        .collect();
    let all = places.len();
    places.sort();
    places.dedup();
    assert_eq!(
        places.len(),
        all,
        "a line told twice at one place: {places:#?}"
    );
}

#[test]
fn the_valid_references_suite_has_no_validator_text() {
    let found = linked(
        &["UsesReferencesBP", "ReferencedAssetsBP"],
        &["UsesReferencesRP", "ReferencedAssetsRP"],
        game(),
    );
    let validator: Vec<String> = found
        .iter()
        .map(|f| format!("{} | {}", f.location.json_path, f.code))
        .filter(|t| validator_text(t))
        .collect();
    // The catalogue's merge resolves names against the stack's items and blocks: the spawn
    // egg the game registers for the spawnable `test:entity` is no item to it yet (the link
    // pass derives it).
    assert_eq!(
        validator,
        [
            "minecraft:crafting_items_catalog/categories/0/groups/0/items/1 | -> categories[0] -> groups[0] -> items[1]:  Missing referenced asset test:entity_spawn_egg"
        ]
    );
}

#[test]
fn the_vanilla_references_suite_has_no_validator_text() {
    // The game's own content the packs name, as a host that has the game's packs answers:
    // every name of the `minecraft` namespace (a bare name is one, canonically), and the
    // names of the game's resource and behaviour packs that have no namespace.
    let mut load = LoadOptions {
        built_ins: Some(std::sync::Arc::new(Game)),
        ..LoadOptions::default()
    };
    load.experiments = game().experiments;
    let found = linked(
        &["UsesVanillaReferencesBP"],
        &["UsesVanillaReferencesRP"],
        load,
    );
    let validator: Vec<String> = found
        .iter()
        .map(|f| {
            format!(
                "{} | {} | {}",
                f.location
                    .path
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                f.location.json_path,
                f.code
            )
        })
        .filter(|t| validator_text(t))
        .collect();
    assert_eq!(validator, Vec::<String>::new());
}

/// The game's own content of the vanilla suite's packs.
struct Game;

impl bedrock_addon::load::BuiltIns for Game {
    fn contains(&self, kind: &'static str, id: &str) -> bool {
        const UNNAMESPACED: [(&str, &str); 10] = [
            // A template location without a namespace is the game's file of that path.
            ("structure", "trial_chambers/chamber/chamber_1"),
            ("loot table", "loot_tables/entities/fish.json"),
            ("named sound", "ambient.crimson_forest.additions"),
            ("named sound", "ambient.crimson_forest.loop"),
            ("named sound", "ambient.crimson_forest.mood"),
            ("music definition", "crimson_forest"),
            ("texture", "textures/items/apple"),
            ("texture", "textures/particle/campfire_smoke"),
            ("atlas key", "cobblestone"),
            ("atlas key", "cake"),
        ];
        id.starts_with("minecraft:") || UNNAMESPACED.contains(&(kind, id))
    }
}

/// One finding of `found` at `path` in `file`, as `(code, severity, text)`.
fn at(
    found: &Diagnostics,
    file: &str,
    path: &str,
) -> Vec<(&'static str, bedrock_addon::diagnostic::Severity, String)> {
    found
        .iter()
        .filter(|f| {
            f.location.path.as_ref().is_some_and(|p| p.as_str() == file)
                && f.location.json_path.to_string() == path
                && f.code.kind() == bedrock_addon::diagnostic::Kind::UnresolvedReference
        })
        .map(|f| (f.code.id(), f.severity, f.code.to_string()))
        .collect()
}

/// The three further references on both sides: `item_specific_speeds[].item`, the
/// catalogue's icon and items, `biome_music.music_definition` — missing in the referencing
/// packs alone, defined with the referenced assets (code, severity, path and text).
#[test]
fn the_new_references_on_both_sides() {
    use bedrock_addon::diagnostic::Severity;
    let alone = linked(&["UsesReferencesBP"], &["UsesReferencesRP"], game());
    let defined = linked(
        &["UsesReferencesBP", "ReferencedAssetsBP"],
        &["UsesReferencesRP", "ReferencedAssetsRP"],
        game(),
    );
    let cases = [
        (
            "blocks/block_with_references.json",
            "minecraft:block/components/minecraft:destructible_by_mining/item_specific_speeds/0/item",
            "-> components -> minecraft:destructible_by_mining -> item_specific_speeds[0] -> item:  Missing referenced asset test:item",
            "core/unresolved_reference_worded",
        ),
        (
            "item_catalog/crafting_item_catalog.json",
            "minecraft:crafting_items_catalog/categories/0/groups/0/group_identifier/icon",
            "-> categories[0] -> groups[0] -> group_identifier -> icon:  Missing referenced asset test:item",
            "crafting_catalog/missing_item",
        ),
        (
            "item_catalog/crafting_item_catalog.json",
            "minecraft:crafting_items_catalog/categories/0/groups/0/items/0",
            "-> categories[0] -> groups[0] -> items[0]:  Missing referenced asset test:item",
            "crafting_catalog/missing_item",
        ),
        (
            "biomes/biome_with_references.client_biome.json",
            "minecraft:client_biome/components/minecraft:biome_music/music_definition",
            "-> components -> minecraft:biome_music -> music_definition:  Missing referenced asset test_music",
            "core/unresolved_reference_worded",
        ),
    ];
    for (file, path, text, code) in cases {
        assert_eq!(
            at(&alone, file, path),
            [(code, Severity::Error, text.to_owned())],
            "{path}"
        );
        assert_eq!(at(&defined, file, path), [], "{path}");
    }
}
