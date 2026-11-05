//! The rules an attachable shares with a client entity (the entry-level rules and the
//! description members, which refer to the client entity's schema): the version gate and the description's schema. The tests keep the
//! client entity fixture ids (`v00`–`v09`, `l01`–`l23`) the rules were first pinned under.
//! Resource pack at `min_engine_version` 1.21.0 unless the row says otherwise.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::attachable::AnimationKind;
use bedrock_addon::version::{ENGINE_VERSION, FormatVersion};
use bedrock_addon::visual::{Color, ControllerUse, HasVisuals, HeldItemScale};

use super::common::*;

const NOT_VALID: &str = "UnrecognizedField";
const TYPE_ERROR: &str = "unknown child schema option type.  Allowed types: ";

fn child(name: &str) -> String {
    format!("child '{name}' not valid here.")
}

// ---- version gate ----

#[test]
fn v00_missing_version() {
    let text = r#"{"minecraft:attachable": {"description": {"identifier": "demo:ghost"}}}"#;
    let document = loaded(attachable(text, &cx()));
    expect(
        document.diagnostics(),
        "MissingFormatVersion",
        Severity::Warning,
        "Warning: The \"format_version\" field was not given. Fields from format version 1.8.0 and older will be loaded. Update your targeted \"format_version\" to use fields from newer versions.",
    );
    // Parsed as the floor struct and upgraded from there.
    assert_eq!(document.version().written, None);
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 8, 0));
    assert!(document.version().upgraded);
}

#[test]
fn v00_missing_version_deviation_the_floor_struct_is_as_strict_as_its_schema() {
    // Vanilla's legacy path would read geometry and textures only; the
    // 1.8.0 struct rejects a member its schema lacks.
    let text = r#"{"minecraft:attachable": {"description": {"identifier": "demo:ghost", "queryable_geometry": "default"}}}"#;
    let d = rejected(attachable(text, &cx()));
    expect(&d, NOT_VALID, Severity::Error, &child("queryable_geometry"));
}

#[test]
fn v01_major_zero() {
    let d = rejected(attachable(&ghost("0.9.0", ""), &cx()));
    expect(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "Unknown/unsupported version value specified in format_version field in attachables/x.json",
    );
}

#[test]
fn v02_number_version() {
    let text = r#"{"format_version": 1.10, "minecraft:attachable": {"description": {"identifier": "demo:ghost"}}}"#;
    let d = rejected(attachable(text, &cx()));
    expect(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "Unknown/unsupported version value specified in format_version field in attachables/x.json",
    );
}

#[test]
fn v03_array_version() {
    let text = r#"{"format_version": [1, 10, 0], "minecraft:attachable": {"description": {"identifier": "demo:ghost"}}}"#;
    let document = loaded(attachable(text, &cx()));
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 10, 0));
    silent(document.diagnostics());
}

#[test]
fn v04_partial_version() {
    let document = loaded(attachable(&ghost("1.10", ""), &cx()));
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 10, 0));
}

#[test]
fn v05_below_floor() {
    let d = rejected(attachable(&ghost("1.7.0", ""), &cx()));
    expect(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "Version 1.7.0 below min required version (1.10.0)",
    );
}

#[test]
fn v05_the_floor_itself_and_one_dot_zero_bound_the_range() {
    // 1.8.0 is the first version that loads; 1.0.0 is the first that gets the schema's text.
    loaded(attachable(&ghost("1.8.0", ""), &cx()));
    let d = rejected(attachable(&ghost("1.0.0", ""), &cx()));
    expect(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "Version 1.0.0 below min required version (1.10.0)",
    );
    // A pre-release of the floor sorts below it.
    let d = rejected(attachable(&ghost("1.8.0-beta.1", ""), &cx()));
    expect(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "Version 1.8.0-beta.1 below min required version (1.10.0)",
    );
}

#[test]
fn v06_above_engine() {
    let e = ENGINE_VERSION;
    let above = format!("{}.{}.{}", e.major, e.minor, e.patch + 1);
    let d = rejected(attachable(&ghost(&above, ""), &cx()));
    expect(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        &format!(
            "Error: actor resource file has format version: {above} exceeds game version: {e}. Skipping."
        ),
    );
    // The engine version itself loads, as the newest struct.
    let document = loaded(attachable(&ghost(&e.to_string(), ""), &cx()));
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 26, 0));
    assert_eq!(document.format_version, e);
}

#[test]
fn v07_beta_literal() {
    let d = rejected(attachable(&ghost("beta", ""), &cx()));
    expect(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "Unknown/unsupported version value specified in format_version field in attachables/x.json",
    );
    none(&d, "BetaFeaturesNotEnabled");
    // Neither the Beta APIs toggle nor a base-game pack makes it a version.
    let mut beta = cx();
    beta.beta_apis = true;
    beta.is_base_game_pack = true;
    let d = rejected(attachable(&ghost("beta", ""), &beta));
    assert!(find(&d, "UnsupportedVersion", Severity::Error).is_some());
    // The any-version is no version either.
    let d = rejected(attachable(&ghost("*", ""), &cx()));
    expect_prefix(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "Unknown/unsupported version value",
    );
}

#[test]
fn v08_prerelease() {
    let members = r#""animations": {"walk": "animation.ghost.walk"}"#;
    let document = loaded(attachable(&ghost("1.10.0-beta.1", members), &cx()));
    assert!(
        find(
            document.diagnostics(),
            "PrereleaseFormatVersion",
            Severity::Info
        )
        .is_some()
    );
    // Below 1.10.0, so the 1.8 struct and its step.
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 8, 0));
    assert!(document.version().upgraded);
}

#[test]
fn v09_empty_root() {
    for text in ["{}", "null", "[]", "3"] {
        let d = rejected(attachable(text, &cx()));
        expect(&d, "MalformedInput", Severity::Error, "nothing to parse in");
    }
}

#[test]
fn comments_before_the_root_and_inside_it_are_read_over() {
    let text = "// a line comment\n/* a block */ {\"format_version\": \"1.10.0\", /* here */ \"minecraft:attachable\": // and here\n {\"description\": /* too */ {\"identifier\": \"demo:ghost\"}}}";
    silent(loaded(attachable(text, &cx())).diagnostics());
}

#[test]
fn rule_1_text_that_is_no_json_is_malformed_input() {
    let d = rejected(attachable(r#"{"format_version": "1.10.0", "#, &cx()));
    assert!(find(&d, "MalformedInput", Severity::Error).is_some());
}

// ---- loader rules ----

#[test]
fn l01_identifier_lowercase() {
    let document = loaded(attachable(
        &doc("1.10.0", r#"{"identifier": "Demo:Ghost"}"#),
        &cx(),
    ));
    // Kept as written, keyed in lower case.
    assert_eq!(document.description.identifier.as_str(), "Demo:Ghost");
    assert_eq!(document.identifier().as_str(), "demo:ghost");
}

#[test]
fn l02_missing_description() {
    let text = r#"{"format_version": "1.10.0", "minecraft:attachable": {}}"#;
    let d = rejected(attachable(text, &cx()));
    expect(
        &d,
        "MissingRequiredField",
        Severity::Error,
        "Required child description not found",
    );
    // A description that is no object is a wrong type.
    let text = r#"{"format_version": "1.10.0", "minecraft:attachable": {"description": 3}}"#;
    let d = rejected(attachable(text, &cx()));
    expect(
        &d,
        "InvalidType",
        Severity::Error,
        &format!("{TYPE_ERROR}object"),
    );
}

#[test]
fn l03_missing_identifier_without_a_version_is_the_schema_s_finding() {
    // Vanilla's "Missing identifier field in description" belongs to its legacy path, which
    // this crate never takes (deviation (a)): the 1.8.0 struct requires the member.
    let text = r#"{"minecraft:attachable": {"description": {}}}"#;
    let d = rejected(attachable(text, &cx()));
    expect(
        &d,
        "MissingRequiredField",
        Severity::Error,
        "Required child identifier not found",
    );
}

#[test]
fn rule_7_a_document_without_the_payload_is_rejected() {
    let d = rejected(attachable(r#"{"format_version": "1.10.0"}"#, &cx()));
    assert!(find(&d, "MissingRequiredField", Severity::Error).is_some());
}

#[test]
fn rule_12_an_empty_identifier_is_rejected() {
    let d = rejected(attachable(&doc("1.10.0", r#"{"identifier": ""}"#), &cx()));
    expect_at(
        &d,
        "InvalidValue",
        Severity::Error,
        "minecraft:attachable/description/identifier",
    );
}

#[test]
fn l04_held_item_scale_zero() {
    let document = loaded(attachable(
        &ghost("1.26.0", r#""held_item_scale": 0"#),
        &cx(),
    ));
    expect(
        document.diagnostics(),
        "InvalidValue",
        Severity::Error,
        "In file 'attachables/x.json', parameter \"held_item_scale\" has invalid value: must be greater than zero, was '0'.",
    );
    assert_eq!(document.held_item_scale(), 1.0);
    // Kept as written.
    assert_eq!(
        document.description.held_item_scale,
        Some(HeldItemScale::Number(0.0))
    );
}

#[test]
fn l04_held_item_scale_bounds() {
    // Below zero is refused the same way; just above it is the scale.
    let below = loaded(attachable(
        &ghost("1.26.0", r#""held_item_scale": -1.5"#),
        &cx(),
    ));
    expect(
        below.diagnostics(),
        "InvalidValue",
        Severity::Error,
        "In file 'attachables/x.json', parameter \"held_item_scale\" has invalid value: must be greater than zero, was '-1.5'.",
    );
    assert_eq!(below.held_item_scale(), 1.0);
    let above = loaded(attachable(
        &ghost("1.26.0", r#""held_item_scale": 0.25"#),
        &cx(),
    ));
    none(above.diagnostics(), "InvalidValue");
    assert_eq!(above.held_item_scale(), 0.25);
    // Absent: 1.0, nothing said.
    let absent = loaded(attachable(&ghost("1.26.0", ""), &cx()));
    silent(absent.diagnostics());
    assert_eq!(absent.held_item_scale(), 1.0);
}

#[test]
fn l05_held_item_scale_pre_1_26() {
    let d = rejected(attachable(
        &ghost("1.10.0", r#""held_item_scale": 2.0"#),
        &cx(),
    ));
    expect(&d, NOT_VALID, Severity::Error, &child("held_item_scale"));
    let d = rejected(attachable(
        &ghost("1.8.0", r#""held_item_scale": 2.0"#),
        &cx(),
    ));
    expect(&d, NOT_VALID, Severity::Error, &child("held_item_scale"));
    let document = loaded(attachable(
        &ghost("1.26.0", r#""held_item_scale": 2.0"#),
        &cx(),
    ));
    assert_eq!(document.held_item_scale(), 2.0);
    silent(document.diagnostics());
}

#[test]
fn l07_an_undeclared_script_fails_the_file_at_every_version() {
    for version in ["1.8.0", "1.10.0", "1.26.0"] {
        let d = rejected(attachable(
            &ghost(version, r#""scripts": {"My_Script": "1.0"}"#),
            &cx(),
        ));
        expect(&d, NOT_VALID, Severity::Error, &child("My_Script"));
        expect_at(
            &d,
            NOT_VALID,
            Severity::Error,
            "minecraft:attachable/description/scripts/My_Script",
        );
    }
}

#[test]
fn l08_render_controller_object_first_member() {
    let members = r#""render_controllers": [{"controller.render.a": "q.is_baby", "controller.render.b": "1"}]"#;
    let document = loaded(attachable(&ghost("1.10.0", members), &cx()));
    let controllers = document.visuals().render_controllers;
    assert_eq!(
        controllers[0].controller().unwrap().as_str(),
        "controller.render.a"
    );
    expect_at(
        document.diagnostics(),
        "DroppedField",
        Severity::Info,
        "minecraft:attachable/description/render_controllers/0/controller.render.b",
    );
    // Both members are kept as written.
    assert!(matches!(&controllers[0], ControllerUse::Conditional(m) if m.len() == 2));
}

#[test]
fn l14_first_member_byte_order() {
    let members = r#""render_controllers": [{"controller.render.b": "1", "controller.render.a": "q.is_baby"}]"#;
    let document = loaded(attachable(&ghost("1.26.0", members), &cx()));
    let controllers = document.visuals().render_controllers;
    assert_eq!(
        controllers[0].controller().unwrap().as_str(),
        "controller.render.a"
    );
    expect_at(
        document.diagnostics(),
        "DroppedField",
        Severity::Info,
        "minecraft:attachable/description/render_controllers/0/controller.render.b",
    );
    // The same rule for `scripts.animate`.
    let members = r#""animations": {"a": "animation.a", "b": "animation.b"}, "scripts": {"animate": [{"b": "1", "a": "q.is_baby"}]}"#;
    let document = loaded(attachable(&ghost("1.26.0", members), &cx()));
    expect_at(
        document.diagnostics(),
        "DroppedField",
        Severity::Info,
        "minecraft:attachable/description/scripts/animate/0/b",
    );
}

#[test]
fn l09_animations_prefix_filter() {
    let members = r#""animations": {"x": "geometry.nope", "walk": "Animation.Ghost.Walk", "move": "controller.animation.ghost.move", "short": "animation"}"#;
    let document = loaded(attachable(&ghost("1.10.0", members), &cx()));
    silent(document.diagnostics());
    assert_eq!(document.animation_kind("x"), None);
    assert_eq!(
        document.animation_kind("walk"),
        Some(AnimationKind::Animation)
    );
    assert_eq!(
        document.animation_kind("move"),
        Some(AnimationKind::Controller)
    );
    // `animation` without the dot is no animation; an unknown name is nothing.
    assert_eq!(document.animation_kind("short"), None);
    assert_eq!(document.animation_kind("absent"), None);
}

#[test]
fn l10_scale_case_collapse() {
    let members = r#""scripts": {"scalex": "2.0", "scaleX": "3.0"}"#;
    let document = loaded(attachable(&ghost("1.10.0", members), &cx()));
    let scripts = document.description.scripts.as_ref().unwrap();
    assert_eq!(scripts.effective_scale_x().unwrap().as_str(), "2.0");
    // Both members are kept.
    assert!(scripts.scale_x.is_some() && scripts.scalex.is_some());
}

#[test]
fn l12_render_distance_unknown() {
    for version in ["1.8.0", "1.10.0", "1.26.0"] {
        let d = rejected(attachable(
            &ghost(version, r#""render_distance": 32"#),
            &cx(),
        ));
        expect(&d, NOT_VALID, Severity::Error, &child("render_distance"));
    }
}

#[test]
fn rule_6a_an_undeclared_root_or_payload_member_fails_the_file() {
    let text = r#"{"format_version": "1.10.0", "minecraft:attachable": {"description": {"identifier": "demo:ghost"}}, "extra": 1}"#;
    let d = rejected(attachable(text, &cx()));
    expect(&d, NOT_VALID, Severity::Error, &child("extra"));
    let text = r#"{"format_version": "1.10.0", "minecraft:attachable": {"description": {"identifier": "demo:ghost"}, "components": {}}}"#;
    let d = rejected(attachable(text, &cx()));
    expect(&d, NOT_VALID, Severity::Error, &child("components"));
    expect_at(
        &d,
        NOT_VALID,
        Severity::Error,
        "minecraft:attachable/components",
    );
}

#[test]
fn l13_particle_emitters_kept() {
    let members = r#""particle_emitters": {"smoke": "demo:smoke"}"#;
    for version in ["1.8.0", "1.10.0", "1.26.0"] {
        let document = loaded(attachable(&ghost(version, members), &cx()));
        silent(document.diagnostics());
        let emitters = document.description.particle_emitters.as_ref().unwrap();
        assert_eq!(emitters["smoke"], "demo:smoke");
    }
}

#[test]
fn l16_required_pattern_empty() {
    for version in ["1.8.0", "1.10.0", "1.26.0"] {
        for map in ["materials", "textures", "geometry"] {
            let d = rejected(attachable(
                &ghost(version, &format!(r#""{map}": {{}}"#)),
                &cx(),
            ));
            expect(
                &d,
                "MissingRequiredField",
                Severity::Error,
                "Required child [a-zA-Z0-9_.-]+ not found",
            );
            expect_at(
                &d,
                "MissingRequiredField",
                Severity::Error,
                &format!("minecraft:attachable/description/{map}"),
            );
        }
        // The pattern child of the other maps is optional.
        let optional = r#""animations": {}, "sound_effects": {}, "particle_effects": {}, "particle_emitters": {}"#;
        silent(loaded(attachable(&ghost(version, optional), &cx())).diagnostics());
        // An empty `variables` lacks its required pattern child too.
        if version != "1.8.0" {
            let d = rejected(attachable(
                &ghost(version, r#""scripts": {"variables": {}}"#),
                &cx(),
            ));
            expect(
                &d,
                "MissingRequiredField",
                Severity::Error,
                "Required child variable.[a-zA-Z_][a-zA-Z0-9_]* not found",
            );
        }
    }
}

#[test]
fn l17_missing_identifier_schema() {
    for version in ["1.8.0", "1.10.0", "1.26.0"] {
        let d = rejected(attachable(&doc(version, "{}"), &cx()));
        expect(
            &d,
            "MissingRequiredField",
            Severity::Error,
            "Required child identifier not found",
        );
        // Reported at the object that lacks it.
        expect_at(
            &d,
            "MissingRequiredField",
            Severity::Error,
            "minecraft:attachable/description",
        );
        // A number is no identifier.
        let d = rejected(attachable(&doc(version, r#"{"identifier": 5}"#), &cx()));
        expect(
            &d,
            "InvalidType",
            Severity::Error,
            &format!("{TYPE_ERROR}string"),
        );
    }
}

#[test]
fn l18_key_pattern_full_match() {
    for version in ["1.8.0", "1.10.0", "1.26.0"] {
        for map in [
            "materials",
            "textures",
            "geometry",
            "animations",
            "sound_effects",
            "particle_effects",
            "particle_emitters",
        ] {
            let members = format!(r#""{map}": {{"my:tex": "textures/x"}}"#);
            let d = rejected(attachable(&ghost(version, &members), &cx()));
            expect(&d, NOT_VALID, Severity::Error, &child("my:tex"));
            expect_at(
                &d,
                NOT_VALID,
                Severity::Error,
                &format!("minecraft:attachable/description/{map}/my:tex"),
            );
        }
    }
    // A conditional object's key may hold a colon, but no space.
    let ok = r#""render_controllers": [{"demo:controller.render.a": "1"}]"#;
    loaded(attachable(&ghost("1.10.0", ok), &cx()));
    let bad = r#""render_controllers": [{"controller render": "1"}]"#;
    let d = rejected(attachable(&ghost("1.10.0", bad), &cx()));
    expect(&d, NOT_VALID, Severity::Error, &child("controller render"));
    let bad = r#""scripts": {"animate": [{"a b": "1"}]}"#;
    let d = rejected(attachable(&ghost("1.26.0", bad), &cx()));
    expect(&d, NOT_VALID, Severity::Error, &child("a b"));
    // The elements of the 1.8 controller list take the slot pattern.
    let bad = r#""animations": {"a": "animation.a"}, "animation_controllers": [{"my:move": "controller.animation.x"}]"#;
    let d = rejected(attachable(&ghost("1.8.0", bad), &cx()));
    expect(&d, NOT_VALID, Severity::Error, &child("my:move"));
}

#[test]
fn l19_queryable_geometry_1_8() {
    let members = r#""queryable_geometry": "default""#;
    let d = rejected(attachable(&ghost("1.8.0", members), &cx()));
    expect(&d, NOT_VALID, Severity::Error, &child("queryable_geometry"));
    for version in ["1.10.0", "1.26.0"] {
        let document = loaded(attachable(&ghost(version, members), &cx()));
        assert_eq!(
            document.description.queryable_geometry.as_ref().unwrap(),
            "default"
        );
    }
}

#[test]
fn l20_variables_not_public() {
    for version in ["1.10.0", "1.26.0"] {
        let members = r#""scripts": {"variables": {"variable.foo": "Public"}}"#;
        let d = rejected(attachable(&ghost(version, members), &cx()));
        expect(
            &d,
            "InvalidValue",
            Severity::Error,
            "Value 'Public' not allowed.  Must match at least one of the following regex patterns: public",
        );
        let members = r#""scripts": {"variables": {"variable.foo": "public"}}"#;
        silent(loaded(attachable(&ghost(version, members), &cx())).diagnostics());
        // The key must be a variable name.
        let members = r#""scripts": {"variables": {"foo": "public"}}"#;
        let d = rejected(attachable(&ghost(version, members), &cx()));
        expect(&d, NOT_VALID, Severity::Error, &child("foo"));
    }
    // 1.8 has no `variables`.
    let members = r#""scripts": {"variables": {"variable.foo": "public"}}"#;
    let d = rejected(attachable(&ghost("1.8.0", members), &cx()));
    expect(&d, NOT_VALID, Severity::Error, &child("variables"));
}

#[test]
fn l21_empty_conditional_object_skipped_vanilla_deviation() {
    // Vanilla: an empty {} render_controllers / animate element passes the schema and is
    // read through begin() of an empty object (undefined). Ours: the file loads; the entry is held where the
    // file has it, names nothing, and is told once, at the entry. A warning: an error would
    // reject the document, as for the other rules of that section.
    let members = r#""render_controllers": ["controller.render.a", {}, {"controller.render.b": "q.is_baby"}]"#;
    for version in ["1.10.0", "1.26.0"] {
        let document = loaded(attachable(&ghost(version, members), &cx()));
        let controllers = document.visuals().render_controllers;
        let drawn: Vec<Option<String>> = controllers
            .iter()
            .map(|entry| entry.controller().map(|id| id.as_str().to_owned()))
            .collect();
        assert_eq!(
            drawn,
            [
                Some("controller.render.a".to_owned()),
                None,
                Some("controller.render.b".to_owned())
            ]
        );
        assert!(matches!(&controllers[1], ControllerUse::Conditional(m) if m.is_empty()));
        assert!(controllers[1].condition().is_none());
        let at = "minecraft:attachable/description/render_controllers/1";
        only_info_at(document.diagnostics(), "InvalidValue", at);
        // The written text holds the entry, and reads back to the same value with the same
        // warning: a fixed point.
        let text = round_trip(&document, |text| attachable(text, &cx()));
        only_info_at(
            loaded(attachable(&text, &cx())).diagnostics(),
            "InvalidValue",
            at,
        );
    }
    // Alone in its list.
    let document = loaded(attachable(
        &ghost("1.10.0", r#""render_controllers": [{}]"#),
        &cx(),
    ));
    assert_eq!(document.visuals().render_controllers.len(), 1);
    only_info_at(
        document.diagnostics(),
        "InvalidValue",
        "minecraft:attachable/description/render_controllers/0",
    );
    let members = r#""animations": {"a": "animation.a", "b": "animation.b"}, "scripts": {"animate": ["a", {}, {"b": "q.is_baby"}, {}]}"#;
    for version in ["1.10.0", "1.26.0"] {
        let document = loaded(attachable(&ghost(version, members), &cx()));
        let scripts = document.description.scripts.as_ref().unwrap();
        let played: Vec<Option<&str>> = scripts
            .animate
            .iter()
            .flatten()
            .map(|entry| entry.name())
            .collect();
        assert_eq!(played, [Some("a"), None, Some("b"), None]);
        let d = document.diagnostics();
        for index in [1, 3] {
            expect_at(
                d,
                "InvalidValue",
                Severity::Info,
                &format!("minecraft:attachable/description/scripts/animate/{index}"),
            );
        }
        let told = d.at_least(Severity::Warning).count();
        assert_eq!(told, 0, "{}", lines(d).join("\n"));
        let text = round_trip(&document, |text| attachable(text, &cx()));
        let again = loaded(attachable(&text, &cx()));
        assert_eq!(again.diagnostics().at_least(Severity::Warning).count(), 0);
    }
    // What is told of an entry behind an empty one is at the index the file has.
    let members = r#""scripts": {"animate": [{}, "missing"]}"#;
    let document = loaded(attachable(&ghost("1.26.0", members), &cx()));
    expect_at(
        document.diagnostics(),
        "UnresolvedReference",
        Severity::Info,
        "minecraft:attachable/description/scripts/animate/1",
    );
}

#[test]
fn l22_hide_held_items_engine_gate() {
    let members = r#""scripts": {"hide_held_items": "variable.x"}"#;
    // The engine this crate tracks declares the member.
    assert!(ENGINE_VERSION >= FormatVersion::new(1, 26, 30));
    let document = loaded(attachable(&ghost("1.26.0", members), &cx()));
    let scripts = document.description.scripts.as_ref().unwrap();
    assert!(scripts.hide_held_items.is_some());
    // 1.26.30 is the first engine that does; 1.26.20 does not.
    let mut first = cx();
    first.engine_version = FormatVersion::new(1, 26, 30);
    loaded(attachable(&ghost("1.26.0", members), &first));
    let mut older = cx();
    older.engine_version = FormatVersion::new(1, 26, 20);
    let d = rejected(attachable(&ghost("1.26.0", members), &older));
    expect(&d, NOT_VALID, Severity::Error, &child("hide_held_items"));
    // Below format 1.26.0 the member is undeclared at every engine.
    for cx in [cx(), older] {
        let d = rejected(attachable(&ghost("1.10.0", members), &cx));
        expect(&d, NOT_VALID, Severity::Error, &child("hide_held_items"));
    }
}

#[test]
fn l23_spawn_egg_defaults() {
    let document = loaded(attachable(&ghost("1.10.0", r#""spawn_egg": {}"#), &cx()));
    let egg = document.description.spawn_egg.as_ref().unwrap().resolved();
    assert_eq!(egg.texture, "");
    assert_eq!(egg.texture_index, 0);
    assert_eq!(egg.base_color, Color::NIL);
    assert_eq!(egg.overlay_color, Color::NIL);
    // An undeclared member of the egg fails the file.
    let d = rejected(attachable(
        &ghost("1.10.0", r#""spawn_egg": {"colour": "red"}"#),
        &cx(),
    ));
    expect(&d, NOT_VALID, Severity::Error, &child("colour"));
}

#[test]
fn l15_spawn_egg_colour() {
    let members = r##""spawn_egg": {"base_color": "#ff0000", "overlay_color": "red"}"##;
    let document = loaded(attachable(&ghost("1.10.0", members), &cx()));
    silent(document.diagnostics());
    let egg = document.description.spawn_egg.as_ref().unwrap().resolved();
    assert_eq!(
        egg.base_color,
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 0.0
        }
    );
    assert_eq!(egg.overlay_color, Color::NIL);
}

#[test]
fn rule_6_wrong_json_types_fail_the_file() {
    for (members, allowed) in [
        (r#""enable_attachables": "true""#, "bool"),
        (r#""materials": {"default": 3}"#, "string"),
        (r#""spawn_egg": {"texture_index": "3"}"#, "int"),
        (r#""min_engine_version": 3"#, "string, array"),
        (r#""scripts": {"scale": [1]}"#, "molang"),
        (r#""render_controllers": [3]"#, "string, object"),
        (r#""geometry": []"#, "object"),
    ] {
        let d = rejected(attachable(&ghost("1.10.0", members), &cx()));
        expect(
            &d,
            "InvalidType",
            Severity::Error,
            &format!("{TYPE_ERROR}{allowed}"),
        );
    }
    // A string that is no version passes the schema and reads as 0.0.0.
    let document = loaded(attachable(
        &ghost("1.10.0", r#""min_engine_version": "soon""#),
        &cx(),
    ));
    silent(document.diagnostics());
    assert_eq!(
        document.min_engine_version(),
        bedrock_addon::version::SemVersion::new(0, 0, 0)
    );
}

#[test]
fn r6_an_animate_entry_that_names_no_animation_is_noted() {
    // String entries compare in lower case, object keys as written.
    let members = r#""animations": {"Walk": "animation.walk"}, "scripts": {"animate": ["WALK", {"walk": "1"}, "run", {"Walk": "1"}]}"#;
    let document = loaded(attachable(&ghost("1.26.0", members), &cx()));
    let unresolved: Vec<String> = document
        .diagnostics()
        .iter()
        .filter(|f| f.code.name() == "UnresolvedReference")
        .map(|f| {
            assert_eq!(f.severity, Severity::Info);
            f.location.json_path.to_string()
        })
        .collect();
    assert_eq!(
        unresolved,
        [
            "minecraft:attachable/description/scripts/animate/2",
            "minecraft:attachable/description/scripts/animate/3"
        ]
    );
}

#[test]
fn booleans_default_to_false_and_read_what_is_written() {
    let absent = loaded(attachable(&ghost("1.10.0", ""), &cx()));
    assert!(!absent.enable_attachables());
    assert!(!absent.held_item_ignores_lighting());
    assert!(!absent.hide_armor());
    let members =
        r#""enable_attachables": true, "held_item_ignores_lighting": true, "hide_armor": true"#;
    let set = loaded(attachable(&ghost("1.10.0", members), &cx()));
    assert!(set.enable_attachables());
    assert!(set.held_item_ignores_lighting());
    assert!(set.hide_armor());
}

// ---- more rows ----

#[test]
fn l09_the_controller_prefix_is_all_21_characters() {
    // `controller.animation.` with its last dot, then `animation.`.
    let members = r#""animations": {"no_dot": "controller.animationx", "bare": "controller.animation", "exact": "controller.animation.", "plain": "animation.x", "plain_exact": "animation."}"#;
    let document = loaded(attachable(&ghost("1.10.0", members), &cx()));
    assert_eq!(document.animation_kind("no_dot"), None);
    assert_eq!(document.animation_kind("bare"), None);
    assert_eq!(
        document.animation_kind("exact"),
        Some(AnimationKind::Controller)
    );
    assert_eq!(
        document.animation_kind("plain"),
        Some(AnimationKind::Animation)
    );
    assert_eq!(
        document.animation_kind("plain_exact"),
        Some(AnimationKind::Animation)
    );
}

#[test]
fn a_held_item_scale_beyond_f32_is_read_as_the_largest_finite_float_deviation() {
    // vanilla's schema takes any number and the loader reads `asFloat`.
    for (written, expected) in [("1e39", f32::MAX), ("1e999", f32::MAX), ("-1e39", f32::MIN)] {
        let members = format!(r#""held_item_scale": {written}"#);
        let document = loaded(attachable(&ghost("1.26.0", &members), &cx()));
        assert_eq!(
            document.description.held_item_scale,
            Some(HeldItemScale::Number(expected)),
            "{written}"
        );
        assert!(
            document
                .diagnostics()
                .iter()
                .any(|f| f.code.name() == "InvalidValue"
                    && f.severity == Severity::Info
                    && f.location.json_path.to_string()
                        == "minecraft:attachable/description/held_item_scale"
                    && f.code.to_string() == "invalid value"),
            "{written}: {}",
            lines(document.diagnostics()).join("\n")
        );
        // What was read is written, and reads back without the finding.
        let text = bedrock_addon::document::Content::to_json(document.value()).unwrap();
        let again = loaded(attachable(&text, &cx()));
        assert_eq!(again.value(), document.value());
        assert!(
            again
                .diagnostics()
                .iter()
                .all(|f| f.code.to_string() != "invalid value"),
            "{text}"
        );
    }
    // The largest float itself is no deviation.
    let members = format!(r#""held_item_scale": {:?}"#, f32::MAX);
    let document = loaded(attachable(&ghost("1.26.0", &members), &cx()));
    silent(document.diagnostics());
}

#[test]
fn a_texture_index_that_is_no_integer_is_a_wrong_type() {
    // the schema's child is an int; a real token is not one.
    for value in ["1e39", "1.5"] {
        let members = format!(r#""spawn_egg": {{"texture_index": {value}}}"#);
        let d = rejected(attachable(&ghost("1.10.0", &members), &cx()));
        expect(
            &d,
            "InvalidType",
            Severity::Error,
            &format!("{TYPE_ERROR}int"),
        );
    }
}

// ---- the clamp of `held_item_scale` changes no other finding ----

const SCALE_AT: &str = "minecraft:attachable/description/held_item_scale";

fn entity_findings(text: &str) -> bedrock_addon::diagnostic::Diagnostics {
    findings(attachable(text, &cx()))
}

#[test]
fn a_clamped_scale_leaves_the_span_of_a_later_wrong_type_where_it_is() {
    let text = r#"{
  "format_version": "1.26.0",
  "minecraft:attachable": {
    "description": {
      "identifier": "demo:ghost",
      "held_item_scale": 1e39,
      "textures": {
        "default": "textures/ghost"
      },
      "hide_armor": 3
    }
  }
}"#;
    clamp_changes_no_other_finding(text, &[("1e39", "1e30", SCALE_AT)], entity_findings);
    // And the span is the member's value, in the file's own bytes.
    let found = entity_findings(text);
    let wrong = find(&found, "InvalidType", Severity::Error).unwrap();
    let span = wrong.location.span.unwrap();
    let covered = &text[span.offset as usize..][..span.len as usize];
    assert!(
        text[..span.offset as usize].ends_with("\"hide_armor\": ")
            || covered.contains("hide_armor"),
        "{covered:?}"
    );
}

#[test]
fn a_clamped_scale_leaves_the_path_of_a_later_wrong_type_where_it_is() {
    // An array is no Molang: the finding is `scripts.scale`'s, not the clamped member's. (A
    // number beyond `f32` there is clamped as the scale is: tests/attachable/numbers.rs.)
    let members = r#""held_item_scale": 1e39, "scripts": {"scale": []}"#;
    let text = ghost("1.26.0", members);
    clamp_changes_no_other_finding(&text, &[("1e39", "1e30", SCALE_AT)], entity_findings);
    let found = entity_findings(&text);
    expect_at(
        &found,
        "InvalidType",
        Severity::Error,
        "minecraft:attachable/description/scripts/scale",
    );
    expect(
        &found,
        "InvalidType",
        Severity::Error,
        "unknown child schema option type.  Allowed types: molang",
    );
}

#[test]
fn a_clamped_scale_after_a_comment_leaves_every_later_finding_where_it_is() {
    for (beyond, within) in [("1e39", "1e30"), ("-1e39", "-1e30"), ("1e999", "1e-09")] {
        // Marks the members that reject the document.
        const REJECTED: &str = "/*rejected*/";
        for later in [
            r#"/*rejected*/"spawn_egg": {"texture_index": 1.5}"#,
            r#"/*rejected*/"scripts": {"animate": ["walk", {"walk": []}]}"#,
            r#"/*rejected*/"render_controllers": ["a", 7]"#,
            r#"/*rejected*/"materials": {"default": {"x": 1}}"#,
            r#"/*rejected*/"unknown_member": 1"#,
            r#""scripts": {"pre_animation": ["1 +"], "animate": ["missing"]}"#,
            r#""hide_armor": true"#,
        ] {
            // A document that loads with a scale below zero names the value read in rule
            // 21's text, which is the clamped one: not the same text, by design.
            if beyond.starts_with('-') && !later.contains(REJECTED) {
                continue;
            }
            let later = later.replace(REJECTED, "");
            let text = format!(
                r#"{{"format_version": "1.26.0", "minecraft:attachable": {{
  /* a comment
     over two lines, with a number: 1e40 */
  "description": {{
    "identifier": "demo:ghost", // 1e41
    "spawn_egg": {{ "texture": "spawn_egg" }},
    "held_item_scale": /* here */ {beyond},
    {later}
  }}
}}}}"#
            );
            clamp_changes_no_other_finding(&text, &[(beyond, within, SCALE_AT)], entity_findings);
        }
    }
}

#[test]
fn a_scale_beyond_f32_is_not_clamped_where_the_member_does_not_exist() {
    // `held_item_scale` is a member of the 1.26.0 struct alone. In an older document
    // it is an undeclared member, whatever its value: one finding.
    for version in ["1.8.0", "1.10.0", "1.21.0"] {
        let d = rejected(attachable(
            &ghost(version, r#""held_item_scale": 1e39"#),
            &cx(),
        ));
        expect(
            &d,
            "UnrecognizedField",
            Severity::Error,
            "child 'held_item_scale' not valid here.",
        );
        assert_eq!(d.len(), 1, "{version}: {}", lines(&d).join("\n"));
    }
}

// ---- the clamped number is found by its member, not by its text ----

#[test]
fn a_clamped_scale_whose_text_occurs_earlier_is_found_at_its_member() {
    // The same characters in a string value, in a string key's neighbour, in comments and in
    // an earlier member: only the member's own token is the number.
    for token in ["1e999", "1e39", "-1e39"] {
        let text = format!(
            r#"{{"format_version": "1.26.0", /* {token} */ "minecraft:attachable": {{"description": {{
  // "held_item_scale": {token}
  "identifier": "demo:ghost",
  "min_engine_version": "{token}",
  "textures": {{"default": "textures/{token}", "{token}": "{token}"}},
  "particle_emitters": {{"a": "held_item_scale: {token}"}},
  "held_item_scale": {token},
  "queryable_geometry": "{token}"
}}}}}}"#
        );
        let member = "\"held_item_scale\": ";
        let at = text.rfind(member).unwrap() + member.len();
        assert!(text.matches(token).count() >= 8, "{text}");
        assert!(text.find(token).unwrap() < at);
        let document = loaded(attachable(&text, &cx()));
        clamp_warning_is_at(document.diagnostics(), SCALE_AT, at, token.len());
        let limit = if token.starts_with('-') {
            f32::MIN
        } else {
            f32::MAX
        };
        assert_eq!(
            document.description.held_item_scale,
            Some(bedrock_addon::visual::HeldItemScale::Number(limit))
        );
        // Every other occurrence is as written, in the value and in what is written back.
        let d = &document.description;
        assert_eq!(d.textures["default"].as_str(), format!("textures/{token}"));
        assert_eq!(d.textures[token].as_str(), token);
        let written = document.to_json().unwrap();
        for kept in [
            format!(r#""min_engine_version": "{token}""#),
            format!(r#""default": "textures/{token}""#),
            format!(r#""{token}": "{token}""#),
            format!(r#""a": "held_item_scale: {token}""#),
            format!(r#""queryable_geometry": "{token}""#),
        ] {
            assert!(written.contains(&kept), "{kept} in {written}");
        }
        assert_eq!(written.matches(token).count(), 6, "{written}");
        // And a wrong type after it is where it is in the same text with a number in range.
        let wrong = text.replace("\"queryable_geometry\": \"", "\"hide_armor\": \"");
        let in_range = wrong.replace(
            &format!("{member}{token}"),
            &format!("{member}{}", &"0.25000"[..token.len()]),
        );
        let spans = |text: &str| -> Vec<String> {
            findings(attachable(text, &cx()))
                .iter()
                .filter(|f| f.severity == Severity::Error)
                .map(|f| spanned(f, text))
                .collect()
        };
        assert_eq!(spans(&wrong), spans(&in_range));
        assert_eq!(spans(&wrong).len(), 1);
    }
}

#[test]
fn a_clamped_scale_with_comments_around_its_token_is_found() {
    // Whatever stands around the token, the token alone is the number.
    for (before, after) in [
        ("", " /* after */"),
        ("", " // after\n"),
        ("/* before */ ", " /* after */ "),
        ("\n\t", "\n\t"),
        ("", ""),
    ] {
        let text = format!(
            r#"{{"format_version": "1.26.0", "minecraft:attachable": {{"description": {{"identifier": "demo:ghost", "held_item_scale":{before}1e39{after}, "hide_armor": true}}}}}}"#
        );
        let document = loaded(attachable(&text, &cx()));
        clamp_warning_is_at(
            document.diagnostics(),
            SCALE_AT,
            text.find("1e39").unwrap(),
            4,
        );
        assert_eq!(
            document.description.held_item_scale,
            Some(bedrock_addon::visual::HeldItemScale::Number(f32::MAX))
        );
        // As the last member too.
        let last = text.replace(r#", "hide_armor": true"#, "");
        let document = loaded(attachable(&last, &cx()));
        clamp_warning_is_at(
            document.diagnostics(),
            SCALE_AT,
            last.find("1e39").unwrap(),
            4,
        );
    }
}

/// The kind of an `animations` entry is one fact, so one type: a linker that asks a client
/// entity and an attachable for it compares the answers without a conversion.
#[cfg(feature = "client_entity")]
#[test]
fn the_kind_of_an_animations_entry_is_one_type_for_entities_and_attachables() {
    use std::any::TypeId;
    assert_eq!(
        TypeId::of::<bedrock_addon::families::client_entity::AnimationKind>(),
        TypeId::of::<AnimationKind>(),
        "the entity and attachable families share one AnimationKind"
    );
    assert_eq!(
        TypeId::of::<bedrock_addon::visual::AnimationKind>(),
        TypeId::of::<AnimationKind>()
    );
}
