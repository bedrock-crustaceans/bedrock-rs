//! Every finding of a material file or list read on its own, loaded or rejected, is in the
//! file's pack and file.

use bedrock_addon_core::diagnostic::Diagnostic;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::pack_identity::PackRef;

use crate::common::{parse, parse_list, wrapped};

#[track_caller]
fn assert_in_the_file<'a>(path: &str, findings: impl IntoIterator<Item = &'a Diagnostic>) {
    let mut count = 0;
    for finding in findings {
        assert_eq!(
            finding.location.pack,
            Some(PackRef::app_data()),
            "{finding:?}"
        );
        assert_eq!(
            finding.location.path,
            PackPath::new(path).ok(),
            "{finding:?}"
        );
        count += 1;
    }
    assert!(count > 0, "the input gives findings");
}

#[test]
fn material_findings_carry_the_file_and_pack() {
    let file = "materials/test.material";
    let loaded = parse(&wrapped(r#""a": {"varient": []}"#)).expect("it loads");
    assert_in_the_file(file, loaded.diagnostics().iter());
    let rejected = parse("{").expect_err("no JSON");
    assert_in_the_file(file, rejected.diagnostics.iter());
    let list = "materials/common.json";
    let rejected = parse_list(list, "{").expect_err("no JSON");
    assert_in_the_file(list, rejected.diagnostics.iter());
}
