//! The `texture_list` family's codes: the game's content-log lines, byte for byte, each a
//! definition beside the rules that raise it. Each [`was`](CodeDef::was) the legacy code its sites
//! raised, so a pin or a filter on the code's name keeps matching.
//!
//! The game's line goes to its developer log, not the content log: the crate reports it as an
//! information with the game's text.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// A base-game pack with a `textures/` directory and no list; the arguments are the list's path and
/// the pack.
pub static NO_LIST_IN_BASE_PACK: CodeDef<2> = CodeDef::new(
    "texture_list/no_list_in_base_pack",
    Kind::MissingRequired,
    Class::Lint,
    "base pack {} has a textures directory and no list {}",
)
.game("Failed to find {} in {} for a base pack with a texture directory, falling back to pack iteration")
.was("MissingRequiredField");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[NO_LIST_IN_BASE_PACK.meta()];
