//! 20 R34 and R35: each flipbook against the terrain atlas (its tile, the tile's index, the
//! index's variant) and against its image (the file, and with the file's header the frames).

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail, Diagnostics, Phase, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{LinkOptions, PackOrigin};

use crate::common::{hosted, in_file, leak, resource_pack_of, stacks_from};

const FLIPBOOKS: &str = "textures/flipbook_textures.json";
const ATLAS: &str = "textures/terrain_texture.json";
const LAVA: &str = "textures/blocks/lava.png";

/// A terrain atlas: `lava` with two indices, the second with two variations.
const TERRAIN: &str = r#"{ "texture_name": "atlas.terrain", "texture_data": {
    "lava": { "textures": [ "textures/blocks/lava",
        { "variations": [ { "path": "textures/blocks/lava" }, { "path": "textures/blocks/lava" } ] } ] } } }"#;

/// The first bytes of a PNG of this size (the header is all a rule looks at).
fn png(width: u32, height: u32) -> &'static [u8] {
    let mut bytes = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
    bytes.extend(width.to_be_bytes());
    bytes.extend(height.to_be_bytes());
    bytes.extend([8, 6, 0, 0, 0]);
    Box::leak(bytes.into_boxed_slice())
}

/// One flipbook entry with the members `members` beside its image `textures/blocks/lava`.
fn entry(members: &str) -> String {
    format!(r#"[ {{ "flipbook_texture": "textures/blocks/lava", {members} }} ]"#)
}

/// The link findings of a world whose one resource pack has `files`.
fn linked_with(
    files: &[(&'static str, &'static [u8])],
    load: LoadOptions,
    link: LinkOptions,
) -> Diagnostics {
    // The terrain atlas of these worlds is a pack the game ships: R34's tile half
    // is tested at its own severity. `tests/link/view.rs` has the other views.
    let (shipped, own): (Vec<_>, Vec<_>) =
        files.iter().copied().partition(|(path, _)| *path == ATLAS);
    stacks_from(
        Vec::new(),
        vec![
            (resource_pack_of(2, &own), PackOrigin::User),
            (resource_pack_of(3, &shipped), PackOrigin::Package),
        ],
        load,
    )
    .link_with(&link)
}

/// The link findings at the flipbook file of a world with the atlas [`TERRAIN`], a 16x32
/// image of two frames and the flipbooks `flipbooks`.
fn linked(flipbooks: &str) -> Vec<Diagnostic> {
    let found = linked_with(
        &[
            (FLIPBOOKS, leak(flipbooks)),
            (ATLAS, leak(TERRAIN)),
            (LAVA, png(16, 32)),
        ],
        LoadOptions::default(),
        LinkOptions::default(),
    );
    at_flipbooks(&found)
}

/// The findings at the flipbook file.
fn at_flipbooks(found: &Diagnostics) -> Vec<Diagnostic> {
    in_file(found, FLIPBOOKS).into_iter().cloned().collect()
}

/// What a finding says: its code, severity, JSON path and text.
fn said(finding: &Diagnostic) -> (&'static str, Severity, String, String) {
    (
        finding.code.name(),
        finding.severity,
        finding.location.json_path.to_string(),
        finding.code.to_string(),
    )
}

fn all_said(found: &[Diagnostic]) -> Vec<(&'static str, Severity, String, String)> {
    found.iter().map(said).collect()
}

fn told(
    code: &'static str,
    severity: Severity,
    at: &str,
    text: &str,
) -> (&'static str, Severity, String, String) {
    (code, severity, at.to_owned(), text.to_owned())
}

#[test]
fn r34_a_tile_index_and_variant_the_terrain_atlas_has_are_no_finding() {
    for members in [
        r#""atlas_tile": "lava""#,
        r#""atlas_tile": "lava", "atlas_index": 1"#,
        r#""atlas_tile": "lava", "atlas_index": 1, "atlas_tile_variant": 1"#,
        r#""atlas_tile": "lava", "atlas_index": 0, "atlas_tile_variant": 0, "frames": [0, 1]"#,
    ] {
        assert_eq!(linked(&entry(members)), [], "{members}");
    }
}

#[test]
fn r34_a_tile_no_atlas_has_is_an_information_at_the_tile_inside_the_missing_tile() {
    // The game animates its `missing` tile silently while the index is inside it.
    let found = linked(&entry(r#""atlas_tile": "ghost""#));
    assert_eq!(
        all_said(&found),
        [told(
            "UnresolvedReference",
            Severity::Info,
            "0/atlas_tile",
            "unresolved reference"
        )]
    );
    let outside = linked(&entry(r#""atlas_tile": "ghost", "atlas_index": 1"#));
    assert_eq!(
        all_said(&outside),
        [told(
            "UnresolvedReference",
            Severity::Error,
            "0/atlas_tile",
            "Texture ghost is missing from atlas textures"
        )]
    );
    let finding = &found[0];
    assert_eq!(finding.phase, Phase::Link);
    assert_eq!(
        finding.location.pack.as_ref().expect("the pack").name(),
        "rp 2"
    );
    assert_eq!(
        finding.location.path.as_ref().expect("the file").as_str(),
        FLIPBOOKS
    );
    match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { kind, id, hint }) => {
            assert_eq!((*kind, id.as_str(), hint), ("atlas key", "ghost", &None));
        }
        other => panic!("no reference: {other:?}"),
    }
}

// The atlas is compared as written.
#[test]
fn r34_a_tile_is_compared_as_written() {
    let found = linked(&entry(r#""atlas_tile": "Lava""#));
    assert_eq!(all_said(&found).len(), 1, "{found:#?}");
    assert_eq!(said(&found[0]).0, "UnresolvedReference");
}

#[test]
fn r34_a_tile_only_the_game_has_resolves_when_the_host_says_so() {
    let world = |options: LoadOptions| {
        let found = linked_with(
            &[
                (
                    FLIPBOOKS,
                    leak(&entry(r#""atlas_tile": "water", "atlas_index": 7"#)),
                ),
                (ATLAS, leak(TERRAIN)),
                (LAVA, png(16, 32)),
            ],
            options,
            LinkOptions::default(),
        );
        at_flipbooks(&found)
            .iter()
            .map(|f| f.code.name())
            .collect::<Vec<_>>()
    };
    // The game's tile: its indices are not known here and not checked.
    assert_eq!(world(hosted(&[("atlas key", "water")])), [] as [&str; 0]);
    assert_eq!(
        world(hosted(&[("atlas key", "lava")])),
        ["UnresolvedReference"]
    );
    assert_eq!(world(LoadOptions::default()), ["UnresolvedReference"]);
}

#[test]
fn r34_an_index_outside_the_tile_is_an_error_on_both_sides() {
    let text = |index: i32| {
        format!("Texture lava is using an invalid atlas index {index} for the expected UV count 2")
    };
    // `lava` has the indices 0 and 1.
    assert_eq!(
        linked(&entry(r#""atlas_tile": "lava", "atlas_index": 1"#)),
        []
    );
    for index in [2, -1] {
        let found = linked(&entry(&format!(
            r#""atlas_tile": "lava", "atlas_index": {index}"#
        )));
        assert_eq!(
            all_said(&found),
            [told(
                "InvalidValue",
                Severity::Error,
                "0/atlas_index",
                &text(index)
            )]
        );
        assert_eq!(found[0].phase, Phase::Link);
    }
}

#[test]
fn r34_a_variant_outside_the_index_is_a_clamp_warning_on_both_sides() {
    // Index 1 of `lava` has the variants 0 and 1.
    let variant = |variant: i32| {
        linked(&entry(&format!(
            r#""atlas_tile": "lava", "atlas_index": 1, "atlas_tile_variant": {variant}"#
        )))
    };
    assert_eq!(variant(1), []);
    for outside in [2, -1] {
        let found = variant(outside);
        assert_eq!(
            all_said(&found),
            [told(
                "ClampedValue",
                Severity::Info,
                "0/atlas_tile_variant",
                "value clamped into its range"
            )]
        );
        assert_eq!(found[0].phase, Phase::Link);
    }
}

#[test]
fn r35_an_image_no_pack_has_is_a_lint() {
    let flipbooks = r#"[ { "flipbook_texture": "textures/blocks/nope", "atlas_tile": "lava" } ]"#;
    let files: &[(&'static str, &'static [u8])] = &[
        (FLIPBOOKS, leak(flipbooks)),
        (ATLAS, leak(TERRAIN)),
        (LAVA, png(16, 32)),
    ];
    let found = at_flipbooks(&linked_with(
        files,
        LoadOptions::default(),
        LinkOptions::default(),
    ));
    assert_eq!(
        all_said(&found),
        [told(
            "TextureNotFound",
            Severity::Info,
            "0/flipbook_texture",
            "Texture metadata for texture: textures/blocks/nope is invalid"
        )]
    );
    assert_eq!(found[0].phase, Phase::Link);
    assert_eq!(
        found[0].location.pack.as_ref().expect("the pack").name(),
        "rp 2"
    );
    // Left out with the lints off.
    let quiet = linked_with(
        files,
        LoadOptions::default(),
        LinkOptions {
            lints: false,
            ..Default::default()
        },
    );
    assert_eq!(at_flipbooks(&quiet), []);
}

/// The findings of a flipbook with `frames` whose image has this size.
fn framed(frames: &str, width: u32, height: u32) -> Vec<(&'static str, Severity, String, String)> {
    let flipbooks = entry(&format!(r#""atlas_tile": "lava", "frames": {frames}"#));
    let found = linked_with(
        &[
            (FLIPBOOKS, leak(&flipbooks)),
            (ATLAS, leak(TERRAIN)),
            (LAVA, png(width, height)),
        ],
        LoadOptions::default(),
        LinkOptions::default(),
    );
    all_said(&at_flipbooks(&found))
}

const FRAME: &str = "frame index out of range!";

#[test]
fn r35_a_frame_outside_the_image_is_an_information_on_both_sides() {
    // 16x48: the frames 0, 1 and 2.
    assert_eq!(framed("[0, 1, 2]", 16, 48), []);
    assert_eq!(
        framed("[2, 3, 0, -1]", 16, 48),
        [
            told("InvalidValue", Severity::Info, "0/frames/1", FRAME),
            told("InvalidValue", Severity::Info, "0/frames/3", FRAME),
        ]
    );
}

#[test]
fn r35_an_image_wider_than_tall_has_no_frame() {
    assert_eq!(framed("[]", 16, 16), []);
    assert_eq!(
        framed("[]", 32, 16),
        [told(
            "InvalidValue",
            Severity::Info,
            "0/flipbook_texture",
            "Flipbook texture should have at least one frame. "
        )]
    );
}

// The frame rules need the header: a file that is no image of its extension has none.
#[test]
fn r35_an_image_whose_header_does_not_read_is_held_to_no_frame_rule() {
    let flipbooks = entry(r#""atlas_tile": "lava", "frames": [9]"#);
    let found = linked_with(
        &[
            (FLIPBOOKS, leak(&flipbooks)),
            (ATLAS, leak(TERRAIN)),
            (LAVA, b"not a png"),
        ],
        LoadOptions::default(),
        LinkOptions::default(),
    );
    assert_eq!(at_flipbooks(&found), []);
}
