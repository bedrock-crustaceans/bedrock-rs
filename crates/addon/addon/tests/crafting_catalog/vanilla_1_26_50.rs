//! Vanilla's 1.26.50 catalogue (`behavior_packs/vanilla_1.26.50/item_catalog/
//! crafting_item_catalog.json` of 1.26.60), the strict slice's oracle.
//! It ships MCB-encoded (header `7f 4d 43 42 01 00 1a 00 32 00`, 1.26.50); its decoded twin (a
//! scratch decode with the bind order) is the strict slice's fixture, and the storage's
//! catalogue body decoder reads the original to the same document.

use bedrock_addon::document::Content;
use bedrock_addon::families::crafting_catalog::CategoryName;
use bedrock_addon::version::FormatVersion;

use super::common::*;
use bedrock_addon::pack::PackOrigin;

use super::stacks::{Spec, addon, stack_of, vanilla};

/// The decoded twin of the MCB original.
fn decoded() -> String {
    fixture("vanilla_1_26_50/crafting_item_catalog.decoded.json")
}

#[test]
fn the_1_26_50_catalogue_loads_under_the_strict_slice_without_a_finding() {
    let document = loads(&decoded());
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 26, 50));
    let names: Vec<CategoryName> = document
        .categories
        .iter()
        .map(|category| category.category_name)
        .collect();
    assert_eq!(
        names,
        [
            CategoryName::Construction,
            CategoryName::Nature,
            CategoryName::Equipment,
            CategoryName::Items
        ]
    );
    let planks = &document.categories[0].groups[0];
    let identifier = planks.group_identifier.as_ref().unwrap();
    assert_eq!(identifier.name, "minecraft:itemGroup.name.planks");
    assert_eq!(
        identifier.icon.as_ref().map(|icon| icon.name.as_str()),
        Some("minecraft:oak_planks")
    );
    assert_eq!(planks.items.len(), 14);
}

#[test]
fn the_1_26_50_catalogue_round_trips_at_its_version() {
    let document = loads(&decoded());
    let written = document.to_json().unwrap();
    assert!(written.contains(r#""format_version": "1.26.50""#));
    let again = loads(&written);
    assert_eq!(again.value(), document.value());
    assert_eq!(
        again.to_json().unwrap(),
        written,
        "the second write differs"
    );
}

/// The MCB originals, the files the game ships (1.21.60, 1.26.40, 1.26.50 and the drop 4
/// pack's 1.26.40), each load on a vanilla stack without a finding; the 1.26.50 one is
/// the decoded fixture.
#[test]
fn the_mcb_originals_load_on_a_vanilla_stack_without_a_finding() {
    for (name, version) in [
        ("vanilla", FormatVersion::new(1, 21, 60)),
        ("vanilla_1.26.40", FormatVersion::new(1, 26, 40)),
        ("vanilla_1.26.50", FormatVersion::new(1, 26, 50)),
        ("experimental_y_2026_drop_4", FormatVersion::new(1, 26, 40)),
    ] {
        let bytes =
            std::fs::read(format!("{FIXTURES}/mcb/{name}_crafting_item_catalog.mcb")).unwrap();
        assert_eq!(&bytes[..4], b"\x7fMCB", "{name}");
        // The pack's name seeds its UUID: a short one (the vanilla slice keeps the vanilla UUID).
        // The drop 4 pack adds to vanilla's catalogue: it is stacked on the 1.26.50 one, as in
        // the game.
        let mut packs = Vec::new();
        if name == "experimental_y_2026_drop_4" {
            let base = std::fs::read(format!(
                "{FIXTURES}/mcb/vanilla_1.26.50_crafting_item_catalog.mcb"
            ))
            .unwrap();
            packs.push(vanilla("vanilla").file(PATH, base));
            // The experiment's pack is a game pack of its own UUID.
            packs.push(
                Spec {
                    origin: PackOrigin::Package,
                    ..addon("drop4")
                }
                .file(PATH, bytes),
            );
        } else {
            packs.push(vanilla("vanilla").file(PATH, bytes));
        }
        let stack = stack_of(&packs, Default::default());
        let merged = stack.crafting_catalog();
        let layers = merged.layers();
        assert_eq!(layers.len(), packs.len(), "{name}");
        for layer in layers {
            assert!(
                layer.diagnostics().is_empty(),
                "{name}: {:#?}",
                layer.diagnostics()
            );
        }
        // The pack's own layer (the other one is the 1.26.50 base under the drop 4 pack).
        let layer = layers
            .iter()
            .find(|layer| layer.version().written == Some(version))
            .unwrap_or_else(|| panic!("{name}: no layer at {version}"));
        // The link pass: the drop 4 catalogue places the experiment's own items (that pack's
        // `items/` defines them; it has no `blocks/`), which the game registers in 1.26.60,
        // so nothing is told.
        let referenced: Vec<String> = stack
            .report()
            .iter()
            .map(|d| match d.detail.as_deref() {
                Some(bedrock_addon::diagnostic::DiagnosticDetail::Reference { id, .. })
                    if d.code.name() == "UnresolvedReference" =>
                {
                    id.clone()
                }
                _ => format!("{} {}", d.code.name(), d.code),
            })
            .collect();
        assert_eq!(referenced, Vec::<String>::new(), "{name}");
        if name == "vanilla_1.26.50" {
            assert_eq!(layer.value(), loads(&decoded()).value());
        }
    }
}
