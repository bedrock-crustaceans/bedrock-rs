//! The pack helpers of the client-biome tests: resource packs opened from memory and the
//! stacks of a world that lists them.

#![allow(dead_code, reason = "each test module uses part of the helpers")]

use bedrock_addon::pack::{OpenOptions, Pack, PackOrigin};
use bedrock_addon::storage::PackPath;
use uuid::Uuid;

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
