//! Writing a material file and a list back: what was read is
//! written, nothing else, and what the loader would read differently is refused.

use bedrock_addon::document::{Content, Extra};
use bedrock_addon::families::material::{
    KeyRules, ListEntry, Listed, MaterialDef, MaterialFile, MaterialFileForm, MaterialKey,
    MaterialList, MaterialListKind, SamplerState, StencilFace, StencilRefOverride, Variant,
    VertexField,
};
use bedrock_addon::json::{EncodeError, Unrepresentable, Value};
use bedrock_addon::version::SemVersion;
use indexmap::IndexMap;

use crate::common::{
    Rewrites, assert_written_as_authored, canonical, file, fixture_text, flat, parse, parse_at,
    parse_list, rendered, rewrites, wrapped,
};

/// Reads `text`, writes it, reads the result and requires the same value, the same findings
/// and an identical second writing. Returns the written text.
#[track_caller]
fn round_trip_at(path: &str, text: &str) -> String {
    let first = parse_at(path, text).expect("loads");
    let written = first.to_json().expect("writes");
    let second = parse_at(path, &written).unwrap_or_else(|rejected| {
        panic!(
            "the written text is refused: {:#?}\n{written}",
            rejected.diagnostics
        )
    });
    assert_eq!(*first, *second, "{written}");
    // The same findings; the writer's member order may put them in another order.
    let sorted = |document: &bedrock_addon::document::Document<MaterialFile>| {
        let mut findings: Vec<String> = rendered(document.diagnostics().iter())
            .into_iter()
            .map(|finding| format!("{finding:?}"))
            .collect();
        findings.sort();
        findings
    };
    assert_eq!(sorted(&first), sorted(&second), "{written}");
    assert_eq!(second.to_json().expect("writes again"), written);
    written
}

#[track_caller]
fn round_trip(text: &str) -> String {
    round_trip_at("materials/test.material", text)
}

/// The path of the refusal of writing `file`.
#[track_caller]
fn refused(file: &MaterialFile) -> String {
    match file.to_json() {
        Err(EncodeError::NotRepresentable { at, .. }) => at.to_string(),
        other => panic!("not refused: {other:?}"),
    }
}

fn flat_file(materials: Vec<(&str, MaterialDef)>) -> MaterialFile {
    MaterialFile {
        form: MaterialFileForm::Flat,
        materials: materials
            .into_iter()
            .map(|(key, def)| (MaterialKey::parse(key), def))
            .collect(),
        extra: Extra::default(),
    }
}

fn wrapped_file(version: &str, materials: Vec<(&str, MaterialDef)>) -> MaterialFile {
    MaterialFile {
        form: MaterialFileForm::Wrapped {
            version: SemVersion::parse(version).expect("a version"),
        },
        ..flat_file(materials)
    }
}

// The vanilla files are gdk-retail 1.26.45 `resource_packs/{vanilla,chemistry}/materials/`,
// copied unchanged.
#[test]
fn the_vanilla_files_round_trip() {
    for name in [
        "vanilla/sky.material",
        "vanilla/entity.material",
        "vanilla/cameraFacingSprite.material",
        "vanilla/chemistry/entity.material",
    ] {
        // Written back as authored: the written text is the file's own JSON value.
        let text = fixture_text(name);
        assert_written_as_authored(&text, &round_trip(&text));
    }
}

#[test]
fn the_vanilla_lists_round_trip() {
    for (name, path) in [
        ("vanilla/common.json", "materials/common.json"),
        ("vanilla/fancy.json", "materials/fancy.json"),
        ("vanilla/sad.json", "materials/sad.json"),
    ] {
        let text = fixture_text(name);
        let first = parse_list(path, &text).expect("a list");
        let written = first.to_json().expect("writes");
        assert_written_as_authored(&text, &written);
        let second = parse_list(path, &written).expect("reads back");
        assert_eq!(*first, *second);
        assert_eq!(rendered(second.diagnostics().iter()), []);
        assert_eq!(second.to_json().unwrap(), written);
    }
}

#[test]
fn a_wrapped_file_is_written_in_its_wrapper_with_the_version_first() {
    let written = round_trip(&wrapped(r#""b": {}, "a:b": {"vertexShader": "s"}"#));
    assert_eq!(
        written,
        "{\n  \"materials\": {\n    \"version\": \"1.0.0\",\n    \"b\": {},\n    \"a:b\": {\n      \"vertexShader\": \"s\"\n    }\n  }\n}\n"
    );
    // The version as written, build metadata included.
    let build = round_trip(r#"{"materials": {"version": "1.0.0+build.7", "a": {}}}"#);
    assert!(build.contains("\"version\": \"1.0.0+build.7\""), "{build}");
    // A wrapper with no material keeps its version.
    let empty = round_trip(r#"{"materials": {"version": "1.0.0"}}"#);
    assert!(empty.contains("\"version\": \"1.0.0\""), "{empty}");
}

#[test]
fn a_flat_file_and_a_single_material_are_written_without_a_wrapper() {
    assert_eq!(
        round_trip(&flat(r#""+x:p": {"defines": ["A"]}"#)),
        "{\n  \"+x:p\": {\n    \"defines\": [\n      \"A\"\n    ]\n  }\n}\n"
    );
    assert_eq!(round_trip("{}"), "{}\n");
    let single = round_trip_at(
        "materials/foo.material",
        r#"{"vertexShader": "shaders/entity.vertex", "custom": 1}"#,
    );
    assert_eq!(
        single,
        "{\n  \"vertexShader\": \"shaders/entity.vertex\",\n  \"custom\": 1\n}\n"
    );
}

#[test]
fn every_member_is_written_back_and_none_is_invented() {
    let members = r#""states": ["Blending"], "+states": ["Wireframe"], "-states": ["DisableCulling"],
        "defines": ["A"], "+defines": ["B 4"], "-defines": [],
        "samplerStates": [{"samplerIndex": 0, "textureFilter": "PCF", "textureWrap": "Clamp",
                           "shaderStage": "Pixel", "comparisonFunc": "Less", "note": 1}, {}],
        "+samplerStates": [], "-samplerStates": [{"samplerIndex": 9}],
        "vertexShader": "v", "fragmentShader": "f", "geometryShader": "g",
        "depthFunc": "Less",
        "frontFace": {"stencilFunc": "Always", "stencilFailOp": "Keep",
                      "stencilDepthFailOp": "Replace", "stencilPassOp": "Keep", "stencilPass": "Keep"},
        "backFace": {},
        "stencilRefOverride": true, "stencilRef": 300, "stencilReadMask": 0,
        "stencilWriteMask": 4294967295,
        "blendSrc": "One", "blendDst": "Zero", "alphaSrc": "One", "alphaDst": "Zero",
        "vertexFields": [{"field": "UV0"}, {}, {"field": "Color", "note": null}],
        "depthBias": 1.5, "slopeScaledDepthBias": -3,
        "msaaSupport": "MSAA", "primitiveMode": "Line",
        "renderTargetFormats": [["R8_UNORM", "nope"], []],
        "isAnimatedTexture": false,
        "variants": [{"skinning": {"+defines": ["USE_SKINNING"]}}, {}, {"a": {}, "a.b": {"variants": []}}],
        "+variants": [], "-variants": [{"gone": {"blendSrc": "Nope"}}],
        "vrGeometryShader": "x", "+vertexShader": "y", "varient": [], "depthFunc2": null,
        "stencilRefOverride2": {"deep": [1, 2.5, "x", true, null]}"#;
    for text in [
        wrapped(&format!(r#""x": {{{members}}}"#)),
        flat(&format!(r#""x": {{{members}}}"#)),
    ] {
        let written = round_trip(&text);
        // A whole float is written as the integer it reads back from.
        assert!(
            written.contains("\"slopeScaledDepthBias\": -3,"),
            "{written}"
        );
        // Member for member what was read: the same JSON value, order aside.
        let read = canonical;
        assert_eq!(read(&written), read(&text));
    }
}

#[test]
fn a_member_the_reader_does_not_take_is_written_back_as_it_was() {
    // Wrong types, nulls, values out of range, a mis-versioned wrapper, root members that are
    // no material: each is kept where it was read from.
    for text in [
        flat(
            r#""x": {"vertexShader": 5, "states": "Blending", "stencilRef": -1,
                    "depthBias": "1", "frontFace": [], "variants": {"a": {}},
                    "renderTargetFormats": "R8_UNORM", "isAnimatedTexture": null}"#,
        ),
        r#"{"materials": {"version": "2.0.0", "a": {"vertexShader": "s"}}}"#.to_owned(),
        r#"{"materials": {"version": "1.0.0", "a": {}}, "other": {"+defines": ["A"]}}"#.to_owned(),
        r#"{"a": {}, "format_version": "1.10.0", "n": 5, "nothing": null}"#.to_owned(),
    ] {
        let written = round_trip(&text);
        let read = canonical;
        assert_eq!(read(&written), read(&text));
    }
}

#[test]
fn what_the_reader_drops_is_not_written_back() {
    // Entries the reader drops with a warning are gone from the value, and stay gone.
    let text = flat(
        r#""x": {"states": ["Blending", 5], "samplerStates": [7, {"samplerIndex": 0}],
                "vertexFields": ["UV0"], "renderTargetFormats": ["R8_UNORM", ["R8_UNORM", 1]],
                "variants": [3, {"a": 1, "b": {}}]}"#,
    );
    let first = file(&text);
    let written = first.to_json().expect("writes");
    let second = file(&written);
    assert_eq!(*first, *second);
    assert_eq!(rendered(second.diagnostics().iter()), []);
    let read = canonical;
    assert_eq!(
        read(&written),
        read(
            r#"{"x": {"states": ["Blending"], "samplerStates": [{"samplerIndex": 0}],
                     "vertexFields": [], "renderTargetFormats": [["R8_UNORM"]],
                     "variants": [{"b": {}}]}}"#
        )
    );
}

#[test]
fn a_clamped_bias_is_written_as_the_float_it_was_read_as() {
    let first = file(&flat(
        r#""x": {"depthBias": 1e39, "slopeScaledDepthBias": -1e999}"#,
    ));
    let written = first.to_json().expect("writes");
    let second = file(&written);
    assert_eq!(*first, *second);
    assert_eq!(rendered(second.diagnostics().iter()), []);
}

/// What reading `text` on its own finds, as code and path.
fn found_at(text: &str) -> Vec<(&'static str, String)> {
    let found = crate::common::found(text);
    found.into_iter().map(|f| (f.0, f.2)).collect()
}

/// The material `x` of `text` with what `text` is written as.
#[track_caller]
fn x_and_written(text: &str) -> (MaterialDef, String) {
    let first = file(text);
    let written = first.to_json().expect("writes");
    assert_eq!(*first, *file(&written), "{written}");
    (first.materials[&MaterialKey::new("x")].clone(), written)
}

/// A list: a define that is a number is dropped, an entry whose `path` is one too. `number`
/// spells the number.
fn a_list_drops_the_numbers_where_text_belongs(number: &str) {
    let text = format!(
        r#"[{{"path": "materials/a.material", "defines": ["A", {number}]}}, {{"path": {number}}}]"#
    );
    let list = parse_list("materials/common.json", &text).expect("loads");
    let found: Vec<_> = rendered(list.diagnostics().iter())
        .into_iter()
        .map(|f| (f.0, f.2))
        .collect();
    assert_eq!(
        found,
        [
            ("InvalidType", "0/defines/1".to_owned()),
            ("InvalidType", "1/path".to_owned())
        ]
    );
    assert_eq!(list.entries.len(), 1);
    assert_eq!(list.entries[0].defines, Some(vec!["A".to_owned()]));
    let written = list.to_json().expect("writes");
    let dropped = Rewrites {
        dropped: 2,
        ..Rewrites::default()
    };
    assert_eq!(rewrites(&text, &written), Ok(dropped));
}

#[test]
fn an_integer_beyond_64_bits_is_the_double_it_spells_in_every_member() {
    // Core reads an integer token too large for 64 bits as the double it spells, an
    // infinity for one past the doubles, which is clamped like `1e999`.
    const TWO_64: &str = "18446744073709551616";
    const TWO_128: &str = "340282366920938463463374607431768211456";
    let digits = "9".repeat(400);
    let digits = digits.as_str();
    let two_64 = 18_446_744_073_709_551_616.0_f64;
    let member = |key: &str, number: &str| flat(&format!(r#""x": {{"{key}": {number}}}"#));
    let invalid = |key: &str| vec![("InvalidValue", format!("x/{key}"))];
    let number_of = |def: &MaterialDef, key: &str| {
        let kept = def.extra.get(key).and_then(|value| value.as_number());
        kept.map(facet_value::VNumber::to_f64_lossy)
    };

    // `stencilRefOverride`: a number, on, written as the shortest decimal of the double.
    let text = member("stencilRefOverride", TWO_64);
    assert_eq!(found_at(&text), []);
    let (x, written) = x_and_written(&text);
    let held = x.stencil_ref_override.expect("read");
    assert_eq!(held, StencilRefOverride::Number(two_64));
    assert!(held.is_on());
    assert!(
        written.contains(r#""stencilRefOverride": 1.8446744073709552e19"#),
        "{written}"
    );
    assert_eq!(rewrites(&text, &written), Ok(Rewrites::default()));
    let text = member("stencilRefOverride", digits);
    assert_eq!(found_at(&text), invalid("stencilRefOverride"));
    let (x, written) = x_and_written(&text);
    assert_eq!(
        x.stencil_ref_override,
        Some(StencilRefOverride::Number(f64::MAX))
    );
    let clamped = Rewrites {
        clamped: 1,
        ..Rewrites::default()
    };
    assert_eq!(rewrites(&text, &written), Ok(clamped));

    // The biases: 2^64 is a float, rounded without a finding; past the floats, the largest
    // float of its sign with one finding.
    for key in ["depthBias", "slopeScaledDepthBias"] {
        let bias = |def: &MaterialDef| {
            if key == "depthBias" {
                def.depth_bias
            } else {
                def.slope_scaled_depth_bias
            }
        };
        let text = member(key, TWO_64);
        assert_eq!(found_at(&text), [], "{key}");
        let (x, written) = x_and_written(&text);
        assert_eq!(bias(&x), Some(1.844_674_4e19), "{key}");
        let rounded = Rewrites {
            rounded: 1,
            ..Rewrites::default()
        };
        assert_eq!(rewrites(&text, &written), Ok(rounded), "{key}\n{written}");
        for wide in [TWO_128, digits] {
            for (sign, expected) in [("", f32::MAX), ("-", f32::MIN)] {
                let text = member(key, &format!("{sign}{wide}"));
                assert_eq!(found_at(&text), invalid(key), "{key} {sign}{wide}");
                let (x, written) = x_and_written(&text);
                assert_eq!(bias(&x), Some(expected), "{key} {sign}{wide}");
                assert_eq!(rewrites(&text, &written), Ok(clamped), "{key} {sign}{wide}");
            }
        }
    }

    // `stencilRef` and the masks: out of range, kept in `extra` as the number.
    for key in ["stencilRef", "stencilReadMask", "stencilWriteMask"] {
        for (wide, kept, rewritten) in [
            (TWO_64, two_64, Rewrites::default()),
            (digits, f64::MAX, clamped),
        ] {
            let text = member(key, wide);
            assert_eq!(found_at(&text), invalid(key), "{key} {wide}");
            let (x, written) = x_and_written(&text);
            assert_eq!(number_of(&x, key), Some(kept), "{key} {wide}");
            assert_eq!(
                (x.stencil_ref, x.stencil_read_mask, x.stencil_write_mask),
                (None, None, None)
            );
            assert_eq!(rewrites(&text, &written), Ok(rewritten), "{key} {wide}");
        }
    }

    // A member no reader looks up is kept as the number.
    let text = member("custom", TWO_64);
    assert_eq!(found_at(&text), []);
    let (x, written) = x_and_written(&text);
    assert_eq!(number_of(&x, "custom"), Some(two_64));
    assert!(
        written.contains(r#""custom": 1.8446744073709552e19"#),
        "{written}"
    );
    let text = member("custom", digits);
    assert_eq!(found_at(&text), invalid("custom"));
    assert_eq!(number_of(&x_and_written(&text).0, "custom"), Some(f64::MAX));

    a_list_drops_the_numbers_where_text_belongs(TWO_64);
}

#[test]
fn the_key_rules_of_the_merge_are_not_written() {
    let mut value = flat_file(vec![("x", MaterialDef::default())]);
    value.materials[0].key_rules = Some(KeyRules::Versioned);
    assert_eq!(value.to_json().unwrap(), "{\n  \"x\": {}\n}\n");
    // The rules of a file read on its own are its form's.
    assert_eq!(
        value.key_rules(&MaterialKey::new("x")),
        Some(KeyRules::Versioned)
    );
    assert_eq!(
        file(&value.to_json().unwrap()).key_rules(&MaterialKey::new("x")),
        Some(KeyRules::Legacy)
    );
}

#[test]
fn a_wrapper_version_the_client_would_not_unwrap_is_refused() {
    for version in ["1.0.1", "2.0.0", "0.0.0", "1.0.0-beta"] {
        let value = wrapped_file(version, vec![("a", MaterialDef::default())]);
        assert_eq!(refused(&value), "materials/version", "{version}");
    }
    let mut any = wrapped_file("1.0.0", vec![]);
    any.form = MaterialFileForm::Wrapped {
        version: SemVersion::ANY,
    };
    assert_eq!(refused(&any), "materials/version");
    // 1.0.0 with build metadata passes the gate and is written.
    let build = wrapped_file("1.0.0+b", vec![("a", MaterialDef::default())]);
    assert!(build.to_json().is_ok());
}

#[test]
fn a_key_that_would_read_back_as_another_is_refused() {
    let odd = |key: MaterialKey| MaterialFile {
        form: MaterialFileForm::Flat,
        materials: IndexMap::from([(key, MaterialDef::default())]),
        extra: Extra::default(),
    };
    // A parent with a colon splits at another place on the way back.
    let parent = odd(MaterialKey {
        extend: false,
        name: "a".to_owned(),
        parent: Some("b:c".to_owned()),
    });
    assert_eq!(refused(&parent), "a:b:c");
    // A name with a colon and no parent reads back with one.
    assert_eq!(refused(&odd(MaterialKey::new("a:b"))), "a:b");
    // The wrapper's own members.
    let version = wrapped_file("1.0.0", vec![("version", MaterialDef::default())]);
    assert_eq!(refused(&version), "materials/version");
    let materials = flat_file(vec![("materials", MaterialDef::default())]);
    assert_eq!(refused(&materials), "materials");
    // In a wrapper it is a name like any other.
    let inside = wrapped_file("1.0.0", vec![("materials", MaterialDef::default())]);
    assert!(inside.to_json().is_ok());
}

#[test]
fn a_value_the_loader_refuses_or_reads_differently_is_refused() {
    let with = |def: MaterialDef| flat_file(vec![("x", def)]);
    // An unknown keyword is an error of the loader.
    let keyword = with(MaterialDef {
        blend_src: Some("Nope".to_owned()),
        ..MaterialDef::default()
    });
    assert_eq!(refused(&keyword), "x/blendSrc");
    // A known member hidden in `extra` would be read into its field.
    let mut hidden = MaterialDef::default();
    hidden
        .extra
        .insert("vertexShader".to_owned(), Value::from("s"));
    assert_eq!(refused(&with(hidden)), "x");
    // A null `isAnimatedTexture` reads back as the absent member.
    let null = with(MaterialDef {
        is_animated_texture: Some(Value::NULL),
        ..MaterialDef::default()
    });
    assert_eq!(refused(&null), "x");
    // A root member of `extra` that is an object would be a material.
    let mut object = flat_file(vec![]);
    object
        .extra
        .insert("y".to_owned(), facet_json::from_str::<Value>("{}").unwrap());
    assert_eq!(refused(&object), "");
    // Key rules on a variant body: nothing writes them, nothing reads them back.
    let body = MaterialDef {
        key_rules: Some(KeyRules::Legacy),
        ..MaterialDef::default()
    };
    let variant = with(MaterialDef {
        variants: Listed {
            replace: Some(vec![Variant(IndexMap::from([("v".to_owned(), body)]))]),
            ..Listed::default()
        },
        ..MaterialDef::default()
    });
    assert_eq!(refused(&variant), "x");
}

#[test]
fn a_bias_that_is_no_finite_number_is_refused_with_its_path() {
    for (bias, what) in [
        (f32::NAN, Unrepresentable::NaN),
        (f32::INFINITY, Unrepresentable::Infinity),
        (f32::NEG_INFINITY, Unrepresentable::NegativeInfinity),
    ] {
        let depth = flat_file(vec![(
            "x",
            MaterialDef {
                depth_bias: Some(bias),
                ..MaterialDef::default()
            },
        )]);
        match depth.to_json() {
            Err(EncodeError::NotRepresentable { at, what: found }) => {
                assert_eq!(at.to_string(), "x/depthBias");
                assert_eq!(found, what);
            }
            other => panic!("not refused: {other:?}"),
        }
        let slope = wrapped_file(
            "1.0.0",
            vec![(
                "x",
                MaterialDef {
                    slope_scaled_depth_bias: Some(bias),
                    ..MaterialDef::default()
                },
            )],
        );
        assert_eq!(refused(&slope), "materials/x/slopeScaledDepthBias");
    }
    // The largest finite floats are written and read back.
    let finite = flat_file(vec![(
        "x",
        MaterialDef {
            depth_bias: Some(f32::MAX),
            slope_scaled_depth_bias: Some(f32::MIN),
            ..MaterialDef::default()
        },
    )]);
    assert_eq!(*file(&finite.to_json().unwrap()), finite);
}

#[test]
fn the_single_form_is_written_only_as_one_parentless_material_with_a_vertex_shader() {
    let single = |materials: Vec<(&str, MaterialDef)>| MaterialFile {
        form: MaterialFileForm::Single {
            stem: "foo".to_owned(),
            wrapper: None,
        },
        ..flat_file(materials)
    };
    let shader = || MaterialDef {
        vertex_shader: Some("s".to_owned()),
        ..MaterialDef::default()
    };
    let good = single(vec![("foo", shader())]);
    let written = good.to_json().expect("writes");
    assert_eq!(*parse_at("materials/foo.material", &written).unwrap(), good);
    // No material, two, a parent, another name, or no vertex shader: no single form.
    assert_eq!(refused(&single(vec![])), "");
    assert_eq!(
        refused(&single(vec![("foo", shader()), ("b", shader())])),
        ""
    );
    assert_eq!(refused(&single(vec![("foo:p", shader())])), "");
    assert_eq!(refused(&single(vec![("+foo", shader())])), "");
    assert_eq!(refused(&single(vec![("bar", shader())])), "");
    assert_eq!(
        refused(&single(vec![("foo", MaterialDef::default())])),
        "vertexShader"
    );
}

#[test]
fn a_built_value_with_every_member_reads_back_equal() {
    let strings = |names: &[&str]| Some(names.iter().map(|n| (*n).to_owned()).collect::<Vec<_>>());
    let def = MaterialDef {
        states: Listed {
            add: strings(&["Blending"]),
            sub: strings(&["Wireframe"]),
            ..Listed::default()
        },
        defines: Listed {
            replace: strings(&["A", "B 2"]),
            ..Listed::default()
        },
        sampler_states: Listed {
            add: Some(vec![SamplerState {
                sampler_index: Some(7),
                texture_filter: Some("Bilinear".to_owned()),
                ..SamplerState::default()
            }]),
            ..Listed::default()
        },
        vertex_shader: Some("shaders/entity.vertex".to_owned()),
        front_face: Some(StencilFace {
            stencil_func: Some("Equal".to_owned()),
            ..StencilFace::default()
        }),
        stencil_ref: Some(300),
        vertex_fields: Some(vec![
            VertexField {
                field: Some("Position".to_owned()),
                ..VertexField::default()
            },
            VertexField::default(),
        ]),
        depth_bias: Some(0.25),
        render_target_formats: Some(vec![vec!["R8_UNORM".to_owned()]]),
        is_animated_texture: Some(Value::from(true)),
        variants: Listed {
            add: Some(vec![Variant(IndexMap::from([(
                "skinning".to_owned(),
                MaterialDef {
                    defines: Listed {
                        add: strings(&["USE_SKINNING"]),
                        ..Listed::default()
                    },
                    ..MaterialDef::default()
                },
            )]))]),
            ..Listed::default()
        },
        ..MaterialDef::default()
    };
    let value = wrapped_file(
        "1.0.0",
        vec![("base", MaterialDef::default()), ("x:base", def)],
    );
    let written = value.to_json().expect("writes");
    let read = file(&written);
    assert_eq!(*read, value);
    assert_eq!(rendered(read.diagnostics().iter()), []);
}

#[test]
fn a_list_is_written_entry_by_entry_and_reads_back_equal() {
    let value = MaterialList {
        kind: MaterialListKind::Sad,
        entries: vec![
            ListEntry {
                path: "materials/a.material".to_owned(),
                add_defines: Some(vec!["FANCY".to_owned()]),
                ..ListEntry::default()
            },
            ListEntry {
                path: String::new(),
                defines: Some(vec![]),
                sub_defines: Some(vec!["X".to_owned()]),
                ..ListEntry::default()
            },
        ],
    };
    let written = value.to_json().expect("writes");
    assert_eq!(
        written,
        "[\n  {\n    \"path\": \"materials/a.material\",\n    \"+defines\": [\n      \"FANCY\"\n    ]\n  },\n  {\n    \"path\": \"\",\n    \"defines\": [],\n    \"-defines\": [\n      \"X\"\n    ]\n  }\n]\n"
    );
    assert_eq!(*parse_list("materials/sad.json", &written).unwrap(), value);
    // Unknown members and a `tag` are written back.
    let text = r#"[{"path": "a", "tag": "t", "defines": 5, "note": null}]"#;
    let first = parse_list("materials/common.json", text).unwrap();
    let again = first.to_json().unwrap();
    let read = canonical;
    assert_eq!(read(&again), read(text));
}

#[test]
fn a_list_entry_that_would_read_back_differently_is_refused() {
    let list = |entry: ListEntry| MaterialList {
        kind: MaterialListKind::Common,
        entries: vec![ListEntry::default(), entry],
    };
    let mut hidden = ListEntry::default();
    hidden.extra.insert(
        "+defines".to_owned(),
        facet_json::from_str("[\"A\"]").unwrap(),
    );
    match list(hidden).to_json() {
        Err(EncodeError::NotRepresentable { at, .. }) => assert_eq!(at.to_string(), "1"),
        other => panic!("not refused: {other:?}"),
    }
    let mut path = ListEntry::default();
    path.extra.insert("path".to_owned(), Value::from("b"));
    assert!(matches!(
        list(path).to_json(),
        Err(EncodeError::NotRepresentable { .. })
    ));
}

/// A file whose one material nests `levels` variant bodies.
fn nested(levels: usize) -> MaterialFile {
    let mut def = MaterialDef::default();
    for _ in 0..levels {
        def = MaterialDef {
            variants: Listed {
                replace: Some(vec![Variant(IndexMap::from([("v".to_owned(), def)]))]),
                ..Listed::default()
            },
            ..MaterialDef::default()
        };
    }
    flat_file(vec![("x", def)])
}

#[test]
fn variants_nested_as_deep_as_the_loader_reads_are_written_and_deeper_ones_refused() {
    // The loader reads 128 levels of JSON: the root, the material, and three per variant body.
    let deepest = (0..60)
        .rev()
        .find(|levels| nested(*levels).to_json().is_ok())
        .expect("some depth is written");
    assert_eq!(deepest, 42);
    let written = nested(deepest).to_json().unwrap();
    assert_eq!(*file(&written), nested(deepest));
    assert!(matches!(
        nested(deepest + 1).to_json(),
        Err(EncodeError::NotRepresentable { .. })
    ));
    // And the loader agrees about the next one.
    let too_deep = format!(
        "{{\"x\": {}{{}}{}}}",
        "{\"variants\": [{\"v\": ".repeat(deepest + 1),
        "}]}".repeat(deepest + 1)
    );
    assert!(parse(&too_deep).is_err());
}

#[test]
fn a_value_nested_two_hundred_thousand_deep_is_refused_and_dropped_without_recursion() {
    // Built by a caller, never read: neither the writer nor `Drop` may recurse through it.
    let deep = nested(200_000);
    assert!(matches!(
        deep.to_json(),
        Err(EncodeError::NotRepresentable { .. })
    ));
    drop(deep);
}

#[test]
fn the_stencil_reference_override_is_written_as_it_was_written() {
    // Vanilla's `ui.material` writes `"stencilRefOverride": 0`,
    // and its file is written back as authored. A number stays the number, a boolean the
    // boolean.
    for value in ["0", "1", "-1", "2.5", "true", "false"] {
        let text = wrapped(&format!(r#""x": {{"stencilRefOverride": {value}}}"#));
        let written = round_trip(&text);
        assert_written_as_authored(&text, &written);
        assert!(
            written.contains(&format!("\"stencilRefOverride\": {value}\n")),
            "{value}: {written}"
        );
    }
    // `0.0` is the number 0.
    let text = wrapped(r#""x": {"stencilRefOverride": 0.0}"#);
    assert_written_as_authored(&text, &round_trip(&text));
    // A built number that JSON cannot spell is refused where it is.
    for number in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let def = MaterialDef {
            stencil_ref_override: Some(StencilRefOverride::Number(number)),
            ..MaterialDef::default()
        };
        assert_eq!(
            refused(&flat_file(vec![("x", def)])),
            "x/stencilRefOverride",
            "{number}"
        );
    }
}

#[test]
fn the_comparison_with_the_input_sees_each_kind_of_difference() {
    // The oracle of the as-authored checks, on differences written by hand.
    let none = |input: &str, written: &str| rewrites(input, written).map(|found| found.is_none());
    assert_eq!(
        none(
            r#"{"a": 1, "b": [1.0, "x", null, true]}"#,
            r#"{"b": [1, "x", null, true], "a": 1.0}"#
        ),
        Ok(true)
    );
    assert_eq!(none("// c\n{\"a\": 1, \"a\": 2}", r#"{"a": 2}"#), Ok(true));
    let found = rewrites(
        r#"{"a": [1, "x", 2], "v": [{"k": 5, "m": {}}], "n": 1e999, "f": 4294967295}"#,
        r#"{"a": ["x"], "v": [{"m": {}}], "n": 1.7976931348623157e308, "f": 4294967296}"#,
    )
    .unwrap();
    assert_eq!((found.dropped, found.clamped, found.rounded), (3, 1, 1));
    for (input, written) in [
        (r#"{"a": 0}"#, r#"{"a": false}"#),
        (r#"{"a": 1}"#, r#"{"a": 2}"#),
        (r#"{"a": 1}"#, r#"{"a": 1, "b": 2}"#),
        (r#"{"a": {"k": 1}}"#, r"{}"),
        (r#"{"a": "x"}"#, r#"{"a": "y"}"#),
        (r"[1, 2]", r"[2, 1]"),
        (r"[1]", r"[1, 1]"),
        (r#"{"a": null}"#, r#"{"a": false}"#),
    ] {
        assert!(rewrites(input, written).is_err(), "{input} {written}");
    }
}

#[test]
fn a_bias_no_float_holds_is_compared_as_the_float_and_counted_as_rounded() {
    // The writer writes the shortest decimal that reads back to the float the client reads:
    // the comparison with the input must take each of these as rounded, not as an error.
    for (value, shortest) in [
        ("0.30000000000000004", "0.3"),
        ("9007199254740993", "9007199000000000"),
        ("18446744073709551615", "18446744000000000000"),
        ("16777217", "16777216"),
    ] {
        for member in ["depthBias", "slopeScaledDepthBias"] {
            let text = wrapped(&format!(r#""x": {{"{member}": {value}}}"#));
            let written = round_trip(&text);
            let float: f32 = value.parse().unwrap();
            let number = written
                .split_once(&format!("\"{member}\": "))
                .and_then(|(_, rest)| rest.lines().next())
                .unwrap_or_else(|| panic!("{member} {value}: {written}"));
            assert_eq!(number.parse::<f32>().ok(), Some(float), "{member} {value}");
            assert_eq!(
                number.parse::<f64>().ok(),
                shortest.parse::<f64>().ok(),
                "{member} {value}: {number}"
            );
            let found = rewrites(&text, &written);
            let expected = Rewrites {
                rounded: 1,
                ..Rewrites::default()
            };
            assert_eq!(found, Ok(expected), "{member} {value}: {written}");
        }
    }
    // A number that is another float is still an error.
    assert!(rewrites(r#"{"a": 0.30000000000000004}"#, r#"{"a": 0.31}"#).is_err());
    assert!(rewrites(r#"{"a": 16777217}"#, r#"{"a": 16777218}"#).is_err());
}

/// A negative zero is not lost in silence. Where the member holds a float (the two biases,
/// and the number form of `stencilRefOverride`) the sign is held and written: `-0.0`. Where
/// it holds an unsigned integer (the reference and the two masks) there is no sign to hold:
/// the file's `-0.0` is written `0`, and the load notes it at the member. The integer token
/// `-0` is the integer 0 everywhere; every other number is written as before.
// Amends the family's earlier rule that a negative zero is written `0` with no word.
#[test]
fn a_negative_zero_is_written_with_its_sign_or_noted() {
    let floats = ["depthBias", "slopeScaledDepthBias", "stencilRefOverride"];
    let unsigned = ["stencilRef", "stencilReadMask", "stencilWriteMask"];
    for member in floats.into_iter().chain(unsigned) {
        let noted = |text: &str| -> Vec<String> {
            rendered(parse(text).expect("loads").diagnostics().iter())
                .into_iter()
                .map(|finding| format!("{finding:?}"))
                .collect()
        };
        for zero in ["-0.0", "-0e0"] {
            let text = wrapped(&format!(r#""x": {{"{member}": {zero}}}"#));
            let found = noted(&text);
            let written = if floats.contains(&member) {
                round_trip(&text)
            } else {
                // The note is of the file's `-0.0`: the written `0` reads back to the same
                // value without it.
                let first = parse(&text).expect("loads");
                let written = first.to_json().expect("writes");
                let second = parse(&written).expect("the written text loads");
                assert_eq!(*first, *second, "{written}");
                assert_eq!(noted(&written), [] as [&str; 0], "{written}");
                written
            };
            if floats.contains(&member) {
                assert!(
                    written.contains(&format!("\"{member}\": -0.0\n")),
                    "{member} {zero}: {written}"
                );
                assert_eq!(found, [] as [&str; 0], "{member} {zero}");
            } else {
                assert!(
                    written.contains(&format!("\"{member}\": 0\n")),
                    "{member} {zero}: {written}"
                );
                assert_eq!(found.len(), 1, "{member} {zero}: {found:?}");
                assert!(
                    found[0].contains("ValueRewritten") && found[0].contains("Info"),
                    "{member} {zero}: {found:?}"
                );
            }
            assert_written_as_authored(&text, &written);
        }
        // An integer zero has no sign, and every other value is written as before.
        for (value, token) in [("-0", "0"), ("0.0", "0"), ("0", "0"), ("1.0", "1")] {
            let text = wrapped(&format!(r#""x": {{"{member}": {value}}}"#));
            assert_eq!(noted(&text), [] as [&str; 0], "{member} {value}");
            let written = round_trip(&text);
            assert!(
                written.contains(&format!("\"{member}\": {token}\n")),
                "{member} {value}: {written}"
            );
            assert_written_as_authored(&text, &written);
        }
    }
}

/// The members of a material body are written in the reader's order (`states` before
/// `defines`, each with its `+` and `-` forms), whatever order the file had them in. The
/// game reads each member by its key, in its own fixed order (it parses states, then defines, …, each looking up the plain, the `+` and the `-` key),
/// so the order of the members in the text changes nothing it reads: the written file is
/// the same material, and reads back as the same value.
#[test]
fn k108_the_members_of_a_body_are_written_in_the_readers_order_and_read_back_equal() {
    let text = flat(
        r#""base": {"+defines": ["FANCY"], "+states": ["Blending"], "vertexShader": "a.vertex"},
           "child:base": {"-defines": ["FANCY"], "+defines": ["PLAIN"], "-states": ["Blending"]}"#,
    );
    let written = round_trip(&text);
    let at = |member: &str| {
        written
            .find(member)
            .unwrap_or_else(|| panic!("no {member} in {written}"))
    };
    assert!(at(r#""+states""#) < at(r#""+defines""#), "{written}");
    assert!(at(r#""-states""#) < at(r#""-defines""#), "{written}");
    // Every member and every value of the file is in the text, and nothing else.
    let sorted = |text: &str| {
        let mut tokens: Vec<String> = text
            .split(|c: char| c.is_whitespace() || matches!(c, '{' | '}' | '[' | ']' | ',' | ':'))
            .filter(|token| !token.is_empty())
            .map(str::to_owned)
            .collect();
        tokens.sort();
        tokens
    };
    assert_eq!(sorted(&written), sorted(&text), "{written}");
}
