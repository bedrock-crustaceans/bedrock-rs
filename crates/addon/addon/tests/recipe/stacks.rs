//! The pack helper of the recipe tests: a behaviour pack opened from memory.

use bedrock_addon::pack::{OpenOptions, Pack, PackOrigin};
use bedrock_addon::storage::PackPath;
use bytes::Bytes;
use uuid::Uuid;

/// A behaviour pack `n` with `min_engine_version` `mev`, opened from memory with `origin`, holding
/// `files` (pack-relative paths).
pub fn pack(n: u128, mev: [u16; 3], origin: PackOrigin, files: Vec<(String, Vec<u8>)>) -> Pack {
    let uuid = Uuid::from_u128(n);
    let module = Uuid::from_u128(n ^ (0xabcd_u128 << 64));
    let [a, b, c] = mev;
    let manifest = format!(
        r#"{{"format_version": 2, "header": {{"name": "bp {n}", "description": "", "uuid": "{uuid}", "version": [1, 0, 0], "min_engine_version": [{a}, {b}, {c}]}}, "modules": [{{"type": "data", "uuid": "{module}", "version": [1, 0, 0]}}]}}"#
    );
    let mut all: Vec<(PackPath, Bytes)> = vec![(
        PackPath::from_static("manifest.json"),
        Bytes::from(manifest),
    )];
    all.extend(
        files
            .into_iter()
            .map(|(path, bytes)| (PackPath::new(&path).unwrap(), Bytes::from(bytes))),
    );
    let options = OpenOptions {
        origin,
        ..OpenOptions::default()
    };
    Pack::from_files(all, &options).expect("the pack opens")
}
