//! `server.properties` (fixtures sp01–sp16; parser rules P1–P12; validation R1–R14), and the
//! generated round trip of its line format.

use std::collections::BTreeMap;

use bedrock_addon::diagnostic::{DiagnosticCode, Diagnostics, Severity, Span};
use bedrock_addon::families::script::config::{
    AutoAttach, ContentLogLevel, PROPERTIES_LINE_MAX, PropertyLine, ScriptProperties,
    ServerProperties,
};
use bedrock_addon::json::EncodeError;

use crate::common::{check_properties, seen};

fn parse(text: &str) -> ServerProperties {
    ServerProperties::parse(text.as_bytes())
}

/// (key, value) of each entry.
fn entries(properties: &ServerProperties) -> Vec<(&str, &str)> {
    properties
        .entries
        .iter()
        .map(|entry| (entry.key.as_str(), entry.value.as_str()))
        .collect()
}

/// The script keys of `text` and what reading them said, as (code, severity, key).
fn script(text: &str) -> (ScriptProperties, Vec<(String, String, String)>) {
    let properties = parse(text);
    let mut findings = Diagnostics::default();
    let script = properties.script(&mut findings);
    let findings = seen(&findings)
        .into_iter()
        .map(|f| (f.code, f.severity, f.path))
        .collect();
    (script, findings)
}

/// A finding of ours: the server has no content-log line for reading `server.properties`
/// (BDS 1.26.52.3), so every note of the reader is an information.
fn info(code: &str, key: &str) -> (String, String, String) {
    (code.to_owned(), "info".to_owned(), key.to_owned())
}

fn error(code: &str, key: &str) -> (String, String, String) {
    (code.to_owned(), "error".to_owned(), key.to_owned())
}

// ---- the line format ----

#[test]
fn sp01_key_trailing_space_is_trimmed_and_the_value_keeps_its_lead() {
    let (properties, script) = check_properties("sp01_key_trailing_space");
    assert_eq!(entries(&properties), [("content-log-level", " bogus1")]);
    assert_eq!(properties.entries[0].line, 1);
    assert_eq!(script.content_log.level, ContentLogLevel::Info);
}

#[test]
fn sp02_value_trailing_space_is_trimmed() {
    let (properties, script) = check_properties("sp02_value_trailing_space");
    assert_eq!(
        entries(&properties),
        [("script-debugger-auto-attach", "bogus2")]
    );
    assert_eq!(script.debugger.auto_attach, AutoAttach::Disabled);
}

#[test]
fn sp03_key_case_matches_nothing() {
    let (properties, script) = check_properties("sp03_key_case");
    assert_eq!(entries(&properties), [("CHAT-RESTRICTION", "bogus3")]);
    assert_eq!(script, ScriptProperties::default());
}

#[test]
fn r4_an_unknown_key_is_an_info() {
    let properties = parse("my-own-key=1\nserver-name=x\n");
    let found = seen(properties.diagnostics());
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].code, "UnrecognizedField");
    assert_eq!(found[0].severity, "info");
    assert_eq!(
        properties
            .diagnostics()
            .iter()
            .next()
            .unwrap()
            .location
            .span,
        Some(Span { offset: 0, len: 12 })
    );
    assert_eq!(properties.entries.len(), 2);
}

#[test]
fn sp04_leading_space_skips_the_line() {
    let (properties, _) = check_properties("sp04_leading_space");
    assert!(properties.entries.is_empty());
    // A commented or free-text line led by a space is nothing to warn about.
    for quiet in [" # transport=raknet\n", "  \n", " just words\n"] {
        let properties = parse(quiet);
        assert!(properties.entries.is_empty());
        assert!(properties.diagnostics().is_empty(), "{quiet:?}");
    }
}

#[test]
fn sp05_inline_hash_is_value_text() {
    let (properties, script) = check_properties("sp05_inline_hash");
    assert_eq!(
        entries(&properties),
        [
            ("chat-restriction", "bogusA # c"),
            ("script-debugger-auto-attach", "bogusB#c"),
            ("content-log-level", "info#c")
        ]
    );
    assert_eq!(script.content_log.level, ContentLogLevel::Info);
    assert_eq!(script.debugger.auto_attach, AutoAttach::Disabled);
}

#[test]
fn p2_a_hash_is_a_comment_only_as_the_first_byte() {
    let properties = parse("#content-log-level=error\n# words\n#\n");
    assert!(properties.entries.is_empty());
    assert!(properties.diagnostics().is_empty());
}

#[test]
fn sp06_second_equals_ends_the_value() {
    let (properties, _) = check_properties("sp06_second_equals");
    assert_eq!(entries(&properties), [("content-log-level", "bog")]);
}

#[test]
fn sp07_crlf_is_trimmed() {
    let (properties, _) = check_properties("sp07_crlf");
    assert_eq!(entries(&properties), [("chat-restriction", "bogus7")]);
}

#[test]
fn sp08_tab_led_key_matches_nothing() {
    let (properties, script) = check_properties("sp08_tab");
    assert_eq!(
        entries(&properties),
        [("\tscript-debugger-auto-attach", "bogus8")]
    );
    assert_eq!(script.debugger.auto_attach, AutoAttach::Disabled);
}

#[test]
fn sp09_empty_values_leave_the_keys_absent() {
    let (properties, script) = check_properties("sp09_empty_values");
    assert!(properties.entries.is_empty());
    assert_eq!(script, ScriptProperties::default());
    assert_eq!(properties.get("content-log-level"), None);
}

#[test]
fn p2_short_lines_and_p12_lines_without_an_equals_sign() {
    // Shorter than two bytes: skipped. Longer without `=`: nothing is applied, with an
    // information.
    let properties = parse("\nx\n\nwords\n");
    assert!(properties.entries.is_empty());
    let found = seen(properties.diagnostics());
    assert_eq!(found.len(), 1);
    assert_eq!(
        (found[0].code.as_str(), found[0].severity.as_str()),
        ("NoEffect", "info")
    );
    assert_eq!(
        properties
            .diagnostics()
            .iter()
            .next()
            .unwrap()
            .location
            .span,
        Some(Span { offset: 4, len: 5 })
    );
}

#[test]
fn p12_bytes_that_are_not_utf8_are_replaced_with_an_information() {
    let properties = ServerProperties::parse(b"server-name=caf\xe9\n");
    assert_eq!(entries(&properties), [("server-name", "caf\u{fffd}")]);
    let finding = properties.diagnostics().iter().next().unwrap();
    assert!(matches!(
        finding.code,
        DiagnosticCode::MalformedInput { .. }
    ));
    assert_eq!(finding.severity, Severity::Info);
}

#[test]
fn the_last_line_needs_no_newline_and_lines_are_numbered_from_one() {
    let properties = parse("# head\n\nserver-name=a\ngamemode=creative");
    assert_eq!(
        entries(&properties),
        [("server-name", "a"), ("gamemode", "creative")]
    );
    assert_eq!(
        properties
            .entries
            .iter()
            .map(|e| e.line)
            .collect::<Vec<_>>(),
        [3, 4]
    );
    assert!(ServerProperties::parse(b"").entries.is_empty());
    assert!(ServerProperties::parse(b"\n").entries.is_empty());
}

#[test]
fn sp13_long_line_is_an_error_and_the_rest_is_read_deviation() {
    // Vanilla stops reading at such a line (inferred).
    let (properties, _) = check_properties("sp13_long_line");
    assert_eq!(entries(&properties), [("content-log-level", "bogus")]);
    assert_eq!(properties.entries[0].line, 2);
    let finding = properties.diagnostics().first_error().unwrap();
    assert_eq!(
        finding.location.span,
        Some(Span {
            offset: 0,
            len: 5000
        })
    );
}

#[test]
fn r1_the_longest_line_is_4095_bytes() {
    assert_eq!(PROPERTIES_LINE_MAX, 4095);
    let line = |len: usize| format!("server-name={}\n", "x".repeat(len - "server-name=".len()));
    let fits = parse(&line(4095));
    assert!(fits.diagnostics().is_empty());
    assert_eq!(fits.entries[0].value.len(), 4095 - 12);
    let long = parse(&line(4096));
    assert!(long.entries.is_empty());
    assert!(long.diagnostics().has_errors());
}

#[test]
fn sp15_duplicate_key_checks_each_line_and_keeps_the_last() {
    let (properties, script) = check_properties("sp15_duplicate_key");
    assert_eq!(properties.entries.len(), 3);
    assert_eq!(script.debugger.auto_attach, AutoAttach::Connect);
    assert_eq!(
        properties.get("script-debugger-auto-attach"),
        Some("connect")
    );
}

#[test]
fn p11_an_empty_or_refused_later_line_leaves_the_earlier_value() {
    let (script, findings) =
        script("content-log-level=error\ncontent-log-level=\ncontent-log-level=loud\n");
    assert_eq!(script.content_log.level, ContentLogLevel::Error);
    assert_eq!(findings, [error("InvalidEnumValue", "content-log-level")]);
}

// ---- the script keys ----

#[test]
fn a_file_without_script_keys_has_the_defaults_of_the_schema() {
    let (script, findings) = script("server-name=Dedicated Server\n");
    assert!(findings.is_empty());
    assert!(script.enable_script);
    assert!(!script.debugger.allow_outbound);
    assert!(!script.debugger.allow_inbound);
    assert_eq!(script.debugger.force_inbound_port, None);
    assert_eq!(script.debugger.auto_attach, AutoAttach::Disabled);
    assert_eq!(script.debugger.connect_address, None);
    assert_eq!(script.debugger.timeout_sec, 0);
    assert_eq!(script.debugger.passcode, None);
    assert!(script.watchdog.enable);
    assert!(script.watchdog.exception_handling);
    assert!(script.watchdog.shutdown);
    assert!(script.watchdog.hang_exception);
    assert_eq!(script.watchdog.hang_ms, 10_000);
    assert_eq!(script.watchdog.spike_ms, None);
    assert_eq!(script.watchdog.slow_ms, None);
    assert_eq!(script.watchdog.memory_warning_mb, 100);
    assert_eq!(script.watchdog.memory_limit_mb, 250);
    assert!(!script.diagnostics.auto_start);
    assert_eq!(script.diagnostics.max_files, 5);
    assert_eq!(script.diagnostics.max_file_size, 2_097_152);
    assert_eq!(script.sentry.window_s, None);
    assert_eq!(script.sentry.max_events, None);
    assert!(!script.content_log.file);
    assert!(!script.content_log.console);
    assert_eq!(script.content_log.level, ContentLogLevel::Info);
    assert!(!script.emit_server_telemetry);
    assert_eq!(script.enable_profiler, None);
    assert_eq!(script.enable_editor_network_metrics, None);
    assert!(!script.convert_world_to_editor_project);
}

#[test]
fn every_script_key_sets_its_member() {
    let text = "\
enable-script=false
allow-outbound-script-debugging=true
allow-inbound-script-debugging=true
force-inbound-debug-port=19150
script-debugger-auto-attach=listen
script-debugger-auto-attach-connect-address=localhost:19144
script-debugger-auto-attach-timeout=7
script-debugger-passcode=open sesame
script-watchdog-enable=false
script-watchdog-enable-exception-handling=false
script-watchdog-enable-shutdown=false
script-watchdog-hang-exception=false
script-watchdog-hang-threshold=30
script-watchdog-spike-threshold=100
script-watchdog-slow-threshold=10
script-watchdog-memory-warning=1
script-watchdog-memory-limit=2000
diagnostics-capture-auto-start=true
diagnostics-capture-max-files=9
diagnostics-capture-max-file-size=1024
sentry-rate-limit-window=0
sentry-max-events-per-window=3
content-log-file-enabled=true
content-log-console-output-enabled=true
content-log-level=verbose
emit-server-telemetry=true
enable-profiler=true
enable-editor-network-metrics=false
convert-world-to-editor-project=true
";
    let (script, findings) = script(text);
    assert!(findings.is_empty(), "{findings:?}");
    assert!(!script.enable_script);
    assert!(script.debugger.allow_outbound);
    assert!(script.debugger.allow_inbound);
    assert_eq!(script.debugger.force_inbound_port, Some(19150));
    assert_eq!(script.debugger.auto_attach, AutoAttach::Listen);
    assert_eq!(
        script.debugger.connect_address,
        Some(("localhost".to_owned(), 19144))
    );
    assert_eq!(script.debugger.timeout_sec, 7);
    assert_eq!(script.debugger.passcode.as_deref(), Some("open sesame"));
    assert!(!script.watchdog.enable);
    assert!(!script.watchdog.exception_handling);
    assert!(!script.watchdog.shutdown);
    assert!(!script.watchdog.hang_exception);
    assert_eq!(script.watchdog.hang_ms, 30);
    assert_eq!(script.watchdog.spike_ms, Some(100));
    assert_eq!(script.watchdog.slow_ms, Some(10));
    assert_eq!(script.watchdog.memory_warning_mb, 1);
    assert_eq!(script.watchdog.memory_limit_mb, 2000);
    assert!(script.diagnostics.auto_start);
    assert_eq!(script.diagnostics.max_files, 9);
    assert_eq!(script.diagnostics.max_file_size, 1024);
    assert_eq!(script.sentry.window_s, Some(0));
    assert_eq!(script.sentry.max_events, Some(3));
    assert!(script.content_log.file);
    assert!(script.content_log.console);
    assert_eq!(script.content_log.level, ContentLogLevel::Verbose);
    assert!(script.emit_server_telemetry);
    assert_eq!(script.enable_profiler, Some(true));
    assert_eq!(script.enable_editor_network_metrics, Some(false));
    assert!(script.convert_world_to_editor_project);
}

#[test]
fn sp10_bools_are_true_only_for_true() {
    let (_, script) = check_properties("sp10_bools");
    assert!(!script.content_log.console);
    let key = "content-log-console-output-enabled";
    for (value, expected, warned) in [
        ("true", true, false),
        ("false", false, false),
        ("TRUE", false, true),
        ("FALSE", false, true),
        ("1", false, true),
        ("yes", false, true),
        ("true # on", false, true),
        (" true", false, true),
    ] {
        let (script, findings) = self::script(&format!("{key}={value}\n"));
        assert_eq!(script.content_log.console, expected, "{value}");
        let expected = if warned {
            vec![info("InvalidValue", key)]
        } else {
            vec![]
        };
        assert_eq!(findings, expected, "{value}");
    }
}

#[test]
fn sp11_int_forms_follow_atoi() {
    let (properties, script) = check_properties("sp11_int_forms");
    assert_eq!(properties.entries.len(), 5);
    assert_eq!(script.watchdog.spike_ms, Some(-5));
    let key = "script-watchdog-spike-threshold";
    for (value, expected, warnings) in [
        ("20", 20, 0),
        (" 20", 20, 0),
        ("+20", 20, 0),
        ("20ms", 20, 1),
        ("20 # ms", 20, 1),
        ("abc", 0, 1),
        ("-", 0, 1),
        ("0", 0, 0),
        ("-5", -5, 1),
        ("-5x", -5, 2),
        ("2147483647", i32::MAX, 0),
        ("2147483648", i32::MAX, 1),
        ("-2147483648", i32::MIN, 1),
        ("-2147483649", i32::MIN, 2),
        ("99999999999999999999999999", i32::MAX, 1),
    ] {
        let (script, findings) = self::script(&format!("{key}={value}\n"));
        assert_eq!(script.watchdog.spike_ms, Some(expected), "{value}");
        assert_eq!(findings.len(), warnings, "{value}: {findings:?}");
        assert!(findings.iter().all(|f| f.1 == "info" && f.2 == key));
    }
    // Beyond `i32` the value is held to it: `ClampedValue`.
    let (_, findings) = self::script(&format!("{key}=2147483648\n"));
    assert_eq!(findings, [info("ClampedValue", key)]);
}

#[test]
fn r8_a_negative_threshold_is_kept_with_an_information() {
    for (key, read) in [
        (
            "script-watchdog-hang-threshold",
            (|s| s.watchdog.hang_ms) as fn(&ScriptProperties) -> i32,
        ),
        ("script-watchdog-spike-threshold", |s| {
            s.watchdog.spike_ms.unwrap()
        }),
        ("script-watchdog-slow-threshold", |s| {
            s.watchdog.slow_ms.unwrap()
        }),
        ("script-watchdog-memory-warning", |s| {
            s.watchdog.memory_warning_mb
        }),
        ("script-watchdog-memory-limit", |s| {
            s.watchdog.memory_limit_mb
        }),
    ] {
        let (script, findings) = self::script(&format!("{key}=-1\n"));
        assert_eq!(read(&script), -1, "{key}");
        assert_eq!(findings, [info("InvalidValue", key)], "{key}");
        let (script, findings) = self::script(&format!("{key}=0\n"));
        assert_eq!(read(&script), 0, "{key}");
        assert!(findings.is_empty(), "{key}");
    }
}

#[test]
fn r9_the_memory_keys_are_held_to_2000() {
    for (key, read) in [
        (
            "script-watchdog-memory-warning",
            (|s| s.watchdog.memory_warning_mb) as fn(&ScriptProperties) -> i32,
        ),
        ("script-watchdog-memory-limit", |s| {
            s.watchdog.memory_limit_mb
        }),
    ] {
        let (script, findings) = self::script(&format!("{key}=2000\n"));
        assert_eq!(read(&script), 2000);
        assert!(findings.is_empty());
        let (script, findings) = self::script(&format!("{key}=2001\n"));
        assert_eq!(read(&script), 2000);
        assert_eq!(findings, [info("ClampedValue", key)]);
    }
    // The thresholds have no such limit.
    let (script, findings) = self::script("script-watchdog-hang-threshold=600000\n");
    assert_eq!(script.watchdog.hang_ms, 600_000);
    assert!(findings.is_empty());
}

#[test]
fn a_negative_count_leaves_the_key_as_it_was() {
    for key in [
        "script-debugger-auto-attach-timeout",
        "diagnostics-capture-max-files",
        "diagnostics-capture-max-file-size",
        "sentry-rate-limit-window",
        "sentry-max-events-per-window",
    ] {
        let (script, findings) = self::script(&format!("{key}=-1\n"));
        assert_eq!(script, ScriptProperties::default(), "{key}");
        assert_eq!(findings, [info("InvalidValue", key)], "{key}");
    }
}

#[test]
fn sp12_enable_script_false_disables_scripting() {
    let (_, script) = check_properties("sp12_enable_script");
    assert!(!script.enable_script);
}

#[test]
fn sp16_enum_case_is_exact() {
    let (_, script) = check_properties("sp16_enum_case");
    assert_eq!(script.debugger.auto_attach, AutoAttach::Disabled);
    assert_eq!(script.content_log.level, ContentLogLevel::Info);
}

#[test]
fn the_enum_words_are_those_of_the_schema_both_ways() {
    for (name, mode) in [
        ("disabled", AutoAttach::Disabled),
        ("connect", AutoAttach::Connect),
        ("listen", AutoAttach::Listen),
    ] {
        assert_eq!(AutoAttach::from_name(name), Some(mode));
        assert_eq!(mode.name(), name);
    }
    for (name, level) in [
        ("verbose", ContentLogLevel::Verbose),
        ("info", ContentLogLevel::Info),
        ("warning", ContentLogLevel::Warning),
        ("error", ContentLogLevel::Error),
    ] {
        assert_eq!(ContentLogLevel::from_name(name), Some(level));
        assert_eq!(level.name(), name);
    }
    for other in ["Disabled", "LISTEN", "warn", "", "none"] {
        assert_eq!(AutoAttach::from_name(other), None);
        assert_eq!(ContentLogLevel::from_name(other), None);
    }
}

#[test]
fn r2_a_hash_after_a_valid_word_is_pointed_out() {
    let key = "script-debugger-auto-attach";
    let (_, findings) = script(&format!(
        "allow-outbound-script-debugging=true\n{key}=connect # dev\n"
    ));
    assert_eq!(
        findings,
        [error("InvalidEnumValue", key), info("InvalidValue", key)]
    );
    // Not when the word before it is no better.
    let (_, findings) = script(&format!("{key}=sometimes # dev\n"));
    assert_eq!(findings, [error("InvalidEnumValue", key)]);
}

#[test]
fn r11_the_debug_port_is_1_to_65535() {
    let key = "force-inbound-debug-port";
    for inside in [1, 19144, 65535] {
        let (script, findings) = script(&format!("{key}={inside}\n"));
        assert_eq!(script.debugger.force_inbound_port, Some(inside));
        assert!(findings.is_empty());
    }
    for outside in ["0", "65536", "-1", "abc"] {
        let properties = parse(&format!("{key}={outside}\n"));
        let mut findings = Diagnostics::default();
        let script = properties.script(&mut findings);
        assert_eq!(script.debugger.force_inbound_port, None, "{outside}");
        let finding = findings.first_error().unwrap();
        assert!(matches!(finding.code, DiagnosticCode::InvalidValue { .. }));
        assert_eq!(
            finding.code.to_string(),
            "Invalid script debugger port, must be in the range [ 1, 65535 ]"
        );
        assert_eq!(findings.len(), 1);
    }
}

#[test]
fn r12_the_connect_address_is_host_and_port() {
    let key = "script-debugger-auto-attach-connect-address";
    for (value, host, port) in [
        ("localhost:19144", "localhost", 19144),
        ("10.0.0.1:1", "10.0.0.1", 1),
        ("[::1]:65535", "[::1]", 65535),
    ] {
        let (script, findings) = script(&format!("{key}={value}\n"));
        assert_eq!(
            script.debugger.connect_address,
            Some((host.to_owned(), port))
        );
        assert!(findings.is_empty());
    }
    for bad in [
        "localhost",
        ":19144",
        "localhost:",
        "localhost:0",
        "localhost:65536",
        "localhost:port",
        "localhost:80x",
    ] {
        let properties = parse(&format!("{key}={bad}\n"));
        let mut findings = Diagnostics::default();
        let script = properties.script(&mut findings);
        assert_eq!(script.debugger.connect_address, None, "{bad}");
        let finding = findings.first_error().unwrap();
        assert!(matches!(finding.code, DiagnosticCode::InvalidValue { .. }));
        assert_eq!(
            finding.code.to_string(),
            "Invalid script debugger connect address, format must be host:port with port in the range [ 1, 65535 ]"
        );
    }
}

#[test]
fn r12_the_port_of_the_connect_address_is_digits_only() {
    // A sign or a space before the digits is `atoi`'s reading of a number, not a port.
    let key = "script-debugger-auto-attach-connect-address";
    for bad in [
        "localhost:+80",
        "localhost: 80",
        "localhost:-80",
        "localhost:\t80",
        "localhost:8 0",
    ] {
        let properties = parse(&format!("{key}={bad}\n"));
        let mut findings = Diagnostics::default();
        let script = properties.script(&mut findings);
        assert_eq!(script.debugger.connect_address, None, "{bad}");
        let finding = findings.first_error().unwrap_or_else(|| panic!("{bad}"));
        assert!(
            matches!(finding.code, DiagnosticCode::InvalidValue { text: Some(_) }),
            "{bad}"
        );
    }
    // Leading zeros are digits.
    let (script, findings) = script(&format!("{key}=localhost:0080\n"));
    assert_eq!(
        script.debugger.connect_address,
        Some(("localhost".to_owned(), 80))
    );
    assert!(findings.is_empty());
}

#[test]
fn r12_the_host_of_the_connect_address_is_a_name_or_a_bracketed_address() {
    // The address is split at its last `:`. A host is a name or an address: it holds no
    // white space, and a `:` of its own only inside the brackets of an IPv6 literal, or
    // nothing tells the address's last group from the port.
    let key = "script-debugger-auto-attach-connect-address";
    for bad in [
        // A bare IPv6 address is no host and port.
        "::1",
        "::80",
        "fe80::1:9229",
        "a:b:80",
        // Brackets that do not close around the whole host.
        "[::1:80",
        "::1]:80",
        "[]:80",
        // Brackets around what is no IPv6 address (K124): a name, a dotted address, a
        // group too many, a scope without a name.
        "[x]:80",
        "[localhost]:80",
        "[1.2.3.4]:80",
        "[1:2:3:4:5:6:7:8:9]:80",
        "[::g]:80",
        "[fe80::1%]:80",
        // White space in the host; the value keeps the spaces after `=`.
        "h :80",
        " h:80",
        "h\t:80",
        "my host:80",
    ] {
        let properties = parse(&format!("{key}={bad}\n"));
        let mut findings = Diagnostics::default();
        let script = properties.script(&mut findings);
        assert_eq!(script.debugger.connect_address, None, "{bad:?}");
        let finding = findings.first_error().unwrap_or_else(|| panic!("{bad:?}"));
        assert!(
            matches!(finding.code, DiagnosticCode::InvalidValue { text: Some(_) }),
            "{bad:?}"
        );
        assert_eq!(
            finding.code.to_string(),
            "Invalid script debugger connect address, format must be host:port with port in the range [ 1, 65535 ]",
            "{bad:?}"
        );
    }
    for (value, host, port) in [
        ("[fe80::1]:9229", "[fe80::1]", 9229),
        ("[::]:1", "[::]", 1),
        // The address in each form of RFC 4291, and with a scope (RFC 4007).
        ("[2001:db8:0:0:0:0:0:1]:80", "[2001:db8:0:0:0:0:0:1]", 80),
        ("[::ffff:10.0.0.1]:80", "[::ffff:10.0.0.1]", 80),
        ("[FE80::A]:80", "[FE80::A]", 80),
        ("[fe80::1%eth0]:80", "[fe80::1%eth0]", 80),
        ("debug.example.org:19144", "debug.example.org", 19144),
        ("h-1_x:80", "h-1_x", 80),
    ] {
        let (script, findings) = script(&format!("{key}={value}\n"));
        assert_eq!(
            script.debugger.connect_address,
            Some((host.to_owned(), port)),
            "{value}"
        );
        assert!(findings.is_empty(), "{value}");
    }
}

#[test]
fn r14_an_auto_attach_mode_without_what_it_needs_is_noted() {
    let key = "script-debugger-auto-attach";
    let unmet = [info("NoEffect", key)];
    let cases: [(&str, bool); 8] = [
        ("script-debugger-auto-attach=connect\n", true),
        (
            "script-debugger-auto-attach=connect\nallow-outbound-script-debugging=true\n",
            false,
        ),
        (
            "script-debugger-auto-attach=connect\nallow-inbound-script-debugging=true\n",
            true,
        ),
        ("script-debugger-auto-attach=listen\n", true),
        (
            "script-debugger-auto-attach=listen\nallow-inbound-script-debugging=true\n",
            true,
        ),
        (
            "script-debugger-auto-attach=listen\nforce-inbound-debug-port=19144\n",
            true,
        ),
        (
            "script-debugger-auto-attach=listen\nallow-inbound-script-debugging=true\nforce-inbound-debug-port=19144\n",
            false,
        ),
        ("script-debugger-auto-attach=disabled\n", false),
    ];
    for (text, warned) in cases {
        let (_, findings) = script(text);
        assert_eq!(
            findings,
            if warned { unmet.to_vec() } else { vec![] },
            "{text}"
        );
    }
}

#[test]
fn a_finding_of_a_script_key_is_on_its_line() {
    let text = "server-name=x\ncontent-log-level=loud\n";
    let properties = parse(text);
    let mut findings = Diagnostics::default();
    properties.script(&mut findings);
    let finding = findings.first_error().unwrap();
    assert_eq!(
        finding.location.span,
        Some(Span {
            offset: 14,
            len: 22
        })
    );
    assert_eq!(
        finding.location.path.as_ref().unwrap().to_string(),
        "server.properties"
    );
}

#[test]
fn sp14_shipped_file_parses_without_a_finding() {
    let Some(corpus) = std::env::var_os("BEDROCK_ADDON_CORPUS") else {
        eprintln!("skipped: BEDROCK_ADDON_CORPUS is not set");
        return;
    };
    let path = std::path::Path::new(&corpus).join("bds-1.26.36.1/server.properties");
    let properties = ServerProperties::parse(&std::fs::read(path).unwrap());
    let mut findings = properties.diagnostics().clone();
    let script = properties.script(&mut findings);
    assert!(findings.is_empty(), "{findings:?}");
    // The script keys the file sets are set to their defaults; the rest are commented out.
    assert_eq!(script, ScriptProperties::default());
    // 41 lines set a key; `level-seed=` has no value and is not applied.
    assert_eq!(properties.entries.len(), 40);
    assert_eq!(properties.get("level-seed"), None);
    assert_eq!(properties.get("server-name"), Some("Dedicated Server"));
}

#[test]
fn p2_a_line_of_white_space_only_is_a_blank_line() {
    // An indented blank line, also one of a CRLF file, says nothing: no entry, no finding.
    for line in ["\t\t", "\t\r", "\x0b\x0c", "\t \t", "\r\r", "\t\t\t\r"] {
        let properties = parse(&format!("server-name=a\n{line}\ngamemode=b\n"));
        assert!(properties.diagnostics().is_empty(), "{line:?}");
        assert_eq!(
            entries(&properties),
            [("server-name", "a"), ("gamemode", "b")],
            "{line:?}"
        );
    }
    // Anything else on a line without `=` is still pointed out.
    let properties = parse("\tx\t\n");
    assert_eq!(properties.diagnostics().len(), 1);
}

#[test]
fn p7_a_space_led_line_with_an_empty_value_is_silent_like_a_tab_led_one() {
    // P2 and P5 skip a space-led and a tab-led line alike; P7 makes an empty value nothing.
    for line in [
        "  = ",
        "  =\n",
        " =",
        " level-seed=",
        "  level-seed= \r",
        "\t\t=",
        "\tlevel-seed=",
    ] {
        let properties = parse(&format!("server-name=a\n{line}\n"));
        assert!(properties.diagnostics().is_empty(), "{line:?}");
        assert_eq!(entries(&properties), [("server-name", "a")], "{line:?}");
    }
    // With a value, both are the same R3 information.
    for line in [" level-seed=5", "\tlevel-seed=5", "  =5", "\t=5"] {
        let properties = parse(&format!("{line}\n"));
        assert_eq!(properties.diagnostics().len(), 1, "{line:?}");
    }
}

#[test]
fn r4_the_keys_the_shipped_file_only_names_in_comments_are_known() {
    for key in ["allow-player-joining", "server-ip", "server-udp-ports"] {
        let properties = parse(&format!("{key}=x\n"));
        assert!(properties.diagnostics().is_empty(), "{key}");
        // And in another case each is the near miss of a known key.
        let upper = parse(&format!("{}=x\n", key.to_ascii_uppercase()));
        let finding = upper.diagnostics().iter().next().unwrap();
        assert!(matches!(
            finding.code,
            DiagnosticCode::UnrecognizedField { .. }
        ));
        assert_eq!(finding.severity, Severity::Info, "{key}");
    }
}

#[test]
fn sp14_every_key_of_the_shipped_file_is_known() {
    let Some(corpus) = std::env::var_os("BEDROCK_ADDON_CORPUS") else {
        eprintln!("skipped: BEDROCK_ADDON_CORPUS is not set");
        return;
    };
    let path = std::path::Path::new(&corpus).join("bds-1.26.36.1/server.properties");
    let text = std::fs::read_to_string(path).unwrap();
    // A key is set (`key=value`) or commented out (`#key=value`, `# key=value`); prose
    // never has a `=` right after its first word.
    let mut keys = std::collections::BTreeSet::new();
    for line in text.lines() {
        let line = line
            .strip_prefix('#')
            .map_or(line, |rest| rest.strip_prefix(' ').unwrap_or(rest));
        if let Some((key, _)) = line.split_once('=')
            && !key.is_empty()
            && key
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            keys.insert(key);
        }
    }
    assert_eq!(keys.len(), 66);
    let unknown: Vec<&str> = keys
        .into_iter()
        .filter(|key| !parse(&format!("{key}=x\n")).diagnostics().is_empty())
        .collect();
    assert_eq!(unknown, <[&str; 0]>::default());
}

// ---- writing ----

#[test]
fn a_file_without_a_final_line_break_is_written_with_one() {
    for (input, written) in [
        ("server-name=b", "server-name=b\n"),
        ("server-name=b\ngamemode=d", "server-name=b\ngamemode=d\n"),
        ("server-name=b\r", "server-name=b\n"),
        ("server-name=b\n# c", "server-name=b\n"),
        ("# c", ""),
        ("", ""),
    ] {
        let properties = parse(input);
        assert!(properties.diagnostics().is_empty(), "{input:?}");
        assert_eq!(properties.to_text().unwrap(), written, "{input:?}");
        // With the line break the file reads the same.
        assert_eq!(entries(&parse(written)), entries(&properties), "{input:?}");
    }
}

#[test]
fn to_text_writes_one_line_per_entry() {
    let mut properties = ServerProperties::default();
    for (key, value) in [("server-name", "My Server"), ("enable-script", "false")] {
        properties.entries.push(PropertyLine {
            key: key.to_owned(),
            value: value.to_owned(),
            line: 0,
        });
    }
    let text = properties.to_text().unwrap();
    assert_eq!(text, "server-name=My Server\nenable-script=false\n");
    assert_eq!(entries(&parse(&text)), entries(&properties));
}

#[test]
fn a_property_line_built_with_new_is_written_and_read_back() {
    let line = PropertyLine::new("enable-script", "false", 1);
    assert_eq!(
        line,
        PropertyLine {
            key: "enable-script".to_owned(),
            value: "false".to_owned(),
            line: 1,
        }
    );
    let mut properties = ServerProperties::default();
    properties.entries.push(line.clone());
    let text = properties.to_text().unwrap();
    assert_eq!(text, "enable-script=false\n");
    assert_eq!(parse(&text).entries, [line]);
}

#[test]
fn to_text_refuses_what_the_format_cannot_hold() {
    let refused = |key: &str, value: &str| {
        let mut properties = parse("server-name=x\n");
        properties.entries.push(PropertyLine {
            key: key.to_owned(),
            value: value.to_owned(),
            line: 2,
        });
        match properties.to_text() {
            Err(EncodeError::NotRepresentable { at, .. }) => Some(at.to_string()),
            Ok(_) => None,
            Err(other) => panic!("{other:?}"),
        }
    };
    for (key, value, at) in [
        ("a=b", "v", "1/key"),
        ("a\nb", "v", "1/key"),
        ("key ", "v", "1/key"),
        (" key", "v", "1/key"),
        ("#key", "v", "1/key"),
        ("key", "", "1/value"),
        ("key", "a=b", "1/value"),
        ("key", "a\nb", "1/value"),
        ("key", "v ", "1/value"),
        ("key", "v\r", "1/value"),
    ] {
        assert_eq!(
            refused(key, value).as_deref(),
            Some(at),
            "{key:?}={value:?}"
        );
    }
    // What reads back as written is written: a tab-led key, an empty key, a led value.
    for (key, value) in [("\tkey", "v"), ("", "v"), ("key", " v"), ("key", "a # b")] {
        assert_eq!(refused(key, value), None, "{key:?}={value:?}");
    }
    // Both sides of the line limit.
    let fits = "x".repeat(PROPERTIES_LINE_MAX - 4);
    assert_eq!(refused("key", &fits), None);
    assert_eq!(
        refused("key", &format!("{fits}x")).as_deref(),
        Some("1/value")
    );
}

/// xorshift64*: a fixed sequence per seed.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % n as u64).unwrap()
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len())]
    }
}

/// One generated line: the bytes written to the input, the class of the form, and the line
/// the writer gives for it (`None` when the server applies nothing).
fn line(rng: &mut Rng) -> (String, &'static str, Option<String>) {
    let key = rng.pick(&[
        "server-name",
        "content-log-level",
        "script-watchdog-spike-threshold",
        "my-key",
        "UPPER",
        "k",
        "",
    ]);
    let value = rng.pick(&[
        "x",
        "Dedicated Server",
        " led",
        "20ms",
        "a # b",
        "true",
        "é",
    ]);
    let plain = format!("{key}={value}");
    match rng.below(12) {
        0 => ("# a comment=here".to_owned(), "comment", None),
        1 => (String::new(), "blank line", None),
        2 => (format!("{plain}\r"), "CRLF", Some(plain)),
        3 => (
            format!("{plain} \t"),
            "trailing white space of the value",
            Some(plain),
        ),
        // Without a key the line starts with the spaces, so the server skips it.
        4 if key.is_empty() => (format!("  ={value}"), "line led by a space", None),
        4 => (
            format!("{key}  ={value}"),
            "trailing white space of the key",
            Some(plain),
        ),
        5 => (
            format!("{plain}=tail=more"),
            "text after a second =",
            Some(plain),
        ),
        6 => (format!(" {plain}"), "line led by a space", None),
        7 => (format!("{key}=  "), "empty value", None),
        8 => ("just words".to_owned(), "line without =", None),
        // A tab alone before the `=` is the key's trailing white space.
        9 if key.is_empty() => (
            format!("\t{plain}"),
            "trailing white space of the key",
            Some(plain),
        ),
        9 => (
            format!("\t{plain}"),
            "identical",
            Some(format!("\t{plain}")),
        ),
        _ => (plain.clone(), "identical", Some(plain)),
    }
}

#[test]
fn generated_files_round_trip_and_every_difference_from_the_input_is_of_a_known_class() {
    let mut rng = Rng(0x5eed_c0f1_6000_0001);
    let mut classes: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut identical_files = 0;
    for _ in 0..3000 {
        let mut input = String::new();
        let mut expected = String::new();
        let mut plain = true;
        for _ in 0..rng.below(12) {
            let (bytes, class, written) = line(&mut rng);
            *classes.entry(class).or_default() += 1;
            plain &= class == "identical";
            input.push_str(&bytes);
            input.push('\n');
            if let Some(written) = written {
                expected.push_str(&written);
                expected.push('\n');
            }
        }
        // One file in four ends without a line break: the server reads its last line all the
        // same, and the writer ends every line it writes.
        if rng.below(4) == 0 && input.ends_with('\n') {
            input.pop();
            if !input.is_empty() && !input.ends_with('\n') {
                *classes.entry("no final line break").or_default() += 1;
                plain = false;
            }
        }
        let properties = parse(&input);
        let written = properties.to_text().unwrap();
        // What is written is the input but for the differences of each line's class.
        assert_eq!(written, expected, "input: {input:?}");
        if plain {
            assert_eq!(written, input);
            identical_files += 1;
        }
        // And it reads back to the same entries, and writes the same text again.
        let again = parse(&written);
        assert_eq!(entries(&again), entries(&properties), "input: {input:?}");
        assert_eq!(again.to_text().unwrap(), written);
    }
    let names: Vec<&str> = classes.keys().copied().collect();
    assert_eq!(
        names,
        [
            "CRLF",
            "blank line",
            "comment",
            "empty value",
            "identical",
            "line led by a space",
            "line without =",
            "no final line break",
            "text after a second =",
            "trailing white space of the key",
            "trailing white space of the value",
        ]
    );
    assert!(classes.values().all(|count| *count > 500), "{classes:?}");
    assert!(identical_files > 100, "{identical_files}");
}
