//! A material with its parent chain applied (`Materials::resolve`): the defaults of
//! rules of steps.md MG.5.

use bedrock_addon::families::material::{KeyRules, MaterialGroupKind, ResolvedMaterial};
use bedrock_addon::pack::{PackStack, Resource};

use crate::common::{Files, fixture_text, flat, stack, wrapped};

const COMMON: &str = "materials/common.json";
const FANCY: &str = "materials/fancy.json";
const SAD: &str = "materials/sad.json";
const A: &str = "materials/a.material";
const B: &str = "materials/b.material";
const ENTITY: &str = "materials/entity.material";

/// A stack of one pack whose common list names `a.material` with the content `file`.
fn one(file: &str) -> PackStack<Resource> {
    let listing = format!(r#"[{{"path": "{A}"}}]"#);
    stack(&[&[(COMMON, &listing), (A, file)]])
}

/// The material `name` of the common group of `stack`, resolved.
#[track_caller]
fn common(stack: &PackStack<Resource>, name: &str) -> ResolvedMaterial {
    stack
        .materials()
        .resolve(MaterialGroupKind::Common, name)
        .unwrap_or_else(|| panic!("no material {name}"))
}

/// `name` of a wrapped file of `members`, resolved.
#[track_caller]
fn versioned(members: &str, name: &str) -> ResolvedMaterial {
    common(&one(&wrapped(members)), name)
}

/// `name` of a flat file of `members`, resolved.
#[track_caller]
fn legacy(members: &str, name: &str) -> ResolvedMaterial {
    common(&one(&flat(members)), name)
}

fn owned(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

#[expect(clippy::float_cmp, reason = "the reader gives exact values")]
#[test]
fn a_material_without_a_parent_starts_from_the_clients_defaults() {
    for resolved in [versioned(r#""x": {}"#, "x"), legacy(r#""x": {}"#, "x")] {
        assert!(resolved.states.is_empty());
        assert!(resolved.defines.is_empty());
        assert_eq!(resolved.samplers.len(), 8);
        for sampler in &resolved.samplers {
            assert!(!sampler.set);
            assert_eq!(sampler.texture_filter, "Point");
            assert_eq!(sampler.texture_wrap, "Clamp");
            assert_eq!(sampler.shader_stage, "Pixel");
            assert_eq!(sampler.comparison_func, None);
        }
        assert_eq!(resolved.vertex_shader, "");
        assert_eq!(resolved.fragment_shader, "");
        assert_eq!(resolved.geometry_shader, "");
        assert_eq!(resolved.depth_func, "Less");
        for face in [&resolved.front_face, &resolved.back_face] {
            assert_eq!(face.stencil_func, "Always");
            assert_eq!(face.stencil_fail_op, "Keep");
            assert_eq!(face.stencil_depth_fail_op, "Keep");
            assert_eq!(face.stencil_pass_op, "Keep");
        }
        assert_eq!(resolved.stencil_ref, 0);
        assert!(!resolved.stencil_ref_override);
        assert_eq!(resolved.stencil_read_mask, u32::MAX);
        assert_eq!(resolved.stencil_write_mask, u32::MAX);
        assert_eq!(resolved.blend_src, "SourceAlpha");
        assert_eq!(resolved.blend_dst, "OneMinusSrcAlpha");
        assert_eq!(resolved.alpha_src, "One");
        assert_eq!(resolved.alpha_dst, "OneMinusSrcAlpha");
        assert!(resolved.vertex_fields.is_empty());
        assert_eq!(resolved.depth_bias, 0.0);
        assert_eq!(resolved.slope_scaled_depth_bias, 0.0);
        assert_eq!(resolved.msaa_support, "NonMSAA");
        assert_eq!(resolved.primitive_mode, "TriangleList");
        assert_eq!(resolved.render_target_formats, [owned(&["B8G8R8A8_UNORM"])]);
    }
    assert_eq!(versioned(r#""x": {}"#, "x").key_rules, KeyRules::Versioned);
    assert_eq!(legacy(r#""x": {}"#, "x").key_rules, KeyRules::Legacy);
}

#[test]
fn a_name_the_group_lacks_does_not_resolve() {
    let stack = one(&wrapped(
        r#""x": {"variants": [{"v": {}}]}, "bad": {"blendSrc": "Nope"}"#,
    ));
    let materials = stack.materials();
    assert!(materials.resolve(MaterialGroupKind::Common, "x").is_some());
    for name in ["nope", "x.v", "bad", ""] {
        assert!(
            materials.resolve(MaterialGroupKind::Common, name).is_none(),
            "{name}"
        );
    }
    assert!(materials.resolve(MaterialGroupKind::Fancy, "x").is_none());
}

/// The vanilla lists, entity file and sprite file of the fixtures.
fn vanilla() -> Vec<(&'static str, String)> {
    vec![
        (COMMON, fixture_text("vanilla/common.json")),
        (FANCY, fixture_text("vanilla/fancy.json")),
        (SAD, fixture_text("vanilla/sad.json")),
        (ENTITY, fixture_text("vanilla/entity.material")),
    ]
}

fn over_vanilla(packs: &[Files<'_>]) -> PackStack<Resource> {
    let vanilla = vanilla();
    let vanilla: Vec<(&str, &str)> = vanilla.iter().map(|(p, t)| (*p, t.as_str())).collect();
    let mut all: Vec<Files<'_>> = vec![&vanilla];
    all.extend(packs);
    stack(&all)
}

#[test]
fn m01_every_material_of_the_entity_file_has_fancy_in_the_fancy_group_only() {
    let stack = over_vanilla(&[]);
    let materials = stack.materials();
    for material in materials.fancy.iter() {
        let fancy = materials
            .resolve(MaterialGroupKind::Fancy, &material.name)
            .unwrap();
        assert!(
            fancy.defines.contains(&"FANCY".to_owned()),
            "{}",
            material.name
        );
        let sad = materials
            .resolve(MaterialGroupKind::Sad, &material.name)
            .unwrap();
        assert!(
            !sad.defines.contains(&"FANCY".to_owned()),
            "{}",
            material.name
        );
    }
    let entity = materials
        .resolve(MaterialGroupKind::Fancy, "entity")
        .unwrap();
    assert_eq!(entity.vertex_shader, "shaders/entity.vertex");
    assert_eq!(entity.vertex_shader_stem(), "entity");
    assert_eq!(entity.fragment_shader, "shaders/entity.fragment");
    assert!(entity.defines.contains(&"USE_OVERLAY".to_owned()));
    assert_eq!(entity.key_rules, KeyRules::Versioned);
    assert!(entity.vertex_fields.contains(&"Position".to_owned()));
    assert!(entity.samplers[0].set);
}

#[test]
fn s02_a_packs_member_is_the_resolved_one() {
    let pack = wrapped(r#""entity_alphatest:entity_nocull": {"msaaSupport": "MSAA"}"#);
    let plain = over_vanilla(&[]);
    let stack = over_vanilla(&[&[(ENTITY, &pack)]]);
    let resolve = |stack: &PackStack<Resource>, name: &str| {
        stack
            .materials()
            .resolve(MaterialGroupKind::Sad, name)
            .unwrap()
    };
    assert_eq!(resolve(&stack, "entity_alphatest").msaa_support, "MSAA");
    assert_ne!(resolve(&plain, "entity_alphatest").msaa_support, "MSAA");
    // A child of it inherits the change; everything else is as it was.
    assert_eq!(resolve(&stack, "map").msaa_support, "MSAA");
    let mut expected = resolve(&plain, "entity_alphatest");
    expected.msaa_support = "MSAA".to_owned();
    assert_eq!(resolve(&stack, "entity_alphatest"), expected);
    assert_eq!(resolve(&stack, "entity"), resolve(&plain, "entity"));
}

#[test]
fn s07_the_highest_listing_of_a_path_gives_its_defines() {
    let listing = format!(r#"[{{"path": "{ENTITY}"}}]"#);
    let stack = over_vanilla(&[&[(FANCY, &listing)]]);
    let entity = stack
        .materials()
        .resolve(MaterialGroupKind::Fancy, "entity")
        .unwrap();
    // The pack's entry has no defines; vanilla's `+defines: ["FANCY"]` entry is skipped.
    assert!(!entity.defines.contains(&"FANCY".to_owned()));
    assert!(entity.defines.contains(&"USE_OVERLAY".to_owned()));
}

#[test]
fn s08_the_first_listing_of_a_path_gives_its_defines() {
    let listing = format!(
        r#"[{{"path": "{A}", "defines": ["D"], "+defines": ["ONE", "GONE"], "-defines": ["GONE"]}},
            {{"path": "{A}", "+defines": ["TWO"]}}]"#
    );
    let file = wrapped(r#""a": {}, "b:a": {"+defines": ["OWN"]}, "c:a": {"defines": ["ONLY"]}"#);
    let stack = stack(&[&[(COMMON, &listing), (A, &file)]]);
    assert_eq!(common(&stack, "a").defines, ["D", "ONE"]);
    assert_eq!(common(&stack, "b").defines, ["D", "ONE", "OWN"]);
    assert_eq!(common(&stack, "c").defines, ["ONLY"]);
}

#[test]
fn the_same_file_in_two_groups_starts_from_each_lists_defines() {
    let file = wrapped(r#""a": {}"#);
    let stack = stack(&[&[
        (COMMON, &format!(r#"[{{"path": "{A}"}}]"#)),
        (
            FANCY,
            &format!(r#"[{{"path": "{A}", "+defines": ["FANCY"]}}]"#),
        ),
        (A, &file),
    ]]);
    let materials = stack.materials();
    assert!(
        materials
            .resolve(MaterialGroupKind::Common, "a")
            .unwrap()
            .defines
            .is_empty()
    );
    assert_eq!(
        materials
            .resolve(MaterialGroupKind::Fancy, "a")
            .unwrap()
            .defines,
        ["FANCY"]
    );
}

#[test]
fn m15_an_extension_applies_on_top_and_reaches_only_later_children() {
    let a = wrapped(r#""x": {"+defines": ["BASE"], "depthFunc": "Always"}, "before:x": {}"#);
    let b = wrapped(r#""+x": {"+defines": ["USE_EMISSIVE"]}, "after:x": {}"#);
    let listing = format!(r#"[{{"path": "{A}"}}, {{"path": "{B}"}}]"#);
    let stack = stack(&[&[(COMMON, &listing), (A, &a), (B, &b)]]);
    assert_eq!(common(&stack, "x").defines, ["BASE", "USE_EMISSIVE"]);
    assert_eq!(common(&stack, "x").depth_func, "Always");
    assert_eq!(common(&stack, "after").defines, ["BASE", "USE_EMISSIVE"]);
    assert_eq!(common(&stack, "before").defines, ["BASE"]);
}

#[test]
fn m18_in_a_versioned_material_states_replaces_and_hides_plus_states() {
    let members = r#""p": {"states": ["Blending"]},
                     "c:p": {"states": ["DisableCulling"], "+states": ["Wireframe"]}"#;
    assert_eq!(versioned(members, "c").states, ["DisableCulling"]);
    assert_eq!(versioned(members, "p").states, ["Blending"]);
}

#[test]
fn m19_in_a_legacy_material_states_adds_and_plus_states_is_unread() {
    let members = r#""p": {"states": ["Blending"]},
                     "c:p": {"states": ["DisableCulling"], "+states": ["Wireframe"]}"#;
    assert_eq!(legacy(members, "c").states, ["Blending", "DisableCulling"]);
}

#[test]
fn versioned_states_without_the_plain_key_add_then_remove() {
    let members = r#""p": {"states": ["Blending", "Wireframe", "StencilWrite"]},
                     "c:p": {"+states": ["DisableCulling", "Wireframe"], "-states": ["Wireframe", "Blending"]},
                     "d:p": {"-states": ["StencilWrite"]},
                     "e:p": {"states": []}"#;
    assert_eq!(
        versioned(members, "c").states,
        ["DisableCulling", "StencilWrite"]
    );
    assert_eq!(versioned(members, "d").states, ["Blending", "Wireframe"]);
    assert!(versioned(members, "e").states.is_empty());
    // A legacy material reads neither prefixed key.
    assert_eq!(
        legacy(members, "c").states,
        ["Blending", "StencilWrite", "Wireframe"]
    );
    // An ignored and an unknown state set nothing.
    let odd = r#""x": {"states": ["PolygonOffset", "Nope", "InvertCulling"]}"#;
    assert_eq!(versioned(odd, "x").states, ["InvertCulling"]);
}

#[test]
fn the_alpha_to_coverage_state_inserts_its_define() {
    let members = r#""p": {"+states": ["EnableAlphaToCoverage"]},
                     "kept:p": {"+defines": ["A"]},
                     "cleared:p": {"states": [], "defines": ["A"]}"#;
    assert_eq!(versioned(members, "p").defines, ["ALPHA_TO_COVERAGE"]);
    assert_eq!(
        versioned(members, "kept").defines,
        ["A", "ALPHA_TO_COVERAGE"]
    );
    assert_eq!(versioned(members, "cleared").defines, ["A"]);
    // The states are read before the defines: a `defines` of the same material clears it.
    let same = r#""x": {"states": ["EnableAlphaToCoverage"], "defines": ["A"]}"#;
    assert_eq!(versioned(same, "x").defines, ["A"]);
    assert_eq!(legacy(same, "x").defines, ["A", "ALPHA_TO_COVERAGE"]);
}

#[test]
fn m20_in_a_legacy_material_minus_defines_is_unread() {
    let members = r#""p": {"defines": ["A", "B"]}, "c:p": {"-defines": ["A"]}"#;
    assert_eq!(legacy(members, "c").defines, ["A", "B"]);
    assert_eq!(versioned(members, "c").defines, ["B"]);
}

#[test]
fn versioned_defines_replace_or_add_then_remove_and_legacy_ones_add() {
    let members = r#""p": {"defines": ["A", "B"]},
                     "replace:p": {"defines": ["C"], "+defines": ["D"], "-defines": ["C"]},
                     "prefixed:p": {"+defines": ["C", "A"], "-defines": ["B", "X"]},
                     "plus:p": {"+defines": ["C"]}"#;
    assert_eq!(versioned(members, "replace").defines, ["C"]);
    assert_eq!(versioned(members, "prefixed").defines, ["A", "C"]);
    // Legacy: `defines` inserts without clearing and hides `+defines`; alone, `+defines`
    // inserts; `-defines` is never read.
    assert_eq!(legacy(members, "replace").defines, ["A", "B", "C"]);
    assert_eq!(legacy(members, "prefixed").defines, ["A", "B", "C"]);
    assert_eq!(legacy(members, "plus").defines, ["A", "B", "C"]);
}

#[test]
fn m25_versioned_sampler_states_reset_every_slot_first() {
    let members = r#""p": {"samplerStates": [{"samplerIndex": 1, "textureFilter": "Bilinear", "textureWrap": "Repeat",
                                               "shaderStage": "Vertex", "comparisonFunc": "Less"}]},
                     "reset:p": {"samplerStates": [{"textureFilter": "Trilinear"}]},
                     "adds:p": {"+samplerStates": [{"samplerIndex": 0, "textureFilter": "PCF"}, {"samplerIndex": 8, "textureFilter": "Trilinear"},
                                                   {"samplerIndex": 1, "textureWrap": "Clamp"}]},
                     "hidden:p": {"samplerStates": [], "+samplerStates": [{"samplerIndex": 3}]}"#;
    let parent = versioned(members, "p");
    assert!(parent.samplers[1].set);
    assert_eq!(parent.samplers[1].texture_filter, "Bilinear");
    assert_eq!(parent.samplers[1].texture_wrap, "Repeat");
    assert_eq!(parent.samplers[1].shader_stage, "Vertex");
    assert_eq!(parent.samplers[1].comparison_func.as_deref(), Some("Less"));
    assert!(!parent.samplers[0].set);
    // An entry without an index is skipped, but the plain key still reset the slots.
    let reset = versioned(members, "reset");
    let untouched = versioned(r#""x": {}"#, "x");
    assert_eq!(reset.samplers, untouched.samplers);
    // `+samplerStates` applies onto what is inherited; an index of 8 is skipped.
    let adds = versioned(members, "adds");
    assert_eq!(adds.samplers[0].texture_filter, "PCF");
    assert!(adds.samplers[0].set);
    assert_eq!(adds.samplers[1].texture_filter, "Bilinear");
    assert_eq!(adds.samplers[1].texture_wrap, "Clamp");
    assert_eq!(adds.samplers[1].shader_stage, "Vertex");
    assert!(adds.samplers[2..].iter().all(|sampler| !sampler.set));
    assert_eq!(versioned(members, "hidden").samplers, untouched.samplers);
}

#[test]
fn legacy_sampler_states_apply_onto_the_inherited_slots_and_read_three_members() {
    let members = r#""p": {"samplerStates": [{"samplerIndex": 1, "textureFilter": "Bilinear", "shaderStage": "Vertex",
                                               "comparisonFunc": "Less"}]},
                     "c:p": {"samplerStates": [{"samplerIndex": 2, "textureWrap": "Repeat"}, {"samplerIndex": 9}],
                             "+samplerStates": [{"samplerIndex": 3}]}"#;
    let child = legacy(members, "c");
    // No reset: the parent's slot stays.
    assert_eq!(child.samplers[1].texture_filter, "Bilinear");
    // The stage and the comparison are not read.
    assert_eq!(child.samplers[1].shader_stage, "Pixel");
    assert_eq!(child.samplers[1].comparison_func, None);
    assert_eq!(child.samplers[2].texture_wrap, "Repeat");
    assert!(child.samplers[2].set);
    // `+samplerStates` is never read.
    assert!(!child.samplers[3].set);
}

#[test]
fn m21_minus_sampler_states_changes_no_slot() {
    let members = r#""p": {"samplerStates": [{"samplerIndex": 0, "textureFilter": "Bilinear"}]},
                     "c:p": {"-samplerStates": [{"samplerIndex": 0}]}"#;
    assert_eq!(
        versioned(members, "c").samplers,
        versioned(members, "p").samplers
    );
    assert_eq!(legacy(members, "c").samplers, legacy(members, "p").samplers);
}

#[test]
fn m23_m24_the_render_target_formats() {
    let default = [owned(&["B8G8R8A8_UNORM"])];
    // The flat form: every element is skipped, the cleared list takes the default.
    let flat_form = r#""x": {"renderTargetFormats": ["R8G8B8A8_UNORM"]}"#;
    assert_eq!(versioned(flat_form, "x").render_target_formats, default);
    // A list is kept; an unknown name is the unknown format.
    let members = r#""p": {"renderTargetFormats": [["R8_UNORM", "nope"], ["D32_FLOAT"]]},
                     "inherits:p": {},
                     "empties:p": {"renderTargetFormats": []},
                     "ignored:p": {"renderTargetFormats": "R8_UNORM"},
                     "animated:p": {"isAnimatedTexture": false, "renderTargetFormats": [["R8_UNORM"]]}"#;
    let listed = [
        owned(&["R8_UNORM", "UNKNOWN_TEXTURE_FORMAT"]),
        owned(&["D32_FLOAT"]),
    ];
    assert_eq!(versioned(members, "p").render_target_formats, listed);
    assert_eq!(versioned(members, "inherits").render_target_formats, listed);
    assert_eq!(versioned(members, "empties").render_target_formats, default);
    // A value that is no array is ignored: the inherited list stays.
    assert_eq!(versioned(members, "ignored").render_target_formats, listed);
    // m24: the member's presence is the test, whatever its value.
    assert_eq!(
        versioned(members, "animated").render_target_formats,
        [owned(&["R8G8B8A8_UNORM"])]
    );
}

#[test]
fn m26_the_stencil_reference_is_the_low_byte_and_turns_the_override_on() {
    let members = r#""x": {"stencilRef": 300}, "c:x": {"stencilRefOverride": false},
                     "max": {"stencilRef": 4294967295, "stencilReadMask": 7, "stencilWriteMask": 0},
                     "only": {"stencilRefOverride": true}"#;
    let resolved = versioned(members, "x");
    assert_eq!(resolved.stencil_ref, 44);
    assert!(resolved.stencil_ref_override);
    // A child can turn it off again and keeps the reference.
    let child = versioned(members, "c");
    assert_eq!((child.stencil_ref, child.stencil_ref_override), (44, false));
    let max = versioned(members, "max");
    assert_eq!(max.stencil_ref, 255);
    assert_eq!((max.stencil_read_mask, max.stencil_write_mask), (7, 0));
    let only = versioned(members, "only");
    assert_eq!((only.stencil_ref, only.stencil_ref_override), (0, true));
}

#[test]
fn an_alpha_factor_copies_the_colour_factor_written_beside_it() {
    let members = r#""p": {"blendSrc": "One", "blendDst": "Zero"},
                     "own:p": {"alphaSrc": "DestAlpha", "alphaDst": "DestColor"},
                     "again:own": {"blendSrc": "SourceColor"},
                     "both:own": {"blendDst": "One", "alphaDst": "Zero"},
                     "none:own": {}"#;
    let parent = versioned(members, "p");
    assert_eq!(
        (parent.blend_src.as_str(), parent.alpha_src.as_str()),
        ("One", "One")
    );
    assert_eq!(
        (parent.blend_dst.as_str(), parent.alpha_dst.as_str()),
        ("Zero", "Zero")
    );
    let own = versioned(members, "own");
    assert_eq!(
        (own.blend_src.as_str(), own.alpha_src.as_str()),
        ("One", "DestAlpha")
    );
    assert_eq!(
        (own.blend_dst.as_str(), own.alpha_dst.as_str()),
        ("Zero", "DestColor")
    );
    // Only a colour factor of the same material is copied; the other keeps what it inherits.
    let again = versioned(members, "again");
    assert_eq!(
        (again.blend_src.as_str(), again.alpha_src.as_str()),
        ("SourceColor", "SourceColor")
    );
    assert_eq!(again.alpha_dst, "DestColor");
    let both = versioned(members, "both");
    assert_eq!(
        (both.blend_dst.as_str(), both.alpha_dst.as_str()),
        ("One", "Zero")
    );
    assert_eq!(versioned(members, "none"), {
        let mut same = own.clone();
        same.key_rules = KeyRules::Versioned;
        same
    });
}

#[test]
fn vertex_fields_are_a_set_that_always_has_the_position() {
    let members = r#""p": {"vertexFields": [{"field": "UV0"}, {"field": "Color"}, {"field": "UV0"}]},
                     "c:p": {"vertexFields": [{"field": "Normal"}, {}]},
                     "empty:p": {"vertexFields": []},
                     "inherits:p": {}"#;
    // In the order of the client's table, each once.
    assert_eq!(
        versioned(members, "p").vertex_fields,
        ["Position", "Color", "UV0"]
    );
    // Replaced, not added to.
    assert_eq!(
        versioned(members, "c").vertex_fields,
        ["Position", "Normal"]
    );
    assert_eq!(versioned(members, "empty").vertex_fields, ["Position"]);
    assert_eq!(
        versioned(members, "inherits").vertex_fields,
        ["Position", "Color", "UV0"]
    );
}

#[expect(clippy::float_cmp, reason = "the reader gives exact values")]
#[test]
fn scalar_members_are_inherited_until_a_material_writes_them() {
    let members = r#""p": {"vertexShader": "shaders/entity.vertex", "fragmentShader": "f", "geometryShader": "g",
                           "depthFunc": "Always", "depthBias": 1.5, "slopeScaledDepthBias": -2,
                           "msaaSupport": "Both", "primitiveMode": "Line",
                           "frontFace": {"stencilFunc": "Equal", "stencilPassOp": "Replace"},
                           "backFace": {"stencilFailOp": "Replace"}},
                     "c:p": {"fragmentShader": "f2", "depthBias": 3, "frontFace": {"stencilDepthFailOp": "Replace"}},
                     "g:c": {"vertexShader": "dir/banner.one.vertex", "primitiveMode": "None"}"#;
    let parent = versioned(members, "p");
    assert_eq!(parent.geometry_shader, "g");
    assert_eq!(parent.depth_bias, 1.5);
    assert_eq!(parent.slope_scaled_depth_bias, -2.0);
    assert_eq!(parent.front_face.stencil_func, "Equal");
    assert_eq!(parent.front_face.stencil_pass_op, "Replace");
    assert_eq!(parent.front_face.stencil_fail_op, "Keep");
    assert_eq!(parent.back_face.stencil_fail_op, "Replace");
    let child = versioned(members, "c");
    assert_eq!(child.vertex_shader, "shaders/entity.vertex");
    assert_eq!(child.fragment_shader, "f2");
    assert_eq!(child.depth_func, "Always");
    assert_eq!(child.depth_bias, 3.0);
    assert_eq!(child.slope_scaled_depth_bias, -2.0);
    assert_eq!(child.msaa_support, "Both");
    // A face written again keeps the members it does not write.
    assert_eq!(child.front_face.stencil_func, "Equal");
    assert_eq!(child.front_face.stencil_depth_fail_op, "Replace");
    assert_eq!(child.back_face, parent.back_face);
    let grandchild = versioned(members, "g");
    assert_eq!(grandchild.fragment_shader, "f2");
    assert_eq!(grandchild.primitive_mode, "None");
    assert_eq!(grandchild.vertex_shader_stem(), "banner");
    assert_eq!(parent.vertex_shader_stem(), "entity");
    assert_eq!(
        versioned(r#""x": {"vertexShader": "plain"}"#, "x").vertex_shader_stem(),
        "plain"
    );
    assert_eq!(versioned(r#""x": {}"#, "x").vertex_shader_stem(), "");
}

#[test]
fn a_single_material_starts_from_its_list_entrys_defines_with_the_legacy_rules() {
    let listing = r#"[{"path": "materials/foo.material", "+defines": ["FANCY"]}]"#;
    let foo = r#"{"vertexShader": "shaders/entity.vertex", "defines": ["A"], "+defines": ["B"], "+states": ["Blending"]}"#;
    let stack = stack(&[&[(COMMON, listing), ("materials/foo.material", foo)]]);
    let resolved = common(&stack, "foo");
    assert_eq!(resolved.key_rules, KeyRules::Legacy);
    assert_eq!(resolved.defines, ["A", "FANCY"]);
    assert!(resolved.states.is_empty());
    assert_eq!(resolved.vertex_shader_stem(), "entity");
}

#[test]
fn a_flat_copy_over_a_wrapped_file_makes_the_materials_it_names_legacy() {
    let low = wrapped(
        r#""p": {"states": ["Blending"]}, "a:p": {"states": ["Wireframe"]}, "b:p": {"states": ["Wireframe"]}"#,
    );
    let high = r#"{"a:p": {}}"#;
    let listing = format!(r#"[{{"path": "{A}"}}]"#);
    let stack = stack(&[&[(COMMON, &listing), (A, &low)], &[(A, high)]]);
    // Legacy `states` adds to the inherited ones; versioned `states` replaces them.
    assert_eq!(common(&stack, "a").states, ["Blending", "Wireframe"]);
    assert_eq!(common(&stack, "a").key_rules, KeyRules::Legacy);
    assert_eq!(common(&stack, "b").states, ["Wireframe"]);
    assert_eq!(common(&stack, "b").key_rules, KeyRules::Versioned);
}

/// Every material of every group of `stack`: `resolve_all` gives what `resolve` gives for
/// each, in creation order.
#[track_caller]
fn assert_resolve_all_agrees(stack: &PackStack<Resource>) -> usize {
    let materials = stack.materials();
    let mut compared = 0;
    for kind in MaterialGroupKind::ALL {
        let all = materials.resolve_all(kind);
        let names: Vec<&str> = materials
            .group(kind)
            .iter()
            .map(|material| material.name.as_str())
            .collect();
        assert_eq!(all.keys().map(String::as_str).collect::<Vec<_>>(), names);
        for (name, resolved) in &all {
            assert_eq!(
                Some(resolved),
                materials.resolve(kind, name).as_ref(),
                "{name}"
            );
            compared += 1;
        }
    }
    compared
}

#[test]
fn resolving_all_gives_what_resolving_each_gives_for_the_vanilla_entity_file() {
    // 99 materials in the common group's files of the fixture are not loaded here; the entity
    // file fills the fancy and the sad group.
    let compared = assert_resolve_all_agrees(&over_vanilla(&[]));
    assert!(compared > 400, "{compared}");
}

#[test]
fn resolving_all_shares_a_parent_between_its_children_and_follows_an_extension() {
    // Siblings under one parent, a chain below one of them, an extension in a later file that
    // reaches only the children created after it, a refused material and a single material.
    let a = wrapped(
        r#""x": {"+defines": ["BASE"], "depthFunc": "Always", "+states": ["Blending"]},
           "before:x": {"states": ["Wireframe"]},
           "left:x": {"+defines": ["L"], "samplerStates": [{"samplerIndex": 1, "textureFilter": "Bilinear"}]},
           "right:x": {"-defines": ["BASE"], "vertexFields": [{"field": "Color"}]},
           "deep:left": {"defines": ["ONLY"], "stencilRef": 258},
           "deeper:deep": {"+defines": ["MORE"]},
           "bad": {"blendSrc": "Nope"}"#,
    );
    let b = wrapped(
        r#""+x": {"+defines": ["USE_EMISSIVE"]}, "after:x": {}, "+left": {"msaaSupport": "MSAA"}"#,
    );
    let single = r#"{"vertexShader": "shaders/one.vertex", "defines": ["S"]}"#;
    let listing = format!(
        r#"[{{"path": "{A}", "+defines": ["LIST"]}}, {{"path": "{B}"}}, {{"path": "materials/one.material"}}]"#
    );
    let stack = stack(&[&[
        (COMMON, &listing),
        (FANCY, &listing),
        (A, &a),
        (B, &b),
        ("materials/one.material", single),
    ]]);
    assert_eq!(assert_resolve_all_agrees(&stack), 2 * 8);
    let all = stack.materials().resolve_all(MaterialGroupKind::Common);
    assert_eq!(all["before"].defines, ["BASE", "LIST"]);
    assert_eq!(all["after"].defines, ["BASE", "LIST", "USE_EMISSIVE"]);
    assert_eq!(all["deeper"].defines, ["MORE", "ONLY"]);
    assert_eq!(all["left"].msaa_support, "MSAA");
    assert_eq!(all["deep"].msaa_support, "NonMSAA");
    assert!(!all.contains_key("bad"));
    assert!(
        stack
            .materials()
            .resolve_all(MaterialGroupKind::Sad)
            .is_empty()
    );
}
