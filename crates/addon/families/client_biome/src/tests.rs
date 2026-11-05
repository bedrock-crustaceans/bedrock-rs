//! Every scalar member of a client biome, in the newest slice (1.21.130), an older one
//! (1.21.110) and the beta document, read from every JSON type: each line pins what the load says
//! (code, severity, path, span) and, when the document loads, the document written back. The
//! members were strict wrappers; this table is what they read, and the oracle of the change
//! that makes them plain.

use std::fmt::Write as _;

use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackPath, PackType};

use super::{Beta, ClientBiome};

/// A resource pack's context with the Beta APIs toggle on.
fn context() -> LoadContext {
    let mut cx = LoadContext::detached();
    cx.pack_type = PackType::Resource;
    cx.beta_apis = true;
    cx
}

/// What a load gave: the document written back, or nothing, and the findings.
fn outcome<T: bedrock_addon_core::document::WriteWith>(
    read: Result<Document<T>, Rejected>,
) -> (Option<String>, Diagnostics) {
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
    let path = PackPath::from_static("biomes/demo.client_biome.json");
    if text.contains(r#""format_version": "beta""#) {
        outcome(<Beta as Content>::parse(&path, text.as_bytes(), &context()))
    } else {
        outcome(<ClientBiome as Content>::parse(
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
        "1.21.130 grass_appearance.grass_is_shaded",
        r#"{"format_version": "1.21.130", "minecraft:client_biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:grass_appearance": {"grass_is_shaded": @@}}}}"#,
    ),
    (
        "1.21.130 water_appearance.surface_opacity",
        r#"{"format_version": "1.21.130", "minecraft:client_biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:water_appearance": {"surface_opacity": @@}}}}"#,
    ),
    (
        "1.21.130 precipitation.ash",
        r#"{"format_version": "1.21.130", "minecraft:client_biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:precipitation": {"ash": @@}}}}"#,
    ),
    (
        "1.21.130 ambient_sounds.addition.chance",
        r#"{"format_version": "1.21.130", "minecraft:client_biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:ambient_sounds": {"addition": {"asset": "a.b", "chance": @@}}}}}"#,
    ),
    (
        "1.21.130 biome_music.volume_multiplier",
        r#"{"format_version": "1.21.130", "minecraft:client_biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:biome_music": {"volume_multiplier": @@}}}}"#,
    ),
    (
        "1.21.130 biome_music.underwater_music",
        r#"{"format_version": "1.21.130", "minecraft:client_biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:biome_music": {"underwater_music": @@}}}}"#,
    ),
    (
        "1.21.110 water_appearance.surface_opacity",
        r#"{"format_version": "1.21.110", "minecraft:client_biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:water_appearance": {"surface_opacity": @@}}}}"#,
    ),
    (
        "1.21.110 precipitation.ash",
        r#"{"format_version": "1.21.110", "minecraft:client_biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:precipitation": {"ash": @@}}}}"#,
    ),
    (
        "1.21.110 biome_music.volume_multiplier",
        r#"{"format_version": "1.21.110", "minecraft:client_biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:biome_music": {"volume_multiplier": @@}}}}"#,
    ),
    (
        "beta bool_attr.value",
        r#"{"format_version": "beta", "minecraft:client_biome": {"description": {"identifier": "demo:a"}, "components": {}, "environment_attributes": {"minecraft:experimental_bool_attr": {"value": @@, "operation": "override"}}}}"#,
    ),
    (
        "beta float_attr.value",
        r#"{"format_version": "beta", "minecraft:client_biome": {"description": {"identifier": "demo:a"}, "components": {}, "environment_attributes": {"minecraft:experimental_float_attr_1": {"value": @@, "operation": "override"}}}}"#,
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
const EXPECTED: &str = r#"1.21.130 grass_appearance.grass_is_shaded "x" => rejected [InvalidType Error minecraft:client_biome/components/minecraft:grass_appearance/grass_is_shaded 145+17; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 grass_appearance.grass_is_shaded "5" => rejected [InvalidType Error minecraft:client_biome/components/minecraft:grass_appearance/grass_is_shaded 145+17; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 grass_appearance.grass_is_shaded 5 => rejected [InvalidType Error minecraft:client_biome/components/minecraft:grass_appearance/grass_is_shaded 145+17; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 grass_appearance.grass_is_shaded 0.5 => rejected [InvalidType Error minecraft:client_biome/components/minecraft:grass_appearance/grass_is_shaded 145+17; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 grass_appearance.grass_is_shaded -1 => rejected [InvalidType Error minecraft:client_biome/components/minecraft:grass_appearance/grass_is_shaded 145+17; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 grass_appearance.grass_is_shaded true => loaded {"format_version": "1.21.130","minecraft:client_biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:grass_appearance": {"grass_is_shaded": true}}}} []
1.21.130 grass_appearance.grass_is_shaded null => rejected [InvalidType Error minecraft:client_biome/components/minecraft:grass_appearance/grass_is_shaded 164+4; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 grass_appearance.grass_is_shaded 1e39 => rejected [InvalidType Error minecraft:client_biome/components/minecraft:grass_appearance/grass_is_shaded 145+17; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 grass_appearance.grass_is_shaded 18446744073709551616 => rejected [InvalidType Error minecraft:client_biome/components/minecraft:grass_appearance/grass_is_shaded 145+17; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 grass_appearance.grass_is_shaded [] => rejected [InvalidType Error minecraft:client_biome/components/minecraft:grass_appearance/grass_is_shaded/0 165+1; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 grass_appearance.grass_is_shaded {} => rejected [InvalidType Error minecraft:client_biome/components/minecraft:grass_appearance/grass_is_shaded 165+1; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 water_appearance.surface_opacity "x" => rejected [InvalidType Error minecraft:client_biome/components/minecraft:water_appearance/surface_opacity 145+17; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 water_appearance.surface_opacity "5" => rejected [InvalidType Error minecraft:client_biome/components/minecraft:water_appearance/surface_opacity 145+17; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 water_appearance.surface_opacity 5 => rejected [InvalidValue Error minecraft:client_biome/components/minecraft:water_appearance/surface_opacity -]
1.21.130 water_appearance.surface_opacity 0.5 => loaded {"format_version": "1.21.130","minecraft:client_biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:water_appearance": {"surface_opacity": 0.5}}}} []
1.21.130 water_appearance.surface_opacity -1 => rejected [InvalidValue Error minecraft:client_biome/components/minecraft:water_appearance/surface_opacity -]
1.21.130 water_appearance.surface_opacity true => rejected [InvalidType Error minecraft:client_biome/components/minecraft:water_appearance/surface_opacity 145+17; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 water_appearance.surface_opacity null => rejected [InvalidType Error minecraft:client_biome/components/minecraft:water_appearance/surface_opacity 164+4; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 water_appearance.surface_opacity 1e39 => rejected [InvalidType Error minecraft:client_biome/components/minecraft:water_appearance/surface_opacity 145+17; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 water_appearance.surface_opacity 18446744073709551616 => rejected [InvalidValue Error minecraft:client_biome/components/minecraft:water_appearance/surface_opacity -]
1.21.130 water_appearance.surface_opacity [] => rejected [InvalidType Error minecraft:client_biome/components/minecraft:water_appearance/surface_opacity/0 165+1; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 water_appearance.surface_opacity {} => rejected [InvalidType Error minecraft:client_biome/components/minecraft:water_appearance/surface_opacity 165+1; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 precipitation.ash "x" => rejected [InvalidType Error minecraft:client_biome/components/minecraft:precipitation/ash 142+5; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 precipitation.ash "5" => rejected [InvalidType Error minecraft:client_biome/components/minecraft:precipitation/ash 142+5; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 precipitation.ash 5 => loaded {"format_version": "1.21.130","minecraft:client_biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:precipitation": {"ash": 5.0}}}} []
1.21.130 precipitation.ash 0.5 => loaded {"format_version": "1.21.130","minecraft:client_biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:precipitation": {"ash": 0.5}}}} []
1.21.130 precipitation.ash -1 => loaded {"format_version": "1.21.130","minecraft:client_biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:precipitation": {"ash": -1.0}}}} []
1.21.130 precipitation.ash true => rejected [InvalidType Error minecraft:client_biome/components/minecraft:precipitation/ash 142+5; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 precipitation.ash null => rejected [InvalidType Error minecraft:client_biome/components/minecraft:precipitation/ash 149+4; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 precipitation.ash 1e39 => rejected [InvalidType Error minecraft:client_biome/components/minecraft:precipitation/ash 142+5; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 precipitation.ash 18446744073709551616 => loaded {"format_version": "1.21.130","minecraft:client_biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:precipitation": {"ash": 1.8446744e19}}}} []
1.21.130 precipitation.ash [] => rejected [InvalidType Error minecraft:client_biome/components/minecraft:precipitation/ash/0 150+1; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 precipitation.ash {} => rejected [InvalidType Error minecraft:client_biome/components/minecraft:precipitation/ash 150+1; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 ambient_sounds.addition.chance "x" => rejected [InvalidType Error minecraft:client_biome/components/minecraft:ambient_sounds/addition/chance 172+8; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 ambient_sounds.addition.chance "5" => rejected [InvalidType Error minecraft:client_biome/components/minecraft:ambient_sounds/addition/chance 172+8; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 ambient_sounds.addition.chance 5 => rejected [InvalidValue Error minecraft:client_biome/components/minecraft:ambient_sounds/addition/chance -; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 ambient_sounds.addition.chance 0.5 => loaded {"format_version": "1.21.130","minecraft:client_biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:ambient_sounds": {"addition": {"asset": "a.b","chance": 0.5}}}}} []
1.21.130 ambient_sounds.addition.chance -1 => rejected [InvalidValue Error minecraft:client_biome/components/minecraft:ambient_sounds/addition/chance -; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 ambient_sounds.addition.chance true => rejected [InvalidType Error minecraft:client_biome/components/minecraft:ambient_sounds/addition/chance 172+8; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 ambient_sounds.addition.chance null => rejected [InvalidType Error minecraft:client_biome/components/minecraft:ambient_sounds/addition/chance 172+8; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 ambient_sounds.addition.chance 1e39 => rejected [InvalidValue Error minecraft:client_biome/components/minecraft:ambient_sounds/addition/chance -; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 ambient_sounds.addition.chance 18446744073709551616 => rejected [InvalidValue Error minecraft:client_biome/components/minecraft:ambient_sounds/addition/chance -; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 ambient_sounds.addition.chance [] => rejected [InvalidType Error minecraft:client_biome/components/minecraft:ambient_sounds/addition/chance/0 183+1; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 ambient_sounds.addition.chance {} => rejected [InvalidType Error minecraft:client_biome/components/minecraft:ambient_sounds/addition/chance 183+1; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 biome_music.volume_multiplier "x" => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/volume_multiplier 140+19; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 biome_music.volume_multiplier "5" => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/volume_multiplier 140+19; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 biome_music.volume_multiplier 5 => rejected [InvalidValue Error minecraft:client_biome/components/minecraft:biome_music/volume_multiplier -]
1.21.130 biome_music.volume_multiplier 0.5 => loaded {"format_version": "1.21.130","minecraft:client_biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:biome_music": {"volume_multiplier": 0.5}}}} []
1.21.130 biome_music.volume_multiplier -1 => rejected [InvalidValue Error minecraft:client_biome/components/minecraft:biome_music/volume_multiplier -]
1.21.130 biome_music.volume_multiplier true => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/volume_multiplier 140+19; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 biome_music.volume_multiplier null => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/volume_multiplier 161+4; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 biome_music.volume_multiplier 1e39 => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/volume_multiplier 140+19; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 biome_music.volume_multiplier 18446744073709551616 => rejected [InvalidValue Error minecraft:client_biome/components/minecraft:biome_music/volume_multiplier -]
1.21.130 biome_music.volume_multiplier [] => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/volume_multiplier/0 162+1; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 biome_music.volume_multiplier {} => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/volume_multiplier 162+1; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 biome_music.underwater_music "x" => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/underwater_music 140+18; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 biome_music.underwater_music "5" => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/underwater_music 140+18; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 biome_music.underwater_music 5 => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/underwater_music 140+18; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 biome_music.underwater_music 0.5 => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/underwater_music 140+18; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 biome_music.underwater_music -1 => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/underwater_music 140+18; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 biome_music.underwater_music true => loaded {"format_version": "1.21.130","minecraft:client_biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:biome_music": {"underwater_music": true}}}} []
1.21.130 biome_music.underwater_music null => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/underwater_music 160+4; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 biome_music.underwater_music 1e39 => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/underwater_music 140+18; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 biome_music.underwater_music 18446744073709551616 => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/underwater_music 140+18; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 biome_music.underwater_music [] => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/underwater_music/0 161+1; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.130 biome_music.underwater_music {} => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/underwater_music 161+1; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
1.21.110 water_appearance.surface_opacity "x" => rejected [InvalidType Error minecraft:client_biome/components/minecraft:water_appearance/surface_opacity 145+17; MalformedInput Error  -; MalformedInput Error  -]
1.21.110 water_appearance.surface_opacity "5" => rejected [InvalidType Error minecraft:client_biome/components/minecraft:water_appearance/surface_opacity 145+17; MalformedInput Error  -; MalformedInput Error  -]
1.21.110 water_appearance.surface_opacity 5 => rejected [InvalidValue Error minecraft:client_biome/components/minecraft:water_appearance/surface_opacity -]
1.21.110 water_appearance.surface_opacity 0.5 => loaded {"format_version": "1.21.130","minecraft:client_biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:water_appearance": {"surface_opacity": 0.5}}}} []
1.21.110 water_appearance.surface_opacity -1 => rejected [InvalidValue Error minecraft:client_biome/components/minecraft:water_appearance/surface_opacity -]
1.21.110 water_appearance.surface_opacity true => rejected [InvalidType Error minecraft:client_biome/components/minecraft:water_appearance/surface_opacity 145+17; MalformedInput Error  -; MalformedInput Error  -]
1.21.110 water_appearance.surface_opacity null => rejected [InvalidType Error minecraft:client_biome/components/minecraft:water_appearance/surface_opacity 164+4; MalformedInput Error  -; MalformedInput Error  -]
1.21.110 water_appearance.surface_opacity 1e39 => rejected [InvalidType Error minecraft:client_biome/components/minecraft:water_appearance/surface_opacity 145+17; MalformedInput Error  -; MalformedInput Error  -]
1.21.110 water_appearance.surface_opacity 18446744073709551616 => rejected [InvalidValue Error minecraft:client_biome/components/minecraft:water_appearance/surface_opacity -]
1.21.110 water_appearance.surface_opacity [] => rejected [InvalidType Error minecraft:client_biome/components/minecraft:water_appearance/surface_opacity/0 165+1; MalformedInput Error  -; MalformedInput Error  -]
1.21.110 water_appearance.surface_opacity {} => rejected [InvalidType Error minecraft:client_biome/components/minecraft:water_appearance/surface_opacity 165+1; MalformedInput Error  -; MalformedInput Error  -]
1.21.110 precipitation.ash "x" => rejected [InvalidType Error minecraft:client_biome/components/minecraft:precipitation/ash 142+5; MalformedInput Error  -; MalformedInput Error  -]
1.21.110 precipitation.ash "5" => rejected [InvalidType Error minecraft:client_biome/components/minecraft:precipitation/ash 142+5; MalformedInput Error  -; MalformedInput Error  -]
1.21.110 precipitation.ash 5 => loaded {"format_version": "1.21.130","minecraft:client_biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:precipitation": {"ash": 5.0}}}} []
1.21.110 precipitation.ash 0.5 => loaded {"format_version": "1.21.130","minecraft:client_biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:precipitation": {"ash": 0.5}}}} []
1.21.110 precipitation.ash -1 => loaded {"format_version": "1.21.130","minecraft:client_biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:precipitation": {"ash": -1.0}}}} []
1.21.110 precipitation.ash true => rejected [InvalidType Error minecraft:client_biome/components/minecraft:precipitation/ash 142+5; MalformedInput Error  -; MalformedInput Error  -]
1.21.110 precipitation.ash null => rejected [InvalidType Error minecraft:client_biome/components/minecraft:precipitation/ash 149+4; MalformedInput Error  -; MalformedInput Error  -]
1.21.110 precipitation.ash 1e39 => rejected [InvalidType Error minecraft:client_biome/components/minecraft:precipitation/ash 142+5; MalformedInput Error  -; MalformedInput Error  -]
1.21.110 precipitation.ash 18446744073709551616 => loaded {"format_version": "1.21.130","minecraft:client_biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:precipitation": {"ash": 1.8446744e19}}}} []
1.21.110 precipitation.ash [] => rejected [InvalidType Error minecraft:client_biome/components/minecraft:precipitation/ash/0 150+1; MalformedInput Error  -; MalformedInput Error  -]
1.21.110 precipitation.ash {} => rejected [InvalidType Error minecraft:client_biome/components/minecraft:precipitation/ash 150+1; MalformedInput Error  -; MalformedInput Error  -]
1.21.110 biome_music.volume_multiplier "x" => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/volume_multiplier 140+19; MalformedInput Error  -; MalformedInput Error  -]
1.21.110 biome_music.volume_multiplier "5" => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/volume_multiplier 140+19; MalformedInput Error  -; MalformedInput Error  -]
1.21.110 biome_music.volume_multiplier 5 => rejected [InvalidValue Error minecraft:client_biome/components/minecraft:biome_music/volume_multiplier -]
1.21.110 biome_music.volume_multiplier 0.5 => loaded {"format_version": "1.21.130","minecraft:client_biome": {"description": {"identifier": "demo:a"},"components": {"minecraft:biome_music": {"volume_multiplier": 0.5}}}} []
1.21.110 biome_music.volume_multiplier -1 => rejected [InvalidValue Error minecraft:client_biome/components/minecraft:biome_music/volume_multiplier -]
1.21.110 biome_music.volume_multiplier true => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/volume_multiplier 140+19; MalformedInput Error  -; MalformedInput Error  -]
1.21.110 biome_music.volume_multiplier null => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/volume_multiplier 161+4; MalformedInput Error  -; MalformedInput Error  -]
1.21.110 biome_music.volume_multiplier 1e39 => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/volume_multiplier 140+19; MalformedInput Error  -; MalformedInput Error  -]
1.21.110 biome_music.volume_multiplier 18446744073709551616 => rejected [InvalidValue Error minecraft:client_biome/components/minecraft:biome_music/volume_multiplier -]
1.21.110 biome_music.volume_multiplier [] => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/volume_multiplier/0 162+1; MalformedInput Error  -; MalformedInput Error  -]
1.21.110 biome_music.volume_multiplier {} => rejected [InvalidType Error minecraft:client_biome/components/minecraft:biome_music/volume_multiplier 162+1; MalformedInput Error  -; MalformedInput Error  -]
beta bool_attr.value "x" => rejected [InvalidType Error minecraft:client_biome/environment_attributes/minecraft:experimental_bool_attr/value 177+7; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
beta bool_attr.value "5" => rejected [InvalidType Error minecraft:client_biome/environment_attributes/minecraft:experimental_bool_attr/value 177+7; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
beta bool_attr.value 5 => rejected [InvalidType Error minecraft:client_biome/environment_attributes/minecraft:experimental_bool_attr/value 177+7; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
beta bool_attr.value 0.5 => rejected [InvalidType Error minecraft:client_biome/environment_attributes/minecraft:experimental_bool_attr/value 177+7; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
beta bool_attr.value -1 => rejected [InvalidType Error minecraft:client_biome/environment_attributes/minecraft:experimental_bool_attr/value 177+7; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
beta bool_attr.value true => loaded {"format_version": "beta","minecraft:client_biome": {"description": {"identifier": "demo:a"},"components": {},"environment_attributes": {"minecraft:experimental_bool_attr": {"value": true,"operation": "override"}}}} []
beta bool_attr.value null => rejected [InvalidType Error minecraft:client_biome/environment_attributes/minecraft:experimental_bool_attr/value 177+7; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
beta bool_attr.value 1e39 => rejected [InvalidType Error minecraft:client_biome/environment_attributes/minecraft:experimental_bool_attr/value 177+7; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
beta bool_attr.value 18446744073709551616 => rejected [InvalidType Error minecraft:client_biome/environment_attributes/minecraft:experimental_bool_attr/value 177+7; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
beta bool_attr.value [] => rejected [InvalidType Error minecraft:client_biome/environment_attributes/minecraft:experimental_bool_attr/value/0 187+1; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
beta bool_attr.value {} => rejected [InvalidType Error minecraft:client_biome/environment_attributes/minecraft:experimental_bool_attr/value 187+1; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
beta float_attr.value "x" => rejected [InvalidType Error minecraft:client_biome/environment_attributes/minecraft:experimental_float_attr_1/value 180+7; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
beta float_attr.value "5" => rejected [InvalidType Error minecraft:client_biome/environment_attributes/minecraft:experimental_float_attr_1/value 180+7; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
beta float_attr.value 5 => loaded {"format_version": "beta","minecraft:client_biome": {"description": {"identifier": "demo:a"},"components": {},"environment_attributes": {"minecraft:experimental_float_attr_1": {"operation": "override","value": 5.0}}}} []
beta float_attr.value 0.5 => loaded {"format_version": "beta","minecraft:client_biome": {"description": {"identifier": "demo:a"},"components": {},"environment_attributes": {"minecraft:experimental_float_attr_1": {"operation": "override","value": 0.5}}}} []
beta float_attr.value -1 => loaded {"format_version": "beta","minecraft:client_biome": {"description": {"identifier": "demo:a"},"components": {},"environment_attributes": {"minecraft:experimental_float_attr_1": {"operation": "override","value": -1.0}}}} []
beta float_attr.value true => rejected [InvalidType Error minecraft:client_biome/environment_attributes/minecraft:experimental_float_attr_1/value 180+7; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
beta float_attr.value null => rejected [InvalidType Error minecraft:client_biome/environment_attributes/minecraft:experimental_float_attr_1/value 180+7; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
beta float_attr.value 1e39 => rejected [InvalidType Error minecraft:client_biome/environment_attributes/minecraft:experimental_float_attr_1/value 180+7; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
beta float_attr.value 18446744073709551616 => loaded {"format_version": "beta","minecraft:client_biome": {"description": {"identifier": "demo:a"},"components": {},"environment_attributes": {"minecraft:experimental_float_attr_1": {"operation": "override","value": 1.8446744e19}}}} []
beta float_attr.value [] => rejected [InvalidType Error minecraft:client_biome/environment_attributes/minecraft:experimental_float_attr_1/value/0 190+1; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
beta float_attr.value {} => rejected [InvalidType Error minecraft:client_biome/environment_attributes/minecraft:experimental_float_attr_1/value 190+1; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
"#;

/// Documents with a struct member written `null`: what the slice's null rule makes of it,
/// told once (rejected where the input rejects nulls, read as its defaults elsewhere).
const NULL_STRUCTS: &[&str] = &[
    r#"{"format_version": "1.21.130", "minecraft:client_biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:water_appearance": null}}}"#,
    r#"{"format_version": "1.21.110", "minecraft:client_biome": {"description": {"identifier": "demo:a"}, "components": {"minecraft:precipitation": null}}}"#,
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
const NULL_STRUCTS_EXPECTED: &str = r"rejected [InvalidType Error minecraft:client_biome/components/minecraft:water_appearance -; MalformedInput Error minecraft:client_biome -; MalformedInput Error  -]
rejected [InvalidType Error minecraft:client_biome/components/minecraft:precipitation -; MalformedInput Error  -; MalformedInput Error  -]
";
