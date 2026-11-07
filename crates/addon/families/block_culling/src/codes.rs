//! The `block_culling` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each [`was`](CodeDef::was) the legacy code its sites raised,
//! so a pin or a filter on the code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Disposition, Kind};

/// Rules 1 and 2: the file's path.
pub static MISSING_OBJECT_UNSUPPORTED_VERSION: CodeDef<1> = CodeDef::new(
    "block_culling/missing_object_unsupported_version",
    Kind::UnsupportedVersion,
    Class::Error,
    "unsupported format_version",
)
.game("Failed to parse block culling file '{}'. Missing object \"minecraft:block_culling_rules\"")
.was("UnsupportedVersion");

/// Rules 1 and 2: the file's path.
pub static MISSING_OBJECT_MISSING_FORMAT_VERSION: CodeDef<1> = CodeDef::new(
    "block_culling/missing_object_missing_format_version",
    Kind::FormatVersion,
    Class::Error,
    "no format_version",
)
.game("Failed to parse block culling file '{}'. Missing object \"minecraft:block_culling_rules\"")
.was("MissingFormatVersion");

/// Rules 1 and 2: the file's path.
pub static MISSING_OBJECT_MALFORMED_FORMAT_VERSION: CodeDef<1> = CodeDef::new(
    "block_culling/missing_object_malformed_format_version",
    Kind::FormatVersion,
    Class::Error,
    "Malformed format version",
)
.game("Failed to parse block culling file '{}'. Missing object \"minecraft:block_culling_rules\"")
.was("MalformedFormatVersion");

/// Rules 1 and 2: the file's path.
pub static MISSING_OBJECT_MISSING_PAYLOAD: CodeDef<1> = CodeDef::new(
    "block_culling/missing_object_missing_payload",
    Kind::Envelope,
    Class::Error,
    "the document has no payload member",
)
.game("Failed to parse block culling file '{}'. Missing object \"minecraft:block_culling_rules\"")
.was("MissingPayload");

/// Rules 1 and 2: the file's path.
pub static MISSING_OBJECT_UNRECOGNIZED_FIELD: CodeDef<1> = CodeDef::new(
    "block_culling/missing_object_unrecognized_field",
    Kind::UnknownMember(Disposition::Kept),
    Class::Unknown,
    "unrecognized field",
)
.game("Failed to parse block culling file '{}'. Missing object \"minecraft:block_culling_rules\"")
.was("UnrecognizedField");

/// Rules 1 and 2: the file's path.
pub static MISSING_OBJECT_DOCUMENT_REJECTED: CodeDef<1> = CodeDef::new(
    "block_culling/missing_object_document_rejected",
    Kind::Closing,
    Class::Error,
    "document failed schema validation",
)
.game("Failed to parse block culling file '{}'. Missing object \"minecraft:block_culling_rules\"")
.was("DocumentRejected");

/// Rules 2 and 3: the file's path.
pub static FAILED_TO_PARSE: CodeDef<1> = CodeDef::new(
    "block_culling/failed_to_parse",
    Kind::Closing,
    Class::Error,
    "document failed schema validation",
)
.game("Failed to parse block culling file '{}'.")
.was("DocumentRejected");

/// Rules 4 and 5: the file's path.
pub static FAILED_INTERNAL: CodeDef<1> = CodeDef::new(
    "block_culling/failed_internal",
    Kind::Closing,
    Class::Error,
    "document failed schema validation",
)
.game("Failed to parse block culling file '{}' into internal format")
.was("DocumentRejected");

/// Rule 4.
pub static INVALID_IDENTIFIER: CodeDef<0> = CodeDef::new(
    "block_culling/invalid_identifier",
    Kind::Identifier,
    Class::Error,
    "namespace rule failed",
)
.game("Invalid identifier")
.was("NamespaceRule");

/// Rule 4.
pub static NEEDS_NAMESPACE: CodeDef<0> = CodeDef::new(
    "block_culling/needs_namespace",
    Kind::Identifier,
    Class::Error,
    "namespace rule failed",
)
.game("Identifier must have a namespace")
.was("NamespaceRule");

/// Rule 4.
pub static MINECRAFT_NAMESPACE: CodeDef<0> = CodeDef::new(
    "block_culling/minecraft_namespace",
    Kind::Identifier,
    Class::Error,
    "namespace rule failed",
)
.game("Identifier not allowed to use Minecraft namespace")
.was("NamespaceRule");

/// Rule 4.
pub static ILLEGAL_CHARACTERS: CodeDef<0> = CodeDef::new(
    "block_culling/illegal_characters",
    Kind::Identifier,
    Class::Error,
    "namespace rule failed",
)
.game("Identifier cannot have illegal characters")
.was("NamespaceRule");

/// Rule 4.
pub static NAMESPACE_INVALID: CodeDef<0> = CodeDef::new(
    "block_culling/namespace_invalid",
    Kind::Identifier,
    Class::Error,
    "namespace rule failed",
)
.game("Namespace is invalid")
.was("NamespaceRule");

/// Rule 4.
pub static NAME_INVALID: CodeDef<0> = CodeDef::new(
    "block_culling/name_invalid",
    Kind::Identifier,
    Class::Error,
    "namespace rule failed",
)
.game("Identifier name is invalid")
.was("NamespaceRule");

/// Rule 4: the identifier.
pub static IDENTIFIER_FAILED: CodeDef<1> = CodeDef::new(
    "block_culling/identifier_failed",
    Kind::MissingRequired,
    Class::Error,
    "no usable identifier",
)
.game("Identifier '{}' failed validation when registering block culling json")
.was("MissingIdentifier");

/// Rule 5.1: the rule's index, the bone.
pub static CUBE_WITHOUT_FACE: CodeDef<2> = CodeDef::new(
    "block_culling/cube_without_face",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("Rule {} with geometry part '{}' had a cube index but no face specified")
.was("InvalidValue");

/// Rule 5.2: the rule's index, the bone.
pub static FACE_WITHOUT_CUBE: CodeDef<2> = CodeDef::new(
    "block_culling/face_without_cube",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("Rule {} with geometry part '{}' had a face specified, but no cube index")
.was("InvalidValue");

/// Rule 5.3: the rule's index, the bone, the cube index (-1 without one), the face
/// (`Unknown facing` without one).
pub static DUPLICATE_PART: CodeDef<4> = CodeDef::new(
    "block_culling/duplicate_part",
    Kind::Duplicate,
    Class::Error,
    "duplicate identifier",
)
.game("Rule {} had geometry part '{}' cube index {} face index '{}' that matches an earlier rule")
.was("DuplicateIdentifier");

/// Every definition of this module, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    MISSING_OBJECT_UNSUPPORTED_VERSION.meta(),
    MISSING_OBJECT_MISSING_FORMAT_VERSION.meta(),
    MISSING_OBJECT_MALFORMED_FORMAT_VERSION.meta(),
    MISSING_OBJECT_MISSING_PAYLOAD.meta(),
    MISSING_OBJECT_UNRECOGNIZED_FIELD.meta(),
    MISSING_OBJECT_DOCUMENT_REJECTED.meta(),
    FAILED_TO_PARSE.meta(),
    FAILED_INTERNAL.meta(),
    INVALID_IDENTIFIER.meta(),
    NEEDS_NAMESPACE.meta(),
    MINECRAFT_NAMESPACE.meta(),
    ILLEGAL_CHARACTERS.meta(),
    NAMESPACE_INVALID.meta(),
    NAME_INVALID.meta(),
    IDENTIFIER_FAILED.meta(),
    CUBE_WITHOUT_FACE.meta(),
    FACE_WITHOUT_CUBE.meta(),
    DUPLICATE_PART.meta(),
];
