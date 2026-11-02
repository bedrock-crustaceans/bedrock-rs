//! Rows p01 to p09: both structs are strict, every failure of the
//! payload is reported with vanilla's breadcrumb and text, and the file is skipped.

use super::common::{
    KEY, base_game, error, errors, failed_to_parse, fixture, floor, latest, loads, rejects,
    rejects_in,
};

/// The path of `member` below the payload.
fn at(member: &str) -> String {
    format!("{KEY}/{member}")
}

const NOT_IN_SCHEMA: &str = "this member was found in the input, but is not present in the Schema";

#[test]
fn p01_a_condition_at_1_20_60_is_an_unknown_member() {
    // Vanilla `BlockCullingR20U6WithConditionJson_…`.
    assert_eq!(
        errors(&rejects(&fixture("p01_1_20_60_with_condition.json"))),
        [
            error(
                "UnrecognizedField",
                &format!("-> rules[0] -> condition: {NOT_IN_SCHEMA}"),
                &at("rules/0/condition")
            ),
            failed_to_parse()
        ]
    );
}

#[test]
fn p02_the_flag_at_1_20_60_is_an_unknown_member() {
    // Vanilla `BlockCullingR20U6WithFallbackJson_…`.
    assert_eq!(
        errors(&rejects(&fixture("p02_1_20_60_with_flag.json"))),
        [
            error(
                "UnrecognizedField",
                &format!("-> rules[0] -> cull_against_full_and_opaque: {NOT_IN_SCHEMA}"),
                &at("rules/0/cull_against_full_and_opaque")
            ),
            failed_to_parse()
        ]
    );
}

#[test]
fn the_1_21_80_members_are_unknown_one_patch_below_and_known_at_1_21_80() {
    let rule = r#"{"condition": "same_block", "cull_against_full_and_opaque": true, "geometry_part": {"bone": "b"}, "direction": "up"}"#;
    let below = super::common::document("1.21.79", "a:b", &[rule]);
    let found = errors(&rejects(&below));
    let codes: Vec<&str> = found.iter().map(|f| f.0).collect();
    assert_eq!(
        codes,
        ["UnrecognizedField", "UnrecognizedField", "DocumentRejected"]
    );
    loads(&super::common::document("1.21.80", "a:b", &[rule]));
}

#[test]
fn p03_a_condition_outside_the_three_names() {
    // Vanilla `BlockCullingR21U8WithBadConditionJson_…` asserts the text up to "bound to".
    assert_eq!(
        errors(&rejects(&fixture("p03_bad_condition.json"))),
        [
            error(
                "InvalidEnumValue",
                "-> rules[0] -> condition: This value is bound to 'Rule Condition' and expects one of the following strings: 'same_culling_layer', 'same_block', 'same_block_permutation'. Found: 'invalid_condition'",
                &at("rules/0/condition")
            ),
            failed_to_parse()
        ]
    );
    // `NotDefined` (0) has no JSON name.
    let not_defined = latest(&[
        r#"{"condition": "not_defined", "geometry_part": {"bone": "b"}, "direction": "up"}"#,
    ]);
    assert_eq!(errors(&rejects(&not_defined))[0].0, "InvalidEnumValue");
}

#[test]
fn p04_a_flag_that_is_not_a_boolean() {
    // Vanilla `BlockCullingR21U8WithInvalidFallbackValue_…` (`"test_invalid"`).
    assert_eq!(
        errors(&rejects(&fixture("p04_bad_flag_value.json"))),
        [
            error(
                "InvalidType",
                "-> rules[0] -> cull_against_full_and_opaque: invalid numeric value",
                &at("rules/0/cull_against_full_and_opaque")
            ),
            failed_to_parse()
        ]
    );
    for value in ["1", "0", "null", "[true]", r#""true""#] {
        let rule = format!(
            r#"{{"cull_against_full_and_opaque": {value}, "geometry_part": {{"bone": "b"}}, "direction": "up"}}"#
        );
        let found = errors(&rejects(&latest(&[&rule])));
        assert_eq!(found[0].0, "InvalidType", "{value}");
    }
    for value in ["true", "false"] {
        let rule = format!(
            r#"{{"cull_against_full_and_opaque": {value}, "geometry_part": {{"bone": "b"}}, "direction": "up"}}"#
        );
        let document = loads(&latest(&[&rule]));
        assert_eq!(
            document.rules[0].cull_against_full_and_opaque,
            Some(value == "true")
        );
    }
}

#[test]
fn p05_a_rule_without_direction() {
    // Vanilla `BlockCullingMissingFieldsInJson_…`.
    assert_eq!(
        errors(&rejects(&fixture("p05_missing_direction.json"))),
        [
            error(
                "MissingRequiredField",
                "-> rules[0] -> direction: missing required field",
                &at("rules/0/direction")
            ),
            failed_to_parse()
        ]
    );
}

#[test]
fn p06_unknown_members_and_both_missing_ones_are_all_reported() {
    // Vanilla `BlockCullingInvalidFieldsInJson_…` asserts the two missing-required texts; the
    // unknown-member texts beside them are the folder's inference.
    assert_eq!(
        errors(&rejects(&fixture("p06_unknown_fields_only.json"))),
        [
            error(
                "UnrecognizedField",
                &format!("-> rules[0] -> unrecognized_field1: {NOT_IN_SCHEMA}"),
                &at("rules/0/unrecognized_field1")
            ),
            error(
                "UnrecognizedField",
                &format!("-> rules[0] -> unrecognized_field2: {NOT_IN_SCHEMA}"),
                &at("rules/0/unrecognized_field2")
            ),
            error(
                "MissingRequiredField",
                "-> rules[0] -> geometry_part: missing required field",
                &at("rules/0/geometry_part")
            ),
            error(
                "MissingRequiredField",
                "-> rules[0] -> direction: missing required field",
                &at("rules/0/direction")
            ),
            failed_to_parse()
        ]
    );
}

#[test]
fn every_required_member_is_required() {
    let cases = [
        (
            r#"{"format_version": "1.21.80", "minecraft:block_culling_rules": {"rules": []}}"#,
            "-> description: missing required field",
        ),
        (
            r#"{"format_version": "1.21.80", "minecraft:block_culling_rules": {"description": {}, "rules": []}}"#,
            "-> description -> identifier: missing required field",
        ),
        (
            r#"{"format_version": "1.21.80", "minecraft:block_culling_rules": {"description": {"identifier": "a:b"}}}"#,
            "-> rules: missing required field",
        ),
        (
            r#"{"format_version": "1.21.80", "minecraft:block_culling_rules": {"description": {"identifier": "a:b"}, "rules": [{"geometry_part": {}, "direction": "up"}]}}"#,
            "-> rules[0] -> geometry_part -> bone: missing required field",
        ),
    ];
    for (json, text) in cases {
        let found = errors(&rejects(json));
        assert_eq!(found.len(), 2, "{json}: {found:#?}");
        assert_eq!(
            (found[0].0, found[0].1.as_str()),
            ("MissingRequiredField", text)
        );
        assert_eq!(found[1], failed_to_parse());
    }
    // The second rule's index is in the breadcrumb.
    let second = latest(&[
        r#"{"geometry_part": {"bone": "a"}, "direction": "up"}"#,
        r#"{"geometry_part": {"bone": "b"}}"#,
    ]);
    assert_eq!(
        errors(&rejects(&second))[0],
        error(
            "MissingRequiredField",
            "-> rules[1] -> direction: missing required field",
            &at("rules/1/direction")
        )
    );
}

#[test]
fn p07_a_cube_below_zero() {
    let rule = |cube: &str| {
        format!(
            r#"{{"geometry_part": {{"bone": "b", "cube": {cube}, "face": "up"}}, "direction": "up"}}"#
        )
    };
    assert_eq!(
        errors(&rejects(&latest(&[&rule("-1")]))),
        [
            error(
                "InvalidValue",
                "-> rules[0] -> geometry_part -> cube: the input must be a number >= 0",
                &at("rules/0/geometry_part/cube")
            ),
            failed_to_parse()
        ]
    );
    // The minimum is inclusive and there is no maximum.
    assert_eq!(
        loads(&latest(&[&rule("0")])).rules[0].geometry_part.cube,
        Some(0)
    );
    assert_eq!(
        loads(&latest(&[&rule("2147483647")])).rules[0]
            .geometry_part
            .cube,
        Some(i32::MAX)
    );
    assert_eq!(
        errors(&rejects(&latest(&[&rule("-0.5")])))[0].0,
        "InvalidValue"
    );
    // Ours: a fraction, an index beyond `int` or beyond any float is no cube index.
    for cube in ["1.5", "2147483648", "1e39", "1e999"] {
        let found = errors(&rejects(&latest(&[&rule(cube)])));
        assert_eq!(found[0].0, "InvalidValue", "{cube}");
        assert_eq!(found[0].2, at("rules/0/geometry_part/cube"), "{cube}");
    }
    for cube in [r#""0""#, "true", "null", "[0]"] {
        let found = errors(&rejects(&latest(&[&rule(cube)])));
        assert_eq!(
            (found[0].0, found[0].1.as_str()),
            (
                "InvalidType",
                "-> rules[0] -> geometry_part -> cube: invalid numeric value"
            ),
            "{cube}"
        );
    }
}

#[test]
fn p08_a_direction_or_face_outside_the_six_names() {
    // At most three names, then the count of the others (validation-messages E2).
    let facing = "This value is bound to 'Facing' and expects one of the following strings: 'down', 'up', 'north', ... (3 others). Found:";
    let bad_direction = latest(&[r#"{"geometry_part": {"bone": "b"}, "direction": "northeast"}"#]);
    assert_eq!(
        errors(&rejects(&bad_direction)),
        [
            error(
                "InvalidEnumValue",
                &format!("-> rules[0] -> direction: {facing} 'northeast'"),
                &at("rules/0/direction")
            ),
            failed_to_parse()
        ]
    );
    let bad_face = latest(&[
        r#"{"geometry_part": {"bone": "b", "cube": 0, "face": "North"}, "direction": "up"}"#,
    ]);
    assert_eq!(
        errors(&rejects(&bad_face))[0],
        error(
            "InvalidEnumValue",
            &format!("-> rules[0] -> geometry_part -> face: {facing} 'North'"),
            &at("rules/0/geometry_part/face")
        )
    );
    // An enum takes a string only.
    let number = latest(&[r#"{"geometry_part": {"bone": "b"}, "direction": 2}"#]);
    assert_eq!(
        errors(&rejects(&number))[0],
        error(
            "InvalidType",
            "-> rules[0] -> direction: This value is bound to 'Facing' and expects a \"named value\" (i.e. a string)",
            &at("rules/0/direction")
        )
    );
    let condition =
        latest(&[r#"{"condition": 1, "geometry_part": {"bone": "b"}, "direction": "up"}"#]);
    assert_eq!(
        errors(&rejects(&condition))[0].1,
        "-> rules[0] -> condition: This value is bound to 'Rule Condition' and expects a \"named value\" (i.e. a string)"
    );
}

#[test]
fn p09_an_unknown_member_at_every_level() {
    let unknown = |json: &str, crumb: &str, path: &str| {
        let found = errors(&rejects(json));
        assert_eq!(
            found,
            [
                error(
                    "UnrecognizedField",
                    &format!("{crumb}: {NOT_IN_SCHEMA}"),
                    &at(path)
                ),
                failed_to_parse()
            ],
            "{json}"
        );
    };
    unknown(
        r#"{"format_version": "1.21.80", "minecraft:block_culling_rules": {"description": {"identifier": "a:b", "foo": 1}, "rules": []}}"#,
        "-> description -> foo",
        "description/foo",
    );
    unknown(
        r#"{"format_version": "1.21.80", "minecraft:block_culling_rules": {"description": {"identifier": "a:b"}, "rules": [], "foo": 1}}"#,
        "-> foo",
        "foo",
    );
    unknown(
        &latest(&[r#"{"geometry_part": {"bone": "b", "foo": 1}, "direction": "up"}"#]),
        "-> rules[0] -> geometry_part -> foo",
        "rules/0/geometry_part/foo",
    );
    // In a base-game pack too: strictness is the slice's, not the pack's.
    let json = latest(&[r#"{"geometry_part": {"bone": "b"}, "direction": "up", "foo": 1}"#]);
    assert_eq!(
        errors(&rejects_in(&json, &base_game()))[0].0,
        "UnrecognizedField"
    );
}

#[test]
fn members_of_the_wrong_json_type_fail_the_file() {
    let cases = [
        r#"{"format_version": "1.21.80", "minecraft:block_culling_rules": {"description": [], "rules": []}}"#,
        r#"{"format_version": "1.21.80", "minecraft:block_culling_rules": {"description": {"identifier": 5}, "rules": []}}"#,
        r#"{"format_version": "1.21.80", "minecraft:block_culling_rules": {"description": {"identifier": "a:b"}, "rules": {}}}"#,
        r#"{"format_version": "1.21.80", "minecraft:block_culling_rules": {"description": {"identifier": "a:b"}, "rules": [5]}}"#,
        r#"{"format_version": "1.21.80", "minecraft:block_culling_rules": {"description": {"identifier": "a:b"}, "rules": [{"geometry_part": "b", "direction": "up"}]}}"#,
        r#"{"format_version": "1.21.80", "minecraft:block_culling_rules": {"description": {"identifier": "a:b"}, "rules": [{"geometry_part": {"bone": 1}, "direction": "up"}]}}"#,
        r#"{"format_version": "1.21.80", "minecraft:block_culling_rules": []}"#,
    ];
    for json in cases {
        let found = errors(&rejects(json));
        assert_eq!(found[0].0, "InvalidType", "{json}: {found:#?}");
        assert_eq!(found.last(), Some(&failed_to_parse()), "{json}");
    }
}

#[test]
fn a_repeated_member_is_read_as_its_last_value() {
    // the last value of a duplicate member is the member.
    let json = floor(&[
        r#"{"geometry_part": {"bone": "a", "bone": "b"}, "direction": "sideways", "direction": "up"}"#,
    ]);
    let document = loads(&json);
    assert_eq!(document.rules[0].geometry_part.bone, "b");
}

#[test]
fn a_cube_beyond_a_double_fails_the_file() {
    // `1e999` is no integer and no double: the file is refused at load, so no infinity is
    // ever held or written.
    // An integer of 310 digits is a number beyond every double as well.
    let beyond = format!("1{}", "0".repeat(309));
    for cube in ["1e999", "-1e999", "1e39", beyond.as_str()] {
        let d = rejects(&latest(&[&format!(
            r#"{{"geometry_part": {{"bone": "b", "cube": {cube}, "face": "up"}}, "direction": "up"}}"#
        )]));
        let found = errors(&d);
        assert!(
            found.iter().any(|(code, _, path)| *code == "InvalidValue"
                && *path == at("rules/0/geometry_part/cube")),
            "{cube}: {found:#?}"
        );
    }
}

#[test]
fn a_cube_beyond_64_bits_is_a_number_out_of_range() {
    // An integer no 64-bit type holds is still a number token: it is refused for its value,
    // as every index beyond `int` is, not for its type as a string would be.
    for cube in ["18446744073709551616", "123456789012345678901234567890"] {
        let d = rejects(&latest(&[&format!(
            r#"{{"geometry_part": {{"bone": "b", "cube": {cube}, "face": "up"}}, "direction": "up"}}"#
        )]));
        let found: Vec<_> = errors(&d)
            .into_iter()
            .filter(|(_, _, path)| *path == at("rules/0/geometry_part/cube"))
            .map(|(code, _, _)| code)
            .collect();
        assert_eq!(found, ["InvalidValue"], "{cube}");
    }
    // Below zero it is vanilla's minimum that refuses it.
    let d = rejects(&latest(&[
        r#"{"geometry_part": {"bone": "b", "cube": -18446744073709551616, "face": "up"}, "direction": "up"}"#,
    ]));
    let found: Vec<_> = errors(&d)
        .into_iter()
        .filter(|(_, _, path)| *path == at("rules/0/geometry_part/cube"))
        .collect();
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].0, "InvalidValue");
    assert!(
        found[0].1.contains("rules[0] -> geometry_part -> cube"),
        "{found:#?}"
    );
}

/// The reader works on a parsed value, and each of its findings gets the span of
/// the value its path names from the text.
#[test]
fn findings_carry_the_span_of_the_value_their_path_names() {
    let good = r#"{"geometry_part": {"bone": "b"}, "direction": "up"}"#;
    let text = super::common::document(
        "1.21.80",
        "a:b",
        &[
            good,
            r#"{"geometry_part": {"bone": 5}, "direction": "sideways", "more": [1]}"#,
            good,
            // A member written twice: the last, the one the reader takes.
            r#"{"geometry_part": {"bone": "b"}, "direction": "up", "direction": 7}"#,
        ],
    )
    .replace(
        r#""description": {"#,
        r#""stray": {"x": 1}, "description": {"#,
    );
    let found = rejects(&text);
    let mut seen = Vec::new();
    for finding in &found {
        let at = finding.location.json_path.to_string();
        let Some(span) = finding.location.span else {
            // The line that says the file failed is about the file, not about a value.
            assert_eq!(finding.code.name(), "DocumentRejected", "no span at {at}");
            continue;
        };
        assert!(finding.location.path.is_some(), "{at}");
        let covered = &text[span.offset as usize..(span.offset + span.len) as usize];
        seen.push((at.strip_prefix(KEY).unwrap().to_owned(), covered.to_owned()));
    }
    seen.sort();
    assert_eq!(
        seen,
        [
            ("/rules/1/direction".to_owned(), r#""sideways""#.to_owned()),
            ("/rules/1/geometry_part/bone".to_owned(), "5".to_owned()),
            ("/rules/1/more".to_owned(), "[1]".to_owned()),
            ("/rules/3/direction".to_owned(), "7".to_owned()),
            ("/stray".to_owned(), r#"{"x": 1}"#.to_owned()),
        ],
        "{found:#?}"
    );
}
