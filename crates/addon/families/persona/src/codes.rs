//! The persona family's codes: the game's lines for persona pieces, byte for byte, each a
//! definition beside the load that raises it. Each id is `persona/<name>`, the name its constant's
//! in lower case.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// A piece's meta file that cannot be read; the argument is its path. The finding carries the
/// host's error kind as its `DiagnosticDetail::Io`.
pub static META_NOT_FOUND: CodeDef<1> = CodeDef::new(
    "persona/meta_not_found",
    Kind::Io,
    Class::Error,
    "cannot read the meta file {}",
)
.game("Could not find expected metafile json file {}");

/// A meta file that is not JSON, or whose root is no object; the argument is its path. The
/// finding carries the reader's message as its `DiagnosticDetail::Syntax`.
pub static META_NOT_PARSED: CodeDef<1> = CodeDef::new(
    "persona/meta_not_parsed",
    Kind::Syntax,
    Class::Error,
    "the meta file {} is not a JSON object",
)
.game("Could not parse metafile json file {}");

/// A geometry file that cannot be read; the argument is its path. The geometry is left empty
/// and the piece kept: a kept error, with the host's error kind as its `DiagnosticDetail::Io`.
pub static GEOMETRY_NOT_FOUND: CodeDef<1> = CodeDef::new(
    "persona/geometry_not_found",
    Kind::Io,
    Class::KeptError,
    "cannot read the geometry file {}",
)
.game("Missing expected geometry file {}");

/// A geometry file that is not a JSON object; the argument is its path. The geometry is left
/// empty and the piece kept: a kept error, with the reader's message as its
/// `DiagnosticDetail::Syntax`.
pub static GEOMETRY_NOT_PARSED: CodeDef<1> = CodeDef::new(
    "persona/geometry_not_parsed",
    Kind::Syntax,
    Class::KeptError,
    "the geometry file {} is not a JSON object",
)
.game("Could not parse geometry file {}");

/// A `geometry_sources` entry names a geometry the piece's file lacks; the arguments are the
/// name and the folder searched. The piece is kept: a kept error.
pub static GEOMETRY_NAME_NOT_FOUND: CodeDef<2> = CodeDef::new(
    "persona/geometry_name_not_found",
    Kind::UnresolvedReference,
    Class::KeptError,
    "no geometry {} in {}",
)
.game("Couldn't find geometry name {} on {}.");

/// A `side` on a piece that is not an arm or leg; the arguments are the piece type and the
/// side. The game shows it on the error display only, not in the content log: an information.
pub static SIDE_NOT_ARM_OR_LEG: CodeDef<2> = CodeDef::new(
    "persona/side_not_arm_or_leg",
    Kind::InvalidValue,
    Class::Lint,
    "a side on a piece that is not an arm or leg ({}, {})",
)
.game("Trying to assign the side attribute to a piece that is not an arm or leg (pieceType: {}, side: '{}')");

/// An emote's animation file that is missing or cannot be read; the arguments are the piece id
/// and the files. The piece is kept: a kept error, with the host's error kind as its
/// `DiagnosticDetail::Io`.
pub static ANIMATION_FILE_MISSING: CodeDef<2> = CodeDef::new(
    "persona/animation_file_missing",
    Kind::Io,
    Class::KeptError,
    "emote {} without its animation file {}",
)
.game("Trying to parse emote {} with missing anim file: {}");

/// An animation an emote piece declares that its file lacks; the arguments are the animation,
/// the piece id and the files. The piece is kept: a kept error.
pub static ANIMATION_NOT_IN_FILE: CodeDef<3> = CodeDef::new(
    "persona/animation_not_in_file",
    Kind::UnresolvedReference,
    Class::KeptError,
    "no animation {} of emote {} in {}",
)
.game("Animation {} declared by emote piece {} was not found in file {}");

/// An emote piece without animation sources; the argument is the piece id. The game stops
/// reading the emote and keeps the piece: a kept error.
pub static EMOTE_INVALID: CodeDef<1> = CodeDef::new(
    "persona/emote_invalid",
    Kind::MissingRequired,
    Class::KeptError,
    "emote {} without animation sources",
)
.game("Trying to parse invalid emote animation definition with pieceId {}");

/// Every definition of this table, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    META_NOT_FOUND.meta(),
    META_NOT_PARSED.meta(),
    GEOMETRY_NOT_FOUND.meta(),
    GEOMETRY_NOT_PARSED.meta(),
    GEOMETRY_NAME_NOT_FOUND.meta(),
    SIDE_NOT_ARM_OR_LEG.meta(),
    ANIMATION_FILE_MISSING.meta(),
    ANIMATION_NOT_IN_FILE.meta(),
    EMOTE_INVALID.meta(),
];
