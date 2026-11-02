//! Vanilla's content-log texts of `blocks.json`, each with `{}` where vanilla's `%s` / `%d` / `{}`
//! goes.

use bedrock_addon_core::diagnostic::VanillaText;

// The texts about blocks that do not exist (a key that names no block) are raised by the link
// stage, which sees both the resource and the behaviour stack; it holds those texts.

/// The file has no `format_version`.
pub(super) const VERSION_NOT_GIVEN: &str = "Warning: The blocks.json \"format_version\" field was not given. \"format_version\" 1.1.0 will be used.";
/// The `format_version` is not a version.
pub(super) const VERSION_INVALID: &str = "Warning: The blocks.json \"format_version\" field is invalid. \"format_version\" 1.1.0 will be used.";
/// The `format_version` is above the newest the client reads: takes the file's version, then the
/// ceiling. The trailing line break is vanilla's.
pub(super) const VERSION_ABOVE_CEILING: &str = "Warning: The blocks.json \"format_version\": {} exceeds largest currently supported version: {}.\"format_version\" 1.1.0 will be used.\n";
/// A base-game pack's entry asks for a `blockshape`: takes the key.
pub(super) const BASE_GAME_BLOCKSHAPE: &str =
    "Unexpected: blocks.json in a base game pack specifies a blockshape. Json entry: '{}'";
/// A key is a legacy name that was split into several blocks: takes `minecraft:` and the
/// lower-cased key, then `"\n - <name>"` per post-split block.
pub(super) const OLD_NAME: &str = "blocks.json file contains a block using an old name that was used for multiple blocks: {}. \nPlease use their new names instead: {}";
/// The `textures` of a block are invalid or incomplete: takes the block's full name.
pub(super) const BAD_TEXTURES: &str = "'{}' | Invalid or incomplete texture list.";
/// The `carried_textures` of a block are invalid or incomplete: takes the block's full name.
pub(super) const BAD_CARRIED_TEXTURES: &str = "'{}' | Invalid or incomplete carried_textures list.";
/// The `blockshape` is not a known shape name, or the conversion is refused: takes the block's
/// full name.
pub(super) const BAD_BLOCKSHAPE: &str = "'{}' | Invalid blockshape name.";
/// A custom block has a `blockshape`, which only vanilla blocks take: takes the block's full
/// name.
pub(super) const CUSTOM_BLOCKSHAPE: &str = "'{}' | Cannot set 'blockshape' on custom blocks.";

/// Why the file is no JSON object. Vanilla skips such a file without a word; this text is the
/// crate's own.
pub(super) const NOT_JSON: &str = "blocks.json is not valid JSON ({}); clients ignore this file";

/// `template` with `args`.
pub(super) fn formatted(template: &'static str, args: &[&str]) -> VanillaText {
    VanillaText::Formatted {
        template,
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
    }
}

/// Every template of this module with the number of its arguments, for the arity test.
#[cfg(test)]
pub(super) const TEMPLATES: &[(&str, usize)] = &[
    (VERSION_NOT_GIVEN, 0),
    (NOT_JSON, 1),
    (VERSION_INVALID, 0),
    (VERSION_ABOVE_CEILING, 2),
    (BASE_GAME_BLOCKSHAPE, 1),
    (OLD_NAME, 2),
    (BAD_TEXTURES, 1),
    (BAD_CARRIED_TEXTURES, 1),
    (BAD_BLOCKSHAPE, 1),
    (CUSTOM_BLOCKSHAPE, 1),
];
