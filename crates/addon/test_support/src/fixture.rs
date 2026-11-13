//! Fixture reads. The root is the caller's: a test names its file below its own
//! `env!("CARGO_MANIFEST_DIR")`.

use std::path::Path;

/// The text of the file at `path`; panics with the path when it cannot be read.
#[track_caller]
pub fn text(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The bytes of the file at `path`; panics with the path when it cannot be read.
#[track_caller]
pub fn bytes(path: impl AsRef<Path>) -> Vec<u8> {
    let path = path.as_ref();
    std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}
