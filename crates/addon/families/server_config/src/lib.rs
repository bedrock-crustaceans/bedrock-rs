//! Dedicated server script configuration (`ScriptConfig`, `ScriptConfigFiles`, `ConfigFolder`, `PackScriptConfig`, `CommandsConfig`, `ServerProperties`, `ScriptProperties`).
//!
//! The script configuration of a dedicated server: the `config/`
//! folder (`permissions.json`, `variables.json` and `secrets.json` per folder, and
//! `commands.json`) and the script keys of `server.properties`. None of it is in a pack: the
//! server's administrator writes it, a driver reads the files (`bedrock_addon_std`) and the
//! functions here parse their contents.
//!
//! The readers stay hand walks: the server's config files are read from their slices with the
//! server's own texts.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod commands;
mod net;
mod permissions;
mod properties;
mod read;
mod stack;
mod values;
mod write;

#[cfg(test)]
mod tests;

pub use net::{NetModuleConfig, ServerNet};
pub use properties::{
    AutoAttach, ContentLogLevel, ContentLogProperties, DebuggerProperties,
    DiagnosticsCaptureProperties, PropertyLine, ScriptProperties, SentryProperties,
    ServerProperties, WatchdogProperties,
};
pub use stack::PluginVerdict;

use std::fmt;

use bytes::Bytes;
use indexmap::IndexMap;
use uuid::Uuid;

use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::FormatVersion;

/// The folder a dedicated server reads its script configuration from, in its working
/// directory.
pub const CONFIG_DIR: &str = "config";
/// The folder under [`CONFIG_DIR`] whose files apply to every script module without a folder
/// of its own.
pub const DEFAULT_DIR: &str = "default";
/// The file of a folder that says which script modules a pack may import.
pub const PERMISSIONS_FILE: &str = "permissions.json";
/// The file of a folder that holds the variables `@minecraft/server-admin` exposes.
pub const VARIABLES_FILE: &str = "variables.json";
/// The file of a folder that holds the secrets `@minecraft/server-admin` exposes.
pub const SECRETS_FILE: &str = "secrets.json";
/// The server-wide command permission file, directly under [`CONFIG_DIR`].
pub const COMMANDS_FILE: &str = "commands.json";
/// The server's properties file, in its working directory.
pub const PROPERTIES_FILE: &str = "server.properties";
/// The `allowed_modules` element that allows every module.
pub const WILDCARD_MODULE: &str = "*";
/// The only module whose `module_permissions` entry anything reads.
pub const SERVER_NET_MODULE: &str = "@minecraft/server-net";
/// The longest line of `server.properties` the server reads, in bytes.
// The server reads a line into a buffer of 4096 bytes, the last of which is the terminator.
pub const PROPERTIES_LINE_MAX: usize = 4095;
/// The oldest engine that accepts `max_websocket_payload_size` in a server-net entry; an
/// older one refuses the member.
pub const PAYLOAD_SIZE_SINCE: FormatVersion = FormatVersion::new(1, 26, 36);

/// The files of a server's `config/` folder, as a driver read them.
///
/// `bedrock_addon_std::read_script_config` builds it from a directory; a test or a tool that
/// holds the bytes builds it by hand.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[doc(alias = "config")]
pub struct ScriptConfigFiles {
    /// One entry per sub-directory of `config/`, whatever its name: the server loads all of
    /// them.
    pub folders: Vec<ConfigFolder>,
    /// `config/commands.json`.
    pub commands: Option<Bytes>,
    /// The files that exist but could not be read. The server logs each and uses the folder
    /// without that file.
    pub unreadable: Vec<UnreadableFile>,
}

impl ScriptConfigFiles {
    /// The server-wide command permission file, directly under `config/`.
    pub const COMMANDS_FILE: &'static str = COMMANDS_FILE;
}

/// One sub-directory of `config/` with the three files the server reads in it.
#[derive(Clone, PartialEq, Eq)]
pub struct ConfigFolder {
    /// The folder's name, byte for byte: `default`, or the UUID of a script module in its
    /// lower-case hyphenated form.
    pub name: Box<str>,
    /// `permissions.json`, when the folder has one.
    pub permissions: Option<Bytes>,
    /// `variables.json`, when the folder has one.
    pub variables: Option<Bytes>,
    /// `secrets.json`, when the folder has one.
    pub secrets: Option<Bytes>,
    /// The folder holds a `commands.json`, which the server never reads: only the one directly
    /// under `config/` counts.
    pub stray_commands: bool,
}

impl ConfigFolder {
    /// The three files read in each folder; a missing file is silent.
    pub const FILES: [&'static str; 3] = [PERMISSIONS_FILE, VARIABLES_FILE, SECRETS_FILE];

    /// A folder called `name` without files.
    pub fn new(name: impl Into<Box<str>>) -> Self {
        Self {
            name: name.into(),
            permissions: None,
            variables: None,
            secrets: None,
            stray_commands: false,
        }
    }
}

/// The folder's name and files; of `secrets.json` only its length, never its content.
impl fmt::Debug for ConfigFolder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigFolder")
            .field("name", &self.name)
            .field("permissions", &self.permissions)
            .field("variables", &self.variables)
            .field(
                "secrets",
                &self
                    .secrets
                    .as_ref()
                    .map(|bytes| format!("<{} bytes>", bytes.len())),
            )
            .field("stray_commands", &self.stray_commands)
            .finish()
    }
}

/// A file of `config/` that exists but could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnreadableFile {
    /// The folder the file is in; `None` for `config/commands.json`.
    pub folder: Option<Box<str>>,
    /// The file's name (`variables.json`).
    pub file: Box<str>,
    /// What the host reported.
    pub kind: std::io::ErrorKind,
}

impl UnreadableFile {
    /// The file `file` of `folder` (`None`: `config/commands.json`), which the host could
    /// not read for `kind`.
    pub fn new(folder: Option<&str>, file: &str, kind: std::io::ErrorKind) -> Self {
        Self {
            folder: folder.map(Box::from),
            file: file.into(),
            kind,
        }
    }
}

/// A server's script configuration: the `config/` folder, parsed.
///
/// A script module uses the folder named after its UUID and, without one, `default`; the two
/// are never merged. What the files said while they were read is in
/// [`diagnostics`](Self::diagnostics).
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "config")]
pub struct ScriptConfig {
    /// `config/default/`: the configuration of every script module without a folder of its
    /// own. Empty when the folder is absent, and then every module is refused.
    pub default: PackScriptConfig,
    /// The other folders by name, sorted by name. The server loads every folder; only one
    /// named after a script module's UUID is ever used.
    pub packs: IndexMap<String, PackScriptConfig>,
    /// `config/commands.json`; `None` when it is absent or was refused.
    pub commands: Option<CommandsConfig>,
    pub(crate) diagnostics: Diagnostics,
}

impl ScriptConfig {
    /// Parses the files of a `config/` folder. Nothing fails as a whole: a file that cannot
    /// be used leaves its part empty and is in [`diagnostics`](Self::diagnostics).
    ///
    /// Folders are processed sorted by name, so the order of the findings does not depend
    /// on the order the driver listed them in. Of two folders of one name the later stands.
    pub fn parse(files: &ScriptConfigFiles) -> ScriptConfig {
        read::parse(files)
    }

    /// The configuration of a world a client hosts: no file is read and every module is
    /// allowed.
    // A client-hosted world lets every script module through, whatever a server would say.
    pub fn client_hosted() -> ScriptConfig {
        let mut config = ScriptConfig::default();
        config.default.permissions.allowed_modules = vec![WILDCARD_MODULE.to_owned()];
        config.default.permissions.allowed_modules_written = true;
        config
    }

    /// What the files said while they were read, folders in name order and `commands.json`
    /// last.
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    /// The configuration the script module `script_module_uuid` uses: the folder named with
    /// the UUID's lower-case hyphenated form, else the whole of `default`.
    // The folder's name is the UUID's string form, which is lower case with hyphens.
    pub fn for_module(&self, script_module_uuid: Uuid) -> &PackScriptConfig {
        let mut buffer = Uuid::encode_buffer();
        let key: &str = script_module_uuid.hyphenated().encode_lower(&mut buffer);
        self.packs.get(key).unwrap_or(&self.default)
    }
}

/// The configuration of one folder: what a script module with that folder may import, and
/// the variables and secrets it sees.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PackScriptConfig {
    /// `permissions.json`.
    pub permissions: PackPermissions,
    /// `variables.json`: name → any JSON value, in the file's order (the server keeps them
    /// in no order a script may rely on).
    pub variables: IndexMap<String, Value>,
    /// `secrets.json`: name → string, in the file's order.
    pub secrets: IndexMap<String, SecretString>,
    /// Which folder this is.
    pub source: ConfigSource,
}

/// Which folder a [`PackScriptConfig`] was read from.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum ConfigSource {
    /// `config/default/`, or no folder at all.
    #[default]
    Default,
    /// `config/<name>/`.
    Folder(String),
}

/// One `permissions.json`.
///
/// JSON: `{"allowed_modules": [...], "module_permissions": {...},
/// "commands_permission_level": "..."}`. Every member is optional; a member of the wrong
/// type counts as absent.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "permissions.json")]
pub struct PackPermissions {
    /// The modules a pack may name as a dependency (JSON `allowed_modules`): exact module
    /// names, or [`WILDCARD_MODULE`]. Empty when absent, and then every module is refused.
    pub allowed_modules: Vec<String>,
    /// Whether the file has an `allowed_modules` member. It tells a list left empty on
    /// purpose from a file without the member: both refuse every module, but only the second
    /// is warned of in a pack folder that hides a `default` with a list, and the writer
    /// writes an empty list only when this is set. A list that is not empty is written
    /// whatever this says, and the value read back then has this set, so
    /// it does not equal the hand-built one (this takes part in `PartialEq`).
    pub allowed_modules_written: bool,
    /// Settings per module (JSON `module_permissions`): module name → key → any JSON. Only
    /// the entry of [`SERVER_NET_MODULE`] is read by anything; it is kept here as written
    /// and its meaning is [`server_net`](Self::server_net).
    pub module_permissions: IndexMap<String, IndexMap<String, Value>>,
    /// The permission level of the commands a script runs (JSON `commands_permission_level`);
    /// `None` leaves them at their normal level.
    pub commands_permission_level: Option<CommandPermissionLevel>,
    /// What `@minecraft/server-net` may reach, from its `module_permissions` entry.
    pub server_net: ServerNet,
    /// Unknown members, kept for writing. The server ignores them.
    pub extra: UnknownMembers,
}

impl PackPermissions {
    /// Whether a pack may depend on the module `name`: the list holds exactly that name
    /// (case-sensitive, without a version), or [`WILDCARD_MODULE`].
    // The comparison is exact: the server does not fold case or strip a version.
    pub fn is_module_allowed(&self, name: &str) -> bool {
        self.allowed_modules
            .iter()
            .any(|allowed| allowed == name || allowed == WILDCARD_MODULE)
    }
}

/// A secret of `secrets.json`. `Debug` and `Display` print `<secret>`, never the value;
/// [`expose`](Self::expose) gives it.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretString(Box<str>);

impl SecretString {
    /// A secret holding `value`.
    pub fn new(value: impl Into<Box<str>>) -> Self {
        Self(value.into())
    }

    /// The secret's value.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl From<String> for SecretString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SecretString {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<secret>")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<secret>")
    }
}

/// Who may run a command (`CommandPermissionLevel`), from the lowest to the highest.
///
/// JSON: one of the lower-case names `any`, `game_directors`, `admin`, `host`, `owner`,
/// `internal`.
// The levels are numbered 0 to 5 in this order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
#[repr(u8)]
pub enum CommandPermissionLevel {
    /// `any`: anything can run this level.
    Any = 0,
    /// `game_directors`: any operator, command blocks included.
    GameDirectors = 1,
    /// `admin`: any operator, but not command blocks.
    Admin = 2,
    /// `host`: any server host.
    Host = 3,
    /// `owner`: only the dedicated server.
    Owner = 4,
    /// `internal`: in effect disables the command.
    Internal = 5,
}

impl CommandPermissionLevel {
    /// Every level, lowest first.
    pub const ALL: [Self; 6] = [
        Self::Any,
        Self::GameDirectors,
        Self::Admin,
        Self::Host,
        Self::Owner,
        Self::Internal,
    ];

    /// The level's name in the files.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::GameDirectors => "game_directors",
            Self::Admin => "admin",
            Self::Host => "host",
            Self::Owner => "owner",
            Self::Internal => "internal",
        }
    }

    /// The level called `name`, exactly (case-sensitive).
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|level| level.name() == name)
    }
}

/// `config/commands.json`: the permission levels of the server's commands.
///
/// JSON: `{"default_permission_level": "...", "permission_levels": {"<command>": "..."}}`.
/// An invalid level name refuses the whole file. Of a member written twice the first stands.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "commands.json")]
pub struct CommandsConfig {
    /// The level of every command `permission_levels` does not name (JSON
    /// `default_permission_level`); `None` leaves them at their normal level.
    pub default_permission_level: Option<CommandPermissionLevel>,
    /// Command name → level (JSON `permission_levels`), in the file's order.
    pub permission_levels: IndexMap<String, CommandPermissionLevel>,
    /// Unknown members, kept for writing. The server accepts them silently.
    pub extra: UnknownMembers,
}
