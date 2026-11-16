//! The written text of a geometry file, pinned in `geometry/written.pins`: what `to_json`
//! writes for every fixture that loads and for probes of each member of the newest shape and
//! of the two older ones (`bedrock_addon_test_support::written_pins`).

use bedrock_addon_family_geometry::GeometryFile;
use bedrock_addon_test_support::written_pins::{
    assert_as_pinned, fixtures_below, parse, render, written,
};

const PATH: &str = "models/entity/x.json";

/// Files that reach each member: a geometry of every newest member, floats whole, real and
/// signed zero, the legacy side-cars, and the two shapes older than 1.12.0.
const PROBES: &[&str] = &[
    r#"{"format_version": "1.21.0", "debug": true, "minecraft:geometry": [{"description": {"identifier": "geometry.a", "visible_bounds_width": 2, "visible_bounds_height": 1.5, "visible_bounds_offset": [0, 0.25, -0.0], "texture_width": 64, "texture_height": 32}, "cape": "geometry.cape", "item_display_transforms": {"gui": {"translation": [1, 2, 3], "rotation": [0, 90, 0], "scale": [0.5, 0.5, 0.5], "rotation_pivot": [0, 0, 0], "scale_pivot": [1, 1, 1], "fit_to_frame": true}}, "bones": [{"name": "body", "parent": "root", "pivot": [0, 24, 0], "rotation": [0.1, 0, 0], "mirror": true, "inflate": 0.5, "debug": false, "render_group_id": 1, "binding": "q.item_slot_to_bone_name('main_hand')", "cubes": [{"origin": [-4, 12, -2], "size": [8, 12, 4], "rotation": [0, 0, 1], "pivot": [0, 0, 0], "inflate": 0.25, "mirror": false, "uv": [16, 16]}, {"origin": [0, 0, 0], "size": [1, 1, 1], "uv": {"north": {"uv": [0, 0], "uv_size": [1, 1], "uv_rotation": 90, "material_instance": "m"}, "down": {"uv": [1.5, 2]}}}], "locators": {"a": [0, 1, 2], "b": {"offset": [1, 2, 3], "rotation": [0, 0, 0], "ignore_inherited_scale": true}}, "poly_mesh": {"normalized_uvs": true, "positions": [[0, 0, 0]], "normals": [[0, 1, 0]], "uvs": [[0, 1]], "polys": [[[0, 0, 0], [0, 0, 0], [0, 0, 0]]]}, "texture_meshes": [{"texture": "t", "position": [0, 0, 0], "local_pivot": [1, 1, 1], "rotation": [0, 0, 0], "scale": [1, 1, 1], "use_pixel_depth": true}]}]}]}"#,
    r#"{"format_version": "1.12.0", "minecraft:geometry": [{"description": {"identifier": "geometry.b"}, "bones": [{"name": "b", "poly_mesh": {"polys": "quad_list"}}]}]}"#,
    r#"{"format_version": "1.8.0", "geometry.old": {"texturewidth": 64, "textureheight": 64, "visible_bounds_width": 1, "bones": [{"name": "body", "reset": true, "neverRender": false, "pivot": [0, 24, 0], "bind_pose_rotation": [0, 0, 0], "cubes": [{"origin": [0, 0, 0], "size": [1, 2, 3], "uv": [0, 0], "inflate": 0.5}], "locators": {"l": [1, 2, 3]}}]}}"#,
    r#"{"format_version": "1.6.0", "geometry.older:geometry.parent": {"texturewidth": 16, "bones": [{"name": "a", "mirror": "yes", "cubes": [null, {"origin": [0, 0, 0], "size": [1, 1, 1]}]}]}}"#,
];

#[test]
fn the_geometry_file_is_written_as_pinned() {
    let mut cases: Vec<(String, String)> = Vec::new();
    let addon = concat!(env!("CARGO_MANIFEST_DIR"), "/../../addon");
    for (name, text) in fixtures_below(addon, "geometry", ".json") {
        if parse::<GeometryFile>(PATH, &text).is_ok() {
            cases.push((name, written::<GeometryFile>(PATH, &text)));
        }
    }
    for probe in PROBES {
        cases.push(((*probe).to_owned(), written::<GeometryFile>(PATH, probe)));
    }
    assert_as_pinned(
        env!("CARGO_MANIFEST_DIR"),
        "geometry/written.pins",
        &render(&cases),
    );
}
