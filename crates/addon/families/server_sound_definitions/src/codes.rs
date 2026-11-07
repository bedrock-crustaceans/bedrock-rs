//! The `server_sound_definitions` family's codes: the game's content-log lines, byte for byte, each
//! a definition beside the rules that raise it. Each [`was`](CodeDef::was) the legacy code its
//! sites raised, so a pin or a filter on the code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// The namespace constraint's text for a name without `namespace:value`.
pub static NEEDS_NAMESPACE: CodeDef<0> = CodeDef::new(
    "server_sound_definitions/needs_namespace",
    Kind::Identifier,
    Class::Error,
    "sound event name without a namespace",
)
.game("string must be prefixed with a namespace (eg. namespace:value)")
.was("NamespaceRule");

/// The namespace constraint's text for a disallowed namespace; the argument is the namespace.
pub static INVALID_NAMESPACE: CodeDef<1> = CodeDef::new(
    "server_sound_definitions/invalid_namespace",
    Kind::Identifier,
    Class::Error,
    "sound event name in the {} namespace",
)
.game("string has invalid namespace ({})")
.was("NamespaceRule");

/// A sound event name that is registered already: an error in the sound log area, printed once by
/// the game (raised for each later definition here).
pub static DUPLICATE: CodeDef<1> = CodeDef::new(
    "server_sound_definitions/duplicate",
    Kind::Duplicate,
    Class::Error,
    "server sound event {} defined twice",
)
.game(
    "Duplicate server sound definition for sound event '{}'; the later definition will be ignored.",
)
.was("DuplicateIdentifier");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[
    NEEDS_NAMESPACE.meta(),
    INVALID_NAMESPACE.meta(),
    DUPLICATE.meta(),
];
