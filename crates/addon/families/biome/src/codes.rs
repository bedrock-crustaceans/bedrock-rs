//! The biome family's codes: vanilla's content-log texts for biome files, byte for byte, each a
//! definition beside the rules that raise it; findings of the crate's own raise the core's generic
//! definitions. Each id is `biome/<name>`, the name its constant's in lower case.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

// ---- the load ----

/// A document in `[1.21.110, 1.21.120)` that the strict load of the whole document rejects; the
/// finding carries the sentence again as its `DiagnosticDetail::Syntax`.
pub static INVALID_BIOME_FILE: CodeDef<0> = CodeDef::new(
    "biome/invalid_biome_file",
    Kind::Syntax,
    Class::Error,
    "invalid biome file",
)
.game("Invalid Biome file");

/// The world gate of a non-base-game pack.
pub static WORLD_GATE: CodeDef<0> = CodeDef::new(
    "biome/world_gate",
    Kind::Io,
    Class::Error,
    "custom biomes need a world of version 1.21.110 or later",
)
.game("Custom Biomes can only be added for world versions 1.21.110 and up!");

/// A second document with an identifier already loaded.
pub static DUPLICATE: CodeDef<1> = CodeDef::new(
    "biome/duplicate",
    Kind::Duplicate,
    Class::Error,
    "another biome document has the identifier {}",
)
.game("Multiple biome documents with same identifier: {}");

// ---- the identifier and the components ----

/// The identifier's regex constraint.
pub static IDENTIFIER: CodeDef<0> = CodeDef::new(
    "biome/identifier",
    Kind::Identifier,
    Class::Error,
    "the biome identifier has an upper case letter or no namespace",
)
.game("identifier has an upper case letter or does not specify any namespace");

/// `minecraft:climate` validation.
pub static SNOW_ACCUMULATION: CodeDef<0> = CodeDef::new(
    "biome/snow_accumulation",
    Kind::InvalidValue,
    Class::Error,
    "snow_accumulation minimum above its maximum",
)
.game("Incorrect values for snow_accumulation - first element (minimum) cannot be larger than second element (maximum)");

/// `minecraft:overworld_height` validation.
pub static BOTH_NOISE: CodeDef<0> = CodeDef::new(
    "biome/both_noise",
    Kind::InvalidValue,
    Class::Error,
    "both noise_type and noise_params",
)
.game("Cannot specify both noise_type and noise_params");

/// `minecraft:overworld_generation_rules` validation, a malformed transformation entry.
pub static WEIGHTED_BIOME: CodeDef<0> = CodeDef::new(
    "biome/weighted_biome",
    Kind::InvalidValue,
    Class::Error,
    "a malformed weighted biome entry",
)
.game("Weighted biome name entries must be arrays with 2 elements, the first being a string name and the second being a positive weight integer");

/// `minecraft:overworld_generation_rules` validation, a malformed climate entry.
pub static WEIGHTED_CLIMATE: CodeDef<0> = CodeDef::new(
    "biome/weighted_climate",
    Kind::InvalidValue,
    Class::Error,
    "a malformed weighted climate entry",
)
.game("Weighted temperature categories must be arrays with 2 elements, the first being a string temperature category and the second being a positive weight integer");

/// `minecraft:replace_biomes` validation: replacements in two dimensions (a warning).
pub static MULTIPLE_DIMENSIONS: CodeDef<0> = CodeDef::new(
    "biome/multiple_dimensions",
    Kind::InvalidValue,
    Class::Warning,
    "replacements in several dimensions",
)
.game("Replacement present in multiple Dimensions");

/// `minecraft:surface_material_adjustments` validation: a constant `height_range` whose upper
/// bound is not above the lower one.
pub static HEIGHT_RANGE_BOUNDS: CodeDef<0> = CodeDef::new(
    "biome/height_range_bounds",
    Kind::InvalidValue,
    Class::Error,
    "height_range upper bound not above its lower bound",
)
.game("Incorrect values for height_range bounds - upper bound (second value) should be strictly greater than lower bound (first value)");

/// The same for `noise_range`.
pub static NOISE_RANGE_BOUNDS: CodeDef<0> = CodeDef::new(
    "biome/noise_range_bounds",
    Kind::InvalidValue,
    Class::Error,
    "noise_range upper bound not above its lower bound",
)
.game("Incorrect values for noise_range bounds - upper bound (second value) should be strictly greater than lower bound (first value)");

/// A `height_range` expression that fails to compile.
pub static EXPRESSION: CodeDef<1> = CodeDef::new(
    "biome/expression",
    Kind::InvalidValue,
    Class::Error,
    "the height_range expression {} does not parse",
)
.game("Expression Text: '{}', failed to parse.");

/// A nether replacement with a builder the Nether does not support.
pub static NETHER_BUILDERS: CodeDef<0> = CodeDef::new(
    "biome/nether_builders",
    Kind::InvalidValue,
    Class::Error,
    "a surface builder the Nether does not take",
)
.game("Biomes in Nether are only compatible with following Surface Builders: overworld, frozen_ocean, capped, the_end");

/// A nether replacement with a subsurface builder (a warning).
pub static NO_EFFECT_IN_NETHER: CodeDef<1> = CodeDef::new(
    "biome/no_effect_in_nether",
    Kind::NeverApplies,
    Class::Warning,
    "{} has no effect on Nether biomes",
)
.game("{} will have no effect on Biomes in the Nether");

/// `minecraft:multinoise_generation_rules` outside the `minecraft` namespace: the component is
/// skipped and the biome registered.
pub static PRE_CAVES: CodeDef<2> = CodeDef::new(
    "biome/pre_caves",
    Kind::InvalidValue,
    Class::KeptError,
    "the biome {} has the pre-caves-and-cliffs component {}",
)
.game("Biome named: {} contains {} component which is a pre-caves and cliffs component and is unused for custom biomes");

/// Overlapping noise-gradient ranges.
pub static OVERLAP: CodeDef<0> = CodeDef::new(
    "biome/overlap",
    Kind::InvalidValue,
    Class::Error,
    "overlapping noise gradient ranges",
)
.game("The provided NoiseBlockSpecifiers have overlapping ranges. They should be specified as closed intervals [a, b] with optionally overlapping endpoints.");

/// Any failure of the surface builder's validation, after the specific text.
pub static SURFACE_BUILDER_UNRESOLVED: CodeDef<0> = CodeDef::new(
    "biome/surface_builder_unresolved",
    Kind::InvalidValue,
    Class::Error,
    "the surface builder does not resolve",
)
.game("The SurfaceBuilderComponent could not be resolved.");

/// A value without `namespace:`.
pub static NAMESPACE_MISSING: CodeDef<0> = CodeDef::new(
    "biome/namespace_missing",
    Kind::Identifier,
    Class::Error,
    "the value has no namespace",
)
.game("string must be prefixed with a namespace (eg. namespace:value)");

/// A disallowed namespace.
pub static NAMESPACE_INVALID: CodeDef<1> = CodeDef::new(
    "biome/namespace_invalid",
    Kind::Identifier,
    Class::Error,
    "the value has the namespace {}",
)
.game("string has invalid namespace ({})");

// ---- minecraft:replace_biomes ----

/// A document older than 1.26.0: no replacement.
pub static EMPTY_REPLACEMENTS_1_21_80: CodeDef<0> = CodeDef::new(
    "biome/empty_replacements_1_21_80",
    Kind::InvalidValue,
    Class::Error,
    "no replacement",
)
.game("expected array with at least 1 element");

/// A document of 1.26.0 or later: no replacement.
pub static EMPTY_REPLACEMENTS_1_26_0: CodeDef<0> = CodeDef::new(
    "biome/empty_replacements_1_26_0",
    Kind::InvalidValue,
    Class::Error,
    "no replacement",
)
.game("Expected array with at least 1 element");

/// A document older than 1.26.0: a dimension other than the overworld; the arguments are the
/// overworld and the dimension.
pub static DIMENSION_1_21_80: CodeDef<2> = CodeDef::new(
    "biome/dimension_1_21_80",
    Kind::InvalidValue,
    Class::Error,
    "the dimension must be {} but is {}",
)
.game("Dimension must be '{}' but is '{}'");

/// A document of 1.26.0 or later: a dimension other than the overworld and the Nether.
pub static DIMENSION_1_26_0: CodeDef<3> = CodeDef::new(
    "biome/dimension_1_26_0",
    Kind::InvalidValue,
    Class::Error,
    "the dimension must be {} or {} but is {}",
)
.game("Dimension must be either '{}' or '{}' but is '{}'");

/// Both constraints: a repeated `(dimension, target)` pair.
pub static DUPLICATE_PAIR: CodeDef<2> = CodeDef::new(
    "biome/duplicate_pair",
    Kind::InvalidValue,
    Class::Error,
    "the dimension and target pair {}, {} is given twice",
)
.game("Dimension and target biome pairing already defined: '{}', '{}'");

/// Every definition of this table, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    INVALID_BIOME_FILE.meta(),
    WORLD_GATE.meta(),
    DUPLICATE.meta(),
    IDENTIFIER.meta(),
    SNOW_ACCUMULATION.meta(),
    BOTH_NOISE.meta(),
    WEIGHTED_BIOME.meta(),
    WEIGHTED_CLIMATE.meta(),
    MULTIPLE_DIMENSIONS.meta(),
    HEIGHT_RANGE_BOUNDS.meta(),
    NOISE_RANGE_BOUNDS.meta(),
    EXPRESSION.meta(),
    NETHER_BUILDERS.meta(),
    NO_EFFECT_IN_NETHER.meta(),
    PRE_CAVES.meta(),
    OVERLAP.meta(),
    SURFACE_BUILDER_UNRESOLVED.meta(),
    NAMESPACE_MISSING.meta(),
    NAMESPACE_INVALID.meta(),
    EMPTY_REPLACEMENTS_1_21_80.meta(),
    EMPTY_REPLACEMENTS_1_26_0.meta(),
    DIMENSION_1_21_80.meta(),
    DIMENSION_1_26_0.meta(),
    DUPLICATE_PAIR.meta(),
];
