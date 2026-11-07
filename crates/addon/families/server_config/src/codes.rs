//! The `server_config` family's codes: the game's content-log lines, byte for byte, each a
//! definition beside the rules that raise it. Each id is `server_config/<name>`, the name its
//! constant's in lower case.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// The server's text for an auto attach mode it does not know (sic).
pub static BAD_AUTO_ATTACH: CodeDef<1> = CodeDef::new(
    "server_config/bad_auto_attach",
    Kind::InvalidEnumValue,
    Class::Error,
    "invalid enum value",
)
.game("Unkown script debugger auto attach mode {}, expected one of [ 'connect', 'listen', 'disabled' ]");

/// The server's text for a debugger port out of range.
pub static BAD_PORT: CodeDef<0> = CodeDef::new(
    "server_config/bad_port",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("Invalid script debugger port, must be in the range [ 1, 65535 ]");

/// The server's text for a connect address it cannot use.
pub static BAD_ADDRESS: CodeDef<0> = CodeDef::new(
    "server_config/bad_address",
    Kind::InvalidValue,
    Class::Error,
    "invalid value",
)
.game("Invalid script debugger connect address, format must be host:port with port in the range [ 1, 65535 ]");

/// The server's text for a content log level it does not know.
pub static BAD_LOG_LEVEL: CodeDef<1> = CodeDef::new(
    "server_config/bad_log_level",
    Kind::InvalidEnumValue,
    Class::Error,
    "invalid enum value",
)
.game("content-log-level was set to invalid value '{}'.  Accepted values are verbose, info, warning, and error.  Defaulting to info");

/// The two lines the server prints for a level name it does not know; `{}` are the file,
/// the member and the value found.
pub static BAD_LEVEL: CodeDef<3> = CodeDef::new(
    "server_config/bad_level",
    Kind::InvalidEnumValue,
    Class::Error,
    "invalid enum value",
)
.game("Failed to parse JSON in file '{}'\n -> {}:  This value is bound to 'command_permission_level' and expects one of the following strings: 'any', 'game_directors', 'admin', ... (3 others). Found: '{}'");

/// The server's text for a secret that is not a string, which it logs as a warning.
pub static SECRET_NOT_A_STRING: CodeDef<2> = CodeDef::new(
    "server_config/secret_not_a_string",
    Kind::WrongType,
    Class::Warning,
    "wrong JSON type",
)
.game("Secret '{}' in file '{}' is not a string. Secrets can only be strings.");

/// `MalformedInput`: the game's line `Failed to parse JSON in file '{}'\n:  malformed input`.
pub static COMMANDS_MALFORMED_MALFORMED_INPUT: CodeDef<1> = CodeDef::new(
    "server_config/commands_malformed_malformed_input",
    Kind::Syntax,
    Class::Error,
    "malformed input",
)
.game("Failed to parse JSON in file '{}'\n:  malformed input");

/// `EmptyDocument`: the game's line `Failed to parse JSON in file '{}'\n:  malformed input`.
pub static COMMANDS_MALFORMED_EMPTY_DOCUMENT: CodeDef<1> = CodeDef::new(
    "server_config/commands_malformed_empty_document",
    Kind::Syntax,
    Class::Lint,
    "empty document",
)
.game("Failed to parse JSON in file '{}'\n:  malformed input");

/// `MalformedInput`: the game's line `Failed to parse JSON in file '{}'`.
pub static CONFIG_PARSE_FAILED_MALFORMED_INPUT: CodeDef<1> = CodeDef::new(
    "server_config/config_parse_failed_malformed_input",
    Kind::Syntax,
    Class::Error,
    "malformed input",
)
.game("Failed to parse JSON in file '{}'");

/// `EmptyDocument`: the game's line `Failed to parse JSON in file '{}'`.
pub static CONFIG_PARSE_FAILED_EMPTY_DOCUMENT: CodeDef<1> = CodeDef::new(
    "server_config/config_parse_failed_empty_document",
    Kind::Syntax,
    Class::Lint,
    "empty document",
)
.game("Failed to parse JSON in file '{}'");

/// A required dependency the config folder does not allow: the game's line for a module the
/// plugin requests but the folder's `allowed_modules` does not list.
pub static PLUGIN_MODULE_NOT_CONFIGURED: CodeDef<2> = CodeDef::new(
    "server_config/plugin_module_not_configured",
    Kind::ScriptModule,
    Class::Error,
    "native module not permitted",
)
.game("Plugin [{}] - requesting dependency on module [{}] but it is not configured to use it.");

/// The file exists but could not be opened: `Failed to open file '%s'`. The kind of the I/O
/// error rides as the finding's `DiagnosticDetail::Io`.
pub static CONFIG_OPEN_FAILED: CodeDef<1> = CodeDef::new(
    "server_config/config_open_failed",
    Kind::Io,
    Class::Error,
    "cannot read the file",
)
.game("Failed to open file '{}'");

/// Every definition of this module, for the id registry.
pub static ALL: &[&CodeMeta] = &[
    BAD_AUTO_ATTACH.meta(),
    BAD_PORT.meta(),
    BAD_ADDRESS.meta(),
    BAD_LOG_LEVEL.meta(),
    BAD_LEVEL.meta(),
    SECRET_NOT_A_STRING.meta(),
    COMMANDS_MALFORMED_MALFORMED_INPUT.meta(),
    COMMANDS_MALFORMED_EMPTY_DOCUMENT.meta(),
    CONFIG_PARSE_FAILED_MALFORMED_INPUT.meta(),
    CONFIG_PARSE_FAILED_EMPTY_DOCUMENT.meta(),
    PLUGIN_MODULE_NOT_CONFIGURED.meta(),
    CONFIG_OPEN_FAILED.meta(),
];
