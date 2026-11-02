//! Stacks in memory and a host that answers from a list.

#![allow(dead_code, reason = "each feature set uses part of the helpers")]

use std::sync::Arc;

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticDetail, Diagnostics, Severity};
use bedrock_addon_core::document::Origin;
use bedrock_addon_core::load::{BuiltIns, LoadOptions};
use bedrock_addon_core::pack_identity::{
    PackIdVersion, PackOrigin, PackPath, PackRef, PackSide, PackType,
};
use bedrock_addon_core::version::SemVersion;
use uuid::Uuid;

use crate::pack::tests::pack_from;
use crate::pack::{Pack, PackStack, StackSet};

/// A format 2 manifest of pack `n` with one module of `module` (`data`, `resources`).
fn manifest(n: u8, module: &str) -> String {
    format!(
        r#"{{ "format_version": 2,
  "header": {{ "name": "pack {n}", "description": "", "uuid": "00000000-0000-4000-8000-0000000000{n:02}",
              "version": [1, 0, 0], "min_engine_version": [1, 21, 0] }},
  "modules": [{{ "type": "{module}", "uuid": "00000000-0000-4000-9000-0000000000{n:02}", "version": [1, 0, 0] }}] }}"#
    )
}

/// `text` for the life of the test process: a pack's files are `'static`.
pub(in crate::pack::link) fn leak(text: &str) -> &'static [u8] {
    Box::leak(text.to_owned().into_boxed_str()).as_bytes()
}

/// One file of a test pack: its path and its text, or `None` for a file that is indexed and
/// not resident.
pub(in crate::pack::link) type File<'f> = (&'static str, Option<&'f str>);

/// The pack `n` with `files` and a module of `module`.
pub(in crate::pack::link) fn pack(n: u8, module: &str, files: &[File<'_>]) -> Pack {
    let files: Vec<(&'static str, Option<&'static [u8]>)> = files
        .iter()
        .map(|(path, text)| (*path, text.map(leak)))
        .collect();
    pack_from(&manifest(n, module), &files, PackOrigin::World)
}

/// The stacks of the given packs, top first on each side.
pub(in crate::pack::link) fn set_of(
    behavior: Vec<Pack>,
    resource: Vec<Pack>,
    options: &LoadOptions,
) -> StackSet {
    StackSet {
        behavior: PackStack::of(behavior, options.clone()).expect("behaviour packs"),
        resource: PackStack::of(resource, options.clone()).expect("resource packs"),
    }
}

/// A world of one behaviour pack with `behavior` and one resource pack with `resource` (each
/// file resident), loaded with `options`.
pub(in crate::pack::link) fn set_with(
    behavior: &[(&'static str, &str)],
    resource: &[(&'static str, &str)],
    options: &LoadOptions,
) -> StackSet {
    let files_of = |files: &[(&'static str, &str)]| -> Vec<File<'static>> {
        files
            .iter()
            .map(|(path, text)| {
                let text: &'static str = Box::leak((*text).to_owned().into_boxed_str());
                (*path, Some(text))
            })
            .collect()
    };
    set_of(
        vec![pack(1, "data", &files_of(behavior))],
        vec![pack(2, "resources", &files_of(resource))],
        options,
    )
}

/// [`set_with`] and the default options: no host.
pub(in crate::pack::link) fn set(
    behavior: &[(&'static str, &str)],
    resource: &[(&'static str, &str)],
) -> StackSet {
    set_with(behavior, resource, &LoadOptions::default())
}

/// A world without a pack.
pub(in crate::pack::link) fn empty_set() -> StackSet {
    set_of(Vec::new(), Vec::new(), &LoadOptions::default())
}

/// A host whose game has exactly the listed `(kind, id)` pairs.
pub(in crate::pack::link) struct Host(
    pub(in crate::pack::link) &'static [(&'static str, &'static str)],
);

impl BuiltIns for Host {
    fn contains(&self, kind: &'static str, id: &str) -> bool {
        self.0.contains(&(kind, id))
    }
}

/// The options of a world whose host is `host`.
pub(in crate::pack::link) fn hosted(host: &'static [(&'static str, &'static str)]) -> LoadOptions {
    LoadOptions {
        built_ins: Some(Arc::new(Host(host))),
        ..LoadOptions::default()
    }
}

/// An origin of a document at `path` of a behaviour pack called `pack 7`.
pub(in crate::pack::link) fn origin(path: &'static str) -> Origin {
    let id = PackIdVersion::new(
        Uuid::from_u128(7),
        SemVersion::new(1, 0, 0),
        PackType::Behavior,
    );
    Origin::new(
        PackRef::new(0, id, "pack 7".to_owned()),
        PackSide::Behavior,
        PackPath::from_static(path),
        None,
        Vec::new(),
    )
}

/// What a finding says of a reference: the code's name, the severity, the JSON path, the kind
/// and the name as written.
pub(in crate::pack::link) fn said(
    finding: &Diagnostic,
) -> (&'static str, Severity, String, &'static str, String) {
    let (kind, id) = match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { kind, id, .. }) => (*kind, id.clone()),
        _ => ("", String::new()),
    };
    (
        finding.code.name(),
        finding.severity,
        finding.location.json_path.to_string(),
        kind,
        id,
    )
}

/// [`said`] of every finding.
pub(in crate::pack::link) fn all_said(
    findings: &Diagnostics,
) -> Vec<(&'static str, Severity, String, &'static str, String)> {
    findings.iter().map(said).collect()
}

/// A custom block `id` with the JSON `states` object (`{}` for none).
pub(in crate::pack::link) fn block(id: &str, states: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.20", "minecraft:block": {{"description": {{"identifier": "{id}", "states": {states}}}, "components": {{}}}}}}"#
    )
}

/// A custom item `id` with the JSON members `components`.
pub(in crate::pack::link) fn item(id: &str, components: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.30", "minecraft:item": {{"description": {{"identifier": "{id}"}}, "components": {{{components}}}}}}}"#
    )
}

/// A custom entity `id`.
pub(in crate::pack::link) fn entity(id: &str) -> String {
    format!(
        r#"{{"format_version": "1.21.0", "minecraft:entity": {{"description": {{"identifier": "{id}", "is_spawnable": true, "is_summonable": true, "is_experimental": false}}, "components": {{}}}}}}"#
    )
}

/// The text, in lower case, of every file below the folders `folders` of every behaviour
/// pack of the game's installs.
#[allow(
    dead_code,
    reason = "which tests exist depends on the enabled features"
)]
pub(in crate::pack::link) fn vanilla_behavior_text(
    game_files: &std::path::Path,
    folders: &[&str],
) -> String {
    fn read(dir: &std::path::Path, text: &mut String) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for path in entries.flatten().map(|entry| entry.path()) {
            if path.is_dir() {
                read(&path, text);
            } else if let Ok(file) = std::fs::read_to_string(&path) {
                text.push_str(&file.to_ascii_lowercase());
            }
        }
    }
    let mut roots = vec![game_files.join("bedrock-samples").join("behavior_pack")];
    for install in ["bds-1.26.36.1", "gdk-retail"] {
        roots.extend(
            std::fs::read_dir(game_files.join(install).join("behavior_packs"))
                .into_iter()
                .flatten()
                .flatten()
                .map(|entry| entry.path()),
        );
    }
    assert!(roots.len() > 100, "{} behaviour packs", roots.len());
    let mut text = String::new();
    for root in roots {
        for folder in folders {
            read(&root.join(folder), &mut text);
        }
    }
    text
}

/// Whether `text` (lower case) writes `name` (lower case, with its namespace) as a JSON
/// string: with its namespace or bare, alone or before an aux value.
#[allow(
    dead_code,
    reason = "which tests exist depends on the enabled features"
)]
pub(in crate::pack::link) fn written_in(text: &str, name: &str) -> bool {
    let bare = name.trim_start_matches("minecraft:");
    [
        format!("\"{name}\""),
        format!("\"{name}:"),
        format!("\"{bare}\""),
        format!("\"{bare}:"),
    ]
    .iter()
    .any(|written| text.contains(written.as_str()))
}
