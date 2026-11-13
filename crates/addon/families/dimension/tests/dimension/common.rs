//! Helpers of the dimension tests.

#![allow(dead_code)]

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_dimension::Dimension;

#[allow(unused_imports, reason = "each test file uses part of the helpers")]
pub use bedrock_addon_test_support::with_code;

/// The text of the fixture file `path` (relative to the family's fixture folder).
pub fn fixture(path: &str) -> String {
    let file = bedrock_addon_test_support::fixture::root(env!("CARGO_MANIFEST_DIR"))
        .join("worldgen/dimension")
        .join(path);
    std::fs::read_to_string(&file).unwrap_or_else(|error| panic!("{}: {error}", file.display()))
}

/// The 1.26.60 engine test input `name` (`engine/`, copied byte for byte from the
/// game's own unit tests).
pub fn engine(name: &str) -> String {
    fixture(&format!("engine/{name}"))
}

/// The engine's `fmt` template `name` (`engine/<name>.fmt.txt`) with `args` in its
/// `{}` placeholders, in order, and `{{` / `}}` read as `{` / `}`: the document the test
/// builds. Each test names its values.
#[track_caller]
pub fn render(name: &str, args: &[&str]) -> String {
    let template = engine(&format!("{name}.fmt.txt"));
    let mut out = String::with_capacity(template.len());
    let mut args = args.iter();
    let mut rest = template.as_str();
    while let Some(at) = rest.find(['{', '}']) {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        if let Some(after) = tail.strip_prefix("{{") {
            out.push('{');
            rest = after;
        } else if let Some(after) = tail.strip_prefix("}}") {
            out.push('}');
            rest = after;
        } else if let Some(after) = tail.strip_prefix("{}") {
            out.push_str(
                args.next()
                    .unwrap_or_else(|| panic!("{name}: too few arguments")),
            );
            rest = after;
        } else {
            panic!("{name}: a lone brace at {at}");
        }
    }
    out.push_str(rest);
    assert!(args.next().is_none(), "{name}: too many arguments");
    out
}

/// Loads `json` as `dimensions/test.json`.
pub fn load(json: &str) -> Result<Document<Dimension>, Rejected> {
    Dimension::parse(
        &PackPath::from_static("dimensions/test.json"),
        json.as_bytes(),
        &LoadContext::detached(),
    )
}

/// Loads `json`, which must load.
#[track_caller]
pub fn loads(json: &str) -> Document<Dimension> {
    match load(json) {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {json}\n{:#?}", rejected.diagnostics),
    }
}

/// Loads `json`, which must be rejected; its findings.
#[track_caller]
pub fn rejects(json: &str) -> Diagnostics {
    match load(json) {
        Ok(document) => panic!("loaded: {json}\n{:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// The texts of the rejecting errors of `d`.
pub fn errors(d: &Diagnostics) -> Vec<String> {
    d.iter()
        .filter(|f| f.rejects())
        .map(|f| f.code.to_string())
        .collect()
}

/// The findings above info.
pub fn above_info(d: &Diagnostics) -> Vec<&Diagnostic> {
    d.iter().filter(|f| f.severity > Severity::Info).collect()
}

/// A dimension document at `version` for `identifier` with `components` (a JSON object's
/// members).
pub fn dimension(version: &str, identifier: &str, components: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:dimension": {{"description": {{"identifier": "{identifier}"}}, "components": {{{components}}}}}}}"#
    )
}

/// `minecraft:dimension_bounds` `{min, max}` and a void generator.
pub fn bounds(min: i32, max: i32) -> String {
    format!(
        r#""minecraft:dimension_bounds": {{"min": {min}, "max": {max}}}, "minecraft:generation": {{"generator_type": "void"}}"#
    )
}

/// A void generator only.
pub const VOID: &str = r#""minecraft:generation": {"generator_type": "void"}"#;

/// The two texts of a validator failure: its content-log text and the Puv validation text.
pub fn validator(content_log: &str, puv: &str) -> String {
    format!("{content_log}\n{puv}")
}

/// The texts of a height failure (part A of the 1.26.60 validator).
pub fn height_failure(content_log: &str) -> String {
    validator(content_log, "Invalid dimension height.")
}

/// The texts of an identifier failure (part B of the 1.26.60 validator).
pub fn custom_failure(content_log: &str) -> String {
    validator(content_log, "Invalid custom dimension document.")
}
