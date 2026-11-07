//! The `texture_atlas` family's codes: the game's content-log lines, byte for byte, each a
//! definition beside the rules that raise it. Each id is `texture_atlas/<name>`, the name its
//! constant's in lower case.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// The merged atlas document has no `texture_name`; the argument is the file's path.
pub static NO_TEXTURE_NAME: CodeDef<1> = CodeDef::new(
    "texture_atlas/no_texture_name",
    Kind::MissingRequired,
    Class::Error,
    "missing required field",
)
.game("failed to read texture_name in atlas file {}");

/// The debug assert behind an empty `texture_name`.
pub static NO_TEXTURE_PAGE: CodeDef<0> = CodeDef::new(
    "texture_atlas/no_texture_page",
    Kind::InvalidValue,
    Class::Lint,
    "invalid value",
)
.game("Atlas textures must contain at least one defined texture page");

/// A `texture_data` value that is no object; the argument is its key.
pub static ENTRY_NOT_OBJECT: CodeDef<1> = CodeDef::new(
    "texture_atlas/entry_not_object",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("Expecting \"{}\" to be a JSON object.");

/// A tile without `textures`; the argument is its key.
pub static MISSING_TEXTURES: CodeDef<1> = CodeDef::new(
    "texture_atlas/missing_textures",
    Kind::MissingRequired,
    Class::Error,
    "missing required field",
)
.game("Missing textures \"{}\".");

/// A colour that is no array of three or more and no string.
pub static BAD_COLOR: CodeDef<0> = CodeDef::new(
    "texture_atlas/bad_color",
    Kind::InvalidValue,
    Class::KeptError,
    "invalid value",
)
.game("Expected [ r, g, b ] or [r, g, b, a] or \"#RRGGBB\".");

/// A colour component that is no integer.
pub static NOT_INTEGRAL: CodeDef<0> = CodeDef::new(
    "texture_atlas/not_integral",
    Kind::InvalidValue,
    Class::KeptError,
    "invalid value",
)
.game("Expected Integral Number.");

/// A layer weight below the minimum; the argument is the minimum.
pub static WEIGHT_LOW: CodeDef<1> = CodeDef::new(
    "texture_atlas/weight_low",
    Kind::Clamped,
    Class::Warning,
    "value clamped into its range",
)
.game("Variations texture weight cannot be lower than {}");

/// A layer weight above the maximum; the argument is the maximum.
pub static WEIGHT_HIGH: CodeDef<1> = CodeDef::new(
    "texture_atlas/weight_high",
    Kind::Clamped,
    Class::Warning,
    "value clamped into its range",
)
.game("Variations texture weight cannot be higher than {}");

/// Every definition of this module, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    NO_TEXTURE_NAME.meta(),
    NO_TEXTURE_PAGE.meta(),
    ENTRY_NOT_OBJECT.meta(),
    MISSING_TEXTURES.meta(),
    BAD_COLOR.meta(),
    NOT_INTEGRAL.meta(),
    WEIGHT_LOW.meta(),
    WEIGHT_HIGH.meta(),
];
