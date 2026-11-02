//! 20 SR1, SR2 and E23: the blocks and entities spawn rules and spawn groups name, and the
//! entity spawn rules are for.

#![allow(unused_imports, reason = "each feature set compiles part of the tests")]

use bedrock_addon::diagnostic::{DiagnosticDetail, Phase, Severity};
use bedrock_addon::load::{AppData, Experiment, Experiments, LoadOptions};
use bedrock_addon::pack::LinkOptions;
use bedrock_addon::storage::PackPath;

use super::super::common::{behavior_pack, hosted, link, link_with, stacks_of};
use super::{Said, block, entity_with, miss, said_of};

const RULES: &str = "spawn_rules/pet.json";

/// Standard spawn rules for `id` with the JSON members `conditions` in one condition.
fn rules(id: &str, condition: &str) -> String {
    format!(
        r#"{{"format_version": "1.8.0", "minecraft:spawn_rules": {{"description": {{"identifier": "{id}", "population_control": "animal"}}, "conditions": [{{"minecraft:weight": {{"default": 1}}{condition}}}]}}}}"#
    )
}

fn at(path: &str, severity: Severity, kind: &'static str, id: &str) -> Said {
    (
        "UnresolvedReference",
        severity,
        RULES.to_owned(),
        format!("minecraft:spawn_rules/{path}"),
        kind,
        id.to_owned(),
    )
}

const FILTERS: &str = r#",
    "minecraft:spawns_on_block_filter": ["minecraft:grass_block", "demo:lamp", "demo:none1", {"name": "demo:none2"}, {"tags": "query.any_tag('demo:none')"}],
    "minecraft:spawns_on_block_prevented_filter": "demo:none3",
    "minecraft:spawns_above_block_filter": {"blocks": ["demo:none4", "stone"], "distance": 2}"#;

#[cfg(feature = "block")]
#[test]
fn sr1_a_block_filter_naming_a_block_nothing_defines_is_an_error() {
    let found = link(
        &[
            (RULES, &rules("minecraft:pig", FILTERS)),
            ("blocks/lamp.json", &block("demo:lamp")),
        ],
        &[],
    );
    let c = "conditions/0/minecraft:";
    let w = |path: &str, id: &str| at(&format!("{c}{path}"), Severity::Error, "block", id);
    assert_eq!(
        said_of(&found, "block"),
        [
            w("spawns_on_block_filter/2", "demo:none1"),
            w("spawns_on_block_filter/3/name", "demo:none2"),
            w("spawns_on_block_prevented_filter", "demo:none3"),
            w("spawns_above_block_filter/blocks/0", "demo:none4"),
        ]
    );
}

/// With a host only the host's blocks are the game's.
#[cfg(feature = "block")]
#[test]
fn sr1_a_host_answers_for_the_games_blocks() {
    let text = rules(
        "minecraft:pig",
        r#", "minecraft:spawns_on_block_filter": ["grass_block", "minecraft:stone"]"#,
    );
    let files = [(RULES, &*text)];
    assert!(said_of(&link(&files, &[]), "block").is_empty());
    let found = link_with(
        &files,
        &[],
        hosted(&[("block", "minecraft:stone"), ("entity", "minecraft:pig")]),
        LinkOptions::default(),
    );
    assert_eq!(
        said_of(&found, "block"),
        [at(
            "conditions/0/minecraft:spawns_on_block_filter/0",
            Severity::Error,
            "block",
            "grass_block"
        )]
    );
}

#[cfg(not(feature = "block"))]
#[test]
fn sr1_without_the_block_feature_no_block_filter_is_told() {
    let found = link(&[(RULES, &rules("minecraft:pig", FILTERS))], &[]);
    assert!(said_of(&found, "block").is_empty(), "{found:?}");
}

const ENTITIES: &str = r#",
    "minecraft:permute_type": [{"weight": 1, "entity_type": "demo:none1"}, {"weight": 1, "entity_type": "minecraft:zombie<minecraft:as_baby>"}, {"weight": 1, "entity_type": "demo:pet"}, {"weight": 1}],
    "minecraft:delay_filter": {"min": 1, "max": 2, "identifier": "demo:none2", "spawn_chance": 1}"#;

/// A delay's identifier is a key of the delay, not a reference the game's validator holds and
/// not one the game looks up with a line: an information of ours.
#[cfg(feature = "entity")]
#[test]
fn sr2_a_permuted_entity_nothing_defines_is_an_error_a_delayed_one_an_information() {
    let found = link(
        &[
            (RULES, &rules("demo:pet", ENTITIES)),
            (
                "entities/pet.json",
                &entity_with("1.26.50", "demo:pet", "", ""),
            ),
        ],
        &[],
    );
    assert_eq!(
        said_of(&found, "entity"),
        [
            at(
                "conditions/0/minecraft:permute_type/0/entity_type",
                Severity::Error,
                "entity",
                "demo:none1"
            ),
            at(
                "conditions/0/minecraft:delay_filter/identifier",
                Severity::Info,
                "entity",
                "demo:none2"
            ),
        ]
    );
}

/// The single-object form of `permute_type`, and a host that answers for the game.
#[cfg(feature = "entity")]
#[test]
fn sr2_one_permutation_and_a_host() {
    let text = rules(
        "minecraft:pig",
        r#", "minecraft:permute_type": {"weight": 1, "entity_type": "minecraft:cow"}"#,
    );
    let files = [(RULES, &*text)];
    assert!(said_of(&link(&files, &[]), "entity").is_empty());
    let found = link_with(
        &files,
        &[],
        hosted(&[("entity", "minecraft:pig")]),
        LinkOptions::default(),
    );
    assert_eq!(
        said_of(&found, "entity"),
        [at(
            "conditions/0/minecraft:permute_type/entity_type",
            Severity::Error,
            "entity",
            "minecraft:cow"
        )]
    );
}

/// Simplified spawn rules (the experiment on, format 1.21.60) name entities in
/// `permute_types`.
#[cfg(feature = "entity")]
#[test]
fn sr2_the_permute_types_of_simplified_spawn_rules() {
    let text = r#"{"format_version": "1.21.60", "minecraft:spawn_rules": {"description": {"identifier": "minecraft:pig"}, "conditions": [{"weight": 1, "permute_types": [{"weight": 1, "entity_type": "minecraft:cow"}, {"weight": 1, "entity_type": "demo:none"}]}]}}"#;
    let mut experiments = Experiments::new();
    experiments.insert(Experiment::SimplifiedSpawnRules);
    let options = LoadOptions {
        experiments,
        ..LoadOptions::default()
    };
    let found = link_with(&[(RULES, text)], &[], options, LinkOptions::default());
    assert_eq!(
        said_of(&found, "entity"),
        [at(
            "conditions/0/permute_types/1/entity_type",
            Severity::Error,
            "entity",
            "demo:none"
        )],
        "{found:?}"
    );
}

/// SR1 and SR2 on a raid spawn group of the app data: the same members, told at the group's
/// file.
#[cfg(all(feature = "entity", feature = "block"))]
#[test]
fn sr1_sr2_a_spawn_group_is_checked_like_spawn_rules() {
    let group = r#"{"format_version": "1.11.0", "minecraft:spawn_groups": {"description": {"identifier": "minecraft:raid_group_1"}, "conditions": [{
        "minecraft:spawns_on_block_filter": ["demo:none1", "stone"],
        "minecraft:permute_type": [{"weight": 1, "entity_type": "demo:none2"}, {"weight": 1, "entity_type": "minecraft:pillager"}],
        "minecraft:delay_filter": {"min": 1, "max": 2, "identifier": "demo:none3", "spawn_chance": 1}}]}}"#;
    let app_data = AppData::from_files([(
        PackPath::new("definitions/spawn_groups/raid.json").expect("a path"),
        group.as_bytes().to_vec(),
    )]);
    let options = LoadOptions {
        app_data: Some(app_data),
        ..LoadOptions::default()
    };
    let set = stacks_of(vec![behavior_pack(1, &[])], Vec::new(), options);
    let found = set.link();
    let told: Vec<_> = found
        .iter()
        .filter_map(|finding| {
            assert_eq!(finding.phase, Phase::Link);
            let Some(DiagnosticDetail::Reference { kind, id, .. }) = finding.detail.as_deref()
            else {
                return None;
            };
            Some((
                finding.code.name(),
                finding.severity,
                finding.location.path.as_ref()?.as_str().to_owned(),
                finding.location.json_path.to_string(),
                *kind,
                id.clone(),
            ))
        })
        .collect();
    let g = |path: &str, kind: &'static str, id: &str| {
        // The game's link pass holds a block filter and a permutation; the delay's identifier is no reference of its, nor one the game looks up
        // with a line: an information.
        let severity = if path.starts_with("delay_filter") {
            Severity::Info
        } else {
            Severity::Error
        };
        (
            "UnresolvedReference",
            severity,
            "definitions/spawn_groups/raid.json".to_owned(),
            format!("minecraft:spawn_groups/conditions/0/minecraft:{path}"),
            kind,
            id.to_owned(),
        )
    };
    assert_eq!(
        told,
        [
            g("spawns_on_block_filter/0", "block", "demo:none1"),
            g("permute_type/0/entity_type", "entity", "demo:none2"),
            g("delay_filter/identifier", "entity", "demo:none3"),
        ]
    );
}

/// A document of the 1.8.0 shape whose reader leaves an element out: a condition and a
/// permutation that are no objects, a block descriptor the reader refuses.
const LEFT_OUT: &str = r#"{"description": {"identifier": "minecraft:pig", "population_control": "animal"}, "conditions": [7, {
    "minecraft:spawns_on_block_filter": [7, "demo:none1", {"tags": 3}, {"name": "demo:none2"}],
    "minecraft:spawns_on_block_prevented_filter": [{"tags": 3}, "demo:none3"],
    "minecraft:spawns_above_block_filter": {"blocks": [7, "demo:none4"]},
    "minecraft:permute_type": [7, {"weight": 1, "entity_type": "demo:none5"}]}]}"#;

/// SR1 and SR2 behind an element the reader of the 1.8.0 shape leaves out (a condition or a
/// permutation that is no object, a block descriptor it refuses): the index is the file's,
/// at every level.
#[cfg(all(feature = "entity", feature = "block"))]
#[test]
fn sr1_sr2_a_member_behind_an_element_the_reader_leaves_out_is_at_the_index_of_the_file() {
    let text = format!(r#"{{"format_version": "1.8.0", "minecraft:spawn_rules": {LEFT_OUT}}}"#);
    let found = link(&[(RULES, &text)], &[]);
    let c = "conditions/1/minecraft:";
    let w = |path: &str, kind: &'static str, id: &str| {
        at(&format!("{c}{path}"), Severity::Error, kind, id)
    };
    assert_eq!(
        said_of(&found, "block"),
        [
            w("spawns_on_block_filter/1", "block", "demo:none1"),
            w("spawns_on_block_filter/3/name", "block", "demo:none2"),
            w("spawns_on_block_prevented_filter/1", "block", "demo:none3"),
            w("spawns_above_block_filter/blocks/1", "block", "demo:none4"),
        ],
        "{found:#?}"
    );
    assert_eq!(
        said_of(&found, "entity"),
        [w("permute_type/1/entity_type", "entity", "demo:none5")]
    );
}

/// The same for a raid spawn group of the app data.
#[cfg(all(feature = "entity", feature = "block"))]
#[test]
fn sr1_sr2_a_member_of_a_spawn_group_behind_an_element_left_out_is_at_the_index_of_the_file() {
    let group = format!(r#"{{"format_version": "1.11.0", "minecraft:spawn_groups": {LEFT_OUT}}}"#)
        .replace("minecraft:pig", "minecraft:raid_group_1");
    let app_data = AppData::from_files([(
        PackPath::new("definitions/spawn_groups/raid.json").expect("a path"),
        group.into_bytes(),
    )]);
    let options = LoadOptions {
        app_data: Some(app_data),
        ..LoadOptions::default()
    };
    let set = stacks_of(vec![behavior_pack(1, &[])], Vec::new(), options);
    let found = set.link();
    let told: Vec<String> = found
        .iter()
        .filter(|finding| {
            matches!(
                finding.detail.as_deref(),
                Some(DiagnosticDetail::Reference { .. })
            )
        })
        .map(|finding| finding.location.json_path.to_string())
        .collect();
    let c = "minecraft:spawn_groups/conditions/1/minecraft:";
    assert_eq!(
        told,
        [
            format!("{c}spawns_on_block_filter/1"),
            format!("{c}spawns_on_block_filter/3/name"),
            format!("{c}spawns_on_block_prevented_filter/1"),
            format!("{c}spawns_above_block_filter/blocks/1"),
            format!("{c}permute_type/1/entity_type"),
        ],
        "{found:#?}"
    );
}

/// E23: spawn rules for an entity nothing defines are kept by the game, which checks only
/// that the file has an identifier: a lint.
#[cfg(feature = "entity")]
#[test]
fn e23_spawn_rules_for_an_entity_nothing_defines_are_an_information() {
    let files = [
        (RULES, &*rules("demo:ghost", "")),
        ("spawn_rules/own.json", &*rules("demo:pet", "")),
        ("spawn_rules/pig.json", &*rules("minecraft:pig", "")),
        (
            "entities/pet.json",
            &*entity_with("1.26.50", "demo:pet", "", ""),
        ),
    ];
    let found = link(&files, &[]);
    assert_eq!(
        said_of(&found, "entity"),
        [at(
            "description/identifier",
            Severity::Info,
            "entity",
            "demo:ghost"
        )]
    );
    let quiet = link_with(
        &files,
        &[],
        LoadOptions::default(),
        LinkOptions {
            lints: false,
            ..Default::default()
        },
    );
    assert!(quiet.is_empty(), "{quiet:?}");
    // With a host, the game's entities are the host's.
    let hosted = link_with(
        &files,
        &[],
        hosted(&[("entity", "demo:ghost")]),
        LinkOptions::default(),
    );
    assert_eq!(
        said_of(&hosted, "entity"),
        [at(
            "description/identifier",
            Severity::Info,
            "entity",
            "minecraft:pig"
        )
        .with_file("spawn_rules/pig.json")]
    );
}

/// E23, the other half: an entity without spawn rules, and one without a client entity, is
/// nothing this group tells (a client entity without an entity is row R10's).
#[cfg(feature = "entity")]
#[test]
fn e23_an_entity_without_spawn_rules_or_a_client_entity_is_not_told() {
    let found = link(
        &[(
            "entities/pet.json",
            &entity_with("1.26.50", "demo:pet", "", ""),
        )],
        &[],
    );
    assert!(found.is_empty(), "{found:?}");
}

#[cfg(not(feature = "entity"))]
#[test]
fn sr2_e23_without_the_entity_feature_no_entity_of_spawn_rules_is_told() {
    let found = link(&[(RULES, &rules("demo:ghost", ENTITIES))], &[]);
    assert!(said_of(&found, "entity").is_empty(), "{found:?}");
}

/// A finding at another file.
trait WithFile {
    fn with_file(self, file: &str) -> Self;
}

impl WithFile for Said {
    fn with_file(mut self, file: &str) -> Self {
        file.clone_into(&mut self.2);
        self
    }
}

/// The game's own spawn rules write `minecraft:grass` and are for `minecraft:pillager_patrol`,
/// delayed by `minecraft:pillager_patrol_easy`: names of the game's own files resolve.
#[cfg(all(feature = "entity", feature = "block"))]
#[test]
fn sr1_sr2_e23_the_legacy_names_of_the_games_own_spawn_rules_resolve() {
    let text = rules(
        "minecraft:pillager_patrol",
        r#", "minecraft:spawns_on_block_filter": "minecraft:grass",
           "minecraft:delay_filter": {"min": 1, "max": 2, "identifier": "minecraft:pillager_patrol_easy", "spawn_chance": 1}"#,
    );
    let found = link(&[(RULES, &text)], &[]);
    assert!(found.is_empty(), "{found:?}");
    for delay in [
        "minecraft:pillager_patrol_normal",
        "minecraft:pillager_patrol_hard",
    ] {
        let text = rules(
            "minecraft:pillager_patrol",
            &format!(
                r#", "minecraft:delay_filter": {{"min": 1, "max": 2, "identifier": "{delay}", "spawn_chance": 1}}"#
            ),
        );
        let found = link(&[(RULES, &text)], &[]);
        assert!(found.is_empty(), "{delay}: {found:?}");
    }
}

/// The patrol names are the identifier of the game's spawn rules and
/// of their delays, not actors: in the two members the game's files write them they are let
/// through, and as the entity a rule permutes to they are told like any name.
#[cfg(feature = "entity")]
#[test]
fn sr2_a_patrol_name_is_no_entity_a_spawn_rule_permutes_to() {
    let text = rules(
        "minecraft:pillager_patrol",
        r#", "minecraft:permute_type": [{"weight": 1, "entity_type": "minecraft:pillager_patrol_hard"}, {"weight": 1, "entity_type": "minecraft:pig"}]"#,
    );
    let found = link(&[(RULES, &text)], &[]);
    let named: Vec<_> = super::super::common::unresolved(&found)
        .into_iter()
        .map(|(kind, id, severity, _)| (kind, id, severity))
        .collect();
    assert_eq!(
        named,
        [(
            "entity",
            "minecraft:pillager_patrol_hard".to_owned(),
            Severity::Error
        )]
    );
}
