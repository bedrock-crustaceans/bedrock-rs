//! Every finding of a POI document read on its own, loaded or rejected, is in the document's
//! pack and file.

use bedrock_addon::diagnostic::Diagnostic;
use bedrock_addon::families::poi::PoiType;
use bedrock_addon::pack::PackRef;
use bedrock_addon::storage::PackPath;

use crate::common::{block, load, loads, rejected};

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
            PackPath::new("poi/document.json").ok(),
            "{finding:?}"
        );
        count += 1;
    }
    assert!(count > 0, "the input gives findings");
}

#[test]
fn poi_findings_carry_the_file_and_pack() {
    let no_blocks = loads::<PoiType>(&block(
        r#"{"description": {"identifier": "test:empty"}, "blocks": []}"#,
    ));
    assert_in_the_file(no_blocks.diagnostics().iter());
    assert_in_the_file(rejected::<PoiType>("{").iter());
    assert!(load::<PoiType>("{").is_err());
}
