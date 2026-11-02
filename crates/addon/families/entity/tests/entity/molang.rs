//! Molang at document level: property
//! defaults, `scripts.animate`, event `set_property` values; read and versioned in both builds,
//! compiled with the `molang` feature.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_family_entity::{AnimateEntry, PropertyDefault};

#[cfg(feature = "molang")]
use super::common::with_code;
use super::common::{codes, cx, doc, json, stable};

fn properties(defaults: &str) -> String {
    format!(
        r#"{{ "description": {{ "identifier": "demo:x", "properties": {defaults} }}, "components": {{}} }}"#
    )
}

fn set_property(value: &str) -> String {
    format!(
        r#"{{ "description": {{ "identifier": "demo:x", "properties": {{ "demo:p": {{ "type": "int", "range": [0, 9], "default": 0 }} }} }},
            "components": {{}}, "events": {{ "demo:go": {{ "set_property": {{ "demo:p": {value} }} }} }} }}"#
    )
}

/// A context that compiles Molang (feature `molang`).
#[cfg(feature = "molang")]
fn compiling() -> bedrock_addon_core::load::LoadContext {
    let mut cx = cx();
    cx.molang = Some(std::sync::Arc::new(
        bedrock_addon_core::molang::MolangCheck::new(),
    ));
    cx
}

#[test]
fn a_property_default_string_takes_the_packs_molang_version() {
    let text = doc(
        "1.26.50",
        &properties(
            r#"{ "demo:size": { "type": "float", "range": [0, 1], "default": "math.random(0, 1)" } }"#,
        ),
    );
    let (entity, _) = stable(&text, &cx());
    let Some(PropertyDefault::Expression(source)) = &entity.properties()["demo:size"].default
    else {
        panic!("an expression");
    };
    assert_eq!(source.as_str(), "math.random(0, 1)");
    assert!(source.raw_version().is_some(), "pass 1 set the version");
}

#[test]
fn an_animate_condition_is_a_molang_string() {
    let text = doc(
        "1.26.50",
        r#"{ "description": { "identifier": "demo:x", "scripts": { "animate": [ "a", { "b": "query.is_moving" }, { "c": 1 } ] } }, "components": {} }"#,
    );
    let (entity, found) = stable(&text, &cx());
    let animate = &entity.description.scripts.as_ref().unwrap().animate;
    assert_eq!(animate.len(), 2);
    assert!(
        matches!(&animate[1], AnimateEntry::Conditional(m) if m["b"].as_str() == "query.is_moving")
    );
    // The crate's own: an information, the entry dropped.
    assert!(codes(&found, Severity::Warning).is_empty(), "{found:#?}");
    assert!(codes(&found, Severity::Info).contains(&"InvalidType"));
}

#[test]
fn an_animate_object_with_two_members_is_reported() {
    let text = doc(
        "1.26.50",
        r#"{ "description": { "identifier": "demo:x", "scripts": { "animate": [ { "a": "1", "b": "2" } ] } }, "components": {} }"#,
    );
    let (_, found) = stable(&text, &cx());
    assert_eq!(codes(&found, Severity::Warning), ["InvalidValue"]);
}

#[cfg(not(feature = "molang"))]
#[test]
fn without_the_feature_expressions_are_not_checked() {
    let text = doc("1.26.50", &set_property(r#""query.is_baby ? 1 : 0""#));
    let (_, found) = stable(&text, &cx());
    assert!(
        codes(&found, Severity::Info).contains(&"MolangNotChecked"),
        "{found:#?}"
    );
}

/// A rejected condition behind an entry the reader does not hold is told at the index the
/// file has it at.
#[cfg(feature = "molang")]
#[test]
fn a_rejected_animate_condition_behind_a_dropped_entry_is_at_the_index_of_the_file() {
    let text = doc(
        "1.26.50",
        r#"{ "description": { "identifier": "demo:x", "scripts": { "animate": [ 7, { "c": 1 }, { "b": "1 +" } ] } }, "components": {} }"#,
    );
    let (_, found) = stable(&text, &compiling());
    let at: Vec<String> = found
        .iter()
        .filter(|finding| finding.code.name() == "InvalidMolang")
        .map(|finding| finding.location.json_path.to_string())
        .collect();
    assert!(!at.is_empty(), "{found:#?}");
    for path in at {
        assert!(path.ends_with("description/scripts/animate/2/b"), "{path}");
    }
}

#[cfg(feature = "molang")]
#[test]
fn m05_allowed_property_defaults_compile_without_a_finding() {
    let text = doc(
        "1.26.50",
        &properties(
            r#"{
            "demo:size": { "type": "float", "range": [0, 1], "default": "query.had_component_group('baby') ? 1 : 0" },
            "demo:r": { "type": "float", "range": [0, 1], "default": "math.random(0, 1)" },
            "demo:mood": { "type": "enum", "values": ["a", "b"], "default": "a" }
        }"#,
        ),
    );
    let (entity, found) = stable(&text, &compiling());
    assert!(codes(&found, Severity::Info).is_empty(), "{found:#?}");
    assert_eq!(entity.properties().len(), 3);
}

#[cfg(feature = "molang")]
#[test]
fn m05_a_default_outside_the_allow_list_drops_the_property_group() {
    for default in ["query.is_baby", "v.x = 1; return v.x;"] {
        let text = doc(
            "1.26.50",
            &properties(&format!(
                r#"{{ "demo:a": {{ "type": "bool", "default": true }}, "demo:b": {{ "type": "float", "range": [0, 1], "default": "{default}" }} }}"#
            )),
        );
        let (entity, found) = stable(&text, &compiling());
        assert!(entity.properties().is_empty(), "{default}");
        // "Error loading Actor Properties": the game's level-3 line, a kept error.
        let group = with_code(&found, "DroppedField");
        assert_eq!(
            group.last().map(|f| (f.severity, f.kept)),
            Some((Severity::Error, true)),
            "{default}"
        );
        let found = codes(&found, Severity::Warning);
        assert!(found.contains(&"InvalidMolang"), "{default}: {found:?}");
        assert_eq!(found.last(), Some(&"DroppedField"), "{default}");
    }
}

#[cfg(feature = "molang")]
#[test]
fn m06_set_property_scalars_are_not_compiled() {
    for value in ["true", "3"] {
        let text = doc("1.26.50", &set_property(value));
        let (entity, found) = stable(&text, &compiling());
        assert!(found.is_empty(), "{value}: {found:#?}");
        assert!(
            entity.events["demo:go"]
                .node
                .as_object()
                .unwrap()
                .get("set_property")
                .is_some()
        );
    }
}

#[cfg(feature = "molang")]
#[test]
fn m06_a_set_property_expression_is_compiled() {
    let text = doc("1.26.50", &set_property(r#""query.is_baby ? 1 : 0""#));
    let (_, found) = stable(&text, &compiling());
    assert!(found.is_empty(), "{found:#?}");
    let text = doc("1.26.50", &set_property(r#""1 +""#));
    let (entity, found) = stable(&text, &compiling());
    // The parser's level-3 line, and the entity loads without the response: a kept error.
    assert!(
        found
            .iter()
            .any(|f| f.code.name() == "InvalidMolang" && f.severity == Severity::Error && f.kept),
        "{found:#?}"
    );
    assert_eq!(entity.events["demo:go"].node, json("{}"));
}

#[test]
fn m06_an_object_set_property_value_is_invalid_and_dropped() {
    let text = doc(
        "1.26.50",
        &set_property(r#"{ "expression": "1", "version": 13 }"#),
    );
    let (entity, found) = stable(&text, &cx());
    // The response schema's type line, a kept error, and the response dropped.
    assert_eq!(codes(&found, Severity::Warning), ["InvalidType"]);
    assert!(
        found.iter().any(|f| f.kept
            && f.code.to_string() == "unknown child schema option type.  Allowed types:  'molang'"),
        "{found:#?}"
    );
    assert_eq!(entity.events["demo:go"].node, json("{}"));
}

/// The Molang walk over a list of a typed component read by the lenient conversion: a
/// rejected condition behind elements the conversion leaves out is told at the index the file
/// has it at. Below 1.15.0 the legacy schema reader takes out the element that is no object
/// and the condition no option takes, each with the game's type line; the third
/// element's condition (a string, a `molang` value) reaches the compile.
#[cfg(feature = "molang")]
#[test]
fn a_rejected_condition_behind_an_element_the_lenient_conversion_leaves_out_is_at_the_index_of_the_file()
 {
    let text = doc(
        "1.14.0",
        r#"{ "description": { "identifier": "demo:x" }, "components": {
            "minecraft:anger_level": { "on_increase_sounds": [ 7, { "sound": "a", "condition": { "expression": "1" } }, { "sound": "b", "condition": "1 +" } ] }
        } }"#,
    );
    let (_, found) = stable(&text, &compiling());
    let kept: Vec<String> = found
        .iter()
        .filter(|finding| finding.kept)
        .map(|finding| finding.location.json_path.to_string())
        .collect();
    assert_eq!(
        kept,
        [
            "minecraft:entity/components/minecraft:anger_level/on_increase_sounds/0",
            "minecraft:entity/components/minecraft:anger_level/on_increase_sounds/1/condition",
            "minecraft:entity/components/minecraft:anger_level/on_increase_sounds/2/condition"
        ],
        "{found:#?}"
    );
    let at: Vec<String> = found
        .iter()
        .filter(|finding| finding.code.name() == "InvalidMolang")
        .map(|finding| finding.location.json_path.to_string())
        .collect();
    assert!(!at.is_empty(), "{found:#?}");
    for path in at {
        assert!(
            path.ends_with("minecraft:anger_level/on_increase_sounds/2/condition"),
            "{path}"
        );
    }
}

/// `behavior.eat_block.success_chance` in the lenient conversion: a rejected
/// expression is a warning and the member takes its default (validate applies the
/// `ReplaceWith(0.02)` outcome); in a document at its typed version it is kept.
#[cfg(feature = "molang")]
#[test]
fn d5_a_rejected_lenient_eat_block_success_chance_takes_the_default() {
    use bedrock_addon_family_entity::goal::EatBlock;
    let text = |version: &str| {
        doc(
            version,
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:behavior.eat_block": {"success_chance": "1 +"}}}"#,
        )
    };
    let (entity, found) = stable(&text("1.26.30"), &compiling());
    assert!(
        codes(&found, Severity::Warning).contains(&"InvalidMolang"),
        "{found:#?}"
    );
    let goal = entity
        .components
        .get::<EatBlock>()
        .expect("the goal is kept");
    assert!(goal.success_chance.is_none(), "{goal:?}");
}

/// `m07_dynamic_component_typed_node`: `heartbeat.interval` (a `Dynamic` component)
/// compiles at the pack's version; a rejected expression is `InvalidMolang` and `heartbeat` is
/// dropped with `DroppedComponent` (a warning: the entity loads). The group's component was
/// `rideable`, typed from 1.26.60; `reflect_projectiles` is still `Dynamic`.
#[cfg(feature = "molang")]
#[test]
fn m07_dynamic_component_typed_node() {
    let text = |interval: &str| {
        doc(
            "1.26.50",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:heartbeat": {{"interval": "{interval}"}}}}, "component_groups": {{"g": {{"minecraft:reflect_projectiles": {{"azimuth_angle": "{interval}"}}}}}}}}"#
            ),
        )
    };
    let (entity, found) = stable(
        &text("2.0 - math.clamp(query.anger_level / 80, 0, 1)"),
        &compiling(),
    );
    assert!(found.is_empty(), "{found:#?}");
    assert_eq!(entity.components.len(), 1);
    let (entity, found) = stable(&text("1 +"), &compiling());
    assert_eq!(
        codes(&found, Severity::Warning),
        ["InvalidMolang", "InvalidMolang"],
        "{found:#?}"
    );
    assert!(entity.components.is_empty());
    assert!(entity.component_groups["g"].components.is_empty());
}

/// `rideable.seats[].rotate_rider_by` of the typed `rideable` (from 1.26.60): below
/// 1.26.60 the legacy schema's reader takes a rejected expression as a warning and the seat
/// keeps its default 0, the component kept; at 1.26.60 it is the typed expression node, whose
/// rejected expression is an error.
#[cfg(feature = "molang")]
#[test]
fn a_seat_rotation_expression_of_the_typed_rideable() {
    use bedrock_addon_family_entity::component::Rideable;
    let text = |version: &str, rotation: &str| {
        doc(
            version,
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:rideable": {{"seats": [{{"rotate_rider_by": "{rotation}"}}]}}}}}}"#
            ),
        )
    };
    let (entity, found) = stable(&text("1.26.60", "query.is_baby ? 90 : 0"), &compiling());
    assert!(found.is_empty(), "{found:#?}");
    assert!(entity.components.get::<Rideable>().is_some());
    let (entity, found) = stable(&text("1.26.50", "1 +"), &compiling());
    assert_eq!(
        codes(&found, Severity::Warning),
        ["InvalidMolang"],
        "{found:#?}"
    );
    let rideable = entity.components.get::<Rideable>().expect("kept");
    assert_eq!(
        rideable.seats[0].rotate_rider_by,
        bedrock_addon_core::molang::Molang::Const(0.0)
    );
    let found = super::common::outcome(&text("1.26.60", "1 +"), &compiling()).diagnostics;
    assert!(
        found
            .iter()
            .any(|f| f.severity == Severity::Error && f.code.name() == "InvalidMolang"),
        "{found:#?}"
    );
}

/// Without a Molang check (or the feature) the members are counted as not checked and kept.
#[test]
fn m07_dynamic_component_members_without_a_check_are_kept() {
    let text = doc(
        "1.26.50",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:heartbeat": {"interval": "1 +"}}}"#,
    );
    let (entity, found) = stable(&text, &cx());
    assert!(codes(&found, Severity::Warning).is_empty(), "{found:#?}");
    assert_eq!(entity.components.len(), 1);
}

/// The findings of the entity with the payload members `rest`, as `<code> <severity> <path
/// below the payload>`, and the entity written on one line.
fn told_of(rest: &str, cx: &bedrock_addon_core::load::LoadContext) -> (Vec<String>, String) {
    use bedrock_addon_core::document::Content;
    let text = doc(
        "1.26.50",
        &format!(r#"{{"description": {{"identifier": "demo:x"}}, {rest}}}"#),
    );
    let (entity, found) = stable(&text, cx);
    let said = found
        .iter()
        .map(|f| {
            let path = f.location.json_path.to_string();
            let path = path.strip_prefix("minecraft:entity/").unwrap_or(&path);
            format!("{} {:?} {path}", f.code.name(), f.severity)
        })
        .collect();
    let written = entity.to_json().expect("the entity writes");
    (
        said,
        written.split_whitespace().collect::<Vec<_>>().join(" "),
    )
}

/// K100 (X-CORPUS-A, item 5, the descriptor row: "pack in `Dynamic`
/// (legacy JSON) components", rule M3): the `tags` of a block descriptor of a navigator's
/// `blocks_to_avoid` and of an item descriptor of `behavior.tempt.items` were compiled by
/// nothing. A rejected expression is a warning (the entity loads) and the
/// descriptor is kept with it.
#[cfg(feature = "molang")]
#[test]
fn m08_descriptor_tags_in_a_dynamic_component_compile_with_the_tag_queries_only() {
    // The game's own: breeze, sulfur_cube.
    let (found, _) = told_of(
        r#""components": {"minecraft:navigation.walk": {"blocks_to_avoid": [{"tags": "query.any_tag('trapdoors')"}]}, "minecraft:behavior.tempt": {"priority": 1, "items": ["minecraft:wheat", {"tags": "q.any_tag('demo:a', 'demo:b')"}]}}"#,
        &compiling(),
    );
    assert!(found.is_empty(), "{found:#?}");
    // A query outside the tag set, in a list and as the member itself.
    for (member, at) in [
        (r#"[{"tags": "query.is_baby"}]"#, "blocks_to_avoid/0/tags"),
        (r#"{"tags": "query.is_baby"}"#, "blocks_to_avoid/tags"),
    ] {
        for navigator in ["climb", "float", "fly", "generic", "hover", "swim", "walk"] {
            let key = format!("minecraft:navigation.{navigator}");
            let (found, written) = told_of(
                &format!(r#""components": {{"{key}": {{"blocks_to_avoid": {member}}}}}"#),
                &compiling(),
            );
            let finding = format!("InvalidMolang Error components/{key}/{at}");
            assert!(!found.is_empty(), "{key} {member}");
            assert!(found.iter().all(|f| *f == finding), "{key}: {found:#?}");
            assert!(written.contains(r#""tags": "query.is_baby""#), "{written}");
        }
    }
    // An expression that is none, in a component group.
    let (found, written) = told_of(
        r#""component_groups": {"g": {"minecraft:behavior.tempt": {"priority": 1, "items": ["minecraft:wheat", {"tags": "1 +"}]}}}"#,
        &compiling(),
    );
    assert!(!found.is_empty());
    assert!(
        found
            .iter()
            .all(|f| f
                == "InvalidMolang Error component_groups/g/minecraft:behavior.tempt/items/1/tags"),
        "{found:#?}"
    );
    assert!(written.contains(r#""tags": "1 +""#), "{written}");
    // A `tags` that is no string is no expression, and another member's `tags` is not a
    // descriptor's.
    let (found, _) = told_of(
        r#""components": {"minecraft:navigation.walk": {"blocks_to_avoid": [{"tags": 7}], "zz": {"tags": "1 +"}}}"#,
        &compiling(),
    );
    assert!(
        found.iter().all(|f| !f.starts_with("InvalidMolang")),
        "{found:#?}"
    );
}

/// Without a check the descriptor's expression is counted as not checked.
#[test]
fn m08_descriptor_tags_in_a_dynamic_component_are_counted_without_a_check() {
    let (found, _) = told_of(
        r#""components": {"minecraft:navigation.walk": {"blocks_to_avoid": [{"tags": "query.any_tag('trapdoors')"}]}}"#,
        &cx(),
    );
    assert_eq!(found, ["MolangNotChecked Info minecraft:entity"]);
}

/// A rejected `tags` of a descriptor in a TYPED component was
/// an error, so the entity was refused where the game loads it with the descriptor kept
/// (the descriptor row) and where the same file loads without a check.
/// It is a warning, as in a `Dynamic` component.
#[cfg(feature = "molang")]
#[test]
fn m08_a_rejected_descriptor_tags_in_a_typed_component_is_a_warning_and_the_descriptor_is_kept() {
    for (component, at) in [
        (
            r#""minecraft:block_movement_slowdown_immunity": {"blocks": [{"tags": "1 +"}]}"#,
            "minecraft:block_movement_slowdown_immunity/blocks/0/tags",
        ),
        (
            r#""minecraft:behavior.pickup_items": {"priority": 1, "excluded_items": ["minecraft:stick", {"tags": "1 +"}]}"#,
            "minecraft:behavior.pickup_items/excluded_items/1/tags",
        ),
    ] {
        for (section, below) in [
            (format!(r#""components": {{{component}}}"#), "components"),
            (
                format!(r#""component_groups": {{"g": {{{component}}}}}"#),
                "component_groups/g",
            ),
        ] {
            let (found, written) = told_of(&section, &compiling());
            assert_eq!(
                found,
                [format!("InvalidMolang Error {below}/{at}")],
                "{section}"
            );
            assert!(written.contains(r#""tags": "1 +""#), "{written}");
        }
    }
}

/// The descriptor row is of every block
/// or item descriptor of a `Dynamic` component, not of the two members the game's own files
/// write a `tags` at. Each member of the table is compiled, and counted without a check.
#[cfg(feature = "molang")]
#[test]
fn m08_descriptor_tags_of_every_descriptor_member_of_a_dynamic_component_are_compiled() {
    for (component, at) in [
        (
            r#""minecraft:behavior.float_tempt": {"priority": 1, "items": [{"tags": "1 +"}]}"#,
            "minecraft:behavior.float_tempt/items/0/tags",
        ),
        (
            r#""minecraft:breathable": {"breathe_blocks": [{"tags": "1 +"}]}"#,
            "minecraft:breathable/breathe_blocks/0/tags",
        ),
        (
            r#""minecraft:breathable": {"non_breathe_blocks": ["minecraft:stone", {"tags": "1 +"}]}"#,
            "minecraft:breathable/non_breathe_blocks/1/tags",
        ),
        (
            r#""minecraft:buoyant": {"liquid_blocks": [{"tags": "1 +"}]}"#,
            "minecraft:buoyant/liquid_blocks/0/tags",
        ),
        (
            r#""minecraft:break_blocks": {"breakable_blocks": {"tags": "1 +"}}"#,
            "minecraft:break_blocks/breakable_blocks/tags",
        ),
        // A member below a list of entries, and the descriptor an entry's `item` is.
        (
            r#""minecraft:breedable": {"environment_requirements": [{"blocks": [{"tags": "1 +"}], "count": 1, "radius": 2}]}"#,
            "minecraft:breedable/environment_requirements/0/blocks/0/tags",
        ),
        (
            r#""minecraft:healable": {"items": [{"item": {"tags": "1 +"}, "heal_amount": 2}]}"#,
            "minecraft:healable/items/0/item/tags",
        ),
    ] {
        let section = format!(r#""components": {{{component}}}"#);
        let (found, written) = told_of(&section, &compiling());
        assert_eq!(
            found,
            [format!("InvalidMolang Error components/{at}")],
            "{component}"
        );
        assert!(written.contains(r#""tags": "1 +""#), "{written}");
        let (found, _) = told_of(&section, &cx());
        assert_eq!(
            found,
            ["MolangNotChecked Info minecraft:entity"],
            "{component}"
        );
        // An expression of the tag queries says nothing.
        let (found, _) = told_of(
            &section.replace("1 +", "query.any_tag('demo:a')"),
            &compiling(),
        );
        assert!(found.is_empty(), "{component}: {found:#?}");
    }
}
