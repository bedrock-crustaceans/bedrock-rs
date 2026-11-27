//! The `spawn` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each id is `spawn/<name>`, the name its constant's in lower
//! case.
//!
//! The texts of the spawn loaders, as the folders quote them. Every condition text is a warning
//! in the game (code 2, level 2); `{}` is the document's identifier where the text names it.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// `minecraft:weight` is not an object; the argument is the identifier.
pub static WEIGHT_OBJECT: CodeDef<1> = CodeDef::new(
    "spawn/weight_object",
    Kind::WrongType,
    Class::Warning,
    "minecraft:weight is not an object",
)
.game("Spawn rules for minecraft:weight in {} need to be put inside a json object - it has to start with an \"{\" and end with a \"}\" - using default values");

/// `minecraft:density_limit` is not an object; the argument is the identifier.
pub static DENSITY_OBJECT: CodeDef<1> = CodeDef::new(
    "spawn/density_object",
    Kind::WrongType,
    Class::Warning,
    "minecraft:density_limit is not an object",
)
.game("Spawn rules for minecraft:density_limit in {} need to be put inside a json object - it has to start with an \"{\" and end with a \"}\" - using default values");

/// `minecraft:brightness_filter` is not an object; the argument is the identifier.
pub static BRIGHTNESS_OBJECT: CodeDef<1> = CodeDef::new(
    "spawn/brightness_object",
    Kind::WrongType,
    Class::Warning,
    "minecraft:brightness_filter is not an object",
)
.game("Spawn rules for minecraft:brightness_filter in {} need to be put inside a json object - it has to start with an \"{\" and end with a \"}\" - using default values");

/// `minecraft:height_filter` is not an object; the argument is the identifier.
pub static HEIGHT_OBJECT: CodeDef<1> = CodeDef::new(
    "spawn/height_object",
    Kind::WrongType,
    Class::Warning,
    "minecraft:height_filter is not an object",
)
.game("Spawn rules for minecraft:height_filter in {} need to be put inside a json object - it has to start with an \"{\" and end with a \"}\" - using default values");

/// `minecraft:distance_filter` is not an object; the argument is the identifier.
pub static DISTANCE_OBJECT: CodeDef<1> = CodeDef::new(
    "spawn/distance_object",
    Kind::WrongType,
    Class::Warning,
    "minecraft:distance_filter is not an object",
)
.game("Spawn rules for minecraft:distance_filter in {} need to be put inside a json object - it has to start with an \"{\" and end with a \"}\" - using default values");

/// `minecraft:delay_filter` is not an object; the argument is the identifier.
pub static DELAY_OBJECT: CodeDef<1> = CodeDef::new(
    "spawn/delay_object",
    Kind::WrongType,
    Class::Warning,
    "minecraft:delay_filter is not an object",
)
.game("Spawn rules for minecraft:delay_filter in {} need to be put inside a json object - it has to start with an \"{\" and end with a \"}\" - using default values");

/// `minecraft:mob_event_filter` is not an object; the argument is the identifier.
pub static MOB_EVENT_OBJECT: CodeDef<1> = CodeDef::new(
    "spawn/mob_event_object",
    Kind::WrongType,
    Class::Warning,
    "minecraft:mob_event_filter is not an object",
)
.game("Spawn rules for minecraft:mob_event_filter in {} need to be put inside a json object - it has to start with an \"{\" and end with a \"}\" - using default values");

/// `minecraft:world_age_filter` is not an object. A literal copy of the text: no identifier, single
/// braces.
pub static WORLD_AGE_OBJECT: CodeDef<0> = CodeDef::new(
    "spawn/world_age_object",
    Kind::WrongType,
    Class::Warning,
    "minecraft:world_age_filter is not an object",
)
.game("Spawn rules for minecraft:world_age_filter need to be put inside a json object - it has to start with an \"{\" and end with a \"}\" - using default values");

/// `height_filter.min > max`. The game's text names `distance_filter`.
pub static HEIGHT_SWAPPED: CodeDef<0> = CodeDef::new(
    "spawn/height_swapped",
    Kind::InvalidValue,
    Class::Warning,
    "height_filter min above max",
)
.game("Spawn rules for minecraft:distance_filter specified a min height > max.");

/// `distance_filter.min > max`.
pub static DISTANCE_SWAPPED: CodeDef<0> = CodeDef::new(
    "spawn/distance_swapped",
    Kind::InvalidValue,
    Class::Warning,
    "distance_filter min above max",
)
.game("Spawn rules for minecraft:distance_filter specified a min distance > max.");

/// `world_age_filter.min > max`.
pub static WORLD_AGE_SWAPPED: CodeDef<0> = CodeDef::new(
    "spawn/world_age_swapped",
    Kind::InvalidValue,
    Class::Warning,
    "world_age_filter min above max",
)
.game("Spawn rules for minecraft:world_age_filter specified a min world age > max.");

/// `delay_filter.min > max`; the argument is the identifier.
pub static DELAY_SWAPPED: CodeDef<1> = CodeDef::new(
    "spawn/delay_swapped",
    Kind::InvalidValue,
    Class::Warning,
    "delay_filter min above max in {}",
)
.game("Spawn rules for {} specified a min delay > max.");

/// `delay_filter.identifier` empty or missing; the argument is the identifier.
pub static DELAY_WITHOUT_IDENTIFIER: CodeDef<1> = CodeDef::new(
    "spawn/delay_without_identifier",
    Kind::MissingRequired,
    Class::Warning,
    "delay_filter without an identifier in {}",
)
.game("Spawn rules for {} specified a delay without a target identifier.");

/// `player_in_village_filter.distance` missing or `null`.
pub static VILLAGE_DISTANCE_MISSING: CodeDef<0> = CodeDef::new(
    "spawn/village_distance_missing",
    Kind::MissingRequired,
    Class::Warning,
    "player_in_village_filter without a distance",
)
.game("Missing \"distance\" parameter for minecraft:player_in_village_filter");

/// `description.identifier` missing, `null` or not a string. The literal ends in a newline in the
/// game, which is trimmed.
pub static MISSING_IDENTIFIER: CodeDef<0> = CodeDef::new(
    "spawn/missing_identifier",
    Kind::MissingRequired,
    Class::Error,
    "no description.identifier",
)
.game("Missing identifier in spawn rule file");

/// The same text for a spawn group file whose payload key is absent.
pub static MISSING_IDENTIFIER_PAYLOAD: CodeDef<0> = CodeDef::new(
    "spawn/missing_identifier_payload",
    Kind::Envelope,
    Class::Error,
    "no payload",
)
.game("Missing identifier in spawn rule file");

/// The root is not a JSON object; the finding carries "the root is not an object" as its syntax
/// detail.
pub static NOT_AN_OBJECT: CodeDef<0> = CodeDef::new(
    "spawn/not_an_object",
    Kind::Syntax,
    Class::Error,
    "the root is not an object",
)
.game("Spawn rules json was not formatted as an object");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[
    WEIGHT_OBJECT.meta(),
    DENSITY_OBJECT.meta(),
    BRIGHTNESS_OBJECT.meta(),
    HEIGHT_OBJECT.meta(),
    DISTANCE_OBJECT.meta(),
    DELAY_OBJECT.meta(),
    MOB_EVENT_OBJECT.meta(),
    WORLD_AGE_OBJECT.meta(),
    HEIGHT_SWAPPED.meta(),
    DISTANCE_SWAPPED.meta(),
    WORLD_AGE_SWAPPED.meta(),
    DELAY_SWAPPED.meta(),
    DELAY_WITHOUT_IDENTIFIER.meta(),
    VILLAGE_DISTANCE_MISSING.meta(),
    MISSING_IDENTIFIER.meta(),
    MISSING_IDENTIFIER_PAYLOAD.meta(),
    NOT_AN_OBJECT.meta(),
];
