//! Helpers of the data-driven UI tests.

#![allow(
    dead_code,
    reason = "a helper module shared by the test files of this folder; none of them uses every helper"
)]

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::families::ddui::{UiComposition, UiRoot};
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::pack::{OpenOptions, Pack, PackOrigin, PackStack, Resource};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::FormatVersion;

#[allow(unused_imports, reason = "each test file uses part of the helpers")]
pub use bedrock_addon_test_support::{loaded as loads, rejected as rejects};

/// The fixture folder of this family.
pub const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ui/ddui");

/// The text of the fixture file `path` (relative to the family's fixture folder).
pub fn fixture(path: &str) -> String {
    std::fs::read_to_string(format!("{FIXTURES}/{path}"))
        .unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// The `.json` files of the fixture folder `dir`, as `(file name, text)`, sorted.
pub fn fixture_files(dir: &str) -> Vec<(String, String)> {
    let mut files: Vec<(String, String)> = std::fs::read_dir(format!("{FIXTURES}/{dir}"))
        .unwrap_or_else(|error| panic!("{dir}: {error}"))
        .map(|entry| {
            let path = entry.unwrap().path();
            (
                path.file_name().unwrap().to_string_lossy().into_owned(),
                std::fs::read_to_string(&path).unwrap(),
            )
        })
        .collect();
    files.sort();
    files
}

/// Engine 1.26.20: the catalogue the fixtures are written against unless they say otherwise.
pub const E20: FormatVersion = FormatVersion::new(1, 26, 20);
/// Engine 1.26.45: the later catalogue.
pub const E45: FormatVersion = FormatVersion::new(1, 26, 45);
/// Engine 1.26.60: the catalogue of the five `Form.*` types.
pub const E60: FormatVersion = FormatVersion::new(1, 26, 60);

/// The load context of a pack read by an engine of version `engine`; `builtin` is a pack the
/// game ships.
pub fn cx(engine: FormatVersion, builtin: bool) -> LoadContext {
    let mut cx = LoadContext::detached();
    cx.engine_version = engine;
    cx.is_base_game_pack = builtin;
    cx
}

/// Loads `json` as a root file of an add-on pack on engine 1.26.20.
pub fn root(json: &str) -> Result<Document<UiRoot>, Rejected> {
    root_in(json, &cx(E20, false))
}

/// Loads `json` as a root file with `cx`.
pub fn root_in(json: &str, cx: &LoadContext) -> Result<Document<UiRoot>, Rejected> {
    UiRoot::parse(
        &PackPath::from_static("ddui/root/test.json"),
        json.as_bytes(),
        cx,
    )
}

/// Loads `json` as a composition file of an add-on pack on engine 1.26.20.
pub fn composition(json: &str) -> Result<Document<UiComposition>, Rejected> {
    composition_in(json, &cx(E20, false))
}

/// Loads `json` as a composition file with `cx`.
pub fn composition_in(json: &str, cx: &LoadContext) -> Result<Document<UiComposition>, Rejected> {
    UiComposition::parse(
        &PackPath::from_static("ddui/composition/test.json"),
        json.as_bytes(),
        cx,
    )
}

/// One finding as the tests compare it: severity, code name, JSON path, message.
pub type Found = (Severity, &'static str, String, String);

/// Every finding of `d`.
pub fn found<'a>(d: impl IntoIterator<Item = &'a Diagnostic>) -> Vec<Found> {
    d.into_iter()
        .map(|f| {
            let detail = f.detail.as_ref().map(ToString::to_string);
            (
                f.severity,
                f.code.id(),
                f.location.json_path.to_string(),
                format!("{}{}", f.code, detail.unwrap_or_default()),
            )
        })
        .collect()
}

/// An expected error.
pub fn error(code: &'static str, path: &str, message: &str) -> Found {
    (Severity::Error, code, path.to_owned(), message.to_owned())
}

/// An expected warning.
pub fn warning(code: &'static str, path: &str, message: &str) -> Found {
    (Severity::Warning, code, path.to_owned(), message.to_owned())
}

/// An expected info finding.
pub fn info(code: &'static str, path: &str, message: &str) -> Found {
    (Severity::Info, code, path.to_owned(), message.to_owned())
}

/// The loader's closing line of a file the Puv load or a validator failed.
pub fn failed_to_load() -> Found {
    error(
        "ddui/failed_to_load",
        "",
        "Failed to load DataDrivenUI file",
    )
}

/// The loader's text for a file whose payload member is not `key`.
pub fn missing_object(key: &str) -> Found {
    error(
        "ddui/missing_object",
        "",
        &format!("Failed to parse DataDrivenUI file. Missing object \"{key}\""),
    )
}

/// The text of the file a finding's span covers.
#[track_caller]
pub fn spanned<'t>(text: &'t str, finding: &Diagnostic) -> &'t str {
    let span = finding.location.span.expect("a span");
    &text[span.offset as usize..(span.offset + span.len) as usize]
}

/// `Panel` → `Panel.Text`: the markup of the fixtures' root `R`.
pub const R_MARKUP: &str = r#"{ "component": "Panel", "children": [ { "component": "Panel.Text", "attribs": { "text": "hi" } } ] }"#;

/// A root document named `identifier` with `markup` (the elements of the JSON array).
pub fn root_json(identifier: &str, markup: &str) -> String {
    format!(
        r#"{{ "format_version": "1.21.130", "minecraft:ui-root": {{
  "description": {{ "identifier": "{identifier}" }},
  "attribs": {{ "root_point": "minecraft:screen" }},
  "layout": {{ "markup": [ {markup} ] }} }} }}"#
    )
}

/// The fixtures' root `R`: `demo:screen`.
pub fn r() -> String {
    root_json("demo:screen", R_MARKUP)
}

/// `R` with `children` as the children of its panel.
pub fn panel(children: &str) -> String {
    root_json(
        "demo:screen",
        &format!(r#"{{ "component": "Panel", "children": [ {children} ] }}"#),
    )
}

/// A composition document named `identifier` with `attribs` (the members of the JSON object)
/// and `markup` (the elements of the JSON array).
pub fn composition_json(identifier: &str, attribs: &str, markup: &str) -> String {
    format!(
        r#"{{ "format_version": "1.21.130", "minecraft:ui-composition": {{
  "description": {{ "identifier": "{identifier}" }},
  "attribs": {{ {attribs} }},
  "layout": {{ "markup": [ {markup} ] }} }} }}"#
    )
}

/// The manifest of a resource pack.
pub fn manifest(uuid: u128, name: &str) -> String {
    format!(
        r#"{{ "format_version": 2,
  "header": {{ "name": "{name}", "description": "", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 21, 0] }},
  "modules": [{{ "type": "resources", "uuid": "{}", "version": [1, 0, 0] }}] }}"#,
        uuid::Uuid::from_u128(uuid),
        uuid::Uuid::from_u128(uuid ^ (0xabcd << 64)),
    )
}

/// A resource pack named `name` with `files`; `PackOrigin::Package` is a pack the game ships.
pub fn pack(uuid: u128, name: &str, origin: PackOrigin, files: Vec<(&str, String)>) -> Pack {
    let mut all = vec![(
        PackPath::from_static("manifest.json"),
        bytes::Bytes::from(manifest(uuid, name)),
    )];
    all.extend(files.into_iter().map(|(path, text)| {
        (
            PackPath::new(path).expect("a pack path"),
            bytes::Bytes::from(text),
        )
    }));
    let options = OpenOptions {
        origin,
        ..OpenOptions::default()
    };
    Pack::from_files(all, &options).expect("the pack opens")
}

/// An add-on pack.
pub fn external(uuid: u128, name: &str, files: Vec<(&str, String)>) -> Pack {
    pack(uuid, name, PackOrigin::User, files)
}

/// A pack the game ships.
pub fn builtin(uuid: u128, name: &str, files: Vec<(&str, String)>) -> Pack {
    pack(uuid, name, PackOrigin::Package, files)
}

/// The resource stack of `packs`, top first.
pub fn stack(packs: Vec<Pack>) -> PackStack<Resource> {
    PackStack::<Resource>::of(packs, LoadOptions::default()).expect("a stack")
}
