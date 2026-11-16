//! What the legacy reader refuses of a block document below 1.26.20 and how it says it: the
//! shapes of `description`, `components`, `permutations` and the name lists, each told in the
//! game's words at the member it is about, the read given up at the first; a component written
//! twice keeps its first value; a member the reader does not read is kept without a word.

use std::fmt::Write as _;

use bedrock_addon_core::diagnostic::Severity;

use super::common::{context, document, load};

/// What a `1.20.0` document of `payload` gives: `loaded|refused`, then one line per finding at
/// information or above, `severity kept code path`.
fn told(payload: &str) -> String {
    let (found, loaded) = match load(&document("1.20.0", payload), &context()) {
        Ok(document) => (document.diagnostics().clone(), true),
        Err(rejected) => (rejected.diagnostics, false),
    };
    let mut out = String::from(if loaded { "loaded\n" } else { "refused\n" });
    for finding in found.iter().filter(|f| f.severity >= Severity::Info) {
        let _ = writeln!(
            out,
            "  {:?}{} {} {}",
            finding.severity,
            if finding.kept { " kept" } else { "" },
            finding.code.id(),
            finding.location.json_path
        );
    }
    out
}

#[test]
fn the_legacy_reader_refuses_each_shape_in_its_own_words() {
    let cases = [
        r#"{"components": {}}"#,
        r#"{"description": [], "components": {}}"#,
        r#"{"description": null, "components": {}}"#,
        r#"{"description": {"identifier": "demo:x", "states": [1]}, "components": {}}"#,
        r#"{"description": {"identifier": "demo:x"}, "components": []}"#,
        r#"{"description": {"identifier": "demo:x"}, "components": {}, "permutations": {}}"#,
        r#"{"description": {"identifier": "demo:x"}, "components": {}, "permutations": [{"condition": "1", "components": {}}, 2]}"#,
        r#"{"description": {"identifier": "demo:x"}, "components": {}, "permutations": [{"condition": "1", "components": null}]}"#,
        r#"{"description": {"identifier": "demo:x"}, "components": {}, "permutations": [{"ordered_v1_custom_components": 3, "condition": "1"}]}"#,
        r#"{"components": {"minecraft:friction": 0.5, "minecraft:friction": 0.8}, "description": 1}"#,
    ];
    let mut out = String::new();
    for payload in cases {
        let _ = write!(out, "{payload}\n{}", told(payload));
    }
    assert_eq!(
        out,
        r#"{"components": {}}
refused
  Error block/member_missing minecraft:block/description
{"description": [], "components": {}}
refused
  Error block/member_missing minecraft:block/description
{"description": null, "components": {}}
refused
  Error block/member_missing minecraft:block/description
{"description": {"identifier": "demo:x", "states": [1]}, "components": {}}
refused
  Error block/states_object minecraft:block/description/states
  Error block/parse_block_failed minecraft:block/description
{"description": {"identifier": "demo:x"}, "components": []}
refused
  Error block/components_object minecraft:block/components
{"description": {"identifier": "demo:x"}, "components": {}, "permutations": {}}
refused
  Error block/permutations_array minecraft:block/permutations
  Error block/permutations_array_object minecraft:block/permutations
{"description": {"identifier": "demo:x"}, "components": {}, "permutations": [{"condition": "1", "components": {}}, 2]}
refused
  Error block/permutation_object minecraft:block/permutations/1
  Error block/permutations_objects minecraft:block/permutations/1
{"description": {"identifier": "demo:x"}, "components": {}, "permutations": [{"condition": "1", "components": null}]}
refused
  Error block/permutation_object minecraft:block/permutations/0
{"description": {"identifier": "demo:x"}, "components": {}, "permutations": [{"ordered_v1_custom_components": 3, "condition": "1"}]}
refused
  Info core/invalid_type minecraft:block/permutations/0/ordered_v1_custom_components
  Error block/permutation_object minecraft:block/permutations/0
{"components": {"minecraft:friction": 0.5, "minecraft:friction": 0.8}, "description": 1}
refused
  Error kept block/duplicate_component minecraft:block/components/minecraft:friction
  Error block/member_missing minecraft:block/description
"#,
        "\n{out}"
    );
}

#[test]
fn the_legacy_reader_keeps_the_first_component_of_two_and_reads_on() {
    let found = told(
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:friction": 0.5, "minecraft:friction": 0.8},
            "permutations": [{"condition": "1", "components": {"minecraft:light_dampening": 1, "minecraft:light_dampening": 2}, "ordered_v1_custom_components": ["a", 1]}],
            "ordered_v1_custom_components": "a", "made_up": 1}"#,
    );
    assert_eq!(
        found,
        r"loaded
  Error kept block/duplicate_component minecraft:block/components/minecraft:friction
  Error kept block/duplicate_component minecraft:block/permutations/0/components/minecraft:light_dampening
  Info core/invalid_type minecraft:block/permutations/0/ordered_v1_custom_components/1
  Info core/invalid_type minecraft:block/ordered_v1_custom_components
  Warning core/unrecognized_field minecraft:block/made_up
  Info core/molang_not_checked minecraft:block
",
        "\n{found}"
    );
}
