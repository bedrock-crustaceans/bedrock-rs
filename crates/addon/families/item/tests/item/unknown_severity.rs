//! Characterization of the severity of an unknown member or component of an item (phase 7,
//! grading): what each format version tells of the same small documents. A component-based
//! document from 1.16.100 on is read by the modern (cereal) slices, where an unknown member is
//! an error; an older one, or one the route sends to the legacy format, by the legacy slice, where
//! it is a warning. The tables pin what the reader tells today, whatever mechanism grades it, so
//! that moving the grading from the post-parse lift to the raise sites changes nothing.

use std::fmt::Write as _;

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::document::Content;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_item::Item;

/// The versions, oldest first: the legacy format, the first modern ones, the 1.26.30 struct,
/// the newest, and beta.
const VERSIONS: [&str; 6] = [
    "1.10.0", "1.16.100", "1.21.60", "1.26.30", "1.26.60", "beta",
];

/// What each version tells of `payload`: `version loaded|refused` then one line per finding
/// at information or above, `severity code path`. The beta document is read with the Beta APIs
/// toggle on, which its envelope needs.
fn told(payload: &str) -> String {
    let mut out = String::new();
    for version in VERSIONS {
        let mut context = LoadContext::detached();
        context.beta_apis = version == "beta";
        let path = PackPath::new("items/test.json").unwrap();
        let text = format!(r#"{{"format_version": "{version}", "minecraft:item": {payload}}}"#);
        let (found, loaded) = match Item::parse(&path, text.as_bytes(), &context) {
            Ok(document) => (document.diagnostics().clone(), true),
            Err(rejected) => (rejected.diagnostics, false),
        };
        out.push_str(version);
        out.push_str(if loaded { " loaded\n" } else { " refused\n" });
        for finding in found.iter().filter(|f| f.severity >= Severity::Info) {
            let _ = writeln!(
                out,
                "  {:?} {} {}",
                finding.severity,
                finding.code.name(),
                finding.location.json_path
            );
        }
    }
    out
}

#[test]
fn an_unknown_root_member_is_graded_by_the_version() {
    let found = told(
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:max_stack_size": 16}, "made_up": {}}"#,
    );
    assert_eq!(
        found,
        r"1.10.0 loaded
  Info LegacyItemFormat minecraft:item
1.16.100 refused
  Warning UnrecognizedField minecraft:item/made_up
  Error MissingToggles 
1.21.60 loaded
  Warning UnrecognizedField minecraft:item/made_up
  Info ValueRewritten minecraft:item/description/menu_category
  Info InvalidValue minecraft:item/made_up
1.26.30 refused
  Error UnrecognizedField minecraft:item/made_up
1.26.60 refused
  Error UnrecognizedField minecraft:item/made_up
beta refused
  Error UnrecognizedField minecraft:item/made_up
",
        "\n{found}"
    );
}

#[test]
fn an_unknown_description_member_is_graded_by_the_version() {
    let found = told(
        r#"{"description": {"identifier": "demo:x", "note": 1}, "components": {"minecraft:max_stack_size": 16}}"#,
    );
    assert_eq!(
        found,
        r"1.10.0 loaded
  Info LegacyItemFormat minecraft:item
1.16.100 refused
  Warning UnrecognizedField minecraft:item/description/note
  Error MissingToggles 
1.21.60 loaded
  Warning UnrecognizedField minecraft:item/description/note
  Info ValueRewritten minecraft:item/description/menu_category
  Info InvalidValue minecraft:item/description/note
1.26.30 refused
  Error UnrecognizedField minecraft:item/description/note
1.26.60 refused
  Error UnrecognizedField minecraft:item/description/note
beta refused
  Error UnrecognizedField minecraft:item/description/note
",
        "\n{found}"
    );
}

#[test]
fn an_unknown_component_is_graded_by_the_version() {
    let found = told(
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:max_stack_size": 16, "demo:made_up": {}}}"#,
    );
    assert_eq!(
        found,
        r"1.10.0 loaded
  Info LegacyItemFormat minecraft:item
1.16.100 refused
  Error MissingToggles 
1.21.60 loaded
  Info UnrecognizedComponent minecraft:item/components/demo:made_up
  Info ValueRewritten minecraft:item/description/menu_category
1.26.30 loaded
1.26.60 loaded
beta refused
  Error UnsupportedVersion 
",
        "\n{found}"
    );
}

#[test]
fn an_unknown_member_of_a_component_is_graded_by_the_version() {
    let found = told(
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:tags": {"tags": [], "note": 1}}}"#,
    );
    assert_eq!(
        found,
        r"1.10.0 loaded
  Info LegacyItemFormat minecraft:item
1.16.100 refused
  Error MissingToggles 
1.21.60 loaded
  Warning UnrecognizedField minecraft:item/components/minecraft:tags/note
  Info ValueRewritten minecraft:item/description/menu_category
  Info InvalidValue minecraft:item/components/minecraft:tags/note
1.26.30 refused
  Error UnrecognizedField minecraft:item/components/minecraft:tags/note
1.26.60 refused
  Error UnrecognizedField minecraft:item/components/minecraft:tags/note
beta refused
  Error UnrecognizedField minecraft:item/components/minecraft:tags/note
",
        "\n{found}"
    );
}

/// `events` is a member of the legacy format only.
#[test]
fn a_legacy_only_member_is_graded_by_the_version() {
    let found = told(
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:max_stack_size": 16}, "events": {}}"#,
    );
    assert_eq!(
        found,
        r"1.10.0 loaded
  Info LegacyItemFormat minecraft:item
1.16.100 refused
  Error MissingToggles 
1.21.60 loaded
  Info ValueRewritten minecraft:item/description/menu_category
  Info InvalidValue minecraft:item/events
1.26.30 refused
  Error UnrecognizedField minecraft:item/events
1.26.60 refused
  Error UnrecognizedField minecraft:item/events
beta refused
  Error UnrecognizedField minecraft:item/events
",
        "\n{found}"
    );
}
