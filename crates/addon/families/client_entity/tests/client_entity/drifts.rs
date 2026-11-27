//! What the client entity reader does where the attachable's twin does otherwise (the drifts
//! between the two copies of one schema). Each test pins today's client entity side; which
//! side is the game's is not decided here.

use super::common::*;

/// D2: a number no double holds below `min_engine_version`'s top level. The attachable keeps
/// the largest finite double with a warning (`attachable/numbers.rs`); the client entity does
/// not clamp there: the file loads and nothing is told.
#[test]
fn a_number_no_double_holds_in_min_engine_version_is_not_clamped() {
    for version in ["1.8.0", "1.10.0", "1.26.0"] {
        let document = loaded(entity(
            &ghost(version, r#""min_engine_version": [1e999, 0, 0]"#),
            &cx(),
        ));
        assert!(document.description.min_engine_version.is_some());
        assert_eq!(lines(document.diagnostics()), [""; 0], "{version}");
    }
}

/// D3: an array or an object as the condition of a list entry is a wrong type the client
/// entity words by the member's name ("string"); the attachable says "molang".
#[test]
fn a_list_entry_condition_of_the_wrong_type_allows_a_string() {
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
            let found = rejected(entity(&ghost(version, members), &cx()));
            assert_eq!(
                lines(&found),
                [format!(
                    "Error core/unknown_child_option minecraft:client_entity/description/{at} unknown child schema option type.  Allowed types: string"
                )],
                "{version} {members}"
            );
        }
    }
}
