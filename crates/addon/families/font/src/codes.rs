//! The `font` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each [`was`](CodeDef::was) the legacy code its sites raised,
//! so a pin or a filter on the code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// A file that is not JSON.
pub static NOT_JSON: CodeDef<0> = CodeDef::new(
    "font/not_json",
    Kind::Syntax,
    Class::Lint,
    "malformed input",
)
.game("Couldn't parse font_metadata.json")
.was("MalformedInput");

/// A `version` that is not the integer 1.
pub static VERSION: CodeDef<0> = CodeDef::new(
    "font/version",
    Kind::UnsupportedVersion,
    Class::Lint,
    "unsupported format_version",
)
.game("Unsupported version number.")
.was("UnsupportedVersion");

/// `MissingRequiredField`: the game's line `Font is missing font_format.`.
pub static NO_FORMAT: CodeDef<0> = CodeDef::new(
    "font/no_format",
    Kind::MissingRequired,
    Class::Lint,
    "missing required field",
)
.game("Font is missing font_format.")
.was("MissingRequiredField");

/// `MissingRequiredField`: the game's line `Font is missing font_name.`.
pub static NO_NAME: CodeDef<0> = CodeDef::new(
    "font/no_name",
    Kind::MissingRequired,
    Class::Lint,
    "missing required field",
)
.game("Font is missing font_name.")
.was("MissingRequiredField");

/// `MissingRequiredField`: the game's line `Bitmap font must have either an ascii_font_file or a unicode_file_prefix specified`.
pub static NO_BITMAP_FILE: CodeDef<0> = CodeDef::new(
    "font/no_bitmap_file",
    Kind::MissingRequired,
    Class::Lint,
    "missing required field",
)
.game("Bitmap font must have either an ascii_font_file or a unicode_file_prefix specified")
.was("MissingRequiredField");

/// `MissingRequiredField`: the game's line `TTF is missing font_file.`.
pub static NO_TTF_FILE: CodeDef<0> = CodeDef::new(
    "font/no_ttf_file",
    Kind::MissingRequired,
    Class::Lint,
    "missing required field",
)
.game("TTF is missing font_file.")
.was("MissingRequiredField");

/// `MissingRequiredField`: the game's line `MSDF is missing font_file.`.
pub static NO_MSDF_FILE: CodeDef<0> = CodeDef::new(
    "font/no_msdf_file",
    Kind::MissingRequired,
    Class::Lint,
    "missing required field",
)
.game("MSDF is missing font_file.")
.was("MissingRequiredField");

/// `MissingRequiredField`: the game's line `Font Alias missing alias.`.
pub static NO_ALIAS: CodeDef<0> = CodeDef::new(
    "font/no_alias",
    Kind::MissingRequired,
    Class::Lint,
    "missing required field",
)
.game("Font Alias missing alias.")
.was("MissingRequiredField");

/// `MissingRequiredField`: the game's line `Font alias missing font_reference`.
pub static NO_REFERENCE: CodeDef<0> = CodeDef::new(
    "font/no_reference",
    Kind::MissingRequired,
    Class::Lint,
    "missing required field",
)
.game("Font alias missing font_reference")
.was("MissingRequiredField");

/// `InvalidValue`: the game's line `Invalid format for the generate_pages_on_start json object.`.
pub static PAGES_FORMAT: CodeDef<0> = CodeDef::new(
    "font/pages_format",
    Kind::InvalidValue,
    Class::Lint,
    "invalid value",
)
.game("Invalid format for the generate_pages_on_start json object.")
.was("InvalidValue");

/// `InvalidType`: the game's line `Font Alias property fonts must be an array.`.
pub static ALIAS_FONTS: CodeDef<0> = CodeDef::new(
    "font/alias_fonts",
    Kind::WrongType,
    Class::Lint,
    "wrong JSON type",
)
.game("Font Alias property fonts must be an array.")
.was("InvalidType");

/// `InvalidType`: the game's line `Font alias font_language_code must be a string`.
pub static LANGUAGE_CODE: CodeDef<0> = CodeDef::new(
    "font/language_code",
    Kind::WrongType,
    Class::Lint,
    "wrong JSON type",
)
.game("Font alias font_language_code must be a string")
.was("InvalidType");

/// `InvalidType`: the game's line `Font alias font_ranges must be an array`.
pub static RANGES: CodeDef<0> = CodeDef::new(
    "font/ranges",
    Kind::WrongType,
    Class::Lint,
    "wrong JSON type",
)
.game("Font alias font_ranges must be an array")
.was("InvalidType");

/// `InvalidType`: the game's line `Font alias font_ranges first and last values must be integers.`.
pub static RANGE_BOUNDS: CodeDef<0> = CodeDef::new(
    "font/range_bounds",
    Kind::WrongType,
    Class::Lint,
    "wrong JSON type",
)
.game("Font alias font_ranges first and last values must be integers.")
.was("InvalidType");

/// The game's assert for a metrics file whose header is not 0.
pub static FONTDATA_VERSION: CodeDef<0> = CodeDef::new(
    "font/fontdata_version",
    Kind::Syntax,
    Class::Lint,
    "malformed input",
)
.game("Version of MSDF data file should be 0")
.was("MalformedInput");

/// The game's assert for an emoticon file it cannot use.
pub static MALFORMED: CodeDef<0> = CodeDef::new(
    "font/malformed",
    Kind::Syntax,
    Class::Lint,
    "malformed input",
)
.game("Malformed JSON")
.was("MalformedInput");

/// `FontFileMissing`: the game's line `Failed to load font from resource pack`.
pub static FONT_LOAD_FAILED: CodeDef<0> = CodeDef::new(
    "font/font_load_failed",
    Kind::UnresolvedReference,
    Class::Lint,
    "font file not found",
)
.game("Failed to load font from resource pack")
.was("FontFileMissing");

/// `FontFileMissing`: the game's line `Couldn't load MSDF Font information`.
pub static MSDF_INFO_MISSING: CodeDef<0> = CodeDef::new(
    "font/msdf_info_missing",
    Kind::UnresolvedReference,
    Class::Lint,
    "font file not found",
)
.game("Couldn't load MSDF Font information")
.was("FontFileMissing");

/// Every definition of this module, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    NOT_JSON.meta(),
    VERSION.meta(),
    NO_FORMAT.meta(),
    NO_NAME.meta(),
    NO_BITMAP_FILE.meta(),
    NO_TTF_FILE.meta(),
    NO_MSDF_FILE.meta(),
    NO_ALIAS.meta(),
    NO_REFERENCE.meta(),
    PAGES_FORMAT.meta(),
    ALIAS_FONTS.meta(),
    LANGUAGE_CODE.meta(),
    RANGES.meta(),
    RANGE_BOUNDS.meta(),
    FONTDATA_VERSION.meta(),
    MALFORMED.meta(),
    FONT_LOAD_FAILED.meta(),
    MSDF_INFO_MISSING.meta(),
];
