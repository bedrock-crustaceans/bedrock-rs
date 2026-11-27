//! 20 R32 with the texture atlas family off: the rule is not compiled and an icon is held to
//! nothing.

use bedrock_addon::load::LoadOptions;

use crate::common::{in_file, stacks};

#[test]
fn r32_without_the_texture_atlas_family_no_icon_is_checked() {
    const ITEM: &str = "items/thing.json";
    let item = r#"{ "format_version": "1.21.80", "minecraft:item": {
        "description": { "identifier": "demo:thing" },
        "components": { "minecraft:icon": { "textures": { "dyed": "nothing", "spare": "none" } },
            "minecraft:tags": { "tags": ["minecraft:trimmable_armors"] } } } }"#;
    let set = stacks(
        &[(ITEM, item)],
        &[(
            "textures/item_texture.json",
            r#"{ "texture_name": "atlas.items", "texture_data": {} }"#,
        )],
        LoadOptions::default(),
    );
    assert_eq!(set.behavior.items().iter().count(), 1);
    assert!(in_file(&set.link(), ITEM).is_empty());
}
