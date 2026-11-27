//! The game's text shared by the event loaders of entities, items and blocks.

/// The level-2 line of an event that holds both `randomize` and `sequence`. The game logs it
/// for item and block events and, with the same text, for entity events; the text says "Item"
/// in every family.
pub const EVENT_BOTH_RANDOMIZE_AND_SEQUENCE: &str =
    "Item event has both randomize and sequence tags. Sequence will be ignored.";
