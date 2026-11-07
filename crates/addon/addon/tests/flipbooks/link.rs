//! Fixture rows v00 to v02 and v04 to v08: each flipbook against the terrain atlas
//! (rules 9 to 11) and against its image (rules 12 to 14). Both
//! are link rules, run by `StackSet::link`; the checker itself is tested in
//! `src/pack/link/resource/flipbooks_texture.rs`.

use bedrock_addon::diagnostic::Severity::{self, Error, Info};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Pack, PackOrigin, PackRepository, PackSelection, StackSet, WorldPacks};

use super::{FILE, packs};

/// The stacks of a world whose resource packs are `packs`, the first on top.
fn stacks(packs: Vec<Pack>) -> StackSet {
    let world = WorldPacks {
        behavior: Vec::new(),
        resource: packs
            .iter()
            .map(|pack| PackSelection {
                pack_id: pack.id().uuid,
                version: pack.id().version.clone(),
                subpack: None,
            })
            .collect(),
    };
    let mut repository = PackRepository::new();
    for pack in packs {
        // The packs, which hold the terrain atlas of these tests, are packs the game
        // ships, so R34's tile half has its own severity
        repository.add_pack(pack, PackOrigin::Package);
    }
    repository.stacks(&world, LoadOptions::default())
}

/// The link findings at the flipbook file: code, severity, place and text.
fn linked(packs: Vec<Pack>) -> Vec<(&'static str, Severity, String, String)> {
    stacks(packs)
        .link()
        .iter()
        .filter(|f| f.location.path.as_ref().is_some_and(|p| p.as_str() == FILE))
        .map(|f| {
            (
                f.code.id(),
                f.severity,
                f.location.json_path.to_string(),
                f.code.to_string(),
            )
        })
        .collect()
}

fn found(
    code: &'static str,
    severity: Severity,
    at: &str,
    text: &str,
) -> (&'static str, Severity, String, String) {
    (code, severity, at.to_owned(), text.to_owned())
}

const FRAME: &str = "frame index out of range!";

#[test]
fn v00_an_unknown_tile_is_an_unresolved_reference_deviation() {
    // `vanilla = "falls back to the atlas's 'missing' tile; error only when atlas_index is
    // outside that tile's UV-set count, otherwise the missing tile is animated silently"`.
    assert_eq!(
        linked(packs("v00_unknown_tile")),
        [found(
            "core/unresolved_reference",
            Info,
            "0/atlas_tile",
            "unresolved reference"
        )]
    );
}

#[test]
fn v07_an_unknown_tile_at_index_1_gets_vanilla_s_text() {
    assert_eq!(
        linked(packs("v07_unknown_tile_index_1")),
        [found(
            "flipbooks/tile_missing",
            Error,
            "0/atlas_tile",
            "Texture ghost is missing from atlas textures"
        )]
    );
}

#[test]
fn v01_an_atlas_index_out_of_range_is_an_error() {
    let text = |index: i32| {
        format!("Texture dirt is using an invalid atlas index {index} for the expected UV count 2")
    };
    assert_eq!(
        linked(packs("v01_atlas_index_out_of_range")),
        [
            found(
                "flipbooks/bad_atlas_index",
                Error,
                "0/atlas_index",
                &text(2)
            ),
            found(
                "flipbooks/bad_atlas_index",
                Error,
                "1/atlas_index",
                &text(-1)
            ),
        ]
    );
}

#[test]
fn v02_a_variant_out_of_range_is_clamped_deviation() {
    // `vanilla = "silent clamp to 0"`.
    assert_eq!(
        linked(packs("v02_variant_out_of_range")),
        [found(
            "core/clamped_value",
            Info,
            "0/atlas_tile_variant",
            "value clamped into its range"
        )]
    );
}

#[test]
fn v04_a_frame_index_out_of_range_is_a_warning_deviation() {
    // `vanilla = "none (assert reads the element before it is assigned)"`. A 16x32 image has
    // two frames: `2` and `-1` are outside.
    assert_eq!(
        linked(packs("v04_frame_index_out_of_range")),
        [
            found("flipbooks/frame_out_of_range", Info, "0/frames/2", FRAME),
            found("flipbooks/frame_out_of_range", Info, "0/frames/3", FRAME),
        ]
    );
}

#[test]
fn v05_an_image_without_a_frame_is_a_warning_deviation() {
    // `vanilla = "debug assert only"`.
    assert_eq!(
        linked(packs("v05_no_frames")),
        [found(
            "flipbooks/no_frames",
            Info,
            "0/flipbook_texture",
            "Flipbook texture should have at least one frame. "
        )]
    );
}

#[test]
fn v06_a_missing_image_is_an_info_deviation() {
    // `vanilla = "debug assert only"`; info.
    assert_eq!(
        linked(packs("v06_missing_image")),
        [found(
            "flipbooks/no_metadata",
            Info,
            "0/flipbook_texture",
            "Texture metadata for texture: textures/blocks/nope is invalid"
        )]
    );
}

#[test]
fn v08_the_highest_pack_wins_and_inside_a_pack_the_extension_order() {
    // The top pack has `lava.tga` (16x64, four frames) and `lava.png` (16x32); the lower pack
    // `lava.jpg` (16x16). The entry plays frame 3.
    assert_eq!(linked(packs("v08_image_extension_order")), []);
    // Without the `.tga`, the `.png` of the same pack: two frames.
    let dir = super::fixture_dir(super::FAMILY, "v08_image_extension_order").join("packs");
    let pack = |n: u128, name: &str, skip: &[&str]| {
        let files = super::common::files_below(&dir.join(name));
        let files: Vec<(&str, &[u8])> = files
            .iter()
            .filter(|(path, _)| !skip.iter().any(|suffix| path.ends_with(suffix)))
            .map(|(path, bytes)| (path.as_str(), bytes.as_slice()))
            .collect();
        super::common::pack_of(n, &files, PackOrigin::User)
    };
    assert_eq!(
        linked(vec![pack(2, "1", &[".tga"]), pack(1, "0", &[])]),
        [found(
            "flipbooks/frame_out_of_range",
            Info,
            "0/frames/0",
            FRAME
        )]
    );
    // With no image in the top pack, the lower pack's `.jpg`: one frame.
    assert_eq!(
        linked(vec![pack(2, "1", &[".tga", ".png"]), pack(1, "0", &[])]),
        [found(
            "flipbooks/frame_out_of_range",
            Info,
            "0/frames/0",
            FRAME
        )]
    );
}
