//! Reading `permissions.json`: the file derives its read ([`File`]), in which a member of the
//! wrong type counts as absent, as on the server. The members are `commands_permission_level`,
//! `allowed_modules` and `module_permissions`.

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::json::{self, Value};
use bedrock_addon_core::schema::{self, Schema, read_document, read_entries};
use bedrock_addon_family_script::meta;

use super::read::{Sink, folder_read, note, with_root};
use super::{
    CommandPermissionLevel, PackPermissions, SERVER_NET_MODULE, ServerNet, WILDCARD_MODULE, net,
};

/// What one `permissions.json` gave.
#[derive(Default)]
pub(super) struct Outcome {
    pub permissions: PackPermissions,
    /// The file was read and has no `allowed_modules` member (of any type).
    pub lacks_allowed_modules: bool,
}

/// `permissions.json` as written.
#[derive(Default, facet::Facet, Schema)]
struct File {
    /// `Some` whenever the file has the member, of whatever type.
    #[facet(schema::reads_null, schema::read_with = allowed_modules)]
    allowed_modules: Option<Vec<String>>,
    #[facet(schema::read_with = module_permissions)]
    module_permissions: Option<Modules>,
    // Silently unset on the server, whatever the value is.
    #[facet(schema::wrong_type = core_codes::INVALID_ENUM_VALUE)]
    commands_permission_level: Option<CommandPermissionLevel>,
    /// Ignored silently by the server.
    extra: UnknownMembers,
}

/// `module_permissions`: module name → settings, and what the server-net entry means.
#[derive(Default, facet::Facet)]
struct Modules {
    settings: IndexMap<String, IndexMap<String, Value>>,
    server_net: ServerNet,
}

bedrock_addon_core::impl_schema_leaf!(Modules);
bedrock_addon_core::impl_no_molang!(Modules);

/// Reads one `permissions.json`. A file that is refused gives nothing.
pub(super) fn read(bytes: &[u8], sink: &mut Sink<'_>) -> Outcome {
    let mut file = File::default();
    let parsed = with_root(bytes, json::Winner::Last, sink, |root, found| {
        let mut read = folder_read(found);
        // A root that is no object: silent on the server, every member absent.
        file = read_document(root, &mut read).unwrap_or_default();
    });
    let modules = file.module_permissions.unwrap_or_default();
    Outcome {
        lacks_allowed_modules: parsed && file.allowed_modules.is_none(),
        permissions: PackPermissions {
            allowed_modules_written: file.allowed_modules.is_some(),
            allowed_modules: file.allowed_modules.unwrap_or_default(),
            module_permissions: modules.settings,
            commands_permission_level: file.commands_permission_level,
            server_net: modules.server_net,
            extra: file.extra,
        },
    }
}

/// `allowed_modules`: the string elements of an array; another value is an empty list (silent
/// on the server).
#[allow(
    clippy::unnecessary_wraps,
    reason = "the `schema::read_with` signature"
)]
fn allowed_modules(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<String>> {
    let Some(elements) = read.array(node) else {
        return Some(Vec::new());
    };
    let mut modules = Vec::with_capacity(elements.len());
    let mut wildcard = None;
    for element in elements.iter() {
        // The string elements still apply.
        let Some(module) = read.string(&element) else {
            continue;
        };
        if module == WILDCARD_MODULE {
            wildcard.get_or_insert_with(|| element.clone());
        } else if near_miss(module) {
            // Matches nothing on the server.
            note(read, &element, &core_codes::INVALID_VALUE);
        }
        modules.push(module.to_owned());
    }
    if let Some(wildcard) = wildcard
        && modules.len() > 1
    {
        // `*` allows every module, so the other elements say nothing.
        note(read, &wildcard, &core_codes::NO_EFFECT);
    }
    Some(modules)
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

/// `module_permissions`: module name → settings object. An entry that is no object gives the
/// module no settings (silent on the server); a server-net entry that cannot be read denies the
/// pack's egress here (a deliberate deviation; vanilla: unrestricted).
#[allow(
    clippy::unnecessary_wraps,
    reason = "the `schema::read_with` signature"
)]
fn module_permissions(node: &Node<'_>, read: &mut Read<'_>) -> Option<Modules> {
    let mut modules = Modules::default();
    let Some(members) = read.object(node) else {
        return Some(modules);
    };
    for (module, entry) in members.iter() {
        let is_net = module == SERVER_NET_MODULE;
        let Some(settings) = read.object(&entry) else {
            if is_net {
                modules.server_net = ServerNet::Denied;
            }
            continue;
        };
        let settings = if is_net {
            modules.server_net = net::load(&entry, read);
            // The load has said what there is to say about its numbers.
            settings
                .iter()
                .map(|(key, value)| {
                    let mut value = value.value().clone();
                    json::clamp_non_finite(&mut value, None);
                    (key.to_owned(), value)
                })
                .collect()
        } else {
            // Kept, read by nothing.
            note(read, &entry, &core_codes::NO_EFFECT);
            read_entries(&entry, read, <Value as schema::FromJson>::from_json).unwrap_or_default()
        };
        modules.settings.insert(module.to_owned(), settings);
    }
    Some(modules)
}
