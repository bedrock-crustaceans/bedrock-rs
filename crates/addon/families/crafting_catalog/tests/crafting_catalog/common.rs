//! Helpers of the catalogue tests.

#![allow(dead_code)]

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, Severity};
use bedrock_addon_core::document::{Content, Document, Rejected};
use bedrock_addon_core::id::ItemId;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_crafting_catalog::{
    CatalogItem, Category, CategoryName, CraftingCatalog, Group, GroupIdentifier,
};

#[allow(unused_imports, reason = "each test file uses part of the helpers")]
pub use bedrock_addon_test_support::errors;

/// The fixture folder of this family. The file is included by the family crate's tests and by
/// `bedrock_addon`'s, whose manifest folders sit at different depths, so the folder is found
/// below the fixture root of the package that compiles it.
pub fn fixtures() -> std::path::PathBuf {
    bedrock_addon_test_support::fixture::root(env!("CARGO_MANIFEST_DIR")).join("item/catalog")
}

/// The one path a catalogue is read from.
pub const PATH: &str = "item_catalog/crafting_item_catalog.json";

/// The text of the fixture file `path` (relative to the family's fixture folder).
pub fn fixture(path: &str) -> String {
    std::fs::read_to_string(fixtures().join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// Loads `json` as a pack's catalogue file.
pub fn load(json: &str) -> Result<Document<CraftingCatalog>, Rejected> {
    CraftingCatalog::parse(
        &PackPath::from_static(PATH),
        json.as_bytes(),
        &LoadContext::detached(),
    )
}

/// Loads `json`, which must load.
#[track_caller]
pub fn loads(json: &str) -> Document<CraftingCatalog> {
    match load(json) {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {:#?}", rejected.diagnostics),
    }
}

/// Loads `json`, which must be rejected; its findings.
#[track_caller]
pub fn rejects(json: &str) -> Diagnostics {
    match load(json) {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// The one error-level finding of `d`.
#[track_caller]
pub fn one_error(d: &Diagnostics) -> &Diagnostic {
    let errors = errors(d);
    assert_eq!(errors.len(), 1, "one error expected: {d:#?}");
    errors[0]
}

/// The findings as `(code, severity, text)`, the shape of a `<name>.expected.json` list.
pub fn listed(d: &Diagnostics) -> Vec<(String, String, String)> {
    d.iter()
        .filter(|f| f.severity > Severity::Info)
        .map(|f| {
            (
                f.code.id().to_owned(),
                f.severity.to_string(),
                f.code.to_string(),
            )
        })
        .collect()
}

/// The `(code, severity, text)` rows of the expected-diagnostics fixture `path`.
pub fn expected_list(path: &str) -> Vec<(String, String, String)> {
    let value: facet_value::Value = facet_json::from_str(&fixture(path)).unwrap();
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let member = |key: &str| {
                row.as_object()
                    .unwrap()
                    .get(key)
                    .unwrap()
                    .as_string()
                    .unwrap()
                    .as_str()
                    .to_owned()
            };
            (member("code"), member("severity"), member("text"))
        })
        .collect()
}

/// An item written as `name`.
pub fn item(name: &str) -> CatalogItem {
    CatalogItem {
        name: ItemId::parse(name),
    }
}

/// A named group.
pub fn named(name: &str, icon: Option<&str>, items: &[&str]) -> Group {
    Group {
        group_identifier: Some(GroupIdentifier {
            name: name.to_owned(),
            icon: icon.map(item),
        }),
        items: items.iter().copied().map(item).collect(),
    }
}

/// A group of loose items.
pub fn loose(items: &[&str]) -> Group {
    Group {
        group_identifier: None,
        items: items.iter().copied().map(item).collect(),
    }
}

/// One `categories` entry.
pub fn category(category_name: CategoryName, groups: Vec<Group>) -> Category {
    Category {
        category_name,
        groups,
    }
}

/// A catalogue of `categories`.
pub fn catalog(categories: Vec<Category>) -> CraftingCatalog {
    CraftingCatalog { categories }
}

/// The document of `cat01_all_categories.json`, built by hand.
pub fn cat01() -> CraftingCatalog {
    catalog(vec![
        category(
            CategoryName::Construction,
            vec![named(
                "custom:building",
                Some("custom:item1"),
                &["custom:item1", "custom:item2"],
            )],
        ),
        category(
            CategoryName::Equipment,
            vec![named(
                "custom:tools",
                Some("custom:my_icon:1"),
                &["custom:item3"],
            )],
        ),
        category(
            CategoryName::Items,
            vec![loose(&["custom:item4", "minecraft:stone"])],
        ),
        category(
            CategoryName::Nature,
            vec![named("custom:plants", None, &[])],
        ),
    ])
}
