//! Helpers of the camera tests.

#![allow(
    dead_code,
    reason = "a helper module shared by the test files of this folder; none of them uses every helper"
)]

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_camera::{CameraEntity, CameraList, CameraPreset, CameraSpline, Cameras};

#[allow(unused_imports, reason = "each test file uses part of the helpers")]
pub use bedrock_addon_test_support::{loaded as loads, rejected as rejects};

/// The fixture folder of this family, below the fixture root of the package that compiles this
/// file: the family crate, or `bedrock_addon`, whose tests include it.
pub fn fixtures() -> std::path::PathBuf {
    bedrock_addon_test_support::fixture::root(env!("CARGO_MANIFEST_DIR")).join("camera/camera")
}

/// The text of the fixture file `path` (relative to the family's fixture folder).
pub fn fixture(path: &str) -> String {
    std::fs::read_to_string(fixtures().join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// Loads `json` as a file of `T` at `path`.
pub fn load<T: Content>(path: &'static str, json: &str) -> Result<Document<T>, Rejected> {
    T::parse(
        &PackPath::from_static(path),
        json.as_bytes(),
        &LoadContext::detached(),
    )
}

/// Loads `json` as a preset file.
pub fn preset(json: &str) -> Result<Document<CameraPreset>, Rejected> {
    load("cameras/presets/test.json", json)
}

/// Loads `json` as a list file.
pub fn list(json: &str) -> Result<Document<CameraList>, Rejected> {
    load("cameras/lists/test.json", json)
}

/// Loads `json` as a spline file.
pub fn spline(json: &str) -> Result<Document<CameraSpline>, Rejected> {
    load("cameras/splines/test.json", json)
}

/// Loads `json` as a camera entity document.
pub fn entity(json: &str) -> Result<Document<CameraEntity>, Rejected> {
    load("cameras/test.json", json)
}

/// `(code name, JSON path, message)` of each finding at `severity`.
pub fn at_severity<'a>(
    d: impl IntoIterator<Item = &'a Diagnostic>,
    severity: Severity,
) -> Vec<(&'static str, String, String)> {
    d.into_iter()
        .filter(|f| f.severity == severity)
        .map(|f| {
            (
                f.code.id(),
                f.location.json_path.to_string(),
                f.code.to_string(),
            )
        })
        .collect()
}

/// `(code name, JSON path, message)` of each error.
pub fn errors(d: &Diagnostics) -> Vec<(&'static str, String, String)> {
    at_severity(d, Severity::Error)
}

/// `(code name, JSON path, message)` of each warning.
pub fn warnings(d: &Diagnostics) -> Vec<(&'static str, String, String)> {
    at_severity(d, Severity::Warning)
}

/// `(code name, JSON path, message)` of each information.
pub fn infos(d: &Diagnostics) -> Vec<(&'static str, String, String)> {
    at_severity(d, Severity::Info)
}

/// One expected finding.
pub fn finding(code: &'static str, path: &str, message: &str) -> (&'static str, String, String) {
    (code, path.to_owned(), message.to_owned())
}

/// The closing line of a preset file that failed.
pub fn failed_presets() -> (&'static str, String, String) {
    finding("camera/failed_presets", "", "Failed to load camera presets")
}

/// A preset document at `version` with `members` (a JSON object's members).
pub fn preset_json(version: &str, members: &str) -> String {
    format!(r#"{{"format_version": "{version}", "minecraft:camera_preset": {{{members}}}}}"#)
}

/// A preset `ex:a` inheriting from `minecraft:free`, at `version`, with `members` more.
pub fn preset_with(version: &str, members: &str) -> String {
    preset_json(
        version,
        &format!(r#""identifier": "ex:a", "inherit_from": "minecraft:free", {members}"#),
    )
}

/// A list document at `version` named `identifier` of `cameras` (a JSON array).
pub fn list_json(version: &str, identifier: &str, cameras: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:camera_list": {{"identifier": "{identifier}", "cameras": {cameras}}}}}"#
    )
}

/// A spline document at 1.26.0 with `splines` (the elements of the JSON array).
pub fn splines_json(splines: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.0", "minecraft:camera_custom_splines": {{"splines": [{splines}]}}}}"#
    )
}

/// One spline named `name` with `members` more; the required members are present unless
/// `members` repeats one.
pub fn spline_json(name: &str, members: &str) -> String {
    let default = |key: &str, value: &str| {
        if members.contains(&format!("\"{key}\"")) {
            String::new()
        } else {
            format!(r#", "{key}": {value}"#)
        }
    };
    let separator = if members.is_empty() { "" } else { ", " };
    format!(
        r#"{{"name": "{name}"{separator}{members}{}{}{}{}}}"#,
        default("total_time", "4.0"),
        default(
            "control_points",
            r#"[{"position": [0, 0, 0]}, {"position": [1, 2, 3]}]"#
        ),
        default("progress_key_frames", r#"[{"progress": 0.0, "time": 0.0}]"#),
        default(
            "rotation_key_frames",
            r#"[{"rotation": [0, 0, 0], "time": 0.0}]"#
        ),
    )
}

/// A camera entity document with `identifier` and `components` (a JSON object's members).
pub fn entity_json(identifier: &str, components: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.10", "minecraft:camera_entity": {{"description": {{"identifier": "{identifier}"}}, "components": {{{components}}}}}}}"#
    )
}

/// A text that lives as long as the test binary (the pack helper takes `'static` files).
pub fn leak(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

/// `(code name, message)` of each finding of the group at `severity`: the camera loader's own;
/// the game's validator line beside an unknown reference is `validator_lines`'.
pub fn group(cameras: &Cameras, severity: Severity) -> Vec<(&'static str, String)> {
    cameras
        .diagnostics()
        .filter(|f| f.severity == severity && !is_validator_line(f))
        .map(|f| (f.code.id(), f.code.to_string()))
        .collect()
}

/// Whether `finding` is the game's link-pass line (`LinkedAssetValidator`).
pub fn is_validator_line(finding: &bedrock_addon_core::diagnostic::Diagnostic) -> bool {
    finding
        .code
        .to_string()
        .contains(":  Missing referenced asset ")
}

/// `(severity, json path, message)` of each validator line of the group.
pub fn validator_lines(cameras: &Cameras) -> Vec<(Severity, String, String)> {
    cameras
        .diagnostics()
        .filter(|f| is_validator_line(f))
        .map(|f| {
            (
                f.severity,
                f.location.json_path.to_string(),
                f.code.to_string(),
            )
        })
        .collect()
}
