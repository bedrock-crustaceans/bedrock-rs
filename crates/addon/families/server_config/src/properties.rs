//! `server.properties`: the line format the server reads, and the typed view of its script
//! keys.
// The server writes no content-log line about problems in the file (it only logs
// `Contents of server.properties: %s`), so every note of the reader and of the
// script keys is an information of the crate's own; the values the server refuses with its
// own text stay errors.

use crate::codes;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, Span,
};
use bedrock_addon_core::json::{EncodeError, Unrepresentable};
use bedrock_addon_core::pack_identity::PackPath;

use super::{PROPERTIES_FILE, PROPERTIES_LINE_MAX};

/// One `server.properties`: every line the server applies, in file order.
///
/// The format is lines of `key=value`. A line that starts with a space or `#`, or is shorter
/// than two bytes, is skipped; `#` is a comment marker nowhere else. The key is the text
/// before the first `=` and the value the text between the first and the second `=`, each
/// without its trailing white space; a line whose value is empty is not applied. Keys are
/// case-sensitive, and a key written twice is applied twice: the last one stands.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "server.properties")]
pub struct ServerProperties {
    /// Every applied `key=value`, in file order, whatever the key.
    pub entries: Vec<PropertyLine>,
    diagnostics: Diagnostics,
    /// Where each line that gave an entry lies in the file, by line number, ascending.
    spans: Vec<(u32, Span)>,
}

/// One applied line of `server.properties`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertyLine {
    /// The key: the text before the first `=`, without trailing white space (leading white
    /// space is part of it, and such a key matches nothing).
    pub key: String,
    /// The value: the text between the first and the second `=`, without trailing white
    /// space; never empty.
    pub value: String,
    /// The line's number in the file, from 1.
    pub line: u32,
}

impl PropertyLine {
    /// The line `key=value` at line `line` of the file (from 1).
    pub fn new(key: impl Into<String>, value: impl Into<String>, line: u32) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            line,
        }
    }
}

/// The white space C's `isspace` knows: what the server trims.
const SPACE: [char; 6] = [' ', '\t', '\n', '\r', '\x0b', '\x0c'];

/// Every key of the server's file: those of the `server.properties` the dedicated server ships
/// (set or commented out) and the three more the family folder names. Sorted.
const KNOWN_KEYS: &[&str] = &[
    "allow-cheats",
    "allow-inbound-script-debugging",
    "allow-list",
    "allow-outbound-script-debugging",
    "allow-player-joining",
    "block-network-ids-are-hashes",
    "chat-restriction",
    "client-side-chunk-generation-enabled",
    "compression-algorithm",
    "compression-threshold",
    "content-log-console-output-enabled",
    "content-log-file-enabled",
    "content-log-level",
    "convert-world-to-editor-project",
    "default-player-permission-level",
    "diagnostics-capture-auto-start",
    "diagnostics-capture-max-file-size",
    "diagnostics-capture-max-files",
    "difficulty",
    "disable-client-vibrant-visuals",
    "disable-custom-skins",
    "disable-persona",
    "disable-player-interaction",
    "emit-server-telemetry",
    "enable-editor-network-metrics",
    "enable-lan-visibility",
    "enable-packet-rate-limiter",
    "enable-profiler",
    "enable-script",
    "force-gamemode",
    "force-inbound-debug-port",
    "gamemode",
    "level-name",
    "level-seed",
    "max-players",
    "max-threads",
    "online-mode",
    "player-idle-timeout",
    "player-movement-action-direction-threshold",
    "player-position-acceptance-threshold",
    "script-debugger-auto-attach",
    "script-debugger-auto-attach-connect-address",
    "script-debugger-auto-attach-timeout",
    "script-debugger-passcode",
    "script-watchdog-enable",
    "script-watchdog-enable-exception-handling",
    "script-watchdog-enable-shutdown",
    "script-watchdog-hang-exception",
    "script-watchdog-hang-threshold",
    "script-watchdog-memory-limit",
    "script-watchdog-memory-warning",
    "script-watchdog-slow-threshold",
    "script-watchdog-spike-threshold",
    "sentry-max-events-per-window",
    "sentry-rate-limit-window",
    "server-authoritative-block-breaking-pick-range-scalar",
    "server-authoritative-dismount-strict",
    "server-authoritative-entity-interactions-strict",
    "server-authoritative-movement-strict",
    "server-build-radius-ratio",
    "server-ip",
    "server-name",
    "server-port",
    "server-portv6",
    "server-udp-ports",
    "texturepack-required",
    "tick-distance",
    "transport",
    "view-distance",
];

/// Whether the server's file has a key `key`, exactly.
fn is_known(key: &str) -> bool {
    KNOWN_KEYS.binary_search(&key).is_ok()
}

fn location(key: Option<&str>, span: Option<Span>) -> Location {
    let mut path = JsonPath::new();
    if let Some(key) = key {
        path.push(key);
    }
    Location::new(
        None,
        Some(PackPath::from_static(PROPERTIES_FILE)),
        path,
        span,
    )
}

fn finding(
    code: DiagnosticCode,
    severity: Severity,
    key: Option<&str>,
    span: Option<Span>,
) -> Diagnostic {
    Diagnostic::new(code, severity, Phase::Parse, location(key, span))
}

impl ServerProperties {
    /// The file's name, in the server's working directory.
    pub const FILE: &'static str = PROPERTIES_FILE;

    /// Parses the bytes of a `server.properties`. Nothing fails: a line the server does not
    /// apply gives no entry, and what is worth knowing about it is in
    /// [`diagnostics`](Self::diagnostics).
    ///
    /// Differences from the server: a line longer than [`PROPERTIES_LINE_MAX`] bytes is an
    /// error and the lines after it are still read (the server stops reading there); bytes
    /// that are not UTF-8 are replaced by U+FFFD with an information.
    pub fn parse(bytes: &[u8]) -> ServerProperties {
        let mut properties = ServerProperties::default();
        let mut offset = 0;
        let mut number = 0_u32;
        // A file that ends in a newline has no line after it.
        let body = bytes.strip_suffix(b"\n").unwrap_or(bytes);
        if bytes.is_empty() {
            return properties;
        }
        for line in body.split(|byte| *byte == b'\n') {
            number = number.saturating_add(1);
            let span = Span {
                offset: u32::try_from(offset).unwrap_or(u32::MAX),
                len: u32::try_from(line.len()).unwrap_or(u32::MAX),
            };
            offset += line.len() + 1;
            properties.line(line, number, span);
        }
        properties
    }

    /// What the file said while it was read.
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    /// The value the server ends with for `key`: that of its last applied line.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .rev()
            .find(|entry| entry.key == key)
            .map(|entry| entry.value.as_str())
    }

    fn report(&mut self, code: DiagnosticCode, severity: Severity, key: Option<&str>, span: Span) {
        self.diagnostics
            .push(finding(code, severity, key, Some(span)));
    }

    /// A line led by a space: the server skips it. R3: worth an information when it would be a property
    /// otherwise; with an empty value it would be none, as with a tab-led line.
    fn skipped_space_led(&mut self, text: &str, span: Span) {
        if let Some((key, rest)) = text.trim_start_matches(SPACE).split_once('=')
            && !key.starts_with('#')
            && !rest
                .split('=')
                .next()
                .unwrap_or_default()
                .trim_matches(SPACE)
                .is_empty()
        {
            let key = key.trim_end_matches(SPACE);
            self.report(
                DiagnosticCode::from(&core_codes::NO_EFFECT),
                Severity::Info,
                is_known(key).then_some(key),
                span,
            );
        }
    }

    /// One line, without its `\n`.
    fn line(&mut self, line: &[u8], number: u32, span: Span) {
        if line.len() > PROPERTIES_LINE_MAX {
            // The server reads a line into a 4096-byte buffer; the crate assumes it stops
            // reading here.
            let code = DiagnosticCode::from(bedrock_addon_core::diagnostic::builders::syntax(
                format!("a line longer than {PROPERTIES_LINE_MAX} bytes"),
            ));
            self.report(code, Severity::Error, None, span);
            return;
        }
        // A line shorter than two bytes or starting with `#` is skipped.
        if line.len() < 2 || line[0] == b'#' {
            return;
        }
        // A line of white space only is a blank line: its key and its value are empty once
        // trimmed, so the server applies nothing, and nothing in it was meant as a property.
        if line
            .iter()
            .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | 0x0b | 0x0c))
        {
            return;
        }
        let text = String::from_utf8_lossy(line);
        if matches!(text, std::borrow::Cow::Owned(_)) {
            // The server works on bytes; the crate reads the line lossily and says so.
            let code = DiagnosticCode::from(core_codes::SYNTAX.with(["the line is not UTF-8"]));
            self.report(code, Severity::Info, None, span);
        }
        if line[0] == b' ' {
            self.skipped_space_led(&text, span);
            return;
        }
        // The key ends at the first `=`, the value at the second.
        let Some((key, rest)) = text.split_once('=') else {
            // The whole line is a key without a value, so nothing is applied.
            self.report(
                DiagnosticCode::from(&core_codes::NO_EFFECT),
                Severity::Info,
                None,
                span,
            );
            return;
        };
        let (value, dropped) = match rest.split_once('=') {
            Some((value, _)) => (value, true),
            None => (rest, false),
        };
        // White space around the key and the value is trimmed.
        let key = key.trim_end_matches(SPACE);
        let value = value.trim_end_matches(SPACE);
        if dropped {
            // R5: the text after a second `=` is dropped.
            self.report(
                DiagnosticCode::from(&core_codes::NO_EFFECT),
                Severity::Info,
                is_known(key).then_some(key),
                span,
            );
        }
        // An empty value leaves the key as it was.
        if value.is_empty() {
            return;
        }
        if !is_known(key) {
            let unled = key.trim_start_matches(SPACE);
            if unled.len() < key.len() {
                // R3: a key led by a tab matches nothing.
                self.report(
                    DiagnosticCode::from(&core_codes::NO_EFFECT),
                    Severity::Info,
                    is_known(unled).then_some(unled),
                    span,
                );
            } else {
                // R4: ignored silently by the server, and a known key in another case alike
                // (likely a mistake, but the server says nothing: an information either way).
                self.report(
                    DiagnosticCode::from(&core_codes::UNRECOGNIZED_FIELD),
                    Severity::Info,
                    None,
                    span,
                );
            }
        }
        self.spans.push((number, span));
        self.entries.push(PropertyLine {
            key: key.to_owned(),
            value: value.to_owned(),
            line: number,
        });
    }

    /// Where the line of `entry` lies in the file this was parsed from.
    fn span_of(&self, entry: &PropertyLine) -> Option<Span> {
        let at = self
            .spans
            .binary_search_by_key(&entry.line, |(line, _)| *line)
            .ok()?;
        Some(self.spans[at].1)
    }

    /// The file as text: one `key=value` line per entry, in order. Comments, blank lines and
    /// the lines the server skips are not kept, and every line ends with a line break, also
    /// the last one of a file that was read without one.
    ///
    /// An entry the format cannot hold is refused, with its index and `key` or `value` as the
    /// path: an empty value, a `=` or a line break in either, trailing white space in either,
    /// a key that starts with a space or `#`, a line longer than [`PROPERTIES_LINE_MAX`]
    /// bytes.
    ///
    /// # Errors
    ///
    /// Returns [`EncodeError::NotRepresentable`] for the first entry the format cannot hold.
    pub fn to_text(&self) -> Result<String, EncodeError> {
        let mut out = String::new();
        for (index, entry) in self.entries.iter().enumerate() {
            let refuse = |member: &str| {
                let at = JsonPath::new().index(index).child(member);
                EncodeError::NotRepresentable {
                    at,
                    what: Unrepresentable::Unsupported,
                }
            };
            let whole = |text: &str| !text.contains(['=', '\n']) && !text.ends_with(SPACE);
            if !whole(&entry.key) || entry.key.starts_with([' ', '#']) {
                return Err(refuse("key"));
            }
            let length = entry.key.len() + 1 + entry.value.len();
            if !whole(&entry.value) || entry.value.is_empty() || length > PROPERTIES_LINE_MAX {
                return Err(refuse("value"));
            }
            out.push_str(&entry.key);
            out.push('=');
            out.push_str(&entry.value);
            out.push('\n');
        }
        Ok(out)
    }

    /// The script keys, typed, with the server's defaults for the keys the file leaves out.
    /// Every line of a key is checked in file order and the last valid one stands; what the
    /// values said goes to `diagnostics`.
    pub fn script(&self, diagnostics: &mut Diagnostics) -> ScriptProperties {
        let mut read = Typed {
            properties: ScriptProperties::default(),
            d: diagnostics,
            span: None,
        };
        for entry in &self.entries {
            read.span = self.span_of(entry);
            read.apply(&entry.key, &entry.value);
        }
        read.auto_attach_needs();
        read.properties
    }
}

/// The script-related keys of `server.properties`, typed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptProperties {
    /// `enable-script`: scripting as a whole. Default true.
    pub enable_script: bool,
    /// The `*-script-debugging` and `script-debugger-*` keys.
    pub debugger: DebuggerProperties,
    /// The `script-watchdog-*` keys.
    pub watchdog: WatchdogProperties,
    /// The `diagnostics-capture-*` keys.
    pub diagnostics: DiagnosticsCaptureProperties,
    /// The `sentry-*` keys.
    pub sentry: SentryProperties,
    /// The `content-log-*` keys.
    pub content_log: ContentLogProperties,
    /// `emit-server-telemetry`. Default false.
    pub emit_server_telemetry: bool,
    /// `enable-profiler`; `None` leaves the server's own default.
    pub enable_profiler: Option<bool>,
    /// `enable-editor-network-metrics`; `None` leaves the server's own default.
    pub enable_editor_network_metrics: Option<bool>,
    /// `convert-world-to-editor-project`. Default false.
    pub convert_world_to_editor_project: bool,
}

impl Default for ScriptProperties {
    fn default() -> Self {
        Self {
            enable_script: true,
            debugger: DebuggerProperties::default(),
            watchdog: WatchdogProperties::default(),
            diagnostics: DiagnosticsCaptureProperties::default(),
            sentry: SentryProperties::default(),
            content_log: ContentLogProperties::default(),
            emit_server_telemetry: false,
            enable_profiler: None,
            enable_editor_network_metrics: None,
            convert_world_to_editor_project: false,
        }
    }
}

/// The script debugger's keys.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DebuggerProperties {
    /// `allow-outbound-script-debugging`: the debugger's `connect`. Default false.
    pub allow_outbound: bool,
    /// `allow-inbound-script-debugging`: the debugger's `listen`. Default false.
    pub allow_inbound: bool,
    /// `force-inbound-debug-port`, 1 to 65535; `None` uses 19144.
    pub force_inbound_port: Option<u16>,
    /// `script-debugger-auto-attach`. Default `disabled`.
    pub auto_attach: AutoAttach,
    /// `script-debugger-auto-attach-connect-address`: `host:port`.
    pub connect_address: Option<(String, u16)>,
    /// `script-debugger-auto-attach-timeout`, in seconds. Default 0.
    pub timeout_sec: u32,
    /// `script-debugger-passcode`.
    pub passcode: Option<String>,
}

/// `script-debugger-auto-attach`: whether the debugger attaches when the level loads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AutoAttach {
    /// `disabled`.
    #[default]
    Disabled,
    /// `connect`: the server connects to a debugger that listens.
    Connect,
    /// `listen`: the server listens for a debugger.
    Listen,
}

impl AutoAttach {
    /// The mode's word in the file.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Connect => "connect",
            Self::Listen => "listen",
        }
    }

    /// The mode called `name`, exactly (case-sensitive).
    pub fn from_name(name: &str) -> Option<Self> {
        [Self::Disabled, Self::Connect, Self::Listen]
            .into_iter()
            .find(|mode| mode.name() == name)
    }
}

/// The script watchdog's keys.
#[allow(
    clippy::struct_excessive_bools,
    reason = "one member per key of the file, each a switch of its own"
)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WatchdogProperties {
    /// `script-watchdog-enable`. Default true.
    pub enable: bool,
    /// `script-watchdog-enable-exception-handling`. Default true.
    pub exception_handling: bool,
    /// `script-watchdog-enable-shutdown`. Default true.
    pub shutdown: bool,
    /// `script-watchdog-hang-exception`. Default true.
    pub hang_exception: bool,
    /// `script-watchdog-hang-threshold`, in milliseconds. Default 10000.
    pub hang_ms: i32,
    /// `script-watchdog-spike-threshold`, in milliseconds; `None` gives no spike warnings.
    pub spike_ms: Option<i32>,
    /// `script-watchdog-slow-threshold`, in milliseconds; `None` gives no slow warnings.
    pub slow_ms: Option<i32>,
    /// `script-watchdog-memory-warning`, in megabytes; 0 disables it. Default 100, at most
    /// 2000.
    pub memory_warning_mb: i32,
    /// `script-watchdog-memory-limit`, in megabytes; 0 disables it. Default 250, at most 2000.
    pub memory_limit_mb: i32,
}

impl Default for WatchdogProperties {
    fn default() -> Self {
        Self {
            enable: true,
            exception_handling: true,
            shutdown: true,
            hang_exception: true,
            hang_ms: 10_000,
            spike_ms: None,
            slow_ms: None,
            memory_warning_mb: 100,
            memory_limit_mb: 250,
        }
    }
}

/// The `diagnostics-capture-*` keys.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticsCaptureProperties {
    /// `diagnostics-capture-auto-start`. Default false.
    pub auto_start: bool,
    /// `diagnostics-capture-max-files`. Default 5.
    pub max_files: u32,
    /// `diagnostics-capture-max-file-size`, in bytes. Default 2097152.
    pub max_file_size: u32,
}

impl Default for DiagnosticsCaptureProperties {
    fn default() -> Self {
        Self {
            auto_start: false,
            max_files: 5,
            max_file_size: 2_097_152,
        }
    }
}

/// The `sentry-*` keys.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SentryProperties {
    /// `sentry-rate-limit-window`, in seconds; 0 is no limit. `None` leaves the server's
    /// default (60).
    pub window_s: Option<u32>,
    /// `sentry-max-events-per-window`; 0 sends none. `None` leaves the server's default
    /// (10).
    pub max_events: Option<u32>,
}

/// The `content-log-*` keys.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ContentLogProperties {
    /// `content-log-file-enabled`. Default false.
    pub file: bool,
    /// `content-log-console-output-enabled`. Default false.
    pub console: bool,
    /// `content-log-level`. Default `info`.
    pub level: ContentLogLevel,
}

/// `content-log-level`: the lowest level the content log keeps.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ContentLogLevel {
    /// `verbose`.
    Verbose,
    /// `info`.
    #[default]
    Info,
    /// `warning`.
    Warning,
    /// `error`.
    Error,
}

impl ContentLogLevel {
    /// The level's word in the file.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Verbose => "verbose",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }

    /// The level called `name`, exactly (case-sensitive).
    pub fn from_name(name: &str) -> Option<Self> {
        [Self::Verbose, Self::Info, Self::Warning, Self::Error]
            .into_iter()
            .find(|level| level.name() == name)
    }
}

/// The most megabytes the memory keys take, by the comments of the `server.properties` the
/// dedicated server ships.
const MEMORY_MAX_MB: i32 = 2000;

/// What C's `atoi` reads: leading white space, a sign, digits.
struct Atoi {
    /// The value, held to `i32`.
    value: i32,
    /// The digits spell a number outside `i32`.
    overflow: bool,
    /// There was at least one digit.
    digits: bool,
    /// Something follows the digits.
    trailing: bool,
}

fn atoi(text: &str) -> Atoi {
    let text = text.trim_start_matches(SPACE);
    let (negative, digits) = match text.as_bytes().first() {
        Some(b'-') => (true, &text[1..]),
        Some(b'+') => (false, &text[1..]),
        _ => (false, text),
    };
    let end = digits
        .bytes()
        .position(|byte| !byte.is_ascii_digit())
        .unwrap_or(digits.len());
    let mut value = 0_i64;
    for byte in digits[..end].bytes() {
        value = (value * 10 + i64::from(byte - b'0')).min(i64::from(u32::MAX) * 2);
    }
    if negative {
        value = -value;
    }
    let held = i32::try_from(value).unwrap_or(if negative { i32::MIN } else { i32::MAX });
    Atoi {
        value: held,
        overflow: i64::from(held) != value,
        digits: end > 0,
        trailing: end < digits.len(),
    }
}

/// The part of a value before a `#`, when it has one: what the author likely meant.
fn before_hash(value: &str) -> Option<&str> {
    value
        .split_once('#')
        .map(|(head, _)| head.trim_end_matches(SPACE))
}

/// The state of one typed read.
struct Typed<'d> {
    properties: ScriptProperties,
    d: &'d mut Diagnostics,
    /// The line being applied.
    span: Option<Span>,
}

impl Typed<'_> {
    fn push(&mut self, code: DiagnosticCode, severity: Severity, key: &str) {
        self.d.push(Diagnostic::new(
            code,
            severity,
            Phase::Validate,
            location(Some(key), self.span),
        ));
    }

    fn invalid(&mut self, key: &str) {
        self.push(
            DiagnosticCode::from(&core_codes::INVALID_VALUE),
            Severity::Info,
            key,
        );
    }

    /// Exactly `true` is true, every other value false. R6: an information unless the value
    /// is `true` or `false`; R2 when a `#` follows a valid value.
    fn boolean(&mut self, key: &str, value: &str) -> bool {
        if !matches!(value, "true" | "false") {
            self.invalid(key);
        }
        value == "true"
    }

    /// Read as C's `atoi` does. R7: an information for trailing text or no digits (R2's `#` among them); a
    /// number outside `i32` is held to it with an information.
    fn integer(&mut self, key: &str, value: &str) -> i32 {
        let read = atoi(value);
        if read.trailing || !read.digits {
            self.invalid(key);
        }
        if read.overflow {
            self.push(
                DiagnosticCode::from(&core_codes::CLAMPED_VALUE),
                Severity::Info,
                key,
            );
        }
        read.value
    }

    /// An integer that cannot be negative; a negative one is an information and leaves the key
    /// as it was.
    fn unsigned(&mut self, key: &str, value: &str) -> Option<u32> {
        let read = self.integer(key, value);
        let value = u32::try_from(read).ok();
        if value.is_none() {
            self.invalid(key);
        }
        value
    }

    /// R8: a negative threshold is used as it is, with an information.
    fn threshold(&mut self, key: &str, value: &str) -> i32 {
        let read = self.integer(key, value);
        if read < 0 {
            self.invalid(key);
        }
        read
    }

    /// R9: a memory key above 2000 is held to it.
    fn memory(&mut self, key: &str, value: &str) -> i32 {
        let read = self.threshold(key, value);
        if read > MEMORY_MAX_MB {
            self.push(
                DiagnosticCode::from(&core_codes::CLAMPED_VALUE),
                Severity::Info,
                key,
            );
        }
        read.min(MEMORY_MAX_MB)
    }

    /// A value the server refuses with `template`: an error, and the key stays as it was.
    fn refuse(&mut self, code: DiagnosticCode, key: &str) {
        self.push(code, Severity::Error, key);
    }

    /// R2: `#` is no comment after `=`. An information when the text before it would have been
    /// accepted.
    fn hash(&mut self, key: &str, value: &str, valid: impl Fn(&str) -> bool) {
        if before_hash(value).is_some_and(valid) {
            self.invalid(key);
        }
    }

    /// Whether `host` can be the host of a `host:port` address: a name or an address
    /// without white space, with a `:` of its own only as a bracketed IPv6 literal
    /// (`[::1]`). Without the brackets nothing tells the last group of an address from the
    /// port, so a bare IPv6 address is no `host:port`; and the brackets hold an IPv6 address
    /// and nothing else (`[localhost]` is no host).
    // The server's text only says the format is `host:port`; what counts as a host is
    // the crate's own reading of that.
    fn is_host(host: &str) -> bool {
        if host.is_empty() || host.contains(char::is_whitespace) {
            return false;
        }
        match host
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        {
            Some(address) => Self::is_ipv6(address),
            None => !host.contains([':', '[', ']']),
        }
    }

    /// Whether `address` is what the brackets of a host may hold: an IPv6 address in one of
    /// its text forms, with an optional scope after `%`.
    fn is_ipv6(address: &str) -> bool {
        let (address, scope) = match address.split_once('%') {
            Some((address, scope)) => (address, Some(scope)),
            None => (address, None),
        };
        address.parse::<std::net::Ipv6Addr>().is_ok()
            && scope.is_none_or(|scope| !scope.is_empty() && !scope.contains(['[', ']', '%']))
    }

    /// The port of a `host:port` address: decimal digits and nothing else, in 1–65535. A
    /// sign or white space before the digits is no port (unlike the integer keys, which
    /// read as `atoi` does).
    fn port(text: &str) -> Option<u16> {
        if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        // Leading zeros do not count towards the number's length.
        let digits = text.trim_start_matches('0');
        if digits.is_empty() {
            return None;
        }
        digits.parse::<u16>().ok()
    }

    #[allow(
        clippy::too_many_lines,
        reason = "one arm per key of the schema's table, in its order"
    )]
    fn apply(&mut self, key: &str, value: &str) {
        match key {
            "enable-script" => self.properties.enable_script = self.boolean(key, value),
            "allow-outbound-script-debugging" => {
                self.properties.debugger.allow_outbound = self.boolean(key, value);
            }
            "allow-inbound-script-debugging" => {
                self.properties.debugger.allow_inbound = self.boolean(key, value);
            }
            "force-inbound-debug-port" => {
                // R11.
                let read = atoi(value);
                match u16::try_from(read.value).ok().filter(|port| *port != 0) {
                    Some(port) if read.digits => {
                        if read.trailing {
                            self.invalid(key);
                        }
                        self.properties.debugger.force_inbound_port = Some(port);
                    }
                    _ => {
                        self.refuse(DiagnosticCode::from(&codes::BAD_PORT), key);
                    }
                }
            }
            "script-debugger-auto-attach" => {
                if let Some(mode) = AutoAttach::from_name(value) {
                    self.properties.debugger.auto_attach = mode;
                } else {
                    // R10.
                    self.refuse(
                        DiagnosticCode::from(codes::BAD_AUTO_ATTACH.with([value])),
                        key,
                    );
                    self.hash(key, value, |head| AutoAttach::from_name(head).is_some());
                }
            }
            "script-debugger-auto-attach-connect-address" => {
                // R12.
                let address = value
                    .rsplit_once(':')
                    .filter(|(host, _)| Self::is_host(host))
                    .and_then(|(host, port)| Some((host.to_owned(), Self::port(port)?)));
                if address.is_some() {
                    self.properties.debugger.connect_address = address;
                } else {
                    self.refuse(DiagnosticCode::from(&codes::BAD_ADDRESS), key);
                }
            }
            "script-debugger-auto-attach-timeout" => {
                if let Some(seconds) = self.unsigned(key, value) {
                    self.properties.debugger.timeout_sec = seconds;
                }
            }
            "script-debugger-passcode" => {
                self.properties.debugger.passcode = Some(value.to_owned());
            }
            "script-watchdog-enable" => self.properties.watchdog.enable = self.boolean(key, value),
            "script-watchdog-enable-exception-handling" => {
                self.properties.watchdog.exception_handling = self.boolean(key, value);
            }
            "script-watchdog-enable-shutdown" => {
                self.properties.watchdog.shutdown = self.boolean(key, value);
            }
            "script-watchdog-hang-exception" => {
                self.properties.watchdog.hang_exception = self.boolean(key, value);
            }
            "script-watchdog-hang-threshold" => {
                self.properties.watchdog.hang_ms = self.threshold(key, value);
            }
            "script-watchdog-spike-threshold" => {
                self.properties.watchdog.spike_ms = Some(self.threshold(key, value));
            }
            "script-watchdog-slow-threshold" => {
                self.properties.watchdog.slow_ms = Some(self.threshold(key, value));
            }
            "script-watchdog-memory-warning" => {
                self.properties.watchdog.memory_warning_mb = self.memory(key, value);
            }
            "script-watchdog-memory-limit" => {
                self.properties.watchdog.memory_limit_mb = self.memory(key, value);
            }
            "diagnostics-capture-auto-start" => {
                self.properties.diagnostics.auto_start = self.boolean(key, value);
            }
            "diagnostics-capture-max-files" => {
                if let Some(files) = self.unsigned(key, value) {
                    self.properties.diagnostics.max_files = files;
                }
            }
            "diagnostics-capture-max-file-size" => {
                if let Some(size) = self.unsigned(key, value) {
                    self.properties.diagnostics.max_file_size = size;
                }
            }
            "sentry-rate-limit-window" => {
                if let Some(window) = self.unsigned(key, value) {
                    self.properties.sentry.window_s = Some(window);
                }
            }
            "sentry-max-events-per-window" => {
                if let Some(events) = self.unsigned(key, value) {
                    self.properties.sentry.max_events = Some(events);
                }
            }
            "content-log-file-enabled" => {
                self.properties.content_log.file = self.boolean(key, value);
            }
            "content-log-console-output-enabled" => {
                self.properties.content_log.console = self.boolean(key, value);
            }
            "content-log-level" => {
                if let Some(level) = ContentLogLevel::from_name(value) {
                    self.properties.content_log.level = level;
                } else {
                    // R13: the value is printed with its leading spaces.
                    self.refuse(
                        DiagnosticCode::from(codes::BAD_LOG_LEVEL.with([value])),
                        key,
                    );
                    self.hash(key, value, |head| {
                        ContentLogLevel::from_name(head).is_some()
                    });
                }
            }
            "emit-server-telemetry" => {
                self.properties.emit_server_telemetry = self.boolean(key, value);
            }
            "enable-profiler" => self.properties.enable_profiler = Some(self.boolean(key, value)),
            "enable-editor-network-metrics" => {
                self.properties.enable_editor_network_metrics = Some(self.boolean(key, value));
            }
            "convert-world-to-editor-project" => {
                self.properties.convert_world_to_editor_project = self.boolean(key, value);
            }
            _ => {}
        }
    }

    /// R14: an auto attach mode whose requirements the file does not meet. The shipped
    /// file's comments name them; whether the server enforces them is not known.
    fn auto_attach_needs(&mut self) {
        let debugger = &self.properties.debugger;
        let unmet = match debugger.auto_attach {
            AutoAttach::Disabled => false,
            AutoAttach::Connect => !debugger.allow_outbound,
            AutoAttach::Listen => !debugger.allow_inbound || debugger.force_inbound_port.is_none(),
        };
        if unmet {
            self.span = None;
            self.push(
                DiagnosticCode::from(&core_codes::NO_EFFECT),
                Severity::Info,
                "script-debugger-auto-attach",
            );
        }
    }
}
