//! Reading `permissions.json`: a hand walk like the server's, in which a member of the
//! wrong type counts as absent. The members are `commands_permission_level`,
//! `allowed_modules` and `module_permissions`.

use facet_json::RawJson;
use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{DiagnosticCode, JsonPath};
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_family_script::meta;

use super::read::{Sink, Twice, array, object, raw_value, span, string, value, with_root};
use super::{
    CommandPermissionLevel, PackPermissions, SERVER_NET_MODULE, ServerNet, WILDCARD_MODULE, net,
};

/// What one `permissions.json` gave.
#[derive(Default)]
pub(super) struct Read {
    pub permissions: PackPermissions,
    /// The file was read and has no `allowed_modules` member (of any type).
    pub lacks_allowed_modules: bool,
}

const ALLOWED_MODULES: &str = "allowed_modules";
const MODULE_PERMISSIONS: &str = "module_permissions";
const COMMANDS_PERMISSION_LEVEL: &str = "commands_permission_level";

/// Reads one `permissions.json`. A file that is refused gives nothing.
pub(super) fn read(bytes: &[u8], sink: &mut Sink<'_>) -> Read {
    let mut read = Read::default();
    let mut written = false;
    let parsed = with_root(bytes, Twice::Last, sink, |root, sink| {
        let at = JsonPath::new();
        let Some(members) = object(root) else {
            // Silent on the server, every member absent.
            sink.warn(
                DiagnosticCode::from(&core_codes::INVALID_TYPE),
                at,
                span(root),
            );
            return;
        };
        for (name, raw) in &members {
            let Some(slice) = root.nested(raw) else {
                continue;
            };
            let at = at.child(name);
            match name.as_str() {
                ALLOWED_MODULES => {
                    written = true;
                    read.permissions.allowed_modules = allowed_modules(slice, &at, sink);
                }
                MODULE_PERMISSIONS => module_permissions(slice, &at, sink, &mut read.permissions),
                COMMANDS_PERMISSION_LEVEL => {
                    read.permissions.commands_permission_level = level(slice, at, sink);
                }
                _ => {
                    // Ignored silently by the server.
                    sink.info(
                        DiagnosticCode::from(&core_codes::UNRECOGNIZED_FIELD),
                        at.clone(),
                        span(slice),
                    );
                    if let Some(value) = value(slice, &at, sink) {
                        read.permissions.extra.insert(name.clone(), value);
                    }
                }
            }
        }
    });
    read.permissions.allowed_modules_written = written;
    read.lacks_allowed_modules = parsed && !written;
    read
}

/// `allowed_modules`: the string elements of an array.
fn allowed_modules(slice: JsonSlice<'_>, at: &JsonPath, sink: &mut Sink<'_>) -> Vec<String> {
    let Some(elements) = array(slice) else {
        // Silent on the server, the list is empty.
        sink.warn(
            DiagnosticCode::from(&core_codes::INVALID_TYPE),
            at.clone(),
            span(slice),
        );
        return Vec::new();
    };
    let mut modules = Vec::with_capacity(elements.len());
    let mut wildcard = None;
    for (index, raw) in elements.iter().enumerate() {
        let Some(slice) = slice.nested(raw) else {
            continue;
        };
        let at = at.index(index);
        let Some(module) = string(slice) else {
            // The string elements still apply.
            sink.warn(
                DiagnosticCode::from(&core_codes::INVALID_TYPE),
                at,
                span(slice),
            );
            continue;
        };
        if module == WILDCARD_MODULE {
            wildcard.get_or_insert((at, span(slice)));
        } else if near_miss(&module) {
            // Matches nothing on the server.
            sink.warn(
                DiagnosticCode::from(&core_codes::INVALID_VALUE),
                at,
                span(slice),
            );
        }
        modules.push(module);
    }
    if let Some((at, span)) = wildcard
        && modules.len() > 1
    {
        // `*` allows every module, so the other elements say nothing.
        sink.info(DiagnosticCode::from(&core_codes::NO_EFFECT), at, span);
    }
    modules
}

/// Whether `module` names no module of the catalog but would with its `@<version>` suffix
/// removed or in lower case: the server compares the name byte for byte.
fn near_miss(module: &str) -> bool {
    if meta::module(module).is_some() {
        return false;
    }
    let unversioned = match module.rfind('@') {
        Some(at) if at > 0 => &module[..at],
        _ => module,
    };
    meta::module(unversioned).is_some() || meta::module(&unversioned.to_ascii_lowercase()).is_some()
}

/// `module_permissions`: module name → settings object.
fn module_permissions(
    slice: JsonSlice<'_>,
    at: &JsonPath,
    sink: &mut Sink<'_>,
    permissions: &mut PackPermissions,
) {
    let Some(modules) = object(slice) else {
        sink.warn(
            DiagnosticCode::from(&core_codes::INVALID_TYPE),
            at.clone(),
            span(slice),
        );
        return;
    };
    for (module, raw) in &modules {
        let Some(slice) = slice.nested(raw) else {
            continue;
        };
        let at = at.child(module);
        let is_net = module == SERVER_NET_MODULE;
        let Some(settings) = object(slice) else {
            // The module has no settings; silent on the server. A
            // server-net entry that cannot be read denies the pack's egress here
            // (a deliberate deviation; vanilla: unrestricted).
            sink.warn(
                DiagnosticCode::from(&core_codes::INVALID_TYPE),
                at,
                span(slice),
            );
            if is_net {
                permissions.server_net = ServerNet::Denied;
            }
            continue;
        };
        if is_net {
            permissions.server_net = net::read(slice, &settings, &at, sink);
        } else {
            // Kept, read by nothing.
            sink.info(
                DiagnosticCode::from(&core_codes::NO_EFFECT),
                at.clone(),
                span(slice),
            );
        }
        let settings = values_of(slice, &settings, &at, sink, is_net);
        permissions
            .module_permissions
            .insert(module.clone(), settings);
    }
}

/// The members of a settings object as untyped values. `loaded`: the entry was loaded as
/// well (server-net), and that load has said what there is to say about its numbers.
fn values_of(
    slice: JsonSlice<'_>,
    settings: &IndexMap<String, RawJson<'_>>,
    at: &JsonPath,
    sink: &mut Sink<'_>,
    loaded: bool,
) -> IndexMap<String, bedrock_addon_core::json::Value> {
    settings
        .iter()
        .filter_map(|(key, raw)| {
            let slice = slice.nested(raw)?;
            let value = if loaded {
                let mut value = raw_value(slice)?;
                bedrock_addon_core::json::clamp_non_finite(&mut value, None);
                value
            } else {
                value(slice, &at.child(key), sink)?
            };
            Some((key.clone(), value))
        })
        .collect()
}

/// `commands_permission_level`: one of the six names, else unset.
fn level(
    slice: JsonSlice<'_>,
    at: JsonPath,
    sink: &mut Sink<'_>,
) -> Option<CommandPermissionLevel> {
    let level = string(slice).and_then(|name| CommandPermissionLevel::from_name(&name));
    if level.is_none() {
        // Silently unset on the server.
        sink.warn(
            DiagnosticCode::from(&core_codes::INVALID_ENUM_VALUE),
            at,
            span(slice),
        );
    }
    level
}
