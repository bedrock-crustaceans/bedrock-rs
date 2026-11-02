#![allow(clippy::pedantic, missing_docs)]
//! `#[derive(Components)]` rejects malformed input with an error at the offending tokens. Each
//! case is a file under `tests/ui/` with the expected compiler output beside it
//! (`TRYBUILD=overwrite cargo test -p bedrock_addon_derive --test compile_fail` regenerates it).

#[test]
fn malformed_component_enums_do_not_compile() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/*.rs");
}
