//! Every scalar member of a biome, in the newest slice (1.26.30), the 1.21.110 one and the beta
//! document, read from every JSON type: each line pins what the load says
//! (code, severity, path, span) and, when the document loads, the document written back. The
//! members were strict wrappers; this table is what they read, and the oracle of the change
//! that makes them plain.

use std::fmt::Write as _;

use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::{Content, Document, Rejected, WriteWith};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackPath, PackType};
use bedrock_addon_family_biome::{Beta, Biome};

/// A behaviour pack's context with the Beta APIs toggle on.
fn context() -> LoadContext {
    let mut cx = LoadContext::detached();
    cx.pack_type = PackType::Behavior;
    cx.beta_apis = true;
    cx
}

/// What a load gave: the document written back, or nothing, and the findings.
fn outcome<T: WriteWith>(read: Result<Document<T>, Rejected>) -> (Option<String>, Diagnostics) {
    match read {
        Ok(document) => (
            Some(document.to_json().unwrap_or_default()),
            document.diagnostics().clone(),
        ),
        Err(rejected) => (None, rejected.diagnostics),
    }
}

/// Loads `text` as the document it is.
fn load(text: &str) -> (Option<String>, Diagnostics) {
    let path = PackPath::from_static("biomes/demo.biome.json");
    if text.contains(r#""format_version": "beta""#) {
        outcome(<Beta as Content>::parse(&path, text.as_bytes(), &context()))
    } else {
        outcome(<Biome as Content>::parse(
            &path,
            text.as_bytes(),
            &context(),
        ))
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
        "1.26.30 climate.downfall",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:climate": {"downfall": @@}}}}"#,
    ),
    (
        "1.26.30 climate.snow_accumulation[0]",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:climate": {"snow_accumulation": [@@, 0.5]}}}}"#,
    ),
    (
        "1.26.30 climate.temperature",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:climate": {"temperature": @@}}}}"#,
    ),
    (
        "1.26.30 creature_spawn_probability.probability",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:creature_spawn_probability": {"probability": @@}}}}"#,
    ),
    (
        "1.26.30 humidity.is_humid",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:humidity": {"is_humid": @@}}}}"#,
    ),
    (
        "1.26.30 mountain_parameters.north_slopes",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:mountain_parameters": {"steep_material_adjustment": {"north_slopes": @@}}}}}"#,
    ),
    (
        "1.26.30 mountain_parameters.top_slide.enabled",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:mountain_parameters": {"top_slide": {"enabled": @@}}}}}"#,
    ),
    (
        "1.26.30 multinoise.target_temperature",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:multinoise_generation_rules": {"target_temperature": @@}}}}"#,
    ),
    (
        "1.26.30 multinoise.weight",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:multinoise_generation_rules": {"weight": @@}}}}"#,
    ),
    (
        "1.26.30 overworld_height.noise_params[0]",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:overworld_height": {"noise_params": [@@, 0.2]}}}}"#,
    ),
    (
        "1.26.30 replace_biomes.dimension",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:replace_biomes": {"replacements": [{"dimension": @@, "targets": ["minecraft:plains"], "amount": 0.5, "noise_frequency_scale": 1}]}}}}"#,
    ),
    (
        "1.26.30 replace_biomes.targets[0]",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:replace_biomes": {"replacements": [{"dimension": "minecraft:overworld", "targets": [@@], "amount": 0.5, "noise_frequency_scale": 1}]}}}}"#,
    ),
    (
        "1.26.30 replace_biomes.amount",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:replace_biomes": {"replacements": [{"dimension": "minecraft:overworld", "targets": ["minecraft:plains"], "amount": @@, "noise_frequency_scale": 1}]}}}}"#,
    ),
    (
        "1.26.30 replace_biomes.noise_frequency_scale",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:replace_biomes": {"replacements": [{"dimension": "minecraft:overworld", "targets": ["minecraft:plains"], "amount": 0.5, "noise_frequency_scale": @@}]}}}}"#,
    ),
    (
        "1.26.30 tags.tags[0]",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:tags": {"tags": [@@]}}}}"#,
    ),
    (
        "1.26.30 surface_builder.overworld.sea_floor_depth",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:overworld", "sea_floor_depth": @@, "sea_floor_material": "minecraft:stone", "foundation_material": "minecraft:stone", "mid_material": "minecraft:stone", "top_material": "minecraft:stone", "sea_material": "minecraft:stone"}}}}}"#,
    ),
    (
        "1.26.30 surface_builder.swamp.max_puddle",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:swamp", "sea_floor_depth": 7, "sea_floor_material": "minecraft:stone", "foundation_material": "minecraft:stone", "mid_material": "minecraft:stone", "top_material": "minecraft:stone", "sea_material": "minecraft:stone", "max_puddle_depth_below_sea_level": @@}}}}}"#,
    ),
    (
        "1.26.30 surface_builder.mesa.bryce_pillars",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:mesa", "sea_floor_depth": 7, "sea_floor_material": "minecraft:stone", "foundation_material": "minecraft:stone", "mid_material": "minecraft:stone", "top_material": "minecraft:stone", "sea_material": "minecraft:stone", "clay_material": "minecraft:stone", "hard_clay_material": "minecraft:stone", "bryce_pillars": @@, "has_forest": false}}}}}"#,
    ),
    (
        "1.26.30 surface_builder.noise_gradient.name",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:noise_gradient", "noise_descriptor": {"name": @@, "first_octave": -4, "amplitudes": [1.0]}, "noise_block_specifiers": [{"range": {"min": 0.5, "max": 0.7}, "block": "minecraft:calcite"}]}}}}}"#,
    ),
    (
        "1.26.30 surface_builder.noise_gradient.first_octave",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:noise_gradient", "noise_descriptor": {"name": "test:gradient", "first_octave": @@, "amplitudes": [1.0]}, "noise_block_specifiers": [{"range": {"min": 0.5, "max": 0.7}, "block": "minecraft:calcite"}]}}}}}"#,
    ),
    (
        "1.26.30 surface_builder.noise_gradient.amplitudes[0]",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:noise_gradient", "noise_descriptor": {"name": "test:gradient", "first_octave": -4, "amplitudes": [@@]}, "noise_block_specifiers": [{"range": {"min": 0.5, "max": 0.7}, "block": "minecraft:calcite"}]}}}}}"#,
    ),
    (
        "1.26.30 surface_builder.noise_gradient.noise_block_specifiers.noise",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:noise_gradient", "noise_descriptor": {"name": "test:gradient", "first_octave": -4, "amplitudes": [1.0]}, "noise_block_specifiers": [{"range": {"min": 0.5, "max": 0.7}, "block": "minecraft:calcite", "noise": @@}]}}}}}"#,
    ),
    (
        "1.26.30 surface_builder.noise_gradient.noise_block_specifiers.threshold",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:noise_gradient", "noise_descriptor": {"name": "test:gradient", "first_octave": -4, "amplitudes": [1.0]}, "noise_block_specifiers": [{"range": {"min": 0.5, "max": 0.7}, "block": "minecraft:calcite", "threshold": @@}]}}}}}"#,
    ),
    (
        "1.26.30 surface_material_adjustments.noise_range[0]",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:surface_material_adjustments": {"adjustments": [{"materials": {"top_material": "minecraft:sand"}, "noise_range": [@@, 0.5]}]}}}}"#,
    ),
    (
        "1.26.30 surface_material_adjustments.noise_frequency_scale",
        r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:surface_material_adjustments": {"adjustments": [{"materials": {"top_material": "minecraft:sand"}, "noise_frequency_scale": @@}]}}}}"#,
    ),
    (
        "1.21.110 climate.temperature",
        r#"{"format_version": "1.21.110", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:climate": {"temperature": @@}}}}"#,
    ),
    (
        "1.21.110 humidity.is_humid",
        r#"{"format_version": "1.21.110", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:humidity": {"is_humid": @@}}}}"#,
    ),
    (
        "1.21.110 replace_biomes.amount",
        r#"{"format_version": "1.21.110", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:replace_biomes": {"replacements": [{"dimension": "minecraft:overworld", "targets": ["minecraft:plains"], "amount": @@, "noise_frequency_scale": 1}]}}}}"#,
    ),
    (
        "1.21.110 surface_builder.overworld.sea_floor_depth",
        r#"{"format_version": "1.21.110", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:overworld", "sea_floor_depth": @@, "sea_floor_material": "minecraft:stone", "foundation_material": "minecraft:stone", "mid_material": "minecraft:stone", "top_material": "minecraft:stone", "sea_material": "minecraft:stone"}}}}}"#,
    ),
    (
        "beta bool_attr.value",
        r#"{"format_version": "beta", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {}, "environment_attributes": {"minecraft:experimental_bool_attr": {"value": @@, "operation": "override"}}}}"#,
    ),
    (
        "beta float_attr.value",
        r#"{"format_version": "beta", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {}, "environment_attributes": {"minecraft:experimental_float_attr_1": {"value": @@, "operation": "override"}}}}"#,
    ),
];

/// `text` on one line: each line trimmed, the lines joined.
fn compact(text: &str) -> String {
    text.lines().map(str::trim).collect()
}

/// One line of the table: what loading `text` gave.
fn line(text: &str) -> String {
    let (written, diagnostics) = load(text);
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
    for (name, place) in PLACES {
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
const EXPECTED: &str = r#"1.26.30 climate.downfall "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/downfall 128+10]
1.26.30 climate.downfall "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/downfall 128+10]
1.26.30 climate.downfall 5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:climate": {"downfall": 5.0}}}} []
1.26.30 climate.downfall 0.5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:climate": {"downfall": 0.5}}}} []
1.26.30 climate.downfall -1 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:climate": {"downfall": -1.0}}}} []
1.26.30 climate.downfall true => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/downfall 128+10]
1.26.30 climate.downfall null => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/downfall 140+4]
1.26.30 climate.downfall 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/downfall 128+10]
1.26.30 climate.downfall 18446744073709551616 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:climate": {"downfall": 1.8446744e19}}}} []
1.26.30 climate.downfall [] => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/downfall/0 141+1]
1.26.30 climate.downfall {} => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/downfall 141+1]
1.26.30 climate.snow_accumulation[0] "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/snow_accumulation/0 150+3]
1.26.30 climate.snow_accumulation[0] "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/snow_accumulation/0 150+3]
1.26.30 climate.snow_accumulation[0] 5 => rejected [InvalidValue Error minecraft:biome/components/minecraft:climate/snow_accumulation -]
1.26.30 climate.snow_accumulation[0] 0.5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:climate": {"snow_accumulation": [0.5,0.5]}}}} []
1.26.30 climate.snow_accumulation[0] -1 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:climate": {"snow_accumulation": [-1.0,0.5]}}}} []
1.26.30 climate.snow_accumulation[0] true => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/snow_accumulation/0 150+4]
1.26.30 climate.snow_accumulation[0] null => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/snow_accumulation/0 150+4]
1.26.30 climate.snow_accumulation[0] 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/snow_accumulation/0 150+4]
1.26.30 climate.snow_accumulation[0] 18446744073709551616 => rejected [InvalidValue Error minecraft:biome/components/minecraft:climate/snow_accumulation -]
1.26.30 climate.snow_accumulation[0] [] => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/snow_accumulation/0/0 151+1]
1.26.30 climate.snow_accumulation[0] {} => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/snow_accumulation/0 151+1]
1.26.30 climate.temperature "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/temperature 128+13]
1.26.30 climate.temperature "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/temperature 128+13]
1.26.30 climate.temperature 5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:climate": {"temperature": 5.0}}}} []
1.26.30 climate.temperature 0.5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:climate": {"temperature": 0.5}}}} []
1.26.30 climate.temperature -1 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:climate": {"temperature": -1.0}}}} []
1.26.30 climate.temperature true => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/temperature 128+13]
1.26.30 climate.temperature null => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/temperature 143+4]
1.26.30 climate.temperature 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/temperature 128+13]
1.26.30 climate.temperature 18446744073709551616 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:climate": {"temperature": 1.8446744e19}}}} []
1.26.30 climate.temperature [] => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/temperature/0 144+1]
1.26.30 climate.temperature {} => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/temperature 144+1]
1.26.30 creature_spawn_probability.probability "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:creature_spawn_probability/probability 147+13]
1.26.30 creature_spawn_probability.probability "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:creature_spawn_probability/probability 147+13]
1.26.30 creature_spawn_probability.probability 5 => rejected [InvalidValue Error minecraft:biome/components/minecraft:creature_spawn_probability/probability -]
1.26.30 creature_spawn_probability.probability 0.5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:creature_spawn_probability": {"probability": 0.5}}}} []
1.26.30 creature_spawn_probability.probability -1 => rejected [InvalidValue Error minecraft:biome/components/minecraft:creature_spawn_probability/probability -]
1.26.30 creature_spawn_probability.probability true => rejected [InvalidType Error minecraft:biome/components/minecraft:creature_spawn_probability/probability 147+13]
1.26.30 creature_spawn_probability.probability null => rejected [InvalidType Error minecraft:biome/components/minecraft:creature_spawn_probability/probability 162+4]
1.26.30 creature_spawn_probability.probability 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:creature_spawn_probability/probability 147+13]
1.26.30 creature_spawn_probability.probability 18446744073709551616 => rejected [InvalidValue Error minecraft:biome/components/minecraft:creature_spawn_probability/probability -]
1.26.30 creature_spawn_probability.probability [] => rejected [InvalidType Error minecraft:biome/components/minecraft:creature_spawn_probability/probability/0 163+1]
1.26.30 creature_spawn_probability.probability {} => rejected [InvalidType Error minecraft:biome/components/minecraft:creature_spawn_probability/probability 163+1]
1.26.30 humidity.is_humid "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:humidity/is_humid 129+10]
1.26.30 humidity.is_humid "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:humidity/is_humid 129+10]
1.26.30 humidity.is_humid 5 => rejected [InvalidType Error minecraft:biome/components/minecraft:humidity/is_humid 129+10]
1.26.30 humidity.is_humid 0.5 => rejected [InvalidType Error minecraft:biome/components/minecraft:humidity/is_humid 129+10]
1.26.30 humidity.is_humid -1 => rejected [InvalidType Error minecraft:biome/components/minecraft:humidity/is_humid 129+10]
1.26.30 humidity.is_humid true => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:humidity": {"is_humid": true}}}} []
1.26.30 humidity.is_humid null => rejected [InvalidType Error minecraft:biome/components/minecraft:humidity/is_humid 129+10]
1.26.30 humidity.is_humid 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:humidity/is_humid 129+10]
1.26.30 humidity.is_humid 18446744073709551616 => rejected [InvalidType Error minecraft:biome/components/minecraft:humidity/is_humid 129+10]
1.26.30 humidity.is_humid [] => rejected [InvalidType Error minecraft:biome/components/minecraft:humidity/is_humid/0 142+1]
1.26.30 humidity.is_humid {} => rejected [InvalidType Error minecraft:biome/components/minecraft:humidity/is_humid 142+1]
1.26.30 mountain_parameters.north_slopes "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:mountain_parameters/steep_material_adjustment/north_slopes 170+14]
1.26.30 mountain_parameters.north_slopes "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:mountain_parameters/steep_material_adjustment/north_slopes 170+14]
1.26.30 mountain_parameters.north_slopes 5 => rejected [InvalidType Error minecraft:biome/components/minecraft:mountain_parameters/steep_material_adjustment/north_slopes 170+14]
1.26.30 mountain_parameters.north_slopes 0.5 => rejected [InvalidType Error minecraft:biome/components/minecraft:mountain_parameters/steep_material_adjustment/north_slopes 170+14]
1.26.30 mountain_parameters.north_slopes -1 => rejected [InvalidType Error minecraft:biome/components/minecraft:mountain_parameters/steep_material_adjustment/north_slopes 170+14]
1.26.30 mountain_parameters.north_slopes true => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:mountain_parameters": {"steep_material_adjustment": {"north_slopes": true}}}}} []
1.26.30 mountain_parameters.north_slopes null => rejected [InvalidType Error minecraft:biome/components/minecraft:mountain_parameters/steep_material_adjustment/north_slopes 186+4]
1.26.30 mountain_parameters.north_slopes 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:mountain_parameters/steep_material_adjustment/north_slopes 170+14]
1.26.30 mountain_parameters.north_slopes 18446744073709551616 => rejected [InvalidType Error minecraft:biome/components/minecraft:mountain_parameters/steep_material_adjustment/north_slopes 170+14]
1.26.30 mountain_parameters.north_slopes [] => rejected [InvalidType Error minecraft:biome/components/minecraft:mountain_parameters/steep_material_adjustment/north_slopes/0 187+1]
1.26.30 mountain_parameters.north_slopes {} => rejected [InvalidType Error minecraft:biome/components/minecraft:mountain_parameters/steep_material_adjustment/north_slopes 187+1]
1.26.30 mountain_parameters.top_slide.enabled "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:mountain_parameters/top_slide/enabled 154+9]
1.26.30 mountain_parameters.top_slide.enabled "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:mountain_parameters/top_slide/enabled 154+9]
1.26.30 mountain_parameters.top_slide.enabled 5 => rejected [InvalidType Error minecraft:biome/components/minecraft:mountain_parameters/top_slide/enabled 154+9]
1.26.30 mountain_parameters.top_slide.enabled 0.5 => rejected [InvalidType Error minecraft:biome/components/minecraft:mountain_parameters/top_slide/enabled 154+9]
1.26.30 mountain_parameters.top_slide.enabled -1 => rejected [InvalidType Error minecraft:biome/components/minecraft:mountain_parameters/top_slide/enabled 154+9]
1.26.30 mountain_parameters.top_slide.enabled true => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:mountain_parameters": {"top_slide": {"enabled": true}}}}} []
1.26.30 mountain_parameters.top_slide.enabled null => rejected [InvalidType Error minecraft:biome/components/minecraft:mountain_parameters/top_slide/enabled 154+9]
1.26.30 mountain_parameters.top_slide.enabled 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:mountain_parameters/top_slide/enabled 154+9]
1.26.30 mountain_parameters.top_slide.enabled 18446744073709551616 => rejected [InvalidType Error minecraft:biome/components/minecraft:mountain_parameters/top_slide/enabled 154+9]
1.26.30 mountain_parameters.top_slide.enabled [] => rejected [InvalidType Error minecraft:biome/components/minecraft:mountain_parameters/top_slide/enabled/0 166+1]
1.26.30 mountain_parameters.top_slide.enabled {} => rejected [InvalidType Error minecraft:biome/components/minecraft:mountain_parameters/top_slide/enabled 166+1]
1.26.30 multinoise.target_temperature "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:multinoise_generation_rules/target_temperature 148+20]
1.26.30 multinoise.target_temperature "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:multinoise_generation_rules/target_temperature 148+20]
1.26.30 multinoise.target_temperature 5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:multinoise_generation_rules": {"target_temperature": 5.0}}}} [InvalidValue Error minecraft:biome/components/minecraft:multinoise_generation_rules -]
1.26.30 multinoise.target_temperature 0.5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:multinoise_generation_rules": {"target_temperature": 0.5}}}} [InvalidValue Error minecraft:biome/components/minecraft:multinoise_generation_rules -]
1.26.30 multinoise.target_temperature -1 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:multinoise_generation_rules": {"target_temperature": -1.0}}}} [InvalidValue Error minecraft:biome/components/minecraft:multinoise_generation_rules -]
1.26.30 multinoise.target_temperature true => rejected [InvalidType Error minecraft:biome/components/minecraft:multinoise_generation_rules/target_temperature 148+20]
1.26.30 multinoise.target_temperature null => rejected [InvalidType Error minecraft:biome/components/minecraft:multinoise_generation_rules/target_temperature 170+4]
1.26.30 multinoise.target_temperature 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:multinoise_generation_rules/target_temperature 148+20]
1.26.30 multinoise.target_temperature 18446744073709551616 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:multinoise_generation_rules": {"target_temperature": 1.8446744e19}}}} [InvalidValue Error minecraft:biome/components/minecraft:multinoise_generation_rules -]
1.26.30 multinoise.target_temperature [] => rejected [InvalidType Error minecraft:biome/components/minecraft:multinoise_generation_rules/target_temperature/0 171+1]
1.26.30 multinoise.target_temperature {} => rejected [InvalidType Error minecraft:biome/components/minecraft:multinoise_generation_rules/target_temperature 171+1]
1.26.30 multinoise.weight "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:multinoise_generation_rules/weight 148+8]
1.26.30 multinoise.weight "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:multinoise_generation_rules/weight 148+8]
1.26.30 multinoise.weight 5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:multinoise_generation_rules": {"weight": 5.0}}}} [InvalidValue Error minecraft:biome/components/minecraft:multinoise_generation_rules -]
1.26.30 multinoise.weight 0.5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:multinoise_generation_rules": {"weight": 0.5}}}} [InvalidValue Error minecraft:biome/components/minecraft:multinoise_generation_rules -]
1.26.30 multinoise.weight -1 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:multinoise_generation_rules": {"weight": -1.0}}}} [InvalidValue Error minecraft:biome/components/minecraft:multinoise_generation_rules -]
1.26.30 multinoise.weight true => rejected [InvalidType Error minecraft:biome/components/minecraft:multinoise_generation_rules/weight 148+8]
1.26.30 multinoise.weight null => rejected [InvalidType Error minecraft:biome/components/minecraft:multinoise_generation_rules/weight 158+4]
1.26.30 multinoise.weight 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:multinoise_generation_rules/weight 148+8]
1.26.30 multinoise.weight 18446744073709551616 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:multinoise_generation_rules": {"weight": 1.8446744e19}}}} [InvalidValue Error minecraft:biome/components/minecraft:multinoise_generation_rules -]
1.26.30 multinoise.weight [] => rejected [InvalidType Error minecraft:biome/components/minecraft:multinoise_generation_rules/weight/0 159+1]
1.26.30 multinoise.weight {} => rejected [InvalidType Error minecraft:biome/components/minecraft:multinoise_generation_rules/weight 159+1]
1.26.30 overworld_height.noise_params[0] "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:overworld_height/noise_params/0 154+3]
1.26.30 overworld_height.noise_params[0] "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:overworld_height/noise_params/0 154+3]
1.26.30 overworld_height.noise_params[0] 5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:overworld_height": {"noise_params": [5.0,0.2]}}}} []
1.26.30 overworld_height.noise_params[0] 0.5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:overworld_height": {"noise_params": [0.5,0.2]}}}} []
1.26.30 overworld_height.noise_params[0] -1 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:overworld_height": {"noise_params": [-1.0,0.2]}}}} []
1.26.30 overworld_height.noise_params[0] true => rejected [InvalidType Error minecraft:biome/components/minecraft:overworld_height/noise_params/0 154+4]
1.26.30 overworld_height.noise_params[0] null => rejected [InvalidType Error minecraft:biome/components/minecraft:overworld_height/noise_params/0 154+4]
1.26.30 overworld_height.noise_params[0] 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:overworld_height/noise_params/0 154+4]
1.26.30 overworld_height.noise_params[0] 18446744073709551616 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:overworld_height": {"noise_params": [1.8446744e19,0.2]}}}} []
1.26.30 overworld_height.noise_params[0] [] => rejected [InvalidType Error minecraft:biome/components/minecraft:overworld_height/noise_params/0/0 155+1]
1.26.30 overworld_height.noise_params[0] {} => rejected [InvalidType Error minecraft:biome/components/minecraft:overworld_height/noise_params/0 155+1]
1.26.30 replace_biomes.dimension "x" => rejected [InvalidValue Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/dimension -]
1.26.30 replace_biomes.dimension "5" => rejected [InvalidValue Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/dimension -]
1.26.30 replace_biomes.dimension 5 => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/dimension 153+11]
1.26.30 replace_biomes.dimension 0.5 => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/dimension 153+11]
1.26.30 replace_biomes.dimension -1 => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/dimension 153+11]
1.26.30 replace_biomes.dimension true => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/dimension 153+11]
1.26.30 replace_biomes.dimension null => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/dimension 153+11]
1.26.30 replace_biomes.dimension 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/dimension 153+11]
1.26.30 replace_biomes.dimension 18446744073709551616 => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/dimension 153+11]
1.26.30 replace_biomes.dimension [] => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/dimension/0 167+1]
1.26.30 replace_biomes.dimension {} => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/dimension 167+1]
1.26.30 replace_biomes.targets[0] "x" => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:replace_biomes": {"replacements": [{"dimension": "minecraft:overworld","targets": ["x"],"amount": 0.5,"noise_frequency_scale": 1.0}]}}}} []
1.26.30 replace_biomes.targets[0] "5" => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:replace_biomes": {"replacements": [{"dimension": "minecraft:overworld","targets": ["5"],"amount": 0.5,"noise_frequency_scale": 1.0}]}}}} []
1.26.30 replace_biomes.targets[0] 5 => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/targets/0 201+1]
1.26.30 replace_biomes.targets[0] 0.5 => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/targets/0 201+3]
1.26.30 replace_biomes.targets[0] -1 => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/targets/0 201+2]
1.26.30 replace_biomes.targets[0] true => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/targets/0 201+4]
1.26.30 replace_biomes.targets[0] null => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/targets/0 201+4]
1.26.30 replace_biomes.targets[0] 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/targets/0 201+4]
1.26.30 replace_biomes.targets[0] 18446744073709551616 => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/targets/0 201+20]
1.26.30 replace_biomes.targets[0] [] => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/targets/0/0 202+1]
1.26.30 replace_biomes.targets[0] {} => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/targets/0 202+1]
1.26.30 replace_biomes.amount "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount 222+8]
1.26.30 replace_biomes.amount "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount 222+8]
1.26.30 replace_biomes.amount 5 => rejected [InvalidValue Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount -]
1.26.30 replace_biomes.amount 0.5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:replace_biomes": {"replacements": [{"dimension": "minecraft:overworld","targets": ["minecraft:plains"],"amount": 0.5,"noise_frequency_scale": 1.0}]}}}} []
1.26.30 replace_biomes.amount -1 => rejected [InvalidValue Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount -]
1.26.30 replace_biomes.amount true => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount 222+8]
1.26.30 replace_biomes.amount null => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount 222+8]
1.26.30 replace_biomes.amount 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount 222+8]
1.26.30 replace_biomes.amount 18446744073709551616 => rejected [InvalidValue Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount -]
1.26.30 replace_biomes.amount [] => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount/0 233+1]
1.26.30 replace_biomes.amount {} => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount 233+1]
1.26.30 replace_biomes.noise_frequency_scale "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/noise_frequency_scale 237+23]
1.26.30 replace_biomes.noise_frequency_scale "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/noise_frequency_scale 237+23]
1.26.30 replace_biomes.noise_frequency_scale 5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:replace_biomes": {"replacements": [{"dimension": "minecraft:overworld","targets": ["minecraft:plains"],"amount": 0.5,"noise_frequency_scale": 5.0}]}}}} []
1.26.30 replace_biomes.noise_frequency_scale 0.5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:replace_biomes": {"replacements": [{"dimension": "minecraft:overworld","targets": ["minecraft:plains"],"amount": 0.5,"noise_frequency_scale": 0.5}]}}}} []
1.26.30 replace_biomes.noise_frequency_scale -1 => rejected [InvalidValue Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/noise_frequency_scale -]
1.26.30 replace_biomes.noise_frequency_scale true => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/noise_frequency_scale 237+23]
1.26.30 replace_biomes.noise_frequency_scale null => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/noise_frequency_scale 237+23]
1.26.30 replace_biomes.noise_frequency_scale 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/noise_frequency_scale 237+23]
1.26.30 replace_biomes.noise_frequency_scale 18446744073709551616 => rejected [InvalidValue Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/noise_frequency_scale -]
1.26.30 replace_biomes.noise_frequency_scale [] => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/noise_frequency_scale/0 263+1]
1.26.30 replace_biomes.noise_frequency_scale {} => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/noise_frequency_scale 263+1]
1.26.30 tags.tags[0] "x" => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:tags": {"tags": ["x"]}}}} []
1.26.30 tags.tags[0] "5" => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:tags": {"tags": ["5"]}}}} []
1.26.30 tags.tags[0] 5 => rejected [InvalidType Error minecraft:biome/components/minecraft:tags/tags/0 134+1]
1.26.30 tags.tags[0] 0.5 => rejected [InvalidType Error minecraft:biome/components/minecraft:tags/tags/0 134+3]
1.26.30 tags.tags[0] -1 => rejected [InvalidType Error minecraft:biome/components/minecraft:tags/tags/0 134+2]
1.26.30 tags.tags[0] true => rejected [InvalidType Error minecraft:biome/components/minecraft:tags/tags/0 134+4]
1.26.30 tags.tags[0] null => rejected [InvalidType Error minecraft:biome/components/minecraft:tags/tags/0 134+4]
1.26.30 tags.tags[0] 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:tags/tags/0 134+4]
1.26.30 tags.tags[0] 18446744073709551616 => rejected [InvalidType Error minecraft:biome/components/minecraft:tags/tags/0 134+20]
1.26.30 tags.tags[0] [] => rejected [InvalidType Error minecraft:biome/components/minecraft:tags/tags/0/0 135+1]
1.26.30 tags.tags[0] {} => rejected [InvalidType Error minecraft:biome/components/minecraft:tags/tags/0 135+1]
1.26.30 surface_builder.overworld.sea_floor_depth "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/sea_floor_depth 179+17]
1.26.30 surface_builder.overworld.sea_floor_depth "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/sea_floor_depth 179+17]
1.26.30 surface_builder.overworld.sea_floor_depth 5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:overworld","sea_floor_depth": 5,"sea_floor_material": "minecraft:stone","foundation_material": "minecraft:stone","mid_material": "minecraft:stone","top_material": "minecraft:stone","sea_material": "minecraft:stone"}}}}} []
1.26.30 surface_builder.overworld.sea_floor_depth 0.5 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/sea_floor_depth 179+17]
1.26.30 surface_builder.overworld.sea_floor_depth -1 => rejected [InvalidValue Error minecraft:biome/components/minecraft:surface_builder/builder/sea_floor_depth -]
1.26.30 surface_builder.overworld.sea_floor_depth true => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/sea_floor_depth 179+17]
1.26.30 surface_builder.overworld.sea_floor_depth null => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/sea_floor_depth 179+17]
1.26.30 surface_builder.overworld.sea_floor_depth 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/sea_floor_depth 179+17]
1.26.30 surface_builder.overworld.sea_floor_depth 18446744073709551616 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/sea_floor_depth 179+17]
1.26.30 surface_builder.overworld.sea_floor_depth [] => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/sea_floor_depth/0 199+1]
1.26.30 surface_builder.overworld.sea_floor_depth {} => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/sea_floor_depth 199+1]
1.26.30 surface_builder.swamp.max_puddle "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/max_puddle_depth_below_sea_level 385+34]
1.26.30 surface_builder.swamp.max_puddle "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/max_puddle_depth_below_sea_level 385+34]
1.26.30 surface_builder.swamp.max_puddle 5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:swamp","sea_floor_depth": 7,"sea_floor_material": "minecraft:stone","foundation_material": "minecraft:stone","mid_material": "minecraft:stone","top_material": "minecraft:stone","sea_material": "minecraft:stone","max_puddle_depth_below_sea_level": 5}}}}} []
1.26.30 surface_builder.swamp.max_puddle 0.5 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/max_puddle_depth_below_sea_level 385+34]
1.26.30 surface_builder.swamp.max_puddle -1 => rejected [InvalidValue Error minecraft:biome/components/minecraft:surface_builder/builder/max_puddle_depth_below_sea_level -]
1.26.30 surface_builder.swamp.max_puddle true => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/max_puddle_depth_below_sea_level 385+34]
1.26.30 surface_builder.swamp.max_puddle null => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/max_puddle_depth_below_sea_level 385+34]
1.26.30 surface_builder.swamp.max_puddle 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/max_puddle_depth_below_sea_level 385+34]
1.26.30 surface_builder.swamp.max_puddle 18446744073709551616 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/max_puddle_depth_below_sea_level 385+34]
1.26.30 surface_builder.swamp.max_puddle [] => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/max_puddle_depth_below_sea_level/0 422+1]
1.26.30 surface_builder.swamp.max_puddle {} => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/max_puddle_depth_below_sea_level 422+1]
1.26.30 surface_builder.mesa.bryce_pillars "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/bryce_pillars 461+15]
1.26.30 surface_builder.mesa.bryce_pillars "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/bryce_pillars 461+15]
1.26.30 surface_builder.mesa.bryce_pillars 5 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/bryce_pillars 461+15]
1.26.30 surface_builder.mesa.bryce_pillars 0.5 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/bryce_pillars 461+15]
1.26.30 surface_builder.mesa.bryce_pillars -1 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/bryce_pillars 461+15]
1.26.30 surface_builder.mesa.bryce_pillars true => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:mesa","sea_floor_depth": 7,"sea_floor_material": "minecraft:stone","foundation_material": "minecraft:stone","mid_material": "minecraft:stone","top_material": "minecraft:stone","sea_material": "minecraft:stone","clay_material": "minecraft:stone","hard_clay_material": "minecraft:stone","bryce_pillars": true,"has_forest": false}}}}} []
1.26.30 surface_builder.mesa.bryce_pillars null => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/bryce_pillars 461+15]
1.26.30 surface_builder.mesa.bryce_pillars 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/bryce_pillars 461+15]
1.26.30 surface_builder.mesa.bryce_pillars 18446744073709551616 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/bryce_pillars 461+15]
1.26.30 surface_builder.mesa.bryce_pillars [] => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/bryce_pillars/0 479+1]
1.26.30 surface_builder.mesa.bryce_pillars {} => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/bryce_pillars 479+1]
1.26.30 surface_builder.noise_gradient.name "x" => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:noise_gradient","noise_descriptor": {"name": "x","first_octave": -4,"amplitudes": [1.0]},"noise_block_specifiers": [{"range": {"min": 0.5,"max": 0.7},"block": "minecraft:calcite"}]}}}}} []
1.26.30 surface_builder.noise_gradient.name "5" => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:noise_gradient","noise_descriptor": {"name": "5","first_octave": -4,"amplitudes": [1.0]},"noise_block_specifiers": [{"range": {"min": 0.5,"max": 0.7},"block": "minecraft:calcite"}]}}}}} []
1.26.30 surface_builder.noise_gradient.name 5 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/name 205+6]
1.26.30 surface_builder.noise_gradient.name 0.5 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/name 205+6]
1.26.30 surface_builder.noise_gradient.name -1 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/name 205+6]
1.26.30 surface_builder.noise_gradient.name true => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/name 205+6]
1.26.30 surface_builder.noise_gradient.name null => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/name 205+6]
1.26.30 surface_builder.noise_gradient.name 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/name 205+6]
1.26.30 surface_builder.noise_gradient.name 18446744073709551616 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/name 205+6]
1.26.30 surface_builder.noise_gradient.name [] => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/name/0 214+1]
1.26.30 surface_builder.noise_gradient.name {} => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/name 214+1]
1.26.30 surface_builder.noise_gradient.first_octave "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/first_octave 230+14]
1.26.30 surface_builder.noise_gradient.first_octave "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/first_octave 230+14]
1.26.30 surface_builder.noise_gradient.first_octave 5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:noise_gradient","noise_descriptor": {"name": "test:gradient","first_octave": 5,"amplitudes": [1.0]},"noise_block_specifiers": [{"range": {"min": 0.5,"max": 0.7},"block": "minecraft:calcite"}]}}}}} []
1.26.30 surface_builder.noise_gradient.first_octave 0.5 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/first_octave 230+14]
1.26.30 surface_builder.noise_gradient.first_octave -1 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:noise_gradient","noise_descriptor": {"name": "test:gradient","first_octave": -1,"amplitudes": [1.0]},"noise_block_specifiers": [{"range": {"min": 0.5,"max": 0.7},"block": "minecraft:calcite"}]}}}}} []
1.26.30 surface_builder.noise_gradient.first_octave true => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/first_octave 230+14]
1.26.30 surface_builder.noise_gradient.first_octave null => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/first_octave 230+14]
1.26.30 surface_builder.noise_gradient.first_octave 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/first_octave 230+14]
1.26.30 surface_builder.noise_gradient.first_octave 18446744073709551616 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/first_octave 230+14]
1.26.30 surface_builder.noise_gradient.first_octave [] => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/first_octave/0 247+1]
1.26.30 surface_builder.noise_gradient.first_octave {} => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/first_octave 247+1]
1.26.30 surface_builder.noise_gradient.amplitudes[0] "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/amplitudes/0 265+3]
1.26.30 surface_builder.noise_gradient.amplitudes[0] "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/amplitudes/0 265+3]
1.26.30 surface_builder.noise_gradient.amplitudes[0] 5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:noise_gradient","noise_descriptor": {"name": "test:gradient","first_octave": -4,"amplitudes": [5.0]},"noise_block_specifiers": [{"range": {"min": 0.5,"max": 0.7},"block": "minecraft:calcite"}]}}}}} []
1.26.30 surface_builder.noise_gradient.amplitudes[0] 0.5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:noise_gradient","noise_descriptor": {"name": "test:gradient","first_octave": -4,"amplitudes": [0.5]},"noise_block_specifiers": [{"range": {"min": 0.5,"max": 0.7},"block": "minecraft:calcite"}]}}}}} []
1.26.30 surface_builder.noise_gradient.amplitudes[0] -1 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:noise_gradient","noise_descriptor": {"name": "test:gradient","first_octave": -4,"amplitudes": [-1.0]},"noise_block_specifiers": [{"range": {"min": 0.5,"max": 0.7},"block": "minecraft:calcite"}]}}}}} []
1.26.30 surface_builder.noise_gradient.amplitudes[0] true => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/amplitudes/0 265+4]
1.26.30 surface_builder.noise_gradient.amplitudes[0] null => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/amplitudes/0 265+4]
1.26.30 surface_builder.noise_gradient.amplitudes[0] 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/amplitudes/0 265+4]
1.26.30 surface_builder.noise_gradient.amplitudes[0] 18446744073709551616 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:noise_gradient","noise_descriptor": {"name": "test:gradient","first_octave": -4,"amplitudes": [1.8446744e19]},"noise_block_specifiers": [{"range": {"min": 0.5,"max": 0.7},"block": "minecraft:calcite"}]}}}}} []
1.26.30 surface_builder.noise_gradient.amplitudes[0] [] => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/amplitudes/0/0 266+1]
1.26.30 surface_builder.noise_gradient.amplitudes[0] {} => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_descriptor/amplitudes/0 266+1]
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.noise "x" => rejected [NamespaceRule Error minecraft:biome/components/minecraft:surface_builder/builder/noise_block_specifiers/0/noise -]
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.noise "5" => rejected [NamespaceRule Error minecraft:biome/components/minecraft:surface_builder/builder/noise_block_specifiers/0/noise -]
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.noise 5 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_block_specifiers/0/noise 365+7]
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.noise 0.5 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_block_specifiers/0/noise 365+7]
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.noise -1 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_block_specifiers/0/noise 365+7]
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.noise true => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_block_specifiers/0/noise 365+7]
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.noise null => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_block_specifiers/0/noise 374+4]
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.noise 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_block_specifiers/0/noise 365+7]
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.noise 18446744073709551616 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_block_specifiers/0/noise 365+7]
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.noise [] => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_block_specifiers/0/noise/0 375+1]
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.noise {} => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_block_specifiers/0/noise 375+1]
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.threshold "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_block_specifiers/0/threshold 365+11]
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.threshold "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_block_specifiers/0/threshold 365+11]
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.threshold 5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:noise_gradient","noise_descriptor": {"name": "test:gradient","first_octave": -4,"amplitudes": [1.0]},"noise_block_specifiers": [{"threshold": 5.0,"range": {"min": 0.5,"max": 0.7},"block": "minecraft:calcite"}]}}}}} []
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.threshold 0.5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:noise_gradient","noise_descriptor": {"name": "test:gradient","first_octave": -4,"amplitudes": [1.0]},"noise_block_specifiers": [{"threshold": 0.5,"range": {"min": 0.5,"max": 0.7},"block": "minecraft:calcite"}]}}}}} []
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.threshold -1 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:noise_gradient","noise_descriptor": {"name": "test:gradient","first_octave": -4,"amplitudes": [1.0]},"noise_block_specifiers": [{"threshold": -1.0,"range": {"min": 0.5,"max": 0.7},"block": "minecraft:calcite"}]}}}}} []
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.threshold true => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_block_specifiers/0/threshold 365+11]
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.threshold null => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_block_specifiers/0/threshold 378+4]
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.threshold 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_block_specifiers/0/threshold 365+11]
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.threshold 18446744073709551616 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:noise_gradient","noise_descriptor": {"name": "test:gradient","first_octave": -4,"amplitudes": [1.0]},"noise_block_specifiers": [{"threshold": 1.8446744e19,"range": {"min": 0.5,"max": 0.7},"block": "minecraft:calcite"}]}}}}} []
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.threshold [] => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_block_specifiers/0/threshold/0 379+1]
1.26.30 surface_builder.noise_gradient.noise_block_specifiers.threshold {} => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/noise_block_specifiers/0/threshold 379+1]
1.26.30 surface_material_adjustments.noise_range[0] "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0/noise_range/0 231+3]
1.26.30 surface_material_adjustments.noise_range[0] "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0/noise_range/0 231+3]
1.26.30 surface_material_adjustments.noise_range[0] 5 => rejected [InvalidValue Error minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0/noise_range -]
1.26.30 surface_material_adjustments.noise_range[0] 0.5 => rejected [InvalidValue Error minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0/noise_range -]
1.26.30 surface_material_adjustments.noise_range[0] -1 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_material_adjustments": {"adjustments": [{"materials": {"top_material": "minecraft:sand"},"noise_range": [-1.0,0.5]}]}}}} []
1.26.30 surface_material_adjustments.noise_range[0] true => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0/noise_range/0 231+4]
1.26.30 surface_material_adjustments.noise_range[0] null => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0/noise_range/0 231+4]
1.26.30 surface_material_adjustments.noise_range[0] 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0/noise_range/0 231+4]
1.26.30 surface_material_adjustments.noise_range[0] 18446744073709551616 => rejected [InvalidValue Error minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0/noise_range -]
1.26.30 surface_material_adjustments.noise_range[0] [] => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0/noise_range/0/0 232+1]
1.26.30 surface_material_adjustments.noise_range[0] {} => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0/noise_range/0 232+1]
1.26.30 surface_material_adjustments.noise_frequency_scale "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0/noise_frequency_scale 215+23]
1.26.30 surface_material_adjustments.noise_frequency_scale "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0/noise_frequency_scale 215+23]
1.26.30 surface_material_adjustments.noise_frequency_scale 5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_material_adjustments": {"adjustments": [{"materials": {"top_material": "minecraft:sand"},"noise_frequency_scale": 5.0}]}}}} []
1.26.30 surface_material_adjustments.noise_frequency_scale 0.5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_material_adjustments": {"adjustments": [{"materials": {"top_material": "minecraft:sand"},"noise_frequency_scale": 0.5}]}}}} []
1.26.30 surface_material_adjustments.noise_frequency_scale -1 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_material_adjustments": {"adjustments": [{"materials": {"top_material": "minecraft:sand"},"noise_frequency_scale": -1.0}]}}}} []
1.26.30 surface_material_adjustments.noise_frequency_scale true => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0/noise_frequency_scale 215+23]
1.26.30 surface_material_adjustments.noise_frequency_scale null => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0/noise_frequency_scale 240+4]
1.26.30 surface_material_adjustments.noise_frequency_scale 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0/noise_frequency_scale 215+23]
1.26.30 surface_material_adjustments.noise_frequency_scale 18446744073709551616 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_material_adjustments": {"adjustments": [{"materials": {"top_material": "minecraft:sand"},"noise_frequency_scale": 1.8446744e19}]}}}} []
1.26.30 surface_material_adjustments.noise_frequency_scale [] => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0/noise_frequency_scale/0 241+1]
1.26.30 surface_material_adjustments.noise_frequency_scale {} => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0/noise_frequency_scale 241+1]
1.21.110 climate.temperature "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/temperature 129+13; MalformedInput Error  -]
1.21.110 climate.temperature "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/temperature 129+13; MalformedInput Error  -]
1.21.110 climate.temperature 5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:climate": {"temperature": 5.0}}}} []
1.21.110 climate.temperature 0.5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:climate": {"temperature": 0.5}}}} []
1.21.110 climate.temperature -1 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:climate": {"temperature": -1.0}}}} []
1.21.110 climate.temperature true => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/temperature 129+13; MalformedInput Error  -]
1.21.110 climate.temperature null => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/temperature 144+4; MalformedInput Error  -]
1.21.110 climate.temperature 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/temperature 129+13; MalformedInput Error  -]
1.21.110 climate.temperature 18446744073709551616 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:climate": {"temperature": 1.8446744e19}}}} []
1.21.110 climate.temperature [] => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/temperature/0 145+1; MalformedInput Error  -]
1.21.110 climate.temperature {} => rejected [InvalidType Error minecraft:biome/components/minecraft:climate/temperature 145+1; MalformedInput Error  -]
1.21.110 humidity.is_humid "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:humidity/is_humid 130+10; MalformedInput Error  -]
1.21.110 humidity.is_humid "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:humidity/is_humid 130+10; MalformedInput Error  -]
1.21.110 humidity.is_humid 5 => rejected [InvalidType Error minecraft:biome/components/minecraft:humidity/is_humid 130+10; MalformedInput Error  -]
1.21.110 humidity.is_humid 0.5 => rejected [InvalidType Error minecraft:biome/components/minecraft:humidity/is_humid 130+10; MalformedInput Error  -]
1.21.110 humidity.is_humid -1 => rejected [InvalidType Error minecraft:biome/components/minecraft:humidity/is_humid 130+10; MalformedInput Error  -]
1.21.110 humidity.is_humid true => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:humidity": {"is_humid": true}}}} []
1.21.110 humidity.is_humid null => rejected [InvalidType Error minecraft:biome/components/minecraft:humidity/is_humid 130+10; MalformedInput Error  -]
1.21.110 humidity.is_humid 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:humidity/is_humid 130+10; MalformedInput Error  -]
1.21.110 humidity.is_humid 18446744073709551616 => rejected [InvalidType Error minecraft:biome/components/minecraft:humidity/is_humid 130+10; MalformedInput Error  -]
1.21.110 humidity.is_humid [] => rejected [InvalidType Error minecraft:biome/components/minecraft:humidity/is_humid/0 143+1; MalformedInput Error  -]
1.21.110 humidity.is_humid {} => rejected [InvalidType Error minecraft:biome/components/minecraft:humidity/is_humid 143+1; MalformedInput Error  -]
1.21.110 replace_biomes.amount "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount 223+8; MalformedInput Error  -]
1.21.110 replace_biomes.amount "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount 223+8; MalformedInput Error  -]
1.21.110 replace_biomes.amount 5 => rejected [InvalidValue Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount -]
1.21.110 replace_biomes.amount 0.5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:replace_biomes": {"replacements": [{"dimension": "minecraft:overworld","targets": ["minecraft:plains"],"amount": 0.5,"noise_frequency_scale": 1.0}]}}}} []
1.21.110 replace_biomes.amount -1 => rejected [InvalidValue Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount -]
1.21.110 replace_biomes.amount true => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount 223+8; MalformedInput Error  -]
1.21.110 replace_biomes.amount null => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount 223+8; MalformedInput Error  -]
1.21.110 replace_biomes.amount 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount 223+8; MalformedInput Error  -]
1.21.110 replace_biomes.amount 18446744073709551616 => rejected [InvalidValue Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount -]
1.21.110 replace_biomes.amount [] => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount/0 234+1; MalformedInput Error  -]
1.21.110 replace_biomes.amount {} => rejected [InvalidType Error minecraft:biome/components/minecraft:replace_biomes/replacements/0/amount 234+1; MalformedInput Error  -]
1.21.110 surface_builder.overworld.sea_floor_depth "x" => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/sea_floor_depth 180+17; MalformedInput Error  -]
1.21.110 surface_builder.overworld.sea_floor_depth "5" => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/sea_floor_depth 180+17; MalformedInput Error  -]
1.21.110 surface_builder.overworld.sea_floor_depth 5 => loaded {"format_version": "1.26.30","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:surface_builder": {"builder": {"type": "minecraft:overworld","sea_floor_depth": 5,"sea_floor_material": "minecraft:stone","foundation_material": "minecraft:stone","mid_material": "minecraft:stone","top_material": "minecraft:stone","sea_material": "minecraft:stone"}}}}} []
1.21.110 surface_builder.overworld.sea_floor_depth 0.5 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/sea_floor_depth 180+17; MalformedInput Error  -]
1.21.110 surface_builder.overworld.sea_floor_depth -1 => rejected [InvalidValue Error minecraft:biome/components/minecraft:surface_builder/builder/sea_floor_depth -]
1.21.110 surface_builder.overworld.sea_floor_depth true => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/sea_floor_depth 180+17; MalformedInput Error  -]
1.21.110 surface_builder.overworld.sea_floor_depth null => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/sea_floor_depth 180+17; MalformedInput Error  -]
1.21.110 surface_builder.overworld.sea_floor_depth 1e39 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/sea_floor_depth 180+17; MalformedInput Error  -]
1.21.110 surface_builder.overworld.sea_floor_depth 18446744073709551616 => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/sea_floor_depth 180+17; MalformedInput Error  -]
1.21.110 surface_builder.overworld.sea_floor_depth [] => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/sea_floor_depth/0 200+1; MalformedInput Error  -]
1.21.110 surface_builder.overworld.sea_floor_depth {} => rejected [InvalidType Error minecraft:biome/components/minecraft:surface_builder/builder/sea_floor_depth 200+1; MalformedInput Error  -]
beta bool_attr.value "x" => rejected [InvalidType Error minecraft:biome/environment_attributes/minecraft:experimental_bool_attr/value 170+7]
beta bool_attr.value "5" => rejected [InvalidType Error minecraft:biome/environment_attributes/minecraft:experimental_bool_attr/value 170+7]
beta bool_attr.value 5 => rejected [InvalidType Error minecraft:biome/environment_attributes/minecraft:experimental_bool_attr/value 170+7]
beta bool_attr.value 0.5 => rejected [InvalidType Error minecraft:biome/environment_attributes/minecraft:experimental_bool_attr/value 170+7]
beta bool_attr.value -1 => rejected [InvalidType Error minecraft:biome/environment_attributes/minecraft:experimental_bool_attr/value 170+7]
beta bool_attr.value true => loaded {"format_version": "beta","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {},"environment_attributes": {"minecraft:experimental_bool_attr": {"value": true,"operation": "override"}}}} []
beta bool_attr.value null => rejected [InvalidType Error minecraft:biome/environment_attributes/minecraft:experimental_bool_attr/value 170+7]
beta bool_attr.value 1e39 => rejected [InvalidType Error minecraft:biome/environment_attributes/minecraft:experimental_bool_attr/value 170+7]
beta bool_attr.value 18446744073709551616 => rejected [InvalidType Error minecraft:biome/environment_attributes/minecraft:experimental_bool_attr/value 170+7]
beta bool_attr.value [] => rejected [InvalidType Error minecraft:biome/environment_attributes/minecraft:experimental_bool_attr/value/0 180+1]
beta bool_attr.value {} => rejected [InvalidType Error minecraft:biome/environment_attributes/minecraft:experimental_bool_attr/value 180+1]
beta float_attr.value "x" => rejected [InvalidType Error minecraft:biome/environment_attributes/minecraft:experimental_float_attr_1/value 173+7]
beta float_attr.value "5" => rejected [InvalidType Error minecraft:biome/environment_attributes/minecraft:experimental_float_attr_1/value 173+7]
beta float_attr.value 5 => loaded {"format_version": "beta","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {},"environment_attributes": {"minecraft:experimental_float_attr_1": {"operation": "override","value": 5.0}}}} []
beta float_attr.value 0.5 => loaded {"format_version": "beta","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {},"environment_attributes": {"minecraft:experimental_float_attr_1": {"operation": "override","value": 0.5}}}} []
beta float_attr.value -1 => loaded {"format_version": "beta","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {},"environment_attributes": {"minecraft:experimental_float_attr_1": {"operation": "override","value": -1.0}}}} []
beta float_attr.value true => rejected [InvalidType Error minecraft:biome/environment_attributes/minecraft:experimental_float_attr_1/value 173+7]
beta float_attr.value null => rejected [InvalidType Error minecraft:biome/environment_attributes/minecraft:experimental_float_attr_1/value 173+7]
beta float_attr.value 1e39 => rejected [InvalidType Error minecraft:biome/environment_attributes/minecraft:experimental_float_attr_1/value 173+7]
beta float_attr.value 18446744073709551616 => loaded {"format_version": "beta","minecraft:biome": {"description": {"identifier": "demo:a"},"components": {},"environment_attributes": {"minecraft:experimental_float_attr_1": {"operation": "override","value": 1.8446744e19}}}} []
beta float_attr.value [] => rejected [InvalidType Error minecraft:biome/environment_attributes/minecraft:experimental_float_attr_1/value/0 183+1]
beta float_attr.value {} => rejected [InvalidType Error minecraft:biome/environment_attributes/minecraft:experimental_float_attr_1/value 183+1]
"#;

/// Documents with a struct member written `null`: what the slice's null rule makes of it,
/// told once (rejected where the input rejects nulls, read as its defaults elsewhere).
const NULL_STRUCTS: &[&str] = &[
    r#"{"format_version": "1.26.30", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:climate": null}}}"#,
    r#"{"format_version": "1.21.110", "minecraft:biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:surface_material_adjustments": {"adjustments": [null]}}}}"#,
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
const NULL_STRUCTS_EXPECTED: &str = r"rejected [InvalidType Error minecraft:biome/components/minecraft:climate 127+4]
rejected [InvalidType Error minecraft:biome/components/minecraft:surface_material_adjustments/adjustments/0 166+4; MalformedInput Error  -]
";
