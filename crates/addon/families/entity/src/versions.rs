//! The per-version facts of the family: what a new format version touches, in one place.
//!
//! A new format version moves [`VERSION`] (and the history chain's newest struct), and, where
//! the game types members or components anew, the version of the first typed read; the typed
//! rows of [`EntityComponent`](crate::component::EntityComponent) state each component's own
//! (`typed = ".."`).

use bedrock_addon_core::version::{FormatVersion, known};

/// The newest format version: the newest struct of the history chain (`Versioned::VERSION`).
pub(crate) const VERSION: FormatVersion = known::V1_26_60;

/// The newest format version a goal's typed read has rules for: what a strict goal parse
/// without a document version reads as.
pub(crate) const NEWEST_GOAL_RULES: FormatVersion = known::V1_26_50;

/// The format version that types `panic`, `tempt`, `float_tempt` and `send_event`
/// (the format version from which the game's typed schema carries them).
pub(crate) const GOALS_TYPED_1_26_60: FormatVersion = known::V1_26_60;

/// The first slice registered with the strict mask.
pub(crate) const STRICT_SLICES: FormatVersion = FormatVersion::new(1, 21, 89);

/// The newest struct that holds the payload as JSON: a component typed first after it converts
/// in the read of its step to 1.26.50.
pub(crate) const LAST_JSON_STRUCT: FormatVersion = known::V1_26_40;

/// The version that types the four ram / jump Brain keys, which the 1.26.50 format introduces
/// (with no reader before it).
pub(crate) const BRAIN_1_26_50: FormatVersion = known::V1_26_50;

/// The Brain keys first bound in the 1.26.50 slice ([`BRAIN_1_26_50`]).
pub(crate) const BRAIN_1_26_50_KEYS: [&str; 4] = [
    "minecraft:memory_behavior.jump_to_block",
    "minecraft:memory_behavior.prepare_ram_attack",
    "minecraft:memory_behavior.ram_attack",
    "minecraft:memory_sensor.find_ram_charge_position",
];
