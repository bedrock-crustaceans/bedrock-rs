//! Validation on one file: an unknown pass, the structural errors
//! and the runtime scatter checks.

use bedrock_addon_core::diagnostic::Severity;

use super::common::*;

#[test]
fn r04_an_unknown_placement_pass_is_logged_and_the_rule_still_loads() {
    let document = loads(&rule("1.13.0", "nowhere_pass", None));
    let found = with_code(document.diagnostics(), "feature_rules/unknown_pass");
    assert_eq!(found.len(), 1, "{:#?}", document.diagnostics());
    // a kept error
    assert_eq!((found[0].severity, found[0].kept), (Severity::Error, true));
    assert_eq!(
        found[0].code.to_string(),
        "Feature rule identifier 'demo:test' specifies unknown pass 'nowhere_pass'."
    );
    assert_eq!(
        found[0].location.json_path.to_string(),
        "minecraft:feature_rules/conditions/placement_pass"
    );
    assert_eq!(document.conditions.pass(), None);
    assert_eq!(document.conditions.placement_pass, "nowhere_pass");
}

#[test]
fn rule_4_pass_names_are_compared_exactly() {
    let document = loads(&rule("1.13.0", "Surface_Pass", None));
    assert!(has(document.diagnostics(), |c| c.id()
        == "feature_rules/unknown_pass"));
    for pass in bedrock_addon_family_feature_rules::PlacementPass::ALL {
        let document = loads(&rule("1.13.0", pass.name(), None));
        assert!(
            above_info(document.diagnostics()).is_empty(),
            "{}: {:#?}",
            pass.name(),
            document.diagnostics()
        );
    }
}
