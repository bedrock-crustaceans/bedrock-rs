//! The storage tests that go through the pack layer, the loader or a family: the stack's MCB
//! text, the document source it marks, and the family readers on decoded bodies. The
//! container tests themselves are beside the code in `bedrock_addon_storage`, whose fixtures
//! these read too (`crates/addon/storage/tests/fixtures/`).

mod mcb;

use std::path::PathBuf;

/// `crates/addon/storage/tests/fixtures/<relative>`.
pub(super) fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../storage/tests/fixtures")
        .join(relative)
}

/// The bytes of `crates/addon/storage/tests/fixtures/<relative>`.
pub(super) fn fixture(relative: &str) -> Vec<u8> {
    let path = fixture_path(relative);
    std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}
