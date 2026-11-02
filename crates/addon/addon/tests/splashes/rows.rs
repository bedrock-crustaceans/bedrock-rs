//! `splashes.json`: reading one copy, the stack's pool, the round trip and the cost of a long
//! list.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::{Content, Placed};
use bedrock_addon::families::splashes::{SplashClient, Splashes, splash_pool};
use bedrock_addon::load::LoadContext;
use bedrock_addon::pack::{PackStack, Resource};
use bedrock_addon::storage::PackPath;

use super::common::{codes, json, located, pack_number, stack_of, subpack_stacks};

const BOTTOM: &str = r#"{ "splashes": ["a", "b"] }"#;

/// The full game's pool of a stack.
fn pool(stack: &PackStack<Resource>) -> Vec<String> {
    let merged = stack.splashes();
    let pool = merged.texts();
    assert_eq!(
        pool,
        splash_pool(merged.layers(), &SplashClient::default()),
        "the merged value is the default client's pool"
    );
    pool
}

fn stack_codes(stack: &PackStack<Resource>) -> Vec<(&'static str, Severity)> {
    codes(stack.splashes().diagnostics())
}

fn parse(text: &str) -> bedrock_addon::document::Document<Splashes> {
    Splashes::parse(
        &PackPath::from_static("splashes.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .expect("a splashes file")
}

#[test]
fn sp01_a_copy_without_can_merge_replaces_the_lower_splashes() {
    let stack = stack_of(&[
        &[("splashes.json", BOTTOM)],
        &[("splashes.json", r#"{ "splashes": ["c"] }"#)],
    ]);
    assert_eq!(pool(&stack), ["c"]);
    let found: Vec<_> = stack.splashes().diagnostics().collect();
    assert_eq!(
        codes(found.iter().copied()),
        [("SplashesReplaced", Severity::Info)]
    );
    assert_eq!(pack_number(found[0], 2), Some(1), "about the top pack");
    assert_eq!(
        found[0].location.path.as_ref().unwrap().as_str(),
        "splashes.json"
    );
}

#[test]
fn s3_the_lowest_copy_replaces_nothing_and_is_not_reported() {
    let stack = stack_of(&[&[("other.txt", "")], &[("splashes.json", BOTTOM)]]);
    assert_eq!(pool(&stack), ["a", "b"]);
    assert_eq!(stack_codes(&stack), []);
}

#[test]
fn s3_can_merge_false_replaces_and_is_reported() {
    let stack = stack_of(&[
        &[("splashes.json", BOTTOM)],
        &[(
            "splashes.json",
            r#"{ "canMerge": false, "splashes": ["c"] }"#,
        )],
    ]);
    assert_eq!(pool(&stack), ["c"]);
    assert_eq!(stack_codes(&stack), [("SplashesReplaced", Severity::Info)]);
}

#[test]
fn sp02_can_merge_true_adds_after_the_lower_splashes() {
    let stack = stack_of(&[
        &[("splashes.json", BOTTOM)],
        &[(
            "splashes.json",
            r#"{ "canMerge": true, "splashes": ["c"] }"#,
        )],
    ]);
    assert_eq!(pool(&stack), ["a", "b", "c"]);
    assert_eq!(stack_codes(&stack), []);
    let merged = stack.splashes();
    let origin = |text: &str| pack_number_of(merged.origin(&text.to_owned()).unwrap());
    assert_eq!((origin("a"), origin("c")), (0, 1));
    assert_eq!(merged.layers().len(), 2);
    assert_eq!(merged.layers()[0].texts(), ["c"], "layers are top first");
}

fn pack_number_of(origin: &bedrock_addon::document::Origin) -> usize {
    1 - origin.pack.index().unwrap()
}

#[test]
fn sp03_a_string_true_does_not_merge() {
    let stack = stack_of(&[
        &[("splashes.json", BOTTOM)],
        &[(
            "splashes.json",
            r#"{ "canMerge": "true", "splashes": ["c"] }"#,
        )],
    ]);
    assert_eq!(pool(&stack), ["c"]);
    assert_eq!(
        located(stack.splashes().diagnostics()),
        [
            ("InvalidValue", Severity::Info, "canMerge".to_owned()),
            ("SplashesReplaced", Severity::Info, String::new()),
        ]
    );
}

#[test]
fn s2_every_non_boolean_can_merge_replaces() {
    for written in ["1", "null", "[true]", "{}", "1.0"] {
        let doc = parse(&format!(r#"{{ "canMerge": {written}, "splashes": [] }}"#));
        assert!(!doc.merges(), "{written}");
        assert_eq!(
            located(doc.diagnostics()),
            [("InvalidValue", Severity::Info, "canMerge".to_owned())],
            "{written}"
        );
    }
    let doc = parse(r#"{ "canMerge": false }"#);
    assert!(!doc.merges());
    assert_eq!(codes(doc.diagnostics()), []);
    assert!(parse(r#"{ "canMerge": true }"#).merges());
}

#[test]
fn sp04_three_packs_the_middle_one_replaces_and_the_top_one_adds() {
    let stack = stack_of(&[
        &[("splashes.json", r#"{ "splashes": ["a"] }"#)],
        &[(
            "splashes.json",
            r#"{ "canMerge": false, "splashes": ["b"] }"#,
        )],
        &[(
            "splashes.json",
            r#"{ "canMerge": true, "splashes": ["c"] }"#,
        )],
    ]);
    assert_eq!(pool(&stack), ["b", "c"]);
    let found: Vec<_> = stack.splashes().diagnostics().collect();
    assert_eq!(
        codes(found.iter().copied()),
        [("SplashesReplaced", Severity::Info)]
    );
    assert_eq!(pack_number(found[0], 3), Some(1));
    assert!(
        stack.splashes().origin(&"a".to_owned()).is_none(),
        "replaced"
    );
}

#[test]
fn sp05_an_empty_replacing_copy_leaves_no_splash() {
    let stack = stack_of(&[
        &[("splashes.json", BOTTOM)],
        &[("splashes.json", r#"{ "splashes": [] }"#)],
    ]);
    assert_eq!(pool(&stack), [] as [&str; 0]);
    assert_eq!(
        stack_codes(&stack),
        [
            ("SplashesReplaced", Severity::Info),
            ("NoSplashes", Severity::Info)
        ]
    );
}

#[test]
fn s9_a_stack_without_the_file_reports_nothing() {
    let stack = stack_of(&[&[("other.txt", "")]]);
    assert_eq!(pool(&stack), [] as [&str; 0]);
    assert_eq!(stack_codes(&stack), []);
}

#[test]
fn sp06_conditional_splashes_are_added_after_the_pool_is_emptied() {
    let stack = stack_of(&[
        &[("splashes.json", BOTTOM)],
        &[(
            "splashes.json",
            r#"{ "canMerge": false, "splashes": [],
                 "conditional": [{ "requires": {}, "splashes": ["x"] }] }"#,
        )],
    ]);
    assert_eq!(pool(&stack), ["x"]);
    assert_eq!(stack_codes(&stack), [("SplashesReplaced", Severity::Info)]);
}

#[test]
fn sp07_a_conditional_entry_without_requires_is_never_used() {
    let doc = parse(
        r#"{ "conditional": [{ "splashes": ["x"] }, 4, { "requires": [] , "splashes": ["y"] }] }"#,
    );
    let mut pool = Vec::new();
    doc.apply(&mut pool, &SplashClient::default());
    assert_eq!(pool, [] as [&str; 0]);
    assert_eq!(
        located(doc.diagnostics()),
        [
            ("InvalidValue", Severity::Info, "conditional/0".to_owned()),
            ("InvalidValue", Severity::Info, "conditional/1".to_owned()),
            ("InvalidValue", Severity::Info, "conditional/2".to_owned()),
        ]
    );
}

#[test]
fn s5_a_conditional_that_is_no_array_is_ignored() {
    let doc =
        parse(r#"{ "splashes": ["a"], "conditional": { "requires": {}, "splashes": ["x"] } }"#);
    let mut pool = Vec::new();
    doc.apply(&mut pool, &SplashClient::default());
    assert_eq!(pool, ["a"]);
    assert_eq!(
        located(doc.diagnostics()),
        [("InvalidValue", Severity::Info, "conditional".to_owned())]
    );
    assert_eq!(codes(parse(r#"{ "conditional": null }"#).diagnostics()), []);
}

#[test]
fn sp08_requirement_categories() {
    let doc = parse(
        r#"{ "conditional": [{ "requires": { "platforms": [], "stores": 5, "treatments": [1, "t"] },
                               "splashes": ["x"] }] }"#,
    );
    assert_eq!(codes(doc.diagnostics()), []);
    let entry = &doc.conditional_entries()[0];
    let requires = entry.requirements().expect("an object");
    assert!(
        !requires.passes(&SplashClient::default()),
        "the linter has no treatment"
    );
    let tagged = SplashClient {
        treatments: vec!["t".to_owned()],
        ..SplashClient::default()
    };
    assert!(requires.passes(&tagged));
    let other = SplashClient {
        treatments: vec!["u".to_owned()],
        platforms: vec!["t".to_owned()],
        stores: vec!["t".to_owned()],
        ..SplashClient::default()
    };
    assert!(!requires.passes(&other), "each category asks its own list");
    let mut pool = Vec::new();
    doc.apply(&mut pool, &tagged);
    assert_eq!(pool, ["x"]);
}

#[test]
fn s7_each_category_is_matched_against_its_own_list_and_only_strings_count() {
    for (member, client) in [
        (
            "platforms",
            SplashClient {
                platforms: vec!["p".to_owned()],
                ..SplashClient::default()
            },
        ),
        (
            "stores",
            SplashClient {
                stores: vec!["p".to_owned()],
                ..SplashClient::default()
            },
        ),
        (
            "treatments",
            SplashClient {
                treatments: vec!["p".to_owned()],
                ..SplashClient::default()
            },
        ),
    ] {
        let doc = parse(&format!(
            r#"{{ "conditional": [{{ "requires": {{ "{member}": ["q", "p"] }}, "splashes": ["x"] }}] }}"#
        ));
        let requires = doc.conditional_entries()[0].requirements().unwrap();
        assert!(requires.passes(&client), "{member}: any match");
        assert!(!requires.passes(&SplashClient::default()), "{member}");
        // S7: only non-strings: the category fails for every client.
        let doc = parse(&format!(
            r#"{{ "conditional": [{{ "requires": {{ "{member}": [1, true] }}, "splashes": ["x"] }}] }}"#
        ));
        let requires = doc.conditional_entries()[0].requirements().unwrap();
        assert!(!requires.passes(&client), "{member}");
        assert_eq!(
            located(doc.diagnostics()),
            [(
                "InvalidValue",
                Severity::Info,
                format!("conditional/0/requires/{member}")
            )],
        );
        // Null, a non-array and an empty array hold.
        for written in ["null", "\"p\"", "[]", "{}"] {
            let doc = parse(&format!(
                r#"{{ "conditional": [{{ "requires": {{ "{member}": {written} }}, "splashes": ["x"] }}] }}"#
            ));
            let requires = doc.conditional_entries()[0].requirements().unwrap();
            assert!(
                requires.passes(&SplashClient::default()),
                "{member}: {written}"
            );
        }
    }
}

#[test]
fn sp09_a_trial_collects_only_conditional_splashes_and_never_empties() {
    let stack = stack_of(&[
        &[("splashes.json", r#"{ "splashes": ["a"] }"#)],
        &[(
            "splashes.json",
            r#"{ "canMerge": false, "splashes": ["b"], "conditional": [
                 { "requires": { "trialMode": true }, "splashes": ["t"] },
                 { "requires": { "trialMode": false }, "splashes": ["n"] } ] }"#,
        )],
    ]);
    assert_eq!(pool(&stack), ["b", "n"]);
    let trial = SplashClient {
        trial: true,
        ..SplashClient::default()
    };
    assert_eq!(splash_pool(stack.splashes().layers(), &trial), ["t"]);
}

#[test]
fn sp09_a_trial_keeps_the_lower_packs_conditional_splashes() {
    let stack = stack_of(&[
        &[(
            "splashes.json",
            r#"{ "splashes": ["a"], "conditional": [{ "requires": {}, "splashes": ["low"] }] }"#,
        )],
        &[(
            "splashes.json",
            r#"{ "splashes": ["b"], "conditional": [{ "requires": { "trialMode": 1 }, "splashes": ["any"] }] }"#,
        )],
    ]);
    assert_eq!(pool(&stack), ["b", "any"], "a non-boolean trialMode holds");
    let trial = SplashClient {
        trial: true,
        ..SplashClient::default()
    };
    assert_eq!(
        splash_pool(stack.splashes().layers(), &trial),
        ["low", "any"]
    );
}

#[test]
fn sp10_a_copy_that_is_not_json_is_skipped_deviation() {
    let stack = stack_of(&[
        &[("splashes.json", r#"{ "splashes": ["a"] }"#)],
        &[("splashes.json", "{ nope")],
        &[(
            "splashes.json",
            r#"{ "canMerge": true, "splashes": ["c"] }"#,
        )],
    ]);
    assert_eq!(pool(&stack), ["a", "c"]);
    let found: Vec<_> = stack.splashes().diagnostics().collect();
    assert_eq!(
        codes(found.iter().copied()),
        // A refused file is an error.
        [("MalformedInput", Severity::Error)]
    );
    assert_eq!(pack_number(found[0], 3), Some(1));
    assert_eq!(stack.splashes().layers().len(), 2);
}

#[test]
fn sp10_a_broken_top_copy_does_not_replace() {
    let stack = stack_of(&[
        &[("splashes.json", r#"{ "splashes": ["a"] }"#)],
        &[("splashes.json", "[")],
    ]);
    assert_eq!(pool(&stack), ["a"]);
    assert_eq!(stack_codes(&stack), [("MalformedInput", Severity::Error)]);
}

#[test]
fn sp11_non_strings_are_converted() {
    let doc = parse(r#"{ "splashes": ["a", 3, true, null, {}] }"#);
    assert_eq!(doc.texts(), ["a", "", "true", "", ""]);
    assert_eq!(
        located(doc.diagnostics()),
        [
            ("InvalidValue", Severity::Info, "splashes/1".to_owned()),
            ("InvalidValue", Severity::Info, "splashes/2".to_owned()),
            ("InvalidValue", Severity::Info, "splashes/3".to_owned()),
            ("InvalidValue", Severity::Info, "splashes/4".to_owned()),
        ]
    );
    // A conditional entry's splashes are read the same way.
    let doc = parse(r#"{ "conditional": [{ "requires": {}, "splashes": [false, "x"] }] }"#);
    assert_eq!(doc.conditional_entries()[0].texts(), ["false", "x"]);
    assert_eq!(
        located(doc.diagnostics()),
        [(
            "InvalidValue",
            Severity::Info,
            "conditional/0/splashes/0".to_owned()
        )]
    );
}

#[test]
fn a_splashes_member_that_is_no_array_is_iterated_as_the_game_does() {
    let doc = parse(r#"{ "splashes": { "b": "second", "a": "first" } }"#);
    assert_eq!(
        doc.texts(),
        ["first", "second"],
        "an object's values, by key"
    );
    assert_eq!(
        located(doc.diagnostics()),
        [("InvalidType", Severity::Info, "splashes".to_owned())]
    );
    let doc = parse(r#"{ "splashes": "one" }"#);
    assert_eq!(doc.texts(), [] as [&str; 0], "a scalar gives nothing");
    assert_eq!(codes(doc.diagnostics()), [("InvalidType", Severity::Info)]);
}

#[test]
fn sp12_the_active_subpack_s_copy_stands_for_the_pack_s() {
    let set = subpack_stacks(&[
        ("splashes.json", br#"{ "splashes": ["root"] }"#),
        ("subpacks/hi/splashes.json", br#"{ "splashes": ["sub"] }"#),
    ]);
    assert_eq!(pool(&set.resource), ["sub"]);
    assert_eq!(
        set.resource.splashes().layers()[0].origin().path.as_str(),
        "subpacks/hi/splashes.json"
    );
}

#[test]
fn sp13_a_root_that_is_no_object_replaces_and_adds_nothing() {
    let stack = stack_of(&[
        &[("splashes.json", r#"{ "splashes": ["a"] }"#)],
        &[("splashes.json", r#"["b"]"#)],
    ]);
    assert_eq!(pool(&stack), [] as [&str; 0]);
    assert_eq!(
        stack_codes(&stack),
        [
            ("InvalidValue", Severity::Info),
            ("SplashesReplaced", Severity::Info),
            ("NoSplashes", Severity::Info),
        ]
    );
    for root in ["3", "\"a\"", "null", "true"] {
        let doc = parse(root);
        // An empty copy; the root is kept as written beside it.
        let kept = Splashes {
            written: Some(json(root)),
            ..Splashes::default()
        };
        assert_eq!(doc.value(), &kept, "{root}");
        assert_eq!(json(&doc.to_json().unwrap()), json(root));
        assert_eq!(
            codes(doc.diagnostics()),
            [("InvalidValue", Severity::Info)],
            "{root}"
        );
    }
}

#[test]
fn s8_unknown_members_are_kept_and_reported() {
    let text = r#"{ "note": 1, "splashes": ["a"], "conditional": [{ "requires": { "trialMode": false, "why": "x" }, "splashes": [], "tag": 2 }] }"#;
    let doc = parse(text);
    assert_eq!(
        located(doc.diagnostics()),
        [
            ("UnrecognizedField", Severity::Info, "note".to_owned()),
            (
                "UnrecognizedField",
                Severity::Info,
                "conditional/0/tag".to_owned()
            ),
            (
                "UnrecognizedField",
                Severity::Info,
                "conditional/0/requires/why".to_owned()
            ),
        ]
    );
    assert_eq!(doc.extra.get("note"), Some(&json("1")));
    assert_eq!(
        json(&doc.to_json().unwrap()),
        json(text),
        "every member is written back"
    );
}

#[test]
fn a_member_named_extra_is_an_unknown_member() {
    let doc = parse(r#"{ "extra": 5, "splashes": ["a"] }"#);
    assert_eq!(doc.extra.get("extra"), Some(&json("5")));
    assert_eq!(doc.texts(), ["a"]);
}

#[test]
fn a_member_written_twice_keeps_its_last_value() {
    let doc = parse(r#"{ "splashes": ["a"], "canMerge": true, "splashes": ["b"], "canMerge": 0 }"#);
    assert_eq!(doc.texts(), ["b"]);
    assert!(!doc.merges());
}

#[test]
fn splashes_are_placed_at_the_pack_root() {
    assert_eq!(Splashes::default().default_path().as_str(), "splashes.json");
}

#[test]
fn the_merged_value_writes_the_pool_and_nothing_the_packs_did_not_say() {
    let stack = stack_of(&[
        &[("splashes.json", BOTTOM)],
        &[(
            "splashes.json",
            r#"{ "canMerge": true, "splashes": ["c"] }"#,
        )],
    ]);
    assert_eq!(
        json(&stack.splashes().value().to_json().unwrap()),
        json(r#"{ "splashes": ["a", "b", "c"] }"#)
    );
    assert_eq!(Splashes::default().to_json().unwrap().trim(), "{}");
}

/// A few thousand generated copies: each loads, writes text that loads back to an equal
/// value with the same findings, and takes the same step on a pool.
#[test]
fn generated_copies_round_trip() {
    let scalars = [
        "true",
        "false",
        "null",
        "1",
        "1.5",
        "\"true\"",
        "\"x\"",
        "[]",
        "{}",
        "[\"a\", 2]",
        "{\"k\": \"v\"}",
    ];
    let mut count = 0;
    for can_merge in [
        None,
        Some("true"),
        Some("false"),
        Some("\"true\""),
        Some("null"),
    ] {
        for splashes in std::iter::once(None).chain(scalars.iter().map(Some)) {
            for conditional in std::iter::once(None).chain(scalars.iter().map(Some)) {
                for requires in [
                    None,
                    Some("{}"),
                    Some("{\"trialMode\": true, \"platforms\": [\"p\", 1]}"),
                    Some("3"),
                ] {
                    let mut members = Vec::new();
                    if let Some(v) = can_merge {
                        members.push(format!("\"canMerge\": {v}"));
                    }
                    if let Some(v) = splashes {
                        members.push(format!("\"splashes\": {v}"));
                    }
                    let entry = match requires {
                        Some(r) => format!(
                            "{{\"requires\": {r}, \"splashes\": {}, \"zz\": 1}}",
                            splashes.unwrap_or(&"[\"s\"]")
                        ),
                        None => "{\"splashes\": [\"s\"]}".to_owned(),
                    };
                    match conditional {
                        Some(v) if *v == "[]" => {
                            members.push(format!("\"conditional\": [{entry}]"))
                        }
                        Some(v) => members.push(format!("\"conditional\": {v}")),
                        None => {}
                    }
                    let text = format!("{{{}}}", members.join(", "));
                    let first = parse(&text);
                    let written = first.to_json().unwrap();
                    // As authored: no generated document is rewritten.
                    assert_eq!(json(&written), json(&text), "{text}");
                    let second = parse(&written);
                    assert_eq!(first.value(), second.value(), "{text}");
                    assert_eq!(
                        located(first.diagnostics()),
                        located(second.diagnostics()),
                        "{text}"
                    );
                    assert_eq!(second.to_json().unwrap(), written, "{text}");
                    for trial in [false, true] {
                        let client = SplashClient {
                            trial,
                            platforms: vec!["p".to_owned()],
                            ..SplashClient::default()
                        };
                        let (mut a, mut b) = (vec!["low".to_owned()], vec!["low".to_owned()]);
                        first.apply(&mut a, &client);
                        second.apply(&mut b, &client);
                        assert_eq!(a, b, "{text}");
                    }
                    count += 1;
                }
            }
        }
    }
    assert!(count >= 2000, "{count} documents");
    println!("{count} generated documents written as authored");
}
