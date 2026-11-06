//! `minecraft:crafting_items_catalog` bodies at 1.21.60, 1.26.40 and 1.26.50 and at the other
//! versions the game bakes (1.21.70, 1.21.110, 1.26.10 and 1.26.30): the members in the
//! document's order (`categories`; per category `category_name` (an enum, baked as its name)
//! and `groups`; per group `group_identifier` (optional: a flag) and `items`; per group
//! identifier `name` and `icon` (optional); per item `name`, the struct of one string).

use std::path::PathBuf;

use facet_value::Value;

use super::{Body, assert_unsupported, document};
use crate::mcb::to_json_text;

const KEY: &str = "minecraft:crafting_items_catalog";

fn decode(version: (u16, u16, u32), body: &Body) -> Result<String, crate::StorageError> {
    to_json_text(&document(KEY, version, &body.0))
}

/// One category, `items`, with a named group (icon) and a loose group.
fn catalog() -> Body {
    Body::default()
        .raw(&[1])
        .string("items")
        .raw(&[2, 1])
        .string("demo:group")
        .raw(&[1])
        .string("demo:icon")
        .raw(&[1])
        .string("demo:icon")
        .raw(&[0, 2])
        .string("demo:a")
        .string("demo:b:1")
}

#[test]
fn a_catalogue_decodes_its_members_in_bind_order() {
    for version in [
        (1, 21, 60),
        (1, 21, 70),
        (1, 21, 110),
        (1, 26, 10),
        (1, 26, 30),
        (1, 26, 40),
        (1, 26, 50),
    ] {
        assert_eq!(
            decode(version, &catalog()).unwrap(),
            format!(
                concat!(
                    r#"{{"format_version":"{}.{}.{}","minecraft:crafting_items_catalog":{{"categories":["#,
                    r#"{{"category_name":"items","groups":["#,
                    r#"{{"group_identifier":{{"name":"demo:group","icon":"demo:icon"}},"items":["demo:icon"]}},"#,
                    r#"{{"items":["demo:a","demo:b:1"]}}]}}]}}}}"#
                ),
                version.0, version.1, version.2
            )
        );
    }
}

#[test]
fn a_catalogue_body_with_bytes_after_it_a_bad_flag_or_at_another_version_is_unsupported() {
    assert_unsupported(decode((1, 26, 50), &catalog().raw(&[0])));
    let bad_flag = Body::default().raw(&[1]).string("items").raw(&[1, 2]);
    assert_unsupported(decode((1, 26, 50), &bad_flag));
    // A version vanilla bakes no catalogue at, inside and outside the `v1_21_60` slice.
    assert_unsupported(decode((1, 26, 20), &catalog()));
    assert_unsupported(decode((1, 21, 50), &catalog()));
    assert_unsupported(decode((1, 26, 60), &catalog()));
}

/// `crates/addon/addon/tests/fixtures/item/catalog/<relative>` (the catalogue's fixtures).
fn catalog_fixture(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../addon/tests/fixtures/item/catalog")
        .join(relative)
}

fn bake(name: &str) -> Value {
    let bytes = std::fs::read(catalog_fixture(&format!(
        "mcb/{name}_crafting_item_catalog.mcb"
    )))
    .unwrap();
    let text = to_json_text(&bytes).unwrap_or_else(|e| panic!("{name}: {e:?}"));
    bedrock_addon_core::json::from_str(&text).unwrap()
}

#[test]
fn every_vanilla_bake_decodes_to_its_last_byte() {
    for (name, version) in [
        ("vanilla", "1.21.60"),
        ("vanilla_1.26.40", "1.26.40"),
        ("vanilla_1.26.50", "1.26.50"),
        ("experimental_y_2026_drop_4", "1.26.40"),
        // The bakes of the slices 1.21.70 to 1.26.30 carry `categories` only, over the same
        // element types as 1.26.50.
        ("vanilla_1.21.70", "1.21.70"),
        ("vanilla_1.21.80", "1.21.70"),
        ("vanilla_1.21.90", "1.21.70"),
        ("vanilla_1.21.100", "1.21.70"),
        ("vanilla_1.21.110", "1.21.110"),
        ("vanilla_1.21.120", "1.21.110"),
        ("vanilla_1.21.130", "1.21.110"),
        ("vanilla_1.26.10", "1.26.10"),
        ("vanilla_1.26.30", "1.26.30"),
    ] {
        let value = bake(name);
        let root = value.as_object().unwrap();
        assert_eq!(
            root.get("format_version")
                .and_then(Value::as_string)
                .map(facet_value::VString::as_str),
            Some(version),
            "{name}"
        );
    }
}

#[test]
fn the_1_26_40_bake_is_its_loose_twin() {
    let twin = std::fs::read_to_string(catalog_fixture("cat14_line_comments.json")).unwrap();
    let twin: Value = bedrock_addon_core::json::from_str_jsonc(&twin).unwrap();
    assert_eq!(bake("vanilla_1.26.40"), twin);
}

#[test]
fn the_1_26_50_bake_is_the_decoded_fixture() {
    let twin = std::fs::read_to_string(catalog_fixture(
        "vanilla_1_26_50/crafting_item_catalog.decoded.json",
    ))
    .unwrap();
    let twin: Value = bedrock_addon_core::json::from_str(&twin).unwrap();
    assert_eq!(bake("vanilla_1.26.50"), twin);
}
