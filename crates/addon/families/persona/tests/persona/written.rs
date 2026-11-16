//! The written text of the piece files, pinned in `persona/written.pins`: what `to_json`
//! writes for every meta and store-info fixture that loads and for probes of each member
//! (`bedrock_addon_test_support::written_pins`).

use bedrock_addon_family_persona::{PieceMetadata, PieceStoreInfo};
use bedrock_addon_test_support::written_pins::{
    assert_as_pinned, fixtures_below, parse, render, written,
};

const META: &str = "pieces/hat/hat.meta.json";
const STORE: &str = "pieces/hat/hat.store_info.json";

/// Meta files that reach each member: every source list, each tint form, names in and out of
/// their sets, members kept as written, the envelope.
const META_PROBES: &[&str] = &[
    r##"{"piece_id": "a", "piece_name": "n", "piece_type": "persona_skin", "piece_sub_type": "Odd", "body_variant": "small", "clothing_map": "c", "face_clothing_map": "f", "zone": ["z1", "z2"], "texture_sources": [{"texture": "t", "tint_map": "m", "animated": true, "frames": 3, "use_face_uv": false, "odd": 1}], "geometry_sources": [{"geometry": "g", "arm_size": "slim", "body_size": "Weird", "side": "left", "zone": "z", "is_face": "true", "texture": "t", "tint_map": "m", "animated": false, "frames": 1}], "tint_color": {"r_color": "#ff0000", "g_color": "#00ff00", "b_color": "#0000ff", "a_color": "#ffffffff", "x": null}, "tint_base_color": "#123456", "allow_tint_override": true, "extra": [1]}"##,
    r#"{"piece_id": "e", "piece_name": "e", "piece_type": "persona_emote", "animation_sources": [{"name": "a", "animationFile": "f.json"}], "zone": "z", "piece_sub_type": 5, "allow_tint_override": "yes"}"#,
    r#"{"format_version": "1.26.40", "minecraft:persona_piece": {"piece_id": "v", "piece_name": "v", "piece_type": "persona_hat"}}"#,
    r#"{"piece_id": "b", "piece_name": "b", "piece_type": "persona_body", "geometry_sources": [{"geometry": "g", "is_face": true}], "tint_color": {"r_color": 3}}"#,
];

const STORE_PROBES: &[&str] = &[
    r#"{"piece_store_name": "s", "piece_thumbnail": "t.png", "odd": {"a": 1}}"#,
    r#"{"piece_store_name": null, "piece_thumbnail": 4}"#,
    "{}",
];

#[test]
fn the_piece_files_are_written_as_pinned() {
    let mut cases: Vec<(String, String)> = Vec::new();
    let addon = concat!(env!("CARGO_MANIFEST_DIR"), "/../../addon");
    for (name, text) in fixtures_below(addon, "persona_piece", ".json") {
        if name.ends_with(".meta.json") && parse::<PieceMetadata>(META, &text).is_ok() {
            cases.push((name, written::<PieceMetadata>(META, &text)));
        } else if name.ends_with(".store_info.json")
            && parse::<PieceStoreInfo>(STORE, &text).is_ok()
        {
            cases.push((name, written::<PieceStoreInfo>(STORE, &text)));
        }
    }
    for probe in META_PROBES {
        cases.push(((*probe).to_owned(), written::<PieceMetadata>(META, probe)));
    }
    for probe in STORE_PROBES {
        cases.push(((*probe).to_owned(), written::<PieceStoreInfo>(STORE, probe)));
    }
    assert_as_pinned(
        env!("CARGO_MANIFEST_DIR"),
        "persona/written.pins",
        &render(&cases),
    );
}
