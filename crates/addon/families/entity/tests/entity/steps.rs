//! The upgrade steps, one `Upgrade::upgrade` per fixture: rows e00a
//! to e34 under `tests/fixtures/entity/entity/upgrade/` (`<name>.before.json` at the step's
//! source version, `<name>.after.json` at its target).

use bedrock_addon_core::diagnostic::{
    DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct};
#[allow(
    clippy::wildcard_imports,
    reason = "one module per struct of the chain"
)]
use bedrock_addon_family_entity::history::*;

use bedrock_addon_core::document::Content;

use super::common::{cx, fixture, json, latest_components, vanilla};

/// How a step runs.
#[derive(Clone, Copy, Default)]
pub struct Run {
    /// The document's original version; the struct's when `None`.
    pub original: Option<FormatVersion>,
    /// Section-A steps suppressed (the skip key is also read from the payload).
    pub skip_section_a: bool,
}

/// The payload of a whole document.
pub fn payload_of(document: &Value) -> Value {
    document
        .as_object()
        .and_then(|o| o.get("minecraft:entity"))
        .cloned()
        .expect("a minecraft:entity payload")
}

/// Runs `S`'s step over the document `before` (its text); the next struct's payload and the
/// step's findings.
pub fn run_text<S>(before: &str, run: Run, cx: &LoadContext) -> (Value, Diagnostics)
where
    S: Upgrade + VersionStruct,
    S::Next: VersionStruct,
{
    let document = json(before);
    let payload = payload_of(&document);
    let text = bedrock_addon_core::json::to_json(&payload).unwrap();
    let original = run.original.unwrap_or(S::VERSION);
    let mut at = JsonPath::new();
    at.push("minecraft:entity");
    let parse = ParseCx {
        load: cx,
        written: FormatVersionInput {
            version: Some(original),
            beta_literal: false,
            prerelease: false,
        },
        original_version: original,
        at,
    };
    let mut diagnostics = Diagnostics::default();
    let value = S::parse(JsonSlice::new(&text), &parse, &mut diagnostics)
        .unwrap_or_else(|| panic!("the source struct parses: {diagnostics:#?}"));
    let description = payload.as_object().and_then(|p| p.get("description"));
    let member = |key: &str| {
        description
            .and_then(|d| d.as_object())
            .and_then(|d| d.get(key))
            .and_then(Value::as_string)
            .map(|s| s.as_str().to_owned())
    };
    let identifier = member("identifier").unwrap_or_default();
    let runtime = member("runtime_identifier");
    let skip = run.skip_section_a
        || payload
            .as_object()
            .is_some_and(|p| p.get("do_not_upgrade").is_some());
    let mut upgrade = UpgradeCx {
        load: cx,
        original_version: original,
        identifier: &identifier,
        actor_type: ActorType::for_entity(&identifier, runtime.as_deref()),
        skip_section_a: skip,
        reading: cx.reading,
        diagnostics: &mut diagnostics,
    };
    let next = value.upgrade(&mut upgrade);
    // As a load does when its chain is done: what the step moved is named as the file has
    // it.
    diagnostics.name_as_the_file_has_it();
    (next.write().expect("the next struct writes"), diagnostics)
}

/// Runs `S`'s step over the fixture `<name>.before.json` and compares the result with
/// `<name>.after.json`; returns the step's findings.
pub fn step<S>(name: &str, run: Run) -> Diagnostics
where
    S: Upgrade + VersionStruct,
    S::Next: VersionStruct,
{
    let before = fixture(&format!("upgrade/{name}.before.json"));
    let after = payload_of(&json(&fixture(&format!("upgrade/{name}.after.json"))));
    let (out, diagnostics) = run_text::<S>(&before, run, &cx());
    assert_eq!(
        out,
        after,
        "{name}: the step's output\n{}\ndiffers from the fixture",
        bedrock_addon_core::json::to_json(&out).unwrap()
    );
    diagnostics
}

/// Each finding as `<path below the payload key> | <text>`: the removal texts of
/// ("`%s` was removed by the %s upgrade", ours).
fn texts(found: &Diagnostics) -> Vec<String> {
    found
        .iter()
        .map(|d| {
            let path = d.location.json_path.to_string();
            let path = path
                .strip_prefix("minecraft:entity/")
                .unwrap_or(&path)
                .to_owned();
            format!(
                "{path} | {}",
                d.to_string().rsplit(" | ").next().unwrap_or_default()
            )
        })
        .collect()
}

/// The names of the codes at warning or above, in order.
pub fn warned(diagnostics: &Diagnostics) -> Vec<&'static str> {
    super::common::codes(diagnostics, Severity::Warning)
}

/// The names of the codes of the removals the steps tell with the crate's text
/// (`RemovedByUpgrade`), in order: informations, none above.
pub fn removed(diagnostics: &Diagnostics) -> Vec<&'static str> {
    let names: Vec<&'static str> = diagnostics
        .iter()
        .filter(|d| {
            matches!(
                &d.code,
                DiagnosticCode::DroppedField {
                    text: Some(VanillaText::RemovedByUpgrade { .. })
                } | DiagnosticCode::DroppedComponent {
                    text: Some(VanillaText::RemovedByUpgrade { .. })
                }
            )
        })
        .inspect(|d| assert_eq!(d.severity, Severity::Info, "{d:?}"))
        .map(|d| d.code.name())
        .collect();
    names
}

#[test]
fn e00a_1_8_0_boss() {
    let found = step::<v1_8_0::Entity>("e00a_1_8_0_boss_wither", Run::default());
    assert!(found.is_empty(), "{found:#?}");
    step::<v1_8_0::Entity>("e00a_1_8_0_boss_ender_dragon", Run::default());
    step::<v1_8_0::Entity>("e00a_1_8_0_boss_custom", Run::default());
}

/// Runs `S`'s step over each fixture and requires no finding.
fn quiet<S>(names: &[&str])
where
    S: Upgrade + VersionStruct,
    S::Next: VersionStruct,
{
    for name in names {
        let found = step::<S>(name, Run::default());
        assert!(found.is_empty(), "{name}: {found:#?}");
    }
}

#[test]
fn e00b_1_10_0_player_hidden() {
    quiet::<v1_10_0::Entity>(&[
        "e00b_1_10_0_player_hidden",
        "e00b_1_10_0_player_hidden_kept",
        "e00b_1_10_0_player_hidden_custom",
    ]);
}

#[test]
fn e00c_1_11_0_water_movement() {
    quiet::<v1_11_0::Entity>(&[
        "e00c_1_11_0_water_movement_panda",
        "e00c_1_11_0_water_movement_polar_bear",
        "e00c_1_11_0_water_movement_turtle",
        "e00c_1_11_0_water_movement_turtle_absent",
        "e00c_1_11_0_water_movement_custom",
    ]);
}

#[test]
fn e00d_1_12_0_triggers_pushable() {
    let found = step::<v1_12_0::Entity>("e00d_1_12_0_triggers_custom", Run::default());
    // `hurt_when_wet: false` and the scheduler entry without `scheduled_event` are dropped with
    // our text; the members `on_interact` hides are lost. Each is at the member as the file
    // has it, in the element of its list; the `deals_damage: false` of the `fall` trigger that
    // `fall_damage: 0.5` replaces by `true` is told as dropped.
    assert_eq!(
        removed(&found),
        ["DroppedComponent", "DroppedField", "DroppedField"],
        "{found:#?}"
    );
    // The last is a note (INFO): the game's own fixer writes over the member.
    assert_eq!(
        texts(&found),
        [
            "components/minecraft:damage_sensor/deals_damage | field dropped",
            "component_groups/g1/minecraft:hurt_when_wet | `minecraft:hurt_when_wet` was removed by the 1.13.0 upgrade",
            "component_groups/g1/minecraft:interact/0/event | `event` was removed by the 1.13.0 upgrade",
            "component_groups/g1/minecraft:scheduler/scheduled_events/1 | `scheduled_events` was removed by the 1.13.0 upgrade",
        ]
    );
    assert_eq!(
        found.iter().next().map(|f| f.severity),
        Some(Severity::Info)
    );
    quiet::<v1_12_0::Entity>(&[
        "e00d_1_12_0_iron_golem",
        "e00d_1_12_0_zombie_experience",
        "e00d_1_12_0_sheep_experience",
        "e00d_1_12_0_salmon_flocking",
        "e00d_1_12_0_turtle_lightning_appended",
    ]);
    // The turtle's own `damage_multiplier` of the lightning trigger is the fixer's to
    // replace by 2000: noted at the trigger as the file has it, an
    // element of the sensor's list.
    let found = step::<v1_12_0::Entity>("e00d_1_12_0_turtle_lightning", Run::default());
    let noted: Vec<String> = found
        .iter()
        .map(|f| {
            format!(
                "{} {:?} {}",
                f.code.name(),
                f.severity,
                f.location.json_path
            )
        })
        .collect();
    assert_eq!(
        noted,
        [
            "DroppedField Info minecraft:entity/components/minecraft:damage_sensor/0/damage_multiplier"
        ]
    );
}

#[test]
fn e00d_the_pufferfish_gets_the_puff_state_machine_and_no_mob_family() {
    let before = r#"{"format_version": "1.12.0", "minecraft:entity": {
        "description": {"identifier": "minecraft:pufferfish"},
        "components": {"minecraft:type_family": {"family": ["fish"]}},
        "component_groups": {"minecraft:pufferfish_puffed": {}, "minecraft:pufferfish_deflated": {}, "keep": {}},
        "events": {"minecraft:on_puff": {}, "minecraft:on_deflate": {}, "minecraft:entity_spawned": {}, "keep": {}}
    }}"#;
    let (out, found) = run_text::<v1_12_0::Entity>(before, Run::default(), &cx());
    assert!(found.is_empty(), "{found:#?}");
    let payload = out.as_object().unwrap();
    let groups = payload
        .get("component_groups")
        .unwrap()
        .as_object()
        .unwrap();
    let names: Vec<&str> = groups.keys().map(facet_value::VString::as_str).collect();
    assert_eq!(
        names,
        [
            "keep",
            "minecraft:normal_puff",
            "minecraft:half_puff_inflate",
            "minecraft:half_puff_deflate",
            "minecraft:start_deflate",
            "minecraft:full_puff",
            "minecraft:deflate_sensor"
        ]
    );
    let events = payload.get("events").unwrap().as_object().unwrap();
    assert_eq!(events.len(), 7);
    let components = payload.get("components").unwrap();
    assert_eq!(
        components.as_object().unwrap().get("minecraft:type_family"),
        Some(&json(r#"{"family": ["fish"]}"#))
    );
}

/// op 8 (vanilla: "nested `minecraft:type_family`"): a string `family` becomes
/// `[s, "mob"]` at the component root; vanilla writes a nested `minecraft:type_family` member and
/// leaves `family` alone.
#[test]
fn e00d_a_string_family_gets_mob_at_the_root_deviation() {
    let before = r#"{"format_version": "1.12.0", "minecraft:entity": {
        "description": {"identifier": "x:a"},
        "components": {"minecraft:type_family": {"family": "monster"}}
    }}"#;
    let (out, found) = run_text::<v1_12_0::Entity>(before, Run::default(), &cx());
    assert!(found.is_empty(), "{found:#?}");
    let family = out
        .as_object()
        .and_then(|p| p.get("components"))
        .and_then(Value::as_object)
        .and_then(|c| c.get("minecraft:type_family"))
        .cloned()
        .unwrap();
    assert_eq!(family, json(r#"{"family": ["monster", "mob"]}"#));
}

/// section B (vanilla: "an element without `scheduled_event` becomes null"): a
/// `scheduled_events` element without `scheduled_event` is dropped with `DroppedField`; vanilla
/// leaves a `null` element.
#[test]
fn e00d_a_scheduled_event_without_its_member_is_dropped_deviation() {
    let before = r#"{"format_version": "1.12.0", "minecraft:entity": {
        "description": {"identifier": "x:a"},
        "components": {"minecraft:scheduler": {"scheduled_events": [{"scheduled_event": {"event": "x:e"}}, {"weight": 1}]}}
    }}"#;
    let (out, found) = run_text::<v1_12_0::Entity>(before, Run::default(), &cx());
    assert_eq!(removed(&found), ["DroppedField"], "{found:#?}");
    let scheduler = out
        .as_object()
        .and_then(|p| p.get("components"))
        .and_then(Value::as_object)
        .and_then(|c| c.get("minecraft:scheduler"))
        .cloned()
        .unwrap();
    assert_eq!(
        scheduler,
        json(r#"{"scheduled_events": [{"event": "x:e"}]}"#)
    );
}

#[test]
fn e00e_1_13_0_melee_chance_spawn() {
    let found = step::<v1_13_0::Entity>("e00e_1_13_0_bee_melee", Run::default());
    assert_eq!(removed(&found), ["DroppedField"; 4]);
    let melee = "components/minecraft:behavior.melee_attack";
    assert_eq!(
        texts(&found),
        [
            "look_distance",
            "untrackable_cooldown_delay",
            "target_tracking",
            "target_dist"
        ]
        .map(|m| format!("{melee}/{m} | `{m}` was removed by the 1.15.0 upgrade"))
    );
    for name in [
        "e00e_1_13_0_blaze_wolf_horse",
        "e00e_1_13_0_blaze",
        "e00e_1_13_0_wolf",
    ] {
        let found = step::<v1_13_0::Entity>(name, Run::default());
        assert_eq!(removed(&found), ["DroppedField"], "{name}");
    }
    // Diagnostics: the old member a group loses without a replacement warns.
    for name in [
        "e00e_1_13_0_circle_around_anchor",
        "e00e_1_13_0_circle_only_in_a_group",
    ] {
        let found = step::<v1_13_0::Entity>(name, Run::default());
        let expected: &[&str] = if name.ends_with("anchor") {
            &["DroppedField"]
        } else {
            &[]
        };
        assert_eq!(removed(&found), expected, "{name}");
        // The fixture's own `radius_adjustment_chance` in `components` is replaced by what
        // the fixer makes of `radius_change_chance`: told, as INFO.
        let notes: Vec<String> = found
            .iter()
            .filter(|finding| finding.severity == Severity::Info)
            .map(|finding| finding.location.json_path.to_string())
            .collect();
        // And the removal of `radius_change_chance`, an information.
        let expected_notes: &[&str] = if name.ends_with("anchor") {
            &[
                "minecraft:entity/component_groups/g/minecraft:behavior.circle_around_anchor/radius_change_chance",
                "minecraft:entity/components/minecraft:behavior.circle_around_anchor/radius_adjustment_chance",
            ]
        } else {
            &[]
        };
        assert_eq!(notes, expected_notes, "{name}");
        for finding in found
            .iter()
            .filter(|finding| finding.severity != Severity::Info)
        {
            assert_eq!(
                finding.location.json_path.to_string(),
                "minecraft:entity/component_groups/g/minecraft:behavior.circle_around_anchor/radius_change_chance"
            );
            assert_eq!(
                finding.to_string().rsplit(" | ").next(),
                Some("`radius_change_chance` was removed by the 1.15.0 upgrade")
            );
        }
    }
    quiet::<v1_13_0::Entity>(&[
        "e00e_1_13_0_fish_swim_wander",
        "e00e_1_13_0_mob_keeps_damage_sensor",
        "e00e_1_13_0_spawn_entity",
    ]);
    let found = step::<v1_13_0::Entity>("e00e_1_13_0_search_count", Run::default());
    assert_eq!(removed(&found), ["DroppedField", "DroppedField"]);
    let found = step::<v1_13_0::Entity>("e00e_1_13_0_snowball_damage_sensor", Run::default());
    let sensor = DiagnosticCode::DroppedComponent {
        text: Some(VanillaText::Fixed(
            "The component minecraft:damage_sensor will not be added to this entity as it won't work unless the format version is 1.15 or greater.",
        )),
    };
    // One finding at every location.
    assert_eq!(
        found
            .iter()
            .map(|d| (d.code.clone(), d.location.json_path.to_string()))
            .collect::<Vec<_>>(),
        [
            (
                sensor.clone(),
                "minecraft:entity/components/minecraft:damage_sensor".to_owned()
            ),
            (
                sensor,
                "minecraft:entity/component_groups/g/minecraft:damage_sensor".to_owned()
            ),
        ]
    );
}

#[test]
fn e00f_1_15_0_despawn_persistent() {
    quiet::<v1_15_0::Entity>(&[
        "e00f_1_15_0_instant_despawn",
        "e00f_1_15_0_cow_despawn",
        "e00f_1_15_0_zombie_despawn",
        "e00f_1_15_0_drowned_despawn",
        "e00f_1_15_0_cod_despawn",
        "e00f_1_15_0_persistent_no_despawn",
    ]);
    // Through the whole chain, the 1.20.80 -> 1.21.0 op (`damage_sources: ["all"]` when
    // absent) leaves the list the 1.15.0 op wrote.
    let text = fixture("upgrade/e00f_1_15_0_instant_despawn.before.json");
    let (entity, _) = super::common::stable(&text, &cx());
    let written = json(&bedrock_addon_core::document::Content::to_json(&entity).unwrap());
    let panic = payload_of(&written)
        .as_object()
        .and_then(|p| p.get("components"))
        .and_then(Value::as_object)
        .and_then(|c| c.get("minecraft:behavior.panic"))
        .and_then(Value::as_object)
        .and_then(|p| p.get("damage_sources"))
        .cloned();
    assert_eq!(panic, Some(json(r#"["fire", "fire_tick", "magma"]"#)));
}

/// The `minecraft:navigation.walk` of `text` loaded through the whole chain.
fn navigation(text: &str, cx: &LoadContext) -> (Option<Value>, Option<Value>) {
    let (entity, _) = super::common::stable(text, cx);
    let written =
        super::common::json(&bedrock_addon_core::document::Content::to_json(&entity).unwrap());
    let payload = payload_of(&written);
    let payload = payload.as_object().unwrap();
    let get = |section: Option<&Value>| {
        section
            .and_then(Value::as_object)
            .and_then(|s| s.get("minecraft:navigation.walk"))
            .cloned()
    };
    let group = payload
        .get("component_groups")
        .and_then(Value::as_object)
        .and_then(|g| g.get("g"));
    (get(payload.get("components")), get(group))
}

#[test]
fn e00g_door_annotation() {
    let document = |version: &str, member: &str, rest: &str| {
        format!(
            r#"{{"format_version": "{version}", "minecraft:entity": {{"description": {{"identifier": "demo:x"}},
            "components": {{"minecraft:navigation.walk": {{{member}}}}},
            "component_groups": {{"g": {{"minecraft:navigation.walk": {{}}}}}}{rest}}}}}"#
        )
    };
    let walk =
        |version: &str, rest: &str| document(version, r#""using_door_annotation": false"#, rest);
    // Below 1.10.0 the op writes `false` into a component that had no member, so a missing op
    // would show.
    let (components, group) = navigation(&document("1.8.0", "", ""), &cx());
    assert_eq!(
        components,
        Some(json(r#"{"using_door_annotation": false}"#))
    );
    assert_eq!(group, Some(json("{}")), "groups are not visited");
    let (components, _) = navigation(&walk("1.21.0", ""), &cx());
    assert_eq!(components, Some(json(r#"{"using_door_annotation": true}"#)));
    let (components, _) = navigation(&walk("1.21.0", r#", "do_not_upgrade": true"#), &cx());
    assert_eq!(
        components,
        Some(json(r#"{"using_door_annotation": false}"#))
    );
    let (components, _) = navigation(&walk("1.26.50", ""), &cx());
    assert_eq!(
        components,
        Some(json(r#"{"using_door_annotation": false}"#))
    );
}

#[test]
fn e00e_silverfish_attack_interval() {
    let before = r#"{"format_version": "1.13.0", "minecraft:entity": {"description": {"identifier": "minecraft:silverfish"},
        "components": {"minecraft:behavior.nearest_attackable_target": {"entity_types": [{"attack_interval": 10}]}}}}"#;
    let (out, found) = run_text::<v1_13_0::Entity>(before, Run::default(), &cx());
    let components = out.as_object().unwrap().get("components").unwrap();
    assert_eq!(
        components,
        &json(r#"{"minecraft:behavior.nearest_attackable_target": {"entity_types": [{}]}}"#)
    );
    assert_eq!(removed(&found), ["DroppedField"]);
}

#[test]
fn e01_1_16_0_restriction_split() {
    quiet::<v1_16_0::Entity>(&[
        "e01_1_16_0_restriction_split_dweller",
        "e01_1_16_0_restriction_split_guardian",
    ]);
    let found = step::<v1_16_0::Entity>("e01_1_16_0_restriction_split_custom", Run::default());
    assert_eq!(removed(&found), ["DroppedComponent", "DroppedComponent"]);
    let goal = "minecraft:behavior.move_towards_restriction";
    assert_eq!(
        texts(&found),
        ["components", "component_groups/g"]
            .map(|at| format!("{at}/{goal} | `{goal}` was removed by the 1.16.100 upgrade"))
    );
}

#[test]
fn e01b_1_16_0_boat_bubble_columns() {
    quiet::<v1_16_0::Entity>(&["e01b_1_16_0_boat_bubble_columns_custom"]);
    let before = r#"{"format_version": "1.16.0", "minecraft:entity": {"description": {"identifier": "minecraft:boat"},
        "components": {"minecraft:rideable": {"seat_count": 2}}}}"#;
    let (out, found) = run_text::<v1_16_0::Entity>(before, Run::default(), &cx());
    assert!(found.is_empty(), "{found:#?}");
    let payload = out.as_object().unwrap();
    let groups = payload
        .get("component_groups")
        .unwrap()
        .as_object()
        .unwrap();
    for name in [
        "minecraft:floating",
        "minecraft:can_ride",
        "minecraft:above_bubble_column_down",
        "minecraft:above_bubble_column_up",
    ] {
        assert!(groups.contains_key(name), "{name}");
    }
    assert_eq!(
        groups.get("minecraft:can_ride"),
        Some(&json(r#"{"minecraft:rideable": {"seat_count": 2}}"#))
    );
    let components = payload.get("components").unwrap().as_object().unwrap();
    for name in [
        "minecraft:buoyant",
        "minecraft:timer",
        "minecraft:out_of_control",
        "minecraft:inside_block_notifier",
    ] {
        assert!(components.contains_key(name), "{name}");
    }
    assert_eq!(payload.get("events").unwrap().as_object().unwrap().len(), 4);
}

#[test]
fn e01c_1_16_0_component_rewrites() {
    let found = step::<v1_16_0::Entity>("e01c_1_16_0_component_rewrites", Run::default());
    assert_eq!(removed(&found), ["DroppedComponent"]);
    assert_eq!(
        texts(&found),
        [
            "components/minecraft:foot_size | `minecraft:foot_size` was removed by the 1.16.100 upgrade"
        ]
    );
}

#[test]
fn e02_1_16_100_knockback_roar_v1() {
    quiet::<v1_16_100::Entity>(&["e02_1_16_100_knockback_roar_v1"]);
}

#[test]
fn e03_1_16_200_block_climber() {
    quiet::<v1_16_200::Entity>(&["e03_1_16_200_block_climber"]);
}

#[test]
fn e04_1_17_10_knockback_roar_v2() {
    quiet::<v1_17_10::Entity>(&["e04_1_17_10_knockback_roar_v2"]);
}

#[test]
fn e05_1_17_20_controlling_seat() {
    let found = step::<v1_17_20::Entity>("e05_1_17_20_controlling_seat", Run::default());
    assert_eq!(removed(&found), ["DroppedField"]);
    assert_eq!(
        texts(&found),
        [
            "components/minecraft:rideable/controlling_seat | `controlling_seat` was removed by the 1.18.0 upgrade"
        ]
    );
}

#[test]
fn e06_1_18_0_ocelot_events() {
    quiet::<v1_18_0::Entity>(&[
        "e06_1_18_0_ocelot_events",
        "e06_1_18_0_ocelot_no_event",
        "e06_1_18_0_ocelot_events_custom",
    ]);
}

#[test]
fn e07_1_18_10_go_home_arrays() {
    quiet::<v1_18_10::Entity>(&["e07_1_18_10_go_home_arrays"]);
}

#[test]
fn e08_1_18_20_player_exhaustion_deviation() {
    // Vanilla writes `""`; we write `{}`, which reads as the same defaults.
    quiet::<v1_18_20::Entity>(&[
        "e08_1_18_20_player_exhaustion",
        "e08_1_18_20_player_exhaustion_kept",
    ]);
}

#[test]
fn e09_1_18_30_sleep_riding() {
    quiet::<v1_18_30::Entity>(&["e09_1_18_30_sleep_riding"]);
}

#[test]
fn e10_1_19_0_flowers_auto_step() {
    quiet::<v1_19_0::Entity>(&[
        "e10_1_19_0_flowers_and_auto_step",
        "e10_1_19_0_horse_untouched",
        "e10_1_19_0_enderman_auto_step",
    ]);
}

#[test]
fn e10b_1_19_0_slime_area_attack() {
    quiet::<v1_19_0::Entity>(&[
        "e10b_1_19_0_magma_cube_area_attack",
        "e10b_1_19_0_slime_area_attack",
    ]);
    // A size group the document lacks is not created.
    let before = r#"{"format_version": "1.19.0", "minecraft:entity": {"description": {"identifier": "minecraft:slime"},
        "components": {}, "component_groups": {"minecraft:slime_small": {}}}}"#;
    let (out, found) = run_text::<v1_19_0::Entity>(before, Run::default(), &cx());
    assert!(found.is_empty(), "{found:#?}");
    let groups = out
        .as_object()
        .and_then(|p| p.get("component_groups"))
        .and_then(Value::as_object)
        .unwrap();
    let names: Vec<&str> = groups.keys().map(facet_value::VString::as_str).collect();
    assert_eq!(names, ["minecraft:slime_small"]);
}

#[test]
fn e11_1_19_50_trade_with_player() {
    quiet::<v1_19_50::Entity>(&["e11_1_19_50_trade_with_player"]);
}

#[test]
fn e11b_1_19_50_witch() {
    let before = r#"{"format_version": "1.19.50", "minecraft:entity": {"description": {"identifier": "minecraft:witch"},
        "components": {"minecraft:shooter": {"def": "minecraft:splash_potion", "aux_val": 23}}}}"#;
    let (out, found) = run_text::<v1_19_50::Entity>(before, Run::default(), &cx());
    // The document's own shooter is replaced by the fixer's: told, as INFO.
    assert!(warned(&found).is_empty(), "{found:#?}");
    assert_eq!(
        found
            .iter()
            .map(|d| (d.severity, d.location.json_path.to_string()))
            .collect::<Vec<_>>(),
        [(
            Severity::Info,
            "minecraft:entity/components/minecraft:shooter".to_owned()
        )]
    );
    let components = out
        .as_object()
        .unwrap()
        .get("components")
        .unwrap()
        .as_object()
        .unwrap();
    let shooter = components
        .get("minecraft:shooter")
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(shooter.get("power"), Some(&json("0.75")));
    assert_eq!(shooter.get("aux_val"), Some(&json("23")));
    let projectiles = shooter.get("projectiles").unwrap().as_array().unwrap();
    let aux: Vec<_> = projectiles
        .iter()
        .map(|p| p.as_object().unwrap().get("aux_val").cloned().unwrap())
        .collect();
    assert_eq!(aux, [json("17"), json("25"), json("34")]);
    let potions = components
        .get("minecraft:behavior.drink_potion")
        .unwrap()
        .as_object()
        .unwrap()
        .get("potions")
        .unwrap()
        .as_array()
        .unwrap();
    let ids: Vec<_> = potions
        .iter()
        .map(|p| p.as_object().unwrap().get("id").cloned().unwrap())
        .collect();
    assert_eq!(ids, [json("19"), json("12"), json("21"), json("14")]);
    // The `_custom` twin: the same shooter on a custom entity is left alone.
    let twin = before.replace("minecraft:witch", "demo:custom");
    let (out, found) = run_text::<v1_19_50::Entity>(&twin, Run::default(), &cx());
    assert!(found.is_empty(), "{found:#?}");
    assert_eq!(
        out.as_object().unwrap().get("components"),
        Some(&json(
            r#"{"minecraft:shooter": {"def": "minecraft:splash_potion", "aux_val": 23}}"#
        ))
    );
}

#[test]
fn e12_1_19_60_boostable() {
    quiet::<v1_19_60::Entity>(&["e12_1_19_60_boostable"]);
}

#[test]
fn e13_1_20_0_entity_sensor_subsensors() {
    quiet::<v1_20_0::Entity>(&["e13_1_20_0_entity_sensor_subsensors"]);
}

#[test]
fn e14_1_20_60_subsensor_range() {
    quiet::<v1_20_60::Entity>(&["e14_1_20_60_subsensor_range"]);
}

#[test]
fn e15_1_20_70_interact_slots_spawn_category() {
    quiet::<v1_20_70::Entity>(&[
        "e15_1_20_70_interact_slots_spawn_category",
        "e15_1_20_70_spawn_category_custom",
        "e15_1_20_70_spawn_category_kept",
    ]);
    // A null `description` gets nothing: the source struct refuses the payload (no
    // identifier), so the step never runs and the file is skipped, as vanilla
    // skips an entity without an identifier.
    let text = r#"{"format_version": "1.20.70", "minecraft:entity": {"description": null, "components": {}}}"#;
    let outcome = super::common::outcome(text, &cx());
    assert!(outcome.value.is_none());
    assert_eq!(
        super::common::codes(&outcome.diagnostics, Severity::Warning),
        ["MissingIdentifier"]
    );
}

#[test]
fn e16_1_20_80_panic_sources() {
    quiet::<v1_20_80::Entity>(&["e16_1_20_80_panic_sources"]);
}

#[test]
fn e17_1_21_0_armor_slot_mapping() {
    quiet::<v1_21_0::Entity>(&["e17_1_21_0_armor_slot_mapping"]);
}

#[test]
fn e18_1_21_10_follow_owner_dolphin() {
    quiet::<v1_21_10::Entity>(&["e18_1_21_10_follow_owner_dolphin"]);
}

#[test]
fn e19_1_21_20_looked_at_home_damage_sensor() {
    let found =
        step::<v1_21_20::Entity>("e19_1_21_20_looked_at_home_damage_sensor", Run::default());
    assert_eq!(removed(&found), ["DroppedField"]);
    assert_eq!(
        texts(&found),
        [
            "components/minecraft:lookat/allow_invulnerable | `allow_invulnerable` was removed by the 1.21.40 upgrade"
        ]
    );
    quiet::<v1_21_20::Entity>(&["e19_1_21_20_fishing_hook", "e19_1_21_20_arrow"]);
}

#[test]
fn e20_1_21_40_renders_when_invisible() {
    quiet::<v1_21_40::Entity>(&["e20_1_21_40_spider", "e20_1_21_40_strider"]);
}

#[test]
fn e21_1_21_60_float_wander_collidable() {
    quiet::<v1_21_60::Entity>(&["e21_1_21_60_float_wander", "e21_1_21_60_boat_collidable"]);
}

#[test]
fn e22_1_21_70_leashable_presets() {
    quiet::<v1_21_70::Entity>(&["e22_1_21_70_leashable_presets"]);
}

#[test]
fn e23_1_21_80_null_groups() {
    let found = step::<v1_21_80::Entity>("e23_1_21_80_null_groups", Run::default());
    assert_eq!(
        found.iter().map(|d| d.code.clone()).collect::<Vec<_>>(),
        [DiagnosticCode::DroppedComponent {
            text: Some(VanillaText::NullGroupRemoved {
                group: "gone".to_owned()
            })
        }]
    );
}

#[test]
fn e24_1_21_89_leashable_breed() {
    quiet::<v1_21_89::Entity>(&["e24_1_21_89_leashable_breed"]);
}

#[test]
fn e25_1_21_90_enderman_renames() {
    // The fixture's own `chance: 9` is replaced by the step's 0.05: told as dropped.
    let found = step::<v1_21_90::Entity>("e25_1_21_90_enderman_renames", Run::default());
    assert_eq!(
        texts(&found),
        ["components/minecraft:behavior.enderman_take_block/chance | field dropped"]
    );
}

#[test]
fn e26_1_21_100_free_camera_dash() {
    quiet::<v1_21_100::Entity>(&["e26_1_21_100_free_camera_dash"]);
}

#[test]
fn e27_1_21_110_projectile_markers() {
    quiet::<v1_21_110::Entity>(&["e27_1_21_110_projectile_markers"]);
}

#[test]
fn e28_1_21_120_ageable_breedable_interval() {
    quiet::<v1_21_120::Entity>(&["e28_1_21_120_ageable_breedable_interval"]);
}

#[test]
fn e29_1_21_130_breedable_split() {
    quiet::<v1_21_130::Entity>(&["e29_1_21_130_breedable_split"]);
}

#[test]
fn e29b_1_21_130_look_at_interact_jump() {
    // `target_distance` is removed without a replacement: a `DroppedField` information, the
    // game's upgrade drops it silently.
    // The fixture's own `regular_skip_data` is replaced by the step's: told as dropped
    // (a note, INFO: the game's step writes over it).
    let found = step::<v1_21_130::Entity>("e29b_1_21_130_look_at_interact_jump", Run::default());
    assert_eq!(warned(&found), Vec::<&str>::new());
    let noted: Vec<String> = found
        .iter()
        .filter(|f| f.severity == Severity::Info)
        .map(|f| format!("{} {}", f.code.name(), f.location.json_path))
        .collect();
    assert!(
        noted.contains(
            &"DroppedField minecraft:entity/components/minecraft:jump.dynamic/regular_skip_data"
                .to_owned()
        ),
        "{noted:?}"
    );
    assert!(
        noted
            .iter()
            .any(|n| n.starts_with("DroppedField ") && n.ends_with("/target_distance")),
        "{noted:?}"
    );
    assert_eq!(
        found
            .iter()
            .next()
            .unwrap()
            .to_string()
            .rsplit(" | ")
            .next(),
        Some("`target_distance` was removed by the 1.26.0 upgrade")
    );
}

#[test]
fn e29c_1_21_130_typed_to_typed() {
    quiet::<v1_21_130::Entity>(&["e29c_1_21_130_typed_to_typed"]);
}

#[test]
fn e30_1_26_0_buoyant_pushable() {
    quiet::<v1_26_0::Entity>(&["e30_1_26_0_buoyant_pushable"]);
}

#[test]
fn e31_1_26_10_legacy_friction() {
    quiet::<v1_26_10::Entity>(&["e31_1_26_10_legacy_friction"]);
    // Skipped with `do_not_upgrade`.
    let before = r#"{"format_version": "1.26.10", "minecraft:entity": {"description": {"identifier": "demo:x"},
        "components": {}, "do_not_upgrade": true}}"#;
    let (out, _) = run_text::<v1_26_10::Entity>(before, Run::default(), &cx());
    assert_eq!(
        out.as_object().unwrap().get("components"),
        Some(&json("{}"))
    );
}

#[test]
fn e31b_1_26_10_knockback_rules_vanilla_deviation() {
    // 1.26.20 writes 2.0 / 1.3 / 1.5 / 1.0 with a runtime ×0.4; we write the products.
    quiet::<v1_26_10::Entity>(&["e31b_1_26_10_knockback_rules_egg"]);
    // The drowned's own `apply_knockback_rules` is replaced by the fixer's: told, as INFO.
    let found = step::<v1_26_10::Entity>("e31b_1_26_10_knockback_rules_drowned", Run::default());
    assert!(warned(&found).is_empty(), "{found:#?}");
    assert_eq!(
        found
            .iter()
            .map(|d| (d.severity, d.location.json_path.to_string()))
            .collect::<Vec<_>>(),
        [(
            Severity::Info,
            "minecraft:entity/components/minecraft:apply_knockback_rules".to_owned()
        )]
    );
    // The iron golem of at 1.19.0: the constants of 1.18.30 and later.
    let at_1_19_0 = Run {
        original: Some(FormatVersion::new(1, 19, 0)),
        skip_section_a: false,
    };
    let found = step::<v1_26_10::Entity>("e31b_1_26_10_knockback_rules_iron_golem", at_1_19_0);
    assert!(found.is_empty(), "{found:#?}");
    // An iron golem originally below 1.18.30 gets the older constants.
    let before = r#"{"format_version": "1.26.10", "minecraft:entity": {"description": {"identifier": "minecraft:iron_golem"}, "components": {}}}"#;
    let old = Run {
        original: Some(FormatVersion::new(1, 16, 0)),
        skip_section_a: false,
    };
    let (out, _) = run_text::<v1_26_10::Entity>(before, old, &cx());
    let rules = out
        .as_object()
        .unwrap()
        .get("components")
        .unwrap()
        .as_object()
        .unwrap()
        .get("minecraft:apply_knockback_rules")
        .cloned();
    assert_eq!(
        rules,
        Some(json(
            r#"{"presets": [{"horizontal_power": 0.8, "vertical_power": 1.6, "vertical_velocity_cap": 0.8, "check_if_target_is_immersed_in_water": false}]}"#
        ))
    );
}

#[test]
fn e31c_1_26_10_follow_target_leader() {
    quiet::<v1_26_10::Entity>(&["e31c_1_26_10_follow_target_leader"]);
}

#[test]
fn e32_1_26_20_to_1_26_30() {
    quiet::<v1_26_20::Entity>(&["e32_1_26_20_to_1_26_30"]);
}

#[test]
fn e33_1_26_30_on_kill_overlap() {
    // The `on_attack` of `melee_attack` that its `on_kill` replaces is told as dropped.
    let found = step::<v1_26_30::Entity>("e33_1_26_30_on_kill_overlap", Run::default());
    assert_eq!(
        found.iter().map(|d| d.code.clone()).collect::<Vec<_>>(),
        [
            DiagnosticCode::DroppedField { text: None },
            DiagnosticCode::InvalidValue {
                text: Some(VanillaText::NotAValidValue {
                    value: "bogus".to_owned(),
                    member: "should_bounce"
                })
            }
        ]
    );
    assert_eq!(
        found.iter().next().unwrap().location.json_path.to_string(),
        "minecraft:entity/components/minecraft:behavior.melee_attack/on_attack"
    );
}

#[test]
fn e33b_1_26_30_shape_migrations() {
    quiet::<v1_26_30::Entity>(&["e33b_1_26_30_shape_migrations"]);
    // Both forms: the pair wins, the scalar is dropped.
    let before = r#"{"format_version": "1.26.30", "minecraft:entity": {"description": {"identifier": "demo:x"},
        "components": {"minecraft:behavior.ranged_attack": {"attack_interval": 2, "attack_interval_min": 1}}}}"#;
    let (out, found) = run_text::<v1_26_30::Entity>(before, Run::default(), &cx());
    assert_eq!(removed(&found), ["DroppedField"]);
    assert_eq!(
        texts(&found),
        [
            "components/minecraft:behavior.ranged_attack/attack_interval | `attack_interval` was removed by the 1.26.40 upgrade"
        ]
    );
    let goal = out
        .as_object()
        .unwrap()
        .get("components")
        .unwrap()
        .as_object()
        .unwrap()
        .get("minecraft:behavior.ranged_attack")
        .cloned();
    assert_eq!(
        goal,
        Some(json(r#"{"attack_interval": {"min": 1, "max": 0}}"#))
    );
}

#[test]
fn e34_1_26_40_ranged_projectile() {
    let before = r#"{"format_version": "1.26.40", "minecraft:entity": {"description": {"identifier": "demo:x"},
        "components": {"minecraft:behavior.ranged_attack": {"priority": 2, "attack_radius": 15, "attack_radius_min": 4, "attack_interval": {"min": 1, "max": 3}},
                       "minecraft:projectile": {"anchor": 1}},
        "component_groups": {"a": {"minecraft:projectile": {"anchor": 0}}, "b": {"minecraft:projectile": {"anchor": 2}}, "c": {"minecraft:projectile": {"anchor": 7}}}}}"#;
    let (out, found) = run_text::<v1_26_40::Entity>(before, Run::default(), &cx());
    let payload = out.as_object().unwrap();
    // The step converts to the typed structs: the expected members are compared as the newest
    // struct writes them.
    assert_eq!(
        payload.get("components"),
        Some(&latest_components(
            r#"{"minecraft:behavior.ranged_attack": {"priority": 2, "attack_range": {"min": 4, "max": 15}, "attack_interval": {"min": 1, "max": 3}, "in_range_movement_mode": "hold_position"},
                "minecraft:projectile": {"anchor": "eye_height", "multiple_targets": true}, "minecraft:uses_legacy_ambient_sounds": {}}"#
        ))
    );
    let groups = payload
        .get("component_groups")
        .unwrap()
        .as_object()
        .unwrap();
    for (name, anchor) in [("a", "origin"), ("b", "middle"), ("c", "origin")] {
        assert_eq!(
            groups.get(name),
            Some(&latest_components(&format!(
                r#"{{"minecraft:projectile": {{"anchor": "{anchor}", "multiple_targets": true}}}}"#
            ))),
            "{name}"
        );
    }
    assert_eq!(
        found
            .iter()
            .filter(|d| d.code.name() == "InvalidValue")
            .count(),
        1,
        "{found:#?}"
    );
}

/// op 7: the pufferfish's `movement` / `underwater_movement` 0.13 are inserted when
/// absent; an author's value is kept.
#[test]
fn e00d_the_pufferfish_movement_is_inserted_only_when_absent() {
    let before = r#"{"format_version": "1.12.0", "minecraft:entity": {
        "description": {"identifier": "minecraft:pufferfish"},
        "components": {"minecraft:movement": {"value": 0.2}}
    }}"#;
    let (out, _) = run_text::<v1_12_0::Entity>(before, Run::default(), &cx());
    let components = out
        .as_object()
        .and_then(|p| p.get("components"))
        .and_then(Value::as_object)
        .cloned()
        .unwrap();
    assert_eq!(
        components.get("minecraft:movement"),
        Some(&json(r#"{"value": 0.2}"#))
    );
    assert_eq!(
        components.get("minecraft:underwater_movement"),
        Some(&json(&format!(r#"{{"value": {}}}"#, f64::from(0.13_f32))))
    );
}

/// preamble: the `_custom` twin of a vanilla-only fixture is the same document with
/// a custom identifier (`ActorType::Mob`); it comes out of the step unchanged except for the
/// switch default branches that apply to `Mob`. Runs `S`'s step over `<name>.before.json` with
/// the identifier `demo:custom` and returns the payload before and after.
fn custom_twin<S>(name: &str) -> (Value, Value)
where
    S: Upgrade + VersionStruct,
    S::Next: VersionStruct,
{
    let before = fixture(&format!("upgrade/{name}.before.json"));
    let mut document = json(&before);
    let identifier = document
        .as_object_mut()
        .and_then(|d| d.get_mut("minecraft:entity"))
        .and_then(Value::as_object_mut)
        .and_then(|p| p.get_mut("description"))
        .and_then(Value::as_object_mut)
        .expect("a description");
    identifier.insert("identifier", Value::from("demo:custom"));
    let text = bedrock_addon_core::json::to_json(&document).unwrap();
    let (out, found) = run_text::<S>(&text, Run::default(), &cx());
    assert!(found.is_empty(), "{name}_custom: {found:#?}");
    let mut input = payload_of(&document);
    // The step's output carries the description as read.
    if let (Some(out), Some(input)) = (out.as_object(), input.as_object_mut())
        && let Some(description) = out.get("description")
    {
        input.insert("description", description.clone());
    }
    (input, out)
}

/// Each listed vanilla-only fixture's `_custom` twin comes out of its step unchanged.
fn unchanged_twins<S>(names: &[&str])
where
    S: Upgrade + VersionStruct,
    S::Next: VersionStruct,
{
    for name in names {
        let (input, out) = custom_twin::<S>(name);
        assert_eq!(
            out,
            input,
            "{name}_custom: the step changed a custom entity:\n{}",
            bedrock_addon_core::json::to_json(&out).unwrap()
        );
    }
}

#[test]
fn custom_twins_come_out_unchanged() {
    unchanged_twins::<v1_18_20::Entity>(&["e08_1_18_20_player_exhaustion"]);
    unchanged_twins::<v1_19_0::Entity>(&[
        "e10b_1_19_0_magma_cube_area_attack",
        "e10b_1_19_0_slime_area_attack",
    ]);
    // e18: the dolphin op is vanilla-only; the B op on `follow_owner` applies to every type.
    let (_, out) = custom_twin::<v1_21_10::Entity>("e18_1_21_10_follow_owner_dolphin");
    assert_eq!(
        out.as_object().unwrap().get("components"),
        Some(&json(
            r#"{"minecraft:behavior.follow_owner": {"speed_multiplier": 1, "post_teleport_distance": 1}}"#
        ))
    );
    unchanged_twins::<v1_21_20::Entity>(&["e19_1_21_20_fishing_hook", "e19_1_21_20_arrow"]);
    unchanged_twins::<v1_21_40::Entity>(&["e20_1_21_40_spider", "e20_1_21_40_strider"]);
    unchanged_twins::<v1_21_60::Entity>(&["e21_1_21_60_boat_collidable"]);
}

/// e33b: the old shapes (`beg.look_time` and `float_wander.float_duration` arrays,
/// `ranged_attack.attack_interval_min`) in a 1.26.40 document are strict errors.
#[test]
fn e33b_the_old_shapes_are_strict_errors_at_1_26_40() {
    for (goal, members) in [
        ("minecraft:behavior.beg", r#""look_time": [2, 4]"#),
        (
            "minecraft:behavior.float_wander",
            r#""float_duration": [0.1, 0.35]"#,
        ),
        (
            "minecraft:behavior.ranged_attack",
            r#""attack_interval_min": 1"#,
        ),
    ] {
        let text = format!(
            r#"{{"format_version": "1.26.40", "minecraft:entity": {{"description": {{"identifier": "demo:x"}}, "components": {{"{goal}": {{"priority": 1, {members}}}}}}}}}"#
        );
        let outcome = super::common::outcome(&text, &cx());
        let errors: Vec<String> = outcome
            .diagnostics
            .iter()
            .filter(|d| d.severity >= Severity::Error)
            .map(|d| d.location.json_path.to_string())
            .collect();
        assert!(
            errors
                .iter()
                .any(|at| at.starts_with(&format!("minecraft:entity/components/{goal}"))),
            "{goal}: {:#?}",
            outcome.diagnostics
        );
        // The same members at 1.26.30 convert without an error.
        let older = text.replace("1.26.40", "1.26.30");
        let outcome = super::common::outcome(&older, &cx());
        assert!(
            super::common::codes(&outcome.diagnostics, Severity::Error).is_empty(),
            "{goal} at 1.26.30: {:#?}",
            outcome.diagnostics
        );
    }
}

// ---- a step that reads a value as its default says so ----

/// The upgrade-phase `InvalidType` warnings of `components` at `version`, by path, and the
/// written entity.
fn misread(version: &str, components: &str) -> (Vec<String>, String) {
    use bedrock_addon_core::diagnostic::Phase;
    use bedrock_addon_core::document::Content;
    let text = super::common::doc(
        version,
        &format!(r#"{{"description": {{"identifier": "demo:x"}}, "components": {components}}}"#),
    );
    let (entity, found) = super::common::stable(&text, &cx());
    let told = found
        .iter()
        .filter(|f| {
            f.phase == Phase::Upgrade
                && f.severity == Severity::Info
                && matches!(f.code, DiagnosticCode::InvalidType { .. })
        })
        .map(|f| f.location.json_path.to_string())
        .collect();
    (told, entity.to_json().expect("the entity writes"))
}

#[test]
fn a_simulate_waves_that_is_no_bool_is_read_as_false_and_told() {
    let at = "minecraft:entity/components/minecraft:buoyant/simulate_waves";
    for value in [r#""true""#, "1", "null"] {
        let (told, written) = misread(
            "1.26.0",
            &format!(r#"{{"minecraft:buoyant": {{"simulate_waves": {value}}}}}"#),
        );
        assert_eq!(told, [at], "{value}");
        assert!(
            written.contains(r#""movement_type": "none""#),
            "{value}: {written}"
        );
    }
    for (value, movement) in [("true", "waves"), ("false", "none")] {
        let (told, written) = misread(
            "1.26.0",
            &format!(r#"{{"minecraft:buoyant": {{"simulate_waves": {value}}}}}"#),
        );
        assert!(told.is_empty(), "{value}: {told:?}");
        assert!(
            written.contains(&format!(r#""movement_type": "{movement}""#)),
            "{value}: {written}"
        );
    }
}

#[test]
fn a_pushable_member_that_is_no_bool_is_read_as_true_and_told() {
    let base = "minecraft:entity/components/minecraft:pushable";
    for (pushable, at) in [
        (r#"{"is_pushable": 0}"#, format!("{base}/is_pushable")),
        (
            r#"{"is_pushable_by_piston": "no"}"#,
            format!("{base}/is_pushable_by_piston"),
        ),
        ("7", base.to_owned()),
        ("false", base.to_owned()),
    ] {
        let (told, written) = misread(
            "1.26.0",
            &format!(r#"{{"minecraft:pushable": {pushable}}}"#),
        );
        assert_eq!(told, [at], "{pushable}");
        assert!(
            written.contains("minecraft:pushable_by_entity")
                && written.contains("minecraft:pushable_by_block"),
            "{pushable}: {written}"
        );
    }
    let (told, written) = misread(
        "1.26.0",
        r#"{"minecraft:pushable": {"is_pushable": false, "is_pushable_by_piston": true}}"#,
    );
    assert!(told.is_empty(), "{told:?}");
    assert!(
        !written.contains("minecraft:pushable_by_entity")
            && written.contains("minecraft:pushable_by_block"),
        "{written}"
    );
}

#[test]
fn a_semi_random_diff_damage_that_is_no_bool_is_read_as_false_and_told() {
    let at = "minecraft:entity/components/minecraft:projectile/on_hit/impact_damage/semi_random_diff_damage";
    let projectile = |value: &str| {
        format!(
            r#"{{"minecraft:projectile": {{"on_hit": {{"impact_damage": {{"damage": 1, "semi_random_diff_damage": {value}}}}}}}}}"#
        )
    };
    for value in [r#""true""#, "1"] {
        let (told, written) = misread("1.26.30", &projectile(value));
        assert_eq!(told, [at], "{value}");
        assert!(
            written.contains(r#""difficulty_randomization": "none""#),
            "{value}: {written}"
        );
    }
    for (value, randomization) in [("true", "additive"), ("false", "none")] {
        let (told, written) = misread("1.26.30", &projectile(value));
        assert!(told.is_empty(), "{value}: {told:?}");
        assert!(
            written.contains(&format!(r#""difficulty_randomization": "{randomization}""#)),
            "{value}: {written}"
        );
    }
}

// ---- no step and no legacy reader rewrites or drops a value in silence ----

/// Every finding of the entity `identifier` with `components` at `version`, as
/// `<code> <severity> <path below components>`, and the written entity.
fn said(version: &str, identifier: &str, components: &str) -> (Vec<String>, String) {
    use bedrock_addon_core::document::Content;
    let text = super::common::doc(
        version,
        &format!(
            r#"{{"description": {{"identifier": "{identifier}"}}, "components": {components}}}"#
        ),
    );
    let (entity, found) = super::common::stable(&text, &cx());
    let said = found
        .iter()
        .map(|f| {
            let path = f.location.json_path.to_string();
            let path = path
                .strip_prefix("minecraft:entity/components/")
                .unwrap_or(&path);
            format!("{} {:?} {path}", f.code.name(), f.severity)
        })
        .collect();
    (said, entity.to_json().expect("the entity writes"))
}

/// The list: each value a step or the last step's typed read took as
/// another value, replaced or dropped is told at the member as the file has it, and the
/// entity holds what it held before.
#[test]
#[expect(clippy::too_many_lines, reason = "one row per case of the table")]
fn a_value_a_step_or_the_typed_read_takes_as_another_is_told_at_the_member() {
    // (version, identifier, components, the finding, what the written entity holds)
    let rows: &[(&str, &str, &str, &str, &str)] = &[
        (
            "1.21.120",
            "demo:x",
            r#"{"minecraft:breedable": {"breeds_with": [{"mate_type": 7, "baby_type": "demo:x", "breed_event": {"event": "e"}}]}}"#,
            "InvalidType Info minecraft:breedable/breeds_with/0/mate_type",
            r#""breeds_with": {}"#,
        ),
        (
            "1.21.120",
            "demo:x",
            r#"{"minecraft:breedable": {"breeds_with": "demo:x"}}"#,
            "InvalidType Info minecraft:breedable/breeds_with",
            r#""breeds_with": {}"#,
        ),
        (
            "1.21.130",
            "demo:x",
            r#"{"minecraft:breedable": {"breeds_with": {"baby_type": "demo:x"}}}"#,
            "MissingRequiredField Info minecraft:breedable/breeds_with",
            r#""breeds_with": {}"#,
        ),
        (
            "1.21.130",
            "demo:x",
            r#"{"minecraft:breedable": {"breeds_with": [{"mate_type": "demo:x", "weight": 2}]}}"#,
            "UnrecognizedField Info minecraft:breedable/breeds_with/0/weight",
            r#""breeds_with": { "demo:x": {} }"#,
        ),
        (
            "1.21.120",
            "demo:x",
            r#"{"minecraft:ageable": {"feed_items": "wheat", "transform_to_item": "bowl"}}"#,
            "DroppedField Info minecraft:ageable/transform_to_item",
            r#""minecraft:ageable": {"#,
        ),
        (
            "1.21.120",
            "demo:x",
            // No `feed_items` at all (vanilla's own axolotl): nothing is fed, so nothing is
            // lost but the member; the game drops it without a word: INFO.
            r#"{"minecraft:ageable": {"transform_to_item": "bowl"}}"#,
            "DroppedField Info minecraft:ageable/transform_to_item",
            r#""minecraft:ageable": {"#,
        ),
        (
            "1.12.0",
            "demo:x",
            r#"{"minecraft:fall_damage": {"value": "2"}}"#,
            "InvalidType Info minecraft:fall_damage/value",
            r#""deals_damage": "no""#,
        ),
        (
            "1.12.0",
            "demo:x",
            r#"{"minecraft:damage_sensor": [{"cause": "fall", "deals_damage": false}], "minecraft:fall_damage": {"value": 2}}"#,
            "DroppedField Info minecraft:damage_sensor/0/deals_damage",
            r#""damage_multiplier": 2.0"#,
        ),
        (
            "1.16.0",
            "demo:x",
            r#"{"minecraft:behavior.explore_outskirts": {"wait_time": "600"}}"#,
            "InvalidType Info minecraft:behavior.explore_outskirts/wait_time",
            r#""max_wait_time": 0.0"#,
        ),
        (
            "1.16.0",
            "demo:x",
            r#"{"minecraft:behavior.explore_outskirts": {"wait_time": 1e39}}"#,
            "InvalidValue Info minecraft:behavior.explore_outskirts/wait_time",
            r#""max_wait_time": 1"#,
        ),
        (
            "1.13.0",
            "demo:x",
            r#"{"minecraft:behavior.circle_around_anchor": {"radius_change_chance": "5", "height_change_chance": 4}}"#,
            "InvalidType Info minecraft:behavior.circle_around_anchor/radius_change_chance",
            r#""radius_adjustment_chance": 1.0"#,
        ),
        (
            "1.13.0",
            "demo:x",
            r#"{"minecraft:behavior.circle_around_anchor": {"radius_change_chance": 5, "height_change_chance": 0.5}}"#,
            "InvalidValue Info minecraft:behavior.circle_around_anchor/height_change_chance",
            r#""height_adjustment_chance": 1.0"#,
        ),
        (
            "1.13.0",
            "minecraft:cod",
            r#"{"minecraft:behavior.swim_wander": {"interval": "x"}}"#,
            "InvalidType Info minecraft:behavior.swim_wander/interval",
            r#""interval": 1.0"#,
        ),
        (
            "1.21.70",
            "demo:x",
            r#"{"minecraft:leashable": {"soft_distance": 2, "presets": {"soft_distance": 4}}}"#,
            "InvalidType Info minecraft:leashable/presets",
            r#""presets": [ { "soft_distance": 2, "spring_type": "bouncy""#,
        ),
        (
            "1.19.0",
            "demo:x",
            r#"{"minecraft:behavior.take_flower": {"filters": {"all_of": {"test": "is_family", "value": "x"}}}}"#,
            "InvalidType Info minecraft:behavior.take_flower/filters/all_of",
            r#""all_of": [ { "test": "is_daytime", "value": true } ]"#,
        ),
        (
            "1.19.0",
            "demo:x",
            r#"{"minecraft:behavior.offer_flower": {"filters": {"all_of": 7}}}"#,
            "InvalidType Info minecraft:behavior.offer_flower/filters/all_of",
            r#""all_of": [ { "test": "is_daytime", "value": true } ]"#,
        ),
        (
            "1.19.50",
            "demo:x",
            r#"{"minecraft:behavior.trade_with_player": {"filters": {"all_of": "x"}}}"#,
            "InvalidType Info minecraft:behavior.trade_with_player/filters/all_of",
            r#""all_of": [ { "test": "in_water", "value": false }"#,
        ),
        (
            "1.12.0",
            "minecraft:iron_golem",
            r#"{"minecraft:behavior.target_when_pushed": {"priority": 1, "entity_types": 7}}"#,
            "InvalidType Info minecraft:behavior.target_when_pushed/entity_types",
            // The fixer's `minecraft:filters` is no member the descriptor reads: told, not
            // kept.
            r#""entity_types": [ {} ]"#,
        ),
        // The typed read of the last step: a goal that is `null` is absent.
        (
            "1.26.40",
            "demo:x",
            r#"{"minecraft:behavior.beg": null}"#,
            "ValueRewritten Info minecraft:behavior.beg",
            r#""description""#,
        ),
        // An object where a number or a range stands: the member the range does not read.
        // (From 1.21.89 the strict slice refuses it: told below.)
        (
            "1.21.80",
            "demo:x",
            // The legacy schema reader takes a number only: its type line, the goal at its
            // defaults.
            r#"{"minecraft:behavior.ranged_attack": {"attack_interval": {"a": 1}}}"#,
            "InvalidType Error minecraft:behavior.ranged_attack/attack_interval",
            r#""minecraft:behavior.ranged_attack": { "in_range_movement_mode": "hold_position" }"#,
        ),
        (
            "1.26.30",
            "demo:x",
            r#"{"minecraft:behavior.beg": {"look_time": {"a": 1, "min": 2}}}"#,
            "UnrecognizedField Info minecraft:behavior.beg/look_time/a",
            // beg's `look_time` is read by the integer range reader, which reads `min`.
            r#""look_time": { "min": 2 }"#,
        ),
        (
            "1.21.100",
            "demo:x",
            r#"{"minecraft:behavior.take_block": {"xz_range": {"a": 1}}}"#,
            "UnrecognizedField Info minecraft:behavior.take_block/xz_range/a",
            r#""xz_range": {}"#,
        ),
        (
            "1.16.0",
            "demo:x",
            r#"{"minecraft:projectile": {"on_hit": {"impact_damage": {"damage": {"a": 1}}}}}"#,
            "UnrecognizedField Info minecraft:projectile/on_hit/impact_damage/damage/a",
            r#""damage": {}"#,
        ),
    ];
    // Every row is run; the rows that do not hold are shown together.
    let mut wrong = Vec::new();
    for (version, identifier, components, finding, holds) in rows {
        let (said, written) = said(version, identifier, components);
        // On one line, as `{ "a": 1 }`.
        let written = written.split_whitespace().collect::<Vec<_>>().join(" ");
        if !said.iter().any(|one| one == finding) {
            wrong.push(format!(
                "{version} {components}: {finding} is not among {said:?}"
            ));
        }
        if !written.contains(holds) {
            wrong.push(format!(
                "{version} {components}: `{holds}` is not in {written}"
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    // A `components` that is `null` is an absent one (an information marks it), and a well-formed value says nothing.
    let (null, _) = said("1.26.40", "demo:x", "null");
    assert_eq!(null, ["ValueRewritten Info minecraft:entity/components"]);
    for (version, components) in [
        (
            "1.21.120",
            r#"{"minecraft:ageable": {"feed_items": ["wheat"], "transform_to_item": "bowl"}}"#,
        ),
        (
            "1.13.0",
            r#"{"minecraft:behavior.circle_around_anchor": {"radius_change_chance": 5}}"#,
        ),
    ] {
        let (said, _) = said(version, "demo:x", components);
        assert!(said.is_empty(), "{version} {components}: {said:#?}");
    }
}

// ---- the five `collectUpgraders` ops of 1.26.40 → 1.26.50 ----

/// The components the 1.26.50 step inserts.
const AFTER_OPS_1_26_50: [&str; 5] = [
    "minecraft:can_stand_on_powder_snow",
    "minecraft:uses_legacy_ambient_sounds",
    "minecraft:block_movement_slowdown_immunity",
    "minecraft:freezing_immune",
    "minecraft:freezing_vulnerable",
];

/// The written component `key` of `section` (`components` or a group) of `entity`, if any.
fn written_component(
    entity: &bedrock_addon_family_entity::Entity,
    group: Option<&str>,
    key: &str,
) -> Option<Value> {
    let document = json(&entity.to_json().expect("the entity writes"));
    let payload = payload_of(&document);
    let section = match group {
        None => payload.as_object()?.get("components")?.clone(),
        Some(name) => payload
            .as_object()?
            .get("component_groups")?
            .as_object()?
            .get(name)?
            .clone(),
    };
    section.as_object()?.get(key).cloned()
}

/// The game's uses-legacy-ambient-sounds tests (1.26.60): the
/// builder's entity without components at 1.26.20 has the component after the load, the
/// same entity at 1.26.50 has not ("Uses Legacy Ambient Sounds flag should (not) be set.").
#[test]
fn uses_legacy_ambient_sounds_is_set_below_1_26_50_and_not_at_it() {
    use super::common::stable;
    for (fixture_name, set) in [
        ("engine/ambient_sounds-1.json", true),
        ("engine/ambient_sounds-2.json", false),
    ] {
        let (entity, found) = stable(&fixture(fixture_name), &cx());
        assert!(
            super::common::codes(&found, Severity::Warning).is_empty(),
            "{fixture_name}: {found:#?}"
        );
        assert_eq!(
            written_component(&entity, None, "minecraft:uses_legacy_ambient_sounds").is_some(),
            set,
            "{fixture_name}"
        );
    }
}

/// The step sets `uses_legacy_ambient_sounds` whatever the file wrote (`setComponent`), with
/// the value the file wrote replaced by the default-constructed one; a replaced value is
/// noted where it was.
#[test]
fn uses_legacy_ambient_sounds_replaces_what_the_file_wrote() {
    let before = r#"{"format_version": "1.26.40", "minecraft:entity": {"description": {"identifier": "demo:x"},
        "components": {"minecraft:uses_legacy_ambient_sounds": {"made_up": 1}}}}"#;
    let (out, found) = run_text::<v1_26_40::Entity>(before, Run::default(), &cx());
    let components = out.as_object().unwrap().get("components").unwrap();
    assert_eq!(
        components
            .as_object()
            .unwrap()
            .get("minecraft:uses_legacy_ambient_sounds"),
        Some(&json("{}"))
    );
    assert_eq!(
        texts(&found)
            .iter()
            .map(|t| t.split(" | ").next().unwrap_or_default().to_owned())
            .collect::<Vec<_>>(),
        ["components/minecraft:uses_legacy_ambient_sounds"],
        "{found:#?}"
    );
    // `do_not_upgrade` does not stop it: the op is a cereal upgrader, not the fixer.
    let skipped = before.replace(
        r#""description": {"identifier": "demo:x"},"#,
        r#""description": {"identifier": "demo:x"}, "do_not_upgrade": true,"#,
    );
    let (out, _) = run_text::<v1_26_40::Entity>(&skipped, Run::default(), &cx());
    assert!(
        out.as_object()
            .and_then(|p| p.get("components"))
            .and_then(Value::as_object)
            .is_some_and(|c| c.contains_key("minecraft:uses_legacy_ambient_sounds"))
    );
}

/// The vanilla oracle of the other four ops: an older vanilla copy of each entity, upgraded,
/// carries exactly what the 1.26.50 copy writes for the inserted components (the 1.26.50 copy
/// itself is not upgraded, so it has no `uses_legacy_ambient_sounds`).
#[test]
fn the_1_26_50_step_inserts_what_the_vanilla_1_26_50_files_write() {
    use super::common::stable;
    for (name, older, group) in [
        ("spider", "1_26_0", None),
        ("cave_spider", "1_26_0", None),
        ("fox", "1_26_0", None),
        (
            "sulfur_cube",
            "1_26_40",
            Some("minecraft:sulfur_cube_medium_with_block"),
        ),
        ("strider", "1_26_30", None),
        ("skeleton", "1_26_30", None),
    ] {
        let (upgraded, found) = stable(&vanilla(&format!("{name}_{older}.json")), &cx());
        assert!(
            super::common::codes(&found, Severity::Error).is_empty(),
            "{name} {older}: {found:#?}"
        );
        let (retail, _) = stable(&vanilla(&format!("{name}_1_26_50.json")), &cx());
        for key in AFTER_OPS_1_26_50 {
            if key == "minecraft:uses_legacy_ambient_sounds" {
                assert!(written_component(&upgraded, None, key).is_some(), "{name}");
                assert!(written_component(&retail, None, key).is_none(), "{name}");
                continue;
            }
            assert_eq!(
                written_component(&upgraded, None, key),
                written_component(&retail, None, key),
                "{name}: {key} in components"
            );
            if let Some(group) = group {
                assert_eq!(
                    written_component(&upgraded, Some(group), key),
                    written_component(&retail, Some(group), key),
                    "{name}: {key} in {group}"
                );
            }
        }
    }
}

/// The 1.26.40 document with the given identifier, `components` and `component_groups`, run
/// through the steps to the newest version.
fn run_inserting_step(identifier: &str, components: &str, groups: &str) -> Value {
    let before = format!(
        r#"{{"format_version": "1.26.40", "minecraft:entity": {{"description": {{"identifier": "{identifier}"}}, "components": {components}, "component_groups": {groups}}}}}"#
    );
    run_text::<v1_26_40::Entity>(&before, Run::default(), &cx()).0
}

/// The member `key` of the object at `section` below the written document's root.
fn member_at(out: &Value, section: &[&str], key: &str) -> Option<Value> {
    let mut node = out.clone();
    for step in section {
        node = node.as_object()?.get(step)?.clone();
    }
    node.as_object()?.get(key).cloned()
}

/// The identifier-bound ops one by one, on documents without the component: each of the
/// seven freezing-immune mobs, the three freezing-vulnerable ones and the two spiders; a
/// component the file has is left as written.
#[test]
fn the_1_26_50_step_inserts_by_identifier_only_when_absent() {
    for id in [
        "minecraft:creaking",
        "minecraft:skeleton",
        "minecraft:wither",
        "minecraft:ender_dragon",
        "minecraft:polar_bear",
        "minecraft:snow_golem",
        "minecraft:stray",
    ] {
        let out = run_inserting_step(id, "{}", "{}");
        assert_eq!(
            member_at(&out, &["components"], "minecraft:freezing_immune"),
            Some(json("{}")),
            "{id}"
        );
    }
    for id in [
        "minecraft:strider",
        "minecraft:blaze",
        "minecraft:magma_cube",
    ] {
        let out = run_inserting_step(id, "{}", "{}");
        assert_eq!(
            member_at(&out, &["components"], "minecraft:freezing_vulnerable"),
            Some(json("{}")),
            "{id}"
        );
    }
    for id in ["minecraft:spider", "minecraft:cave_spider"] {
        let out = run_inserting_step(id, "{}", "{}");
        assert_eq!(
            member_at(
                &out,
                &["components"],
                "minecraft:block_movement_slowdown_immunity"
            ),
            Some(json(r#"{"blocks": ["minecraft:web"]}"#)),
            "{id}"
        );
        let kept = run_inserting_step(
            id,
            r#"{"minecraft:block_movement_slowdown_immunity": {"blocks": ["minecraft:honey_block"]}}"#,
            "{}",
        );
        assert_eq!(
            member_at(
                &kept,
                &["components"],
                "minecraft:block_movement_slowdown_immunity"
            ),
            Some(json(r#"{"blocks": ["minecraft:honey_block"]}"#)),
            "{id}: present, left as written"
        );
    }
}

/// The sulfur cube's goes to its group, a group's own `type_family` with `lightweight` gets
/// `can_stand_on_powder_snow`, and a custom entity gets none of them.
#[test]
fn the_1_26_50_step_inserts_into_groups_and_by_family() {
    // The sulfur cube's goes to its group, when the group exists, and not to `components`.
    let out = run_inserting_step(
        "minecraft:sulfur_cube",
        "{}",
        r#"{"minecraft:sulfur_cube_medium_with_block": {}, "other": {}}"#,
    );
    let group = [
        "component_groups",
        "minecraft:sulfur_cube_medium_with_block",
    ];
    assert_eq!(
        member_at(&out, &group, "minecraft:freezing_immune"),
        Some(json("{}"))
    );
    assert_eq!(
        member_at(&out, &["components"], "minecraft:freezing_immune"),
        None
    );
    assert_eq!(
        member_at(
            &out,
            &["component_groups", "other"],
            "minecraft:freezing_immune"
        ),
        None
    );
    // `lightweight`: in `components`, and in a group by the group's own family.
    let out = run_inserting_step(
        "demo:x",
        r#"{"minecraft:type_family": {"family": ["mob", "lightweight"]}}"#,
        r#"{"light": {"minecraft:type_family": {"family": ["lightweight"]}}, "heavy": {"minecraft:type_family": {"family": ["mob"]}}}"#,
    );
    assert_eq!(
        member_at(&out, &["components"], "minecraft:can_stand_on_powder_snow"),
        Some(json("{}"))
    );
    assert_eq!(
        member_at(
            &out,
            &["component_groups", "light"],
            "minecraft:can_stand_on_powder_snow"
        ),
        Some(json("{}"))
    );
    assert_eq!(
        member_at(
            &out,
            &["component_groups", "heavy"],
            "minecraft:can_stand_on_powder_snow"
        ),
        None
    );
    // A custom entity without the family: only `uses_legacy_ambient_sounds`.
    let out = run_inserting_step("demo:skeleton", "{}", "{}");
    let components = out.as_object().unwrap().get("components").unwrap();
    assert_eq!(
        components
            .as_object()
            .unwrap()
            .keys()
            .map(facet_value::VString::as_str)
            .collect::<Vec<_>>(),
        ["minecraft:uses_legacy_ambient_sounds"]
    );
}

// ---- the legacy fixer's 1.26.50 gates ----

/// The enderman's teleport projectile filter `_createEnderManProjectileFilter12650` builds.
const ENDERMAN_PROJECTILE_FILTER: &str = r#"{"none_of": [
    {"test": "is_family", "subject": "other", "value": "splash_potion"},
    {"test": "is_family", "subject": "other", "value": "lingering_potion"},
    {"test": "is_family", "subject": "other", "value": "snowball"},
    {"test": "is_family", "subject": "other", "value": "xp_bottle"}]}"#;

/// The 1.26.50 enderman teleport fixer, for `minecraft:enderman`
/// below 1.26.50: `teleports_on_projectile_hit: true` and the projectile filter, each when
/// absent, in `components` and in the groups; not for another type, not with
/// `do_not_upgrade`.
#[test]
fn the_enderman_teleport_gets_its_projectile_members_below_1_26_50() {
    let document = |identifier: &str, skip: bool| {
        format!(
            r#"{{"format_version": "1.26.40", "minecraft:entity": {{"description": {{"identifier": "{identifier}"}}{},
            "components": {{"minecraft:teleport": {{"random_teleports": true}}}},
            "component_groups": {{"g": {{"minecraft:teleport": {{"teleports_on_projectile_hit": false}}}}}}}}}}"#,
            if skip {
                r#", "do_not_upgrade": true"#
            } else {
                ""
            }
        )
    };
    let teleport = |out: &Value, section: &[&str]| -> Value {
        let mut node = out.clone();
        for step in section {
            node = node.as_object().unwrap().get(step).unwrap().clone();
        }
        node.as_object()
            .unwrap()
            .get("minecraft:teleport")
            .unwrap()
            .clone()
    };
    let (out, _) = run_text::<v1_26_40::Entity>(
        &document("minecraft:enderman", false),
        Run::default(),
        &cx(),
    );
    // (The written typed teleport holds its other members too; the two the op inserts:)
    let members = |t: Value| {
        let t = t.as_object().cloned().unwrap();
        (
            t.get("teleports_on_projectile_hit").cloned(),
            t.get("projectile_filter").cloned(),
        )
    };
    let filter = Some(json(ENDERMAN_PROJECTILE_FILTER));
    assert_eq!(
        members(teleport(&out, &["components"])),
        (Some(Value::from(true)), filter.clone())
    );
    assert_eq!(
        members(teleport(&out, &["component_groups", "g"])),
        (Some(Value::from(false)), filter)
    );
    for (identifier, skip) in [("demo:enderman", false), ("minecraft:enderman", true)] {
        let (out, _) =
            run_text::<v1_26_40::Entity>(&document(identifier, skip), Run::default(), &cx());
        assert_eq!(
            members(teleport(&out, &["components"])),
            (Some(Value::from(false)), None),
            "{identifier} skip {skip}: the typed default `false`, no filter"
        );
    }
    // The vanilla oracle: the 1.26.10 enderman upgraded writes what the 1.26.50 one writes.
    let (upgraded, _) = super::common::stable(&vanilla("enderman_1_26_10.json"), &cx());
    let (retail, _) = super::common::stable(&vanilla("enderman_1_26_50.json"), &cx());
    let flag = |entity: &bedrock_addon_family_entity::Entity| {
        written_component(entity, None, "minecraft:teleport").and_then(|t| {
            t.as_object()
                .and_then(|t| t.get("teleports_on_projectile_hit"))
                .cloned()
        })
    };
    assert_eq!(flag(&upgraded), Some(Value::from(true)));
    assert_eq!(flag(&upgraded), flag(&retail));
}

/// The 1.26.50 components fixer, below 1.26.50:
/// `projectile.multiple_targets: true` and
/// `behavior.follow_mob.abort_navigation_on_stop: false`, each when absent, in
/// every section, `do_not_upgrade` or not; a written value is kept.
#[test]
fn projectile_and_follow_mob_get_their_1_26_50_members_below_it() {
    use bedrock_addon_family_entity::goal::FollowMob;
    let before = r#"{"format_version": "1.26.40", "minecraft:entity": {"description": {"identifier": "demo:x"}, "do_not_upgrade": true,
        "components": {"minecraft:projectile": {"power": 1}, "minecraft:behavior.follow_mob": {"priority": 2}},
        "component_groups": {"g": {"minecraft:projectile": {"multiple_targets": false}, "minecraft:behavior.follow_mob": {"abort_navigation_on_stop": true}}}}}"#;
    let (entity, found) = super::common::stable(before, &cx());
    assert!(
        super::common::codes(&found, Severity::Warning).is_empty(),
        "{found:#?}"
    );
    let member = |group: Option<&str>, key: &str, name: &str| {
        written_component(&entity, group, key)
            .and_then(|c| c.as_object().and_then(|c| c.get(name)).cloned())
    };
    assert_eq!(
        member(None, "minecraft:projectile", "multiple_targets"),
        Some(Value::from(true))
    );
    assert_eq!(
        member(Some("g"), "minecraft:projectile", "multiple_targets"),
        Some(Value::from(false))
    );
    // The goal holds the value the conversion gives (the fixer's `false`, the file's
    // `true`), and the write-back leaves the member out: 1.26.60's follow_mob deprecates
    // the name, so no document of the newest version may hold it.
    let goal = |map: &bedrock_addon_core::document::ComponentMap<
        bedrock_addon_family_entity::component::EntityComponent,
    >| {
        map.get::<FollowMob>()
            .map(FollowMob::abort_navigation_on_stop)
    };
    assert_eq!(goal(&entity.components), Some(false));
    assert_eq!(goal(&entity.component_groups["g"].components), Some(true));
    for group in [None, Some("g")] {
        assert_eq!(
            member(
                group,
                "minecraft:behavior.follow_mob",
                "abort_navigation_on_stop"
            ),
            None,
            "{group:?}"
        );
    }
    for at in [
        "minecraft:entity/components/minecraft:behavior.follow_mob/abort_navigation_on_stop",
        "minecraft:entity/component_groups/g/minecraft:behavior.follow_mob/abort_navigation_on_stop",
    ] {
        assert!(
            found.iter().any(|f| f.code.name() == "MemberNotKept"
                && f.severity == Severity::Info
                && f.location.json_path.to_string() == at),
            "{at}: {found:#?}"
        );
    }
    // The vanilla oracle: the 1.26.40 trident keeps its written `false`, as 1.26.50 writes.
    let (upgraded, _) = super::common::stable(&vanilla("thrown_trident_1_26_40.json"), &cx());
    let (retail, _) = super::common::stable(&vanilla("thrown_trident_1_26_50.json"), &cx());
    let targets = |entity: &bedrock_addon_family_entity::Entity| {
        written_component(entity, None, "minecraft:projectile").and_then(|p| {
            p.as_object()
                .and_then(|p| p.get("multiple_targets"))
                .cloned()
        })
    };
    assert_eq!(targets(&upgraded), Some(Value::from(false)));
    assert_eq!(targets(&upgraded), targets(&retail));
    // A 1.26.50 document is not upgraded: nothing is inserted; one that writes the member is
    // refused, the name being deprecated at its typed version.
    let newest = before.replace("1.26.40", "1.26.50").replace(
        r#", "minecraft:behavior.follow_mob": {"abort_navigation_on_stop": true}"#,
        "",
    );
    let (newest, _) = super::common::stable(&newest, &cx());
    assert!(
        written_component(&newest, None, "minecraft:behavior.follow_mob")
            .and_then(|c| c
                .as_object()
                .and_then(|c| c.get("abort_navigation_on_stop"))
                .cloned())
            .is_none()
    );
    let refused = super::common::outcome(&before.replace("1.26.40", "1.26.50"), &cx());
    assert!(refused.value.is_none());
    assert!(
        refused
            .diagnostics
            .iter()
            .any(|f| f.severity == Severity::Error
                && f.code.to_string() == "member \"abort_navigation_on_stop\" is deprecated"),
        "{:#?}",
        refused.diagnostics
    );
}

/// The adjacent versions of every 1.26.50 gate of this step: a 1.26.49 document goes through
/// the 1.26.40 → 1.26.50 step (the fixer's sections A and B, the five `collectUpgraders` ops)
/// and may still write `follow_mob`'s `abort_navigation_on_stop`; a 1.26.50 document is past
/// the step, and the name is deprecated in it.
#[test]
fn the_1_26_50_gates_hold_at_1_26_49_and_not_at_1_26_50() {
    let spider = |version: &str| {
        format!(
            r#"{{"format_version": "{version}", "minecraft:entity": {{"description": {{"identifier": "minecraft:spider"}},
            "components": {{"minecraft:projectile": {{"power": 1}}}}}}}}"#
        )
    };
    let enderman = |version: &str| {
        format!(
            r#"{{"format_version": "{version}", "minecraft:entity": {{"description": {{"identifier": "minecraft:enderman"}},
            "components": {{"minecraft:teleport": {{"random_teleports": true}}}}}}}}"#
        )
    };
    let member = |entity: &bedrock_addon_family_entity::Entity, key: &str, name: &str| {
        written_component(entity, None, key)
            .and_then(|c| c.as_object().and_then(|c| c.get(name)).cloned())
    };
    // 1.26.49: every gate fires.
    let (old, _) = super::common::stable(&spider("1.26.49"), &cx());
    assert_eq!(
        written_component(&old, None, "minecraft:block_movement_slowdown_immunity")
            .and_then(|c| c.as_object().and_then(|c| c.get("blocks")).cloned()),
        Some(json(r#"["minecraft:web"]"#))
    );
    assert!(written_component(&old, None, "minecraft:uses_legacy_ambient_sounds").is_some());
    assert_eq!(
        member(&old, "minecraft:projectile", "multiple_targets"),
        Some(Value::from(true))
    );
    let (old, _) = super::common::stable(&enderman("1.26.49"), &cx());
    assert_eq!(
        member(&old, "minecraft:teleport", "teleports_on_projectile_hit"),
        Some(Value::from(true))
    );
    assert_eq!(
        member(&old, "minecraft:teleport", "projectile_filter"),
        Some(json(ENDERMAN_PROJECTILE_FILTER))
    );
    // 1.26.50: none does.
    let (new, _) = super::common::stable(&spider("1.26.50"), &cx());
    assert!(written_component(&new, None, "minecraft:block_movement_slowdown_immunity").is_none());
    assert!(written_component(&new, None, "minecraft:uses_legacy_ambient_sounds").is_none());
    assert_ne!(
        member(&new, "minecraft:projectile", "multiple_targets"),
        Some(Value::from(true))
    );
    let (new, _) = super::common::stable(&enderman("1.26.50"), &cx());
    assert_ne!(
        member(&new, "minecraft:teleport", "teleports_on_projectile_hit"),
        Some(Value::from(true))
    );
    assert_eq!(
        member(&new, "minecraft:teleport", "projectile_filter"),
        None
    );
    // follow_mob's deprecated name: told and not kept at 1.26.49, refused at 1.26.50.
    let follow = |version: &str| {
        format!(
            r#"{{"format_version": "{version}", "minecraft:entity": {{"description": {{"identifier": "demo:x"}},
            "components": {{"minecraft:behavior.follow_mob": {{"abort_navigation_on_stop": true}}}}}}}}"#
        )
    };
    let at = "minecraft:entity/components/minecraft:behavior.follow_mob/abort_navigation_on_stop";
    let said = |found: &bedrock_addon_core::diagnostic::Diagnostics| -> Vec<String> {
        found
            .iter()
            .filter(|f| f.location.json_path.to_string() == at)
            .map(|f| format!("{} {:?}", f.code.name(), f.severity))
            .collect()
    };
    let (_, found) = super::common::stable(&follow("1.26.49"), &cx());
    assert_eq!(said(&found), ["MemberNotKept Info"], "{found:#?}");
    let refused = super::common::outcome(&follow("1.26.50"), &cx());
    assert!(refused.value.is_none());
    assert_eq!(
        said(&refused.diagnostics),
        ["DeprecatedMember Error"],
        "{:#?}",
        refused.diagnostics
    );
}

/// The 1.12.0 fixer gives the iron golem's `target_when_pushed` an entity type whose
/// filter is under `minecraft:filters` / `minecraft:all_of` (the fixer writes them,
/// and nothing else in the game reads either string; the schema
/// reader matches member names by their exact hash): the reader does not read it. The crate
/// tells it at the member and does not keep it.
#[test]
fn the_iron_golem_fixer_s_prefixed_filter_is_not_read() {
    use super::common::stable;
    let text = r#"{"format_version": "1.12.0", "minecraft:entity": {"description": {"identifier": "minecraft:iron_golem"}, "components": {}}}"#;
    let (entity, found) = stable(text, &cx());
    let at = "minecraft:entity/components/minecraft:behavior.target_when_pushed/entity_types/0/minecraft:filters";
    assert!(
        found
            .iter()
            .any(|f| f.code.name() == "UnrecognizedField" && f.location.json_path.to_string() == at),
        "{found:#?}"
    );
    let written = entity.to_json().unwrap();
    assert!(!written.contains("minecraft:filters"), "{written}");
}

/// A swim-wander interval beyond an int: what the game's `asInt` makes of it is a developer
/// assert at most, no content-log line, so the crate's note is an information;
/// a fraction is truncated with an information as before.
#[test]
fn e00e_an_interval_beyond_an_int_is_an_information() {
    for interval in ["1e10", "2.5"] {
        let before = format!(
            r#"{{"format_version": "1.13.0", "minecraft:entity": {{"description": {{"identifier": "minecraft:cod"}},
        "components": {{"minecraft:behavior.swim_wander": {{"interval": {interval}, "look_ahead": 2.0}}}}}}}}"#
        );
        let (_, found) = run_text::<v1_13_0::Entity>(&before, Run::default(), &cx());
        let severities: Vec<Severity> = found
            .iter()
            .filter(|d| d.code.name() == "InvalidValue")
            .map(|d| d.severity)
            .collect();
        assert_eq!(severities, [Severity::Info], "{interval}");
    }
}
