//! Rows r01 to r14: the identifier, then the rule checks. Any of
//! them skips the file with "… into internal format".

use bedrock_addon_core::diagnostic::Kind;

use super::common::{
    BONE1_EAST, KEY, base_game, document, error, errors, failed_internal, latest, load_in, loads,
    rejects, rejects_in, user_pack,
};

/// `(code, text)` of every finding.
fn texts(json: &str) -> Vec<(&'static str, String)> {
    errors(&rejects(json))
        .into_iter()
        .map(|(code, text, _)| (code, text))
        .collect()
}

fn text(code: &'static str, text: &str) -> (&'static str, String) {
    (code, text.to_owned())
}

fn internal() -> (&'static str, String) {
    let (code, text, _) = failed_internal();
    (code, text)
}

/// A 1.21.80 document named `identifier` with one sound rule.
fn named(identifier: &str) -> String {
    document("1.21.80", identifier, &[BONE1_EAST])
}

#[test]
fn r01_a_cube_without_a_face() {
    // Vanilla `BlockCullingPuVDataWithInvalidGeoPartNoFace_Parse_ExpectFail`.
    let json = latest(&[
        BONE1_EAST,
        r#"{"geometry_part": {"bone": "bone2", "cube": 1}, "direction": "south"}"#,
    ]);
    assert_eq!(
        errors(&rejects(&json)),
        [
            error(
                "block_culling/cube_without_face",
                "Rule 1 with geometry part 'bone2' had a cube index but no face specified",
                &format!("{KEY}/rules/1/geometry_part/cube")
            ),
            failed_internal()
        ]
    );
}

#[test]
fn r02_a_face_without_a_cube() {
    // Vanilla `…NoCube_Parse_ExpectFail`.
    let json = latest(&[
        BONE1_EAST,
        r#"{"geometry_part": {"bone": "bone2", "face": "east"}, "direction": "south"}"#,
    ]);
    assert_eq!(
        errors(&rejects(&json)),
        [
            error(
                "block_culling/face_without_cube",
                "Rule 1 with geometry part 'bone2' had a face specified, but no cube index",
                &format!("{KEY}/rules/1/geometry_part/face")
            ),
            failed_internal()
        ]
    );
}

#[test]
fn r03_a_part_an_earlier_rule_has_whatever_the_direction() {
    // Vanilla `…GeoPartCollision_Parse_ExpectFail`: the two rules differ only in direction.
    let json = latest(&[
        BONE1_EAST,
        r#"{"geometry_part": {"bone": "bone1", "cube": 2, "face": "east"}, "direction": "south"}"#,
    ]);
    assert_eq!(
        errors(&rejects(&json)),
        [
            error(
                "block_culling/duplicate_part",
                "Rule 1 had geometry part 'bone1' cube index 2 face index 'East' that matches an earlier rule",
                &format!("{KEY}/rules/1/geometry_part")
            ),
            failed_internal()
        ]
    );
}

#[test]
fn the_duplicate_key_is_bone_cube_and_face_and_nothing_else() {
    let rule = |bone: &str, cube: i32, face: &str, extra: &str| {
        format!(
            r#"{{{extra}"geometry_part": {{"bone": "{bone}", "cube": {cube}, "face": "{face}"}}, "direction": "up"}}"#
        )
    };
    let first = rule("b", 0, "up", "");
    // A different bone, cube or face is another part.
    for other in [
        rule("c", 0, "up", ""),
        rule("b", 1, "up", ""),
        rule("b", 0, "down", ""),
        rule("B", 0, "up", ""),
    ] {
        assert_eq!(loads(&latest(&[&first, &other])).rules.len(), 2, "{other}");
    }
    // The condition and the flag do not make it another part.
    for other in [
        rule("b", 0, "up", r#""condition": "same_block", "#),
        rule("b", 0, "up", r#""cull_against_full_and_opaque": false, "#),
    ] {
        let found = texts(&latest(&[&first, &other]));
        assert_eq!(found[0].0, "block_culling/duplicate_part", "{other}");
    }
    // The third of three is reported as well as the second, each against an earlier rule.
    let found = texts(&latest(&[&first, &first, &first]));
    assert_eq!(
        found,
        [
            text(
                "block_culling/duplicate_part",
                "Rule 1 had geometry part 'b' cube index 0 face index 'Up' that matches an earlier rule"
            ),
            text(
                "block_culling/duplicate_part",
                "Rule 2 had geometry part 'b' cube index 0 face index 'Up' that matches an earlier rule"
            ),
            internal()
        ]
    );
}

#[test]
fn r04_two_whole_bone_rules_of_one_bone_are_duplicates() {
    let json = latest(&[
        r#"{"geometry_part": {"bone": "b"}, "direction": "up"}"#,
        r#"{"geometry_part": {"bone": "b"}, "direction": "down"}"#,
    ]);
    assert_eq!(
        texts(&json),
        [
            text(
                "block_culling/duplicate_part",
                "Rule 1 had geometry part 'b' cube index -1 face index 'Unknown facing' that matches an earlier rule"
            ),
            internal()
        ]
    );
}

#[test]
fn r05_every_offending_rule_is_logged_before_the_one_rejection() {
    let json = latest(&[
        BONE1_EAST,
        r#"{"geometry_part": {"bone": "no_face", "cube": 1}, "direction": "south"}"#,
        r#"{"geometry_part": {"bone": "no_cube", "face": "west"}, "direction": "south"}"#,
        BONE1_EAST,
    ]);
    assert_eq!(
        texts(&json),
        [
            text(
                "block_culling/cube_without_face",
                "Rule 1 with geometry part 'no_face' had a cube index but no face specified"
            ),
            text(
                "block_culling/face_without_cube",
                "Rule 2 with geometry part 'no_cube' had a face specified, but no cube index"
            ),
            text(
                "block_culling/duplicate_part",
                "Rule 3 had geometry part 'bone1' cube index 2 face index 'East' that matches an earlier rule"
            ),
            internal()
        ]
    );
}

#[test]
fn r06_the_minecraft_namespace_is_not_a_user_pack_s() {
    // Vanilla `…MinecraftNamespaceNotBasePack_Parse_ExpectFail`.
    assert_eq!(
        errors(&rejects(&named("minecraft:culling_parsing"))),
        [
            error(
                "block_culling/minecraft_namespace",
                "Identifier not allowed to use Minecraft namespace",
                &format!("{KEY}/description/identifier")
            ),
            error(
                "block_culling/identifier_failed",
                "Identifier 'minecraft:culling_parsing' failed validation when registering block culling json",
                &format!("{KEY}/description/identifier")
            ),
            failed_internal()
        ]
    );
}

#[test]
fn r07_the_minecraft_namespace_is_a_base_game_pack_s() {
    // Vanilla `…MinecraftNamespaceBasePack_Parse_ExpectSuccess`.
    let document = load_in(&named("minecraft:culling_parsing"), &base_game()).unwrap();
    assert!(document.diagnostics().is_empty());
    assert_eq!(
        document.description.identifier.as_str(),
        "minecraft:culling_parsing"
    );
    // And a base-game pack may use another namespace.
    assert!(load_in(&named("test:x"), &base_game()).is_ok());
}

#[test]
fn r08_an_identifier_needs_a_namespace() {
    assert_eq!(
        texts(&named("lamp")),
        [
            text(
                "block_culling/needs_namespace",
                "Identifier must have a namespace"
            ),
            text(
                "block_culling/identifier_failed",
                "Identifier 'lamp' failed validation when registering block culling json"
            ),
            internal()
        ]
    );
    // In a base-game pack too.
    let found = errors(&rejects_in(&named("lamp"), &base_game()));
    assert_eq!(found[0].1, "Identifier must have a namespace");
}

#[test]
fn r09_white_space_in_the_name_or_the_namespace() {
    assert_eq!(
        texts(&named("demo:my lamp")),
        [
            text(
                "block_culling/illegal_characters",
                "Identifier cannot have illegal characters"
            ),
            text("block_culling/name_invalid", "Identifier name is invalid"),
            text(
                "block_culling/identifier_failed",
                "Identifier 'demo:my lamp' failed validation when registering block culling json"
            ),
            internal()
        ]
    );
    assert_eq!(
        texts(&named("de mo:lamp"))[..2],
        [
            text(
                "block_culling/illegal_characters",
                "Identifier cannot have illegal characters"
            ),
            text("block_culling/namespace_invalid", "Namespace is invalid"),
        ]
    );
    // Space, tab, line feed and carriage return, in either part; the namespace is checked first.
    for identifier in [
        "a:b\\tc", "a:b\\nc", "a:b\\rc", "a\\tb:c", "a\\nb:c", "a\\rb:c",
    ] {
        let found = texts(&named(identifier));
        assert_eq!(
            found[0].1, "Identifier cannot have illegal characters",
            "{identifier}"
        );
    }
    assert_eq!(texts(&named("a b:c d"))[1].1, "Namespace is invalid");
    // No other character is refused: upper case, `.`, `/`, `-` and non-ASCII pass.
    for identifier in ["Demo:Lamp", "a.b:c.d", "a/b:c/d", "a-b:c-d", "dé:mö", "1:2"] {
        let document = loads(&named(identifier));
        assert_eq!(document.description.identifier.as_str(), identifier);
    }
}

#[test]
fn r10_a_bad_identifier_skips_the_rule_checks() {
    let json = document("1.21.80", "lamp", &[BONE1_EAST, BONE1_EAST]);
    let found = texts(&json);
    assert_eq!(found.len(), 3, "{found:#?}");
    assert!(
        rejects(&json)
            .iter()
            .all(|d| d.code.kind() != Kind::Duplicate)
    );
    // With a sound identifier the duplicate is found.
    let json = document("1.21.80", "a:lamp", &[BONE1_EAST, BONE1_EAST]);
    assert_eq!(texts(&json)[0].0, "block_culling/duplicate_part");
}

#[test]
fn r12_an_empty_identifier_fails_without_a_reason() {
    assert_eq!(
        texts(&named("")),
        [
            text(
                "block_culling/identifier_failed",
                "Identifier '' failed validation when registering block culling json"
            ),
            internal()
        ]
    );
}

#[test]
fn r13_the_minecraft_namespace_is_compared_without_case() {
    for identifier in ["Minecraft:hedge", "MINECRAFT:hedge", "mineCraft:hedge"] {
        let found = texts(&named(identifier));
        assert_eq!(
            found[0],
            text(
                "block_culling/minecraft_namespace",
                "Identifier not allowed to use Minecraft namespace"
            ),
            "{identifier}"
        );
        assert!(
            load_in(&named(identifier), &base_game()).is_ok(),
            "{identifier}"
        );
    }
    // Only the whole namespace: a longer one is another namespace.
    loads(&named("minecraftx:hedge"));
    loads(&named("xminecraft:hedge"));
}

#[test]
fn r14_more_than_one_colon_or_an_empty_side() {
    for identifier in ["demo:a:b", ":b", "a:", ":", "a::b"] {
        assert_eq!(
            texts(&named(identifier)),
            [
                text("block_culling/invalid_identifier", "Invalid identifier"),
                text(
                    "block_culling/identifier_failed",
                    &format!(
                        "Identifier '{identifier}' failed validation when registering block culling json"
                    )
                ),
                internal()
            ],
            "{identifier}"
        );
    }
}

#[test]
fn the_checks_run_on_the_upgraded_document() {
    // `prepareCullingData` takes the 1.21.80 value: a 1.20.60 file fails the same checks.
    let json = document("1.20.60", "lamp", &[BONE1_EAST]);
    assert_eq!(
        errors(&rejects(&json))[0].1,
        "Identifier must have a namespace"
    );
    let json = document(
        "1.20.60",
        "a:b",
        &[r#"{"geometry_part": {"bone": "b", "cube": 1}, "direction": "up"}"#],
    );
    assert_eq!(
        errors(&rejects_in(&json, &user_pack()))[0].1,
        "Rule 0 with geometry part 'b' had a cube index but no face specified"
    );
}
