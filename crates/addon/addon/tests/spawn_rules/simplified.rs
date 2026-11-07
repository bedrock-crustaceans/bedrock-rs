//! Simplified spawn rules and the routing gates: the fixture rows `s02`, `s03`,
//! `s10`–`s22`. The first five inputs are vanilla's
//! unit-test documents (`ActorSpawnRuleDataLoaderUpgradeUnitTests`).

use bedrock_addon::diagnostic::{Code, Severity};
use bedrock_addon::families::spawn_rules::{Placement, SpawnRules};
use bedrock_addon::version::ENGINE_VERSION;

use super::common::{
    closes_with, context, document, experiment, fixture, has_text, legacy, load, loads, names,
    paths, rejected, round_trip, simplified, standard, with,
};

fn unresolved(code: &Code) -> bool {
    // By legacy name: the variant and the definitions that were it.
    matches!(code.name(), "UnresolvedReference")
}

fn unrecognized(code: &Code) -> bool {
    // By legacy name: the variant and the definitions that were it.
    matches!(code.name(), "UnrecognizedField")
}

/// A simplified document of `x:mob` at `version` with `body` after the description.
fn modern(version: &str, body: &str) -> String {
    document(
        version,
        &format!(r#"{{"description": {{"identifier": "x:mob"}}{body}}}"#),
    )
}

/// The placement of vanilla's `ExistantPlacements_Parse_NoWarnings`.
const REGISTERED: &str = r#", "placement": {"type": "minecraft:on_ground", "heightmap": "minecraft:world_surface", "spawn_predicate": "minecraft:light_monster", "spawn_obstruction_predicate": "minecraft:unobstructed"}, "conditions": [{"weight": 99}]"#;

#[test]
fn s02_spawn_simplified_routing() {
    let new_shape = modern("1.21.60", REGISTERED);
    // Experiment off: the hand parser (`spawn`), which knows none of the new keys.
    let off = loads(&new_shape, &context());
    assert!(matches!(off.value(), SpawnRules::Standard(_)));
    // Experiment on: `spawn_simplified`.
    let on = loads(&new_shape, &experiment());
    assert!(matches!(on.value(), SpawnRules::Simplified(_)));
    assert!(on.diagnostics().is_empty(), "{:#?}", on.diagnostics());
    // A 1.8.0 file with the experiment on stays in `spawn`.
    let old = loads(
        &legacy(r#"{"minecraft:weight": {"default": 3}}"#),
        &experiment(),
    );
    assert!(matches!(old.value(), SpawnRules::Standard(_)));
}

#[test]
fn s02_a_legacy_file_under_the_experiment_keeps_its_semantics_deviation() {
    // the upgrade steps G4: vanilla parses `[1.8.0, 1.21.60)` with the `Legacy` cereal parser and
    // upgrades it lossily; ours keeps the standard rules.
    let text = legacy(
        r#"{"minecraft:weight": {"default": 3}, "minecraft:density_limit": {"surface": 2}, "minecraft:permute_type": [{"weight": 1, "entity_type": "x:a", "guaranteed_count": 1}]}"#,
    );
    let document = loads(&text, &experiment());
    let condition = &standard(&document).conditions.as_ref().unwrap()[0];
    assert_eq!(condition.to_spawn_rules().surface_cap, 2);
    assert_eq!(condition.to_spawn_rules().guaranteed.len(), 1);
}

#[test]
fn s03_spawn_simplified_placement() {
    for kind in [
        "minecraft:on_ground",
        "minecraft:in_water",
        "minecraft:in_lava",
        "minecraft:no_restrictions",
    ] {
        let text = modern(
            "1.21.60",
            &format!(
                r#", "placement": {{"type": "{kind}", "heightmap": "minecraft:motion_blocking", "spawn_predicate": "minecraft:animal", "spawn_obstruction_predicate": "minecraft:mob"}}, "conditions": [{{"weight": 8, "herds": [{{"max": 4}}, {{}}], "permute_types": [{{"weight": 2, "min_guaranteed": 1, "entity_type": "x:baby"}}], "mob_event_filter": "x:event"}}]"#
            ),
        );
        let document = loads(&text, &experiment());
        assert!(
            document.diagnostics().is_empty(),
            "{kind}: {:#?}",
            document.diagnostics()
        );
        let rules = simplified(&document);
        let placement = rules.placement.as_ref().unwrap();
        assert_eq!(placement.resolved_type(), kind);
        assert_eq!(placement.resolved_heightmap(), "minecraft:motion_blocking");
        assert_eq!(placement.resolved_spawn_predicate(), "minecraft:animal");
        assert_eq!(
            placement.resolved_spawn_obstruction_predicate(),
            "minecraft:mob"
        );
        let condition = &rules.conditions.as_ref().unwrap()[0];
        let herds = condition.herds.as_ref().unwrap();
        assert_eq!((herds[0].min(), herds[0].max()), (1, 4));
        assert_eq!((herds[1].min(), herds[1].max()), (1, 1));
        let permute = &condition.permute_types.as_ref().unwrap()[0];
        assert_eq!((permute.weight, permute.min_guaranteed), (2, Some(1)));
        assert_eq!(condition.mob_event_filter, Some("x:event".to_owned()));
    }
    // An unknown type is a warning; the document loads.
    let unknown = loads(
        &modern(
            "1.21.60",
            r#", "placement": {"type": "x:floating"}, "conditions": []"#,
        ),
        &experiment(),
    );
    assert_eq!(
        names(unknown.diagnostics(), Severity::Info),
        ["UnresolvedReference"]
    );
    assert!(!unknown.diagnostics().has_errors());
    assert_eq!(
        simplified(&unknown)
            .placement
            .as_ref()
            .unwrap()
            .resolved_type(),
        "minecraft:no_restrictions"
    );
}

#[test]
fn s10_simplified_registered_placement() {
    let document = loads(&modern("1.21.60", REGISTERED), &experiment());
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    let placement = simplified(&document).placement.as_ref().unwrap();
    assert_eq!(
        placement.resolved_spawn_predicate(),
        "minecraft:light_monster"
    );
    round_trip(document.value(), "spawn_rules/x.json", &experiment());
}

#[test]
fn s11_simplified_unregistered_placement() {
    let text = document(
        "1.21.60",
        r#"{"description": {"identifier": "test:mob"}, "placement": {"type": "test:doesnt_exist", "heightmap": "test:doesnt_exist", "spawn_predicate": "test:doesnt_exist", "spawn_obstruction_predicate": "test:doesnt_exist"}, "conditions": [{"weight": 1}]}"#,
    );
    let document = loads(&text, &experiment());
    let found = with(document.diagnostics(), unresolved);
    let texts: Vec<String> = found.iter().map(|d| d.code.to_string()).collect();
    assert_eq!(
        texts,
        [
            "Spawn Rules for 'type' in test:mob must use a registered placement type predicate",
            "Spawn Rules for 'heightmap' in test:mob must use a registered heightmap type",
            "Spawn Rules for 'spawn_obstruction_predicate' in test:mob must use a registered obstruction predicate",
            "Spawn Rules for 'spawn_predicate' in test:mob must use a registered entity spawn predicate",
        ]
    );
    assert!(found.iter().all(|d| d.severity == Severity::Warning));
    assert_eq!(
        paths(&found),
        [
            "minecraft:spawn_rules/placement/type",
            "minecraft:spawn_rules/placement/heightmap",
            "minecraft:spawn_rules/placement/spawn_obstruction_predicate",
            "minecraft:spawn_rules/placement/spawn_predicate",
        ]
    );
}

#[test]
fn s12_simplified_defaults() {
    let document = loads(
        &modern("1.21.60", r#", "conditions": [{"weight": 99}]"#),
        &experiment(),
    );
    assert!(document.diagnostics().is_empty());
    let rules = simplified(&document);
    assert!(rules.placement.is_none());
    let condition = &rules.conditions.as_ref().unwrap()[0];
    assert_eq!(condition.weight, 99);
    assert!(condition.herds.is_none() && condition.permute_types.is_none());
    assert!(condition.mob_event_filter.is_none() && condition.biome_filter.is_none());
    // Written back without inventing members.
    let text = round_trip(document.value(), "spawn_rules/x.json", &experiment());
    assert!(
        !text.contains("herds") && !text.contains("placement"),
        "{text}"
    );
}

#[test]
fn s13_simplified_multiple_conditions() {
    let text = modern(
        "1.21.60",
        r#", "conditions": [{"weight": 99}, {"weight": 1, "herds": [{"min": 1}, {"max": 6}, {"min": 2, "max": 6, "event": "minecraft:make_white"}], "mob_event_filter": "mob:filter", "biome_filter": {"test": "has_biome_tag", "operator": "==", "value": "plains"}}]"#,
    );
    let document = loads(&text, &experiment());
    assert!(
        !document.diagnostics().has_errors(),
        "{:#?}",
        document.diagnostics()
    );
    let herds = simplified(&document).conditions.as_ref().unwrap()[1]
        .herds
        .clone()
        .unwrap();
    assert_eq!((herds[1].min(), herds[1].max()), (1, 6));
    round_trip(document.value(), "spawn_rules/x.json", &experiment());
}

#[test]
fn s14_simplified_missing_identifier() {
    let d = rejected(
        &document("1.21.60", r#"{"description": {}}"#),
        &experiment(),
    );
    let found = with(&d, |c| {
        // By legacy name: the variant and the definitions that were it.
        matches!(c.name(), "MissingRequiredField")
    });
    assert_eq!(
        paths(&found),
        ["minecraft:spawn_rules/description/identifier"]
    );
    assert!(closes_with(&d, "Failed to load spawn rules"), "{d:#?}");
}

#[test]
fn s15_simplified_unprefixed_names() {
    let document = loads(
        &modern(
            "1.21.60",
            r#", "placement": {"heightmap": "motion_blocking", "spawn_predicate": "monster"}, "conditions": []"#,
        ),
        &experiment(),
    );
    assert_eq!(with(document.diagnostics(), unresolved).len(), 2);
    let placement = simplified(&document).placement.as_ref().unwrap();
    assert_eq!(
        placement.resolved_heightmap(),
        "minecraft:motion_blocking_no_leaves"
    );
    assert_eq!(placement.resolved_spawn_predicate(), "minecraft:mob");
}

#[test]
fn s16_simplified_constraints() {
    for (body, at) in [
        (
            r#"{"weight": -1}"#,
            "minecraft:spawn_rules/conditions/0/weight",
        ),
        (
            r#"{"weight": 1, "herds": [{"min": -1}]}"#,
            "minecraft:spawn_rules/conditions/0/herds/0/min",
        ),
        (
            r#"{"weight": 1, "permute_types": [{"weight": -1}]}"#,
            "minecraft:spawn_rules/conditions/0/permute_types/0/weight",
        ),
        (
            r#"{"weight": 1, "permute_types": [{"weight": 1, "min_guaranteed": -1}]}"#,
            "minecraft:spawn_rules/conditions/0/permute_types/0/min_guaranteed",
        ),
    ] {
        let d = rejected(
            &modern("1.21.60", &format!(r#", "conditions": [{body}]"#)),
            &experiment(),
        );
        // By legacy name: the variant and the definitions that were it.
        let found = with(&d, |c| matches!(c.name(), "InvalidValue"));
        assert_eq!(paths(&found), [at], "{body}");
        assert_eq!(found[0].severity, Severity::Error);
        assert!(closes_with(&d, "Failed to load spawn rules"));
    }
    let document = loads(
        &modern(
            "1.21.60",
            r#", "conditions": [{"weight": 1, "herds": [{"min": 5, "max": 2}]}]"#,
        ),
        &experiment(),
    );
    assert!(document.diagnostics().is_empty());
}

#[test]
fn s17_simplified_required_weights() {
    for body in [
        r"{}",
        r#"{"weight": 1, "permute_types": [{"entity_type": "x:a"}]}"#,
    ] {
        let d = rejected(
            &modern("1.21.60", &format!(r#", "conditions": [{body}]"#)),
            &experiment(),
        );
        assert!(
            d.iter().any(
                // By legacy name: the variant and the definitions that were it.
                |f| matches!(f.code.name(), "MissingRequiredField")
                    && f.severity == Severity::Error
            ),
            "{body}: {d:#?}"
        );
    }
}

#[test]
fn s18_simplified_legacy_shape_at_1_21_60() {
    let text = document(
        "1.21.60",
        r#"{"description": {"identifier": "x:mob", "population_control": "animal"}, "conditions": [{"minecraft:weight": {"default": 8}, "minecraft:herd": {"min_size": 2}}]}"#,
    );
    let d = rejected(&text, &experiment());
    let mut unknown = paths(&with(&d, unrecognized));
    unknown.sort();
    assert_eq!(
        unknown,
        [
            "minecraft:spawn_rules/conditions/0/minecraft:herd",
            "minecraft:spawn_rules/conditions/0/minecraft:weight",
            "minecraft:spawn_rules/description/population_control",
        ]
    );
    let missing = with(&d, |c| {
        // By legacy name: the variant and the definitions that were it.
        matches!(c.name(), "MissingRequiredField")
    });
    assert_eq!(
        paths(&missing),
        ["minecraft:spawn_rules/conditions/0/weight"]
    );
}

#[test]
fn s19_simplified_envelope() {
    let third = r#"{"format_version": "1.21.60", "minecraft:spawn_rules": {"description": {"identifier": "x:mob"}}, "extra": 1}"#;
    let d = rejected(third, &experiment());
    assert!(
        has_text(
            &d,
            // By legacy name: the variant and the definitions that were it.
            |c| matches!(c.name(), "MalformedInput"),
            Severity::Error,
            "A Minecraft Document must have exactly two members: \"format_version\" and a payload member"
        ),
        "{d:#?}"
    );
    let missing = r#"{"format_version": "1.21.60", "minecraft:spawn_groups": {}}"#;
    let d = rejected(missing, &experiment());
    // `MissingPayload` with vanilla's text: the loader builds its input with the key
    // `minecraft:spawn_rules` (`getResources`), so `initAsJson`'s
    // `checkPayloadKey` refuses the other key.
    assert_eq!(names(&d, Severity::Error)[0], "MissingPayload");
    assert!(
        has_text(
            &d,
            // By legacy name: the variant and the definitions that were it.
            |c| matches!(c.name(), "MissingPayload"),
            Severity::Error,
            "The payload key does not match the expected one. Expected: 'minecraft:spawn_rules'. Actual: 'minecraft:spawn_groups'."
        ),
        "{d:#?}"
    );
}

#[test]
fn s20_simplified_routing_edges() {
    let cx = experiment();
    // Missing format_version: `spawn` at 1.8.0 with the warning.
    let missing = loads(
        r#"{"minecraft:spawn_rules": {"description": {"identifier": "x:mob", "population_control": "animal"}}}"#,
        &cx,
    );
    assert!(matches!(missing.value(), SpawnRules::Standard(_)));
    assert_eq!(
        names(missing.diagnostics(), Severity::Info),
        ["MissingFormatVersion"]
    );
    for version in ["1.7.0", "99.0.0"] {
        let d = rejected(&modern(version, r#", "conditions": []"#), &cx);
        assert_eq!(
            names(&d, Severity::Error)[0],
            "UnsupportedVersion",
            "{version}"
        );
    }
    let old = loads(
        &document(
            "1.21.50",
            r#"{"description": {"identifier": "x:mob", "population_control": "animal"}, "conditions": [{"minecraft:weight": {"default": 2}}]}"#,
        ),
        &cx,
    );
    assert!(matches!(old.value(), SpawnRules::Standard(_)));
    // Experiment off: the s10 document reads as `spawn`, its new keys unknown (G2).
    let off = loads(&modern("1.21.60", REGISTERED), &context());
    let mut unknown = paths(&with(off.diagnostics(), unrecognized));
    unknown.sort();
    assert_eq!(
        unknown,
        [
            "minecraft:spawn_rules/conditions/0/weight",
            "minecraft:spawn_rules/placement",
        ]
    );
}

#[test]
fn s20_new_keys_without_the_experiment_deviation() {
    // G2: vanilla never looks at `placement` or the unprefixed keys; ours warns, and the missing pool leaves the rules unregistered.
    let off = loads(&modern("1.21.60", REGISTERED), &context());
    assert!(
        names(off.diagnostics(), Severity::Info).contains(&"MissingRequiredField"),
        "{:#?}",
        off.diagnostics()
    );
}

#[test]
fn s20b_simplified_at_engine() {
    let engine = ENGINE_VERSION.to_string();
    let d = rejected(&modern(&engine, r#", "conditions": []"#), &experiment());
    assert!(
        has_text(
            &d,
            // By legacy name: the variant and the definitions that were it.
            |c| matches!(c.name(), "UnsupportedVersion"),
            Severity::Error,
            &format!("unsupported version: no parser available for version {engine}")
        ),
        "{d:#?}"
    );
    let below = format!(
        "{}.{}.{}",
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch - 10
    );
    let loaded = loads(&modern(&below, r#", "conditions": []"#), &experiment());
    assert!(matches!(loaded.value(), SpawnRules::Simplified(_)));
    let legacy_at_engine = document(
        &engine,
        r#"{"description": {"identifier": "x:mob", "population_control": "animal"}, "conditions": [{"minecraft:weight": {"default": 2}}]}"#,
    );
    let off = loads(&legacy_at_engine, &context());
    assert!(matches!(off.value(), SpawnRules::Standard(_)));
}

#[test]
fn s21_simplified_herd_object() {
    let d = rejected(
        &modern(
            "1.21.60",
            r#", "conditions": [{"weight": 1, "herds": {"min": 2}}]"#,
        ),
        &experiment(),
    );
    assert!(d.has_errors());
    assert_eq!(names(&d, Severity::Error)[0], "InvalidType", "{d:#?}");
}

#[test]
fn s22_vanilla_upgrade_table() {
    // The vanilla upgrade-table data file, for the comparison (not a load test):
    // ten names placed in water, one in lava, nine without restrictions, fifty on the ground,
    // and the water / lava override.
    let table: bedrock_addon::json::Value =
        facet_json::from_str(&fixture("spawn", "s22_vanilla_upgrade_table.json")).unwrap();
    let table = table.as_object().unwrap();
    let count = |key: &str| {
        table
            .get(key)
            .and_then(|v| v.as_array())
            .map_or(0, facet_value::VArray::len)
    };
    assert_eq!(count("minecraft:in_water"), 10);
    assert_eq!(count("minecraft:in_lava"), 1);
    assert_eq!(count("minecraft:no_restrictions"), 9);
    assert_eq!(count("minecraft:on_ground"), 50);
    assert!(table.contains_key("override"));
    // Our load of a vanilla-shaped 1.8.0 file never produces a placement, with or without the
    // experiment: no upgrade into the simplified shape exists.
    let text = legacy(r#"{"minecraft:spawns_underwater": {}, "minecraft:weight": {"default": 4}}"#);
    for cx in [context(), experiment()] {
        assert!(matches!(loads(&text, &cx).value(), SpawnRules::Standard(_)));
    }
}

#[test]
fn no_placement_is_the_runtime_default() {
    let placement = Placement::default();
    assert_eq!(placement.resolved_type(), "minecraft:no_restrictions");
    assert_eq!(
        placement.resolved_heightmap(),
        "minecraft:motion_blocking_no_leaves"
    );
    assert_eq!(
        placement.resolved_spawn_obstruction_predicate(),
        "minecraft:mob"
    );
    // An empty name is the runtime default too, without a warning.
    let document = loads(
        &modern(
            "1.21.60",
            r#", "placement": {"type": ""}, "conditions": []"#,
        ),
        &experiment(),
    );
    assert!(document.diagnostics().is_empty());
    assert!(load(&modern("1.21.60", ""), &experiment()).is_ok());
}

/// A scalar of another type than its member's is refused: an `InvalidType` error at the member,
/// its span the member's key, and the file rejected.
#[test]
fn s16b_simplified_wrong_typed_scalars_are_refused_at_their_member() {
    for (body, path, key) in [
        (r#"{"weight": "2"}"#, "conditions/0/weight", r#""weight""#),
        (r#"{"weight": 2.0}"#, "conditions/0/weight", r#""weight""#),
        (r#"{"weight": true}"#, "conditions/0/weight", r#""weight""#),
        (
            r#"{"weight": 1, "herds": [{"min": 1.5}]}"#,
            "conditions/0/herds/0/min",
            r#""min""#,
        ),
        (
            r#"{"weight": 1, "mob_event_filter": 5}"#,
            "conditions/0/mob_event_filter",
            r#""mob_event_filter""#,
        ),
    ] {
        let text = modern("1.21.60", &format!(r#", "conditions": [{body}]"#));
        let d = rejected(&text, &experiment());
        let found: Vec<_> = d
            .iter()
            // By legacy name: the variant and the definitions that were it.
            .filter(|f| matches!(f.code.name(), "InvalidType"))
            .collect();
        assert_eq!(found.len(), 1, "{body}: {d:#?}");
        assert_eq!(found[0].severity, Severity::Error, "{body}");
        assert_eq!(
            found[0].location.json_path.to_string(),
            format!("minecraft:spawn_rules/{path}"),
            "{body}"
        );
        let at = text.find(key).expect("the key");
        let span = found[0].location.span.expect("a span");
        assert_eq!(
            (span.offset as usize, span.len as usize),
            (at, key.len()),
            "{body}"
        );
    }
}
