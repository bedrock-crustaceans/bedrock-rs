//! The node catalogue of the schema: every node type, every literal member with its type and
//! default, every key member with its default key.

use bedrock_addon::families::behavior_tree::Node;
use bedrock_addon::json::Value;

use super::common::*;

/// One parameter of a leaf or decorator: the node type, the field of `Node` (as the dump
/// names it), the literal member with a value of its type, the literal's default, the key
/// member and the default key.
struct Row {
    kind: &'static str,
    field: &'static str,
    literal_key: &'static str,
    literal: &'static str,
    default: &'static str,
    id_key: &'static str,
    default_key: &'static str,
}

const fn row(
    kind: &'static str,
    field: &'static str,
    literal_key: &'static str,
    literal: &'static str,
    default: &'static str,
    id_key: &'static str,
    default_key: &'static str,
) -> Row {
    Row {
        kind,
        field,
        literal_key,
        literal,
        default,
        id_key,
        default_key,
    }
}

const ORIGIN: &str = "[0, 0, 0]";
const TICKS: (&str, &str) = ("num_of_ticks_id", "target_num_of_ticks");
const POS: (&str, &str) = ("block_pos_id", "target_block_pos");
const RADIUS: (&str, &str) = ("search_radius_id", "target_search_radius");

/// The schema and "Decorators", one row per literal / key pair. `search_radius`'s
/// default is the 32 a failed literal leaves.
#[rustfmt::skip]
const ROWS: [Row; 27] = [
    row("minecraft:repeat_until_success", "max_attempts", "desired_max_number_of_attempts", "4", "0", "max_number_of_attempts_id", "max_number_of_attempts"),
    row("minecraft:activate_tool", "block_pos", "block_pos", "[1, -2, 3]", ORIGIN, POS.0, POS.1),
    row("minecraft:attack", "ticks", "num_of_ticks", "7", "0", TICKS.0, TICKS.1),
    row("minecraft:break_block", "block_pos", "block_pos", "[1, -2, 3]", ORIGIN, POS.0, POS.1),
    row("minecraft:break_block", "ticks", "num_of_ticks", "7", "0", TICKS.0, TICKS.1),
    row("minecraft:find_block", "block_name", "block_name", "\"log\"", "\"\"", "block_name_id", "block_name"),
    row("minecraft:find_block", "search_radius", "search_radius", "5", "32", RADIUS.0, RADIUS.1),
    row("minecraft:find_entity", "entity_name", "entity_name", "\"item\"", "\"\"", "entity_type", "entity_name"),
    row("minecraft:find_entity", "search_radius", "search_radius", "5", "32", RADIUS.0, RADIUS.1),
    row("minecraft:fly", "should_be_flying", "should_be_flying_bool", "true", "false", "should_be_flying_id", "should_be_flying"),
    row("minecraft:fly", "throw_if_no_change", "should_throw_event_if_no_state_change_necessary_bool", "true", "false", "should_throw_event_if_no_state_change_necessary_id", "should_throw_event_if_no_state_change_necessary"),
    row("minecraft:get_interaction_position_for_block", "target", "target_block_pos", "[1, -2, 3]", ORIGIN, "target_block_pos_id", "target_block_pos"),
    row("minecraft:get_interaction_position_for_block", "anchor", "anchor_block_pos", "[4, 5, 6]", ORIGIN, "anchor_block_pos_id", "anchor_block_pos"),
    row("minecraft:get_interaction_position_for_block", "facing", "facing_string", "\"north\"", "\"\"", "facing_string_id", "facing_string"),
    row("minecraft:get_interaction_position_for_block", "max_search_distance", "max_search_distance", "9", "0", "max_search_distance_id", "max_search_distance"),
    row("minecraft:interact", "ticks", "num_of_ticks", "7", "1", TICKS.0, TICKS.1),
    row("minecraft:look_at", "block_pos", "block_pos", "[1, -2, 3]", ORIGIN, POS.0, POS.1),
    row("minecraft:look_at_entity", "entity_name", "entity_name", "\"item\"", "\"\"", "entity_type", "entity_name"),
    row("minecraft:look_at_entity", "search_radius", "search_radius", "5", "32", RADIUS.0, RADIUS.1),
    row("minecraft:move", "direction", "desired_direction", "\"forward\"", "\"\"", "direction_id", "direction"),
    row("minecraft:move", "second_direction", "desired_second_direction", "\"left\"", "\"\"", "second_direction_id", "second_direction"),
    row("minecraft:move", "ticks", "max_number_of_ticks", "100", "0", "number_of_ticks_id", "number_of_ticks"),
    row("minecraft:move_to", "pos", "block_pos", "[0.5, -4, 2.25]", "[0.0, 0.0, 0.0]", POS.0, POS.1),
    row("minecraft:move_to", "distance_epsilon", "distance_epsilon", "1.5", "0.0", "distance_epsilon_id", "target_distance_epsilon"),
    row("minecraft:shoot_bow", "shots", "num_of_shots", "3", "0", "num_of_shots_id", "target_num_of_shots"),
    row("minecraft:wait", "seconds", "wait_duration", "3", "0", "wait_duration", "duration"),
    row("minecraft:wait_ticks", "ticks", "num_of_ticks", "7", "0", TICKS.0, TICKS.1),
];

fn json(text: &str) -> Value {
    facet_json::from_str(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}

/// The field `field` of the root node `{kind: body}`, as the dump writes a parameter, and the
/// number of findings.
fn field_of(kind: &str, body: &str, field: &str) -> (String, usize) {
    let (node, findings) = root_of(&format!(r#"{{"{kind}": {body}}}"#));
    let dump = dump_node(&node);
    let object = dump.as_object().unwrap();
    assert_eq!(
        object
            .get("type")
            .and_then(|t| t.as_string())
            .unwrap()
            .as_str(),
        kind
    );
    let value = object
        .get(field)
        .unwrap_or_else(|| panic!("{kind}: no field `{field}` in {}", canon(&dump)));
    // the decorator of the table has no child here: rule 9's information is not about its parameter
    let about_the_parameter = findings
        .iter()
        .filter(|finding| finding.code.id() != "core/invalid_node")
        .count();
    (canon(value), about_the_parameter)
}

fn expected(literal: &str, key: &str) -> String {
    canon(&json(&format!(
        r#"{{"literal": {literal}, "key": "{key}"}}"#
    )))
}

#[test]
fn a_literal_of_its_type_is_read_and_the_key_stays_empty() {
    for row in &ROWS {
        let body = format!(r#"{{"{}": {}}}"#, row.literal_key, row.literal);
        let (found, findings) = field_of(row.kind, &body, row.field);
        // find_block presets its block-name key before reading the literal
        let key = if (row.kind, row.field) == ("minecraft:find_block", "block_name") {
            "block_name"
        } else {
            ""
        };
        assert_eq!(found, expected(row.literal, key), "{} {body}", row.kind);
        assert_eq!(findings, 0, "{} {body}", row.kind);
    }
}

#[test]
fn a_literal_beside_its_key_member_wins() {
    for row in ROWS.iter().filter(|row| row.literal_key != row.id_key) {
        let body = format!(
            r#"{{"{}": "elsewhere", "{}": {}}}"#,
            row.id_key, row.literal_key, row.literal
        );
        let (found, findings) = field_of(row.kind, &body, row.field);
        let key = if (row.kind, row.field) == ("minecraft:find_block", "block_name") {
            "block_name"
        } else {
            ""
        };
        // the key member is not read when the literal is, and is no unknown member either
        assert_eq!(found, expected(row.literal, key), "{} {body}", row.kind);
        assert_eq!(findings, 0, "{} {body}", row.kind);
    }
}

#[test]
fn without_a_literal_the_key_member_names_the_blackboard_entry() {
    for row in &ROWS {
        let body = format!(r#"{{"{}": "elsewhere"}}"#, row.id_key);
        let (found, findings) = field_of(row.kind, &body, row.field);
        assert_eq!(
            found,
            expected(row.default, "elsewhere"),
            "{} {body}",
            row.kind
        );
        assert_eq!(findings, 0, "{} {body}", row.kind);
    }
}

#[test]
fn without_either_member_the_default_literal_and_the_default_key() {
    for row in &ROWS {
        for body in ["{}", "5", "null", "[]"] {
            // a body that is not an object reads as an empty one
            let (found, findings) = field_of(row.kind, body, row.field);
            assert_eq!(
                found,
                expected(row.default, row.default_key),
                "{} {body}",
                row.kind
            );
            // …and a body that is neither an object nor `null` is told (ours).
            let told = usize::from(matches!(body, "5" | "[]"));
            assert_eq!(findings, told, "{} {body}", row.kind);
        }
    }
}

#[test]
fn the_three_leaves_without_parameters() {
    for kind in [
        "minecraft:consume_item",
        "minecraft:place_block",
        "minecraft:use_entity",
    ] {
        let (node, findings) = root_of(&format!(r#"{{"{kind}": {{}}}}"#));
        assert!(
            matches!(
                node,
                Node::ConsumeItem { .. } | Node::PlaceBlock { .. } | Node::UseEntity { .. }
            ),
            "{kind}: {node:?}"
        );
        assert_eq!(
            canon(&dump_node(&node)),
            canon(&json(&format!(r#"{{"type": "{kind}"}}"#)))
        );
        silent(&findings);
    }
}
