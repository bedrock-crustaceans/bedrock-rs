//! The written text of a render controller file, pinned in `render_controller/written.pins`:
//! what `to_json` writes for every fixture that loads and for probes of each member
//! (`bedrock_addon_test_support::written_pins`).

use bedrock_addon_family_render_controller::RenderControllerFile;
use bedrock_addon_test_support::written_pins::{
    assert_as_pinned, fixtures_below, parse, render, written,
};

const PATH: &str = "render_controllers/x.json";

/// Files that reach each member: the arrays of each kind, both bone lists (a material that is
/// a number), every colour, `uv_anim`, the flags, Molang in each form, keys out of order.
const PROBES: &[&str] = &[
    r#"{"format_version": "1.8.0", "render_controllers": {"controller.render.z": {"arrays": {"geometries": {"Array.g": ["Geometry.a", "Geometry.b"]}, "materials": {"Array.m": ["Material.default"]}, "textures": {"Array.t": ["Texture.default"]}}, "geometry": "Array.g[q.variant]", "materials": [{"*": "Material.default"}, {"head": 0}], "textures": ["Texture.default", "Array.t[0]"], "part_visibility": [{"*": true}, {"leg*": "q.is_baby"}, {"arm": 1}, {"tail": {"expression": "q.x", "version": "1.20.0"}}], "color": {"r": 1, "g": 0.5, "b": "q.c", "a": 1.0}, "overlay_color": {"r": 0}, "on_fire_color": {"a": 0.25}, "is_hurt_color": {"g": 1}, "uv_anim": {"offset": [0, "q.t"], "scale": [1, 0.5]}, "light_color_multiplier": 1.5, "ignore_lighting": true, "filter_lighting": true, "rebuild_animation_matrices": false}, "controller.render.a": {"geometry": "Geometry.default"}}}"#,
    r#"{"format_version": "1.10.0", "render_controllers": {"controller.render.b": {"geometry": "Geometry.default", "textures": [], "arrays": {"textures": {}}}}}"#,
];

#[test]
fn the_render_controller_file_is_written_as_pinned() {
    let mut cases: Vec<(String, String)> = Vec::new();
    let addon = concat!(env!("CARGO_MANIFEST_DIR"), "/../../addon");
    for (name, text) in fixtures_below(addon, "render/render_controller", ".json") {
        if parse::<RenderControllerFile>(PATH, &text).is_ok() {
            cases.push((name, written::<RenderControllerFile>(PATH, &text)));
        }
    }
    for probe in PROBES {
        cases.push((
            (*probe).to_owned(),
            written::<RenderControllerFile>(PATH, probe),
        ));
    }
    assert_as_pinned(
        env!("CARGO_MANIFEST_DIR"),
        "render_controller/written.pins",
        &render(&cases),
    );
}
