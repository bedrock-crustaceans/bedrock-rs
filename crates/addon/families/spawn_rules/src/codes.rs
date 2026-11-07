//! The `spawn_rules` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each [`was`](CodeDef::was) the legacy code its sites raised, so
//! a pin or a filter on the code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// The closing line of a spawn-rules file that failed.
pub static FAILED_RULES: CodeDef<0> = CodeDef::new(
    "spawn_rules/failed_rules",
    Kind::Closing,
    Class::Error,
    "spawn rules file failed to load",
)
.game("Failed to load spawn rules")
.was("DocumentRejected");

/// An unknown `population_control`; the argument is the identifier. The game logs this to its
/// developer log only.
pub static NO_POPULATION_CONTROL: CodeDef<1> = CodeDef::new(
    "spawn_rules/no_population_control",
    Kind::InvalidEnumValue,
    Class::Lint,
    "unknown population control for {}",
)
.game("No population control specified for '{}'")
.was("InvalidEnumValue");

/// `placement.type` not registered; the argument is the identifier.
pub static PLACEMENT_TYPE: CodeDef<1> = CodeDef::new(
    "spawn_rules/placement_type",
    Kind::UnresolvedReference,
    Class::Warning,
    "unregistered placement type in {}",
)
.game("Spawn Rules for 'type' in {} must use a registered placement type predicate")
.was("UnresolvedReference");

/// `placement.heightmap` not registered; the argument is the identifier.
pub static PLACEMENT_HEIGHTMAP: CodeDef<1> = CodeDef::new(
    "spawn_rules/placement_heightmap",
    Kind::UnresolvedReference,
    Class::Warning,
    "unregistered heightmap in {}",
)
.game("Spawn Rules for 'heightmap' in {} must use a registered heightmap type")
.was("UnresolvedReference");

/// `placement.spawn_obstruction_predicate` not registered; the argument is the identifier.
pub static PLACEMENT_OBSTRUCTION: CodeDef<1> = CodeDef::new(
    "spawn_rules/placement_obstruction",
    Kind::UnresolvedReference,
    Class::Warning,
    "unregistered obstruction predicate in {}",
)
.game("Spawn Rules for 'spawn_obstruction_predicate' in {} must use a registered obstruction predicate")
.was("UnresolvedReference");

/// `placement.spawn_predicate` not registered; the argument is the identifier.
pub static PLACEMENT_PREDICATE: CodeDef<1> = CodeDef::new(
    "spawn_rules/placement_predicate",
    Kind::UnresolvedReference,
    Class::Warning,
    "unregistered spawn predicate in {}",
)
.game("Spawn Rules for 'spawn_predicate' in {} must use a registered entity spawn predicate")
.was("UnresolvedReference");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[
    FAILED_RULES.meta(),
    NO_POPULATION_CONTROL.meta(),
    PLACEMENT_TYPE.meta(),
    PLACEMENT_HEIGHTMAP.meta(),
    PLACEMENT_OBSTRUCTION.meta(),
    PLACEMENT_PREDICATE.meta(),
];
