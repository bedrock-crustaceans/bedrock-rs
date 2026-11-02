//! Fixtures `am01`–`am06` and `a17c`: the conditions of
//! `description.item` and `scripts.parent_setup` under the client catalogue, at the pack's
//! Molang version. The description members shared with client entities are in `molang.rs`.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::attachable::{Condition, ItemBinding};
use bedrock_addon::molang::Molang;

use super::common::*;

const ITEM_AT: &str = "minecraft:attachable/description/item";

/// A 1.10.0 or 1.26.0 `demo:ghost` whose `item` is `{ "example:wand": <condition> }`.
fn bound(version: &str, condition: &str) -> String {
    ghost(
        version,
        &format!(r#""item": {{"example:wand": {condition}}}"#),
    )
}

/// The stored condition of `example:wand`.
fn stored(document: &bedrock_addon::families::attachable::Attachable) -> &Molang {
    match &document.description.item {
        Some(ItemBinding::Conditional(members)) => &members["example:wand"],
        other => panic!("{other:?}"),
    }
}

#[test]
fn am05_bool_and_number_conditions() {
    // The stored value keeps its JSON type; the binding holds vanilla's `asString("")`.
    let boolean = loaded(attachable(&bound("1.10.0", "true"), &cx()));
    assert_eq!(stored(&boolean), &Molang::Bool(true));
    let (_, condition) = boolean.binding().unwrap();
    assert!(matches!(&condition, Condition::Molang(source) if source.as_str() == "true"));
    clean(boolean.diagnostics());
    none(boolean.diagnostics(), "InvalidMolang");
    let number = loaded(attachable(&bound("1.10.0", "1"), &cx()));
    assert_eq!(stored(&number), &Molang::Const(1.0));
    let (_, condition) = number.binding().unwrap();
    assert!(matches!(&condition, Condition::Molang(source) if source.as_str().is_empty()));
    // The a16 warning only: the empty text is not compiled, so not reported as Molang.
    none(number.diagnostics(), "InvalidMolang");
    // the drop is an information
    let told: Vec<_> = number
        .diagnostics()
        .at_least(Severity::Info)
        .map(|f| f.code.name())
        .collect();
    assert_eq!(told, ["InvalidValue"]);
}

#[cfg(feature = "molang")]
mod compiled {
    use super::*;

    #[test]
    fn am01_item_condition_owner_query() {
        let text = bound(
            "1.10.0",
            r#""query.owner_identifier == 'minecraft:player'""#,
        );
        let document = loaded(attachable(&text, &cx()));
        silent(document.diagnostics());
        assert!(matches!(stored(&document), Molang::Expr(_)));
    }

    #[test]
    fn am02_item_condition_client_only() {
        let text = bound("1.10.0", r#""query.distance_from_camera < 8""#);
        silent(loaded(attachable(&text, &cx())).diagnostics());
    }

    #[test]
    fn am03_item_condition_is_on_screen() {
        let text = bound("1.26.0", r#""query.is_on_screen""#);
        silent(loaded(attachable(&text, &cx())).diagnostics());
    }

    #[test]
    fn am04_parent_setup() {
        let members = r#""scripts": {"parent_setup": "variable.helmet_layer_visible = 0.0;"}"#;
        for version in ["1.8.0", "1.10.0", "1.26.0"] {
            let document = loaded(attachable(&ghost(version, members), &cx()));
            silent(document.diagnostics());
        }
        // The schema types it as Molang at every version: a text the parser rejects fails
        // the file, the 1.8.0 one inside the v1_8 check.
        let members = r#""scripts": {"parent_setup": "variable.x = (;"}"#;
        for version in ["1.8.0", "1.10.0", "1.26.0"] {
            let d = rejected(attachable(&ghost(version, members), &cx()));
            expect_at(
                &d,
                "InvalidMolang",
                Severity::Error,
                "minecraft:attachable/description/scripts/parent_setup",
            );
        }
    }

    #[test]
    fn am06_item_unknown_query() {
        for version in ["1.10.0", "1.26.0"] {
            let d = rejected(attachable(&bound(version, r#""query.not_a_query""#), &cx()));
            let invalid: Vec<_> = d
                .iter()
                .filter(|f| f.code.name() == "InvalidMolang")
                .collect();
            assert!(!invalid.is_empty());
            for found in &invalid {
                assert_eq!(found.severity, Severity::Error);
                assert_eq!(
                    found.location.json_path.to_string(),
                    format!("{ITEM_AT}/example:wand")
                );
            }
            assert!(
                invalid.iter().any(|f| f.to_string().contains(
                    "Failed to resolve query query.not_a_query.  Either the query does not exist or it is not supported in this context."
                )),
                "{}",
                lines(&d).join("\n")
            );
        }
    }

    #[test]
    fn a17c_item_condition_does_not_compile() {
        for version in ["1.10.0", "1.26.0"] {
            let d = rejected(attachable(&bound(version, r#""q.is_sneaking &&""#), &cx()));
            expect_at(
                &d,
                "InvalidMolang",
                Severity::Error,
                &format!("{ITEM_AT}/example:wand"),
            );
        }
    }

    #[test]
    fn a17c_a_member_that_does_not_bind_is_compiled_too() {
        // "every member is still schema-validated" (rule 17).
        let members = r#""item": {"example:a": "q.is_sneaking", "example:b": "q.is_sneaking &&"}"#;
        let d = rejected(attachable(&ghost("1.10.0", members), &cx()));
        expect_at(
            &d,
            "InvalidMolang",
            Severity::Error,
            &format!("{ITEM_AT}/example:b"),
        );
    }

    #[test]
    fn the_string_form_is_no_molang() {
        // `"item": "q.is_sneaking &&"` is an item name (an odd one): never compiled.
        let document = loaded(attachable(
            &ghost("1.10.0", r#""item": "q.is_sneaking &&""#),
            &cx(),
        ));
        none(document.diagnostics(), "InvalidMolang");
    }
}

#[cfg(not(feature = "molang"))]
#[test]
fn a17c_without_the_molang_feature_the_binding_loads_deviation() {
    // Vanilla rejects the file by the schema's Molang compile: without the compile
    // check nothing can tell, and the binding is kept.
    let document = loaded(attachable(&bound("1.10.0", r#""q.is_sneaking &&""#), &cx()));
    none(document.diagnostics(), "InvalidMolang");
    assert!(find(document.diagnostics(), "MolangNotChecked", Severity::Info).is_some());
    assert!(document.binding().is_some());
}
