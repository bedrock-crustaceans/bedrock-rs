//! The single-file rows (`cat01` … `cat14`).

use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_family_crafting_catalog::CategoryName;

use super::common::*;

const FORMAT_VERSION_TEXT: &str = "The file cannot be processed without a valid format_version";

#[test]
fn cat01_all_four_categories_load_and_both_item_forms_read_alike() {
    let document = loads(&fixture("cat01_all_categories.json"));
    assert_eq!(document.value(), &cat01());
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(
        document.version().written,
        Some(FormatVersion::new(1, 21, 60))
    );
    // The expected file writes every item as a string: the object form read the same value.
    let strings = loads(&fixture("cat01_all_categories.expected.json"));
    assert_eq!(strings.value(), document.value());
    // The aux value stays in the name and is read off it.
    let icon = document.categories[1].groups[0]
        .group_identifier
        .as_ref()
        .and_then(|identifier| identifier.icon.as_ref())
        .unwrap();
    assert_eq!(icon.name.as_str(), "custom:my_icon:1");
    assert_eq!(icon.name.aux(), Some(1));
    assert_eq!(document.categories[3].category_name, CategoryName::Nature);
}

#[test]
fn cat02_a_version_below_1_21_60_is_rejected_with_vanilla_s_text() {
    let found = rejects(&fixture("cat02_below_floor.json"));
    assert_eq!(
        listed(&found),
        expected_list("cat02_below_floor.expected.json")
    );
    assert!(matches!(
        one_error(&found).code,
        DiagnosticCode::UnsupportedVersion { .. }
    ));
}

#[test]
fn cat02_the_floor_is_exactly_1_21_60() {
    let text = fixture("cat01_all_categories.json");
    let below = rejects(&text.replace("1.21.60", "1.21.59"));
    assert!(matches!(
        one_error(&below).code,
        DiagnosticCode::UnsupportedVersion { .. }
    ));
    assert_eq!(one_error(&below).code.to_string(), FORMAT_VERSION_TEXT);
    loads(&text);
}

#[test]
fn cat03_an_unknown_member_is_a_warning_and_the_file_loads() {
    let document = loads(&fixture("cat03_unknown_member.json"));
    let found: Vec<_> = document.diagnostics().iter().collect();
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(matches!(
        found[0].code,
        DiagnosticCode::UnrecognizedField { .. }
    ));
    assert_eq!(found[0].severity, Severity::Warning);
    assert_eq!(
        found[0].location.json_path.to_string(),
        "minecraft:crafting_items_catalog/categories/0/groups/0/colour"
    );
    let group = &document.categories[0].groups[0];
    assert_eq!(group.items, [item("custom:item1")]);
    // Told, not kept.
    assert!(document.unknown().is_empty(), "the member is not kept");
}

#[test]
fn cat04_a_missing_format_version_is_an_error_with_vanilla_s_text() {
    let found = rejects(&fixture("cat04_missing_format_version.json"));
    assert_eq!(
        listed(&found),
        expected_list("cat04_missing_format_version.expected.json")
    );
    assert!(matches!(
        one_error(&found).code,
        DiagnosticCode::MissingFormatVersion { .. }
    ));
}

#[test]
fn cat04b_an_unreadable_format_version_is_the_missing_version_error() {
    for case in ["string_not_a_version", "third_root_member"] {
        let found = rejects(&fixture(&format!("cat04b_envelope_edges/{case}.json")));
        let error = one_error(&found);
        assert!(
            matches!(error.code, DiagnosticCode::MissingFormatVersion { .. }),
            "{case}: {found:#?}"
        );
        assert_eq!(error.code.to_string(), FORMAT_VERSION_TEXT, "{case}");
    }
    // A number is read as its text on the document input: `1.21` is 1.21.0, below
    // 1.21.60, and vanilla's one text says so as the floor error (L2).
    let found = rejects(&fixture("cat04b_envelope_edges/number_version.json"));
    let error = one_error(&found);
    assert!(
        matches!(error.code, DiagnosticCode::UnsupportedVersion { .. }),
        "{found:#?}"
    );
    assert_eq!(error.code.to_string(), FORMAT_VERSION_TEXT);
}

#[test]
fn cat04b_a_format_version_of_three_integers_loads_like_the_string() {
    let document = loads(&fixture("cat04b_envelope_edges/array_version.json"));
    assert_eq!(document.value(), &cat01());
    assert_eq!(
        document.version().written,
        Some(FormatVersion::new(1, 21, 60))
    );
    assert!(!document.diagnostics().has_errors());
}

#[test]
fn cat05_a_payload_under_another_key_is_vanilla_s_missing_object_error() {
    let found = rejects(&fixture("cat05_wrong_payload_key.json"));
    assert_eq!(
        listed(&found),
        expected_list("cat05_wrong_payload_key.expected.json")
    );
    assert!(matches!(
        one_error(&found).code,
        DiagnosticCode::MissingRequiredField { .. }
    ));
}

#[test]
fn cat06_a_payload_without_categories_is_rejected() {
    let found = rejects(&fixture("cat06_missing_categories.json"));
    let error = one_error(&found);
    assert!(matches!(
        error.code,
        DiagnosticCode::MissingRequiredField { .. }
    ));
    assert_eq!(
        error.location.json_path.to_string(),
        "minecraft:crafting_items_catalog/categories"
    );
}

#[test]
fn cat07_an_unknown_category_name_is_rejected() {
    let found = rejects(&fixture("cat07_bad_category_name.json"));
    let error = one_error(&found);
    assert!(matches!(error.code, DiagnosticCode::InvalidValue { .. }));
    assert_eq!(
        error.location.json_path.to_string(),
        "minecraft:crafting_items_catalog/categories/0/category_name"
    );
}

#[test]
fn cat08_a_group_name_without_a_namespace_is_rejected_with_vanilla_s_text() {
    let found = rejects(&fixture("cat08_group_name_no_namespace.json"));
    assert_eq!(
        listed(&found),
        expected_list("cat08_group_name_no_namespace.expected.json")
    );
    let error = one_error(&found);
    assert!(matches!(error.code, DiagnosticCode::InvalidValue { .. }));
    assert_eq!(
        error.location.json_path.to_string(),
        "minecraft:crafting_items_catalog/categories/0/groups/0/group_identifier/name"
    );
}

#[test]
fn cat09_an_item_name_without_a_namespace_is_rejected_with_vanilla_s_text() {
    let found = rejects(&fixture("cat09_item_name_no_namespace.json"));
    assert_eq!(
        listed(&found),
        expected_list("cat09_item_name_no_namespace.expected.json")
    );
    let error = one_error(&found);
    assert!(matches!(error.code, DiagnosticCode::InvalidValue { .. }));
    assert_eq!(
        error.location.json_path.to_string(),
        "minecraft:crafting_items_catalog/categories/0/groups/0/items/0"
    );
}

#[test]
fn cat09b_a_colon_with_a_character_on_each_side_is_enough() {
    for (case, name) in [("item_double_colon", "::x"), ("item_four_parts", "a:b:c:d")] {
        let document = loads(&fixture(&format!("cat09b_name_regex_edges/{case}.json")));
        assert_eq!(
            document.categories[0].groups[0].items,
            [item(name)],
            "{case}"
        );
        assert!(document.diagnostics().is_empty(), "{case}");
    }
    let document = loads(&fixture(
        "cat09b_name_regex_edges/group_name_mixed_case.json",
    ));
    let identifier = document.categories[0].groups[0].group_identifier.as_ref();
    assert_eq!(identifier.unwrap().name, "MyNs:Group");
    assert!(document.diagnostics().is_empty());
}

#[test]
fn cat09b_an_empty_side_of_the_colon_fails_the_item_regex() {
    for case in ["item_empty_namespace", "item_empty_name"] {
        let found = rejects(&fixture(&format!("cat09b_name_regex_edges/{case}.json")));
        assert_eq!(
            listed(&found),
            expected_list("cat09_item_name_no_namespace.expected.json"),
            "{case}"
        );
    }
}

#[test]
fn cat10_empty_arrays_load_without_a_finding() {
    let empty = loads(&fixture("cat10_empty_arrays/categories.json"));
    assert_eq!(empty.value(), &catalog(Vec::new()));
    assert!(empty.diagnostics().is_empty());
    let document = loads(&fixture("cat10_empty_arrays/groups_and_items.json"));
    assert_eq!(
        document.value(),
        &catalog(vec![
            category(CategoryName::Nature, Vec::new()),
            category(CategoryName::Items, vec![loose(&[])]),
        ])
    );
    assert!(document.diagnostics().is_empty());
}

#[test]
fn cat11_a_version_above_the_engine_is_rejected() {
    let found = rejects(&fixture("cat11_above_engine.json"));
    let error = one_error(&found);
    assert!(
        matches!(&error.code, DiagnosticCode::FormatVersionAboveEngine { version } if version == "1.27.0"),
        "{found:#?}"
    );
}

#[test]
fn cat11_the_ceiling_is_exactly_the_engine_version() {
    let text = fixture("cat01_all_categories.json");
    let engine = bedrock_addon_core::version::ENGINE_VERSION;
    let at = format!("{}.{}.{}", engine.major, engine.minor, engine.patch);
    let above = format!("{}.{}.{}", engine.major, engine.minor, engine.patch + 1);
    loads(&text.replace("1.21.60", &at));
    let found = rejects(&text.replace("1.21.60", &above));
    assert!(matches!(
        one_error(&found).code,
        DiagnosticCode::FormatVersionAboveEngine { .. }
    ));
}

#[test]
fn cat12_format_version_1_26_50_loads_the_same_document() {
    let text = fixture("cat01_all_categories.json").replace("1.21.60", "1.26.50");
    let document = loads(&text);
    assert_eq!(document.value(), &cat01());
    assert_eq!(
        document.version().written,
        Some(FormatVersion::new(1, 26, 50))
    );
    // The strict 1.26.50 slice: the same members, no finding.
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 26, 50));
    assert!(document.diagnostics().is_empty());
}

#[test]
fn cat13_every_vanilla_stamp_loads_the_same_document_without_a_finding() {
    for stamp in ["1.21.70", "1.21.110", "1.26.10", "1.26.30", "1.26.40"] {
        let text = fixture("cat01_all_categories.json").replace("1.21.60", stamp);
        let document = loads(&text);
        assert_eq!(document.value(), &cat01(), "{stamp}");
        assert!(document.diagnostics().is_empty(), "{stamp}");
        assert_eq!(document.version().parsed_as, FormatVersion::new(1, 21, 60));
    }
}

/// `vanilla_1.26.40`'s loose file of the retail 1.26.45 build, verbatim: `//` comments inside
/// `items` arrays.
#[test]
fn cat14_vanilla_s_file_with_line_comments_loads() {
    let text = fixture("cat14_line_comments.json");
    assert!(text.contains("//"));
    let document = loads(&text);
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(document.categories.len(), 4);
    let groups: usize = document.categories.iter().map(|c| c.groups.len()).sum();
    let items: usize = document
        .categories
        .iter()
        .flat_map(|c| &c.groups)
        .map(|g| g.items.len())
        .sum();
    assert_eq!((groups, items), (119, 1700));
    assert_eq!(
        document.version().written,
        Some(FormatVersion::new(1, 26, 40))
    );
}
