//! `minecraft:item` bodies.

use super::{Body, assert_unsupported, document};
use crate::storage::mcb::to_json_text;

const VERSION: (u16, u16, u32) = (1, 26, 30);

#[expect(
    clippy::needless_pass_by_value,
    reason = "a test hands over what it built"
)]
fn item(body: Body) -> Result<String, crate::storage::StorageError> {
    to_json_text(&document("minecraft:item", VERSION, &body.0))
}

/// The description: identifier, menu category, the two pinned bytes.
fn description(identifier: &str, category: &str) -> Body {
    Body::default()
        .string(identifier)
        .string(category)
        .raw(&[0, 0])
}

#[test]
fn an_item_decodes_component_by_component() {
    let body = description("demo:ruby", "items")
        .u32(14)
        .component("minecraft:block_placer")
        .string("demo:ruby_block")
        .raw(&[0, 1, 0])
        .component("minecraft:bundle_interaction")
        .i32(12)
        .component("minecraft:compostable")
        .raw(&[45])
        .component("minecraft:cooldown")
        .string("gem")
        .f32(0.25)
        .string("use")
        .component("minecraft:display_name")
        .string("item.ruby.name")
        .component("minecraft:hand_equipped")
        .raw(&[1])
        .component("minecraft:icon")
        .raw(&[2])
        .string("default")
        .string("ruby")
        .string("dyed")
        .string("ruby_dyed")
        .component("minecraft:max_stack_size")
        .raw(&[1, 0])
        .component("minecraft:projectile")
        .f32(0.0)
        .string("demo:thrown_ruby")
        .component("minecraft:storage_item")
        .i32(64)
        .raw(&[1, 1])
        .raw(&[1])
        .string("name")
        .string("demo:box")
        .raw(&[0])
        .component("minecraft:storage_weight_limit")
        .i32(64)
        .component("minecraft:storage_weight_modifier")
        .i32(4)
        .component("minecraft:tags")
        .raw(&[2])
        .string("demo:gem")
        .string("demo:red")
        .component("minecraft:throwable")
        .raw(&[1])
        .f32s(&[0.0, 0.0, 1.5, 1.5])
        .raw(&[0]);
    assert_eq!(
        item(body).unwrap(),
        concat!(
            r#"{"format_version":"1.26.30","minecraft:item":{"description":{"#,
            r#""identifier":"demo:ruby","menu_category":{"category":"items"}},"#,
            r#""components":{"#,
            r#""minecraft:block_placer":{"block":"demo:ruby_block","replace_block_item":true},"#,
            r#""minecraft:bundle_interaction":{},"#,
            r#""minecraft:compostable":{"composting_chance":45},"#,
            r#""minecraft:cooldown":{"category":"gem","duration":0.25},"#,
            r#""minecraft:display_name":{"value":"item.ruby.name"},"#,
            r#""minecraft:hand_equipped":{"value":true},"#,
            r#""minecraft:icon":{"textures":{"default":"ruby","dyed":"ruby_dyed"}},"#,
            r#""minecraft:max_stack_size":{"value":1},"#,
            r#""minecraft:projectile":{"projectile_entity":"demo:thrown_ruby"},"#,
            r#""minecraft:storage_item":{"banned_items":[{"name":"demo:box"}]},"#,
            r#""minecraft:storage_weight_limit":{},"#,
            r#""minecraft:storage_weight_modifier":{},"#,
            r#""minecraft:tags":{"tags":["demo:gem","demo:red"]},"#,
            r#""minecraft:throwable":{"do_swing_animation":true,"launch_power_scale":1.5,"#,
            r#""max_launch_power":1.5}}}}"#
        )
    );
}

#[test]
fn an_item_without_a_menu_category_has_the_category_none() {
    let body = description("demo:ruby", "none")
        .u32(1)
        .component("minecraft:display_name")
        .string("item.ruby.name");
    assert_eq!(
        item(body).unwrap(),
        concat!(
            r#"{"format_version":"1.26.30","minecraft:item":{"description":{"#,
            r#""identifier":"demo:ruby"},"components":{"#,
            r#""minecraft:display_name":{"value":"item.ruby.name"}}}}"#
        )
    );
}

#[test]
fn a_component_vanilla_does_not_bake_is_unsupported() {
    // nothing says where an unknown component ends
    let body = description("demo:ruby", "items")
        .u32(1)
        .component("minecraft:food")
        .i32(4);
    assert_unsupported(item(body));
}

#[test]
fn a_field_seen_at_one_value_is_read_at_that_value_only() {
    // `bundle_interaction` is 12 in every vanilla bundle, and the twins have `{}`
    let slots = |count: i32| {
        description("demo:bag", "items")
            .u32(1)
            .component("minecraft:bundle_interaction")
            .i32(count)
    };
    assert!(item(slots(12)).is_ok());
    assert_unsupported(item(slots(8)));
}

#[test]
fn a_component_twice_is_unsupported() {
    let body = description("demo:ruby", "items")
        .u32(2)
        .component("minecraft:display_name")
        .string("a")
        .component("minecraft:display_name")
        .string("b");
    assert_unsupported(item(body));
}

#[test]
fn a_component_count_past_the_body_is_unsupported() {
    let body = description("demo:ruby", "items")
        .u32(u32::MAX)
        .component("minecraft:display_name")
        .string("a");
    assert_unsupported(item(body));
}

/// The item bodies the game bakes at 1.26.50 (`red_shrub`, `shelf_mushroom` and the Drop 4
/// `ice_ball`) and at 1.26.40 (the Drop 4 `ice_crystal` and `icicle`): the layout of 1.26.30.
/// `red_shrub` and `shelf_mushroom` read as their published JSON twins read (members in name
/// order); `ice_ball` has no twin, and its stack size and its two launch powers are read as
/// the bytes give them.
mod baked_after_1_26_30 {
    use super::super::super::fixture;
    use crate::storage::mcb::to_json_text;

    fn decoded(name: &str) -> String {
        to_json_text(&fixture(&format!("mcb/{name}.json")))
            .unwrap_or_else(|error| panic!("{name}: {error:?}"))
    }

    #[test]
    fn the_1_26_50_vanilla_items_decode() {
        assert_eq!(
            decoded("red_shrub_1_26_50"),
            concat!(
                r#"{"format_version":"1.26.50","minecraft:item":{"description":{"#,
                r#""identifier":"minecraft:red_shrub"},"components":{"#,
                r#""minecraft:block_placer":{"block":"minecraft:red_shrub","replace_block_item":true},"#,
                r#""minecraft:compostable":{"composting_chance":30},"#,
                r#""minecraft:display_name":{"value":"tile.red_shrub.name"},"#,
                r#""minecraft:icon":{"textures":{"default":"red_shrub"}}}}}"#
            )
        );
        assert_eq!(
            decoded("shelf_mushroom_1_26_50"),
            concat!(
                r#"{"format_version":"1.26.50","minecraft:item":{"description":{"#,
                r#""identifier":"minecraft:shelf_mushroom"},"components":{"#,
                r#""minecraft:block_placer":{"block":"minecraft:shelf_mushroom","replace_block_item":true},"#,
                r#""minecraft:compostable":{"composting_chance":65},"#,
                r#""minecraft:display_name":{"value":"tile.shelf_mushroom.name"},"#,
                r#""minecraft:tags":{"tags":["minecraft:mushrooms_for_stew"]}}}}"#
            )
        );
        assert_eq!(
            decoded("ice_ball_1_26_50"),
            concat!(
                r#"{"format_version":"1.26.50","minecraft:item":{"description":{"#,
                r#""identifier":"minecraft:ice_ball"},"components":{"#,
                r#""minecraft:display_name":{"value":"item.ice_ball.name"},"#,
                r#""minecraft:icon":{"textures":{"default":"ice_ball"}},"#,
                r#""minecraft:max_stack_size":{"value":16},"#,
                r#""minecraft:projectile":{"projectile_entity":"ice_ball"},"#,
                r#""minecraft:throwable":{"do_swing_animation":true,"launch_power_scale":1.0,"#,
                r#""max_launch_power":1.0}}}}"#
            )
        );
    }

    #[test]
    fn the_1_26_40_vanilla_items_decode() {
        for (name, identifier, display) in [
            (
                "ice_crystal_1_26_40",
                "ice_crystal",
                "tile.ice_crystal.name",
            ),
            ("icicle_1_26_40", "icicle", "tile.icicle.name"),
        ] {
            assert_eq!(
                decoded(name),
                format!(
                    concat!(
                        r#"{{"format_version":"1.26.40","minecraft:item":{{"description":{{"#,
                        r#""identifier":"minecraft:{id}"}},"components":{{"#,
                        r#""minecraft:block_placer":{{"block":"minecraft:{id}","replace_block_item":true}},"#,
                        r#""minecraft:display_name":{{"value":"{display}"}},"#,
                        r#""minecraft:icon":{{"textures":{{"default":"{id}"}}}}}}}}}}"#
                    ),
                    id = identifier,
                    display = display
                )
            );
        }
    }
}

/// The items the game bakes after 1.26.30: each decodes, loads from a base-game pack as an MCB
/// source, and is written back with its payload unchanged. The three at 1.26.50 are parsed by the newest struct and not upgraded;
/// the two at 1.26.40 select the 1.26.30 struct and run the step, an identity on them; the
/// 1.26.30 `trial_key` runs it too.
#[cfg(feature = "item")]
mod baked_items_load {
    use bedrock_addon_core::document::Content;
    use bedrock_addon_core::load::{DocumentSource, LoadContext};
    use bedrock_addon_core::pack_identity::PackPath;
    use bedrock_addon_core::version::FormatVersion;

    use crate::families::item::Item;
    use crate::json::Value;
    use crate::storage::mcb::to_json_text;

    /// The JSON text of the MCB fixture `name` (`tests/fixtures/storage/mcb/`).
    fn decoded(name: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/storage/mcb")
            .join(format!("{name}.json"));
        #[allow(clippy::disallowed_methods, reason = "a test reads its fixture")]
        let bytes = std::fs::read(&path).unwrap();
        to_json_text(&bytes).unwrap()
    }

    fn base_game_mcb() -> LoadContext {
        let mut cx = LoadContext::detached();
        cx.is_base_game_pack = true;
        cx.source = DocumentSource::Mcb;
        cx
    }

    fn payload(text: &str) -> Value {
        let value: Value = crate::json::from_str(text).unwrap();
        value.as_object().unwrap()["minecraft:item"].clone()
    }

    #[test]
    fn the_baked_vanilla_items_load_and_write_back_their_payload() {
        for (name, parsed_as) in [
            ("red_shrub_1_26_50", FormatVersion::new(1, 26, 50)),
            ("shelf_mushroom_1_26_50", FormatVersion::new(1, 26, 50)),
            ("ice_ball_1_26_50", FormatVersion::new(1, 26, 50)),
            ("ice_crystal_1_26_40", FormatVersion::new(1, 26, 30)),
            ("icicle_1_26_40", FormatVersion::new(1, 26, 30)),
            ("trial_key", FormatVersion::new(1, 26, 30)),
        ] {
            let text = decoded(name);
            let document = Item::parse(
                &PackPath::from_static("items/x.json"),
                text.as_bytes(),
                &base_game_mcb(),
            )
            .unwrap_or_else(|rejected| panic!("{name}: {:#?}", rejected.diagnostics));
            assert!(
                document.diagnostics().is_empty(),
                "{name}: {:#?}",
                document.diagnostics()
            );
            assert_eq!(document.version().parsed_as, parsed_as, "{name}");
            assert_eq!(
                document.version().upgraded,
                parsed_as != FormatVersion::new(1, 26, 50),
                "{name}"
            );
            let written = document.value().to_json().unwrap();
            assert_eq!(payload(&written), payload(&text), "{name}: {written}");
            assert!(
                written.contains("\"format_version\": \"1.26.50\""),
                "{name}: {written}"
            );
        }
    }
}
