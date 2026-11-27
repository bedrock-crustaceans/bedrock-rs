//! The fixtures m01–m06: bone `binding`, read by the schema's expression node and compiled at
//! the pack's version on the client catalogue.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::molang::Molang;

#[cfg(feature = "molang")]
use crate::common::{DID_NOT_VALIDATE, expect, rp};
use crate::common::{clean, context, count, load, load_at, loaded, rejected, with_bone};

fn binding(value: &str) -> String {
    with_bone("1.16.0", &format!(r#", "binding": {value}"#))
}

fn read(value: &str) -> Molang {
    let document = loaded(load(&binding(value)));
    clean(document.diagnostics());
    document.value().geometries[0].bones()[0]
        .binding
        .clone()
        .unwrap()
}

#[test]
fn m01_binding_forms() {
    let expression = read(r#""q.item_slot_to_bone_name(c.item_slot)""#);
    let source = expression.source().expect("a string is an expression");
    assert_eq!(source.as_str(), "q.item_slot_to_bone_name(c.item_slot)");
    // The pack's `min_engine_version` 1.21.100 is Molang version 13.
    assert_eq!(source.raw_version().map(|v| v.0), Some(13));
    assert!(matches!(read("0"), Molang::Const(x) if x == 0.0));
    assert!(matches!(read("true"), Molang::Bool(true)));
    assert!(read("0").source().is_none() && read("true").source().is_none());
    // A wrong JSON type is the schema's type error. From 1.19.30 it fails the file;
    // below, the lenient validation removes the member (the fixtures m01 has the file rejected at
    // 1.16.0, against the upgrade steps).
    for value in ["{}", "[]", "null"] {
        let document = loaded(load(&binding(value)));
        crate::common::expect_at(
            document.diagnostics(),
            "core/unknown_child_option",
            Severity::Info,
            "minecraft:geometry/0/bones/0/binding",
            "unknown child schema option type.  Allowed types: ExpressionNode",
        );
        assert!(document.value().geometries[0].bones()[0].binding.is_none());
        crate::common::fails_schema(
            load(&binding(value).replace("1.16.0", "1.19.30")),
            "core/unknown_child_option",
            "minecraft:geometry/0/bones/0/binding",
            "unknown child schema option type.  Allowed types: ExpressionNode",
        );
    }
}

#[test]
fn the_binding_version_is_the_pack_s_not_the_file_s() {
    // The file is 1.16.0 in every case (Molang version 0 if the file's were taken); the pack's
    // `min_engine_version` gives 0 up to 1.16.220, 1 from 1.17.0, 13 from 1.21.100, and -1 when
    // the manifest has none (vanilla `MolangVersionMapping`).
    for (pack, version) in [
        (Some((1, 16, 0)), 0),
        (Some((1, 17, 0)), 1),
        (Some((1, 21, 100)), 13),
        (None, -1),
    ] {
        let cx = context(pack);
        let text = binding(r#""1""#);
        let document = loaded(load_at("models/entity/x.json", &text, &cx));
        let binding = document.value().geometries[0].bones()[0].binding.clone();
        let raw = binding
            .unwrap()
            .source()
            .and_then(bedrock_addon_core::molang::MolangSource::raw_version);
        assert_eq!(
            raw,
            Some(bedrock_addon_core::molang::RawVersion(version)),
            "{pack:?}"
        );
        assert_eq!(cx.molang_version.0, version, "{pack:?}");
    }
}

#[cfg(feature = "molang")]
#[test]
fn m02_binding_client_only_query() {
    let document = loaded(load(&binding(
        r#""q.camera_rotation(0) > 0 ? 'rightitem' : 'leftitem'""#,
    )));
    assert!(
        document.diagnostics().is_empty(),
        "{:?}",
        document.diagnostics()
    );
}

#[cfg(feature = "molang")]
#[test]
fn m03_binding_is_on_screen() {
    let document = loaded(load(&binding(
        r#""query.is_on_screen ? 'rightitem' : 'leftitem'""#,
    )));
    assert!(
        document.diagnostics().is_empty(),
        "{:?}",
        document.diagnostics()
    );
}

#[cfg(feature = "molang")]
#[test]
fn m04_binding_compile_error() {
    let text = r#"{"format_version": "1.16.0", "minecraft:geometry": [
        {"description": {"identifier": "geometry.bad"}, "bones": [{"name": "a", "binding": "q.item_slot_to_bone_name(c.item_slot) +"}]},
        {"description": {"identifier": "geometry.good"}, "bones": [{"name": "a"}]}]}"#;
    let d = rejected(load(text));
    let finding = d
        .iter()
        .find(|f| f.code.id() == "core/invalid_molang")
        .expect("the parser's message");
    assert_eq!(finding.severity, Severity::Error);
    assert_eq!(
        finding.location.json_path.to_string(),
        "minecraft:geometry/0/bones/0/binding"
    );
    // The finding's text is the expression; the parser's message follows it in the content-log
    // line, with its leading "Error: ".
    assert_eq!(
        finding.to_string(),
        "[error] app data | models/entity/x.json | minecraft:geometry/0/bones/0/binding | q.item_slot_to_bone_name(c.item_slot) + | Error: binary Add '+' operator at end of expression"
    );
    assert_eq!(count(&d, "core/invalid_molang", Severity::Error), 1);
    expect(
        &d,
        "geometry/did_not_validate",
        Severity::Error,
        DID_NOT_VALIDATE,
    );
    // The same file in the strict era fails the same way.
    let d = rejected(load(&text.replace("1.16.0", "1.21.0")));
    assert_eq!(count(&d, "core/invalid_molang", Severity::Error), 1);
    // A constant is not compiled.
    loaded(load_at("models/entity/x.json", &binding("7"), &rp()));
}

#[cfg(feature = "molang")]
#[test]
fn m05_binding_invalid_pack_version() {
    const AT: &str = "app data | models/entity/x.json | minecraft:geometry/0/bones/0/binding | q.item_slot_to_bone_name(c.item_slot) | ";

    // A pack whose `min_engine_version` is unreadable compiles at version -1, where no query
    // resolves.
    let mut cx = rp();
    cx.molang_version = bedrock_addon_core::molang::RawVersion(-1);
    let d = rejected(load_at(
        "models/entity/x.json",
        &binding(r#""q.item_slot_to_bone_name(c.item_slot)""#),
        &cx,
    ));
    // Every finding of the file, in order: the crate's two notes (`InvalidVersion`,
    // `QueryDeprecated`), the parser's two messages, and the schema's closing line.
    let found: Vec<String> = d
        .iter()
        .map(|f| format!("{:?} {} {f}", f.severity, f.code.id()))
        .collect();
    assert_eq!(
        found,
        [
            format!(
                "Info core/molang_lint [info] {AT}MolangVersion Invalid (-1): version-0 parser rules, no query resolves"
            ),
            format!(
                "Info core/molang_lint [info] {AT}query.item_slot_to_bone_name exists, but not at MolangVersion -1"
            ),
            format!(
                "Error core/invalid_molang [error] {AT}Failed to resolve query query.item_slot_to_bone_name.  Either the query does not exist or it is not supported in this context."
            ),
            format!(
                "Error core/invalid_molang [error] {AT}unrecognized token: q.item_slot_to_bone_name(c.item_slot)"
            ),
            "Error geometry/did_not_validate [error] app data | models/entity/x.json | geometry file didn't validate"
                .to_owned(),
        ]
    );
    expect(
        &d,
        "geometry/did_not_validate",
        Severity::Error,
        DID_NOT_VALIDATE,
    );
}

#[cfg(not(feature = "molang"))]
#[test]
fn m06_binding_feature_off() {
    let document = loaded(load(&binding(r#""1 +""#)));
    let d = document.diagnostics();
    assert_eq!(count(d, "core/molang_not_checked", Severity::Info), 1);
    assert_eq!(d.len(), 1);
    let binding = document.value().geometries[0].bones()[0].binding.clone();
    assert_eq!(
        binding.unwrap().source().map(|s| s.as_str().to_owned()),
        Some("1 +".to_owned())
    );
    let _ = (rejected::<()>, load_at, context);
}

#[cfg(feature = "molang")]
#[test]
fn m06_binding_is_stored_unchecked_without_a_compile_check() {
    // The feature is on but the load has no check: the same outcome as the feature off.
    let mut cx = rp();
    cx.molang = None;
    let document = loaded(load_at("models/entity/x.json", &binding(r#""1 +""#), &cx));
    assert_eq!(
        count(
            document.diagnostics(),
            "core/molang_not_checked",
            Severity::Info
        ),
        1
    );
    assert_eq!(document.diagnostics().len(), 1);
}
