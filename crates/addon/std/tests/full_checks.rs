//! Questions about the game's own packs, each answered with a number that is pinned here. The
//! packs are not in the repository: the tests read them from the folder `BEDROCK_ADDON_CORPUS`
//! names and skip without it.
//!
//! - **Filter tests.** The table of filter tests has 107 of the game's 110 names; a name it
//!   lacks is the warning "Filter test not found -> …". Every entity, spawn-rule, feature-rule
//!   and item document of the four installs is loaded, each pack alone so that no document is
//!   shadowed, and the warning is counted by name: none. The warning is raised where a family
//!   checks a filter, and not every place does, so the names are also taken from the
//!   documents as they write themselves and asked of the table: every test the game's packs
//!   use (89 names over the four installs) is in it.
//! - **Base-game slices.** A world whose base-game version is `*` (any version) or invalid
//!   gets every slice: for each install, the slices on the behaviour stack of a world whose
//!   base-game version is no version are counted against the slices the install ships.
//! - **Molang occurrences.** Every (pack, file, JSON pointer) occurrence of
//!   `corpus_pairs.json` is some family's to check; the families' smoke tests check their own
//!   share (`common/pairs.rs`). Here the total is taken: every occurrence is in a family whose
//!   check is named, or is counted as unclaimed by its path prefix. The behaviour-pack
//!   entities, which no smoke test checks, are checked here.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<folder> cargo nextest run -p bedrock_addon_std --test
//! full_checks -- --show-output`; without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;
#[path = "common/pairs.rs"]
mod pairs;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticCode, JsonPath, VanillaText};
use bedrock_addon::document::Content;
use bedrock_addon::families::entity::Entity;
use bedrock_addon::json::{JsonSlice, Value, parse_at};
use bedrock_addon::load::{Collection, CollectionSet, LoadOptions, StackOptions};
use bedrock_addon::pack::{PackOrigin, PackRepository, PackStack, PackType, StackSet, WorldPacks};
use bedrock_addon::shared::Filter;
use bedrock_addon::version::{BaseGameVersion, SemVersion};
use bedrock_addon_std::{DirSource, Driver, Options, read_app_data};

use common::game_files;

/// A driver that opens packs as the app's own.
fn base_game_driver() -> Driver {
    let mut options = Options::default();
    options.open.origin = PackOrigin::Package;
    Driver::new(options).expect("a driver")
}

/// The pack folders directly below `dir` (those with a manifest), sorted.
fn packs_in(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.join("manifest.json").is_file())
        .collect();
    dirs.sort();
    dirs
}

// ---- filter tests -------------------------------------------------------------------------------

/// What the filter-carrying documents of one install came to.
#[derive(Debug, Default, PartialEq, Eq)]
struct Filters {
    /// Behaviour packs loaded, and whether the install's app data was (its feature rules).
    packs: usize,
    entities: usize,
    spawn_rules: usize,
    feature_rules: usize,
    items: usize,
    /// "Filter test not found -> <name>" warnings, by name.
    unknown: BTreeMap<String, usize>,
    /// Every filter test of the loaded documents as they write themselves, by name: the uses
    /// at the places the families check and at those they do not.
    used: BTreeMap<String, usize>,
    /// Documents that do not write themselves, and so are not in `used`.
    unwritten: usize,
}

impl Filters {
    /// The names of `used` the table lacks, asked of the table itself: the warning
    /// `Filter::check` gives for a test of that name alone.
    fn used_and_unknown(&self) -> Vec<&str> {
        self.used
            .keys()
            .map(String::as_str)
            .filter(|name| {
                let text = format!(r#"{{"test": "{name}"}}"#);
                let filter: Filter = parse_at(JsonSlice::new(&text)).expect("a filter test");
                filter
                    .check(&JsonPath::new())
                    .iter()
                    .any(|finding| unknown_filter_test(finding).is_some())
            })
            .collect()
    }
}

/// Counts the filter tests of `document` by name: every object with a string member `test`
/// (in the four families a member of that name is a filter test's and nothing else's).
fn tests_of(document: &Value, into: &mut BTreeMap<String, usize>) {
    if let Some(object) = document.as_object() {
        if let Some(name) = object.get("test").and_then(Value::as_string) {
            *into.entry(name.as_str().to_owned()).or_insert(0) += 1;
        }
        for (_, member) in object {
            tests_of(member, into);
        }
    } else if let Some(array) = document.as_array() {
        for element in array {
            tests_of(element, into);
        }
    }
}

/// Adds the filter tests of one loaded document, as it writes itself, to `found`.
fn count_written<T: Content>(document: &T, found: &mut Filters) {
    match document.to_json() {
        Ok(text) => {
            let value: Value = parse_at(JsonSlice::new(&text)).expect("a written document");
            tests_of(&value, &mut found.used);
        }
        Err(_) => found.unwritten += 1,
    }
}

/// The four collections whose documents hold filters.
fn filter_collections() -> StackOptions {
    StackOptions {
        collections: CollectionSet::only([
            Collection::Entities,
            Collection::SpawnRules,
            Collection::FeatureRules,
            Collection::Items,
        ]),
        ..StackOptions::default()
    }
}

/// Adds what the behaviour stack `set` holds to `found`.
fn count_filters(set: &StackSet, found: &mut Filters) {
    let stack = &set.behavior;
    found.entities += stack.entities().len();
    found.spawn_rules += stack.spawn_rules().len();
    found.feature_rules += stack.feature_rules().len();
    found.items += stack.items().len();
    for (_, document) in stack.entities() {
        count_written(document.value(), found);
    }
    for (_, document) in stack.spawn_rules() {
        count_written(document.value(), found);
    }
    for (_, document) in stack.feature_rules() {
        count_written(document.value(), found);
    }
    for (_, document) in stack.items() {
        count_written(document.value(), found);
    }
    let findings = stack
        .entities()
        .diagnostics()
        .chain(stack.spawn_rules().diagnostics())
        .chain(stack.feature_rules().diagnostics())
        .chain(stack.items().diagnostics());
    for finding in findings {
        if let Some(name) = unknown_filter_test(finding) {
            *found.unknown.entry(name.to_owned()).or_insert(0) += 1;
        }
    }
}

/// The name of the filter test `finding` did not find, when it is that warning.
fn unknown_filter_test(finding: &Diagnostic) -> Option<&str> {
    match &finding.code {
        DiagnosticCode::UnresolvedReference {
            text: Some(VanillaText::FilterTestNotFound { name }),
        } => Some(name),
        _ => None,
    }
}

/// Loads every behaviour pack in `packs`, each as a stack of its own, and the app data below
/// `app_data` (an install's feature rules are there) on a stack without packs.
fn filters_of(packs: &[PathBuf], app_data: Option<&Path>) -> Filters {
    let mut found = Filters::default();
    let mut driver = base_game_driver();
    for dir in packs {
        let pack = driver
            .pack_dir(dir)
            .unwrap_or_else(|error| panic!("{}: {error:?}", dir.display()));
        if pack.pack_type() != PackType::Behavior {
            continue;
        }
        let set = StackSet {
            behavior: PackStack::of([pack], LoadOptions::default()).expect("a stack"),
            resource: PackStack::of([], LoadOptions::default()).expect("a stack"),
        };
        let set = driver
            .load_stacks(set, filter_collections())
            .expect("the stack loads");
        found.packs += 1;
        count_filters(&set, &mut found);
    }
    if let Some(root) = app_data {
        let options = LoadOptions {
            app_data: Some(read_app_data(root).expect("the app data")),
            ..LoadOptions::default()
        };
        let set = StackSet {
            behavior: PackStack::of([], options.clone()).expect("a stack"),
            resource: PackStack::of([], options).expect("a stack"),
        };
        let set = driver
            .load_stacks(set, filter_collections())
            .expect("the stack loads");
        count_filters(&set, &mut found);
    }
    found
}

/// `(packs, entities, spawn rules, feature rules, items, uses of a filter test, distinct filter
/// tests)` of `found`, after requiring that no filter test is unknown: neither to the loader,
/// where a family checks a filter, nor among the names the written documents use anywhere.
#[track_caller]
fn assert_every_filter_test_is_known(source: &str, found: &Filters) -> [usize; 7] {
    eprintln!(
        "{source}: {} packs, {} entities, {} spawn rules, {} feature rules, {} items; {} warnings; {} uses of {} filter tests in the written documents ({} documents not written): {:?}",
        found.packs,
        found.entities,
        found.spawn_rules,
        found.feature_rules,
        found.items,
        found.unknown.values().sum::<usize>(),
        found.used.values().sum::<usize>(),
        found.used.len(),
        found.unwritten,
        found.used.keys().collect::<Vec<_>>()
    );
    assert!(
        found.unknown.is_empty(),
        "{source}: filter tests the table lacks: {:?}",
        found.unknown
    );
    assert_eq!(found.unwritten, 0, "{source}: every document writes itself");
    assert_eq!(
        found.used_and_unknown(),
        [""; 0],
        "{source}: filter tests in use that the table lacks"
    );
    [
        found.packs,
        found.entities,
        found.spawn_rules,
        found.feature_rules,
        found.items,
        found.used.values().sum(),
        found.used.len(),
    ]
}

/// The counts see what they count: a pack whose one entity uses four tests of no name the
/// table has, beside one it has. Needs no game files.
#[test]
fn filter_tests_the_table_lacks_are_counted_by_name() {
    let dir = tempfile::tempdir().unwrap();
    let pack = dir.path().join("bp");
    std::fs::create_dir_all(pack.join("entities")).unwrap();
    std::fs::write(
        pack.join("manifest.json"),
        r#"{
    "format_version": 2,
    "header": { "name": "filters", "description": "", "uuid": "5e1f0a3c-3a61-4b1e-9e0e-3c1f5b0a6d21",
        "version": [1, 0, 0], "min_engine_version": [1, 21, 0] },
    "modules": [{ "type": "data", "uuid": "9a0d3c1e-3f7b-4c55-8a39-5b8e6c1d2f40", "version": [1, 0, 0] }]
}"#,
    )
    .unwrap();
    std::fs::write(
        pack.join("entities/probe.json"),
        r#"{
    "format_version": "1.21.0",
    "minecraft:entity": {
        "description": { "identifier": "demo:probe", "is_spawnable": true, "is_summonable": true },
        "components": {
            "minecraft:on_death": {
                "filters": { "all_of": [
                    { "test": "is_family", "subject": "other", "value": "player" },
                    { "test": "no_such_test", "value": 1 }
                ] },
                "event": "demo:noticed",
                "target": "self"
            },
            "minecraft:environment_sensor": {
                "triggers": [
                    { "filters": { "test": "no_such_test_b", "value": 1 }, "event": "demo:noticed" }
                ]
            },
            "minecraft:damage_sensor": {
                "triggers": [
                    { "on_damage": { "filters": { "test": "no_such_test_c", "value": 1 } }, "deals_damage": "no" }
                ]
            },
            "minecraft:behavior.nearest_attackable_target": {
                "priority": 1,
                "entity_types": [ { "filters": { "test": "no_such_test_d", "value": 1 }, "max_dist": 8 } ]
            }
        },
        "events": { "demo:noticed": {} }
    }
}"#,
    )
    .unwrap();
    let found = filters_of(&[pack], None);
    assert_eq!((found.packs, found.entities), (1, 1), "{found:?}");
    // The loader's warning, where the family checks the filter: a trigger component and a
    // goal's `entity_types`.
    assert_eq!(found.unknown.get("no_such_test"), Some(&1), "{found:?}");
    assert_eq!(found.unknown.get("no_such_test_d"), Some(&1), "{found:?}");
    // The written document names all four, whoever checks them: the triggers of
    // `environment_sensor` and of `damage_sensor` raise no warning for an unknown test name,
    // which is why the game's packs are also read this way.
    assert_eq!(found.unwritten, 0);
    assert_eq!(
        found.used_and_unknown(),
        [
            "no_such_test",
            "no_such_test_b",
            "no_such_test_c",
            "no_such_test_d"
        ]
    );
    assert_eq!(found.used.get("is_family"), Some(&1));
}

#[test]
fn samples_use_no_filter_test_the_table_lacks() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let found = filters_of(&[samples.join("behavior_pack")], None);
    assert_eq!(
        assert_every_filter_test_is_known("samples", &found),
        [1, 128, 60, 0, 79, 2085, 86]
    );
}

#[test]
fn server_install_uses_no_filter_test_the_table_lacks() {
    let server = skip!(game_files::SERVER_INSTALL);
    let found = filters_of(&packs_in(&server.join("behavior_packs")), Some(&server));
    assert_eq!(
        assert_every_filter_test_is_known("server_install", &found),
        // 68 of these uses are the four `is_family` tests of the enderman's teleport
        // `projectile_filter`, which the 1.26.50 upgrade step writes into every older enderman.
        [68, 1190, 143, 422, 111, 20950, 88]
    );
}

#[test]
fn client_install_a_uses_no_filter_test_the_table_lacks() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    let found = filters_of(&packs_in(&client.join("behavior_packs")), Some(&client));
    assert_eq!(
        assert_every_filter_test_is_known("client_install_a", &found),
        // Two creaking documents carry a root member beside the version and the payload
        // (`use_beta_features`); the game passes it over, so they load and their filters
        // count. The enderman uses of the 1.26.50 upgrade step are in the numbers as above.
        [75, 1166, 137, 421, 111, 20412, 86]
    );
}

#[test]
fn client_install_b_uses_no_filter_test_the_table_lacks() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let found = filters_of(&packs_in(&client.join("behavior_packs")), Some(&client));
    assert_eq!(
        assert_every_filter_test_is_known("client_install_b", &found),
        // The enderman uses of the 1.26.50 upgrade step are in the numbers, as above.
        [71, 1198, 146, 434, 113, 21166, 89]
    );
}

// ---- base-game slices ---------------------------------------------------------------------------

/// The `vanilla` behaviour pack and its slices (one identity, the slice's version).
const VANILLA_BEHAVIOR: uuid::Uuid = uuid::Uuid::from_u128(0xfe9f8597_5454_481a_8730_8d070a8e2e58);

/// Whether `pack` is a `vanilla` behaviour slice: `vanilla`'s identity at a version of its
/// own (the pack itself is 0.0.1).
fn is_slice(id: &bedrock_addon::pack::PackIdVersion) -> bool {
    id.uuid == VANILLA_BEHAVIOR
        && id.pack_type == PackType::Behavior
        && id.version != SemVersion::new(0, 0, 1)
}

/// The packs of an install, registered as the app's own.
fn install_repository(root: &Path) -> PackRepository {
    base_game_driver()
        .repository([
            DirSource::packs(root.join("behavior_packs"), PackOrigin::Package),
            DirSource::packs(root.join("resource_packs"), PackOrigin::Package),
        ])
        .expect("the repository")
}

/// How many slices are on the behaviour stack of a world that lists no pack and has `world`
/// as its base-game version.
fn slices_selected(repository: &PackRepository, world: BaseGameVersion) -> usize {
    let options = LoadOptions {
        base_game_version: world,
        ..LoadOptions::default()
    };
    let nothing = WorldPacks {
        behavior: Vec::new(),
        resource: Vec::new(),
    };
    let set = repository.stacks(&nothing, options);
    let slices: Vec<SemVersion> = set
        .behavior
        .packs()
        .filter(|pack| is_slice(pack.id()))
        .map(|pack| pack.id().version.clone())
        .collect();
    assert!(
        slices.windows(2).all(|pair| pair[0] > pair[1]),
        "newest first: {slices:?}"
    );
    slices.len()
}

/// For the install at `root`: the slices it ships, and how many of them a world selects whose
/// base-game version is `*`, unset, unparsable, never compatible, 1.20.0 and 1.12.0.
///
/// The first five are equal: a world whose base-game version is `*`, unset, unparsable or never
/// compatible gets every slice, as the game's slice selection asks only whether the version is
/// any or invalid. The fixture test is `crates/addon/addon/tests/pack_vanilla.rs`. A version
/// selects the slices not above it: 26 of each install are at or below 1.20.0, and none is as
/// old as 1.12.0. The sample packs are one pack and no install: they have no slice.
fn slices_of(source: &str, root: &Path) -> [usize; 7] {
    let repository = install_repository(root);
    let shipped = repository
        .packs()
        .filter(|pack| is_slice(pack.id()))
        .count();
    let unparsable = BaseGameVersion {
        version: SemVersion::parse("1.twenty"),
        never_compatible: false,
    };
    assert_eq!(unparsable.version, None);
    let at = |major, minor, patch| BaseGameVersion::new(SemVersion::new(major, minor, patch));
    let counts = [
        shipped,
        slices_selected(&repository, BaseGameVersion::ANY),
        slices_selected(&repository, BaseGameVersion::EMPTY),
        slices_selected(&repository, unparsable),
        slices_selected(&repository, BaseGameVersion::INCOMPATIBLE),
        slices_selected(&repository, at(1, 20, 0)),
        slices_selected(&repository, at(1, 12, 0)),
    ];
    eprintln!(
        "{source}: {} slices shipped; selected for `*` {}, unset {}, unparsable {}, never compatible {}, 1.20.0 {}, 1.12.0 {}",
        counts[0], counts[1], counts[2], counts[3], counts[4], counts[5], counts[6]
    );
    counts
}

#[test]
fn a_world_without_a_base_game_version_gets_every_slice_of_the_server_install() {
    let server = skip!(game_files::SERVER_INSTALL);
    assert_eq!(
        slices_of("server_install", &server),
        [53, 53, 53, 53, 53, 26, 0]
    );
}

#[test]
fn a_world_without_a_base_game_version_gets_every_slice_of_client_install_a() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    assert_eq!(
        slices_of("client_install_a", &client),
        [51, 51, 51, 51, 51, 26, 0]
    );
}

#[test]
fn a_world_without_a_base_game_version_gets_every_slice_of_client_install_b() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    assert_eq!(
        slices_of("client_install_b", &client),
        [54, 54, 54, 54, 54, 26, 0]
    );
}

// ---- Molang occurrences -------------------------------------------------------------------------

/// The occurrences of `corpus_pairs.json` per family (the first folder or file name of the
/// occurrence's path), as `(pack type folder, family) → count`.
fn occurrences_by_family(pairs: &Path) -> BTreeMap<(String, String), usize> {
    let text = std::fs::read_to_string(pairs).unwrap();
    let document: Value = parse_at(JsonSlice::new(&text)).unwrap();
    let mut families = BTreeMap::new();
    let pairs = document.as_object().unwrap().get("pairs").unwrap();
    for pair in pairs.as_array().unwrap() {
        let occurrences = pair.as_object().unwrap().get("occurrences").unwrap();
        for occurrence in occurrences.as_array().unwrap() {
            let o = occurrence.as_object().unwrap();
            let field = |key: &str| o.get(key).unwrap().as_string().unwrap().as_str().to_owned();
            *families
                .entry((field("pack_type"), field("family")))
                .or_insert(0) += 1;
        }
    }
    families
}

/// Who checks the occurrences of one family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Claim {
    /// The smoke test of the family, by file name: it holds the family's name and reads
    /// `corpus_pairs.json`.
    Smoke(&'static str),
    /// [`every_behavior_entity_molang_occurrence_loads_with_its_text_and_version`], below.
    Here,
    /// No family reads the file, or the text is no Molang: the reason.
    Unclaimed(&'static str),
}

/// A file a pack keeps outside every folder a family reads.
const STRAY: Claim = Claim::Unclaimed(
    "a copy of an entity file at the root of the behaviour pack `vanilla_1.20.10`, outside `entities/`: the game reads no such path and no family walks it",
);

/// Every family of `corpus_pairs.json`: the pack type folder, the family, its occurrences and
/// who checks them.
const FAMILIES: [(&str, &str, usize, Claim); 22] = [
    (
        "behavior_packs",
        "biomes",
        4,
        Claim::Smoke("smoke_biome.rs"),
    ),
    (
        "behavior_packs",
        "blocks",
        9,
        Claim::Smoke("smoke_block.rs"),
    ),
    ("behavior_packs", "donkey.json", 2, STRAY),
    ("behavior_packs", "entities", 1587, Claim::Here),
    (
        "behavior_packs",
        "feature_rules",
        214,
        Claim::Smoke("smoke_worldgen.rs"),
    ),
    (
        "behavior_packs",
        "features",
        155,
        Claim::Smoke("smoke_feature.rs"),
    ),
    ("behavior_packs", "horse.json", 2, STRAY),
    ("behavior_packs", "items", 29, Claim::Smoke("smoke_item.rs")),
    ("behavior_packs", "llama.json", 2, STRAY),
    ("behavior_packs", "mule.json", 1, STRAY),
    ("behavior_packs", "trader_llama.json", 2, STRAY),
    (
        "resource_packs",
        "animation_controllers",
        2141,
        Claim::Smoke("smoke_animation.rs"),
    ),
    (
        "resource_packs",
        "animations",
        3283,
        Claim::Smoke("smoke_animation.rs"),
    ),
    (
        "resource_packs",
        "attachables",
        152,
        Claim::Smoke("smoke_attachable.rs"),
    ),
    (
        "resource_packs",
        "ddui",
        1,
        Claim::Unclaimed(
            "`Context.List` in `ddui/root/custom_form.json`: a data binding of the data-driven UI, which the pairs' selection takes for Molang by its `Context.` prefix; the family holds `attribs` opaque and compiles no Molang",
        ),
    ),
    (
        "resource_packs",
        "entity",
        2549,
        Claim::Smoke("smoke_client_entity.rs"),
    ),
    (
        "resource_packs",
        "materials",
        12,
        Claim::Unclaimed(
            "`shaders/texture.fragment`, a shader path of a `.material` file, which the pairs' selection takes for Molang by its `texture.` token; material files hold no Molang",
        ),
    ),
    (
        "resource_packs",
        "models",
        10,
        Claim::Smoke("smoke_geometry.rs"),
    ),
    (
        "resource_packs",
        "particles",
        1730,
        Claim::Smoke("smoke_particle.rs"),
    ),
    (
        "resource_packs",
        "render_controllers",
        2745,
        Claim::Smoke("smoke_render_controller.rs"),
    ),
    (
        "resource_packs",
        "root",
        1,
        Claim::Unclaimed(
            "`root/custom_form.json` of the resource pack `vanilla_1.21.130`, a copy of the data-driven UI root outside `ddui/`: no family reads the folder `root/`, and its one string is the `Context.List` binding",
        ),
    ),
    (
        "resource_packs",
        "sounds.json",
        18,
        Claim::Smoke("smoke_sounds.rs"),
    ),
];

/// Every occurrence of `corpus_pairs.json`.
const OCCURRENCES: usize = 14_649;
/// The occurrences no family checks: 9 in stray entity files, 12 shader paths, 2 UI bindings.
const UNCLAIMED: usize = 23;

/// The total over the families' checks: every occurrence of the file is in a family of the
/// table, each family's count is the file's, a family with a smoke test is named in that test's
/// source beside the pairs file, and what no family checks is 23 occurrences in eight path
/// prefixes, none of them a Molang expression the game compiles from that place.
#[test]
fn every_molang_occurrence_of_the_molang_pairs_is_some_family_s_or_counted() {
    let pairs_path = skip!(game_files::MOLANG_PAIRS);
    let found = occurrences_by_family(&pairs_path);
    let expected: BTreeMap<(String, String), usize> = FAMILIES
        .iter()
        .map(|(pack_type, family, count, _)| {
            (((*pack_type).to_owned(), (*family).to_owned()), *count)
        })
        .collect();
    assert_eq!(found, expected, "the families of corpus_pairs.json");
    assert_eq!(found.values().sum::<usize>(), OCCURRENCES);

    let tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut unclaimed = 0;
    let mut claimed = 0;
    for (pack_type, family, count, claim) in FAMILIES {
        match claim {
            Claim::Smoke(file) => {
                let source = std::fs::read_to_string(tests.join(file)).unwrap();
                assert!(
                    source.contains(&format!("\"{family}\"")) && source.contains("CORPUS_PAIRS"),
                    "{file} checks the `{family}` occurrences of the pairs"
                );
                claimed += count;
            }
            Claim::Here => claimed += count,
            Claim::Unclaimed(why) => {
                eprintln!("unclaimed: {count} in {pack_type}/*/{family}: {why}");
                unclaimed += count;
            }
        }
    }
    eprintln!(
        "molang pairs: {OCCURRENCES} occurrences, {claimed} in a family's check, {unclaimed} unclaimed"
    );
    assert_eq!(unclaimed, UNCLAIMED);
    assert_eq!(claimed + unclaimed, OCCURRENCES);
}

/// The members of a behaviour entity that hold Molang and are no Molang field of the loaded
/// document: the family keeps their components as JSON and compiles the members itself when it
/// validates the document (`rideable`, `ambient_sound_interval`, `heartbeat`,
/// `reflect_projectiles`; the `set_property` responses of an event), or holds a descriptor's
/// `tags` as text. Each with its occurrences in the pairs.
// `rideable.seats[].rotate_rider_by` and `behavior.tempt.items[].tags` are typed fields of the
// 1.26.60 component and goal keys, so they are read back from the loaded document and are not
// listed here.
const READ_FROM_THE_WRITTEN_DOCUMENT: [(&str, usize); 5] = [
    (
        "minecraft:ambient_sound_interval/event_names/*/condition",
        32,
    ),
    ("minecraft:heartbeat/interval", 5),
    ("minecraft:reflect_projectiles/azimuth_angle", 3),
    ("minecraft:navigation.walk/blocks_to_avoid/*/tags", 3),
    ("set_property/*", 2),
];

/// The member kind of [`READ_FROM_THE_WRITTEN_DOCUMENT`] the pointer names: the component and
/// what follows it, an index written `*`; a `set_property` response by that name alone.
fn member_kind(pointer: &str) -> String {
    let segments: Vec<&str> = pointer.split('/').collect();
    if let Some(at) = segments
        .iter()
        .position(|segment| *segment == "set_property")
    {
        return format!("{}/*", segments[at]);
    }
    // `/minecraft:entity/components/<component>/…` or
    // `/minecraft:entity/component_groups/<group>/<component>/…`.
    let from = match segments.get(2) {
        Some(&"components") => 3,
        Some(&"component_groups") => 4,
        _ => 0,
    };
    segments[from..]
        .iter()
        .map(|segment| {
            if segment.parse::<usize>().is_ok() {
                "*"
            } else {
                segment
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Adds every string of `value` to `into` under its JSON pointer, with `version`, unless a
/// member is there already.
fn strings_of(
    value: &Value,
    pointer: &mut String,
    version: bedrock_addon::molang::RawVersion,
    into: &mut pairs::Members,
) {
    let length = pointer.len();
    if let Some(text) = value.as_string() {
        into.entry(pointer.clone())
            .or_insert_with(|| (text.as_str().to_owned(), Some(version)));
    } else if let Some(object) = value.as_object() {
        for (key, member) in object {
            pointer.push('/');
            pointer.push_str(key.as_str());
            strings_of(member, pointer, version, into);
            pointer.truncate(length);
        }
    } else if let Some(array) = value.as_array() {
        for (index, element) in array.iter().enumerate() {
            pointer.push('/');
            pointer.push_str(&index.to_string());
            strings_of(element, pointer, version, into);
            pointer.truncate(length);
        }
    }
}

/// Every behaviour-pack entity occurrence of `corpus_pairs.json` (family `entities`: 1587 in
/// 955 files of 51 packs) loads: the file through the whole chain with no error, and the
/// expression at the occurrence's pointer with the pair's text and version.
///
/// 1542 occurrences are Molang fields of the loaded document and are read back from it, text
/// and version. The other 45 are the five member kinds of
/// [`READ_FROM_THE_WRITTEN_DOCUMENT`]: the loaded document holds no Molang value for them, so
/// the text is read from the document as it writes itself, at the same pointer, and the version
/// is the load context's, which is the one the family compiles them with.
#[test]
fn every_behavior_entity_molang_occurrence_loads_with_its_text_and_version() {
    use std::collections::BTreeSet;

    let pairs_path = skip!(game_files::MOLANG_PAIRS);
    let install = skip!(game_files::CLIENT_INSTALL_B);
    // Per file, the pointers that are Molang fields of the loaded document.
    let mut fields: BTreeMap<PathBuf, BTreeSet<String>> = BTreeMap::new();
    let found = pairs::check(&pairs_path, &install, "entities", |file| {
        let document = Entity::parse(&file.at, file.text.as_bytes(), &file.cx)
            .map_err(|rejected| rejected.diagnostics)?;
        let diagnostics = document.diagnostics().clone();
        let written = document.to_json().expect("the entity writes itself");
        let written: Value =
            parse_at(JsonSlice::new(&written)).expect("the written entity is JSON");
        let mut entity = document.into_value();
        let mut members = pairs::Members::new();
        pairs::members(&mut entity, Some("minecraft:entity"), &mut members);
        fields.insert(file.path.to_path_buf(), members.keys().cloned().collect());
        strings_of(
            &written,
            &mut String::new(),
            file.cx.molang_version,
            &mut members,
        );
        Ok((members, diagnostics))
    });
    assert!(
        found.failures.is_empty(),
        "{} failures: {:#?}",
        found.failures.len(),
        &found.failures[..found.failures.len().min(20)]
    );
    let mut written: BTreeMap<String, usize> = BTreeMap::new();
    for (path, pointer) in &found.matched {
        if !fields[path].contains(pointer) {
            *written.entry(member_kind(pointer)).or_insert(0) += 1;
        }
    }
    let from_fields = found.compared - written.values().sum::<usize>();
    eprintln!(
        "molang pairs, entities: {} occurrences in {} files; {from_fields} are Molang fields of the loaded document, the others are read from the written document: {written:?}",
        found.compared, found.files
    );
    assert_eq!((found.compared, found.keys, found.files), (1587, 0, 955));
    // The typed fields of the 1.26.60 keys account for 1542 of the occurrences.
    assert_eq!(from_fields, 1542);
    let expected: BTreeMap<String, usize> = READ_FROM_THE_WRITTEN_DOCUMENT
        .iter()
        .map(|(kind, count)| ((*kind).to_owned(), *count))
        .collect();
    assert_eq!(written, expected);
}
