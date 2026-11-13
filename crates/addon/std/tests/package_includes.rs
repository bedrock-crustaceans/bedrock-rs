//! The package-include rail of `bedrock_addon_std`: which of its test files include a file
//! outside the package (`bedrock_addon_test_support::package` says what is seen and why it
//! matters).

use bedrock_addon_test_support::package::{includes_outside_the_package, rust_files};

/// The test files of this package that include a file outside it, and are known to, as
/// `"<file>: <path>"`: the stack snapshot of `bedrock_addon`'s tests (`collections.rs` and
/// `unprinted.rs` of its `tests/common/`), which the corpus strategy targets compare two loads
/// by. It names `bedrock_addon` alone and is kept as one copy, there. Nothing else is included
/// from outside the package.
const KNOWN_OUTSIDE: &[&str] = &[
    "tests/common/strategies.rs: ../../../addon/tests/common/unprinted.rs",
    "tests/full_strategies.rs: ../../addon/tests/common/collections.rs",
    "tests/full_strategies_client_install_a.rs: ../../addon/tests/common/collections.rs",
    "tests/full_strategies_client_install_b.rs: ../../addon/tests/common/collections.rs",
];

/// This package builds from its own files: no test file includes one of another crate but the
/// ones [`KNOWN_OUTSIDE`] lists.
#[test]
fn no_test_file_of_this_package_includes_a_file_outside_it() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(root, &root.join("tests"), &mut files);
    let mut found: Vec<String> = files
        .iter()
        .flat_map(|(file, text)| includes_outside_the_package(file, text))
        .collect();
    found.sort();
    let mut known: Vec<&str> = KNOWN_OUTSIDE.to_vec();
    known.sort_unstable();
    assert_eq!(
        found,
        known,
        "includes that leave {}",
        env!("CARGO_PKG_NAME")
    );
    // The file this test is in is one of the package's.
    assert!(
        files
            .iter()
            .any(|(file, _)| file == "tests/package_includes.rs"),
        "the walk finds this file"
    );
}
