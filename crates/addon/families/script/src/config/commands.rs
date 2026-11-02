//! Reading `config/commands.json` (the server's `commands` schema): unknown members are accepted, an invalid level name refuses the whole file,
//! and of a member written twice the first stands.

use bedrock_addon_core::diagnostic::{
    DiagnosticCode, Diagnostics, JsonPath, Severity, Span, VanillaText,
};
use bedrock_addon_core::json::JsonSlice;

use super::read::{Sink, Twice, child, object, span, string, value, with_root};
use super::{COMMANDS_FILE, CommandPermissionLevel, CommandsConfig, UnreadableFile};

/// The two lines the server prints for a syntax error; `{}` is the file.
pub(super) const MALFORMED: &str = "Failed to parse JSON in file '{}'\n:  malformed input";
/// The two lines the server prints for a level name it does not know; `{}` are the file,
/// the member and the value found.
pub(super) const BAD_LEVEL: &str = "Failed to parse JSON in file '{}'\n -> {}:  This value is bound to 'command_permission_level' and expects one of the following strings: 'any', 'game_directors', 'admin', ... (3 others). Found: '{}'";

const DEFAULT_PERMISSION_LEVEL: &str = "default_permission_level";
const PERMISSION_LEVELS: &str = "permission_levels";

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
    let mut config = CommandsConfig::default();
    let mut refused = false;
    // The names of `permission_levels` that are no command, with where each is.
    let mut unknown: Vec<(String, JsonPath, Option<Span>)> = Vec::new();
    let parsed = with_root(bytes?, Twice::First, &mut sink, |root, sink| {
        let at = JsonPath::new();
        let Some(members) = object(root) else {
            sink.push(
                DiagnosticCode::InvalidType { text: None },
                Severity::Error,
                at,
                span(root),
            );
            refused = true;
            return;
        };
        for (name, raw) in &members {
            let Some(slice) = root.nested(raw) else {
                continue;
            };
            let here = child(&at, name);
            match name.as_str() {
                DEFAULT_PERMISSION_LEVEL => match level(slice, here, true, sink) {
                    Some(level) => config.default_permission_level = Some(level),
                    None => refused = true,
                },
                PERMISSION_LEVELS => {
                    refused |= !levels(slice, &here, sink, &mut config, &mut unknown);
                }
                _ => {
                    // Accepted silently by the server.
                    sink.info(
                        DiagnosticCode::UnrecognizedField { text: None },
                        here.clone(),
                        span(slice),
                    );
                    if let Some(value) = value(slice, &here, sink) {
                        config.extra.insert(name.clone(), value);
                    }
                }
            }
        }
    });
    if !parsed || refused {
        return None;
    }
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

/// `permission_levels`: command name → level. False when anything in it refuses the file.
fn levels(
    slice: JsonSlice<'_>,
    at: &JsonPath,
    sink: &mut Sink<'_>,
    config: &mut CommandsConfig,
    unknown: &mut Vec<(String, JsonPath, Option<Span>)>,
) -> bool {
    let Some(members) = object(slice) else {
        sink.push(
            DiagnosticCode::InvalidType { text: None },
            Severity::Error,
            at.clone(),
            span(slice),
        );
        return false;
    };
    let mut accepted = true;
    config.permission_levels.reserve(members.len());
    for (command, raw) in &members {
        let Some(slice) = slice.nested(raw) else {
            continue;
        };
        match level(slice, child(at, command), false, sink) {
            Some(level) => {
                // A name as typed, without `/`: a command's own or one of its aliases.
                if bedrock_addon_core::vanilla::command(command).is_none() {
                    unknown.push((command.clone(), child(at, command), span(slice)));
                }
                config.permission_levels.insert(command.clone(), level);
            }
            None => accepted = false,
        }
    }
    accepted
}

/// The name of the root member at `at`, as the file has it: what the server's text names.
fn member_name(at: &JsonPath) -> String {
    at.last_key().unwrap_or_default().to_owned()
}

/// One level name. Anything else is an error that refuses the file; the server's text is
/// known for a root member only (`root`).
fn level(
    slice: JsonSlice<'_>,
    at: JsonPath,
    root: bool,
    sink: &mut Sink<'_>,
) -> Option<CommandPermissionLevel> {
    let Some(name) = string(slice) else {
        sink.push(
            DiagnosticCode::InvalidType { text: None },
            Severity::Error,
            at,
            span(slice),
        );
        return None;
    };
    let level = CommandPermissionLevel::from_name(&name);
    if level.is_none() {
        let text = root.then(|| VanillaText::Formatted {
            template: BAD_LEVEL,
            args: vec![sink.name.clone(), member_name(&at), name],
        });
        sink.push(
            DiagnosticCode::InvalidEnumValue { text },
            Severity::Error,
            at,
            span(slice),
        );
    }
    level
}
