//! The shipped game's re-serialised vanilla items, as a check of the upgrade chain. The
//! files are read from the out-of-repo corpus (1.26.20 development and 1.26.45 shipped data), never
//! committed; the tests are skipped without `BEDROCK_ADDON_CORPUS`.

use std::path::{Path, PathBuf};

use bedrock_addon_core::document::ComponentRef;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_family_item::Item;
use bedrock_addon_family_item::component::{ItemComponent, MaxStackSize, StartUsing, UseModifiers};
use bedrock_addon_shared_shapes::CreativeCategory;

use super::common::*;
use bedrock_addon_test_support::corpus;

/// The corpus entry `entry`, or `None` after printing the skip line of `check`: the variable
/// is not set, or the corpus has no such entry.
fn corpus(check: &str, entry: &str) -> Option<PathBuf> {
    let Some(root) = std::env::var_os(corpus::CORPUS) else {
        corpus::skipped(check, corpus::not_set(corpus::CORPUS));
        return None;
    };
    let path = Path::new(&root).join(entry);
    if path.exists() {
        Some(path)
    } else {
        corpus::skipped(check, format_args!("{} is missing", path.display()));
        None
    }
}

/// The 22 pairs: pack folder and item file.
fn pairs() -> Vec<(&'static str, String)> {
    let mut out = vec![("vanilla_1.20.50", "apple".to_owned())];
    for name in ["ominous_trial_key", "trial_key", "wind_charge"] {
        out.push(("vanilla_1.21.0", name.to_owned()));
    }
    out.push(("vanilla_1.21.10", "breeze_rod".to_owned()));
    for colour in [
        "",
        "black_",
        "blue_",
        "brown_",
        "cyan_",
        "gray_",
        "green_",
        "light_blue_",
        "light_gray_",
        "lime_",
        "magenta_",
        "orange_",
        "pink_",
        "purple_",
        "red_",
        "white_",
        "yellow_",
    ] {
        out.push(("vanilla_1.21.40", format!("{colour}bundle")));
    }
    out
}

/// A base-game behaviour pack's context.
fn base_game() -> LoadContext {
    let mut cx = allowlisted();
    cx.is_base_game_pack = true;
    cx
}

/// `item` normalised for comparison: its components by name (the writer reorders them), its
/// description without unknown members (the shipped game drops them), no legacy reading (each
/// side read its own JSON) and no side-car (the newest format reads none).
fn normalised(item: &Item) -> (String, Vec<String>) {
    // the shorthand and object forms of an item descriptor are the same item
    let mut components: Vec<String> = item
        .components
        .iter()
        .map(|e| {
            format!("{e:?}")
                .replace("form: Name", "form: String")
                .replace("form: Item,", "form: String,")
        })
        .collect();
    components.sort();
    (format!("{:?}", item.description), components)
}

#[test]
fn the_full_chain_matches_the_retail_reserialisation() {
    let Some(dev) = corpus("row o01", "gdk-dev") else {
        return;
    };
    let Some(retail) = corpus("row o01", "gdk-retail") else {
        return;
    };
    let pairs = pairs();
    assert_eq!(pairs.len(), 22);
    for (pack, name) in pairs {
        let path = |root: &Path| {
            root.join("behavior_packs")
                .join(pack)
                .join("items")
                .join(format!("{name}.json"))
        };
        let before = std::fs::read_to_string(path(&dev)).unwrap();
        let after = std::fs::read_to_string(path(&retail)).unwrap();
        let old = loads(&before, &base_game());
        let new = loads(&after, &base_game());
        for d in [old.diagnostics(), new.diagnostics()] {
            assert!(above_info(d).is_empty(), "{pack}/{name}: {d:#?}");
        }
        let (a, b) = (normalised(old.value()), normalised(new.value()));
        let only = |x: &[String], y: &[String]| -> Vec<String> {
            x.iter().filter(|e| !y.contains(e)).cloned().collect()
        };
        assert_eq!(a.0, b.0, "{pack}/{name}: description");
        assert!(
            a.1 == b.1,
            "{pack}/{name}: before only {:#?}\nafter only {:#?}",
            only(&a.1, &b.1),
            only(&b.1, &a.1)
        );
        assert_eq!(
            new.description.menu_category.category,
            Some(CreativeCategory::Items),
            "{name}"
        );
        if let Some(modifiers) = new.components.get::<UseModifiers>() {
            assert_eq!(modifiers.start_using, StartUsing::Always, "{name}");
        }
        if name.ends_with("bundle") {
            // `max_stack_size {}` reads as 1 (inferred)
            assert_eq!(
                new.components.get::<MaxStackSize>().unwrap().value,
                1,
                "{name}"
            );
        }
    }
}

#[test]
fn the_new_1_26_30_items_load_and_upgrade_to_1_26_50() {
    let Some(retail) = corpus("row o02", "gdk-retail") else {
        return;
    };
    for name in ["red_shrub", "shelf_mushroom"] {
        let path = retail
            .join("behavior_packs/experimental_y_2026_drop_3/items")
            .join(format!("{name}.json"));
        let document = loads(&std::fs::read_to_string(path).unwrap(), &base_game());
        assert!(
            document.diagnostics().is_empty(),
            "{name}: {:#?}",
            document.diagnostics()
        );
        // the 1.26.30 struct, then the step to the newest format, 1.26.50
        assert!(document.version().upgraded, "{name}");
        assert_eq!(
            document.version().parsed_as,
            bedrock_addon_core::version::FormatVersion::new(1, 26, 30),
            "{name}"
        );
        assert_eq!(
            document.description.menu_category.category,
            Some(CreativeCategory::ItemCommandOnly),
            "{name}"
        );
        assert!(document.components.iter().all(|e| matches!(e, ComponentRef::Typed { value, .. } if !matches!(value, ItemComponent::Unknown(_)))));
    }
}
