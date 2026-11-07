//! The client-biome family's codes: vanilla's content-log texts for client biome files, byte for
//! byte, each a definition beside the rules that raise it; findings of the crate's own raise the
//! core's generic definitions. Each [`was`](CodeDef::was) the legacy code its sites raised, so a
//! pin or a filter on the code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

// ---- the load ----

/// Any failure of a version struct's strict load; the finding carries the sentence again as its
/// `DiagnosticDetail::Syntax`.
pub static INVALID_FILE: CodeDef<0> = CodeDef::new(
    "client_biome/invalid_file",
    Kind::Syntax,
    Class::Error,
    "invalid client biome file",
)
.game("Invalid Client Biome file")
.was("MalformedInput");

/// The content log after [`INVALID_FILE`]: takes the file. The finding's
/// `DiagnosticDetail::Syntax` is "the document was not loaded".
pub static FAILED_TO_LOAD: CodeDef<1> = CodeDef::new(
    "client_biome/failed_to_load",
    Kind::Syntax,
    Class::Error,
    "the client biome document {} was not loaded",
)
.game("Failed to load client biome document: {}")
.was("MalformedInput");

/// A `format_version` the pipeline found malformed: the game clears the input's version and then
/// finds none. Core's [`NO_VALID_VERSION`](bedrock_addon_core::diagnostic::codes::NO_VALID_VERSION)
/// says the same for an absent version, under another legacy name.
pub static NO_VALID_VERSION: CodeDef<0> = CodeDef::new(
    "client_biome/no_valid_version",
    Kind::FormatVersion,
    Class::Error,
    "no valid format_version",
)
.game("no valid version passed by the caller or found in the input")
.was("MalformedFormatVersion");

/// A version from 1.0.0 up to the floor: takes the file and the version.
pub static CANNOT_LOAD: CodeDef<2> = CodeDef::new(
    "client_biome/cannot_load",
    Kind::UnsupportedVersion,
    Class::Error,
    "{}: no client biome loads with format_version {}",
)
.game("{}: Cannot load client_biome with format_version {}")
.was("UnsupportedVersion");

/// A document below 1.21.130 whose root has no `format_version` (the version was found
/// elsewhere in the text): takes the file.
pub static NO_VERSION_STRING: CodeDef<1> = CodeDef::new(
    "client_biome/no_version_string",
    Kind::MissingRequired,
    Class::Error,
    "{}: no format_version at the root",
)
.game("{}: No format_version string found")
.was("MissingRequiredField");

/// A syntax error in a document below 1.21.130: takes the parser's message, which the finding
/// also carries as its `DiagnosticDetail::Syntax`.
pub static LEGACY_SYNTAX: CodeDef<1> = CodeDef::new(
    "client_biome/legacy_syntax",
    Kind::Syntax,
    Class::Error,
    "the client biome file is no JSON document: {}",
)
.game("failed to parse file into a json document: {}")
.was("MalformedInput");

/// A root that is no object in a document below 1.21.130: takes the file. The finding carries
/// the parser's message as its `DiagnosticDetail::Syntax`.
pub static LEGACY_ROOT: CodeDef<1> = CodeDef::new(
    "client_biome/legacy_root",
    Kind::Syntax,
    Class::Error,
    "the root of the client biome file {} is no JSON object",
)
.game("failed to parse client biome file {} into a json document, root value needs to be a json object")
.was("MalformedInput");

// ---- the identifier ----

/// The identifier's regex below 1.21.90.
pub static IDENTIFIER_UPPER: CodeDef<0> = CodeDef::new(
    "client_biome/identifier_upper",
    Kind::InvalidValue,
    Class::Error,
    "the client biome identifier has an upper case letter",
)
.game("identifier has an upper case letter")
.was("InvalidValue");

/// The identifier's regex from 1.21.90.
pub static IDENTIFIER_NAMESPACE: CodeDef<0> = CodeDef::new(
    "client_biome/identifier_namespace",
    Kind::Identifier,
    Class::Error,
    "the client biome identifier has an upper case letter or no namespace",
)
.game("identifier has an upper case letter or does not specify any namespace")
.was("NamespaceRule");

// ---- the components ----

/// `minecraft:water_appearance` without a member.
pub static WATER_EMPTY: CodeDef<0> = CodeDef::new(
    "client_biome/water_empty",
    Kind::InvalidValue,
    Class::Error,
    "water_appearance has neither surface_color nor surface_opacity",
)
.game("No surface color or opacity specified. Specify one or remove the component.")
.was("InvalidValue");

/// A colour that is neither `"#RRGGBB"` nor three integers.
pub static COLOR: CodeDef<0> = CodeDef::new(
    "client_biome/color",
    Kind::WrongType,
    Class::Error,
    "a colour that is neither #RRGGBB nor three integers",
)
.game("Expected [r, g, b], with values in the 0-255 range, or a hex value with 6 characters (e.g. RGB as #FFFFFF).")
.was("InvalidType");

/// `minecraft:biome_music` check.
pub static VOLUME: CodeDef<0> = CodeDef::new(
    "client_biome/volume",
    Kind::InvalidValue,
    Class::Error,
    "the volume multiplier is outside [0, 1]",
)
.game("Volume multiplier must be between 0 and 1, inclusive.")
.was("InvalidValue");

/// `minecraft:water_appearance` check.
pub static OPACITY: CodeDef<0> = CodeDef::new(
    "client_biome/opacity",
    Kind::InvalidValue,
    Class::Error,
    "surface_opacity is outside [0, 1]",
)
.game("surface_opacity field must be between 0 and 1, inclusive.")
.was("InvalidValue");

/// `minecraft:ambient_sounds` check.
pub static NO_SOUND: CodeDef<0> = CodeDef::new(
    "client_biome/no_sound",
    Kind::InvalidValue,
    Class::Error,
    "ambient_sounds names no sound",
)
.game("No addition, loop, mood, or underwater variant sound specified. Specify at least one or remove the component.")
.was("InvalidValue");

/// `minecraft:precipitation` check.
pub static PRECIPITATION: CodeDef<0> = CodeDef::new(
    "client_biome/precipitation",
    Kind::InvalidValue,
    Class::Error,
    "more than one precipitation type",
)
.game("precipitation: multiple precipitation types set for biome")
.was("InvalidValue");

/// Every definition of this table, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    INVALID_FILE.meta(),
    FAILED_TO_LOAD.meta(),
    NO_VALID_VERSION.meta(),
    CANNOT_LOAD.meta(),
    NO_VERSION_STRING.meta(),
    LEGACY_SYNTAX.meta(),
    LEGACY_ROOT.meta(),
    IDENTIFIER_UPPER.meta(),
    IDENTIFIER_NAMESPACE.meta(),
    WATER_EMPTY.meta(),
    COLOR.meta(),
    VOLUME.meta(),
    OPACITY.meta(),
    NO_SOUND.meta(),
    PRECIPITATION.meta(),
];
