//! The geometry checks, in vanilla's order, each the file's one finding: rows vs08 to vs11,
//! vs13 to vs15, vs17 (vs12, vs16 and vs18 are the accepted sides, in `rows.rs`).

use super::common::*;

const BOXES: &str = "minecraft:voxel_shape/shape/boxes";

/// The findings of the base-game shape `minecraft:<name>` made of `boxes`.
#[track_caller]
fn rejected(name: &str, boxes: &[(&str, &str)]) -> Vec<(&'static str, String, String)> {
    let text = shape(&format!("minecraft:{name}"), boxes);
    errors(&rejects_in(&text, &base_game()))
}

/// The base-game shape `minecraft:ok` made of `boxes` loads.
#[track_caller]
fn accepted(boxes: &[(&str, &str)]) {
    loads_in(&shape("minecraft:ok", boxes), &base_game());
}

/// V3's finding for box `index` of `minecraft:<name>`.
fn invalid(index: usize, name: &str) -> [(&'static str, String, String); 1] {
    [error(
        "voxel_shape/box_invalid",
        &format!("Box at index {index} is invalid for voxel shape `minecraft:{name}`"),
        &format!("{BOXES}/{index}"),
    )]
}

/// V4's finding for box `index` of `minecraft:<name>`.
fn outside(index: usize, name: &str) -> [(&'static str, String, String); 1] {
    [error(
        "voxel_shape/box_out_of_bounds",
        &format!(
            "Box at index {index} in voxel shape 'minecraft:{name}' is outside the allowed bounds of (-14, -14, -14) to (30, 30, 30)."
        ),
        &format!("{BOXES}/{index}"),
    )]
}

/// V5's finding for `minecraft:<name>`.
fn no_overlap(name: &str) -> [(&'static str, String, String); 1] {
    [error(
        "voxel_shape/no_overlap",
        &format!(
            "Voxel shape 'minecraft:{name}' has no boxes that overlap the base block (0-16 pixels on all axes)."
        ),
        BOXES,
    )]
}

/// V6's finding for `minecraft:<name>`: `extent` pixels on `axis`.
fn too_wide(name: &str, axis: &str, extent: &str) -> [(&'static str, String, String); 1] {
    [error(
        "voxel_shape/extent_too_wide",
        &format!(
            "Total extent of voxel shape 'minecraft:{name}' on axis {axis} is {extent} pixels, which exceeds the maximum of 30 pixels (1 block + 14 pixels)."
        ),
        BOXES,
    )]
}

const SLAB: (&str, &str) = ("[0, 0, 0]", "[16, 8, 16]");

// ---- V3 ----------------------------------------------------------------------------------

#[test]
fn vs08_a_box_whose_min_equals_its_max_on_an_axis_is_invalid() {
    assert_eq!(
        rejected("invalid_box", &[("[0, 0, 0]", "[0, 8, 16]")]),
        invalid(0, "invalid_box")
    );
}

#[test]
fn vs09_a_box_whose_min_is_above_its_max_is_invalid() {
    assert_eq!(
        rejected("invalid_box_reversed", &[("[16, 0, 0]", "[0, 8, 16]")]),
        invalid(0, "invalid_box_reversed")
    );
}

#[test]
fn vs10_the_first_invalid_box_is_named_by_its_index() {
    assert_eq!(
        rejected("mixed_validity", &[SLAB, ("[0, 8, 8]", "[0, 16, 16]")]),
        invalid(1, "mixed_validity")
    );
}

#[test]
fn v3_every_axis_needs_min_below_max() {
    for bad in [
        ("[0, 8, 0]", "[16, 8, 16]"),
        ("[0, 9, 0]", "[16, 8, 16]"),
        ("[0, 0, 16]", "[16, 8, 16]"),
        ("[0, 0, 16.5]", "[16, 8, 16]"),
        ("[4, 0, 0]", "[4, 8, 16]"),
        ("[4.5, 0, 0]", "[4, 8, 16]"),
    ] {
        assert_eq!(rejected("axis", &[bad]), invalid(0, "axis"), "{bad:?}");
    }
    // The thinnest box there is.
    accepted(&[("[1, 1, 1]", "[1.001, 1.001, 1.001]")]);
}

#[test]
fn the_namespace_is_checked_before_the_boxes() {
    let text = shape("minecraft:invalid_box", &[("[0, 0, 0]", "[0, 8, 16]")]);
    let found = errors(&rejects(&text));
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0]
            .1
            .starts_with("The `minecraft` namespace is not valid")
    );
}

// ---- V4 ----------------------------------------------------------------------------------

#[test]
fn vs11_a_box_below_minus_14_is_outside_the_bounds() {
    assert_eq!(
        rejected("outside_error_bounds", &[("[-15, 0, 0]", "[16, 16, 16]")]),
        outside(0, "outside_error_bounds")
    );
}

#[test]
fn v4_every_min_is_at_least_minus_14_and_every_max_at_most_30() {
    for bad in [
        ("[-14.001, 0, 0]", "[16, 16, 16]"),
        ("[0, -15, 0]", "[16, 16, 16]"),
        ("[0, -14.001, 0]", "[16, 16, 16]"),
        ("[0, 0, -15]", "[16, 16, 16]"),
        ("[0, 0, -14.001]", "[16, 16, 16]"),
        ("[0, 0, 0]", "[31, 16, 16]"),
        ("[0, 0, 0]", "[30.001, 16, 16]"),
        ("[0, 0, 0]", "[16, 31, 16]"),
        ("[0, 0, 0]", "[16, 30.001, 16]"),
        ("[0, 0, 0]", "[16, 16, 31]"),
        ("[0, 0, 0]", "[16, 16, 30.001]"),
    ] {
        assert_eq!(rejected("bounds", &[bad]), outside(0, "bounds"), "{bad:?}");
    }
    // Exactly at a bound, one axis at a time (vs12 is min.x, vs18 every max).
    accepted(&[("[0, -14, 0]", "[16, 16, 16]")]);
    accepted(&[("[0, 0, -14]", "[16, 16, 16]")]);
    accepted(&[("[0, 0, 0]", "[30, 16, 16]")]);
    accepted(&[("[0, 0, 0]", "[16, 30, 16]")]);
    accepted(&[("[0, 0, 0]", "[16, 16, 30]")]);
}

#[test]
fn vs13_only_the_first_box_outside_the_bounds_is_reported() {
    assert_eq!(
        rejected(
            "outside_twice",
            &[
                ("[-15, 0, 0]", "[16, 16, 16]"),
                ("[0, 0, 0]", "[16, 31, 16]")
            ]
        ),
        outside(0, "outside_twice")
    );
}

#[test]
fn a_box_is_checked_for_validity_before_its_bounds_and_before_the_next_box() {
    // Invalid and outside at once: invalid.
    assert_eq!(
        rejected("both", &[("[40, 0, 0]", "[35, 16, 16]")]),
        invalid(0, "both")
    );
    // An earlier box outside the bounds comes before a later invalid one.
    assert_eq!(
        rejected(
            "order",
            &[("[-15, 0, 0]", "[16, 16, 16]"), ("[0, 0, 0]", "[0, 8, 16]")]
        ),
        outside(0, "order")
    );
    assert_eq!(
        rejected(
            "order",
            &[
                SLAB,
                ("[0, 0, 0]", "[16, 31, 16]"),
                ("[0, 0, 0]", "[0, 8, 16]")
            ]
        ),
        outside(1, "order")
    );
    assert_eq!(
        rejected(
            "order",
            &[
                SLAB,
                ("[0, 0, 0]", "[0, 8, 16]"),
                ("[0, 0, 0]", "[16, 31, 16]")
            ]
        ),
        invalid(1, "order")
    );
}

// ---- V5 ----------------------------------------------------------------------------------

#[test]
fn vs14_a_box_that_only_touches_the_block_does_not_overlap_it() {
    assert_eq!(
        rejected("at_boundary_box", &[("[16, 0, 0]", "[20, 16, 16]")]),
        no_overlap("at_boundary_box")
    );
}

#[test]
fn v5_the_overlap_is_strict_on_both_sides_of_every_axis() {
    for touching in [
        ("[-4, 0, 0]", "[0, 16, 16]"),
        ("[0, 16, 0]", "[16, 20, 16]"),
        ("[0, -4, 0]", "[16, 0, 16]"),
        ("[0, 0, 16]", "[16, 16, 20]"),
        ("[0, 0, -4]", "[16, 16, 0]"),
        // Apart, not only touching.
        ("[18, 0, 0]", "[20, 16, 16]"),
        ("[0, 0, -6]", "[16, 16, -2]"),
    ] {
        assert_eq!(
            rejected("touching", &[touching]),
            no_overlap("touching"),
            "{touching:?}"
        );
    }
    // Any positive overlap counts (vs16 is the high side of x).
    accepted(&[("[-4, 0, 0]", "[0.5, 16, 16]")]);
    accepted(&[("[0, 15.5, 0]", "[16, 20, 16]")]);
    accepted(&[("[0, -4, 0]", "[16, 0.5, 16]")]);
    accepted(&[("[0, 0, 15.5]", "[16, 16, 20]")]);
    accepted(&[("[0, 0, -4]", "[16, 16, 0.5]")]);
}

#[test]
fn vs15_the_boxes_overlap_the_block_one_by_one_not_as_a_whole() {
    // Their bounding box spans the block; neither box enters it.
    assert_eq!(
        rejected(
            "multi_box_no_overlap",
            &[
                ("[-8, 0, 0]", "[0, 16, 16]"),
                ("[0, 16, 0]", "[16, 24, 16]")
            ]
        ),
        no_overlap("multi_box_no_overlap")
    );
}

#[test]
fn v5_one_overlapping_box_is_enough_wherever_it_is() {
    accepted(&[("[16, 0, 0]", "[20, 16, 16]"), SLAB]);
    accepted(&[SLAB, ("[16, 0, 0]", "[20, 16, 16]")]);
}

#[test]
fn every_box_is_checked_before_the_overlap() {
    assert_eq!(
        rejected(
            "order",
            &[
                ("[16, 0, 0]", "[20, 16, 16]"),
                ("[0, 0, 0]", "[31, 16, 16]")
            ]
        ),
        outside(1, "order")
    );
}

// ---- V6 ----------------------------------------------------------------------------------

#[test]
fn vs17_a_shape_wider_than_30_pixels_on_x_is_rejected() {
    assert_eq!(
        rejected(
            "oversized_extent",
            &[
                ("[-8, 0, 0]", "[16, 16, 16]"),
                ("[16, 0, 0]", "[24, 16, 16]")
            ]
        ),
        too_wide("oversized_extent", "x", "32")
    );
}

#[test]
fn v6_the_extent_is_measured_on_y_and_z_too() {
    assert_eq!(
        rejected(
            "tall",
            &[
                ("[0, -8, 0]", "[16, 16, 16]"),
                ("[0, 16, 0]", "[16, 24, 16]")
            ]
        ),
        too_wide("tall", "y", "32")
    );
    assert_eq!(
        rejected(
            "deep",
            &[
                ("[0, 0, 16]", "[16, 16, 24]"),
                ("[0, 0, -8]", "[16, 16, 16]")
            ]
        ),
        too_wide("deep", "z", "32")
    );
}

#[test]
fn v6_only_the_first_axis_over_the_limit_is_reported() {
    let wide_and_tall = [
        ("[-8, -9, 0]", "[16, 16, 16]"),
        ("[16, 16, 0]", "[24, 24, 16]"),
    ];
    assert_eq!(
        rejected("first_axis", &wide_and_tall),
        too_wide("first_axis", "x", "32")
    );
    let tall_and_deep = [
        ("[0, -9, -10]", "[16, 16, 16]"),
        ("[0, 16, 16]", "[16, 24, 24]"),
    ];
    assert_eq!(
        rejected("first_axis", &tall_and_deep),
        too_wide("first_axis", "y", "33")
    );
}

#[test]
fn v6_the_extent_is_printed_as_vanilla_prints_a_float() {
    // `{:g}`: six significant digits, no trailing zeros.
    assert_eq!(
        rejected(
            "half",
            &[
                ("[-14, 0, 0]", "[16, 16, 16]"),
                ("[0, 0, 0]", "[16.5, 16, 16]")
            ]
        ),
        too_wide("half", "x", "30.5")
    );
    assert_eq!(
        rejected(
            "digits",
            &[
                ("[-14, 0, 0]", "[16, 16, 16]"),
                ("[0, 0, 0]", "[16.123457, 16, 16]")
            ]
        ),
        too_wide("digits", "x", "30.1235")
    );
}

#[test]
fn v6_an_extent_of_exactly_30_over_several_boxes_loads() {
    accepted(&[
        ("[-14, 0, 0]", "[8, 16, 16]"),
        ("[8, 0, 0]", "[16, 16, 16]"),
    ]);
    accepted(&[
        ("[0, -14, 0]", "[16, 8, 16]"),
        ("[0, 8, 0]", "[16, 16, 16]"),
    ]);
    accepted(&[("[0, 0, 0]", "[16, 16, 8]"), ("[0, 0, 8]", "[16, 16, 30]")]);
}

#[test]
fn the_overlap_is_checked_before_the_extent() {
    assert_eq!(
        rejected(
            "apart",
            &[
                ("[-14, 0, 0]", "[0, 16, 16]"),
                ("[16, 0, 0]", "[18, 16, 16]")
            ]
        ),
        no_overlap("apart")
    );
}

#[test]
fn v6_the_limit_is_30_pixels_to_the_thousandth_on_every_axis() {
    // Review round 1: a thousandth of a pixel over is rejected, exactly 30 is not.
    assert_eq!(
        rejected(
            "wide",
            &[
                ("[-14, 0, 0]", "[16, 16, 16]"),
                ("[0, 0, 0]", "[16.001, 16, 16]")
            ]
        ),
        too_wide("wide", "x", "30.001")
    );
    assert_eq!(
        rejected(
            "tall",
            &[
                ("[0, -14, 0]", "[16, 16, 16]"),
                ("[0, 0, 0]", "[16, 16.001, 16]")
            ]
        ),
        too_wide("tall", "y", "30.001")
    );
    assert_eq!(
        rejected(
            "deep",
            &[
                ("[0, 0, -14]", "[16, 16, 16]"),
                ("[0, 0, 0]", "[16, 16, 16.001]")
            ]
        ),
        too_wide("deep", "z", "30.001")
    );
    // And well below a thousandth: 30.00006 is over the limit (printed with six digits).
    for (axis, name) in ["x", "y", "z"].into_iter().enumerate() {
        let mut low = ["0"; 3];
        let mut high = ["16"; 3];
        low[axis] = "-14";
        high[axis] = "16.00006";
        let low = format!("[{}]", low.join(", "));
        let high = format!("[{}]", high.join(", "));
        assert_eq!(
            rejected("hair", &[(&low, "[16, 16, 16]"), ("[0, 0, 0]", &high)]),
            too_wide("hair", name, "30.0001")
        );
    }
    accepted(&[
        ("[-14, 0, 0]", "[16, 16, 16]"),
        ("[0, 0, 0]", "[16, 16, 16]"),
    ]);
    accepted(&[
        ("[0, -14, 0]", "[16, 16, 16]"),
        ("[0, 0, 0]", "[16, 16, 16]"),
    ]);
    accepted(&[
        ("[0, 0, -14]", "[16, 16, 16]"),
        ("[0, 0, 0]", "[16, 16, 16]"),
    ]);
    accepted(&[
        ("[-13.999, 0, 0]", "[16, 16, 16]"),
        ("[0, 0, 0]", "[16, 16, 16]"),
    ]);
}

#[test]
fn v5_an_overlap_of_a_thousandth_of_a_pixel_counts_on_both_sides_of_every_axis() {
    // Review round 1: the low side enters the block at 0, the high side at 16.
    for entering in [
        ("[-4, 0, 0]", "[0.001, 16, 16]"),
        ("[15.999, 0, 0]", "[20, 16, 16]"),
        ("[0, -4, 0]", "[16, 0.001, 16]"),
        ("[0, 15.999, 0]", "[16, 20, 16]"),
        ("[0, 0, -4]", "[16, 16, 0.001]"),
        ("[0, 0, 15.999]", "[16, 16, 20]"),
    ] {
        accepted(&[entering]);
    }
    // Two boxes that each only touch a face of the block, per axis.
    for (low, high) in [
        (
            ("[-4, 0, 0]", "[0, 16, 16]"),
            ("[16, 0, 0]", "[20, 16, 16]"),
        ),
        (
            ("[0, -4, 0]", "[16, 0, 16]"),
            ("[0, 16, 0]", "[16, 20, 16]"),
        ),
        (
            ("[0, 0, -4]", "[16, 16, 0]"),
            ("[0, 0, 16]", "[16, 16, 20]"),
        ),
    ] {
        assert_eq!(rejected("touching", &[low, high]), no_overlap("touching"));
    }
}

// ---- the two comparisons at the neighbouring floats ---------

/// The `f32` next above `value` (both positive).
fn next_up(value: f32) -> f32 {
    f32::from_bits(value.to_bits() + 1)
}

/// `[a, b, c]` with the shortest text of each float that reads back to it.
fn corner(coordinates: [f32; 3]) -> String {
    let [x, y, z] = coordinates;
    format!("[{x}, {y}, {z}]")
}

/// V6 compares the extent with 30 itself: the first `f32` above 30 is over the limit, on every
/// axis. The extent is exact in `f32` (16 and 30 have the same spacing), and printed with six
/// significant digits it is "30".
#[test]
fn v6_the_first_float_above_30_is_over_the_limit_on_every_axis() {
    let over = next_up(30.0);
    assert!(over > 30.0 && over < 30.000_01);
    assert_eq!(next_up(16.0) - -14.0_f32, over, "the extent is exact");
    for (axis, name) in ["x", "y", "z"].into_iter().enumerate() {
        let mut low = [0.0_f32; 3];
        let mut high = [16.0_f32; 3];
        low[axis] = -14.0;
        high[axis] = next_up(16.0);
        let (low, high) = (corner(low), corner(high));
        assert_eq!(
            rejected("hair", &[(&low, "[16, 16, 16]"), ("[0, 0, 0]", &high)]),
            too_wide("hair", name, "30"),
            "{name}"
        );
        // The same two boxes ending at 16 span exactly 30 and load.
        accepted(&[(&low, "[16, 16, 16]"), ("[0, 0, 0]", "[16, 16, 16]")]);
    }
}

/// V5 compares with the block's faces themselves: a box that enters the block by the smallest
/// positive `f32` on the low side, or starts one `f32` below 16 on the high side, overlaps it,
/// on every axis.
#[test]
fn v5_the_smallest_overlap_a_float_can_hold_counts_on_both_sides_of_every_axis() {
    let smallest = f32::from_bits(1);
    let below_16 = f32::from_bits(16.0_f32.to_bits() - 1);
    assert!(smallest > 0.0 && smallest < 1e-44);
    assert!(below_16 < 16.0 && below_16 > 15.999_99);
    for axis in 0..3 {
        // The low side: from -4 to just inside the block.
        let mut low = [0.0_f32; 3];
        let mut high = [16.0_f32; 3];
        low[axis] = -4.0;
        high[axis] = smallest;
        let entering = loads_in(
            &shape("minecraft:ok", &[(&corner(low), &corner(high))]),
            &base_game(),
        );
        assert_eq!(
            entering.shape.boxes[0].max[axis].to_bits(),
            1,
            "the text reads as the smallest positive float"
        );
        // The high side: from just inside the block to 20.
        let mut low = [0.0_f32; 3];
        let mut high = [16.0_f32; 3];
        low[axis] = below_16;
        high[axis] = 20.0;
        let entering = loads_in(
            &shape("minecraft:ok", &[(&corner(low), &corner(high))]),
            &base_game(),
        );
        assert_eq!(entering.shape.boxes[0].min[axis], below_16);
    }
}
