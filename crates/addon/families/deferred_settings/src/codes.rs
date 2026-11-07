//! The deferred settings family's codes: the game's lines for its lighting, water,
//! atmosphere, colour grading, cubemap, PBR, shadow and local-light files, byte for byte, each a
//! definition beside the reader or check that raises it. Each [`was`](CodeDef::was) the legacy
//! code its sites raised, so a pin or a filter on the code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Disposition, Kind};

// ---- the envelope ----

/// A `format_version` that is no version (the envelope's text).
pub static MALFORMED_VERSION: CodeDef<0> = CodeDef::new(
    "deferred_settings/malformed_version",
    Kind::FormatVersion,
    Class::Error,
    "the format_version is no version",
)
.game("Malformed format version")
.was("MalformedFormatVersion");

/// The literal `"beta"` read with the Beta APIs toggle: no struct takes it, and the game's
/// envelope refuses it as no version.
pub static BETA_UNSUPPORTED: CodeDef<0> = CodeDef::new(
    "deferred_settings/beta_unsupported",
    Kind::UnsupportedVersion,
    Class::Error,
    "the beta format_version is no version",
)
.game("Malformed format version")
.was("UnsupportedVersion");

/// The literal `"beta"` read without the Beta APIs toggle, with the envelope's text.
pub static BETA_NOT_ENABLED: CodeDef<0> = CodeDef::new(
    "deferred_settings/beta_not_enabled",
    Kind::Toggle,
    Class::Error,
    "the beta format_version is no version",
)
.game("Malformed format version")
.was("BetaFeaturesNotEnabled");

/// A root of other than two members, none of them `format_version`.
pub static MEMBER_COUNT_WITHOUT_VERSION: CodeDef<0> = CodeDef::new(
    "deferred_settings/member_count_without_version",
    Kind::FormatVersion,
    Class::Error,
    "the root is not \"format_version\" and a payload member",
)
.game("A Minecraft Document must have exactly two members: \"format_version\" and a payload member")
.was("MissingFormatVersion");

// ---- the identifiers ----

/// A second file of one pack with an identifier already defined there; the argument is the
/// identifier.
pub static IDENTIFIER_IN_PACK: CodeDef<1> = CodeDef::new(
    "deferred_settings/identifier_in_pack",
    Kind::Duplicate,
    Class::Error,
    "identifier {} defined twice in the pack",
)
.game("Identifier {} already exists in resource pack")
.was("DuplicateIdentifier");

/// A default identifier no file defines; the arguments are the kind's noun and the
/// identifier. The game keeps its own values: a kept error.
pub static NO_MATCHING_SETTINGS: CodeDef<2> = CodeDef::new(
    "deferred_settings/no_matching_settings",
    Kind::UnresolvedReference,
    Class::KeptError,
    "no {} settings {}",
)
.game("Could not find any matching settings for {} identifier '{}'")
.was("UnresolvedReference");

// ---- the link pass ----

/// A local light whose key is no block; the argument is the key. The game drops the entry: a
/// kept error.
pub static NOT_A_BLOCK: CodeDef<1> = CodeDef::new(
    "deferred_settings/not_a_block",
    Kind::UnresolvedReference,
    Class::KeptError,
    "{} is no block",
)
.game("{} is not a block")
.was("UnresolvedReference");

/// A caustics texture no pack has; the argument is its path. The game uses its own: a kept
/// error.
pub static CAUSTICS_NOT_FOUND: CodeDef<1> = CodeDef::new(
    "deferred_settings/caustics_not_found",
    Kind::UnresolvedReference,
    Class::KeptError,
    "no caustics texture {}",
)
.game("Resource pack caustics texture could not be loaded. No texture could be found at \"{}\".")
.was("TextureNotFound");

/// A caustics texture that cannot be decoded; the argument is its path. A kept error.
pub static CAUSTICS_UNDECODABLE: CodeDef<1> = CodeDef::new(
    "deferred_settings/caustics_undecodable",
    Kind::Asset,
    Class::KeptError,
    "caustics texture {} cannot be decoded",
)
.game("Caustics texture \"{}\" provided in resource pack could not be loaded due to an unknown error. Ensure the texture is a valid image and is in one of the supported image formats.")
.was("InvalidTexture");

/// A caustics texture whose height is no multiple of its width; the argument is its path. A
/// kept error.
pub static CAUSTICS_FRAMES: CodeDef<1> = CodeDef::new(
    "deferred_settings/caustics_frames",
    Kind::Asset,
    Class::KeptError,
    "caustics texture {} does not divide into frames",
)
.game("Caustics texture \"{}\" provided in resource pack does not evenly divide into frames. Ensure the texture height is evenly divisible by the width.")
.was("InvalidTexture");

// ---- the payload's values ----

/// An unknown member a later struct declares; the arguments are the member and the version
/// that brings it. The game is silent about a file that loads: an information.
pub static NEEDS_FORMAT_VERSION: CodeDef<2> = CodeDef::new(
    "deferred_settings/needs_format_version",
    Kind::UnknownMember(Disposition::Kept),
    Class::Lint,
    "{} needs format_version {}",
)
.game("{} needs format_version {}")
.was("UnrecognizedField");

/// An unknown member an older struct declared; the arguments are the member and the version
/// from which it is no longer read. An information.
pub static NO_LONGER_USED: CodeDef<2> = CodeDef::new(
    "deferred_settings/no_longer_used",
    Kind::UnknownMember(Disposition::Kept),
    Class::Lint,
    "{} is no longer used (format_version {})",
)
.game("{} is no longer used (format_version {})")
.was("UnrecognizedField");

/// A value that is no number.
pub static MUST_BE_NUMBER: CodeDef<0> = CodeDef::new(
    "deferred_settings/must_be_number",
    Kind::WrongType,
    Class::Error,
    "not a number",
)
.game("the input must be a number")
.was("InvalidType");

/// A colour without alpha that cannot be read.
pub static RGB: CodeDef<0> = CodeDef::new(
    "deferred_settings/rgb",
    Kind::WrongType,
    Class::Error,
    "not an RGB colour",
)
.game("Expected [r, g, b], with values in the 0-255 range, or a hex value with 6 characters (e.g. RGB as #FFFFFF).")
.was("InvalidType");

/// A colour with alpha that cannot be read.
pub static RGBA: CodeDef<0> = CodeDef::new(
    "deferred_settings/rgba",
    Kind::WrongType,
    Class::Error,
    "not an ARGB colour",
)
.game("Expected [r, g, b, a], with values in the 0-255 range, or a hex value with 8 characters (e.g. ARGB as #FFFFFFFF).")
.was("InvalidType");

/// Keyframes of the wrong JSON type.
pub static KEYFRAMES: CodeDef<0> = CodeDef::new(
    "deferred_settings/keyframes",
    Kind::WrongType,
    Class::Error,
    "not keyframes",
)
.game("Expected keyframes.")
.was("InvalidType");

/// A triple of numbers that cannot be read.
pub static VEC3: CodeDef<0> = CodeDef::new(
    "deferred_settings/vec3",
    Kind::WrongType,
    Class::Error,
    "not [x, y, z]",
)
.game("Expected [x, y, z].")
.was("InvalidType");

/// A keyframe key that is no number; the argument is the name of the target type.
pub static KEY_CONVERSION: CodeDef<1> = CodeDef::new(
    "deferred_settings/key_conversion",
    Kind::WrongType,
    Class::Error,
    "the keyframe key is no {}",
)
.game("String-to-{} conversion failed")
.was("InvalidType");

/// A recent pack of deferred-rendering settings that declares no capability the deferred
/// pipeline needs: the files take effect only with Vibrant Visuals. Told as an information.
pub static PBR_CAPABILITY_HINT: CodeDef<0> = CodeDef::new(
    "deferred_settings/pbr_capability_hint",
    Kind::Toggle,
    Class::Lint,
    "render configuration files need the pbr capability",
)
.game(
    "render configuration files are only used when Vibrant Visuals is available; add the \"pbr\" capability",
)
.was("CapabilityHint");

/// Every definition of this table, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    MALFORMED_VERSION.meta(),
    BETA_UNSUPPORTED.meta(),
    BETA_NOT_ENABLED.meta(),
    MEMBER_COUNT_WITHOUT_VERSION.meta(),
    IDENTIFIER_IN_PACK.meta(),
    NO_MATCHING_SETTINGS.meta(),
    NOT_A_BLOCK.meta(),
    CAUSTICS_NOT_FOUND.meta(),
    CAUSTICS_UNDECODABLE.meta(),
    CAUSTICS_FRAMES.meta(),
    NEEDS_FORMAT_VERSION.meta(),
    NO_LONGER_USED.meta(),
    MUST_BE_NUMBER.meta(),
    RGB.meta(),
    RGBA.meta(),
    KEYFRAMES.meta(),
    VEC3.meta(),
    KEY_CONVERSION.meta(),
    PBR_CAPABILITY_HINT.meta(),
];
