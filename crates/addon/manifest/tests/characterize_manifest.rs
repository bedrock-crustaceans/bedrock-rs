//! Characterization of the two manifest readers (phase 5, one set of manifest primitives): every
//! row of the table of differences between the legacy and the comprehensive reader (string,
//! boolean, UUID, version, header, `min_engine_version`, `lock_template_options`, modules,
//! dependencies, capabilities, subpacks, settings, metadata, `format_version`, unknown members,
//! read order), read by each profile. Each case pins its findings in order (code with its text,
//! severity, phase, the path as text and as segments) and the value read (formats, the typed
//! members the writer does not show, and the written text), in `characterize_manifest.txt`,
//! made before the readers shared their primitives.
//!
//! Two path edges are pinned on purpose: a member named by digits is an array index of the path
//! (`"0"` and `"+5"` are `Index(0)` and `Index(5)`), and a member named `""` adds no segment.
//!
//! `BEDROCK_ADDON_CHARACTERIZE=write` writes the pin file instead of comparing.

#![allow(clippy::disallowed_methods, clippy::disallowed_types, missing_docs)]

use std::fmt::Write as _;
use std::path::PathBuf;

use bedrock_addon_core::diagnostic::JsonPathSegment;
use bedrock_addon_core::document::Document;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_manifest::{CapabilityRegistry, Manifest};

/// The pin file.
fn pins() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/characterize_manifest.txt")
}

const HEADER_UUID: &str = "11111111-1111-1111-1111-111111111111";
const MODULE_UUID: &str = "22222222-2222-2222-2222-222222222222";

/// A legacy manifest of `format` with `header` members and `rest` (root members after
/// `modules`, each with a leading comma); `modules` is the modules array.
fn legacy_with(format: &str, header: &str, modules: &str, rest: &str) -> String {
    format!(
        r#"{{ "format_version": {format}, "header": {{ {header} }}, "modules": {modules}{rest} }}"#
    )
}

/// The members of a valid legacy header, with `more` after them.
fn legacy_header(more: &str) -> String {
    format!(
        r#""name": "P", "uuid": "{HEADER_UUID}", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]{more}"#
    )
}

/// A data module of a legacy manifest.
fn legacy_modules(kind: &str) -> String {
    format!(r#"[ {{ "type": "{kind}", "uuid": "{MODULE_UUID}", "version": [1, 0, 0] }} ]"#)
}

/// A valid format-2 behaviour pack with `header_more` in the header and `rest` at the root.
fn v2(header_more: &str, rest: &str) -> String {
    legacy_with(
        "2",
        &legacy_header(header_more),
        &legacy_modules("data"),
        rest,
    )
}

/// A format-2 manifest whose header is exactly `header`.
fn v2_header(header: &str) -> String {
    legacy_with("2", header, &legacy_modules("data"), "")
}

/// A format-1 behaviour pack with `header_more` in the header and `rest` at the root.
fn v1(header_more: &str, rest: &str) -> String {
    legacy_with(
        "1",
        &format!(r#""name": "P", "uuid": "{HEADER_UUID}", "version": [1, 0, 0]{header_more}"#),
        &legacy_modules("data"),
        rest,
    )
}

/// A format-0 manifest whose header is `header` (modules inside it).
fn v0(header: &str) -> String {
    format!(r#"{{ "format_version": 0, "header": {{ {header} }} }}"#)
}

/// The members of a valid format-0 header, with `more` after them.
fn v0_header(more: &str) -> String {
    format!(
        r#""name": "P", "pack_id": "{HEADER_UUID}", "packs_version": [1, 0, 0], "modules": [ {{ "type": "resources", "uuid": "{MODULE_UUID}", "version": [1, 0, 0] }} ]{more}"#
    )
}

/// The members of a valid format-3 header, with `more` after them.
fn v3_header(more: &str) -> String {
    format!(
        r#""name": "P", "uuid": "{HEADER_UUID}", "version": "1.0.0", "min_engine_version": "1.21.0"{more}"#
    )
}

/// A format-3 data module.
fn v3_modules(kind: &str) -> String {
    format!(r#"[ {{ "type": "{kind}", "uuid": "{MODULE_UUID}", "version": "1.0.0" }} ]"#)
}

/// A valid format-3 behaviour pack with `header_more` in the header and `rest` at the root.
fn v3(header_more: &str, rest: &str) -> String {
    v3_with(&v3_header(header_more), &v3_modules("data"), rest)
}

/// A format-3 manifest from its header members, its modules array and `rest`.
fn v3_with(header: &str, modules: &str, rest: &str) -> String {
    format!(r#"{{ "format_version": 3, "header": {{ {header} }}, "modules": {modules}{rest} }}"#)
}

/// Every case: its name and its text.
#[allow(clippy::too_many_lines, reason = "one line per case")]
fn cases() -> Vec<(&'static str, String)> {
    let dep = |element: &str| format!(r#", "dependencies": [ {element} ]"#);
    let native = |version: &str| {
        format!(r#"{{ "module_name": "@minecraft/server", "version": {version} }}"#)
    };
    let pack_dep =
        format!(r#"{{ "uuid": "{HEADER_UUID}", "version": [1, 0, 0] }}"#).replace("1111", "3333");
    let script_modules_v2 = format!(
        r#"[ {{ "type": "data", "uuid": "{MODULE_UUID}", "version": [1, 0, 0] }}, {{ "type": "script", "language": "javascript", "entry": "scripts/main.js", "uuid": "44444444-4444-4444-4444-444444444444", "version": [1, 0, 0] }} ]"#
    );
    vec![
        // ---- legacy: string ----
        ("legacy string valid", v2("", "")),
        ("legacy string null optional", v2(r#", "description": null"#, "")),
        ("legacy string wrong type", v2(r#", "description": 5"#, "")),
        ("legacy string empty optional", v2(r#", "description": """#, "")),
        ("legacy name absent v2", v2_header(&format!(r#""uuid": "{HEADER_UUID}", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]"#))),
        ("legacy name null v2", v2_header(&format!(r#""name": null, "uuid": "{HEADER_UUID}", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]"#))),
        ("legacy name empty v2", v2_header(&format!(r#""name": "", "uuid": "{HEADER_UUID}", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]"#))),
        ("legacy name wrong type v2", v2_header(&format!(r#""name": 5, "uuid": "{HEADER_UUID}", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]"#))),
        ("legacy name empty v1", legacy_with("1", &format!(r#""name": "", "uuid": "{HEADER_UUID}", "version": [1, 0, 0]"#), &legacy_modules("data"), "")),
        ("legacy name absent v1", legacy_with("1", &format!(r#""uuid": "{HEADER_UUID}", "version": [1, 0, 0]"#), &legacy_modules("data"), "")),
        ("legacy name empty v0", v0(&v0_header("").replace(r#""name": "P""#, r#""name": """#))),
        ("legacy name absent v0", v0(&v0_header("").replace(r#""name": "P", "#, ""))),
        ("legacy subpack folder empty", v2("", r#", "subpacks": [ { "folder_name": "", "name": "a", "memory_tier": 1 } ]"#)),
        ("legacy subpack folder null", v2("", r#", "subpacks": [ { "folder_name": null, "name": "a", "memory_tier": 1 } ]"#)),
        // ---- legacy: boolean ----
        ("legacy boolean null", v2(r#", "platform_locked": null"#, "")),
        ("legacy boolean wrong type", v2(r#", "platform_locked": "yes", "allow_random_seed": 1"#, "")),
        ("legacy boolean root", v2("", r#", "has_education_metadata": 1"#)),
        ("legacy boolean root valid", v2("", r#", "has_education_metadata": true"#)),
        // ---- legacy: uuid ----
        ("legacy uuid absent", v2_header(r#""name": "P", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]"#)),
        ("legacy uuid null", v2_header(r#""name": "P", "uuid": null, "version": [1, 0, 0], "min_engine_version": [1, 21, 0]"#)),
        ("legacy uuid wrong type", v2_header(r#""name": "P", "uuid": 5, "version": [1, 0, 0], "min_engine_version": [1, 21, 0]"#)),
        ("legacy uuid empty", v2_header(r#""name": "P", "uuid": "", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]"#)),
        ("legacy uuid malformed", v2_header(r#""name": "P", "uuid": "abc", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]"#)),
        ("legacy uuid malformed v0", v0(&v0_header("").replace(HEADER_UUID, "abc"))),
        ("legacy module uuid malformed v0", v0(&v0_header("").replace(MODULE_UUID, "abc"))),
        // ---- legacy: version ----
        ("legacy version absent", v2_header(&format!(r#""name": "P", "uuid": "{HEADER_UUID}", "min_engine_version": [1, 21, 0]"#))),
        ("legacy version absent v0", v0(&v0_header("").replace(r#""packs_version": [1, 0, 0], "#, ""))),
        ("legacy version partial", v2_header(&format!(r#""name": "P", "uuid": "{HEADER_UUID}", "version": "1.0", "min_engine_version": [1, 21, 0]"#))),
        ("legacy version string", v2_header(&format!(r#""name": "P", "uuid": "{HEADER_UUID}", "version": "1.2.3-beta", "min_engine_version": [1, 21, 0]"#))),
        ("legacy version partial v0", v0(&v0_header("").replace(r#""packs_version": [1, 0, 0]"#, r#""packs_version": "1.0""#))),
        ("legacy version short array", v2_header(&format!(r#""name": "P", "uuid": "{HEADER_UUID}", "version": [1, 0], "min_engine_version": [1, 21, 0]"#))),
        ("legacy version short array v0", v0(&v0_header("").replace(r#""packs_version": [1, 0, 0]"#, r#""packs_version": [1, 0]"#))),
        ("legacy version coerced elements", v2_header(&format!(r#""name": "P", "uuid": "{HEADER_UUID}", "version": [1, null, 2.5], "min_engine_version": [1, 21, true]"#))),
        ("legacy version wrong type", v2_header(&format!(r#""name": "P", "uuid": "{HEADER_UUID}", "version": true, "min_engine_version": [1, 21, 0]"#))),
        ("legacy module version wrong", legacy_with("2", &legacy_header(""), &legacy_modules("data").replace("[1, 0, 0] }", r#""1" }"#), "")),
        // ---- legacy: header ----
        ("legacy header absent", r#"{ "format_version": 2, "modules": [] }"#.replace("[]", &legacy_modules("data"))),
        ("legacy header null", format!(r#"{{ "format_version": 2, "header": null, "modules": {} }}"#, legacy_modules("data"))),
        ("legacy header wrong type", format!(r#"{{ "format_version": 2, "header": [], "modules": {} }}"#, legacy_modules("data"))),
        ("legacy header unknown members", v2(r#", "x": 1, "0": 2, "": 3, "+5": 4, "a/b~c": 5"#, "")),
        ("legacy root unknown members", v2("", r#", "x": 1, "0": 2, "": 3"#)),
        ("legacy pack_scope", v2(r#", "pack_scope": "world""#, "")),
        ("legacy pack_scope unknown", v2(r#", "pack_scope": "nope""#, "")),
        ("legacy pack_scope wrong type", v2(r#", "pack_scope": 5"#, "")),
        ("legacy pack_optimization_version", v2(r#", "pack_optimization_version": "1.2""#, "")),
        ("legacy pack_optimization_version array", v2(r#", "pack_optimization_version": [1, 2.5, 3]"#, "")),
        ("legacy pack_optimization_version bad", v2(r#", "pack_optimization_version": true"#, "")),
        ("legacy base_game_version on data", v2(r#", "base_game_version": [1, 21, 0]"#, "")),
        ("legacy base_game_version template", legacy_with("2", &legacy_header(r#", "base_game_version": "*", "lock_template_options": true"#).replace(r#", "min_engine_version": [1, 21, 0]"#, ""), &legacy_modules("world_template"), "")),
        ("legacy base_game_version template absent", legacy_with("2", &legacy_header(r#", "lock_template_options": true"#).replace(r#", "min_engine_version": [1, 21, 0]"#, ""), &legacy_modules("world_template"), "")),
        ("legacy base_game_version template low", legacy_with("2", &legacy_header(r#", "base_game_version": "1.0.0", "lock_template_options": true"#).replace(r#", "min_engine_version": [1, 21, 0]"#, ""), &legacy_modules("world_template"), "")),
        ("legacy base_game_version template bad", legacy_with("2", &legacy_header(r#", "base_game_version": "1.0", "lock_template_options": true"#).replace(r#", "min_engine_version": [1, 21, 0]"#, ""), &legacy_modules("world_template"), "")),
        // ---- legacy: min_engine_version ----
        ("legacy min_engine absent v2", v2_header(&format!(r#""name": "P", "uuid": "{HEADER_UUID}", "version": [1, 0, 0]"#))),
        ("legacy min_engine string v2", v2_header(&format!(r#""name": "P", "uuid": "{HEADER_UUID}", "version": [1, 0, 0], "min_engine_version": "1.21.0""#))),
        ("legacy min_engine too high", v2_header(&format!(r#""name": "P", "uuid": "{HEADER_UUID}", "version": [1, 0, 0], "min_engine_version": [9, 0, 0]"#))),
        ("legacy min_engine too low", v2_header(&format!(r#""name": "P", "uuid": "{HEADER_UUID}", "version": [1, 0, 0], "min_engine_version": [1, 1, 0]"#))),
        ("legacy min_engine absent v1", v1("", "")),
        ("legacy min_engine default v1", v1(r#", "min_engine_version": [1, 2, 5]"#, "")),
        ("legacy min_engine capped v1", v1(r#", "min_engine_version": [1, 20, 0]"#, "")),
        ("legacy min_engine low v1", v1(r#", "min_engine_version": [1, 1, 0]"#, "")),
        ("legacy min_engine on skin pack", legacy_with("2", &legacy_header(""), &legacy_modules("skin_pack"), "")),
        ("legacy min_engine on skin pack v1", legacy_with("1", &legacy_header(""), &legacy_modules("skin_pack"), "")),
        // ---- legacy: lock_template_options ----
        ("legacy lock absent template", legacy_with("2", &legacy_header(r#", "base_game_version": "*""#).replace(r#", "min_engine_version": [1, 21, 0]"#, ""), &legacy_modules("world_template"), "")),
        ("legacy lock on data", v2(r#", "lock_template_options": true"#, "")),
        ("legacy lock on data v1", v1(r#", "lock_template_options": true"#, "")),
        ("legacy lock wrong type", legacy_with("2", &legacy_header(r#", "base_game_version": "*", "lock_template_options": "x""#).replace(r#", "min_engine_version": [1, 21, 0]"#, ""), &legacy_modules("world_template"), "")),
        ("legacy lock null template", legacy_with("2", &legacy_header(r#", "base_game_version": "*", "lock_template_options": null"#).replace(r#", "min_engine_version": [1, 21, 0]"#, ""), &legacy_modules("world_template"), "")),
        // ---- legacy: modules ----
        ("legacy modules absent", format!(r#"{{ "format_version": 2, "header": {{ {} }} }}"#, legacy_header(""))),
        ("legacy modules null", format!(r#"{{ "format_version": 2, "header": {{ {} }}, "modules": null }}"#, legacy_header(""))),
        ("legacy modules wrong type", legacy_with("2", &legacy_header(""), r#""x""#, "")),
        ("legacy modules empty", legacy_with("2", &legacy_header(""), "[]", "")),
        ("legacy modules element wrong type", legacy_with("2", &legacy_header(""), &legacy_modules("data").replace("[ {", "[ 5, {"), "")),
        ("legacy module type invalid", legacy_with("2", &legacy_header(""), &legacy_modules("foo"), "")),
        ("legacy module type absent", legacy_with("2", &legacy_header(""), &format!(r#"[ {{ "uuid": "{MODULE_UUID}", "version": [1, 0, 0] }} ]"#), "")),
        ("legacy module type invalid v0", v0(&v0_header("").replace("resources", "foo"))),
        ("legacy modules inconsistent", legacy_with("2", &legacy_header(""), &format!(r#"[ {{ "type": "data", "uuid": "{MODULE_UUID}", "version": [1, 0, 0] }}, {{ "type": "resources", "uuid": "55555555-5555-5555-5555-555555555555", "version": [1, 0, 0] }} ]"#), "")),
        ("legacy modules inconsistent v0", v0(&format!(r#""name": "P", "pack_id": "{HEADER_UUID}", "packs_version": [1, 0, 0], "modules": [ {{ "type": "data", "uuid": "{MODULE_UUID}", "version": [1, 0, 0] }}, {{ "type": "resources", "uuid": "55555555-5555-5555-5555-555555555555", "version": [1, 0, 0] }} ]"#))),
        ("legacy modules absent v0", v0(&format!(r#""name": "P", "pack_id": "{HEADER_UUID}", "packs_version": [1, 0, 0]"#))),
        ("legacy module unknown members", legacy_with("2", &legacy_header(""), &legacy_modules("data").replace(r#""type""#, r#""0": 1, "": 2, "x": 3, "type""#), "")),
        ("legacy module uuid is the header's", legacy_with("2", &legacy_header(""), &legacy_modules("data").replace(MODULE_UUID, HEADER_UUID), "")),
        ("legacy module uuid is the header's v0", v0(&v0_header("").replace(MODULE_UUID, HEADER_UUID))),
        ("legacy modules error stops the read", legacy_with("2", r#""name": 5"#, &legacy_modules("foo"), r#", "dependencies": 5"#)),
        ("legacy script modules", legacy_with("2", &legacy_header(""), &script_modules_v2, "")),
        // ---- legacy: dependencies ----
        ("legacy dependencies wrong type", v2("", r#", "dependencies": 5"#)),
        ("legacy dependencies null", v2("", r#", "dependencies": null"#)),
        ("legacy dependency element wrong type", v2("", &dep("5"))),
        ("legacy dependency pack", v2("", &dep(&pack_dep))),
        ("legacy dependency pack no version", v2("", &dep(&format!(r#"{{ "uuid": "{HEADER_UUID}" }}"#)))),
        ("legacy dependency module_name and uuid", v2("", &dep(&format!(r#"{{ "module_name": "@minecraft/server", "uuid": "{HEADER_UUID}", "version": "1.0.0" }}"#)))),
        ("legacy dependency module_name null", v2("", &dep(&format!(r#"{{ "module_name": null, "uuid": "{HEADER_UUID}", "version": [1, 0, 0] }}"#)))),
        ("legacy dependency module_name uuid null", v2("", &dep(r#"{ "module_name": "@minecraft/server", "uuid": null, "version": "1.0.0" }"#))),
        ("legacy dependency native selector", v2("", &dep(&native(r#""beta""#)))),
        ("legacy dependency native version", v2("", &dep(&native(r#""1.2.0""#)))),
        ("legacy dependency native array", v2("", &dep(&native("[1, 2, 0]")))),
        ("legacy dependency native relaxed", v2("", &dep(&native(r#""2.0""#)))),
        ("legacy dependency native no version", v2("", &dep(r#"{ "module_name": "@minecraft/server" }"#))),
        ("legacy dependency native name empty", v2("", &dep(r#"{ "module_name": "", "version": "1.0.0" }"#))),
        ("legacy dependency native name wrong type", v2("", &dep(r#"{ "module_name": 5, "version": "1.0.0" }"#))),
        ("legacy dependency optional", v2("", &dep(r#"{ "module_name": "@minecraft/server", "version": "1.0.0", "optional": true }"#))),
        ("legacy dependency unknown member", v2("", &dep(r#"{ "module_name": "@minecraft/server", "version": "1.0.0", "x": true, "0": 1, "": 2 }"#))),
        // ---- legacy: capabilities ----
        ("legacy capabilities", v2("", r#", "capabilities": [ 5, "script_eval", "nope", "pbr" ]"#)),
        ("legacy capabilities wrong type", v2("", r#", "capabilities": "pbr""#)),
        // ---- legacy: subpacks ----
        ("legacy subpacks", v2("", r#", "subpacks": [ { "folder_name": "a", "name": "A", "memory_tier": 24 }, { "folder_name": "b", "name": "B", "memory_performance_tier": 2 } ]"#)),
        ("legacy subpack memory_tier negative", v2("", r#", "subpacks": [ { "folder_name": "a", "name": "A", "memory_tier": -1 } ]"#)),
        ("legacy subpack performance tier high", v2("", r#", "subpacks": [ { "folder_name": "a", "name": "A", "memory_performance_tier": 9 } ]"#)),
        ("legacy subpack no tier", v2("", r#", "subpacks": [ { "folder_name": "a", "name": "A" } ]"#)),
        ("legacy subpack tier wrong type", v2("", r#", "subpacks": [ { "folder_name": "a", "name": "A", "memory_tier": "x" } ]"#)),
        ("legacy subpacks element wrong type", v2("", r#", "subpacks": [ 5 ]"#)),
        ("legacy subpacks wrong type", v2("", r#", "subpacks": {}"#)),
        ("legacy subpack missing names", v2("", r#", "subpacks": [ { "memory_tier": 1 } ]"#)),
        ("legacy subpack unknown member", v2("", r#", "subpacks": [ { "folder_name": "a", "name": "A", "memory_tier": 1, "x": 1 } ]"#)),
        // ---- legacy: settings ----
        ("legacy settings", v2("", r#", "settings": [ { "type": "label", "text": "hi" }, { "type": "toggle", "name": "a:b", "text": "t", "default": true }, 5, { "type": "label" } ]"#)),
        ("legacy settings wrong type", v2("", r#", "settings": {}"#)),
        // ---- legacy: metadata ----
        ("legacy metadata", v2("", r#", "metadata": { "authors": ["a"], "license": "MIT", "url": "u", "product_type": "addon", "generated_with": { "t": ["1.0", [1, 2, 3]] } }"#)),
        ("legacy metadata null members", v2("", r#", "metadata": { "authors": null, "license": null, "url": null, "product_type": null, "generated_with": null }"#)),
        ("legacy metadata wrong types", v2("", r#", "metadata": { "authors": 5, "license": 5, "url": [], "product_type": "foo", "generated_with": 5 }"#)),
        ("legacy metadata authors element", v2("", r#", "metadata": { "authors": ["a", 5] }"#)),
        ("legacy metadata generated_with", v2("", r#", "metadata": { "generated_with": { "1": "x", "": ["bad", [1, 0], true], "t/u": [] } }"#)),
        ("legacy metadata unknown members", v2("", r#", "metadata": { "x": 1, "0": 2, "": 3, "+5": 4 }"#)),
        ("legacy metadata wrong type", v2("", r#", "metadata": 5"#)),
        ("legacy metadata null", v2("", r#", "metadata": null"#)),
        ("legacy metadata v0", v0(&v0_header(r#", "metadata": { "x": 1, "authors": 5 }"#))),
        // ---- legacy: format_version ----
        ("legacy format absent", v2("", "").replace(r#""format_version": 2, "#, "")),
        ("legacy format null", v2("", "").replace(r#""format_version": 2"#, r#""format_version": null"#)),
        ("legacy format real", v2("", "").replace(r#""format_version": 2"#, r#""format_version": 1.5"#)),
        ("legacy format bool", v2("", "").replace(r#""format_version": 2"#, r#""format_version": true"#)),
        ("legacy format string", v2("", "").replace(r#""format_version": 2"#, r#""format_version": "2""#)),
        ("legacy format string version", v2("", "").replace(r#""format_version": 2"#, r#""format_version": "2.0.0""#)),
        ("legacy format array", v2("", "").replace(r#""format_version": 2"#, r#""format_version": [2, 0, 0]"#)),
        ("legacy format negative", v2("", "").replace(r#""format_version": 2"#, r#""format_version": -254"#)),
        ("legacy format prerelease 3", v2("", "").replace(r#""format_version": 2"#, r#""format_version": "3.0.0-beta""#)),
        ("format beyond 3", v2("", "").replace(r#""format_version": 2"#, r#""format_version": 4"#)),
        ("format malformed", v2("", "").replace(r#""format_version": 2"#, r#""format_version": "x.y""#)),
        ("legacy root wrong type", "[1, 2]".to_owned()),
        ("legacy read order", legacy_with("2", r#""name": 5, "uuid": "abc", "version": "1", "platform_locked": 1, "pack_scope": "q""#, &legacy_modules("data"), r#", "dependencies": [5], "metadata": { "license": 5 }, "settings": [5], "subpacks": [5], "capabilities": [5], "has_education_metadata": 5"#)),
        // ---- comprehensive: unknown members ----
        ("v3 valid", v3("", "")),
        ("v3 root unknown members", v3("", r#", "x": 1, "0": 2, "": 3, "+5": 4, "a/b~c": 5"#)),
        ("v3 header unknown members", v3(r#", "x": 1, "0": 2, "": 3, "+5": 4, "a/b~c": 5"#, "")),
        ("v3 module unknown members", v3_with(&v3_header(""), &v3_modules("data").replace(r#""type""#, r#""0": 1, "": 2, "x": 3, "type""#), "")),
        ("v3 dependency unknown members", v3("", &dep(r#"{ "uuid": "33333333-3333-3333-3333-333333333333", "version": "1.0.0", "0": 1, "": 2 }"#))),
        ("v3 native dependency unknown members", v3("", &dep(r#"{ "module_name": "@minecraft/server", "version": "1.0.0", "uuid": "33333333-3333-3333-3333-333333333333", "0": 1 }"#))),
        ("v3 subpack unknown members", v3("", r#", "subpacks": [ { "folder_name": "a", "name": "A", "memory_performance_tier": 1, "memory_tier": 1, "0": 2 } ]"#)),
        ("v3 metadata unknown members", v3("", r#", "metadata": { "x": 1, "0": 2, "": 3 }"#)),
        // ---- comprehensive: string ----
        ("v3 string null", v3(r#", "description": null"#, "")),
        ("v3 string wrong type", v3(r#", "description": 5"#, "")),
        ("v3 string empty optional", v3(r#", "description": """#, "")),
        ("v3 name absent", v3_with(&format!(r#""uuid": "{HEADER_UUID}", "version": "1.0.0", "min_engine_version": "1.21.0""#), &v3_modules("data"), "")),
        ("v3 name empty", v3_with(&v3_header("").replace(r#""name": "P""#, r#""name": """#), &v3_modules("data"), "")),
        ("v3 name null", v3_with(&v3_header("").replace(r#""name": "P""#, r#""name": null"#), &v3_modules("data"), "")),
        // ---- comprehensive: boolean ----
        ("v3 boolean null", v3(r#", "platform_locked": null"#, "")),
        ("v3 boolean wrong type", v3(r#", "platform_locked": "yes", "allow_random_seed": 1"#, "")),
        ("v3 boolean root", v3("", r#", "has_education_metadata": 1"#)),
        ("v3 boolean root valid", v3("", r#", "has_education_metadata": false"#)),
        // ---- comprehensive: uuid ----
        ("v3 uuid absent", v3_with(r#""name": "P", "version": "1.0.0", "min_engine_version": "1.21.0""#, &v3_modules("data"), "")),
        ("v3 uuid null", v3_with(&v3_header("").replace(&format!(r#""{HEADER_UUID}""#), "null"), &v3_modules("data"), "")),
        ("v3 uuid wrong type", v3_with(&v3_header("").replace(&format!(r#""{HEADER_UUID}""#), "5"), &v3_modules("data"), "")),
        ("v3 uuid empty", v3_with(&v3_header("").replace(HEADER_UUID, ""), &v3_modules("data"), "")),
        ("v3 uuid malformed", v3_with(&v3_header("").replace(HEADER_UUID, "abc"), &v3_modules("data"), "")),
        // ---- comprehensive: version ----
        ("v3 version relaxed", v3_with(&v3_header("").replace(r#""version": "1.0.0""#, r#""version": "1.2""#), &v3_modules("data"), "")),
        ("v3 version malformed", v3_with(&v3_header("").replace(r#""version": "1.0.0""#, r#""version": "abc""#), &v3_modules("data"), "")),
        ("v3 version any", v3_with(&v3_header("").replace(r#""version": "1.0.0""#, r#""version": "*""#), &v3_modules("data"), "")),
        ("v3 version array", v3_with(&v3_header("").replace(r#""version": "1.0.0""#, r#""version": [1, 0, 0]"#), &v3_modules("data"), "")),
        ("v3 version absent", v3_with(&v3_header("").replace(r#", "version": "1.0.0""#, ""), &v3_modules("data"), "")),
        ("v3 version null", v3_with(&v3_header("").replace(r#""version": "1.0.0""#, r#""version": null"#), &v3_modules("data"), "")),
        // ---- comprehensive: header ----
        ("v3 header absent", format!(r#"{{ "format_version": 3, "modules": {} }}"#, v3_modules("data"))),
        ("v3 header null", format!(r#"{{ "format_version": 3, "header": null, "modules": {} }}"#, v3_modules("data"))),
        ("v3 pack_scope unknown", v3(r#", "pack_scope": "nope""#, "")),
        ("v3 pack_scope", v3(r#", "pack_scope": "global""#, "")),
        ("v3 pack_optimization_version", v3(r#", "pack_optimization_version": "1.2""#, "")),
        ("v3 pack_optimization_version bad", v3(r#", "pack_optimization_version": [1, 2, 3]"#, "")),
        ("v3 base_game_version wrong type", v3(r#", "base_game_version": [1, 21, 0]"#, "")),
        ("v3 base_game_version on data", v3(r#", "base_game_version": "1.21.0""#, "")),
        ("v3 base_game_version template", v3_with(&v3_header(r#", "base_game_version": "1.21.0", "lock_template_options": false"#).replace(r#", "min_engine_version": "1.21.0""#, ""), &v3_modules("world_template"), "")),
        ("v3 lock absent template", v3_with(&v3_header(r#", "base_game_version": "*""#).replace(r#", "min_engine_version": "1.21.0""#, ""), &v3_modules("world_template"), "")),
        ("v3 lock null template", v3_with(&v3_header(r#", "base_game_version": "*", "lock_template_options": null"#).replace(r#", "min_engine_version": "1.21.0""#, ""), &v3_modules("world_template"), "")),
        ("v3 lock on data", v3(r#", "lock_template_options": true"#, "")),
        ("v3 lock wrong type on data", v3(r#", "lock_template_options": "x""#, "")),
        // ---- comprehensive: min_engine_version ----
        ("v3 min_engine absent", v3_with(&v3_header("").replace(r#", "min_engine_version": "1.21.0""#, ""), &v3_modules("data"), "")),
        ("v3 min_engine array", v3_with(&v3_header("").replace(r#""min_engine_version": "1.21.0""#, r#""min_engine_version": [1, 21, 0]"#), &v3_modules("data"), "")),
        ("v3 min_engine malformed", v3_with(&v3_header("").replace(r#""min_engine_version": "1.21.0""#, r#""min_engine_version": "abc""#), &v3_modules("data"), "")),
        ("v3 min_engine relaxed", v3_with(&v3_header("").replace(r#""min_engine_version": "1.21.0""#, r#""min_engine_version": "1.21""#), &v3_modules("data"), "")),
        ("v3 min_engine too high", v3_with(&v3_header("").replace(r#""min_engine_version": "1.21.0""#, r#""min_engine_version": "9.0.0""#), &v3_modules("data"), "")),
        ("v3 min_engine too low", v3_with(&v3_header("").replace(r#""min_engine_version": "1.21.0""#, r#""min_engine_version": "1.0.0""#), &v3_modules("data"), "")),
        ("v3 min_engine null", v3_with(&v3_header("").replace(r#""min_engine_version": "1.21.0""#, r#""min_engine_version": null"#), &v3_modules("data"), "")),
        ("v3 min_engine on skin pack", v3_with(&v3_header(""), &v3_modules("skin_pack"), "")),
        // ---- comprehensive: modules ----
        ("v3 modules absent", format!(r#"{{ "format_version": 3, "header": {{ {} }} }}"#, v3_header(""))),
        ("v3 modules null", format!(r#"{{ "format_version": 3, "header": {{ {} }}, "modules": null }}"#, v3_header(""))),
        ("v3 modules empty", v3_with(&v3_header(""), "[]", "")),
        ("v3 modules wrong type", v3_with(&v3_header(""), r#""x""#, "")),
        ("v3 modules element wrong type", v3_with(&v3_header(""), &v3_modules("data").replace("[ {", "[ 5, {"), "")),
        ("v3 module type invalid", v3_with(&v3_header(""), &v3_modules("foo"), "")),
        ("v3 module type absent", v3_with(&v3_header(""), &format!(r#"[ {{ "uuid": "{MODULE_UUID}", "version": "1.0.0" }} ]"#), "")),
        ("v3 module uuid duplicate", v3_with(&v3_header(""), &format!(r#"[ {{ "type": "data", "uuid": "{MODULE_UUID}", "version": "1.0.0" }}, {{ "type": "script", "uuid": "{MODULE_UUID}", "version": "1.0.0" }} ]"#), "")),
        ("v3 modules inconsistent", v3_with(&v3_header(""), &format!(r#"[ {{ "type": "data", "uuid": "{MODULE_UUID}", "version": "1.0.0" }}, {{ "type": "resources", "uuid": "55555555-5555-5555-5555-555555555555", "version": "1.0.0" }} ]"#), "")),
        ("v3 module uuid is the header's", v3_with(&v3_header(""), &v3_modules("data").replace(MODULE_UUID, HEADER_UUID), "")),
        ("v3 module members", v3_with(&v3_header(""), &v3_modules("script").replace(r#""type""#, r#""description": "d", "language": "javascript", "entry": "scripts/main.js", "type""#), "")),
        // ---- comprehensive: dependencies ----
        ("v3 dependencies wrong type", v3("", r#", "dependencies": 5"#)),
        ("v3 dependencies null", v3("", r#", "dependencies": null"#)),
        ("v3 dependency element wrong type", v3("", &dep("5"))),
        ("v3 dependency pack", v3("", &dep(r#"{ "uuid": "33333333-3333-3333-3333-333333333333", "version": "1.0" }"#))),
        ("v3 dependency pack array version", v3("", &dep(r#"{ "uuid": "33333333-3333-3333-3333-333333333333", "version": [1, 0, 0] }"#))),
        ("v3 dependency module_name null", v3("", &dep(r#"{ "module_name": null, "version": "1.0.0" }"#))),
        ("v3 dependency native selectors", v3("", r#", "dependencies": [ { "module_name": "a", "version": "alpha" }, { "module_name": "b", "version": "beta" }, { "module_name": "c", "version": "internal" } ]"#)),
        ("v3 dependency native relaxed", v3("", &dep(&native(r#""2.0""#)))),
        ("v3 dependency native any", v3("", &dep(&native(r#""*""#)))),
        ("v3 dependency native malformed", v3("", &dep(&native(r#""abc""#)))),
        ("v3 dependency native array", v3("", &dep(&native("[2, 0, 0]")))),
        ("v3 dependency native no version", v3("", &dep(r#"{ "module_name": "@minecraft/server" }"#))),
        ("v3 dependency native name wrong type", v3("", &dep(r#"{ "module_name": 5, "version": "1.0.0" }"#))),
        ("v3 dependency optional", v3("", &dep(r#"{ "module_name": "@minecraft/server", "version": "1.0.0", "optional": true }"#))),
        ("v3 dependency optional wrong type", v3("", &dep(r#"{ "module_name": "@minecraft/server", "version": "1.0.0", "optional": "x" }"#))),
        ("v3 dependency optional null", v3("", &dep(r#"{ "module_name": "@minecraft/server", "version": "1.0.0", "optional": null }"#))),
        // ---- comprehensive: capabilities ----
        ("v3 capabilities", v3("", r#", "capabilities": [ 5, "script_eval", "nope", "pbr" ]"#)),
        ("v3 capabilities wrong type", v3("", r#", "capabilities": "pbr""#)),
        ("v3 capabilities null", v3("", r#", "capabilities": null"#)),
        // ---- comprehensive: subpacks ----
        ("v3 subpacks", v3("", r#", "subpacks": [ { "folder_name": "a", "name": "A", "memory_performance_tier": 5 } ]"#)),
        ("v3 subpack tier absent", v3("", r#", "subpacks": [ { "folder_name": "a", "name": "A" } ]"#)),
        ("v3 subpack tier wrong type", v3("", r#", "subpacks": [ { "folder_name": "a", "name": "A", "memory_performance_tier": "x" } ]"#)),
        ("v3 subpack tier high", v3("", r#", "subpacks": [ { "folder_name": "a", "name": "A", "memory_performance_tier": 9 } ]"#)),
        ("v3 subpack tier negative", v3("", r#", "subpacks": [ { "folder_name": "a", "name": "A", "memory_performance_tier": -1 } ]"#)),
        ("v3 subpack names missing", v3("", r#", "subpacks": [ { "memory_performance_tier": 1 } ]"#)),
        ("v3 subpack names empty", v3("", r#", "subpacks": [ { "folder_name": "", "name": "", "memory_performance_tier": 1 } ]"#)),
        ("v3 subpacks element wrong type", v3("", r#", "subpacks": [ 5 ]"#)),
        ("v3 subpacks wrong type", v3("", r#", "subpacks": {}"#)),
        // ---- comprehensive: settings ----
        ("v3 settings", v3("", r#", "settings": [ { "type": "label", "text": "hi" }, { "type": "toggle", "name": "a:b", "text": "t", "default": true } ]"#)),
        ("v3 settings empty", v3("", r#", "settings": []"#)),
        ("v3 settings wrong type", v3("", r#", "settings": {}"#)),
        ("v3 settings null", v3("", r#", "settings": null"#)),
        ("v3 settings misfit", v3("", r#", "settings": [ 5, { "type": "label" }, { "type": "nope" }, { "type": "label", "text": "a", "0": 1 }, { "type": "toggle", "name": "a:b", "text": "t", "default": 1 } ]"#)),
        ("v3 settings constraints", v3("", r#", "settings": [ { "type": "dropdown", "name": "minecraft:a", "text": "t", "options": [ { "name": "", "text": "x" } ], "default": "" }, { "type": "slider", "name": "a:s", "text": "t", "min": 2, "max": 1, "step": 0, "default": 5 } ]"#)),
        ("v3 settings multiselect", v3("", r#", "settings": [ { "type": "multiselect", "name": "a:m", "text": "t", "options": [ { "name": "x", "text": "X" }, { "name": "y", "text": "Y" } ], "defaults": ["z"] }, { "type": "toggle", "name": "a:m", "text": "t", "default": true } ]"#)),
        // ---- comprehensive: metadata ----
        ("v3 metadata", v3("", r#", "metadata": { "authors": ["a"], "license": "MIT", "url": "u", "product_type": "", "generated_with": { "t": ["1.0", "x"] } }"#)),
        ("v3 metadata null members", v3("", r#", "metadata": { "authors": null, "license": null, "url": null, "product_type": null, "generated_with": null }"#)),
        ("v3 metadata wrong types", v3("", r#", "metadata": { "authors": 5, "license": 5, "url": [], "product_type": "foo", "generated_with": 5 }"#)),
        ("v3 metadata generated_with", v3("", r#", "metadata": { "generated_with": { "1": "x", "": [[1, 0, 0], true] } }"#)),
        ("v3 metadata wrong type", v3("", r#", "metadata": 5"#)),
        ("v3 metadata null", v3("", r#", "metadata": null"#)),
        // ---- comprehensive: format_version ----
        ("v3 format string", v3("", "").replace(r#""format_version": 3"#, r#""format_version": "3.0.0""#)),
        ("v3 format array", v3("", "").replace(r#""format_version": 3"#, r#""format_version": [3, 0, 0]"#)),
        ("v3 format real", v3("", "").replace(r#""format_version": 3"#, r#""format_version": 3.0"#)),
        ("v3 root wrong type", r#"{ "format_version": 3 }"#.to_owned()),
        ("v3 read order", v3_with(r#""name": 5, "uuid": "abc", "version": "x", "platform_locked": 1, "pack_scope": "q", "z": 1"#, &v3_modules("foo"), r#", "dependencies": [5], "metadata": { "license": 5 }, "settings": [5], "subpacks": [5], "capabilities": [5], "has_education_metadata": 5, "q": 1"#)),
        // ---- the text ----
        ("syntax error", r#"{ "format_version": 2, "#.to_owned()),
        ("empty text", String::new()),
        ("duplicate members", v2("", r#", "dependencies": 5, "dependencies": []"#)),
    ]
}

/// A path as text and as its segments.
fn path_text(path: &bedrock_addon_core::diagnostic::JsonPath) -> String {
    let segments: Vec<String> = path
        .segments()
        .map(|segment| match segment {
            JsonPathSegment::Key(key) => format!("{key:?}"),
            JsonPathSegment::Index(index) => format!("#{index}"),
        })
        .collect();
    format!("`{path}` [{}]", segments.join(", "))
}

/// What the reader makes of `text`: the findings in order, then the value.
fn outcome(text: &str) -> String {
    let result = Manifest::parse(
        &PackPath::from_static("manifest.json"),
        text.as_bytes(),
        &CapabilityRegistry::default(),
    );
    let (diagnostics, document): (_, Option<&Document<Manifest>>) = match &result {
        Ok(document) => (document.diagnostics(), Some(document)),
        Err(rejected) => (&rejected.diagnostics, None),
    };
    let mut out = String::new();
    for finding in diagnostics {
        let _ = writeln!(
            out,
            "{:?} | {:?} | {:?} | {}",
            finding.code,
            finding.severity,
            finding.phase,
            path_text(&finding.location.json_path)
        );
    }
    match document {
        None => out.push_str("rejected\n"),
        Some(document) => {
            let manifest = document.value();
            let _ = writeln!(
                out,
                "format {:?}/{:?} pack {:?} min_engine {:?} scope {:?}",
                manifest.format,
                manifest.original_format,
                manifest.pack_type(),
                manifest.header.min_engine_version,
                manifest.header.pack_scope,
            );
            let _ = writeln!(out, "dependencies {:?}", manifest.dependencies);
            let _ = writeln!(out, "subpacks {:?}", manifest.subpacks);
            let _ = writeln!(out, "settings {:?}", manifest.settings);
            match manifest.to_json() {
                Ok(written) => {
                    let _ = writeln!(out, "{written}");
                }
                Err(error) => {
                    let _ = writeln!(out, "not written: {error:?}");
                }
            }
        }
    }
    out
}

#[test]
fn the_readers_findings_and_values_are_pinned() {
    let mut actual = String::new();
    let mut names = std::collections::HashSet::new();
    for (name, text) in cases() {
        assert!(names.insert(name), "two cases are named {name}");
        let _ = write!(actual, "## {name}\n{}\n", outcome(&text));
    }
    if std::env::var("BEDROCK_ADDON_CHARACTERIZE").as_deref() == Ok("write") {
        std::fs::write(pins(), &actual).expect("pin file written");
        return;
    }
    let expected = std::fs::read_to_string(pins()).expect("pin file");
    let sections = |text: &str| -> Vec<(String, String)> {
        text.split("## ")
            .filter(|section| !section.is_empty())
            .map(|section| {
                let (name, body) = section.split_once('\n').unwrap_or((section, ""));
                (name.to_owned(), body.to_owned())
            })
            .collect()
    };
    let (expected, actual) = (sections(&expected), sections(&actual));
    let differing: Vec<String> = expected
        .iter()
        .zip(&actual)
        .filter(|(e, a)| e != a)
        .map(|((name, e), (_, a))| format!("## {name}\n-- pinned\n{e}-- now\n{a}"))
        .collect();
    assert!(differing.is_empty(), "{}", differing.join("\n"));
    assert_eq!(expected.len(), actual.len(), "the number of cases");
}
