//! The `blocks.json` family's codes: vanilla's content-log texts of `blocks.json`, byte for byte,
//! and the crate's own line for a file that is no JSON. Each id is `blocks_json/<name>`, the name
//! its constant's in lower case.
//!
//! The texts about blocks that do not exist (a key that names no block) are raised by the link
//! stage, which sees both the resource and the behaviour stack; it holds those texts.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Disposition, Kind};

// ---- the file ----

/// Why the file is no JSON object; the argument is the reader's message, which the finding also
/// carries as its `DiagnosticDetail::Syntax`. Vanilla skips such a file without a word; this text
/// is the crate's own.
pub static NOT_JSON: CodeDef<1> = CodeDef::new(
    "blocks_json/not_json",
    Kind::Syntax,
    Class::Lint,
    "blocks.json is not valid JSON ({}); clients ignore this file",
)
.game("blocks.json is not valid JSON ({}); clients ignore this file");

/// The file has no `format_version`.
pub static VERSION_NOT_GIVEN: CodeDef<0> = CodeDef::new(
    "blocks_json/version_not_given",
    Kind::FormatVersion,
    Class::Warning,
    "blocks.json has no format_version; 1.1.0 is used",
)
.game("Warning: The blocks.json \"format_version\" field was not given. \"format_version\" 1.1.0 will be used.");

/// The `format_version` is not a version.
pub static VERSION_INVALID: CodeDef<0> = CodeDef::new(
    "blocks_json/version_invalid",
    Kind::FormatVersion,
    Class::Warning,
    "the blocks.json format_version is invalid; 1.1.0 is used",
)
.game("Warning: The blocks.json \"format_version\" field is invalid. \"format_version\" 1.1.0 will be used.");

/// The `format_version` is above the newest the client reads: takes the file's version, then the
/// ceiling. The trailing line break is vanilla's.
pub static VERSION_ABOVE_CEILING: CodeDef<2> = CodeDef::new(
    "blocks_json/version_above_ceiling",
    Kind::UnsupportedVersion,
    Class::Warning,
    "the blocks.json format_version {} is above {}; 1.1.0 is used",
)
.game("Warning: The blocks.json \"format_version\": {} exceeds largest currently supported version: {}.\"format_version\" 1.1.0 will be used.\n");

// ---- the entries ----

/// A base-game pack's entry asks for a `blockshape`: takes the key. The entry is skipped and the
/// file loads.
pub static BASE_GAME_BLOCKSHAPE: CodeDef<1> = CodeDef::new(
    "blocks_json/base_game_blockshape",
    Kind::InvalidValue,
    Class::KeptError,
    "a base game pack's entry {} has a blockshape",
)
.game("Unexpected: blocks.json in a base game pack specifies a blockshape. Json entry: '{}'");

/// A key is a legacy name that was split into several blocks: takes `minecraft:` and the
/// lower-cased key, then `"\n - <name>"` per post-split block.
pub static OLD_NAME: CodeDef<2> = CodeDef::new(
    "blocks_json/old_name",
    Kind::Deprecated,
    Class::Error,
    "the old name {} was split into: {}",
)
.game("blocks.json file contains a block using an old name that was used for multiple blocks: {}. \nPlease use their new names instead: {}");

/// The `textures` of a block are invalid or incomplete: takes the block's full name.
pub static BAD_TEXTURES: CodeDef<1> = CodeDef::new(
    "blocks_json/bad_textures",
    Kind::WrongType,
    Class::Warning,
    "invalid or incomplete textures of {}",
)
.game("'{}' | Invalid or incomplete texture list.");

/// The `carried_textures` of a block are invalid or incomplete: takes the block's full name.
pub static BAD_CARRIED_TEXTURES: CodeDef<1> = CodeDef::new(
    "blocks_json/bad_carried_textures",
    Kind::WrongType,
    Class::Warning,
    "invalid or incomplete carried_textures of {}",
)
.game("'{}' | Invalid or incomplete carried_textures list.");

/// The `blockshape` is not a known shape name, or the conversion is refused: takes the block's
/// full name.
pub static BAD_BLOCKSHAPE: CodeDef<1> = CodeDef::new(
    "blocks_json/bad_blockshape",
    Kind::InvalidValue,
    Class::Warning,
    "invalid blockshape of {}",
)
.game("'{}' | Invalid blockshape name.");

/// A custom block has a `blockshape`, which only vanilla blocks take: takes the block's full
/// name. The entry loads without a shape.
pub static CUSTOM_BLOCKSHAPE: CodeDef<1> = CodeDef::new(
    "blocks_json/custom_blockshape",
    Kind::UnknownMember(Disposition::Ignored),
    Class::KeptError,
    "a blockshape on the custom block {}",
)
.game("'{}' | Cannot set 'blockshape' on custom blocks.");

/// Every definition of this table, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    NOT_JSON.meta(),
    VERSION_NOT_GIVEN.meta(),
    VERSION_INVALID.meta(),
    VERSION_ABOVE_CEILING.meta(),
    BASE_GAME_BLOCKSHAPE.meta(),
    OLD_NAME.meta(),
    BAD_TEXTURES.meta(),
    BAD_CARRIED_TEXTURES.meta(),
    BAD_BLOCKSHAPE.meta(),
    CUSTOM_BLOCKSHAPE.meta(),
];
