//! The forms of a material file read on its own: the wrapper gate, the single-material form, the keys (V2) and the root members that
//! are no material.

use bedrock_addon::diagnostic::Severity::{Error, Info};
use bedrock_addon::families::material::{KeyRules, MaterialFileForm, MaterialKey};

use crate::common::{file, finding, fixture_text, found, parse_at, rendered, wrapped};

/// The keys of a file as written, in order.
fn keys(text: &str) -> Vec<String> {
    file(text)
        .materials
        .keys()
        .map(ToString::to_string)
        .collect()
}

/// The one finding of a wrapper the client does not unwrap.
fn v15() -> Vec<crate::common::Rendered> {
    vec![finding(
        "UnsupportedVersion",
        Info,
        "materials/version",
        "unsupported format_version",
    )]
}

#[test]
fn g01_a_wrapped_1_0_0_file_is_unwrapped() {
    let text = r#"{"materials": {"version": "1.0.0", "a": {"vertexShader": "s"}, "b:a": {}}}"#;
    let document = file(text);
    let MaterialFileForm::Wrapped { version } = &document.form else {
        panic!("not wrapped: {:?}", document.form);
    };
    assert_eq!(version.to_string(), "1.0.0");
    assert_eq!(keys(text), ["a", "b:a"]);
    // `version` is the wrapper's, no material.
    assert!(document.extra.is_empty());
    assert_eq!(found(text), []);
    let key = MaterialKey::parse("b:a");
    assert_eq!(document.key_rules(&key), Some(KeyRules::Versioned));
    assert_eq!(document.materials[&key].key_rules, None);
    assert_eq!(document.key_rules(&MaterialKey::new("nope")), None);
}

// The vanilla file is gdk-retail 1.26.45 `resource_packs/vanilla/materials/sky.material`,
// copied unchanged: the one vanilla file in the flat form.
#[test]
fn g02_the_flat_vanilla_sky_file_is_read_as_the_material_map() {
    let text = fixture_text("vanilla/sky.material");
    let document = file(&text);
    assert_eq!(document.form, MaterialFileForm::Flat);
    assert_eq!(
        keys(&text),
        ["sun_moon", "stars", "cubemap", "skyplane", "end_sky"]
    );
    assert_eq!(found(&text), []);
    let key = MaterialKey::new("sun_moon");
    assert_eq!(document.key_rules(&key), Some(KeyRules::Legacy));
    assert_eq!(
        document.materials[&key].vertex_shader.as_deref(),
        Some("shaders/uv.vertex")
    );
}

#[test]
fn g03_g04_g05_a_wrapper_whose_version_is_not_1_0_0_is_read_flat_with_a_warning_deviation() {
    // vanilla = "no message; root used as flat map, one material named 'materials'": ours
    // warns and creates no such material, keeping the member for write-back.
    for version in [
        r#""version": "1.0.1","#,
        r#""version": "1.1.0","#,
        r#""version": "2.0.0","#,
        r#""version": "0.0.0","#,
        r#""version": "0.9.9","#,
        r#""version": "1.0.0-beta","#,
        r#""version": "","#,
        r#""version": 1,"#,
        r#""version": "one","#,
        "",
    ] {
        let text = format!(r#"{{"materials": {{{version} "a": {{}}}}}}"#);
        let document = file(&text);
        assert_eq!(document.form, MaterialFileForm::Flat, "{text}");
        assert!(document.materials.is_empty(), "{text}");
        assert!(document.extra.get("materials").is_some(), "{text}");
        assert_eq!(found(&text), v15(), "{text}");
    }
}

#[test]
fn an_empty_materials_object_is_no_wrapper() {
    // "root.materials is a non-empty object": an empty one fails the gate like a bad version.
    let text = r#"{"materials": {}}"#;
    let document = file(text);
    assert_eq!(document.form, MaterialFileForm::Flat);
    assert!(document.materials.is_empty());
    assert_eq!(found(text), v15());
}

#[test]
fn a_wrapper_with_only_its_version_is_unwrapped_to_no_material() {
    let text = r#"{"materials": {"version": "1.0.0"}}"#;
    let document = file(text);
    assert!(matches!(document.form, MaterialFileForm::Wrapped { .. }));
    assert!(document.materials.is_empty());
    assert_eq!(found(text), []);
}

#[test]
fn g06_a_second_root_member_makes_the_root_the_material_map_deviation() {
    let text = r#"{"materials": {"version": "1.0.0", "a": {}}, "other": {}}"#;
    let document = file(text);
    assert_eq!(document.form, MaterialFileForm::Flat);
    assert_eq!(keys(text), ["other"]);
    assert!(document.extra.get("materials").is_some());
    assert_eq!(found(text), v15());
    // The other member is a legacy material: the wrapper's version reaches nothing.
    assert_eq!(
        document.key_rules(&MaterialKey::new("other")),
        Some(KeyRules::Legacy)
    );
}

#[test]
fn g09_build_metadata_takes_no_part_in_the_gate() {
    let text = r#"{"materials": {"version": "1.0.0+build.7", "a": {}}}"#;
    let document = file(text);
    let MaterialFileForm::Wrapped { version } = &document.form else {
        panic!("not wrapped: {:?}", document.form);
    };
    assert_eq!(version.build, "build.7");
    assert_eq!(keys(text), ["a"]);
    assert_eq!(found(text), []);
}

#[test]
fn g10_a_root_with_a_string_vertex_shader_is_one_material_named_after_the_file() {
    let text =
        r#"{"vertexShader": "shaders/entity.vertex", "fragmentShader": "shaders/entity.fragment"}"#;
    let document = parse_at("materials/foo.material", text).expect("loads");
    assert_eq!(
        document.form,
        MaterialFileForm::Single {
            stem: "foo".to_owned(),
            wrapper: None
        }
    );
    let key = MaterialKey::new("foo");
    assert_eq!(document.materials.len(), 1);
    let material = &document.materials[&key];
    assert_eq!(
        material.vertex_shader.as_deref(),
        Some("shaders/entity.vertex")
    );
    assert_eq!(
        material.fragment_shader.as_deref(),
        Some("shaders/entity.fragment")
    );
    assert_eq!(document.key_rules(&key), Some(KeyRules::Legacy));
    assert_eq!(rendered(document.diagnostics().iter()), []);
    // The stem is the name without directory and extension.
    let nested = parse_at("materials/sub/bar.baz.material", text).expect("loads");
    assert_eq!(
        nested.form,
        MaterialFileForm::Single {
            stem: "bar.baz".to_owned(),
            wrapper: None
        }
    );
}

#[test]
fn a_vertex_shader_that_is_no_string_does_not_make_the_single_form() {
    // A string check on the value: an object under that name is a material called `vertexShader`.
    let text = r#"{"vertexShader": {"fragmentShader": "f"}}"#;
    let document = file(text);
    assert_eq!(document.form, MaterialFileForm::Flat);
    assert_eq!(keys(text), ["vertexShader"]);
}

#[test]
fn m17_a_key_splits_at_its_last_colon_and_merges_on_its_first() {
    let key = MaterialKey::parse("a:b:c");
    assert_eq!(key.name, "a:b");
    assert_eq!(key.parent.as_deref(), Some("c"));
    assert!(!key.extend);
    assert_eq!(key.base_name(), "a");
    assert_eq!(key.to_string(), "a:b:c");
    let text = wrapped(r#""c": {}, "a:b:c": {}"#);
    let document = file(&text);
    assert!(document.materials.contains_key(&key));
}

#[test]
fn a_key_is_plus_name_colon_parent_and_writes_back_as_written() {
    let plain = MaterialKey::parse("x");
    assert_eq!(plain, MaterialKey::new("x"));
    assert_eq!((plain.extend, plain.parent.clone()), (false, None));
    assert_eq!(plain.base_name(), "x");
    let extend = MaterialKey::parse("+x:p");
    assert!(extend.extend);
    assert_eq!(extend.name, "x");
    assert_eq!(extend.parent.as_deref(), Some("p"));
    // The merge compares the key as written, `+` included.
    assert_eq!(extend.base_name(), "+x");
    for text in [
        "x", "+x", "x:p", "+x:p", "a:b:c", "x:", ":p", "+", "", "++x",
    ] {
        assert_eq!(MaterialKey::parse(text).to_string(), text);
    }
    // Only one `+` is the prefix.
    assert_eq!(MaterialKey::parse("++x").name, "+x");
    assert_eq!(MaterialKey::parse("x:").parent.as_deref(), Some(""));
}

#[test]
fn m02_two_keys_with_one_base_name_are_reported_with_the_clients_text() {
    // the list loader logs it at level 3 and both keys stay in
    // the document: a kept error.
    let text = wrapped(r#""b": {}, "c": {}, "a:b": {}, "a:c": {}"#);
    assert_eq!(
        found(&text),
        [finding(
            "DuplicateIdentifier",
            Error,
            "materials/a:c",
            "Material \"a\" is defined twice in \"materials/test.material\" as \"a:b\" and \"a:c\". You cannot redefine a material.",
        )]
    );
    // Both keys stay.
    assert_eq!(file(&text).materials.len(), 4);
}

#[test]
fn every_later_key_of_a_base_name_is_reported_against_the_first_in_key_order() {
    // The client walks its sorted map: the bytewise first key is "the" definition whatever the
    // order in the file, and the base name is the text before the first colon.
    let text = wrapped(r#""a:z": {}, "a": {}, "a:m:k": {}, "ab": {}, "+a": {}"#);
    let twice = |first: &str, second: &str| {
        finding(
            "DuplicateIdentifier",
            Error,
            &format!("materials/{second}"),
            &format!(
                "Material \"a\" is defined twice in \"materials/test.material\" as \"{first}\" and \"{second}\". You cannot redefine a material."
            ),
        )
    };
    assert_eq!(found(&text), [twice("a", "a:m:k"), twice("a", "a:z")]);
}

#[test]
fn a_root_member_that_is_no_object_is_no_material_deviation() {
    // vanilla creates a material from any member (all defaults); ours keeps the member for
    // write-back, warns and creates none.
    let text = r#"{"a": {}, "format_version": "1.10.0", "n": 5, "l": []}"#;
    let document = file(text);
    assert_eq!(keys(text), ["a"]);
    assert_eq!(
        document
            .extra
            .iter()
            .map(|(key, _)| key)
            .collect::<Vec<_>>(),
        ["format_version", "n", "l"]
    );
    let wrong = |at: &str| finding("InvalidType", Info, at, "wrong JSON type");
    assert_eq!(
        found(text),
        [wrong("format_version"), wrong("n"), wrong("l")]
    );
}

#[test]
fn a_null_root_member_is_kept_and_says_nothing() {
    // The game's patching treats a null as "no change": a pack writes one to leave a material alone.
    let text = r#"{"a": null, "b": {}}"#;
    let document = file(text);
    assert_eq!(keys(text), ["b"]);
    assert!(document.extra.get("a").is_some());
    assert_eq!(found(text), []);
}

#[test]
fn a_copy_that_is_no_json_is_rejected_with_the_clients_text() {
    let rejected = parse_at("materials/x.material", r#"{"a": "#).expect_err("no JSON");
    let findings = rendered(rejected.diagnostics.iter());
    assert_eq!(findings.len(), 1);
    let (code, severity, at, text) = &findings[0];
    assert_eq!(
        (*code, *severity, at.as_str()),
        (
            "MalformedInput",
            bedrock_addon::diagnostic::Severity::Error,
            ""
        )
    );
    assert!(
        text.starts_with("Error parsing materials/x.material :\n "),
        "{text}"
    );
    assert!(text.len() > "Error parsing materials/x.material :\n ".len());
}

#[test]
fn a_root_that_is_no_object_is_rejected() {
    for text in ["[]", "5", "\"x\"", "null"] {
        let rejected = parse_at("materials/x.material", text).expect_err("no object");
        assert_eq!(
            rendered(rejected.diagnostics.iter()),
            [finding(
                "InvalidType",
                bedrock_addon::diagnostic::Severity::Error,
                "",
                "wrong JSON type"
            )],
            "{text}"
        );
    }
}

#[test]
fn comments_a_bom_and_a_member_written_twice_are_read_as_the_client_reads_them() {
    let text = "\u{feff}{ // a comment\n \"a\": {\"vertexShader\": \"one\"}, /* b */ \"a\": {\"vertexShader\": \"two\"} }";
    let document = file(text);
    assert_eq!(document.materials.len(), 1);
    assert_eq!(
        document.materials[&MaterialKey::new("a")]
            .vertex_shader
            .as_deref(),
        Some("two")
    );
    assert_eq!(found(text), []);
}

#[test]
fn a_wrapped_file_whose_map_has_a_string_vertex_shader_is_a_single_material_too() {
    // The client looks for the `vertexShader` after it unwrapped the copy. The single material
    // gets no version stamped on it, so its key rules are the legacy ones.
    let text = r#"{"materials": {"version": "1.0.0", "vertexShader": "s", "+states": []}}"#;
    let document = parse_at("materials/foo.material", text).expect("loads");
    let MaterialFileForm::Single { stem, wrapper } = &document.form else {
        panic!("not single: {:?}", document.form);
    };
    assert_eq!(stem, "foo");
    assert_eq!(
        wrapper.as_ref().map(ToString::to_string).as_deref(),
        Some("1.0.0")
    );
    let key = MaterialKey::new("foo");
    assert_eq!(document.key_rules(&key), Some(KeyRules::Legacy));
    // `+states` is unread in a legacy material.
    assert_eq!(
        rendered(document.diagnostics().iter()),
        [finding(
            "NoEffect",
            Info,
            "materials/+states",
            "member has no effect"
        )]
    );
    // Written back in its wrapper.
    use bedrock_addon::document::Content;
    let written = document.to_json().expect("writes");
    assert!(written.contains("\"version\": \"1.0.0\""), "{written}");
    let again = parse_at("materials/foo.material", &written).expect("reads back");
    assert_eq!(*again, *document);
}

#[test]
fn content_after_the_root_value_is_reported_and_the_file_loads() {
    // The client's reader stops at the end of the root value; ours says what it
    // left unread, once per file.
    let trailing = |at: &str| {
        finding(
            "MalformedInput",
            Info,
            at,
            "content after the JSON value is ignored",
        )
    };
    let text = r#"{"a": {"vertexShader": "s"}} {"b": {}}"#;
    let document = file(text);
    assert_eq!(keys(text), ["a"]);
    assert_eq!(rendered(document.diagnostics().iter()), [trailing("")]);
    let finding = document.diagnostics().iter().next().unwrap();
    let span = finding.location.span.expect("the content's place");
    assert_eq!(
        &text[span.offset as usize..][..span.len as usize],
        r#"{"b": {}}"#
    );
    // White space and comments are no content.
    assert_eq!(found("{\"a\": {}}  // done\n"), []);
    // A list the same.
    let list = crate::common::parse_list("materials/common.json", "[] x").expect("loads");
    assert_eq!(rendered(list.diagnostics().iter()), [trailing("")]);
    // And each copy of a stack, in its own pack.
    let stack = crate::common::stack(&[
        &[
            (
                "materials/common.json",
                "[{\"path\": \"materials/a.material\"}]",
            ),
            ("materials/a.material", r#"{"a": {}}"#),
        ],
        &[
            ("materials/a.material", r#"{"b": {}} true"#),
            ("materials/sad.json", "[] []"),
        ],
    ]);
    let found: Vec<_> = stack
        .materials()
        .diagnostics()
        .map(|f| {
            (
                f.code.name(),
                crate::common::pack_name(f).unwrap(),
                f.location.path.as_ref().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            (
                "MalformedInput",
                "pack 2".to_owned(),
                "materials/sad.json".to_owned()
            ),
            (
                "MalformedInput",
                "pack 2".to_owned(),
                "materials/a.material".to_owned()
            ),
        ]
    );
    assert_eq!(crate::common::names(&stack.materials().common), ["a", "b"]);
}
