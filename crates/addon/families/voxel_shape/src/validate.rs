//! The checks the game runs on a shape its schema accepted, in its order, stopping at the
//! first failure: the file is not loaded and one line says why.

use super::{ShapeBox, VoxelShape, codes};
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath, Phase};
use bedrock_addon_core::id::syntax::split_namespace;

/// Runs the namespace, box, bounds, overlap and width checks on `shape` as a file of a base-game
/// pack or of any other; the first failure is the one error pushed to `d`, at a path below `at`
/// (the payload's).
pub(crate) fn rules(shape: &VoxelShape, base_game: bool, at: &JsonPath, d: &mut Diagnostics) {
    if let Some((code, path)) = first_failure(shape, base_game, at) {
        d.push(Diagnostic::of(code).at(path).phase(Phase::Validate));
    }
}

/// The first check `shape` fails: vanilla's text and where.
fn first_failure(shape: &VoxelShape, base_game: bool, at: &JsonPath) -> Option<(Code, JsonPath)> {
    let identifier = shape.description.identifier.as_str();
    // The game raises a developer assert here and then skips the file the same way; the crate
    // reports an error.
    if base_game && !is_vanilla_namespace(identifier) {
        return Some((
            codes::NAMESPACE_REQUIRED.with([identifier]),
            at.with(&["description", "identifier"]),
        ));
    }
    if !base_game && is_vanilla_namespace(identifier) {
        return Some((
            codes::NAMESPACE_NOT_VALID.with([identifier]),
            at.with(&["description", "identifier"]),
        ));
    }
    let boxes = &shape.shape.boxes;
    let all = at.with(&["shape", "boxes"]);
    // Each box in order, at the first bad box: the corners must be ordered, then inside the
    // bounds.
    for (index, one) in boxes.iter().enumerate() {
        let here = all.index(index);
        if one.is_inverted() {
            return Some((
                codes::BOX_INVALID.with([index.to_string(), identifier.to_owned()]),
                here,
            ));
        }
        if one.is_out_of_bounds() {
            return Some((
                codes::BOX_OUT_OF_BOUNDS.with([index.to_string(), identifier.to_owned()]),
                here,
            ));
        }
    }
    // Some box must enter the block itself.
    if !boxes.iter().any(ShapeBox::overlaps_block) {
        return Some((codes::NO_OVERLAP.with([identifier]), all));
    }
    // The whole shape on x, then y, then z, at the first axis over the limit.
    for (axis, name) in ["x", "y", "z"].into_iter().enumerate() {
        let extent = extent(boxes, axis);
        if extent > WIDEST {
            return Some((extent_too_wide(identifier, name, extent), all));
        }
    }
    None
}

/// The span of `boxes` on `axis`: the largest `max` minus the smallest `min`.
fn extent(boxes: &[ShapeBox], axis: usize) -> f32 {
    let highest = boxes
        .iter()
        .map(|one| one.max[axis])
        .fold(f32::NEG_INFINITY, f32::max);
    let lowest = boxes
        .iter()
        .map(|one| one.min[axis])
        .fold(f32::INFINITY, f32::min);
    highest - lowest
}

/// The widest a shape is on an axis, in pixels: a block and 14 more.
const WIDEST: f32 = 30.0;

/// Whether `identifier` has a namespace (the text before its first `:`) that is exactly
/// `minecraft`; a bare name has none.
fn is_vanilla_namespace(identifier: &str) -> bool {
    split_namespace(identifier).is_some_and(|(namespace, _)| namespace == "minecraft")
}

impl ShapeBox {
    /// Whether `min` is not below `max` on some axis (equal is not a box).
    fn is_inverted(&self) -> bool {
        (0..3).any(|axis| self.min[axis] >= self.max[axis])
    }

    /// Whether a `min` coordinate is below -14 or a `max` coordinate above 30.
    fn is_out_of_bounds(&self) -> bool {
        self.min.iter().any(|&min| min < LOWEST) || self.max.iter().any(|&max| max > HIGHEST)
    }

    /// Whether the box shares volume with the block (0 to 16 on every axis); touching a face is
    /// not enough.
    fn overlaps_block(&self) -> bool {
        (0..3).all(|axis| self.max[axis] > 0.0 && self.min[axis] < BLOCK)
    }
}

/// The block's size in pixels.
const BLOCK: f32 = 16.0;

/// The lowest coordinate of a box, in pixels: 14 below the block.
const LOWEST: f32 = -14.0;
/// The highest coordinate of a box, in pixels: 14 above the block.
const HIGHEST: f32 = 30.0;

/// The shape spans `extent` pixels on `axis` (`x`, `y` or `z`), more than 30.
pub(crate) fn extent_too_wide(identifier: &str, axis: &str, extent: f32) -> Code {
    codes::EXTENT_TOO_WIDE.with([
        identifier.to_owned(),
        axis.to_owned(),
        general(extent),
        "30".to_owned(),
    ])
}

/// `x` as vanilla's `{:g}` prints it: six significant digits without trailing zeros. Only for
/// the extents the width check reports, which lie between 30 and 44 (every box passed the
/// bounds check), so the form is
/// always two digits, a point and up to four more.
fn general(x: f32) -> String {
    let fixed = format!("{x:.4}");
    fixed.trim_end_matches('0').trim_end_matches('.').to_owned()
}
