//! `SchemaCx`, where a derived `Schema::check` reports: the slice's `on_member_failure` grades
//! a failed member, not the struct.

use bedrock_addon_core::diagnostic::cereal::NumericRange;
use bedrock_addon_core::diagnostic::codes;
use bedrock_addon_core::diagnostic::{DiagnosticCode, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::schema::{SchemaCx, Verdict};
use bedrock_addon_core::versioning::{MemberFailure, Reading};

fn at(keys: &[&str]) -> JsonPath {
    let mut path = JsonPath::new();
    for key in keys {
        path.push(key);
    }
    path
}

#[test]
fn a_refusing_slice_fails_the_owner_with_an_error_at_the_member() {
    let mut out = Diagnostics::default();
    let mut cx = SchemaCx::new(Reading::DOCUMENT, at(&["boss"]), &mut out);
    cx.enter("radius");
    assert_eq!(cx.range(NumericRange::at_least(0.0), true), Verdict::Keep);
    cx.leave();
    assert!(cx.failed());
    assert_eq!(cx.path(), &at(&["boss"]));
    let findings: Vec<_> = out.iter().collect();
    let [finding] = findings.as_slice() else {
        panic!("one finding: {out:?}")
    };
    assert_eq!(finding.severity, Severity::Error);
    assert_eq!(finding.location.json_path, at(&["boss", "radius"]));
    assert_eq!(finding.code, DiagnosticCode::from(&codes::INVALID_VALUE));
}

#[test]
fn a_range_finding_has_the_game_range_text_only_where_the_family_opts_in() {
    let range = NumericRange::inclusive(0.0, 0.9);
    let mut out = Diagnostics::default();
    let mut cx = SchemaCx::new(Reading::DOCUMENT, JsonPath::new(), &mut out).with_range_text(true);
    cx.range(range, true);
    let mut plain = Diagnostics::default();
    SchemaCx::new(Reading::DOCUMENT, JsonPath::new(), &mut plain).range(range, true);
    let codes: Vec<_> = out
        .iter()
        .chain(plain.iter())
        .map(|d| d.code.clone())
        .collect();
    assert_eq!(
        codes,
        [range.code(), DiagnosticCode::from(&codes::INVALID_VALUE)]
    );
}

#[test]
fn default_and_tell_resets_the_member_with_an_information() {
    let mut out = Diagnostics::default();
    let reading = Reading::DOCUMENT.with_on_member_failure(MemberFailure::DefaultAndTell);
    let mut cx = SchemaCx::new(reading, JsonPath::new(), &mut out);
    cx.enter("speed");
    let exclusive = NumericRange {
        exclusive_min: true,
        ..NumericRange::at_least(1.0)
    };
    assert_eq!(cx.range(exclusive, true), Verdict::Reset);
    assert!(!cx.failed());
    let severities: Vec<Severity> = out.iter().map(|d| d.severity).collect();
    assert_eq!(severities, [Severity::Info]);
    assert!(out.iter().all(|d| d.is(&codes::INVALID_VALUE)));
}

#[test]
fn a_member_with_nothing_to_reset_to_fails_its_owner_at_the_rules_severity() {
    let mut out = Diagnostics::default();
    let reading = Reading::DOCUMENT.with_on_member_failure(MemberFailure::DefaultAndTell);
    let mut cx = SchemaCx::new(reading, JsonPath::new(), &mut out);
    assert_eq!(cx.range(NumericRange::at_least(1.0), false), Verdict::Keep);
    assert!(cx.failed());
    assert_eq!(
        out.iter().map(|d| d.severity).collect::<Vec<_>>(),
        [Severity::Info]
    );
}
