//! The input the game reads the four JSON documents from (the
//! game builds a document input for each, read by its strict JSON reader): the version as it
//! picks it, and rule N2, a member
//! written as `null` is a type error that rejects the document.

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::families::jigsaw::{JigsawStructure, ProcessorList, StructureSet, TemplatePool};
use bedrock_addon::version::FormatVersion;

use super::common::{loads, rejected};

/// A structure payload with `extra` members added after the required ones.
fn structure(extra: &str) -> String {
    format!(
        r#"{{"description": {{"identifier": "demo:a"}}, "step": "lakes", "start_pool": "demo:p", "max_depth": 1, "start_height": {{"type": "constant", "value": {{"absolute": 0}}}}{extra}}}"#
    )
}

/// The errors of `d` as `(code, path)`.
fn errors(d: &Diagnostics) -> Vec<(&'static str, String)> {
    d.iter()
        .filter(|f| f.severity == Severity::Error && f.code.name() != "DocumentRejected")
        .map(|f| (f.code.name(), f.location.json_path.to_string()))
        .collect()
}

#[test]
fn a_number_format_version_is_read_as_its_text() {
    let text = format!(
        r#"{{"format_version": 1.22, "minecraft:jigsaw": {}}}"#,
        structure("")
    );
    let loaded = loads::<JigsawStructure>("worldgen/structures/a.json", &text);
    assert_eq!(loaded.version().written, Some(FormatVersion::new(1, 22, 0)));
    // `1` is 1.0.0, below the one slice: no parser, not malformed.
    let text = format!(
        r#"{{"format_version": 1, "minecraft:jigsaw": {}}}"#,
        structure("")
    );
    let d = rejected::<JigsawStructure>("worldgen/structures/a.json", &text);
    assert_eq!(
        errors(&d),
        [("UnsupportedVersion", "format_version".to_owned())]
    );
}

#[test]
fn a_null_optional_member_of_a_structure_rejects_it() {
    for member in [
        "start_jigsaw_name",
        "terrain_adaptation",
        "heightmap_projection",
        "pool_aliases",
        "liquid_settings",
        "biome_filters",
    ] {
        let text = format!(
            r#"{{"format_version": "1.21.20", "minecraft:jigsaw": {}}}"#,
            structure(&format!(r#", "{member}": null"#))
        );
        let d = rejected::<JigsawStructure>("worldgen/structures/a.json", &text);
        assert_eq!(
            errors(&d),
            [("InvalidType", format!("minecraft:jigsaw/{member}"))],
            "{member}"
        );
    }
}

#[test]
fn a_null_member_of_a_pool_a_set_and_a_processor_list_rejects_each() {
    let pool = r#"{"format_version": "1.21.20", "minecraft:template_pool": {"description": {"identifier": "demo:p"}, "elements": [], "fallback": null}}"#;
    let d = rejected::<TemplatePool>("worldgen/template_pools/p.json", pool);
    assert_eq!(
        errors(&d),
        [("InvalidType", "minecraft:template_pool/fallback".to_owned())]
    );
    let set = r#"{"format_version": "1.21.20", "minecraft:structure_set": {"description": {"identifier": "demo:s"}, "placement": {"type": "minecraft:random_spread", "salt": 1, "separation": 4, "spacing": 12, "spread_type": null}, "structures": [{"structure": "demo:a", "weight": 1}]}}"#;
    let d = rejected::<StructureSet>("worldgen/structure_sets/s.json", set);
    assert_eq!(
        errors(&d),
        [(
            "InvalidType",
            "minecraft:structure_set/placement/spread_type".to_owned()
        )]
    );
    let list = r#"{"format_version": "1.21.20", "minecraft:processor_list": {"description": {"identifier": "demo:l"}, "processors": null}}"#;
    let d = rejected::<ProcessorList>("worldgen/processors/l.json", list);
    assert_eq!(
        errors(&d),
        [(
            "InvalidType",
            "minecraft:processor_list/processors".to_owned()
        )]
    );
}
