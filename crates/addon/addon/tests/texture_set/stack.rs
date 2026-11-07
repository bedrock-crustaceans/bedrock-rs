//! Which file the client loads for a texture path, where a layer's
//! image is looked up, and the capability lints.

use bedrock_addon::diagnostic::Severity::{Error, Info};
use bedrock_addon::families::texture_file::{ImageFormat, TextureSource};
use bedrock_addon::families::texture_set::{LayerKind, LayerValue, Rgba, TextureName};
use bedrock_addon::id::TexturePath;
use bedrock_addon::pack::{OpenOptions, Pack, PackOrigin, PackStack, Resource};
use bedrock_addon::storage::PackPath;

use crate::common::{finding, fixture_file, rendered, stack_of};
use crate::{FAMILY, STONE, STONE_SET, file, findings, rejected, set, stack};

const PNG: &str = "textures/blocks/stone.png";

fn name(text: &str) -> LayerValue<Rgba> {
    LayerValue::Texture(TextureName(text.to_owned()))
}

#[test]
fn ts50_a_texture_set_with_its_image() {
    let stack = stack("ts50_explicit_only");
    let file = file(&stack, STONE);
    assert_eq!(file.stack_index, 0);
    assert_eq!(file.pack.index(), Some(0));
    let document = file.set.expect("the set");
    assert_eq!(document.color, Some(name("stone")));
    assert_eq!(document.origin().path.as_str(), STONE_SET);
    // The colour's image is the layer's, in the set's pack.
    let image = file.image.expect("the colour image");
    assert_eq!(image.key, PNG);
    assert_eq!(image.path.as_str(), PNG);
    assert_eq!(image.format, Some(ImageFormat::Png));
    assert_eq!(findings(&stack), []);
}

#[test]
fn ts51_an_image_alone_is_an_implicit_set() {
    let stack = stack("ts51_implicit_only");
    let file = file(&stack, STONE);
    assert!(file.set.is_none() && file.rejected.is_none());
    assert!(!file.is_missing());
    assert_eq!(file.image.expect("the image").key, PNG);
    assert_eq!(stack.texture_sets().sets().count(), 0);
}

#[test]
fn ts52_an_image_in_a_higher_pack_wins_over_a_set_below() {
    let stack = stack("ts52_implicit_higher_wins");
    let file = file(&stack, STONE);
    assert_eq!(file.stack_index, 0);
    assert!(file.set.is_none());
    assert_eq!(file.image.expect("the image").key, PNG);
    // The lower pack's set is loaded and listed, and not what the client uses.
    let listed: Vec<_> = stack.texture_sets().sets().collect();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].0, STONE);
    assert_eq!(listed[0].1.origin().pack.index(), Some(1));
}

#[test]
fn ts53_a_set_in_a_higher_pack_wins_over_an_image_below() {
    let stack = stack("ts53_explicit_higher_wins");
    let file = file(&stack, STONE);
    assert_eq!(file.stack_index, 0);
    assert!(file.set.is_some());
}

#[test]
fn ts54_in_one_pack_the_set_wins_the_tie() {
    let stack = stack("ts54_same_pack_tie");
    assert!(file(&stack, STONE).set.is_some());
    assert_eq!(
        stack.texture_files().source(STONE),
        Some(TextureSource::TextureSet { stack_index: 0 })
    );
}

#[test]
fn ts55_sets_of_two_packs_are_never_merged() {
    let stack = stack("ts55_explicit_both_packs");
    let document = set(&stack, STONE);
    assert_eq!(document.origin().pack.index(), Some(0));
    assert_eq!(
        document.uniform(LayerKind::Color),
        Some(Rgba {
            r: 1.0,
            g: 2.0,
            b: 3.0,
            a: 4.0
        })
    );
    // Ours reads the lower copy too (the load is eager); it is listed and never used.
    assert_eq!(stack.texture_sets().sets().count(), 2);
}

#[test]
fn ts56_a_layer_s_image_is_the_one_in_the_set_s_own_pack() {
    let stack = stack("ts56_layer_image_same_pack");
    let file = file(&stack, STONE);
    assert_eq!(file.stack_index, 1, "the set is the lower pack's");
    let images = stack.layer_images();
    assert_eq!(images.len(), 1);
    assert_eq!(images[0].layer, LayerKind::Mer);
    assert_eq!(images[0].pack.index(), Some(1));
    assert_eq!(images[0].image.key, "textures/blocks/stone_mer.png");
    assert_eq!(findings(&stack), []);
}

#[test]
fn v11_an_image_only_another_pack_has_does_not_serve_the_layer() {
    let set = br#"{ "format_version": "1.21.30", "minecraft:texture_set": { "color": [1, 2, 3, 4], "normal": "stone_n" } }"#;
    let top: &[(&str, &[u8])] = &[(STONE_SET, set)];
    let below: &[(&str, &[u8])] = &[("textures/blocks/stone_n.png", b"")];
    // The image is in the pack below the set's, and then in the pack above it.
    for packs in [[top, below], [below, top]] {
        let stack = crate::common::stack_of_files(&packs);
        assert_eq!(rejected(&stack, STONE), [no_image("normal")]);
        assert!(stack.layer_images().is_empty());
    }
}

// `vanilla = "image lookup uses BackwardsCompatTextureGroup path when canUse()"`: the group is
// client data we do not have, so `getBackCompatResourceLocation` is the identity. A texture
// path finds the image at that very path, and an image under any other name is not a stand-in.
#[test]
fn a_texture_path_is_looked_up_as_written_without_a_back_compat_name_deviation() {
    let pack: &[(&str, &[u8])] = &[
        ("textures/blocks/planks_oak.png", b""),
        ("textures/blocks/planks.png", b""),
    ];
    let stack = crate::common::stack_of_files(&[pack]);
    let image = file(&stack, "textures/blocks/planks_oak")
        .image
        .expect("the image at the path");
    assert_eq!(image.key, "textures/blocks/planks_oak.png");
    // No pack has the path of the other image's old name, and nothing is looked up under it.
    let old = TexturePath::parse("textures/blocks/planks_old".to_owned());
    assert!(stack.resolve_texture(&old).is_none());
}

#[test]
fn ts57_tga_comes_before_png_and_jpg() {
    let stack = stack("ts57_extension_priority");
    let image = file(&stack, STONE).image.expect("the colour image");
    assert_eq!(image.key, "textures/blocks/stone.tga");
    assert_eq!(image.format, Some(ImageFormat::Tga));
}

#[test]
fn ts58_a_name_with_an_extension_is_that_file() {
    let stack = stack("ts58_explicit_extension");
    let grass = file(&stack, "textures/blocks/grass");
    assert_eq!(grass.image.expect("grass").key, "textures/blocks/grass.tga");
    let checker = file(&stack, "textures/blocks/checker");
    let image = checker.image.expect("checkerboard");
    assert_eq!(image.key, "textures/blocks/checkerboard.ktx");
    assert_eq!(image.format, Some(ImageFormat::Ktx));
    assert_eq!(findings(&stack), []);
}

/// V11 at layer `key`.
fn no_image(key: &str) -> super::Rendered {
    // `_addImageToLayer` logs it with `_contentLog(true, 1, 32)`, an information;
    // the set is still not loaded.
    finding(
        "texture_set/set_no_image",
        Info,
        &format!("minecraft:texture_set/{key}"),
        "Specified image could not be found",
    )
}

// `vanilla = "Inform, no path in the text; whole texture replaced by the missing texture"`.
#[test]
fn ts59_a_missing_layer_image_fails_the_whole_set_deviation() {
    let stack = stack("ts59_missing_layer_image");
    assert_eq!(rejected(&stack, STONE), [no_image("normal")]);
    assert_eq!(stack.texture_sets().sets().count(), 0);
    assert_eq!(stack.texture_sets().rejected_sets().count(), 1);
}

#[test]
fn ts60_a_name_with_a_folder_is_joined_as_written() {
    let stack = stack("ts60_subdirectory_name");
    let image = file(&stack, STONE).image.expect("the colour image");
    assert_eq!(image.key, "textures/blocks/sub/stone.png");
    assert_eq!(findings(&stack), []);
}

// `vanilla = "no message on the ResourceLocation path"`: the reference's own family reports
// the miss (here the terrain atlas), and the resolver has no file.
#[test]
fn ts61_a_path_no_pack_has_resolves_to_nothing_deviation() {
    let stack = stack("ts61_not_found");
    let path = TexturePath::parse("textures/blocks/nothing");
    assert!(stack.resolve_texture(&path).is_none());
    let found = findings(&stack);
    assert_eq!(found.len(), 1, "{found:?}");
    // atlas rule 14; an information
    assert_eq!((found[0].0, found[0].1), ("core/texture_not_found", Info));
}

// Loading reads no image, so the channel rule is the caller's to run on the bytes it
// reads (`vanilla` checks the image while it loads the set).
#[test]
fn ts62_a_three_channel_image_on_mers_is_the_wrong_format_deviation() {
    let stack = stack("ts62_mers_rgb_image");
    let images = stack.layer_images();
    assert_eq!(images.len(), 1);
    let image = &images[0];
    assert_eq!(image.layer, LayerKind::Mers);
    let bytes = fixture_file(
        FAMILY,
        "ts62_mers_rgb_image",
        &format!("pack_a/{}", image.image.path),
    );
    let found = image.check(&bytes).expect("a finding");
    assert_eq!(
        rendered([&found]),
        [finding(
            "core/layer_image_format",
            Error,
            "minecraft:texture_set/metalness_emissive_roughness_subsurface",
            "layer image has an unsupported channel count"
        )]
    );
    assert_eq!(found.location.path.as_ref().unwrap().as_str(), STONE_SET);
    assert_eq!(found.location.pack.as_ref().unwrap().index(), Some(0));
    // The same bytes pass on a layer that takes three channels.
    let four = fixture_file(FAMILY, "ts50_explicit_only", &format!("pack_a/{PNG}"));
    assert!(image.check(&four).is_none(), "four channels pass");
}

#[test]
fn ts63_a_missing_colour_image_fails_the_set() {
    let stack = stack("ts63_color_image_missing");
    assert_eq!(rejected(&stack, STONE), [no_image("color")]);
}

// ---- V12, V13: the image rules on bytes the caller supplies --------------------------------

/// A PNG header of colour type `kind` (0 grey, 2 RGB, 4 grey-alpha, 6 RGBA).
fn png(kind: u8) -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
    bytes.extend([0, 0, 0, 1, 0, 0, 0, 1, 8, kind, 0, 0, 0]);
    bytes
}

/// The finding of `bytes` as the image of the one layer of `layers` (a 1.21.30 payload beside
/// `color`) that names `textures/blocks/x.png`.
fn checked(layers: &str, bytes: &[u8]) -> Option<&'static str> {
    let text =
        format!(r#"{{ "format_version": "1.21.30", "minecraft:texture_set": {{ {layers} }} }}"#);
    let files: &[(&str, &[u8])] = &[(STONE_SET, text.as_bytes()), ("textures/blocks/x.png", b"")];
    let stack = crate::common::stack_of_files(&[files]);
    let images = stack.layer_images();
    assert_eq!(images.len(), 1, "{:?}", findings(&stack));
    images[0].check(bytes).map(|found| {
        assert_eq!(found.severity, Error);
        found.code.id()
    })
}

#[test]
fn v13_each_layer_takes_its_own_channel_counts() {
    const C: &str = r#""color": [1, 2, 3, 4]"#;
    let (grey, rgb, grey_alpha, rgba) = (png(0), png(2), png(4), png(6));
    let bad = Some("core/layer_image_format");
    // color, normal and metalness_emissive_roughness: 3 or 4.
    for layer in [
        r#""color": "x""#.to_owned(),
        format!(r#"{C}, "normal": "x""#),
        format!(r#"{C}, "metalness_emissive_roughness": "x""#),
    ] {
        assert_eq!(checked(&layer, &grey), bad, "{layer}");
        assert_eq!(checked(&layer, &grey_alpha), bad, "{layer}");
        assert_eq!(checked(&layer, &rgb), None, "{layer}");
        assert_eq!(checked(&layer, &rgba), None, "{layer}");
    }
    // metalness_emissive_roughness_subsurface: 4 only.
    let mers = format!(r#"{C}, "metalness_emissive_roughness_subsurface": "x""#);
    assert_eq!(checked(&mers, &rgb), bad);
    assert_eq!(checked(&mers, &rgba), None);
    // heightmap: 1 to 4.
    let heightmap = format!(r#"{C}, "heightmap": "x""#);
    for bytes in [&grey, &grey_alpha, &rgb, &rgba] {
        assert_eq!(checked(&heightmap, bytes), None);
    }
}

// Only the header is read, and by the caller (`vanilla` decodes the image at load).
#[test]
fn v12_an_image_whose_header_does_not_decode_is_undecodable_deviation() {
    assert_eq!(
        checked(r#""color": "x""#, b"not a png"),
        Some("core/layer_image_undecodable")
    );
    assert_eq!(
        checked(r#""color": "x""#, b""),
        Some("core/layer_image_undecodable")
    );
}

// ---- 5. capability lints ---------------------------------------------------------------------

const SET: &[u8] =
    br#"{ "format_version": "1.21.30", "minecraft:texture_set": { "color": [1, 2, 3, 4] } }"#;

/// A resource pack with UUID `n`, the given `min_engine_version` and capabilities, and `files`.
fn pack(
    n: u128,
    min_engine: [u16; 3],
    capabilities: &[&str],
    files: &[(&str, &[u8])],
    origin: PackOrigin,
) -> Pack {
    let capabilities: Vec<String> = capabilities.iter().map(|c| format!("\"{c}\"")).collect();
    let manifest = format!(
        r#"{{ "format_version": 2,
  "header": {{ "name": "pack {n}", "description": "", "uuid": "{}", "version": [1, 0, 0],
              "min_engine_version": [{}, {}, {}] }},
  "modules": [{{ "type": "resources", "uuid": "{}", "version": [1, 0, 0] }}],
  "capabilities": [{}] }}"#,
        uuid::Uuid::from_u128(n),
        min_engine[0],
        min_engine[1],
        min_engine[2],
        uuid::Uuid::from_u128(n | 0xffff_0000_0000_0000_0000),
        capabilities.join(", "),
    );
    let mut all = vec![(
        PackPath::new("manifest.json").unwrap(),
        manifest.into_bytes(),
    )];
    for (path, bytes) in files {
        all.push((PackPath::new(path).unwrap(), bytes.to_vec()));
    }
    let options = OpenOptions {
        origin,
        ..OpenOptions::default()
    };
    Pack::from_files(all, &options).unwrap_or_else(|error| panic!("pack {n}: {error}"))
}

/// The lints of the group: code, severity and the pack's place in the stack.
fn lints(stack: &PackStack<Resource>) -> Vec<(&'static str, usize)> {
    crate::group(stack)
        .map(|found| {
            assert_eq!(found.severity, Info, "{found}");
            assert_eq!(
                found.location.path.as_ref().unwrap().as_str(),
                "manifest.json"
            );
            (found.code.id(), crate::common::pack_index(found).unwrap())
        })
        .collect()
}

const WITH_SET: &[(&str, &[u8])] = &[(STONE_SET, SET)];
const OLD: [u16; 3] = [1, 21, 0];

#[test]
fn ts70_texture_sets_without_a_capability_get_a_hint() {
    let stack = stack_of(vec![pack(1, OLD, &[], WITH_SET, PackOrigin::User)]);
    assert_eq!(lints(&stack), [("core/capability_hint", 0)]);
    // A pack without texture sets gets none.
    let none = stack_of(vec![pack(1, OLD, &[], &[], PackOrigin::User)]);
    assert_eq!(lints(&none), []);
}

#[test]
fn ts71_a_pack_with_pbr_gets_no_lint() {
    let stack = stack_of(vec![pack(
        1,
        [1, 21, 120],
        &["pbr"],
        WITH_SET,
        PackOrigin::User,
    )]);
    assert_eq!(lints(&stack), []);
}

#[test]
fn ts72_raytraced_alone_is_enough_for_the_hint() {
    let stack = stack_of(vec![pack(
        1,
        OLD,
        &["raytraced"],
        WITH_SET,
        PackOrigin::User,
    )]);
    assert_eq!(lints(&stack), []);
}

#[test]
fn r3_a_capability_of_any_pack_of_the_stack_silences_the_hint() {
    let stack = stack_of(vec![
        pack(1, OLD, &[], WITH_SET, PackOrigin::User),
        pack(2, OLD, &["pbr"], &[], PackOrigin::User),
    ]);
    assert_eq!(lints(&stack), []);
    // Without it, each pack that ships sets gets its own hint, and the other none.
    let stack = stack_of(vec![
        pack(1, OLD, &[], &[], PackOrigin::User),
        pack(2, OLD, &["other"], WITH_SET, PackOrigin::User),
    ]);
    assert_eq!(lints(&stack), [("core/capability_hint", 1)]);
}

#[test]
fn ts73_a_pack_from_1_21_110_without_pbr_turns_vibrant_visuals_off() {
    let stack = stack_of(vec![pack(1, [1, 21, 110], &[], &[], PackOrigin::User)]);
    assert_eq!(
        lints(&stack),
        [("core/vibrant_visuals_disabled_by_pack", 0)]
    );
    // `raytraced` alone does not count.
    let stack = stack_of(vec![pack(
        1,
        [1, 21, 110],
        &["raytraced"],
        &[],
        PackOrigin::User,
    )]);
    assert_eq!(
        lints(&stack),
        [("core/vibrant_visuals_disabled_by_pack", 0)]
    );
    // `pbr` does.
    let stack = stack_of(vec![pack(1, [1, 21, 110], &["pbr"], &[], PackOrigin::User)]);
    assert_eq!(lints(&stack), []);
}

#[test]
fn ts74_a_pack_below_1_21_110_never_turns_it_off() {
    let stack = stack_of(vec![pack(1, [1, 21, 100], &[], &[], PackOrigin::User)]);
    assert_eq!(lints(&stack), []);
    // The last version below the gate.
    let stack = stack_of(vec![pack(1, [1, 21, 109], &[], &[], PackOrigin::User)]);
    assert_eq!(lints(&stack), []);
}

#[test]
fn r1_a_base_game_pack_never_turns_vibrant_visuals_off() {
    let stack = stack_of(vec![pack(1, [1, 26, 0], &[], &[], PackOrigin::Package)]);
    assert_eq!(lints(&stack), []);
}

// `getFileTypeExtensions(1)`: the path as written is tried before any image extension.
#[test]
fn a_path_that_names_a_file_is_that_file_before_any_extension_is_tried() {
    let files: &[(&str, &[u8])] = &[
        ("textures/x.png", b""),
        ("textures/x.png.tga", b""),
        ("textures/y.ktx", b""),
        ("textures/y.ktx.png", b""),
        ("textures/y.ktx.tga", b""),
    ];
    let stack = crate::common::stack_of_files(&[files]);
    for (path, key, format) in [
        ("textures/x.png", "textures/x.png", ImageFormat::Png),
        ("textures/y.ktx", "textures/y.ktx", ImageFormat::Ktx),
        // Without the verbatim file the extensions follow in their order.
        ("textures/x", "textures/x.png", ImageFormat::Png),
    ] {
        let image = file(&stack, path).image.expect("the image");
        assert_eq!(image.key, key, "{path}");
        assert_eq!(image.format, Some(format), "{path}");
        assert_eq!(
            stack.texture_files().source(path),
            Some(TextureSource::Image {
                stack_index: 0,
                file: key.to_owned()
            }),
            "{path}"
        );
    }
}
