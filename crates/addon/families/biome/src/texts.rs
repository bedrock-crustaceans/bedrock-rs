//! The biome family's vanilla content-log texts (`VanillaText::Fixed`, and the templates of
//! `VanillaText::Formatted`, whose `{}` stand for vanilla's `%s` / `{}`). Every text here is
//! vanilla's, quoted as the game prints it; findings of the crate's own carry no text.

use bedrock_addon_core::diagnostic::VanillaText;

/// A document in `[1.21.110, 1.21.120)` that the strict load of the whole document rejects.
pub(crate) const INVALID_BIOME_FILE: &str = "Invalid Biome file";
/// The identifier's regex constraint.
pub(crate) const IDENTIFIER: &str =
    "identifier has an upper case letter or does not specify any namespace";
/// The world gate of a non-base-game pack.
pub(crate) const WORLD_GATE: &str =
    "Custom Biomes can only be added for world versions 1.21.110 and up!";
/// A second document with an identifier already loaded.
pub(crate) const DUPLICATE: &str = "Multiple biome documents with same identifier: {}";
/// `minecraft:climate` validation.
pub(crate) const SNOW_ACCUMULATION: &str = "Incorrect values for snow_accumulation - first element (minimum) cannot be larger than second element (maximum)";
/// `minecraft:overworld_height` validation.
pub(crate) const BOTH_NOISE: &str = "Cannot specify both noise_type and noise_params";
/// `minecraft:overworld_generation_rules` validation, a malformed transformation entry.
pub(crate) const WEIGHTED_BIOME: &str = "Weighted biome name entries must be arrays with 2 elements, the first being a string name and the second being a positive weight integer";
/// `minecraft:overworld_generation_rules` validation, a malformed climate entry.
pub(crate) const WEIGHTED_CLIMATE: &str = "Weighted temperature categories must be arrays with 2 elements, the first being a string temperature category and the second being a positive weight integer";
/// `minecraft:replace_biomes` validation: replacements in two dimensions (a warning).
pub(crate) const MULTIPLE_DIMENSIONS: &str = "Replacement present in multiple Dimensions";
/// `minecraft:surface_material_adjustments` validation: a constant `height_range` whose upper
/// bound is not above the lower one.
pub(crate) const HEIGHT_RANGE_BOUNDS: &str = "Incorrect values for height_range bounds - upper bound (second value) should be strictly greater than lower bound (first value)";
/// The same for `noise_range`.
pub(crate) const NOISE_RANGE_BOUNDS: &str = "Incorrect values for noise_range bounds - upper bound (second value) should be strictly greater than lower bound (first value)";
/// A `height_range` expression that fails to compile.
#[cfg(feature = "molang")]
pub(crate) const EXPRESSION: &str = "Expression Text: '{}', failed to parse.";
/// A nether replacement with a builder the Nether does not support.
pub(crate) const NETHER_BUILDERS: &str = "Biomes in Nether are only compatible with following Surface Builders: overworld, frozen_ocean, capped, the_end";
/// A nether replacement with a subsurface builder (a warning).
pub(crate) const NO_EFFECT_IN_NETHER: &str = "{} will have no effect on Biomes in the Nether";
/// `minecraft:multinoise_generation_rules` outside the `minecraft` namespace.
pub(crate) const PRE_CAVES: &str = "Biome named: {} contains {} component which is a pre-caves and cliffs component and is unused for custom biomes";
/// Overlapping noise-gradient ranges.
pub(crate) const OVERLAP: &str = "The provided NoiseBlockSpecifiers have overlapping ranges. They should be specified as closed intervals [a, b] with optionally overlapping endpoints.";
/// Any failure of the surface builder's validation, after the specific text.
pub(crate) const SURFACE_BUILDER_UNRESOLVED: &str =
    "The SurfaceBuilderComponent could not be resolved.";
/// A value without `namespace:`.
pub(crate) const NAMESPACE_MISSING: &str =
    "string must be prefixed with a namespace (eg. namespace:value)";
/// A disallowed namespace.
pub(crate) const NAMESPACE_INVALID: &str = "string has invalid namespace ({})";
/// A document older than 1.26.0: no replacement.
pub(crate) const EMPTY_REPLACEMENTS_1_21_80: &str = "expected array with at least 1 element";
/// A document of 1.26.0 or later: no replacement.
pub(crate) const EMPTY_REPLACEMENTS_1_26_0: &str = "Expected array with at least 1 element";
/// A document older than 1.26.0: a dimension other than the overworld.
pub(crate) const DIMENSION_1_21_80: &str = "Dimension must be '{}' but is '{}'";
/// A document of 1.26.0 or later: a dimension other than the overworld and the Nether.
pub(crate) const DIMENSION_1_26_0: &str = "Dimension must be either '{}' or '{}' but is '{}'";
/// Both constraints: a repeated `(dimension, target)` pair.
pub(crate) const DUPLICATE_PAIR: &str =
    "Dimension and target biome pairing already defined: '{}', '{}'";

/// A fixed text.
#[allow(
    clippy::unnecessary_wraps,
    reason = "every caller fills an `Option<VanillaText>` field"
)]
pub(crate) fn fixed(text: &'static str) -> Option<VanillaText> {
    Some(VanillaText::Fixed(text))
}

/// A template with its arguments.
#[allow(
    clippy::unnecessary_wraps,
    reason = "every caller fills an `Option<VanillaText>` field"
)]
pub(crate) fn text(template: &'static str, args: &[&str]) -> Option<VanillaText> {
    Some(VanillaText::Formatted {
        template,
        args: args.iter().map(|a| (*a).to_owned()).collect(),
    })
}
