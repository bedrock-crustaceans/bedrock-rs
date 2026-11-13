//! What the block tests share: loading one document, in-memory packs, the fixture files.

#![allow(dead_code, reason = "each test module uses part of the helpers")]

use std::path::PathBuf;

use bedrock_addon::diagnostic::{Code, Diagnostic, Diagnostics, Severity};
use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::families::block::Block;
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::pack::{OpenOptions, Pack, PackOrigin, PackType};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::MinEngineVersion;
use uuid::Uuid;

#[allow(unused_imports, reason = "each test file uses part of the helpers")]
pub use bedrock_addon_test_support::errors;

/// Whether `code` asks for a toggle the pack does not have (`upcoming_creator_features`,
/// the beta APIs): the toggle rules a block document meets, its own and the core's. Not the
/// beta-features gate, which shares the kind.
pub fn missing_toggles(code: &Code) -> bool {
    matches!(
        code.id(),
        "core/missing_toggles"
            | "core/missing_toggles_worded"
            | "core/toggle_use_format_version"
            | "core/turn_on_toggle"
            | "core/turn_on_toggle_before"
            | "block/custom_properties_version"
            | "block/custom_permutations_version"
            | "block/multi_block_horizontal_toggle"
            | "block/member_needs_creator_features"
            | "block/needs_creator_features"
            | "block/collision_min_max_toggle"
    )
}

/// One of the ten packs allowed the legacy creator features.
pub const ALLOW_LISTED: Uuid = Uuid::from_u128(0xda09_e399_6f5e_4287_a013_4cf1_3de3_2166);

/// A behaviour-pack context: the engine version, no toggles.
pub fn context() -> LoadContext {
    let mut cx = LoadContext::for_pack(
        &LoadOptions::default(),
        Uuid::from_u128(7),
        PackType::Behavior,
        Some(MinEngineVersion::new(1, 26, 20)),
        false,
    );
    cx.molang_version = bedrock_addon::molang::pack_molang_version(cx.min_engine_version.as_ref());
    cx
}

/// The context with Upcoming Creator Features on.
pub fn ucf() -> LoadContext {
    let options = LoadOptions {
        upcoming_creator_features: true,
        ..LoadOptions::default()
    };
    let mut cx = LoadContext::for_pack(
        &options,
        Uuid::from_u128(7),
        PackType::Behavior,
        Some(MinEngineVersion::new(1, 26, 20)),
        false,
    );
    cx.molang_version = bedrock_addon::molang::pack_molang_version(cx.min_engine_version.as_ref());
    cx
}

/// The context of an allow-listed pack (legacy creator features).
pub fn allow_listed() -> LoadContext {
    let mut cx = context();
    cx.pack_uuid = ALLOW_LISTED;
    cx
}

/// A context whose pack's `min_engine_version` is `version`.
pub fn pack_version(major: u16, minor: u16, patch: u16) -> LoadContext {
    let mut cx = context();
    cx.min_engine_version = Some(MinEngineVersion::new(major, minor, patch));
    cx.molang_version = bedrock_addon::molang::pack_molang_version(cx.min_engine_version.as_ref());
    cx
}

/// Loads `text` as `blocks/demo/x.json`.
pub fn load(text: &str, cx: &LoadContext) -> Result<Document<Block>, Rejected> {
    Block::parse(
        &PackPath::from_static("blocks/demo/x.json"),
        text.as_bytes(),
        cx,
    )
}

/// Loads `text` and requires it to load.
#[track_caller]
pub fn loads(text: &str, cx: &LoadContext) -> Document<Block> {
    match load(text, cx) {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {:#?}", rejected.diagnostics),
    }
}

/// Loads `text` and requires it to be rejected.
#[track_caller]
pub fn rejected(text: &str, cx: &LoadContext) -> Diagnostics {
    match load(text, cx) {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// A document of `version` with `payload` as `minecraft:block`.
pub fn document(version: &str, payload: &str) -> String {
    format!(r#"{{"format_version": "{version}", "minecraft:block": {payload}}}"#)
}

/// The findings whose code matches `code`.
pub fn with<'a>(
    diagnostics: &'a Diagnostics,
    code: impl Fn(&Code) -> bool + 'a,
) -> impl Iterator<Item = &'a Diagnostic> + 'a {
    diagnostics.iter().filter(move |d| code(&d.code))
}

/// Whether a finding has `code`, `severity` and a text containing `text`.
pub fn has(
    diagnostics: &Diagnostics,
    code: impl Fn(&Code) -> bool,
    severity: Severity,
    text: &str,
) -> bool {
    diagnostics
        .iter()
        .any(|d| code(&d.code) && d.severity == severity && d.code.to_string().contains(text))
}

/// The fixture directory `tests/fixtures/block/block/<rest>`.
pub fn fixture_dir(rest: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/block/block")
        .join(rest)
}

/// A fixture file's text.
pub fn fixture(rest: &str) -> String {
    let path = fixture_dir(rest);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// A format-2 manifest of one module.
pub fn manifest(uuid: u128, module: &str, name: &str) -> String {
    format!(
        r#"{{"format_version": 2, "header": {{"name": "{name}", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 26, 20]}}, "modules": [{{"type": "{module}", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
        Uuid::from_u128(uuid),
        Uuid::from_u128(uuid + 1000),
    )
}

/// A pack from its files.
pub fn pack(files: Vec<(&str, Vec<u8>)>) -> Pack {
    let files = files
        .into_iter()
        .map(|(path, bytes)| (PackPath::new(path).expect("a pack path"), bytes));
    let options = OpenOptions {
        origin: PackOrigin::User,
        ..OpenOptions::default()
    };
    Pack::from_files(files, &options).expect("the pack opens")
}

/// The block in the shape of the newest format, as `{"minecraft:block": payload}`: the value
/// an upgrade gave, whatever version the writer would write it at.
pub fn newest_shape(
    block: &bedrock_addon::document::Document<Block>,
) -> bedrock_addon::json::Value {
    use bedrock_addon::versioning::VersionStruct;
    let mut object = facet_value::VObject::new();
    object.insert(
        "minecraft:block",
        block.value().write().expect("the block has a JSON form"),
    );
    // With the document's unknown members, where the document has them.
    let mut value = object.into();
    let _ = bedrock_addon::json::merge_unknown(&mut value, block.unknown());
    value
}
