//! The `dimension` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each [`was`](CodeDef::was) the legacy code its sites raised, so
//! a pin or a filter on the code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// Part C: a `generator_type` other than `void`; logged at the error level, but the validator
/// returns no validation error: a kept error.
pub static GENERATOR_UNSUPPORTED: CodeDef<0> = CodeDef::new(
    "dimension/generator_unsupported",
    Kind::InvalidValue,
    Class::KeptError,
    "generator_type is not void",
)
.game("minecraft:generation: generator_type unsupported")
.was("InvalidValue");

/// A validator failure: the content-log text, then the validation error text (`Invalid dimension
/// height.` for the height rules of part A, `Invalid custom dimension document.` for the identifier
/// rules of part B). The arguments are the two texts.
pub static VALIDATION_FAILED: CodeDef<2> = CodeDef::new(
    "dimension/validation_failed",
    Kind::InvalidValue,
    Class::Error,
    "dimension validation failed: {} ({})",
)
.game("{}\n{}")
.was("InvalidValue");

/// The runtime's failure to create a dimension; the arguments are the identifier and the pack's
/// UUID. Logged at the warning level.
pub static CREATION_FAILED: CodeDef<2> = CodeDef::new(
    "dimension/creation_failed",
    Kind::InvalidValue,
    Class::Warning,
    "dimension {} from pack {} could not be created",
)
.game("Dimension '{}' from pack id '{}' was invalid and could not be created!")
.was("InvalidValue");

/// A pack definition that replaces a script registration; the arguments are the identifier and the
/// pack's UUID. Logged at the warning level.
pub static REPLACED_REGISTRATION: CodeDef<2> = CodeDef::new(
    "dimension/replaced_registration",
    Kind::Shadowed,
    Class::Warning,
    "dimension {} registered via scripting was replaced by pack {}",
)
.game("Dimension definition identifier '{}' registered via scripting was replaced by the definition from pack id '{}'")
.was("ShadowedDefinition");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[
    GENERATOR_UNSUPPORTED.meta(),
    VALIDATION_FAILED.meta(),
    CREATION_FAILED.meta(),
    REPLACED_REGISTRATION.meta(),
];
