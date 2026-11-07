//! The jigsaw family's codes: the game's content-log lines for jigsaw structures, template
//! pools, structure sets and processor lists, byte for byte, and the lines of the crate's own.
//! Each [`was`](CodeDef::was) the legacy code its sites raised, so a pin or a filter on the
//! code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

// ---- the closing line of a file that failed to parse ----

/// A jigsaw structure file failed to parse; the argument is its path.
pub static STRUCTURE_FAILED: CodeDef<1> = CodeDef::new(
    "jigsaw/structure_failed",
    Kind::Closing,
    Class::Error,
    "the jigsaw structure {} failed to parse",
)
.game("Failed to parse jigsaw structure definition '{}'.")
.was("DocumentRejected");

/// A template pool file failed to parse; the argument is its path.
pub static POOL_FAILED: CodeDef<1> = CodeDef::new(
    "jigsaw/pool_failed",
    Kind::Closing,
    Class::Error,
    "the template pool {} failed to parse",
)
.game("Failed to parse jigsaw structure template pool '{}'.")
.was("DocumentRejected");

/// A structure set file failed to parse; the argument is its path.
pub static SET_FAILED: CodeDef<1> = CodeDef::new(
    "jigsaw/set_failed",
    Kind::Closing,
    Class::Error,
    "the structure set {} failed to parse",
)
.game("Failed to parse structure set definition '{}'.")
.was("DocumentRejected");

/// A processor list file failed to parse; the argument is its path.
pub static PROCESSORS_FAILED: CodeDef<1> = CodeDef::new(
    "jigsaw/processors_failed",
    Kind::Closing,
    Class::Error,
    "the processor list {} failed to parse",
)
.game("Failed to parse jigsaw structure processor list '{}'.")
.was("DocumentRejected");

// ---- the identifier check, in the game's order ----

/// An identifier without a `:`.
pub static NO_NAMESPACE: CodeDef<0> = CodeDef::new(
    "jigsaw/no_namespace",
    Kind::InvalidValue,
    Class::Error,
    "the identifier has no namespace",
)
.game("Identifier must have a namespace")
.was("InvalidValue");

/// An empty namespace or name, or a second `:`.
pub static INVALID_IDENTIFIER: CodeDef<0> = CodeDef::new(
    "jigsaw/invalid_identifier",
    Kind::InvalidValue,
    Class::Error,
    "the identifier is invalid",
)
.game("Invalid identifier")
.was("InvalidValue");

/// A space, tab, LF or CR in the namespace or the name; the next line says which.
pub static ILLEGAL_CHARACTERS: CodeDef<0> = CodeDef::new(
    "jigsaw/illegal_characters",
    Kind::InvalidValue,
    Class::Error,
    "the identifier has illegal characters",
)
.game("Identifier cannot have illegal characters")
.was("InvalidValue");

/// The namespace holds an illegal character.
pub static NAMESPACE_INVALID: CodeDef<0> = CodeDef::new(
    "jigsaw/namespace_invalid",
    Kind::InvalidValue,
    Class::Error,
    "the namespace is invalid",
)
.game("Namespace is invalid")
.was("InvalidValue");

/// The name holds an illegal character.
pub static NAME_INVALID: CodeDef<0> = CodeDef::new(
    "jigsaw/name_invalid",
    Kind::InvalidValue,
    Class::Error,
    "the identifier name is invalid",
)
.game("Identifier name is invalid")
.was("InvalidValue");

/// What a jigsaw structure's identifier failure says after the check's own reason.
pub static STRUCTURE_REGISTER: CodeDef<1> = CodeDef::new(
    "jigsaw/structure_register",
    Kind::InvalidValue,
    Class::Error,
    "the jigsaw structure identifier {} failed validation",
)
.game("Identifier '{}' failed validation when registering jigsaw structure json")
.was("InvalidValue");

/// The template pool's.
pub static POOL_REGISTER: CodeDef<1> = CodeDef::new(
    "jigsaw/pool_register",
    Kind::InvalidValue,
    Class::Error,
    "the template pool identifier {} failed validation",
)
.game("Identifier '{}' failed validation when registering jigsaw structure template pool json")
.was("InvalidValue");

/// The structure set's (also for a set entry).
pub static SET_REGISTER: CodeDef<1> = CodeDef::new(
    "jigsaw/set_register",
    Kind::InvalidValue,
    Class::Error,
    "the structure set identifier {} failed validation",
)
.game("Identifier '{}' failed validation when registering jigsaw structure set json")
.was("InvalidValue");

/// The processor list's.
pub static PROCESSORS_REGISTER: CodeDef<1> = CodeDef::new(
    "jigsaw/processors_register",
    Kind::InvalidValue,
    Class::Error,
    "the processor list identifier {} failed validation",
)
.game("Identifier '{}' failed validation when registering jigsaw structure processor list json")
.was("InvalidValue");

// ---- the registration checks of one document ----

/// An empty `start_pool`: the game's text, logged only by the builder of the hard-coded structures,
/// so an information of the crate's own; the document stays loaded.
pub static START_POOL_EMPTY: CodeDef<0> = CodeDef::new(
    "jigsaw/start_pool_empty",
    Kind::InvalidValue,
    Class::Lint,
    "the jigsaw structure's start_pool is empty",
)
.game("[Jigsaw Structure Builder] start_pool is empty. Fill it in with a valid template pool e.g. minecraft:example/structure")
.was("InvalidValue");

/// A structure set whose spacing is not more than twice its separation; the argument is the set's identifier.
pub static SPACING_SEPARATION: CodeDef<1> = CodeDef::new(
    "jigsaw/spacing_separation",
    Kind::InvalidValue,
    Class::Error,
    "the structure set {} has a spacing not above twice its separation",
)
.game("Invalid spacing and separation combination for structure set '{}'. Value 'separation' must be less than half of 'spacing'.")
.was("InvalidValue");

/// An `append_loot` modifier with an empty loot table: the modifier is cleared, the rule stays.
pub static EMPTY_LOOT_TABLE: CodeDef<0> = CodeDef::new(
    "jigsaw/empty_loot_table",
    Kind::InvalidValue,
    Class::KeptError,
    "an append_loot modifier with an empty loot table",
)
.game("Empty loot table")
.was("InvalidValue");

/// A metadata file whose leading format version string does not parse; the argument is its path.
pub static BAD_FORMAT_VERSION_STRING: CodeDef<1> = CodeDef::new(
    "jigsaw/bad_format_version_string",
    Kind::FormatVersion,
    Class::Error,
    "the metadata file {} has no readable format version string",
)
.game("Failed to parse format version string of file {}; it will be skipped.")
.was("MalformedFormatVersion");

// ---- the checks over several documents ----

/// A template pool name registered twice (or a built-in one): the game only asserts, so an
/// information of the crate's own.
pub static POOL_NAME_REGISTERED: CodeDef<1> = CodeDef::new(
    "jigsaw/pool_name_registered",
    Kind::Duplicate,
    Class::Lint,
    "the template pool name {} is already registered",
)
.game("Jigsaw pool name `{}` already registered")
.was("DuplicateIdentifier");

/// A pool element whose location another element holds with other contents: the game keeps one
/// silently, so the text is the crate's own (an information).
pub static ELEMENT_REDEFINED: CodeDef<1> = CodeDef::new(
    "jigsaw/element_redefined",
    Kind::InvalidValue,
    Class::Lint,
    "the element location {} is redefined",
)
.game("Redefinition of existing element with location '{}'.")
.was("InvalidValue");

/// A pack's `minecraft:trial_chambers` set against the game's own record, which is refused.
pub static SET_ALREADY_REGISTERED: CodeDef<1> = CodeDef::new(
    "jigsaw/set_already_registered",
    Kind::Duplicate,
    Class::Error,
    "the structure set {} is already registered",
)
.game("Structure set '{}' has already been registered")
.was("DuplicateIdentifier");

// ---- the link pass ----

/// The linked-asset validator's line: the member path as the game writes it, then the identifier.
pub static MISSING_ASSET: CodeDef<2> = CodeDef::new(
    "jigsaw/missing_asset",
    Kind::UnresolvedReference,
    Class::Error,
    "{}: missing referenced asset {}",
)
.game("{}:  Missing referenced asset {}")
.was("UnresolvedReference");

/// The validator's line for a file reference (a structure, a loot table) no pack holds.
pub static INVALID_ASSET_PATH: CodeDef<2> = CodeDef::new(
    "jigsaw/invalid_asset_path",
    Kind::UnresolvedReference,
    Class::Error,
    "{}: invalid asset path {}",
)
.game("{}:  Invalid asset path {}")
.was("UnresolvedReference");

/// The game's generation-failure line for a missing structure, written at generation time: an
/// information of the crate's own at link.
pub static STRUCTURE_MISSING: CodeDef<2> = CodeDef::new(
    "jigsaw/structure_missing",
    Kind::UnresolvedReference,
    Class::Lint,
    "the structure {} referenced in the template pool {} does not exist",
)
.game("[JIGSAW GENERATION FAILURE] Structure name [{}] referenced in template pool [{}] does not exist.")
.was("UnresolvedReference");

/// The game's line for a feature element that does not resolve, written when the pool is
/// generated: an information of the crate's own at link.
pub static FEATURE_MISSING: CodeDef<1> = CodeDef::new(
    "jigsaw/feature_missing",
    Kind::UnresolvedReference,
    Class::Lint,
    "the feature {} of a feature_pool_element does not resolve",
)
.game("Cannot find 'feature' '{}' for 'feature_pool_element'. Make sure to include the 'namespace_name:' prefix.")
.was("UnresolvedReference");

/// A rule's block that does not resolve: the game skips the rule and keeps the list.
pub static BLOCK_MISSING: CodeDef<1> = CodeDef::new(
    "jigsaw/block_missing",
    Kind::UnresolvedReference,
    Class::KeptError,
    "the block {} does not resolve",
)
.game("Cannot find block '{}'")
.was("UnresolvedReference");

/// Every definition of this table, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    STRUCTURE_FAILED.meta(),
    POOL_FAILED.meta(),
    SET_FAILED.meta(),
    PROCESSORS_FAILED.meta(),
    NO_NAMESPACE.meta(),
    INVALID_IDENTIFIER.meta(),
    ILLEGAL_CHARACTERS.meta(),
    NAMESPACE_INVALID.meta(),
    NAME_INVALID.meta(),
    STRUCTURE_REGISTER.meta(),
    POOL_REGISTER.meta(),
    SET_REGISTER.meta(),
    PROCESSORS_REGISTER.meta(),
    START_POOL_EMPTY.meta(),
    SPACING_SEPARATION.meta(),
    EMPTY_LOOT_TABLE.meta(),
    BAD_FORMAT_VERSION_STRING.meta(),
    POOL_NAME_REGISTERED.meta(),
    ELEMENT_REDEFINED.meta(),
    SET_ALREADY_REGISTERED.meta(),
    MISSING_ASSET.meta(),
    INVALID_ASSET_PATH.meta(),
    STRUCTURE_MISSING.meta(),
    FEATURE_MISSING.meta(),
    BLOCK_MISSING.meta(),
];
