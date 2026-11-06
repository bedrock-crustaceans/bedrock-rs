//! Vanilla's content-log texts of the flipbooks, as the link rule quotes them. `{}` is where
//! `VanillaText::Formatted` puts each argument.

/// A flipbook's tile is not in the terrain atlas; the argument is the tile.
pub const TILE_MISSING: &str = "Texture {} is missing from atlas textures";
/// A flipbook's index is outside its tile; the tile, the index, the tile's index count.
pub const BAD_ATLAS_INDEX: &str =
    "Texture {} is using an invalid atlas index {} for the expected UV count {}";
/// A flipbook's image is in no pack; the argument is the path.
pub const NO_METADATA: &str = "Texture metadata for texture: {} is invalid";
/// A flipbook image wider than tall (the trailing space is vanilla's).
pub const NO_FRAMES: &str = "Flipbook texture should have at least one frame. ";
/// A frame index outside the image's frames.
pub const FRAME_OUT_OF_RANGE: &str = "frame index out of range!";

/// Every template and the number of arguments its call site gives it, for the arity test.
#[cfg(test)]
const TEMPLATES: &[(&str, usize)] = &[
    (TILE_MISSING, 1),
    (BAD_ATLAS_INDEX, 3),
    (NO_METADATA, 1),
    (NO_FRAMES, 0),
    (FRAME_OUT_OF_RANGE, 0),
];

#[cfg(test)]
mod tests {
    #[test]
    fn every_template_has_as_many_holes_as_its_call_site_has_arguments() {
        bedrock_addon_core::diagnostic::texts::assert_arity(super::TEMPLATES);
    }
}
