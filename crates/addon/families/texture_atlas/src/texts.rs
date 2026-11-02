//! Vanilla's content-log texts of the texture atlases. `{}` is where `VanillaText::Formatted`
//! puts each argument.

/// The merged atlas document has no `texture_name`; the argument is the file's path.
pub(super) const NO_TEXTURE_NAME: &str = "failed to read texture_name in atlas file {}";
/// The debug assert behind an empty `texture_name`.
pub(super) const NO_TEXTURE_PAGE: &str =
    "Atlas textures must contain at least one defined texture page";
/// A `texture_data` value that is no object; the argument is its key.
pub(super) const ENTRY_NOT_OBJECT: &str = "Expecting \"{}\" to be a JSON object.";
/// A tile without `textures`; the argument is its key.
pub(super) const MISSING_TEXTURES: &str = "Missing textures \"{}\".";
/// A colour that is no array of three or more and no string.
pub(super) const BAD_COLOR: &str = "Expected [ r, g, b ] or [r, g, b, a] or \"#RRGGBB\".";
/// A colour component that is no integer.
pub(super) const NOT_INTEGRAL: &str = "Expected Integral Number.";
/// A layer weight below the minimum; the argument is the minimum.
pub(super) const WEIGHT_LOW: &str = "Variations texture weight cannot be lower than {}";
/// A layer weight above the maximum; the argument is the maximum.
pub(super) const WEIGHT_HIGH: &str = "Variations texture weight cannot be higher than {}";

/// Every template and the number of arguments its call site gives it, for the arity test.
#[cfg(test)]
const TEMPLATES: &[(&str, usize)] = &[
    (NO_TEXTURE_NAME, 1),
    (NO_TEXTURE_PAGE, 0),
    (ENTRY_NOT_OBJECT, 1),
    (MISSING_TEXTURES, 1),
    (BAD_COLOR, 0),
    (NOT_INTEGRAL, 0),
    (WEIGHT_LOW, 1),
    (WEIGHT_HIGH, 1),
];

#[cfg(test)]
mod tests {
    #[test]
    fn every_template_has_as_many_holes_as_its_call_site_has_arguments() {
        bedrock_addon_support::texture::texts::assert_arity(super::TEMPLATES);
    }
}
