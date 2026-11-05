//! Every finding of a file read alone carries the file's pack and path, at a member or at a
//! text that is no JSON.

use bedrock_addon::diagnostic::{Diagnostic, JsonPath};
use bedrock_addon::document::Content;
use bedrock_addon::families::client_entity::ClientEntity;
use bedrock_addon::load::LoadContext;
use bedrock_addon::pack::PackRef;
use bedrock_addon::storage::PackPath;

const PATH: &str = "entity/a.json";

/// Whether `text` loaded, and its findings.
fn read(text: &str) -> (bool, Vec<Diagnostic>) {
    let path = PackPath::from_static(PATH);
    match ClientEntity::parse(&path, text.as_bytes(), &LoadContext::detached()) {
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
fn client_entity_unknown_member_findings_carry_the_file_and_pack() {
    // Loaded or refused, as the family grades an unknown member.
    let (_, findings) = read(
        r#"{"format_version": "1.10.0", "minecraft:client_entity": {"description": {"identifier": "demo:a", "extra": 1}}}"#,
    );
    assert_located(&findings);
}

#[test]
fn client_entity_rejection_findings_carry_the_file_and_pack() {
    let (loaded, findings) = read("{");
    assert!(!loaded);
    assert_located(&findings);
}

/// Where the document's findings place a member the 1.8.0 step moved: an animation made of
/// the later of two controllers named `move`.
#[test]
fn a_moved_member_is_placed_as_the_file_has_it() {
    let text = r#"{"format_version": "1.8.0", "minecraft:client_entity": {"description": {"identifier": "demo:ghost", "animations": {"controller__move": "animation.x.own"}, "animation_controllers": [{"move": "controller.animation.x.a"}, {"move": "controller.animation.x.b"}]}}}"#;
    let path = PackPath::from_static(PATH);
    let document = ClientEntity::parse(&path, text.as_bytes(), &LoadContext::detached())
        .expect("the file loads");
    let held = JsonPath::of_keys([
        "minecraft:client_entity",
        "description",
        "animations",
        "controller__move",
    ]);
    // Today the loader rebuilds the list when it locates it, so the places are gone and the
    // held path comes back unchanged.
    assert_eq!(
        document.diagnostics().place_of(&held).to_string(),
        "minecraft:client_entity/description/animations/controller__move"
    );
}
