//! Fixtures and pack builders for the tests that need both the `.lang` files and the language
//! list: the files under `tests/fixtures/text/text/` and packs and stacks built from them.

use std::path::PathBuf;

/// The `tests/fixtures` folder of `bedrock_addon`: this file is compiled by the family crate and
/// included by `bedrock_addon`'s own test root, which owns the folder.
fn fixtures_root() -> std::path::PathBuf {
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let own = manifest.join("tests/fixtures");
    if own.is_dir() {
        own
    } else {
        manifest.join("../../addon/tests/fixtures")
    }
}

/// The fixture directory of the family.
pub fn fixtures() -> PathBuf {
    fixtures_root().join("text/text")
}

/// The bytes of fixture `name`.
pub fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}
