//! The `@minecraft/server-net` entry of `module_permissions`, which the server loads
//! strictly: an unknown member or a wrong type fails the entry. The entry derives its read
//! ([`Entry`]); what fails it is gathered into the one finding the server prints.

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostics, Severity};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::kit::{Node, Read};
use bedrock_addon_core::schema::{self, Schema, read_document};
use bedrock_addon_core::versioning::{NullHandling, Reading};

use super::SERVER_NET_MODULE;
use super::read::note;

/// What `@minecraft/server-net` may reach for the packs of one folder.
#[derive(Clone, Debug, Default, PartialEq, Eq, facet::Facet)]
#[repr(u8)]
pub enum ServerNet {
    /// No server-net entry: every URI is allowed.
    #[default]
    Unrestricted,
    /// The entry loaded.
    Configured(NetModuleConfig),
    /// The entry is there but failed to load: nothing is allowed. (The server ignores such an
    /// entry and leaves requests unrestricted; the crate denies them.)
    Denied,
}

impl ServerNet {
    /// Whether a request to `uri` passes the configuration (before any TLS rule).
    pub fn allows_uri(&self, uri: &str) -> bool {
        match self {
            Self::Unrestricted => true,
            Self::Configured(config) => config.allows_uri(uri),
            Self::Denied => false,
        }
    }

    /// Whether `http://` and `ws://` requests are refused.
    pub fn requires_tls(&self) -> bool {
        match self {
            Self::Unrestricted | Self::Denied => false,
            Self::Configured(config) => config.requires_tls(),
        }
    }
}

/// A loaded server-net entry (`script_net_module_config`).
///
/// JSON: `{"allowed_uris": [...], "force_tls": true, "max_body_bytes": 1,
/// "max_concurrent_requests": 1, "session_headers": {...}, "max_websocket_connections": 1,
/// "max_websocket_payload_size": 1}`; every member optional, any other member refused. The
/// legacy `force_https` reads as `force_tls`.
#[derive(Clone, Debug, Default, PartialEq, Eq, facet::Facet)]
#[doc(alias = "script_net_module_config")]
pub struct NetModuleConfig {
    /// The URIs a request may go to (JSON `allowed_uris`): each a plain prefix of the request
    /// URI, scheme included, without wildcards. `None` allows every URI; an empty list none.
    pub allowed_uris: Option<Vec<String>>,
    /// JSON `force_tls`. Whenever the member is there, `true` or `false`, the server refuses
    /// `http://` and `ws://`: see [`requires_tls`](Self::requires_tls).
    pub force_tls: Option<bool>,
    /// The most bytes a request body may have (JSON `max_body_bytes`); `None` leaves the
    /// engine's limit.
    pub max_body_bytes: Option<u32>,
    /// The most requests in flight at once (JSON `max_concurrent_requests`).
    pub max_concurrent_requests: Option<u32>,
    /// Headers sent with every request (JSON `session_headers`).
    pub session_headers: Option<IndexMap<String, String>>,
    /// The most open websocket connections (JSON `max_websocket_connections`).
    pub max_websocket_connections: Option<u16>,
    /// The largest websocket payload (JSON `max_websocket_payload_size`); engines from
    /// [`PAYLOAD_SIZE_SINCE`](super::PAYLOAD_SIZE_SINCE) know it, older ones refuse the entry.
    pub max_websocket_payload_size: Option<u32>,
}

impl NetModuleConfig {
    /// Whether `http://` and `ws://` requests are refused: true whenever `force_tls` was
    /// written, whatever its value.
    // The server refuses `http://` and `ws://` even for `force_tls: false`, answering with
    // a TLS-only error.
    pub fn requires_tls(&self) -> bool {
        self.force_tls.is_some()
    }

    /// Whether a request to `uri` passes `allowed_uris`: some entry is a prefix of it. The
    /// request's scheme compares in lower case.
    pub fn allows_uri(&self, uri: &str) -> bool {
        let Some(allowed) = &self.allowed_uris else {
            return true;
        };
        let lowered;
        let uri = match uri.find("://") {
            Some(end) if uri[..end].bytes().any(|byte| byte.is_ascii_uppercase()) => {
                lowered = format!("{}{}", uri[..end].to_ascii_lowercase(), &uri[end..]);
                lowered.as_str()
            }
            _ => uri,
        };
        allowed
            .iter()
            .any(|prefix| uri.starts_with(prefix.as_str()))
    }
}

const FORCE_TLS: &str = "force_tls";
const FORCE_HTTPS: &str = "force_https";
pub(super) const MAX_WEBSOCKET_PAYLOAD_SIZE: &str = "max_websocket_payload_size";

/// cereal's text for a member the schema lacks.
const UNKNOWN_MEMBER: &str = bedrock_addon_core::diagnostic::cereal::MEMBER_NOT_IN_SCHEMA_TEXT;
/// cereal's text for a wrong type and for a number out of range.
const INVALID_NUMERIC: &str = bedrock_addon_core::diagnostic::cereal::INVALID_NUMERIC_VALUE_TEXT;

/// The server's load of an entry: cereal's, strict, every failure an error and every member
/// told where the file has it; an unknown member is told (and then made a line of the
/// entry's error, [`load`]).
const LOAD: Reading = Reading::OWN
    .strict()
    .with_null(NullHandling::Rejected)
    .with_unknown(Severity::Error);

/// The entry as the server loads it (`script_net_module_config`). The legacy `force_https`
/// is copied over `force_tls` and removed before the load: where it stands, `force_tls` is
/// read as written and not used (`Tls` = `Value`), else as the flag (`Tls` = `bool`).
#[derive(facet::Facet, Schema)]
#[facet(schema::unknown = "report")]
struct Entry<Tls> {
    #[facet(schema::read_with = uris)]
    allowed_uris: Option<Vec<String>>,
    force_tls: Option<Tls>,
    #[facet(schema::reads_null, schema::read_with = legacy_force_tls)]
    force_https: Option<bool>,
    #[facet(schema::read_with = count_u32)]
    max_body_bytes: Option<u32>,
    #[facet(schema::read_with = count_u32)]
    max_concurrent_requests: Option<u32>,
    session_headers: Option<IndexMap<String, String>>,
    #[facet(schema::read_with = count_u16)]
    max_websocket_connections: Option<u16>,
    #[facet(schema::read_with = count_u32)]
    max_websocket_payload_size: Option<u32>,
}

impl<Tls> Entry<Tls> {
    /// The configuration, `force_tls` being `force_tls`.
    fn config(self, force_tls: Option<bool>) -> NetModuleConfig {
        NetModuleConfig {
            allowed_uris: self.allowed_uris,
            force_tls,
            max_body_bytes: self.max_body_bytes,
            max_concurrent_requests: self.max_concurrent_requests,
            session_headers: self.session_headers,
            max_websocket_connections: self.max_websocket_connections,
            max_websocket_payload_size: self.max_websocket_payload_size,
        }
    }
}

/// `force_https`: told deprecated, then read as the flag.
fn legacy_force_tls(node: &Node<'_>, read: &mut Read<'_>) -> Option<bool> {
    note(read, node, &core_codes::DEPRECATED_MEMBER);
    <bool as schema::FromJson>::from_json(node, read)
}

/// An unsigned integer of at most `max`.
fn count(node: &Node<'_>, read: &mut Read<'_>, max: u64) -> Option<u64> {
    let Some(number) = node.value().as_number() else {
        read.wrong_type(node);
        return None;
    };
    if number.is_float() {
        // An integer too wide for 64 bits reads as the double it spells, and a number
        // beyond every double as an infinity: a number out of range. Any other float is
        // not an integer.
        let wide = number.to_f64_lossy();
        if wide.is_infinite() || (wide.fract() == 0.0 && wide.abs() >= 18_446_744_073_709_551_616.0)
        {
            read.refuse(node, (&core_codes::INVALID_VALUE).into());
        } else {
            read.wrong_type(node);
        }
        return None;
    }
    let value = number.to_u64().filter(|value| *value <= max);
    if value.is_none() {
        read.refuse(node, (&core_codes::INVALID_VALUE).into());
    }
    value
}

fn count_u32(node: &Node<'_>, read: &mut Read<'_>) -> Option<u32> {
    count(node, read, u64::from(u32::MAX)).and_then(|value| u32::try_from(value).ok())
}

fn count_u16(node: &Node<'_>, read: &mut Read<'_>) -> Option<u16> {
    count(node, read, u64::from(u16::MAX)).and_then(|value| u16::try_from(value).ok())
}

/// `allowed_uris`: an array of strings.
fn uris(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<String>> {
    let elements = read.array(node)?;
    let mut uris = Vec::with_capacity(elements.len());
    let mut failed = false;
    for element in elements.iter() {
        let Some(uri) = read.string(&element) else {
            failed = true;
            continue;
        };
        if uri.contains('*') {
            // A literal prefix, not a wildcard.
            note(read, &element, &core_codes::INVALID_VALUE);
        }
        if !is_absolute(uri) {
            // Not checked by the server at load; such an entry admits no request (one
            // with a capital in its scheme included).
            note(read, &element, &core_codes::INVALID_VALUE);
        }
        uris.push(uri.to_owned());
    }
    if uris.is_empty() && !failed {
        // Every request is refused.
        note(read, node, &core_codes::NO_EFFECT);
    }
    (!failed).then_some(uris)
}

/// Whether `uri` starts with `http://`, `https://`, `ws://` or `wss://`, in lower case, and
/// has something after it. A scheme with a capital is none of them: a request's scheme
/// compares in lower case, so such an entry is the prefix of no request.
fn is_absolute(uri: &str) -> bool {
    uri.split_once("://").is_some_and(|(scheme, rest)| {
        !rest.is_empty() && ["http", "https", "ws", "wss"].contains(&scheme)
    })
}

/// Loads the entry `entry`, an object, telling to `outer` what the load says: each failure at
/// its member, then, when anything failed, the server's one error at the entry, a line per
/// failure (` -> <member>:  <message>`, the member as the server names it after the rename).
pub(super) fn load(entry: &Node<'_>, outer: &mut Read<'_>) -> ServerNet {
    let legacy = entry
        .value()
        .as_object()
        .is_some_and(|object| object.contains_key(FORCE_HTTPS));
    let mut found = Diagnostics::default();
    let config = {
        let mut read = Read::new(LOAD, &mut found);
        if legacy {
            read_document::<Entry<Value>>(entry, &mut read).map(|entry| {
                let force_tls = entry.force_https;
                entry.config(force_tls)
            })
        } else {
            read_document::<Entry<bool>>(entry, &mut read).map(|entry| {
                let force_tls = entry.force_tls;
                entry.config(force_tls)
            })
        }
    };
    let mut errors = Vec::new();
    for finding in &found {
        let member = finding
            .location
            .json_path
            .strip_prefix(entry.path())
            .and_then(|below| below.prefix(1).last_key().map(str::to_owned))
            .unwrap_or_default();
        let member = if member == FORCE_HTTPS {
            FORCE_TLS.to_owned()
        } else {
            member
        };
        if finding.is(&core_codes::UNRECOGNIZED_FIELD) {
            // An unknown member refuses the entry; the entry's error names it.
            errors.push(format!(" -> {member}:  {UNKNOWN_MEMBER}"));
            continue;
        }
        if finding.severity == Severity::Error {
            errors.push(format!(" -> {member}:  {INVALID_NUMERIC}"));
        }
        outer.push(finding.clone());
    }
    if !errors.is_empty() {
        // "Failed to load config for %s due to error(s): %s". The server warns
        // and leaves requests unrestricted; the crate denies them.
        let errors = errors.join("\n");
        let mut finding = bedrock_addon_core::diagnostic::Diagnostic::error(
            core_codes::NET_CONFIG_REJECTED.with([SERVER_NET_MODULE.to_owned(), errors]),
        )
        .at(entry.path().clone());
        finding.location.span = entry.span();
        outer.push(finding);
        return ServerNet::Denied;
    }
    let Some(config) = config else {
        return ServerNet::Denied;
    };
    if config.force_tls == Some(false) {
        // TLS is required whenever the member is present.
        let name = if legacy { FORCE_HTTPS } else { FORCE_TLS };
        if let Some(member) = outer.object(entry).and_then(|members| members.get(name)) {
            note(outer, &member, &core_codes::NO_EFFECT);
        }
    }
    ServerNet::Configured(config)
}
