//! A `null` where the schema declares a member (vanilla's schema
//! states it for `item`: "null included"; the same `findChildSchema` reads every
//! child): no type option takes it, so the file fails. A typed reader would take a `null`
//! for an absent member and write the document back without it.

use bedrock_addon_core::diagnostic::Severity;

use super::common::*;

const TYPE_ERROR: &str = "unknown child schema option type.  Allowed types: ";
const DESCRIPTION: &str = "minecraft:attachable/description";

/// Requires `members` to reject the document with one finding: a wrong type at `path`
/// (below the description), naming `allowed`.
#[track_caller]
fn refused(version: &str, members: &str, path: &str, allowed: &str) {
    let d = rejected(attachable(&ghost(version, members), &cx()));
    expect(
        &d,
        "core/unknown_child_option",
        Severity::Error,
        &format!("{TYPE_ERROR}{allowed}"),
    );
    expect_at(
        &d,
        "core/unknown_child_option",
        Severity::Error,
        &format!("{DESCRIPTION}/{path}"),
    );
    assert_eq!(d.len(), 1, "{version} {members}: {}", lines(&d).join("\n"));
}

#[test]
fn a_null_description_member_fails_the_file_at_every_version() {
    for version in ["1.8.0", "1.10.0", "1.26.0"] {
        for (member, allowed) in [
            ("min_engine_version", "string, array"),
            ("materials", "object"),
            ("textures", "object"),
            ("geometry", "object"),
            ("spawn_egg", "object"),
            ("scripts", "object"),
            ("particle_effects", "object"),
            ("particle_emitters", "object"),
            ("animations", "object"),
            ("render_controllers", "array"),
            ("sound_effects", "object"),
            ("enable_attachables", "bool"),
            ("held_item_ignores_lighting", "bool"),
            ("hide_armor", "bool"),
        ] {
            refused(version, &format!(r#""{member}": null"#), member, allowed);
        }
    }
    // The members a version does not have are undeclared there, `null` or not.
    for version in ["1.10.0", "1.26.0"] {
        refused(
            version,
            r#""queryable_geometry": null"#,
            "queryable_geometry",
            "string",
        );
    }
    refused(
        "1.26.0",
        r#""held_item_scale": null"#,
        "held_item_scale",
        "float",
    );
    refused(
        "1.8.0",
        r#""animation_controllers": null"#,
        "animation_controllers",
        "array",
    );
    let d = rejected(attachable(
        &ghost("1.8.0", r#""queryable_geometry": null"#),
        &cx(),
    ));
    expect(
        &d,
        "core/child_not_valid",
        Severity::Error,
        "child 'queryable_geometry' not valid here.",
    );
    assert_eq!(d.len(), 1);
}

#[test]
fn a_null_identifier_is_a_wrong_type() {
    for version in ["1.8.0", "1.10.0", "1.26.0"] {
        let d = rejected(attachable(&doc(version, r#"{"identifier": null}"#), &cx()));
        expect(
            &d,
            "core/unknown_child_option",
            Severity::Error,
            &format!("{TYPE_ERROR}string"),
        );
        expect_at(
            &d,
            "core/unknown_child_option",
            Severity::Error,
            &format!("{DESCRIPTION}/identifier"),
        );
    }
}

#[test]
fn a_null_member_of_scripts_or_spawn_egg_fails_the_file() {
    for version in ["1.8.0", "1.10.0", "1.26.0"] {
        for member in ["pre_animation", "scale", "scaleX", "scalez", "parent_setup"] {
            let allowed = if member == "pre_animation" {
                "array"
            } else {
                "molang"
            };
            refused(
                version,
                &format!(r#""scripts": {{"{member}": null}}"#),
                &format!("scripts/{member}"),
                allowed,
            );
        }
        for (member, allowed) in [
            ("texture", "string"),
            ("texture_index", "int"),
            ("base_color", "string"),
            ("overlay_color", "string"),
        ] {
            refused(
                version,
                &format!(r#""spawn_egg": {{"{member}": null}}"#),
                &format!("spawn_egg/{member}"),
                allowed,
            );
        }
    }
    for version in ["1.10.0", "1.26.0"] {
        for (member, allowed) in [
            ("variables", "object"),
            ("initialize", "array"),
            ("animate", "array"),
            ("should_update_bones_and_effects_offscreen", "molang"),
            ("should_update_effects_offscreen", "molang"),
        ] {
            refused(
                version,
                &format!(r#""scripts": {{"{member}": null}}"#),
                &format!("scripts/{member}"),
                allowed,
            );
        }
    }
    refused(
        "1.26.0",
        r#""scripts": {"hide_held_items": null}"#,
        "scripts/hide_held_items",
        "molang",
    );
}

#[test]
fn a_null_inside_a_map_or_a_list_fails_the_file() {
    for version in ["1.8.0", "1.10.0", "1.26.0"] {
        for map in [
            "materials",
            "textures",
            "geometry",
            "particle_effects",
            "particle_emitters",
            "animations",
            "sound_effects",
        ] {
            refused(
                version,
                &format!(r#""{map}": {{"default": null}}"#),
                &format!("{map}/default"),
                "string",
            );
        }
        refused(
            version,
            r#""render_controllers": ["controller.render.a", null]"#,
            "render_controllers/1",
            if version == "1.8.0" {
                "string"
            } else {
                "string, object"
            },
        );
        refused(
            version,
            r#""scripts": {"pre_animation": ["variable.a = 1;", null]}"#,
            "scripts/pre_animation/1",
            "string",
        );
    }
    for version in ["1.10.0", "1.26.0"] {
        refused(
            version,
            r#""scripts": {"animate": [null]}"#,
            "scripts/animate/0",
            "string, object",
        );
        refused(
            version,
            r#""scripts": {"variables": {"variable.a": null}}"#,
            "scripts/variables/variable.a",
            "string",
        );
        // The condition of an object form is Molang: a number, a boolean or a string.
        refused(
            version,
            r#""render_controllers": [{"controller.render.a": null}]"#,
            "render_controllers/0/controller.render.a",
            "molang",
        );
        refused(
            version,
            r#""animations": {"a": "animation.a"}, "scripts": {"animate": [{"a": null}]}"#,
            "scripts/animate/0/a",
            "molang",
        );
    }
}

#[test]
fn a_null_payload_or_description_is_no_object() {
    let d = rejected(attachable(
        r#"{"format_version": "1.10.0", "minecraft:attachable": {"description": null}}"#,
        &cx(),
    ));
    expect(
        &d,
        "core/unknown_child_option",
        Severity::Error,
        &format!("{TYPE_ERROR}object"),
    );
    let d = rejected(attachable(
        r#"{"format_version": "1.10.0", "minecraft:attachable": null}"#,
        &cx(),
    ));
    assert!(d.iter().any(|f| f.severity == Severity::Error));
}
