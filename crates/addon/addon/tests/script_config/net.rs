//! The `@minecraft/server-net` entry of `module_permissions` (fixtures sc37–sc45, sc47;
//! validation N1–N7, N9).

use bedrock_addon::diagnostic::{DiagnosticCode, Severity, Span};
use bedrock_addon::families::server_config::{
    ConfigFolder, NetModuleConfig, ScriptConfig, ScriptConfigFiles, ServerNet,
};
use bytes::Bytes;

use crate::common::{check_config, seen};

/// The configuration of a `default` folder whose server-net entry is `entry`.
fn net(entry: &str) -> ScriptConfig {
    let text = format!(r#"{{"module_permissions": {{"@minecraft/server-net": {entry}}}}}"#);
    let mut folder = ConfigFolder::new("default");
    folder.permissions = Some(Bytes::from(text));
    ScriptConfig::parse(&ScriptConfigFiles {
        folders: vec![folder],
        ..ScriptConfigFiles::default()
    })
}

fn configured(config: &ScriptConfig) -> &NetModuleConfig {
    match &config.default.permissions.server_net {
        ServerNet::Configured(net) => net,
        other => panic!("{other:?}"),
    }
}

/// (code, severity, path below the entry) of each finding.
fn findings(config: &ScriptConfig) -> Vec<(String, String, String)> {
    seen(config.diagnostics())
        .into_iter()
        .map(|f| {
            let path = f
                .path
                .strip_prefix("module_permissions/@minecraft~1server-net")
                .unwrap()
                .trim_start_matches('/')
                .to_owned();
            (f.code, f.severity, path)
        })
        .collect()
}

fn one(code: &str, severity: &str, path: &str) -> Vec<(String, String, String)> {
    vec![(code.to_owned(), severity.to_owned(), path.to_owned())]
}

#[test]
fn no_entry_leaves_requests_unrestricted() {
    let config = check_config("sc11_wildcard");
    let net = &config.default.permissions.server_net;
    assert_eq!(*net, ServerNet::Unrestricted);
    assert!(net.allows_uri("http://anything.example/"));
    assert!(!net.requires_tls());
}

#[test]
fn sc37_net_valid_loads_every_member() {
    let config = check_config("sc37_net_valid");
    let net = configured(&config);
    assert_eq!(
        net.allowed_uris.as_deref(),
        Some(&["https://example.com/api".to_owned()][..])
    );
    assert_eq!(net.force_tls, Some(true));
    assert_eq!(net.max_body_bytes, Some(1000));
    assert_eq!(net.max_concurrent_requests, Some(4));
    assert_eq!(net.session_headers.as_ref().unwrap()["x-a"], "b");
    assert_eq!(net.max_websocket_connections, Some(2));
    assert_eq!(net.max_websocket_payload_size, None);
    // The entry is kept as written beside its meaning.
    let raw = &config.default.permissions.module_permissions["@minecraft/server-net"];
    assert_eq!(raw.len(), 6);
}

#[test]
fn sc38_net_unknown_member_denies_egress_deviation() {
    // Vanilla ignores the entry and leaves requests unrestricted.
    let config = check_config("sc38_net_unknown_member");
    let net = &config.default.permissions.server_net;
    assert_eq!(*net, ServerNet::Denied);
    assert!(!net.allows_uri("https://127.0.0.1:9/allowed"));
    let finding = config.diagnostics().first_error().unwrap();
    assert!(matches!(
        &finding.code,
        DiagnosticCode::NetConfigRejected { module, errors }
            if module == "@minecraft/server-net" && errors.starts_with(" -> bogus_key:")
    ));
}

#[test]
fn sc39_net_bad_type_denies_egress_deviation() {
    let config = check_config("sc39_net_bad_type");
    assert_eq!(config.default.permissions.server_net, ServerNet::Denied);
}

#[test]
fn sc40_net_range_denies_egress_deviation() {
    let config = check_config("sc40_net_range");
    assert_eq!(
        config.packs["neg"].permissions.server_net,
        ServerNet::Denied
    );
    assert_eq!(
        config.packs["wsbig"].permissions.server_net,
        ServerNet::Denied
    );
}

#[test]
fn n2_each_integer_member_has_its_range() {
    // Both sides of each bound: the largest value loads, the next is refused.
    for (member, max) in [
        ("max_body_bytes", u64::from(u32::MAX)),
        ("max_concurrent_requests", u64::from(u32::MAX)),
        ("max_websocket_connections", u64::from(u16::MAX)),
        ("max_websocket_payload_size", u64::from(u32::MAX)),
    ] {
        for inside in [0, max] {
            let config = net(&format!("{{\"{member}\": {inside}}}"));
            assert!(config.diagnostics().is_empty(), "{member} {inside}");
            assert!(matches!(
                config.default.permissions.server_net,
                ServerNet::Configured(_)
            ));
        }
        for outside in [format!("{}", max + 1), "-1".to_owned()] {
            let config = net(&format!("{{\"{member}\": {outside}}}"));
            assert_eq!(
                findings(&config),
                [
                    (
                        "InvalidValue".to_owned(),
                        "error".to_owned(),
                        member.to_owned()
                    ),
                    (
                        "NetConfigRejected".to_owned(),
                        "error".to_owned(),
                        String::new()
                    )
                ],
                "{member} {outside}"
            );
            assert_eq!(config.default.permissions.server_net, ServerNet::Denied);
        }
    }
    let config = net(r#"{"max_body_bytes": 4294967295, "max_websocket_connections": 65535}"#);
    let net = configured(&config);
    assert_eq!(net.max_body_bytes, Some(u32::MAX));
    assert_eq!(net.max_websocket_connections, Some(u16::MAX));
}

#[test]
fn n2_an_integer_wider_than_64_bits_is_out_of_range_not_a_wrong_type() {
    let config = net(r#"{"max_body_bytes": 123456789012345678901234567890}"#);
    assert_eq!(
        findings(&config)[0],
        one("InvalidValue", "error", "max_body_bytes")[0]
    );
    // So is a number beyond every double: one finding, the range.
    let config = net(r#"{"max_body_bytes": 1e999}"#);
    assert_eq!(
        findings(&config),
        [
            (
                "InvalidValue".to_owned(),
                "error".to_owned(),
                "max_body_bytes".to_owned()
            ),
            (
                "NetConfigRejected".to_owned(),
                "error".to_owned(),
                String::new()
            )
        ]
    );
    // A fraction, a string and a boolean are the wrong type.
    for wrong in ["1.5", "5.0", "\"5\"", "true", "null", "[5]"] {
        let config = net(&format!("{{\"max_body_bytes\": {wrong}}}"));
        assert_eq!(
            findings(&config)[0],
            one("InvalidType", "error", "max_body_bytes")[0],
            "{wrong}"
        );
        assert_eq!(config.default.permissions.server_net, ServerNet::Denied);
    }
}

#[test]
fn n2_wrong_types_of_the_other_members_refuse_the_entry() {
    for (entry, path) in [
        (r#"{"allowed_uris": "https://a.example/"}"#, "allowed_uris"),
        (
            r#"{"allowed_uris": ["https://a.example/", 5]}"#,
            "allowed_uris/1",
        ),
        (r#"{"force_tls": 1}"#, "force_tls"),
        (r#"{"session_headers": ["a"]}"#, "session_headers"),
        (r#"{"session_headers": []}"#, "session_headers"),
        (
            r#"{"session_headers": {"a": "b", "c": 5}}"#,
            "session_headers/c",
        ),
    ] {
        let config = net(entry);
        assert_eq!(
            findings(&config),
            [
                (
                    "InvalidType".to_owned(),
                    "error".to_owned(),
                    path.to_owned()
                ),
                (
                    "NetConfigRejected".to_owned(),
                    "error".to_owned(),
                    String::new()
                )
            ],
            "{entry}"
        );
        assert_eq!(config.default.permissions.server_net, ServerNet::Denied);
    }
}

#[test]
fn n1_every_failing_member_is_a_line_of_the_one_error() {
    let config = net(r#"{"bogus_key": 1, "force_tls": "yes", "other": 2}"#);
    let rejected = config
        .diagnostics()
        .iter()
        .find_map(|d| match &d.code {
            DiagnosticCode::NetConfigRejected { errors, .. } => Some(errors.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        rejected,
        " -> bogus_key:  this member was found in the input, but is not present in the Schema\n -> force_tls:  invalid numeric value\n -> other:  this member was found in the input, but is not present in the Schema"
    );
    assert_eq!(config.diagnostics().len(), 2);
}

#[test]
fn sc41_net_force_https_reads_as_force_tls() {
    let config = check_config("sc41_net_force_https");
    let net = configured(&config);
    assert_eq!(net.force_tls, Some(true));
    assert!(net.requires_tls());
}

#[test]
fn n3_force_https_replaces_a_force_tls_beside_it() {
    // `v["force_tls"] = v["force_https"]`: the legacy member's value stands.
    let config = net(r#"{"force_tls": true, "force_https": false}"#);
    assert_eq!(configured(&config).force_tls, Some(false));
    assert_eq!(
        findings(&config),
        [
            (
                "DeprecatedMember".to_owned(),
                "info".to_owned(),
                "force_https".to_owned()
            ),
            (
                "NoEffect".to_owned(),
                "info".to_owned(),
                "force_https".to_owned()
            )
        ]
    );
    // A wrong type under the legacy name is reported as `force_tls`, as after the rename.
    let config = net(r#"{"force_https": "yes"}"#);
    let errors = config
        .diagnostics()
        .iter()
        .find_map(|d| match &d.code {
            DiagnosticCode::NetConfigRejected { errors, .. } => Some(errors.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(errors, " -> force_tls:  invalid numeric value");
}

#[test]
fn sc42_net_force_tls_false_still_requires_tls_and_is_warned_deviation() {
    let config = check_config("sc42_net_force_tls_false");
    let net = configured(&config);
    assert_eq!(net.force_tls, Some(false));
    assert!(net.requires_tls());
    assert!(config.default.permissions.server_net.requires_tls());
    // Without the member plain HTTP is allowed; `true` requires TLS without a finding.
    let absent = self::net("{}");
    assert!(!configured(&absent).requires_tls());
    let forced = self::net(r#"{"force_tls": true}"#);
    assert!(configured(&forced).requires_tls());
    assert!(forced.diagnostics().is_empty());
}

#[test]
fn sc43_net_wildcard_uri_is_a_literal_prefix() {
    let config = check_config("sc43_net_wildcard_uri");
    let net = configured(&config);
    assert!(!net.allows_uri("http://127.0.0.1:9/allowed/x"));
    assert!(net.allows_uri("http://127.0.0.1:9/*"));
}

#[test]
fn n6_an_entry_that_is_no_absolute_uri_is_a_warning() {
    for (uri, count) in [
        ("example.com/api", 1),
        ("ftp://example.com/", 1),
        ("https://", 1),
        ("*", 2),
        ("https://example.com/", 0),
        ("ws://example.com/", 0),
        ("wss://example.com/", 0),
        ("http://example.com", 0),
    ] {
        let config = net(&format!("{{\"allowed_uris\": [\"{uri}\"]}}"));
        let found = findings(&config);
        assert_eq!(found.len(), count, "{uri}: {found:?}");
        for finding in found {
            assert_eq!(finding, one("InvalidValue", "info", "allowed_uris/0")[0]);
        }
        // A warning only: the entry loads.
        assert_eq!(
            configured(&config).allowed_uris.as_deref(),
            Some(&[uri.to_owned()][..])
        );
    }
}

#[test]
fn n6_an_entry_with_an_upper_case_scheme_admits_no_request_and_is_warned() {
    // The request's scheme compares in lower case and the entry is a plain prefix, so an
    // entry whose scheme has a capital can match nothing, however the request is written.
    for uri in ["HTTPS://x/", "Https://x/", "WSS://x/", "hTTp://x/"] {
        let config = net(&format!("{{\"allowed_uris\": [\"{uri}\"]}}"));
        assert_eq!(
            findings(&config),
            one("InvalidValue", "info", "allowed_uris/0"),
            "{uri}"
        );
        let net = configured(&config);
        let lower = format!("{}y", uri.to_ascii_lowercase());
        for request in [format!("{uri}y"), lower, uri.to_ascii_uppercase()] {
            assert!(!net.allows_uri(&request), "{uri} admits {request}");
        }
    }
}

#[test]
fn n7_an_empty_allow_list_refuses_every_request() {
    let config = net(r#"{"allowed_uris": []}"#);
    assert_eq!(findings(&config), one("NoEffect", "info", "allowed_uris"));
    let net = configured(&config);
    assert_eq!(net.allowed_uris.as_deref(), Some(&[][..]));
    assert!(!net.allows_uri("https://example.com/"));
}

#[test]
fn sc44_net_payload_size_loads() {
    let config = check_config("sc44_net_payload_size");
    let net = configured(&config);
    assert_eq!(net.max_websocket_payload_size, Some(100));
    // The probe's requests [bds-run F `netpayload`].
    assert!(net.allows_uri("http://127.0.0.1:9/allowed/x"));
    assert!(net.allows_uri("http://127.0.0.1:9/allowedXYZ"));
    assert!(!net.allows_uri("http://127.0.0.1:9/other"));
    assert!(!net.allows_uri("https://127.0.0.1:9/allowed/x"));
    assert!(net.allows_uri("HTTP://127.0.0.1:9/allowed/x"));
    assert!(!net.requires_tls());
}

#[test]
fn sc45_net_not_object_denies_egress_deviation() {
    // Vanilla: no settings, requests unrestricted, silent.
    let config = check_config("sc45_net_not_object");
    assert_eq!(config.default.permissions.server_net, ServerNet::Denied);
    assert!(config.default.permissions.module_permissions.is_empty());
    assert_eq!(config.diagnostics().max_severity(), Some(Severity::Info));
}

#[test]
fn sc47_net_duplicate_member_keeps_the_last() {
    // As on the server (the entry passes through a jsoncpp map first); validation N9's
    // "rejected" was written before the core kept the last of two members.
    let config = check_config("sc47_net_duplicate_member");
    let net = configured(&config);
    assert!(net.allows_uri("http://127.0.0.1:9/b"));
    assert!(!net.allows_uri("http://127.0.0.1:9/a"));
}

#[test]
fn a_span_after_a_member_written_twice_is_where_the_value_is() {
    let text = r#"{"module_permissions": {"@minecraft/server-net": {"force_tls": true, "force_tls": false, "max_body_bytes": -1}}}"#;
    let mut folder = ConfigFolder::new("default");
    folder.permissions = Some(Bytes::from_static(text.as_bytes()));
    let config = ScriptConfig::parse(&ScriptConfigFiles {
        folders: vec![folder],
        ..ScriptConfigFiles::default()
    });
    let finding = config.diagnostics().iter().next().unwrap();
    assert!(matches!(finding.code, DiagnosticCode::InvalidValue { .. }));
    let offset = u32::try_from(text.find("-1").unwrap()).unwrap();
    assert_eq!(finding.location.span, Some(Span { offset, len: 2 }));
}
