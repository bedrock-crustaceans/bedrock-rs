//! What the attachable reader does where the client entity's twin does otherwise (the drifts
//! between the two copies of one schema). Each test pins today's attachable side; which side
//! is the game's is not decided here.

use super::common::*;

/// D3: an array or an object as the condition of a list entry is a wrong type the attachable
/// words as Molang; the client entity says "string".
#[test]
fn a_list_entry_condition_of_the_wrong_type_allows_molang() {
    for version in ["1.10.0", "1.26.0"] {
        for (members, at) in [
            (
                r#""render_controllers": [{"c": []}]"#,
                "render_controllers/0/c",
            ),
            (
                r#""scripts": {"animate": [{"w": []}]}"#,
                "scripts/animate/0/w",
            ),
        ] {
            let found = rejected(attachable(&ghost(version, members), &cx()));
            assert_eq!(
                lines(&found),
                [format!(
                    "Error InvalidType minecraft:attachable/description/{at} unknown child schema option type.  Allowed types: molang"
                )],
                "{version} {members}"
            );
        }
    }
}

/// D4: below 1.18.20 the loader lower-cases a named script's whole text, quoted literals
/// included. The client entity notes a literal whose case that changes
/// (`client_entity/steps.rs`); the attachable folds it and says nothing.
#[test]
fn a_script_literal_the_lower_casing_changes_is_folded_silently() {
    let members = r#""scripts": {"pre_animation": ["Variable.Foo = 1;", "Variable.Title = 'Big Boss';", "Variable.Tag = 'small';"], "scale": "Query.Get_Name == 'Steve' ? 2 : 1", "parent_setup": "Variable.Hidden = 0.0;"}"#;
    for version in ["1.10.0", "1.8.0"] {
        let document = loaded(attachable(&ghost(version, members), &cx()));
        let scripts = document.description.scripts.as_ref().unwrap();
        assert_eq!(
            scripts.pre_animation.as_ref().unwrap()[1],
            "variable.title = 'big boss';",
            "{version}"
        );
        none(document.diagnostics(), "ValueRewritten");
    }
}
