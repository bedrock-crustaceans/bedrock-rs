//! The two slices of 1.26.60: `v1_21_60` `[1.21.60, 1.26.50)`
//! registered with 0xfbf8 (an unknown member is a warning, the file loads) and `v1_26_50`
//! `[1.26.50, engine]` with 0xfffc (an unknown member is an error and the file is skipped,
//! C2), joined by a plain copy step.

use bedrock_addon_core::diagnostic::{Diagnostics, Severity};
use bedrock_addon_core::document::Content;
use bedrock_addon_core::version::{ENGINE_VERSION, FormatVersion};

use super::common::*;

const ROOT: &str = "minecraft:crafting_items_catalog";

/// A document at `version` whose payload has unknown members at every level.
fn with_unknown_members(version: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "{ROOT}": {{"a": 1, "categories": [{{"b": 1, "category_name": "items", "groups": [{{"c": 1,
            "group_identifier": {{"d": 1, "name": "g:g", "icon": {{"e": 1, "name": "i:i"}}}},
            "items": [{{"f": 1, "name": "i:i"}}]}}]}}]}}}}"#
    )
}

/// The unknown-member paths of `with_unknown_members`, in document order.
fn unknown_paths() -> Vec<String> {
    [
        "a",
        "categories/0/b",
        "categories/0/groups/0/c",
        "categories/0/groups/0/group_identifier/d",
        "categories/0/groups/0/group_identifier/icon/e",
        "categories/0/groups/0/items/0/f",
    ]
    .iter()
    .map(|tail| format!("{ROOT}/{tail}"))
    .collect()
}

/// `(severity, path)` of every `UnrecognizedField` finding.
fn unrecognized(d: &Diagnostics) -> Vec<(Severity, String)> {
    d.iter()
        .filter(|f| f.code.name() == "UnrecognizedField")
        .map(|f| (f.severity, f.location.json_path.to_string()))
        .collect()
}

#[test]
fn c2_at_1_26_50_and_above_an_unknown_member_is_an_error_and_the_file_is_skipped() {
    let engine = ENGINE_VERSION.to_string();
    for version in ["1.26.50", "1.26.51", engine.as_str()] {
        let d = rejects(&with_unknown_members(version));
        let expected: Vec<(Severity, String)> = unknown_paths()
            .into_iter()
            .map(|path| (Severity::Error, path))
            .collect();
        assert_eq!(unrecognized(&d), expected, "{version}");
        assert_eq!(
            errors(&d).len(),
            expected.len(),
            "{version}: only the unknown members: {d:#?}"
        );
    }
}

#[test]
fn c1_below_1_26_50_an_unknown_member_is_a_warning_and_the_file_loads() {
    for version in ["1.21.60", "1.26.40", "1.26.49", "1.26.50-beta.1"] {
        let document = loads(&with_unknown_members(version));
        let expected: Vec<(Severity, String)> = unknown_paths()
            .into_iter()
            .map(|path| (Severity::Warning, path))
            .collect();
        assert_eq!(unrecognized(document.diagnostics()), expected, "{version}");
    }
}

#[test]
fn the_step_to_1_26_50_is_a_plain_copy() {
    let text = fixture("cat01_all_categories.json");
    let at_1_21_60 = loads(&text);
    let at_1_26_50 = loads(&text.replace("1.21.60", "1.26.50"));
    assert_eq!(at_1_21_60.value(), at_1_26_50.value());
    assert_eq!(
        at_1_21_60.version().parsed_as,
        FormatVersion::new(1, 21, 60)
    );
    assert_eq!(
        at_1_26_50.version().parsed_as,
        FormatVersion::new(1, 26, 50)
    );
    assert!(at_1_26_50.diagnostics().is_empty());
}

#[test]
fn a_catalogue_without_unknown_members_is_written_at_1_26_50() {
    let written = loads(&fixture("cat01_all_categories.json"))
        .to_json()
        .unwrap();
    assert!(
        written.contains(r#""format_version": "1.26.50""#),
        "{written}"
    );
    let again = loads(&written);
    assert_eq!(again.version().parsed_as, FormatVersion::new(1, 26, 50));
}

#[test]
fn parse_as_written_then_upgrade_is_the_load() {
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::pack_identity::PackPath;
    use bedrock_addon_family_crafting_catalog::CraftingCatalog;
    use bedrock_addon_family_crafting_catalog::history::CatalogAny;
    let path = PackPath::from_static(PATH);
    let cx = LoadContext::detached();
    for (version, older) in [("1.21.60", true), ("1.26.50", false)] {
        let text = fixture("cat01_all_categories.json").replace("1.21.60", version);
        let written = CraftingCatalog::parse_as_written(&path, text.as_bytes(), &cx).unwrap();
        assert_eq!(
            matches!(written.value(), CatalogAny::V1_21_60(_)),
            older,
            "{version}"
        );
        let upgraded = CatalogAny::upgrade(written, &cx).unwrap();
        assert_eq!(upgraded.value(), loads(&text).value(), "{version}");
    }
}

/// The 1.21.60 slice tells an unknown member
/// (the warning stays) and never keeps it, as the game's reader under 0xfbf8 does;
/// the value holds no `extra`, the writer stamps
/// 1.26.50 and the written file reads back to the same value without a finding.
#[test]
fn an_unknown_member_below_1_26_50_is_told_and_not_kept() {
    let document = loads(&with_unknown_members("1.21.60"));
    assert_eq!(unrecognized(document.diagnostics()).len(), 6);
    let catalog = document.value();
    assert!(catalog.extra.is_empty());
    let group = &catalog.categories[0].groups[0];
    assert!(catalog.categories[0].extra.is_empty() && group.extra.is_empty());
    let identifier = group.group_identifier.as_ref().unwrap();
    assert!(identifier.extra.is_empty());
    assert!(identifier.icon.as_ref().unwrap().extra.is_empty());
    assert!(group.items[0].extra.is_empty());
    let written = document.to_json().unwrap();
    assert!(
        written.contains(r#""format_version": "1.26.50""#),
        "{written}"
    );
    let again = loads(&written);
    assert!(again.diagnostics().is_empty(), "{:#?}", again.diagnostics());
    assert_eq!(again.value(), catalog);
}
