//! The material family's codes: the client's lines for `.material` files, byte for byte, each a
//! definition beside the reader that raises it. Each [`was`](CodeDef::was) the legacy code its
//! sites raised, so a pin or a filter on the code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// Two keys of one copy with the same name before the first `:`; the arguments are the name,
/// the file and the two keys. Both keys stay in the document: a kept error.
pub static DEFINED_TWICE: CodeDef<4> = CodeDef::new(
    "material/defined_twice",
    Kind::Duplicate,
    Class::KeptError,
    "material {} defined twice in {} ({}, {})",
)
.game("Material \"{}\" is defined twice in \"{}\" as \"{}\" and \"{}\". You cannot redefine a material.")
.was("DuplicateIdentifier");

/// The render state the client names and ignores; the argument is the state.
pub static STATE_IGNORED: CodeDef<1> = CodeDef::new(
    "material/state_ignored",
    Kind::NeverApplies,
    Class::Lint,
    "render state {} has no effect",
)
.game("Render state {} is ignored and will have no effect")
.was("NoEffect");

/// A render state the client does not know; the argument is the state. The state is ignored
/// and the material loads: a kept error.
pub static STATE_UNKNOWN: CodeDef<1> = CodeDef::new(
    "material/state_unknown",
    Kind::InvalidValue,
    Class::KeptError,
    "unknown render state {}",
)
.game("Trying to use a non-existing render state {} for this material!")
.was("InvalidValue");

/// A list entry's removed keyword; the argument is the key. The entry still loads: a kept
/// error.
pub static KEY_REMOVED: CodeDef<1> = CodeDef::new(
    "material/key_removed",
    Kind::Deprecated,
    Class::KeptError,
    "the key {} is no longer read",
)
.game("key \"{}\" is no longer supported, please remove")
.was("DeprecatedMember");

/// A copy that is no JSON; the arguments are the path and the reader's messages. The finding
/// carries the messages again as its `DiagnosticDetail::Syntax`.
pub static PARSE_ERROR: CodeDef<2> = CodeDef::new(
    "material/parse_error",
    Kind::Syntax,
    Class::Error,
    "{} is not JSON: {}",
)
.game("Error parsing {} :\n {}")
.was("MalformedInput");

/// A material variant a later group replaces; the argument is the variant's name.
pub static VARIANT_REPLACED: CodeDef<1> = CodeDef::new(
    "material/variant_replaced",
    Kind::Shadowed,
    Class::Lint,
    "material variant {} replaced",
)
.game("Variant: {} was replaced with a new version")
.was("ShadowedDefinition");

/// Every definition of this table, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    DEFINED_TWICE.meta(),
    STATE_IGNORED.meta(),
    STATE_UNKNOWN.meta(),
    KEY_REMOVED.meta(),
    PARSE_ERROR.meta(),
    VARIANT_REPLACED.meta(),
];
