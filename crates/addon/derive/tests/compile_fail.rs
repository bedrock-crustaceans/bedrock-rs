//! `#[derive(Components)]` and `#[derive(Schema)]` reject malformed input with an error at the
//! offending tokens. Each case is a file under `tests/ui/` (`tests/ui/schema/` for `Schema`, the
//! forced declarations among them) with the expected compiler output beside it
//! (`TRYBUILD=overwrite cargo nextest run -p bedrock_addon_derive --test compile_fail` regenerates it).

#[test]
fn malformed_input_does_not_compile() {
    // One `TestCases` for both derives: they share one trybuild project.
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/*.rs");
    cases.compile_fail("tests/ui/schema/*.rs");
}
