//! validation.md V6, and V7 as far as one file shows
//! it. The validators run only on a file whose parse succeeded.

use super::common::*;

const EXTERNAL: &str =
    "external packs must not use the 'minecraft' namespace for DataDrivenUI documents";
const BUILTIN: &str = "builtin packs must use the 'minecraft' namespace for DataDrivenUI documents";
const OVERRIDEN: &str = "extension_point cannot be overriden at external packs.";
const ROOT_ID: &str = "minecraft:ui-root/description/identifier";
const COMPOSITION_ID: &str = "minecraft:ui-composition/description/identifier";
const DIVIDER: &str = r#"{ "component": "Form.Divider" }"#;

fn named(identifier: &str) -> String {
    root_json(identifier, R_MARKUP)
}

#[test]
fn n01_an_add_on_root_in_the_minecraft_namespace_is_refused() {
    let text = named("minecraft:x");
    let d = rejects(root(&text));
    assert_eq!(
        found(&d),
        [
            error("ddui/external_namespace", ROOT_ID, EXTERNAL),
            failed_to_load()
        ]
    );
    assert_eq!(spanned(&text, d.iter().next().unwrap()), r#""minecraft:x""#);
}

/// The client looks for the bytes `minecraft` anywhere in the identifier.
#[test]
fn n02_minecraft_anywhere_in_an_add_on_identifier_is_refused_with_a_hint_deviation() {
    for identifier in [
        "demo:minecraft_style",
        "notminecraft:x",
        "demo:xminecraft",
        "minecraftx:y",
        "a:b:minecraft",
    ] {
        assert_eq!(
            found(&rejects(root(&named(identifier)))),
            [
                error("ddui/external_namespace", ROOT_ID, EXTERNAL),
                // Ours: the identifier is not in the `minecraft` namespace and is refused all
                // the same.
                info("core/namespace_rule", ROOT_ID, "namespace rule failed"),
                failed_to_load()
            ],
            "{identifier}"
        );
    }
}

#[test]
fn n03_the_test_is_case_sensitive_and_needs_the_whole_word() {
    for identifier in [
        "Minecraft:x",
        "MINECRAFT:x",
        "demo:Minecraft",
        "minecraf:t",
        "mine:craft",
    ] {
        let document = loads(root(&named(identifier)));
        assert!(document.diagnostics().is_empty(), "{identifier}");
    }
}

#[test]
fn n04_a_shipped_root_without_minecraft_is_refused() {
    for identifier in ["demo:x", "Minecraft:x", "minecraf:t"] {
        assert_eq!(
            found(&rejects(root_in(&named(identifier), &cx(E20, true)))),
            [
                error("ddui/builtin_namespace", ROOT_ID, BUILTIN),
                failed_to_load()
            ],
            "{identifier}"
        );
    }
}

#[test]
fn n05_a_shipped_root_with_minecraft_anywhere_loads() {
    for identifier in ["demo:minecraft_x", "minecraft:x", "notminecraft:x"] {
        let document = loads(root_in(&named(identifier), &cx(E20, true)));
        assert!(document.diagnostics().is_empty(), "{identifier}");
    }
}

#[test]
fn n06_the_rule_holds_for_compositions() {
    let text = composition_json("minecraft:c", "", DIVIDER);
    assert_eq!(
        found(&rejects(composition(&text))),
        [
            error("ddui/external_namespace", COMPOSITION_ID, EXTERNAL),
            failed_to_load()
        ]
    );
    loads(composition_in(&text, &cx(E20, true)));
    let text = composition_json("demo:c", "", DIVIDER);
    assert_eq!(
        found(&rejects(composition_in(&text, &cx(E20, true)))),
        [
            error("ddui/builtin_namespace", COMPOSITION_ID, BUILTIN),
            failed_to_load()
        ]
    );
    loads(composition(&text));
}

/// V7: only a pack the game ships may name an extension point.
#[test]
fn an_add_on_composition_with_an_extension_point_is_refused() {
    let text = composition_json("demo:c", r#""extension_point": "demo:slot""#, DIVIDER);
    let d = rejects(composition(&text));
    assert_eq!(
        found(&d),
        [
            error(
                "ddui/extension_point_overridden",
                "minecraft:ui-composition/attribs/extension_point",
                OVERRIDEN
            ),
            failed_to_load()
        ]
    );
    assert_eq!(spanned(&text, d.iter().next().unwrap()), r#""demo:slot""#);
    // The same member in a shipped pack is the stack's to check, not the file's.
    let shipped = composition_json("minecraft:c", r#""extension_point": "demo:slot""#, DIVIDER);
    let document = loads(composition_in(&shipped, &cx(E20, true)));
    assert_eq!(
        document.attribs.extension_point.as_deref(),
        Some("demo:slot")
    );
}

/// The namespace rule does not end the validation: both messages, in the client's order.
#[test]
fn the_namespace_rule_and_the_extension_point_rule_are_both_reported() {
    let text = composition_json("minecraft:c", r#""extension_point": "demo:slot""#, DIVIDER);
    assert_eq!(
        found(&rejects(composition(&text))),
        [
            error("ddui/external_namespace", COMPOSITION_ID, EXTERNAL),
            error(
                "ddui/extension_point_overridden",
                "minecraft:ui-composition/attribs/extension_point",
                OVERRIDEN
            ),
            failed_to_load()
        ]
    );
}

/// The validators run after a successful parse only.
#[test]
fn a_file_that_fails_the_parse_is_not_validated() {
    let text = root_json("minecraft:x", r#"{ "component": "Nope" }"#);
    let rejected = rejects(root(&text));
    let got = found(&rejected);
    assert!(
        rejected
            .iter()
            .all(|f| f.code.kind() != bedrock_addon::diagnostic::Kind::Identifier),
        "{got:#?}"
    );
}
