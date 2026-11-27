//! Every finding of a file read alone carries the file's pack and path, at a member or at a
//! text that is no JSON.

use bedrock_addon_core::diagnostic::Diagnostic;
use bedrock_addon_core::document::Content;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::pack_identity::PackRef;
use bedrock_addon_family_recipe::Recipe;

const PATH: &str = "recipes/a.json";

/// Whether `text` loaded, and its findings.
fn read(text: &str) -> (bool, Vec<Diagnostic>) {
    let path = PackPath::from_static(PATH);
    match Recipe::parse(&path, text.as_bytes(), &LoadContext::detached()) {
        Ok(document) => (true, document.diagnostics().iter().cloned().collect()),
        Err(rejected) => (false, rejected.diagnostics.iter().cloned().collect()),
    }
}

fn assert_located(findings: &[Diagnostic]) {
    assert!(!findings.is_empty(), "the text gives findings");
    for finding in findings {
        assert_eq!(
            finding.location.path,
            Some(PackPath::from_static(PATH)),
            "{finding}"
        );
        assert_eq!(
            finding.location.pack,
            Some(PackRef::app_data()),
            "{finding}"
        );
    }
}

#[test]
fn recipe_unknown_member_findings_carry_the_file_and_pack() {
    // Loaded or refused, as the family grades an unknown member.
    let (_, findings) = read(
        r#"{"format_version": "1.20.10", "minecraft:recipe_shapeless": {"description": {"identifier": "demo:a"}, "tags": ["crafting_table"], "ingredients": [{"item": "minecraft:stick"}], "result": {"item": "minecraft:stick"}, "extra": 1}}"#,
    );
    assert_located(&findings);
}

#[test]
fn recipe_rejection_findings_carry_the_file_and_pack() {
    let (loaded, findings) = read("{");
    assert!(!loaded);
    assert_located(&findings);
}
