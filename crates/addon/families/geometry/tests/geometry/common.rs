//! What the geometry test files share: contexts, loading one file, finding diagnostics by code
//! and text, round trips, and in-memory resource pack stacks.

#![allow(dead_code, reason = "each test file uses part of the helpers")]

use std::sync::Arc;

use bedrock_addon_core::diagnostic::{Diagnostics, Severity};
use bedrock_addon_core::document::{Document, Rejected};
use bedrock_addon_core::load::{LoadContext, LoadOptions};
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::pack_identity::PackType;
use bedrock_addon_core::version::{FormatVersion, MinEngineVersion};
use bedrock_addon_family_geometry::{Geometry, GeometryFile, load_file};
use uuid::Uuid;

#[allow(unused_imports, reason = "each test file uses part of the helpers")]
pub use bedrock_addon_test_support::{clean, lines, loaded, rejected};

/// A resource pack's context at `min_engine_version` `version` (`None`: absent), the Molang
/// compile check on when the feature is.
pub fn context(version: Option<(u16, u16, u16)>) -> LoadContext {
    let mut cx = LoadContext::for_pack(
        &LoadOptions::default(),
        Uuid::from_u128(7),
        PackType::Resource,
        version.map(|(a, b, c)| MinEngineVersion::new(a, b, c)),
        false,
    );
    cx.molang_version =
        bedrock_addon_core::molang::pack_molang_version(cx.min_engine_version.as_ref());
    #[cfg(feature = "molang")]
    {
        cx.molang = Some(Arc::new(bedrock_addon_core::molang::MolangCheck::new()));
    }
    let _ = Arc::<()>::new;
    cx
}

/// A custom resource pack at `min_engine_version` 1.21.100.
pub fn rp() -> LoadContext {
    context(Some((1, 21, 100)))
}

/// A base-game resource pack.
pub fn base_game() -> LoadContext {
    let mut cx = rp();
    cx.is_base_game_pack = true;
    cx
}

/// A custom resource pack loaded by a game of version `a.b.c`.
pub fn engine(a: u16, b: u16, c: u16) -> LoadContext {
    let mut cx = rp();
    cx.engine_version = FormatVersion::new(a, b, c);
    cx
}

/// Loads `text` as `models/entity/x.json` of a custom pack.
pub fn load(text: &str) -> Result<Document<GeometryFile>, Rejected> {
    load_at("models/entity/x.json", text, &rp())
}

/// Loads `text` as the file at `path` of the pack of `cx`.
pub fn load_at(
    path: &'static str,
    text: &str,
    cx: &LoadContext,
) -> Result<Document<GeometryFile>, Rejected> {
    load_file(&PackPath::from_static(path), text.as_bytes(), cx)
}

/// The first geometry of a file that loads.
#[track_caller]
pub fn first(text: &str) -> Geometry {
    loaded(load(text)).value().geometries[0].clone()
}

/// How many findings have code `name` and `severity`.
pub fn count(d: &Diagnostics, name: &str, severity: Severity) -> usize {
    d.iter()
        .filter(|f| f.code.id() == name && f.severity == severity)
        .count()
}

/// Requires a finding with code `name`, `severity` and the text exactly.
#[track_caller]
pub fn expect(d: &Diagnostics, name: &str, severity: Severity, text: &str) {
    assert!(
        d.iter()
            .any(|f| f.code.id() == name && f.severity == severity && f.code.to_string() == text),
        "no {severity:?} {name} \"{text}\" in\n{}",
        lines(d).join("\n")
    );
}

/// Requires a finding with code `name`, `severity` and the text exactly, at JSON path `at`.
#[track_caller]
pub fn expect_at(d: &Diagnostics, name: &str, severity: Severity, at: &str, text: &str) {
    assert!(
        d.iter().any(|f| f.code.id() == name
            && f.severity == severity
            && f.location.json_path.to_string() == at
            && f.code.to_string() == text),
        "no {severity:?} {name} at {at} \"{text}\" in\n{}",
        lines(d).join("\n")
    );
}

/// The text every file that fails its schema ends with.
pub const DID_NOT_VALIDATE: &str = "geometry file didn't validate";

/// Requires the rejection of a file whose schema validation failed: the finding `name` with
/// `text` at the JSON path `at`, then "geometry file didn't validate" at the root.
#[track_caller]
pub fn fails_schema(
    result: Result<Document<GeometryFile>, Rejected>,
    name: &str,
    at: &str,
    text: &str,
) {
    let d = rejected(result);
    expect_at(&d, name, Severity::Error, at, text);
    expect_at(
        &d,
        "geometry/did_not_validate",
        Severity::Error,
        "",
        DID_NOT_VALIDATE,
    );
}

/// Writes `document`, reads the text again and requires an equal value and an identical second
/// write. The written text is returned.
#[track_caller]
pub fn round_trip(document: &Document<GeometryFile>) -> String {
    let text = document.to_json().unwrap();
    let again = loaded(load(&text));
    assert_eq!(
        again.value().geometries,
        document.value().geometries,
        "{text}"
    );
    assert_eq!(again.value().debug, document.value().debug);
    assert_eq!(again.to_json().unwrap(), text);
    clean(again.diagnostics());
    text
}

/// A one-geometry file of `version` whose only bone has the extra members `bone` (JSON members,
/// with a leading comma).
pub fn with_bone(version: &str, bone: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:geometry": [{{"description": {{"identifier": "geometry.x"}}, "bones": [{{"name": "body"{bone}}}]}}]}}"#
    )
}
