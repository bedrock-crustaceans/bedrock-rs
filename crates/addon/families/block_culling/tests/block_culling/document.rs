//! Rows u01 to u03 (the upgrade from 1.20.60), l01 to l05 (documents at 1.21.80) and r11 (the
//! client's view of a rule).

use bedrock_addon_core::document::{Content, Placed};
use bedrock_addon_core::id::CullingRulesId;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_family_block_culling::history::v1_20_60;
use bedrock_addon_family_block_culling::{
    Condition, CullingRule, CullingRules, Facing, GeometryPart, Rule,
};

use super::common::{BONE1_EAST, fixture, floor, latest, loads};

fn id(text: &str) -> CullingRulesId {
    CullingRulesId::new(text).unwrap()
}

/// The JSON of `text`, for comparing documents whatever their layout.
fn json(text: &str) -> Value {
    facet_json::from_str::<Value>(text).unwrap_or_else(|e| panic!("{e}: {text}"))
}

#[test]
fn u01_a_1_20_60_document_gains_the_flag_and_no_condition() {
    // Vanilla `BlockCullingR20U6Json_ParseWithCereal_ExpectPUVBlockCullingCorrect`: "Expected
    // culling against full and opaque to be the default value of true", "Expected the
    // condition to be default value of nullopt", then an empty log.
    let document = loads(&fixture("u01_1_20_60_basic.before.json"));
    assert_eq!(document.description.identifier, id("test:culling_parsing"));
    assert_eq!(
        document.rules,
        [Rule {
            condition: None,
            cull_against_full_and_opaque: Some(true),
            geometry_part: GeometryPart::cube_face("root_bone", 0, Facing::North),
            direction: Facing::North,
        }]
    );
    assert_eq!(
        document.source_format_version,
        FormatVersion::new(1, 20, 60)
    );
    assert!(document.version().upgraded);
    assert_eq!(document.version().parsed_as, FormatVersion::new(1, 20, 60));
    // Serialised at 1.21.80.
    assert_eq!(
        json(&document.to_json().unwrap()),
        json(&fixture("u01_1_20_60_basic.after.json"))
    );
}

#[test]
fn u02_every_part_and_direction_of_a_1_20_60_document_is_copied() {
    // The docs' sushi example: two face rules and one whole-bone rule.
    let document = loads(&floor(&[
        r#"{"geometry_part": {"bone": "bb_main", "cube": 0, "face": "up"}, "direction": "up"}"#,
        r#"{"geometry_part": {"bone": "bb_main", "cube": 0, "face": "down"}, "direction": "down"}"#,
        r#"{"geometry_part": {"bone": "rice"}, "direction": "west"}"#,
    ]));
    let parts: Vec<(&GeometryPart, Facing)> = document
        .rules
        .iter()
        .map(|rule| (&rule.geometry_part, rule.direction))
        .collect();
    assert_eq!(
        parts,
        [
            (
                &GeometryPart::cube_face("bb_main", 0, Facing::Up),
                Facing::Up
            ),
            (
                &GeometryPart::cube_face("bb_main", 0, Facing::Down),
                Facing::Down
            ),
            (&GeometryPart::whole_bone("rice"), Facing::West),
        ]
    );
    // The whole-bone rule keeps `cube` and `face` absent, in the value and in the text.
    assert_eq!(document.rules[2].geometry_part.cube, None);
    assert_eq!(document.rules[2].geometry_part.face, None);
    let written = json(&document.to_json().unwrap());
    let part = &written.as_object().unwrap()["minecraft:block_culling_rules"]
        .as_object()
        .unwrap()["rules"]
        .as_array()
        .unwrap()[2]
        .as_object()
        .unwrap()["geometry_part"];
    assert_eq!(part, &json(r#"{"bone": "rice"}"#));
    for rule in &document.rules {
        assert_eq!(rule.condition, None);
        assert_eq!(rule.cull_against_full_and_opaque, Some(true));
    }
}

#[test]
fn u03_an_empty_rule_list_upgrades_and_loads() {
    let document = loads(&floor(&[]));
    assert!(document.rules.is_empty());
    let written = document.to_json().unwrap();
    assert_eq!(
        json(&written),
        json(
            r#"{"format_version": "1.21.80", "minecraft:block_culling_rules": {"description": {"identifier": "test:culling_parsing"}, "rules": []}}"#
        )
    );
    assert!(loads(&latest(&[])).rules.is_empty(), "at 1.21.80 too");
}

#[test]
fn the_1_20_60_struct_is_what_the_file_holds_before_the_step() {
    use bedrock_addon_core::diagnostic::Diagnostics;
    use bedrock_addon_core::json::JsonSlice;
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::versioning::VersionChain;
    use bedrock_addon_family_block_culling::history::{Any, CullingFamily};
    let payload = r#"{"description": {"identifier": "a:b"}, "rules": [{"geometry_part": {"bone": "x"}, "direction": "up"}]}"#;
    let mut d = Diagnostics::default();
    let parsed = <Any as VersionChain<CullingFamily>>::parse(
        JsonSlice::new(payload),
        FormatVersion::new(1, 21, 79),
        &LoadContext::detached(),
        &mut d,
    );
    assert!(d.is_empty(), "{d:#?}");
    let Some(Any::V1_20_60(document)) = parsed else {
        panic!("1.21.79 is the 1.20.60 struct's: {parsed:#?}");
    };
    assert_eq!(
        document.rules,
        [v1_20_60::Rule {
            geometry_part: GeometryPart::whole_bone("x"),
            direction: Facing::Up,
        }]
    );
    let parsed = <Any as VersionChain<CullingFamily>>::parse(
        JsonSlice::new(payload),
        FormatVersion::new(1, 21, 80),
        &LoadContext::detached(),
        &mut d,
    );
    assert!(matches!(parsed, Some(Any::V1_21_80(_))), "{parsed:#?}");
    assert_eq!(
        <Any as VersionChain<CullingFamily>>::VERSIONS,
        [FormatVersion::new(1, 20, 60), FormatVersion::new(1, 21, 80)]
    );
}

#[test]
fn l01_a_1_21_80_rule_holds_its_condition_and_flag() {
    // Vanilla `BlockCullingR21U8Json_…`.
    let document = loads(&fixture("l01_1_21_80_condition_and_flag.json"));
    assert_eq!(
        document.rules,
        [Rule {
            condition: Some(Condition::SameCullingLayer),
            cull_against_full_and_opaque: Some(false),
            geometry_part: GeometryPart::cube_face("root_bone", 0, Facing::North),
            direction: Facing::North,
        }]
    );
    assert!(!document.version().upgraded);
    assert_eq!(document.rules[0].to_client_rule().condition, Some(0));
    assert!(
        !document.rules[0]
            .to_client_rule()
            .cull_against_full_and_opaque
    );
}

#[test]
fn l02_a_1_21_80_rule_without_condition_or_flag_keeps_both_absent() {
    // Vanilla `BlockCullingR21U8WithNoConditionJson_…`: condition absent, flag `true`.
    let document = loads(&fixture("l02_1_21_80_no_condition.json"));
    let rule = &document.rules[0];
    assert_eq!(rule.condition, None);
    assert_eq!(
        rule.cull_against_full_and_opaque, None,
        "not written, not invented"
    );
    assert!(rule.culls_against_full_and_opaque(), "the client's default");
    assert_eq!(rule.to_client_rule().condition, None);
    assert!(rule.to_client_rule().cull_against_full_and_opaque);
    // Write-back does not invent the member.
    assert!(!document.to_json().unwrap().contains("cull_against"));
}

#[test]
fn l03_each_condition_has_the_client_s_number() {
    let rule = |condition: &str, bone: &str| {
        format!(
            r#"{{"condition": "{condition}", "geometry_part": {{"bone": "{bone}"}}, "direction": "up"}}"#
        )
    };
    let rules = [
        rule("same_culling_layer", "a"),
        rule("same_block", "b"),
        rule("same_block_permutation", "c"),
    ];
    let rules: Vec<&str> = rules.iter().map(String::as_str).collect();
    let document = loads(&latest(&rules));
    let conditions: Vec<Option<Condition>> =
        document.rules.iter().map(|rule| rule.condition).collect();
    assert_eq!(
        conditions,
        [
            Some(Condition::SameCullingLayer),
            Some(Condition::SameBlock),
            Some(Condition::SameBlockPermutation)
        ]
    );
    let converted: Vec<Option<u8>> = document
        .to_client_rules()
        .iter()
        .map(|rule| rule.condition)
        .collect();
    assert_eq!(converted, [Some(0), Some(1), Some(2)]);
    assert_eq!(Condition::SameCullingLayer as i32, 1);
    assert_eq!(Condition::SameBlockPermutation as i32, 3);
}

#[test]
fn l04_a_whole_bone_rule_has_the_client_s_sentinels() {
    let document = loads(&latest(&[
        r#"{"geometry_part": {"bone": "b"}, "direction": "east"}"#,
    ]));
    let converted = &document.to_client_rules()[0];
    assert_eq!(converted.cube_index, -1);
    assert_eq!(converted.face_to_cull, 6);
    assert_eq!(converted.neighbor_to_check, 5);
    assert_eq!(converted.bone, "b");
}

#[test]
fn l05_vanilla_s_perf_asset_loads_with_six_rules() {
    let document = loads(&fixture("l05_vanilla_perf_asset.json"));
    assert_eq!(
        document.description.identifier,
        id("puv_perf:block_culling_test")
    );
    assert_eq!(document.rules.len(), 6);
    assert!(
        document
            .rules
            .iter()
            .all(|r| r.geometry_part.bone == "root_bone")
    );
    assert_eq!(
        document.rules[2],
        Rule::new(
            GeometryPart::cube_face("root_bone", 1, Facing::South),
            Facing::East
        )
    );
    let directions: Vec<Facing> = document.rules.iter().map(|r| r.direction).collect();
    assert_eq!(
        directions,
        [
            Facing::South,
            Facing::North,
            Facing::East,
            Facing::West,
            Facing::Up,
            Facing::Down
        ]
    );
}

#[test]
fn r11_the_converted_view_is_vanilla_s() {
    // Vanilla `BlockCullingPuVData_ParseToCullingData_ExpectSuccess`, rebuilt as JSON.
    let document = loads(&latest(&[
        BONE1_EAST,
        r#"{"geometry_part": {"bone": "bone2"}, "direction": "south"}"#,
    ]));
    assert_eq!(
        document.description.identifier.as_str(),
        "test:culling_parsing"
    );
    let converted = document.to_client_rules();
    assert_eq!(converted.len(), 2);
    let CullingRule {
        neighbor_to_check,
        condition,
        cull_against_full_and_opaque,
        bone,
        cube_index,
        face_to_cull,
        ..
    } = &converted[0];
    assert_eq!(*neighbor_to_check, Facing::North.number());
    assert_eq!(*condition, None);
    assert!(*cull_against_full_and_opaque);
    assert_eq!(bone, "bone1");
    assert_eq!(*cube_index, 2);
    assert_eq!(*face_to_cull, Facing::East.number());
    assert_eq!(converted[1].bone, "bone2");
    assert_eq!(converted[1].cube_index, -1);
    assert_eq!(converted[1].face_to_cull, 6);
    assert_eq!(converted[1].neighbor_to_check, Facing::South.number());
}

#[test]
fn facing_and_condition_have_vanilla_s_names_and_numbers() {
    let facings: Vec<(&str, u8)> = Facing::ALL.iter().map(|f| (f.name(), f.number())).collect();
    assert_eq!(
        facings,
        [
            ("down", 0),
            ("up", 1),
            ("north", 2),
            ("south", 3),
            ("west", 4),
            ("east", 5)
        ]
    );
    for facing in Facing::ALL {
        assert_eq!(Facing::from_name(facing.name()), Some(facing));
    }
    assert_eq!(Facing::from_name("North"), None, "names compare as written");
    for condition in Condition::ALL {
        assert_eq!(Condition::from_name(condition.name()), Some(condition));
    }
    assert_eq!(Condition::from_name("not_defined"), None);
}

#[test]
fn the_default_path_is_the_identifier_below_block_culling() {
    let rules = CullingRules::new(id("demo:hedge"), Vec::new());
    assert_eq!(
        rules.default_path().as_str(),
        "block_culling/demo/hedge.json"
    );
    assert_eq!(rules.source_format_version, FormatVersion::new(1, 21, 80));
    assert_eq!(CullingRules::NAME, "block culling rules");
    // Vanilla's own identifiers have a dot in the name. Only a base-game pack may hold a
    // `minecraft:` document, and its walk is flat: the file sits in the folder itself, as
    // vanilla's do.
    let vanilla = CullingRules::new(id("minecraft:culling.leaves"), Vec::new());
    assert_eq!(
        vanilla.default_path().as_str(),
        "block_culling/culling.leaves.json"
    );
    for (identifier, path) in [
        ("Minecraft:Leaves", "block_culling/Leaves.json"),
        ("minecraft:a/b", "block_culling/a_b.json"),
        ("demo:a/b", "block_culling/demo/a/b.json"),
        ("minecraftish:x", "block_culling/minecraftish/x.json"),
    ] {
        let rules = CullingRules::new(id(identifier), Vec::new());
        assert_eq!(rules.default_path().as_str(), path, "{identifier}");
    }
}

#[test]
fn the_default_path_of_an_unusual_identifier_is_still_one_file_below_block_culling() {
    // As the loader keeps them: bare, malformed, or with characters a path cannot hold.
    for (identifier, path) in [
        ("a", "block_culling/a.json"),
        ("a/b", "block_culling/a/b.json"),
        ("minecraft:a/../b", "block_culling/a___b.json"),
        ("minecraft:./a", "block_culling/__a.json"),
        ("minecraft:", "block_culling/_.json"),
        (":a", "block_culling/a.json"),
        ("a:b:c", "block_culling/a/b_c.json"),
        ("minecraft:a:b", "block_culling/a_b.json"),
        ("MINECRAFT:x/y", "block_culling/x_y.json"),
        ("minecraft:a\u{1}b", "block_culling/a_b.json"),
    ] {
        let rules = CullingRules::new(CullingRulesId::parse(identifier.to_owned()), Vec::new());
        assert_eq!(rules.default_path().as_str(), path, "{identifier:?}");
    }
}
