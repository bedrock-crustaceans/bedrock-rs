//! The rules of one material's members, on files read on their own.

use bedrock_addon::diagnostic::Severity::{self, Error, Info};
use bedrock_addon::families::material::{
    BLEND_TARGETS, COMPARISON_FUNCS, MSAA_SUPPORTS, MaterialDef, MaterialKey, PRIMITIVE_MODES,
    RENDER_STATES, SHADER_STAGES, STENCIL_OPS, StencilRefOverride, TEXTURE_FILTERS,
    TEXTURE_FORMATS, TEXTURE_WRAPS, VERTEX_FIELDS,
};

use crate::common::{Rendered, file, finding, flat, parse, rendered, wrapped};

/// The findings of reading `text` on its own, each at its path below the material map.
///
/// A finding names its place in the file, which for a wrapped file starts with the
/// wrapper's `materials` member. The rules of this file are those of a material's members, in
/// a wrapped and in a flat file alike: the helper requires every finding of a wrapped file to
/// be below `materials` and takes that step off, so that one expectation serves both forms. A
/// flat file's paths are compared as they are.
#[track_caller]
fn found(text: &str) -> Vec<Rendered> {
    let all = crate::common::found(text);
    if !text.starts_with(r#"{"materials": {"version": "1.0.0""#) {
        return all;
    }
    all.into_iter()
        .map(|(code, severity, at, text)| {
            let Some(inner) = at.strip_prefix("materials/") else {
                panic!("{code} at {at:?}: a finding of a wrapped file outside its wrapper");
            };
            (code, severity, inner.to_owned(), text)
        })
        .collect()
}

/// The material `x` of a wrapped file whose `x` has `members`.
fn versioned(members: &str) -> String {
    wrapped(&format!(r#""x": {{{members}}}"#))
}

/// The material `x` of a flat file whose `x` has `members`.
fn legacy(members: &str) -> String {
    flat(&format!(r#""x": {{{members}}}"#))
}

/// The material `x` of `text`, which must load.
#[track_caller]
fn x(text: &str) -> MaterialDef {
    file(text).materials[&MaterialKey::new("x")].clone()
}

fn one(code: &'static str, severity: Severity, at: &str, text: &str) -> Vec<Rendered> {
    vec![finding(code, severity, at, text)]
}

/// Where each keyword-valued member sits, and its table: (members with `{}` for the value,
/// path of the value, table, read in a legacy material).
fn keyword_places() -> Vec<(&'static str, &'static str, &'static [&'static str], bool)> {
    vec![
        (r#""depthFunc": {}"#, "x/depthFunc", &COMPARISON_FUNCS, true),
        (r#""blendSrc": {}"#, "x/blendSrc", &BLEND_TARGETS, true),
        (r#""blendDst": {}"#, "x/blendDst", &BLEND_TARGETS, true),
        (r#""alphaSrc": {}"#, "x/alphaSrc", &BLEND_TARGETS, true),
        (r#""alphaDst": {}"#, "x/alphaDst", &BLEND_TARGETS, true),
        (
            r#""msaaSupport": {}"#,
            "x/msaaSupport",
            &MSAA_SUPPORTS,
            true,
        ),
        (
            r#""primitiveMode": {}"#,
            "x/primitiveMode",
            &PRIMITIVE_MODES,
            true,
        ),
        (
            r#""frontFace": {"stencilFunc": {}}"#,
            "x/frontFace/stencilFunc",
            &COMPARISON_FUNCS,
            true,
        ),
        (
            r#""frontFace": {"stencilFailOp": {}}"#,
            "x/frontFace/stencilFailOp",
            &STENCIL_OPS,
            true,
        ),
        (
            r#""backFace": {"stencilDepthFailOp": {}}"#,
            "x/backFace/stencilDepthFailOp",
            &STENCIL_OPS,
            true,
        ),
        (
            r#""backFace": {"stencilPassOp": {}}"#,
            "x/backFace/stencilPassOp",
            &STENCIL_OPS,
            true,
        ),
        (
            r#""samplerStates": [{"samplerIndex": 0, "textureFilter": {}}]"#,
            "x/samplerStates/0/textureFilter",
            &TEXTURE_FILTERS,
            true,
        ),
        (
            r#""samplerStates": [{"samplerIndex": 1}, {"samplerIndex": 0, "textureWrap": {}}]"#,
            "x/samplerStates/1/textureWrap",
            &TEXTURE_WRAPS,
            true,
        ),
        (
            r#""samplerStates": [{"samplerIndex": 0, "shaderStage": {}}]"#,
            "x/samplerStates/0/shaderStage",
            &SHADER_STAGES,
            false,
        ),
        (
            r#""samplerStates": [{"samplerIndex": 0, "comparisonFunc": {}}]"#,
            "x/samplerStates/0/comparisonFunc",
            &COMPARISON_FUNCS,
            false,
        ),
        (
            r#""vertexFields": [{"field": "Position"}, {"field": {}}]"#,
            "x/vertexFields/1/field",
            &VERTEX_FIELDS,
            true,
        ),
    ]
}

#[test]
fn every_keyword_of_every_table_is_accepted_in_its_place() {
    for (members, _, table, _) in keyword_places() {
        for keyword in table {
            let members = members.replace("{}", &format!("\"{keyword}\""));
            assert_eq!(found(&versioned(&members)), [], "{members}");
            assert_eq!(found(&legacy(&members)), [], "{members}");
        }
    }
}

#[test]
fn v4_an_unknown_keyword_is_an_error_that_refuses_the_file_deviation() {
    // vanilla = "assert + read through end iterator": ours is an error, and a file read on its
    // own is rejected.
    for (members, at, _, _) in keyword_places() {
        let members = members.replace("{}", "\"Nope\"");
        let text = versioned(&members);
        assert!(parse(&text).is_err(), "{members}");
        assert_eq!(
            found(&text),
            one("InvalidValue", Error, at, "invalid value"),
            "{members}"
        );
    }
}

#[test]
fn v4_a_keyword_is_matched_byte_for_byte() {
    // Lower case, a neighbouring table's word and the empty string are all unknown.
    for value in ["less", "LESS", "Keep", "", " Less"] {
        let text = versioned(&format!(r#""depthFunc": "{value}""#));
        assert_eq!(
            found(&text),
            one("InvalidValue", Error, "x/depthFunc", "invalid value"),
            "{value:?}"
        );
    }
}

#[test]
fn v4_a_legacy_material_does_not_read_the_stage_and_the_comparison_of_a_sampler() {
    // Legacy `samplerStates` reads `samplerIndex`, `textureFilter` and `textureWrap` only: an
    // unknown word in the other two is never looked up.
    for (members, at, _, in_legacy) in keyword_places() {
        let text = legacy(&members.replace("{}", "\"Nope\""));
        if in_legacy {
            assert_eq!(
                found(&text),
                one("InvalidValue", Error, at, "invalid value"),
                "{members}"
            );
        } else {
            assert_eq!(found(&text), [], "{members}");
        }
    }
}

#[test]
fn m10_an_unknown_blend_factor_is_an_error() {
    let text = versioned(r#""blendSrc": "SrcAlpha""#);
    assert_eq!(
        found(&text),
        one("InvalidValue", Error, "x/blendSrc", "invalid value")
    );
    assert!(parse(&text).is_err());
}

#[test]
fn m27_the_pcf_filter_is_spelt_in_capitals() {
    let pcf = r#""samplerStates": [{"samplerIndex": 0, "textureFilter": "PCF"}]"#;
    assert_eq!(found(&versioned(pcf)), []);
    let sampler = &x(&versioned(pcf)).sampler_states.replace.unwrap()[0];
    assert_eq!(sampler.sampler_index, Some(0));
    assert_eq!(sampler.texture_filter.as_deref(), Some("PCF"));
    let header_spelling = r#""samplerStates": [{"samplerIndex": 0, "textureFilter": "Pcf"}]"#;
    assert_eq!(
        found(&versioned(header_spelling)),
        one(
            "InvalidValue",
            Error,
            "x/samplerStates/0/textureFilter",
            "invalid value"
        )
    );
}

#[test]
fn v4_a_sampler_entry_the_client_skips_has_its_keywords_unread() {
    // Without an index, or with one out of range, the entry is skipped before its members are
    // read: only the skip is reported.
    let no_index = versioned(r#""samplerStates": [{"textureFilter": "Nope"}]"#);
    assert_eq!(
        found(&no_index),
        one(
            "MissingRequiredField",
            Info,
            "x/samplerStates/0",
            "missing required field"
        )
    );
    let out_of_range =
        versioned(r#""samplerStates": [{"samplerIndex": 8, "textureFilter": "Nope"}]"#);
    assert_eq!(
        found(&out_of_range),
        one(
            "InvalidValue",
            Info,
            "x/samplerStates/0/samplerIndex",
            "invalid value"
        )
    );
}

#[test]
fn v4a_an_unknown_render_target_format_is_a_warning_deviation() {
    // vanilla pushes `UNKNOWN_TEXTURE_FORMAT` after its assertion: the material loads.
    for format in TEXTURE_FORMATS {
        let text = versioned(&format!(r#""renderTargetFormats": [["{format}"]]"#));
        assert_eq!(found(&text), [], "{format}");
    }
    let text = versioned(r#""renderTargetFormats": [["R8G8B8A8_UNORM", "RGBA8"], ["nope"]]"#);
    assert_eq!(
        found(&text),
        [
            finding(
                "InvalidValue",
                Info,
                "x/renderTargetFormats/0/1",
                "invalid value"
            ),
            finding(
                "InvalidValue",
                Info,
                "x/renderTargetFormats/1/0",
                "invalid value"
            ),
        ]
    );
    // Kept as written.
    assert_eq!(
        x(&text).render_target_formats,
        Some(vec![
            vec!["R8G8B8A8_UNORM".to_owned(), "RGBA8".to_owned()],
            vec!["nope".to_owned()]
        ])
    );
}

#[test]
fn every_render_state_is_accepted_under_each_of_the_three_keys() {
    for (state, _) in RENDER_STATES {
        for key in ["states", "+states", "-states"] {
            let text = versioned(&format!(r#""{key}": ["{state}"]"#));
            assert_eq!(found(&text), [], "{key} {state}");
        }
        assert_eq!(found(&legacy(&format!(r#""states": ["{state}"]"#))), []);
    }
    let read = x(&versioned(
        r#""+states": ["Blending"], "-states": ["Wireframe"]"#,
    ));
    assert_eq!(read.states.replace, None);
    assert_eq!(read.states.add, Some(vec!["Blending".to_owned()]));
    assert_eq!(read.states.sub, Some(vec!["Wireframe".to_owned()]));
}

#[test]
fn m08_polygon_offset_is_named_as_ignored_and_the_other_states_are_kept() {
    let text = versioned(r#""states": ["PolygonOffset", "Blending"]"#);
    assert_eq!(
        found(&text),
        one(
            "NoEffect",
            Info,
            "x/states/0",
            "Render state PolygonOffset is ignored and will have no effect"
        )
    );
    assert_eq!(
        x(&text).states.replace,
        Some(vec!["PolygonOffset".to_owned(), "Blending".to_owned()])
    );
    // The same under the prefixed keys, and in a legacy material.
    for members in [
        r#""+states": ["Blending", "PolygonOffset"]"#,
        r#""-states": ["Blending", "PolygonOffset"]"#,
    ] {
        let found = found(&versioned(members));
        assert_eq!(found.len(), 1, "{members}");
        assert_eq!((found[0].0, found[0].1), ("NoEffect", Info), "{members}");
        assert!(found[0].2.ends_with("states/1"), "{members}");
    }
    assert_eq!(
        found(&legacy(r#""states": ["PolygonOffset"]"#)),
        one(
            "NoEffect",
            Info,
            "x/states/0",
            "Render state PolygonOffset is ignored and will have no effect"
        )
    );
}

#[test]
fn m09_an_unknown_render_state_is_reported_with_the_clients_text() {
    // the game logs it at level 3 and the material
    // loads without the state: a kept error.
    let text = versioned(r#""+states": ["DisableCull"]"#);
    assert_eq!(
        found(&text),
        one(
            "InvalidValue",
            Error,
            "x/+states/0",
            "Trying to use a non-existing render state DisableCull for this material!"
        )
    );
    assert!(parse(&text).is_ok());
    // Byte for byte: another case is another name.
    let lower = versioned(r#""states": ["Blending", "blending"]"#);
    assert_eq!(
        found(&lower),
        one(
            "InvalidValue",
            Error,
            "x/states/1",
            "Trying to use a non-existing render state blending for this material!"
        )
    );
}

#[test]
fn m07_v8_a_prefix_on_a_key_without_one_is_unrecognized_and_kept_deviation() {
    let text = versioned(r#""+vertexShader": "shaders/x.vertex""#);
    assert_eq!(
        found(&text),
        one(
            "UnrecognizedField",
            Info,
            "x/+vertexShader",
            "unrecognized field"
        )
    );
    let read = x(&text);
    assert_eq!(read.vertex_shader, None);
    assert!(read.extra.get("+vertexShader").is_some());
}

#[test]
fn v8_every_plain_key_is_reported_under_both_prefixes_deviation() {
    let plain = [
        "vertexShader",
        "fragmentShader",
        "geometryShader",
        "depthFunc",
        "frontFace",
        "backFace",
        "stencilRefOverride",
        "stencilRef",
        "stencilReadMask",
        "stencilWriteMask",
        "blendSrc",
        "blendDst",
        "alphaSrc",
        "alphaDst",
        "vertexFields",
        "depthBias",
        "slopeScaledDepthBias",
        "msaaSupport",
        "primitiveMode",
        "renderTargetFormats",
        "isAnimatedTexture",
    ];
    for key in plain {
        for prefix in ["+", "-"] {
            let text = versioned(&format!(r#""{prefix}{key}": "Nope""#));
            assert_eq!(
                found(&text),
                one(
                    "UnrecognizedField",
                    Info,
                    &format!("x/{prefix}{key}"),
                    "unrecognized field"
                ),
                "{prefix}{key}"
            );
        }
    }
}

#[test]
fn v8_a_prefix_on_a_member_of_a_sampler_entry_or_of_a_face_is_unrecognized_deviation() {
    for key in [
        "samplerIndex",
        "textureFilter",
        "textureWrap",
        "shaderStage",
        "comparisonFunc",
    ] {
        for prefix in ["+", "-"] {
            let text = versioned(&format!(
                r#""samplerStates": [{{"samplerIndex": 0, "{prefix}{key}": "Nope"}}]"#
            ));
            assert_eq!(
                found(&text),
                one(
                    "UnrecognizedField",
                    Info,
                    &format!("x/samplerStates/0/{prefix}{key}"),
                    "unrecognized field"
                ),
                "{prefix}{key}"
            );
        }
    }
    for key in [
        "stencilFunc",
        "stencilFailOp",
        "stencilDepthFailOp",
        "stencilPassOp",
    ] {
        for prefix in ["+", "-"] {
            let text = versioned(&format!(r#""backFace": {{"{prefix}{key}": "Nope"}}"#));
            assert_eq!(
                found(&text),
                one(
                    "UnrecognizedField",
                    Info,
                    &format!("x/backFace/{prefix}{key}"),
                    "unrecognized field"
                ),
                "{prefix}{key}"
            );
        }
    }
}

#[test]
fn a_prefix_on_an_unknown_key_says_nothing() {
    // Only the keys the readers look up are checked: `+vrGeometryShader` is as unread as
    // `vrGeometryShader`.
    let text = versioned(r#""+vrGeometryShader": "s", "-custom": 1, "+": 2"#);
    assert_eq!(found(&text), []);
    assert_eq!(x(&text).extra.iter().count(), 3);
}

#[test]
fn m06_v9_a_misspelt_variants_key_is_unrecognized_deviation() {
    for key in ["varient", "varients", "variant"] {
        for prefix in ["", "+", "-"] {
            let text = versioned(&format!(r#""{prefix}{key}": [{{"skinning": {{}}}}]"#));
            assert_eq!(
                found(&text),
                one(
                    "UnrecognizedField",
                    Info,
                    &format!("x/{prefix}{key}"),
                    "unrecognized field"
                ),
                "{prefix}{key}"
            );
            let read = x(&text);
            assert!(read.variants.is_empty());
            assert!(read.extra.get(&format!("{prefix}{key}")).is_some());
        }
    }
}

#[test]
fn m05_v10_variants_that_are_no_array_are_not_read_deviation() {
    for key in ["variants", "+variants"] {
        let text = versioned(&format!(r#""{key}": {{"skinning": {{}}}}"#));
        assert_eq!(
            found(&text),
            one("InvalidType", Info, &format!("x/{key}"), "wrong JSON type"),
            "{key}"
        );
        let read = x(&text);
        assert!(read.variants.is_empty(), "{key}");
        assert!(read.extra.get(key).is_some(), "{key}");
    }
}

#[test]
fn v10_an_element_of_variants_that_is_no_object_and_a_body_that_is_none_are_dropped_deviation() {
    let text = versioned(r#""variants": [{"a": {}}, 5, {"b": "no", "c": {}}]"#);
    assert_eq!(
        found(&text),
        [
            finding("InvalidType", Info, "x/variants/1", "wrong JSON type"),
            finding("InvalidType", Info, "x/variants/2/b", "wrong JSON type"),
        ]
    );
    let variants = x(&text).variants.replace.unwrap();
    let names: Vec<Vec<&str>> = variants
        .iter()
        .map(|variant| variant.0.keys().map(String::as_str).collect())
        .collect();
    assert_eq!(names, [vec!["a"], vec!["c"]]);
}

#[test]
fn variants_are_read_in_order_with_their_bodies_checked_like_materials() {
    let text = versioned(
        r#""variants": [{"skinning": {"+defines": ["USE_SKINNING"]}},
                         {"skinning_color": {"+states": ["Nope"], "blendSrc": "Bad"}}],
           "+variants": [{"extra": {"vertexFields": [{"field": "UV0"}]}}]"#,
    );
    assert_eq!(
        found(&text),
        [
            finding(
                "InvalidValue",
                Error,
                "x/variants/1/skinning_color/+states/0",
                "Trying to use a non-existing render state Nope for this material!"
            ),
            finding(
                "InvalidValue",
                Error,
                "x/variants/1/skinning_color/blendSrc",
                "invalid value"
            ),
        ]
    );
    let ok = versioned(
        r#""variants": [{"skinning": {"+defines": ["USE_SKINNING"]}}, {"two": {}, "three": {}}],
           "+variants": [{"extra": {"vertexFields": [{"field": "UV0"}]}}]"#,
    );
    let read = x(&ok);
    let replace = read.variants.replace.as_ref().unwrap();
    assert_eq!(
        replace[0].0["skinning"].defines.add,
        Some(vec!["USE_SKINNING".to_owned()])
    );
    assert_eq!(replace[1].0.keys().collect::<Vec<_>>(), ["two", "three"]);
    let add = read.variants.add.as_ref().unwrap();
    assert_eq!(
        add[0].0["extra"].vertex_fields.as_ref().unwrap()[0]
            .field
            .as_deref(),
        Some("UV0")
    );
    assert_eq!(found(&ok), []);
}

#[test]
fn m11_v12_a_sampler_index_of_8_is_out_of_range_and_7_is_the_last_deviation() {
    let at_8 = r#""+samplerStates": [{"samplerIndex": 8}]"#;
    assert_eq!(
        found(&versioned(at_8)),
        one(
            "InvalidValue",
            Info,
            "x/+samplerStates/0/samplerIndex",
            "invalid value"
        )
    );
    let at_7 = r#""+samplerStates": [{"samplerIndex": 7}]"#;
    assert_eq!(found(&versioned(at_7)), []);
    assert_eq!(
        found(&versioned(r#""samplerStates": [{"samplerIndex": 0}]"#)),
        []
    );
    // The entry is kept as written either way.
    let read = x(&versioned(at_8));
    assert_eq!(read.sampler_states.add.unwrap()[0].sampler_index, Some(8));
    // A legacy material skips the entry without the client's assertion; ours warns the same.
    let legacy_8 =
        legacy(r#""samplerStates": [{"samplerIndex": 8}, {"samplerIndex": 4000000000}]"#);
    assert_eq!(
        found(&legacy_8),
        [
            finding(
                "InvalidValue",
                Info,
                "x/samplerStates/0/samplerIndex",
                "invalid value"
            ),
            finding(
                "InvalidValue",
                Info,
                "x/samplerStates/1/samplerIndex",
                "invalid value"
            ),
        ]
    );
    assert_eq!(
        found(&legacy(r#""samplerStates": [{"samplerIndex": 7}]"#)),
        []
    );
}

#[test]
fn m21_v21_minus_sampler_states_is_not_supported_deviation() {
    let text = versioned(r#""-samplerStates": [{"samplerIndex": 0}]"#);
    assert_eq!(
        found(&text),
        one(
            "UnrecognizedField",
            Info,
            "x/-samplerStates",
            "unrecognized field"
        )
    );
    // Read all the same, for write-back.
    let read = x(&text);
    assert_eq!(read.sampler_states.sub.unwrap()[0].sampler_index, Some(0));
    // In both modes.
    assert_eq!(
        found(&legacy(r#""-samplerStates": []"#)),
        one(
            "UnrecognizedField",
            Info,
            "x/-samplerStates",
            "unrecognized field"
        )
    );
}

/// The V22 finding at `at`.
fn skipped(at: &str) -> Rendered {
    finding("NoEffect", Info, at, "member has no effect")
}

#[test]
fn v22_in_a_versioned_material_the_plain_key_hides_the_prefixed_ones_deviation() {
    for key in ["states", "defines"] {
        let both = versioned(&format!(r#""{key}": [], "+{key}": [], "-{key}": []"#));
        assert_eq!(
            found(&both),
            [skipped(&format!("x/+{key}")), skipped(&format!("x/-{key}"))],
            "{key}"
        );
        let plus = versioned(&format!(r#""+{key}": [], "{key}": []"#));
        assert_eq!(found(&plus), [skipped(&format!("x/+{key}"))], "{key}");
        let minus = versioned(&format!(r#""{key}": [], "-{key}": []"#));
        assert_eq!(found(&minus), [skipped(&format!("x/-{key}"))], "{key}");
        // Without the plain key both prefixed keys are read; the plain key alone is too.
        let prefixed = versioned(&format!(r#""+{key}": [], "-{key}": []"#));
        assert_eq!(found(&prefixed), [], "{key}");
        assert_eq!(found(&versioned(&format!(r#""{key}": []"#))), [], "{key}");
    }
    let samplers = versioned(r#""samplerStates": [], "+samplerStates": []"#);
    assert_eq!(found(&samplers), [skipped("x/+samplerStates")]);
    assert_eq!(found(&versioned(r#""+samplerStates": []"#)), []);
    assert_eq!(found(&versioned(r#""samplerStates": []"#)), []);
}

#[test]
fn m18_the_plus_states_beside_states_of_a_versioned_material_is_reported_deviation() {
    let text = wrapped(
        r#""p": {"states": ["Blending"]},
           "c:p": {"states": ["DisableCulling"], "+states": ["Wireframe"]}"#,
    );
    assert_eq!(found(&text), [skipped("c:p/+states")]);
}

#[test]
fn v22_a_legacy_material_never_reads_four_of_the_prefixed_keys_deviation() {
    for key in ["+states", "-states", "-defines", "+samplerStates"] {
        let text = legacy(&format!(r#""{key}": []"#));
        assert_eq!(found(&text), [skipped(&format!("x/{key}"))], "{key}");
    }
    // `+defines` is read, unless `defines` is there.
    assert_eq!(found(&legacy(r#""+defines": ["A"]"#)), []);
    assert_eq!(
        found(&legacy(r#""defines": ["A"], "+defines": ["B"]"#)),
        [skipped("x/+defines")]
    );
    // The plain keys are read.
    assert_eq!(
        found(&legacy(
            r#""states": [], "defines": [], "samplerStates": []"#
        )),
        []
    );
    // All of them at once, in the order of the file.
    assert_eq!(
        found(&legacy(
            r#""-defines": [], "states": [], "+states": [], "defines": [], "+defines": [],
               "-states": [], "samplerStates": [], "+samplerStates": []"#
        )),
        [
            skipped("x/-defines"),
            skipped("x/+states"),
            skipped("x/+defines"),
            skipped("x/-states"),
            skipped("x/+samplerStates"),
        ]
    );
}

#[test]
fn m19_m20_the_unread_keys_of_a_flat_file_are_reported_deviation() {
    let states = flat(
        r#""p": {"states": ["Blending"]},
           "c:p": {"states": ["DisableCulling"], "+states": ["Wireframe"]}"#,
    );
    assert_eq!(found(&states), [skipped("c:p/+states")]);
    let defines = flat(r#""p": {"defines": ["A", "B"]}, "c:p": {"-defines": ["A"]}"#);
    assert_eq!(found(&defines), [skipped("c:p/-defines")]);
}

#[test]
fn v22_a_variant_body_follows_the_rules_of_its_material_deviation() {
    // A variant body carries no version of its own.
    let members = r#""variants": [{"v": {"+states": ["Blending"]}}]"#;
    assert_eq!(found(&versioned(members)), []);
    assert_eq!(found(&legacy(members)), [skipped("x/variants/0/v/+states")]);
}

#[test]
fn m25_v23_a_sampler_entry_without_an_index_is_reported_deviation() {
    let text = versioned(r#""samplerStates": [{"textureFilter": "Bilinear"}]"#);
    assert_eq!(
        found(&text),
        one(
            "MissingRequiredField",
            Info,
            "x/samplerStates/0",
            "missing required field"
        )
    );
    // Kept as written: nothing is invented on the way back.
    let sampler = &x(&text).sampler_states.replace.unwrap()[0];
    assert_eq!(sampler.sampler_index, None);
    assert_eq!(sampler.texture_filter.as_deref(), Some("Bilinear"));
    // A null index is no index, and stays in the entry.
    let null = versioned(r#""+samplerStates": [{"samplerIndex": 0}, {"samplerIndex": null}]"#);
    assert_eq!(
        found(&null),
        one(
            "MissingRequiredField",
            Info,
            "x/+samplerStates/1",
            "missing required field"
        )
    );
    let entry = &x(&null).sampler_states.add.unwrap()[1];
    assert!(entry.extra.get("samplerIndex").is_some());
    // In a legacy material too.
    assert_eq!(
        found(&legacy(r#""samplerStates": [{}]"#)),
        one(
            "MissingRequiredField",
            Info,
            "x/samplerStates/0",
            "missing required field"
        )
    );
}

#[test]
fn m22_v24_minus_variants_is_never_read_deviation() {
    let text =
        wrapped(r#""entity_static": {}, "c:entity_static": {"-variants": [{"skinning": {}}]}"#);
    assert_eq!(
        found(&text),
        one(
            "UnrecognizedField",
            Info,
            "c:entity_static/-variants",
            "unrecognized field"
        )
    );
    let document = file(&text);
    let read = &document.materials[&MaterialKey::parse("c:entity_static")];
    assert!(
        read.variants.sub.as_ref().unwrap()[0]
            .0
            .contains_key("skinning")
    );
}

#[test]
fn m23_v25_render_target_formats_must_be_an_array_of_arrays_deviation() {
    // The flat form: each element is skipped.
    let flat_form =
        versioned(r#""renderTargetFormats": ["R8G8B8A8_UNORM", null, ["R8_UNORM"], 5]"#);
    let wrong = |at: &str| finding("InvalidType", Info, at, "wrong JSON type");
    assert_eq!(
        found(&flat_form),
        [
            wrong("x/renderTargetFormats/0"),
            wrong("x/renderTargetFormats/1"),
            wrong("x/renderTargetFormats/3"),
        ]
    );
    assert_eq!(
        x(&flat_form).render_target_formats,
        Some(vec![vec!["R8_UNORM".to_owned()]])
    );
    // Not an array at all: the member is ignored and kept.
    let scalar = versioned(r#""renderTargetFormats": "R8G8B8A8_UNORM""#);
    assert_eq!(found(&scalar), [wrong("x/renderTargetFormats")]);
    let read = x(&scalar);
    assert_eq!(read.render_target_formats, None);
    assert!(read.extra.get("renderTargetFormats").is_some());
    // A name that is no string is skipped inside its array.
    let number = versioned(r#""renderTargetFormats": [["R8_UNORM", 5]]"#);
    assert_eq!(found(&number), [wrong("x/renderTargetFormats/0/1")]);
}

#[test]
fn m24_is_animated_texture_is_kept_whatever_its_value() {
    for value in ["false", "true", "0", "\"x\"", "{}", "[]"] {
        let text = versioned(&format!(r#""isAnimatedTexture": {value}"#));
        assert_eq!(found(&text), [], "{value}");
        assert!(x(&text).is_animated_texture.is_some(), "{value}");
    }
    // A null is the absent member.
    let null = versioned(r#""isAnimatedTexture": null"#);
    assert_eq!(found(&null), []);
    assert_eq!(x(&null).is_animated_texture, None);
    assert!(x(&null).extra.get("isAnimatedTexture").is_some());
}

#[test]
fn v27_a_state_or_define_that_is_no_string_is_dropped_deviation() {
    // vanilla inserts the string conversion of a define that is no string; ours drops the entry.
    let wrong = |at: &str| finding("InvalidType", Info, at, "wrong JSON type");
    let text = versioned(r#""+states": ["Blending", 5, null], "+defines": [true, "A", {}]"#);
    assert_eq!(
        found(&text),
        [
            wrong("x/+states/1"),
            wrong("x/+states/2"),
            wrong("x/+defines/0"),
            wrong("x/+defines/2"),
        ]
    );
    let read = x(&text);
    assert_eq!(read.states.add, Some(vec!["Blending".to_owned()]));
    assert_eq!(read.defines.add, Some(vec!["A".to_owned()]));
    // Each of the six keys.
    for key in [
        "states", "+states", "-states", "defines", "+defines", "-defines",
    ] {
        let found = found(&versioned(&format!(r#""{key}": [7]"#)));
        assert_eq!(found, [wrong(&format!("x/{key}/0"))], "{key}");
    }
}

#[test]
fn defines_are_free_strings() {
    let text = versioned(r#""defines": ["BLUR_STEPS 4", "", "anything at all"]"#);
    assert_eq!(found(&text), []);
    assert_eq!(x(&text).defines.replace.unwrap().len(), 3);
}

#[test]
fn m12_the_dead_keys_say_nothing_and_are_kept_deviation() {
    let text = versioned(
        r#""vrGeometryShader": "shaders/uv.geometry", "depthBiasOGL": 1.5,
           "slopeScaledDepthBiasOGL": 2.5, "frontFace": {"stencilPass": "Replace"},
           "anything": {"else": []}"#,
    );
    assert_eq!(found(&text), []);
    let read = x(&text);
    let extra: Vec<&str> = read.extra.iter().map(|(key, _)| key).collect();
    assert_eq!(
        extra,
        [
            "vrGeometryShader",
            "depthBiasOGL",
            "slopeScaledDepthBiasOGL",
            "anything"
        ]
    );
    let face = read.front_face.unwrap();
    assert_eq!(face.stencil_pass_op, None);
    assert!(face.extra.get("stencilPass").is_some());
}

#[test]
fn every_typed_member_is_read_into_its_field() {
    let text = versioned(
        r#""vertexShader": "v", "fragmentShader": "f", "geometryShader": "g",
           "depthFunc": "Always",
           "frontFace": {"stencilFunc": "Equal", "stencilFailOp": "Replace",
                         "stencilDepthFailOp": "Keep", "stencilPassOp": "Replace"},
           "backFace": {},
           "stencilRefOverride": false, "stencilRef": 300,
           "stencilReadMask": 4294967295, "stencilWriteMask": 0,
           "blendSrc": "One", "blendDst": "Zero", "alphaSrc": "DestAlpha", "alphaDst": "DestColor",
           "vertexFields": [{"field": "Position"}, {}, {"field": "UV0", "note": 1}],
           "depthBias": 100.0, "slopeScaledDepthBias": -0.5,
           "msaaSupport": "Both", "primitiveMode": "Line",
           "renderTargetFormats": [["R8_UNORM"], []],
           "isAnimatedTexture": true,
           "samplerStates": [{"samplerIndex": 2, "textureFilter": "Point", "textureWrap": "Repeat",
                              "shaderStage": "Vertex", "comparisonFunc": "Less", "note": 1}]"#,
    );
    assert_eq!(found(&text), []);
    let read = x(&text);
    assert_eq!(read.vertex_shader.as_deref(), Some("v"));
    assert_eq!(read.fragment_shader.as_deref(), Some("f"));
    assert_eq!(read.geometry_shader.as_deref(), Some("g"));
    assert_eq!(read.depth_func.as_deref(), Some("Always"));
    let front = read.front_face.as_ref().unwrap();
    assert_eq!(front.stencil_func.as_deref(), Some("Equal"));
    assert_eq!(front.stencil_fail_op.as_deref(), Some("Replace"));
    assert_eq!(front.stencil_depth_fail_op.as_deref(), Some("Keep"));
    assert_eq!(front.stencil_pass_op.as_deref(), Some("Replace"));
    assert_eq!(read.back_face, Some(Default::default()));
    assert_eq!(read.stencil_ref_override, Some(false.into()));
    assert_eq!(read.stencil_ref, Some(300));
    assert_eq!(read.stencil_read_mask, Some(u32::MAX));
    assert_eq!(read.stencil_write_mask, Some(0));
    assert_eq!(read.blend_src.as_deref(), Some("One"));
    assert_eq!(read.blend_dst.as_deref(), Some("Zero"));
    assert_eq!(read.alpha_src.as_deref(), Some("DestAlpha"));
    assert_eq!(read.alpha_dst.as_deref(), Some("DestColor"));
    let fields = read.vertex_fields.as_ref().unwrap();
    assert_eq!(fields[0].field.as_deref(), Some("Position"));
    assert_eq!(fields[1].field, None);
    assert!(fields[2].extra.get("note").is_some());
    assert_eq!(read.depth_bias, Some(100.0));
    assert_eq!(read.slope_scaled_depth_bias, Some(-0.5));
    assert_eq!(read.msaa_support.as_deref(), Some("Both"));
    assert_eq!(read.primitive_mode.as_deref(), Some("Line"));
    assert_eq!(
        read.render_target_formats,
        Some(vec![vec!["R8_UNORM".to_owned()], vec![]])
    );
    assert!(read.is_animated_texture.is_some());
    let sampler = &read.sampler_states.replace.as_ref().unwrap()[0];
    assert_eq!(sampler.sampler_index, Some(2));
    assert_eq!(sampler.texture_filter.as_deref(), Some("Point"));
    assert_eq!(sampler.texture_wrap.as_deref(), Some("Repeat"));
    assert_eq!(sampler.shader_stage.as_deref(), Some("Vertex"));
    assert_eq!(sampler.comparison_func.as_deref(), Some("Less"));
    assert!(sampler.extra.get("note").is_some());
    assert!(read.extra.is_empty());
}

#[test]
fn a_member_of_the_wrong_type_is_kept_and_not_read() {
    let wrong = |at: &str| finding("InvalidType", Info, at, "wrong JSON type");
    for (members, key) in [
        (r#""vertexShader": 5"#, "vertexShader"),
        (r#""fragmentShader": []"#, "fragmentShader"),
        (r#""geometryShader": {}"#, "geometryShader"),
        (r#""depthFunc": 3"#, "depthFunc"),
        (r#""blendSrc": true"#, "blendSrc"),
        (r#""msaaSupport": 1"#, "msaaSupport"),
        (r#""primitiveMode": []"#, "primitiveMode"),
        (r#""frontFace": "Always""#, "frontFace"),
        (r#""backFace": []"#, "backFace"),
        (r#""stencilRefOverride": "true""#, "stencilRefOverride"),
        (r#""stencilRef": "1""#, "stencilRef"),
        (r#""stencilReadMask": true"#, "stencilReadMask"),
        (r#""depthBias": "1""#, "depthBias"),
        (r#""slopeScaledDepthBias": []"#, "slopeScaledDepthBias"),
        (r#""states": "Blending""#, "states"),
        (r#""+states": {}"#, "+states"),
        (r#""-defines": 5"#, "-defines"),
        (r#""samplerStates": {}"#, "samplerStates"),
        (r#""+samplerStates": "x""#, "+samplerStates"),
        (r#""vertexFields": {"field": "UV0"}"#, "vertexFields"),
    ] {
        let text = versioned(members);
        assert_eq!(found(&text), [wrong(&format!("x/{key}"))], "{members}");
        let read = x(&text);
        assert!(read.extra.get(key).is_some(), "{members}");
        let expected = MaterialDef {
            extra: read.extra.clone(),
            ..MaterialDef::default()
        };
        assert_eq!(read, expected, "{members}");
    }
    // Inside a face, a sampler entry and a vertex field.
    let nested = versioned(
        r#""frontFace": {"stencilFunc": 2},
           "samplerStates": [5, {"samplerIndex": "0"}, {"samplerIndex": 0, "textureFilter": 1}],
           "vertexFields": ["UV0", {"field": 4}]"#,
    );
    assert_eq!(
        found(&nested),
        [
            wrong("x/frontFace/stencilFunc"),
            wrong("x/samplerStates/0"),
            wrong("x/samplerStates/1/samplerIndex"),
            finding(
                "MissingRequiredField",
                Info,
                "x/samplerStates/1",
                "missing required field"
            ),
            wrong("x/samplerStates/2/textureFilter"),
            wrong("x/vertexFields/0"),
            wrong("x/vertexFields/1/field"),
        ]
    );
}

#[test]
fn a_null_member_is_the_absent_member_and_is_kept() {
    let text = versioned(
        r#""vertexShader": null, "states": null, "frontFace": null, "stencilRef": null,
           "variants": null, "depthBias": null"#,
    );
    assert_eq!(found(&text), []);
    let read = x(&text);
    assert_eq!(read.extra.iter().count(), 6);
    let expected = MaterialDef {
        extra: read.extra.clone(),
        ..MaterialDef::default()
    };
    assert_eq!(read, expected);
}

#[test]
fn an_unsigned_member_takes_whole_numbers_from_0_to_the_largest_u32() {
    let invalid = |at: &str| finding("InvalidValue", Info, at, "invalid value");
    for key in ["stencilRef", "stencilReadMask", "stencilWriteMask"] {
        for ok in ["0", "255", "256", "4294967295", "3.0"] {
            assert_eq!(
                found(&versioned(&format!(r#""{key}": {ok}"#))),
                [],
                "{key} {ok}"
            );
        }
        for bad in ["-1", "4294967296", "1.5", "1e30"] {
            let text = versioned(&format!(r#""{key}": {bad}"#));
            assert_eq!(found(&text), [invalid(&format!("x/{key}"))], "{key} {bad}");
            assert!(x(&text).extra.get(key).is_some(), "{key} {bad}");
        }
    }
    assert_eq!(x(&versioned(r#""stencilRef": 3.0"#)).stencil_ref, Some(3));
    assert_eq!(
        x(&versioned(r#""stencilReadMask": 4294967295"#)).stencil_read_mask,
        Some(u32::MAX)
    );
}

#[test]
fn an_unsigned_member_past_the_doubles_has_one_finding() {
    // A number past the doubles is reported where it is written, as in any member; the
    // reader of the member does not say it again. The member is kept as the largest double.
    let digits = "9".repeat(400);
    for key in ["stencilRef", "stencilReadMask", "stencilWriteMask"] {
        for (number, held) in [
            ("1e999", f64::MAX),
            ("-1e999", f64::MIN),
            (digits.as_str(), f64::MAX),
        ] {
            for form in [versioned, legacy] {
                let text = form(&format!(r#""{key}": {number}"#));
                assert_eq!(
                    found(&text),
                    one("InvalidValue", Info, &format!("x/{key}"), "invalid value"),
                    "{key} {number}"
                );
                let read = x(&text);
                let kept = read.extra.get(key).and_then(|value| value.as_number());
                assert_eq!(
                    kept.map(|number| number.to_f64_lossy()),
                    Some(held),
                    "{key} {number}"
                );
                assert_eq!(
                    (
                        read.stencil_ref,
                        read.stencil_read_mask,
                        read.stencil_write_mask
                    ),
                    (None, None, None)
                );
            }
        }
        // The largest double as written is out of the member's range like `1e30`.
        let text = versioned(&format!(r#""{key}": 1.7976931348623157e308"#));
        assert_eq!(
            found(&text),
            one("InvalidValue", Info, &format!("x/{key}"), "invalid value")
        );
    }
}

#[test]
fn a_bias_beyond_the_floats_is_the_largest_finite_float_with_a_warning_deviation() {
    // The JSON library's handling is not modelled; a number past the float range is never
    // loaded as an infinity the writer would then refuse.
    let invalid = |at: &str| finding("InvalidValue", Info, at, "invalid value");
    for (number, expected) in [
        ("1e39", f32::MAX),
        ("-1e39", f32::MIN),
        ("1e999", f32::MAX),
        ("-1e999", f32::MIN),
        ("3.5e38", f32::MAX),
    ] {
        for key in ["depthBias", "slopeScaledDepthBias"] {
            let text = versioned(&format!(r#""{key}": {number}, "blendSrc": "Nope""#));
            // The finding after the clamped number keeps its place.
            assert_eq!(
                found(&text),
                [
                    invalid(&format!("x/{key}")),
                    finding("InvalidValue", Error, "x/blendSrc", "invalid value")
                ],
                "{key} {number}"
            );
            let read = x(&versioned(&format!(r#""{key}": {number}"#)));
            let value = if key == "depthBias" {
                read.depth_bias
            } else {
                read.slope_scaled_depth_bias
            };
            assert_eq!(value, Some(expected), "{key} {number}");
        }
    }
    // The largest float itself, and an integer, are read as they are.
    let max = versioned(r#""depthBias": 3.4028234e38, "slopeScaledDepthBias": 7"#);
    assert_eq!(found(&max), []);
    assert_eq!(x(&max).depth_bias, Some(f32::MAX));
    assert_eq!(x(&max).slope_scaled_depth_bias, Some(7.0));
}

#[test]
fn a_bias_of_the_largest_double_as_written_is_beyond_the_floats_like_any_other() {
    // The largest double is also what a number past the doubles is read as (`1e999`), whose
    // finding the clamp makes. Written out, it is a number like `1e39`: one finding, the
    // reader's, and the largest float of its sign.
    let invalid = |at: &str| finding("InvalidValue", Info, at, "invalid value");
    for (number, expected) in [
        ("1.7976931348623157e308", f32::MAX),
        ("-1.7976931348623157e308", f32::MIN),
    ] {
        for (key, other) in [
            ("depthBias", "slopeScaledDepthBias"),
            ("slopeScaledDepthBias", "depthBias"),
        ] {
            for form in [versioned, legacy] {
                let text = form(&format!(r#""{key}": {number}"#));
                assert_eq!(
                    found(&text),
                    one("InvalidValue", Info, &format!("x/{key}"), "invalid value"),
                    "{key} {number}"
                );
                let read = x(&text);
                let value = if key == "depthBias" {
                    read.depth_bias
                } else {
                    read.slope_scaled_depth_bias
                };
                assert_eq!(value, Some(expected), "{key} {number}");
                // Beside a number past the doubles: one finding each, the clamp's first.
                let both = form(&format!(r#""{key}": {number}, "{other}": 1e999"#));
                assert_eq!(
                    found(&both),
                    [invalid(&format!("x/{other}")), invalid(&format!("x/{key}"))],
                    "{key} {number}"
                );
            }
        }
    }
}

#[test]
fn a_member_named_extra_or_key_rules_is_an_unknown_member_like_any_other() {
    let text = versioned(r#""extra": 1, "key_rules": "Legacy", "version": "9.9.9""#);
    assert_eq!(found(&text), []);
    let read = x(&text);
    assert_eq!(read.key_rules, None);
    assert_eq!(read.extra.iter().count(), 3);
}

#[test]
fn findings_come_in_the_order_of_the_file() {
    let text = wrapped(
        r#""b": {"+states": ["Nope"]}, "a": {"varient": []}, "c": {"states": ["PolygonOffset"]}"#,
    );
    let found: Vec<String> = rendered(file(&text).diagnostics().iter())
        .into_iter()
        .map(|finding| finding.2)
        .collect();
    assert_eq!(
        found,
        [
            "materials/b/+states/0",
            "materials/a/varient",
            "materials/c/states/0"
        ]
    );
}

#[test]
fn the_stencil_reference_override_takes_a_number_as_the_clients_as_bool_does() {
    // Vanilla's own `ui.material` writes `"stencilRefOverride": 0`: the game's bool conversion reads a number
    // as "not zero". The value keeps what was written.
    let number = StencilRefOverride::Number;
    for (value, held, on) in [
        ("0", number(0.0), false),
        ("1", number(1.0), true),
        ("-1", number(-1.0), true),
        ("2.5", number(2.5), true),
        ("0.0", number(0.0), false),
        ("true", StencilRefOverride::Flag(true), true),
        ("false", StencilRefOverride::Flag(false), false),
    ] {
        let text = versioned(&format!(r#""stencilRefOverride": {value}"#));
        assert_eq!(found(&text), [], "{value}");
        let read = x(&text).stencil_ref_override;
        assert_eq!(read, Some(held), "{value}");
        assert_eq!(read.map(StencilRefOverride::is_on), Some(on), "{value}");
    }
    for value in ["\"true\"", "[]", "{}"] {
        let text = versioned(&format!(r#""stencilRefOverride": {value}"#));
        assert_eq!(
            found(&text),
            [finding(
                "InvalidType",
                Info,
                "x/stencilRefOverride",
                "wrong JSON type"
            )],
            "{value}"
        );
    }
}
