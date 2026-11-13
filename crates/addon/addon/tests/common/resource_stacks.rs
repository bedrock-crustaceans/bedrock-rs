//! Stacks for the tests of the fixed-path files (splashes, loading messages, credits, fonts, UI
//! side files, skins): in-memory resource packs, lowest priority first, with an optional
//! active subpack and optional app data.

#![allow(dead_code, reason = "each test module uses part of the helpers")]

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::document::Content;
use bedrock_addon::load::LoadContext;
use bedrock_addon::load::{AppData, LoadOptions};
use bedrock_addon::pack::{
    OpenOptions, Pack, PackOrigin, PackRepository, PackStack, Resource, StackSet, WorldPacks,
};
use bedrock_addon::storage::PackPath;

#[allow(unused_imports, reason = "each test file uses part of the helpers")]
pub use bedrock_addon_test_support::json;

/// The UUID of pack `n`.
pub fn pack_uuid(n: usize) -> uuid::Uuid {
    uuid::Uuid::from_u128(n as u128 + 1)
}

/// A format-2 manifest for pack `n` with a module of `module` and the declared `subpacks`.
pub fn manifest(n: usize, module: &str, subpacks: &[&str]) -> String {
    let uuid = pack_uuid(n);
    let module_uuid = uuid::Uuid::from_u128((n as u128 + 1) ^ (0xabcd_u128 << 64));
    let subpacks: Vec<String> = subpacks
        .iter()
        .map(|folder| {
            format!(r#"{{"folder_name": "{folder}", "name": "{folder}", "memory_tier": 0}}"#)
        })
        .collect();
    format!(
        r#"{{"format_version": 2, "header": {{"name": "pack {n}", "description": "", "uuid": "{uuid}", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]}}, "modules": [{{"type": "{module}", "uuid": "{module_uuid}", "version": [1, 0, 0]}}], "subpacks": [{}]}}"#,
        subpacks.join(", ")
    )
}

/// Pack `n` of module type `module` holding `files`.
pub fn pack_with(n: usize, module: &str, subpacks: &[&str], files: &[(&str, &[u8])]) -> Pack {
    let mut all: Vec<(PackPath, bytes::Bytes)> = vec![(
        PackPath::from_static("manifest.json"),
        bytes::Bytes::from(manifest(n, module, subpacks)),
    )];
    all.extend(files.iter().map(|(path, bytes)| {
        (
            PackPath::new(path).unwrap(),
            bytes::Bytes::from(bytes.to_vec()),
        )
    }));
    Pack::from_files(all, &OpenOptions::default()).expect("the pack opens")
}

/// A resource stack of in-memory packs of text files, given lowest priority first.
pub fn stack_of(packs: &[&[(&str, &str)]]) -> PackStack<Resource> {
    let bytes: Vec<Vec<(&str, &[u8])>> = packs
        .iter()
        .map(|files| files.iter().map(|(p, t)| (*p, t.as_bytes())).collect())
        .collect();
    let slices: Vec<&[(&str, &[u8])]> = bytes.iter().map(Vec::as_slice).collect();
    stack_with(&slices, LoadOptions::default())
}

/// A resource stack of in-memory packs, given lowest priority first, loaded with `options`.
pub fn stack_with(packs: &[&[(&str, &[u8])]], options: LoadOptions) -> PackStack<Resource> {
    let mut packs: Vec<Pack> = packs
        .iter()
        .enumerate()
        .map(|(n, files)| pack_with(n, "resources", &[], files))
        .collect();
    packs.reverse();
    PackStack::of(packs, options).expect("resource packs")
}

/// Options whose app data holds `files`.
pub fn app_data(files: &[(&str, &str)]) -> LoadOptions {
    LoadOptions {
        app_data: Some(AppData::from_files(files.iter().map(|(path, text)| {
            (PackPath::new(path).unwrap(), text.as_bytes().to_vec())
        }))),
        ..LoadOptions::default()
    }
}

/// The stacks of one resource pack with `files` whose declared subpack `hi` the world selects.
pub fn subpack_stacks(files: &[(&str, &[u8])]) -> StackSet {
    let pack = pack_with(0, "resources", &["hi"], files);
    let entries = format!(
        r#"[{{"pack_id": "{}", "version": [1, 0, 0], "subpack": "hi"}}]"#,
        pack_uuid(0)
    );
    let world = WorldPacks::parse(None, Some(entries.as_bytes())).unwrap();
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::World);
    let set = repository.stacks(&world, LoadOptions::default());
    assert_eq!(set.resource.packs().count(), 1, "the pack is on the stack");
    set
}

/// A finding as `(code name, severity, text)`.
pub fn shown(diagnostic: &Diagnostic) -> (&'static str, Severity, String) {
    (
        diagnostic.code.id(),
        diagnostic.severity,
        diagnostic.code.to_string(),
    )
}

/// Findings as `(code name, severity)`, in order.
pub fn codes<'a>(
    diagnostics: impl IntoIterator<Item = &'a Diagnostic>,
) -> Vec<(&'static str, Severity)> {
    diagnostics
        .into_iter()
        .map(|d| (d.code.id(), d.severity))
        .collect()
}

/// Findings as `(code name, severity, JSON path)`, in order.
pub fn located<'a>(
    diagnostics: impl IntoIterator<Item = &'a Diagnostic>,
) -> Vec<(&'static str, Severity, String)> {
    diagnostics
        .into_iter()
        .map(|d| (d.code.id(), d.severity, d.location.json_path.to_string()))
        .collect()
}

/// The number of the pack (as given to `stack_of`, lowest first) a finding is about, in a
/// stack of `count` packs.
pub fn pack_number(diagnostic: &Diagnostic, count: usize) -> Option<usize> {
    let index = diagnostic.location.pack.as_ref()?.index()?;
    Some(count - 1 - index)
}

/// The stack's own findings about the files its packs carry in vain (the credits
/// and "persona/"): the `IgnoredFile` infos of its report that no collection raised, in the
/// report's load order.
pub fn in_vain(
    stack: &bedrock_addon::pack::PackStack<bedrock_addon::pack::Resource>,
) -> Vec<Diagnostic> {
    stack
        .report()
        .iter()
        .filter(|d| d.code.id() == "core/ignored_file")
        .cloned()
        .collect()
}

/// Every text loads and is written back as the same JSON value.
pub fn as_authored<T: Content>(texts: &[&str]) {
    for text in texts {
        let path = PackPath::from_static("x.json");
        let document = T::parse(&path, text.as_bytes(), &LoadContext::detached())
            .unwrap_or_else(|rejected| panic!("{text}: {:?}", rejected.diagnostics));
        let written = document
            .to_json()
            .unwrap_or_else(|e| panic!("{text}: {e:?}"));
        assert_eq!(json(&written), json(text), "{} wrote {written}", T::NAME);
        // What is written is stable, and loads to the same document.
        let again = T::parse(&path, written.as_bytes(), &LoadContext::detached()).unwrap();
        assert_eq!(again.to_json().unwrap(), written, "{text}");
    }
}
