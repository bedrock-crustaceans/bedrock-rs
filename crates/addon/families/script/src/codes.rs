//! The `script` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each [`was`](CodeDef::was) the legacy code its sites raised,
//! so a pin or a filter on the code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Disposition, Kind};

/// `DuplicateIdentifier`: the game's line `Plugin [{}] - module uuid [{}] with version [{}] already exists, skipping.`.
pub static PLUGIN_MODULE_EXISTS: CodeDef<3> = CodeDef::new(
    "script/plugin_module_exists",
    Kind::Duplicate,
    Class::Warning,
    "duplicate identifier",
)
.game("Plugin [{}] - module uuid [{}] with version [{}] already exists, skipping.")
.was("DuplicateIdentifier");

/// `InvalidEnumValue`: the game's line `Plugin [{}] - using unsupported runtime [{}].`.
pub static PLUGIN_UNSUPPORTED_RUNTIME: CodeDef<2> = CodeDef::new(
    "script/plugin_unsupported_runtime",
    Kind::InvalidEnumValue,
    Class::Error,
    "invalid enum value",
)
.game("Plugin [{}] - using unsupported runtime [{}].")
.was("InvalidEnumValue");

/// `MissingRequiredField`: the game's line `Plugin [{}] - entry [{}] not supported for runtime [{}].`.
pub static PLUGIN_ENTRY_NOT_SUPPORTED_MISSING_REQUIRED_FIELD: CodeDef<3> = CodeDef::new(
    "script/plugin_entry_not_supported_missing_required_field",
    Kind::MissingRequired,
    Class::Error,
    "missing required field",
)
.game("Plugin [{}] - entry [{}] not supported for runtime [{}].")
.was("MissingRequiredField");

/// `InvalidValue`: the game's line `Plugin [{}] - entry [{}] not supported for runtime [{}].`.
pub static PLUGIN_ENTRY_NOT_SUPPORTED_INVALID_VALUE: CodeDef<3> = CodeDef::new(
    "script/plugin_entry_not_supported_invalid_value",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("Plugin [{}] - entry [{}] not supported for runtime [{}].")
.was("InvalidValue");

/// `DeprecatedMember`: the game's line `Plugin [{}] dependency by UUID is deprecated and only supports version 0.1.0, use module_name instead.`.
pub static PLUGIN_DEPENDENCY_BY_UUID: CodeDef<1> = CodeDef::new(
    "script/plugin_dependency_by_uuid",
    Kind::Deprecated,
    Class::Warning,
    "deprecated member",
)
.game("Plugin [{}] dependency by UUID is deprecated and only supports version 0.1.0, use module_name instead.")
.was("DeprecatedMember");

/// `UnresolvedReference`: the game's line `Plugin [{}] - does not contain main file [{}].`.
pub static PLUGIN_MISSING_MAIN_FILE: CodeDef<2> = CodeDef::new(
    "script/plugin_missing_main_file",
    Kind::UnresolvedReference,
    Class::Error,
    "unresolved reference",
)
.game("Plugin [{}] - does not contain main file [{}].")
.was("UnresolvedReference");

/// `UnresolvedReference`: the game's line `Plugin [{}] - module [{}] depends on unknown module [{}].`.
pub static PLUGIN_UNKNOWN_MODULE: CodeDef<3> = CodeDef::new(
    "script/plugin_unknown_module",
    Kind::UnresolvedReference,
    Class::Error,
    "unresolved reference",
)
.game("Plugin [{}] - module [{}] depends on unknown module [{}].")
.was("UnresolvedReference");

/// `DroppedField`: the game's line `Plugin [{}] - skipping dependency on [{}] because this is not an editor world.`.
pub static PLUGIN_NOT_EDITOR_WORLD: CodeDef<2> = CodeDef::new(
    "script/plugin_not_editor_world",
    Kind::UnknownMember(Disposition::Dropped),
    Class::Lint,
    "field dropped",
)
.game("Plugin [{}] - skipping dependency on [{}] because this is not an editor world.")
.was("DroppedField");

/// `MissingToggles`: the game's line `Plugin [{}] - requesting dependency on [{}], but the Beta APIs experiment is not enabled.`.
pub static PLUGIN_BETA_APIS_MODULE: CodeDef<2> = CodeDef::new(
    "script/plugin_beta_apis_module",
    Kind::Toggle,
    Class::Error,
    "needs a toggle that is not enabled",
)
.game("Plugin [{}] - requesting dependency on [{}], but the Beta APIs experiment is not enabled.")
.was("MissingToggles");

/// `MissingToggles`: the game's line `Plugin [{}] - requesting dependency on beta APIs [{}], but the Beta APIs experiment is not enabled.`.
pub static PLUGIN_BETA_APIS: CodeDef<2> = CodeDef::new(
    "script/plugin_beta_apis",
    Kind::Toggle,
    Class::Error,
    "needs a toggle that is not enabled",
)
.game("Plugin [{}] - requesting dependency on beta APIs [{}], but the Beta APIs experiment is not enabled.")
.was("MissingToggles");

/// `ScriptModulePermission`: the game's line `Plugin [{}] - requesting dependency on internal APIs [{}], but does not have permission to access them.`.
pub static PLUGIN_INTERNAL_APIS: CodeDef<2> = CodeDef::new(
    "script/plugin_internal_apis",
    Kind::ScriptModule,
    Class::Error,
    "native module not permitted",
)
.game("Plugin [{}] - requesting dependency on internal APIs [{}], but does not have permission to access them.")
.was("ScriptModulePermission");

/// `ScriptModulePermission`: the game's line `Plugin [{}] - requesting dependency on module [{}] but it is not configured to use it.`.
pub static PLUGIN_MODULE_NOT_CONFIGURED: CodeDef<2> = CodeDef::new(
    "script/plugin_module_not_configured",
    Kind::ScriptModule,
    Class::Error,
    "native module not permitted",
)
.game("Plugin [{}] - requesting dependency on module [{}] but it is not configured to use it.")
.was("ScriptModulePermission");

/// `ScriptSyntaxError`: the game's line `SyntaxError: invalid UTF-8 sequence    at {}:{}`.
pub static SCRIPT_INVALID_UTF8: CodeDef<2> = CodeDef::new(
    "script/script_invalid_utf8",
    Kind::ScriptSource,
    Class::Error,
    "syntax error in a script file",
)
.game("SyntaxError: invalid UTF-8 sequence    at {}:{}")
.was("ScriptSyntaxError");

/// `ScriptSyntaxError`: the game's line `SyntaxError: expecting ';'    at {}:{}`.
pub static SCRIPT_EXPECTING_SEMICOLON: CodeDef<2> = CodeDef::new(
    "script/script_expecting_semicolon",
    Kind::ScriptSource,
    Class::Error,
    "syntax error in a script file",
)
.game("SyntaxError: expecting ';'    at {}:{}")
.was("ScriptSyntaxError");

/// `ScriptSyntaxError`: the game's line `{} is not UTF-8 at line {}; nothing imports the file, so the engine never reads it`.
pub static UNREAD_NOT_UTF8: CodeDef<2> = CodeDef::new(
    "script/unread_not_utf8",
    Kind::ScriptSource,
    Class::Lint,
    "syntax error in a script file",
)
.game("{} is not UTF-8 at line {}; nothing imports the file, so the engine never reads it")
.was("ScriptSyntaxError");

/// `ScriptSyntaxError`: the game's line `{} is not UTF-8 at line {}; a syntax error comes before it`.
pub static NOT_UTF8_AFTER_ERROR: CodeDef<2> = CodeDef::new(
    "script/not_utf8_after_error",
    Kind::ScriptSource,
    Class::Error,
    "syntax error in a script file",
)
.game("{} is not UTF-8 at line {}; a syntax error comes before it")
.was("ScriptSyntaxError");

/// Every definition of this module, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    PLUGIN_MODULE_EXISTS.meta(),
    PLUGIN_UNSUPPORTED_RUNTIME.meta(),
    PLUGIN_ENTRY_NOT_SUPPORTED_MISSING_REQUIRED_FIELD.meta(),
    PLUGIN_ENTRY_NOT_SUPPORTED_INVALID_VALUE.meta(),
    PLUGIN_DEPENDENCY_BY_UUID.meta(),
    PLUGIN_MISSING_MAIN_FILE.meta(),
    PLUGIN_UNKNOWN_MODULE.meta(),
    PLUGIN_NOT_EDITOR_WORLD.meta(),
    PLUGIN_BETA_APIS_MODULE.meta(),
    PLUGIN_BETA_APIS.meta(),
    PLUGIN_INTERNAL_APIS.meta(),
    PLUGIN_MODULE_NOT_CONFIGURED.meta(),
    SCRIPT_INVALID_UTF8.meta(),
    SCRIPT_EXPECTING_SEMICOLON.meta(),
    UNREAD_NOT_UTF8.meta(),
    NOT_UTF8_AFTER_ERROR.meta(),
];
