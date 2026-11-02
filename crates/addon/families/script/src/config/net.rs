//! The `@minecraft/server-net` entry of `module_permissions`, which the server loads
//! strictly: an unknown member or a wrong type fails the entry.

use facet_json::RawJson;
use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{DiagnosticCode, JsonPath, Severity};
use bedrock_addon_core::json::{JsonSlice, Value};

use super::SERVER_NET_MODULE;
use super::read::{Kind, Sink, array, child, element, kind, object, raw_value, span, string};

/// What `@minecraft/server-net` may reach for the packs of one folder.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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

const ALLOWED_URIS: &str = "allowed_uris";
const FORCE_TLS: &str = "force_tls";
const FORCE_HTTPS: &str = "force_https";
const MAX_BODY_BYTES: &str = "max_body_bytes";
const MAX_CONCURRENT_REQUESTS: &str = "max_concurrent_requests";
const SESSION_HEADERS: &str = "session_headers";
const MAX_WEBSOCKET_CONNECTIONS: &str = "max_websocket_connections";
pub(super) const MAX_WEBSOCKET_PAYLOAD_SIZE: &str = "max_websocket_payload_size";

/// cereal's text for a member the schema lacks.
const UNKNOWN_MEMBER: &str = bedrock_addon_core::diagnostic::cereal::MEMBER_NOT_IN_SCHEMA_TEXT;
/// cereal's text for a wrong type and for a number out of range.
const INVALID_NUMERIC: &str = bedrock_addon_core::diagnostic::cereal::INVALID_NUMERIC_VALUE_TEXT;

/// The state of one entry's load: the errors that refuse it.
struct Load<'s, 'd> {
    sink: &'s mut Sink<'d>,
    /// One line per error, as the server prints them: ` -> <member>:  <message>`.
    errors: Vec<String>,
}

impl Load<'_, '_> {
    /// A member whose value refuses the entry: a finding of its own at the member, and a
    /// line of the entry's error. `member` is the name the server reports it under.
    fn refuse(&mut self, code: DiagnosticCode, member: &str, at: JsonPath, slice: JsonSlice<'_>) {
        self.errors
            .push(format!(" -> {member}:  {INVALID_NUMERIC}"));
        self.sink.push(code, Severity::Error, at, span(slice));
    }

    fn wrong_type(&mut self, member: &str, at: JsonPath, slice: JsonSlice<'_>) {
        self.refuse(
            DiagnosticCode::InvalidType { text: None },
            member,
            at,
            slice,
        );
    }

    /// An unsigned integer of at most `max`.
    fn unsigned(
        &mut self,
        member: &str,
        at: JsonPath,
        slice: JsonSlice<'_>,
        max: u64,
    ) -> Option<u64> {
        let number = raw_value(slice);
        let Some(number) = number.as_ref().and_then(Value::as_number) else {
            self.wrong_type(member, at, slice);
            return None;
        };
        if number.is_float() {
            // An integer too wide for 64 bits reads as the double it spells, and a number
            // beyond every double as an infinity: a number out of range. Any other float is
            // not an integer.
            let wide = number.to_f64_lossy();
            if wide.is_infinite()
                || (wide.fract() == 0.0 && wide.abs() >= 18_446_744_073_709_551_616.0)
            {
                self.refuse(
                    DiagnosticCode::InvalidValue { text: None },
                    member,
                    at,
                    slice,
                );
            } else {
                self.wrong_type(member, at, slice);
            }
            return None;
        }
        let value = number.to_u64().filter(|value| *value <= max);
        if value.is_none() {
            self.refuse(
                DiagnosticCode::InvalidValue { text: None },
                member,
                at,
                slice,
            );
        }
        value
    }

    fn boolean(&mut self, member: &str, at: JsonPath, slice: JsonSlice<'_>) -> Option<bool> {
        match slice.text.trim() {
            "true" if kind(slice) == Kind::Bool => Some(true),
            "false" if kind(slice) == Kind::Bool => Some(false),
            _ => {
                self.wrong_type(member, at, slice);
                None
            }
        }
    }

    /// `allowed_uris`: an array of strings.
    fn uris(&mut self, at: &JsonPath, slice: JsonSlice<'_>) -> Option<Vec<String>> {
        let Some(elements) = array(slice) else {
            self.wrong_type(ALLOWED_URIS, at.clone(), slice);
            return None;
        };
        let mut uris = Vec::with_capacity(elements.len());
        let mut failed = false;
        for (index, raw) in elements.iter().enumerate() {
            let Some(slice) = slice.nested(raw) else {
                continue;
            };
            let at = element(at, index);
            let Some(uri) = string(slice) else {
                self.wrong_type(ALLOWED_URIS, at, slice);
                failed = true;
                continue;
            };
            if uri.contains('*') {
                // A literal prefix, not a wildcard.
                self.sink.warn(
                    DiagnosticCode::InvalidValue { text: None },
                    at.clone(),
                    span(slice),
                );
            }
            if !is_absolute(&uri) {
                // Not checked by the server at load; such an entry admits no request (one
                // with a capital in its scheme included).
                self.sink
                    .warn(DiagnosticCode::InvalidValue { text: None }, at, span(slice));
            }
            uris.push(uri);
        }
        if uris.is_empty() && !failed {
            // Every request is refused.
            self.sink.info(
                DiagnosticCode::NoEffect { text: None },
                at.clone(),
                span(slice),
            );
        }
        (!failed).then_some(uris)
    }

    /// `session_headers`: an object of strings.
    fn headers(&mut self, at: &JsonPath, slice: JsonSlice<'_>) -> Option<IndexMap<String, String>> {
        let Some(members) = object(slice) else {
            self.wrong_type(SESSION_HEADERS, at.clone(), slice);
            return None;
        };
        let mut headers = IndexMap::with_capacity(members.len());
        let mut failed = false;
        for (name, raw) in &members {
            let Some(slice) = slice.nested(raw) else {
                continue;
            };
            if let Some(header) = string(slice) {
                headers.insert(name.clone(), header);
            } else {
                self.wrong_type(SESSION_HEADERS, child(at, name), slice);
                failed = true;
            }
        }
        (!failed).then_some(headers)
    }
}

/// Whether `uri` starts with `http://`, `https://`, `ws://` or `wss://`, in lower case, and
/// has something after it. A scheme with a capital is none of them: a request's scheme
/// compares in lower case, so such an entry is the prefix of no request.
fn is_absolute(uri: &str) -> bool {
    uri.split_once("://").is_some_and(|(scheme, rest)| {
        !rest.is_empty() && ["http", "https", "ws", "wss"].contains(&scheme)
    })
}

/// Loads the entry `entry`, whose members are `members`, at the path `at`.
pub(super) fn read(
    entry: JsonSlice<'_>,
    members: &IndexMap<String, RawJson<'_>>,
    at: &JsonPath,
    sink: &mut Sink<'_>,
) -> ServerNet {
    let mut load = Load {
        sink,
        errors: Vec::new(),
    };
    let mut config = NetModuleConfig::default();
    // The legacy `force_https` is copied over `force_tls` and removed before the load.
    let legacy = members.contains_key(FORCE_HTTPS);
    for (name, raw) in members {
        let Some(slice) = entry.nested(raw) else {
            continue;
        };
        let here = child(at, name);
        match name.as_str() {
            ALLOWED_URIS => config.allowed_uris = load.uris(&here, slice),
            FORCE_TLS if legacy => {}
            FORCE_TLS => config.force_tls = load.boolean(FORCE_TLS, here, slice),
            FORCE_HTTPS => {
                load.sink.info(
                    DiagnosticCode::DeprecatedMember { text: None },
                    here.clone(),
                    span(slice),
                );
                config.force_tls = load.boolean(FORCE_TLS, here, slice);
            }
            MAX_BODY_BYTES => {
                config.max_body_bytes = load
                    .unsigned(name, here, slice, u64::from(u32::MAX))
                    .and_then(|value| u32::try_from(value).ok());
            }
            MAX_CONCURRENT_REQUESTS => {
                config.max_concurrent_requests = load
                    .unsigned(name, here, slice, u64::from(u32::MAX))
                    .and_then(|value| u32::try_from(value).ok());
            }
            SESSION_HEADERS => config.session_headers = load.headers(&here, slice),
            MAX_WEBSOCKET_CONNECTIONS => {
                config.max_websocket_connections = load
                    .unsigned(name, here, slice, u64::from(u16::MAX))
                    .and_then(|value| u16::try_from(value).ok());
            }
            MAX_WEBSOCKET_PAYLOAD_SIZE => {
                config.max_websocket_payload_size = load
                    .unsigned(name, here, slice, u64::from(u32::MAX))
                    .and_then(|value| u32::try_from(value).ok());
            }
            // An unknown member refuses the entry; the entry's error names it.
            _ => load.errors.push(format!(" -> {name}:  {UNKNOWN_MEMBER}")),
        }
    }
    if !load.errors.is_empty() {
        // "Failed to load config for %s due to error(s): %s". The server warns
        // and leaves requests unrestricted; the crate denies them.
        let errors = load.errors.join("\n");
        load.sink.push(
            DiagnosticCode::NetConfigRejected {
                module: SERVER_NET_MODULE.to_owned(),
                errors,
            },
            Severity::Error,
            at.clone(),
            span(entry),
        );
        return ServerNet::Denied;
    }
    if config.force_tls == Some(false) {
        // TLS is required whenever the member is present.
        let name = if legacy { FORCE_HTTPS } else { FORCE_TLS };
        let slice = members.get(name).and_then(|raw| entry.nested(raw));
        load.sink.warn(
            DiagnosticCode::NoEffect { text: None },
            child(at, name),
            slice.and_then(span),
        );
    }
    ServerNet::Configured(config)
}
