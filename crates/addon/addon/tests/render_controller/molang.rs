//! `rc20` and `rc34`–`rc41`

use bedrock_addon::diagnostic::Severity;

use super::common::{count, find, loaded, unchecked};
use super::rows::{parse, test_of, with};

/// RC.8 (c), TOML `vanilla = "compile failure skips the file … or the controller"`: without the
/// compile check the texts are stored unchecked and one info says so.
#[test]
fn without_the_compile_check_molang_is_stored_unchecked_deviation() {
    let document = loaded(parse(
        &with(r#""light_color_multiplier": "1 +""#),
        &unchecked(),
    ));
    assert_eq!(
        count(document.diagnostics(), "MolangNotChecked", Severity::Info),
        1
    );
    let info = find(document.diagnostics(), "MolangNotChecked", Severity::Info).unwrap();
    // geometry, one material, one texture and the multiplier.
    // The text names the cause that holds: the feature is off, or it is on and the load was
    // given no check (K98).
    assert_eq!(
        info.code.to_string(),
        if cfg!(feature = "molang") {
            "4 Molang field(s) not checked: no Molang check was given (`LoadOptions::molang` is `None`)"
        } else {
            "4 Molang field(s) not checked: the `molang` feature is off"
        }
    );
    assert_eq!(
        test_of(&document)
            .light_color_multiplier
            .as_ref()
            .unwrap()
            .source()
            .unwrap()
            .as_str(),
        "1 +"
    );
}

#[cfg(feature = "molang")]
mod checked {
    use bedrock_addon::diagnostic::Severity;
    use bedrock_addon::molang::{Molang, RawVersion};

    use super::super::common::{
        clean, context, count, expect, expect_line, lines, loaded, rejected, round_trip, rp, silent,
    };
    use super::super::rows::{OK, file, parse, test_of, with};

    /// `controller.render.test` with `members` only, beside the valid sibling.
    fn beside_ok(members: &str) -> String {
        file(&format!(r#""controller.render.test": {{{members}}}, {OK}"#))
    }

    #[test]
    fn rc20_molang_compile_error_schema_time_skips_the_file_deviation() {
        let text = file(&format!(
            r"{}, {OK}",
            super::super::rows::test_controller(r#""light_color_multiplier": "1 +""#)
        ));
        let d = rejected(parse(&text, &rp(1, 21, 100)));
        assert!(
            count(&d, "InvalidMolang", Severity::Error) >= 1,
            "{:?}",
            lines(&d)
        );
        assert!(
            d.iter().all(|f| f.severity < Severity::Error
                || f.location.json_path.to_string()
                    == "render_controllers/controller.render.test/light_color_multiplier"),
            "{:?}",
            lines(&d)
        );
    }

    #[test]
    fn rc20_every_schema_typed_member_skips_the_file_deviation() {
        for member in [
            r#""materials": [{"*": "1 +"}]"#,
            r#""part_visibility": [{"*": "1 +"}]"#,
            r#""color": {"g": "1 +"}"#,
            r#""overlay_color": {"a": "1 +"}"#,
            r#""on_fire_color": {"r": "1 +"}"#,
            r#""is_hurt_color": {"b": "1 +"}"#,
            r#""uv_anim": {"offset": [0, "1 +"], "scale": [1, 1]}"#,
            r#""uv_anim": {"offset": [0, 0], "scale": ["1 +", 1]}"#,
        ] {
            let text = beside_ok(&format!(
                r#""geometry": "geometry.default", "textures": ["texture.default"], {member}"#
            ));
            let d = rejected(parse(&text, &rp(1, 21, 100)));
            assert!(
                count(&d, "InvalidMolang", Severity::Error) >= 1,
                "{member}: {:?}",
                lines(&d)
            );
        }
    }

    #[test]
    fn rc20_molang_compile_error_parse_time_drops_the_controller_deviation() {
        for members in [
            r#""geometry": "Geometry.default +", "textures": ["Texture.default"]"#,
            r#""geometry": "Geometry.default", "textures": ["Texture.default", "Texture.a +"]"#,
        ] {
            let document = loaded(parse(&beside_ok(members), &rp(1, 21, 100)));
            assert_eq!(
                document.render_controllers.keys().collect::<Vec<_>>(),
                ["controller.render.ok"],
                "{members}"
            );
            assert!(
                count(document.diagnostics(), "InvalidMolang", Severity::Error) >= 1,
                "{:?}",
                lines(document.diagnostics())
            );
        }
    }

    #[test]
    fn rc34_molang_resources_arrays_are_valid_deviation() {
        let text = file(
            r#""controller.render.test": {"geometry": "Geometry.default",
                "textures": ["Array.skins[query.variant]"],
                "arrays": {"textures": {"Array.skins": ["Texture.default", "Texture.brown"]}},
                "materials": [{"*": "Material.default"}]}"#,
        );
        let document = loaded(parse(&text, &rp(1, 21, 100)));
        silent(document.diagnostics());
        assert_eq!(document.render_controllers.len(), 1);
    }

    #[test]
    fn rc35_molang_value_forms() {
        let text = with(
            r#""color": {"r": 0.5, "g": true, "b": "this", "a": "q.is_baby ? 0.5 : 1"},
               "part_visibility": [{"head": 1}, {"body": false}, {"arm*": "query.is_baby"}],
               "uv_anim": {"offset": [0, "math.mod(q.life_time, 1.0)"], "scale": [1, 1]}"#,
        );
        let document = loaded(parse(&text, &rp(1, 21, 100)));
        silent(document.diagnostics());
        let c = test_of(&document);
        let color = c.color.as_ref().unwrap();
        assert_eq!(color.r, Some(Molang::Const(0.5)));
        assert_eq!(color.g, Some(Molang::Bool(true)));
        assert_eq!(color.b.as_ref().unwrap().source().unwrap().as_str(), "this");
        assert_eq!(c.part_visibility[0].1, Molang::Const(1.0));
        assert_eq!(c.part_visibility[1].1, Molang::Bool(false));
        assert_eq!(
            c.part_visibility[2].1.source().unwrap().raw_version(),
            Some(RawVersion(13))
        );
        let uv = c.uv_anim.as_ref().unwrap();
        assert_eq!(uv.offset[0], Molang::Const(0.0));
        assert_eq!(
            uv.offset[1].source().unwrap().as_str(),
            "math.mod(q.life_time, 1.0)"
        );
        round_trip(&document, |t| parse(t, &rp(1, 21, 100)));
    }

    #[test]
    fn rc36_client_only_query() {
        let text = with(
            r#""part_visibility": [{"head": "q.distance_from_camera < 16"}],
               "light_color_multiplier": "q.camera_rotation(0)""#,
        );
        silent(loaded(parse(&text, &rp(1, 21, 100))).diagnostics());
    }

    #[test]
    fn rc37_is_on_screen() {
        let text = with(r#""light_color_multiplier": "query.is_on_screen ? 1.0 : 0.5""#);
        silent(loaded(parse(&text, &rp(1, 21, 100))).diagnostics());
    }

    const UNRESOLVED: &str = "Failed to resolve query query.is_baby.  Either the query does not exist or it is not supported in this context.";

    /// The crate's note for a pack version that did not parse (`DiagCode::InvalidVersion`).
    const INVALID_VERSION: &str =
        "MolangVersion Invalid (-1): version-0 parser rules, no query resolves";
    /// The crate's note for a query of another version (`DiagCode::QueryDeprecated`).
    const NOT_AT_VERSION: &str = "query.is_baby exists, but not at MolangVersion -1";

    #[test]
    fn rc38_molang_version_invalid_resolves_no_query() {
        // A pack whose `min_engine_version` did not parse: Molang version −1.
        let invalid = context(None);
        assert_eq!(invalid.molang_version, RawVersion(-1));
        let a = with(r#""part_visibility": [{"*": "query.is_baby"}]"#);
        let d = rejected(parse(&a, &invalid));
        expect_line(&d, "InvalidMolang", Severity::Error, UNRESOLVED);
        expect_line(
            &d,
            "InvalidMolang",
            Severity::Error,
            "unrecognized token: query.is_baby",
        );

        let b = beside_ok(
            r#""geometry": "query.is_baby ? geometry.a : geometry.b", "textures": ["texture.default"]"#,
        );
        // The crate's own notes: the version that did not parse, and the query that exists at
        // another version. Neither is a parser line; both stay infos.
        expect_line(&d, "MolangLint", Severity::Info, INVALID_VERSION);
        expect_line(&d, "MolangLint", Severity::Info, NOT_AT_VERSION);

        let document = loaded(parse(&b, &invalid));
        assert_eq!(
            document.render_controllers.keys().collect::<Vec<_>>(),
            ["controller.render.ok"]
        );
        let d = document.diagnostics();
        expect_line(d, "InvalidMolang", Severity::Error, UNRESOLVED);
        expect_line(
            d,
            "InvalidMolang",
            Severity::Error,
            "unrecognized token: query.is_baby",
        );
        expect_line(d, "MolangLint", Severity::Info, INVALID_VERSION);
        expect_line(d, "MolangLint", Severity::Info, NOT_AT_VERSION);
        // The errors are the dropped controller's `geometry` alone.
        assert!(
            d.iter().filter(|f| f.severity >= Severity::Error).all(|f| f
                .location
                .json_path
                .to_string()
                == "render_controllers/controller.render.test/geometry"),
            "{:?}",
            lines(d)
        );
        // The same file in a pack with a version loads whole.
        let document = loaded(parse(&b, &rp(1, 21, 100)));
        clean(document.diagnostics());
        assert_eq!(document.render_controllers.len(), 2);
    }

    #[test]
    fn rc39_array_element_error() {
        let text = beside_ok(
            r#""geometry": "geometry.default", "textures": ["texture.default"],
               "arrays": {"textures": {"Array.skins": ["Texture.default", "Texture.a +"]}}"#,
        );
        let document = loaded(parse(&text, &rp(1, 21, 100)));
        let d = document.diagnostics();
        expect_line(
            d,
            "InvalidMolang",
            Severity::Error,
            "Error: binary Add '+' operator at end of expression",
        );
        expect(
            d,
            "InvalidMolang",
            Severity::Error,
            "Error parsing: texture.a +",
        );
        assert!(
            d.iter().filter(|f| f.severity >= Severity::Error).all(|f| f
                .location
                .json_path
                .to_string()
                == "render_controllers/controller.render.test/arrays/textures/Array.skins/1"),
            "{:?}",
            lines(d)
        );
        assert_eq!(
            document.render_controllers.keys().collect::<Vec<_>>(),
            ["controller.render.ok"]
        );
    }

    #[test]
    fn rc40_kept_warning() {
        let document = loaded(parse(
            &with(r#""light_color_multiplier": "1e""#),
            &rp(1, 21, 100),
        ));
        // The game logs it at level 3 and the expression compiles: a kept error.
        expect_line(
            document.diagnostics(),
            "InvalidMolang",
            Severity::Error,
            "error parsing float string, expected '+' or '-' after 'e':",
        );
        assert!(document.diagnostics().iter().all(|d| !d.rejects()));
        assert_eq!(document.render_controllers.len(), 1);
    }

    /// A render controller keeps molang's argument-count lint for the calls the load passes over
    /// in vanilla's files (`query.anger_level(this)`).
    #[test]
    fn a_render_controller_keeps_the_argument_count_lint_the_load_passes_over() {
        use bedrock_addon::diagnostic::DiagnosticDetail;
        use bedrock_addon::molang::DiagCode;
        let document = loaded(parse(
            &with(r#""light_color_multiplier": "query.anger_level(this)""#),
            &rp(1, 21, 100),
        ));
        let arity: Vec<Severity> = document
            .diagnostics()
            .iter()
            // By legacy name: the variant and the definitions that were it.
            .filter(|d| matches!(d.code.name(), "MolangLint"))
            .filter(|d| {
                matches!(d.detail.as_deref(), Some(DiagnosticDetail::Molang(m)) if m.code() == DiagCode::QueryArity)
            })
            .map(|d| d.severity)
            .collect();
        assert_eq!(
            arity,
            [Severity::Info],
            "{:?}",
            lines(document.diagnostics())
        );
        assert_eq!(document.render_controllers.len(), 1);
    }

    #[test]
    fn rc41_material_number() {
        let text = beside_ok(
            r#""geometry": "geometry.default", "textures": ["texture.default"], "materials": [{"*": 1}]"#,
        );
        // Version 13: the empty text is rejected with the parser's own line.
        let document = loaded(parse(&text, &rp(1, 21, 100)));
        expect_line(
            document.diagnostics(),
            "InvalidMolang",
            Severity::Error,
            "No tokens found in expression",
        );
        assert_eq!(
            document.render_controllers.keys().collect::<Vec<_>>(),
            ["controller.render.ok"]
        );
        // Version 0: no parser line, the crate's `Syntax` note alone; the controller is dropped
        // all the same.
        let document = loaded(parse(&text, &rp(1, 16, 0)));
        assert_eq!(
            document.render_controllers.keys().collect::<Vec<_>>(),
            ["controller.render.ok"]
        );
        let d = document.diagnostics();
        assert_eq!(
            count(d, "InvalidMolang", Severity::Error),
            0,
            "{:?}",
            lines(d)
        );
        assert_eq!(d.at_least(Severity::Error).count(), 0, "{:?}", lines(d));
        expect_line(
            d,
            "MolangLint",
            Severity::Info,
            "empty expression: rejected without a message below MolangVersion 4",
        );
        // The number itself is told by the reader, at the member.
        assert_eq!(count(d, "InvalidType", Severity::Info), 1, "{:?}", lines(d));
        // A written empty string is a schema-typed source: it fails the file, at 13 with the
        // parser's line and at 0, where the parser has none, with one error of the member's
        // severity beside the note.
        let written = text.replace(r#"{"*": 1}"#, r#"{"*": ""}"#);
        rejected(parse(&written, &rp(1, 21, 100)));
        let d = rejected(parse(&written, &rp(1, 16, 0)));
        assert_eq!(
            count(&d, "InvalidMolang", Severity::Error),
            1,
            "{:?}",
            lines(&d)
        );
    }

    /// The errors of a load, as (path, the line's text) pairs in the order they were found.
    fn errors_at(d: &bedrock_addon::diagnostic::Diagnostics) -> Vec<String> {
        d.iter()
            .filter(|f| f.severity >= Severity::Error)
            .map(|f| f.location.json_path.to_string())
            .collect()
    }

    /// the sources `RenderController::parse` compiles are checked per
    /// controller in sorted name order, whatever order the file has them in.
    #[test]
    fn g2a_parsed_sources_are_checked_per_controller_in_sorted_name_order() {
        let bad = |name: &str| {
            format!(
                r#""controller.render.{name}": {{"geometry": "geometry.{name} +", "textures": ["texture.default"]}}"#
            )
        };
        let text = file(&format!(
            "{}, {}, {}, {OK}",
            bad("zebra"),
            bad("ant"),
            bad("moth")
        ));
        let document = loaded(parse(&text, &rp(1, 21, 100)));
        assert_eq!(
            document.render_controllers.keys().collect::<Vec<_>>(),
            ["controller.render.ok"]
        );
        let mut at = errors_at(document.diagnostics());
        at.dedup();
        assert_eq!(
            at,
            [
                "render_controllers/controller.render.ant/geometry",
                "render_controllers/controller.render.moth/geometry",
                "render_controllers/controller.render.zebra/geometry",
            ]
        );
    }

    /// a text is compiled once per pack load, and what the compile found is
    /// told at every path that has the text.
    #[test]
    fn g2a_a_text_compiled_once_is_told_at_every_path_that_has_it() {
        const LINE: &str = "Error: binary Add '+' operator at end of expression";
        let same = |name: &str| {
            format!(
                r#""controller.render.{name}": {{"geometry": "geometry.default", "textures": ["texture.a +", "texture.default", "texture.a +"]}}"#
            )
        };
        let text = file(&format!("{}, {}, {OK}", same("one"), same("two")));
        let cx = rp(1, 21, 100);
        let document = loaded(parse(&text, &cx));
        assert_eq!(
            document.render_controllers.keys().collect::<Vec<_>>(),
            ["controller.render.ok"]
        );
        let d = document.diagnostics();
        for path in [
            "render_controllers/controller.render.one/textures/0",
            "render_controllers/controller.render.one/textures/2",
            "render_controllers/controller.render.two/textures/0",
            "render_controllers/controller.render.two/textures/2",
        ] {
            assert!(
                d.iter().any(|f| f.code.name() == "InvalidMolang"
                    && f.severity == Severity::Error
                    && f.location.json_path.to_string() == path
                    && super::super::common::line(f).contains(LINE)),
                "{path}: {:?}",
                lines(d)
            );
        }
        // A second file of the same load hits the cache and is told the same.
        let again = loaded(parse(&text, &cx));
        assert_eq!(lines(again.diagnostics()), lines(d));
    }
}
