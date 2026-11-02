//! The one version the format knows, the wrapper's.

use bedrock_addon::families::material::{MaterialFileForm, MaterialKey};

use crate::common::{file, found};

/// Floor and ceiling are both 1.0.0: the one value the client compares the wrapper against.
/// One step either side of it, in each of the three numbers, is no wrapper.
#[test]
fn the_wrapper_version_has_1_0_0_as_its_floor_and_its_ceiling() {
    let wrapped = |version: &str| {
        let text = format!(r#"{{"materials": {{"version": "{version}", "a": {{}}}}}}"#);
        let document = file(&text);
        let unwrapped = matches!(document.form, MaterialFileForm::Wrapped { .. });
        // Unwrapped and silent, or read flat with the one warning and no material.
        assert_eq!(unwrapped, found(&text).is_empty(), "{version}");
        assert_eq!(
            unwrapped,
            document.materials.contains_key(&MaterialKey::new("a")),
            "{version}"
        );
        unwrapped
    };
    assert!(wrapped("1.0.0"));
    for version in [
        "0.9.9", "0.0.0", "1.0.1", "1.1.0", "2.0.0", "0.0.1", "0.1.0",
    ] {
        assert!(!wrapped(version), "{version}");
    }
}
