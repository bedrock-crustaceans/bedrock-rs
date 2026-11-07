//! The `shapes` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each [`was`](CodeDef::was) the legacy code its sites raised, so
//! a pin or a filter on the code's name keeps matching.
//!
//! Each carries the arguments it takes. The game logs the descriptor texts at level 3 and goes
//! on with an empty descriptor: a kept error.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Disposition, Kind};

/// An item descriptor that is none of the accepted shapes; the argument is the rejected JSON as
/// compact text. The game's line ends in a line break.
pub static ITEM_DESCRIPTOR_SHAPE: CodeDef<1> = CodeDef::new(
    "shapes/item_descriptor_shape",
    Kind::WrongType,
    Class::KeptError,
    "item descriptor of a wrong shape: {}",
)
.game("ItemDescriptor requires a string, {\"item\": string}, or {\"tags\": string} or {\"item_tag\": string}. ItemDescriptor cannot parse {}\n")
.was("InvalidType");

/// A block descriptor that is none of the accepted shapes; the argument is the rejected JSON as
/// compact text.
pub static BLOCK_DESCRIPTOR_SHAPE: CodeDef<1> = CodeDef::new(
    "shapes/block_descriptor_shape",
    Kind::InvalidValue,
    Class::KeptError,
    "block descriptor of a wrong shape: {}",
)
.game("BlockDescriptor requires a string, block object, or any_tag object. Cannot parse {}")
.was("InvalidValue");

/// A block descriptor whose `name` is no string; the argument is the rejected `name` as compact
/// JSON text. The game's line ends in a line break.
pub static BLOCK_DESCRIPTOR_NAME: CodeDef<1> = CodeDef::new(
    "shapes/block_descriptor_name",
    Kind::InvalidValue,
    Class::KeptError,
    "block descriptor name is no string: {}",
)
.game("Only strings are valid for name. The invalid name value: {}\n")
.was("InvalidValue");

/// A block state of a type other than bool, int or string; the arguments are the descriptor's block
/// name, the state's name and jsoncpp's value type number of the rejected value.
pub static BLOCK_DESCRIPTOR_STATE_TYPE: CodeDef<3> = CodeDef::new(
    "shapes/block_descriptor_state_type",
    Kind::InvalidValue,
    Class::KeptError,
    "block {} state {} has type {}",
)
.game("Invalid state type during block parsing: block name {}, state name {}, state type {}. State type can only be bool, int, or string.")
.was("InvalidValue");

/// A block descriptor with a tag expression and block states; the argument is the descriptor's
/// block name (empty without one).
pub static BLOCK_DESCRIPTOR_TAGS_AND_STATES: CodeDef<1> = CodeDef::new(
    "shapes/block_descriptor_tags_and_states",
    Kind::InvalidValue,
    Class::KeptError,
    "block descriptor for {} has both tags and states",
)
.game("Cannot have both a tag expression and block states in a block descriptor for block: {}")
.was("InvalidValue");

/// A block descriptor with a block name and a tag expression; the argument is the descriptor's
/// block name.
pub static BLOCK_DESCRIPTOR_NAME_AND_TAGS: CodeDef<1> = CodeDef::new(
    "shapes/block_descriptor_name_and_tags",
    Kind::InvalidValue,
    Class::KeptError,
    "block descriptor for {} has both a name and tags",
)
.game("Cannot have both a block name and tag expression in a block descriptor for block: {}")
.was("InvalidValue");

/// The level-2 line of an event that holds both `randomize` and `sequence`: the game logs it for
/// item and block events and, with the same text, for entity events; the text says "Item" in every
/// family.
pub static EVENT_BOTH_RANDOMIZE_AND_SEQUENCE: CodeDef<0> = CodeDef::new(
    "shapes/event_both_randomize_and_sequence",
    Kind::UnknownMember(Disposition::Ignored),
    Class::Warning,
    "event with both randomize and sequence",
)
.game("Item event has both randomize and sequence tags. Sequence will be ignored.")
.was("IgnoredMember");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[
    ITEM_DESCRIPTOR_SHAPE.meta(),
    BLOCK_DESCRIPTOR_SHAPE.meta(),
    BLOCK_DESCRIPTOR_NAME.meta(),
    BLOCK_DESCRIPTOR_STATE_TYPE.meta(),
    BLOCK_DESCRIPTOR_TAGS_AND_STATES.meta(),
    BLOCK_DESCRIPTOR_NAME_AND_TAGS.meta(),
    EVENT_BOTH_RANDOMIZE_AND_SEQUENCE.meta(),
];
