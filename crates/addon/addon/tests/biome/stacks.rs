//! The packs of the biome stack tests: a behaviour-pack manifest and a pack opened from its
//! files.

#![allow(dead_code, reason = "each test module uses part of the helpers")]

use bedrock_addon::pack::{OpenOptions, Pack, PackOrigin};
use bedrock_addon::storage::PackPath;
use uuid::Uuid;

/// A format-2 behaviour-pack manifest.
pub fn manifest(uuid: u128, name: &str) -> String {
    format!(
        r#"{{"format_version": 2, "header": {{"name": "{name}", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 26, 30]}}, "modules": [{{"type": "data", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
        Uuid::from_u128(uuid),
        Uuid::from_u128(uuid + 1000),
    )
}

/// A pack from its files, of `origin`.
pub fn pack(files: Vec<(&str, Vec<u8>)>, origin: PackOrigin) -> Pack {
    let files = files
        .into_iter()
        .map(|(path, bytes)| (PackPath::new(path).expect("a pack path"), bytes));
    let options = OpenOptions {
        origin,
        ..OpenOptions::default()
    };
    Pack::from_files(files, &options).expect("the pack opens")
}
