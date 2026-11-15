//! `SchemaCx`, where a derived `Schema::check` reports: the slice's `on_member_failure` grades
//! a failed member, not the struct.

use bedrock_addon_core::diagnostic::cereal::NumericRange;
use bedrock_addon_core::diagnostic::codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostics, JsonPath, Severity};
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
    assert_eq!(finding.code, Code::from(&codes::INVALID_VALUE));
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
    assert_eq!(codes, [range.code(), Code::from(&codes::INVALID_VALUE)]);
}

#[test]
fn a_one_sided_range_has_its_own_text_switch() {
    let both = NumericRange::inclusive(0.0, 0.9);
    let one = NumericRange::at_least(0.0);
    let mut out = Diagnostics::default();
    let mut cx = SchemaCx::new(Reading::DOCUMENT, JsonPath::new(), &mut out)
        .with_range_text(true)
        .with_one_sided_range_text(false);
    cx.range(both, true);
    cx.range(one, true);
    let mut other = Diagnostics::default();
    let mut cx = SchemaCx::new(Reading::DOCUMENT, JsonPath::new(), &mut other)
        .with_one_sided_range_text(true);
    cx.range(both, true);
    cx.range(one, true);
    let codes: Vec<_> = out
        .iter()
        .chain(other.iter())
        .map(|d| d.code.clone())
        .collect();
    assert_eq!(
        codes,
        [
            both.code(),
            Code::from(&codes::INVALID_VALUE),
            Code::from(&codes::INVALID_VALUE),
            one.code()
        ]
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

#[test]
fn a_rule_tells_an_error_that_fails_the_owner_and_a_kept_one_that_does_not() {
    use bedrock_addon_core::diagnostic::Phase;
    let mut out = Diagnostics::default();
    let mut cx =
        SchemaCx::new(Reading::DOCUMENT, at(&["tree"]), &mut out).with_phase(Phase::Validate);
    cx.kept("offset", Code::from(&codes::INVALID_VALUE));
    assert!(!cx.failed());
    cx.error("", Code::from(&codes::INVALID_TYPE));
    assert!(cx.failed());
    let told: Vec<_> = out
        .iter()
        .map(|d| {
            (
                d.location.json_path.to_string(),
                d.kept,
                d.phase,
                d.severity,
            )
        })
        .collect();
    assert_eq!(
        told,
        [
            (
                "tree/offset".to_owned(),
                true,
                Phase::Validate,
                Severity::Error
            ),
            ("tree".to_owned(), false, Phase::Validate, Severity::Error),
        ]
    );
}

#[test]
fn a_leaf_hands_its_own_findings_to_the_walk_at_the_path_it_located_them() {
    let mut own = Diagnostics::default();
    own.push(
        bedrock_addon_core::diagnostic::Diagnostic::warn(&codes::INVALID_VALUE).at(at(&["a", "b"])),
    );
    let mut out = Diagnostics::default();
    let mut cx = SchemaCx::new(Reading::NULL_ABSENT, JsonPath::new(), &mut out);
    assert_eq!(cx.unknown_severity(), Severity::Warning);
    assert!(!cx.leaf_findings());
    cx.extend(own);
    assert_eq!(out.len(), 1);
}

#[test]
fn a_rule_bounds_a_number_as_a_derived_bound_is_worded() {
    use bedrock_addon_core::schema::TreeWording;
    static MIN: bedrock_addon_core::diagnostic::CodeDef<3> =
        bedrock_addon_core::diagnostic::CodeDef::new(
            "test/min",
            bedrock_addon_core::diagnostic::Kind::InvalidValue,
            bedrock_addon_core::diagnostic::Class::Error,
            "{} {} {}",
        );
    let mut out = Diagnostics::default();
    let mut cx = SchemaCx::new(Reading::DOCUMENT, at(&["leveled"]), &mut out).with_tree_wording(
        TreeWording {
            range_min: &MIN,
            range_max: &MIN,
        },
    );
    assert!(!cx.within("max_steepness", 8, Some(0), Some(8)));
    assert!(cx.within("max_steepness", 9, Some(0), Some(8)));
    assert!(cx.failed());
    assert_eq!(cx.path(), &at(&["leveled"]));
    let told: Vec<_> = out
        .iter()
        .map(|d| (d.location.json_path.to_string(), d.code.to_string()))
        .collect();
    assert_eq!(
        told,
        [(
            "leveled/max_steepness".to_owned(),
            "Value '9' outside valid range [0, 8]".to_owned()
        )]
    );
}
