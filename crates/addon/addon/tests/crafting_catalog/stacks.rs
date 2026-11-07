//! Stacks for the merge tests: packs with a catalogue file, the vanilla behaviour pack by its
//! UUID and origin, and the item registry of `merge/items.json`.

#![allow(dead_code)]

use std::sync::Arc;

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::document::Merged;
use bedrock_addon::families::crafting_catalog::CraftingCatalog;
use bedrock_addon::json::Value;
use bedrock_addon::load::{BuiltIns, LoadOptions};
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackOrigin, PackStack};
use bedrock_addon::storage::PackPath;
use uuid::Uuid;

use super::common::*;

/// The header UUID every vanilla behaviour pack has.
pub const VANILLA: Uuid = Uuid::from_u128(0xfe9f8597_5454_481a_8730_8d070a8e2e58);

/// One pack to put on a stack.
#[derive(Clone, Debug)]
pub struct Spec {
    pub name: String,
    pub uuid: Uuid,
    pub origin: PackOrigin,
    pub files: Vec<(String, Vec<u8>)>,
}

impl Spec {
    /// The pack with the file `path`.
    pub fn file(mut self, path: &str, bytes: impl Into<Vec<u8>>) -> Self {
        self.files.push((path.to_owned(), bytes.into()));
        self
    }

    /// The pack with the catalogue `json`.
    pub fn catalog(self, json: &str) -> Self {
        self.file(PATH, json)
    }

    /// The opened pack.
    pub fn open(&self) -> Pack {
        let manifest = format!(
            r#"{{"format_version": 2, "header": {{"name": "{}", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 21, 60]}}, "modules": [{{"type": "data", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
            self.name,
            self.uuid,
            Uuid::from_u128(self.uuid.as_u128() ^ 0xabcd),
        );
        let files = std::iter::once(("manifest.json".to_owned(), manifest.into_bytes()))
            .chain(self.files.iter().cloned())
            .map(|(path, bytes)| (PackPath::new(&path).expect("a pack path"), bytes));
        let options = OpenOptions {
            origin: self.origin,
            ..OpenOptions::default()
        };
        Pack::from_files(files, &options).expect("the pack opens")
    }
}

/// An add-on behaviour pack named `name`.
pub fn addon(name: &str) -> Spec {
    let seed = name
        .bytes()
        .fold(7_u128, |seed, byte| seed * 131 + u128::from(byte));
    Spec {
        name: name.to_owned(),
        uuid: Uuid::from_u128(seed),
        origin: PackOrigin::User,
        files: Vec::new(),
    }
}

/// A vanilla behaviour pack (a slice of it) named `name`: the vanilla UUID, from the package.
pub fn vanilla(name: &str) -> Spec {
    Spec {
        uuid: VANILLA,
        origin: PackOrigin::Package,
        ..addon(name)
    }
}

/// The entries of `merge/items.json`: the name and the `menu_category` JSON, if any.
pub fn registry_items() -> Vec<(String, Option<String>)> {
    let value: Value = facet_json::from_str(&fixture("merge/items.json")).unwrap();
    value
        .as_object()
        .unwrap()
        .iter()
        .map(|(name, entry)| {
            let menu = entry
                .as_object()
                .unwrap()
                .get("menu_category")
                .map(|menu| facet_json::to_string(menu).unwrap());
            (name.as_str().to_owned(), menu)
        })
        .collect()
}

/// The pack that defines the custom items of `merge/items.json`.
pub fn registry_pack() -> Spec {
    let mut pack = addon("registry");
    for (name, menu) in registry_items() {
        if name.starts_with("minecraft:") {
            continue;
        }
        let menu = menu.map_or_else(String::new, |menu| format!(r#", "menu_category": {menu}"#));
        let file = format!("items/{}.json", name.replace(':', "_"));
        pack = pack.file(
            &file,
            format!(
                r#"{{"format_version": "1.21.60", "minecraft:item": {{"description": {{"identifier": "{name}"{menu}}}, "components": {{"minecraft:max_stack_size": 64}}}}}}"#
            ),
        );
    }
    pack
}

/// The built-in items of `merge/items.json` (its `minecraft:` entries), as a host answers
/// them.
#[derive(Debug)]
pub struct RegistryBuiltIns(Vec<String>);

impl BuiltIns for RegistryBuiltIns {
    fn contains(&self, kind: &'static str, id: &str) -> bool {
        kind == "item" && self.0.iter().any(|name| name == id)
    }
}

/// Options whose built-ins are the `minecraft:` entries of `merge/items.json`.
pub fn registry_options() -> LoadOptions {
    let names = registry_items()
        .into_iter()
        .map(|(name, _)| name)
        .filter(|name| name.starts_with("minecraft:"))
        .collect();
    LoadOptions {
        built_ins: Some(Arc::new(RegistryBuiltIns(names))),
        ..LoadOptions::default()
    }
}

/// A behaviour stack of `packs`, given bottom first (load order), under `options`.
pub fn stack_of(packs: &[Spec], options: LoadOptions) -> PackStack<Behavior> {
    let top_first: Vec<Pack> = packs.iter().rev().map(Spec::open).collect();
    PackStack::<Behavior>::of(top_first, options).expect("a behaviour stack")
}

/// The packs of the merge fixture `merge/<name>/`, bottom first: `pack<N>.json` an add-on,
/// `pack<N>_vanilla.json` a vanilla slice, `pack<N>_vanilla.nofile` a vanilla slice without a
/// catalogue; the pack of `merge/items.json` goes on top.
pub fn fixture_packs(name: &str) -> Vec<Spec> {
    let dir = format!("{FIXTURES}/merge/{name}");
    let mut files: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("{dir}: {error}"))
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|file| file.starts_with("pack"))
        .collect();
    files.sort();
    let mut packs: Vec<Spec> = files
        .iter()
        .map(|file| {
            let (stem, extension) = file.rsplit_once('.').unwrap();
            let pack_name = stem.split('_').next().unwrap();
            let pack = if stem.contains("_vanilla") {
                vanilla(pack_name)
            } else {
                addon(pack_name)
            };
            if extension == "json" {
                pack.catalog(&fixture(&format!("merge/{name}/{file}")))
            } else {
                pack
            }
        })
        .collect();
    packs.push(registry_pack());
    packs
}

/// The stack of the merge fixture `merge/<name>/` with the registry of `merge/items.json`.
pub fn fixture_stack(name: &str) -> PackStack<Behavior> {
    stack_of(&fixture_packs(name), registry_options())
}

/// The catalogue `merge/<name>/expected_merge.json` holds.
pub fn expected_merge(name: &str) -> CraftingCatalog {
    loads(&fixture(&format!("merge/{name}/expected_merge.json"))).into_value()
}

/// The findings of a merged catalogue above info, as `(code, severity, pack, text)`.
pub fn found(merged: &Merged<CraftingCatalog>) -> Vec<(String, Severity, String, String)> {
    merged
        .diagnostics()
        .filter(|f| f.severity > Severity::Info)
        .map(|f| {
            (
                f.code.id().to_owned(),
                f.severity,
                pack_of(f),
                f.code.to_string(),
            )
        })
        .collect()
}

/// The name of the pack a finding is in (`""` without one).
pub fn pack_of(finding: &Diagnostic) -> String {
    finding
        .location
        .pack
        .as_ref()
        .map(|pack| pack.name().to_owned())
        .unwrap_or_default()
}
