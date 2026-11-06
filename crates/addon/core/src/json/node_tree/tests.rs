//! The walker's own tests: the patterns, each rule of [`Rules`] as the loader that has it reads,
//! and the leaves. The rule sets are the loaders' (render controllers and fog below 1.26.20,
//! geometry in its two modes, both animation kinds).

use super::*;

/// Render controllers and fog below 1.26.20.
const SORTED: Rules = Rules {
    names: Names::Exact,
    order: Order::Sorted,
    validation: Validation::Strict,
    failure: Failure::MemberFailed,
    lossy: Lossy::Told,
};

/// Both animation kinds.
const ANIMATION: Rules = Rules {
    names: Names::IgnoreAsciiCase,
    order: Order::Document,
    validation: Validation::Strict,
    failure: Failure::MemberFailed,
    lossy: Lossy::Silent,
};

/// Geometry from 1.19.30.
const GEOMETRY: Rules = Rules {
    names: Names::Exact,
    order: Order::Document,
    validation: Validation::Strict,
    failure: Failure::ErrorRaised,
    lossy: Lossy::Told,
};

/// Geometry below 1.19.30.
const LENIENT: Rules = Rules {
    validation: Validation::Lenient {
        removed: "child '{}' removed",
    },
    ..GEOMETRY
};

/// A walk at the root.
fn walk(rules: Rules) -> Walk {
    Walk::new(rules, &JsonPath::new())
}

/// The findings of a walk, one line each: severity, text and path.
fn told(walk: &Walk) -> Vec<String> {
    walk.diagnostics
        .iter()
        .map(|d| format!("{:?} {} at {}", d.severity, d.code, d.location.json_path))
        .collect()
}

/// The keys `object` hands its callback, in the order it hands them, whether the object read,
/// and the findings.
fn walked(
    rules: Rules,
    text: &str,
    groups: &[Group],
    read: Hit,
) -> (bool, Vec<String>, Vec<String>) {
    let mut walk = walk(rules);
    let mut keys = Vec::new();
    let ok = walk.object(Node::root(text).unwrap(), groups, |_, _, key, _| {
        keys.push(key.to_owned());
        read
    });
    (ok, keys, told(&walk))
}

// ---- patterns ----

#[test]
fn data_types_are_comparable_and_hashable() {
    fn assert_hashable<T: Eq + std::hash::Hash>() {}
    assert_hashable::<Pattern>();
    assert_hashable::<Group>();
    assert_hashable::<Kind>();
    assert_hashable::<Rules>();
    assert_eq!(Pattern::IDENTIFIER, Pattern::IDENTIFIER);
}

#[test]
fn prefixed_names_match_in_full_and_as_written() {
    let p = Pattern::Prefixed {
        prefix: "controller.render.",
        extra: b"_.:-",
        regex: r"controller\.render\.[a-zA-Z0-9_.:-]+",
    };
    let exact = |key| p.matches(key, Names::Exact);
    assert!(exact("controller.render.a:b-c_d.E9"));
    assert!(!exact("controller.render."));
    assert!(!exact("Controller.render.a"));
    assert!(!exact("controller.render.a b"));
    assert!(!exact("controller.render.a*"));
}

#[test]
fn prefixed_names_ignore_case_and_match_in_full() {
    let p = Pattern::Prefixed {
        prefix: "animation.",
        extra: b"_.-",
        regex: r"animation\.[a-zA-Z0-9_.-]+",
    };
    let any_case = |key| p.matches(key, Names::IgnoreAsciiCase);
    assert!(any_case("Animation.Foo"));
    assert!(!any_case("animation."));
    assert!(!any_case("animation:foo"));
    assert!(!any_case("animation.a b"));
}

#[test]
fn bone_patterns_take_a_star_and_no_hyphen() {
    let bone = Pattern::Charset {
        extra: b"_.:*",
        regex: "[a-zA-Z0-9_.:*]+",
    };
    assert!(bone.matches("arm*_L.0:x", Names::Exact));
    assert!(!bone.matches("left-arm", Names::Exact));
    assert!(!bone.matches("", Names::Exact));
    assert!(!bone.matches("a b", Names::Exact));
}

#[test]
fn a_locator_name_matches_in_full() {
    assert!(Pattern::IDENTIFIER.matches("lead.hold-1_a", Names::Exact));
    assert!(!Pattern::IDENTIFIER.matches("a b", Names::Exact));
    assert!(!Pattern::IDENTIFIER.matches("", Names::Exact));
    assert_eq!(Pattern::IDENTIFIER.text(), "[a-zA-Z0-9_.-]+");
}

#[test]
fn a_literal_name_compares_as_the_walk_says() {
    let name = Pattern::Name("loop");
    assert!(name.matches("loop", Names::Exact));
    assert!(!name.matches("Loop", Names::Exact));
    assert!(name.matches("Loop", Names::IgnoreAsciiCase));
    assert!(!name.matches("loops", Names::IgnoreAsciiCase));
}

#[test]
fn a_custom_pattern_is_its_own_test_in_either_case_mode() {
    let axis = Pattern::Custom {
        matches: |key| matches!(key, "x" | "y" | "z"),
        regex: "[xyz]",
    };
    for names in [Names::Exact, Names::IgnoreAsciiCase] {
        assert!(axis.matches("x", names));
        assert!(!axis.matches("X", names));
    }
    assert_eq!(axis.text(), "[xyz]");
}

#[test]
fn percent_f_prints_six_decimals_without_an_exponent() {
    assert_eq!(percent_f(-5.0), "-5.000000");
    assert_eq!(
        percent_f(f32::MAX),
        "340282346638528859811704183484516925440.000000"
    );
}

// ---- nodes ----

/// Also without the kind check: no text but an object reads as members, and none but an array
/// as elements (the animation fork read both without the check).
#[test]
fn only_an_object_has_members_and_only_an_array_elements() {
    let walk = walk(SORTED);
    for text in ["null", "[]", "[1]", "\"a\"", "1", "true", "{}"] {
        let node = Node::root(text).unwrap();
        let members: Result<IndexMap<String, RawJson<'_>>, _> = parse_at(node.slice);
        assert_eq!(members.is_ok(), text == "{}", "{text}");
        let elements: Result<Vec<RawJson<'_>>, _> = parse_at(node.slice);
        assert_eq!(elements.is_ok(), text.starts_with('['), "{text}");
        assert_eq!(walk.members(node).is_some(), text == "{}", "{text}");
        assert_eq!(
            walk.elements(node).is_some(),
            text.starts_with('['),
            "{text}"
        );
    }
}

// ---- W1: the order ----

#[test]
fn sorted_members_are_walked_in_the_order_of_their_keys_bytes() {
    let groups = [Group::name("b"), Group::name("a"), Group::name("B")];
    let (ok, keys, _) = walked(SORTED, r#"{"b": 1, "a": 2, "B": 3}"#, &groups, Hit::Ok);
    assert!(ok);
    assert_eq!(keys, ["B", "a", "b"]);
}

#[test]
fn document_members_are_walked_in_the_files_order() {
    let groups = [Group::name("a"), Group::name("b"), Group::name("c")];
    for rules in [ANIMATION, GEOMETRY] {
        let (ok, keys, _) = walked(rules, r#"{"b": 1, "a": 2, "c": 3}"#, &groups, Hit::Ok);
        assert!(ok);
        assert_eq!(keys, ["b", "a", "c"]);
    }
}

// ---- W2: the failure ----

#[test]
fn a_member_failed_without_a_finding_fails_the_object_and_skips_the_required_check() {
    let groups = [Group::name("a"), Group::required("b")];
    for rules in [SORTED, ANIMATION] {
        let (ok, keys, findings) = walked(rules, r#"{"a": 1}"#, &groups, Hit::Failed);
        assert!(!ok);
        assert_eq!(keys, ["a"]);
        assert!(findings.is_empty(), "{findings:?}");
    }
}

#[test]
fn under_error_raised_a_member_failed_without_a_finding_fails_nothing() {
    let (ok, keys, findings) = walked(GEOMETRY, r#"{"a": 1}"#, &[Group::name("a")], Hit::Failed);
    assert!(ok);
    assert_eq!(keys, ["a"]);
    assert!(findings.is_empty(), "{findings:?}");
    let groups = [Group::name("a"), Group::required("b")];
    let (ok, _, findings) = walked(GEOMETRY, r#"{"a": 1}"#, &groups, Hit::Failed);
    assert!(!ok);
    assert_eq!(findings, ["Error Required child b not found at "]);
}

#[test]
fn an_array_fails_by_the_same_rule() {
    let read = |rules| {
        let mut walk = walk(rules);
        let ok = walk.array(Node::root("[1, 2]").unwrap(), |_, index, _| {
            Hit::from(index == 0)
        });
        (ok, told(&walk))
    };
    assert_eq!(read(SORTED), (false, vec![]));
    assert_eq!(read(GEOMETRY), (true, vec![]));
}

#[test]
fn a_required_group_without_a_member_is_missing_when_nothing_failed() {
    let groups = [Group::name("a"), Group::required("b")];
    for rules in [SORTED, ANIMATION, GEOMETRY] {
        let (ok, _, findings) = walked(rules, r#"{"a": 1}"#, &groups, Hit::Ok);
        assert!(!ok);
        assert_eq!(findings, ["Error Required child b not found at "]);
    }
}

// ---- W3: the lossy float ----

#[test]
fn a_float_that_narrows_is_told_as_lossy_or_read_silently() {
    for (rules, findings) in [
        (SORTED, vec!["Info LossyConversion at x"]),
        (GEOMETRY, vec!["Info LossyConversion at x"]),
        (ANIMATION, vec![]),
    ] {
        let mut walk = walk(rules);
        walk.enter("x");
        assert_eq!(
            walk.float(Node::root("0.123456789012").unwrap()),
            Some(0.123_456_79)
        );
        let told: Vec<_> = walk
            .diagnostics
            .iter()
            .map(|d| {
                format!(
                    "{:?} {} at {}",
                    d.severity,
                    d.code.name(),
                    d.location.json_path
                )
            })
            .collect();
        assert_eq!(told, findings);
    }
}

#[test]
fn a_number_beyond_a_float_is_the_largest_float_with_an_information() {
    for rules in [SORTED, ANIMATION, GEOMETRY] {
        let mut walk = walk(rules);
        assert_eq!(walk.float(Node::root("-1e39").unwrap()), Some(-f32::MAX));
        let told: Vec<_> = walk
            .diagnostics
            .iter()
            .map(|d| format!("{:?} {}", d.severity, d.code.name()))
            .collect();
        assert_eq!(told, ["Info InvalidValue"]);
    }
}

// ---- the validation ----

#[test]
fn an_undeclared_member_is_an_error_or_a_removal() {
    let groups = [Group::name("a")];
    let (ok, keys, findings) = walked(GEOMETRY, r#"{"z": 1, "a": 2}"#, &groups, Hit::Ok);
    assert!(!ok);
    assert_eq!(keys, ["a"]);
    assert_eq!(findings, ["Error child 'z' not valid here. at z"]);
    let (ok, keys, findings) = walked(LENIENT, r#"{"z": 1, "a": 2}"#, &groups, Hit::Ok);
    assert!(ok);
    assert_eq!(keys, ["a"]);
    assert_eq!(findings, ["Info child 'z' removed at z"]);
}

#[test]
fn a_wrong_type_is_an_error_or_a_removal() {
    for (rules, severity) in [(GEOMETRY, "Error"), (LENIENT, "Info")] {
        let mut walk = walk(rules);
        assert_eq!(walk.string(Node::root("1").unwrap()), None);
        assert_eq!(
            told(&walk),
            [format!(
                "{severity} unknown child schema option type.  Allowed types: string at "
            )]
        );
    }
}

#[test]
fn no_option_is_an_undeclared_member() {
    let (ok, _, findings) = walked(ANIMATION, r#"{"a": 1}"#, &[Group::name("a")], Hit::NoOption);
    assert!(!ok);
    assert_eq!(findings, ["Error child 'a' not valid here. at a"]);
}

// ---- the names ----

#[test]
fn members_one_name_in_two_cases_tell_the_earlier_as_dropped() {
    let groups = [Group::name("loop"), Group::name("other")];
    let mut walk = walk(ANIMATION);
    let ok = walk.object(
        Node::root(r#"{"loop": true, "LOOP": false, "Other": 1, "OTHER": 1}"#).unwrap(),
        &groups,
        |_, _, _, _| Hit::Ok,
    );
    assert!(ok);
    // The values of `Other` and `OTHER` are one value: nothing is dropped.
    let dropped: Vec<_> = walk
        .diagnostics
        .iter()
        .map(|d| {
            format!(
                "{:?} {} at {}",
                d.severity,
                d.code.name(),
                d.location.json_path
            )
        })
        .collect();
    assert_eq!(dropped, ["Info DroppedField at loop"]);
    let spelled: Vec<_> = walk
        .non_canonical
        .iter()
        .map(|(at, _)| at.to_string())
        .collect();
    assert_eq!(spelled, ["LOOP", "Other", "OTHER"]);
}

#[test]
fn exact_names_neither_note_nor_drop() {
    let groups = [Group::name("loop")];
    let (ok, keys, findings) = walked(SORTED, r#"{"loop": 1, "LOOP": 2}"#, &groups, Hit::Ok);
    assert!(!ok);
    assert_eq!(keys, ["loop"]);
    assert_eq!(findings, ["Error child 'LOOP' not valid here. at LOOP"]);
}

// ---- the leaves ----

/// What `molang3` says of an array, by its length and its elements: the findings as
/// `code at path`.
fn three(text: &str) -> (Option<[Molang; 3]>, Vec<String>) {
    let mut walk = walk(ANIMATION);
    walk.enter("rotation");
    let read = walk.molang3(Node::root(text).unwrap());
    let findings = walk
        .diagnostics
        .iter()
        .map(|d| format!("{} at {}", d.code.name(), d.location.json_path))
        .collect();
    (read, findings)
}

#[test]
fn an_array_of_three_expressions_is_told_of_by_its_length_then_by_its_elements() {
    let (read, findings) = three(r#"[1, "q.x", true]"#);
    assert!(read.is_some() && findings.is_empty(), "{findings:?}");
    // The length first: no element of an array of another length is read.
    for text in ["[]", "[1, {}]", "[1, {}, [], 4]", "7"] {
        let (read, findings) = three(text);
        let code = if text == "7" {
            "InvalidType"
        } else {
            "InvalidValue"
        };
        assert!(read.is_none(), "{text}");
        assert_eq!(findings, [format!("{code} at rotation")], "{text}");
    }
    // Then every element that is no expression, each at its index.
    let (read, findings) = three("[{}, 2, []]");
    assert!(read.is_none());
    assert_eq!(
        findings,
        ["InvalidType at rotation/0", "InvalidType at rotation/2"]
    );
}

/// An array of three expressions is read once, for its length and for its elements, not
/// once for each. The test compares two times rather than bounding growth: `molang3` on an
/// array of three large elements (objects, each refused without being read) against one read of
/// the same array's elements (`kinds`), each the best of seven taken in turn, required to stay
/// under one and a half times it (2.0 with the two reads, 1.0 with the one), and measured once
/// more before a time over the bound is believed.
#[test]
fn an_array_of_three_expressions_is_read_once() {
    use crate::test_cost::{alone_for, time_of};
    let element = format!(r#"{{"k": [{}0]}}"#, "1, ".repeat(40_000));
    let text = format!("[{element}, {element}, {element}]");
    let node = Node::root(&text).unwrap();
    let mut ratios = Vec::new();
    for _attempt in 0..2 {
        let (three, once) = alone_for(|| {
            let mut best = (std::time::Duration::MAX, std::time::Duration::MAX);
            for _ in 0..7 {
                let mut walk = walk(ANIMATION);
                best.0 = best
                    .0
                    .min(time_of(|| assert!(walk.molang3(node).is_none())));
                best.1 = best
                    .1
                    .min(time_of(|| assert_eq!(walk.kinds(node).len(), 3)));
            }
            best
        });
        let ratio = three.as_secs_f64() / once.as_secs_f64().max(1e-9);
        eprintln!(
            "cost: three expressions {three:?}, one read of the array {once:?}, ratio {ratio:.2}"
        );
        if ratio < 1.5 {
            return;
        }
        ratios.push(ratio);
    }
    panic!("an array of three expressions took {ratios:?} times one read of it, twice");
}

#[test]
fn a_child_count_is_exact() {
    let mut walk = walk(ANIMATION);
    assert!(walk.child_count(Node::root(r#"{"a": 1}"#).unwrap(), 1));
    assert!(walk.child_count(Node::root("[]").unwrap(), 1));
    assert!(!walk.child_count(Node::root("{}").unwrap(), 1));
    assert!(!walk.child_count(Node::root(r#"{"a": 1, "b": 2}"#).unwrap(), 1));
    assert_eq!(
        told(&walk),
        [
            "Error Node has too few children (0 < 1) at ",
            "Error Node has too many children (2 > 1) at "
        ]
    );
}

#[test]
fn a_marked_member_is_kept_with_its_path() {
    let mut walk = Walk::new(SORTED, &JsonPath::of_keys(["root"]));
    walk.enter("a");
    walk.mark(Span { offset: 3, len: 2 });
    walk.leave();
    assert_eq!(walk.marked.len(), 1);
    assert_eq!(walk.marked[0].0.to_string(), "root/a");
}

#[test]
fn a_list_and_a_float_map_read_what_their_walk_takes() {
    let mut walk = walk(ANIMATION);
    assert_eq!(
        walk.list(Node::root(r#"["a", "b"]"#).unwrap(), Walk::string),
        Some(vec!["a".to_owned(), "b".to_owned()])
    );
    assert_eq!(walk.strings(Node::root("[1]").unwrap()), None);
    let map = walk
        .float_map(
            Node::root(r#"{"x": 1, "y": 2.5}"#).unwrap(),
            Pattern::IDENTIFIER,
            true,
        )
        .unwrap();
    assert_eq!(
        map.into_iter().collect::<Vec<_>>(),
        [("x".to_owned(), 1.0), ("y".to_owned(), 2.5)]
    );
}
