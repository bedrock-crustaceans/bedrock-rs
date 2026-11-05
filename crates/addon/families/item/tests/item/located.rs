//! Every finding of a loaded or a rejected item is in the item's pack and file, and the places
//! the upgrade records stay for the link pass.

use bedrock_addon_core::diagnostic::{Diagnostic, JsonPath};
use bedrock_addon_core::pack_identity::{PackPath, PackRef};

use super::common::{allowlisted, item, loads, rejects};

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
            Some(PackPath::from_static("items/test.json")),
            "{finding:?}"
        );
        count += 1;
    }
    assert!(count > 0, "the input gives findings");
}

/// A 1.16.100 item whose first repair entry is no entry: the step drops it.
fn first_repair_entry_dropped() -> String {
    item(
        "1.16.100",
        "minecraft:x",
        r#""minecraft:repairable": {"repair_items": [5, {"items": ["minecraft:stick"], "repair_amount": 5}]}"#,
    )
}

#[test]
fn item_findings_carry_the_file_and_pack() {
    let kept = item(
        "1.16.100",
        "minecraft:x",
        r#""minecraft:food": {"saturation_modifier": "weird"}"#,
    );
    assert_in_the_file(loads(&kept, &allowlisted()).diagnostics().iter());
    assert_in_the_file(rejects(&first_repair_entry_dropped(), &allowlisted()).iter());
    assert_in_the_file(rejects("{", &allowlisted()).iter());
}

#[test]
fn item_places_stay_after_the_findings_are_located() {
    let diagnostics = rejects(&first_repair_entry_dropped(), &allowlisted());
    let list = JsonPath::new().with(&[
        "minecraft:item",
        "components",
        "minecraft:repairable",
        "repair_items",
    ]);
    // The entry the item holds first is the file's second.
    assert_eq!(diagnostics.place_of(&list.index(0)), list.index(1));
}
