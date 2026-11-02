//! Writing the files of a folder and `commands.json` back as JSON text: members in the
//! model's order, unknown members after them, nothing the author did not write.

use facet_value::{VArray, VNumber, VObject};

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{
    self, EncodeError, JsonSlice, MAX_JSON_DEPTH, Unrepresentable, Value, WriteStyle, write_json,
};

use super::read::{self, Sink};
use super::{
    CommandsConfig, NetModuleConfig, PERMISSIONS_FILE, PackPermissions, PackScriptConfig,
    SERVER_NET_MODULE, ServerNet, net,
};

/// `value` as JSON text in the default style. The door's writer refuses a value nested past
/// the depth a reader takes (`Unrepresentable::TooDeep`, at the place that is too deep), so
/// no writer here makes a file its reader refuses whole (F6).
fn text(value: VObject) -> Result<String, EncodeError> {
    let mut out = Vec::new();
    write_json(&Value::from(value), &mut out, WriteStyle::default())?;
    // The writer emits only UTF-8.
    Ok(String::from_utf8(out).unwrap_or_default())
}

/// The model cannot be written at `path`: what was read from the text would not be it.
fn refused(path: &[&str]) -> EncodeError {
    let mut at = JsonPath::new();
    for key in path {
        at.push(key);
    }
    EncodeError::NotRepresentable {
        at,
        what: Unrepresentable::Unsupported,
    }
}

/// Adds the unknown members after the known ones; one named like a member the reader knows
/// would be read as that member, so it is refused.
fn extras(root: &mut VObject, extra: &Extra, known: &[&str]) -> Result<(), EncodeError> {
    for (name, value) in extra.iter() {
        if known.contains(&name) {
            return Err(refused(&[name]));
        }
        root.insert(name, value.clone());
    }
    Ok(())
}

/// The settings of one module as a JSON object.
fn object(settings: &IndexMap<String, Value>) -> VObject {
    let mut object = VObject::new();
    for (key, value) in settings {
        object.insert(key.as_str(), value.clone());
    }
    object
}

/// How many levels of a `permissions.json` lie above a `module_permissions` entry: the root
/// and `module_permissions`.
const LEVELS_ABOVE_AN_ENTRY: usize = 2;

/// What the server-net entry `entry` means when it is loaded; `None` when no file loads
/// with it: it nests deeper than a file may below the two levels above it.
fn meaning(entry: &VObject) -> Result<Option<ServerNet>, EncodeError> {
    let text = text(entry.clone())?;
    // The depth bound before the parse, as for a file (F6). The entry may be hand-built.
    if json::check_depth(text.as_bytes(), MAX_JSON_DEPTH - LEVELS_ABOVE_AN_ENTRY).is_err() {
        return Ok(None);
    }
    let slice = JsonSlice {
        text: &text,
        offset: 0,
    };
    // What the load has to say was said when the file was read.
    let mut unused = Diagnostics::default();
    let mut sink = Sink::new(None, PERMISSIONS_FILE, &mut unused);
    Ok(Some(match read::object(slice) {
        Some(members) => net::read(slice, &members, &JsonPath::new(), &mut sink),
        None => ServerNet::Denied,
    }))
}

fn strings<'a>(items: impl IntoIterator<Item = &'a String>) -> Value {
    let mut array = VArray::new();
    for item in items {
        array.push(Value::from(item.clone()));
    }
    Value::from(array)
}

impl PackPermissions {
    /// The file as JSON text: `allowed_modules` when the file had the member or the list is
    /// not empty, `module_permissions` when it is not empty, `commands_permission_level`
    /// when set, then the unknown members.
    ///
    /// `module_permissions` is written as it is held, but for the entry of
    /// `@minecraft/server-net`, where [`server_net`](Self::server_net) decides: while the
    /// entry held still loads to it, the entry is written as its author wrote it; once the
    /// two differ (an edit of either), or without an entry, `server_net` is written as the
    /// entry, and `ServerNet::Unrestricted` as none. A held entry nested so deep that no
    /// file loads with it counts as differing. `ServerNet::Denied` over no entry, or
    /// over one that loads, has no JSON form and is refused, as is an unknown member named
    /// like a known one, and a setting of another module or an unknown member nested so
    /// deep that the file would not load.
    pub fn to_json(&self) -> Result<String, EncodeError> {
        const KNOWN: [&str; 3] = [
            "allowed_modules",
            "module_permissions",
            "commands_permission_level",
        ];
        let mut root = VObject::new();
        if self.allowed_modules_written || !self.allowed_modules.is_empty() {
            root.insert(KNOWN[0], strings(&self.allowed_modules));
        }
        // The server-net entry as held, when it still means `server_net`.
        let authored = match self.module_permissions.get(SERVER_NET_MODULE) {
            Some(settings) => {
                let entry = object(settings);
                (meaning(&entry)?.as_ref() == Some(&self.server_net)).then_some(entry)
            }
            None => None,
        };
        let net = match (authored, &self.server_net) {
            (Some(entry), _) => Some(entry),
            (None, ServerNet::Unrestricted) => None,
            (None, ServerNet::Configured(config)) => Some(config.to_value()),
            (None, ServerNet::Denied) => return Err(refused(&[KNOWN[1], SERVER_NET_MODULE])),
        };
        let mut modules = VObject::new();
        for (module, settings) in &self.module_permissions {
            if module != SERVER_NET_MODULE {
                modules.insert(module.as_str(), Value::from(object(settings)));
            } else if let Some(entry) = &net {
                // In the place its author gave it.
                modules.insert(SERVER_NET_MODULE, Value::from(entry.clone()));
            }
        }
        if let Some(entry) = net
            && !self.module_permissions.contains_key(SERVER_NET_MODULE)
        {
            modules.insert(SERVER_NET_MODULE, Value::from(entry));
        }
        if !modules.is_empty() {
            root.insert(KNOWN[1], Value::from(modules));
        }
        if let Some(level) = self.commands_permission_level {
            root.insert(KNOWN[2], Value::from(level.name().to_owned()));
        }
        extras(&mut root, &self.extra, &KNOWN)?;
        text(root)
    }
}

impl NetModuleConfig {
    /// The entry as a JSON object: the members that are set, in the model's order.
    fn to_value(&self) -> VObject {
        let number = |value: u64| VNumber::from_u64(value).into_value();
        let mut object = VObject::new();
        if let Some(uris) = &self.allowed_uris {
            object.insert("allowed_uris", strings(uris));
        }
        if let Some(force) = self.force_tls {
            object.insert("force_tls", Value::from(force));
        }
        if let Some(bytes) = self.max_body_bytes {
            object.insert("max_body_bytes", number(u64::from(bytes)));
        }
        if let Some(requests) = self.max_concurrent_requests {
            object.insert("max_concurrent_requests", number(u64::from(requests)));
        }
        if let Some(headers) = &self.session_headers {
            let mut map = VObject::new();
            for (name, header) in headers {
                map.insert(name.as_str(), Value::from(header.clone()));
            }
            object.insert("session_headers", Value::from(map));
        }
        if let Some(connections) = self.max_websocket_connections {
            object.insert("max_websocket_connections", number(u64::from(connections)));
        }
        if let Some(size) = self.max_websocket_payload_size {
            object.insert("max_websocket_payload_size", number(u64::from(size)));
        }
        object
    }
}

impl PackScriptConfig {
    /// `variables.json` as JSON text: every variable, in order. A variable nested so deep
    /// that the file would not load is refused, at the place that is too deep.
    pub fn variables_json(&self) -> Result<String, EncodeError> {
        let mut root = VObject::new();
        for (name, value) in &self.variables {
            root.insert(name.as_str(), value.clone());
        }
        text(root)
    }

    /// `secrets.json` as JSON text: every secret **with its value**, in order.
    pub fn secrets_json(&self) -> Result<String, EncodeError> {
        let mut root = VObject::new();
        for (name, secret) in &self.secrets {
            root.insert(name.as_str(), Value::from(secret.expose().to_owned()));
        }
        text(root)
    }
}

impl CommandsConfig {
    /// The file as JSON text: `default_permission_level` when set, `permission_levels` when
    /// not empty, then the unknown members. An unknown member named like a known one is
    /// refused, as is one nested so deep that the file would not load.
    pub fn to_json(&self) -> Result<String, EncodeError> {
        const KNOWN: [&str; 2] = ["default_permission_level", "permission_levels"];
        let mut root = VObject::new();
        if let Some(level) = self.default_permission_level {
            root.insert(KNOWN[0], Value::from(level.name().to_owned()));
        }
        if !self.permission_levels.is_empty() {
            let mut levels = VObject::new();
            for (command, level) in &self.permission_levels {
                levels.insert(command.as_str(), Value::from(level.name().to_owned()));
            }
            root.insert(KNOWN[1], Value::from(levels));
        }
        extras(&mut root, &self.extra, &KNOWN)?;
        text(root)
    }
}
