//! Every finding of a file read alone carries the file's pack and path, at a member or at a
//! text that is no JSON.

use bedrock_addon::diagnostic::Diagnostic;
use bedrock_addon::document::Content;
use bedrock_addon::families::render_controller::RenderControllerFile;
use bedrock_addon::load::LoadContext;
use bedrock_addon::pack::PackRef;
use bedrock_addon::storage::PackPath;

const PATH: &str = "render_controllers/x.json";

/// Whether `text` loaded, and its findings.
fn read(text: &str) -> (bool, Vec<Diagnostic>) {
    let path = PackPath::from_static(PATH);
    match RenderControllerFile::parse(&path, text.as_bytes(), &LoadContext::detached()) {
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
fn render_controller_unknown_member_findings_carry_the_file_and_pack() {
    // Loaded or refused, as the family grades an unknown member.
    let (_, findings) = read(
        r#"{"format_version": "1.8.0", "render_controllers": {"controller.render.a": {"geometry": "Geometry.default", "textures": ["Texture.default"], "extra": 1}}}"#,
    );
    assert_located(&findings);
}

#[test]
fn render_controller_rejection_findings_carry_the_file_and_pack() {
    let (loaded, findings) = read("{");
    assert!(!loaded);
    assert_located(&findings);
}
