//! The rows through the stack accessor `rp.flipbooks()`, the link rules, and the merge's cost.

#[cfg(all(feature = "texture_atlas", feature = "texture_file"))]
#[path = "link.rs"]
mod link;

use super::{common, cost};

use bedrock_addon::diagnostic::Severity::{Error, Info};
use bedrock_addon::document::Content;
use bedrock_addon::families::flipbooks::{Flipbook, FlipbookKey, Flipbooks};
use bedrock_addon::load::LoadContext;
use bedrock_addon::pack::{Pack, PackOrigin, PackStack, Resource};
use bedrock_addon::storage::PackPath;
use common::{Rendered, finding, fixture_dir, pack_in, pack_index, rendered, stack_of};

const FAMILY: &str = "flipbook_textures";
const FILE: &str = "textures/flipbook_textures.json";

/// The packs of fixture `name`, the highest first: `packs/0` is the lowest pack.
pub fn packs(name: &str) -> Vec<Pack> {
    let dir = fixture_dir(FAMILY, name).join("packs");
    let mut dirs: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .collect();
    dirs.sort();
    dirs.reverse();
    let count = dirs.len();
    dirs.iter()
        .enumerate()
        .map(|(top, dir)| pack_in(dir, (count - top) as u128, PackOrigin::User))
        .collect()
}

fn stack(name: &str) -> PackStack<Resource> {
    stack_of(packs(name))
}

fn entries(stack: &PackStack<Resource>) -> &[Flipbook] {
    &stack.flipbooks().entries
}

fn findings(stack: &PackStack<Resource>) -> Vec<Rendered> {
    rendered(stack.flipbooks().diagnostics())
}

fn tiles(stack: &PackStack<Resource>) -> Vec<&str> {
    entries(stack).iter().map(Flipbook::atlas_tile).collect()
}

fn invalid(at: &str) -> Rendered {
    finding("InvalidValue", Info, at, "invalid value")
}

// The vanilla file is `resource_pack/textures/flipbook_textures.json`,
// copied unchanged (its first line is a `//` comment).
#[test]
fn m00_the_vanilla_file_loads() {
    let stack = stack("m00_vanilla_1_26_50");
    assert_eq!(entries(&stack).len(), 83);
    assert_eq!(findings(&stack), []);
    // In file order.
    assert_eq!(entries(&stack)[0].atlas_tile(), "fire_0");
    let water = entries(&stack)
        .iter()
        .find(|e| e.atlas_tile() == "flowing_water_grey");
    assert_eq!(water.unwrap().replicate(), 2);
    let lava = entries(&stack)
        .iter()
        .find(|e| e.atlas_tile() == "flowing_lava");
    assert_eq!(lava.unwrap().replicate, Some(2));
}

#[test]
fn m01_distinct_entries_append_lowest_pack_first() {
    let stack = stack("m01_append_distinct");
    assert_eq!(tiles(&stack), ["magma", "sea_lantern"]);
    let flipbooks = &stack.flipbooks();
    let origin = |entry: &Flipbook| flipbooks.origin(&entry.merge_key()).unwrap().pack.index();
    // The stack is top first: `packs/1` is index 0.
    assert_eq!(origin(&entries(&stack)[0]), Some(1));
    assert_eq!(origin(&entries(&stack)[1]), Some(0));
    assert_eq!(flipbooks.layers().len(), 2);
    assert_eq!(flipbooks.layers()[0].origin.pack.index(), Some(0));
}

#[test]
fn m02_entries_of_one_key_merge_member_by_member() {
    let stack = stack("m02_same_key_field_merge");
    let [entry] = entries(&stack) else {
        panic!("{:?}", entries(&stack));
    };
    assert_eq!(entry.flipbook_texture(), "textures/blocks/y");
    // An array is replaced, never concatenated.
    assert_eq!(entry.frames, Some(vec![2, 1]));
    assert_eq!(entry.ticks_per_frame, Some(10));
    assert_eq!(entry.blend_frames, Some(false));
    assert_eq!(findings(&stack), []);
    // The entry is the higher pack's once it wrote to it.
    let origin = stack.flipbooks().origin(&entry.merge_key());
    assert_eq!(origin.unwrap().pack.index(), Some(0));
}

#[test]
fn m03_an_absent_index_and_variant_are_zero_in_the_key() {
    let stack = stack("m03_key_defaults");
    let [entry] = entries(&stack) else {
        panic!("{:?}", entries(&stack));
    };
    // Merged, and now only variant 0 is animated.
    assert_eq!(entry.atlas_tile_variant, Some(0));
    assert_eq!(entry.ticks_per_frame, Some(2));
}

#[test]
fn m04_another_index_or_variant_is_another_entry() {
    let stack = stack("m04_index_or_variant_differs");
    let keys: Vec<_> = entries(&stack).iter().map(Flipbook::merge_key).collect();
    let key = |tile: &str, index, variant| FlipbookKey {
        tile: tile.to_owned(),
        index,
        variant,
    };
    assert_eq!(
        keys,
        [
            key("x", 0, 0),
            key("y", 0, 1),
            key("x", 1, 0),
            key("y", 0, 2)
        ]
    );
}

#[test]
fn m05_the_image_path_is_not_part_of_the_key() {
    let stack = stack("m05_flipbook_texture_not_in_key");
    let [entry] = entries(&stack) else {
        panic!("{:?}", entries(&stack));
    };
    assert_eq!(entry.flipbook_texture(), "textures/blocks/high");
}

#[test]
fn m06_duplicates_of_the_lowest_file_are_merged_deviation() {
    // `vanilla = "lowest parsable file is copied wholesale"`: vanilla keeps both entries of
    // the lowest file, and only the first gets the higher pack's members.
    let stack = stack("m06_first_match_only");
    let [entry] = entries(&stack) else {
        panic!("{:?}", entries(&stack));
    };
    assert_eq!(entry.ticks_per_frame, Some(3));
    assert_eq!(entry.replicate, Some(2));
    assert_eq!(entry.flipbook_texture(), "textures/blocks/high");
    assert_eq!(
        findings(&stack),
        [finding(
            "DuplicateIdentifier",
            Info,
            "1",
            "duplicate identifier"
        )]
    );
    assert_eq!(
        pack_index(stack.flipbooks().diagnostics().next().unwrap()),
        Some(1)
    );
}

#[test]
fn m07_a_duplicate_in_a_higher_file_merges_into_the_first() {
    let stack = stack("m07_duplicate_in_higher_file");
    assert_eq!(tiles(&stack), ["y", "x"]);
    assert_eq!(entries(&stack)[1].ticks_per_frame, Some(3));
    assert_eq!(entries(&stack)[1].replicate, Some(2));
    assert_eq!(
        findings(&stack),
        [finding(
            "DuplicateIdentifier",
            Info,
            "1",
            "duplicate identifier"
        )]
    );
}

#[test]
fn m08_a_parse_error_in_the_lowest_file_skips_that_file_deviation() {
    let stack = stack("m08_parse_error_lowest");
    assert_eq!(tiles(&stack), ["mine"]);
    assert_eq!(
        entries(&stack)[0].flipbook_texture(),
        "textures/blocks/myTexture1"
    );
    let found: Vec<_> = stack.flipbooks().diagnostics().collect();
    assert_eq!(found.len(), 1);
    assert_eq!(
        (found[0].code.name(), found[0].severity),
        ("MalformedInput", Error)
    );
    // `packs/0` is the lower of the two.
    assert_eq!(pack_index(found[0]), Some(1));
    assert_eq!(found[0].location.path.as_ref().unwrap().as_str(), FILE);
}

#[test]
fn m09_a_parse_error_in_the_middle_skips_that_file_deviation() {
    let stack = stack("m09_parse_error_middle");
    assert_eq!(tiles(&stack), ["magma", "sea_lantern"]);
    let found: Vec<_> = stack.flipbooks().diagnostics().collect();
    assert_eq!(found.len(), 1);
    assert_eq!(
        (found[0].code.name(), found[0].severity),
        ("MalformedInput", Error)
    );
    assert_eq!(pack_index(found[0]), Some(1));
    assert_eq!(stack.flipbooks().layers().len(), 2);
}

#[test]
fn m10_a_root_that_is_no_array_is_ignored_deviation() {
    // `vanilla = "silent; kept as root when lowest"`.
    let stack = stack("m10_non_array_root_higher");
    assert_eq!(tiles(&stack), ["magma"]);
    assert_eq!(findings(&stack), [invalid("")]);
    assert_eq!(
        pack_index(stack.flipbooks().diagnostics().next().unwrap()),
        Some(0)
    );
}

#[test]
fn m11_an_object_root_in_the_lowest_file_is_ignored_deviation() {
    // Vanilla keeps the object as the root and collapses it into an all-`null` entry (tile
    // `""`) before `x`.
    let stack = stack("m11_non_array_root_lowest");
    assert_eq!(tiles(&stack), ["x"]);
    assert_eq!(findings(&stack), [invalid("")]);
}

#[test]
fn m13_an_empty_object_root_in_the_lowest_file_is_ignored() {
    let stack = stack("m13_empty_object_root_lowest");
    assert_eq!(tiles(&stack), ["x"]);
    assert_eq!(findings(&stack), [invalid("")]);
}

#[test]
fn m14_a_null_root_in_the_lowest_file_is_ignored() {
    let stack = stack("m14_null_root_lowest");
    assert_eq!(tiles(&stack), ["x"]);
    assert_eq!(findings(&stack), [invalid("")]);
}

#[test]
fn m12_elements_that_are_no_objects_are_dropped_deviation() {
    // `vanilla = "silent; root-file element becomes {}"`: vanilla keeps four entries, three of
    // them with the tile `""`.
    let stack = stack("m12_non_object_elements");
    assert_eq!(tiles(&stack), ["magma"]);
    assert_eq!(findings(&stack), [invalid("0"), invalid("1"), invalid("2")]);
}

#[test]
fn p00_defaults() {
    let stack = stack("p00_defaults");
    let entry = &entries(&stack)[0];
    assert_eq!(entry.atlas_index(), 0);
    assert_eq!(entry.atlas_tile_variant, None);
    assert_eq!(entry.ticks_per_frame(), 1);
    assert_eq!(entry.frames, None);
    assert_eq!(entry.replicate(), 1);
    assert!(entry.blend_frames());
    // Nothing the entry does not write is held.
    assert_eq!(
        (
            entry.atlas_index,
            entry.ticks_per_frame,
            entry.replicate,
            entry.blend_frames
        ),
        (None, None, None, None)
    );
    assert_eq!(entry.tile().as_str(), "magma");
    assert_eq!(findings(&stack), []);
}

#[test]
fn p01_frames_as_an_integer_is_ignored_deviation() {
    // `vanilla = "silent"`.
    let stack = stack("p01_frames_integer");
    assert_eq!(entries(&stack)[0].frames, None);
    assert_eq!(findings(&stack), [invalid("0/frames")]);
}

#[test]
fn p02_an_empty_frames_array_is_absent() {
    let stack = stack("p02_frames_empty_array");
    assert_eq!(entries(&stack)[0].frames, None);
    assert_eq!(findings(&stack), []);
}

#[test]
fn p03_members_of_the_wrong_type_are_coerced_as_vanilla_does_deviation() {
    // `vanilla = "silent coercion"`.
    let stack = stack("p03_type_coercions");
    let entry = &entries(&stack)[0];
    assert_eq!(entry.ticks_per_frame, Some(0));
    assert_eq!(entry.blend_frames, Some(true));
    assert_eq!(entry.flipbook_texture.as_deref(), Some(""));
    assert_eq!(entry.atlas_index, Some(1));
    assert_eq!(entry.atlas_tile_variant, Some(0));
    assert_eq!(entry.frames, Some(vec![1, 0, 2]));
    assert_eq!(
        findings(&stack),
        [
            invalid("0/flipbook_texture"),
            invalid("0/atlas_index"),
            invalid("0/atlas_tile_variant"),
            invalid("0/ticks_per_frame"),
            invalid("0/blend_frames"),
            invalid("0/frames/1"),
            invalid("0/frames/2"),
            finding(
                "ClampedValue",
                Info,
                "0/ticks_per_frame",
                "value clamped into its range"
            ),
        ]
    );
}

#[test]
fn p04_nulls_are_absent() {
    let stack = stack("p04_nulls");
    let entry = &entries(&stack)[0];
    assert_eq!(entry.atlas_tile_variant, None);
    assert_eq!(entry.ticks_per_frame(), 1);
    // The member is held as absent (it held the default, `Some(1)`, and was written as a
    // `1` the author did not write).
    assert_eq!(entry.ticks_per_frame, None);
    assert_eq!(findings(&stack), []);
}

/// Every member of an entry written `null` is absent, and the file is written
/// without it; the same member with a value is held and written.
#[test]
fn a_null_member_of_an_entry_is_held_as_absent_and_not_written() {
    let read = |text: &str| {
        Flipbooks::parse(
            &PackPath::from_static("textures/flipbook_textures.json"),
            text.as_bytes(),
            &LoadContext::detached(),
        )
        .expect("the file loads")
    };
    let nulls = read(
        r#"[{"flipbook_texture": "textures/x", "atlas_tile": "x", "atlas_index": null, "atlas_tile_variant": null, "ticks_per_frame": null, "replicate": null, "blend_frames": null}]"#,
    );
    let entry = &nulls.value().entries[0];
    assert_eq!(
        (
            entry.atlas_index,
            entry.atlas_tile_variant,
            entry.ticks_per_frame,
            entry.replicate,
            entry.blend_frames
        ),
        (None, None, None, None, None)
    );
    let written = nulls.to_json().expect("written");
    for member in [
        "atlas_index",
        "atlas_tile_variant",
        "ticks_per_frame",
        "replicate",
        "blend_frames",
    ] {
        assert!(!written.contains(member), "{member}: {written}");
    }
    let values = read(
        r#"[{"flipbook_texture": "textures/x", "atlas_tile": "x", "atlas_index": 0, "ticks_per_frame": 1, "replicate": 1, "blend_frames": true}]"#,
    );
    let entry = &values.value().entries[0];
    assert_eq!(
        (
            entry.atlas_index,
            entry.ticks_per_frame,
            entry.replicate,
            entry.blend_frames
        ),
        (Some(0), Some(1), Some(1), Some(true))
    );
}

#[test]
fn p05_comments_are_allowed() {
    let stack = stack("p05_comments");
    assert_eq!(tiles(&stack), ["x"]);
    assert_eq!(findings(&stack), []);
}

#[test]
fn p06_an_unknown_member_is_kept_deviation() {
    let stack = stack("p06_unknown_member");
    assert!(entries(&stack)[0].extra.get("fps").is_some());
    assert_eq!(
        findings(&stack),
        [finding(
            "UnrecognizedField",
            Info,
            "0/fps",
            "unrecognized field"
        )]
    );
}

#[test]
fn p07_replicate() {
    let stack = stack("p07_replicate");
    assert_eq!(entries(&stack)[0].replicate, Some(2));
    // A string reads as 0, not as the default.
    assert_eq!(entries(&stack)[1].replicate, Some(0));
    assert_eq!(findings(&stack), [invalid("1/replicate")]);
}

#[test]
fn p08_a_comment_after_a_member_s_comma_is_allowed() {
    let stack = stack("p08_trailing_line_comment");
    assert_eq!(entries(&stack)[0].flipbook_texture(), "textures/blocks/a");
    assert_eq!(findings(&stack), []);
}

#[test]
fn p09_integers_outside_32_bits_wrap_and_reals_saturate() {
    let stack = stack("p09_integer_wrap");
    let entry = &entries(&stack)[0];
    assert_eq!(entry.atlas_index, Some(1));
    assert_eq!(entry.ticks_per_frame, Some(i32::MIN));
    assert_eq!(entry.effective_ticks_per_frame(), 1);
    assert_eq!(entry.replicate, Some(i32::MAX));
    assert_eq!(
        findings(&stack),
        [
            invalid("0/atlas_index"),
            invalid("0/ticks_per_frame"),
            invalid("0/replicate"),
            finding(
                "ClampedValue",
                Info,
                "0/ticks_per_frame",
                "value clamped into its range"
            ),
        ]
    );
}

#[test]
fn v03_ticks_per_frame_below_one_is_treated_as_one_deviation() {
    // `vanilla = "silent clamp"`; the value is kept as written.
    let stack = stack("v03_ticks_per_frame_zero");
    assert_eq!(entries(&stack)[0].ticks_per_frame, Some(0));
    assert_eq!(entries(&stack)[1].ticks_per_frame, Some(-4));
    assert_eq!(entries(&stack)[1].effective_ticks_per_frame(), 1);
    assert_eq!(
        findings(&stack),
        [
            finding(
                "ClampedValue",
                Info,
                "0/ticks_per_frame",
                "value clamped into its range"
            ),
            finding(
                "ClampedValue",
                Info,
                "1/ticks_per_frame",
                "value clamped into its range"
            ),
        ]
    );
}

#[test]
fn ticks_per_frame_of_one_is_no_finding() {
    let stack =
        common::stack_of_files(&[&[(FILE, br#"[ { "atlas_tile": "x", "ticks_per_frame": 1 } ]"#)]]);
    assert_eq!(findings(&stack), []);
}

/// A flipbook file of `count` entries with distinct tiles.
fn many_flipbooks(count: usize) -> String {
    let entries: Vec<String> = (0..count)
        .map(|n| format!(r#"{{ "flipbook_texture": "textures/f{n}", "atlas_tile": "tile{n}" }}"#))
        .collect();
    format!("[{}]", entries.join(",\n"))
}

// The merge indexes the entries by key once: 20 000 entries took 133 s when each entry scanned
// the merged array. Four times the entries may take about four times
// the time (5 000 against 20 000).
#[test]
fn twenty_thousand_distinct_entries_merge_in_linear_time() {
    let load = |count: usize| {
        let text = many_flipbooks(count);
        let top: &[(&str, &[u8])] = &[(FILE, text.as_bytes())];
        let low: &[(&str, &[u8])] = &[(FILE, text.as_bytes())];
        let stack = common::stack_of_files(&[top, low]);
        let (textures, elapsed) = cost::timed(|| stack.flipbooks());
        let flipbooks = textures;
        assert_eq!(
            flipbooks.entries.len(),
            count,
            "both files hold the same keys"
        );
        let last = format!("tile{}", count - 1);
        assert_eq!(
            flipbooks.entries[count - 1].atlas_tile.as_deref(),
            Some(last.as_str())
        );
        elapsed
    };
    cost::assert_linear_timed_at("the flipbook merge", 5_000, load);
}
