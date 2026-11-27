//! Fixture loading for the `languages` tests: the files under `tests/fixtures/text/text/`.

use std::path::PathBuf;

/// The fixture directory of the family.
pub fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../addon/tests/fixtures/text/text")
}

/// The bytes of fixture `name`.
pub fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}
