//! What the particle tests share: loading one document, contexts, packs, fixture files.

#![allow(dead_code, reason = "each test module uses part of the helpers")]

use std::path::PathBuf;

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::families::particle::ParticleEffect;
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::molang::{Molang, MolangSource, RawVersion};
use bedrock_addon::pack::{OpenOptions, Pack, PackOrigin, PackType};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::MinEngineVersion;
use uuid::Uuid;

/// The file every single-file test loads from.
pub const PATH: &str = "particles/demo.particle.json";

/// The Molang version of the default context's pack (`min_engine_version` 1.21.100).
pub const PACK: RawVersion = RawVersion(13);

/// A resource-pack context with `min_engine_version` `major.minor.patch`.
pub fn pack_context(major: u16, minor: u16, patch: u16) -> LoadContext {
    LoadContext::for_pack(
        &options(),
        Uuid::from_u128(7),
        PackType::Resource,
        Some(MinEngineVersion::new(major, minor, patch)),
        false,
    )
}

/// The load options: with the `molang` feature, the compile check is on.
pub fn options() -> LoadOptions {
    #[allow(unused_mut, reason = "changed with the `molang` feature")]
    let mut options = LoadOptions::default();
    #[cfg(feature = "molang")]
    {
        options.molang = Some(std::sync::Arc::new(
            bedrock_addon::molang::MolangCheck::new(),
        ));
    }
    options
}

/// A resource-pack context of a pack at 1.21.100 (Molang version 13).
pub fn context() -> LoadContext {
    pack_context(1, 21, 100)
}

/// The path every single-file test loads from.
pub fn path() -> PackPath {
    PackPath::from_static(PATH)
}

/// Loads `text` as a particle effect.
pub fn load(text: &str, cx: &LoadContext) -> Result<Document<ParticleEffect>, Rejected> {
    ParticleEffect::parse(&path(), text.as_bytes(), cx)
}

/// Loads `text` and requires it to load.
#[track_caller]
pub fn loads(text: &str, cx: &LoadContext) -> Document<ParticleEffect> {
    match load(text, cx) {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {:#?}", texts(&rejected.diagnostics)),
    }
}

/// Loads `text` in the default context and requires it to load.
#[track_caller]
pub fn loaded(text: &str) -> Document<ParticleEffect> {
    loads(text, &context())
}

/// Loads `text` and requires it to be rejected.
#[track_caller]
pub fn rejected(text: &str) -> Diagnostics {
    match load(text, &context()) {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// The description every test effect has.
pub const DESCRIPTION: &str = r#""description": {"identifier": "demo:spark", "basic_render_parameters": {"material": "particles_alpha", "texture": "textures/particle/particles"}}"#;

/// A lifetime and a rate component: what keeps the legacy parser's two lines away.
pub const BASE: &str = r#""minecraft:emitter_lifetime_once": {}, "minecraft:emitter_rate_instant": {"num_particles": 1}"#;

/// A document of `version` whose payload has `members` after the description.
pub fn document(version: &str, members: &str) -> String {
    format!(r#"{{"format_version": "{version}", "particle_effect": {{{DESCRIPTION}, {members}}}}}"#)
}

/// A document of `version` with the base components and `components` (a JSON object's
/// members; may be empty).
pub fn effect(version: &str, components: &str) -> String {
    let separator = if components.is_empty() { "" } else { ", " };
    document(
        version,
        &format!(r#""components": {{{BASE}{separator}{components}}}"#),
    )
}

/// A document of `version` with the base components and `members` beside them.
pub fn effect_with(version: &str, members: &str) -> String {
    document(version, &format!(r#""components": {{{BASE}}}, {members}"#))
}

/// The two eras the tests name: a legacy version and a cereal one.
pub const LEGACY: &str = "1.10.0";
pub const CEREAL: &str = "1.26.10";

/// The document written back, as JSON.
#[track_caller]
pub fn written(document: &Document<ParticleEffect>) -> facet_value::Value {
    json(&document.to_json().expect("the document writes"))
}

/// `text` as JSON.
#[track_caller]
pub fn json(text: &str) -> facet_value::Value {
    bedrock_addon::json::from_str_jsonc::<facet_value::Value>(text)
        .unwrap_or_else(|e| panic!("{e}: {text}"))
}

/// The value at `pointer` (`/`-separated members and indices) of `value`.
pub fn at<'v>(value: &'v facet_value::Value, pointer: &str) -> Option<&'v facet_value::Value> {
    let mut value = value;
    for step in pointer.split('/').filter(|step| !step.is_empty()) {
        value = match value.as_array() {
            Some(array) => array.get(step.parse::<usize>().ok()?)?,
            None => value.as_object()?.get(step)?,
        };
    }
    Some(value)
}

/// The component `name` a document writes back.
#[track_caller]
pub fn component(document: &Document<ParticleEffect>, name: &str) -> facet_value::Value {
    let written = written(document);
    at(&written, &format!("particle_effect/components/{name}"))
        .cloned()
        .unwrap_or_else(|| panic!("no {name} in {written:?}"))
}

/// The errors.
/// The findings that refuse: a kept error refuses nothing.
pub fn errors(diagnostics: &Diagnostics) -> Vec<&Diagnostic> {
    diagnostics.iter().filter(|d| d.rejects()).collect()
}

/// The findings as `Severity path code-name: text`, for assertions.
pub fn texts(diagnostics: &Diagnostics) -> Vec<String> {
    diagnostics.iter().map(text).collect()
}

/// One finding as `Severity path code-name: text`.
pub fn text(d: &Diagnostic) -> String {
    format!(
        "{:?} {} {}: {}",
        d.severity,
        d.location.json_path,
        d.code.id(),
        d.code
    )
}

/// The findings other than the info that Molang was not checked (a build without `molang`).
pub fn findings(diagnostics: &Diagnostics) -> Vec<String> {
    diagnostics
        .iter()
        .filter(|d| !matches!(d.code.id(), "core/molang_not_checked"))
        .map(text)
        .collect()
}

/// Whether a finding is exactly `expected` (`Severity path code-name: text`).
#[track_caller]
pub fn assert_has(diagnostics: &Diagnostics, expected: &str) {
    let all = texts(diagnostics);
    assert!(
        all.iter().any(|text| text == expected),
        "no {expected:?} in {all:#?}"
    );
}

/// Whether no finding has the text `text` at the end.
#[track_caller]
pub fn assert_lacks(diagnostics: &Diagnostics, text: &str) {
    let all = texts(diagnostics);
    assert!(
        !all.iter().any(|found| found.ends_with(text)),
        "{text:?} in {all:#?}"
    );
}

/// A Molang string as the 1.20.80 reader holds it in the default context.
pub fn expr(text: &str) -> Molang {
    Molang::Expr(MolangSource::string(text, PACK))
}

/// A Molang object form.
pub fn object(text: &str, version: i16) -> Molang {
    Molang::Expr(MolangSource::object(text, RawVersion(version)))
}

/// A Molang constant.
pub fn number(x: f32) -> Molang {
    Molang::Const(x)
}

/// The fixture directory `tests/fixtures/particle/particle/<rest>`.
pub fn fixture_dir(rest: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/particle/particle")
        .join(rest)
}

/// A fixture file's text.
pub fn fixture(rest: &str) -> String {
    let path = fixture_dir(rest);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// A format-2 resource-pack manifest.
pub fn manifest(uuid: u128, name: &str) -> String {
    format!(
        r#"{{"format_version": 2, "header": {{"name": "{name}", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 21, 100]}}, "modules": [{{"type": "resources", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
        Uuid::from_u128(uuid),
        Uuid::from_u128(uuid + 1000),
    )
}

/// A resource pack named `name` with `files`.
pub fn pack(uuid: u128, name: &str, files: Vec<(String, String)>) -> Pack {
    let mut all = vec![(
        "manifest.json".to_owned(),
        manifest(uuid, name).into_bytes(),
    )];
    all.extend(
        files
            .into_iter()
            .map(|(path, text)| (path, text.into_bytes())),
    );
    let files = all
        .into_iter()
        .map(|(path, bytes)| (PackPath::new(&path).expect("a pack path"), bytes));
    let options = OpenOptions {
        origin: PackOrigin::User,
        ..OpenOptions::default()
    };
    Pack::from_files(files, &options).expect("the pack opens")
}
