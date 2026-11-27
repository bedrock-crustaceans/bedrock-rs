//! Every finding of a `blocks.json` read on its own, loaded or rejected, is in the file's pack
//! and file.

use bedrock_addon_core::diagnostic::Diagnostic;
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::pack_identity::PackRef;
use bedrock_addon_family_blocks_json::BlocksJson;

fn parse(text: &str) -> Result<Document<BlocksJson>, Rejected> {
    BlocksJson::parse(
        &PackPath::from_static("blocks.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
}

#[track_caller]
fn assert_in_the_file<'a>(findings: impl IntoIterator<Item = &'a Diagnostic>) {
    let mut count = 0;
    for finding in findings {
        assert_eq!(
            finding.location.pack,
            Some(PackRef::app_data()),
            "{finding:?}"
        );
        assert_eq!(
            finding.location.path,
            Some(PackPath::from_static("blocks.json")),
            "{finding:?}"
        );
        count += 1;
    }
    assert!(count > 0, "the input gives findings");
}

#[test]
fn blocks_json_findings_carry_the_file_and_pack() {
    let loaded = parse(r#"{"stone": {"textures": 5}}"#).expect("it loads");
    assert_in_the_file(loaded.diagnostics().iter());
    let rejected = parse("[]").expect_err("no object");
    assert_in_the_file(rejected.diagnostics.iter());
}
