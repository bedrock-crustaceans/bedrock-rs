//! Fixtures `m01`–`m11` and `l07`: which members are
//! Molang, at which version, where they compile and what a rejected one does. Resource pack at
//! `min_engine_version` 1.21.0 unless the row says otherwise.

use bedrock_addon::diagnostic::Severity;
#[cfg(feature = "molang")]
use bedrock_addon::document::Content;
use bedrock_addon::molang::{Molang, RawVersion, pack_molang_version};
use bedrock_addon::version::{FormatVersion, MinEngineVersion};
use bedrock_addon::visual::HeldItemScale;

use super::common::*;

/// The Molang version of a pack at `min_engine_version` `a.b.c`.
fn pack_version(a: u16, b: u16, c: u16) -> RawVersion {
    pack_molang_version(Some(&MinEngineVersion::new(a, b, c)))
}

const INVALID: &str = "InvalidMolang";

// ---- versions and shapes: with and without the compile check ----

#[test]
fn m05_numeric_and_bool_conditions() {
    let members = r#""animations": {"walk": "animation.walk", "idle": "animation.idle"}, "scripts": {"animate": [{"walk": 1}, {"idle": true}]}, "render_controllers": [{"controller.render.x": false}]"#;
    let document = loaded(attachable(&ghost("1.10.0", members), &cx()));
    silent(document.diagnostics());
    let scripts = document.description.scripts.as_ref().unwrap();
    let animate = scripts.animate.as_ref().unwrap();
    assert_eq!(animate[0].condition(), Some(&Molang::Const(1.0)));
    assert_eq!(animate[1].condition(), Some(&Molang::Bool(true)));
    let controllers = document.description.render_controllers.as_ref().unwrap();
    assert_eq!(controllers[0].condition(), Some(&Molang::Bool(false)));
}

#[test]
fn m06_numeric_named_script() {
    let document = loaded(attachable(
        &ghost(
            "1.10.0",
            r#""scripts": {"scale": 0.9375, "scaleX": 2, "scalex": "3.0"}"#,
        ),
        &cx(),
    ));
    // Nothing at warning level (without the `molang` feature the string script is noted as
    // not checked).
    clean(document.diagnostics());
    let scripts = document.description.scripts.as_ref().unwrap();
    assert_eq!(scripts.scale, Some(Molang::Const(0.9375)));
    // A number builds no script.
    assert!(scripts.scale.as_ref().unwrap().source().is_none());
    assert_eq!(scripts.effective_scale_x().unwrap().as_str(), "3.0");
}

#[test]
fn every_expression_takes_the_pack_s_molang_version() {
    let members = r#""animations": {"walk": "animation.walk"}, "held_item_scale": "query.is_baby ? 0.5 : 1.0", "render_controllers": [{"controller.render.x": "query.is_baby"}], "scripts": {"scale": "1.5", "animate": [{"walk": "query.is_moving"}]}"#;
    for (a, b, c) in [(1, 13, 0), (1, 21, 0), (1, 26, 50)] {
        let document = loaded(attachable(&ghost("1.26.0", members), &rp(a, b, c)));
        let expected = Some(pack_version(a, b, c));
        let d = &document.description;
        let scripts = d.scripts.as_ref().unwrap();
        let sources = [
            scripts.scale.as_ref().unwrap().source().unwrap(),
            scripts.animate.as_ref().unwrap()[0]
                .condition()
                .unwrap()
                .source()
                .unwrap(),
            d.render_controllers.as_ref().unwrap()[0]
                .condition()
                .unwrap()
                .source()
                .unwrap(),
        ];
        for source in sources {
            assert_eq!(source.raw_version(), expected, "{}", source.as_str());
        }
        // `held_item_scale` is built without a version by its reader; the load gives
        // it the pack's.
        let Some(HeldItemScale::Molang(Molang::Expr(scale))) = &d.held_item_scale else {
            panic!("{:?}", d.held_item_scale);
        };
        assert_eq!(scale.raw_version(), expected);
    }
    // `format_version` and `description.min_engine_version` select no Molang version.
    let old = loaded(attachable(
        &ghost(
            "1.10.0",
            r#""min_engine_version": "1.8.0", "scripts": {"scale": "1.5"}"#,
        ),
        &rp(1, 21, 0),
    ));
    let scale = old.description.scripts.as_ref().unwrap().scale.as_ref();
    assert_eq!(
        scale.unwrap().source().unwrap().raw_version(),
        Some(pack_version(1, 21, 0))
    );
}

#[test]
fn an_object_form_expression_is_a_wrong_type() {
    // No member takes `{expression, version}`: the schema's node is a number, a boolean or a
    // string.
    for members in [
        r#""scripts": {"scale": {"expression": "1.0", "version": 5}}"#,
        r#""scripts": {"pre_animation": [{"expression": "1.0", "version": 5}]}"#,
        r#""render_controllers": [{"controller.render.x": {"expression": "1.0", "version": 5}}]"#,
    ] {
        let d = rejected(attachable(&ghost("1.10.0", members), &cx()));
        assert!(
            find(&d, "InvalidType", Severity::Error).is_some(),
            "{members}"
        );
    }
}

#[expect(clippy::float_cmp, reason = "the reader gives exact values")]
#[test]
fn m11_held_item_scale_molang_shapes() {
    // A string and a boolean are Molang from engine 1.26.30; before it the schema's float.
    let string = r#""held_item_scale": "query.is_on_screen ? 1.5 : 1.0""#;
    let document = loaded(attachable(&ghost("1.26.0", string), &cx()));
    assert!(matches!(
        document.description.held_item_scale,
        Some(HeldItemScale::Molang(Molang::Expr(_)))
    ));
    // A Molang scale has no constant value.
    assert_eq!(document.held_item_scale(), 1.0);
    none(document.diagnostics(), "InvalidValue");
    let boolean = loaded(attachable(
        &ghost("1.26.0", r#""held_item_scale": true"#),
        &cx(),
    ));
    assert_eq!(
        boolean.description.held_item_scale,
        Some(HeldItemScale::Molang(Molang::Bool(true)))
    );
    let mut first = cx();
    first.engine_version = FormatVersion::new(1, 26, 30);
    loaded(attachable(&ghost("1.26.0", string), &first));
    let mut older = cx();
    older.engine_version = FormatVersion::new(1, 26, 20);
    let d = rejected(attachable(&ghost("1.26.0", string), &older));
    expect(
        &d,
        "InvalidType",
        Severity::Error,
        "unknown child schema option type.  Allowed types: float",
    );
    // A number is a number at every engine.
    loaded(attachable(
        &ghost("1.26.0", r#""held_item_scale": 2"#),
        &older,
    ));
    // Null, an array and an object are no scale.
    for value in ["null", "[1]", "{}"] {
        let members = format!(r#""held_item_scale": {value}"#);
        // A `null` too: no type option of the schema takes it (`nulls.rs`).
        let d = rejected(attachable(&ghost("1.26.0", &members), &cx()));
        assert!(
            find(&d, "InvalidType", Severity::Error).is_some(),
            "{value}"
        );
    }
}

#[test]
fn m03_array_concatenated() {
    let members = r#""scripts": {"pre_animation": ["variable.a = 1;", "variable.b = query.is_local_player;"], "initialize": ["variable.c = 2;"]}"#;
    let document = loaded(attachable(&ghost("1.26.0", members), &cx()));
    assert_eq!(
        document.pre_animation().as_deref(),
        Some("variable.a = 1;variable.b = query.is_local_player;")
    );
    assert_eq!(document.initialize().as_deref(), Some("variable.c = 2;"));
    clean(document.diagnostics());
    // Absent, or nothing but empty strings: no script.
    let none = loaded(attachable(
        &ghost("1.26.0", r#""scripts": {"initialize": [""]}"#),
        &cx(),
    ));
    assert_eq!(none.initialize(), None);
    assert_eq!(none.pre_animation(), None);
}

#[test]
fn m09_lowercase_quoted_literal() {
    let members = r#""scripts": {"pre_animation": ["variable.n = 'Foo';"]}"#;
    let old = loaded(attachable(&ghost("1.16.0", members), &cx()));
    assert_eq!(old.pre_animation().as_deref(), Some("variable.n = 'foo';"));
    clean(old.diagnostics());
    let new = loaded(attachable(&ghost("1.18.20", members), &cx()));
    assert_eq!(new.pre_animation().as_deref(), Some("variable.n = 'Foo';"));
    clean(new.diagnostics());
}

// ---- the compile check ----

#[cfg(feature = "molang")]
mod compiled {
    use super::*;

    #[test]
    fn every_named_script_is_compiled_at_its_schema_position() {
        // The eleven members the schema types as Molang, each on its own: a text the parser
        // rejects fails the file at that member (found by a mutant that left one unwalked).
        for name in [
            "scale",
            "scaleX",
            "scalex",
            "scaleY",
            "scaley",
            "scaleZ",
            "scalez",
            "parent_setup",
            "should_update_bones_and_effects_offscreen",
            "should_update_effects_offscreen",
            "hide_held_items",
        ] {
            let members = format!(r#""scripts": {{"{name}": "query.is_baby +"}}"#);
            let d = rejected(attachable(&ghost("1.26.0", &members), &cx()));
            expect_at(
                &d,
                INVALID,
                Severity::Error,
                &format!("minecraft:attachable/description/scripts/{name}"),
            );
            let members = format!(r#""scripts": {{"{name}": "query.is_baby"}}"#);
            silent(loaded(attachable(&ghost("1.26.0", &members), &cx())).diagnostics());
        }
    }

    #[test]
    fn m01_string_script_client_only_query() {
        let members = r#""scripts": {"scale": "query.distance_from_camera > 16 ? 0.5 : 1.0"}"#;
        let document = loaded(attachable(&ghost("1.10.0", members), &cx()));
        silent(document.diagnostics());
    }

    #[test]
    fn m02_is_on_screen() {
        let members = r#""scripts": {"should_update_effects_offscreen": "!query.is_on_screen"}"#;
        let document = loaded(attachable(&ghost("1.26.0", members), &cx()));
        silent(document.diagnostics());
    }

    #[test]
    fn m03_the_joined_text_compiles_clean() {
        let members = r#""scripts": {"pre_animation": ["variable.a = 1;", "variable.b = query.is_local_player;"]}"#;
        silent(loaded(attachable(&ghost("1.26.0", members), &cx())).diagnostics());
    }

    #[test]
    fn m04_array_element_schema_compile() {
        let members = r#""scripts": {"pre_animation": ["{ variable.a = 1;", "};"]}"#;
        // 1.10.0: each element is parsed on its own, and the first is no expression.
        let d = rejected(attachable(&ghost("1.10.0", members), &cx()));
        expect_at(
            &d,
            INVALID,
            Severity::Error,
            "minecraft:attachable/description/scripts/pre_animation/0",
        );
        none(&d, "DroppedField");
        // So is a 1.8.0 one, which passes the same schema node.
        let d = rejected(attachable(&ghost("1.8.0", members), &cx()));
        assert!(find(&d, INVALID, Severity::Error).is_some());
        // 1.26.0: plain strings, joined into one valid block.
        let document = loaded(attachable(&ghost("1.26.0", members), &cx()));
        clean(document.diagnostics());
        assert_eq!(
            document.pre_animation().as_deref(),
            Some("{ variable.a = 1;};")
        );
    }

    #[test]
    fn m07_bad_condition_schema() {
        let members = r#""animations": {"walk": "animation.walk"}, "scripts": {"animate": [{"walk": "q.is_baby +"}]}"#;
        for version in ["1.10.0", "1.26.0"] {
            let d = rejected(attachable(&ghost(version, members), &cx()));
            expect_at(
                &d,
                INVALID,
                Severity::Error,
                "minecraft:attachable/description/scripts/animate/0/walk",
            );
            // The schema rejects it: the loader's "Failed to parse" line never comes.
            none(&d, "DroppedField");
        }
        let members = r#""render_controllers": [{"controller.render.x": "q.is_baby +"}]"#;
        let d = rejected(attachable(&ghost("1.10.0", members), &cx()));
        expect_at(
            &d,
            INVALID,
            Severity::Error,
            "minecraft:attachable/description/render_controllers/0/controller.render.x",
        );
        // A named script is a schema node too.
        let d = rejected(attachable(
            &ghost("1.26.0", r#""scripts": {"scale": "1 +"}"#),
            &cx(),
        ));
        expect_at(
            &d,
            INVALID,
            Severity::Error,
            "minecraft:attachable/description/scripts/scale",
        );
    }

    /// Two `InvalidMolang` at one path are two lines of the parser, each told once: every
    /// error line becomes one finding with its own message (the severity mapping).
    #[test]
    fn m07_a_rejected_condition_is_told_once_per_parser_line() {
        let members = r#""render_controllers": [{"controller.render.x": "q.not_a_query"}]"#;
        let d = rejected(attachable(&ghost("1.10.0", members), &cx()));
        let told: Vec<String> = d
            .iter()
            .filter(|f| f.code.name() == INVALID)
            .map(|f| {
                assert_eq!(
                    f.location.json_path.to_string(),
                    "minecraft:attachable/description/render_controllers/0/controller.render.x"
                );
                f.to_string()
            })
            .collect();
        assert_eq!(told.len(), 2, "{told:?}");
        assert!(told[0].contains(
            "Failed to resolve query query.not_a_query.  Either the query does not exist or it is not supported in this context."
        ));
        assert!(told[1].contains("unrecognized token: q.not_a_query"));
        // No line is told twice.
        assert_ne!(told[0], told[1]);
    }

    #[test]
    fn m08_unknown_query_script() {
        let members = r#""scripts": {"pre_animation": ["variable.x = query.not_a_query;"], "initialize": ["variable.y = 1;"], "scale": "2.0"}"#;
        let document = loaded(attachable(&ghost("1.26.0", members), &cx()));
        let d = document.diagnostics();
        // The parser's lines first, as kept errors (the file loads), then the
        // loader's line.
        let invalid: Vec<_> = d.iter().filter(|f| f.code.name() == INVALID).collect();
        assert!(!invalid.is_empty());
        assert!(
            invalid
                .iter()
                .all(|f| f.severity == Severity::Error && f.kept)
        );
        assert!(
            invalid.iter().any(|f| f.to_string().contains(
                "Failed to resolve query query.not_a_query.  Either the query does not exist or it is not supported in this context."
            )),
            "{}",
            lines(d).join("\n")
        );
        expect(
            d,
            "DroppedField",
            Severity::Error,
            "Failed to parse 'pre_animation'!",
        );
        let dropped = d
            .iter()
            .position(|f| f.code.name() == "DroppedField")
            .unwrap();
        let first_invalid = d.iter().position(|f| f.code.name() == INVALID).unwrap();
        assert!(first_invalid < dropped);
        // That script is dropped; the others stay.
        let scripts = document.description.scripts.as_ref().unwrap();
        assert_eq!(scripts.pre_animation, None);
        assert_eq!(document.pre_animation(), None);
        assert_eq!(document.initialize().as_deref(), Some("variable.y = 1;"));
        assert!(scripts.scale.is_some());
    }

    #[test]
    fn l07_bad_script() {
        let members = r#""scripts": {"pre_animation": ["1 +"], "initialize": ["1 +"], "parent_setup": "variable.a = 1;"}"#;
        let document = loaded(attachable(&ghost("1.26.0", members), &cx()));
        let d = document.diagnostics();
        expect(
            d,
            "DroppedField",
            Severity::Error,
            "Failed to parse 'pre_animation'!",
        );
        expect(
            d,
            "DroppedField",
            Severity::Error,
            "Failed to parse 'initialize'!",
        );
        expect_at(
            d,
            "DroppedField",
            Severity::Error,
            "minecraft:attachable/description/scripts/pre_animation",
        );
        // The parser's own lines before "Failed to parse": content-log errors the load goes
        // past, kept errors.
        let parser: Vec<(Severity, bool)> = d
            .iter()
            .filter(|f| f.code.name() == "InvalidMolang")
            .map(|f| (f.severity, f.kept))
            .collect();
        assert_eq!(
            parser,
            [(Severity::Error, true); 2],
            "{}",
            lines(d).join("\n")
        );
        let scripts = document.description.scripts.as_ref().unwrap();
        assert_eq!(scripts.pre_animation, None);
        assert_eq!(scripts.initialize, None);
        assert!(scripts.parent_setup.is_some());
        // At 1.10.0 the element fails the schema's own parse and the file is skipped.
        let d = rejected(attachable(&ghost("1.10.0", members), &cx()));
        assert!(find(&d, INVALID, Severity::Error).is_some());
    }

    #[test]
    fn m10_pack_version_zero() {
        let members = r#""scripts": {"scale": "query.distance_from_camera > 16 ? 0.5 : 1.0"}"#;
        let document = loaded(attachable(&ghost("1.10.0", members), &rp(1, 13, 0)));
        silent(document.diagnostics());
        let scale = document
            .description
            .scripts
            .as_ref()
            .unwrap()
            .scale
            .as_ref();
        assert_eq!(
            scale.unwrap().source().unwrap().raw_version(),
            Some(RawVersion(0))
        );
    }

    #[test]
    fn m11_held_item_scale_molang() {
        let good = r#""held_item_scale": "query.is_on_screen ? 1.5 : 1.0""#;
        silent(loaded(attachable(&ghost("1.26.0", good), &cx())).diagnostics());
        // A text the parser rejects fails the v1_26 schema's typed `ExpressionNode` child
        // and the loader skips the file: a refusal.
        let bad = r#""held_item_scale": "1 +""#;
        let d = rejected(attachable(&ghost("1.26.0", bad), &cx()));
        expect_at(
            &d,
            INVALID,
            Severity::Error,
            "minecraft:attachable/description/held_item_scale",
        );
    }

    #[test]
    fn a_dropped_script_is_not_written_back_and_the_rest_round_trips() {
        let members = r#""scripts": {"pre_animation": ["1 +"], "scale": "2.0"}"#;
        let document = loaded(attachable(&ghost("1.26.0", members), &cx()));
        let text = document.to_json().unwrap();
        assert!(!text.contains("pre_animation"), "{text}");
        assert!(text.contains("\"scale\""), "{text}");
    }
}

#[cfg(not(feature = "molang"))]
#[test]
fn without_the_molang_feature_nothing_compiles() {
    // "Without the `molang` feature nothing is compiled and rules 8 (Molang
    // part) and 20 are not checked".
    let members = r#""scripts": {"pre_animation": ["1 +"], "scale": "1 +"}"#;
    for version in ["1.10.0", "1.26.0"] {
        let document = loaded(attachable(&ghost(version, members), &cx()));
        none(document.diagnostics(), INVALID);
        none(document.diagnostics(), "DroppedField");
        assert!(find(document.diagnostics(), "MolangNotChecked", Severity::Info).is_some());
        assert!(document.pre_animation().is_some());
    }
}
