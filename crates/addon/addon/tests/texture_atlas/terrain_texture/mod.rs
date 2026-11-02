//! The terrain atlas (`bedrock_addon::families::texture_atlas`): every fixture row
//! through the stack accessor `rp.texture_atlases()`, the merge of the copies, the typed
//! read with vanilla's coercions, the stack-stage image checks and the write side.

use crate::common::{Rendered, finding, fixture_file, fixture_stack, pack_index, rendered};
use bedrock_addon::diagnostic::Severity::{Error, Info, Warning};
use bedrock_addon::document::{Content, Placed};
use bedrock_addon::families::texture_atlas::{
    AtlasItem, AtlasKind, AtlasTextures, Color, Face, Isotropic, TextureAtlas, TextureElement,
    TextureLayer, Tint, VariationEntry,
};
use bedrock_addon::id::AtlasKey;
use bedrock_addon::load::LoadContext;
use bedrock_addon::pack::{PackStack, Resource};
use bedrock_addon::storage::PackPath;

const FAMILY: &str = "terrain_texture";

fn stack(name: &str) -> PackStack<Resource> {
    fixture_stack(FAMILY, name)
}

/// The terrain atlas of `stack`.
fn terrain(stack: &PackStack<Resource>) -> &TextureAtlas {
    stack.texture_atlases().terrain().expect("a terrain atlas")
}

/// The tile `x` of the terrain atlas of `stack`.
fn x(stack: &PackStack<Resource>) -> &AtlasItem {
    terrain(stack).get("x").expect("the tile x")
}

/// The findings of the terrain atlas without the missing-image warnings: the fixtures that are
/// not about images ship none.
fn findings(stack: &PackStack<Resource>) -> Vec<Rendered> {
    rendered(
        stack
            .texture_atlases()
            .terrain_atlas
            .diagnostics()
            .filter(|finding| finding.code.name() != "TextureNotFound"),
    )
}

/// The findings of the atlases of `stack`.
fn all(stack: &PackStack<Resource>) -> Vec<Rendered> {
    rendered(stack.texture_atlases().diagnostics())
}

/// The object layers of the elements of a tile.
fn object_layers(item: &AtlasItem) -> Vec<&TextureLayer> {
    item.elements()
        .iter()
        .map(|element| match element {
            TextureElement::Layer(layer) => &**layer,
            other => panic!("not an object layer: {other:?}"),
        })
        .collect()
}

fn path(text: &str) -> TextureElement {
    TextureElement::Path(text.to_owned())
}

// The vanilla file is `resource_pack/textures/terrain_texture.json`,
// copied unchanged (its first line is a `//` comment).
#[test]
fn t01_the_vanilla_file_loads() {
    let stack = stack("t01_vanilla_loads");
    let atlas = terrain(&stack);
    assert_eq!(atlas.len(), 1367);
    assert_eq!(atlas.padding(), 8);
    assert_eq!(atlas.num_mip_levels(), 4);
    assert_eq!(atlas.texture_name, "atlas.terrain");
    assert_eq!(atlas.kind, AtlasKind::Terrain);
    assert!(atlas.ignored.get("resource_pack_name").is_some());
    // The fixture ships no image, so every path is missing; nothing else is found.
    assert_eq!(findings(&stack), []);
    let missing = stack.texture_atlases().terrain_atlas.diagnostics().count();
    assert!(missing > 1367, "one warning per image path: {missing}");
}

#[test]
fn t02_a_string_replaces_an_array() {
    let stack = stack("t02_merge_override_string");
    let tile = terrain(&stack).get("grass_top").unwrap();
    assert_eq!(tile.textures, AtlasTextures::One(path("mine")));
    assert_eq!(findings(&stack), []);
    // The tile belongs to the highest pack that wrote it.
    let origin = stack
        .texture_atlases()
        .terrain_atlas
        .origin(&AtlasKey::parse("grass_top"));
    assert_eq!(origin.unwrap().pack.index(), Some(0));
    assert_eq!(stack.texture_atlases().terrain_atlas.layers().len(), 2);
    assert_eq!(
        stack.texture_atlases().terrain_atlas.layers()[0]
            .origin
            .pack
            .index(),
        Some(0)
    );
}

#[test]
fn t03_arrays_merge_by_index_and_keep_the_lower_tail() {
    let stack = stack("t03_merge_array_by_index");
    let tile = terrain(&stack).get("grass_top").unwrap();
    assert_eq!(
        tile.textures,
        AtlasTextures::Many(vec![path("mine"), path("b"), path("c")])
    );
    assert_eq!(findings(&stack), []);
}

#[test]
fn t04_an_override_of_textures_keeps_the_tile_s_other_members() {
    let stack = stack("t04_merge_keeps_node_fields");
    let tile = terrain(&stack).get("flowing_water").unwrap();
    // Vanilla writes `"quad": 1`.
    assert_eq!(tile.quad, Some(true));
    assert_eq!(tile.textures, AtlasTextures::One(path("mine")));
    assert_eq!(findings(&stack), []);
}

#[test]
fn t05_a_null_cannot_delete() {
    let stack = stack("t05_merge_null_cannot_delete");
    assert_eq!(x(&stack).textures, AtlasTextures::One(path("a")));
    assert_eq!(findings(&stack), []);
    // The pack that wrote `null` wrote nothing.
    let origin = stack
        .texture_atlases()
        .terrain_atlas
        .origin(&AtlasKey::parse("x"));
    assert_eq!(origin.unwrap().pack.index(), Some(0));
}

#[test]
fn t06_array_elements_are_not_merged() {
    let stack = stack("t06_merge_array_elements_not_merged");
    let layers = object_layers(x(&stack));
    assert_eq!(layers.len(), 1);
    assert_eq!(layers[0].path.as_deref(), Some("mine"));
    assert_eq!(layers[0].overlay_color, None);
    assert_eq!(findings(&stack), []);
}

#[test]
fn t07_objects_merge_deep() {
    let stack = stack("t07_merge_objects_deep");
    let layers = object_layers(x(&stack));
    assert_eq!(layers[0].path.as_deref(), Some("mine"));
    assert_eq!(
        layers[0].tint_color,
        Some(Tint::Color(Color::Hex(0x00ff_0000)))
    );
    assert_eq!(findings(&stack), []);
}

#[test]
fn t08_the_top_level_values_are_the_highest_pack_s() {
    let stack = stack("t08_top_level_from_highest");
    let atlas = terrain(&stack);
    assert_eq!(atlas.padding, Some(8));
    assert_eq!(atlas.num_mip_levels, Some(0));
    assert_eq!(atlas.effective_num_mip_levels(), 1);
    assert_eq!(atlas.texture_name, "atlas.terrain");
    // 0 and 1 are no finding.
    assert_eq!(findings(&stack), []);
}

/// `TextureAtlas::loadMetaFile` skips a copy `Json::Reader::parse` refuses without a line: the finding is ours, an information.
#[test]
fn t09_a_copy_that_is_no_json_is_skipped() {
    let stack = stack("t09_bad_json_pack_skipped");
    assert_eq!(x(&stack).textures, AtlasTextures::One(path("a")));
    let atlas = &stack.texture_atlases().terrain_atlas;
    let found: Vec<_> = atlas
        .diagnostics()
        .filter(|finding| finding.code.name() == "MalformedInput")
        .collect();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].severity, Info);
    assert_eq!(pack_index(found[0]), Some(0));
    assert_eq!(
        found[0].location.path.as_ref().unwrap().as_str(),
        "textures/terrain_texture.json"
    );
    // Only the copy that parsed is a layer.
    assert_eq!(atlas.layers().len(), 1);
    assert_eq!(atlas.layers()[0].origin.pack.index(), Some(1));
}

#[test]
fn t10_without_texture_name_no_atlas_is_built() {
    let stack = stack("t10_missing_texture_name");
    assert!(stack.texture_atlases().terrain().is_none());
    assert!(stack.texture_atlases().terrain_atlas.is_empty());
    assert_eq!(
        all(&stack),
        [finding(
            "MissingRequiredField",
            Error,
            "",
            "failed to read texture_name in atlas file textures/terrain_texture.json"
        )]
    );
}

#[test]
fn t11_an_entry_that_is_no_object_is_dropped() {
    let stack = stack("t11_entry_not_object");
    assert!(terrain(&stack).get("x").is_none());
    assert_eq!(
        findings(&stack),
        [finding(
            "InvalidValue",
            Error,
            "texture_data/x",
            "Expecting \"x\" to be a JSON object."
        )]
    );
}

#[test]
fn t12_an_entry_without_textures_is_dropped() {
    let stack = stack("t12_missing_textures");
    assert!(terrain(&stack).get("x").is_none());
    assert_eq!(
        findings(&stack),
        [finding(
            "MissingRequiredField",
            Error,
            "texture_data/x",
            "Missing textures \"x\"."
        )]
    );
}

#[test]
fn t13_the_four_kinds_of_element() {
    let stack = stack("t13_element_kinds");
    let tile = x(&stack);
    let elements = tile.elements();
    assert_eq!(tile.frame_count(), 4);
    assert_eq!(elements[0], path("p"));
    assert!(
        matches!(&elements[1], TextureElement::Layer(layer) if layer.path.as_deref() == Some("q"))
    );
    let TextureElement::Variations { variations, .. } = &elements[2] else {
        panic!("not variations: {:?}", elements[2]);
    };
    assert_eq!(variations.len(), 2);
    assert_eq!(variations[0], VariationEntry::Path("r".to_owned()));
    assert!(matches!(&variations[1], VariationEntry::Layer(layer) if layer.weight == Some(3.0)));
    assert!(matches!(&elements[3], TextureElement::Invalid(value) if value.is_number()));
    // The fourth index has no layer.
    assert_eq!(tile.layers(2).len(), 2);
    assert!(tile.layers(3).is_empty());
    assert_eq!(
        findings(&stack),
        [finding(
            "InvalidValue",
            Info,
            "texture_data/x/textures/3",
            "invalid value"
        )]
    );
}

#[test]
fn t14_variations_that_is_no_array_is_ignored() {
    let stack = stack("t14_variations_not_array");
    let layers = object_layers(x(&stack));
    assert_eq!(layers.len(), 1);
    assert_eq!(layers[0].path.as_deref(), Some("p"));
    assert_eq!(
        findings(&stack),
        [finding(
            "UnrecognizedField",
            Info,
            "texture_data/x/textures/variations",
            "unrecognized field"
        )]
    );
    // Step 6: the element still goes through the variations filter, which leaves its one
    // layer with weight 1.
    let kept = stack.texture_atlases().effective_variations(x(&stack), 0);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].weight.to_bits(), 1.0f32.to_bits());
}

#[test]
fn t15_weights_are_clamped() {
    let stack = stack("t15_weight_clamp");
    let weights: Vec<f32> = object_layers(x(&stack))
        .iter()
        .map(|l| l.weight())
        .collect();
    assert_eq!(weights, [0.05, 0.05, 0.05, 1_000_000.0, 70.0]);
    let low = "Variations texture weight cannot be lower than 0.05";
    let high = "Variations texture weight cannot be higher than 1000000";
    let at = |index: usize| format!("texture_data/x/textures/{index}/weight");
    assert_eq!(
        findings(&stack),
        [
            finding("ClampedValue", Warning, &at(0), low),
            finding("ClampedValue", Warning, &at(1), low),
            finding("ClampedValue", Warning, &at(2), low),
            finding("ClampedValue", Warning, &at(3), high),
        ]
    );
}

/// The plain tints of the object layers of the tile `x`.
fn tints(stack: &PackStack<Resource>) -> Vec<Option<Color>> {
    object_layers(x(stack))
        .iter()
        .map(|layer| match &layer.tint_color {
            Some(Tint::Color(color)) => Some(*color),
            None => None,
            Some(Tint::Map(map)) => panic!("a tint map: {map:?}"),
        })
        .collect()
}

const BAD_COLOR: &str = "Expected [ r, g, b ] or [r, g, b, a] or \"#RRGGBB\".";
const NOT_INTEGRAL: &str = "Expected Integral Number.";

#[test]
fn t16_colours() {
    let stack = stack("t16_colours");
    let tints = tints(&stack);
    // A hex string of six digits has alpha 0: the high byte is the alpha.
    assert_eq!(tints[0], Some(Color::Hex(0x00ff_8000)));
    assert_eq!(tints[0].unwrap().bytes(), [0xff, 0x80, 0x00, 0x00]);
    assert_eq!(tints[1], Some(Color::Rgb([255, 128, 0])));
    assert_eq!(tints[1].unwrap().bytes(), [255, 128, 0, 255]);
    assert_eq!(tints[2], Some(Color::Rgba([255, 128, 0, 64])));
    // A component that is no integer is 255.
    assert_eq!(tints[3], Some(Color::Rgb([255, 0, 0])));
    // `"red"`: the first character is dropped and `ed` is hex.
    assert_eq!(tints[4], Some(Color::Hex(0xed)));
    assert_eq!(tints[4].unwrap().bytes(), [0, 0, 0xed, 0]);
    // `{}` is no colour: the tint stays white.
    assert_eq!(tints[5], None);
    assert_eq!(x(&stack).layers(5)[0].tint_color, [1.0; 4]);
    assert_eq!(
        x(&stack).layers(0)[0].tint_color,
        [1.0, 128.0 / 255.0, 0.0, 0.0]
    );
    assert_eq!(
        findings(&stack),
        [
            finding(
                "InvalidValue",
                Error,
                "texture_data/x/textures/3/tint_color/0",
                NOT_INTEGRAL
            ),
            finding(
                "InvalidValue",
                Error,
                "texture_data/x/textures/5/tint_color",
                BAD_COLOR
            ),
        ]
    );
}

#[test]
fn t17_tint_maps() {
    let stack = stack("t17_tint_map_object");
    let layer = object_layers(x(&stack))[0];
    let Some(Tint::Map(map)) = &layer.tint_color else {
        panic!("no tint map: {:?}", layer.tint_color);
    };
    assert_eq!(map.r_color, Some(Color::Hex(0x00ff_0000)));
    assert_eq!(map.g_color, Some(Color::Rgb([0, 255, 0])));
    assert_eq!((map.b_color, map.a_color), (None, None));
    let base = layer.tint_base_color.unwrap();
    assert_eq!(base.a_color.unwrap().bytes(), [0xff, 0xff, 0xff, 0x80]);
    assert_eq!(findings(&stack), []);
}

#[test]
fn t18_overlay_colours() {
    let stack = stack("t18_overlay_color");
    let overlays: Vec<_> = object_layers(x(&stack))
        .iter()
        .map(|l| l.overlay_rgb())
        .collect();
    // The first character is dropped, whatever it is; an empty string sets nothing.
    assert_eq!(
        overlays,
        [Some([0xdf, 0x68, 0x27]), Some([0xdf, 0x68, 0x27]), None]
    );
    assert_eq!(findings(&stack), []);
}

#[test]
fn t19_isotropic() {
    let stack = stack("t19_isotropic");
    let layers = object_layers(x(&stack));
    assert_eq!(layers[0].isotropic, Some(Isotropic::All(true)));
    assert_eq!(layers[0].isotropic_mask(), 0xFF);
    let faces = |pairs: &[(Face, bool)]| Some(Isotropic::Faces(pairs.iter().copied().collect()));
    assert_eq!(
        layers[1].isotropic,
        faces(&[(Face::Up, true), (Face::Side, false)])
    );
    assert_eq!(layers[1].isotropic_mask(), 2);
    assert_eq!(layers[2].isotropic, faces(&[(Face::North, true)]));
    assert_eq!(layers[2].isotropic_mask(), 4);
    assert_eq!(
        findings(&stack),
        [finding(
            "UnrecognizedField",
            Info,
            "texture_data/x/textures/2/isotropic/bogus",
            "unrecognized field"
        )]
    );
}

#[test]
fn t20_quad_inside_an_element_is_not_read() {
    let stack = stack("t20_quad_in_element");
    assert!(!x(&stack).quad());
    assert_eq!(x(&stack).quad, None);
    assert_eq!(
        findings(&stack),
        [finding(
            "UnrecognizedField",
            Info,
            "texture_data/x/textures/0/quad",
            "unrecognized field"
        )]
    );
}

#[test]
fn t21_variations_keep_the_images_of_the_highest_pack_deviation() {
    // p0 has `v1`; p1 has `v1` and `v2`.
    let stack = stack("t21_variations_pack_filter");
    let tile = x(&stack);
    // The typed model keeps both (vanilla drops at load).
    assert_eq!(tile.layers(0).len(), 2);
    let kept = stack.texture_atlases().effective_variations(tile, 0);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].path, Some("textures/v1"));
    assert_eq!(kept[0].weight.to_bits(), 1.0f32.to_bits());
    assert_eq!(
        all(&stack),
        [finding(
            "DroppedField",
            Info,
            "texture_data/x/textures/variations/1",
            "field dropped"
        )]
    );
}

#[test]
fn t22_one_missing_variation_drops_every_found_one() {
    let stack = stack("t22_variations_one_missing");
    let kept = stack.texture_atlases().effective_variations(x(&stack), 0);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].path, Some("textures/b"));
    assert_eq!(kept[0].weight.to_bits(), 1.0f32.to_bits());
    assert_eq!(
        all(&stack),
        [
            finding(
                "DroppedField",
                Info,
                "texture_data/x/textures/variations/0",
                "field dropped"
            ),
            finding(
                "TextureNotFound",
                Info,
                "texture_data/x/textures/variations/1",
                "texture not found"
            ),
        ]
    );
}

#[test]
fn t23_another_texture_name_loads_with_a_warning() {
    let stack = stack("t23_texture_name_other");
    assert_eq!(terrain(&stack).texture_name, "atlas.items");
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
fn t24_format_version_and_resource_pack_name_are_ignored() {
    let stack = stack("t24_format_version_ignored");
    let atlas = terrain(&stack);
    assert!(atlas.ignored.get("format_version").is_some());
    assert!(atlas.ignored.get("resource_pack_name").is_some());
    assert!(atlas.extra.is_empty());
    assert_eq!(all(&stack), []);
}

#[test]
fn t25_leather_colours() {
    let stack = stack("t25_leather_colors");
    let atlas = terrain(&stack);
    assert_eq!(atlas.default_leather_color.as_deref(), Some("#A06540"));
    assert_eq!(atlas.default_leather_color_rgb(), Some(0x00A0_6540));
    assert_eq!(atlas.default_horse_leather_color_rgb(), None);
    assert_eq!(findings(&stack), []);
}

#[test]
fn t26_vanilla_reads_the_tiles_sorted_and_the_document_order_is_kept() {
    let stack = stack("t26_sorted_node_order");
    let atlas = terrain(&stack);
    let names = |tiles: Vec<&str>| tiles.join(",");
    assert_eq!(
        names(atlas.iter().map(|(key, _)| key.as_str()).collect()),
        "b,a,c"
    );
    assert_eq!(
        names(
            atlas
                .vanilla_order()
                .iter()
                .map(|(key, _)| key.as_str())
                .collect()
        ),
        "a,b,c"
    );
    assert_eq!(findings(&stack), []);
}

#[test]
fn t27_content_after_the_root_is_ignored() {
    let stack = stack("t27_trailing_content_ignored");
    assert_eq!(x(&stack).textures, AtlasTextures::One(path("mine")));
    let atlas = &stack.texture_atlases().terrain_atlas;
    let found: Vec<_> = atlas
        .diagnostics()
        .filter(|finding| finding.code.name() != "TextureNotFound")
        .collect();
    assert_eq!(found.len(), 1);
    assert_eq!(
        (found[0].code.name(), found[0].severity),
        ("MalformedInput", Info)
    );
    assert_eq!(pack_index(found[0]), Some(0));
    assert_eq!(atlas.layers().len(), 2);
}

#[test]
fn t28_a_root_that_is_no_object_wipes_the_lower_packs() {
    let stack = stack("t28_non_object_root_wipes");
    assert!(stack.texture_atlases().terrain().is_none());
    let atlas = &stack.texture_atlases().terrain_atlas;
    assert_eq!(
        rendered(atlas.diagnostics()),
        [
            finding("InvalidValue", Info, "", "invalid value"),
            finding(
                "MissingRequiredField",
                Error,
                "",
                "failed to read texture_name in atlas file textures/terrain_texture.json"
            ),
        ]
    );
    assert_eq!(pack_index(atlas.diagnostics().next().unwrap()), Some(0));
}

#[test]
fn t29_a_null_root_contributes_nothing() {
    let stack = stack("t29_null_root_ignored");
    assert_eq!(x(&stack).textures, AtlasTextures::One(path("a")));
    assert_eq!(findings(&stack), []);
}

#[test]
fn t30_colour_edge_cases_deviation() {
    let stack = stack("t30_colour_edge_cases");
    let tints = tints(&stack);
    // `""`: vanilla throws from `substr(1)`; ours refuses the colour and the tint stays white.
    assert_eq!(tints[0], None);
    // `255.0` is a real, never integral.
    assert_eq!(tints[1], Some(Color::Rgb([255, 0, 0])));
    // A bool counts as 1 or 0.
    assert_eq!(tints[2], Some(Color::Rgba([1, 0, 0, 0])));
    assert_eq!(tints[3], Some(Color::Hex(0x01ff_8000)));
    assert_eq!(tints[3].unwrap().bytes(), [0xff, 0x80, 0x00, 0x01]);
    assert_eq!(
        findings(&stack),
        [
            finding(
                "InvalidValue",
                Error,
                "texture_data/x/textures/0/tint_color",
                BAD_COLOR
            ),
            finding(
                "InvalidValue",
                Error,
                "texture_data/x/textures/1/tint_color/0",
                NOT_INTEGRAL
            ),
        ]
    );
}

#[test]
fn t31_a_variation_without_a_path_drops_every_found_one() {
    let stack = stack("t31_variations_pathless_entry");
    let tile = x(&stack);
    assert_eq!(tile.layers(0).len(), 2);
    let kept = stack.texture_atlases().effective_variations(tile, 0);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].path, None);
    assert_eq!(kept[0].weight.to_bits(), 1.0f32.to_bits());
    assert_eq!(
        all(&stack),
        [finding(
            "DroppedField",
            Info,
            "texture_data/x/textures/variations/0",
            "field dropped"
        )]
    );
}

#[test]
fn t32_a_quad_string_is_true() {
    let stack = stack("t32_quad_string");
    assert!(x(&stack).quad());
    assert_eq!(
        findings(&stack),
        [finding(
            "InvalidValue",
            Info,
            "texture_data/x/quad",
            "invalid value"
        )]
    );
}

#[test]
fn t33_an_empty_textures_array_keeps_the_entry() {
    let stack = stack("t33_empty_textures_array");
    assert_eq!(x(&stack).textures, AtlasTextures::Many(Vec::new()));
    assert_eq!(x(&stack).frame_count(), 0);
    assert_eq!(
        all(&stack),
        [finding(
            "InvalidValue",
            Info,
            "texture_data/x/textures",
            "invalid value"
        )]
    );
}

#[test]
fn t34_side_selects_the_faces_that_are_read() {
    let stack = stack("t34_isotropic_side_precedence");
    let masks: Vec<u8> = object_layers(x(&stack))
        .iter()
        .map(|l| l.isotropic_mask())
        .collect();
    // `side` is not null, so `north` is not read, and `side` itself is no bool: `up` only.
    // Then `east` only (`north` is no bool), then nothing for a number.
    assert_eq!(masks, [2, 32, 0]);
    let at = |rest: &str| format!("texture_data/x/textures/{rest}");
    let invalid = |rest: &str| finding("InvalidValue", Info, &at(rest), "invalid value");
    assert_eq!(
        findings(&stack),
        [
            invalid("0/isotropic/side"),
            invalid("0/isotropic/north"),
            invalid("1/isotropic/north"),
            invalid("2/isotropic"),
        ]
    );
}

#[test]
fn t35_an_invalid_variation_does_not_get_the_tile_s_values() {
    let stack = stack("t35_invalid_variation_no_node_values");
    let layers = x(&stack).layers(0);
    assert_eq!(layers.len(), 2);
    assert_eq!((layers[0].path, layers[0].additive), (None, false));
    assert_eq!(layers[0].mip_fade_amount.to_bits(), 0.0f32.to_bits());
    assert_eq!(layers[0].weight.to_bits(), 1.0f32.to_bits());
    assert_eq!((layers[1].path, layers[1].additive), (Some("v1"), true));
    assert_eq!(layers[1].mip_fade_amount.to_bits(), 0.5f32.to_bits());
    assert_eq!(
        findings(&stack),
        [finding(
            "InvalidValue",
            Info,
            "texture_data/x/textures/variations/0",
            "invalid value"
        )]
    );
}

// `resource_pack/textures/item_texture.json`, copied unchanged.
#[test]
fn i01_the_item_atlas_shares_the_loader() {
    let stack = stack("i01_item_atlas_shares_loader");
    assert!(stack.texture_atlases().terrain().is_none());
    let atlas = stack.texture_atlases().item().expect("an item atlas");
    assert_eq!(atlas.kind, AtlasKind::Item);
    assert_eq!(atlas.len(), 504);
    assert_eq!(atlas.texture_name, "atlas.items");
    assert_eq!((atlas.padding, atlas.num_mip_levels), (None, None));
    assert_eq!((atlas.padding(), atlas.num_mip_levels()), (0, 1));
}

/// Parses `bytes` as the atlas file at `path`.
fn parse(path: &str, bytes: &[u8]) -> TextureAtlas {
    let path = PackPath::new(path).unwrap();
    TextureAtlas::parse(&path, bytes, &LoadContext::detached())
        .unwrap_or_else(|rejected| panic!("{:?}", rejected.diagnostics))
        .into_value()
}

#[test]
fn an_atlas_is_written_back_to_the_same_atlas() {
    for (fixture, file) in [
        ("t01_vanilla_loads", "textures/terrain_texture.json"),
        ("i01_item_atlas_shares_loader", "textures/item_texture.json"),
        ("t13_element_kinds", "textures/terrain_texture.json"),
        ("t17_tint_map_object", "textures/terrain_texture.json"),
        ("t19_isotropic", "textures/terrain_texture.json"),
        ("t16_colours", "textures/terrain_texture.json"),
        (
            "t24_format_version_ignored",
            "textures/terrain_texture.json",
        ),
    ] {
        let first = parse(file, &fixture_file(FAMILY, fixture, &format!("p0/{file}")));
        let written = first.to_json().unwrap();
        let second = parse(file, written.as_bytes());
        assert_eq!(first, second, "{fixture}");
        assert_eq!(
            second.to_json().unwrap(),
            written,
            "{fixture}: the rewrite differs"
        );
    }
}

#[test]
fn a_member_the_document_does_not_write_is_not_written() {
    let atlas = parse(
        "textures/terrain_texture.json",
        br#"{ "texture_name": "atlas.terrain", "texture_data": { "x": { "textures": { "path": "p" } } } }"#,
    );
    assert_eq!(
        atlas.to_json().unwrap(),
        "{\n  \"texture_name\": \"atlas.terrain\",\n  \"texture_data\": {\n    \"x\": {\n      \"textures\": {\n        \"path\": \"p\"\n      }\n    }\n  }\n}\n"
    );
    let bare = parse(
        "textures/terrain_texture.json",
        br#"{ "texture_name": "n" }"#,
    );
    assert_eq!(bare.to_json().unwrap(), "{\n  \"texture_name\": \"n\"\n}\n");
}

#[test]
fn an_atlas_is_placed_by_its_kind() {
    let terrain = parse(
        "textures/terrain_texture.json",
        br#"{ "texture_name": "atlas.terrain" }"#,
    );
    assert_eq!(
        terrain.default_path().as_str(),
        "textures/terrain_texture.json"
    );
    let item = parse(
        "textures/item_texture.json",
        br#"{ "texture_name": "atlas.items" }"#,
    );
    assert_eq!(item.kind, AtlasKind::Item);
    assert_eq!(item.default_path().as_str(), "textures/item_texture.json");
}

#[test]
fn a_file_without_texture_name_is_rejected_on_its_own() {
    let path = PackPath::new("textures/terrain_texture.json").unwrap();
    let rejected = TextureAtlas::parse(&path, b"{}", &LoadContext::detached()).unwrap_err();
    assert_eq!(
        rejected.diagnostics.iter().next().unwrap().code.name(),
        "MissingRequiredField"
    );
    let rejected = TextureAtlas::parse(&path, b"{", &LoadContext::detached()).unwrap_err();
    assert_eq!(
        rejected.diagnostics.iter().next().unwrap().code.name(),
        "MalformedInput"
    );
}

/// The stack of one pack whose terrain atlas file is `text`.
fn inline(text: &str) -> PackStack<Resource> {
    crate::common::stack_of_files(&[&[("textures/terrain_texture.json", text.as_bytes())]])
}

#[test]
fn the_ends_of_the_weight_range_are_inside_it() {
    let stack = inline(
        r#"{ "texture_name": "atlas.terrain", "texture_data": { "x": { "textures": [
            { "weight": 0.05 }, { "weight": 1000000 }, { "weight": 1 }, {} ] } } }"#,
    );
    let weights: Vec<Option<f32>> = object_layers(x(&stack)).iter().map(|l| l.weight).collect();
    assert_eq!(weights, [Some(0.05), Some(1_000_000.0), Some(1.0), None]);
    assert_eq!(
        object_layers(x(&stack))[3].weight().to_bits(),
        1.0f32.to_bits()
    );
    assert_eq!(findings(&stack), []);
}

#[test]
fn rule_3_an_empty_texture_name_builds_the_atlas_with_the_assert_s_text() {
    let stack = inline(r#"{ "texture_name": "" }"#);
    assert_eq!(terrain(&stack).texture_name, "");
    assert_eq!(
        findings(&stack),
        [finding(
            "InvalidValue",
            Info,
            "texture_name",
            "Atlas textures must contain at least one defined texture page"
        )]
    );
    // A `null` member counts as present, and reads as the empty name.
    let stack = inline(r#"{ "texture_name": null }"#);
    assert!(stack.texture_atlases().terrain().is_some());
}

#[test]
fn rules_17_and_18_mip_levels_and_padding_are_read_as_vanilla_reads_them_deviation() {
    // `vanilla = "silent"`: a negative count is passed on, a string reads as 0.
    let stack =
        inline(r#"{ "texture_name": "atlas.terrain", "num_mip_levels": -1, "padding": "8" }"#);
    let atlas = terrain(&stack);
    assert_eq!(atlas.num_mip_levels, Some(-1));
    assert_eq!(atlas.effective_num_mip_levels(), -1);
    assert_eq!(atlas.padding, Some(0));
    let invalid = |at: &str| finding("InvalidValue", Info, at, "invalid value");
    assert_eq!(
        findings(&stack),
        [invalid("padding"), invalid("num_mip_levels")]
    );
    // 1 and 2 are as written, without a finding; a real is truncated, with one.
    let stack = inline(r#"{ "texture_name": "atlas.terrain", "num_mip_levels": 1, "padding": 2 }"#);
    assert_eq!(terrain(&stack).effective_num_mip_levels(), 1);
    assert_eq!(findings(&stack), []);
    let stack =
        inline(r#"{ "texture_name": "atlas.terrain", "num_mip_levels": 2, "padding": 2.9 }"#);
    assert_eq!(terrain(&stack).effective_num_mip_levels(), 2);
    assert_eq!(terrain(&stack).padding, Some(2));
    assert_eq!(findings(&stack), [invalid("padding")]);
}

#[test]
fn rule_19_texture_data_that_is_no_object_gives_no_tile_deviation() {
    // `vanilla = "silent"`.
    let stack = inline(r#"{ "texture_name": "atlas.terrain", "texture_data": [1] }"#);
    assert_eq!(terrain(&stack).texture_data, None);
    assert_eq!(
        findings(&stack),
        [finding(
            "InvalidValue",
            Info,
            "texture_data",
            "invalid value"
        )]
    );
    // An empty object is kept as written, without a finding.
    let stack = inline(r#"{ "texture_name": "atlas.terrain", "texture_data": {} }"#);
    let data = terrain(&stack).texture_data.as_ref();
    assert!(data.is_some_and(|data| data.is_empty()));
    assert_eq!(findings(&stack), []);
}

#[test]
fn a_tint_base_color_that_is_no_object_is_ignored() {
    let stack = inline(
        r##"{ "texture_name": "atlas.terrain", "texture_data": { "x": { "textures": [
            { "tint_base_color": "#ff0000" }, { "tint_base_color": {} },
            { "tint_color": { "x_color": 1 } } ] } } }"##,
    );
    let layers = object_layers(x(&stack));
    assert_eq!(layers[0].tint_base_color, None);
    assert_eq!(layers[1].tint_base_color, None);
    assert!(matches!(layers[2].tint_color, Some(Tint::Map(_))));
    assert_eq!(
        findings(&stack),
        [
            finding(
                "InvalidValue",
                Info,
                "texture_data/x/textures/0/tint_base_color",
                "invalid value"
            ),
            finding(
                "UnrecognizedField",
                Info,
                "texture_data/x/textures/2/tint_color/x_color",
                "unrecognized field"
            ),
        ]
    );
}

#[test]
fn without_an_active_subpack_the_subpack_s_copy_is_not_read() {
    // the walk reads the active subpack's file at the pack's path, and no file of a
    // subpack that is not active.
    let files: &[(&str, &[u8])] = &[
        (
            "textures/terrain_texture.json",
            br#"{ "texture_name": "atlas.terrain" }"#,
        ),
        (
            "subpacks/hi/textures/terrain_texture.json",
            br#"{ "texture_name": "atlas.other" }"#,
        ),
    ];
    let origin = bedrock_addon::pack::PackOrigin::User;
    let pack = crate::common::pack_with(1, "resources", &["hi"], files, origin);
    let stack = crate::common::stack_of(vec![pack]);
    assert_eq!(terrain(&stack).texture_name, "atlas.terrain");
    assert_eq!(stack.texture_atlases().terrain_atlas.layers().len(), 1);
}

// the folder does not say what `asFloat` makes of a
// double beyond a float; ours reads the largest finite float of its sign, with a warning, so
// the atlas can be written back. Vanilla stores the infinity.
#[test]
fn a_mip_fade_amount_beyond_a_float_is_the_largest_finite_float_deviation() {
    const FILE: &str = "textures/terrain_texture.json";
    let text = br#"{ "texture_name": "atlas.terrain", "texture_data": {
        "x": { "textures": { "path": "textures/a", "mip_fade_amount": -1e39 }, "mip_fade_amount": 1e39 },
        "y": { "textures": "textures/a", "mip_fade_amount": 1e400 },
        "z": { "textures": "textures/a", "mip_fade_amount": 3.0e38 } } }"#;
    let files: &[(&str, &[u8])] = &[(FILE, text), ("textures/a.png", b"")];
    let stack = crate::common::stack_of_files(&[files]);
    let atlas = terrain(&stack);
    assert_eq!(atlas.get("x").unwrap().mip_fade_amount(), f32::MAX);
    assert_eq!(
        object_layers(atlas.get("x").unwrap())[0].mip_fade_amount,
        Some(f32::MIN)
    );
    assert_eq!(atlas.get("y").unwrap().mip_fade_amount(), f32::MAX);
    // The largest values a float holds are themselves.
    assert_eq!(atlas.get("z").unwrap().mip_fade_amount(), 3.0e38);
    assert_eq!(
        findings(&stack),
        [
            finding(
                "InvalidValue",
                Info,
                "texture_data/x/mip_fade_amount",
                "invalid value"
            ),
            finding(
                "InvalidValue",
                Info,
                "texture_data/x/textures/mip_fade_amount",
                "invalid value"
            ),
            finding(
                "InvalidValue",
                Info,
                "texture_data/y/mip_fade_amount",
                "invalid value"
            ),
        ]
    );
    // The file read on its own says the same of each number.
    let alone = TextureAtlas::parse(&PackPath::from_static(FILE), text, &LoadContext::detached())
        .expect("the atlas loads");
    let mut told: Vec<String> = alone
        .diagnostics()
        .iter()
        .map(|finding| finding.location.json_path.to_string())
        .collect();
    told.sort();
    assert_eq!(
        told,
        [
            "texture_data/x/mip_fade_amount",
            "texture_data/x/textures/mip_fade_amount",
            "texture_data/y/mip_fade_amount",
        ]
    );
    // The atlas is written, and reads back to the same atlas without a finding.
    let written = atlas.to_json().expect("every number is finite");
    let again: &[(&str, &[u8])] = &[(FILE, written.as_bytes()), ("textures/a.png", b"")];
    let again = crate::common::stack_of_files(&[again]);
    assert_eq!(terrain(&again), atlas);
    assert_eq!(findings(&again), []);
}

/// A three-pack stack whose atlas `file` named `name` gives the tile `x` as an array of three
/// in the lowest pack, a string in the middle pack and an array of one in the top pack.
fn three_packs(file: &str, name: &str) -> PackStack<Resource> {
    let copy = |textures: &str| {
        format!(
            r#"{{ "texture_name": "{name}", "texture_data": {{ "x": {{ "textures": {textures} }} }} }}"#
        )
    };
    let (top, mid, low) = (
        copy(r#"["t0"]"#),
        copy(r#""m""#),
        copy(r#"["l0", "l1", "l2"]"#),
    );
    let top: &[(&str, &[u8])] = &[(file, top.as_bytes())];
    let mid: &[(&str, &[u8])] = &[(file, mid.as_bytes())];
    let low: &[(&str, &[u8])] = &[(file, low.as_bytes())];
    crate::common::stack_of_files(&[top, mid, low])
}

// the copies are folded highest first, `patchJson(base = the lower copy, patch = what
// the higher copies gave)`. The top array over the middle string is `["t0"]`, and that over
// the low array keeps the low tail. Folding from the bottom, the middle string would replace
// the low array first and the result would be `["t0"]`.
#[test]
fn three_copies_are_folded_from_the_top_so_the_lowest_tail_survives() {
    let stack = three_packs("textures/terrain_texture.json", "atlas.terrain");
    assert_eq!(
        x(&stack).textures,
        AtlasTextures::Many(vec![path("t0"), path("l1"), path("l2")])
    );
    let stack = three_packs("textures/item_texture.json", "atlas.items");
    let item = stack.texture_atlases().item().expect("an item atlas");
    assert_eq!(
        item.get("x").unwrap().textures,
        AtlasTextures::Many(vec![path("t0"), path("l1"), path("l2")])
    );
}

// `side` is the four horizontal faces: north, south, west and east.
#[test]
fn each_isotropic_face_has_its_bits_and_side_is_sixty() {
    let cases = [
        ("down", 1),
        ("up", 2),
        ("north", 4),
        ("south", 8),
        ("west", 16),
        ("east", 32),
        ("side", 60),
    ];
    let layers: Vec<String> = cases
        .iter()
        .map(|(face, _)| format!(r#"{{ "path": "a", "isotropic": {{ "{face}": true }} }}"#))
        .collect();
    let text = format!(
        r#"{{ "texture_name": "atlas.terrain", "texture_data": {{ "x": {{ "textures": [{}] }} }} }}"#,
        layers.join(", ")
    );
    let files: &[(&str, &[u8])] = &[("textures/terrain_texture.json", text.as_bytes())];
    let stack = crate::common::stack_of_files(&[files]);
    let layers = object_layers(x(&stack));
    for ((face, bits), layer) in cases.iter().zip(layers) {
        assert_eq!(layer.isotropic_mask(), *bits, "{face}");
        assert_eq!(Face::from_name(face).unwrap().bits(), *bits, "{face}");
    }
    assert_eq!(findings(&stack), []);
}

/// The colour texts are the game's content-log errors (the game's colour value parser,
/// `_contentLog(true, 3, 17)`) after which the atlas loads — kept
/// errors; the crate's own lints of the reader are informations.
#[test]
fn the_atlas_reader_grades_as_the_game_logs() {
    let stack = inline(
        r#"{ "texture_name": "atlas.terrain", "extra": 1, "texture_data": {
            "a": { "textures": [] },
            "b": { "textures": [5] },
            "c": { "textures": { "variations": [5] } },
            "d": { "textures": "t", "isotropic": { "bogus": true } },
            "e": { "textures": [ { "path": "t", "tint_color": [1.5, 2, 3] } ] },
            "f": { "textures": [ { "path": "t", "tint_color": 7 } ] } } }"#,
    );
    let mut told: Vec<(
        String,
        &'static str,
        bedrock_addon::diagnostic::Severity,
        bool,
    )> = stack
        .texture_atlases()
        .terrain_atlas
        .diagnostics()
        .filter(|finding| finding.code.name() != "TextureNotFound")
        .map(|finding| {
            (
                finding.location.json_path.to_string(),
                finding.code.name(),
                finding.severity,
                finding.kept,
            )
        })
        .collect();
    told.sort();
    assert_eq!(
        told,
        [
            ("extra".to_owned(), "UnrecognizedField", Info, false),
            (
                "texture_data/a/textures".to_owned(),
                "InvalidValue",
                Info,
                false
            ),
            (
                "texture_data/b/textures/0".to_owned(),
                "InvalidValue",
                Info,
                false
            ),
            (
                "texture_data/c/textures/variations/0".to_owned(),
                "InvalidValue",
                Info,
                false
            ),
            (
                "texture_data/d/isotropic".to_owned(),
                "UnrecognizedField",
                Info,
                false
            ),
            (
                "texture_data/e/textures/0/tint_color/0".to_owned(),
                "InvalidValue",
                Error,
                true
            ),
            (
                "texture_data/f/textures/0/tint_color".to_owned(),
                "InvalidValue",
                Error,
                true
            ),
        ]
    );
    // The atlas and every tile load.
    assert_eq!(terrain(&stack).len(), 6);
}
