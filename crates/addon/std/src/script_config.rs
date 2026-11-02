//! Reading a dedicated server's script configuration from disk: the `config/` folder and
//! `server.properties`. The parsing is `bedrock_addon`'s and pure;
//! this is the only place that touches the file system.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use bedrock_addon::families::script::config::{
    ConfigFolder, ScriptConfig, ScriptConfigFiles, ServerProperties, UnreadableFile,
};
use bytes::Bytes;

use crate::ReadError;
use crate::read::read_optional;

/// Reads a server's `config/` folder and parses it with the pure `ScriptConfig::parse`.
///
/// A folder that does not exist is an empty configuration, as on the server: every script
/// module with a dependency is then refused. Nothing a file holds fails the read: what the
/// files said is in `ScriptConfig::diagnostics`, a file that exists but cannot be opened
/// included. `ReadError::Io` only when `config_dir` itself exists and cannot be listed.
pub fn read_script_config(config_dir: impl AsRef<Path>) -> Result<ScriptConfig, ReadError> {
    Ok(ScriptConfig::parse(&read_script_config_files(config_dir)?))
}

/// The files of a server's `config/` folder as `ScriptConfig::parse` takes them: the three
/// files of every sub-directory, whatever its name, and `commands.json`.
///
/// Symbolic links are followed. Other files directly under `config_dir` are not read. A file
/// that exists but cannot be read is listed in `unreadable`, not an error. The errors are
/// those of [`read_script_config`].
pub fn read_script_config_files(
    config_dir: impl AsRef<Path>,
) -> Result<ScriptConfigFiles, ReadError> {
    let dir = config_dir.as_ref();
    let mut files = ScriptConfigFiles::default();
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(source) if source.kind() == ErrorKind::NotFound => return Ok(files),
        Err(source) => return Err(ReadError::io(dir, source)),
    };
    let mut file = |folder: Option<&str>, path: &Path, name: &str| -> Option<Bytes> {
        match fs::read(path.join(name)) {
            Ok(bytes) => Some(Bytes::from(bytes)),
            Err(source) if source.kind() == ErrorKind::NotFound => None,
            Err(source) => {
                files.unreadable.push(UnreadableFile {
                    folder: folder.map(Into::into),
                    file: name.into(),
                    kind: source.kind(),
                });
                None
            }
        }
    };
    let mut folders = Vec::new();
    for entry in entries {
        let path = entry.map_err(|source| ReadError::io(dir, source))?.path();
        // `metadata` follows a link; an entry that vanished or cannot be looked at is no
        // folder.
        if !fs::metadata(&path).is_ok_and(|metadata| metadata.is_dir()) {
            continue;
        }
        let Some(name) = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
        else {
            continue;
        };
        let mut folder = ConfigFolder::new(name.as_str());
        let [permissions, variables, secrets] = ConfigFolder::FILES;
        folder.permissions = file(Some(&name), &path, permissions);
        folder.variables = file(Some(&name), &path, variables);
        folder.secrets = file(Some(&name), &path, secrets);
        folder.stray_commands = path.join(ScriptConfigFiles::COMMANDS_FILE).is_file();
        folders.push(folder);
    }
    let commands = file(None, dir, ScriptConfigFiles::COMMANDS_FILE);
    files.folders = folders;
    files.commands = commands;
    Ok(files)
}

/// Reads a `server.properties` and parses it with the pure `ServerProperties::parse`.
///
/// A file that does not exist holds no line: every key has its default. A file that exists
/// but cannot be read is `ReadError::Io`. What the lines said is in
/// `ServerProperties::diagnostics`; the typed script keys are `ServerProperties::script`.
pub fn read_server_properties(path: impl AsRef<Path>) -> Result<ServerProperties, ReadError> {
    Ok(
        read_optional(path.as_ref())?.map_or_else(ServerProperties::default, |bytes| {
            ServerProperties::parse(&bytes)
        }),
    )
}
