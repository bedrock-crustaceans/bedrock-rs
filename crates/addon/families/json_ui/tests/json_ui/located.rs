//! Every finding of a UI file, the `ui_defs` list or the global variables read on its own,
//! loaded or rejected, is in the file's pack and file.

use bedrock_addon_core::diagnostic::Diagnostic;
use bedrock_addon_core::document::Content;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::pack_identity::PackRef;
use bedrock_addon_family_json_ui::{GlobalVariables, UiDefsList, UiFile};

#[track_caller]
fn assert_in_the_file<'a>(path: &PackPath, findings: impl IntoIterator<Item = &'a Diagnostic>) {
    let mut count = 0;
    for finding in findings {
        assert_eq!(
            finding.location.pack,
            Some(PackRef::app_data()),
            "{finding:?}"
        );
        assert_eq!(finding.location.path.as_ref(), Some(path), "{finding:?}");
        count += 1;
    }
    assert!(count > 0, "the input gives findings");
}

#[test]
fn json_ui_findings_carry_the_file_and_pack() {
    let cx = LoadContext::detached();
    let file = PackPath::from_static("ui/x.json");
    let loaded = UiFile::parse(&file, br#"{"namespace": 5}"#, &cx).expect("it loads");
    assert_in_the_file(&file, loaded.diagnostics().iter());
    let rejected = UiFile::parse(&file, b"[]", &cx).expect_err("no object");
    assert_in_the_file(&file, rejected.diagnostics.iter());
    let list = PackPath::from_static("ui/_ui_defs.json");
    let loaded = UiDefsList::parse(&list, br#"{"ui_defs": 5}"#, &cx).expect("it loads");
    assert_in_the_file(&list, loaded.diagnostics().iter());
    let globals = PackPath::from_static("ui/_global_variables.json");
    let rejected = GlobalVariables::parse(&globals, b"{", &cx).expect_err("no JSON");
    assert_in_the_file(&globals, rejected.diagnostics.iter());
}
