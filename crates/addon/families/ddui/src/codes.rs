//! The data-driven UI family's codes: the client's lines for `DataDrivenUI` files, byte for byte,
//! each a definition beside the loader, reader or validator that raises it. Each id is
//! `ddui/<name>`, the name its constant's in lower case.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

// ---- the loader ----

/// A file the host cannot read; the argument is its path. Core's collection walk raises it
/// with this template ([`CodeDef::template`]); the file is skipped.
pub static UNREAD: CodeDef<1> = CodeDef::new(
    "ddui/unread",
    Kind::Io,
    Class::Error,
    "cannot read the DataDrivenUI file {}",
)
.game("Failed to load the contents of DataDrivenUI file {}; it will be skipped.");

/// The loader's "Missing object" error; the argument is the family's payload key.
pub static MISSING_OBJECT: CodeDef<1> = CodeDef::new(
    "ddui/missing_object",
    Kind::Envelope,
    Class::Error,
    "no payload object {}",
)
.game("Failed to parse DataDrivenUI file. Missing object \"{}\"");

/// The loader's closing line.
pub static FAILED_TO_LOAD: CodeDef<0> = CodeDef::new(
    "ddui/failed_to_load",
    Kind::Closing,
    Class::Error,
    "the DataDrivenUI file failed to load",
)
.game("Failed to load DataDrivenUI file");

/// A `format_version` that is no version: told before the "Missing object" error, as an
/// information.
pub static MALFORMED_VERSION: CodeDef<0> = CodeDef::new(
    "ddui/malformed_version",
    Kind::FormatVersion,
    Class::Lint,
    "the format_version is no version",
)
.game("Malformed format version");

/// No version in the input and none from the caller.
pub static NO_VALID_VERSION: CodeDef<0> = CodeDef::new(
    "ddui/no_valid_version",
    Kind::UnsupportedVersion,
    Class::Error,
    "no valid format_version",
)
.game("no valid version passed by the caller or found in the input");

/// A composition's extension point no stored root declares; the argument is its name.
pub static NOT_DEFINED: CodeDef<1> = CodeDef::new(
    "ddui/not_defined",
    Kind::UnresolvedReference,
    Class::Error,
    "no extension point {}",
)
.game("extension_point '{}' is not defined.");

// ---- the reader ----

/// An identifier that is not `namespace:name`.
pub static NOT_NAMESPACED: CodeDef<0> = CodeDef::new(
    "ddui/not_namespaced",
    Kind::InvalidValue,
    Class::Error,
    "the identifier has no namespace",
)
.game("string must be prefixed with a namespace (eg. namespace:value)");

/// A number no double holds: the client logs rapidjson's "number too big" message.
pub static NUMBER_TOO_BIG: CodeDef<0> = CodeDef::new(
    "ddui/number_too_big",
    Kind::InvalidValue,
    Class::Error,
    "a number no double holds",
)
.game("Failed to parse the Minecraft Document JSON: Number too big to be stored in double.");

/// A padding that is no two-value box size.
pub static BOX_SIZE_2: CodeDef<0> = CodeDef::new(
    "ddui/box_size_2",
    Kind::InvalidValue,
    Class::Error,
    "not a two-value box size",
)
.game("Expected [top, right].");

/// A padding that is no four-value box size.
pub static BOX_SIZE_4: CodeDef<0> = CodeDef::new(
    "ddui/box_size_4",
    Kind::InvalidValue,
    Class::Error,
    "not a four-value box size",
)
.game("Expected [top, right, bottom, left].");

/// An RGBA colour that is neither four 0-255 values nor a hex string.
pub static COLOR_RGBA: CodeDef<0> = CodeDef::new(
    "ddui/color_rgba",
    Kind::InvalidValue,
    Class::Error,
    "not an ARGB colour",
)
.game("Expected [r, g, b, a], with values in the 0-255 range, or a hex value with 8 characters (e.g. ARGB as #FFFFFFFF).");

/// An RGB colour that is neither three 0-255 values nor a hex string.
pub static COLOR_RGB: CodeDef<0> = CodeDef::new(
    "ddui/color_rgb",
    Kind::InvalidValue,
    Class::Error,
    "not an RGB colour",
)
.game("Expected [r, g, b], with values in the 0-255 range, or a hex value with 6 characters (e.g. RGB as #FFFFFF).");

// ---- the validators ----

/// A document of a pack the game ships that lacks the vanilla namespace.
pub static BUILTIN_NAMESPACE: CodeDef<0> = CodeDef::new(
    "ddui/builtin_namespace",
    Kind::Identifier,
    Class::Error,
    "a built-in DataDrivenUI document without the minecraft namespace",
)
.game("builtin packs must use the 'minecraft' namespace for DataDrivenUI documents");

/// A document of any other pack that has the vanilla namespace.
pub static EXTERNAL_NAMESPACE: CodeDef<0> = CodeDef::new(
    "ddui/external_namespace",
    Kind::Identifier,
    Class::Error,
    "an add-on DataDrivenUI document with the minecraft namespace",
)
.game("external packs must not use the 'minecraft' namespace for DataDrivenUI documents");

/// An extension point written by an external pack (vanilla's spelling).
pub static EXTENSION_POINT_OVERRIDDEN: CodeDef<0> = CodeDef::new(
    "ddui/extension_point_overridden",
    Kind::InvalidValue,
    Class::Error,
    "an extension point in an add-on pack",
)
.game("extension_point cannot be overriden at external packs.");

/// Every definition of this table, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    UNREAD.meta(),
    MISSING_OBJECT.meta(),
    FAILED_TO_LOAD.meta(),
    MALFORMED_VERSION.meta(),
    NO_VALID_VERSION.meta(),
    NOT_DEFINED.meta(),
    NOT_NAMESPACED.meta(),
    NUMBER_TOO_BIG.meta(),
    BOX_SIZE_2.meta(),
    BOX_SIZE_4.meta(),
    COLOR_RGBA.meta(),
    COLOR_RGB.meta(),
    BUILTIN_NAMESPACE.meta(),
    EXTERNAL_NAMESPACE.meta(),
    EXTENSION_POINT_OVERRIDDEN.meta(),
];
