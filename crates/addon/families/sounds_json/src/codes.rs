//! The `sounds_json` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each id is `sounds_json/<name>`, the name its constant's in
//! lower case.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// Vanilla's text for a table without its `events` object.
pub static EXPECTED_EVENTS: CodeDef<0> = CodeDef::new(
    "sounds_json/expected_events",
    Kind::MissingRequired,
    Class::KeptError,
    "missing required field",
)
.game("Expected \"events\" object");

/// Vanilla's text for `individual_named_sounds` without its `sounds` object.
pub static EXPECTED_SOUNDS: CodeDef<0> = CodeDef::new(
    "sounds_json/expected_sounds",
    Kind::MissingRequired,
    Class::KeptError,
    "missing required field",
)
.game("Expected \"sounds\" object");

/// Vanilla's text for a variant map without a key.
pub static EXPECTED_KEY: CodeDef<0> = CodeDef::new(
    "sounds_json/expected_key",
    Kind::MissingRequired,
    Class::KeptError,
    "missing required field",
)
.game("Expected \"key\" expression for entity with sound variants");

/// Vanilla's text for a variant key without a map.
pub static EXPECTED_MAP: CodeDef<0> = CodeDef::new(
    "sounds_json/expected_map",
    Kind::MissingRequired,
    Class::KeptError,
    "missing required field",
)
.game("Expected \"map\" list for entity with sound variant key");

/// Vanilla's text for a block sound whose `base` names no block sound.
pub static UNKNOWN_BASE: CodeDef<1> = CodeDef::new(
    "sounds_json/unknown_base",
    Kind::UnresolvedReference,
    Class::KeptError,
    "unresolved reference",
)
.game("Unknown block base sound '{}'");

/// Vanilla's text for a block sound whose `base` chain returns to a sound it passed.
pub static RECURSIVE_BASE: CodeDef<1> = CodeDef::new(
    "sounds_json/recursive_base",
    Kind::Cycle,
    Class::KeptError,
    "references form a cycle",
)
.game("Block base sound '{}' is recursively used");

/// Vanilla's text for an event name it does not know.
pub static NOT_AN_EVENT: CodeDef<1> = CodeDef::new(
    "sounds_json/not_an_event",
    Kind::UnresolvedReference,
    Class::KeptError,
    "unknown sound event",
)
.game("Event name '{}' is not a valid LevelSoundEvent");

/// Every definition of this module, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    EXPECTED_EVENTS.meta(),
    EXPECTED_SOUNDS.meta(),
    EXPECTED_KEY.meta(),
    EXPECTED_MAP.meta(),
    UNKNOWN_BASE.meta(),
    RECURSIVE_BASE.meta(),
    NOT_AN_EVENT.meta(),
];
