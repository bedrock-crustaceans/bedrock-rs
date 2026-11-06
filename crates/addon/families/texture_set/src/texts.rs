//! Vanilla's content-log texts of the texture sets. `{}` is where `VanillaText::Formatted` puts
//! each argument.

/// A texture set file that is empty: the game's developer assert.
pub(super) const SET_EMPTY: &str =
    "load function reported OK, but storage is empty - is the file empty? Something went wrong.";
/// A texture set that is no JSON; the argument is the file's path.
pub(super) const SET_NO_JSON: &str = "Failed to parse texture set JSON file ({})";
/// A texture set without a readable version; the argument is the file's path.
pub(super) const SET_BAD_VERSION: &str = "Invalid 'format_version' in JSON file ({})";
/// A texture set of a version that has no schema; the version, the file's path.
pub(super) const SET_UNSUPPORTED_VERSION: &str =
    "Unsupported version value in \"format_version\" field: {} ({})";
/// The line after a failed schema pass.
pub(super) const SET_SCHEMA_FAILED: &str = "Failed to validate schema";
/// The line after a failed post-validation (the trailing space is vanilla's).
pub(super) const SET_POST_FAILED: &str = "Failed to post-validate schema ";
/// A required member is absent; the argument is its name.
pub(super) const SET_REQUIRED_CHILD: &str =
    bedrock_addon_core::diagnostic::cereal::REQUIRED_CHILD_TEXT;
/// A hex colour of a four-channel layer that is not `#` and eight digits.
pub(super) const SET_HEX_8: &str = "Color Value was not in the correct format. Please use a hex value with 8 characters. EX. #FFFFFFFF";
/// A hex colour of a three-channel layer that is not `#` and six digits.
pub(super) const SET_HEX_6: &str = "Color Value was not in the correct format as Alpha is ignored for these colors. Please use a hex value with 6 characters. EX. #FFFFFF";
/// A layer that is `null`.
pub(super) const SET_MALFORMED_LAYER: &str =
    "TextureSet JSON layer information is malformed - please check the contents";
/// A texture set without `color`.
pub(super) const SET_NO_COLOR: &str = "There must at least be a color layer in any TextureSet";
/// The composite layer beside all three single layers.
pub(super) const SET_MER_AND_SINGLE: &str = "TextureSet contains composite layer `metalness_emissive_roughness` AND single layers of `metalness`, `emissive` and `roughness` - in this case, the layer `metalness_emissive_roughness` is unnecessary and should be removed";
/// Both composite layers.
pub(super) const SET_MER_AND_MERS: &str = "TextureSet contains both `metalness_emissive_roughness` AND `metalness_emissive_roughness_subsurface` layers - this is not supported, and one should be removed";
/// Both `heightmap` and `normal`.
pub(super) const SET_HEIGHTMAP_AND_NORMAL: &str = "TextureSet contains both `heightmap` AND `normal` layers - this is not currently supported, and one should be removed";
/// A colour that is no array of three or more and no string.
pub(super) const BAD_COLOR: &str = "Expected [ r, g, b ] or [r, g, b, a] or \"#RRGGBB\".";
/// A colour component that is no integer.
pub(super) const NOT_INTEGRAL: &str = "Expected Integral Number.";
/// A layer's image is not in the texture set's pack.
pub(super) const SET_NO_IMAGE: &str = "Specified image could not be found";

/// Every template and the number of arguments its call site gives it, for the arity test.
#[cfg(test)]
const TEMPLATES: &[(&str, usize)] = &[
    (SET_EMPTY, 0),
    (SET_NO_JSON, 1),
    (SET_BAD_VERSION, 1),
    (SET_UNSUPPORTED_VERSION, 2),
    (SET_SCHEMA_FAILED, 0),
    (SET_POST_FAILED, 0),
    (SET_REQUIRED_CHILD, 1),
    (SET_HEX_8, 0),
    (SET_HEX_6, 0),
    (SET_MALFORMED_LAYER, 0),
    (SET_NO_COLOR, 0),
    (SET_MER_AND_SINGLE, 0),
    (SET_MER_AND_MERS, 0),
    (SET_HEIGHTMAP_AND_NORMAL, 0),
    (SET_NO_IMAGE, 0),
    (BAD_COLOR, 0),
    (NOT_INTEGRAL, 0),
];

#[cfg(test)]
mod tests {
    #[test]
    fn every_template_has_as_many_holes_as_its_call_site_has_arguments() {
        bedrock_addon_core::diagnostic::texts::assert_arity(super::TEMPLATES);
    }
}
