//! `texture_set` V12 and V13: the image of each image layer of each loaded texture set, checked
//! from the bytes the stack holds (the load reads no image).

use bedrock_addon::diagnostic::{Diagnostic, Diagnostics, Phase, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::LinkOptions;

use crate::common::{in_file, leak, resource_pack_of, stacks_of};

const SET: &str = "textures/blocks/stone.texture_set.json";

/// The first bytes of a 1x1 PNG of colour type `kind` (0 grey, 2 RGB, 4 grey-alpha, 6 RGBA).
fn png(kind: u8) -> &'static [u8] {
    let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
    bytes.extend([0, 0, 0, 1, 0, 0, 0, 1, 8, kind, 0, 0, 0]);
    Box::leak(bytes.into_boxed_slice())
}

/// A 1.21.30 texture set with `layers`.
fn set(layers: &str) -> &'static [u8] {
    leak(&format!(
        r#"{{ "format_version": "1.21.30", "minecraft:texture_set": {{ {layers} }} }}"#
    ))
}

/// The link findings of a world whose resource packs have these files, the first on top.
fn linked(packs: &[&[(&'static str, &'static [u8])]]) -> Diagnostics {
    let packs = packs
        .iter()
        .enumerate()
        .map(|(index, files)| resource_pack_of(2 + index as u128, files))
        .collect();
    stacks_of(Vec::new(), packs, LoadOptions::default()).link()
}

/// What a finding says: its code, severity and JSON path.
fn said(finding: &Diagnostic) -> (&'static str, Severity, String) {
    (
        finding.code.name(),
        finding.severity,
        finding.location.json_path.to_string(),
    )
}

/// What the findings at the texture set say.
fn at_set(found: &Diagnostics) -> Vec<(&'static str, Severity, String)> {
    in_file(found, SET).into_iter().map(said).collect()
}

#[test]
fn v12_a_layer_image_whose_header_does_not_decode_is_an_error_at_the_layer() {
    let found = linked(&[&[
        (SET, set(r#""color": "stone""#)),
        ("textures/blocks/stone.png", b"not a png"),
    ]]);
    assert_eq!(
        at_set(&found),
        [(
            "LayerImageUndecodable",
            Severity::Error,
            "minecraft:texture_set/color".to_owned()
        )]
    );
    let finding = in_file(&found, SET)[0];
    assert_eq!(finding.phase, Phase::Link);
    assert_eq!(
        finding.location.pack.as_ref().expect("the pack").name(),
        "rp 2"
    );
}

#[test]
fn v13_a_layer_image_of_a_channel_count_the_layer_does_not_take_is_an_error_at_the_layer() {
    // `metalness_emissive_roughness_subsurface` takes four channels only.
    let world = |kind: u8| {
        at_set(&linked(&[&[
            (
                SET,
                set(
                    r#""color": [1, 2, 3, 4], "metalness_emissive_roughness_subsurface": "stone_mers""#,
                ),
            ),
            ("textures/blocks/stone_mers.png", png(kind)),
        ]]))
    };
    assert_eq!(world(6), []);
    assert_eq!(
        world(2),
        [(
            "LayerImageFormat",
            Severity::Error,
            "minecraft:texture_set/metalness_emissive_roughness_subsurface".to_owned()
        )]
    );
    // `color` takes three or four.
    let color = |kind: u8| {
        at_set(&linked(&[&[
            (SET, set(r#""color": "stone""#)),
            ("textures/blocks/stone.png", png(kind)),
        ]]))
    };
    assert_eq!(color(2), []);
    assert_eq!(color(6), []);
    assert_eq!(
        color(4),
        [(
            "LayerImageFormat",
            Severity::Error,
            "minecraft:texture_set/color".to_owned()
        )]
    );
}

#[test]
fn v12_v13_every_image_layer_of_a_set_is_checked() {
    let found = linked(&[&[
        (
            SET,
            set(
                r#""color": "stone", "normal": "stone_n", "metalness_emissive_roughness": "stone_mer""#,
            ),
        ),
        ("textures/blocks/stone.png", png(6)),
        ("textures/blocks/stone_n.png", png(0)),
        ("textures/blocks/stone_mer.png", b""),
    ]]);
    let mut said = at_set(&found);
    said.sort();
    assert_eq!(
        said,
        [
            (
                "LayerImageFormat",
                Severity::Error,
                "minecraft:texture_set/normal".to_owned()
            ),
            (
                "LayerImageUndecodable",
                Severity::Error,
                "minecraft:texture_set/metalness_emissive_roughness".to_owned()
            ),
        ]
    );
}

// The findings are errors: no lint, and told with the lints off.
#[test]
fn v12_v13_are_told_with_the_lints_off() {
    let set = stacks_of(
        Vec::new(),
        vec![resource_pack_of(
            2,
            &[
                (SET, set(r#""color": "stone""#)),
                ("textures/blocks/stone.png", b"not a png"),
            ],
        )],
        LoadOptions::default(),
    );
    let found = set.link_with(&LinkOptions {
        lints: false,
        ..Default::default()
    });
    assert_eq!(at_set(&found).len(), 1);
}

// The load holds a layer to an image in the set's own pack (V11): the image the rules read is
// that pack's, whatever a higher pack has at the path.
#[test]
fn v12_v13_read_the_image_of_the_set_s_own_pack() {
    let found = linked(&[
        &[("textures/blocks/stone_n.png", b"not a png")],
        &[
            (SET, set(r#""color": [1, 2, 3, 4], "normal": "stone_n""#)),
            ("textures/blocks/stone_n.png", png(6)),
        ],
    ]);
    assert_eq!(at_set(&found), []);
}

/// An 18-byte TGA header of this image type, colour-map entry size and pixel depth.
fn tga(kind: u8, map_entry: u8, depth: u8) -> &'static [u8] {
    let mut bytes = vec![0u8; 18];
    bytes[1] = u8::from(map_entry != 0);
    bytes[2] = kind;
    bytes[7] = map_entry;
    bytes[12] = 16;
    bytes[14] = 16;
    bytes[16] = depth;
    Box::leak(bytes.into_boxed_slice())
}

/// The first chunks of a 1x1 colour-mapped PNG: `IHDR`, a palette of one colour and, with
/// `alpha`, the `tRNS` chunk that gives the palette alpha. (No reader here checks a CRC.)
fn palette_png(alpha: bool) -> &'static [u8] {
    let mut bytes = png(3).to_vec();
    // `png` stops after the `IHDR` data: its CRC first.
    bytes.extend([0; 4]);
    bytes.extend(b"\0\0\0\x03PLTE\x10\x20\x30\0\0\0\0");
    if alpha {
        bytes.extend(b"\0\0\0\x01tRNS\x80\0\0\0\0");
    }
    bytes.extend(b"\0\0\0\0IDAT\0\0\0\0");
    Box::leak(bytes.into_boxed_slice())
}

// The channel count V13 compares is the decoded image's, and every format is judged by it
// (K118). A TGA decodes to four channels whatever its header's depth: the game's own packs put
// 24-bit TGAs on `metalness_emissive_roughness_subsurface` (four channels only) and a
// colour-mapped one on `normal`. A colour-mapped PNG decodes to its palette's colours: three
// channels, four with a `tRNS` chunk.
#[test]
fn v13_judges_a_tga_and_a_colour_mapped_png_by_their_decoded_channels() {
    const MERS: &str =
        r#""color": [1, 2, 3, 4], "metalness_emissive_roughness_subsurface": "stone_mers""#;
    // A true-colour TGA of 24 bits, and one of 8 bits with a colour map.
    for image in [tga(2, 0, 24), tga(10, 0, 24), tga(1, 24, 8)] {
        let found = linked(&[&[(SET, set(MERS)), ("textures/blocks/stone_mers.tga", image)]]);
        assert_eq!(at_set(&found), []);
    }
    let found = linked(&[&[
        (SET, set(r#""color": [1, 2, 3, 4], "normal": "stone_n""#)),
        ("textures/blocks/stone_n.tga", tga(1, 24, 8)),
    ]]);
    assert_eq!(at_set(&found), []);
    // A PNG of colour type 3 has three channels: enough for `color`, with or without alpha.
    for image in [png(3), palette_png(false), palette_png(true)] {
        let found = linked(&[&[
            (SET, set(r#""color": "stone""#)),
            ("textures/blocks/stone.png", image),
        ]]);
        assert_eq!(at_set(&found), []);
    }
    // On the layer that takes four channels only, the palette needs its alpha: without the
    // `tRNS` chunk the image is refused, as an RGB PNG is (ts62).
    let found = linked(&[&[
        (SET, set(MERS)),
        ("textures/blocks/stone_mers.png", palette_png(true)),
    ]]);
    assert_eq!(at_set(&found), []);
    let found = linked(&[&[
        (SET, set(MERS)),
        ("textures/blocks/stone_mers.png", palette_png(false)),
    ]]);
    assert_eq!(
        at_set(&found),
        [(
            "LayerImageFormat",
            Severity::Error,
            "minecraft:texture_set/metalness_emissive_roughness_subsurface".to_owned()
        )]
    );
    // A colour-mapped image is no longer counted as its one index: a grey PNG still is one
    // channel, too few for `color`.
    let found = linked(&[&[
        (SET, set(r#""color": "stone""#)),
        ("textures/blocks/stone.png", png(0)),
    ]]);
    assert_eq!(
        at_set(&found),
        [(
            "LayerImageFormat",
            Severity::Error,
            "minecraft:texture_set/color".to_owned()
        )]
    );
    // A TGA whose header does not read is still no image (V12).
    let found = linked(&[&[
        (SET, set(MERS)),
        ("textures/blocks/stone_mers.tga", tga(0, 0, 24)),
    ]]);
    assert_eq!(
        at_set(&found),
        [(
            "LayerImageUndecodable",
            Severity::Error,
            "minecraft:texture_set/metalness_emissive_roughness_subsurface".to_owned()
        )]
    );
}
