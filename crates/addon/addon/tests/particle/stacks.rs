//! The in-memory resource packs of the particle stack tests.

use bedrock_addon::pack::{OpenOptions, Pack, PackOrigin};
use bedrock_addon::storage::PackPath;
use uuid::Uuid;

/// A format-2 resource-pack manifest.
pub fn manifest(uuid: u128, name: &str) -> String {
    format!(
        r#"{{"format_version": 2, "header": {{"name": "{name}", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 21, 100]}}, "modules": [{{"type": "resources", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
        Uuid::from_u128(uuid),
        Uuid::from_u128(uuid + 1000),
    )
}

/// A resource pack named `name` with `files`.
pub fn pack(uuid: u128, name: &str, files: Vec<(String, String)>) -> Pack {
    let mut all = vec![(
        "manifest.json".to_owned(),
        manifest(uuid, name).into_bytes(),
    )];
    all.extend(
        files
            .into_iter()
            .map(|(path, text)| (path, text.into_bytes())),
    );
    let files = all
        .into_iter()
        .map(|(path, bytes)| (PackPath::new(&path).expect("a pack path"), bytes));
    let options = OpenOptions {
        origin: PackOrigin::User,
        ..OpenOptions::default()
    };
    Pack::from_files(files, &options).expect("the pack opens")
}
