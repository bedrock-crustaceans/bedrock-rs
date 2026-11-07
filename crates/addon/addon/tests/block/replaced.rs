//! What a step of a legacy block replaces, drops or moves is told at the member the FILE has:
//! a legacy member
//! beside the member a step writes it to, a permutation that goes whole, and the findings of
//! the typed read of a member a step moved.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::load::LoadContext;

use super::common::{allow_listed, context, document, loads};

/// The findings of the block `payload` of `version` in an ordinary pack as
/// `Code Severity at path`, and the written text.
#[track_caller]
fn told(version: &str, payload: &str) -> (Vec<String>, String) {
    told_in(version, payload, &context())
}

/// [`told`] in the context `cx`.
#[track_caller]
fn told_in(version: &str, payload: &str, cx: &LoadContext) -> (Vec<String>, String) {
    let block = loads(&document(version, payload), cx);
    let found = block
        .diagnostics()
        .iter()
        .map(|f| {
            let code: String = f.code.id().to_owned();
            format!("{code} {:?} at {}", f.severity, f.location.json_path)
        })
        .collect();
    // The upgraded block in the newest format's shape (the writer writes the members
    // of ours in their legacy shapes, at a legacy version).
    let mut written = Vec::new();
    bedrock_addon::json::write_json(
        &super::common::newest_shape(&block),
        &mut written,
        bedrock_addon::json::WriteStyle::default(),
    )
    .expect("the block has a JSON form");
    let written = String::from_utf8(written).expect("UTF-8");
    (
        found,
        written.split_whitespace().collect::<Vec<_>>().join(" "),
    )
}

/// What a document the load refuses tells, in [`told`]'s shape, without the written form.
#[track_caller]
fn refused(version: &str, payload: &str) -> Vec<String> {
    super::common::rejected(&document(version, payload), &context())
        .iter()
        .map(|f| {
            let code: String = f.code.id().to_owned();
            format!("{code} {:?} at {}", f.severity, f.location.json_path)
        })
        .collect()
}

#[track_caller]
fn has(found: &[String], finding: &str) {
    assert!(
        found.iter().any(|f| f == finding),
        "no `{finding}` in {found:#?}"
    );
}

#[test]
fn case_13_a_menu_category_of_the_file_replaced_by_a_creative_category_is_told() {
    // the component is copied into `description.menu_category` "without
    // checking whether one exists already". The game discards the file's own: INFO.
    let (found, written) = told(
        "1.19.10",
        r#"{"description": {"identifier": "demo:x", "menu_category": {"category": "nature", "group": "itemGroup.name.wood"}}, "components": {"minecraft:creative_category": {"category": "items"}}}"#,
    );
    has(
        &found,
        "core/dropped_field Info at minecraft:block/description/menu_category",
    );
    assert!(
        written.contains(r#""menu_category": { "category": "items""#),
        "{written}"
    );
    assert!(!written.contains("nature"), "{written}");
    // The same category in both: nothing is replaced, nothing is told.
    let (found, _) = told(
        "1.19.10",
        r#"{"description": {"identifier": "demo:x", "menu_category": {"category": "items"}}, "components": {"minecraft:creative_category": {"category": "items"}}}"#,
    );
    assert!(
        !found.iter().any(|f| f.starts_with("core/dropped_field")),
        "{found:#?}"
    );
    // Of two components the later one wins (the base first, then each permutation): the
    // earlier one is told where the file has it.
    let (found, written) = told(
        "1.19.10",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:creative_category": {"category": "items"}}, "permutations": [{"condition": "1", "components": {"minecraft:creative_category": {"category": "nature"}}}]}"#,
    );
    has(
        &found,
        "core/dropped_field Info at minecraft:block/components/minecraft:creative_category",
    );
    assert!(written.contains(r#""category": "nature""#), "{written}");
}

#[test]
fn case_14_a_flammable_modifier_replaced_by_the_odds_is_told() {
    let (found, written) = told(
        "1.16.100",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:flammable": {"flame_odds": 5, "burn_odds": 7, "catch_chance_modifier": 1, "destroy_chance_modifier": 2}}}"#,
    );
    has(
        &found,
        "core/dropped_field Info at minecraft:block/components/minecraft:flammable/catch_chance_modifier",
    );
    has(
        &found,
        "core/dropped_field Info at minecraft:block/components/minecraft:flammable/destroy_chance_modifier",
    );
    assert!(
        written.contains(r#""catch_chance_modifier": 5"#),
        "{written}"
    );
    assert!(
        written.contains(r#""destroy_chance_modifier": 7"#),
        "{written}"
    );
    // The same numbers: nothing is told.
    let (found, _) = told(
        "1.16.100",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:flammable": {"flame_odds": 5, "burn_odds": 7, "catch_chance_modifier": 5, "destroy_chance_modifier": 7.0}}}"#,
    );
    assert!(
        !found.iter().any(|f| f.starts_with("core/dropped_field")),
        "{found:#?}"
    );
    // The table's name beside the description it is made of (1.18.0).
    let (found, written) = told_in(
        "1.18.0",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:crafting_table": {"custom_description": "A", "table_name": "B", "crafting_tags": ["x"]}}}"#,
        &allow_listed(),
    );
    has(
        &found,
        "core/dropped_field Info at minecraft:block/components/minecraft:crafting_table/table_name",
    );
    assert!(written.contains(r#""table_name": "A""#), "{written}");
}

#[test]
fn case_15_a_permutation_that_goes_for_a_gated_component_is_told_at_the_permutation() {
    let found = refused(
        "1.19.40",
        r#"{"description": {"identifier": "demo:x", "properties": {"demo:open": [true, false]}}, "components": {"minecraft:friction": 0.4}, "permutations": [{"condition": "q.block_property('demo:open')", "components": {"minecraft:rotation": [0, 90, 0], "minecraft:friction": 0.3}}]}"#,
    );
    // The gate's finding names the component the file has, not the one the step made of it.
    let gate: Vec<&String> = found
        .iter()
        .filter(|f| f.starts_with("core/toggle_use_format_version"))
        .collect();
    assert_eq!(
        gate,
        [
            "core/toggle_use_format_version Error at minecraft:block/permutations/0/components/minecraft:rotation"
        ],
        "{found:#?}"
    );
    // A permutation's toggle lines are level 3 and a failing permutation fails the component
    // load: the block is refused whole, not the permutation alone.
}

#[test]
fn a_permutation_that_goes_for_a_gated_component_is_told_too() {
    // No component is gated in a cereal document since 1.26.60: a legacy one,
    // whose `sound` the legacy registry gates.
    // A permutation's failure fails the component load with it: the whole block is refused.
    let found = refused(
        "1.21.0",
        r#"{"description": {"identifier": "demo:x", "states": {"demo:open": [true, false]}}, "components": {"minecraft:friction": 0.4}, "permutations": [{"condition": "q.block_state('demo:open')", "components": {"minecraft:friction": 0.3}}, {"condition": "!q.block_state('demo:open')", "components": {"minecraft:sound": "stone", "minecraft:friction": 0.2}}]}"#,
    );
    has(
        &found,
        "core/turn_on_toggle_before Error at minecraft:block/permutations/1/components/minecraft:sound",
    );
}

#[test]
fn a_finding_of_a_member_a_step_moved_names_the_member_of_the_file() {
    // The finding names `destroy_time`, not
    // `destructible_by_mining/seconds_to_destroy`.
    // A member of the wrong type fails the legacy reader's cereal load of the component
    // and the block is refused; each finding still names the member
    // of the file.
    let found = refused(
        "1.16.100",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:destroy_time": "x", "minecraft:explosion_resistance": "y", "minecraft:pick_collision": {"origin": "x"}, "minecraft:entity_collision": {"size": 7}, "minecraft:flammable": {"flame_odds": "x", "burn_odds": 7}}, "permutations": [{"condition": "1", "components": {"minecraft:block_light_emission": "x"}}]}"#,
    );
    for at in [
        "components/minecraft:destroy_time",
        "components/minecraft:explosion_resistance",
        "components/minecraft:pick_collision/origin",
        "components/minecraft:entity_collision/size",
        "components/minecraft:flammable/flame_odds",
        "permutations/0/components/minecraft:block_light_emission",
    ] {
        has(
            &found,
            &format!("core/invalid_type Error at minecraft:block/{at}"),
        );
    }
    for gone in [
        "destructible_by",
        "selection_box",
        "collision_box",
        "chance_modifier",
        "minecraft:light_emission",
    ] {
        assert!(
            !found.iter().any(|f| f.contains(gone)),
            "{gone} in {found:#?}"
        );
    }
    // The object forms of 1.16.100, which the 1.18.0 step unwraps.
    let found = refused(
        "1.16.100",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:destroy_time": {"destroy_time": "x"}}}"#,
    );
    has(
        &found,
        "core/invalid_type Error at minecraft:block/components/minecraft:destroy_time/destroy_time",
    );
    // `rotation` of 1.19.40 and `properties`, in a permutation too.
    let found = refused(
        "1.19.40",
        r#"{"description": {"identifier": "demo:x", "properties": {"demo:open": 7}}, "components": {"minecraft:rotation": [0, "x", 0]}}"#,
    );
    has(
        &found,
        "core/invalid_type Error at minecraft:block/components/minecraft:rotation/1",
    );
    assert!(
        found
            .iter()
            .any(|f| f.ends_with("at minecraft:block/description/properties")
                || f.ends_with("at minecraft:block/description/properties/demo:open")),
        "{found:#?}"
    );
    assert!(
        !found
            .iter()
            .any(|f| f.contains("states") || f.contains("transformation")),
        "{found:#?}"
    );
}

#[test]
fn a_second_version_of_a_component_is_told_at_the_section_that_has_it() {
    // "Two versions of the same component found": the newer-named one of the file goes.
    let (found, written) = told(
        "1.16.100",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:destroy_time": 3}, "permutations": [{"condition": "1", "components": {"minecraft:destructible_by_mining": {"seconds_to_destroy": 9}}}]}"#,
    );
    has(
        &found,
        "core/duplicate_component_version Error at minecraft:block/permutations/0/components/minecraft:destructible_by_mining",
    );
    assert!(!found.iter().any(|f| f == "core/duplicate_component_version Error at minecraft:block/components/minecraft:destructible_by_mining"), "{found:#?}");
    assert!(!written.contains('9'), "{written}");
}

#[test]
fn a_component_that_goes_whole_for_one_member_of_another_type_is_told_at_the_component() {
    // The typed read drops the component for one member of another type; the member's
    // finding alone would not say that its siblings go too.
    // The block is refused under both slices (the legacy reader's cereal load fails the same
    // way), pinned in `latest.rs::a_failed_component_read_rejects_a_block_under_both_slices`;
    // what is told names the member.
    {
        // Every definition that was `DroppedComponent`.
        const DROPPED: [&str; 10] = [
            "block/component_missing_field",
            "block/duplicate_component",
            "block/part_visibility_deprecated",
            "block/part_visibility_unsupported",
            "core/dropped_component",
            "entity/component_removed_by_upgrade",
            "entity/damage_sensor_not_added",
            "entity/null_group_removed",
            "item/foil_ignored",
            "core/dropped_component_worded",
        ];
        let found = refused(
            "1.19.40",
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:flammable": {"catch_chance_modifier": "x", "destroy_chance_modifier": 7}, "minecraft:friction": 0.3}}"#,
        );
        has(
            &found,
            "core/invalid_type Error at minecraft:block/components/minecraft:flammable/catch_chance_modifier",
        );
        // The refusal is the one finding: no component is dropped from a refused block.
        assert!(
            !found
                .iter()
                .any(|f| DROPPED.iter().any(|id| f.starts_with(id))),
            "{found:#?}"
        );
    }
    // A component told of at the component itself is told once.
    let found = refused(
        "1.19.40",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:flammable": 7}}"#,
    );
    let at_component = found
        .iter()
        .filter(|f| f.ends_with("at minecraft:block/components/minecraft:flammable"))
        .count();
    assert_eq!(at_component, 1, "{found:#?}");
}

#[test]
fn case_r2_5_an_is_hidden_in_commands_the_1_19_30_step_writes_over_is_noted() {
    // "`is_hidden_in_commands` is added in every row": the game's fixer
    // writes it whatever the file has there. The file's own, another value: INFO.
    let (found, written) = told(
        "1.19.20",
        r#"{"description": {"identifier": "demo:x", "menu_category": {"category": "construction", "is_hidden_in_commands": true}}, "components": {}}"#,
    );
    has(
        &found,
        "core/dropped_field Info at minecraft:block/description/menu_category/is_hidden_in_commands",
    );
    assert!(
        !written.contains(r#""is_hidden_in_commands": true"#),
        "{written}"
    );
    // The value the step writes: nothing is replaced.
    let (found, _) = told(
        "1.19.20",
        r#"{"description": {"identifier": "demo:x", "menu_category": {"category": "construction", "is_hidden_in_commands": false}}, "components": {}}"#,
    );
    assert!(
        !found.iter().any(|f| f.starts_with("core/dropped_field")),
        "{found:#?}"
    );
    // The 1.19.10 step copies `{category, group}` of the component alone; what else
    // the component holds is not read, and told where the file has it.
    let (found, written) = told(
        "1.19.10",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:creative_category": {"category": "nature", "is_hidden_in_commands": true, "zz": 3}}}"#,
    );
    has(
        &found,
        "core/unrecognized_field Info at minecraft:block/components/minecraft:creative_category/is_hidden_in_commands",
    );
    has(
        &found,
        "core/unrecognized_field Info at minecraft:block/components/minecraft:creative_category/zz",
    );
    assert!(
        !written.contains(r#""is_hidden_in_commands": true"#) && !written.contains("zz"),
        "{written}"
    );
}

#[test]
fn case_r2_6_what_the_queued_ticking_conversion_does_not_name_is_carried_or_told() {
    // `queued_ticking` → `{min_ticks, max_ticks, looping, on_tick}`, "taken
    // from `interval_range[0]` and `interval_range[1]`". A member the conversion does not
    // name is carried into the side-car, where it is told as its sibling's is; an element
    // of the range behind the two is told where it is.
    let (found, written) = told_in(
        "1.19.40",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:queued_ticking": {"interval_range": [5, 2, 9], "on_tick": {"event": "e"}, "zz": 3}, "minecraft:random_ticking": {"on_tick": {"event": "e"}, "zz": 3}}, "events": {"e": {}}}"#,
        &allow_listed(),
    );
    has(
        &found,
        "core/unrecognized_field Warning at minecraft:block/components/minecraft:random_ticking/zz",
    );
    has(
        &found,
        "core/unrecognized_field Warning at minecraft:block/components/minecraft:queued_ticking/zz",
    );
    has(
        &found,
        "core/unrecognized_field Info at minecraft:block/components/minecraft:queued_ticking/interval_range/2",
    );
    assert!(
        written.contains(r#""min_ticks": 5, "max_ticks": 2"#),
        "{written}"
    );
    assert_eq!(written.matches(r#""zz": 3"#).count(), 2, "{written}");
}

#[test]
fn a_menu_category_group_and_the_commands_category_a_step_rewrites_are_told() {
    // The step produces `"minecraft:" + group` unconditionally; told as the item's same step
    // tells it.
    // `commands` → `none`, `is_hidden_in_commands: false`, no log: a note.
    let (found, written) = told(
        "1.21.30",
        r#"{"description": {"identifier": "demo:x", "menu_category": {"category": "nature", "group": "itemGroup.name.ore"}}, "components": {}}"#,
    );
    has(
        &found,
        "core/deprecated_member Info at minecraft:block/description/menu_category/group",
    );
    assert!(
        written.contains(r#""group": "minecraft:itemGroup.name.ore""#),
        "{written}"
    );
    let (found, written) = told(
        "1.19.30",
        r#"{"description": {"identifier": "demo:x", "menu_category": {"category": "commands"}}, "components": {}}"#,
    );
    has(
        &found,
        "core/deprecated_member Info at minecraft:block/description/menu_category/category",
    );
    assert!(written.contains(r#""category": "none""#), "{written}");
    // A category the step keeps says nothing.
    let (found, _) = told(
        "1.19.30",
        r#"{"description": {"identifier": "demo:x", "menu_category": {"category": "nature"}}, "components": {}}"#,
    );
    assert!(
        !found
            .iter()
            .any(|f| f.starts_with("core/deprecated_member")),
        "{found:#?}"
    );
}

#[test]
fn a_write_refusal_of_a_member_a_step_moved_names_the_member_of_the_file() {
    // The write refusal names
    // `minecraft:transformation/rotation`, which a 1.19.40 file does not have. The block has
    // no text at its own version either: a 1.19.40 file with the `minecraft:transformation`
    // the steps made of its member does not read back as it.
    let block = loads(
        &document(
            "1.19.40",
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:rotation": [0, 45, 0]}}"#,
        ),
        &allow_listed(),
    );
    let error = format!("{:?}", block.to_json().expect_err("no text of 1.26.20"));
    assert!(error.starts_with("NotRepresentable"), "{error}");
    assert!(
        error.contains(r#"Key("minecraft:rotation")"#) && !error.contains("transformation"),
        "{error}"
    );
    // A legacy component of an allow-listed pack, kept beside the components, is written
    // back in its legacy shape at the block's own version, where the rule does not hold
    // (a legacy file has no member for the side-car).
    let block = loads(
        &document(
            "1.21.60",
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:transformation": {"rotation": [0, 45, 0]}, "minecraft:breathability": "air"}}"#,
        ),
        &allow_listed(),
    );
    assert!(!block.deprecated.is_empty());
    let written = block.to_json().expect("a text of 1.21.60");
    assert!(
        written.contains(r#""format_version": "1.21.60""#)
            && written.contains(r#""minecraft:breathability": "air""#)
            && !written.contains("deprecated"),
        "{written}"
    );
    assert_eq!(loads(&written, &allow_listed()).value(), block.value());
    // Without it the block is written at its own version and reads back as itself.
    let block = loads(
        &document(
            "1.21.60",
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:transformation": {"rotation": [0, 45, 0]}}}"#,
        ),
        &allow_listed(),
    );
    let written = block.to_json().expect("a text of 1.21.60");
    assert!(
        written.contains(r#""format_version": "1.21.60""#),
        "{written}"
    );
    let again = loads(&written, &allow_listed());
    assert_eq!(again.value(), block.value(), "{written}");
    assert_eq!(again.to_json().ok(), Some(written));
}

#[test]
fn a_custom_component_of_the_file_under_a_name_of_the_v1_list_is_told_and_nothing_panics() {
    // "Each name becomes a keyed custom component `\"ns:name\": {}`". The
    // folder does not say what the game makes of a key of the file with that name: the
    // step's entry stands, and the file's is told (an information: ours).
    for components in [
        r#""minecraft:custom_components": ["demo:a"], "demo:a": {"p": 1}"#,
        r#""demo:a": {"p": 1}, "minecraft:custom_components": ["demo:a", "demo:b"]"#,
        r#""minecraft:friction": 0.4, "demo:b": {"p": 1}, "minecraft:custom_components": ["demo:a", "demo:b"]"#,
    ] {
        let (found, written) = told(
            "1.21.60",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{{components}}}}}"#
            ),
        );
        let name = if components.contains(r#""demo:a": {"p""#) {
            "demo:a"
        } else {
            "demo:b"
        };
        has(
            &found,
            &format!("core/dropped_field Info at minecraft:block/components/{name}"),
        );
        assert!(!written.contains(r#""p": 1"#), "{written}");
        assert!(written.contains(r#""demo:a": {}"#), "{written}");
    }
    // An entry of the file that is what the step writes says nothing.
    let (found, _) = told(
        "1.21.60",
        r#"{"description": {"identifier": "demo:x"}, "components": {"demo:a": {}, "minecraft:custom_components": ["demo:a"]}}"#,
    );
    assert!(
        !found.iter().any(|f| f.starts_with("core/dropped_field")),
        "{found:#?}"
    );
}

#[test]
fn what_a_step_takes_apart_and_does_not_read_is_told_where_the_file_has_it() {
    // (version, payload, the finding, what the written text must not hold)
    let rows = [
        // The 1.19.10 unwraps read one member of the object form.
        (
            "1.16.100",
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:explosion_resistance": {"resistance": 3, "zz": 3}}}"#,
            "block/members_thrown_away Error at minecraft:block/components/minecraft:explosion_resistance",
            "zz",
        ),
        (
            "1.16.100",
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:loot": {"loot_table": "loot_tables/x.json", "zz": 3}}}"#,
            "block/members_thrown_away Error at minecraft:block/components/minecraft:loot",
            "zz",
        ),
        // The members of the block this library writes for an allow-listed legacy block
        // (`deprecated`, `deprecated_event_handlers`) are no members of a legacy file: one
        // of the file is not read as ours.
        (
            "1.19.40",
            r#"{"description": {"identifier": "demo:x"}, "components": {}, "deprecated_event_handlers": {"a": {"responses": [{"run_command": {"command": ["say hi"]}}]}}}"#,
            "core/unrecognized_field Info at minecraft:block/deprecated_event_handlers",
            "say hi",
        ),
        (
            "1.19.40",
            r#"{"description": {"identifier": "demo:x"}, "components": {}, "deprecated": {"unit_cube": true}}"#,
            "core/unrecognized_field Info at minecraft:block/deprecated",
            "unit_cube",
        ),
        (
            "1.19.80",
            r#"{"description": {"identifier": "demo:x", "states": {"demo:open": [true, false]}}, "components": {}, "permutations": [{"condition": "q.block_state('demo:open')", "components": {"minecraft:friction": 0.3}, "deprecated": {"unit_cube": true}}]}"#,
            "core/unrecognized_field Info at minecraft:block/permutations/0/deprecated",
            "unit_cube",
        ),
        // A `minecraft:tags` beside `tag:` components keeps its names; an element
        // that is no string is left out alone.
        (
            "1.26.0",
            r#"{"description": {"identifier": "demo:x"}, "components": {"tag:ore": {}, "minecraft:tags": [7, "stone"]}}"#,
            "core/invalid_type Info at minecraft:block/components/minecraft:tags/0",
            "7",
        ),
    ];
    for (version, payload, finding, gone) in rows {
        let (found, written) = told_in(version, payload, &allow_listed());
        has(&found, finding);
        assert!(!written.contains(gone), "{payload}: {written}");
    }
    let (_, written) = told_in(
        "1.26.0",
        r#"{"description": {"identifier": "demo:x"}, "components": {"tag:ore": {}, "minecraft:tags": [7, "stone"]}}"#,
        &allow_listed(),
    );
    assert!(
        written.contains(r#""minecraft:tags": [ "stone", "ore" ]"#),
        "{written}"
    );
}

#[test]
fn a_set_block_state_beside_the_set_block_property_it_is_made_of_is_noted_and_nothing_panics() {
    // in an event and each entry of its `sequence` the key
    // `set_block_property` becomes `set_block_state`. A `set_block_state` the file has
    // beside it is written over, in either order of the two keys: noted where the file has
    // it (INFO: the game's converter keeps the renamed one without a word).
    for (at, events) in [
        (
            "e",
            r#"{"e": {"set_block_state": {"demo:a": 0}, "set_block_property": {"demo:a": 1}}}"#,
        ),
        (
            "e",
            r#"{"e": {"set_block_property": {"demo:a": 1}, "set_block_state": {"demo:a": 0}}}"#,
        ),
        (
            "e/sequence/0",
            r#"{"e": {"sequence": [{"set_block_state": {"demo:a": 0}, "set_block_property": {"demo:a": 1}}]}}"#,
        ),
        (
            "e/sequence/0",
            r#"{"e": {"sequence": [{"set_block_property": {"demo:a": 1}, "set_block_state": {"demo:a": 0}}]}}"#,
        ),
    ] {
        let payload = format!(
            r#"{{"description": {{"identifier": "demo:x", "properties": {{"demo:a": [0, 1]}}}}, "components": {{}}, "events": {events}}}"#
        );
        let (found, written) = told_in("1.19.70", &payload, &allow_listed());
        has(
            &found,
            &format!("core/dropped_field Info at minecraft:block/events/{at}/set_block_state"),
        );
        assert!(!written.contains(r#""demo:a": 0"#), "{written}");
        if at == "e" {
            assert!(
                written.contains(r#"{ "set_block_state": { "demo:a": 1 } }"#),
                "{written}"
            );
        }
    }
    // The same value under both keys: nothing is replaced, nothing is noted.
    let (found, _) = told_in(
        "1.19.70",
        r#"{"description": {"identifier": "demo:x", "properties": {"demo:a": [0, 1]}}, "components": {}, "events": {"e": {"set_block_state": {"demo:a": 1}, "set_block_property": {"demo:a": 1}}}}"#,
        &allow_listed(),
    );
    assert!(
        !found.iter().any(|f| f.starts_with("core/dropped_field")),
        "{found:#?}"
    );
}

#[test]
fn an_end_of_a_ticking_range_the_step_bounds_or_rounds_is_told_at_the_end() {
    // `ticking.range` (seconds) becomes `interval_range` (ticks). The clamp
    // to `>= 0` and the rounding to a whole tick are this library's: a warning at the end
    // the file has.
    let ticking = |range: &str| {
        format!(
            r#"{{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:ticking": {{"range": {range}, "looping": true, "on_tick": {{"event": "e"}}}}}}, "events": {{"e": {{}}}}}}"#
        )
    };
    let (found, written) = told_in("1.18.0", &ticking("[-5, 3]"), &allow_listed());
    has(
        &found,
        "core/clamped_value Info at minecraft:block/components/minecraft:ticking/range/0",
    );
    assert!(
        written.contains(r#""min_ticks": 0, "max_ticks": 60"#),
        "{written}"
    );
    assert!(!found.iter().any(|f| f.ends_with("range/1")), "{found:#?}");
    // A fraction of a tick: each end that is rounded is told.
    let (found, written) = told_in("1.18.0", &ticking("[0.01, 0.02]"), &allow_listed());
    has(
        &found,
        "core/invalid_value Info at minecraft:block/components/minecraft:ticking/range/0",
    );
    has(
        &found,
        "core/invalid_value Info at minecraft:block/components/minecraft:ticking/range/1",
    );
    assert!(
        written.contains(r#""min_ticks": 0, "max_ticks": 0"#),
        "{written}"
    );
    // The ends are sorted: each is told at the index the FILE has it at.
    let (found, _) = told_in("1.18.0", &ticking("[3, -5]"), &allow_listed());
    has(
        &found,
        "core/clamped_value Info at minecraft:block/components/minecraft:ticking/range/1",
    );
    assert!(!found.iter().any(|f| f.ends_with("range/0")), "{found:#?}");
    // Whole ticks, nothing below 0: nothing is told of the range.
    let (found, written) = told_in("1.18.0", &ticking("[1.5, 0.1]"), &allow_listed());
    assert!(!found.iter().any(|f| f.contains("/range")), "{found:#?}");
    assert!(
        written.contains(r#""min_ticks": 2, "max_ticks": 30"#),
        "{written}"
    );
    // In a permutation.
    let (found, _) = told_in(
        "1.18.0",
        r#"{"description": {"identifier": "demo:x"}, "components": {}, "permutations": [{"condition": "1", "components": {"minecraft:ticking": {"range": [-5, 0.33], "on_tick": {"event": "e"}}}}], "events": {"e": {}}}"#,
        &allow_listed(),
    );
    has(
        &found,
        "core/clamped_value Info at minecraft:block/permutations/0/components/minecraft:ticking/range/0",
    );
    has(
        &found,
        "core/invalid_value Info at minecraft:block/permutations/0/components/minecraft:ticking/range/1",
    );
}

#[test]
fn an_end_of_a_ticking_range_beyond_a_tick_count_is_bounded_and_the_block_is_kept() {
    // "The product is stored as `u32`". An end whose tick count is beyond a
    // `u32` is bounded to the largest one, with a warning at the end the file has; the block
    // is kept (numbers beyond the type are bounded with a warning, never a refusal).
    let ticking = |range: &str| {
        format!(
            r#"{{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:ticking": {{"range": {range}, "looping": true, "on_tick": {{"event": "e"}}}}}}, "events": {{"e": {{}}}}}}"#
        )
    };
    let (found, written) = told_in("1.18.0", &ticking("[1, 1e12]"), &allow_listed());
    assert_eq!(
        found
            .iter()
            .filter(|f| f.contains("/range"))
            .collect::<Vec<_>>(),
        ["core/clamped_value Info at minecraft:block/components/minecraft:ticking/range/1"],
        "{found:#?}"
    );
    assert!(
        written.contains(r#""min_ticks": 20, "max_ticks": 4294967295"#),
        "{written}"
    );
    // The bound is the largest tick count: one tick below it says nothing, one above is told.
    // 214748364.75 s * 20 = 4294967295 ticks.
    let (found, written) = told_in("1.18.0", &ticking("[0, 214748364.75]"), &allow_listed());
    assert!(!found.iter().any(|f| f.contains("/range")), "{found:#?}");
    assert!(written.contains(r#""max_ticks": 4294967295"#), "{written}");
    let (found, written) = told_in("1.18.0", &ticking("[0, 214748364.8]"), &allow_listed());
    has(
        &found,
        "core/clamped_value Info at minecraft:block/components/minecraft:ticking/range/1",
    );
    assert!(written.contains(r#""max_ticks": 4294967295"#), "{written}");
    // The ends are sorted: the large end written first is told at the index of the file.
    let (found, _) = told_in("1.18.0", &ticking("[1e12, 1]"), &allow_listed());
    has(
        &found,
        "core/clamped_value Info at minecraft:block/components/minecraft:ticking/range/0",
    );
    assert!(!found.iter().any(|f| f.ends_with("range/1")), "{found:#?}");
}

#[test]
fn an_end_of_an_interval_range_the_side_car_refuses_is_told_at_the_end() {
    // The findings of a block that loads or is rejected, as `told_in` gives them.
    let told_in = |version: &str, payload: &str, cx: &LoadContext| {
        let diagnostics = match super::common::load(&document(version, payload), cx) {
            Ok(block) => block.diagnostics().clone(),
            Err(rejected) => rejected.diagnostics,
        };
        let found: Vec<String> = diagnostics
            .iter()
            .map(|f| {
                let code: String = f.code.id().to_owned();
                format!("{code} {:?} at {}", f.severity, f.location.json_path)
            })
            .collect();
        (found, ())
    };
    // `interval_range[0]` and `[1]` become the side-car's `min_ticks` and
    // `max_ticks`. What the side-car's reader tells of them names the element of the file.
    let (found, ()) = told_in(
        "1.19.10",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:queued_ticking": {"interval_range": [3, -5], "on_tick": {"event": "e"}}}, "events": {"e": {}}}"#,
        &allow_listed(),
    );
    assert!(
        found.iter().any(|f| f
            .ends_with("at minecraft:block/components/minecraft:queued_ticking/interval_range/1")),
        "{found:#?}"
    );
    assert!(
        !found.iter().any(|f| f.contains("_ticks")),
        "a member the file does not have: {found:#?}"
    );
    // A 1.18.0 `ticking.range` whose ends the step sorted: the end of the file's index.
    let (found, ()) = told_in(
        "1.18.0",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:ticking": {"range": [1e12, 1], "on_tick": {"event": "e"}}}, "events": {"e": {}}}"#,
        &allow_listed(),
    );
    assert!(
        found
            .iter()
            .any(|f| f.ends_with("at minecraft:block/components/minecraft:ticking/range/0")),
        "{found:#?}"
    );
    assert!(
        !found.iter().any(|f| f.contains("_ticks")),
        "a member the file does not have: {found:#?}"
    );
}

#[test]
fn a_grid_size_the_1_16_100_step_removes_is_told_where_the_file_has_it() {
    // "`grid_size` is removed silently"; the input check (it must be 3)
    // looks at the base alone. In a permutation another size goes with the member: the
    // table the author asked for is not there, a warning. A 3 is the table every later
    // format has: noted.
    let (found, written) = told_in(
        "1.16.100",
        r#"{"description": {"identifier": "demo:x"}, "components": {}, "permutations": [{"condition": "1", "components": {"minecraft:crafting_table": {"grid_size": 5, "crafting_tags": ["a"], "custom_description": "t"}}}]}"#,
        &allow_listed(),
    );
    has(
        &found,
        "core/dropped_field Info at minecraft:block/permutations/0/components/minecraft:crafting_table/grid_size",
    );
    assert!(!written.contains("grid_size"), "{written}");
    let (found, written) = told_in(
        "1.16.100",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:crafting_table": {"grid_size": 3, "crafting_tags": ["a"], "custom_description": "t"}}, "permutations": [{"condition": "1", "components": {"minecraft:crafting_table": {"grid_size": 3.0, "crafting_tags": ["a"]}}}]}"#,
        &allow_listed(),
    );
    has(
        &found,
        "core/dropped_field Info at minecraft:block/components/minecraft:crafting_table/grid_size",
    );
    has(
        &found,
        "core/dropped_field Info at minecraft:block/permutations/0/components/minecraft:crafting_table/grid_size",
    );
    assert!(!written.contains("grid_size"), "{written}");
    // No `grid_size`: nothing is told of one.
    let (found, _) = told_in(
        "1.16.100",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:crafting_table": {"crafting_tags": ["a"], "custom_description": "t"}}}"#,
        &allow_listed(),
    );
    assert!(!found.iter().any(|f| f.contains("grid_size")), "{found:#?}");
}

#[test]
fn a_use_beta_features_of_a_legacy_block_is_carried() {
    // The legacy description loader reads the flag for its gate; the conversion carries it as
    // the member the newest format holds it as (an unknown one, told there).
    let mut beta = allow_listed();
    beta.beta_apis = true;
    for (value, cx) in [("true", &beta), ("false", &context()), ("7", &context())] {
        let payload = format!(
            r#"{{"description": {{"identifier": "demo:x", "use_beta_features": {value}}}, "components": {{}}}}"#
        );
        let (found, written) = told_in("1.21.0", &payload, cx);
        assert!(
            written.contains(&format!(r#""use_beta_features": {value}"#)),
            "{written}"
        );
        has(
            &found,
            "core/unrecognized_field Warning at minecraft:block/description/use_beta_features",
        );
        // In a cereal document the member is unknown, `0x4` in a strict slice: the block is
        // not loaded.
        let d = super::common::rejected(&document("1.26.20", &payload), cx);
        assert!(
            d.iter().any(
                // By legacy name: the variant and the definitions that were it.
                |f| matches!(f.code.id(), "core/unrecognized_field")
                    && f.severity == Severity::Error
                    && f.location.json_path.to_string()
                        == "minecraft:block/description/use_beta_features"
            ),
            "{d:#?}"
        );
    }
}

#[test]
fn a_legacy_event_with_a_sequence_or_a_randomize_is_carried() {
    // the legacy `events` become `deprecated_event_handlers`; a node of a
    // `sequence` or a `randomize` is a node like the event itself.
    let (found, written) = told_in(
        "1.19.40",
        r#"{"description": {"identifier": "demo:x"}, "components": {}, "events": {"e": {"sequence": [{"die": {}}, {"trigger": "e2"}, {"sequence": [{"condition": "1", "die": {}}]}], "randomize": [{"weight": 2, "die": {}}]}, "e2": {}}}"#,
        &allow_listed(),
    );
    // (the event `e` holds both lists: the game's level-2 line, and nothing of its
    // `sequence` is evaluated; the value is carried as written)
    assert_eq!(
        found
            .iter()
            .filter(|f| f.starts_with("shapes/event_both_randomize_and_sequence"))
            .count(),
        1,
        "{found:#?}"
    );
    assert!(
        found
            .iter()
            .all(|f| f.starts_with("core/molang_not_checked")
                || f.starts_with("shapes/event_both_randomize_and_sequence")),
        "{found:#?}"
    );
    assert!(
        written.contains(
            r#""deprecated_event_handlers": { "e": { "sequence": [ { "responses": [ { "die": {} } ] }, { "trigger": { "event": "e2" } }, { "sequence": [ { "condition": "1", "responses": [ { "die": {} } ] } ] } ], "randomize": [ { "probability": 2.0, "responses": [ { "die": {} } ] } ] }, "e2": {} }"#
        ),
        "{written}"
    );
    // The newest format has no such member: a cereal document that holds it is refused
    // (an unknown member of a strict slice).
    let d = super::common::rejected(
        &document(
            "1.26.20",
            r#"{"description": {"identifier": "demo:x"}, "components": {}, "deprecated_event_handlers": {"e": {}}}"#,
        ),
        &allow_listed(),
    );
    assert!(
        d.iter().any(
            // By legacy name: the variant and the definitions that were it.
            |f| matches!(f.code.id(), "core/unrecognized_field")
                && f.severity == Severity::Error
                && f.location.json_path.to_string() == "minecraft:block/deprecated_event_handlers"
        ),
        "{d:#?}"
    );
}

#[test]
fn what_is_told_of_a_legacy_event_names_the_member_of_the_file() {
    // A node the side-car's reader does not take: told at the member the file has
    // (`weight`, not the `probability` it was made), and at the event that goes with it.
    let (found, written) = told_in(
        "1.19.40",
        r#"{"description": {"identifier": "demo:x"}, "components": {}, "events": {"e": {"randomize": [{"weight": "x", "die": {}}]}, "f": {"die": {}}}}"#,
        &allow_listed(),
    );
    // The legacy events are hand-written in the game, and the side-car is ours: what is
    // told of them is an information.
    has(
        &found,
        "core/invalid_type Info at minecraft:block/events/e/randomize/0/weight",
    );
    has(
        &found,
        "core/dropped_field Info at minecraft:block/events/e",
    );
    assert!(
        !found
            .iter()
            .any(|f| f.contains("deprecated_event_handlers") || f.contains("extra")),
        "a member the file does not have: {found:#?}"
    );
    assert!(
        written.contains(r#""deprecated_event_handlers": { "f": {"#),
        "{written}"
    );
    // An unknown member of a nested trigger, and of a response moved into `responses`.
    let (found, written) = told_in(
        "1.19.40",
        r#"{"description": {"identifier": "demo:x"}, "components": {}, "events": {"e": {"sequence": [{"trigger": {"event": "e2", "zz": 3}}]}, "e2": {}}}"#,
        &allow_listed(),
    );
    has(
        &found,
        "core/unrecognized_field Info at minecraft:block/events/e/sequence/0/trigger/zz",
    );
    assert!(written.contains(r#""zz": 3"#), "{written}");
    // The newest format has no such member: the document is refused at it.
    let d = super::common::rejected(
        &document(
            "1.26.20",
            r#"{"description": {"identifier": "demo:x"}, "components": {}, "deprecated_event_handlers": {"e": {"sequence": [{"probability": "x"}]}}}"#,
        ),
        &allow_listed(),
    );
    assert!(
        d.iter().any(
            // By legacy name: the variant and the definitions that were it.
            |f| matches!(f.code.id(), "core/unrecognized_field") && f.severity == Severity::Error
        ),
        "{d:#?}"
    );
}

/// The game's 1.19.10 emission upgrade removes the component if it lacks `emission`
/// before its unwrap: an object form without `emission` is discarded
/// with the game's level-3 line, a kept error, and the block goes on.
#[test]
fn an_emission_object_without_its_emission_is_discarded_as_the_game_discards_it() {
    let (found, written) = told(
        "1.16.100",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:block_light_emission": {"a": 1}, "minecraft:friction": 0.4}}"#,
    );
    has(
        &found,
        "block/component_missing_field Error at minecraft:block/components/minecraft:block_light_emission",
    );
    assert!(!written.contains("light_emission"), "{written}");
}
