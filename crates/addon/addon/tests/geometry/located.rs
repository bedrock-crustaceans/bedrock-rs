//! Every finding of a geometry file read on its own, loaded or rejected, is in the file's pack
//! and file.

use bedrock_addon::diagnostic::Diagnostic;
use bedrock_addon::pack::PackRef;
use bedrock_addon::storage::PackPath;

use crate::common::{load, loaded, rejected, with_bone};

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
            Some(PackPath::from_static("models/entity/x.json")),
            "{finding:?}"
        );
        count += 1;
    }
    assert!(count > 0, "the input gives findings");
}

#[test]
fn geometry_findings_carry_the_file_and_pack() {
    let document = loaded(load(&with_bone("1.12.0", r#", "unknown_member": 1"#)));
    assert_in_the_file(document.diagnostics().iter());
    assert_in_the_file(rejected(load("{")).iter());
}
