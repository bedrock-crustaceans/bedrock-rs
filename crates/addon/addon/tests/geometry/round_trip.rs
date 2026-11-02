//! Round trips,
//! the default path, the constructors, and the write-back rule that nothing the author did not
//! write appears.

use bedrock_addon::document::{Content, Placed};
use bedrock_addon::families::geometry::{Bone, Cube, Geometry, GeometryFile, TextureMesh};
use bedrock_addon::json::{EncodeError, Unrepresentable, Value};
use bedrock_addon::version::FormatVersion;

use crate::common::{base_game, clean, load, load_at, loaded, round_trip};

fn json(text: &str) -> Value {
    bedrock_addon::json::from_str(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}

/// Every member of the newest schema, once.
const FULL: &str = r#"{
  "format_version": "1.26.40",
  "debug": true,
  "minecraft:geometry": [
    {
      "description": {
        "identifier": "geometry.full",
        "visible_bounds_width": 2,
        "visible_bounds_height": 3.5,
        "visible_bounds_offset": [0, 1.25, 0],
        "texture_width": 128,
        "texture_height": 32
      },
      "cape": "geometry.cape",
      "item_display_transforms": {
        "gui": {"translation": [1, 2, 3], "rotation": [30, 225, 0], "scale": [0.625, 0.625, 0.625], "rotation_pivot": [0, 8, 0], "scale_pivot": [0, 8, 0], "fit_to_frame": false},
        "firstperson_righthand": {"rotation": [0, 45, 0]},
        "firstperson_lefthand": {"rotation": [0, 225, 0]},
        "thirdperson_righthand": {"scale": [0.375, 0.375, 0.375]},
        "thirdperson_lefthand": {"scale": [0.375, 0.375, 0.375]},
        "ground": {"translation": [0, 3, 0]},
        "fixed": {"scale": [0.5, 0.5, 0.5]},
        "head": {},
        "embedded": {"translation": [0, 0, 1]},
        "shelf": {"scale": [1, 1, 1]}
      },
      "bones": [
        {
          "name": "body",
          "pivot": [0, 24, 0],
          "rotation": [0, 90, 0],
          "mirror": true,
          "inflate": 0.25,
          "debug": false,
          "render_group_id": 3,
          "binding": "q.item_slot_to_bone_name(c.item_slot)",
          "cubes": [
            {"origin": [-4, 12, -2], "size": [8, 12, 4], "rotation": [0, 0, 45], "pivot": [0, 18, 0], "inflate": 0.5, "mirror": false, "uv": [16, 16]},
            {"origin": [0, 0, 0], "size": [1, 1, 1], "uv": {
              "north": {"uv": [0, 0], "uv_size": [1, 1], "uv_rotation": 90, "material_instance": "side"},
              "south": {"uv": [1, 0]},
              "east": {"uv": [2, 0], "uv_rotation": 0},
              "west": {"uv": [3, 0], "uv_rotation": 180},
              "up": {"uv": [4.5, 0], "uv_size": [-1, -1]},
              "down": {"uv": [5, 0], "uv_rotation": 270}
            }},
            {}
          ],
          "locators": {
            "lead": [0, 1, 2],
            "held": {"offset": [0, 1, 2], "rotation": [3, 4, 5], "ignore_inherited_scale": true}
          },
          "poly_mesh": {
            "normalized_uvs": true,
            "positions": [[0, 0, 0], [1, 0, 0], [0, 1, 0]],
            "normals": [[0, 0, 1]],
            "uvs": [[0, 0], [1, 0], [0, 1]],
            "polys": [[[0, 0, 0], [1, 0, 1], [2, 0, 2]]]
          },
          "texture_meshes": [
            {"texture": "default", "position": [0, 1, 0], "local_pivot": [8, 8, 0], "rotation": [90, 0, 0], "scale": [2, 2, 2], "use_pixel_depth": false}
          ]
        },
        {"name": "head", "parent": "body", "poly_mesh": {"polys": "tri_list"}},
        {"name": "empty", "cubes": [], "locators": {}, "texture_meshes": []},
        {"name": "number", "binding": 1.5},
        {"name": "flag", "binding": true}
      ]
    },
    {"description": {"identifier": "geometry.bare"}}
  ]
}"#;

#[test]
fn every_member_of_the_newest_schema_round_trips_as_written() {
    let document = loaded(load(FULL));
    clean(document.diagnostics());
    let written = round_trip(&document);
    assert_eq!(json(&written), json(FULL), "nothing added, nothing lost");
    let file = document.value();
    assert_eq!(file.format_version, FormatVersion::new(1, 26, 40));
    assert_eq!(file.debug, Some(true));
    assert_eq!(file.geometry("GEOMETRY.bare").unwrap().bones, None);
    assert!(file.geometry("geometry.none").is_none());
    let full = file.geometry("geometry.full").unwrap();
    let names: Vec<&str> = full
        .item_display_transforms
        .as_ref()
        .unwrap()
        .iter()
        .map(|(name, _)| name)
        .collect();
    assert_eq!(
        names,
        bedrock_addon::families::geometry::ItemDisplayTransforms::NAMES
    );
    // Empty members keep their presence.
    let empty = full.bone("empty").unwrap();
    assert_eq!(empty.cubes.as_ref().map(Vec::len), Some(0));
    assert_eq!(
        empty.locators.as_ref().map(indexmap::map::IndexMap::len),
        Some(0)
    );
}

#[test]
fn an_upgraded_file_without_legacy_members_round_trips_through_the_newest_shape() {
    let old = r#"{"format_version": "1.8.0", "geometry.pig": {"texturewidth": 64, "textureheight": 32,
        "bones": [{"name": "Body", "pivot": [0, 13, 2], "cubes": [{"origin": [-5, 7, -5], "size": [10, 16, 8], "uv": [28, 8]}], "locators": {"lead": [0, 14, -6]}}]}}"#;
    let document = loaded(load(old));
    clean(document.diagnostics());
    let written = round_trip(&document);
    assert_eq!(
        json(&written),
        json(
            r#"{"format_version": "1.26.40", "minecraft:geometry": [{"description": {"identifier": "geometry.pig", "texture_width": 64, "texture_height": 32},
                "bones": [{"name": "body", "pivot": [0, 13, 2], "cubes": [{"origin": [-5, 7, -5], "size": [10, 16, 8], "pivot": [0, 15, -1], "uv": [28, 8]}], "locators": {"lead": [0, 14, -6]}}]}]}"#
        )
    );
}

/// A file whose geometries carry legacy members is written in
/// the shape and at the version it was read in, and reads back equal.
#[test]
fn a_file_with_legacy_members_is_written_in_the_shape_it_was_read_in() {
    let cases = [
        // Older than 1.8: `preserve_model_pose`, a legacy parent, `neverRender`, `reset`, an
        // object locator.
        r#"{"format_version": "1.1.0", "debug": true,
            "geometry.humanoid": {"texturewidth": 64, "textureheight": 32, "visible_bounds_width": 2, "bones": [{"name": "body", "pivot": [0, 24, 0], "cubes": [{"origin": [-4, 12, -2], "size": [8, 12, 4], "uv": [16, 16], "inflate": 0.5, "mirror": true}]}]},
            "geometry.zombie:geometry.humanoid": {"preserve_model_pose": true,
                "bones": [{"name": "head", "neverRender": true, "reset": true, "mirror": true, "inflate": 1, "debug": false, "render_group_id": 2, "rotation": [0, 90, 0],
                           "locators": {"lead": {"offset": [0, 1, 0], "ignore_inherited_scale": true}, "hand": [1, 2, 3]}}]}}"#,
        // 1.8: the geometry `debug`, `bind_pose_rotation`, a bone parent, a poly mesh, a texture
        // mesh.
        r#"{"format_version": "1.8.0",
            "geometry.npc:geometry.villager": {"debug": true, "visible_bounds_height": 3, "visible_bounds_offset": [0, 1, 0],
                "bones": [{"name": "head", "reset": false, "neverRender": false, "parent": "body", "bind_pose_rotation": [0, 90, 0], "cubes": [{}, {"size": [1, 1, 1]}],
                           "locators": {"lead": [0, 1, 2]}, "poly_mesh": {"polys": "tri_list"}, "texture_meshes": [{"texture": "t"}]},
                          {"name": "body"}]},
            "geometry.villager": {"bones": [{"name": "body"}]}}"#,
    ];
    for text in cases {
        let document = loaded(load(text));
        let written = document.to_json().unwrap();
        assert_eq!(json(&written), json(text), "the shape and version as read");
        let again = loaded(load(&written));
        assert_eq!(again.value(), document.value(), "{written}");
        assert_eq!(again.to_json().unwrap(), written);
        clean(again.diagnostics());
    }
    // A file without a `format_version` was read as 1.0.0 and is written as that.
    let document = loaded(load(r#"{"geometry.a:geometry.b": {}}"#));
    let written = document.to_json().unwrap();
    assert_eq!(
        json(&written),
        json(r#"{"format_version": "1.0.0", "geometry.a:geometry.b": {}}"#)
    );
    assert_eq!(loaded(load(&written)).value(), document.value());
}

type Change = Box<dyn Fn(&mut GeometryFile)>;

const OLD_SHAPE: &str = r#"{"format_version": "1.1.0", "geometry.b:geometry.a": {"preserve_model_pose": true, "bones": [{"name": "x", "cubes": [{"size": [2, 2, 2]}]}]}, "geometry.a": {}}"#;

const V18_SHAPE: &str = r#"{"format_version": "1.8.0", "geometry.b:geometry.a": {"bones": [{"name": "x", "neverRender": true, "cubes": [{"size": [2, 2, 2]}]}]}, "geometry.a": {}}"#;

/// The path `to_json` refuses at once `change` is applied to the file `text` loads as.
fn refusal(text: &str, change: &dyn Fn(&mut GeometryFile)) -> String {
    let mut file = loaded(load(text)).into_value();
    file.to_json().expect("unchanged, it is written");
    change(&mut file);
    match file.to_json() {
        Err(EncodeError::NotRepresentable { at, what }) => {
            assert_eq!(what, Unrepresentable::Unsupported);
            at.to_string()
        }
        other => panic!("{other:?}"),
    }
}

/// What neither shape holds: each change, with the member the refusal names.
fn changes_both_shapes() -> Vec<(&'static str, Change)> {
    use bedrock_addon::families::geometry::{CubeUv, ItemDisplayTransforms};
    use bedrock_addon::molang::Molang;

    vec![
        (
            "bones/0/binding",
            Box::new(|f| {
                f.geometries[1].bones.as_mut().unwrap()[0].binding = Some(Molang::Const(1.0));
            }),
        ),
        (
            "item_display_transforms",
            Box::new(|f| {
                f.geometries[1].item_display_transforms = Some(ItemDisplayTransforms::default());
            }),
        ),
        (
            "cape",
            Box::new(|f| f.geometries[1].cape = Some("c".to_owned())),
        ),
        (
            "bones/0/cubes/0/rotation",
            Box::new(|f| {
                f.geometries[1].bones.as_mut().unwrap()[0]
                    .cubes
                    .as_mut()
                    .unwrap()[0]
                    .rotation = Some([0.0, 0.0, 45.0]);
            }),
        ),
        (
            "bones/0/cubes/0/pivot",
            Box::new(|f| {
                f.geometries[1].bones.as_mut().unwrap()[0]
                    .cubes
                    .as_mut()
                    .unwrap()[0]
                    .pivot = Some([9.0, 9.0, 9.0]);
            }),
        ),
        (
            "bones/0/cubes/0/uv",
            Box::new(|f| {
                f.geometries[1].bones.as_mut().unwrap()[0]
                    .cubes
                    .as_mut()
                    .unwrap()[0]
                    .uv = Some(CubeUv::Faces(Box::default()));
            }),
        ),
        (
            "description/identifier",
            Box::new(|f| f.geometries[1].description.identifier = String::from("geometry.0")),
        ),
        (
            "description/identifier",
            Box::new(|f| f.geometries[1].description.identifier = String::from("demo:geometry.b")),
        ),
        (
            "__legacy/parent",
            Box::new(|f| f.geometries[1].legacy.parent = Some("demo:geometry.a".to_owned())),
        ),
        // The key of the geometry before it, a second time.
        (
            "description/identifier",
            Box::new(|f| {
                f.geometries[1].description.identifier = String::from("geometry.a");
                f.geometries[1].legacy.parent = None;
            }),
        ),
        (
            "__legacy",
            Box::new(|f| f.format_version = FormatVersion::new(1, 12, 0)),
        ),
        (
            "__legacy",
            Box::new(|f| f.format_version = FormatVersion::new(1, 26, 40)),
        ),
    ]
}

/// What only the shape older than 1.8 lacks.
fn changes_only_old_shape() -> Vec<(&'static str, Change)> {
    vec![
        (
            "bones/0/parent",
            Box::new(|f| f.geometries[1].bones.as_mut().unwrap()[0].parent = Some("y".to_owned())),
        ),
        (
            "bones/0/__legacy/reset",
            Box::new(|f| f.geometries[1].bones.as_mut().unwrap()[0].legacy.reset = Some(false)),
        ),
        (
            "bones/0/__legacy/bind_pose_rotation",
            Box::new(|f| {
                f.geometries[1].bones.as_mut().unwrap()[0]
                    .legacy
                    .bind_pose_rotation = Some([0.0; 3]);
            }),
        ),
        (
            "bones/0/poly_mesh",
            Box::new(|f| {
                f.geometries[1].bones.as_mut().unwrap()[0].poly_mesh =
                    Some(bedrock_addon::families::geometry::PolyMesh {
                        normalized_uvs: None,
                        positions: None,
                        normals: None,
                        uvs: None,
                        polys: bedrock_addon::families::geometry::Polys::List(
                            "tri_list".to_owned(),
                        ),
                    });
            }),
        ),
        (
            "bones/0/texture_meshes",
            Box::new(|f| {
                f.geometries[1].bones.as_mut().unwrap()[0].texture_meshes = Some(Vec::new());
            }),
        ),
        (
            "bones/0/cubes/0",
            Box::new(|f| {
                f.geometries[1].bones.as_mut().unwrap()[0]
                    .cubes
                    .as_mut()
                    .unwrap()[0] = Cube::default();
            }),
        ),
        (
            "__legacy/debug",
            Box::new(|f| f.geometries[1].legacy.debug = Some(true)),
        ),
    ]
}

/// What only the 1.8 shape lacks.
fn changes_only_v18_shape() -> Vec<(&'static str, Change)> {
    use bedrock_addon::families::geometry::Locator;

    vec![
        (
            "__legacy/preserve_model_pose",
            Box::new(|f| f.geometries[1].legacy.preserve_model_pose = Some(true)),
        ),
        (
            "bones/0/locators/lead",
            Box::new(|f| {
                f.geometries[1].bones.as_mut().unwrap()[0].locators = Some(
                    [(
                        "lead".to_owned(),
                        Locator::Placed {
                            offset: [0.0; 3],
                            rotation: None,
                            ignore_inherited_scale: None,
                        },
                    )]
                    .into_iter()
                    .collect(),
                );
            }),
        ),
    ]
}

/// `to_json` never gives text neither this loader nor vanilla reads. A legacy file the
/// caller gave members its shape cannot hold is refused, at the member.
#[test]
fn a_legacy_file_changed_beyond_its_shape_is_refused_at_the_member() {
    // The geometries are in key order: `geometry.a` is 0, `geometry.b` is 1.
    let g = "minecraft:geometry/1";
    for text in [OLD_SHAPE, V18_SHAPE] {
        for (at, change) in &changes_both_shapes() {
            assert_eq!(refusal(text, change), format!("{g}/{at}"), "{text}");
        }
    }
    for (at, change) in &changes_only_old_shape() {
        assert_eq!(refusal(OLD_SHAPE, change), format!("{g}/{at}"));
        // The 1.8 shape holds each of them.
        let mut file = loaded(load(V18_SHAPE)).into_value();
        change(&mut file);
        let written = file.to_json().unwrap();
        let again = loaded(load(&written));
        assert_eq!(again.to_json().unwrap(), written, "{at}");
    }
    for (at, change) in &changes_only_v18_shape() {
        assert_eq!(refusal(V18_SHAPE, change), format!("{g}/{at}"));
    }
}

#[test]
fn a_file_of_1_12_or_later_given_a_side_car_has_no_shape_that_holds_it() {
    // A file of 1.12 or later given a side-car has no shape that holds it.
    let mut file = loaded(load(r#"{"format_version": "1.16.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x"}, "bones": [{"name": "a"}]}]}"#)).into_value();
    file.geometries[0].bones.as_mut().unwrap()[0]
        .legacy
        .never_render = Some(true);
    assert!(matches!(
        file.to_json(),
        Err(EncodeError::NotRepresentable { at, .. }) if at.to_string() == "minecraft:geometry/0/bones/0/__legacy"
    ));
}

#[test]
fn a_file_named_extra_or_carrying_an_extra_member_is_judged_like_any_other_member() {
    // No field of the family is a skipped `extra`: the member is undeclared, nothing more.
    let text = r#"{"format_version": "1.12.0", "extra": 1, "minecraft:geometry": [{"description": {"identifier": "geometry.x"}, "extra": {}}]}"#;
    let document = loaded(load(text));
    assert_eq!(document.diagnostics().len(), 2);
}

#[test]
fn a_built_file_writes_the_newest_format_and_loads_again() {
    let mut body = Bone::new("Body");
    body.cubes = Some(vec![Cube::new([0.0, 0.0, 0.0], [16.0, 16.0, 16.0])]);
    body.texture_meshes = Some(vec![TextureMesh::new("default")]);
    let mut geometry = Geometry::new("geometry.ruby_lamp");
    geometry.bones = Some(vec![body]);
    let file = GeometryFile::new(vec![geometry]);
    assert_eq!(file.format_version, FormatVersion::new(1, 26, 40));
    let text = file.to_json().unwrap();
    assert_eq!(
        json(&text),
        json(
            r#"{"format_version": "1.26.40", "minecraft:geometry": [{"description": {"identifier": "geometry.ruby_lamp"},
                "bones": [{"name": "body", "cubes": [{"origin": [0, 0, 0], "size": [16, 16, 16]}], "texture_meshes": [{"texture": "default"}]}]}]}"#
        )
    );
    let again = loaded(load(&text));
    clean(again.diagnostics());
    assert_eq!(*again.value(), file);
    let mut bytes = Vec::new();
    file.write_json(&mut bytes, bedrock_addon::json::WriteStyle::default())
        .unwrap();
    assert_eq!(String::from_utf8(bytes).unwrap(), text);
}

/// The refusal names the JSON path of the value.
#[test]
fn a_number_that_is_not_finite_is_refused_by_the_writer_at_its_path() {
    let refusal = |file: &GeometryFile| match file.to_json() {
        Err(EncodeError::NotRepresentable { at, what }) => (at.to_string(), what),
        other => panic!("{other:?}"),
    };
    let mut geometry = Geometry::new("geometry.x");
    geometry.description.visible_bounds_width = Some(f32::NAN);
    assert_eq!(
        refusal(&GeometryFile::new(vec![
            Geometry::new("geometry.ok"),
            geometry
        ])),
        (
            "minecraft:geometry/1/description/visible_bounds_width".to_owned(),
            Unrepresentable::NaN
        )
    );
    let mut bone = Bone::new("body");
    bone.cubes = Some(vec![
        Cube::new([0.0; 3], [1.0; 3]),
        Cube::new([0.0, 0.0, f32::INFINITY], [1.0; 3]),
    ]);
    let mut geometry = Geometry::new("geometry.x");
    geometry.bones = Some(vec![Bone::new("head"), bone]);
    assert_eq!(
        refusal(&GeometryFile::new(vec![geometry])),
        (
            "minecraft:geometry/0/bones/1/cubes/1/origin/2".to_owned(),
            Unrepresentable::Infinity
        )
    );
    let mut bone = Bone::new("body");
    bone.locators = Some(
        [(
            "lead".to_owned(),
            bedrock_addon::families::geometry::Locator::Placed {
                offset: [0.0; 3],
                rotation: Some([0.0, f32::NEG_INFINITY, 0.0]),
                ignore_inherited_scale: None,
            },
        )]
        .into_iter()
        .collect(),
    );
    let mut geometry = Geometry::new("geometry.x");
    geometry.bones = Some(vec![bone]);
    assert_eq!(
        refusal(&GeometryFile::new(vec![geometry])),
        (
            "minecraft:geometry/0/bones/0/locators/lead/rotation/1".to_owned(),
            Unrepresentable::NegativeInfinity
        )
    );
}

/// Brief: a geometry file is named after its first geometry and written under `models/entity/`
/// (Block geometry belongs in `models/blocks/`, a path the content does not give.)
#[test]
fn the_default_path_is_named_after_the_first_geometry_under_models_entity() {
    let file = GeometryFile::new(vec![
        Geometry::new("geometry.ruby_lamp"),
        Geometry::new("geometry.second"),
    ]);
    assert_eq!(
        file.default_path().as_str(),
        "models/entity/geometry.ruby_lamp.json"
    );
    assert_eq!(
        GeometryFile::new(Vec::new()).default_path().as_str(),
        "models/entity/geometry.json"
    );
    // A base-game block geometry: the namespace is a folder, as for every namespaced key.
    let block = GeometryFile::new(vec![Geometry::new("minecraft:geometry.full_block")]);
    assert!(block.default_path().as_str().starts_with("models/entity/"));
    let loaded_block = loaded(load_at(
        "models/blocks/x.json",
        r#"{"format_version": "1.21.0", "minecraft:geometry": [{"description": {"identifier": "minecraft:geometry.full_block"}}]}"#,
        &base_game(),
    ));
    assert_eq!(
        loaded_block.value().default_path(),
        block.default_path(),
        "the path it was read from is not in the content"
    );
}

/// Files of the 1.26.50 sample packs' `resource_pack/models/`, copied unchanged to
/// `tests/fixtures/geometry/geometry/vanilla/`.
#[test]
fn vanilla_files_load_clean_and_round_trip() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/geometry/geometry/vanilla");
    let cases = [
        (
            "leash_knot.geo.json",
            "models/entity/leash_knot.geo.json",
            1,
        ),
        (
            "item_sprite.geo.json",
            "models/entity/item_sprite.geo.json",
            1,
        ),
        (
            "zombie.v1.0.geo.json",
            "models/entity/zombie.v1.0.geo.json",
            1,
        ),
        ("spyglass.geo.json", "models/entity/spyglass.geo.json", 1),
        (
            "shelf_mushroom_small.geo.json",
            "models/blocks/shelf_mushroom_small.geo.json",
            1,
        ),
    ];
    for (name, path, geometries) in cases {
        let text = std::fs::read_to_string(root.join(name)).unwrap();
        let document = loaded(load_at(path, &text, &base_game()));
        clean(document.diagnostics());
        assert_eq!(document.value().geometries.len(), geometries, "{name}");
        let written = document.to_json().unwrap();
        let again = loaded(load_at(path, &written, &base_game()));
        clean(again.diagnostics());
        assert_eq!(
            again.value().geometries,
            document.value().geometries,
            "{name}"
        );
        assert_eq!(again.to_json().unwrap(), written, "{name}");
    }
    let _ = round_trip;
}

/// The newest-path files of 1.26.60 (copied to
/// `tests/fixtures/geometry/geometry/vanilla_current/`): `vanilla_1.26.60`'s `drowned` (format
/// 1.21.60) and `experimental_y_2026_drop_4`'s `frostbite` (1.21.120), both through
/// `upgradeToV1_26_40`. Each loads from the base-game pack without a warning, upgraded, writes
/// text that reads back to the same geometries, and writes it again unchanged.
#[test]
fn the_1_26_60_files_load_clean_through_the_1_26_40_step_and_round_trip() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/geometry/geometry/vanilla_current");
    for (name, path) in [
        ("drowned.geo.json", "models/entity/drowned.geo.json"),
        ("frostbite.geo.json", "models/entity/frostbite.geo.json"),
    ] {
        let text = std::fs::read_to_string(root.join(name)).unwrap();
        let document = loaded(load_at(path, &text, &base_game()));
        clean(document.diagnostics());
        assert!(document.version().upgraded, "{name}");
        assert_eq!(document.value().geometries.len(), 1, "{name}");
        let written = document.to_json().unwrap();
        let again = loaded(load_at(path, &written, &base_game()));
        clean(again.diagnostics());
        assert_eq!(
            again.value().geometries,
            document.value().geometries,
            "{name}"
        );
        assert_eq!(again.to_json().unwrap(), written, "{name}");
    }
}

/// An object locator without `rotation` exists only in the shape older
/// than 1.8, so a file that holds one is written in that shape, side-car or not; in a file of
/// another version it is refused at the member.
#[test]
fn a_locator_object_without_rotation_is_written_in_the_shape_that_holds_it() {
    use bedrock_addon::json::EncodeError;
    for version in ["1.1.0", "1.7.0"] {
        let text = format!(
            r#"{{"format_version": "{version}", "geometry.x": {{"bones": [{{"name": "a", "locators": {{"k": {{"offset": [0, 1, 0]}}}}}}]}}}}"#
        );
        let document = loaded(load(&text));
        let written = document.to_json().unwrap();
        assert_eq!(json(&written), json(&text));
        let again = loaded(load(&written));
        assert_eq!(again.value(), document.value());
        // In another version no shape holds it.
        for other in [(1, 8, 0), (1, 12, 0), (1, 26, 40)] {
            let mut file = document.value().clone();
            file.format_version = FormatVersion::new(other.0, other.1, other.2);
            assert!(
                matches!(
                    file.to_json(),
                    Err(EncodeError::NotRepresentable { at, .. })
                        if at.to_string() == "minecraft:geometry/0/bones/0/locators/k/rotation"
                ),
                "{other:?}: {:?}",
                file.to_json()
            );
        }
    }
    // With a rotation the newest shape holds it.
    let document = loaded(load(
        r#"{"format_version": "1.1.0", "geometry.x": {"bones": [{"name": "a", "locators": {"k": {"offset": [0, 1, 0], "rotation": [0, 0, 0]}}}]}}"#,
    ));
    assert!(document.to_json().unwrap().contains("minecraft:geometry"));
    round_trip(&document);
}

/// A refusal inside the description of a legacy-shape file names the
/// `description` it is in.
#[test]
fn a_refusal_in_a_legacy_shape_description_names_the_description() {
    let mut file = loaded(load(
        r#"{"format_version": "1.8.0", "geometry.a": {}, "geometry.b:geometry.a": {"visible_bounds_width": 2}}"#,
    ))
    .into_value();
    file.geometries[1].description.visible_bounds_width = Some(f32::NAN);
    assert!(matches!(
        file.to_json(),
        Err(EncodeError::NotRepresentable { at, what: Unrepresentable::NaN })
            if at.to_string() == "minecraft:geometry/1/description/visible_bounds_width"
    ));
    file.geometries[1].description.visible_bounds_width = None;
    file.geometries[1].description.visible_bounds_offset = Some([0.0, f32::INFINITY, 0.0]);
    assert!(matches!(
        file.to_json(),
        Err(EncodeError::NotRepresentable { at, .. })
            if at.to_string() == "minecraft:geometry/1/description/visible_bounds_offset/1"
    ));
}

/// In a file older than 1.8 a cube's `mirror`
/// that is a string other than "true" is dropped by the reader (an info); a cube that had no
/// other member is then a cube without members. Such a file is written in the newest shape,
/// where the cube has the `pivot` the upgrade gave it, unless it must keep its own shape
/// (here: a locator object without `rotation`), which has no `pivot` member: the cube would
/// be `{}`, which the oldest step removes. It is written with the member it had, as the
/// empty string, which the reader does not read either (a file refused or an `origin` the author had not written would be wrong), and the text is stable. A "true" or
/// `true` mirror is kept and the file is written.
#[test]
fn a_loaded_legacy_cube_whose_only_member_is_an_unread_mirror_is_written_with_that_member() {
    let file = |version: &str, locator: &str, mirror: &str| {
        format!(
            r#"{{{version}"geometry.a": {{"bones": [{{"name": "b", "locators": {{"l0": {locator}}}, "cubes": [{{"origin": [0, 0, 0]}}, {{"mirror": {mirror}}}]}}]}}}}"#
        )
    };
    let without_rotation = r#"{"offset": [0, 1, 0]}"#;
    for version in [
        "",
        r#""format_version": "1.0.0", "#,
        r#""format_version": "1.7.0", "#,
    ] {
        let document = loaded(load(&file(version, without_rotation, r#""maybe""#)));
        // The reader said what it dropped, and where.
        let dropped: Vec<String> = document
            .diagnostics()
            .iter()
            .filter(|f| f.code.name() == "DroppedField")
            .map(|f| f.location.json_path.to_string())
            .collect();
        assert_eq!(dropped, ["geometry.a/bones/0/cubes/1/mirror"], "{version}");
        let cube = &document.value().geometries[0].bones()[0]
            .cubes
            .as_ref()
            .unwrap()[1];
        assert_eq!(cube.mirror, None, "{version}");
        let written = document
            .value()
            .to_json()
            .expect("the file as loaded is written");
        assert!(!written.contains("minecraft:geometry"), "{version}");
        let again = loaded(load(&written));
        let cubes = again.value().geometries[0].bones()[0]
            .cubes
            .clone()
            .unwrap();
        assert_eq!(cubes.len(), 2, "{version}: the cube is still a cube");
        assert_eq!(cubes[1], *cube, "{version}: value for value");
        assert!(written.contains(r#""mirror": """#), "{version}: {written}");
        assert_eq!(cubes[1].pivot, cube.pivot, "{version}");
        assert_eq!(again.value().to_json().unwrap(), written, "{version}");
        // A mirror the reader keeps leaves the cube a member: the file is written in its
        // own shape and reads back equal.
        for kept in [r#""true""#, "true"] {
            let document = loaded(load(&file(version, without_rotation, kept)));
            let written = document.value().to_json().unwrap();
            assert!(!written.contains("minecraft:geometry"), "{version} {kept}");
            let again = loaded(load(&written));
            assert_eq!(again.value().geometries, document.value().geometries);
        }
        // Without the locator nothing holds the file to its shape: it is written in the
        // newest one, where the cube has its pivot.
        let free = loaded(load(&file(version, "[0, 1, 0]", r#""maybe""#)));
        let written = free.value().to_json().unwrap();
        assert!(written.contains("minecraft:geometry"), "{version}");
        loaded(load(&written));
    }
}
