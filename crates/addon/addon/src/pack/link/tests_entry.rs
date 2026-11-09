//! The entry point and the sink: what `StackSet::link` promises whatever the rules are.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, JsonPath, Location, Phase, Severity};

use super::support::{empty_set, set};
use crate::pack::link::{Findings, LinkCx, LinkOptions, link_with_rules};

/// A finding of `severity` and `phase` at the JSON member `name`.
fn finding(name: &str, severity: Severity, phase: Phase) -> Diagnostic {
    let mut path = JsonPath::new();
    path.push(name);
    Diagnostic::new(
        Code::from(&core_codes::UNRESOLVED_REFERENCE),
        severity,
        phase,
        Location::new(None, None, path, None),
    )
}

/// A made-up lint: an information, told only when the lints run.
fn lint(cx: &LinkCx<'_>, out: &mut Findings) {
    assert_eq!(cx.lints(), out.lints(), "the context and the sink agree");
    if cx.lints() {
        out.push(finding("lint", Severity::Info, Phase::Link));
    }
}

/// A made-up lint that does not ask whether the lints run.
fn careless_lint(_: &LinkCx<'_>, out: &mut Findings) {
    out.push(finding("careless", Severity::Info, Phase::Link));
    out.push(finding("debug", Severity::Debug, Phase::Link));
}

/// A made-up rule: a warning and an error, built in another phase.
fn rule(_: &LinkCx<'_>, out: &mut Findings) {
    out.extend([
        finding("warning", Severity::Warning, Phase::Validate),
        finding("error", Severity::Error, Phase::Parse),
    ]);
}

fn names(findings: &bedrock_addon_core::diagnostic::Diagnostics) -> Vec<String> {
    findings
        .iter()
        .map(|finding| finding.location.json_path.to_string())
        .collect()
}

#[test]
fn an_empty_world_links_to_nothing() {
    let set = empty_set();
    assert!(set.link().is_empty());
    assert!(
        set.link_with(&LinkOptions {
            lints: false,
            ..Default::default()
        })
        .is_empty()
    );
}

#[test]
fn the_default_options_run_the_lints() {
    assert_eq!(
        LinkOptions::default(),
        LinkOptions {
            lints: true,
            ..Default::default()
        }
    );
}

#[test]
fn the_rules_tell_their_findings_in_the_order_they_run() {
    let set = empty_set();
    let found = link_with_rules(&set, LinkOptions::default(), &[rule, lint, careless_lint]);
    assert_eq!(
        names(&found),
        ["warning", "error", "lint", "careless", "debug"]
    );
    let found = link_with_rules(&set, LinkOptions::default(), &[lint, rule]);
    assert_eq!(names(&found), ["lint", "warning", "error"]);
}

#[test]
fn without_the_lints_exactly_the_informations_are_left_out() {
    let set = empty_set();
    let rules = [rule, lint, careless_lint];
    let with = link_with_rules(
        &set,
        LinkOptions {
            lints: true,
            ..Default::default()
        },
        &rules,
    );
    let without = link_with_rules(
        &set,
        LinkOptions {
            lints: false,
            ..Default::default()
        },
        &rules,
    );
    // The lint did no work, and the sink dropped what a careless lint told all the same.
    assert_eq!(names(&without), ["warning", "error"]);
    let kept: Vec<&Diagnostic> = with
        .iter()
        .filter(|finding| finding.severity > Severity::Info)
        .collect();
    assert_eq!(kept, without.iter().collect::<Vec<_>>());
    assert_eq!(with.iter().count(), 5);
    // An information is left out, a warning never is: both sides of the bound.
    assert!(without.iter().all(|f| f.severity >= Severity::Warning));
    assert!(with.iter().any(|f| f.severity == Severity::Info));
    assert!(with.iter().any(|f| f.severity == Severity::Debug));
}

#[test]
fn every_finding_is_of_the_link_phase() {
    let set = empty_set();
    let found = link_with_rules(&set, LinkOptions::default(), &[rule, lint]);
    assert_eq!(found.iter().count(), 3);
    assert!(found.iter().all(|finding| finding.phase == Phase::Link));
}

#[test]
fn link_is_link_with_the_default_options() {
    let set = set(&[], &[]);
    assert_eq!(set.link(), set.link_with(&LinkOptions::default()));
}

#[test]
fn findings_extend_through_the_std_trait_and_keep_the_sink_rules() {
    fn tell_all<E: Extend<Diagnostic>>(sink: &mut E, findings: Vec<Diagnostic>) {
        sink.extend(findings);
    }
    let mut out = Findings::new(LinkOptions {
        lints: false,
        ..Default::default()
    });
    tell_all(
        &mut out,
        vec![
            finding("info", Severity::Info, Phase::Link),
            finding("error", Severity::Error, Phase::Parse),
        ],
    );
    let told = out.finish();
    assert_eq!(names(&told), ["error"], "the information is dropped");
    assert!(told.iter().all(|finding| finding.phase == Phase::Link));
}
