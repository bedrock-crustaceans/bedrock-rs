//! The fixtures (g01–g29): the version gates, the schema of each era in
//! its lenient and strict mode, the identifier rules and the parse-time checks.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::geometry::{CubeUv, FitToFrame, Locator};
use bedrock_addon::version::FormatVersion;

use crate::common::{
    DID_NOT_VALIDATE, base_game, clean, count, engine, expect, expect_at, fails_schema, first,
    load, load_at, loaded, rejected, rp, with_bone,
};

fn geometry(version: &str, identifier: &str, members: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:geometry": [{{"description": {{"identifier": "{identifier}"}}{members}}}]}}"#
    )
}

fn transforms(version: &str, transforms: &str) -> String {
    geometry(
        version,
        "geometry.x",
        &format!(r#", "item_display_transforms": {transforms}"#),
    )
}

fn cube(version: &str, cube: &str) -> String {
    with_bone(version, &format!(r#", "cubes": [{cube}]"#))
}

// ---- file-level ------------------------------------------------------------------------------

#[test]
fn g01_exceeds_engine() {
    let d = rejected(load(&geometry("9.0.0", "geometry.x", "")));
    expect_at(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "format_version",
        "Error: geometry file format version: 9.0.0 exceeds game version: 1.26.60. Skipping.",
    );
    assert_eq!(d.len(), 1, "nothing else is read");
    // The engine version itself loads; the patch above it does not.
    loaded(load(&geometry("1.26.60", "geometry.x", "")));
    rejected(load(&geometry("1.26.61", "geometry.x", "")));
    let d = rejected(load_at(
        "models/entity/x.json",
        &geometry("1.26.40", "geometry.x", ""),
        &engine(1, 26, 20),
    ));
    expect(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "Error: geometry file format version: 1.26.40 exceeds game version: 1.26.20. Skipping.",
    );
}

#[test]
fn g02_not_object() {
    for text in ["[]", "5", "\"geometry\"", "true"] {
        let d = rejected(load(text));
        expect(
            &d,
            "MalformedInput",
            Severity::Error,
            "geometry file could not be loaded: expected the contents to be inside a JSON object - The file has to start with an \"{\" and end with a \"}\"",
        );
        assert_eq!(d.len(), 1, "{text}");
    }
}

#[test]
fn g03_syntax_error() {
    let d = rejected(load(
        r#"{"format_version": "1.12.0", "minecraft:geometry": ["#,
    ));
    assert_eq!(d.len(), 1);
    let finding = d.iter().next().unwrap();
    assert_eq!(finding.code.name(), "MalformedInput");
    assert_eq!(finding.severity, Severity::Error);
    assert!(
        finding.code.to_string().starts_with("parse error:\n"),
        "{}",
        finding.code
    );
}

#[test]
fn a_file_nested_too_deep_is_a_parse_error() {
    let text = format!("{}{}", "[".repeat(100_000), "]".repeat(100_000));
    let d = rejected(load(&text));
    expect(
        &d,
        "MalformedInput",
        Severity::Error,
        "parse error:\nnesting too deep",
    );
}

#[test]
fn g04_strict_121_unknown_member() {
    let text = r#"{"format_version": "1.21.0", "minecraft:geometry": [
        {"description": {"identifier": "geometry.a"}, "bones": [{"name": "body", "neverRender": true}]},
        {"description": {"identifier": "geometry.b"}}]}"#;
    let d = rejected(load(text));
    expect_at(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "minecraft:geometry/0/bones/0/neverRender",
        "child 'neverRender' not valid here.",
    );
    expect(&d, "InvalidValue", Severity::Error, DID_NOT_VALIDATE);
}

#[test]
fn g05_lenient_112_unknown_member() {
    let document = loaded(load(&with_bone("1.12.0", r#", "neverRender": true"#)));
    expect_at(
        document.diagnostics(),
        "UnrecognizedField",
        Severity::Info,
        "minecraft:geometry/0/bones/0/neverRender",
        "child 'neverRender' not valid here (removed; format_version < 1.19.30)",
    );
    assert!(!document.diagnostics().has_errors());
    let bone = &document.value().geometries[0].bones()[0];
    assert_eq!(bone.name, "body");
    assert!(bone.legacy.is_empty(), "a native 1.12 file has no side-car");
}

#[test]
fn an_unknown_member_is_judged_at_every_level_of_the_file() {
    // Root, geometry, description, cube, face: removed below 1.19.30, fatal from it.
    let file = |version: &str| {
        format!(
            r#"{{"format_version": "{version}", "extra_root": 1, "minecraft:geometry": [{{"extra_geometry": 1,
                "description": {{"identifier": "geometry.x", "debug": true}},
                "bones": [{{"name": "body", "cubes": [{{"extra_cube": 1, "uv": {{"north": {{"uv": [0, 0], "extra_face": 1}}, "top": {{}}}}}}]}}]}}]}}"#
        )
    };
    let document = loaded(load(&file("1.19.29")));
    let d = document.diagnostics();
    assert_eq!(count(d, "UnrecognizedField", Severity::Info), 6);
    for at in [
        "extra_root",
        "minecraft:geometry/0/extra_geometry",
        "minecraft:geometry/0/description/debug",
        "minecraft:geometry/0/bones/0/cubes/0/extra_cube",
        "minecraft:geometry/0/bones/0/cubes/0/uv/north/extra_face",
        "minecraft:geometry/0/bones/0/cubes/0/uv/top",
    ] {
        assert!(
            d.iter().any(|f| f.location.json_path.to_string() == at),
            "{at}"
        );
    }
    let d = rejected(load(&file("1.19.30")));
    assert_eq!(count(&d, "UnrecognizedField", Severity::Error), 6);
    expect(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "child 'debug' not valid here.",
    );
}

#[test]
fn a_member_of_the_wrong_type_is_removed_below_1_19_30_and_fatal_from_it() {
    let bone = r#", "pivot": "centre", "inflate": "1", "mirror": 1, "cubes": {}"#;
    let document = loaded(load(&with_bone("1.19.29", bone)));
    let d = document.diagnostics();
    assert_eq!(count(d, "InvalidType", Severity::Info), 4);
    expect_at(
        d,
        "InvalidType",
        Severity::Info,
        "minecraft:geometry/0/bones/0/pivot",
        "unknown child schema option type.  Allowed types: array",
    );
    expect_at(
        d,
        "InvalidType",
        Severity::Info,
        "minecraft:geometry/0/bones/0/mirror",
        "unknown child schema option type.  Allowed types: bool",
    );
    let body = &document.value().geometries[0].bones()[0];
    assert_eq!((body.pivot, body.inflate, body.mirror), (None, None, None));
    assert!(body.cubes.is_none());
    let d = rejected(load(&with_bone("1.19.30", bone)));
    assert_eq!(count(&d, "InvalidType", Severity::Error), 4);
    expect(&d, "InvalidValue", Severity::Error, DID_NOT_VALIDATE);
}

#[test]
fn g06_missing_identifier() {
    let text = r#"{"format_version": "1.12.0", "minecraft:geometry": [{"description": {}}]}"#;
    fails_schema(
        load(text),
        "MissingRequiredField",
        "minecraft:geometry/0/description",
        "Required child identifier not found",
    );
}

#[test]
fn every_required_member_is_required_in_the_lenient_mode_too() {
    let cases = [
        (
            r#"{"format_version": "1.12.0"}"#.to_owned(),
            "Required child minecraft:geometry not found",
            "",
        ),
        (
            r#"{"format_version": "1.12.0", "minecraft:geometry": [{}]}"#.to_owned(),
            "Required child description not found",
            "minecraft:geometry/0",
        ),
        (
            r#"{"format_version": "1.12.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x"}, "bones": [{}]}]}"#.to_owned(),
            "Required child name not found",
            "minecraft:geometry/0/bones/0",
        ),
        (
            cube("1.12.0", r#"{"uv": {"north": {}}}"#),
            "Required child uv not found",
            "minecraft:geometry/0/bones/0/cubes/0/uv/north",
        ),
        (
            with_bone("1.12.0", r#", "poly_mesh": {}"#),
            "Required child polys not found",
            "minecraft:geometry/0/bones/0/poly_mesh",
        ),
        (
            with_bone("1.12.0", r#", "texture_meshes": [{}]"#),
            "Required child texture not found",
            "minecraft:geometry/0/bones/0/texture_meshes/0",
        ),
        (
            with_bone("1.12.0", r#", "locators": {"a": {"rotation": [0, 0, 0]}}"#),
            "Required child offset not found",
            "minecraft:geometry/0/bones/0/locators/a",
        ),
        (
            r#"{"format_version": "1.8.0"}"#.to_owned(),
            r"Required child geometry\.[a-zA-Z0-9_.'-:]+ not found",
            "",
        ),
        (
            r#"{"format_version": "1.8.0", "geometry.x": {"bones": [{}]}}"#.to_owned(),
            "Required child name not found",
            "geometry.x/bones/0",
        ),
        (
            r#"{"geometry.x": {"bones": [{}]}}"#.to_owned(),
            "Required child name not found",
            "geometry.x/bones/0",
        ),
    ];
    for (text, message, at) in cases {
        fails_schema(load(&text), "MissingRequiredField", at, message);
    }
    // A required member of the wrong type is removed and then missing.
    let document = load(&with_bone("1.12.0", "").replace(r#""name": "body""#, r#""name": 5"#));
    fails_schema(
        document,
        "MissingRequiredField",
        "minecraft:geometry/0/bones/0",
        "Required child name not found",
    );
}

#[test]
fn g07_vec3_length() {
    fails_schema(
        load(&with_bone("1.16.0", r#", "pivot": [0, 1]"#)),
        "InvalidValue",
        "minecraft:geometry/0/bones/0/pivot",
        "Array too small (2 < 3)",
    );
    fails_schema(
        load(&with_bone("1.16.0", r#", "pivot": [0, 1, 2, 3]"#)),
        "InvalidValue",
        "minecraft:geometry/0/bones/0/pivot",
        "Array too large (4 > 3)",
    );
    fails_schema(
        load(&cube("1.16.0", r#"{"uv": [0, 1, 2]}"#)),
        "InvalidValue",
        "minecraft:geometry/0/bones/0/cubes/0/uv",
        "Array too large (3 > 2)",
    );
    fails_schema(
        load(&cube("1.16.0", r#"{"uv": [0]}"#)),
        "InvalidValue",
        "minecraft:geometry/0/bones/0/cubes/0/uv",
        "Array too small (1 < 2)",
    );
    loaded(load(&with_bone("1.16.0", r#", "pivot": [0, 1, 2]"#)));
}

#[test]
fn g08_inheritance_after_112() {
    let file = |version: &str| {
        format!(
            r#"{{"format_version": "{version}", "minecraft:geometry": [
                {{"description": {{"identifier": "geometry.ok"}}}},
                {{"description": {{"identifier": "geometry.a:geometry.b"}}}}]}}"#
        )
    };
    for version in ["1.12.0", "1.16.0", "1.19.29"] {
        let d = rejected(load(&file(version)));
        expect_at(
            &d,
            "InvalidValue",
            Severity::Error,
            "minecraft:geometry/1/description/identifier",
            "Error: namespaces are not supported for custom geometry: identifier: geometry.a:geometry.b",
        );
        expect(&d, "InvalidValue", Severity::Error, DID_NOT_VALIDATE);
    }
    // From 1.19.30 the schema lets the `:` pass and the identifier loop refuses it: no
    // "didn't validate".
    let d = rejected(load(&file("1.19.30")));
    assert_eq!(count(&d, "InvalidValue", Severity::Error), 1);
}

#[test]
fn an_identifier_needs_a_geometry_name_somewhere_in_it() {
    fails_schema(
        load(&geometry("1.12.0", "pig", "")),
        "InvalidValue",
        "minecraft:geometry/0/description/identifier",
        "invalid value",
    );
    fails_schema(
        load(&geometry("1.21.0", "demo:pig", "")),
        "InvalidValue",
        "minecraft:geometry/0/description/identifier",
        "invalid value",
    );
    // A search, not a full match.
    loaded(load(&geometry("1.12.0", "foo.geometry.bar", "")));
    loaded(load_at(
        "models/blocks/x.json",
        &geometry("1.21.0", "minecraft:foo.geometry.bar", ""),
        &base_game(),
    ));
}

#[test]
fn g09_custom_namespace_rejected() {
    let text = r#"{"format_version": "1.21.0", "minecraft:geometry": [
        {"description": {"identifier": "geometry.ok"}},
        {"description": {"identifier": "demo:geometry.x"}},
        {"description": {"identifier": "geometry.after"}}]}"#;
    let d = rejected(load(text));
    expect_at(
        &d,
        "InvalidValue",
        Severity::Error,
        "minecraft:geometry/1/description/identifier",
        "Error: namespaces are not supported for custom geometry: identifier: demo:geometry.x",
    );
    assert_eq!(d.len(), 1, "the file validated: no \"didn't validate\"");
    // A base-game pack's entity geometry gets the other text; `models/mobs.json` too.
    for path in ["models/entity/x.json", "models/mobs.json"] {
        let d = rejected(load_at(path, text, &base_game()));
        expect(
            &d,
            "InvalidValue",
            Severity::Error,
            "Error: namespaces are not supported for internal entity geometry: identifier: demo:geometry.x",
        );
    }
    // A custom pack's block geometry is custom geometry.
    let d = rejected(load_at("models/blocks/x.json", text, &rp()));
    expect(
        &d,
        "InvalidValue",
        Severity::Error,
        "Error: namespaces are not supported for custom geometry: identifier: demo:geometry.x",
    );
}

#[test]
fn g10_base_blocks_floor() {
    let file = |version: &str| geometry(version, "minecraft:geometry.x", "");
    let d = rejected(load_at(
        "models/blocks/x.json",
        &file("1.16.0"),
        &base_game(),
    ));
    expect_at(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        "format_version",
        "Internal blocks are required to use geometry version >= 1.19.30",
    );
    assert_eq!(d.len(), 1);
    rejected(load_at(
        "models/blocks/x.json",
        &file("1.19.29"),
        &base_game(),
    ));
    loaded(load_at(
        "models/blocks/x.json",
        &file("1.19.30"),
        &base_game(),
    ));
    // Neither a custom pack's blocks nor the base game's entities have the floor.
    loaded(load_at(
        "models/blocks/x.json",
        &geometry("1.16.0", "geometry.x", ""),
        &rp(),
    ));
    loaded(load_at(
        "models/entity/x.json",
        &geometry("1.16.0", "geometry.x", ""),
        &base_game(),
    ));
    loaded(load_at(
        "models/blocks/sub/x.json",
        &file("1.21.0"),
        &base_game(),
    ));
}

#[test]
fn g11_base_blocks_namespace() {
    let at = "models/blocks/x.json";
    for identifier in ["geometry.x", "demo:geometry.x", "minecraft:pig"] {
        let d = rejected(load_at(
            at,
            &geometry("1.21.0", identifier, ""),
            &base_game(),
        ));
        if identifier == "minecraft:pig" {
            // The schema's validator refuses it first.
            expect(&d, "InvalidValue", Severity::Error, DID_NOT_VALIDATE);
            continue;
        }
        expect(
            &d,
            "InvalidValue",
            Severity::Error,
            &format!(
                "Error: Internal block geometries required the minecraft namespace: {identifier}"
            ),
        );
    }
    let document = loaded(load_at(
        at,
        &geometry("1.21.0", "minecraft:geometry.x", ""),
        &base_game(),
    ));
    clean(document.diagnostics());
}

#[test]
fn g12_embedded_before_121110() {
    let both = r#"{"embedded": {"scale": [1, 1, 1]}, "gui": {"scale": [2, 2, 2]}}"#;
    let document = loaded(load(&transforms("1.21.100", both)));
    expect_at(
        document.diagnostics(),
        "InvalidValue",
        Severity::Error,
        "minecraft:geometry/0/item_display_transforms/embedded",
        "\"item_display_transform\" for \"geometry.x\" is invalid - \"embedded\" can only be used in a file versioned 1.21.110 or higher",
    );
    let t = document.value().geometries[0]
        .item_display_transforms
        .clone()
        .unwrap();
    assert_eq!(t.embedded, None);
    assert_eq!(t.gui.unwrap().scale, Some([2.0, 2.0, 2.0]));
    for (version, read) in [("1.21.109", false), ("1.21.110", true)] {
        let geometry = first(&transforms(version, both));
        let t = geometry.item_display_transforms.unwrap();
        assert_eq!(t.embedded.is_some(), read, "{version}");
    }
    clean(loaded(load(&transforms("1.21.110", both))).diagnostics());
}

#[test]
fn g13_fit_to_frame_mode() {
    let mode = |version: &str, gui: &str| {
        let document = loaded(load(&transforms(version, &format!(r#"{{"gui": {gui}}}"#))));
        clean(document.diagnostics());
        let file = document.value();
        file.geometries[0]
            .item_display_transforms
            .clone()
            .unwrap()
            .gui
            .unwrap()
            .fit_to_frame(file.format_version)
    };
    let off = r#"{"fit_to_frame": false}"#;
    assert_eq!(mode("1.21.120", off), FitToFrame::Legacy);
    assert_eq!(mode("1.21.129", off), FitToFrame::Legacy);
    assert_eq!(mode("1.21.130", off), FitToFrame::Unscaled);
    assert_eq!(
        mode("1.21.120", r#"{"fit_to_frame": true}"#),
        FitToFrame::Fit
    );
    assert_eq!(mode("1.21.130", "{}"), FitToFrame::Fit);
}

#[test]
fn g14_transform_ranges() {
    let document = loaded(load(&transforms(
        "1.21.0",
        r#"{"gui": {"scale": [5, 1, 1], "translation": [0, 90, 0], "rotation_pivot": [0, 0, 100], "rotation": [10, 20, 30]},
            "head": {"scale_pivot": [-81, 0, 0], "rotation": [0, 0, 361]}}"#,
    )));
    let d = document.diagnostics();
    let at = "minecraft:geometry/0/item_display_transforms";
    for (member, text) in [
        (
            "gui/scale",
            "\"item_display_transform\" - scale (5.000000, 1.000000, 1.000000) is out of bounds. Allowed range: (0, 0, 0) to (4, 4, 4).",
        ),
        (
            "gui/translation",
            "\"item_display_transform\" - translation (0.000000, 90.000000, 0.000000) is out of bounds. Allowed range: (-80, -80, -80) to (80, 80, 80).",
        ),
        (
            "gui/rotation_pivot",
            "\"item_display_transform\" - pivot (0.000000, 0.000000, 100.000000) is out of bounds. Allowed range: (-80, -80, -80) to (80, 80, 80).",
        ),
        (
            "head/scale_pivot",
            "\"item_display_transform\" - pivot (-81.000000, 0.000000, 0.000000) is out of bounds. Allowed range: (-80, -80, -80) to (80, 80, 80).",
        ),
        (
            "head/rotation",
            "\"item_display_transform\" - rotation (0.000000, 0.000000, 361.000000) is out of bounds. Allowed range: (-360, -360, -360) to (360, 360, 360).",
        ),
    ] {
        expect_at(
            d,
            "InvalidValue",
            Severity::Error,
            &format!("{at}/{member}"),
            text,
        );
    }
    assert_eq!(count(d, "InvalidValue", Severity::Error), 5);
    let t = document.value().geometries[0]
        .item_display_transforms
        .clone()
        .unwrap();
    let gui = t.gui.unwrap();
    assert_eq!(
        (gui.scale, gui.translation, gui.rotation_pivot),
        (None, None, None)
    );
    assert_eq!(gui.rotation, Some([10.0, 20.0, 30.0]));
}

#[test]
fn a_transform_vector_on_the_edge_of_its_range_is_applied() {
    let edges = r#"{"gui": {"translation": [-80, 80, 0], "rotation": [-360, 360, 0], "scale": [0, 4, 2], "rotation_pivot": [-80, 80, 0], "scale_pivot": [80, -80, 0]}}"#;
    let document = loaded(load(&transforms("1.21.0", edges)));
    clean(document.diagnostics());
    for (member, vector) in [
        ("translation", "[-80.001, 0, 0]"),
        ("translation", "[0, 80.001, 0]"),
        ("rotation", "[-360.001, 0, 0]"),
        ("rotation", "[0, 0, 360.001]"),
        ("scale", "[-0.001, 1, 1]"),
        ("scale", "[1, 4.001, 1]"),
        ("rotation_pivot", "[0, 0, -80.001]"),
        ("scale_pivot", "[80.001, 0, 0]"),
    ] {
        let text = transforms("1.21.0", &format!(r#"{{"gui": {{"{member}": {vector}}}}}"#));
        let document = loaded(load(&text));
        assert_eq!(
            count(document.diagnostics(), "InvalidValue", Severity::Error),
            1,
            "{member} {vector}"
        );
    }
}

#[test]
fn g15_uv_rotation() {
    let face = |version: &str, rotation: &str| {
        cube(
            version,
            &format!(r#"{{"uv": {{"north": {{"uv": [0, 0], "uv_rotation": {rotation}}}}}}}"#),
        )
    };
    let north = |text: &str| {
        let document = loaded(load(text));
        let cube = document.value().geometries[0].bones()[0].cubes()[0].clone();
        let Some(CubeUv::Faces(faces)) = cube.uv else {
            panic!("not the face form");
        };
        (
            faces.north.unwrap().uv_rotation,
            document.diagnostics().clone(),
        )
    };
    for (rotation, shown) in [
        ("45", "45"),
        ("90.5", "90.5"),
        ("\"90\"", "\"90\""),
        ("-90", "-90"),
        ("360", "360"),
    ] {
        let (read, d) = north(&face("1.21.0", rotation));
        assert_eq!(read, None, "{rotation}");
        expect_at(
            &d,
            "InvalidValue",
            Severity::Error,
            "minecraft:geometry/0/bones/0/cubes/0/uv/north/uv_rotation",
            &format!("Invalid uv rotation value '{shown}', valid values are 0, 90, 180 and 270"),
        );
    }
    for turn in [0_i32, 90, 180, 270] {
        let (read, d) = north(&face("1.21.0", &turn.to_string()));
        assert_eq!(read, Some(turn));
        clean(&d);
    }
    // The integer the file has is compared, so a zero written with a sign is the zero it is
    // (the JSON library reads `-0` as the int 0).
    let (read, d) = north(&face("1.21.0", "-0"));
    assert_eq!(read, Some(0_i32));
    clean(&d);
    // Below 1.21 the member is undeclared: removed leniently, fatal from 1.19.30.
    let (read, d) = north(&face("1.16.0", "90"));
    assert_eq!(read, None);
    expect(
        &d,
        "UnrecognizedField",
        Severity::Info,
        "child 'uv_rotation' not valid here (removed; format_version < 1.19.30)",
    );
    fails_schema(
        load(&face("1.19.30", "90")),
        "UnrecognizedField",
        "minecraft:geometry/0/bones/0/cubes/0/uv/north/uv_rotation",
        "child 'uv_rotation' not valid here.",
    );
    fails_schema(
        load(&face("1.20.99", "90")),
        "UnrecognizedField",
        "minecraft:geometry/0/bones/0/cubes/0/uv/north/uv_rotation",
        "child 'uv_rotation' not valid here.",
    );
}

#[test]
fn g16_poly_mesh_counts() {
    const AT: &str = "Error InvalidValue minecraft:geometry/0/bones/0/poly_mesh";

    let mesh = |positions: usize, normals: usize, uvs: usize, polys: &str| {
        let list = |n: usize, item: &str| vec![item; n].join(", ");
        with_bone(
            "1.16.0",
            &format!(
                r#", "poly_mesh": {{"positions": [{}], "normals": [{}], "uvs": [{}], "polys": "{polys}"}}"#,
                list(positions, "[0, 0, 0]"),
                list(normals, "[0, 1, 0]"),
                list(uvs, "[0, 0]")
            ),
        )
    };
    let warnings = |text: &str| {
        let document = loaded(load(text));
        document
            .diagnostics()
            .at_least(Severity::Warning)
            .map(|f| {
                format!(
                    "{:?} {} {} {}",
                    f.severity,
                    f.code.name(),
                    f.location.json_path,
                    f.code
                )
            })
            .collect::<Vec<_>>()
    };
    // Rules 18 and 19 are the game's content-log errors at the mesh, kept.
    assert_eq!(
        warnings(&mesh(4, 4, 4, "tri_list")),
        [format!(
            "{AT} \"polys\" specifies \"tri_list\" but positions[] isn't a multiple of three?"
        )]
    );
    assert_eq!(
        warnings(&mesh(6, 6, 6, "quad_list")),
        [format!(
            "{AT} \"polys\" specifies \"quad_list\" but positions[] isn't a multiple of four?"
        )]
    );
    assert_eq!(
        warnings(&mesh(3, 2, 3, "tri_list")),
        [format!(
            "{AT} No 'polys' section, and the number of positions, normals, and UVs aren't equal (3, 2, 3 resp)"
        )]
    );
    assert_eq!(
        warnings(&mesh(4, 4, 3, "quad_list")),
        [format!(
            "{AT} No 'polys' section, and the number of positions, normals, and UVs aren't equal (4, 4, 3 resp)"
        )]
    );
    assert!(warnings(&mesh(3, 3, 3, "tri_list")).is_empty());
    assert!(warnings(&mesh(8, 8, 8, "quad_list")).is_empty());
    // An indexed mesh has no count rule.
    let indexed = with_bone(
        "1.16.0",
        r#", "poly_mesh": {"positions": [[0, 0, 0]], "polys": [[[0, 0, 0], [0, 0, 0], [0, 0, 0]]]}"#,
    );
    assert!(warnings(&indexed).is_empty());
}

/// an element of `positions` or `normals` has three
/// numbers and one of `uvs` two; another length fails the file in the lenient era as in the
/// strict one, at the element.
#[test]
fn a_poly_mesh_position_normal_or_uv_of_another_length_fails_the_file() {
    let mesh = |member: &str, element: &str| {
        let v = "[0, 0, 0]";
        format!(r#", "poly_mesh": {{"{member}": [{element}], "polys": [[{v}, {v}, {v}]]}}"#)
    };
    for (member, element, text) in [
        ("positions", "[0, 0]", "Array too small (2 < 3)"),
        ("positions", "[0, 0, 0, 0]", "Array too large (4 > 3)"),
        ("positions", "[]", "Array too small (0 < 3)"),
        ("normals", "[0, 1]", "Array too small (2 < 3)"),
        ("normals", "[0, 1, 0, 0]", "Array too large (4 > 3)"),
        ("uvs", "[0]", "Array too small (1 < 2)"),
        ("uvs", "[0, 0, 0]", "Array too large (3 > 2)"),
    ] {
        for version in ["1.8.0", "1.12.0", "1.16.0", "1.19.30", "1.21.0", "1.26.40"] {
            let (file, at) = if version == "1.8.0" {
                (
                    format!(
                        r#"{{"format_version": "1.8.0", "geometry.x": {{"bones": [{{"name": "body"{}}}]}}}}"#,
                        mesh(member, element)
                    ),
                    format!("geometry.x/bones/0/poly_mesh/{member}/0"),
                )
            } else {
                (
                    with_bone(version, &mesh(member, element)),
                    format!("minecraft:geometry/0/bones/0/poly_mesh/{member}/0"),
                )
            };
            fails_schema(load(&file), "InvalidValue", &at, text);
        }
    }
    // The lengths the schema has load.
    loaded(load(&with_bone(
        "1.16.0",
        r#", "poly_mesh": {"positions": [[0, 0, 0]], "normals": [[0, 1, 0]], "uvs": [[0, 0]], "polys": [[[0, 0, 0], [0, 0, 0], [0, 0, 0]]]}"#,
    )));
}

#[test]
fn a_polygon_has_three_or_four_vertices_of_three_indices() {
    let polys =
        |polys: &str| with_bone("1.16.0", &format!(r#", "poly_mesh": {{"polys": {polys}}}"#));
    let v = "[0, 1, 2]";
    loaded(load(&polys(&format!(
        "[[{v}, {v}, {v}], [{v}, {v}, {v}, {v}]]"
    ))));
    fails_schema(
        load(&polys(&format!("[[{v}, {v}]]"))),
        "InvalidValue",
        "minecraft:geometry/0/bones/0/poly_mesh/polys/0",
        "Array too small (2 < 3)",
    );
    fails_schema(
        load(&polys(&format!("[[{v}, {v}, {v}, {v}, {v}]]"))),
        "InvalidValue",
        "minecraft:geometry/0/bones/0/poly_mesh/polys/0",
        "Array too large (5 > 4)",
    );
    fails_schema(
        load(&polys(&format!("[[{v}, {v}, [0, 1]]]"))),
        "InvalidValue",
        "minecraft:geometry/0/bones/0/poly_mesh/polys/0/2",
        "Array too small (2 < 3)",
    );
}

#[test]
fn g17_bone_case_merge() {
    let text = r#"{"format_version": "1.16.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x"}, "bones": [
        {"name": "Head", "inflate": 1}, {"name": "Tail", "parent": "HEAD"}, {"name": "head", "inflate": 2}]}]}"#;
    let document = loaded(load(text));
    let d = document.diagnostics();
    expect_at(
        d,
        "DuplicateBone",
        Severity::Info,
        "minecraft:geometry/0/bones/2",
        "two bones collide after lower-casing",
    );
    // ours, an information: nothing reaches the warning channel
    assert_eq!(d.at_least(Severity::Warning).count(), 0);
    let geometry = &document.value().geometries[0];
    let names: Vec<&str> = geometry.bones().iter().map(|b| b.name.as_str()).collect();
    assert_eq!(
        names,
        ["head", "tail"],
        "one bone, at the first one's place"
    );
    assert_eq!(
        geometry.bone("HEAD").unwrap().inflate,
        Some(2.0),
        "the later one wins"
    );
    assert_eq!(
        geometry.bone("tail").unwrap().parent.as_deref(),
        Some("head")
    );
}

#[test]
fn a_bone_parent_that_is_no_bone_is_a_warning_and_a_cycle_is_an_information() {
    let bones = |bones: &str| {
        format!(
            r#"{{"format_version": "1.16.0", "minecraft:geometry": [{{"description": {{"identifier": "geometry.x"}}, "bones": [{bones}]}}]}}"#
        )
    };
    let document = loaded(load(&bones(
        r#"{"name": "a", "parent": "ghost"}, {"name": "b", "parent": "a"}, {"name": "c"}"#,
    )));
    let d = document.diagnostics();
    expect_at(
        d,
        "UnresolvedReference",
        Severity::Info,
        "minecraft:geometry/0/bones/0/parent",
        "bone 'a' has parent 'ghost', which is not a bone of geometry 'geometry.x'",
    );
    assert_eq!(d.len(), 1);
    // A bone that is its own parent has vanilla's text.
    let document = loaded(load(&bones(r#"{"name": "a", "parent": "A"}"#)));
    expect_at(
        document.diagnostics(),
        "ReferenceCycle",
        Severity::Info,
        "minecraft:geometry/0/bones/0/parent",
        "Error: parenting model part to itself!",
    );
    // A longer cycle: its members are reported, a bone hanging on it is not.
    let document = loaded(load(&bones(
        r#"{"name": "a", "parent": "c"}, {"name": "b", "parent": "a"}, {"name": "c", "parent": "b"},
           {"name": "leaf", "parent": "a"}, {"name": "root"}, {"name": "child", "parent": "root"}"#,
    )));
    let d = document.diagnostics();
    assert_eq!(count(d, "ReferenceCycle", Severity::Info), 3);
    assert_eq!(d.len(), 3);
    for index in 0..3 {
        assert!(d.iter().any(|f| f.location.json_path.to_string()
            == format!("minecraft:geometry/0/bones/{index}/parent")));
    }
}

#[test]
fn g18_locator_object_needs_rotation() {
    let locator = |version: &str, locator: &str| {
        with_bone(version, &format!(r#", "locators": {{"lead": {locator}}}"#))
    };
    for version in ["1.12.0", "1.26.40"] {
        fails_schema(
            load(&locator(version, r#"{"offset": [0, 0, 0]}"#)),
            "MissingRequiredField",
            "minecraft:geometry/0/bones/0/locators/lead",
            "Required child rotation not found",
        );
    }
    let geometry = first(&locator(
        "1.12.0",
        r#"{"offset": [0, 1, 2], "rotation": [3, 4, 5], "ignore_inherited_scale": true}"#,
    ));
    assert_eq!(
        geometry.bones()[0].locators.as_ref().unwrap()["lead"],
        Locator::Placed {
            offset: [0.0, 1.0, 2.0],
            rotation: Some([3.0, 4.0, 5.0]),
            ignore_inherited_scale: Some(true)
        }
    );
    // A name outside `[a-zA-Z0-9_.-]+` is no declared member.
    fails_schema(
        load(&with_bone("1.21.0", r#", "locators": {"a b": [0, 0, 0]}"#)),
        "UnrecognizedField",
        "minecraft:geometry/0/bones/0/locators/a b",
        "child 'a b' not valid here.",
    );
}

#[test]
fn g19_strict_from_11930() {
    let bone = r#", "neverRender": true"#;
    fails_schema(
        load(&with_bone("1.19.30", bone)),
        "UnrecognizedField",
        "minecraft:geometry/0/bones/0/neverRender",
        "child 'neverRender' not valid here.",
    );
    let document = loaded(load(&with_bone("1.19.20", bone)));
    expect(
        document.diagnostics(),
        "UnrecognizedField",
        Severity::Info,
        "child 'neverRender' not valid here (removed; format_version < 1.19.30)",
    );
    loaded(load(&with_bone("1.19.29", bone)));
}

#[test]
fn g20_empty_and_null() {
    for text in ["", "   \n", "null", "\u{feff}", " null "] {
        let d = rejected(load(text));
        assert_eq!(d.len(), 1, "{text:?}");
        let finding = d.iter().next().unwrap();
        assert_eq!(finding.code.name(), "EmptyDocument");
        assert_eq!(
            finding.severity,
            Severity::Info,
            "skipped, which is no failure"
        );
    }
}

#[test]
fn g21_unknown_transform_key() {
    fails_schema(
        load(&transforms("1.21.0", r#"{"gui2": {}}"#)),
        "UnrecognizedField",
        "minecraft:geometry/0/item_display_transforms/gui2",
        "child 'gui2' not valid here.",
    );
    // A game older than 1.26.40 does not know `shelf` at all.
    fails_schema(
        load_at(
            "models/entity/x.json",
            &transforms("1.21.0", r#"{"shelf": {}}"#),
            &engine(1, 26, 20),
        ),
        "UnrecognizedField",
        "minecraft:geometry/0/item_display_transforms/shelf",
        "child 'shelf' not valid here.",
    );
    fails_schema(
        load_at(
            "models/entity/x.json",
            &transforms("1.21.0", r#"{"shelf": {}}"#),
            &engine(1, 26, 39),
        ),
        "UnrecognizedField",
        "minecraft:geometry/0/item_display_transforms/shelf",
        "child 'shelf' not valid here.",
    );
    // `item_display_transforms` itself is undeclared below 1.21.
    fails_schema(
        load(&transforms("1.20.80", "{}")),
        "UnrecognizedField",
        "minecraft:geometry/0/item_display_transforms",
        "child 'item_display_transforms' not valid here.",
    );
    let document = loaded(load(&transforms("1.16.0", "{}")));
    assert!(
        document.value().geometries[0]
            .item_display_transforms
            .is_none()
    );
    // A transform that is not an object fails its type.
    fails_schema(
        load(&transforms("1.21.0", r#"{"gui": null}"#)),
        "InvalidType",
        "minecraft:geometry/0/item_display_transforms/gui",
        "unknown child schema option type.  Allowed types: object",
    );
    fails_schema(
        load(&transforms("1.21.0", "[]")),
        "InvalidType",
        "minecraft:geometry/0/item_display_transforms",
        "unknown child schema option type.  Allowed types: object",
    );
}

#[test]
fn g21b_shelf_before_12640_engine_12645() {
    for engine_version in [engine(1, 26, 40), engine(1, 26, 45), rp()] {
        let d = rejected(load_at(
            "models/entity/x.json",
            &transforms("1.21.0", r#"{"shelf": {"scale": [1, 1, 1]}, "gui": {}}"#),
            &engine_version,
        ));
        expect_at(
            &d,
            "InvalidValue",
            Severity::Error,
            "minecraft:geometry/0/item_display_transforms/shelf",
            "\"item_display_transform\" for \"geometry.x\" is invalid - \"shelf\" can only be used in a file version 1.26.40 or higher",
        );
        assert_eq!(d.len(), 1, "the step's message alone");
    }
    rejected(load(&transforms("1.26.39", r#"{"shelf": {}}"#)));
    loaded(load(&transforms("1.26.40", r#"{"shelf": {}}"#)));
}

#[test]
fn g22_fit_to_frame_outside_gui() {
    for name in [
        "fixed",
        "head",
        "ground",
        "shelf",
        "embedded",
        "firstperson_righthand",
    ] {
        fails_schema(
            load(&transforms(
                "1.26.40",
                &format!(r#"{{"{name}": {{"fit_to_frame": false}}}}"#),
            )),
            "UnrecognizedField",
            &format!("minecraft:geometry/0/item_display_transforms/{name}/fit_to_frame"),
            "child 'fit_to_frame' not valid here.",
        );
    }
    // the fixtures g22 as written: 1.21.130, `fixed`. Every type the 1.21.130 schema declares
    // refuses the member there too (`shelf` is not a type of a file below 1.26.40).
    for name in [
        "fixed",
        "head",
        "ground",
        "embedded",
        "firstperson_righthand",
    ] {
        fails_schema(
            load(&transforms(
                "1.21.130",
                &format!(r#"{{"{name}": {{"fit_to_frame": false}}}}"#),
            )),
            "UnrecognizedField",
            &format!("minecraft:geometry/0/item_display_transforms/{name}/fit_to_frame"),
            "child 'fit_to_frame' not valid here.",
        );
    }
    loaded(load(&transforms(
        "1.21.130",
        r#"{"gui": {"fit_to_frame": false}}"#,
    )));
}

#[test]
fn g23_locator_any_length_deviation() {
    // Vanilla declares the locator arrays without a size; ours: reported, locator left out.
    let document = loaded(load(&with_bone(
        "1.16.0",
        r#", "locators": {"short": [0, 1], "long": {"offset": [0, 1, 2, 3], "rotation": [0]}, "bad_rotation": {"offset": [0, 1, 2], "rotation": [0]}, "ok": [0, 1, 2]}"#,
    )));
    let d = document.diagnostics();
    assert!(!d.has_errors());
    for at in [
        "short",
        "long/offset",
        "long/rotation",
        "bad_rotation/rotation",
    ] {
        expect_at(
            d,
            "InvalidValue",
            Severity::Info,
            &format!("minecraft:geometry/0/bones/0/locators/{at}"),
            "invalid value",
        );
    }
    assert_eq!(count(d, "InvalidValue", Severity::Info), 4);
    let locators = document.value().geometries[0].bones()[0]
        .locators
        .clone()
        .unwrap();
    assert_eq!(locators.len(), 1);
    assert_eq!(locators["ok"], Locator::Position([0.0, 1.0, 2.0]));
    // Strict mode: still no failure of the file.
    let document = loaded(load(&with_bone(
        "1.21.0",
        r#", "locators": {"long": {"offset": [0, 1, 2, 3], "rotation": [0, 0, 0]}}"#,
    )));
    assert!(!document.diagnostics().has_errors());
}

#[test]
fn g24_v18_version_ceiling() {
    let file = |version: &str| format!(r#"{{"format_version": "{version}", "geometry.x": {{}}}}"#);
    fails_schema(
        load(&file("1.11.1")),
        "UnsupportedVersion",
        "format_version",
        "Version 1.11.1 greater than max allowed version (1.11.0)",
    );
    fails_schema(
        load(&file("1.11.99")),
        "UnsupportedVersion",
        "format_version",
        "Version 1.11.99 greater than max allowed version (1.11.0)",
    );
    for version in ["1.11.0", "1.10.0", "1.8.0"] {
        clean(loaded(load(&file(version))).diagnostics());
    }
}

#[test]
fn g25_render_group_id_negative() {
    for version in ["1.16.0", "1.21.0"] {
        fails_schema(
            load(&with_bone(version, r#", "render_group_id": -1"#)),
            "InvalidValue",
            "minecraft:geometry/0/bones/0/render_group_id",
            "Value '-1' outside valid range [0, 2147483647]",
        );
    }
    fails_schema(
        load(
            r#"{"format_version": "1.8.0", "geometry.x": {"bones": [{"name": "a", "render_group_id": -7}]}}"#,
        ),
        "InvalidValue",
        "geometry.x/bones/0/render_group_id",
        "Value '-7' outside valid range [0, 2147483647]",
    );
    fails_schema(
        load(r#"{"geometry.x": {"bones": [{"name": "a", "render_group_id": -1}]}}"#),
        "InvalidValue",
        "geometry.x/bones/0/render_group_id",
        "Value '-1' outside valid range [0, 2147483647]",
    );
    let geometry = first(&with_bone("1.16.0", r#", "render_group_id": 0"#));
    assert_eq!(geometry.bones()[0].render_group_id, Some(0));
    let geometry = first(&with_bone("1.16.0", r#", "render_group_id": 2147483647"#));
    assert_eq!(geometry.bones()[0].render_group_id, Some(2_147_483_647));
}

#[test]
fn g26_geometry_array_empty() {
    for version in ["1.12.0", "1.21.0"] {
        let text = format!(r#"{{"format_version": "{version}", "minecraft:geometry": []}}"#);
        let result = load(&text);
        let d = rejected(result);
        expect_at(
            &d,
            "InvalidValue",
            Severity::Error,
            "minecraft:geometry",
            "Array too small (0 < 1)",
        );
        expect(&d, "InvalidValue", Severity::Error, DID_NOT_VALIDATE);
    }
}

#[test]
fn g27_polys_string_value() {
    fails_schema(
        load(&with_bone(
            "1.16.0",
            r#", "poly_mesh": {"polys": "fan_list"}"#,
        )),
        "InvalidValue",
        "minecraft:geometry/0/bones/0/poly_mesh/polys",
        "Value 'fan_list' not allowed.  Must match at least one of the following regex patterns: tri_list, quad_list",
    );
    // The valid values are searched in the string.
    loaded(load(&with_bone(
        "1.16.0",
        r#", "poly_mesh": {"polys": "my_tri_list"}"#,
    )));
    loaded(load(&with_bone(
        "1.16.0",
        r#", "poly_mesh": {"polys": "quad_list"}"#,
    )));
    fails_schema(
        load(&with_bone("1.21.0", r#", "poly_mesh": {"polys": 5}"#)),
        "InvalidType",
        "minecraft:geometry/0/bones/0/poly_mesh/polys",
        "unknown child schema option type.  Allowed types: array, string",
    );
}

/// A file below 1.12 has its geometries as members of the root, and what
/// `Geometry::parse` makes of a member is told at the member as the file has it.
#[test]
fn a_coercion_of_a_keyed_file_is_told_at_the_member_of_the_file() {
    let text = r#"{"format_version": "1.8.0", "geometry.x:geometry.p": {"visible_bounds_width": 80,
        "bones": [{"name": "body", "cubes": [{"origin": [0, 0, 0], "size": [1, 1, 1], "uv": [2, 3.5]}]}]},
        "geometry.a": {"visible_bounds_height": 0, "bones": [{"name": "b"}, {"name": "B"}]}}"#;
    let document = loaded(load(text));
    let d = document.diagnostics();
    expect_at(
        d,
        "InvalidValue",
        Severity::Info,
        "geometry.x:geometry.p/bones/0/cubes/0/uv/1",
        "box uv 3.5 is read as the whole number 3",
    );
    expect_at(
        d,
        "ClampedValue",
        Severity::Info,
        "geometry.x:geometry.p/visible_bounds_width",
        "visible_bounds_width 80.000000 clamped to 50.000000",
    );
    expect_at(
        d,
        "ClampedValue",
        Severity::Info,
        "geometry.a/visible_bounds_height",
        "visible_bounds_height 0.000000 clamped to 0.100000",
    );
    assert!(
        d.iter()
            .any(|f| f.location.json_path.to_string() == "geometry.a/bones/1"),
        "the second bone of a name is told where the file has it: {d:?}"
    );
    // The bone it replaces is noted where the file has it.
    expect_at(
        d,
        "DroppedField",
        Severity::Info,
        "geometry.a/bones/0",
        "field dropped",
    );
    // And the name `B` it is one bone with `b` by.
    expect_at(
        d,
        "InvalidValue",
        Severity::Info,
        "geometry.a/bones/1/name",
        "bone name 'B' is read as 'b'",
    );
    assert_eq!(d.len(), 6);
    let x = document
        .value()
        .geometries
        .iter()
        .find(|g| g.description.identifier == "geometry.x")
        .expect("geometry.x");
    assert_eq!(x.description.visible_bounds_width, Some(50.0));
    assert_eq!(x.bones()[0].cubes()[0].uv, Some(CubeUv::Box([2.0, 3.0])));
}

#[test]
fn g28_parse_coercions() {
    let text = r#"{"format_version": "1.21.0", "minecraft:geometry": [{
        "description": {"identifier": "geometry.x", "visible_bounds_width": 0, "visible_bounds_height": 80},
        "bones": [{"name": "body", "cubes": [{"uv": [0.5, 3.7]}, {"uv": [-1.5, 2]}, {"uv": {"north": {"uv": [0.5, 3.7]}}}]}]}]}"#;
    let document = loaded(load(text));
    let d = document.diagnostics();
    expect_at(
        d,
        "ClampedValue",
        Severity::Info,
        "minecraft:geometry/0/description/visible_bounds_width",
        "visible_bounds_width 0.000000 clamped to 0.100000",
    );
    expect_at(
        d,
        "ClampedValue",
        Severity::Info,
        "minecraft:geometry/0/description/visible_bounds_height",
        "visible_bounds_height 80.000000 clamped to 50.000000",
    );
    // A box `uv` component with a fraction is told where it stands.
    for (at, message) in [
        ("0/uv/0", "box uv 0.5 is read as the whole number 0"),
        ("0/uv/1", "box uv 3.7 is read as the whole number 3"),
        ("1/uv/0", "box uv -1.5 is read as the whole number -1"),
    ] {
        expect_at(
            d,
            "InvalidValue",
            Severity::Info,
            &format!("minecraft:geometry/0/bones/0/cubes/{at}"),
            message,
        );
    }
    assert_eq!(d.len(), 5);
    let geometry = &document.value().geometries[0];
    assert_eq!(geometry.description.visible_bounds_width, Some(0.1));
    assert_eq!(geometry.description.visible_bounds_height, Some(50.0));
    let cubes = geometry.bones()[0].cubes();
    assert_eq!(cubes[0].uv, Some(CubeUv::Box([0.0, 3.0])));
    assert_eq!(cubes[1].uv, Some(CubeUv::Box([-1.0, 2.0])), "towards zero");
    let Some(CubeUv::Faces(faces)) = &cubes[2].uv else {
        panic!("not the face form");
    };
    assert_eq!(
        faces.north.as_ref().unwrap().uv,
        [0.5, 3.7],
        "a face is read as floats"
    );
}

#[test]
fn bounds_inside_the_range_are_kept_and_absent_ones_default_to_one() {
    let bounds = |members: &str| {
        let text = r#"{"format_version": "1.12.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x"MEMBERS}}]}"#
            .replace("MEMBERS", members);
        let document = loaded(load(&text));
        (
            document.value().geometries[0].description.clone(),
            document.diagnostics().len(),
        )
    };
    let (d, findings) = bounds(r#", "visible_bounds_width": 0.1, "visible_bounds_height": 50"#);
    assert_eq!(
        (d.visible_bounds_width, d.visible_bounds_height),
        (Some(0.1), Some(50.0))
    );
    assert_eq!(findings, 0);
    let (d, findings) = bounds(r#", "visible_bounds_width": 0.09, "visible_bounds_height": 50.5"#);
    assert_eq!(
        (d.visible_bounds_width, d.visible_bounds_height),
        (Some(0.1), Some(50.0))
    );
    assert_eq!(findings, 2);
    let (d, findings) = bounds("");
    assert_eq!(findings, 0);
    assert_eq!(
        (d.visible_bounds_width, d.visible_bounds_height),
        (None, None)
    );
    assert_eq!(
        (d.visible_bounds_width(), d.visible_bounds_height()),
        (1.0, 1.0)
    );
    assert_eq!(d.visible_bounds_offset(), [0.0; 3]);
    assert_eq!((d.texture_width(), d.texture_height()), (64, 64));
    let (d, _) = bounds(
        r#", "texture_width": 128, "texture_height": 32, "visible_bounds_offset": [0, 1, 0]"#,
    );
    assert_eq!((d.texture_width(), d.texture_height()), (128, 32));
    assert_eq!(d.visible_bounds_offset(), [0.0, 1.0, 0.0]);
}

#[test]
fn g29_texture_mesh_defaults() {
    let geometry = first(&with_bone(
        "1.16.0",
        r#", "texture_meshes": [{"texture": "t"}, {"texture": "u", "position": [1, 2, 3], "local_pivot": [4, 5, 6], "rotation": [7, 8, 9], "scale": [2, 2, 2], "use_pixel_depth": false}]"#,
    ));
    let meshes = geometry.bones()[0].texture_meshes.clone().unwrap();
    let t = &meshes[0];
    assert_eq!(t.texture, "t");
    assert_eq!((t.position, t.scale, t.use_pixel_depth), (None, None, None));
    assert_eq!(t.position(), [0.0; 3]);
    assert_eq!(t.local_pivot(), [0.0; 3]);
    assert_eq!(t.rotation(), [0.0; 3]);
    assert_eq!(t.scale(), [1.0; 3]);
    assert!(t.use_pixel_depth());
    let u = &meshes[1];
    assert_eq!(u.position(), [1.0, 2.0, 3.0]);
    assert_eq!(u.local_pivot(), [4.0, 5.0, 6.0]);
    assert_eq!(u.rotation(), [7.0, 8.0, 9.0]);
    assert_eq!(u.scale(), [2.0; 3]);
    assert!(!u.use_pixel_depth());
}

#[test]
fn a_cube_takes_inflate_and_mirror_from_its_bone() {
    let geometry = first(&with_bone(
        "1.16.0",
        r#", "inflate": 0.5, "mirror": true, "cubes": [{"size": [1, 1, 1]}, {"size": [1, 1, 1], "inflate": 2, "mirror": false}]"#,
    ));
    let bone = &geometry.bones()[0];
    let cubes = bone.cubes();
    assert_eq!((cubes[0].inflate(bone), cubes[0].mirror(bone)), (0.5, true));
    assert_eq!(
        (cubes[1].inflate(bone), cubes[1].mirror(bone)),
        (2.0, false)
    );
    let plain = first(&with_bone("1.16.0", r#", "cubes": [{}]"#));
    let bone = &plain.bones()[0];
    assert_eq!((bone.inflate(), bone.mirror()), (0.0, false));
    assert_eq!(bone.cubes()[0].pivot(), [0.0; 3]);
}

// ---- the version member ---------------------------------------------------------------------

#[test]
fn a_missing_format_version_is_an_information_and_the_file_is_1_0_0_deviation() {
    let document = loaded(load(r#"{"geometry.x": {}}"#));
    let d = document.diagnostics();
    assert_eq!(d.len(), 1);
    expect(
        d,
        "MissingFormatVersion",
        Severity::Info,
        "no format_version",
    );
    assert_eq!(document.version().written, None);
    assert_eq!(document.version().effective, FormatVersion::new(1, 0, 0));
    assert_eq!(document.version().parsed_as, FormatVersion::new(0, 0, 0));
}

#[test]
fn an_unreadable_format_version_is_a_warning_and_the_file_is_1_0_0_deviation() {
    for (written, shown) in [
        ("\"beta\"", "beta"),
        ("\"one\"", "one"),
        ("5", "5"),
        ("[1, 8, 0]", "[1, 8, 0]"),
        ("\"*\"", "*"),
    ] {
        let text = format!(r#"{{"format_version": {written}, "geometry.x": {{}}}}"#);
        let document = loaded(load(&text));
        let d = document.diagnostics();
        assert_eq!(d.len(), 1, "{written}");
        expect_at(
            d,
            "MalformedFormatVersion",
            Severity::Info,
            "format_version",
            &format!("format_version '{shown}' is not a valid version; treated as 1.0.0"),
        );
        assert_eq!(document.version().effective, FormatVersion::new(1, 0, 0));
    }
    // The file is then of the oldest shape: a modern payload does not validate.
    let d = rejected(load(
        r#"{"format_version": "beta", "minecraft:geometry": [{"description": {"identifier": "geometry.x"}}]}"#,
    ));
    expect(
        &d,
        "MissingRequiredField",
        Severity::Error,
        r"Required child geometry\.[a-zA-Z0-9_.'-:]+ not found",
    );
}

#[test]
fn a_legacy_key_outside_the_geometry_pattern_is_an_unknown_member() {
    let document = loaded(load(
        r#"{"format_version": "1.8.0", "geometry.x": {}, "geometry.": {}, "model.pig": {}, "debug": true}"#,
    ));
    assert_eq!(document.value().geometries.len(), 1);
    assert_eq!(document.value().debug, Some(true));
    assert_eq!(
        count(document.diagnostics(), "UnrecognizedField", Severity::Info),
        2
    );
}

#[test]
fn a_binding_of_the_wrong_json_type_fails_its_schema() {
    fails_schema(
        load(&with_bone("1.21.0", r#", "binding": {}"#)),
        "InvalidType",
        "minecraft:geometry/0/bones/0/binding",
        "unknown child schema option type.  Allowed types: ExpressionNode",
    );
    fails_schema(
        load(&with_bone("1.21.0", r#", "binding": null"#)),
        "InvalidType",
        "minecraft:geometry/0/bones/0/binding",
        "unknown child schema option type.  Allowed types: ExpressionNode",
    );
    let _ = DID_NOT_VALIDATE;
}

// ---- extra pins -------------------------------------------------------------------

/// The required members are looked for once every member of the object validated: an object
/// that already failed reports its failure alone.
#[test]
fn a_missing_required_member_is_not_reported_beside_another_failure_of_its_object() {
    let text = r#"{"format_version": "1.21.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x"}, "bones": [{"neverRender": true}]}]}"#;
    let d = rejected(load(text));
    assert_eq!(count(&d, "UnrecognizedField", Severity::Error), 1);
    assert_eq!(count(&d, "MissingRequiredField", Severity::Error), 0);
}

#[test]
fn an_int_member_with_a_fraction_or_beyond_an_i32_is_a_wrong_type() {
    let size = |value: &str| {
        format!(
            r#"{{"format_version": "1.21.0", "minecraft:geometry": [{{"description": {{"identifier": "geometry.x", "texture_width": {value}}}}}]}}"#
        )
    };
    for value in ["64.5", "2147483648", "-2147483649", "\"64\""] {
        fails_schema(
            load(&size(value)),
            "InvalidType",
            "minecraft:geometry/0/description/texture_width",
            "unknown child schema option type.  Allowed types: int",
        );
    }
    for (value, read) in [
        ("64.0", 64),
        ("2147483647", i32::MAX),
        ("-2147483648", i32::MIN),
    ] {
        assert_eq!(first(&size(value)).description.texture_width, Some(read));
    }
}

#[test]
fn the_last_geometry_of_an_identifier_is_the_one_a_file_gives() {
    let text = r#"{"format_version": "1.16.0", "minecraft:geometry": [
        {"description": {"identifier": "geometry.x", "texture_width": 1}},
        {"description": {"identifier": "geometry.y"}},
        {"description": {"identifier": "geometry.x", "texture_width": 2}}]}"#;
    let document = loaded(load(text));
    let file = document.value();
    assert_eq!(file.geometries.len(), 3, "the file keeps what was written");
    assert_eq!(
        file.geometry("geometry.X")
            .unwrap()
            .description
            .texture_width,
        Some(2)
    );
}

/// Of a member written twice the last one stands,
/// whole and at its own place, at the root, in a bone and in a locator map.
#[test]
fn of_a_member_written_twice_the_last_one_stands_at_its_place() {
    let text = r#"{"format_version": "1.21.0",
        "minecraft:geometry": [{"description": {"identifier": "geometry.first"}, "nope": 1}],
        "minecraft:geometry": [{"description": {"identifier": "geometry.last"}, "bones": [
            {"name": "first", "name": "last", "locators": {"a": [1, 1, 1], "b": [2, 2, 2], "a": [9, 9, 9]}}]}]}"#;
    let document = loaded(load(text));
    assert!(
        document.diagnostics().is_empty(),
        "the first copies are not read"
    );
    let file = document.value();
    assert_eq!(file.geometries.len(), 1);
    assert_eq!(file.geometries[0].description.identifier, "geometry.last");
    let bone = &file.geometries[0].bones()[0];
    assert_eq!(bone.name, "last");
    let locators = bone.locators.as_ref().unwrap();
    let order: Vec<&str> = locators.keys().map(String::as_str).collect();
    assert_eq!(
        order,
        ["b", "a"],
        "the repeated locator is where its last copy is"
    );
    assert_eq!(locators["a"], Locator::Position([9.0, 9.0, 9.0]));
    // The legacy shape too: a geometry key written twice.
    let legacy = loaded(load(
        r#"{"format_version": "1.8.0", "geometry.x": {"texturewidth": 1, "bones": [{}]}, "geometry.x": {"texturewidth": 2}}"#,
    ));
    assert_eq!(
        legacy.value().geometries[0].description.texture_width,
        Some(2)
    );
}

// Vanilla's persona piece geometry says `format_version` "1.12.0" and keys its
// models at the root. The file reader refuses it, as
// the game's file schema would; `load_keyed_file` reads it by the keyed layout's rules and
// keeps the version the file states.
#[test]
fn a_keyed_file_of_1_12_0_is_read_by_the_keyed_reader_with_its_version() {
    use bedrock_addon::families::geometry::{load_file, load_keyed_file};
    use bedrock_addon::storage::PackPath;

    const KEYED: &str = r#"{ "format_version": "1.12.0", "geometry.hat": { "texturewidth": 64, "bones": [ { "name": "waist" }, { "name": "head", "parent": "waist", "poly_mesh": { "normalized_uvs": true, "positions": [[0, 0, 0]], "normals": [[0, 1, 0]], "uvs": [[0, 0]], "polys": [[[0, 0, 0], [0, 0, 0], [0, 0, 0]]] } } ] } }"#;
    let path = PackPath::from_static("pieces/hat/hat.geometry.json");
    let d = rejected(load_file(&path, KEYED.as_bytes(), &rp()));
    assert_eq!(
        count(&d, "MissingRequiredField", Severity::Error),
        1,
        "{d:?}"
    );
    let document = loaded(load_keyed_file(&path, KEYED.as_bytes(), &rp()));
    let file = document.value();
    assert_eq!(file.format_version, FormatVersion::new(1, 12, 0));
    assert_eq!(
        document.version().written,
        Some(FormatVersion::new(1, 12, 0))
    );
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 8, 0));
    assert_eq!(file.geometries.len(), 1);
    assert_eq!(
        file.geometries[0].description.identifier.as_str(),
        "geometry.hat"
    );
    let bone = &file.geometries[0].bones.as_ref().unwrap()[1];
    assert_eq!(bone.parent.as_deref(), Some("waist"));
    assert!(bone.poly_mesh.is_some());
    assert_eq!(
        document.diagnostics().at_least(Severity::Warning).count(),
        0,
        "{:?}",
        document.diagnostics()
    );
    // A keyed file below 1.12.0 reads the same through both doors.
    let older = KEYED.replace("1.12.0", "1.8.0");
    let through_file = loaded(load_file(&path, older.as_bytes(), &rp()));
    let through_keyed = loaded(load_keyed_file(&path, older.as_bytes(), &rp()));
    assert_eq!(through_file.value(), through_keyed.value());
    // The engine's ceiling holds for the version the file states.
    let d = rejected(load_keyed_file(&path, KEYED.as_bytes(), &engine(1, 11, 0)));
    assert_eq!(count(&d, "UnsupportedVersion", Severity::Error), 1, "{d:?}");
    // A modern file has no keyed model: its `minecraft:geometry` is no model of this door.
    let modern = r#"{ "format_version": "1.12.0", "minecraft:geometry": [ { "description": { "identifier": "geometry.x" } } ] }"#;
    assert!(load_keyed_file(&path, modern.as_bytes(), &rp()).is_err());
}

/// W1: the members of an object are read in the file's order: the findings follow it.
#[test]
fn members_are_read_in_document_order() {
    let d = rejected(load(&with_bone(
        "1.19.30",
        r#", "pivot": "x", "inflate": "y""#,
    )));
    let told: Vec<_> = d
        .iter()
        .filter(|f| f.code.name() == "InvalidType")
        .map(|f| f.location.json_path.to_string())
        .collect();
    assert_eq!(
        told,
        [
            "minecraft:geometry/0/bones/0/pivot",
            "minecraft:geometry/0/bones/0/inflate"
        ],
        "{:?}",
        crate::common::lines(&d)
    );
}

/// W3: a number a float does not hold is read with a `LossyConversion` information at the
/// number.
#[test]
fn a_float_that_narrows_is_told_as_lossy() {
    let document = loaded(load(&with_bone(
        "1.19.30",
        r#", "inflate": 0.123456789012"#,
    )));
    let told: Vec<_> = document
        .diagnostics()
        .iter()
        .filter(|f| f.code.name() == "LossyConversion")
        .map(|f| (f.severity, f.location.json_path.to_string()))
        .collect();
    assert_eq!(
        told,
        [(
            Severity::Info,
            "minecraft:geometry/0/bones/0/inflate".to_owned()
        )],
        "{:?}",
        crate::common::lines(document.diagnostics())
    );
}
