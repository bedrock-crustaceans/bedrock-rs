//! The item atlas (`bedrock_addon::families::texture_atlas`): every fixture row.
//! The document rows run through `rp.texture_atlases().item_atlas`; the rows that check
//! a behaviour pack's `minecraft:icon` against the atlas wait for the link pass.

#[cfg(feature = "item")]
mod link;

use crate::common::{Rendered, finding, fixture_stack, pack_index, rendered};
use bedrock_addon::diagnostic::Severity::{Error, Info, Warning};
use bedrock_addon::families::texture_atlas::{
    AtlasKind, AtlasTextures, TextureAtlas, TextureElement,
};
use bedrock_addon::id::AtlasKey;
use bedrock_addon::pack::{PackStack, Resource};

const FAMILY: &str = "item_texture";

fn stack(name: &str) -> PackStack<Resource> {
    fixture_stack(FAMILY, name)
}

/// The item atlas of `stack`.
fn item(stack: &PackStack<Resource>) -> &TextureAtlas {
    stack.texture_atlases().item().expect("an item atlas")
}

/// The findings of the item atlas without the missing-image warnings (the fixtures ship no
/// image).
fn findings(stack: &PackStack<Resource>) -> Vec<Rendered> {
    rendered(
        stack
            .texture_atlases()
            .item_atlas
            .diagnostics()
            .filter(|finding| finding.code.name() != "TextureNotFound"),
    )
}

fn path(text: &str) -> TextureElement {
    TextureElement::Path(text.to_owned())
}

const NO_NAME: &str = "failed to read texture_name in atlas file textures/item_texture.json";

// The vanilla file is `resource_pack/textures/item_texture.json`,
// copied unchanged (its first line is a `//` comment).
#[test]
fn t01_the_vanilla_file_loads() {
    let stack = stack("t01_vanilla_shape");
    let atlas = item(&stack);
    assert_eq!(atlas.kind, AtlasKind::Item);
    assert_eq!(atlas.len(), 504);
    let frames = atlas.frame_counts();
    assert_eq!(frames.len(), 504);
    assert_eq!(frames["apple"], 1);
    assert_eq!(frames["axe"], 7);
    assert_eq!(findings(&stack), []);
    assert!(stack.texture_atlases().terrain().is_none());
}

#[test]
fn t02_a_higher_pack_adds_a_key() {
    let stack = stack("t02_merge_new_key");
    let atlas = item(&stack);
    assert_eq!(atlas.len(), 505);
    assert_eq!(
        atlas.get("demo_gem").unwrap().textures,
        AtlasTextures::One(path("textures/items/demo_gem"))
    );
    let origin = |key: &str| {
        let origin = stack
            .texture_atlases()
            .item_atlas
            .origin(&AtlasKey::parse(key));
        origin.unwrap().pack.index()
    };
    assert_eq!(origin("demo_gem"), Some(0));
    assert_eq!(origin("apple"), Some(1));
}

#[test]
fn t03_a_higher_pack_overrides_a_string_and_inherits_the_name() {
    let stack = stack("t03_merge_override_string");
    let atlas = item(&stack);
    assert_eq!(
        atlas.get("apple").unwrap().textures,
        AtlasTextures::One(path("textures/items/demo_apple"))
    );
    assert_eq!(atlas.texture_name, "atlas.items");
    assert_eq!(atlas.len(), 504);
    assert_eq!(findings(&stack), []);
}

#[test]
fn t04_a_shorter_array_keeps_the_lower_tail() {
    let stack = stack("t04_merge_array_tail");
    let lower = crate::common::fixture_stack(FAMILY, "t01_vanilla_shape");
    let vanilla = item(&lower).get("axe").unwrap().elements().to_vec();
    let merged = item(&stack).get("axe").unwrap().elements();
    assert_eq!(merged.len(), 7);
    assert_eq!(merged[..2], [path("a"), path("b")]);
    assert_eq!(merged[2..], vanilla[2..]);
}

#[test]
fn t05_a_string_replaces_the_array_whole() {
    let stack = stack("t05_merge_type_replace");
    let axe = item(&stack).get("axe").unwrap();
    assert_eq!(axe.frame_count(), 1);
    assert_eq!(
        axe.textures,
        AtlasTextures::One(path("textures/items/demo_axe"))
    );
}

#[test]
fn t06_a_null_entry_deletes_nothing() {
    let stack = stack("t06_merge_null_no_delete");
    let lower = crate::common::fixture_stack(FAMILY, "t01_vanilla_shape");
    assert_eq!(item(&stack).get("apple"), item(&lower).get("apple"));
    // The pack that wrote `null` wrote nothing: the tile is still the lower pack's.
    let origin = stack
        .texture_atlases()
        .item_atlas
        .origin(&AtlasKey::parse("apple"));
    assert_eq!(origin.unwrap().pack.index(), Some(1));
    assert_eq!(item(&stack).len(), 504);
    assert_eq!(findings(&stack), []);
}

#[test]
fn t07_object_elements_merge_deep() {
    let stack = stack("t07_merge_object_deep");
    let k = item(&stack).get("k").unwrap();
    let AtlasTextures::One(TextureElement::Layer(layer)) = &k.textures else {
        panic!("not a layer: {:?}", k.textures);
    };
    assert_eq!(layer.path.as_deref(), Some("x"));
    assert_eq!(layer.overlay_color.as_deref(), Some("#ff0000"));
    assert_eq!(layer.overlay_rgb(), Some([0xff, 0, 0]));
}

#[test]
fn t08_a_copy_that_is_no_json_is_skipped_deviation() {
    // `vanilla = "silent"` (`loadMetaFile`): ours names the pack
    // with an information.
    let stack = stack("t08_invalid_json_copy");
    assert_eq!(item(&stack).len(), 504);
    let found: Vec<_> = stack.texture_atlases().item_atlas.diagnostics().collect();
    let malformed: Vec<_> = found
        .iter()
        .filter(|finding| finding.code.name() == "MalformedInput")
        .collect();
    assert_eq!(malformed.len(), 1);
    assert_eq!(malformed[0].severity, Info);
    assert_eq!(pack_index(malformed[0]), Some(0));
    assert_eq!(findings(&stack).len(), 1);
}

#[test]
fn t09_without_texture_name_there_is_no_item_atlas() {
    let stack = stack("t09_missing_texture_name");
    assert!(stack.texture_atlases().item().is_none());
    assert!(stack.texture_atlases().item_atlas.frame_counts().is_empty());
    assert_eq!(
        rendered(stack.texture_atlases().diagnostics()),
        [finding("MissingRequiredField", Error, "", NO_NAME)]
    );
}

#[test]
fn t10_an_entry_that_is_no_object_is_dropped() {
    let stack = stack("t10_non_object_entry");
    assert!(item(&stack).get("a").is_none());
    assert_eq!(
        findings(&stack),
        [finding(
            "InvalidValue",
            Error,
            "texture_data/a",
            "Expecting \"a\" to be a JSON object."
        )]
    );
}

#[test]
fn t11_an_entry_without_textures_is_dropped() {
    let stack = stack("t11_missing_textures");
    assert!(item(&stack).frame_counts().is_empty());
    assert_eq!(
        findings(&stack),
        [finding(
            "MissingRequiredField",
            Error,
            "texture_data/a",
            "Missing textures \"a\"."
        )]
    );
}

#[test]
fn t12_weights_are_clamped() {
    let stack = stack("t12_weight_clamp");
    let a = item(&stack).get("a").unwrap();
    assert_eq!(a.layers(0)[0].weight.to_bits(), 0.05f32.to_bits());
    assert_eq!(a.layers(1)[0].weight.to_bits(), 1_000_000f32.to_bits());
    assert_eq!(
        findings(&stack),
        [
            finding(
                "ClampedValue",
                Warning,
                "texture_data/a/textures/0/weight",
                "Variations texture weight cannot be lower than 0.05"
            ),
            finding(
                "ClampedValue",
                Warning,
                "texture_data/a/textures/1/weight",
                "Variations texture weight cannot be higher than 1000000"
            ),
        ]
    );
}

#[test]
fn t13_another_texture_name_loads_with_a_warning_deviation() {
    // `vanilla = "accepted"`: the page is registered and nothing binds it.
    let stack = stack("t13_texture_name_other");
    assert_eq!(item(&stack).texture_name, "atlas.mine");
    assert_eq!(
        findings(&stack),
        [finding(
            "InvalidValue",
            Info,
            "texture_name",
            "invalid value"
        )]
    );
}

#[test]
fn the_expected_name_is_the_kind_s() {
    // `atlas.terrain` in the item file is the other atlas's name.
    let stack = crate::common::stack_of_files(&[&[(
        "textures/item_texture.json",
        br#"{ "texture_name": "atlas.terrain" }"#,
    )]]);
    assert_eq!(
        findings(&stack),
        [finding(
            "InvalidValue",
            Info,
            "texture_name",
            "invalid value"
        )]
    );
    let stack = crate::common::stack_of_files(&[&[(
        "textures/item_texture.json",
        br#"{ "texture_name": "atlas.items" }"#,
    )]]);
    assert_eq!(findings(&stack), []);
}

#[test]
fn t22_an_element_of_the_wrong_type_is_a_frame_without_a_layer_deviation() {
    let stack = stack("t22_element_wrong_type");
    let a = item(&stack).get("a").unwrap();
    assert_eq!(item(&stack).frame_counts()["a"], 2);
    assert_eq!(a.layers(0).len(), 1);
    assert!(a.layers(1).is_empty());
    assert_eq!(
        findings(&stack),
        [finding(
            "InvalidValue",
            Info,
            "texture_data/a/textures/1",
            "invalid value"
        )]
    );
}

#[test]
fn t23_the_members_vanilla_writes_and_never_reads_are_no_finding() {
    let stack = stack("t23_known_ignored_members");
    let atlas = item(&stack);
    assert!(atlas.ignored.get("format_version").is_some());
    assert!(atlas.ignored.get("resource_pack_name").is_some());
    assert!(atlas.extra.get("foo").is_some());
    assert_eq!(
        findings(&stack),
        [finding(
            "UnrecognizedField",
            Info,
            "foo",
            "unrecognized field"
        )]
    );
}

#[test]
fn t25_a_bad_colour_keeps_the_tile() {
    let stack = stack("t25_colour_error");
    let a = item(&stack).get("a").unwrap();
    assert_eq!(a.mip_fade_color, None);
    assert_eq!(
        findings(&stack),
        [finding(
            "InvalidValue",
            Error,
            "texture_data/a/mip_fade_color",
            "Expected [ r, g, b ] or [r, g, b, a] or \"#RRGGBB\"."
        )]
    );
}

#[test]
fn a_reference_that_misses_by_case_has_a_hint() {
    let stack = crate::common::stack_of_files(&[&[(
        "textures/item_texture.json",
        br#"{ "texture_name": "atlas.items", "texture_data": { "Demo_Gem": { "textures": "g" } } }"#,
    )]]);
    let atlas = item(&stack);
    assert!(atlas.get("demo_gem").is_none());
    assert_eq!(atlas.name_ignoring_case("demo_gem"), Some("Demo_Gem"));
    assert_eq!(atlas.name_ignoring_case("gem"), None);
}
