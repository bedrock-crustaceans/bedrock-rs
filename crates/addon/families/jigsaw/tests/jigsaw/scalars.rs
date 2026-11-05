//! Every scalar member of the jigsaw documents (structure, template pool, structure set,
//! processor list) and of the metadata registry at 1.21.80 and 1.21.50, read from every JSON type: each line pins what the load says
//! (code, severity, path, span) and, when the document loads, the document written back. The
//! members were strict wrappers; this table is what they read, and the oracle of the change
//! that makes them plain.

use std::fmt::Write as _;

use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::{Content, Rejected};
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::versioning::{Loaded, load as load_chain};
use bedrock_addon_family_jigsaw::history::MetadataFamily;
use bedrock_addon_family_jigsaw::{JigsawStructure, ProcessorList, StructureSet, TemplatePool};

/// What a load gave: the document written back, or nothing, and the findings.
struct Outcome {
    written: Option<String>,
    diagnostics: Diagnostics,
}

/// Loads `text` as the `T` at `path`.
fn content<T: Content>(path: &str, text: &str) -> Outcome {
    let read: Result<_, Rejected> = T::parse(
        &PackPath::new(path).expect("a path"),
        text.as_bytes(),
        &LoadContext::detached(),
    );
    match read {
        Ok(document) => Outcome {
            written: Some(document.to_json().unwrap_or_default()),
            diagnostics: document.diagnostics().clone(),
        },
        Err(rejected) => Outcome {
            written: None,
            diagnostics: rejected.diagnostics,
        },
    }
}

/// Loads `text` by the file it is: the payload key names it.
fn load(text: &str) -> Outcome {
    if text.contains("minecraft:jigsaw_structure_metadata") {
        let outcome = load_chain::<MetadataFamily>(JsonSlice::new(text), &LoadContext::detached());
        return Outcome {
            written: matches!(outcome.value, Some(Loaded::Stable(_))).then(String::new),
            diagnostics: outcome.diagnostics,
        };
    }
    if text.contains("minecraft:jigsaw\"") {
        content::<JigsawStructure>("worldgen/structures/a.json", text)
    } else if text.contains("minecraft:template_pool") {
        content::<TemplatePool>("worldgen/template_pools/a.json", text)
    } else if text.contains("minecraft:structure_set") {
        content::<StructureSet>("worldgen/structure_sets/a.json", text)
    } else {
        content::<ProcessorList>("worldgen/processors/a.json", text)
    }
}

/// The values each member is read from.
const VALUES: &[&str] = &[
    "\"x\"",
    "\"5\"",
    "5",
    "0.5",
    "-1",
    "true",
    "null",
    "1e39",
    "18446744073709551616",
    "[]",
    "{}",
];

/// The documents the table reads, `@@` standing for the value.
const PLACES: &[(&str, &str)] = &[
    (
        "structure.max_depth",
        r#"{"format_version": "1.21.20", "minecraft:jigsaw": {"description": {"identifier": "demo:a"}, "step": "surface_structures", "start_pool": "demo:start", "max_depth": @@, "start_height": {"type": "constant", "value": {"absolute": 0}}}}"#,
    ),
    (
        "structure.start_jigsaw_name",
        r#"{"format_version": "1.21.20", "minecraft:jigsaw": {"description": {"identifier": "demo:a"}, "step": "surface_structures", "start_pool": "demo:start", "max_depth": 1, "start_height": {"type": "constant", "value": {"absolute": 0}}, "start_jigsaw_name": @@}}"#,
    ),
    (
        "structure.on_placed",
        r#"{"format_version": "1.21.20", "minecraft:jigsaw": {"description": {"identifier": "demo:a"}, "step": "surface_structures", "start_pool": "demo:start", "max_depth": 1, "start_height": {"type": "constant", "value": {"absolute": 0}}, "on_placed": @@}}"#,
    ),
    (
        "structure.start_height.absolute",
        r#"{"format_version": "1.21.20", "minecraft:jigsaw": {"description": {"identifier": "demo:a"}, "step": "surface_structures", "start_pool": "demo:start", "max_depth": 1, "start_height": {"type": "constant", "value": {"absolute": @@}}}}"#,
    ),
    (
        "structure.dimension_padding.top",
        r#"{"format_version": "1.21.20", "minecraft:jigsaw": {"description": {"identifier": "demo:a"}, "step": "surface_structures", "start_pool": "demo:start", "max_depth": 1, "start_height": {"type": "constant", "value": {"absolute": 0}}, "dimension_padding": {"top": @@}}}"#,
    ),
    (
        "structure.max_distance.horizontal",
        r#"{"format_version": "1.21.20", "minecraft:jigsaw": {"description": {"identifier": "demo:a"}, "step": "surface_structures", "start_pool": "demo:start", "max_depth": 1, "start_height": {"type": "constant", "value": {"absolute": 0}}, "max_distance_from_center": {"horizontal": @@}}}"#,
    ),
    (
        "structure.max_distance.vertical",
        r#"{"format_version": "1.21.20", "minecraft:jigsaw": {"description": {"identifier": "demo:a"}, "step": "surface_structures", "start_pool": "demo:start", "max_depth": 1, "start_height": {"type": "constant", "value": {"absolute": 0}}, "max_distance_from_center": {"horizontal": 10, "vertical": @@}}}"#,
    ),
    (
        "structure.pool_aliases.alias",
        r#"{"format_version": "1.21.20", "minecraft:jigsaw": {"description": {"identifier": "demo:a"}, "step": "surface_structures", "start_pool": "demo:start", "max_depth": 1, "start_height": {"type": "constant", "value": {"absolute": 0}}, "pool_aliases": [{"alias": @@, "target": "demo:p"}]}}"#,
    ),
    (
        "structure.pool_aliases.targets.weight",
        r#"{"format_version": "1.21.20", "minecraft:jigsaw": {"description": {"identifier": "demo:a"}, "step": "surface_structures", "start_pool": "demo:start", "max_depth": 1, "start_height": {"type": "constant", "value": {"absolute": 0}}, "pool_aliases": [{"alias": "x", "targets": [{"weight": @@, "data": "demo:p"}]}]}}"#,
    ),
    (
        "pool.weight",
        r#"{"format_version": "1.21.20", "minecraft:template_pool": {"description": {"identifier": "demo:p"}, "elements": [{"weight": @@, "element": {"element_type": "minecraft:empty_pool_element"}}]}}"#,
    ),
    (
        "set.salt",
        r#"{"format_version": "1.21.20", "minecraft:structure_set": {"description": {"identifier": "demo:s"}, "placement": {"type": "minecraft:random_spread", "salt": @@, "separation": 1, "spacing": 4}, "structures": [{"structure": "demo:a", "weight": 1}]}}"#,
    ),
    (
        "set.separation",
        r#"{"format_version": "1.21.20", "minecraft:structure_set": {"description": {"identifier": "demo:s"}, "placement": {"type": "minecraft:random_spread", "salt": 1, "separation": @@, "spacing": 4}, "structures": [{"structure": "demo:a", "weight": 1}]}}"#,
    ),
    (
        "set.spacing",
        r#"{"format_version": "1.21.20", "minecraft:structure_set": {"description": {"identifier": "demo:s"}, "placement": {"type": "minecraft:random_spread", "salt": 1, "separation": 1, "spacing": @@}, "structures": [{"structure": "demo:a", "weight": 1}]}}"#,
    ),
    (
        "set.structures.weight",
        r#"{"format_version": "1.21.20", "minecraft:structure_set": {"description": {"identifier": "demo:s"}, "placement": {"type": "minecraft:random_spread", "salt": 1, "separation": 1, "spacing": 4}, "structures": [{"structure": "demo:a", "weight": @@}]}}"#,
    ),
    (
        "processors.protected_blocks.value",
        r#"{"format_version": "1.21.20", "minecraft:processor_list": {"description": {"identifier": "demo:l"}, "processors": [{"processor_type": "minecraft:protected_blocks", "value": @@}]}}"#,
    ),
    (
        "processors.capped.limit.value",
        r#"{"format_version": "1.21.20", "minecraft:processor_list": {"description": {"identifier": "demo:l"}, "processors": [{"processor_type": "minecraft:capped", "limit": {"type": "constant", "value": @@}, "delegate": {"processor_type": "minecraft:block_ignore", "blocks": ["minecraft:air"]}}]}}"#,
    ),
    (
        "processors.capped.limit.min_inclusive",
        r#"{"format_version": "1.21.20", "minecraft:processor_list": {"description": {"identifier": "demo:l"}, "processors": [{"processor_type": "minecraft:capped", "limit": {"type": "uniform", "min_inclusive": @@, "max_inclusive": 3}, "delegate": {"processor_type": "minecraft:block_ignore", "blocks": ["minecraft:air"]}}]}}"#,
    ),
    (
        "processors.rule.random_block_match.probability",
        r#"{"format_version": "1.21.20", "minecraft:processor_list": {"description": {"identifier": "demo:l"}, "processors": [{"processor_type": "minecraft:rule", "rules": [{"input_predicate": {"predicate_type": "minecraft:random_block_match", "block": "minecraft:dirt", "probability": @@}, "output_state": {"name": "minecraft:stone"}}]}]}}"#,
    ),
    (
        "processors.rule.tag_match.tag",
        r#"{"format_version": "1.21.20", "minecraft:processor_list": {"description": {"identifier": "demo:l"}, "processors": [{"processor_type": "minecraft:rule", "rules": [{"input_predicate": {"predicate_type": "minecraft:tag_match", "tag": @@}, "output_state": {"name": "minecraft:stone"}}]}]}}"#,
    ),
    (
        "processors.rule.position.min_chance",
        r#"{"format_version": "1.21.20", "minecraft:processor_list": {"description": {"identifier": "demo:l"}, "processors": [{"processor_type": "minecraft:rule", "rules": [{"position_predicate": {"predicate_type": "minecraft:axis_aligned_linear_pos", "min_chance": @@}, "output_state": {"name": "minecraft:stone"}}]}]}}"#,
    ),
    (
        "processors.rule.position.min_dist",
        r#"{"format_version": "1.21.20", "minecraft:processor_list": {"description": {"identifier": "demo:l"}, "processors": [{"processor_type": "minecraft:rule", "rules": [{"position_predicate": {"predicate_type": "minecraft:axis_aligned_linear_pos", "min_dist": @@}, "output_state": {"name": "minecraft:stone"}}]}]}}"#,
    ),
    (
        "metadata 1.21.80 size[0]",
        r#"{"format_version": "1.21.80", "minecraft:jigsaw_structure_metadata": {"metadata": {"42": {"size": [@@, 2, 3], "markers": [{"name": "a", "pool": "p", "target": "t", "block": "b", "final_block": "f", "joint_type": "rollable", "pos": [0, 1, 2], "selection_priority": 0, "placement_priority": 0}]}}}}"#,
    ),
    (
        "metadata 1.21.80 markers.name",
        r#"{"format_version": "1.21.80", "minecraft:jigsaw_structure_metadata": {"metadata": {"42": {"size": [1, 2, 3], "markers": [{"name": @@, "pool": "p", "target": "t", "block": "b", "final_block": "f", "joint_type": "rollable", "pos": [0, 1, 2], "selection_priority": 0, "placement_priority": 0}]}}}}"#,
    ),
    (
        "metadata 1.21.80 markers.pos[0]",
        r#"{"format_version": "1.21.80", "minecraft:jigsaw_structure_metadata": {"metadata": {"42": {"size": [1, 2, 3], "markers": [{"name": "a", "pool": "p", "target": "t", "block": "b", "final_block": "f", "joint_type": "rollable", "pos": [@@, 1, 2], "selection_priority": 0, "placement_priority": 0}]}}}}"#,
    ),
    (
        "metadata 1.21.80 markers.selection_priority",
        r#"{"format_version": "1.21.80", "minecraft:jigsaw_structure_metadata": {"metadata": {"42": {"size": [1, 2, 3], "markers": [{"name": "a", "pool": "p", "target": "t", "block": "b", "final_block": "f", "joint_type": "rollable", "pos": [0, 1, 2], "selection_priority": @@, "placement_priority": 0}]}}}}"#,
    ),
    (
        "metadata 1.21.50 size[0]",
        r#"{"format_version": "1.21.50", "minecraft:jigsaw_structure_metadata": {"metadata": {"42": {"size": [@@, 2, 3], "markers": [{"name": "a", "pool": "p", "target": "t", "block": "b", "final_block": "f", "joint_type": "rollable", "pos": [0, 1, 2], "selection_priority": 0, "placement_priority": 0}]}}}}"#,
    ),
    (
        "metadata 1.21.50 markers.name",
        r#"{"format_version": "1.21.50", "minecraft:jigsaw_structure_metadata": {"metadata": {"42": {"size": [1, 2, 3], "markers": [{"name": @@, "pool": "p", "target": "t", "block": "b", "final_block": "f", "joint_type": "rollable", "pos": [0, 1, 2], "selection_priority": 0, "placement_priority": 0}]}}}}"#,
    ),
    (
        "metadata 1.21.50 markers.pos[0]",
        r#"{"format_version": "1.21.50", "minecraft:jigsaw_structure_metadata": {"metadata": {"42": {"size": [1, 2, 3], "markers": [{"name": "a", "pool": "p", "target": "t", "block": "b", "final_block": "f", "joint_type": "rollable", "pos": [@@, 1, 2], "selection_priority": 0, "placement_priority": 0}]}}}}"#,
    ),
    (
        "metadata 1.21.50 markers.selection_priority",
        r#"{"format_version": "1.21.50", "minecraft:jigsaw_structure_metadata": {"metadata": {"42": {"size": [1, 2, 3], "markers": [{"name": "a", "pool": "p", "target": "t", "block": "b", "final_block": "f", "joint_type": "rollable", "pos": [0, 1, 2], "selection_priority": @@, "placement_priority": 0}]}}}}"#,
    ),
    (
        "metadata 1.21.50 use_legacy_rotation",
        r#"{"format_version": "1.21.50", "minecraft:jigsaw_structure_metadata": {"metadata": {"42": {"size": [1, 2, 3], "use_legacy_rotation": @@, "markers": [{"name": "a", "pool": "p", "target": "t", "block": "b", "final_block": "f", "joint_type": "rollable", "pos": [0, 1, 2], "selection_priority": 0, "placement_priority": 0}]}}}}"#,
    ),
];

/// The plain integers of the untagged arms, behind the family's integer-or-object check.
const PLAIN: &[(&str, &str)] = &[
    (
        "structure.dimension_padding",
        r#"{"format_version": "1.21.20", "minecraft:jigsaw": {"description": {"identifier": "demo:a"}, "step": "surface_structures", "start_pool": "demo:start", "max_depth": 1, "start_height": {"type": "constant", "value": {"absolute": 0}}, "dimension_padding": @@}}"#,
    ),
    (
        "structure.max_distance",
        r#"{"format_version": "1.21.20", "minecraft:jigsaw": {"description": {"identifier": "demo:a"}, "step": "surface_structures", "start_pool": "demo:start", "max_depth": 1, "start_height": {"type": "constant", "value": {"absolute": 0}}, "max_distance_from_center": @@}}"#,
    ),
    (
        "processors.capped.limit",
        r#"{"format_version": "1.21.20", "minecraft:processor_list": {"description": {"identifier": "demo:l"}, "processors": [{"processor_type": "minecraft:capped", "limit": @@, "delegate": {"processor_type": "minecraft:block_ignore", "blocks": ["minecraft:air"]}}]}}"#,
    ),
];

/// `text` on one line: each line trimmed, the lines joined.
fn compact(text: &str) -> String {
    text.lines().map(str::trim).collect()
}

/// One line of the table: what loading `text` gave.
fn line(text: &str) -> String {
    let Outcome {
        written,
        diagnostics,
    } = load(text);
    let loaded = written.map_or("rejected".to_owned(), |text| {
        format!("loaded {}", compact(&text))
    });
    let findings: Vec<String> = diagnostics
        .iter()
        .map(|d| {
            let span = d
                .location
                .span
                .map_or("-".to_owned(), |s| format!("{}+{}", s.offset, s.len));
            format!(
                "{} {:?} {} {span}",
                d.code.name(),
                d.severity,
                d.location.json_path
            )
        })
        .collect();
    format!("{loaded} [{}]", findings.join("; "))
}

#[test]
fn every_scalar_member_reads_as_pinned() {
    let mut table = String::new();
    for (name, place) in PLACES.iter().chain(PLAIN) {
        for value in VALUES {
            let _ = writeln!(
                table,
                "{name} {value} => {}",
                line(&place.replace("@@", value))
            );
        }
    }
    assert!(table == EXPECTED, "--- actual ---\n{table}--- end ---");
}

/// The table as the strict wrappers read it.
const EXPECTED: &str = r#"structure.max_depth "x" => rejected [InvalidType Error minecraft:jigsaw/max_depth 150+11; DocumentRejected Error  -]
structure.max_depth "5" => rejected [InvalidType Error minecraft:jigsaw/max_depth 150+11; DocumentRejected Error  -]
structure.max_depth 5 => loaded {"format_version": "1.21.20","minecraft:jigsaw": {"description": {"identifier": "demo:a"},"step": "surface_structures","start_pool": "demo:start","max_depth": 5,"start_height": {"type": "constant","value": {"absolute": 0}}}} []
structure.max_depth 0.5 => rejected [InvalidType Error minecraft:jigsaw/max_depth 150+11; DocumentRejected Error  -]
structure.max_depth -1 => rejected [InvalidType Error minecraft:jigsaw/max_depth 150+11; DocumentRejected Error  -]
structure.max_depth true => rejected [InvalidType Error minecraft:jigsaw/max_depth 150+11; DocumentRejected Error  -]
structure.max_depth null => rejected [InvalidType Error minecraft:jigsaw/max_depth 150+11; DocumentRejected Error  -]
structure.max_depth 1e39 => rejected [InvalidType Error minecraft:jigsaw/max_depth 150+11; DocumentRejected Error  -]
structure.max_depth 18446744073709551616 => rejected [InvalidType Error minecraft:jigsaw/max_depth 150+11; DocumentRejected Error  -]
structure.max_depth [] => rejected [InvalidType Error minecraft:jigsaw/max_depth/0 164+1; DocumentRejected Error  -]
structure.max_depth {} => rejected [InvalidType Error minecraft:jigsaw/max_depth 164+1; DocumentRejected Error  -]
structure.start_jigsaw_name "x" => loaded {"format_version": "1.21.20","minecraft:jigsaw": {"description": {"identifier": "demo:a"},"step": "surface_structures","start_pool": "demo:start","start_jigsaw_name": "x","max_depth": 1,"start_height": {"type": "constant","value": {"absolute": 0}}}} []
structure.start_jigsaw_name "5" => loaded {"format_version": "1.21.20","minecraft:jigsaw": {"description": {"identifier": "demo:a"},"step": "surface_structures","start_pool": "demo:start","start_jigsaw_name": "5","max_depth": 1,"start_height": {"type": "constant","value": {"absolute": 0}}}} []
structure.start_jigsaw_name 5 => rejected [InvalidType Error minecraft:jigsaw/start_jigsaw_name 230+19; DocumentRejected Error  -]
structure.start_jigsaw_name 0.5 => rejected [InvalidType Error minecraft:jigsaw/start_jigsaw_name 230+19; DocumentRejected Error  -]
structure.start_jigsaw_name -1 => rejected [InvalidType Error minecraft:jigsaw/start_jigsaw_name 230+19; DocumentRejected Error  -]
structure.start_jigsaw_name true => rejected [InvalidType Error minecraft:jigsaw/start_jigsaw_name 230+19; DocumentRejected Error  -]
structure.start_jigsaw_name null => rejected [InvalidType Error minecraft:jigsaw/start_jigsaw_name 251+4; DocumentRejected Error  -]
structure.start_jigsaw_name 1e39 => rejected [InvalidType Error minecraft:jigsaw/start_jigsaw_name 230+19; DocumentRejected Error  -]
structure.start_jigsaw_name 18446744073709551616 => rejected [InvalidType Error minecraft:jigsaw/start_jigsaw_name 230+19; DocumentRejected Error  -]
structure.start_jigsaw_name [] => rejected [InvalidType Error minecraft:jigsaw/start_jigsaw_name/0 252+1; DocumentRejected Error  -]
structure.start_jigsaw_name {} => rejected [InvalidType Error minecraft:jigsaw/start_jigsaw_name 252+1; DocumentRejected Error  -]
structure.on_placed "x" => loaded {"format_version": "1.21.20","minecraft:jigsaw": {"description": {"identifier": "demo:a"},"step": "surface_structures","start_pool": "demo:start","on_placed": "x","max_depth": 1,"start_height": {"type": "constant","value": {"absolute": 0}}}} []
structure.on_placed "5" => loaded {"format_version": "1.21.20","minecraft:jigsaw": {"description": {"identifier": "demo:a"},"step": "surface_structures","start_pool": "demo:start","on_placed": "5","max_depth": 1,"start_height": {"type": "constant","value": {"absolute": 0}}}} []
structure.on_placed 5 => rejected [InvalidType Error minecraft:jigsaw/on_placed 230+11; DocumentRejected Error  -]
structure.on_placed 0.5 => rejected [InvalidType Error minecraft:jigsaw/on_placed 230+11; DocumentRejected Error  -]
structure.on_placed -1 => rejected [InvalidType Error minecraft:jigsaw/on_placed 230+11; DocumentRejected Error  -]
structure.on_placed true => rejected [InvalidType Error minecraft:jigsaw/on_placed 230+11; DocumentRejected Error  -]
structure.on_placed null => rejected [InvalidType Error minecraft:jigsaw/on_placed 243+4; DocumentRejected Error  -]
structure.on_placed 1e39 => rejected [InvalidType Error minecraft:jigsaw/on_placed 230+11; DocumentRejected Error  -]
structure.on_placed 18446744073709551616 => rejected [InvalidType Error minecraft:jigsaw/on_placed 230+11; DocumentRejected Error  -]
structure.on_placed [] => rejected [InvalidType Error minecraft:jigsaw/on_placed/0 244+1; DocumentRejected Error  -]
structure.on_placed {} => rejected [InvalidType Error minecraft:jigsaw/on_placed 244+1; DocumentRejected Error  -]
structure.start_height.absolute "x" => rejected [InvalidType Error minecraft:jigsaw/start_height/value/absolute 213+10; DocumentRejected Error  -]
structure.start_height.absolute "5" => rejected [InvalidType Error minecraft:jigsaw/start_height/value/absolute 213+10; DocumentRejected Error  -]
structure.start_height.absolute 5 => loaded {"format_version": "1.21.20","minecraft:jigsaw": {"description": {"identifier": "demo:a"},"step": "surface_structures","start_pool": "demo:start","max_depth": 1,"start_height": {"type": "constant","value": {"absolute": 5}}}} []
structure.start_height.absolute 0.5 => rejected [InvalidType Error minecraft:jigsaw/start_height/value/absolute 213+10; DocumentRejected Error  -]
structure.start_height.absolute -1 => loaded {"format_version": "1.21.20","minecraft:jigsaw": {"description": {"identifier": "demo:a"},"step": "surface_structures","start_pool": "demo:start","max_depth": 1,"start_height": {"type": "constant","value": {"absolute": -1}}}} []
structure.start_height.absolute true => rejected [InvalidType Error minecraft:jigsaw/start_height/value/absolute 213+10; DocumentRejected Error  -]
structure.start_height.absolute null => rejected [InvalidType Error minecraft:jigsaw/start_height/value/absolute 213+10; DocumentRejected Error  -]
structure.start_height.absolute 1e39 => rejected [InvalidType Error minecraft:jigsaw/start_height/value/absolute 213+10; DocumentRejected Error  -]
structure.start_height.absolute 18446744073709551616 => rejected [InvalidType Error minecraft:jigsaw/start_height/value/absolute 213+10; DocumentRejected Error  -]
structure.start_height.absolute [] => rejected [InvalidType Error minecraft:jigsaw/start_height/value/absolute/0 226+1; DocumentRejected Error  -]
structure.start_height.absolute {} => rejected [InvalidType Error minecraft:jigsaw/start_height/value/absolute 226+1; DocumentRejected Error  -]
structure.dimension_padding.top "x" => rejected [InvalidType Error minecraft:jigsaw/dimension_padding/top 252+5; DocumentRejected Error  -]
structure.dimension_padding.top "5" => rejected [InvalidType Error minecraft:jigsaw/dimension_padding/top 252+5; DocumentRejected Error  -]
structure.dimension_padding.top 5 => loaded {"format_version": "1.21.20","minecraft:jigsaw": {"description": {"identifier": "demo:a"},"step": "surface_structures","start_pool": "demo:start","max_depth": 1,"start_height": {"type": "constant","value": {"absolute": 0}},"dimension_padding": {"top": 5}}} []
structure.dimension_padding.top 0.5 => rejected [InvalidType Error minecraft:jigsaw/dimension_padding/top 252+5; DocumentRejected Error  -]
structure.dimension_padding.top -1 => rejected [InvalidType Error minecraft:jigsaw/dimension_padding/top 252+5; DocumentRejected Error  -]
structure.dimension_padding.top true => rejected [InvalidType Error minecraft:jigsaw/dimension_padding/top 252+5; DocumentRejected Error  -]
structure.dimension_padding.top null => rejected [InvalidType Error minecraft:jigsaw/dimension_padding/top 259+4; DocumentRejected Error  -]
structure.dimension_padding.top 1e39 => rejected [InvalidType Error minecraft:jigsaw/dimension_padding/top 252+5; DocumentRejected Error  -]
structure.dimension_padding.top 18446744073709551616 => rejected [InvalidType Error minecraft:jigsaw/dimension_padding/top 252+5; DocumentRejected Error  -]
structure.dimension_padding.top [] => rejected [InvalidType Error minecraft:jigsaw/dimension_padding/top/0 260+1; DocumentRejected Error  -]
structure.dimension_padding.top {} => rejected [InvalidType Error minecraft:jigsaw/dimension_padding/top 260+1; DocumentRejected Error  -]
structure.max_distance.horizontal "x" => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center/horizontal 259+12; DocumentRejected Error  -]
structure.max_distance.horizontal "5" => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center/horizontal 259+12; DocumentRejected Error  -]
structure.max_distance.horizontal 5 => loaded {"format_version": "1.21.20","minecraft:jigsaw": {"description": {"identifier": "demo:a"},"step": "surface_structures","start_pool": "demo:start","max_depth": 1,"start_height": {"type": "constant","value": {"absolute": 0}},"max_distance_from_center": {"horizontal": 5}}} []
structure.max_distance.horizontal 0.5 => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center/horizontal 259+12; DocumentRejected Error  -]
structure.max_distance.horizontal -1 => rejected [InvalidValue Error minecraft:jigsaw/max_distance_from_center/horizontal -; DocumentRejected Error  -]
structure.max_distance.horizontal true => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center/horizontal 259+12; DocumentRejected Error  -]
structure.max_distance.horizontal null => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center/horizontal 259+12; DocumentRejected Error  -]
structure.max_distance.horizontal 1e39 => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center/horizontal 259+12; DocumentRejected Error  -]
structure.max_distance.horizontal 18446744073709551616 => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center/horizontal 259+12; DocumentRejected Error  -]
structure.max_distance.horizontal [] => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center/horizontal/0 274+1; DocumentRejected Error  -]
structure.max_distance.horizontal {} => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center/horizontal 274+1; DocumentRejected Error  -]
structure.max_distance.vertical "x" => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center/vertical 277+10; DocumentRejected Error  -]
structure.max_distance.vertical "5" => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center/vertical 277+10; DocumentRejected Error  -]
structure.max_distance.vertical 5 => loaded {"format_version": "1.21.20","minecraft:jigsaw": {"description": {"identifier": "demo:a"},"step": "surface_structures","start_pool": "demo:start","max_depth": 1,"start_height": {"type": "constant","value": {"absolute": 0}},"max_distance_from_center": {"horizontal": 10,"vertical": 5}}} []
structure.max_distance.vertical 0.5 => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center/vertical 277+10; DocumentRejected Error  -]
structure.max_distance.vertical -1 => rejected [InvalidValue Error minecraft:jigsaw/max_distance_from_center/vertical -; DocumentRejected Error  -]
structure.max_distance.vertical true => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center/vertical 277+10; DocumentRejected Error  -]
structure.max_distance.vertical null => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center/vertical 289+4; DocumentRejected Error  -]
structure.max_distance.vertical 1e39 => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center/vertical 277+10; DocumentRejected Error  -]
structure.max_distance.vertical 18446744073709551616 => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center/vertical 277+10; DocumentRejected Error  -]
structure.max_distance.vertical [] => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center/vertical/0 290+1; DocumentRejected Error  -]
structure.max_distance.vertical {} => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center/vertical 290+1; DocumentRejected Error  -]
structure.pool_aliases.alias "x" => loaded {"format_version": "1.21.20","minecraft:jigsaw": {"description": {"identifier": "demo:a"},"step": "surface_structures","start_pool": "demo:start","max_depth": 1,"start_height": {"type": "constant","value": {"absolute": 0}},"pool_aliases": [{"alias": "x","target": "demo:p"}]}} []
structure.pool_aliases.alias "5" => loaded {"format_version": "1.21.20","minecraft:jigsaw": {"description": {"identifier": "demo:a"},"step": "surface_structures","start_pool": "demo:start","max_depth": 1,"start_height": {"type": "constant","value": {"absolute": 0}},"pool_aliases": [{"alias": "5","target": "demo:p"}]}} []
structure.pool_aliases.alias 5 => rejected [InvalidType Error minecraft:jigsaw/pool_aliases/0/alias 248+7; DocumentRejected Error  -]
structure.pool_aliases.alias 0.5 => rejected [InvalidType Error minecraft:jigsaw/pool_aliases/0/alias 248+7; DocumentRejected Error  -]
structure.pool_aliases.alias -1 => rejected [InvalidType Error minecraft:jigsaw/pool_aliases/0/alias 248+7; DocumentRejected Error  -]
structure.pool_aliases.alias true => rejected [InvalidType Error minecraft:jigsaw/pool_aliases/0/alias 248+7; DocumentRejected Error  -]
structure.pool_aliases.alias null => rejected [InvalidType Error minecraft:jigsaw/pool_aliases/0/alias 248+7; DocumentRejected Error  -]
structure.pool_aliases.alias 1e39 => rejected [InvalidType Error minecraft:jigsaw/pool_aliases/0/alias 248+7; DocumentRejected Error  -]
structure.pool_aliases.alias 18446744073709551616 => rejected [InvalidType Error minecraft:jigsaw/pool_aliases/0/alias 248+7; DocumentRejected Error  -]
structure.pool_aliases.alias [] => rejected [InvalidType Error minecraft:jigsaw/pool_aliases/0/alias/0 258+1; DocumentRejected Error  -]
structure.pool_aliases.alias {} => rejected [InvalidType Error minecraft:jigsaw/pool_aliases/0/alias 258+1; DocumentRejected Error  -]
structure.pool_aliases.targets.weight "x" => rejected [InvalidType Error minecraft:jigsaw/pool_aliases/0/targets/0/weight 275+8; DocumentRejected Error  -]
structure.pool_aliases.targets.weight "5" => rejected [InvalidType Error minecraft:jigsaw/pool_aliases/0/targets/0/weight 275+8; DocumentRejected Error  -]
structure.pool_aliases.targets.weight 5 => loaded {"format_version": "1.21.20","minecraft:jigsaw": {"description": {"identifier": "demo:a"},"step": "surface_structures","start_pool": "demo:start","max_depth": 1,"start_height": {"type": "constant","value": {"absolute": 0}},"pool_aliases": [{"alias": "x","targets": [{"weight": 5,"data": "demo:p"}]}]}} []
structure.pool_aliases.targets.weight 0.5 => rejected [InvalidType Error minecraft:jigsaw/pool_aliases/0/targets/0/weight 275+8; DocumentRejected Error  -]
structure.pool_aliases.targets.weight -1 => rejected [InvalidType Error minecraft:jigsaw/pool_aliases/0/targets/0/weight 275+8; DocumentRejected Error  -]
structure.pool_aliases.targets.weight true => rejected [InvalidType Error minecraft:jigsaw/pool_aliases/0/targets/0/weight 275+8; DocumentRejected Error  -]
structure.pool_aliases.targets.weight null => rejected [InvalidType Error minecraft:jigsaw/pool_aliases/0/targets/0/weight 275+8; DocumentRejected Error  -]
structure.pool_aliases.targets.weight 1e39 => rejected [InvalidType Error minecraft:jigsaw/pool_aliases/0/targets/0/weight 275+8; DocumentRejected Error  -]
structure.pool_aliases.targets.weight 18446744073709551616 => rejected [InvalidType Error minecraft:jigsaw/pool_aliases/0/targets/0/weight 275+8; DocumentRejected Error  -]
structure.pool_aliases.targets.weight [] => rejected [InvalidType Error minecraft:jigsaw/pool_aliases/0/targets/0/weight/0 286+1; DocumentRejected Error  -]
structure.pool_aliases.targets.weight {} => rejected [InvalidType Error minecraft:jigsaw/pool_aliases/0/targets/0/weight 286+1; DocumentRejected Error  -]
pool.weight "x" => rejected [InvalidType Error minecraft:template_pool/elements/0/weight 113+8; DocumentRejected Error  -]
pool.weight "5" => rejected [InvalidType Error minecraft:template_pool/elements/0/weight 113+8; DocumentRejected Error  -]
pool.weight 5 => loaded {"format_version": "1.21.20","minecraft:template_pool": {"description": {"identifier": "demo:p"},"elements": [{"element": {"element_type": "minecraft:empty_pool_element"},"weight": 5}]}} []
pool.weight 0.5 => rejected [InvalidType Error minecraft:template_pool/elements/0/weight 113+8; DocumentRejected Error  -]
pool.weight -1 => rejected [InvalidType Error minecraft:template_pool/elements/0/weight 113+8; DocumentRejected Error  -]
pool.weight true => rejected [InvalidType Error minecraft:template_pool/elements/0/weight 113+8; DocumentRejected Error  -]
pool.weight null => rejected [InvalidType Error minecraft:template_pool/elements/0/weight 123+4; DocumentRejected Error  -]
pool.weight 1e39 => rejected [InvalidType Error minecraft:template_pool/elements/0/weight 113+8; DocumentRejected Error  -]
pool.weight 18446744073709551616 => rejected [InvalidType Error minecraft:template_pool/elements/0/weight 113+8; DocumentRejected Error  -]
pool.weight [] => rejected [InvalidType Error minecraft:template_pool/elements/0/weight/0 124+1; DocumentRejected Error  -]
pool.weight {} => rejected [InvalidType Error minecraft:template_pool/elements/0/weight 124+1; DocumentRejected Error  -]
set.salt "x" => rejected [InvalidType Error minecraft:structure_set/placement/salt 148+6; DocumentRejected Error  -]
set.salt "5" => rejected [InvalidType Error minecraft:structure_set/placement/salt 148+6; DocumentRejected Error  -]
set.salt 5 => loaded {"format_version": "1.21.20","minecraft:structure_set": {"description": {"identifier": "demo:s"},"placement": {"type": "minecraft:random_spread","salt": 5,"separation": 1,"spacing": 4},"structures": [{"structure": "demo:a","weight": 1}]}} []
set.salt 0.5 => rejected [InvalidType Error minecraft:structure_set/placement/salt 148+6; DocumentRejected Error  -]
set.salt -1 => loaded {"format_version": "1.21.20","minecraft:structure_set": {"description": {"identifier": "demo:s"},"placement": {"type": "minecraft:random_spread","salt": -1,"separation": 1,"spacing": 4},"structures": [{"structure": "demo:a","weight": 1}]}} []
set.salt true => rejected [InvalidType Error minecraft:structure_set/placement/salt 148+6; DocumentRejected Error  -]
set.salt null => rejected [InvalidType Error minecraft:structure_set/placement/salt 148+6; DocumentRejected Error  -]
set.salt 1e39 => rejected [InvalidType Error minecraft:structure_set/placement/salt 148+6; DocumentRejected Error  -]
set.salt 18446744073709551616 => rejected [InvalidType Error minecraft:structure_set/placement/salt 148+6; DocumentRejected Error  -]
set.salt [] => rejected [InvalidType Error minecraft:structure_set/placement/salt/0 157+1; DocumentRejected Error  -]
set.salt {} => rejected [InvalidType Error minecraft:structure_set/placement/salt 157+1; DocumentRejected Error  -]
set.separation "x" => rejected [InvalidType Error minecraft:structure_set/placement/separation 159+12; DocumentRejected Error  -]
set.separation "5" => rejected [InvalidType Error minecraft:structure_set/placement/separation 159+12; DocumentRejected Error  -]
set.separation 5 => rejected [InvalidValue Error minecraft:structure_set/placement -]
set.separation 0.5 => rejected [InvalidType Error minecraft:structure_set/placement/separation 159+12; DocumentRejected Error  -]
set.separation -1 => rejected [InvalidType Error minecraft:structure_set/placement/separation 159+12; DocumentRejected Error  -]
set.separation true => rejected [InvalidType Error minecraft:structure_set/placement/separation 159+12; DocumentRejected Error  -]
set.separation null => rejected [InvalidType Error minecraft:structure_set/placement/separation 159+12; DocumentRejected Error  -]
set.separation 1e39 => rejected [InvalidType Error minecraft:structure_set/placement/separation 159+12; DocumentRejected Error  -]
set.separation 18446744073709551616 => rejected [InvalidType Error minecraft:structure_set/placement/separation 159+12; DocumentRejected Error  -]
set.separation [] => rejected [InvalidType Error minecraft:structure_set/placement/separation/0 174+1; DocumentRejected Error  -]
set.separation {} => rejected [InvalidType Error minecraft:structure_set/placement/separation 174+1; DocumentRejected Error  -]
set.spacing "x" => rejected [InvalidType Error minecraft:structure_set/placement/spacing 176+9; DocumentRejected Error  -]
set.spacing "5" => rejected [InvalidType Error minecraft:structure_set/placement/spacing 176+9; DocumentRejected Error  -]
set.spacing 5 => loaded {"format_version": "1.21.20","minecraft:structure_set": {"description": {"identifier": "demo:s"},"placement": {"type": "minecraft:random_spread","salt": 1,"separation": 1,"spacing": 5},"structures": [{"structure": "demo:a","weight": 1}]}} []
set.spacing 0.5 => rejected [InvalidType Error minecraft:structure_set/placement/spacing 176+9; DocumentRejected Error  -]
set.spacing -1 => rejected [InvalidType Error minecraft:structure_set/placement/spacing 176+9; DocumentRejected Error  -]
set.spacing true => rejected [InvalidType Error minecraft:structure_set/placement/spacing 176+9; DocumentRejected Error  -]
set.spacing null => rejected [InvalidType Error minecraft:structure_set/placement/spacing 176+9; DocumentRejected Error  -]
set.spacing 1e39 => rejected [InvalidType Error minecraft:structure_set/placement/spacing 176+9; DocumentRejected Error  -]
set.spacing 18446744073709551616 => rejected [InvalidType Error minecraft:structure_set/placement/spacing 176+9; DocumentRejected Error  -]
set.spacing [] => rejected [InvalidType Error minecraft:structure_set/placement/spacing/0 188+1; DocumentRejected Error  -]
set.spacing {} => rejected [InvalidType Error minecraft:structure_set/placement/spacing 188+1; DocumentRejected Error  -]
set.structures.weight "x" => rejected [InvalidType Error minecraft:structure_set/structures/0/weight 230+8; DocumentRejected Error  -]
set.structures.weight "5" => rejected [InvalidType Error minecraft:structure_set/structures/0/weight 230+8; DocumentRejected Error  -]
set.structures.weight 5 => loaded {"format_version": "1.21.20","minecraft:structure_set": {"description": {"identifier": "demo:s"},"placement": {"type": "minecraft:random_spread","salt": 1,"separation": 1,"spacing": 4},"structures": [{"structure": "demo:a","weight": 5}]}} []
set.structures.weight 0.5 => rejected [InvalidType Error minecraft:structure_set/structures/0/weight 230+8; DocumentRejected Error  -]
set.structures.weight -1 => rejected [InvalidType Error minecraft:structure_set/structures/0/weight 230+8; DocumentRejected Error  -]
set.structures.weight true => rejected [InvalidType Error minecraft:structure_set/structures/0/weight 230+8; DocumentRejected Error  -]
set.structures.weight null => rejected [InvalidType Error minecraft:structure_set/structures/0/weight 230+8; DocumentRejected Error  -]
set.structures.weight 1e39 => rejected [InvalidType Error minecraft:structure_set/structures/0/weight 230+8; DocumentRejected Error  -]
set.structures.weight 18446744073709551616 => rejected [InvalidType Error minecraft:structure_set/structures/0/weight 230+8; DocumentRejected Error  -]
set.structures.weight [] => rejected [InvalidType Error minecraft:structure_set/structures/0/weight/0 241+1; DocumentRejected Error  -]
set.structures.weight {} => rejected [InvalidType Error minecraft:structure_set/structures/0/weight 241+1; DocumentRejected Error  -]
processors.protected_blocks.value "x" => loaded {"format_version": "1.21.20","minecraft:processor_list": {"description": {"identifier": "demo:l"},"processors": [{"processor_type": "minecraft:protected_blocks","value": "x"}]}} []
processors.protected_blocks.value "5" => loaded {"format_version": "1.21.20","minecraft:processor_list": {"description": {"identifier": "demo:l"},"processors": [{"processor_type": "minecraft:protected_blocks","value": "5"}]}} []
processors.protected_blocks.value 5 => rejected [InvalidType Error minecraft:processor_list/processors/0/value 164+7; DocumentRejected Error  -]
processors.protected_blocks.value 0.5 => rejected [InvalidType Error minecraft:processor_list/processors/0/value 164+7; DocumentRejected Error  -]
processors.protected_blocks.value -1 => rejected [InvalidType Error minecraft:processor_list/processors/0/value 164+7; DocumentRejected Error  -]
processors.protected_blocks.value true => rejected [InvalidType Error minecraft:processor_list/processors/0/value 164+7; DocumentRejected Error  -]
processors.protected_blocks.value null => rejected [InvalidType Error minecraft:processor_list/processors/0/value 164+7; DocumentRejected Error  -]
processors.protected_blocks.value 1e39 => rejected [InvalidType Error minecraft:processor_list/processors/0/value 164+7; DocumentRejected Error  -]
processors.protected_blocks.value 18446744073709551616 => rejected [InvalidType Error minecraft:processor_list/processors/0/value 164+7; DocumentRejected Error  -]
processors.protected_blocks.value [] => rejected [InvalidType Error minecraft:processor_list/processors/0/value/0 174+1; DocumentRejected Error  -]
processors.protected_blocks.value {} => rejected [InvalidType Error minecraft:processor_list/processors/0/value 174+1; DocumentRejected Error  -]
processors.capped.limit.value "x" => rejected [InvalidType Error minecraft:processor_list/processors/0/limit/value 184+7; DocumentRejected Error  -]
processors.capped.limit.value "5" => rejected [InvalidType Error minecraft:processor_list/processors/0/limit/value 184+7; DocumentRejected Error  -]
processors.capped.limit.value 5 => loaded {"format_version": "1.21.20","minecraft:processor_list": {"description": {"identifier": "demo:l"},"processors": [{"processor_type": "minecraft:capped","delegate": {"processor_type": "minecraft:block_ignore","blocks": ["minecraft:air"]},"limit": {"type": "constant","value": 5}}]}} []
processors.capped.limit.value 0.5 => rejected [InvalidType Error minecraft:processor_list/processors/0/limit/value 184+7; DocumentRejected Error  -]
processors.capped.limit.value -1 => loaded {"format_version": "1.21.20","minecraft:processor_list": {"description": {"identifier": "demo:l"},"processors": [{"processor_type": "minecraft:capped","delegate": {"processor_type": "minecraft:block_ignore","blocks": ["minecraft:air"]},"limit": {"type": "constant","value": -1}}]}} []
processors.capped.limit.value true => rejected [InvalidType Error minecraft:processor_list/processors/0/limit/value 184+7; DocumentRejected Error  -]
processors.capped.limit.value null => rejected [InvalidType Error minecraft:processor_list/processors/0/limit/value 184+7; DocumentRejected Error  -]
processors.capped.limit.value 1e39 => rejected [InvalidType Error minecraft:processor_list/processors/0/limit/value 184+7; DocumentRejected Error  -]
processors.capped.limit.value 18446744073709551616 => rejected [InvalidType Error minecraft:processor_list/processors/0/limit/value 184+7; DocumentRejected Error  -]
processors.capped.limit.value [] => rejected [InvalidType Error minecraft:processor_list/processors/0/limit/value/0 194+1; DocumentRejected Error  -]
processors.capped.limit.value {} => rejected [InvalidType Error minecraft:processor_list/processors/0/limit/value 194+1; DocumentRejected Error  -]
processors.capped.limit.min_inclusive "x" => rejected [InvalidType Error minecraft:processor_list/processors/0/limit/min_inclusive 183+15; DocumentRejected Error  -]
processors.capped.limit.min_inclusive "5" => rejected [InvalidType Error minecraft:processor_list/processors/0/limit/min_inclusive 183+15; DocumentRejected Error  -]
processors.capped.limit.min_inclusive 5 => loaded {"format_version": "1.21.20","minecraft:processor_list": {"description": {"identifier": "demo:l"},"processors": [{"processor_type": "minecraft:capped","delegate": {"processor_type": "minecraft:block_ignore","blocks": ["minecraft:air"]},"limit": {"type": "uniform","min_inclusive": 5,"max_inclusive": 3}}]}} []
processors.capped.limit.min_inclusive 0.5 => rejected [InvalidType Error minecraft:processor_list/processors/0/limit/min_inclusive 183+15; DocumentRejected Error  -]
processors.capped.limit.min_inclusive -1 => loaded {"format_version": "1.21.20","minecraft:processor_list": {"description": {"identifier": "demo:l"},"processors": [{"processor_type": "minecraft:capped","delegate": {"processor_type": "minecraft:block_ignore","blocks": ["minecraft:air"]},"limit": {"type": "uniform","min_inclusive": -1,"max_inclusive": 3}}]}} []
processors.capped.limit.min_inclusive true => rejected [InvalidType Error minecraft:processor_list/processors/0/limit/min_inclusive 183+15; DocumentRejected Error  -]
processors.capped.limit.min_inclusive null => rejected [InvalidType Error minecraft:processor_list/processors/0/limit/min_inclusive 183+15; DocumentRejected Error  -]
processors.capped.limit.min_inclusive 1e39 => rejected [InvalidType Error minecraft:processor_list/processors/0/limit/min_inclusive 183+15; DocumentRejected Error  -]
processors.capped.limit.min_inclusive 18446744073709551616 => rejected [InvalidType Error minecraft:processor_list/processors/0/limit/min_inclusive 183+15; DocumentRejected Error  -]
processors.capped.limit.min_inclusive [] => rejected [InvalidType Error minecraft:processor_list/processors/0/limit/min_inclusive/0 201+1; DocumentRejected Error  -]
processors.capped.limit.min_inclusive {} => rejected [InvalidType Error minecraft:processor_list/processors/0/limit/min_inclusive 201+1; DocumentRejected Error  -]
processors.rule.random_block_match.probability "x" => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/input_predicate/probability 260+13; DocumentRejected Error  -]
processors.rule.random_block_match.probability "5" => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/input_predicate/probability 260+13; DocumentRejected Error  -]
processors.rule.random_block_match.probability 5 => rejected [InvalidValue Error minecraft:processor_list/processors/0/rules/0/input_predicate/probability -; DocumentRejected Error  -]
processors.rule.random_block_match.probability 0.5 => loaded {"format_version": "1.21.20","minecraft:processor_list": {"description": {"identifier": "demo:l"},"processors": [{"processor_type": "minecraft:rule","rules": [{"input_predicate": {"predicate_type": "minecraft:random_block_match","block": "minecraft:dirt","probability": 0.5},"output_state": {"name": "minecraft:stone"}}]}]}} []
processors.rule.random_block_match.probability -1 => rejected [InvalidValue Error minecraft:processor_list/processors/0/rules/0/input_predicate/probability -; DocumentRejected Error  -]
processors.rule.random_block_match.probability true => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/input_predicate/probability 260+13; DocumentRejected Error  -]
processors.rule.random_block_match.probability null => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/input_predicate/probability 260+13; DocumentRejected Error  -]
processors.rule.random_block_match.probability 1e39 => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/input_predicate/probability 260+13; DocumentRejected Error  -]
processors.rule.random_block_match.probability 18446744073709551616 => rejected [InvalidValue Error minecraft:processor_list/processors/0/rules/0/input_predicate/probability -; DocumentRejected Error  -]
processors.rule.random_block_match.probability [] => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/input_predicate/probability/0 276+1; DocumentRejected Error  -]
processors.rule.random_block_match.probability {} => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/input_predicate/probability 276+1; DocumentRejected Error  -]
processors.rule.tag_match.tag "x" => loaded {"format_version": "1.21.20","minecraft:processor_list": {"description": {"identifier": "demo:l"},"processors": [{"processor_type": "minecraft:rule","rules": [{"input_predicate": {"predicate_type": "minecraft:tag_match","tag": "x"},"output_state": {"name": "minecraft:stone"}}]}]}} []
processors.rule.tag_match.tag "5" => loaded {"format_version": "1.21.20","minecraft:processor_list": {"description": {"identifier": "demo:l"},"processors": [{"processor_type": "minecraft:rule","rules": [{"input_predicate": {"predicate_type": "minecraft:tag_match","tag": "5"},"output_state": {"name": "minecraft:stone"}}]}]}} []
processors.rule.tag_match.tag 5 => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/input_predicate/tag 224+5; DocumentRejected Error  -]
processors.rule.tag_match.tag 0.5 => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/input_predicate/tag 224+5; DocumentRejected Error  -]
processors.rule.tag_match.tag -1 => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/input_predicate/tag 224+5; DocumentRejected Error  -]
processors.rule.tag_match.tag true => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/input_predicate/tag 224+5; DocumentRejected Error  -]
processors.rule.tag_match.tag null => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/input_predicate/tag 224+5; DocumentRejected Error  -]
processors.rule.tag_match.tag 1e39 => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/input_predicate/tag 224+5; DocumentRejected Error  -]
processors.rule.tag_match.tag 18446744073709551616 => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/input_predicate/tag 224+5; DocumentRejected Error  -]
processors.rule.tag_match.tag [] => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/input_predicate/tag/0 232+1; DocumentRejected Error  -]
processors.rule.tag_match.tag {} => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/input_predicate/tag 232+1; DocumentRejected Error  -]
processors.rule.position.min_chance "x" => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/position_predicate/min_chance 241+12; DocumentRejected Error  -]
processors.rule.position.min_chance "5" => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/position_predicate/min_chance 241+12; DocumentRejected Error  -]
processors.rule.position.min_chance 5 => rejected [InvalidValue Error minecraft:processor_list/processors/0/rules/0/position_predicate/min_chance -; DocumentRejected Error  -]
processors.rule.position.min_chance 0.5 => loaded {"format_version": "1.21.20","minecraft:processor_list": {"description": {"identifier": "demo:l"},"processors": [{"processor_type": "minecraft:rule","rules": [{"position_predicate": {"predicate_type": "minecraft:axis_aligned_linear_pos","min_chance": 0.5},"output_state": {"name": "minecraft:stone"}}]}]}} []
processors.rule.position.min_chance -1 => rejected [InvalidValue Error minecraft:processor_list/processors/0/rules/0/position_predicate/min_chance -; DocumentRejected Error  -]
processors.rule.position.min_chance true => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/position_predicate/min_chance 241+12; DocumentRejected Error  -]
processors.rule.position.min_chance null => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/position_predicate/min_chance 255+4; DocumentRejected Error  -]
processors.rule.position.min_chance 1e39 => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/position_predicate/min_chance 241+12; DocumentRejected Error  -]
processors.rule.position.min_chance 18446744073709551616 => rejected [InvalidValue Error minecraft:processor_list/processors/0/rules/0/position_predicate/min_chance -; DocumentRejected Error  -]
processors.rule.position.min_chance [] => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/position_predicate/min_chance/0 256+1; DocumentRejected Error  -]
processors.rule.position.min_chance {} => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/position_predicate/min_chance 256+1; DocumentRejected Error  -]
processors.rule.position.min_dist "x" => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/position_predicate/min_dist 241+10; DocumentRejected Error  -]
processors.rule.position.min_dist "5" => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/position_predicate/min_dist 241+10; DocumentRejected Error  -]
processors.rule.position.min_dist 5 => loaded {"format_version": "1.21.20","minecraft:processor_list": {"description": {"identifier": "demo:l"},"processors": [{"processor_type": "minecraft:rule","rules": [{"position_predicate": {"predicate_type": "minecraft:axis_aligned_linear_pos","min_dist": 5},"output_state": {"name": "minecraft:stone"}}]}]}} []
processors.rule.position.min_dist 0.5 => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/position_predicate/min_dist 241+10; DocumentRejected Error  -]
processors.rule.position.min_dist -1 => rejected [InvalidValue Error minecraft:processor_list/processors/0/rules/0/position_predicate/min_dist -; DocumentRejected Error  -]
processors.rule.position.min_dist true => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/position_predicate/min_dist 241+10; DocumentRejected Error  -]
processors.rule.position.min_dist null => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/position_predicate/min_dist 253+4; DocumentRejected Error  -]
processors.rule.position.min_dist 1e39 => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/position_predicate/min_dist 241+10; DocumentRejected Error  -]
processors.rule.position.min_dist 18446744073709551616 => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/position_predicate/min_dist 241+10; DocumentRejected Error  -]
processors.rule.position.min_dist [] => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/position_predicate/min_dist/0 254+1; DocumentRejected Error  -]
processors.rule.position.min_dist {} => rejected [InvalidType Error minecraft:processor_list/processors/0/rules/0/position_predicate/min_dist 254+1; DocumentRejected Error  -]
metadata 1.21.80 size[0] "x" => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/size/0 99+3]
metadata 1.21.80 size[0] "5" => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/size/0 99+3]
metadata 1.21.80 size[0] 5 => loaded  []
metadata 1.21.80 size[0] 0.5 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/size/0 99+3]
metadata 1.21.80 size[0] -1 => loaded  []
metadata 1.21.80 size[0] true => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/size/0 99+4]
metadata 1.21.80 size[0] null => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/size/0 99+4]
metadata 1.21.80 size[0] 1e39 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/size/0 99+4]
metadata 1.21.80 size[0] 18446744073709551616 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/size/0 99+20]
metadata 1.21.80 size[0] [] => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/size/0/0 100+1]
metadata 1.21.80 size[0] {} => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/size/0 100+1]
metadata 1.21.80 markers.name "x" => loaded  []
metadata 1.21.80 markers.name "5" => loaded  []
metadata 1.21.80 markers.name 5 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/name 122+6]
metadata 1.21.80 markers.name 0.5 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/name 122+6]
metadata 1.21.80 markers.name -1 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/name 122+6]
metadata 1.21.80 markers.name true => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/name 122+6]
metadata 1.21.80 markers.name null => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/name 122+6]
metadata 1.21.80 markers.name 1e39 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/name 122+6]
metadata 1.21.80 markers.name 18446744073709551616 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/name 122+6]
metadata 1.21.80 markers.name [] => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/name/0 131+1]
metadata 1.21.80 markers.name {} => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/name 131+1]
metadata 1.21.80 markers.pos[0] "x" => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/pos/0 231+3]
metadata 1.21.80 markers.pos[0] "5" => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/pos/0 231+3]
metadata 1.21.80 markers.pos[0] 5 => loaded  []
metadata 1.21.80 markers.pos[0] 0.5 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/pos/0 231+3]
metadata 1.21.80 markers.pos[0] -1 => loaded  []
metadata 1.21.80 markers.pos[0] true => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/pos/0 231+4]
metadata 1.21.80 markers.pos[0] null => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/pos/0 231+4]
metadata 1.21.80 markers.pos[0] 1e39 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/pos/0 231+4]
metadata 1.21.80 markers.pos[0] 18446744073709551616 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/pos/0 231+20]
metadata 1.21.80 markers.pos[0] [] => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/pos/0/0 232+1]
metadata 1.21.80 markers.pos[0] {} => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/pos/0 232+1]
metadata 1.21.80 markers.selection_priority "x" => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/selection_priority 241+20]
metadata 1.21.80 markers.selection_priority "5" => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/selection_priority 241+20]
metadata 1.21.80 markers.selection_priority 5 => loaded  []
metadata 1.21.80 markers.selection_priority 0.5 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/selection_priority 241+20]
metadata 1.21.80 markers.selection_priority -1 => loaded  []
metadata 1.21.80 markers.selection_priority true => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/selection_priority 241+20]
metadata 1.21.80 markers.selection_priority null => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/selection_priority 241+20]
metadata 1.21.80 markers.selection_priority 1e39 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/selection_priority 241+20]
metadata 1.21.80 markers.selection_priority 18446744073709551616 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/selection_priority 241+20]
metadata 1.21.80 markers.selection_priority [] => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/selection_priority/0 264+1]
metadata 1.21.80 markers.selection_priority {} => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/selection_priority 264+1]
metadata 1.21.50 size[0] "x" => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/size/0 99+3]
metadata 1.21.50 size[0] "5" => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/size/0 99+3]
metadata 1.21.50 size[0] 5 => loaded  []
metadata 1.21.50 size[0] 0.5 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/size/0 99+3]
metadata 1.21.50 size[0] -1 => loaded  []
metadata 1.21.50 size[0] true => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/size/0 99+4]
metadata 1.21.50 size[0] null => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/size/0 99+4]
metadata 1.21.50 size[0] 1e39 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/size/0 99+4]
metadata 1.21.50 size[0] 18446744073709551616 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/size/0 99+20]
metadata 1.21.50 size[0] [] => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/size/0/0 100+1]
metadata 1.21.50 size[0] {} => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/size/0 100+1]
metadata 1.21.50 markers.name "x" => loaded  []
metadata 1.21.50 markers.name "5" => loaded  []
metadata 1.21.50 markers.name 5 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/name 122+6]
metadata 1.21.50 markers.name 0.5 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/name 122+6]
metadata 1.21.50 markers.name -1 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/name 122+6]
metadata 1.21.50 markers.name true => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/name 122+6]
metadata 1.21.50 markers.name null => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/name 122+6]
metadata 1.21.50 markers.name 1e39 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/name 122+6]
metadata 1.21.50 markers.name 18446744073709551616 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/name 122+6]
metadata 1.21.50 markers.name [] => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/name/0 131+1]
metadata 1.21.50 markers.name {} => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/name 131+1]
metadata 1.21.50 markers.pos[0] "x" => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/pos/0 231+3]
metadata 1.21.50 markers.pos[0] "5" => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/pos/0 231+3]
metadata 1.21.50 markers.pos[0] 5 => loaded  []
metadata 1.21.50 markers.pos[0] 0.5 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/pos/0 231+3]
metadata 1.21.50 markers.pos[0] -1 => loaded  []
metadata 1.21.50 markers.pos[0] true => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/pos/0 231+4]
metadata 1.21.50 markers.pos[0] null => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/pos/0 231+4]
metadata 1.21.50 markers.pos[0] 1e39 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/pos/0 231+4]
metadata 1.21.50 markers.pos[0] 18446744073709551616 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/pos/0 231+20]
metadata 1.21.50 markers.pos[0] [] => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/pos/0/0 232+1]
metadata 1.21.50 markers.pos[0] {} => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/pos/0 232+1]
metadata 1.21.50 markers.selection_priority "x" => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/selection_priority 241+20]
metadata 1.21.50 markers.selection_priority "5" => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/selection_priority 241+20]
metadata 1.21.50 markers.selection_priority 5 => loaded  []
metadata 1.21.50 markers.selection_priority 0.5 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/selection_priority 241+20]
metadata 1.21.50 markers.selection_priority -1 => loaded  []
metadata 1.21.50 markers.selection_priority true => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/selection_priority 241+20]
metadata 1.21.50 markers.selection_priority null => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/selection_priority 241+20]
metadata 1.21.50 markers.selection_priority 1e39 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/selection_priority 241+20]
metadata 1.21.50 markers.selection_priority 18446744073709551616 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/selection_priority 241+20]
metadata 1.21.50 markers.selection_priority [] => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/selection_priority/0 264+1]
metadata 1.21.50 markers.selection_priority {} => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/markers/0/selection_priority 264+1]
metadata 1.21.50 use_legacy_rotation "x" => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/use_legacy_rotation 109+21]
metadata 1.21.50 use_legacy_rotation "5" => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/use_legacy_rotation 109+21]
metadata 1.21.50 use_legacy_rotation 5 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/use_legacy_rotation 109+21]
metadata 1.21.50 use_legacy_rotation 0.5 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/use_legacy_rotation 109+21]
metadata 1.21.50 use_legacy_rotation -1 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/use_legacy_rotation 109+21]
metadata 1.21.50 use_legacy_rotation true => loaded  [DroppedField Info minecraft:jigsaw_structure_metadata/metadata/42/use_legacy_rotation -]
metadata 1.21.50 use_legacy_rotation null => loaded  []
metadata 1.21.50 use_legacy_rotation 1e39 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/use_legacy_rotation 109+21]
metadata 1.21.50 use_legacy_rotation 18446744073709551616 => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/use_legacy_rotation 109+21]
metadata 1.21.50 use_legacy_rotation [] => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/use_legacy_rotation/0 133+1]
metadata 1.21.50 use_legacy_rotation {} => rejected [InvalidType Error minecraft:jigsaw_structure_metadata/metadata/42/use_legacy_rotation 133+1]
structure.dimension_padding "x" => rejected [InvalidType Error minecraft:jigsaw/dimension_padding -; DocumentRejected Error  -]
structure.dimension_padding "5" => rejected [InvalidType Error minecraft:jigsaw/dimension_padding -; DocumentRejected Error  -]
structure.dimension_padding 5 => loaded {"format_version": "1.21.20","minecraft:jigsaw": {"description": {"identifier": "demo:a"},"step": "surface_structures","start_pool": "demo:start","max_depth": 1,"start_height": {"type": "constant","value": {"absolute": 0}},"dimension_padding": 5}} []
structure.dimension_padding 0.5 => rejected [InvalidType Error minecraft:jigsaw/dimension_padding -; DocumentRejected Error  -]
structure.dimension_padding -1 => rejected [InvalidValue Error minecraft:jigsaw 50+204; DocumentRejected Error  -]
structure.dimension_padding true => rejected [InvalidType Error minecraft:jigsaw/dimension_padding -; DocumentRejected Error  -]
structure.dimension_padding null => rejected [InvalidType Error minecraft:jigsaw/dimension_padding -; DocumentRejected Error  -]
structure.dimension_padding 1e39 => rejected [InvalidType Error minecraft:jigsaw/dimension_padding -; DocumentRejected Error  -]
structure.dimension_padding 18446744073709551616 => rejected [InvalidType Error minecraft:jigsaw/dimension_padding -; DocumentRejected Error  -]
structure.dimension_padding [] => rejected [InvalidType Error minecraft:jigsaw/dimension_padding -; DocumentRejected Error  -]
structure.dimension_padding {} => loaded {"format_version": "1.21.20","minecraft:jigsaw": {"description": {"identifier": "demo:a"},"step": "surface_structures","start_pool": "demo:start","max_depth": 1,"start_height": {"type": "constant","value": {"absolute": 0}},"dimension_padding": {}}} []
structure.max_distance "x" => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center -; DocumentRejected Error  -]
structure.max_distance "5" => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center -; DocumentRejected Error  -]
structure.max_distance 5 => loaded {"format_version": "1.21.20","minecraft:jigsaw": {"description": {"identifier": "demo:a"},"step": "surface_structures","start_pool": "demo:start","max_depth": 1,"start_height": {"type": "constant","value": {"absolute": 0}},"max_distance_from_center": 5}} []
structure.max_distance 0.5 => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center -; DocumentRejected Error  -]
structure.max_distance -1 => rejected [InvalidValue Error minecraft:jigsaw/max_distance_from_center -; DocumentRejected Error  -]
structure.max_distance true => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center -; DocumentRejected Error  -]
structure.max_distance null => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center -; DocumentRejected Error  -]
structure.max_distance 1e39 => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center -; DocumentRejected Error  -]
structure.max_distance 18446744073709551616 => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center -; DocumentRejected Error  -]
structure.max_distance [] => rejected [InvalidType Error minecraft:jigsaw/max_distance_from_center -; DocumentRejected Error  -]
structure.max_distance {} => rejected [MissingRequiredField Error minecraft:jigsaw/max_distance_from_center/horizontal 259+1; DocumentRejected Error  -]
processors.capped.limit "x" => rejected [InvalidType Error minecraft:processor_list/processors/0/limit -; DocumentRejected Error  -]
processors.capped.limit "5" => rejected [InvalidType Error minecraft:processor_list/processors/0/limit -; DocumentRejected Error  -]
processors.capped.limit 5 => loaded {"format_version": "1.21.20","minecraft:processor_list": {"description": {"identifier": "demo:l"},"processors": [{"processor_type": "minecraft:capped","delegate": {"processor_type": "minecraft:block_ignore","blocks": ["minecraft:air"]},"limit": 5}]}} []
processors.capped.limit 0.5 => rejected [InvalidType Error minecraft:processor_list/processors/0/limit -; DocumentRejected Error  -]
processors.capped.limit -1 => loaded {"format_version": "1.21.20","minecraft:processor_list": {"description": {"identifier": "demo:l"},"processors": [{"processor_type": "minecraft:capped","delegate": {"processor_type": "minecraft:block_ignore","blocks": ["minecraft:air"]},"limit": -1}]}} []
processors.capped.limit true => rejected [InvalidType Error minecraft:processor_list/processors/0/limit -; DocumentRejected Error  -]
processors.capped.limit null => rejected [InvalidType Error minecraft:processor_list/processors/0/limit -; DocumentRejected Error  -]
processors.capped.limit 1e39 => rejected [InvalidType Error minecraft:processor_list/processors/0/limit -; DocumentRejected Error  -]
processors.capped.limit 18446744073709551616 => rejected [InvalidType Error minecraft:processor_list/processors/0/limit -; DocumentRejected Error  -]
processors.capped.limit [] => rejected [InvalidType Error minecraft:processor_list/processors/0/limit -; DocumentRejected Error  -]
processors.capped.limit {} => rejected [MissingRequiredField Error minecraft:processor_list/processors/0/limit/provider_type 164+1; DocumentRejected Error  -]
"#;

/// Documents with a struct member written `null`: what the slice's null rule makes of it,
/// told once (rejected where the input rejects nulls, read as its defaults elsewhere).
const NULL_STRUCTS: &[&str] = &[
    r#"{"format_version": "1.21.20", "minecraft:jigsaw": {"description": {"identifier": "demo:a"}, "step": "surface_structures", "start_pool": "demo:start", "max_depth": 1, "start_height": null}}"#,
    r#"{"format_version": "1.21.20", "minecraft:jigsaw": {"description": {"identifier": "demo:a"}, "step": "surface_structures", "start_pool": "demo:start", "max_depth": 1, "start_height": {"type": "constant", "value": null}}}"#,
    r#"{"format_version": "1.21.20", "minecraft:structure_set": {"description": {"identifier": "demo:s"}, "placement": null, "structures": [{"structure": "demo:a", "weight": 1}]}}"#,
];

#[test]
fn a_null_struct_member_reads_as_pinned() {
    let mut table = String::new();
    for text in NULL_STRUCTS {
        let _ = writeln!(table, "{}", line(text));
    }
    assert!(
        table == NULL_STRUCTS_EXPECTED,
        "--- actual ---\n{table}--- end ---"
    );
}

/// The null struct table.
const NULL_STRUCTS_EXPECTED: &str = r"rejected [InvalidType Error minecraft:jigsaw/start_height 182+4; DocumentRejected Error  -]
rejected [InvalidEnumValue Error minecraft:jigsaw/start_height/value 203+7; DocumentRejected Error  -]
rejected [InvalidType Error minecraft:structure_set/placement 112+4; DocumentRejected Error  -]
";

/// `max_distance_from_center` at the ends of its bounds, as a number and as an object.
const DISTANCES: &[&str] = &[
    "0",
    "1",
    "128",
    "129",
    r#"{"horizontal": 0}"#,
    r#"{"horizontal": 129, "vertical": 0}"#,
    r#"{"horizontal": 128, "vertical": 1}"#,
];

#[test]
fn max_distance_from_center_reads_its_bounds_as_pinned() {
    let mut table = String::new();
    for distance in DISTANCES {
        let text = format!(
            r#"{{"format_version": "1.21.20", "minecraft:jigsaw": {{"description": {{"identifier": "demo:a"}}, "step": "surface_structures", "start_pool": "demo:start", "max_depth": 1, "start_height": {{"type": "constant", "value": {{"absolute": 0}}}}, "max_distance_from_center": {distance}}}}}"#
        );
        let _ = writeln!(table, "{distance} => {}", line(&text));
    }
    assert!(
        table == DISTANCES_EXPECTED,
        "--- actual ---\n{table}--- end ---"
    );
}

/// The bounds table.
const DISTANCES_EXPECTED: &str = r#"0 => rejected [InvalidValue Error minecraft:jigsaw/max_distance_from_center -; DocumentRejected Error  -]
1 => loaded {"format_version": "1.21.20","minecraft:jigsaw": {"description": {"identifier": "demo:a"},"step": "surface_structures","start_pool": "demo:start","max_depth": 1,"start_height": {"type": "constant","value": {"absolute": 0}},"max_distance_from_center": 1}} []
128 => loaded {"format_version": "1.21.20","minecraft:jigsaw": {"description": {"identifier": "demo:a"},"step": "surface_structures","start_pool": "demo:start","max_depth": 1,"start_height": {"type": "constant","value": {"absolute": 0}},"max_distance_from_center": 128}} []
129 => rejected [InvalidValue Error minecraft:jigsaw/max_distance_from_center -; DocumentRejected Error  -]
{"horizontal": 0} => rejected [InvalidValue Error minecraft:jigsaw/max_distance_from_center/horizontal -; DocumentRejected Error  -]
{"horizontal": 129, "vertical": 0} => rejected [InvalidValue Error minecraft:jigsaw/max_distance_from_center/horizontal -; InvalidValue Error minecraft:jigsaw/max_distance_from_center/vertical -; DocumentRejected Error  -]
{"horizontal": 128, "vertical": 1} => loaded {"format_version": "1.21.20","minecraft:jigsaw": {"description": {"identifier": "demo:a"},"step": "surface_structures","start_pool": "demo:start","max_depth": 1,"start_height": {"type": "constant","value": {"absolute": 0}},"max_distance_from_center": {"horizontal": 128,"vertical": 1}}} []
"#;
