//! The cost of the places of this family that look a thing up per rule, each measured on its own
//! on inputs built beforehand, by the crate's one protocol (`bedrock_addon_core::test_cost`): four times the
//! input takes about four times the time.

use super::{Facing, GeometryPart, Rule, validate};
use bedrock_addon_core::test_cost::assert_linear_at;

/// The smaller input size; the larger is four times it.
const SINGLE: usize = 25_000;

/// `count` rules, each a face of a cube of its own: no part comes twice.
fn distinct_rules(count: usize) -> Vec<Rule> {
    (0..count)
        .map(|n| {
            let cube = i32::try_from(n).unwrap_or(i32::MAX);
            Rule::new(
                GeometryPart::cube_face(format!("b{}", n % 7), cube, Facing::Up),
                Facing::Down,
            )
        })
        .collect()
}

#[test]
fn the_rule_checks_are_linear_in_the_rules() {
    assert_linear_at(
        "validate::rules, no part twice",
        2 * SINGLE,
        distinct_rules,
        |rules| assert!(validate::rules(rules).is_empty()),
    );
}

#[test]
fn the_rule_checks_are_linear_when_every_other_rule_repeats_a_part() {
    // Each part twice: the set holds half the rules, and half the rules are findings.
    let repeated = |count: usize| {
        let mut rules = distinct_rules(count / 2);
        rules.extend(distinct_rules(count / 2));
        rules
    };
    assert_linear_at(
        "validate::rules, every part twice",
        2 * SINGLE,
        repeated,
        |rules| assert_eq!(validate::rules(rules).len(), rules.len() / 2),
    );
}

/// A payload of `count` rules, each with a member the schema does not have, so each rule
/// is a finding that needs its span. The payload is split once at its root and once at
/// `rules`; a span looked up from the root per finding would read every rule per finding.
#[test]
fn the_spans_of_a_finding_per_rule_are_linear_in_the_rules() {
    use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
    use bedrock_addon_core::json::JsonSlice;
    let payload = |count: usize| {
        let rules: Vec<String> = (0..count)
            .map(|n| {
                format!(
                    r#"{{"geometry_part": {{"bone": "b{n}"}}, "direction": "up", "stray": {n}}}"#
                )
            })
            .collect();
        format!(
            r#"{{"description": {{"identifier": "demo:rules"}}, "rules": [{}]}}"#,
            rules.join(", ")
        )
    };
    assert_linear_at(
        "culling::read, a finding per rule",
        2_000,
        payload,
        |text| {
            let mut d = Diagnostics::default();
            let base = JsonPath::new().child("minecraft:block_culling_rules");
            let read = super::culling::read(
                JsonSlice { text, offset: 7 },
                super::culling::Slice::V1_21_80,
                &base,
                &mut d,
            );
            assert!(read.is_none());
            // Every finding has the span of its rule's member, at the file's offsets.
            let rules = text.matches("\"stray\"").count();
            assert_eq!(d.len(), rules);
            assert!(d.iter().all(|found| {
                found.location.span.is_some_and(|span| {
                    let from = span.offset as usize - 7;
                    text[from..from + span.len as usize]
                        .parse::<usize>()
                        .is_ok()
                })
            }));
        },
    );
}
