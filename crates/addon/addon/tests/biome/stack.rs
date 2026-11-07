//! The registry of `bp.biomes()`: the walk of `biomes/`, the first-identifier rule and its
//! duplicate error, beta documents beside stable ones, and
//! `Placed::default_path`. Row `b12`.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::{Content, Placed};
use bedrock_addon::families::biome::Biome;
use bedrock_addon::id::BiomeId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, PackOrigin, PackStack};

use super::common::{biome, document, manifest, pack};

/// A behaviour pack of `origin` with the biome files `files`.
fn biome_pack(
    uuid: u128,
    name: &str,
    origin: PackOrigin,
    files: Vec<(&str, String)>,
) -> bedrock_addon::pack::Pack {
    let mut all = vec![("manifest.json", manifest(uuid, name).into_bytes())];
    all.extend(
        files
            .into_iter()
            .map(|(path, text)| (path, text.into_bytes())),
    );
    pack(all, origin)
}

/// A biome named `test:biome` whose temperature tells the packs apart.
fn test_biome(temperature: f32) -> String {
    biome(
        "1.26.30",
        "test:biome",
        &format!(r#""minecraft:climate": {{"temperature": {temperature}}}"#),
    )
}

/// The temperature of the effective `test:biome`.
fn temperature(stack: &PackStack<Behavior>) -> Option<f32> {
    let biomes = stack.biomes();
    let biome = biomes.get(&BiomeId::from_static("test:biome"))?;
    biome
        .components
        .get::<bedrock_addon::families::biome::component::Climate>()?
        .temperature
}

/// `b12_duplicate_identifier`: two packs define `test:biome`; the first loaded (the higher
/// pack's, the stack walked top first) wins, and the other is vanilla's error.
#[test]
fn b12_duplicate_identifier() {
    let top = biome_pack(
        1,
        "top",
        PackOrigin::User,
        vec![("biomes/a.json", test_biome(1.0))],
    );
    let lower = biome_pack(
        2,
        "lower",
        PackOrigin::User,
        vec![("biomes/b.json", test_biome(2.0))],
    );
    let stack = PackStack::<Behavior>::of([top, lower], LoadOptions::default()).expect("a stack");
    assert_eq!(temperature(&stack), Some(1.0));
    let duplicates: Vec<_> = stack
        .biomes()
        .diagnostics()
        .filter(|d| d.code.id() == "biome/duplicate")
        .cloned()
        .collect();
    assert_eq!(duplicates.len(), 1, "{duplicates:#?}");
    assert_eq!(duplicates[0].severity, Severity::Error);
    assert_eq!(
        duplicates[0].code.to_string(),
        "Multiple biome documents with same identifier: test:biome"
    );
    assert_eq!(
        duplicates[0]
            .location
            .path
            .as_ref()
            .map(bedrock_addon::storage::PackPath::as_str),
        Some("biomes/b.json"),
        "reported at the dropped document"
    );
}

/// A dropped base-game document is silent: overriding a vanilla biome logs nothing.
#[test]
fn a_dropped_base_game_document_is_silent() {
    let top = biome_pack(
        1,
        "top",
        PackOrigin::User,
        vec![("biomes/a.json", test_biome(1.0))],
    );
    let vanilla = biome_pack(
        2,
        "vanilla",
        PackOrigin::Package,
        vec![("biomes/b.json", test_biome(2.0))],
    );
    let stack = PackStack::<Behavior>::of([top, vanilla], LoadOptions::default()).expect("a stack");
    assert_eq!(temperature(&stack), Some(1.0));
    assert!(
        stack
            .biomes()
            .diagnostics()
            .all(|d| d.severity < Severity::Warning),
        "{:#?}",
        stack.biomes().diagnostics().collect::<Vec<_>>()
    );
}

/// A failing document does not claim its identifier: the next pack's loads (
/// "a file that fails anywhere … is skipped").
#[test]
fn a_failing_document_does_not_claim_its_identifier() {
    let broken = biome(
        "1.26.30",
        "test:biome",
        r#""minecraft:climate": {"snow_accumulation": [1.0, 0.0]}"#,
    );
    let top = biome_pack(1, "top", PackOrigin::User, vec![("biomes/a.json", broken)]);
    let lower = biome_pack(
        2,
        "lower",
        PackOrigin::User,
        vec![("biomes/b.json", test_biome(2.0))],
    );
    let stack = PackStack::<Behavior>::of([top, lower], LoadOptions::default()).expect("a stack");
    assert_eq!(temperature(&stack), Some(2.0));
    assert_eq!(stack.biomes().rejected().len(), 1);
}

/// Beta documents from a base-game pack go to the beta registry.
#[test]
fn beta_documents_go_to_the_beta_registry() {
    let text = document(
        "beta",
        r#"{"description": {"identifier": "test:beta"}, "components": {}}"#,
    );
    let vanilla = biome_pack(
        1,
        "vanilla",
        PackOrigin::Package,
        vec![("biomes/beta.json", text)],
    );
    let stack = PackStack::<Behavior>::of([vanilla], LoadOptions::default()).expect("a stack");
    let biomes = stack.biomes();
    assert!(biomes.is_empty());
    let beta = biomes.beta().expect("a beta registry");
    assert!(beta.get(&BiomeId::from_static("test:beta")).is_some());
}

/// `Placed::default_path` is `biomes/<namespace>/<name>.json`.
#[test]
fn the_default_path_is_below_biomes() {
    let biome = Biome::new(BiomeId::from_static("demo:glade"));
    assert_eq!(biome.default_path().as_str(), "biomes/demo/glade.json");
}

/// A biome named `identifier`.
fn named(identifier: &str) -> String {
    biome("1.26.30", identifier, "")
}

// The game's walk over `biomes` takes a recursion flag set to false;
// the flag is overridden for a pack: a directory pack outside the app package is
// walked with its sub-folders, a pack shipped in the app package flat.
#[test]
fn a_sub_folder_of_biomes_is_read_in_a_user_pack_and_not_in_a_base_game_pack() {
    let files = || {
        vec![
            ("biomes/flat.json", named("test:flat")),
            ("biomes/sub/nested.json", named("test:nested")),
            ("biomes/sub/deeper/far.json", named("test:far")),
        ]
    };
    let ids = |stack: &PackStack<Behavior>| -> Vec<String> {
        let mut ids: Vec<String> = stack
            .biomes()
            .iter()
            .map(|(id, _)| id.as_str().to_owned())
            .collect();
        ids.sort();
        ids
    };
    let user = biome_pack(1, "user", PackOrigin::User, files());
    let stack = PackStack::<Behavior>::of([user], LoadOptions::default()).expect("a stack");
    assert_eq!(ids(&stack), ["test:far", "test:flat", "test:nested"]);
    let shipped = biome_pack(1, "vanilla", PackOrigin::Package, files());
    let stack = PackStack::<Behavior>::of([shipped], LoadOptions::default()).expect("a stack");
    assert_eq!(ids(&stack), ["test:flat"]);
    assert!(
        stack.biomes().rejected().is_empty() && stack.biomes().diagnostics().count() == 0,
        "the nested files are not read at all"
    );
}

/// A `minecraft:` biome is written to `biomes/<name>.json`, which every pack's walk reads; a
/// base-game pack reads no other place.
#[test]
fn the_default_path_of_a_minecraft_biome_is_read_back_by_a_base_game_pack() {
    let built = Biome::new(BiomeId::from_static("minecraft:glade"));
    let path = built.default_path();
    assert_eq!(path.as_str(), "biomes/glade.json");
    for origin in [PackOrigin::Package, PackOrigin::User] {
        let pack = biome_pack(
            1,
            "pack",
            origin,
            vec![(path.as_str(), built.to_json().unwrap())],
        );
        let stack = PackStack::<Behavior>::of([pack], LoadOptions::default()).expect("a stack");
        assert!(
            stack
                .biomes()
                .get(&BiomeId::from_static("minecraft:glade"))
                .is_some(),
            "{:#?}",
            stack.biomes().diagnostics().collect::<Vec<_>>()
        );
    }
    // Another namespace keeps its folder: a user pack reads it back.
    let built = Biome::new(BiomeId::from_static("demo:glade"));
    let path = built.default_path();
    assert_eq!(path.as_str(), "biomes/demo/glade.json");
    let pack = biome_pack(
        1,
        "pack",
        PackOrigin::User,
        vec![(path.as_str(), built.to_json().unwrap())],
    );
    let stack = PackStack::<Behavior>::of([pack], LoadOptions::default()).expect("a stack");
    assert!(
        stack
            .biomes()
            .get(&BiomeId::from_static("demo:glade"))
            .is_some()
    );
}
