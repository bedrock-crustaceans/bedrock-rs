//! Fixtures for the tests that need both the `.lang` files and the language list: the files
//! under `tests/fixtures/text/text/`. The packs and stacks built from them are in
//! `bedrock_addon`'s `tests/lang_languages/stacks.rs`.

use std::path::PathBuf;

/// The fixture directory of the family.
pub fn fixtures() -> PathBuf {
    bedrock_addon_test_support::fixture::root(env!("CARGO_MANIFEST_DIR")).join("text/text")
}

/// The bytes of fixture `name`.
pub fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}
