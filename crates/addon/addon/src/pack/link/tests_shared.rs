//! The resolvers of the shared descriptor types: every arm of each.

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json;

use super::super::{Findings, LinkCx, LinkOptions, Member, States, link_with_rules};
use super::support::{all_said, block, item, origin, set};

/// The descriptor's path in its file: `minecraft:entity/components/demo:holder`.
fn prefix() -> JsonPath {
    let mut path = JsonPath::new();
    for key in ["minecraft:entity", "components", "demo:holder"] {
        path.push(key);
    }
    path
}

const AT: &str = "minecraft:entity/components/demo:holder";

/// What one resolver call per `(descriptor JSON, severity)` of `CASES` tells, over a world with
/// the block `demo:lamp` (state `demo:lit`: true, false; `demo:level`: 0 to 2) and the item
/// `demo:ruby` (tag `demo:gem`).
fn told(lints: bool, resolve: for<'a, 'b, 'c> fn(&'a LinkCx<'b>, &'c mut Findings)) -> Diagnostics {
    let lamp = block(
        "demo:lamp",
        r#"{"demo:lit": [true, false], "demo:level": {"values": {"min": 0, "max": 2}}}"#,
    );
    let ruby = item("demo:ruby", r#""minecraft:tags": {"tags": ["demo:gem"]}"#);
    let set = set(
        &[("blocks/lamp.json", &lamp), ("items/ruby.json", &ruby)],
        &[],
    );
    link_with_rules(
        &set,
        LinkOptions {
            lints,
            ..Default::default()
        },
        &[resolve],
    )
}

#[cfg(feature = "item")]
mod items {
    use bedrock_addon_core::shared::ItemDescriptor;

    use super::super::super::item_descriptor;
    use super::*;

    fn resolve_all(cx: &LinkCx<'_>, out: &mut Findings, cases: &[(&str, Severity)]) {
        let origin = origin("entities/pet.json");
        let path = prefix();
        for (text, severity) in cases {
            let descriptor: ItemDescriptor = json::from_str(text).expect("a descriptor");
            let member = Member {
                origin: &origin,
                path: &path,
                severity: *severity,
                validator: None,
                states: States::Lint,
            };
            item_descriptor(cx, out, &descriptor, member);
        }
    }

    #[test]
    fn an_item_arm_resolves_or_is_reported_at_the_member_that_names_it() {
        fn rule(cx: &LinkCx<'_>, out: &mut Findings) {
            resolve_all(
                cx,
                out,
                &[
                    // Resolved: a pack's item, with an aux value, the game's, a legacy name.
                    (r#""demo:ruby""#, Severity::Warning),
                    (r#"{"item": "demo:ruby:2"}"#, Severity::Warning),
                    (r#"{"name": "minecraft:apple"}"#, Severity::Warning),
                    (r#""minecraft:horsearmorgold""#, Severity::Warning),
                    // Missing, in each of the three forms and with each row's severity.
                    (r#""demo:none""#, Severity::Warning),
                    (r#"{"item": "demo:none"}"#, Severity::Info),
                    (r#"{"name": "minecraft:not_an_item"}"#, Severity::Error),
                ],
            );
        }
        let miss = |severity, below: &str, id: &str| {
            (
                "UnresolvedReference",
                severity,
                format!("{AT}{below}"),
                "item",
                id.to_owned(),
            )
        };
        let vanilla = miss(Severity::Error, "/name", "minecraft:not_an_item");
        // Without the block family a name outside `minecraft` may be a pack's block and is not
        // judged.
        #[cfg(feature = "block")]
        let expected = vec![
            miss(Severity::Warning, "", "demo:none"),
            miss(Severity::Info, "/item", "demo:none"),
            vanilla,
        ];
        #[cfg(not(feature = "block"))]
        let expected = vec![vanilla];
        assert_eq!(all_said(&told(true, rule)), expected);
    }

    /// With the block family in view a block of the stack has an item.
    #[cfg(feature = "block")]
    #[test]
    fn the_item_of_a_block_of_the_stack_resolves() {
        fn rule(cx: &LinkCx<'_>, out: &mut Findings) {
            resolve_all(cx, out, &[(r#""demo:lamp""#, Severity::Warning)]);
        }
        assert_eq!(all_said(&told(true, rule)), []);
    }

    #[test]
    fn an_item_tag_nothing_carries_is_a_lint_whatever_the_rows_severity() {
        fn rule(cx: &LinkCx<'_>, out: &mut Findings) {
            resolve_all(
                cx,
                out,
                &[
                    (r#"{"item_tag": "demo:gem"}"#, Severity::Warning),
                    (r#"{"item_tag": "demo:none"}"#, Severity::Warning),
                    (r#"{"item_tag": "demo:Gem"}"#, Severity::Error),
                ],
            );
        }
        let lint = |tag: &str| {
            (
                "UnresolvedReference",
                Severity::Info,
                format!("{AT}/item_tag"),
                "item tag",
                tag.to_owned(),
            )
        };
        assert_eq!(
            all_said(&told(true, rule)),
            [lint("demo:none"), lint("demo:Gem")]
        );
        // Without the lints nothing is told of a tag.
        assert_eq!(all_said(&told(false, rule)), []);
    }

    #[test]
    fn a_tag_expression_and_an_invalid_shape_resolve_to_nothing() {
        fn rule(cx: &LinkCx<'_>, out: &mut Findings) {
            resolve_all(
                cx,
                out,
                &[
                    (r#"{"tags": "q.any_tag('demo:none')"}"#, Severity::Error),
                    ("5", Severity::Error),
                    (r#"{"other": "demo:none"}"#, Severity::Error),
                ],
            );
        }
        assert_eq!(all_said(&told(true, rule)), []);
    }
}

#[cfg(feature = "block")]
mod blocks {
    use bedrock_addon_core::shared::{BlockDescriptor, BlockSpecifier};

    use super::super::super::{block_descriptor, block_specifier};
    use super::*;

    fn resolve_all(cx: &LinkCx<'_>, out: &mut Findings, cases: &[(&str, Severity)]) {
        let origin = origin("entities/pet.json");
        let path = prefix();
        for (text, severity) in cases {
            let descriptor: BlockDescriptor = json::from_str(text).expect("a descriptor");
            // A row that grades a miss an error grades its states so.
            let states = if *severity == Severity::Error {
                States::Error
            } else {
                States::Lint
            };
            let member = Member {
                origin: &origin,
                path: &path,
                severity: *severity,
                validator: None,
                states,
            };
            block_descriptor(cx, out, &descriptor, member);
        }
    }

    fn miss(
        code: &'static str,
        severity: Severity,
        below: &str,
        kind: &'static str,
        id: &str,
    ) -> (&'static str, Severity, String, &'static str, String) {
        (code, severity, format!("{AT}{below}"), kind, id.to_owned())
    }

    #[test]
    fn a_block_name_resolves_or_is_reported_at_the_member_that_names_it() {
        fn rule(cx: &LinkCx<'_>, out: &mut Findings) {
            resolve_all(
                cx,
                out,
                &[
                    (r#""demo:lamp""#, Severity::Warning),
                    (r#"{"name": "demo:lamp"}"#, Severity::Warning),
                    (r#""minecraft:stone""#, Severity::Warning),
                    (r#""stone""#, Severity::Warning),
                    (r#""demo:none""#, Severity::Warning),
                    (r#"{"name": "demo:none"}"#, Severity::Error),
                    (
                        r#"{"name": "demo:none", "states": {"demo:lit": true}}"#,
                        Severity::Info,
                    ),
                ],
            );
        }
        assert_eq!(
            all_said(&told(true, rule)),
            [
                miss(
                    "UnresolvedReference",
                    Severity::Warning,
                    "",
                    "block",
                    "demo:none"
                ),
                miss(
                    "UnresolvedReference",
                    Severity::Error,
                    "/name",
                    "block",
                    "demo:none"
                ),
                // A block that is missing is reported once: its states are not looked at.
                miss(
                    "UnresolvedReference",
                    Severity::Info,
                    "/name",
                    "block",
                    "demo:none"
                ),
            ]
        );
        // The information is a lint like any other.
        assert_eq!(told(false, rule).iter().count(), 2);
    }

    #[test]
    fn the_states_of_a_block_of_the_stack_are_held_to_its_states_and_their_values() {
        fn rule(cx: &LinkCx<'_>, out: &mut Findings) {
            resolve_all(
                cx,
                out,
                &[
                    // Every value of each domain, both ends of the range.
                    (
                        r#"{"name": "demo:lamp", "states": {"demo:lit": true}}"#,
                        Severity::Warning,
                    ),
                    (
                        r#"{"name": "demo:lamp", "states": {"demo:lit": false, "demo:level": 0}}"#,
                        Severity::Warning,
                    ),
                    (
                        r#"{"name": "demo:lamp", "states": {"demo:level": 2}}"#,
                        Severity::Warning,
                    ),
                    // An unknown state; a value below, above and of another type.
                    (
                        r#"{"name": "demo:lamp", "states": {"demo:none": 1, "demo:level": -1}}"#,
                        Severity::Warning,
                    ),
                    (
                        r#"{"name": "demo:lamp", "states": {"demo:level": 3}}"#,
                        Severity::Error,
                    ),
                    (
                        r#"{"name": "demo:lamp", "states": {"demo:lit": "true"}}"#,
                        Severity::Warning,
                    ),
                    // The game's own block: its states are not known here.
                    (
                        r#"{"name": "minecraft:stone", "states": {"demo:none": 1}}"#,
                        Severity::Warning,
                    ),
                ],
            );
        }
        // A state of a block of the stack is the crate's own check, which the game does
        // not make: an information unless the row grades its states otherwise.
        let i = Severity::Info;
        assert_eq!(
            all_said(&told(true, rule)),
            [
                miss(
                    "UnresolvedReference",
                    i,
                    "/states/demo:none",
                    "block state",
                    "demo:none"
                ),
                miss(
                    "InvalidReference",
                    i,
                    "/states/demo:level",
                    "block state",
                    "demo:level"
                ),
                miss(
                    "InvalidReference",
                    Severity::Error,
                    "/states/demo:level",
                    "block state",
                    "demo:level"
                ),
                miss(
                    "InvalidReference",
                    i,
                    "/states/demo:lit",
                    "block state",
                    "demo:lit"
                ),
            ]
        );
    }

    #[test]
    fn tags_a_mixed_descriptor_and_an_invalid_shape_resolve_to_nothing() {
        fn rule(cx: &LinkCx<'_>, out: &mut Findings) {
            resolve_all(
                cx,
                out,
                &[
                    (r#"{"tags": "q.any_tag('demo:none')"}"#, Severity::Error),
                    (
                        r#"{"name": "demo:none", "tags": "q.any_tag('stone')"}"#,
                        Severity::Error,
                    ),
                    ("5", Severity::Error),
                ],
            );
        }
        assert_eq!(all_said(&told(true, rule)), []);
    }

    #[test]
    fn a_block_specifier_is_held_to_the_rules_of_a_block_descriptors_block() {
        fn rule(cx: &LinkCx<'_>, out: &mut Findings) {
            let origin = origin("entities/pet.json");
            let path = prefix();
            for (text, severity) in [
                (r#""demo:lamp""#, Severity::Error),
                (
                    r#"{"name": "demo:lamp", "states": {"demo:level": 1}}"#,
                    Severity::Error,
                ),
                (r#""demo:none""#, Severity::Error),
                (r#"{"name": "Demo:Lamp"}"#, Severity::Warning),
                (
                    r#"{"name": "demo:lamp", "states": {"demo:none": 1, "demo:lit": 1}}"#,
                    Severity::Warning,
                ),
            ] {
                let specifier: BlockSpecifier = json::from_str(text).expect("a specifier");
                let member = Member {
                    origin: &origin,
                    path: &path,
                    severity,
                    validator: None,
                    states: States::Lint,
                };
                block_specifier(cx, out, &specifier, member);
            }
        }
        let found = told(true, rule);
        let w = Severity::Warning;
        assert_eq!(
            all_said(&found),
            [
                miss(
                    "UnresolvedReference",
                    Severity::Error,
                    "",
                    "block",
                    "demo:none"
                ),
                miss("UnresolvedReference", w, "/name", "block", "Demo:Lamp"),
                // The states are the crate's own check: informations.
                miss(
                    "UnresolvedReference",
                    Severity::Info,
                    "/states/demo:none",
                    "block state",
                    "demo:none"
                ),
                miss(
                    "InvalidReference",
                    Severity::Info,
                    "/states/demo:lit",
                    "block state",
                    "demo:lit"
                ),
            ]
        );
        // The near miss of the lookup is in the finding.
        let hinted = found.iter().nth(1).expect("the second finding");
        assert!(
            hinted.to_string().ends_with("(did you mean 'demo:lamp'?)"),
            "{hinted}"
        );
    }
}
