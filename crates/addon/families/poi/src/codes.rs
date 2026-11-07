//! The `poi` family's codes: the game's content-log lines, byte for byte, each a definition beside
//! the rules that raise it. Each id is `poi/<name>`, the name its constant's in lower case.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// What a tag of a cycle is told, with the tag's name: the phrasing of the game's own unit
/// test of tag cycles (vanilla has no text for it outside a test).
pub static PART_OF_A_CYCLE: CodeDef<1> = CodeDef::new(
    "poi/part_of_a_cycle",
    Kind::Cycle,
    Class::Lint,
    "references form a cycle",
)
.game("{} was a part of a cycle");

/// `UnresolvedReference`: the game's line `BlockTypeRegistry::lookupByName() failed to find block "{}"`.
pub static BLOCK_LOOKUP_FAILED: CodeDef<1> = CodeDef::new(
    "poi/block_lookup_failed",
    Kind::UnresolvedReference,
    Class::Error,
    "unresolved reference",
)
.game("BlockTypeRegistry::lookupByName() failed to find block \"{}\"");

/// `DocumentRejected`: the game's line `Failed to load POI Type`.
pub static FAILED_TO_LOAD: CodeDef<0> = CodeDef::new(
    "poi/failed_to_load",
    Kind::Closing,
    Class::Error,
    "document failed schema validation",
)
.game("Failed to load POI Type");

/// Every definition of this module, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    PART_OF_A_CYCLE.meta(),
    BLOCK_LOOKUP_FAILED.meta(),
    FAILED_TO_LOAD.meta(),
];
