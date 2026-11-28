//! Reading `config/commands.json` (the server's `commands` schema): [`CommandsConfig`]
//! derives its read. Unknown members are accepted, an invalid level name refuses the whole
//! file, and of a member written twice the first stands.

use crate::codes;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity, Span};
use bedrock_addon_core::json::Winner;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::schema::read_document;
use bedrock_addon_core::versioning::{NullHandling, Reading};

use super::read::{Sink, with_root};
use super::{COMMANDS_FILE, CommandsConfig, UnreadableFile};

const DEFAULT_PERMISSION_LEVEL: &str = "default_permission_level";
const PERMISSION_LEVELS: &str = "permission_levels";

/// The server's reading of the file: every failure an error that refuses the file, each
/// told; an unknown member an information, kept.
const COMMANDS: Reading = Reading::OWN
    .strict()
    .with_null(NullHandling::Rejected)
    .with_unknown(Severity::Info);

/// Reads `config/commands.json`; `None` when it is absent, unreadable or refused.
pub(super) fn read(
    bytes: Option<&[u8]>,
    unreadable: &[&UnreadableFile],
    d: &mut Diagnostics,
) -> Option<CommandsConfig> {
    let mut sink = Sink::new(None, COMMANDS_FILE, d);
    sink.commands = true;
    if let Some(failed) = unreadable
        .iter()
        .find(|failed| &*failed.file == COMMANDS_FILE)
    {
        sink.unreadable(failed.kind);
        return None;
    }
    let file = sink.name.clone();
    let mut config = None;
    // The names of `permission_levels` that are no command, with where each is.
    let mut unknown: Vec<(String, JsonPath, Option<Span>)> = Vec::new();
    with_root(bytes?, Winner::First, &mut sink, |root, found| {
        let mut read = Read::new(COMMANDS, found).clamp_kept();
        config = read_document::<CommandsConfig>(root, &mut read);
        if let Some(config) = &config {
            unknown = unknown_commands(root, &mut read, config);
        }
        drop(read);
        bad_level(root, found, &file);
    });
    let config = config?;
    // A level for a name the game has no command for: the server keeps it and nothing
    // reads it, so the name is told once the file is known to load.
    for (command, at, span) in unknown {
        sink.reference(COMMAND, &command, at, span);
    }
    Some(config)
}

/// What a `permission_levels` key names, in a finding's reference detail (the kind the link
/// pass uses for a command).
const COMMAND: &str = "command";

/// The keys of `permission_levels` that name no command (a name as typed, without `/`: a
/// command's own or one of its aliases), each with where its level is.
fn unknown_commands(
    root: &Node<'_>,
    read: &mut Read<'_>,
    config: &CommandsConfig,
) -> Vec<(String, JsonPath, Option<Span>)> {
    let levels = read
        .object(root)
        .and_then(|members| members.get(PERMISSION_LEVELS))
        .and_then(|levels| read.object(&levels));
    let Some(levels) = levels else {
        return Vec::new();
    };
    config
        .permission_levels
        .keys()
        .filter(|command| bedrock_addon_core::vanilla::command(command).is_none())
        .filter_map(|command| {
            let level = levels.get(command)?;
            Some((command.clone(), level.path().clone(), level.span()))
        })
        .collect()
}

/// Words a level name the root's `default_permission_level` does not know in the server's
/// line, which names the file, the member and the value: only the root's member carries it.
fn bad_level(root: &Node<'_>, found: &mut Diagnostics, file: &str) {
    let at = JsonPath::new().child(DEFAULT_PERMISSION_LEVEL);
    let name = root
        .value()
        .as_object()
        .and_then(|object| object.get(DEFAULT_PERMISSION_LEVEL))
        .and_then(|value| value.as_string())
        .map(|name| name.as_str().to_owned());
    let Some(name) = name else {
        return;
    };
    for finding in found.iter_mut() {
        if finding.location.json_path == at && finding.is(&core_codes::INVALID_ENUM_VALUE) {
            finding.code = codes::BAD_LEVEL.with([
                file.to_owned(),
                DEFAULT_PERMISSION_LEVEL.to_owned(),
                name.clone(),
            ]);
        }
    }
}
