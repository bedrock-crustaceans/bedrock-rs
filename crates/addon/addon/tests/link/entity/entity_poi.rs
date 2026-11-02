//! 20 E21: the point of interest a Brain memory sensor looks for.

#![allow(unused_imports, reason = "each feature set compiles part of the tests")]

use bedrock_addon::load::{Experiment, Experiments, LoadOptions};
use bedrock_addon::pack::LinkOptions;

use super::super::common::{hosted, link_with};
use super::{FILE, pet, said_of, unresolved};

const FORGE: &str = r#"{"format_version": "1.26.40", "minecraft:poi_block": {"description": {"identifier": "demo:forge"}, "blocks": ["minecraft:anvil"], "tags": ["demo:smithy"]}}"#;
const CRAFT: &str = r#"{"format_version": "1.26.40", "minecraft:poi_tag": {"description": {"identifier": "demo:craft"}, "children": ["demo:smithy"]}}"#;

/// The experiments the fixtures need: POI documents and the Brain components.
fn options() -> LoadOptions {
    let mut experiments = Experiments::new();
    experiments.insert(Experiment::Poi);
    experiments.insert(Experiment::BrainPrototype);
    LoadOptions {
        experiments,
        ..LoadOptions::default()
    }
}

fn sensor(names: &[&str]) -> String {
    let entries: Vec<String> = names
        .iter()
        .map(|name| {
            format!(r#"{{"name": "find", "poi_name": "{name}", "found_poi_memory": "demo:found"}}"#)
        })
        .collect();
    pet(&format!(
        r#""minecraft:memory_sensor.find_nearest_poi": {{"sensor_entries": [{}]}}"#,
        entries.join(", ")
    ))
}

/// A name is a POI type, a tag with a document, or a tag a type carries (a leaf tag, made on
/// its first use); any other name is missing.
/// The sensor logs nothing for a name it lacks (the game makes a tag of it on first
/// use): an information of ours.
#[cfg(feature = "poi")]
#[test]
fn e21_a_poi_name_that_is_no_type_and_no_tag_is_an_information() {
    let text = sensor(&[
        "demo:forge",
        "demo:craft",
        "demo:smithy",
        "demo:none",
        "demo:Forge",
    ]);
    let found = link_with(
        &[
            (FILE, &text),
            ("poi/blocks/forge.json", FORGE),
            ("poi/tags/craft.json", CRAFT),
        ],
        &[],
        options(),
        LinkOptions::default(),
    );
    let at = "components/minecraft:memory_sensor.find_nearest_poi/sensor_entries";
    assert_eq!(
        said_of(&found, "poi type"),
        [
            unresolved(&format!("{at}/3/poi_name"), "poi type", "demo:none"),
            unresolved(&format!("{at}/4/poi_name"), "poi type", "demo:Forge"),
        ],
        "{found:?}"
    );
}

/// The game's own POI types are the host's to answer for, as a type or as a tag.
#[cfg(feature = "poi")]
#[test]
fn e21_a_host_answers_for_the_games_poi_types_and_tags() {
    let text = sensor(&[
        "minecraft:bed",
        "minecraft:acquirable_job_site",
        "minecraft:none",
    ]);
    let files = [(FILE, &*text)];
    let found = link_with(&files, &[], options(), LinkOptions::default());
    assert_eq!(said_of(&found, "poi type").len(), 3, "{found:?}");
    let host = LoadOptions {
        built_ins: hosted(&[
            ("poi type", "minecraft:bed"),
            ("poi tag", "minecraft:acquirable_job_site"),
        ])
        .built_ins,
        ..options()
    };
    let found = link_with(&files, &[], host, LinkOptions::default());
    let at = "components/minecraft:memory_sensor.find_nearest_poi/sensor_entries";
    assert_eq!(
        said_of(&found, "poi type"),
        [unresolved(
            &format!("{at}/2/poi_name"),
            "poi type",
            "minecraft:none"
        )]
    );
}

#[cfg(not(feature = "poi"))]
#[test]
fn e21_without_the_poi_feature_no_poi_name_is_told() {
    let found = link_with(
        &[(FILE, &sensor(&["demo:none"]))],
        &[],
        options(),
        &LinkOptions::default(),
    );
    assert!(said_of(&found, "poi type").is_empty(), "{found:?}");
    let _ = (FORGE, CRAFT, hosted(&[]), unresolved("", "", ""));
}
