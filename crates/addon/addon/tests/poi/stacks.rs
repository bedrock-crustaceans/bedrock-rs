//! The pack helpers of the POI tests: packs opened from memory or from a fixture's tree, and
//! the behaviour stacks built from them.

use std::path::{Path, PathBuf};

use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackOrigin, PackStack};
use bedrock_addon::storage::PackPath;
use uuid::Uuid;

use super::common::{fixture_dir, poi_options};

/// A format-2 behaviour-pack manifest.
pub fn manifest(uuid: u128, name: &str) -> String {
    format!(
        r#"{{"format_version": 2, "header": {{"name": "{name}", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 26, 40]}}, "modules": [{{"type": "data", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
        Uuid::from_u128(uuid),
        Uuid::from_u128(uuid + 1000),
    )
}

/// Every file below `dir`, by its path relative to `base`.
fn files(dir: &Path, base: &Path, out: &mut Vec<(String, Vec<u8>)>) {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            files(&path, base, out);
        } else {
            let relative = path.strip_prefix(base).unwrap().to_string_lossy();
            out.push((relative.replace('\\', "/"), std::fs::read(&path).unwrap()));
        }
    }
}

/// A pack of `origin` from its files (a manifest is added).
pub fn pack_from(uuid: u128, origin: PackOrigin, files: Vec<(String, Vec<u8>)>) -> Pack {
    let mut all = vec![(
        PackPath::new("manifest.json").unwrap(),
        manifest(uuid, &format!("pack {uuid}")).into_bytes(),
    )];
    all.extend(
        files
            .into_iter()
            .map(|(path, bytes)| (PackPath::new(&path).expect("a pack path"), bytes)),
    );
    let options = OpenOptions {
        origin,
        ..OpenOptions::default()
    };
    Pack::from_files(all, &options).expect("the pack opens")
}

/// A user pack from `(path, text)` pairs.
pub fn pack(uuid: u128, files: &[(&str, &str)]) -> Pack {
    pack_from(
        uuid,
        PackOrigin::User,
        files
            .iter()
            .map(|(path, text)| ((*path).to_owned(), text.as_bytes().to_vec()))
            .collect(),
    )
}

/// The packs of a fixture, top first, each of `origin`.
pub fn fixture_packs_from(name: &str, origin: PackOrigin) -> Vec<Pack> {
    let dir = fixture_dir(name);
    let stack = dir.join("stack");
    if stack.is_dir() {
        let mut members: Vec<PathBuf> = std::fs::read_dir(&stack)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.is_dir())
            .collect();
        members.sort();
        members
            .iter()
            .enumerate()
            .map(|(i, member)| {
                let mut found = Vec::new();
                files(member, member, &mut found);
                pack_from(10 * (i as u128 + 1), origin, found)
            })
            .collect()
    } else {
        let mut found = Vec::new();
        files(&dir, &dir, &mut found);
        vec![pack_from(10, origin, found)]
    }
}

/// The user packs of a fixture, top first.
pub fn fixture_packs(name: &str) -> Vec<Pack> {
    fixture_packs_from(name, PackOrigin::User)
}

/// A behaviour stack of `packs`, top first, loaded with `options`.
pub fn stack_with(packs: Vec<Pack>, options: LoadOptions) -> PackStack<Behavior> {
    PackStack::<Behavior>::of(packs, options).expect("a behaviour stack")
}

/// A behaviour stack of `packs`, top first, with the `poi` experiment on.
pub fn stack(packs: Vec<Pack>) -> PackStack<Behavior> {
    stack_with(packs, poi_options())
}

/// The stack of a fixture, with the `poi` experiment on.
pub fn fixture_stack(name: &str) -> PackStack<Behavior> {
    stack(fixture_packs(name))
}
