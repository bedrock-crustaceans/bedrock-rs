//! What the geometry test files share: contexts, loading one file, finding diagnostics by code
//! and text, round trips, and in-memory resource pack stacks.

#![allow(dead_code, reason = "each test file uses part of the helpers")]

use std::sync::Arc;

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::document::{Document, Rejected};
use bedrock_addon::families::geometry::{Geometry, GeometryFile, load_file};
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::pack::PackType;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::{FormatVersion, MinEngineVersion};
use uuid::Uuid;

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
    cx.molang_version = bedrock_addon::molang::pack_molang_version(cx.min_engine_version.as_ref());
    #[cfg(feature = "molang")]
    {
        cx.molang = Some(Arc::new(bedrock_addon::molang::MolangCheck::new()));
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

/// Requires a load to succeed.
#[track_caller]
pub fn loaded<T: std::fmt::Debug>(result: Result<Document<T>, Rejected>) -> Document<T> {
    match result {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {}", lines(&rejected.diagnostics).join("\n")),
    }
}

/// Requires a load to fail; its findings.
#[track_caller]
pub fn rejected<T: std::fmt::Debug>(result: Result<Document<T>, Rejected>) -> Diagnostics {
    match result {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// The first geometry of a file that loads.
#[track_caller]
pub fn first(text: &str) -> Geometry {
    loaded(load(text)).value().geometries[0].clone()
}

/// The findings, one line each: severity, code name, path and text.
pub fn lines(d: &Diagnostics) -> Vec<String> {
    d.iter()
        .map(|f| {
            format!(
                "{:?} {} {} {}",
                f.severity,
                f.code.name(),
                f.location.json_path,
                f.code
            )
        })
        .collect()
}

/// How many findings have code `name` and `severity`.
pub fn count(d: &Diagnostics, name: &str, severity: Severity) -> usize {
    d.iter()
        .filter(|f| f.code.name() == name && f.severity == severity)
        .count()
}

/// Requires a finding with code `name`, `severity` and the text exactly.
#[track_caller]
pub fn expect(d: &Diagnostics, name: &str, severity: Severity, text: &str) {
    assert!(
        d.iter()
            .any(|f| f.code.name() == name && f.severity == severity && f.code.to_string() == text),
        "no {severity:?} {name} \"{text}\" in\n{}",
        lines(d).join("\n")
    );
}

/// Requires a finding with code `name`, `severity` and the text exactly, at JSON path `at`.
#[track_caller]
pub fn expect_at(d: &Diagnostics, name: &str, severity: Severity, at: &str, text: &str) {
    assert!(
        d.iter().any(|f| f.code.name() == name
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
    expect_at(&d, "InvalidValue", Severity::Error, "", DID_NOT_VALIDATE);
}

/// Requires no finding at warning level or above.
#[track_caller]
pub fn clean(d: &Diagnostics) {
    let found: Vec<_> = d.at_least(Severity::Warning).collect();
    assert!(found.is_empty(), "{}", lines(d).join("\n"));
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

/// A resource stack read from in-memory packs, highest priority first; each pack is a list of
/// `(path, text)` files and gets a manifest.
pub fn stack(resource: &[&[(&str, &str)]]) -> bedrock_addon::pack::StackSet {
    use crate::responder::{Order, Responder, manifest};
    use bedrock_addon::load::{RepositoryLoader, RootId, Source, StackLoader, StackOptions};
    use bedrock_addon::pack::{OpenOptions, PackOrigin, WorldPacks};

    let mut responder = Responder::new(Order::Issue);
    let mut list = Vec::new();
    for (n, files) in (1_u128..).zip(resource) {
        responder.file(1, &format!("p{n}/manifest.json"), manifest(n, "resources"));
        for (path, text) in *files {
            responder.file(1, &format!("p{n}/{path}"), (*text).to_string());
        }
        list.push(format!(
            r#"{{"pack_id": "{}", "version": [1, 0, 0]}}"#,
            Uuid::from_u128(n)
        ));
    }
    let mut repository = RepositoryLoader::new(
        vec![Source::Directory {
            root: RootId(1),
            origin: PackOrigin::World,
        }],
        OpenOptions::default(),
    );
    let repository = responder.drive(&mut repository);
    let resource = format!("[{}]", list.join(", "));
    let world = WorldPacks::parse(Some(b"[]"), Some(resource.as_bytes())).unwrap();
    let set = repository.stacks(&world, LoadOptions::default());
    let mut loader = StackLoader::new(set, StackOptions::default());
    responder.drive(&mut loader)
}
