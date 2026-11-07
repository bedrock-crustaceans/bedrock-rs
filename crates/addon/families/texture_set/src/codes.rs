//! The `texture_set` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each [`was`](CodeDef::was) the legacy code its sites raised,
//! so a pin or a filter on the code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// A texture set file that is empty: the game's developer assert.
pub static SET_EMPTY: CodeDef<0> = CodeDef::new(
    "texture_set/set_empty",
    Kind::Syntax,
    Class::Lint,
    "empty document",
)
.game("load function reported OK, but storage is empty - is the file empty? Something went wrong.")
.was("EmptyDocument");

/// A texture set that is no JSON; the argument is the file's path.
pub static SET_NO_JSON: CodeDef<1> = CodeDef::new(
    "texture_set/set_no_json",
    Kind::Syntax,
    Class::Warning,
    "malformed input",
)
.game("Failed to parse texture set JSON file ({})")
.was("MalformedInput");

/// A texture set without a readable version; the argument is the file's path.
pub static SET_BAD_VERSION: CodeDef<1> = CodeDef::new(
    "texture_set/set_bad_version",
    Kind::FormatVersion,
    Class::Warning,
    "Malformed format version",
)
.game("Invalid 'format_version' in JSON file ({})")
.was("MalformedFormatVersion");

/// A texture set of a version that has no schema; the version, the file's path.
pub static SET_UNSUPPORTED_VERSION: CodeDef<2> = CodeDef::new(
    "texture_set/set_unsupported_version",
    Kind::UnsupportedVersion,
    Class::Warning,
    "unsupported format_version",
)
.game("Unsupported version value in \"format_version\" field: {} ({})")
.was("UnsupportedVersion");

/// The line after a failed schema pass.
pub static SET_SCHEMA_FAILED: CodeDef<0> = CodeDef::new(
    "texture_set/set_schema_failed",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("Failed to validate schema")
.was("InvalidValue");

/// The line after a failed post-validation (the trailing space is vanilla's).
pub static SET_POST_FAILED: CodeDef<0> = CodeDef::new(
    "texture_set/set_post_failed",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("Failed to post-validate schema ")
.was("InvalidValue");

/// A required member is absent; the argument is its name.
pub static SET_REQUIRED_CHILD: CodeDef<1> = CodeDef::new(
    "texture_set/set_required_child",
    Kind::Envelope,
    Class::Error,
    "the document has no payload member",
)
.game(bedrock_addon_core::diagnostic::cereal::REQUIRED_CHILD_TEXT)
.was("MissingPayload");

/// A hex colour of a four-channel layer that is not `#` and eight digits.
pub static SET_HEX_8: CodeDef<0> = CodeDef::new(
    "texture_set/set_hex_8",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("Color Value was not in the correct format. Please use a hex value with 8 characters. EX. #FFFFFFFF")
.was("InvalidValue");

/// A hex colour of a three-channel layer that is not `#` and six digits.
pub static SET_HEX_6: CodeDef<0> = CodeDef::new(
    "texture_set/set_hex_6",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("Color Value was not in the correct format as Alpha is ignored for these colors. Please use a hex value with 6 characters. EX. #FFFFFF")
.was("InvalidValue");

/// A layer that is `null`.
pub static SET_MALFORMED_LAYER: CodeDef<0> = CodeDef::new(
    "texture_set/set_malformed_layer",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("TextureSet JSON layer information is malformed - please check the contents")
.was("InvalidValue");

/// A texture set without `color`.
pub static SET_NO_COLOR: CodeDef<0> = CodeDef::new(
    "texture_set/set_no_color",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("There must at least be a color layer in any TextureSet")
.was("InvalidValue");

/// The composite layer beside all three single layers.
pub static SET_MER_AND_SINGLE: CodeDef<0> = CodeDef::new(
    "texture_set/set_mer_and_single",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("TextureSet contains composite layer `metalness_emissive_roughness` AND single layers of `metalness`, `emissive` and `roughness` - in this case, the layer `metalness_emissive_roughness` is unnecessary and should be removed")
.was("InvalidValue");

/// Both composite layers.
pub static SET_MER_AND_MERS: CodeDef<0> = CodeDef::new(
    "texture_set/set_mer_and_mers",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("TextureSet contains both `metalness_emissive_roughness` AND `metalness_emissive_roughness_subsurface` layers - this is not supported, and one should be removed")
.was("InvalidValue");

/// Both `heightmap` and `normal`.
pub static SET_HEIGHTMAP_AND_NORMAL: CodeDef<0> = CodeDef::new(
    "texture_set/set_heightmap_and_normal",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("TextureSet contains both `heightmap` AND `normal` layers - this is not currently supported, and one should be removed")
.was("InvalidValue");

/// A colour that is no array of three or more and no string.
pub static BAD_COLOR: CodeDef<0> = CodeDef::new(
    "texture_set/bad_color",
    Kind::WrongType,
    Class::Error,
    "wrong JSON type",
)
.game("Expected [ r, g, b ] or [r, g, b, a] or \"#RRGGBB\".")
.was("InvalidType");

/// A colour component that is no integer.
pub static NOT_INTEGRAL: CodeDef<0> = CodeDef::new(
    "texture_set/not_integral",
    Kind::InvalidValue,
    Class::KeptError,
    "invalid value",
)
.game("Expected Integral Number.")
.was("InvalidValue");

/// A layer's image is not in the texture set's pack.
pub static SET_NO_IMAGE: CodeDef<0> = CodeDef::new(
    "texture_set/set_no_image",
    Kind::UnresolvedReference,
    Class::Lint,
    "texture not found",
)
.game("Specified image could not be found")
.was("TextureNotFound");

/// Every definition of this module, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    SET_EMPTY.meta(),
    SET_NO_JSON.meta(),
    SET_BAD_VERSION.meta(),
    SET_UNSUPPORTED_VERSION.meta(),
    SET_SCHEMA_FAILED.meta(),
    SET_POST_FAILED.meta(),
    SET_REQUIRED_CHILD.meta(),
    SET_HEX_8.meta(),
    SET_HEX_6.meta(),
    SET_MALFORMED_LAYER.meta(),
    SET_NO_COLOR.meta(),
    SET_MER_AND_SINGLE.meta(),
    SET_MER_AND_MERS.meta(),
    SET_HEIGHTMAP_AND_NORMAL.meta(),
    BAD_COLOR.meta(),
    NOT_INTEGRAL.meta(),
    SET_NO_IMAGE.meta(),
];
