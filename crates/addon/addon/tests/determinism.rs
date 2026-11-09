//! The determinism rule: a loader's result is identical for every order in which
//! the caller completes requests and jobs. One fixture stack is loaded with completions in
//! issue order, in reverse and in three seeded random orders; the `Debug` of the `StackReport`
//! and of both stacks (packs and every loaded collection) must be equal. The collections are
//! compared one by one, by the comparison the driver crate's strategy tests use too
//! (`common/collections.rs`, which says why a collection's `Debug` is its iteration order).
//! Where a `Debug` prints less than its value holds, the part it leaves out is compared too
//! (`common/unprinted.rs`).
//!
//! The stack covers every family with a stack accessor: the
//! collections are the stacks' own list, and each must hold something a fragment gave it
//! (`every_collection_of_both_stacks_holds_something_of_a_fragment`).
//!
//! The stack is a minimal behaviour and resource pack of this test plus every pack fragment a
//! family left under `tests/fixtures/<module>/<family>/determinism/`: its files are pack-relative
//! paths, added to both packs, except those under `behavior_pack/` or `resource_pack/`, which go
//! to that pack only, and those under `app_data/`, which are app data
//! (`LoadOptions::app_data`) and belong to no pack. One more fragment is this test's own
//! (`UNREAD`).
//!
//! Two passes: the default options, where a gated family contributes what its closed
//! gate leaves and the app data is absent; and every experiment and toggle on with the app-data
//! fragments, where the gated documents (POI, simplified spawn rules, a loot function behind
//! Upcoming Creator Features) and the app-data ones (spawn groups, features, feature rules) load.

#[path = "common/responder.rs"]
mod responder;
// The comparison the corpus strategy tests of the driver crate use too: one text in both
// packages (`collections.rs` says how it is kept so).
#[path = "common/collections.rs"]
mod collections;
// What a `Debug` leaves out, for the same comparison. The file names the content modules:
// each family's block of it is compiled where the family's feature is on.
macro_rules! family {
    ($feature:literal => $body:block) => {
        #[cfg(feature = $feature)]
        $body
    };
    ($feature:literal => $item:item) => {
        #[cfg(feature = $feature)]
        $item
    };
}
#[path = "common/unprinted.rs"]
mod unprinted;

use std::path::{Path, PathBuf};

use bedrock_addon::load::{
    AppData, CollectionSet, Experiment, Experiments, LoadOptions, RepositoryLoader, Residency,
    RootId, Source, StackLoader, StackOptions,
};
use bedrock_addon::pack::{OpenOptions, PackOrigin, StackSet, WorldPacks};
use bedrock_addon::storage::PackPath;
use collections::Snapshot;
use responder::{Order, Responder};

/// Every file below `dir`, as `(path relative to dir, bytes)`, sorted.
fn files_below(dir: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push((relative, std::fs::read(&path).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

/// The `determinism/` fragments of every family, sorted by path.
fn fragments() -> Vec<PathBuf> {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut found = Vec::new();
    for module in std::fs::read_dir(&fixtures).unwrap() {
        let module = module.unwrap().path();
        if !module.is_dir() {
            continue;
        }
        for family in std::fs::read_dir(&module).unwrap() {
            let fragment = family.unwrap().path().join("determinism");
            if fragment.is_dir() {
                found.push(fragment);
            }
        }
    }
    found.sort();
    // This test's own fragment, last.
    found.push(fixtures.join(UNREAD));
    found
}

/// The fragment of this test alone, below `tests/fixtures/`: a file of `point_lights/`, which
/// no document type reads, a POI tag whose parent names no tag, and under `sounds/` an
/// audio file another extension hides (`click.wav` beside `click.ogg`), a file the client
/// never looks for (`notes.txt`) and an image whose extension is in the wrong case
/// (`demo_upper.PNG`). They fill the
/// members that hold such files and the group findings they cause. They are in no family's
/// `determinism/`, because the writer's round-trip suite loads every family's fragment and
/// requires each of its JSON files to be some type's document that loads cleanly.
const UNREAD: &str = "determinism/unread";

/// The manifest of pack `n` of this test: a module of type `script_module` that starts
/// `scripts/main.js` (the script bundle fragment's entry file), then the module that makes
/// the pack a behaviour (`data`) or a resource (`resources`) pack; the last module decides the
/// pack type. `@minecraft/server` is the one module the entry file may import from the game.
fn pack_manifest(n: u128, script_module: &str, module: &str) -> String {
    format!(
        r#"{{
  "format_version": 2,
  "header": {{ "name": "pack {n}", "description": "", "uuid": "{}",
              "version": [1, 0, 0], "min_engine_version": [1, 21, 0] }},
  "modules": [
    {{ "type": "{script_module}", "language": "javascript", "entry": "scripts/main.js",
      "uuid": "{}", "version": [1, 0, 0] }},
    {{ "type": "{module}", "uuid": "{}", "version": [1, 0, 0] }}
  ],
  "dependencies": [{{ "module_name": "@minecraft/server", "version": "2.0.0" }}]
}}"#,
        uuid::Uuid::from_u128(n),
        uuid::Uuid::from_u128(n | 0xeeee_0000_0000_0000_0000),
        uuid::Uuid::from_u128(n | 0xffff_0000_0000_0000_0000),
    )
}

/// The two packs of this test without any fragment, at root 1: `bp/` and `rp/`.
fn bare_stack() -> Responder {
    let mut responder = Responder::new(Order::Issue);
    let archive = brarchive::serialize([("note.txt", &b"archived"[..])]).unwrap();
    responder
        .file(1, "bp/manifest.json", pack_manifest(1, "script", "data"))
        .file(1, "bp/texts/en_US.lang", &b"pack.name=Determinism\n"[..])
        .file(1, "bp/__brarchive/extra.brarchive", archive)
        .file(
            1,
            "rp/manifest.json",
            pack_manifest(2, "client_script", "resources"),
        )
        .file(1, "rp/texts/en_US.lang", &b"pack.name=Determinism\n"[..]);
    responder
}

/// The packs directory at root 1: `bp/` and `rp/`, with every family's fragment.
fn fixture_stack() -> Responder {
    fixture_stack_without(&[])
}

/// [`fixture_stack`] without the fragment files whose path (below the fragment's
/// `determinism/`) is one of `withheld`.
fn fixture_stack_without(withheld: &[&str]) -> Responder {
    stack_of(&fragment_files(&|path, bytes| {
        (!withheld.contains(&path)).then_some(bytes)
    }))
}

/// What a fragment file becomes in a fixture: its path below the fragment's `determinism/`
/// and its bytes to other bytes, or `None` for a file that is left out.
type Rewrite<'a> = &'a dyn Fn(&str, Vec<u8>) -> Option<Vec<u8>>;

/// The files of every fragment, each as its path below the fragment's `determinism/` and what
/// `rewrite` makes of its bytes, in fragment and then path order.
fn fragment_files(rewrite: Rewrite<'_>) -> Vec<(String, Vec<u8>)> {
    fragments()
        .iter()
        .flat_map(|fragment| files_below(fragment))
        .filter_map(|(path, bytes)| {
            let bytes = rewrite(&path, bytes)?;
            Some((path, bytes))
        })
        .collect()
}

/// The packs directory at root 1: `bp/` and `rp/`, with the pack files of `files`.
fn stack_of(files: &[(String, Vec<u8>)]) -> Responder {
    let mut responder = bare_stack();
    for (path, bytes) in files {
        if path.starts_with(APP_DATA) {
            // App data belongs to no pack: `app_data_of`.
        } else if let Some(rest) = path.strip_prefix("behavior_pack/") {
            responder.file(1, &format!("bp/{rest}"), bytes.clone());
        } else if let Some(rest) = path.strip_prefix("resource_pack/") {
            responder.file(1, &format!("rp/{rest}"), bytes.clone());
        } else {
            responder.file(1, &format!("bp/{path}"), bytes.clone());
            responder.file(1, &format!("rp/{path}"), bytes.clone());
        }
    }
    responder
}

/// The folder of a fragment whose files are app data.
const APP_DATA: &str = "app_data/";

/// The app-data files of every fragment: what lies below its `app_data/`.
fn app_data_fragments() -> AppData {
    app_data_of(&fragment_files(&|_, bytes| Some(bytes)))
}

/// The app data of `files`: what lies below a fragment's `app_data/`.
fn app_data_of(files: &[(String, Vec<u8>)]) -> AppData {
    AppData::from_files(files.iter().filter_map(|(path, bytes)| {
        let rest = path.strip_prefix(APP_DATA)?;
        Some((
            PackPath::new(rest).expect("an app-data path"),
            bytes.clone(),
        ))
    }))
}

/// The fixture stack and the second pass's options with every fragment file, pack file and
/// app data alike, as `rewrite` leaves it.
fn rewritten_fixture(rewrite: Rewrite<'_>) -> (Responder, LoadOptions) {
    let files = fragment_files(rewrite);
    let options = LoadOptions {
        app_data: Some(app_data_of(&files)),
        ..everything_on()
    };
    (stack_of(&files), options)
}

/// The options of the second pass: every experiment a world can turn on, the two toggles, and
/// the app-data fragments.
fn everything_on() -> LoadOptions {
    let mut experiments = Experiments::new();
    for experiment in Experiment::ALL {
        // An experiment without a save name is one vanilla never turns on.
        if experiment.save_name().is_some() {
            experiments.insert(*experiment);
        }
    }
    LoadOptions {
        experiments,
        beta_apis: true,
        upcoming_creator_features: true,
        app_data: Some(app_data_fragments()),
        ..LoadOptions::default()
    }
}

/// The world list naming both packs.
fn world() -> WorldPacks {
    world_of(&[1], &[2])
}

/// The world list of the behaviour packs and of the resource packs with these numbers
/// (`pack_manifest`), the first on top.
fn world_of(behavior: &[u128], resource: &[u128]) -> WorldPacks {
    let list = |packs: &[u128]| {
        let entries: Vec<String> = packs
            .iter()
            .map(|n| {
                format!(
                    r#"{{"pack_id": "{}", "version": [1, 0, 0]}}"#,
                    uuid::Uuid::from_u128(*n)
                )
            })
            .collect();
        format!("[{}]", entries.join(", "))
    };
    WorldPacks::parse(
        Some(list(behavior).as_bytes()),
        Some(list(resource).as_bytes()),
    )
    .unwrap()
}

/// The repository and the stack load with `options`, both completed in `order`: the texts
/// the shared comparison takes (the report in load order and as displayed, and each stack with
/// its collections one by one), and the `Debug` of the whole set.
fn snapshot(
    template: &Responder,
    order: Order,
    residency: Residency,
    options: &LoadOptions,
) -> (Snapshot, String) {
    let set = loaded(template, order, residency, options);
    (snapshot_of(&set), format!("{set:?}"))
}

/// The texts of `set` with every collection loaded, and what their `Debug` leaves out, of the
/// families compiled in.
fn snapshot_of(set: &StackSet) -> Snapshot {
    let report = set.load_all();
    Snapshot::of(set, &report).with_unprinted(unprinted::of(set))
}

/// Both stacks of `template`, the repository and the stack load completed in `order`.
fn loaded(
    template: &Responder,
    order: Order,
    residency: Residency,
    options: &LoadOptions,
) -> StackSet {
    loaded_in(template, order, residency, options, &world())
}

/// [`loaded`] for the packs `world` lists.
fn loaded_in(
    template: &Responder,
    order: Order,
    residency: Residency,
    options: &LoadOptions,
    world: &WorldPacks,
) -> StackSet {
    let mut responder = template.reordered(order);
    let mut repository = RepositoryLoader::new(
        vec![Source::Directory {
            root: RootId(1),
            origin: PackOrigin::World,
        }],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut repository);
    let set = repository.stacks(world, options.clone());
    let options = StackOptions {
        behavior: residency,
        resource: residency,
        ..StackOptions::default()
    };
    let mut loader = StackLoader::new(set, options);
    responder.drive(&mut loader)
}

/// The five completion orders: issue, reverse and three seeded random ones.
/// The seeds are the responder's constants; this is their pin.
const ORDERS: [Order; 5] = [
    Order::Issue,
    Order::Reverse,
    Order::Seeded(1),
    Order::Seeded(0x5eed),
    Order::Seeded(0xdead_beef),
];

#[test]
fn the_five_orders_are_issue_reverse_and_three_seeds() {
    assert_eq!(Order::ALL, ORDERS);
}

/// Loads `template` with `options` in the five orders under both residencies and requires
/// every load to be the one in issue order: the same report (`Debug`, the findings in load
/// order, and `Display`), the same `Debug` of every collection of both stacks, compared one by
/// one so that a failure names the collection (`collections.rs` says why that text is the
/// collection's iteration order), and the same `Debug` of the whole set.
fn assert_every_order_agrees(template: &Responder, options: &LoadOptions) {
    assert_every_order_agrees_in(template, options, &world());
}

/// [`assert_every_order_agrees`] for the packs `world` lists.
fn assert_every_order_agrees_in(template: &Responder, options: &LoadOptions, world: &WorldPacks) {
    let snapshot = |order: Order, residency: Residency| {
        let set = loaded_in(template, order, residency, options, world);
        (snapshot_of(&set), format!("{set:?}"))
    };
    for residency in [Residency::Transient, Residency::All] {
        let (reference, reference_set) = snapshot(Order::Issue, residency);
        for order in ORDERS {
            let (found, set) = snapshot(order, residency);
            collections::assert_same(&format!("{order:?}, {residency:?}"), &found, &reference);
            assert_eq!(set, reference_set, "{order:?}, {residency:?}");
        }
    }
}

#[test]
fn every_completion_order_gives_the_same_report_and_collections() {
    assert_every_order_agrees(&fixture_stack(), &LoadOptions::default());
}

/// What the second pass loads that the default pass cannot, each as the collection it is in
/// and a text only the loaded document has in that collection's `Debug`: the registry entry of
/// a document behind an experiment (the POI fragment) and of the app-data documents
/// (spawn groups; features and feature rules), the simplified spawn rules that
/// the Simplified Spawn Rules experiment routes (the type, since the classic parser loads the
/// same file without it), a loot function behind Upcoming Creator Features (its variant, since
/// without the toggle it is dropped and the table loads), and a name of the game's own
/// credits, which only the app data has.
///
/// An identifier alone is no proof: a file that is refused names its identifier in the
/// finding's text. `document` asks for the registry entry, `<identifier>: Document {`.
fn gated() -> Vec<(&'static str, String)> {
    // The entry of `identifier` in a registry's `Debug`, as a stack's `Debug` holds it (the
    // text of a string literal: the quotes are escaped).
    #[allow(dead_code, reason = "used behind the families' features")]
    fn document(identifier: &str) -> String {
        format!("(\\\"{identifier}\\\"): Document {{")
    }
    #[allow(unused_mut, reason = "every entry is behind its family's feature")]
    let mut gated: Vec<(&'static str, String)> = Vec::new();
    #[cfg(feature = "poi")]
    gated.push(("Poi", document("determinism:forge")));
    #[cfg(all(feature = "spawn_rules", feature = "spawn_groups"))]
    gated.extend([
        ("SpawnGroups", document("determinism:raiders")),
        (
            "SpawnRules",
            String::from("Simplified(SimplifiedSpawnRules {"),
        ),
    ]);
    #[cfg(feature = "loot")]
    gated.push(("LootTables", String::from("CarryOverBlockEntityData {")));
    #[cfg(all(feature = "feature", feature = "feature_rules"))]
    gated.extend([
        // An app-data feature is in the `minecraft` namespace; `determinism:app_ore` of the
        // same folder is refused for it, in every order.
        ("Features", document("minecraft:determinism_app_ore")),
        ("FeatureRules", document("determinism:app_rule")),
    ]);
    // The game's own credits (`app_data/credits/credits.json` of the rp_misc fragment): a
    // name of its one title.
    #[cfg(feature = "credits")]
    gated.push(("Credits", String::from("Determinism Credit")));
    gated
}

/// Whether the collection `name` of `snapshot` (of either stack) has `marker` in its `Debug`.
fn holds(snapshot: &Snapshot, name: &str, marker: &str) -> bool {
    snapshot
        .collections()
        .iter()
        .any(|entry| entry.name == name && entry.text.contains(marker))
}

#[test]
fn every_completion_order_gives_the_same_result_with_experiments_and_app_data() {
    let template = fixture_stack();
    let options = everything_on();
    assert_every_order_agrees(&template, &options);
    for residency in [Residency::Transient, Residency::All] {
        // The pass is not the default one again: the gated documents are in the collections.
        let (loaded, _) = snapshot(&template, Order::Issue, residency, &options);
        let (default, _) = snapshot(&template, Order::Issue, residency, &LoadOptions::default());
        for (collection, marker) in gated() {
            assert!(
                holds(&loaded, collection, &marker),
                "{collection}: {marker} is loaded"
            );
            assert!(
                !holds(&default, collection, &marker),
                "{collection}: {marker} needs the pass"
            );
        }
    }
}

#[test]
fn a_fragment_s_app_data_folder_is_app_data_and_no_pack_file() {
    let app_data = app_data_fragments();
    let paths: Vec<String> = app_data.iter().map(|(path, _)| path.to_string()).collect();
    assert!(
        paths.iter().all(|path| path.starts_with("definitions/")
            || bedrock_addon::load::AppData::FILES.contains(&path.as_str())),
        "{paths:?}"
    );
    for path in [
        "definitions/spawn_groups/determinism.json",
        "definitions/features/determinism_app_ore.json",
        "definitions/features/determinism_app_vanilla_ore.json",
        "definitions/feature_rules/determinism_app_rule.json",
        "credits/credits.json",
    ] {
        assert!(paths.contains(&path.to_owned()), "{path}: {paths:?}");
    }
    // No pack of the fixture stack holds a copy.
    let template = fixture_stack();
    let (_, stacks) = snapshot(
        &template,
        Order::Issue,
        Residency::All,
        &LoadOptions::default(),
    );
    assert!(
        !stacks.contains("app_data/"),
        "an app-data file reached a pack"
    );
}

/// The second pass's options over an app data that holds no file: what the stack of this test
/// loads when no fragment reaches it.
fn everything_on_and_no_file() -> LoadOptions {
    LoadOptions {
        app_data: Some(AppData::from_files(Vec::<(PackPath, Vec<u8>)>::new())),
        ..everything_on()
    }
}

/// The coverage of the fixture stack ("a fixture stack
/// with every family"): after the second pass, every collection of both stacks holds something
/// a fragment gave it, member by member. The collections are the stacks' own list (every
/// accessor line compiled in, `collections.rs`), so an accessor added without a fragment fails
/// here by name.
///
/// "Holds something" is measured against the same two packs without any fragment, loaded with
/// the same options ([`holds_loaded_content`]). This is stricter than `is_empty()`: the
/// behaviour trees built into the game fill their registry on a stack without one tree file,
/// and a tree fragment is still required.
///
/// A group type is not one collection but its members ([`members`]): `Recipes` sorts one root
/// type into a registry per recipe type, `FeatureRules` into one per placement pass, `Materials`
/// into one group per list, and each of them is a collection the five orders must compare. A
/// group with a fragment for one member and none for its neighbour fails with the neighbour's
/// name.
///
/// Families without a stack accessor, by design, have no fragment and are not in the list:
///
/// - trim: `TrimData::vanilla()` is a table of the game with no pack format;
/// - persona pieces: `PersonaPiece::load_all` reads one pack that is on no stack; its own test
///   compares the load over two file orders;
/// - script modules: read from the manifest, no collection of their own; the files
///   they start are the `scripts()` bundles, which are in the list;
/// - pack settings, script config and script metadata: read from
///   files beside the packs or from the game's own tables; no stack takes part.
#[test]
fn every_collection_of_both_stacks_holds_something_of_a_fragment() {
    let missing = collections_without_a_fragment(&fixture_stack());
    assert!(
        missing.is_empty(),
        "{} collections hold nothing of a fragment:\n{}",
        missing.len(),
        missing.join("\n")
    );
}

/// The collections of `stack`, loaded with everything on, that hold nothing a fragment gave
/// them, each as a line naming the accessor's variant and its stack and, for a group, the
/// members that hold nothing.
fn collections_without_a_fragment(stack: &Responder) -> Vec<String> {
    collections_without_a_fragment_with(stack, &everything_on())
}

/// [`collections_without_a_fragment`] for a stack loaded with `options`: the second pass's,
/// with an app data of its own.
fn collections_without_a_fragment_with(stack: &Responder, options: &LoadOptions) -> Vec<String> {
    let (full, _) = snapshot(stack, Order::Issue, Residency::All, options);
    let (bare, _) = snapshot(
        &bare_stack(),
        Order::Issue,
        Residency::All,
        &everything_on_and_no_file(),
    );
    let (full, bare) = (full.collections(), bare.collections());
    // The list is every accessor compiled in: `CollectionSet::ALL` prints the variants.
    let all = format!("{:?}", CollectionSet::ALL);
    let all: Vec<&str> = all
        .trim_matches(['{', '}'])
        .split(", ")
        .filter(|name| !name.is_empty())
        .collect();
    for name in &all {
        assert!(
            full.iter().any(|entry| entry.name == *name),
            "Collection::{name} is compiled in and on neither stack's list"
        );
    }
    for entry in &full {
        assert!(all.contains(&entry.name), "{entry:?} is no collection");
    }
    assert_eq!(
        full.iter().map(|e| (e.side, e.name)).collect::<Vec<_>>(),
        bare.iter().map(|e| (e.side, e.name)).collect::<Vec<_>>(),
        "both stacks list the same accessors with and without fragments"
    );
    // Every row of `PROOFS` is a member some collection has: a row that names none is stale.
    for (collection, member, _) in PROOFS {
        let Some(entry) = full.iter().find(|entry| entry.name == *collection) else {
            // The family's feature is off.
            continue;
        };
        let text = unescaped(entry.text);
        assert!(
            members(&text).iter().any(|(name, _)| name == member),
            "PROOFS: Collection::{collection} has no member `{member}`"
        );
    }
    full.iter()
        .zip(&bare)
        .filter_map(|(full, bare)| {
            let missing =
                members_without_a_fragment(full.name, &unescaped(full.text), &unescaped(bare.text));
            let what = match missing.as_slice() {
                [] => return None,
                [whole] if whole.is_empty() => String::from("its accessor"),
                [whole] if whole.starts_with(' ') => format!("its accessor{whole}"),
                [member] => format!("its member {member}"),
                members => format!("its members {}", members.join(", ")),
            };
            Some(format!(
                "Collection::{} of the {} stack: no determinism fragment reaches {what}",
                full.name, full.side
            ))
        })
        .collect()
}

/// The members of the collection `collection` (its `Debug` is `full`; `bare` is the same
/// collection of the stack without fragments) that hold nothing a fragment gave them. A
/// collection that is no group is its one member, named `""`.
fn members_without_a_fragment(collection: &str, full: &str, bare: &str) -> Vec<String> {
    let (full, bare) = (members(full), members(bare));
    assert_eq!(
        full.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
        bare.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
        "Collection::{collection} has the same members with and without fragments"
    );
    full.iter()
        .zip(&bare)
        .filter_map(|((name, full), (_, bare))| {
            // Every row of the member: one per file that feeds it.
            let proofs: Vec<Proof> = PROOFS
                .iter()
                .filter(|(of, member, _)| *of == collection && member == name)
                .map(|(_, _, proof)| *proof)
                .collect();
            // A flag the loader sets from what the stack's packs are: the fixture stack has no
            // base-game pack, so no fragment can change it.
            if STACK_FLAGS.contains(&(collection, *name)) {
                return None;
            }
            if proofs.is_empty() && !is_documents(full) {
                // What is no registry and no path map says what it holds: a member nobody
                // wrote a row for is not covered by its neighbours, and a text that merely
                // differs from the bare stack's is no content.
                return Some(format!("{name} (it has no row in PROOFS)"));
            }
            let of = |list: &[(&str, &str)]| list.contains(&(collection, *name));
            let held = if of(APP_DATA_ALONE) {
                true
            } else if proofs.is_empty() {
                holds_loaded_content(full, bare, None)
            } else {
                proofs
                    .iter()
                    .all(|proof| holds_loaded_content(full, bare, Some(*proof)))
            };
            if !held {
                return Some((*name).to_owned());
            }
            // The app data's side, where its fragments reach.
            (of(APP_DATA_FED) && documents(full).1 == 0)
                .then(|| format!("{name} (from the app data)"))
        })
        .collect()
}

/// The members of a collection, from its `Debug` (`text`, not escaped), each as its name and
/// its `Debug`.
///
/// A group type is a struct whose fields are collections (`Recipes { crafting: Registry { … },
/// furnace: Registry { … }, … }`): its members are its fields, whatever they hold. A registry,
/// a path map, a `Merged`, and anything that is no struct (the keyed translations and the
/// script bundles print a list) is one collection: its one member has the name `""`.
fn members(text: &str) -> Vec<(&str, &str)> {
    let whole = vec![("", text)];
    if ["Registry { ", "Files { ", "Merged { "]
        .iter()
        .any(|one| text.starts_with(one))
    {
        return whole;
    }
    let Some((name, fields)) = text.split_once(" { ") else {
        return whole;
    };
    let Some(fields) = fields.strip_suffix('}') else {
        return whole;
    };
    if name.is_empty() || !name.chars().all(char::is_alphanumeric) {
        return whole;
    }
    // The fields, cut at the commas outside every bracket and string literal.
    let mut fields_cut = Vec::new();
    let (mut depth, mut in_string, mut escaped, mut from) = (0usize, false, false, 0);
    for (at, c) in fields.char_indices() {
        if in_string {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                fields_cut.push(&fields[from..at]);
                from = at + 1;
            }
            _ => {}
        }
    }
    fields_cut.push(&fields[from..]);
    fields_cut
        .into_iter()
        .map(str::trim)
        // `..` is what `finish_non_exhaustive` prints.
        .filter(|field| !field.is_empty() && *field != "..")
        .map(|field| {
            field
                .split_once(": ")
                .filter(|(name, _)| name.chars().all(|c| c.is_ascii_lowercase() || c == '_'))
                .unwrap_or_else(|| panic!("`{field}` is no field of a struct's Debug"))
        })
        .collect()
}

/// How a member that is no registry and no path map shows that it holds something a fragment
/// gave it: a text of its `Debug` that only a file read as what it is leaves there. A text that
/// merely differs from the bare stack's is no proof: a file the loader keeps and reads
/// nothing from changes the text as well.
#[derive(Clone, Copy, Debug)]
enum Proof {
    /// A text of the loaded value, looked for outside the member's findings (an identifier
    /// alone would also be in the finding that refuses its file).
    Value(&'static str),
    /// A finding, for a member that holds nothing else: its definition's id and the path of the
    /// file it is told at. The path alone would be met by the same file refused for any other reason.
    Finding(&'static str, &'static str),
}

/// The proof of every member that needs one, as `(collection, member, proof)`.
const PROOFS: &[(&str, &str, Proof)] = &[
    // The collections that are one merged file, one keyed map or one list: member `""`.
    (
        "CraftingCatalog",
        "",
        Proof::Value(r#"GroupIdentifier { name: "determinism:stones""#),
    ),
    (
        "TickFunctions",
        "",
        Proof::Value(r#"TickFunction { function: Id<function>("determinism/hello")"#),
    ),
    // The codes are `languages.json`'s, the names `language_names.json`'s.
    (
        "Languages",
        "",
        Proof::Value(r#"codes: [Language("en_US"), Language("xx_YY")]"#),
    ),
    (
        "Languages",
        "",
        Proof::Value(r#"(Language("xx_YY"), "Example")"#),
    ),
    // A string of the fragment's `.lang` file, in the language its `languages.json` adds.
    (
        "Translations",
        "",
        Proof::Value(r#"strings: {"greeting": "Hello %s"}"#),
    ),
    // An import read from the entry file's text.
    (
        "Scripts",
        "",
        Proof::Value(r#"ImportSite { kind: Static, raw: "./lib/b.js""#),
    ),
    (
        "LegacyClientBiomes",
        "",
        Proof::Value(r#"Id<biome>("minecraft:swampland"): LegacyClientBiome {"#),
    ),
    ("Ui", "", Proof::Value(r#""determinism_screen": Control {"#)),
    (
        "Splashes",
        "",
        Proof::Value(r#"splashes: Some(["Determinism!""#),
    ),
    (
        "LoadingMessages",
        "",
        Proof::Value(r#"Text("tips.determinism.1")"#),
    ),
    (
        "Credits",
        "",
        Proof::Value(r#"names: Some(["Determinism Credit""#),
    ),
    // Three path maps and registries whose reader takes any object: an entry alone would be
    // made of `{}` too.
    (
        "LootTables",
        "",
        Proof::Value(
            r#"PackPath("loot_tables/chests/determinism.json"): Document { value: LootTable { pools: Some([Pool { rolls: Some("#,
        ),
    ),
    (
        "UiTextureInfos",
        "",
        Proof::Value("Nineslice(Nineslice { nineslice_size: Some(All(2.0))"),
    ),
    (
        "ClientBlocks",
        "",
        Proof::Value(
            r#"Id<block>("determinism:lamp"): Document { value: ClientBlock { textures: Some(SixFaces { up: "lamp_top""#,
        ),
    ),
    // The members of the groups that are no registry and no path map.
    // The files rejected before their recipe type was known: one without a payload key.
    (
        "Recipes",
        "untyped",
        Proof::Finding("recipe/no_identifier", "recipes/determinism_untyped.json"),
    ),
    // A rule of a pass vanilla does not define, a rule that is refused, and its finding.
    (
        "FeatureRules",
        "unknown_passes",
        Proof::Value(
            r#""determinism_pass": Registry { entries: {Id<feature rule>("demo:determinism_nowhere"): Document {"#,
        ),
    ),
    (
        "FeatureRules",
        "rejected",
        Proof::Finding(
            "core/missing_required_field",
            "feature_rules/determinism_refused.json",
        ),
    ),
    (
        "FeatureRules",
        "diagnostics",
        Proof::Finding(
            "core/missing_required_field",
            "feature_rules/determinism_refused.json",
        ),
    ),
    (
        "Poi",
        "resolved",
        Proof::Value(r#"Id<block>("minecraft:anvil"): Id<poi type>("determinism:forge")"#),
    ),
    // A tag of `poi/tags` whose parent names no tag: the group's own finding (the files
    // that are neither kind went with the directories of 1.26.60).
    (
        "Poi",
        "findings",
        Proof::Finding(
            "core/unresolved_reference",
            "poi/tags/determinism_dangling.json",
        ),
    ),
    (
        "Cameras",
        "splines",
        Proof::Value(r#"Spline { name: "determinism:path""#),
    ),
    (
        "SoundDefinitions",
        "",
        Proof::Value(r#"Id<sound event>("determinism.click"): SoundDefinition {"#),
    ),
    (
        "SoundsJson",
        "",
        Proof::Value(r#""ambient.cave": SoundRef { sound: "determinism.click""#),
    ),
    (
        "Music",
        "",
        Proof::Value(r#"Id<music definition>("determinism"): MusicDefinition {"#),
    ),
    // A sound file is listed by its name and never decoded: the name is what a read gives.
    (
        "SoundFiles",
        "by_name",
        Proof::Value(r#""sounds/determinism/click": SoundAsset {"#),
    ),
    // `click.wav` beside `click.ogg`: the client plays the `.ogg`.
    (
        "SoundFiles",
        "shadowed",
        Proof::Value(r#"PackPath("sounds/determinism/click.wav")"#),
    ),
    (
        "SoundFiles",
        "diagnostics",
        Proof::Finding("core/shadowed_definition", "sounds/determinism/click.wav"),
    ),
    // `notes.txt`: a file under `sounds/` the client never looks for.
    (
        "SoundFiles",
        "ignored",
        Proof::Value(r#"PackPath("sounds/determinism/notes.txt")"#),
    ),
    (
        "SoundFiles",
        "diagnostics",
        Proof::Finding("core/ignored_file", "sounds/determinism/notes.txt"),
    ),
    (
        "DeferredSettings",
        "local_lighting",
        Proof::Value(r#"Id<block>("minecraft:torch"): LocalLight {"#),
    ),
    (
        "DeferredSettings",
        "defaults",
        Proof::Value(r#"lighting: Some(Id<lighting settings>("determinism:lighting"))"#),
    ),
    // A file of `point_lights/`, which the game does not read.
    (
        "DeferredSettings",
        "diagnostics",
        Proof::Finding("core/ignored_file", "point_lights/determinism.json"),
    ),
    (
        "TextureAtlases",
        "terrain_atlas",
        Proof::Value(r#"Id<atlas key>("demo_stone"): AtlasItem {"#),
    ),
    (
        "TextureAtlases",
        "item_atlas",
        Proof::Value(r#"Id<atlas key>("demo_gem"): AtlasItem {"#),
    ),
    // Whether each atlas is built.
    ("TextureAtlases", "built", Proof::Value("[true, true]")),
    // An image is listed by its path and never decoded here.
    (
        "TextureAtlases",
        "files",
        Proof::Value(r#""textures/blocks/demo_brick.png": IndexedFile {"#),
    ),
    (
        "Flipbooks",
        "",
        Proof::Value(r#"Flipbook { flipbook_texture: Some("textures/blocks/demo_stone")"#),
    ),
    (
        "TextureLists",
        "lists",
        Proof::Value(r#"TextureList { root: Array, entries: ["textures/blocks/demo_stone""#),
    ),
    (
        "TextureLists",
        "modes",
        Proof::Value(r#"List { entries: ["textures/blocks/demo_stone""#),
    ),
    // A name the list file has and no image: without the list the claims are the names of
    // the image files.
    (
        "TextureLists",
        "claims",
        Proof::Value(r#""textures/blocks/demo_gone": TextureClaim {"#),
    ),
    // `demo_upper.PNG`: an image whose extension only matches ignoring case, which the client
    // never loads.
    (
        "TextureLists",
        "diagnostics",
        Proof::Finding("core/ignored_file", "textures/blocks/demo_upper.PNG"),
    ),
    (
        "TextureSets",
        "sets",
        Proof::Value(r#"TextureSet { color: Some(Texture(TextureName("demo_brick")))"#),
    ),
    (
        "TextureSets",
        "diagnostics",
        Proof::Finding("core/capability_hint", "manifest.json"),
    ),
    // An image is listed by its path and never decoded here.
    (
        "TextureFiles",
        "files",
        Proof::Value(r#""textures/blocks/demo_brick.png": IndexedFile {"#),
    ),
    (
        "Fonts",
        "metadata",
        Proof::Value(r#"FontEntry { font_format: "ttf", font_name: "Determinism""#),
    ),
    (
        "Fonts",
        "emoticons",
        Proof::Value(r#"Emoticon { name: "determinism", code: "0xE1FF""#),
    ),
    // A font file is listed by its path and never decoded.
    (
        "Fonts",
        "files",
        Proof::Value(r#""font/determinism.ttf": FontFile {"#),
    ),
    // A page is an image found by its name (`smooth_41.png` is page 0x41) and never decoded;
    // the fragment has no metrics file, so nothing of the member comes from a file's content.
    ("Fonts", "pages", Proof::Value("MsdfPage { page: 65,")),
    (
        "Fonts",
        "diagnostics",
        Proof::Finding("font/msdf_info_missing", "font/smooth/smooth_41.png"),
    ),
    (
        "Materials",
        "lists",
        Proof::Value(r#"ListEntry { path: "materials/determinism.material""#),
    ),
    (
        "Materials",
        "files",
        Proof::Value(r#"name: "determinism_base", parent: None }: MaterialDef {"#),
    ),
    (
        "Materials",
        "common",
        Proof::Value(r#""determinism_base": Created(Material {"#),
    ),
    (
        "Materials",
        "fancy",
        Proof::Value(r#""determinism_fancy": Created(Material {"#),
    ),
    (
        "Materials",
        "sad",
        Proof::Value(r#""determinism_sad": Created(Material {"#),
    ),
    // `gameface.json` is read as a list and feeds no group: one that is no list is refused
    // and leaves the three groups their lists.
    (
        "Materials",
        "rejected",
        Proof::Finding("core/invalid_type", "materials/gameface.json"),
    ),
    (
        "Materials",
        "diagnostics",
        Proof::Finding("core/invalid_type", "materials/gameface.json"),
    ),
];

/// Whether a member's `Debug` is a registry's or a path map's: its loaded content is its
/// documents.
fn is_documents(member: &str) -> bool {
    member.starts_with("Registry { entries: ") || member.starts_with("Files { entries: ")
}

/// How many documents a registry or a path map holds, from its `Debug`, as those of a pack
/// and those of the app data: its entries, each `<key>): Document {` (the key is an
/// `Id<…>("…")` or a `PackPath("…")`), counted with the findings taken out. The words alone are
/// no document: the code of a file that holds none is `EmptyDocument { … }`, and a finding may
/// quote anything. An entry is a pack's when the pack of its origin has an index
/// (`origin: Origin { pack: PackRef { index: Some(…`), the app data's when it has none; the
/// origin is the last one before the next entry.
fn documents(member: &str) -> (usize, usize) {
    const ENTRY: &str = "): Document {";
    const ORIGIN: &str = "origin: Origin { pack: PackRef { index: ";
    let text = without_findings(member);
    let (mut of_packs, mut of_app_data) = (0, 0);
    for entry in text.split(ENTRY).skip(1) {
        let origin = entry.rfind(ORIGIN).map_or_else(
            || panic!("an entry without an origin: {entry}"),
            |at| &entry[at + ORIGIN.len()..],
        );
        if origin.starts_with("Some(") {
            of_packs += 1;
        } else {
            of_app_data += 1;
        }
    }
    (of_packs, of_app_data)
}

/// The registries the app-data fragments reach, as `(collection, member)`: each must hold a
/// document of the app data beside what the packs give it. `Credits`, a merged file the app
/// data alone has, is proven by its row in [`PROOFS`].
const APP_DATA_FED: &[(&str, &str)] = &[
    ("SpawnGroups", ""),
    ("Features", ""),
    ("FeatureRules", "underground_pass"),
    ("Attachables", ""),
];

/// The registries no pack can feed: the spawn groups are read from the app's definitions
/// alone, so a document of the app data is all the coverage there is.
const APP_DATA_ALONE: &[(&str, &str)] = &[("SpawnGroups", "")];

/// The members that are a flag of the stack, not content a fragment gives: `SoundFiles`
/// remembers whether a base-game pack of the stack lists audio (the client's own files are in
/// view), and the fixture stack's packs are none of the game's.
const STACK_FLAGS: &[(&str, &str)] = &[("SoundFiles", "client_audio")];

/// Whether the `Debug` text `member` holds a finding of the definition `id` told at the file
/// `path`: a `Diagnostic { code: Code { meta: CodeMeta { id: "<id>", … }, … }, location:
/// Location { …, path: Some(PackPath("<path>")), … } }`.
fn tells(member: &str, id: &str, path: &str) -> bool {
    let at = format!("path: Some(PackPath(\"{path}\"))");
    // A code prints its definition's meta, the id first.
    let meta = format!("Code {{ meta: CodeMeta {{ id: {id:?},");
    member
        .split("Diagnostic { code: ")
        .skip(1)
        .any(|finding| finding.starts_with(&meta) && finding.contains(&at))
}

/// Whether a member whose `Debug` is `full` holds content the same member of the stack
/// without fragments (`bare`) does not: something loaded, not a different text. A file that
/// is refused changes the text too (`rejected`, `diagnostics`) and is no content.
///
/// - A registry or a path map holds more documents of a pack than the bare one
///   ([`documents`]); a document of the app data is none (the app data comes with the
///   options, not through the requests the orders complete). The nine behaviour trees built
///   into the game are in both counts. With a proof, the proof's text as well: an entry made
///   of an empty object is a document and no content.
/// - A member with a [`Proof`] has the proof's text and the bare one has not: a value's
///   outside the findings (every `rejected: […]` list and every `Diagnostics(…)` is taken out
///   first), a finding's anywhere.
/// - Anything else without a proof holds nothing that counts: a `Merged`, the keyed
///   translations and the script bundles each have a row in [`PROOFS`].
fn holds_loaded_content(full: &str, bare: &str, proof: Option<Proof>) -> bool {
    let proven = |proof: Proof| match proof {
        Proof::Value(text) => {
            without_findings(full).contains(text) && !without_findings(bare).contains(text)
        }
        Proof::Finding(id, path) => tells(full, id, path) && !tells(bare, id, path),
    };
    if is_documents(full) {
        documents(full).0 > documents(bare).0 && proof.is_none_or(proven)
    } else {
        proof.is_some_and(proven)
    }
}

/// The text a string literal's inside stands for (`Entry::text` is one): `\"` is `"`, `\\`
/// is `\`; any other escape stays as written, which no bracket or quote is.
fn unescaped(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some(escaped @ ('"' | '\\')) => out.push(escaped),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// `debug` (a `Debug` text) without its findings: every bracketed list after `rejected: ` and
/// every parenthesis after `Diagnostics`, each cut whole, brackets inside string literals not
/// counted.
fn without_findings(debug: &str) -> String {
    const OPENERS: [&str; 2] = ["rejected: [", "Diagnostics("];
    let mut out = String::with_capacity(debug.len());
    let mut rest = debug;
    loop {
        let next = OPENERS
            .iter()
            .filter_map(|opener| rest.find(opener).map(|at| at + opener.len() - 1))
            .min();
        let Some(open) = next else {
            out.push_str(rest);
            return out;
        };
        out.push_str(&rest[..open]);
        // The group from its opening bracket to the one that closes it.
        let (mut depth, mut in_string, mut escaped) = (0usize, false, false);
        let mut end = rest.len();
        for (at, c) in rest[open..].char_indices() {
            if in_string {
                match c {
                    _ if escaped => escaped = false,
                    '\\' => escaped = true,
                    '"' => in_string = false,
                    _ => {}
                }
                continue;
            }
            match c {
                '"' => in_string = true,
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = open + at + c.len_utf8();
                        break;
                    }
                }
                _ => {}
            }
        }
        rest = &rest[end..];
    }
}

/// The text of a registry of `Debug` text, holding `inside`.
fn registry(inside: &str) -> String {
    format!("Registry {{ {inside} .. }}")
}

/// The text of a `Merged` holding `value`, with `findings` as its diagnostics.
fn merged(value: &str, findings: &str) -> String {
    format!(
        "Merged {{ value: {value}, origins: {{}}, layers: [], diagnostics: Diagnostics([{findings}]) }}"
    )
}

/// The measures of [`holds_loaded_content`] and the cut of [`members`], on texts written by
/// hand.
#[test]
fn loaded_content_is_a_document_or_a_value_and_never_a_finding() {
    let empty = registry("entries: {}, rejected: [], diagnostics: Diagnostics([]),");
    let refused = registry(
        r#"entries: {}, rejected: [Rejected { path: "a [1].json" }], diagnostics: Diagnostics([Diagnostic { code: X }]),"#,
    );
    let document = |origin: &str| {
        registry(&format!(
            r#"entries: {{Id("a"): Document {{ value: 1, origin: Origin {{ pack: PackRef {{ index: {origin} }} }} }}}}, rejected: [], diagnostics: Diagnostics([]),"#
        ))
    };
    let loaded = document("Some(0)");
    // A document of the app data is not a pack's.
    let of_app_data = document("None");
    assert_eq!(
        (documents(&loaded), documents(&of_app_data)),
        ((1, 0), (0, 1))
    );
    assert!(
        !holds_loaded_content(&of_app_data, &empty, None),
        "the app data's"
    );
    // An entry with a proof needs the proof's text too.
    assert!(holds_loaded_content(
        &loaded,
        &empty,
        Some(Proof::Value("value: 1,"))
    ));
    assert!(
        !holds_loaded_content(&loaded, &empty, Some(Proof::Value("value: 2,"))),
        "an entry of something else"
    );
    assert!(!holds_loaded_content(&empty, &empty, None));
    assert!(!holds_loaded_content(&refused, &empty, None), "a refusal");
    assert!(holds_loaded_content(&loaded, &empty, None), "a document");
    // A file that holds no document is refused with a code that has the word in it.
    let no_document = registry(
        r#"entries: {}, rejected: [Rejected { diagnostics: Diagnostics([Diagnostic { code: Code { meta: CodeMeta { id: "core/empty_document", kind: EmptyDocument }, args: [] } }]) }], diagnostics: Diagnostics([Diagnostic { code: Code { meta: CodeMeta { id: "core/empty_document", kind: EmptyDocument }, args: [] } }]),"#,
    );
    assert!(
        !holds_loaded_content(&no_document, &empty, None),
        "no document"
    );
    assert_eq!(documents(&no_document), (0, 0));
    // The origin of an entry is the last before the next entry, whatever its value holds.
    assert_eq!(
        documents(
            r#"Files { entries: {PackPath("a"): Document { value: S { origin: Origin { pack: PackRef { index: None } } }, origin: Origin { pack: PackRef { index: Some(1) } } }, PackPath("b"): Document { value: EmptyDocument { x: 1 }, origin: Origin { pack: PackRef { index: None } } }}, .. }"#
        ),
        (1, 1)
    );
    // A built-in document is in both: one more is needed.
    assert!(!holds_loaded_content(&loaded, &loaded, None));
}

#[test]
fn a_merged_value_holds_content_only_by_its_proof() {
    let bare = merged("V { names: [] }", "");
    let told = merged(
        "V { names: [] }",
        r#"Diagnostic { text: "a ) and a \" in it, and the name \"a\"" }"#,
    );
    let value = merged(r#"V { names: ["a"] }"#, "");
    // Without a proof a text that differs is no content, whatever differs.
    assert!(!holds_loaded_content(&value, &bare, None), "no proof");
    // A proof is looked for in the value; a finding that names it is none.
    let proof = Some(Proof::Value(r#""a""#));
    assert!(holds_loaded_content(&value, &bare, proof), "a value");
    assert!(!holds_loaded_content(&bare, &bare, proof));
    assert!(
        !holds_loaded_content(&told, &bare, proof),
        "named by a finding alone, with a bracket and a quote in its text"
    );
    assert!(
        !holds_loaded_content(&value, &value, proof),
        "in the bare one too"
    );
    // A file kept and read as nothing changes the text and gives no proof.
    let kept = merged("V { names: [], written: Some(7) }", "");
    assert!(!holds_loaded_content(&kept, &bare, proof), "kept, not read");
}

#[test]
fn a_member_of_findings_alone_shows_a_finding() {
    // A member of findings alone shows a finding.
    let finding = Some(Proof::Finding("core/missing_payload", "a.json"));
    let rejected = |code: &str, path: &str| {
        format!(
            r#"[Rejected {{ diagnostics: Diagnostics([Diagnostic {{ code: Code {{ meta: CodeMeta {{ id: "{code}", owner: "core" }}, args: [] }}, location: Location {{ pack: None, path: Some(PackPath("{path}")) }} }}]) }}]"#
        )
    };
    assert!(holds_loaded_content(
        &rejected("core/missing_payload", "a.json"),
        "[]",
        finding
    ));
    assert!(
        !holds_loaded_content(&rejected("core/missing_payload", "b.json"), "[]", finding),
        "another file"
    );
    assert!(
        !holds_loaded_content(&rejected("core/syntax", "a.json"), "[]", finding),
        "another reason"
    );
    assert!(
        !holds_loaded_content(
            &rejected("core/missing_payload_worded", "a.json"),
            "[]",
            finding
        ),
        "a longer id"
    );
    // The code of one finding and the path of the next are no finding.
    let two = format!(
        "{}{}",
        rejected("core/missing_payload", "b.json"),
        rejected("core/syntax", "a.json")
    );
    assert!(!holds_loaded_content(&two, "[]", finding));
    assert_eq!(
        without_findings("S { rejected: [R { p: \"]\" }], x: 1, d: Diagnostics([D(2)]), y: [3] }"),
        "S { rejected: , x: 1, d: Diagnostics, y: [3] }"
    );
}

#[test]
fn a_group_is_its_fields_and_the_rest_one_member() {
    let empty = registry("entries: {}, rejected: [], diagnostics: Diagnostics([]),");
    let bare = merged("V { names: [] }", "");
    // A group is its fields; a registry, a path map, a `Merged` and a list are one member.
    assert_eq!(
        members(
            r#"Group { a: Registry { entries: {Id("x, y"): Document { value: [1, 2] }}, .. }, b_c: [S { d: 1, e: "}, f: " }], g: Merged { value: 1 }, .. }"#
        ),
        [
            (
                "a",
                r#"Registry { entries: {Id("x, y"): Document { value: [1, 2] }}, .. }"#
            ),
            ("b_c", r#"[S { d: 1, e: "}, f: " }]"#),
            ("g", "Merged { value: 1 }"),
        ]
    );
    for one in [
        empty.as_str(),
        "Files { entries: {}, .. }",
        bare.as_str(),
        r#"[(Language("en_US"), Merged { value: 1 })]"#,
        "[ScriptBundle { files: {} }]",
    ] {
        assert_eq!(members(one), [("", one)]);
    }
    // `SoundFiles { … }` is a member of a group and no path map.
    assert!(!is_documents("SoundFiles { entries: 1 }"));
    assert!(is_documents("Files { entries: {}, .. }"));
}

/// The script bundles of the fixture stack are started by a module of their manifest, the
/// behaviour pack's by a `script` module and the resource pack's by a `client_script` one: the
/// entry file is found and the import graph built, so the five orders compare a bundle's
/// files, entry and graph and not only the files below `scripts/` of a pack without a module.
#[cfg(feature = "script")]
#[test]
fn the_script_bundle_of_each_stack_has_its_module_s_entry_and_graph() {
    let (full, _) = snapshot(
        &fixture_stack(),
        Order::Issue,
        Residency::All,
        &everything_on(),
    );
    let bundles: Vec<_> = full
        .collections()
        .into_iter()
        .filter(|entry| entry.name == "Scripts")
        .map(|entry| (entry.side, entry.text.to_owned()))
        .collect();
    assert_eq!(
        bundles.iter().map(|(side, _)| *side).collect::<Vec<_>>(),
        ["behavior", "resource"]
    );
    for (side, text) in bundles {
        assert_eq!(text.matches("ScriptBundle {").count(), 1, "{side}: {text}");
        assert!(text.contains("entry: Some("), "{side}: no entry: {text}");
        assert!(text.contains("graph: Some("), "{side}: no graph: {text}");
    }
}

/// The comparison can tell two loads apart, and says where: the fixture stack against the same
/// packs without fragments differs in every collection, each named with its stack.
#[test]
fn two_loads_that_differ_are_told_apart_collection_by_collection() {
    let (full, _) = snapshot(
        &fixture_stack(),
        Order::Issue,
        Residency::All,
        &everything_on(),
    );
    let (bare, _) = snapshot(
        &bare_stack(),
        Order::Issue,
        Residency::All,
        &everything_on_and_no_file(),
    );
    assert!(collections::differences(&full, &full).is_empty());
    let differences = collections::differences(&full, &bare);
    for entry in full.collections() {
        let named = format!("collection {} of the {} stack: ", entry.name, entry.side);
        assert!(
            differences.iter().any(|line| line.starts_with(&named)),
            "{named}{differences:#?}"
        );
    }
    // The packs list their files, so the stacks differ whatever is compiled in.
    for stack in ["the behaviour stack: ", "the resource stack: "] {
        assert!(
            differences.iter().any(|line| line.starts_with(stack)),
            "{stack}{differences:#?}"
        );
    }
}

/// A refused file is no coverage. Without its three valid files the camera fragment is one
/// preset that inherits from a preset nobody defines and is refused: the `Cameras` group then
/// holds no document in any member, only the rejection and its findings among the presets, and
/// the check must name it with all three.
#[cfg(feature = "camera")]
#[test]
fn a_collection_left_with_a_refused_file_alone_is_not_covered() {
    let stack = fixture_stack_without(&[
        "behavior_pack/cameras/presets/orbit.json",
        "behavior_pack/cameras/lists/cameras.json",
        "behavior_pack/cameras/splines/path.json",
    ]);
    // The refused file is there: the group's text is not the bare stack's.
    let (loaded, _) = snapshot(&stack, Order::Issue, Residency::All, &everything_on());
    assert!(holds(&loaded, "Cameras", "orphan.json"));
    assert!(holds(&loaded, "Cameras", "rejected: [Rejected {"));
    assert_eq!(
        collections_without_a_fragment(&stack),
        [
            "Collection::Cameras of the behavior stack: no determinism fragment reaches its members presets, lists, splines"
        ]
    );
}

/// A group is covered member by member: without the furnace recipe, the rule of the sky pass
/// and the list of the Sad group, each group keeps its other fragments and is named with the
/// one member nothing reaches.
#[cfg(all(feature = "recipe", feature = "feature_rules", feature = "material"))]
#[test]
fn a_member_of_a_group_left_without_its_fragment_is_named() {
    let stack = fixture_stack_without(&[
        "behavior_pack/recipes/determinism_furnace.json",
        "behavior_pack/feature_rules/determinism_sky_pass.json",
        "resource_pack/materials/sad.json",
    ]);
    // The groups are not empty: their neighbours are loaded.
    let (loaded, _) = snapshot(&stack, Order::Issue, Residency::All, &everything_on());
    assert!(holds(&loaded, "Recipes", "demo:determinism_brewing_mix"));
    assert!(holds(
        &loaded,
        "FeatureRules",
        "demo:determinism_final_pass"
    ));
    assert!(holds(&loaded, "Materials", "determinism_fancy"));
    assert_eq!(
        collections_without_a_fragment(&stack),
        [
            "Collection::Recipes of the behavior stack: no determinism fragment reaches its member furnace",
            "Collection::FeatureRules of the behavior stack: no determinism fragment reaches its member sky_pass",
            "Collection::Materials of the resource stack: no determinism fragment reaches its member sad",
        ]
    );
}

/// Every collection of both stacks, as the coverage check names it.
fn every_collection() -> Vec<String> {
    let (bare, _) = snapshot(
        &bare_stack(),
        Order::Issue,
        Residency::All,
        &everything_on_and_no_file(),
    );
    bare.collections()
        .iter()
        .map(|entry| format!("Collection::{} of the {} stack", entry.name, entry.side))
        .collect()
}

/// The collection a line of [`collections_without_a_fragment`] names.
fn named(line: &str) -> &str {
    line.split_once(": ")
        .map_or(line, |(collection, _)| collection)
}

/// The collections that stay covered, and the lines of those that do not, with `document` in
/// place of every JSON document of every fragment.
fn covered_with_every_json_as(document: &[u8]) -> (Vec<String>, Vec<String>) {
    let (stack, options) = rewritten_fixture(&|path, bytes| {
        Some(
            if std::path::Path::new(path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            {
                document.to_vec()
            } else {
                bytes
            },
        )
    });
    let uncovered = collections_without_a_fragment_with(&stack, &options);
    let covered = every_collection()
        .into_iter()
        .filter(|collection| !uncovered.iter().any(|line| named(line) == collection))
        .collect();
    (covered, uncovered)
}

/// The collections the fragments' files that are no JSON fill: the structures, the functions,
/// the strings of the `.lang` file and the script bundles.
fn collections_fed_by_no_json() -> Vec<String> {
    #[allow(unused_mut, reason = "every entry is behind its family's feature")]
    let mut fed: Vec<String> = Vec::new();
    #[cfg(feature = "structure")]
    fed.push(String::from("Collection::Structures of the behavior stack"));
    #[cfg(feature = "function")]
    fed.push(String::from("Collection::Functions of the behavior stack"));
    // Both-sides accessors: the behaviour stack's, then the resource stack's.
    for side in ["behavior", "resource"] {
        #[cfg(feature = "lang")]
        fed.push(format!("Collection::Translations of the {side} stack"));
        #[cfg(feature = "script")]
        fed.push(format!("Collection::Scripts of the {side} stack"));
        let _ = side;
    }
    // Listings of the resource stack, which no reader opens: a sound and an image are named
    // by their path.
    #[cfg(feature = "sound_file")]
    fed.push(String::from("Collection::SoundFiles of the resource stack"));
    #[cfg(feature = "texture_file")]
    fed.push(String::from(
        "Collection::TextureFiles of the resource stack",
    ));
    fed
}

/// A file the loader keeps and reads nothing from is no coverage. With `7` in place of every
/// JSON document of every fragment, no file is refused by a reader that takes any JSON root:
/// a merged file keeps a layer or an origin of it and an empty value, and its text differs
/// from the bare stack's. Only what the fragments' other files give is left.
#[test]
fn a_json_root_of_the_wrong_type_is_no_content() {
    let (covered, uncovered) = covered_with_every_json_as(b"7");
    assert_eq!(
        covered,
        collections_fed_by_no_json(),
        "uncovered:\n{}",
        uncovered.join("\n")
    );
}

/// A file that holds no document is no coverage: empty, white space alone, or `null`. Such a
/// file is refused with the code `EmptyDocument`, or skipped with a note, and a registry
/// left with nothing but that holds no entry. Only what the fragments' other files give is
/// left.
#[test]
fn a_json_file_that_holds_no_document_is_no_content() {
    for (what, document) in [
        ("empty", &b""[..]),
        ("white space", b" \n\t\r\n"),
        ("null", b"null"),
    ] {
        let (covered, uncovered) = covered_with_every_json_as(document);
        assert_eq!(
            covered,
            collections_fed_by_no_json(),
            "{what}; uncovered:\n{}",
            uncovered.join("\n")
        );
    }
}

/// Bytes no reader takes are no coverage, whatever the file: with every fragment file made
/// the same bytes that are no UTF-8, no JSON, no NBT and no image, no collection of either
/// stack holds something of a fragment. A file that is only listed by its name (a sound, a
/// font, an image) still is, in its member; the collection is named for the others.
#[test]
fn bytes_no_reader_takes_are_no_content() {
    let (stack, options) =
        rewritten_fixture(&|_, _| Some(b"\xff\xfe\x00 no reader takes this \xc3\x28".to_vec()));
    let uncovered = collections_without_a_fragment_with(&stack, &options);
    // The collections that are nothing but a listing of files stay covered by the listing.
    let listings = [
        "Collection::SoundFiles of the resource stack",
        "Collection::TextureFiles of the resource stack",
    ];
    let expected: Vec<String> = every_collection()
        .into_iter()
        .filter(|line| !listings.contains(&line.as_str()))
        .collect();
    assert_eq!(
        uncovered.iter().map(|line| named(line)).collect::<Vec<_>>(),
        expected,
        "{}",
        uncovered.join("\n")
    );
}

/// The UI definitions hold no document: a `Merged` of the files `_ui_defs.json` lists. With
/// its three `ui/` files unparsable, and with each a JSON root of the wrong type, the
/// collection alone is named, on real data.
#[cfg(feature = "json_ui")]
#[test]
fn the_ui_definitions_left_with_files_that_are_not_read_are_not_covered() {
    const UI: [&str; 3] = [
        "resource_pack/ui/_global_variables.json",
        "resource_pack/ui/_ui_defs.json",
        "resource_pack/ui/determinism_screen.json",
    ];
    for unread in [&b"[1"[..], b"7"] {
        let (stack, options) = rewritten_fixture(&|path, bytes| {
            Some(if UI.contains(&path) {
                unread.to_vec()
            } else {
                bytes
            })
        });
        // The files are there: the collection's text is not the bare stack's.
        let (loaded, _) = snapshot(&stack, Order::Issue, Residency::All, &options);
        assert!(
            holds(&loaded, "Ui", "ui/_global_variables.json")
                || holds(&loaded, "Ui", "ui/_ui_defs.json"),
            "{:?}",
            std::str::from_utf8(unread)
        );
        assert_eq!(
            collections_without_a_fragment_with(&stack, &options),
            ["Collection::Ui of the resource stack: no determinism fragment reaches its accessor"],
            "{:?}",
            std::str::from_utf8(unread)
        );
    }
}

// ---- What a `Debug` does not print (`collections.rs`, "What the text does not show") ----------
//
// Five values of the compared collections print less than they hold. Each test below loads two
// stacks that differ in such a value only where its `Debug` is silent, shows that the `Debug`
// of the value is the same in both, and requires the comparison to tell the two loads apart
// at that value, by its name. The comparison reads the silent part through `unprinted.rs`,
// which needs every family compiled in: so do the tests.

/// The two packs of this test with `files` (paths below root 1, `bp/…` or `rp/…`), loaded with
/// everything on: the set and its snapshot.
#[allow(dead_code, reason = "used behind the families' features")]
fn loaded_with(files: &[(&str, &[u8])]) -> (StackSet, Snapshot) {
    let mut stack = bare_stack();
    for (path, bytes) in files {
        stack.file(1, path, bytes.to_vec());
    }
    let options = everything_on_and_no_file();
    let set = loaded(&stack, Order::Issue, Residency::All, &options);
    let snapshot = snapshot_of(&set);
    (set, snapshot)
}

/// Requires the comparison of `one` with `other` to name `what` as a place the two differ.
#[allow(dead_code, reason = "used behind the families' features")]
#[track_caller]
fn assert_told_apart_at(what: &str, one: &Snapshot, other: &Snapshot) {
    assert!(collections::differences(one, one).is_empty());
    let differences = collections::differences(one, other);
    let named = format!("{what}: ");
    assert!(
        differences.iter().any(|line| line.starts_with(&named)),
        "no difference is told at `{what}`:\n{}",
        differences.join("\n")
    );
}

/// A material prints how many variants it has ("1 names"), not which: a child whose parent
/// has the variant `x` in one load and `y` in the other is the same text.
#[cfg(feature = "material")]
#[test]
fn two_materials_that_differ_in_their_variant_names_are_told_apart() {
    use bedrock_addon::families::material::MaterialGroupKind;
    let list = br#"[ { "path": "materials/a.material" } ]"#;
    let file = |variant: &str| {
        format!(
            r#"{{ "materials": {{ "version": "1.0.0",
                 "base": {{ "variants": [ {{ "{variant}": {{}} }} ] }},
                 "child:base": {{}} }} }}"#
        )
    };
    let load = |variant: &str| {
        loaded_with(&[
            ("rp/materials/common.json", list),
            ("rp/materials/a.material", file(variant).as_bytes()),
        ])
    };
    let ((one, one_text), (other, other_text)) = (load("x"), load("y"));
    let child = |set: &StackSet| {
        let materials = set.resource.materials();
        let child = materials
            .group(MaterialGroupKind::Common)
            .get("child")
            .expect("the child");
        (
            format!("{child:?}"),
            child.variants().map(str::to_owned).collect::<Vec<_>>(),
        )
    };
    let ((one_debug, one_variants), (other_debug, other_variants)) = (child(&one), child(&other));
    assert_eq!(one_debug, other_debug, "the Debug does not show the names");
    assert_eq!(
        (one_variants, other_variants),
        (vec!["x".to_owned()], vec!["y".to_owned()])
    );
    assert_told_apart_at(
        "the variants of material child of the Common group",
        &one_text,
        &other_text,
    );
}

/// What a geometry inherits prints as its parent's identifier, a number of bones and the
/// chain's depth: a parent bone with another pivot is the same text.
#[cfg(feature = "geometry")]
#[test]
fn two_geometries_that_differ_in_an_inherited_bone_are_told_apart() {
    let file = |pivot: &str| {
        format!(
            r#"{{ "format_version": "1.8.0",
                 "geometry.parent": {{ "bones": [ {{ "name": "body", "pivot": [0, {pivot}, 0] }} ] }},
                 "geometry.child:geometry.parent": {{ "bones": [ {{ "name": "head" }} ] }} }}"#
        )
    };
    let load =
        |pivot: &str| loaded_with(&[("rp/models/entity/pair.geo.json", file(pivot).as_bytes())]);
    let ((one, one_text), (other, other_text)) = (load("1"), load("2"));
    let inherited = |set: &StackSet| {
        let geometries = set.resource.geometries();
        let (_, child) = geometries
            .iter()
            .find(|(id, _)| id.as_str() == "geometry.child")
            .expect("the child");
        let inherited = child.inherited.as_ref().expect("a parent");
        (format!("{inherited:?}"), format!("{:?}", inherited.bones()))
    };
    let ((one_debug, one_bones), (other_debug, other_bones)) = (inherited(&one), inherited(&other));
    assert_eq!(one_debug, other_debug, "the Debug does not show the bones");
    assert_ne!(one_bones, other_bones);
    assert_told_apart_at(
        "what geometry geometry.child inherits",
        &one_text,
        &other_text,
    );
}

/// A jigsaw metadata file prints the length of its body, not the bytes.
#[cfg(feature = "jigsaw")]
#[test]
fn two_jigsaw_metadata_files_that_differ_in_a_body_byte_are_told_apart() {
    let load = |last: u8| {
        let mut file = b"\x071.21.90\x01".to_vec();
        file.push(last);
        loaded_with(&[("bp/worldgen/metadata/ruin.dat", &file)])
    };
    let ((one, one_text), (other, other_text)) = (load(2), load(3));
    let metadata = |set: &StackSet| {
        let jigsaw = set.behavior.jigsaw();
        let (_, file) = jigsaw.metadata.iter().next().expect("the file");
        (format!("{:?}", file.value()), file.value().body.clone())
    };
    let ((one_debug, one_body), (other_debug, other_body)) = (metadata(&one), metadata(&other));
    assert_eq!(one_debug, other_debug, "the Debug does not show the body");
    assert_eq!((one_body, other_body), (vec![1, 2], vec![1, 3]));
    assert_told_apart_at(
        "the body of worldgen/metadata/ruin.dat",
        &one_text,
        &other_text,
    );
}

/// A structure prints of its layout only whether the file's tree is kept: the bytes after the
/// root tag, which are written back, are the same text whatever they are.
#[cfg(feature = "structure")]
#[test]
fn two_structures_that_differ_in_what_their_layout_keeps_are_told_apart() {
    let fragment = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "tests/fixtures/structure/structure/determinism/behavior_pack/structures/jigsaw.mcstructure",
    );
    let load = |trailing: u8| {
        let mut file = std::fs::read(&fragment).unwrap();
        file.push(trailing);
        loaded_with(&[("bp/structures/jigsaw.mcstructure", &file)])
    };
    let ((one, one_text), (other, other_text)) = (load(1), load(2));
    let structure = |set: &StackSet| {
        let structures = set.behavior.structures();
        let (_, file) = structures.iter().next().expect("the file");
        (
            format!("{:?}", file.value()),
            file.value().to_bytes().expect("bytes"),
        )
    };
    let ((one_debug, one_bytes), (other_debug, other_bytes)) = (structure(&one), structure(&other));
    assert_eq!(one_debug, other_debug, "the Debug does not show the layout");
    assert_eq!((one_bytes.last(), other_bytes.last()), (Some(&1), Some(&2)));
    assert_told_apart_at(
        "the bytes structures/jigsaw.mcstructure is written as",
        &one_text,
        &other_text,
    );
}

/// A UI definition prints the name of the definition it inherits from, not what that hands
/// on: with another member in the base, a definition that writes nothing of its own is the
/// same text.
#[cfg(feature = "json_ui")]
#[test]
fn two_ui_definitions_that_differ_in_what_they_inherit_are_told_apart() {
    let defs = br#"{ "ui_defs": [ "ui/pair.json" ] }"#;
    let file = |size: &str| {
        format!(
            r#"{{ "namespace": "pair",
                 "base": {{ "type": "panel", "size": [ {size}, 1 ] }},
                 "child@pair.base": {{ "layer": 1 }} }}"#
        )
    };
    let load = |size: &str| {
        loaded_with(&[
            ("rp/ui/_ui_defs.json", defs),
            ("rp/ui/pair.json", file(size).as_bytes()),
        ])
    };
    let ((one, one_text), (other, other_text)) = (load("1"), load("2"));
    let child = |set: &StackSet| {
        let ui = set.resource.ui();
        let child = ui.get("pair", "child").expect("the child");
        (format!("{child:?}"), format!("{:?}", child.body()))
    };
    let ((one_debug, one_body), (other_debug, other_body)) = (child(&one), child(&other));
    assert_eq!(
        one_debug, other_debug,
        "the Debug does not show the inherited members"
    );
    assert_ne!(one_body, other_body);
    assert_told_apart_at(
        "the body of the UI definition pair.child",
        &one_text,
        &other_text,
    );
}

/// The fixture stack has a value of each kind, so the five orders compare what each leaves
/// out: a material with variants, a geometry with a parent, a jigsaw metadata file, a
/// structure, a UI definition with a base, and a beta document in each of the four registries
/// whose family has a beta form. And two loads whose values are not the same ones are told
/// apart by the list of them.
#[cfg(feature = "full")]
#[test]
fn the_fixture_stack_holds_a_value_of_each_kind_whose_debug_leaves_a_part_out() {
    let (full, _) = snapshot(
        &fixture_stack(),
        Order::Issue,
        Residency::All,
        &everything_on(),
    );
    let of_kind = |kind: &str| -> Vec<&str> {
        full.unprinted
            .iter()
            .filter(|value| value.what.starts_with(kind) && !value.content.is_empty())
            .map(|value| value.what.as_str())
            .collect()
    };
    for kind in unprinted::KINDS {
        assert!(!of_kind(kind).is_empty(), "no value named `{kind}…`");
    }
    assert_eq!(
        full.unprinted.len(),
        unprinted::KINDS
            .iter()
            .map(|kind| full
                .unprinted
                .iter()
                .filter(|value| value.what.starts_with(kind))
                .count())
            .sum::<usize>(),
        "every value is of one of the kinds"
    );
    // The inherited ones, by name: the results of the combine step.
    assert!(
        of_kind("what geometry ")
            .contains(&"what geometry geometry.determinism_post_knot inherits")
    );
    assert!(
        of_kind("the body of the UI definition ")
            .contains(&"the body of the UI definition determinism.title_label")
    );
    assert!(
        of_kind("the variants of material ")
            .contains(&"the variants of material determinism_child of the Common group")
    );
    // The beta documents, each in its registry's value as a document.
    for (registry, kind) in [
        ("Entities of the behavior stack", "entity"),
        ("Items of the behavior stack", "item"),
        ("Biomes of the behavior stack", "biome"),
        ("ClientBiomes of the resource stack", "biome"),
    ] {
        let what = format!("{}{registry}", unprinted::LAYERS);
        let value = full
            .unprinted
            .iter()
            .find(|value| value.what == what)
            .unwrap_or_else(|| panic!("no value `{what}`"));
        let text = String::from_utf8_lossy(&value.content);
        let document = format!(
            "beta Id<{kind}>(\"demo:determinism_beta\"): 1 layers, the first effective\n  0: Document {{"
        );
        assert!(text.contains(&document), "{what}: {text}");
    }
    // The same packs without fragments have the registries, and nothing in them but the
    // behaviour trees built into the game.
    let (bare, _) = snapshot(
        &bare_stack(),
        Order::Issue,
        Residency::All,
        &everything_on_and_no_file(),
    );
    for value in &bare.unprinted {
        assert!(
            value.what.starts_with(unprinted::LAYERS)
                && (value.content.is_empty() || value.what.contains("BehaviorTrees")),
            "{value:?}"
        );
    }
    assert!(
        collections::differences(&full, &bare)
            .iter()
            .any(|line| line.starts_with("the values whose Debug leaves a part out: ")),
    );
}

/// `unprinted.rs` lists the registries and path maps by hand; the stacks know them. Every
/// member whose `Debug` is a registry's or a path map's has a value of that name, and there is
/// no other (but the registries inside `FeatureRules::unknown_passes`, which are no member):
/// an accessor or a group member added without a line there fails here by name. And deeper
/// than a member: each collection has as many values as its `Debug` has registries and path
/// maps anywhere in it.
#[cfg(feature = "full")]
#[test]
fn every_registry_and_path_map_gives_its_unprinted_part() {
    let (full, _) = snapshot(
        &fixture_stack(),
        Order::Issue,
        Residency::All,
        &everything_on(),
    );
    let mut shown: Vec<String> = Vec::new();
    for entry in full.collections() {
        let text = unescaped(entry.text);
        for (member, text) in members(&text) {
            if is_documents(text) {
                let dot = if member.is_empty() { "" } else { "." };
                shown.push(format!(
                    "{}{dot}{member} of the {} stack",
                    entry.name, entry.side
                ));
            }
        }
    }
    let given: Vec<&str> = full
        .unprinted
        .iter()
        .filter_map(|value| value.what.strip_prefix(unprinted::LAYERS))
        .filter(|name| !name.contains("unknown_passes["))
        .collect();
    assert_eq!(given, shown);
    assert_eq!(
        shown.len(),
        71,
        "the registries and path maps of the two stacks"
    );
    // At any depth: a collection's `Debug` has as many registries and path maps in it as
    // this collection has values, the ones inside a member's map included. (A map that is
    // empty shows none; the coverage check keeps every member of a group filled.)
    let all: Vec<&str> = full
        .unprinted
        .iter()
        .filter_map(|value| value.what.strip_prefix(unprinted::LAYERS))
        .collect();
    for entry in full.collections() {
        let text = without_findings(&unescaped(entry.text));
        let printed: usize = ["Registry { entries: ", "Files { entries: "]
            .iter()
            .map(|opening| {
                text.match_indices(opening)
                    .filter(|(at, _)| {
                        !text[..*at]
                            .chars()
                            .next_back()
                            .is_some_and(char::is_alphanumeric)
                    })
                    .count()
            })
            .sum();
        let stack = format!(" of the {} stack", entry.side);
        let values = all
            .iter()
            .filter_map(|name| name.strip_suffix(&stack))
            .filter(|name| {
                *name == entry.name
                    || name
                        .strip_prefix(entry.name)
                        .is_some_and(|member| member.starts_with('.'))
            })
            .count();
        assert_eq!(
            values, printed,
            "Collection::{} of the {} stack",
            entry.name, entry.side
        );
    }
    // The one registry inside a member's map.
    assert!(full.unprinted.iter().any(|value| {
        value.what
            == format!(
                "{}FeatureRules.unknown_passes[determinism_pass] of the behavior stack",
                unprinted::LAYERS
            )
    }));
}

/// The fixture stack over a second behaviour pack (3) and a second resource pack (4) that
/// hold the same fragments: every definition and file of the upper packs shadows a copy.
fn shadowing_stack() -> (Responder, WorldPacks) {
    let files = fragment_files(&|_, bytes| Some(bytes));
    let mut stack = stack_of(&files);
    stack
        .file(
            1,
            "bp_low/manifest.json",
            pack_manifest(3, "script", "data"),
        )
        .file(
            1,
            "rp_low/manifest.json",
            pack_manifest(4, "client_script", "resources"),
        );
    for (path, bytes) in &files {
        if path.starts_with(APP_DATA) {
            continue;
        }
        if let Some(rest) = path.strip_prefix("behavior_pack/") {
            stack.file(1, &format!("bp_low/{rest}"), bytes.clone());
        } else if let Some(rest) = path.strip_prefix("resource_pack/") {
            stack.file(1, &format!("rp_low/{rest}"), bytes.clone());
        } else {
            stack.file(1, &format!("bp_low/{path}"), bytes.clone());
            stack.file(1, &format!("rp_low/{path}"), bytes.clone());
        }
    }
    (stack, world_of(&[1, 3], &[2, 4]))
}

/// The five orders with the shadowed copies kept (`LoadOptions::keep_shadowed`): a registry
/// and a path map then hold what no `Debug` prints, and the orders are compared in it. The
/// stack is one where the copies exist.
#[test]
fn every_completion_order_gives_the_same_result_with_the_shadowed_copies_kept() {
    let (stack, world) = shadowing_stack();
    let options = LoadOptions {
        keep_shadowed: true,
        ..everything_on()
    };
    assert_every_order_agrees_in(&stack, &options, &world);
    // With every family compiled in: the copies are there, by name, and are not without
    // the option.
    #[cfg(feature = "full")]
    {
        let set = loaded_in(&stack, Order::Issue, Residency::All, &options, &world);
        let kept = snapshot_of(&set);
        let shadowed = |registry: &str| {
            let what = format!("{}{registry}", unprinted::LAYERS);
            kept.unprinted
                .iter()
                .find(|value| value.what == what)
                .is_some_and(|value| {
                    String::from_utf8_lossy(&value.content).contains(": 2 layers, ")
                })
        };
        for registry in [
            "Blocks of the behavior stack",
            "Entities of the behavior stack",
            "Recipes.crafting of the behavior stack",
            "LootTables of the behavior stack",
            "Structures of the behavior stack",
            "Geometries of the resource stack",
            "DeferredSettings.shadows of the resource stack",
            "Particles of the resource stack",
        ] {
            assert!(shadowed(registry), "{registry} holds no shadowed copy");
        }
        // Without the option the same stack keeps none.
        let dropped = loaded_in(
            &stack,
            Order::Issue,
            Residency::All,
            &everything_on(),
            &world,
        );
        let dropped = snapshot_of(&dropped);
        assert!(
            !dropped
                .unprinted
                .iter()
                .any(|value| String::from_utf8_lossy(&value.content).contains(": 2 layers, ")),
        );
    }
}

/// The lines of the coverage check with `document` in place of the fragment files `paths`.
#[allow(dead_code, reason = "used behind the families' features")]
fn uncovered_with(paths: &[&str], document: &[u8]) -> Vec<String> {
    let (stack, options) = rewritten_fixture(&|path, bytes| {
        Some(if paths.contains(&path) {
            document.to_vec()
        } else {
            bytes
        })
    });
    collections_without_a_fragment_with(&stack, &options)
}

/// A member fed by two files needs both read: `Languages` is the codes of `languages.json`
/// and the names of `language_names.json`, and either file alone as `7` leaves it uncovered
/// on both stacks.
#[cfg(feature = "languages")]
#[test]
fn a_member_fed_by_two_files_needs_each_of_them_read() {
    for file in ["texts/languages.json", "texts/language_names.json"] {
        assert_eq!(
            uncovered_with(&[file], b"7"),
            [
                "Collection::Languages of the behavior stack: no determinism fragment reaches its accessor",
                "Collection::Languages of the resource stack: no determinism fragment reaches its accessor",
            ],
            "{file}"
        );
    }
}

/// A member of findings is proven by the finding its fragment is there for, its code at its
/// path: the same file refused for another reason (here as bytes no reader takes) is not it.
#[cfg(feature = "feature_rules")]
#[test]
fn a_file_refused_for_another_reason_is_not_the_finding_of_its_member() {
    assert_eq!(
        uncovered_with(
            &["behavior_pack/feature_rules/determinism_refused.json"],
            b"\xff\xfe\x00"
        ),
        [
            "Collection::FeatureRules of the behavior stack: no determinism fragment reaches its members rejected, diagnostics"
        ]
    );
}

/// The texture claims are proven by a name only the list file gives: without the list read,
/// the claims are the names of the image files, and the member is not covered by them.
#[cfg(feature = "texture_list")]
#[test]
fn the_texture_claims_need_the_list_file_read() {
    assert_eq!(
        uncovered_with(
            &["resource_pack/textures/textures_list.json"],
            b"\xff\xfe\x00"
        ),
        [
            "Collection::TextureLists of the resource stack: no determinism fragment reaches its members lists, modes, claims"
        ]
    );
}

// ---- A registry's beta documents and the shadowed copies -------------------------------------
//
// A sixth silence, of every registry and path map: `Registry`'s `Debug` prints its beta
// registry as identifiers alone, and neither it nor `Files` prints the copies a higher pack
// shadows (kept with `LoadOptions::keep_shadowed`).

/// The text of the collection `name` of the `side` stack in `snapshot`.
#[allow(dead_code, reason = "used behind the families' features")]
fn text_of(snapshot: &Snapshot, side: &str, name: &str) -> String {
    snapshot
        .collections()
        .iter()
        .find(|entry| entry.side == side && entry.name == name)
        .unwrap_or_else(|| panic!("no Collection::{name} on the {side} stack"))
        .text
        .to_owned()
}

/// Two loads whose one file at `path` is the beta document `document(variant)` for two
/// variants: the collection `name` has the same `Debug` in both, with the identifier in its
/// beta list, and the comparison must name the collection's unprinted part.
#[allow(dead_code, reason = "used behind the families' features")]
#[track_caller]
fn assert_beta_documents_are_compared(
    side: &str,
    name: &str,
    path: &str,
    document: &dyn Fn(&str) -> String,
    variants: [&str; 2],
) {
    let load = |variant: &str| loaded_with(&[(path, document(variant).as_bytes())]).1;
    let (one, other) = (load(variants[0]), load(variants[1]));
    let (one_text, other_text) = (text_of(&one, side, name), text_of(&other, side, name));
    assert_eq!(
        one_text, other_text,
        "the Debug does not show the beta document"
    );
    assert!(
        one_text.contains(r"beta: Some([Id<") && one_text.contains(r#"(\"demo:beta\")])"#),
        "the beta registry holds the document: {one_text}"
    );
    assert_told_apart_at(
        &format!("the layers of {name} of the {side} stack"),
        &one,
        &other,
    );
}

#[cfg(feature = "entity")]
#[test]
fn two_beta_entities_that_differ_are_told_apart() {
    assert_beta_documents_are_compared(
        "behavior",
        "Entities",
        "bp/entities/beta.json",
        &|spawnable| {
            format!(
                r#"{{ "format_version": "beta", "minecraft:entity": {{
                     "description": {{ "identifier": "demo:beta", "is_spawnable": {spawnable} }},
                     "components": {{}} }} }}"#
            )
        },
        ["true", "false"],
    );
}

#[cfg(feature = "item")]
#[test]
fn two_beta_items_that_differ_are_told_apart() {
    assert_beta_documents_are_compared(
        "behavior",
        "Items",
        "bp/items/beta.json",
        &|size| {
            format!(
                r#"{{ "format_version": "beta", "minecraft:item": {{
                     "description": {{ "identifier": "demo:beta" }},
                     "components": {{ "minecraft:max_stack_size": {size} }} }} }}"#
            )
        },
        ["16", "32"],
    );
}

#[cfg(feature = "biome")]
#[test]
fn two_beta_biomes_that_differ_are_told_apart() {
    assert_beta_documents_are_compared(
        "behavior",
        "Biomes",
        "bp/biomes/beta.json",
        &|downfall| {
            format!(
                r#"{{ "format_version": "beta", "minecraft:biome": {{
                     "description": {{ "identifier": "demo:beta" }},
                     "components": {{ "minecraft:climate": {{ "downfall": {downfall} }} }} }} }}"#
            )
        },
        ["0.6", "0.5"],
    );
}

#[cfg(feature = "client_biome")]
#[test]
fn two_beta_client_biomes_that_differ_are_told_apart() {
    assert_beta_documents_are_compared(
        "resource",
        "ClientBiomes",
        "rp/biomes/beta.json",
        &|color| {
            format!(
                r##"{{ "format_version": "beta", "minecraft:client_biome": {{
                     "description": {{ "identifier": "demo:beta" }},
                     "components": {{ "minecraft:sky_color": {{ "sky_color": "#{color}" }} }} }} }}"##
            )
        },
        ["112233", "445566"],
    );
}

/// The two packs of this test over a lower behaviour pack (3) and a lower resource pack (4),
/// with `files`, loaded with everything on and the shadowed copies kept.
#[allow(dead_code, reason = "used behind the families' features")]
fn loaded_over_lower_packs(files: &[(&str, &[u8])]) -> Snapshot {
    loaded_over_lower_packs_keeping(files, true)
}

/// [`loaded_over_lower_packs`] with `LoadOptions::keep_shadowed` as given.
#[allow(dead_code, reason = "used behind the families' features")]
fn loaded_over_lower_packs_keeping(files: &[(&str, &[u8])], keep_shadowed: bool) -> Snapshot {
    let mut stack = bare_stack();
    stack
        .file(
            1,
            "bp_low/manifest.json",
            pack_manifest(3, "script", "data"),
        )
        .file(
            1,
            "rp_low/manifest.json",
            pack_manifest(4, "client_script", "resources"),
        );
    for (path, bytes) in files {
        stack.file(1, path, bytes.to_vec());
    }
    let options = LoadOptions {
        keep_shadowed,
        ..everything_on_and_no_file()
    };
    let set = loaded_in(
        &stack,
        Order::Issue,
        Residency::All,
        &options,
        &world_of(&[1, 3], &[2, 4]),
    );
    snapshot_of(&set)
}

/// A definition a higher pack shadows is kept with `keep_shadowed` and printed by no
/// `Debug`: two loads whose lower pack's block differs have the same text of `Blocks`.
#[cfg(feature = "block")]
#[test]
fn two_shadowed_definitions_that_differ_are_told_apart() {
    let block = |seconds: &str| {
        format!(
            r#"{{ "format_version": "1.26.20", "minecraft:block": {{
                 "description": {{ "identifier": "demo:shadowed" }},
                 "components": {{ "minecraft:destructible_by_mining": {{ "seconds_to_destroy": {seconds} }} }} }} }}"#
        )
    };
    let load = |seconds: &str| {
        loaded_over_lower_packs(&[
            ("bp/blocks/shadowed.json", block("1").as_bytes()),
            ("bp_low/blocks/shadowed.json", block(seconds).as_bytes()),
        ])
    };
    let (one, other) = (load("2"), load("3"));
    assert_eq!(
        text_of(&one, "behavior", "Blocks"),
        text_of(&other, "behavior", "Blocks"),
        "the Debug does not show the shadowed copy"
    );
    assert_told_apart_at("the layers of Blocks of the behavior stack", &one, &other);
}

/// The same for a path map: the lower pack's copy of a loot table.
#[cfg(feature = "loot")]
#[test]
fn two_shadowed_files_that_differ_are_told_apart() {
    let table =
        |rolls: &str| format!(r#"{{ "pools": [ {{ "rolls": {rolls}, "entries": [] }} ] }}"#);
    let load = |rolls: &str| {
        loaded_over_lower_packs(&[
            ("bp/loot_tables/shadowed.json", table("1").as_bytes()),
            ("bp_low/loot_tables/shadowed.json", table(rolls).as_bytes()),
        ])
    };
    let (one, other) = (load("2"), load("3"));
    assert_eq!(
        text_of(&one, "behavior", "LootTables"),
        text_of(&other, "behavior", "LootTables"),
        "the Debug does not show the shadowed copy"
    );
    assert_told_apart_at(
        "the layers of LootTables of the behavior stack",
        &one,
        &other,
    );
}

// ---- The slots `iter()` does not reach ---------------------------------------
//
// A registry and a path map keep documents in slots that have no effective one: `iter()`,
// and so their `Debug`, pass them by.

/// Requires two loads to print the collection `name` the same and the comparison to name its
/// table of layers as a place they differ.
#[allow(dead_code, reason = "used behind the families' features")]
#[track_caller]
fn assert_same_text_and_told_apart(side: &str, name: &str, one: &Snapshot, other: &Snapshot) {
    assert_eq!(
        text_of(one, side, name),
        text_of(other, side, name),
        "the Debug does not show the difference"
    );
    assert_told_apart_at(
        &format!("the layers of {name} of the {side} stack"),
        one,
        other,
    );
}

/// A beta definition under a higher pack's stable one of the same identifier stays in the
/// beta registry, effective nowhere, whatever `keep_shadowed` says.
#[cfg(feature = "biome")]
#[test]
fn two_beta_documents_under_a_stable_definition_that_differ_are_told_apart() {
    let biome = |version: &str, downfall: &str| {
        format!(
            r#"{{ "format_version": "{version}", "minecraft:biome": {{
                 "description": {{ "identifier": "demo:x" }},
                 "components": {{ "minecraft:climate": {{ "downfall": {downfall} }} }} }} }}"#
        )
    };
    for keep_shadowed in [false, true] {
        let load = |downfall: &str| {
            loaded_over_lower_packs_keeping(
                &[
                    ("bp/biomes/x.biome.json", biome("1.26.0", "0.1").as_bytes()),
                    (
                        "bp_low/biomes/x.biome.json",
                        biome("beta", downfall).as_bytes(),
                    ),
                ],
                keep_shadowed,
            )
        };
        let (one, other) = (load("0.6"), load("0.5"));
        assert!(
            text_of(&one, "behavior", "Biomes").contains("beta: Some([])"),
            "the beta definition is effective nowhere"
        );
        assert_same_text_and_told_apart("behavior", "Biomes", &one, &other);
    }
}

/// Under a refused top copy a path has no effective file, and with `keep_shadowed` the lower
/// copy is kept in its slot.
#[cfg(feature = "loot")]
#[test]
fn two_copies_under_a_refused_file_that_differ_are_told_apart() {
    let table =
        |rolls: &str| format!(r#"{{ "pools": [ {{ "rolls": {rolls}, "entries": [] }} ] }}"#);
    let load = |rolls: &str| {
        loaded_over_lower_packs(&[
            ("bp/loot_tables/x.json", &b"[1"[..]),
            ("bp_low/loot_tables/x.json", table(rolls).as_bytes()),
        ])
    };
    let (one, other) = (load("2"), load("3"));
    assert!(text_of(&one, "behavior", "LootTables").starts_with("Files { entries: {}, "));
    assert_same_text_and_told_apart("behavior", "LootTables", &one, &other);
}

/// A refused definition that names its identifier claims it; with `keep_shadowed` the lower
/// pack's definition is kept in the slot, effective nowhere.
#[cfg(feature = "feature")]
#[test]
fn two_definitions_under_a_refused_claim_that_differ_are_told_apart() {
    let refused = br#"{ "format_version": "1.13.0", "minecraft:ore_feature": {
        "description": { "identifier": "demo:f" }, "count": "many", "replace_rules": 5 } }"#;
    let feature = |count: &str| {
        format!(
            r#"{{ "format_version": "1.13.0", "minecraft:ore_feature": {{
                 "description": {{ "identifier": "demo:f" }}, "count": {count},
                 "replace_rules": [ {{ "places_block": "minecraft:diamond_ore", "may_replace": [ "minecraft:stone" ] }} ] }} }}"#
        )
    };
    let load = |count: &str| {
        loaded_over_lower_packs(&[
            ("bp/features/f.json", &refused[..]),
            ("bp_low/features/f.json", feature(count).as_bytes()),
        ])
    };
    let (one, other) = (load("8"), load("9"));
    assert!(text_of(&one, "behavior", "Features").starts_with("Registry { entries: {}, "));
    assert_same_text_and_told_apart("behavior", "Features", &one, &other);
}

/// A client entity whose `min_engine_version` is above the stack's is ignored, also as the
/// first of its identifier; with `keep_shadowed` it is kept in its slot.
#[cfg(feature = "client_entity")]
#[test]
fn two_ignored_first_definitions_that_differ_are_told_apart() {
    let entity = |texture: &str| {
        format!(
            r#"{{ "format_version": "1.10.0", "minecraft:client_entity": {{ "description": {{
                 "identifier": "demo:ahead", "min_engine_version": "9.9.9",
                 "textures": {{ "default": "textures/entity/{texture}" }} }} }} }}"#
        )
    };
    let load = |texture: &str| {
        loaded_over_lower_packs(&[("rp/entity/ahead.json", entity(texture).as_bytes())])
    };
    let (one, other) = (load("one"), load("other"));
    assert!(
        text_of(&one, "resource", "ClientEntities").starts_with("Registry { entries: {}, "),
        "the definition is ignored"
    );
    assert_same_text_and_told_apart("resource", "ClientEntities", &one, &other);
}

// ---- Coverage by origin, and entries that hold nothing -----------------------

/// The lines of the coverage check with `document` in place of every fragment file whose
/// path starts with one of `folders`.
#[allow(dead_code, reason = "used behind the families' features")]
fn uncovered_with_folders(folders: &[&str], document: &[u8]) -> Vec<String> {
    let (stack, options) = rewritten_fixture(&|path, bytes| {
        Some(if folders.iter().any(|folder| path.starts_with(folder)) {
            document.to_vec()
        } else {
            bytes
        })
    });
    collections_without_a_fragment_with(&stack, &options)
}

/// A document of the app data is no coverage of a pack fragment. The features and the
/// attachables are fed by both: with the pack's files made a document of another family the
/// app data's document is left in the registry, and the collection must still be named. The
/// app data comes with the options and not through the requests the five orders complete.
#[cfg(all(
    feature = "feature",
    feature = "item",
    feature = "block",
    feature = "attachable"
))]
#[test]
fn a_document_of_the_app_data_is_no_coverage_of_a_pack_fragment() {
    const BLOCK: &[u8] = br#"{ "format_version": "1.26.20", "minecraft:block": {
        "description": { "identifier": "demo:misplaced" }, "components": {} } }"#;
    assert_eq!(
        uncovered_with_folders(&["behavior_pack/features/"], BLOCK),
        [
            "Collection::Features of the behavior stack: no determinism fragment reaches its accessor"
        ]
    );
    assert_eq!(
        uncovered_with_folders(&["resource_pack/attachables/"], BLOCK),
        [
            "Collection::Attachables of the resource stack: no determinism fragment reaches its accessor"
        ]
    );
}

/// An entry is a document, not yet content: a reader that takes any object makes one of
/// `{}`. The loot tables, the UI texture infos and the client blocks are covered by a member
/// only their fragment gives.
#[cfg(all(
    feature = "loot",
    feature = "ui_texture_info",
    feature = "block",
    feature = "blocks_json"
))]
#[test]
fn an_entry_that_holds_nothing_is_no_content() {
    const BLOCK: &[u8] = br#"{ "format_version": "1.26.20", "minecraft:block": {
        "description": { "identifier": "demo:misplaced" }, "components": {} } }"#;
    assert_eq!(
        uncovered_with_folders(&["resource_pack/blocks.json"], BLOCK),
        [
            "Collection::ClientBlocks of the resource stack: no determinism fragment reaches its accessor"
        ]
    );
}

/// The app data's fragments are required beside the packs': without them every registry
/// they reach is named for that side, and the credits, which the app data alone has, as a
/// whole. And the list of those registries is the stacks' own: the members that hold a
/// document of the app data are exactly `APP_DATA_FED`.
#[cfg(feature = "full")]
#[test]
fn the_app_data_fragments_are_required_beside_the_packs() {
    let (stack, options) =
        rewritten_fixture(&|path, bytes| (!path.starts_with(APP_DATA)).then_some(bytes));
    assert_eq!(
        collections_without_a_fragment_with(&stack, &options),
        [
            "Collection::SpawnGroups of the behavior stack: no determinism fragment reaches its accessor (from the app data)",
            "Collection::Features of the behavior stack: no determinism fragment reaches its accessor (from the app data)",
            "Collection::FeatureRules of the behavior stack: no determinism fragment reaches its member underground_pass (from the app data)",
            "Collection::Credits of the resource stack: no determinism fragment reaches its accessor",
            "Collection::Attachables of the resource stack: no determinism fragment reaches its accessor (from the app data)",
        ]
    );
    let (full, _) = snapshot(
        &fixture_stack(),
        Order::Issue,
        Residency::All,
        &everything_on(),
    );
    // The bare stack has documents of no pack too: the behaviour trees built into the game.
    let (bare, _) = snapshot(
        &bare_stack(),
        Order::Issue,
        Residency::All,
        &everything_on_and_no_file(),
    );
    let mut fed: Vec<(String, String)> = Vec::new();
    for (entry, bare) in full.collections().iter().zip(bare.collections()) {
        let (text, bare) = (unescaped(entry.text), unescaped(bare.text));
        for ((member, text), (_, bare)) in members(&text).into_iter().zip(members(&bare)) {
            if is_documents(text) && documents(text).1 > documents(bare).1 {
                fed.push((entry.name.to_owned(), member.to_owned()));
            }
        }
    }

    assert_eq!(
        uncovered_with_folders(&["behavior_pack/loot_tables/"], b"{}"),
        [
            "Collection::LootTables of the behavior stack: no determinism fragment reaches its accessor"
        ]
    );
    assert_eq!(
        uncovered_with_folders(&["resource_pack/textures/ui/"], b"{}"),
        [
            "Collection::UiTextureInfos of the resource stack: no determinism fragment reaches its accessor"
        ]
    );
    // A block document where `blocks.json` is read: its one key becomes a block's name.
    let listed: Vec<(String, String)> = APP_DATA_FED
        .iter()
        .map(|(collection, member)| ((*collection).to_owned(), (*member).to_owned()))
        .collect();
    assert_eq!(fed, listed);
}

// ---- Layers of the types whose `Debug` is partial----------------------------
//
// A layer that is not effective is written into its table by its `Debug`. For the types of
// the five values above that is the partial text: the table gives each such layer in full.

/// A shadowed jigsaw metadata file prints the length of its body like any other.
#[cfg(feature = "jigsaw")]
#[test]
fn two_shadowed_jigsaw_metadata_files_that_differ_in_a_body_byte_are_told_apart() {
    let file = |last: u8| {
        let mut file = b"\x071.21.90\x01".to_vec();
        file.push(last);
        file
    };
    let load = |last: u8| {
        loaded_over_lower_packs(&[
            ("bp/worldgen/metadata/ruin.dat", &file(9)),
            ("bp_low/worldgen/metadata/ruin.dat", &file(last)),
        ])
    };
    let (one, other) = (load(2), load(3));
    assert_eq!(
        text_of(&one, "behavior", "Jigsaw"),
        text_of(&other, "behavior", "Jigsaw")
    );
    assert_told_apart_at(
        "the layers of Jigsaw.metadata of the behavior stack",
        &one,
        &other,
    );
}

/// A shadowed structure prints of its layout what any structure does.
#[cfg(feature = "structure")]
#[test]
fn two_shadowed_structures_that_differ_in_what_their_layout_keeps_are_told_apart() {
    let fragment = std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "tests/fixtures/structure/structure/determinism/behavior_pack/structures/jigsaw.mcstructure",
    ))
    .unwrap();
    let load = |trailing: u8| {
        let mut lower = fragment.clone();
        lower.push(trailing);
        loaded_over_lower_packs(&[
            ("bp/structures/jigsaw.mcstructure", &fragment),
            ("bp_low/structures/jigsaw.mcstructure", &lower),
        ])
    };
    let (one, other) = (load(1), load(2));
    assert_same_text_and_told_apart("behavior", "Structures", &one, &other);
}

/// A shadowed geometry holds nothing its `Debug` leaves out: what a geometry inherits is
/// resolved for the effective definition alone, so a layer under it has `inherited: None`
/// and its own bones in full. A lower pack's parent that differs is told at the table.
#[cfg(feature = "geometry")]
#[test]
fn a_shadowed_geometry_inherits_nothing_and_is_compared_by_its_debug() {
    let file = |pivot: &str| {
        format!(
            r#"{{ "format_version": "1.8.0",
                 "geometry.parent": {{ "bones": [ {{ "name": "body", "pivot": [0, {pivot}, 0] }} ] }},
                 "geometry.child:geometry.parent": {{ "bones": [ {{ "name": "head" }} ] }} }}"#
        )
    };
    let stack_of = |pivot: &str| {
        let mut stack = bare_stack();
        stack
            .file(
                1,
                "bp_low/manifest.json",
                pack_manifest(3, "script", "data"),
            )
            .file(
                1,
                "rp_low/manifest.json",
                pack_manifest(4, "client_script", "resources"),
            )
            .file(1, "rp/models/entity/pair.geo.json", file("1").into_bytes())
            .file(
                1,
                "rp_low/models/entity/pair.geo.json",
                file(pivot).into_bytes(),
            );
        stack
    };
    let options = LoadOptions {
        keep_shadowed: true,
        ..everything_on_and_no_file()
    };
    let load = |pivot: &str| {
        loaded_in(
            &stack_of(pivot),
            Order::Issue,
            Residency::All,
            &options,
            &world_of(&[1, 3], &[2, 4]),
        )
    };
    let (one, other) = (load("2"), load("3"));
    let mut shadowed = 0;
    for (id, layers) in one.resource.geometries().all_layers() {
        assert_eq!(layers.len(), 2, "{id:?}");
        for layer in layers.skip(1) {
            assert!(layer.inherited.is_none(), "{id:?}: {:?}", layer.inherited);
            shadowed += 1;
        }
    }
    assert_eq!(shadowed, 2);
    let (one, other) = (snapshot_of(&one), snapshot_of(&other));
    assert_same_text_and_told_apart("resource", "Geometries", &one, &other);
}

/// A material file and a UI file of a lower pack are no shadowed layer of a registry: both
/// collections merge every pack's copy and print each copy among their layers, so a lower
/// copy that differs under a higher one is told by the collection's own text.
#[cfg(all(feature = "material", feature = "json_ui"))]
#[test]
fn a_lower_copy_of_a_material_or_a_ui_file_is_told_by_the_collection_s_debug() {
    let material = |variant: &str| {
        format!(
            r#"{{ "materials": {{ "version": "1.0.0",
                 "base": {{ "variants": [ {{ "{variant}": {{}} }} ] }} }} }}"#
        )
    };
    let list = br#"[ { "path": "materials/a.material" } ]"#;
    let load = |variant: &str| {
        loaded_over_lower_packs(&[
            ("rp/materials/common.json", &list[..]),
            ("rp/materials/a.material", material("top").as_bytes()),
            ("rp_low/materials/a.material", material(variant).as_bytes()),
        ])
    };
    let differences = collections::differences(&load("x"), &load("y"));
    assert!(
        differences
            .iter()
            .any(|line| line.starts_with("collection Materials of the resource stack: ")),
        "{differences:#?}"
    );
    let ui = |size: &str| {
        format!(
            r#"{{ "namespace": "pair", "base": {{ "type": "panel", "size": [ {size}, 1 ] }} }}"#
        )
    };
    let defs = br#"{ "ui_defs": [ "ui/pair.json" ] }"#;
    let load = |size: &str| {
        loaded_over_lower_packs(&[
            ("rp/ui/_ui_defs.json", &defs[..]),
            ("rp/ui/pair.json", ui("9").as_bytes()),
            ("rp_low/ui/_ui_defs.json", &defs[..]),
            ("rp_low/ui/pair.json", ui(size).as_bytes()),
        ])
    };
    let differences = collections::differences(&load("1"), &load("2"));
    assert!(
        differences
            .iter()
            .any(|line| line.starts_with("collection Ui of the resource stack: ")),
        "{differences:#?}"
    );
}

// ---- What linking says-------------------------------------------------------
//
// `StackSet::link()` reads what no collection holds: a pack's resident bytes (a legacy client
// item file, an image's header) and the file listing of the behaviour stack's packs. Both are
// built by the completions, so the comparison takes what linking says as one more text.

/// Requires `link()` to be the only place two loads differ.
#[allow(dead_code, reason = "used behind the families' features")]
#[track_caller]
fn assert_told_apart_by_linking_alone(one: &Snapshot, other: &Snapshot) {
    let differences = collections::differences(one, other);
    assert_eq!(
        differences.len(),
        1,
        "one difference, what linking says:\n{}",
        differences.join("\n")
    );
    assert!(
        differences[0].starts_with("what linking says: "),
        "{differences:#?}"
    );
}

/// A legacy client item file (`items/*.json` of a resource pack) is in no collection: the
/// link pass reads its bytes from the pack. Two loads whose file names another item differ
/// in what linking says and nowhere else.
#[cfg(feature = "item")]
#[test]
fn two_files_no_collection_holds_that_differ_are_told_apart_by_what_linking_says() {
    let item = r#"{ "format_version": "1.26.30", "minecraft:item": {
        "description": { "identifier": "demo:twin" }, "components": {} } }"#;
    let client = |identifier: &str| {
        format!(
            r#"{{ "format_version": "1.10", "minecraft:item": {{
                 "description": {{ "identifier": "{identifier}" }},
                 "components": {{ "minecraft:icon": "stick" }} }} }}"#
        )
    };
    let load = |identifier: &str| {
        loaded_with(&[
            ("bp/items/twin.json", item.as_bytes()),
            ("rp/items/x.json", client(identifier).as_bytes()),
        ])
    };
    let ((one_set, one), (other_set, other)) = (load("demo:twin"), load("demo:lone"));
    let (one_link, other_link) = (
        format!("{:?}", one_set.link()),
        format!("{:?}", other_set.link()),
    );
    assert_ne!(one_link, other_link);
    assert!(other_link.contains("demo:lone") && !one_link.contains("demo:lone"));
    assert_told_apart_by_linking_alone(&one, &other);
}

/// The file listing of a behaviour stack's pack is printed as its length. A template pool
/// that names a legacy structure of a game pack (`structures/**/*.nbt`, never read) resolves
/// by the listing: two game packs that list another name, with as many files, differ in what
/// linking says and nowhere else.
#[cfg(feature = "jigsaw")]
#[test]
fn two_listings_that_differ_in_a_name_are_told_apart_by_what_linking_says() {
    let pool = br#"{ "format_version": "1.21.20", "minecraft:template_pool": {
        "description": { "identifier": "demo:start" },
        "elements": [ { "element": { "element_type": "minecraft:single_pool_element",
                                     "location": "trail_ruins/tower/tower_1" } } ] } }"#;
    let load = |structure: &str| {
        let mut stack = bare_stack();
        stack
            .file(1, "bp/worldgen/template_pools/pool.json", pool.to_vec())
            .file(2, "game/manifest.json", pack_manifest(5, "script", "data"))
            .file(
                2,
                &format!("game/structures/trail_ruins/tower/{structure}.nbt"),
                b"not read".to_vec(),
            );
        let mut responder = stack.reordered(Order::Issue);
        let mut repository = RepositoryLoader::new(
            vec![
                Source::Directory {
                    root: RootId(1),
                    origin: PackOrigin::World,
                },
                Source::Directory {
                    root: RootId(2),
                    origin: PackOrigin::Package,
                },
            ],
            OpenOptions::default(),
        );
        let repository = responder.drive(&mut repository);
        let set = repository.stacks(&world_of(&[1, 5], &[2]), everything_on_and_no_file());
        let options = StackOptions {
            behavior: Residency::All,
            resource: Residency::All,
            ..StackOptions::default()
        };
        let mut loader = StackLoader::new(set, options);
        let set = responder.drive(&mut loader);
        let snapshot = snapshot_of(&set);
        (set, snapshot)
    };
    let ((one_set, one), (other_set, other)) = (load("tower_1"), load("tower_2"));
    let (one_link, other_link) = (
        format!("{:?}", one_set.link()),
        format!("{:?}", other_set.link()),
    );
    assert!(
        !one_link.contains("trail_ruins/tower/tower_1")
            && other_link.contains("trail_ruins/tower/tower_1"),
        "the pool's structure resolves by the listing"
    );
    assert_told_apart_by_linking_alone(&one, &other);
}
