//! The severity of an unknown member or component of a block: what each format version tells
//! of the same small documents. The slices from 1.26.20 on are the cereal ones (an unknown
//! member an error); the older ones are the hand-written legacy reader (a warning). Each
//! finding is graded where it is raised; the tables pin what that gives.

use std::fmt::Write as _;

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::document::Content;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_block::Block;

/// The versions, oldest first: legacy slices, the last legacy one, the first cereal one and the
/// newest, and beta.
const VERSIONS: [&str; 6] = [
    "1.16.100", "1.21.90", "1.26.0", "1.26.20", "1.26.60", "beta",
];

/// What each version tells of `payload`: `version loaded|refused` then one line per finding
/// at information or above, `severity code path`. The beta document is read with the Beta APIs
/// toggle on, which its envelope needs.
fn told(payload: &str) -> String {
    let mut out = String::new();
    for version in VERSIONS {
        let mut context = LoadContext::detached();
        context.beta_apis = version == "beta";
        let path = PackPath::new("blocks/demo/x.json").unwrap();
        let text = format!(r#"{{"format_version": "{version}", "minecraft:block": {payload}}}"#);
        let (found, loaded) = match Block::parse(&path, text.as_bytes(), &context) {
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
                finding.code.id(),
                finding.location.json_path
            );
        }
    }
    out
}

#[test]
fn an_unknown_root_member_is_graded_by_the_version() {
    let found =
        told(r#"{"description": {"identifier": "demo:x"}, "components": {}, "made_up": {}}"#);
    assert_eq!(
        found,
        r"1.16.100 loaded
  Warning core/unrecognized_field minecraft:block/made_up
1.21.90 loaded
  Warning core/unrecognized_field minecraft:block/made_up
1.26.0 loaded
  Warning core/unrecognized_field minecraft:block/made_up
1.26.20 refused
  Error core/unrecognized_field minecraft:block/made_up
1.26.60 refused
  Error core/unrecognized_field minecraft:block/made_up
beta refused
  Error core/no_parser_for_version format_version
",
        "\n{found}"
    );
}

#[test]
fn an_unknown_description_member_is_graded_by_the_version() {
    let found = told(r#"{"description": {"identifier": "demo:x", "note": 1}, "components": {}}"#);
    assert_eq!(
        found,
        r"1.16.100 loaded
  Warning core/unrecognized_field minecraft:block/description/note
1.21.90 loaded
  Warning core/unrecognized_field minecraft:block/description/note
1.26.0 loaded
  Warning core/unrecognized_field minecraft:block/description/note
1.26.20 refused
  Error core/unrecognized_field minecraft:block/description/note
1.26.60 refused
  Error core/unrecognized_field minecraft:block/description/note
beta refused
  Error core/no_parser_for_version format_version
",
        "\n{found}"
    );
}

#[test]
fn an_unknown_component_is_graded_by_the_version() {
    let found =
        told(r#"{"description": {"identifier": "demo:x"}, "components": {"demo:made_up": {}}}"#);
    assert_eq!(
        found,
        r"1.16.100 loaded
1.21.90 loaded
1.26.0 loaded
1.26.20 loaded
1.26.60 loaded
beta refused
  Error core/no_parser_for_version format_version
",
        "\n{found}"
    );
}

#[test]
fn an_unknown_member_of_a_component_is_graded_by_the_version() {
    let found = told(
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:destructible_by_explosion": {"explosion_resistance": 1, "zz": 1}}}"#,
    );
    assert_eq!(
        found,
        r"1.16.100 loaded
  Warning core/unrecognized_field minecraft:block/components/minecraft:destructible_by_explosion/zz
1.21.90 loaded
  Warning core/unrecognized_field minecraft:block/components/minecraft:destructible_by_explosion/zz
1.26.0 loaded
  Warning core/unrecognized_field minecraft:block/components/minecraft:destructible_by_explosion/zz
1.26.20 refused
  Error core/unrecognized_field minecraft:block/components/minecraft:destructible_by_explosion/zz
1.26.60 refused
  Error core/unrecognized_field minecraft:block/components/minecraft:destructible_by_explosion/zz
beta refused
  Error core/no_parser_for_version format_version
",
        "\n{found}"
    );
}

#[test]
fn an_unknown_member_of_a_permutation_is_graded_by_the_version() {
    let found = told(
        r#"{"description": {"identifier": "demo:x"}, "components": {}, "permutations": [{"condition": "1", "components": {}, "comment": 1}]}"#,
    );
    assert_eq!(
        found,
        r"1.16.100 loaded
  Warning core/unrecognized_field minecraft:block/permutations/0/comment
  Info core/molang_not_checked minecraft:block
1.21.90 loaded
  Warning core/unrecognized_field minecraft:block/permutations/0/comment
  Info core/molang_not_checked minecraft:block
1.26.0 loaded
  Warning core/unrecognized_field minecraft:block/permutations/0/comment
  Info core/molang_not_checked minecraft:block
1.26.20 refused
  Error core/unrecognized_field minecraft:block/permutations/0/comment
1.26.60 refused
  Error core/unrecognized_field minecraft:block/permutations/0/comment
beta refused
  Error core/no_parser_for_version format_version
",
        "\n{found}"
    );
}
