//! The control-creation rules run statically, the variable and
//! lint fixture rows, and the typed views of a resolved definition.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::json_ui::{
    AnimationType, BindingCondition, BindingType, ButtonMappingType, ControlKind, ControlType,
    ENGINE_VARIABLES, FontSize, InputModeCondition, MappingScope, Orientation, TextAlignment,
    TileDirection, Variables,
};
use bedrock_addon::json::Value;

use super::common::*;

/// The findings of a file with these definitions in namespace `a`: `(code, text, where)`.
/// Every one must be an information: the game logs these only when a screen builds the
/// control, and the crate's own lints are informations too.
fn lint(definitions: &str) -> Vec<(&'static str, String, String)> {
    let stack = one_file(&format!(r#"{{ "namespace": "a", {definitions} }}"#));
    stack
        .ui()
        .diagnostics()
        .map(|d| {
            assert_eq!(d.severity, Severity::Info, "{d}");
            (
                d.code.name(),
                d.code.to_string(),
                d.location.json_path.to_string(),
            )
        })
        .collect()
}

/// The one finding of `definitions`.
fn one(definitions: &str) -> (&'static str, String, String) {
    let mut found = lint(definitions);
    assert_eq!(found.len(), 1, "{definitions}: {found:?}");
    found.remove(0)
}

fn clean(definitions: &str) {
    let found = lint(definitions);
    assert!(found.is_empty(), "{definitions}: {found:?}");
}

/// `definitions` gives exactly this finding.
fn expect(definitions: &str, code: &str, text: &str, at: &str) {
    let found = one(definitions);
    assert_eq!(
        (found.0, found.1.as_str(), found.2.as_str()),
        (code, text, at),
        "{definitions}"
    );
}

// ---- variables ------------------------------------------------------------------------------

#[test]
fn v01_requires_truthiness() {
    let stack = check("v01_requires_truthiness");
    let k = control(&stack, "a", "k");
    assert_eq!(ENGINE_VARIABLES.len(), 88);
    let pocket = Variables::platform(&["$pocket_edition"]);
    assert_eq!(k.variables(&pocket).get("$x"), Some(&Value::from(1_i64)));
    let desktop = Variables::platform(&[]);
    assert_eq!(k.variables(&desktop).get("$x"), Some(&Value::from(2_i64)));
    // numeric and not 0, a non-empty string, a non-empty other value
    let truthy = |text: &str| Variables::default().is_truthy(&json(text));
    for (value, holds) in [
        ("1", true),
        ("0", false),
        ("0.5", true),
        (r#""x""#, true),
        (r#""""#, false),
        ("true", true),
        ("false", false),
        ("null", false),
        ("[1]", true),
        ("[]", false),
        (r#"{ "a": 1 }"#, true),
        ("{}", false),
    ] {
        assert_eq!(truthy(value), holds, "{value}");
    }
}

#[test]
fn v02_property_in_variables() {
    let stack = check("v02_property_in_variables");
    let k = control(&stack, "a", "k");
    let touch = Variables::platform(&["$touch"]);
    assert_eq!(k.value("offset", &touch), Some(json("[0, 5]")));
    assert_eq!(k.value("offset", &Variables::platform(&[])), None);
    // the definition's own member is read before the frame
    assert_eq!(k.value("type", &touch), Some(Value::from("panel")));
}

#[test]
fn a_frame_holds_own_variables_defaults_and_entries() {
    let stack = one_file(
        r#"{ "k": { "type": "panel", "$own": 1, "$fallback|default": 2, "$set|default": 3,
            "$copy": "$outer", "size": "$own",
            "variables": { "$from_object": 4 } } }"#,
    );
    let k = control(&stack, "_root", "k");
    let mut outer = Variables::default();
    outer.set("$set", 9_i64);
    outer.set("$outer", "o");
    let frame = k.variables(&outer);
    assert_eq!(frame.get("$own"), Some(&Value::from(1_i64)));
    assert_eq!(frame.get("$fallback"), Some(&Value::from(2_i64)));
    // a default does not replace a value the enclosing frames define
    assert_eq!(frame.get("$set"), None);
    assert_eq!(frame.get("$copy"), Some(&Value::from("o")));
    assert_eq!(frame.get("$from_object"), Some(&Value::from(4_i64)));
    assert_eq!(k.value("size", &outer), Some(Value::from(1_i64)));
}

// ---- structure ------------------------------------------------------------------------------

#[test]
fn l01_unknown_type() {
    let stack = check("l01_unknown_type");
    assert_eq!(control(&stack, "a", "k").kind(), ControlKind::Unknown);
    assert_eq!(ControlType::ALL.len(), 24);
    for (control_type, name) in ControlType::ALL {
        clean(&format!(r#""k": {{ "type": "{name}" }}"#));
        assert_eq!(ControlType::from_name(name), Some(*control_type));
    }
}

#[test]
fn a_type_is_reported_where_it_is_written() {
    // C1 once, at the base; the derived definition inherits the bad type silently
    expect(
        r#""bad": { "type": "nope" }, "k@a.bad": { "layer": 1 }"#,
        "InvalidEnumValue",
        "Type 'nope' not recognized for control: bad",
        "bad/type",
    );
    expect(
        r#""k": { "type": 5 }"#,
        "InvalidType",
        "JSON UI parse failure - expected a string.",
        "k/type",
    );
    // a variable is the game's to resolve
    clean(r#""k": { "type": "$kind" }"#);
}

#[test]
fn a_definition_without_type_or_base_is_no_finding() {
    // C2 fires only when a base did not resolve: a plain typeless definition is one others
    // inherit from
    clean(r#""mixin": { "size": [1, 1] }, "k@a.mixin": { "type": "panel" }"#);
}

#[test]
fn a_child_that_cannot_be_created_is_reported_with_its_parent() {
    // C4 follows C1 and C2 on a child
    let found = lint(
        r#""p": { "type": "panel", "controls": [ { "c": { "type": "nope" } }, { "d@a.missing": {} }, { "e@a.p": {} } ] }"#,
    );
    let texts: Vec<&str> = found.iter().map(|f| f.1.as_str()).collect();
    assert_eq!(
        texts,
        [
            "Type 'nope' not recognized for control: c",
            "UIControlFactory parent control \"a.p\" failed to create child control \"a.c\", did you add a new type? Did you add it to the string-to-type function?",
            "Type not specified (or @-base not found) for control: d",
            "UIControlFactory parent control \"a.p\" failed to create child control \"a.d\", did you add a new type? Did you add it to the string-to-type function?",
        ]
    );
    assert_eq!(found[0].2, "p/controls/0/c/type");
    assert_eq!(found[2].2, "p/controls/1/d@a.missing");
    // a child whose base carries a variable is the game's to resolve
    clean(r#""p": { "type": "panel", "controls": [ { "c@$some.thing": {} } ] }"#);
    // a child is checked with the type it inherits
    expect(
        r#""leaf": { "type": "label", "text": "t" }, "p": { "type": "panel", "controls": [ { "c@a.leaf": { "button_mappings": [] } } ] }"#,
        "UnusedProperty",
        "property not read by this control type",
        "p/controls/0/c@a.leaf/button_mappings",
    );
}

#[test]
fn creation_only_texts_are_never_raised() {
    // C3, C5 and C9 have no static condition: an internal fallback, a root that a screen
    // asks for, and a cycler without the component the factory always adds
    let found = lint(
        r#""a": { "type": "nope" }, "b": { "type": "image_cycler", "images": [] }, "c": { "type": "label_cycler", "text_labels": [] }"#,
    );
    assert_eq!(found.len(), 1);
    for (_, text, _) in &found {
        assert!(!text.starts_with("Unable to create control"));
        assert!(!text.contains("failed to create root control"));
        assert!(!text.contains("requires page indicator component"));
    }
}

#[test]
fn ignored_takes_a_bool_an_integer_or_a_binding() {
    for value in [
        "true",
        "0",
        "1",
        r##""#bound""##,
        r#""$var""#,
        r#""(not $x)""#,
    ] {
        clean(&format!(
            r#""k": {{ "type": "panel", "ignored": {value} }}"#
        ));
    }
    for value in ["0.5", r#""yes""#, "[]", "{}"] {
        expect(
            &format!(r#""k": {{ "type": "panel", "ignored": {value} }}"#),
            "InvalidType",
            "An invalid value was provided for \"ignored\" field.",
            "k/ignored",
        );
    }
}

#[test]
fn an_anim_reference_must_name_a_definition() {
    let anims = |entries: &str| {
        format!(
            r#""fade": {{ "anim_type": "alpha", "from": 0, "to": 1 }}, "k": {{ "type": "panel", "anims": {entries} }}"#
        )
    };
    clean(&anims(
        r#"[ "@a.fade", "a.fade", "fade", "@fade", "$dynamic" ]"#,
    ));
    let found = one(&anims(r#"[ "@a.fade", "@a.gone" ]"#));
    assert_eq!(found.0, "UnresolvedReference");
    assert_eq!(found.1, "JSON UI parse failure: Invalid anim reference!");
    assert_eq!(found.2, "k/anims/1");
    assert_eq!(one(&anims("[ 5 ]")).0, "UnresolvedReference");
    assert_eq!(
        one(&anims(r#""@a.fade""#)).1,
        "JSON UI parse failure - expected an array."
    );
}

#[test]
fn a_grid_item_template_must_name_a_definition() {
    clean(
        r#""item": { "type": "panel" }, "k": { "type": "grid", "grid_item_template": "a.item" }"#,
    );
    clean(r#""k": { "type": "grid", "grid_item_template": "$template" }"#);
    expect(
        r#""k": { "type": "grid", "grid_item_template": "a.gone" }"#,
        "UnresolvedReference",
        "gridItemTemplate control tree could not be created! Control: k",
        "k/grid_item_template",
    );
}

// ---- bindings -------------------------------------------------------------------------------

#[test]
fn l04_binding_rules() {
    check("l04_binding_rules");
}

#[test]
fn each_binding_type_requires_its_own_members() {
    let binding =
        |members: &str| format!(r#""k": {{ "type": "panel", "bindings": [ {{ {members} }} ] }}"#);
    // global (also the default) and collection need a name; an empty one is none
    for members in [
        r#""binding_type": "global""#,
        r#""binding_type": "collection", "binding_collection_name": "c""#,
        r#""binding_name": """#,
    ] {
        assert_eq!(
            one(&binding(members)).1,
            "JSON UI parse failure: Must define a binding name!",
            "{members}"
        );
    }
    // a collection binding without both gets both
    assert_eq!(lint(&binding(r#""binding_type": "collection""#)).len(), 2);
    clean(&binding(
        r##""binding_type": "collection", "binding_name": "#n", "binding_collection_name": "c""##,
    ));
    // a view binding needs neither a name nor a collection
    clean(&binding(
        r##""binding_type": "view", "source_property_name": "#s", "target_property_name": "#t""##,
    ));
    assert_eq!(lint(&binding(r#""binding_type": "view""#)).len(), 2);
    // one scope flag alone is fine; the rule is the view binding's
    for flags in [
        r#""resolve_ancestor_scope": true, "resolve_sibling_scope": false"#,
        r#""resolve_sibling_scope": true"#,
    ] {
        clean(&binding(&format!(
            r##""binding_type": "view", "source_property_name": "#s", "target_property_name": "#t", {flags}"##
        )));
    }
    clean(&binding(
        r##""binding_name": "#n", "resolve_ancestor_scope": true, "resolve_sibling_scope": true"##,
    ));
    // `none` adds no binding, an ignored entry is skipped, and so is one that is no object
    clean(&binding(r#""binding_type": "none""#));
    clean(&binding(r#""ignored": true"#));
    clean(r#""k": { "type": "panel", "bindings": [ 5, "x", null ] }"#);
}

#[test]
fn l09_binding_defaults() {
    let stack = check("l09_binding_defaults");
    let bindings = control(&stack, "a", "k").bindings();
    assert_eq!(bindings.len(), 3);
    assert_eq!(bindings[0].binding_type, BindingType::Global);
    assert_eq!(bindings[0].condition, BindingCondition::None);
    assert_eq!(bindings[0].name.as_deref(), Some("#x"));
    assert_eq!(bindings[1].binding_type, BindingType::CollectionDetails);
    // an unknown type is treated as `global`
    assert_eq!(bindings[2].binding_type, BindingType::Global);
}

#[test]
fn an_unknown_or_non_string_binding_enum_falls_back_with_a_message() {
    let binding = |members: &str| {
        format!(
            r##""k": {{ "type": "panel", "bindings": [ {{ "binding_name": "#n", {members} }} ] }}"##
        )
    };
    expect(
        &binding(r#""binding_condition": "sometimes""#),
        "InvalidEnumValue",
        "Unknown binding condition value: sometimes",
        "k/bindings/0/binding_condition",
    );
    for key in ["binding_type", "binding_condition"] {
        expect(
            &binding(&format!(r#""{key}": 5"#)),
            "InvalidType",
            "JSON UI parse failure - expected a string.",
            &format!("k/bindings/0/{key}"),
        );
    }
    for (condition, _) in [
        ("none", 0),
        ("once", 1),
        ("always", 2),
        ("always_when_visible", 3),
        ("visible", 4),
        ("visibility_changed", 5),
    ] {
        clean(&binding(&format!(r#""binding_condition": "{condition}""#)));
    }
    let stack = one_file(
        r##"{ "k": { "type": "panel", "bindings": [ { "binding_name": "#n", "binding_condition": "always_when_visible" } ] } }"##,
    );
    assert_eq!(
        control(&stack, "_root", "k").bindings()[0].condition,
        BindingCondition::AlwaysWhenVisible
    );
}

#[test]
fn l16_expression_operators_deviation() {
    // vanilla has no such operators and no message for them
    check("l16_expression_operators");
}

#[test]
fn an_expression_reports_open_quotes_and_bare_hashes() {
    let binding = |name: &str| {
        format!(r#""k": {{ "type": "panel", "bindings": [ {{ "binding_name": "{name}" }} ] }}"#)
    };
    expect(
        &binding("(#a = 'x)"),
        "InvalidValue",
        "You didn't close your quotes >:( \n",
        "k/bindings/0/binding_name",
    );
    expect(
        &binding("(# = 1)"),
        "InvalidValue",
        "The # sign cannot be used without providing quotes around it!",
        "k/bindings/0/binding_name",
    );
    clean(&binding("('%.3s' * #a)"));
    // `source_property_name` and `requires` are expressions too
    assert_eq!(
        one(r##""k": { "type": "panel", "bindings": [ { "binding_type": "view", "source_property_name": "(#a <= 1)", "target_property_name": "#t" } ] }"##).0,
        "UnsupportedOperator"
    );
    assert_eq!(
        one(r#""k": { "type": "panel", "variables": [ { "requires": "($a >= 1)", "$x": 1 } ] }"#).2,
        "k/variables/0/requires"
    );
    assert_eq!(
        one(r#""k": { "type": "panel", "variables": { "requires": "($a % 2)", "$x": 1 } }"#).0,
        "UnsupportedOperator"
    );
    // a string that is no expression is not tokenized
    clean(&binding("#a >= 2"));
}

// ---- factory, focus, input, misc --------------------------------------------------------------

#[test]
fn a_factory_needs_a_name_and_variable_names() {
    let factory =
        |members: &str| format!(r#""k": {{ "type": "panel", "factory": {{ {members} }} }}"#);
    clean(&factory(
        r#""name": "f", "control_name": "a.k", "factory_variables": [ "$a", "$b" ], "insert_location": "front""#,
    ));
    for members in [r#""control_name": "a.k""#, r#""name": """#] {
        expect(
            &factory(members),
            "MissingRequiredField",
            "JSON UI parse failure: The factory name should not be empty!",
            "k/factory/name",
        );
    }
    for variables in [r#"[ "$a", "b" ]"#, r#"[ 5 ]"#, r#""$a""#] {
        expect(
            &factory(&format!(r#""name": "f", "factory_variables": {variables}"#)),
            "InvalidValue",
            "JSON UI parse failure: factory_variables should all be strings of json variables created higher up the tree.",
            "k/factory/factory_variables",
        );
    }
    expect(
        r#""k": { "type": "panel", "factory": [] }"#,
        "InvalidType",
        "JSON UI parse failure - expected an object.",
        "k/factory",
    );
}

#[test]
fn a_focus_mapping_is_an_array_of_identified_overrides() {
    let mapping = |value: &str| format!(r#""k": {{ "type": "button", "focus_mapping": {value} }}"#);
    clean(&mapping(
        r#"[ { "focus_identifier": "a", "focus_change_up": "b" } ]"#,
    ));
    expect(
        &mapping("{}"),
        "InvalidType",
        "JSON UI parse failure: Expected an array for property 'focus_mapping'.",
        "k/focus_mapping",
    );
    expect(
        &mapping("[ 5 ]"),
        "InvalidType",
        "JSON UI parse failure: Expected an object inside array for property 'focus_mapping'.",
        "k/focus_mapping/0",
    );
    expect(
        &mapping(r#"[ { "focus_change_down": "b" } ]"#),
        "MissingRequiredField",
        "JSON UI parse failure: Expected a focus identifier.",
        "k/focus_mapping/0/focus_identifier",
    );
    expect(
        &mapping(r#"[ { "focus_identifier": "a" } ]"#),
        "MissingRequiredField",
        "JSON UI parse failure: Expected a focus direction override.",
        "k/focus_mapping/0",
    );
    for (key, direction) in [
        ("up", "Up"),
        ("down", "Down"),
        ("left", "Left"),
        ("right", "Right"),
    ] {
        // each direction alone satisfies the override rule
        clean(&mapping(&format!(
            r#"[ {{ "focus_identifier": "a", "focus_change_{key}": "b" }} ]"#
        )));
        expect(
            &mapping(&format!(
                r#"[ {{ "focus_identifier": "a", "focus_change_{key}": "" }} ]"#
            )),
            "InvalidValue",
            &format!(
                "JSON UI parse failure: {direction} Override set to blank string. This is redundant as blank strings imply no override."
            ),
            &format!("k/focus_mapping/0/focus_change_{key}"),
        );
    }
}

#[test]
fn l13_mapping_type_absent() {
    let stack = check("l13_mapping_type_absent");
    let mapping = &control(&stack, "a", "k").button_mappings()[0];
    assert_eq!(mapping.mapping_type, ButtonMappingType::Pressed);
    assert_eq!(mapping.from_button_id, "button.menu_select");
    assert_eq!(mapping.to_button_id, "button.x");
}

#[test]
fn only_the_four_mapping_types_are_known() {
    let mapping = |members: &str| {
        format!(
            r#""k": {{ "type": "button", "button_mappings": [ {{ "from_button_id": "a", "to_button_id": "b", {members} }} ] }}"#
        )
    };
    for mapping_type in ["global", "pressed", "focused", "double_pressed", "$dynamic"] {
        clean(&mapping(&format!(r#""mapping_type": "{mapping_type}""#)));
    }
    for (written, shown) in [(r#""released""#, "released"), (r#""""#, ""), ("5", "")] {
        expect(
            &mapping(&format!(r#""mapping_type": {written}"#)),
            "InvalidEnumValue",
            &format!("Unrecognized mapping type String: {shown}"),
            "k/button_mappings/0/mapping_type",
        );
    }
    let stack = one_file(
        r#"{ "k": { "type": "button", "button_mappings": [ { "mapping_type": "double_pressed" }, { "mapping_type": "focused" }, { "mapping_type": "global" } ] } }"#,
    );
    let types: Vec<_> = control(&stack, "_root", "k")
        .button_mappings()
        .iter()
        .map(|mapping| mapping.mapping_type)
        .collect();
    assert_eq!(
        types,
        [
            ButtonMappingType::DoublePressed,
            ButtonMappingType::Focused,
            ButtonMappingType::Global
        ]
    );
}

#[test]
fn l14_scope_and_input_mode_deviation() {
    // vanilla falls back silently for every string it does not compare
    let stack = check("l14_scope_and_input_mode");
    let mapping = &control(&stack, "a", "k").button_mappings()[0];
    assert_eq!(mapping.scope, MappingScope::Controller);
    assert_eq!(mapping.input_mode_condition, InputModeCondition::None);
}

#[test]
fn the_compared_scopes_and_input_modes_are_read() {
    let stack = one_file(
        r#"{ "k": { "type": "button", "button_mappings": [
            { "mapping_type": "pressed", "scope": "view", "input_mode_condition": "gamepad" },
            { "mapping_type": "pressed", "scope": "global", "input_mode_condition": "not_gamepad" },
            { "mapping_type": "pressed", "scope": "screen" } ] } }"#,
    );
    let mappings = control(&stack, "_root", "k").button_mappings();
    assert_eq!(mappings[0].scope, MappingScope::View);
    assert_eq!(
        mappings[0].input_mode_condition,
        InputModeCondition::Gamepad
    );
    assert_eq!(mappings[1].scope, MappingScope::Global);
    assert_eq!(
        mappings[1].input_mode_condition,
        InputModeCondition::NotGamepad
    );
    assert_eq!(mappings[2].scope, MappingScope::Controller);
    // only the unknown scope is reported
    assert_eq!(
        findings(&stack),
        vec![(
            "InvalidEnumValue",
            Severity::Info,
            "invalid enum value".to_owned()
        )]
    );
}

#[test]
fn silent_fallbacks_are_reported_deviation() {
    // vanilla compares one or two strings and maps everything else to a fixed value
    for (definition, at) in [
        (
            r#""k": { "type": "custom", "renderer": "gradient_renderer", "gradient_direction": "diagonal" }"#,
            "k/gradient_direction",
        ),
        (
            r#""k": { "type": "custom", "renderer": "paper_doll_renderer", "rotation": "spin" }"#,
            "k/rotation",
        ),
        (
            r#""k": { "type": "panel", "factory": { "name": "f", "insert_location": "middle" } }"#,
            "k/factory/insert_location",
        ),
        (r#""k": { "anim_type": "spin" }"#, "k/anim_type"),
    ] {
        expect(definition, "InvalidEnumValue", "invalid enum value", at);
    }
    for definition in [
        r#""k": { "type": "custom", "renderer": "gradient_renderer", "gradient_direction": "horizontal" }"#,
        r#""k": { "type": "custom", "renderer": "paper_doll_renderer", "rotation": "custom_y" }"#,
        r#""k": { "type": "panel", "factory": { "name": "f", "insert_location": "back" } }"#,
    ] {
        clean(definition);
    }
}

#[test]
fn focus_navigation_modes_are_checked() {
    for mode in ["", "none", "stop", "contained", "custom"] {
        clean(&format!(
            r#""k": {{ "type": "panel", "focus_navigation_mode_up": "{mode}" }}"#
        ));
    }
    for direction in ["up", "down", "left", "right"] {
        expect(
            &format!(r#""k": {{ "type": "panel", "focus_navigation_mode_{direction}": "wrap" }}"#),
            "InvalidEnumValue",
            "Invalid Input for Focus Navigation",
            &format!("k/focus_navigation_mode_{direction}"),
        );
    }
}

#[test]
fn only_button_event_is_a_sound_event_type() {
    let sounds = |event: &str| {
        format!(
            r#""k": {{ "type": "button", "sounds": [ {{ "sound_name": "random.click"{event} }} ] }}"#
        )
    };
    clean(&sounds(""));
    clean(&sounds(r#", "event_type": "button_event""#));
    expect(
        &sounds(r#", "event_type": "hover""#),
        "InvalidEnumValue",
        "Unrecognized sound event type.\n",
        "k/sounds/0/event_type",
    );
}

#[test]
fn a_renderer_must_be_one_the_game_has() {
    clean(r#""k": { "type": "custom", "renderer": "hud_player_renderer" }"#);
    clean(r#""k": { "type": "custom", "renderer": "$renderer" }"#);
    expect(
        r#""k": { "type": "custom", "renderer": "my_renderer" }"#,
        "InvalidEnumValue",
        "Unrecognized renderer: 'my_renderer'",
        "k/renderer",
    );
}

#[test]
fn the_editor_gizmo_renderer_is_accepted_deviation() {
    // the Android 1.26.20 binary logs "Unrecognized renderer"; vanilla's own hud_screen.json
    // uses it and the Windows retail build has it
    clean(r#""k": { "type": "custom", "renderer": "editor_gizmo_renderer" }"#);
}

#[test]
fn l15_debug_colours() {
    check("l15_debug_colours");
    for colour in ["red", "gray", "blue", "green", "yellow", "purple"] {
        clean(&format!(
            r#""k": {{ "type": "panel", "debug": "{colour}" }}"#
        ));
    }
}

#[test]
fn arrays_with_their_own_texts() {
    expect(
        r#""k": { "type": "selection_wheel", "state_controls": {} }"#,
        "InvalidType",
        "JSON UI parse failure: Must define state_controls as an array!",
        "k/state_controls",
    );
    clean(r#""k": { "type": "selection_wheel", "state_controls": [ { "control_name": "c" } ] }"#);
    expect(
        r#""k": { "type": "image_cycler", "images": {} }"#,
        "InvalidType",
        "JSON UI parse failure: Must define 'images' as an array!",
        "k/images",
    );
    clean(r#""k": { "type": "image_cycler", "images": [ { "texture_path": "t" } ] }"#);
}

#[test]
fn font_types_are_strings() {
    clean(
        r#""k": { "type": "label", "text": "t", "font_type": "MinecraftTen", "backup_font_type": "UIFont" }"#,
    );
    expect(
        r#""k": { "type": "label", "text": "t", "font_type": 5 }"#,
        "InvalidType",
        "JSON UI parse failure: Expected a string for 'font_type'.",
        "k/font_type",
    );
    expect(
        r#""k": { "type": "label", "text": "t", "backup_font_type": [] }"#,
        "InvalidType",
        "JSON UI parse failure: Expected a string for 'backup_font_type'.",
        "k/backup_font_type",
    );
}

// ---- value types and enums ------------------------------------------------------------------

#[test]
fn each_getter_reports_its_type() {
    for (member, text) in [
        (r#""text": 5"#, "JSON UI parse failure - expected a string."),
        (
            r#""shadow": "yes""#,
            "JSON UI parse failure - expected a bool.",
        ),
        (
            r#""layer": "top""#,
            "JSON UI parse failure - expected a number.",
        ),
        (
            r#""layer": true"#,
            "JSON UI parse failure - expected a number.",
        ),
        (
            r#""notify_on_ellipses": "x""#,
            "JSON UI parse failure - expected an array.",
        ),
        (
            r#""color": 5"#,
            "JSON UI parse failure - expected an array or a string.",
        ),
        (
            r#""property_bag": []"#,
            "JSON UI parse failure - expected an object.",
        ),
        (
            r#""anchor_from": 4"#,
            "JSON UI parse failure - expected a string.",
        ),
        (r#""size": 5"#, "JSON UI parse failure - expected an array."),
    ] {
        let found = one(&format!(r#""k": {{ "type": "label", {member} }}"#));
        assert_eq!(
            (found.0, found.1.as_str()),
            ("InvalidType", text),
            "{member}"
        );
    }
    expect(
        r#""k": { "type": "image", "nineslice_size": "4" }"#,
        "InvalidType",
        "JSON UI parse failure - expected an array or a number.",
        "k/nineslice_size",
    );
    // the right types, and values the game resolves itself
    clean(
        r##""k": { "type": "label", "text": "t", "shadow": true, "layer": 2, "color": [1, 1, 1], "locked_color": "red",
            "notify_on_ellipses": [], "property_bag": {}, "visible": "#bound", "enabled": "$var", "alpha": "(1 - $x)",
            "line_padding": { "__string": true, "value": "2" } }"##,
    );
    clean(
        r#""k": { "type": "image", "nineslice_size": 4 }, "j": { "type": "image", "nineslice_size": [1, 2, 3, 4] }"#,
    );
    // a typeless definition is checked by the key alone
    assert_eq!(
        one(r#""mixin": { "layer": "top" }"#).1,
        "JSON UI parse failure - expected a number."
    );
}

#[test]
fn l02_bad_enum() {
    check("l02_bad_enum");
    for anchor in [
        "top_left",
        "top_middle",
        "top_right",
        "left_middle",
        "center",
        "right_middle",
        "bottom_left",
        "bottom_middle",
        "bottom_right",
    ] {
        clean(&format!(
            r#""k": {{ "type": "panel", "anchor_to": "{anchor}" }}"#
        ));
    }
}

#[test]
fn each_enum_miss_has_its_text() {
    for (definition, text) in [
        (
            r#""k": { "type": "panel", "anchor_to": "middle" }"#,
            "Unrecognized anchor or alignment value: 'middle'",
        ),
        (
            r#""k": { "type": "label", "text": "t", "text_alignment": "justify" }"#,
            "JSON UI parse failure - Text Alignment value invalid.",
        ),
        (
            r#""k": { "type": "stack_panel", "orientation": "diagonal" }"#,
            "Unknown Orientation value: diagonal",
        ),
        (
            r#""k": { "type": "grid", "grid_rescaling_type": "both" }"#,
            "Unknown Orientation value: both",
        ),
        (
            r#""k": { "type": "slider", "slider_direction": "up" }"#,
            "Unknown Orientation value: up",
        ),
        (
            r#""k": { "type": "panel", "draggable": "yes" }"#,
            "Unknown draggable value: yes",
        ),
        (
            r#""k": { "type": "panel", "priority_rule": "always" }"#,
            "Unknown priority rule value: always",
        ),
        (
            r#""k": { "type": "panel", "toggle_manager_behavior": "toggle" }"#,
            "Unknown toggle manager behavior type value: toggle",
        ),
        // the slider manager logs the toggle manager's text
        (
            r#""k": { "type": "panel", "slider_manager_behavior": "select" }"#,
            "Unknown toggle manager behavior type value: select",
        ),
        (
            r#""k": { "anim_type": "flip_book", "initial_uv": [0, 0], "orientation": "up" }"#,
            "Unknown Orientation value: up",
        ),
    ] {
        let found = one(definition);
        assert_eq!(
            (found.0, found.1.as_str()),
            ("InvalidEnumValue", text),
            "{definition}"
        );
    }
    for definition in [
        r#""k": { "type": "label", "text": "t", "text_alignment": "center" }"#,
        r#""k": { "type": "stack_panel", "orientation": "none" }"#,
        r#""k": { "type": "panel", "draggable": "both", "priority_rule": "show_when_controls_are_hidden" }"#,
        r#""k": { "type": "panel", "toggle_manager_behavior": "clear", "slider_manager_behavior": "gather" }"#,
    ] {
        clean(definition);
    }
}

#[test]
fn l03_font_size_medium() {
    let stack = check("l03_font_size_medium");
    assert_eq!(control(&stack, "a", "k").font_size(), FontSize::Normal);
}

#[test]
fn enums_vanilla_ignores_silently_stay_silent() {
    clean(
        r#""k": { "type": "image", "texture": "t", "clip_direction": "sideways", "color": "mauve" }"#,
    );
    clean(r#""k": { "anim_type": "alpha", "from": 0, "to": 1, "easing": "wobbly" }"#);
}

#[test]
fn l11_text_defaults() {
    let stack = check("l11_text_defaults");
    let label = control(&stack, "a", "k");
    assert_eq!(label.font_size(), FontSize::Normal);
    assert!(label.line_padding().abs() < f64::EPSILON);
    assert_eq!(label.text_alignment(), TextAlignment::Left);
    assert_eq!(label.kind(), ControlKind::Control(ControlType::Label));
    let stack = one_file(
        r#"{ "k": { "type": "label", "text": "t", "font_size": "extra_large", "line_padding": 2.5, "text_alignment": "right" } }"#,
    );
    let label = control(&stack, "_root", "k");
    assert_eq!(label.font_size(), FontSize::ExtraLarge);
    assert!((label.line_padding() - 2.5).abs() < f64::EPSILON);
    assert_eq!(label.text_alignment(), TextAlignment::Right);
}

#[test]
fn l12_orientation_defaults() {
    let stack = check("l12_orientation_defaults");
    assert_eq!(
        control(&stack, "a", "stack").orientation(),
        Orientation::Vertical
    );
    assert_eq!(
        control(&stack, "a", "grid").grid_rescaling_type(),
        Orientation::None
    );
    assert_eq!(
        control(&stack, "a", "grid").grid_fill_direction(),
        Orientation::None
    );
    assert_eq!(
        control(&stack, "a", "slider").slider_direction(),
        Orientation::Horizontal
    );
    let book = control(&stack, "a", "book");
    assert_eq!(book.orientation(), Orientation::Horizontal);
    assert_eq!(book.kind(), ControlKind::Animation(AnimationType::FlipBook));
    let stack = one_file(
        r#"{ "s": { "type": "stack_panel", "orientation": "horizontal" }, "g": { "type": "grid", "grid_rescaling_type": "vertical" },
            "l": { "type": "slider", "slider_direction": "vertical" } }"#,
    );
    assert_eq!(
        control(&stack, "_root", "s").orientation(),
        Orientation::Horizontal
    );
    assert_eq!(
        control(&stack, "_root", "g").grid_rescaling_type(),
        Orientation::Vertical
    );
    assert_eq!(
        control(&stack, "_root", "l").slider_direction(),
        Orientation::Vertical
    );
}

#[test]
fn l18_animated_property_forms() {
    let stack = check("l18_animated_property_forms");
    let ui = stack.ui();
    let alpha = |name: &str| control(&stack, "a", name).initial_alpha(ui);
    assert!(alpha("c1").abs() < f64::EPSILON);
    assert!(alpha("c2").abs() < f64::EPSILON);
    assert!((alpha("c3") - 0.5).abs() < f64::EPSILON);
    assert!((alpha("c4") - 1.0).abs() < f64::EPSILON);
}

#[test]
fn l19_tiled_forms() {
    let stack = check("l19_tiled_forms");
    let tiled = |name: &str| control(&stack, "a", name).tiled();
    assert_eq!(tiled("t1"), TileDirection::Both);
    assert_eq!(tiled("t2"), TileDirection::X);
    assert_eq!(tiled("t3"), TileDirection::Both);
    assert_eq!(tiled("t4"), TileDirection::None);
    assert_eq!(tiled("t5"), TileDirection::None);
    let stack = one_file(
        r#"{ "y": { "type": "image", "tiled": "y" }, "xy": { "type": "image", "tiled": "xy" }, "off": { "type": "image", "tiled": false }, "none": { "type": "image" } }"#,
    );
    assert!(findings(&stack).is_empty());
    assert_eq!(control(&stack, "_root", "y").tiled(), TileDirection::Y);
    assert_eq!(control(&stack, "_root", "xy").tiled(), TileDirection::Both);
    assert_eq!(control(&stack, "_root", "off").tiled(), TileDirection::None);
    assert_eq!(
        control(&stack, "_root", "none").tiled(),
        TileDirection::None
    );
}

// ---- layout axes ----------------------------------------------------------------------------

#[test]
fn l06_layout_axis_errors() {
    check("l06_layout_axis_errors");
}

#[test]
fn every_axis_message_and_every_layout_key() {
    for (axis, text) in [
        (r#""wide""#, "Unrecognized token!"),
        ("true", "Trying to give JSON value without a number value!"),
        (r#""100""#, "Dangling number (no % or px in Size)"),
    ] {
        for key in ["size", "offset", "min_size", "max_size"] {
            expect(
                &format!(r#""k": {{ "type": "panel", "{key}": [ "100%", {axis} ] }}"#),
                "InvalidValue",
                text,
                &format!("k/{key}/1"),
            );
        }
    }
    clean(
        r#""k": { "type": "panel", "size": [ "100%c + 2px", "default" ], "offset": [ 0, -4 ], "min_size": [ "$w", "fill" ], "max_size": [ "100%sm", "100% - 4px" ] }"#,
    );
}

// ---- animations -----------------------------------------------------------------------------

#[test]
fn l05_anim_rules() {
    check("l05_anim_rules");
}

#[test]
fn every_animation_rule() {
    expect(
        r#""k": { "anim_type": 5 }"#,
        "InvalidType",
        "JSON UI parse failure - expected a string.",
        "k/anim_type",
    );
    for (anim_type, _) in AnimationType::ALL {
        let name = anim_type.name();
        let found = lint(&format!(
            r#""k": {{ "anim_type": "{name}", "from": [0, 0], "to": [0, 0] }}"#
        ));
        assert!(found.is_empty(), "{name}: {found:?}");
    }
    // a frame count of 0 is the one bad value; it is the flip book's rule
    clean(r#""k": { "anim_type": "flip_book", "initial_uv": [0, 0], "frame_count": 1 }"#);
    for kind in ["offset", "size"] {
        let animation = |members: &str| format!(r#""k": {{ "anim_type": "{kind}", {members} }}"#);
        for members in [r#""from": [0, 0]"#, r#""to": [0, 0]"#, r#""duration": 1"#] {
            expect(
                &animation(members),
                "MissingRequiredField",
                "The animation needs to have a specified axis offset.",
                "k",
            );
        }
        for (members, text) in [
            (
                r#""from": ["0px", "0px"], "to": ["0px", "100% + 2px"]"#,
                "The animation needs to have the same number y values.",
            ),
            (
                r#""from": ["0px + 1px", "0px"], "to": ["0px", "0px"]"#,
                "The animation needs to have the same number x values.",
            ),
            (
                r#""from": ["100% + 2px", "0px"], "to": ["100% - 2px", "0px"]"#,
                "All value's operations need to match with each other.",
            ),
            (
                r#""from": ["100%", "0px"], "to": ["100px", "0px"]"#,
                "All value's types need to match with each other.",
            ),
            (
                r#""from": ["100%c", "0px"], "to": ["100%", "0px"]"#,
                "All value's types need to match with each other.",
            ),
        ] {
            let found = one(&animation(members));
            assert_eq!(
                (found.0, found.1.as_str()),
                ("InvalidValue", text),
                "{members}"
            );
        }
        clean(&animation(
            r#""from": ["100% - 2px", 0], "to": ["50% - 8px", 10]"#,
        ));
        clean(&animation(r#""from": "$from", "to": [0, 0]"#));
    }
    // the endpoint rules are the layout animations' only
    clean(r#""k": { "anim_type": "alpha" }"#);
}

#[test]
fn l10_aseprite_flip_book_deviation() {
    // vanilla never reads the flip-book keys for this type, silently
    check("l10_aseprite_flip_book");
}

#[test]
fn animation_keys_only_some_types_read() {
    let unused = |definition: &str| -> Vec<String> {
        lint(definition)
            .into_iter()
            .filter(|f| f.0 == "UnusedProperty")
            .map(|f| f.2)
            .collect()
    };
    for key in [
        "fps",
        "frame_count",
        "frame_step",
        "reversible",
        "orientation",
        "looping",
    ] {
        assert_eq!(
            unused(&format!(r#""k": {{ "anim_type": "alpha", "{key}": 1 }}"#)),
            [format!("k/{key}")]
        );
    }
    assert_eq!(
        unused(r#""k": { "anim_type": "alpha", "initial_uv": [0, 0] }"#),
        ["k/initial_uv"]
    );
    assert_eq!(
        unused(r#""k": { "anim_type": "uv", "scale_from_starting_alpha": true }"#),
        ["k/scale_from_starting_alpha"]
    );
    assert!(unused(r#""k": { "anim_type": "alpha", "scale_from_starting_alpha": true, "duration": 1, "next": "@a.k" }"#).is_empty());
    assert!(
        unused(r#""k": { "anim_type": "aseprite_flip_book", "initial_uv": [0, 0] }"#).is_empty()
    );
    // dead keys vanilla files carry
    assert_eq!(
        unused(r#""k": { "anim_type": "alpha", "activated": true, "start_event": "e" }"#),
        ["k/activated", "k/start_event"]
    );
}

// ---- unknown properties ---------------------------------------------------------------------

#[test]
fn l07_unused_property_on_panel_deviation() {
    // vanilla ignores a key no component of the type reads, silently
    check("l07_unused_property_on_panel");
    for key in ["focus_enabled", "sound_name", "texture", "text"] {
        assert_eq!(
            one(
                &format!(r#""k": {{ "type": "panel", "{key}": null, "layer": 1 }}"#)
                    .replace("null", "\"$v\"")
            )
            .2,
            format!("k/{key}")
        );
    }
}

#[test]
fn l17_dead_renderer_keys() {
    check("l17_dead_renderer_keys");
}

#[test]
fn keys_the_game_never_reads_are_unused_on_every_type() {
    for key in [
        "always_rotate",
        "rotate_speed",
        "hover_color",
        "hover_alpha",
        "pressed_color",
        "pressed_alpha",
        "nine_slice_buttom",
        "nine_slice_left",
        "nine_slice_right",
        "nine_slice_top",
        "place_holder_text",
        "slider_range",
    ] {
        expect(
            &format!(r#""k": {{ "type": "panel", "{key}": 1 }}"#),
            "UnusedProperty",
            "property not read by this control type",
            &format!("k/{key}"),
        );
    }
}

#[test]
fn exempt_and_unknown_keys_are_not_reported() {
    // variables, their defaults, the keys every type has, the merge's `texture:<n>`, and
    // keys no table names (a renderer's own)
    clean(
        r#""leaf": { "type": "label", "text": "t" },
           "k": { "type": "image", "$var": 1, "$other|default": 2, "variables": [], "modifications": [], "controls": [], "bindings": [],
                  "anims": [], "texture:1": "t", "some_future_key": 1, "focus_wrap_enabled": true, "tts_section_header": "h" },
           "c": { "type": "custom", "renderer": "paper_doll_renderer", "renderer_private": 1, "use_uuid": true }"#,
    );
    // a key of a component the type has is read; a typeless or dynamic type reads anything
    clean(
        r#""k": { "type": "button", "button_mappings": [], "sound_name": "s", "focus_enabled": true, "default_control": "d" }"#,
    );
    clean(
        r#""k": { "button_mappings": [], "texture": "t" }, "j": { "type": "$t", "button_mappings": [] }"#,
    );
    // the long tail by type
    clean(r#""k": { "type": "scroll_view", "scroll_speed": 2, "scrollbar_box": "b" }"#);
    assert_eq!(
        one(r#""k": { "type": "panel", "scroll_speed": 2 }"#).0,
        "UnusedProperty"
    );
    assert_eq!(
        one(r#""k": { "type": "collection_panel", "orientation": "vertical" }"#).0,
        "UnusedProperty"
    );
    clean(r#""k": { "type": "collection_panel", "collection_name": "c" }"#);
}

#[test]
fn a_number_endpoint_pairs_with_any_unit() {
    // Vanilla's own screen animations run from `0` to `"100%"` and from `"-100%"` to
    // `0`; a leading minus is the number's sign, not an operation
    for kind in ["offset", "size"] {
        clean(&format!(
            r#""k": {{ "anim_type": "{kind}", "from": [0, "-100%"], "to": ["100%", 0] }}"#
        ));
        clean(&format!(
            r#""k": {{ "anim_type": "{kind}", "from": ["-25%", "0px"], "to": ["25%", "-4px"] }}"#
        ));
    }
}

#[test]
fn a_control_of_an_unknown_type_is_not_checked_for_unused_keys() {
    expect(
        r#""k": { "type": "nope", "button_mappings": [], "texture": "t" }"#,
        "InvalidEnumValue",
        "Type 'nope' not recognized for control: k",
        "k/type",
    );
}

#[test]
fn a_negative_number_holds_and_a_comparison_is_strict() {
    let variables = Variables::default();
    assert!(variables.is_truthy(&json("-1")));
    assert!(!variables.is_truthy(&Value::from("(2 > 2)")));
    assert!(variables.is_truthy(&Value::from("(3 > 2)")));
    assert!(!variables.is_truthy(&Value::from("(2 < 2)")));
    assert!(variables.is_truthy(&Value::from("(1 < 2)")));
}

#[test]
fn an_ignored_binding_is_not_registered() {
    let stack = one_file(
        r##"{ "k": { "type": "panel", "bindings": [
            { "binding_name": "#a", "ignored": true },
            { "binding_name": "#b", "ignored": false },
            { "binding_name": "#c" } ] } }"##,
    );
    let names: Vec<_> = control(&stack, "_root", "k")
        .bindings()
        .into_iter()
        .map(|binding| binding.name.unwrap())
        .collect();
    assert_eq!(names, ["#b", "#c"]);
}

/// A stack of one pack with `common.json` and `screen.json`, and the texts it raises.
fn two_files(common: &str, screen: &str) -> Vec<String> {
    let stack = stack_of(&[&[
        (
            "ui/_ui_defs.json",
            r#"{ "ui_defs": ["ui/common.json", "ui/screen.json"] }"#,
        ),
        ("ui/common.json", common),
        ("ui/screen.json", screen),
    ]]);
    findings(&stack).into_iter().map(|f| f.2).collect()
}

#[test]
fn an_unqualified_child_base_falls_back_to_the_namespace_of_the_parents_base() {
    // vanilla's command_block_screen.json: `default@close_button_panel` under
    // `minimize@common.close_button`, with `close_button_panel` defined in `common`
    let common = r#"{ "namespace": "common",
        "close_button": { "type": "button" },
        "close_button_panel": { "type": "panel" } }"#;
    let screen = |extra: &str| {
        format!(
            r#"{{ "namespace": "command_block" {extra},
                "minimize@common.close_button": {{ "controls": [
                    {{ "default@close_button_panel": {{ "text": "t" }} }} ] }} }}"#
        )
    };
    // found in `common`: a panel, which does not read `text`
    assert_eq!(
        two_files(common, &screen("")),
        ["property not read by this control type"]
    );
    // the file's own namespace comes first: there it is a label, which reads `text`
    let own = r#", "close_button_panel": { "type": "label" }"#;
    assert!(two_files(common, &screen(own)).is_empty());
    // both miss: the child cannot be created
    let texts = two_files(
        r#"{ "namespace": "common", "close_button": { "type": "button" } }"#,
        &screen(""),
    );
    assert_eq!(texts.len(), 2, "{texts:?}");
    assert_eq!(
        texts[0],
        "Type not specified (or @-base not found) for control: default"
    );
    // a parent without a base in another namespace gives no second place to look
    let plain = r#"{ "namespace": "command_block",
        "minimize": { "type": "button", "controls": [ { "default@close_button_panel": {} } ] } }"#;
    assert_eq!(two_files(common, plain).len(), 2);
    // a qualified reference is looked up where it says, and nowhere else
    let qualified = r#"{ "namespace": "command_block",
        "minimize@common.close_button": { "controls": [ { "default@command_block.close_button_panel": {} } ] } }"#;
    assert_eq!(two_files(common, qualified).len(), 2);
}

#[test]
fn a_renderers_own_key_is_not_unused_on_another_type() {
    // validation "Unknown properties": renderer-specific keys are exempt
    clean(r#""k": { "type": "panel", "rotation": "auto", "text_color": "red", "use_uuid": true }"#);
    // `renderer` itself is read by a custom control only
    assert_eq!(
        one(r#""k": { "type": "panel", "renderer": "hud_player_renderer" }"#).0,
        "UnusedProperty"
    );
}

#[test]
fn a_null_type_hides_the_inherited_one_where_it_is_written_and_is_not_handed_on() {
    // `near` writes `"type": null`: it has no type itself, and a definition that inherits
    // from it takes the farther base's, so `text` is a key no component of a panel reads.
    // A child is built from `near` as flattened, where the type is the null: nothing is known
    // of its type and its `text` is not checked
    let found = lint(
        r#""far": { "type": "panel" },
           "near@far": { "type": null, "text": "x" },
           "k@near": { "text": "x" },
           "p": { "type": "panel", "controls": [{ "c@near": { "text": "x" } }] }"#,
    );
    let places: Vec<&str> = found.iter().map(|(_, _, place)| place.as_str()).collect();
    assert_eq!(places, ["k@near/text"], "{found:?}");
    assert_eq!(found[0].0, "UnusedProperty");
}

#[test]
fn two_chains_of_one_table_keep_their_own_inherited_types() {
    // `text` is read by a label and by no component of a panel. Each chain's type is written
    // three bases away; a type found for one chain says nothing of the other, whichever of
    // them the table holds first
    let chain = |name: &str, kind: &str| {
        format!(
            r#""{name}0": {{ "type": "{kind}" }}, "{name}1@{name}0": {{}}, "{name}2@{name}1": {{}},
               "{name}3@{name}2": {{ "text": "x" }}"#
        )
    };
    for (panel, label) in [("a", "z"), ("z", "a")] {
        let found = lint(&format!(
            "{}, {}",
            chain(panel, "panel"),
            chain(label, "label")
        ));
        let places: Vec<&str> = found.iter().map(|(_, _, place)| place.as_str()).collect();
        assert_eq!(places, [format!("{panel}3@{panel}2/text")], "{found:?}");
        assert_eq!(found[0].0, "UnusedProperty");
    }
}
