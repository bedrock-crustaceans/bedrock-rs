//! Helpers of the voxel-shape tests.

#![allow(dead_code)]

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_voxel_shape::VoxelShape;

/// The fixture folder of this family.
pub const FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../addon/tests/fixtures/block/voxel_shape"
);

/// The text of the fixture file `path` (relative to the family's fixture folder).
pub fn fixture(path: &str) -> String {
    std::fs::read_to_string(format!("{FIXTURES}/{path}"))
        .unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// The context of a pack that is not the base game's.
pub fn user_pack() -> LoadContext {
    LoadContext::detached()
}

/// The context of a base-game pack.
pub fn base_game() -> LoadContext {
    let mut cx = LoadContext::detached();
    cx.is_base_game_pack = true;
    cx
}

/// Loads `json` as `shapes/test.json` of a pack with `cx`.
pub fn load(json: &str, cx: &LoadContext) -> Result<Document<VoxelShape>, Rejected> {
    VoxelShape::parse(
        &PackPath::from_static("shapes/test.json"),
        json.as_bytes(),
        cx,
    )
}

/// Loads `json` from a pack with `cx`; it must load without any finding.
#[track_caller]
pub fn loads_in(json: &str, cx: &LoadContext) -> Document<VoxelShape> {
    match load(json, cx) {
        Ok(document) => {
            assert!(
                document.diagnostics().is_empty(),
                "{json}\n{:#?}",
                document.diagnostics()
            );
            document
        }
        Err(rejected) => panic!("rejected: {json}\n{:#?}", rejected.diagnostics),
    }
}

/// Loads `json` from a user pack; it must load without any finding.
#[track_caller]
pub fn loads(json: &str) -> Document<VoxelShape> {
    loads_in(json, &user_pack())
}

/// Loads `json` from a pack with `cx`; it must be rejected. Its findings.
#[track_caller]
pub fn rejects_in(json: &str, cx: &LoadContext) -> Diagnostics {
    match load(json, cx) {
        Ok(document) => panic!("loaded: {json}\n{:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// Loads `json` from a user pack; it must be rejected. Its findings.
#[track_caller]
pub fn rejects(json: &str) -> Diagnostics {
    rejects_in(json, &user_pack())
}

/// The findings whose code is `name`.
pub fn with_code<'a>(d: &'a Diagnostics, name: &str) -> Vec<&'a Diagnostic> {
    d.iter().filter(|f| f.code.id() == name).collect()
}

/// `(code name, text, JSON path)` of every finding, all of which must be errors.
#[track_caller]
pub fn errors(d: &Diagnostics) -> Vec<(&'static str, String, String)> {
    d.iter()
        .map(|f| {
            assert_eq!(f.severity, Severity::Error, "{f:#?}");
            (
                f.code.id(),
                f.code.to_string(),
                f.location.json_path.to_string(),
            )
        })
        .collect()
}

/// `(code name, JSON path)` of every finding, all of which must be errors.
#[track_caller]
pub fn kinds(d: &Diagnostics) -> Vec<(&'static str, String)> {
    errors(d)
        .into_iter()
        .map(|(code, _, path)| (code, path))
        .collect()
}

/// One expected finding of [`kinds`].
pub fn kind(code: &'static str, path: &str) -> (&'static str, String) {
    (code, path.to_owned())
}

/// One expected finding of [`errors`].
pub fn error(code: &'static str, text: &str, path: &str) -> (&'static str, String, String) {
    (code, text.to_owned(), path.to_owned())
}

/// The closing line vanilla logs after a file its schema loader rejected, as [`errors`] lists
/// it for `shapes/test.json`.
pub fn failed() -> (&'static str, String, String) {
    error(
        "voxel_shape/failed_to_load",
        "Failed to load voxel shape file shapes/test.json",
        "",
    )
}

/// One box, `{"min": …, "max": …}`, from the two corners as JSON texts.
pub fn corners(min: &str, max: &str) -> String {
    format!(r#"{{"min": {min}, "max": {max}}}"#)
}

/// A document at `version` naming `identifier` with `boxes` (the elements of the array).
pub fn document(version: &str, identifier: &str, boxes: &[String]) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:voxel_shape": {{"description": {{"identifier": "{identifier}"}}, "shape": {{"boxes": [{}]}}}}}}"#,
        boxes.join(", ")
    )
}

/// A 1.21.110 document naming `identifier` with `boxes`, each `(min, max)` as JSON texts.
pub fn shape(identifier: &str, boxes: &[(&str, &str)]) -> String {
    let boxes: Vec<String> = boxes.iter().map(|(min, max)| corners(min, max)).collect();
    document("1.21.110", identifier, &boxes)
}

/// The docs example (fixture `vs04`): `mypack:slab_bottom`, the lower half of a block.
pub fn slab() -> String {
    shape("mypack:slab_bottom", &[("[0, 0, 0]", "[16, 8, 16]")])
}
