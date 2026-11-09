//! What the `biomes_client.json` tests share: the pack context, a client biome document's text,
//! one resource pack of in-memory files and the stacks of a world.

#![allow(dead_code, reason = "each test file uses part of the helpers")]
#![allow(
    clippy::disallowed_methods,
    reason = "calls facet-json directly: parses a fixture or written text into a json Value to compare it; not yet moved to the door"
)]

use std::path::PathBuf;

use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::pack::{OpenOptions, Pack, PackOrigin, PackType};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::MinEngineVersion;
use uuid::Uuid;

/// The file every single-file test loads from.
pub const PATH: &str = "biomes/demo.client_biome.json";

/// A resource-pack context of a pack other than a base-game pack: the engine version, no
/// toggles, no world version (every world-gated entry applies).
pub fn context() -> LoadContext {
    LoadContext::for_pack(
        &LoadOptions::default(),
        Uuid::from_u128(7),
        PackType::Resource,
        Some(MinEngineVersion::new(1, 26, 30)),
        false,
    )
}

/// The path every single-file test loads from.
pub fn path() -> PackPath {
    PackPath::from_static(PATH)
}

pub fn json(text: &str) -> facet_value::Value {
    facet_json::from_str_jsonc(text).unwrap_or_else(|e| panic!("{e}: {text}"))
}

/// The fixture directory `tests/fixtures/biome/biomes_client/<rest>`.
pub fn fixture_dir(rest: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/biome/biomes_client")
        .join(rest)
}

/// A fixture file's text.
pub fn fixture(rest: &str) -> String {
    let path = fixture_dir(rest);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// A format-2 resource-pack manifest.
pub fn manifest(uuid: u128, name: &str) -> String {
    format!(
        r#"{{"format_version": 2, "header": {{"name": "{name}", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 26, 30]}}, "modules": [{{"type": "resources", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
        Uuid::from_u128(uuid),
        Uuid::from_u128(uuid + 1000),
    )
}

/// A resource pack of `origin` named `name` with `files`.
pub fn pack(uuid: u128, name: &str, origin: PackOrigin, files: Vec<(&str, String)>) -> Pack {
    let mut all = vec![(
        "manifest.json".to_owned(),
        manifest(uuid, name).into_bytes(),
    )];
    all.extend(
        files
            .into_iter()
            .map(|(path, text)| (path.to_owned(), text.into_bytes())),
    );
    let files = all
        .into_iter()
        .map(|(path, bytes)| (PackPath::new(&path).expect("a pack path"), bytes));
    let options = OpenOptions {
        origin,
        ..OpenOptions::default()
    };
    Pack::from_files(files, &options).expect("the pack opens")
}

/// The stacks of a world that lists `packs` as its resource packs, top first, in a world with
/// `options`: what the linking pass runs over.
pub fn world_stacks(
    packs: Vec<Pack>,
    options: bedrock_addon::load::LoadOptions,
) -> bedrock_addon::pack::StackSet {
    use bedrock_addon::pack::{PackRepository, PackSelection, WorldPacks};
    let mut repository = PackRepository::new();
    let mut resource = Vec::new();
    for pack in packs {
        resource.push(PackSelection {
            pack_id: pack.id().uuid,
            version: pack.id().version.clone(),
            subpack: None,
        });
        repository.add_pack(pack, PackOrigin::User);
    }
    let world = WorldPacks {
        behavior: Vec::new(),
        resource,
    };
    repository.stacks(&world, options)
}

/// A document of `version` with `payload` as `minecraft:client_biome`.
pub fn document(version: &str, payload: &str) -> String {
    format!(r#"{{"format_version": "{version}", "minecraft:client_biome": {payload}}}"#)
}

/// A document of `version` named `identifier` with `components` (a JSON object's members).
pub fn biome(version: &str, identifier: &str, components: &str) -> String {
    document(
        version,
        &format!(
            r#"{{"description": {{"identifier": "{identifier}"}}, "components": {{{components}}}}}"#
        ),
    )
}
