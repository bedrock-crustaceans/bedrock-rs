//! Round trips: vanilla's own files, values built in code, and a few thousand generated
//! documents. Whatever loads writes text that loads back to the same rules; whatever the
//! loader would refuse is not written.

use bedrock_addon_core::diagnostic::Kind;
use bedrock_addon_core::document::{Content, Rejected};
use bedrock_addon_core::id::CullingRulesId;
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_family_block_culling::{
    Condition, CullingRules, Description, Facing, GeometryPart, Rule,
};

use super::common::{base_game, fixture, load_in, user_pack};

/// Writes `rules`, reads the text back from a base-game pack (so every identifier a pack may
/// carry reads), and requires the same value and an identical second writing.
#[track_caller]
fn round_trips(rules: &CullingRules) -> String {
    let written = rules
        .to_json()
        .unwrap_or_else(|e| panic!("{e}: {rules:#?}"));
    let read = load_in(&written, &base_game())
        .unwrap_or_else(|r| panic!("written text is refused: {written}\n{:#?}", r.diagnostics));
    assert!(
        read.diagnostics().is_empty(),
        "{written}\n{:#?}",
        read.diagnostics()
    );
    assert_eq!(read.description, rules.description, "{written}");
    assert_eq!(read.rules, rules.rules, "{written}");
    assert_eq!(read.source_format_version, FormatVersion::new(1, 21, 80));
    assert_eq!(read.to_json().unwrap(), written, "a second writing differs");
    written
}

#[test]
fn vanilla_s_own_files_round_trip() {
    // Copied from the retail 1.26.45 game data,
    // `resource_packs/experimental_y_2026_drop_3/block_culling/`.
    for name in [
        "shelf_mushroom_large",
        "shelf_mushroom_small",
        "straw_bed_foot",
        "straw_bed_head",
    ] {
        let text = fixture(&format!("vanilla/{name}.culling.json"));
        let document =
            load_in(&text, &base_game()).unwrap_or_else(|r| panic!("{name}: {:#?}", r.diagnostics));
        assert!(document.diagnostics().is_empty(), "{name}");
        assert_eq!(
            document.description.identifier.as_str(),
            format!("minecraft:culling.{name}")
        );
        assert!(!document.rules.is_empty(), "{name}");
        assert_eq!(
            document.source_format_version,
            FormatVersion::new(1, 20, 60)
        );
        round_trips(document.value());
        // The same file in a user pack is refused: the namespace is the base game's.
        assert!(load_in(&text, &user_pack()).is_err(), "{name}");
    }
}

/// `experimental_y_2026_drop_4/block_culling/icicle_base.culling.json` of 1.26.60
/// (the drop's one culling file, format 1.20.60): it loads from
/// the base-game pack without a finding, its five rules one per face of the matching bone, and
/// round-trips.
#[test]
fn the_drop_4_icicle_base_round_trips() {
    let text = fixture("vanilla/icicle_base.culling.json");
    let document = load_in(&text, &base_game()).unwrap_or_else(|r| panic!("{:#?}", r.diagnostics));
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(
        document.description.identifier.as_str(),
        "minecraft:culling.icicle_base"
    );
    assert_eq!(
        document.source_format_version,
        FormatVersion::new(1, 20, 60)
    );
    let directions: Vec<Facing> = document.rules.iter().map(|r| r.direction).collect();
    assert_eq!(
        directions,
        [
            Facing::Up,
            Facing::North,
            Facing::South,
            Facing::West,
            Facing::East
        ]
    );
    round_trips(document.value());
    assert!(load_in(&text, &user_pack()).is_err());
}

/// The game's loader performance input for culling rules of 1.26.60, the fixture
/// `l05_vanilla_perf_asset.json` byte for byte: it loads from any pack without a finding and
/// round-trips.
#[test]
fn loader_perf_tests_3_the_perf_culling_rules_round_trip() {
    let text = fixture("l05_vanilla_perf_asset.json");
    let document = load_in(&text, &user_pack()).unwrap_or_else(|r| panic!("{:#?}", r.diagnostics));
    assert!(
        document.diagnostics().is_empty(),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(document.rules.len(), 6);
    round_trips(document.value());
}

#[test]
fn a_value_built_by_hand_writes_only_what_it_holds() {
    let rules = CullingRules::new(
        CullingRulesId::new("demo:hedge").unwrap(),
        vec![
            Rule::new(GeometryPart::whole_bone("trunk"), Facing::Down),
            Rule {
                condition: Some(Condition::SameBlockPermutation),
                cull_against_full_and_opaque: Some(false),
                geometry_part: GeometryPart::cube_face("leaves", 3, Facing::West),
                direction: Facing::East,
            },
        ],
    );
    let written = round_trips(&rules);
    assert_eq!(
        written,
        r#"{
  "format_version": "1.21.80",
  "minecraft:block_culling_rules": {
    "description": {
      "identifier": "demo:hedge"
    },
    "rules": [
      {
        "geometry_part": {
          "bone": "trunk"
        },
        "direction": "down"
      },
      {
        "condition": "same_block_permutation",
        "cull_against_full_and_opaque": false,
        "geometry_part": {
          "bone": "leaves",
          "cube": 3,
          "face": "west"
        },
        "direction": "east"
      }
    ]
  }
}
"#
    );
    // Struct-update syntax works from outside the crate.
    let other = CullingRules {
        description: Description {
            identifier: CullingRulesId::new("demo:other").unwrap(),
        },
        ..rules
    };
    round_trips(&other);
}

/// The path of a refusal to write.
fn refused(rules: &CullingRules) -> String {
    match rules.to_json() {
        Ok(text) => panic!("written: {text}"),
        Err(EncodeError::NotRepresentable { at, .. }) => at.to_string(),
        Err(other) => panic!("{other}"),
    }
}

#[test]
fn a_value_the_loader_would_refuse_is_not_written() {
    let key = "minecraft:block_culling_rules";
    let named = |identifier: &str| {
        CullingRules::new(
            CullingRulesId::parse(identifier),
            vec![Rule::new(GeometryPart::whole_bone("b"), Facing::Up)],
        )
    };
    for identifier in ["", "lamp", "a:b:c", ":b", "a:", "a b:c", "a:b c", "a:b\tc"] {
        assert_eq!(
            refused(&named(identifier)),
            format!("{key}/description/identifier"),
            "{identifier:?}"
        );
    }
    // The `minecraft` namespace is a base-game pack's to use: it is written.
    round_trips(&named("minecraft:culling.leaves"));
    let with_rules = |rules: Vec<Rule>| CullingRules::new(CullingRulesId::parse("a:b"), rules);
    let cube_only = GeometryPart {
        bone: "b".to_owned(),
        cube: Some(1),
        face: None,
    };
    let face_only = GeometryPart {
        bone: "b".to_owned(),
        cube: None,
        face: Some(Facing::Up),
    };
    let sound = || Rule::new(GeometryPart::whole_bone("a"), Facing::Up);
    assert_eq!(
        refused(&with_rules(vec![sound(), Rule::new(cube_only, Facing::Up)])),
        format!("{key}/rules/1/geometry_part")
    );
    assert_eq!(
        refused(&with_rules(vec![Rule::new(face_only, Facing::Up)])),
        format!("{key}/rules/0/geometry_part")
    );
    assert_eq!(
        refused(&with_rules(vec![
            sound(),
            Rule::new(GeometryPart::whole_bone("c"), Facing::Up),
            Rule::new(GeometryPart::whole_bone("a"), Facing::Down)
        ])),
        format!("{key}/rules/2/geometry_part")
    );
    assert_eq!(
        refused(&with_rules(vec![Rule::new(
            GeometryPart::cube_face("b", -1, Facing::Up),
            Facing::Up
        )])),
        format!("{key}/rules/0/geometry_part/cube")
    );
    // The boundary: cube 0 is written.
    round_trips(&with_rules(vec![Rule::new(
        GeometryPart::cube_face("b", 0, Facing::Up),
        Facing::Up,
    )]));
    round_trips(&with_rules(vec![Rule::new(
        GeometryPart::cube_face("b", i32::MAX, Facing::Up),
        Facing::Up,
    )]));
}

/// A small deterministic generator (an LCG; no clock, no host randomness).
struct Generator(u64);

impl Generator {
    fn next(&mut self, below: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from((self.0 >> 33) % (below as u64)).unwrap()
    }

    fn pick<'a>(&mut self, of: &[&'a str]) -> &'a str {
        of[self.next(of.len())]
    }
}

/// One generated document.
struct Generated {
    /// The document's text.
    text: String,
    /// The same document as the writer spells it: the version a string, the members in
    /// vanilla's order, a whole cube index without a fraction.
    canonical: String,
    /// `format_version` is an array of three numbers.
    array_version: bool,
    /// A `cube` is a whole number written as a float (`2.0`, `1e1`).
    float_cube: bool,
    /// The members of some object are not in vanilla's order.
    reordered: bool,
}

/// `members` as an object's text; one time in `one_in`, in another order than given (which
/// sets `reordered`).
fn object(g: &mut Generator, members: &[String], one_in: usize, reordered: &mut bool) -> String {
    let mut members = members.to_vec();
    if members.len() > 1 && g.next(one_in) == 0 {
        let by = 1 + g.next(members.len() - 1);
        members.rotate_left(by);
        *reordered = true;
    }
    format!("{{{}}}", members.join(", "))
}

/// One rule as written and as the writer spells it.
fn generate_rule(
    g: &mut Generator,
    index: usize,
    float_cube: &mut bool,
    reordered: &mut bool,
) -> (String, String) {
    let mut members = Vec::new();
    match g.next(6) {
        0 => members.push(r#""condition": "same_culling_layer""#.to_owned()),
        1 => members.push(r#""condition": "same_block""#.to_owned()),
        2 => members.push(r#""condition": "sideways""#.to_owned()),
        _ => {}
    }
    match g.next(6) {
        0 => members.push(r#""cull_against_full_and_opaque": true"#.to_owned()),
        1 => members.push(r#""cull_against_full_and_opaque": false"#.to_owned()),
        2 => members.push(r#""cull_against_full_and_opaque": "x""#.to_owned()),
        _ => {}
    }
    let mut canonical_members = members.clone();
    let bone = g.pick(&["b", "b", "bone with space", "ü"]);
    // A cube index as written and as the writer spells it.
    let (cube, whole) = [
        ("", ""),
        ("", ""),
        ("0", "0"),
        ("3", "3"),
        ("-1", "-1"),
        ("1.5", "1.5"),
        ("1e39", "1e39"),
        ("1e999", "1e999"),
        ("2.0", "2"),
        ("1e1", "10"),
        ("0.0", "0"),
        ("3.00", "3"),
        ("18446744073709551616", "18446744073709551616"),
    ][g.next(13)];
    *float_cube |= cube != whole;
    let face = g.pick(&["", "", "up", "east", "East"]);
    let mut part = vec![format!(r#""bone": "{bone}{index}""#)];
    let mut canonical_part = part.clone();
    if !cube.is_empty() {
        part.push(format!(r#""cube": {cube}"#));
        canonical_part.push(format!(r#""cube": {whole}"#));
    }
    if !face.is_empty() {
        part.push(format!(r#""face": "{face}""#));
        canonical_part.push(part.last().cloned().unwrap());
    }
    if g.next(12) == 0 {
        part.push(r#""extra": 1"#.to_owned());
        canonical_part.push(part.last().cloned().unwrap());
    }
    if g.next(15) != 0 {
        members.push(format!(
            r#""geometry_part": {}"#,
            object(g, &part, 6, reordered)
        ));
        canonical_members.push(format!(
            r#""geometry_part": {{{}}}"#,
            canonical_part.join(", ")
        ));
    }
    match g.next(10) {
        0 => {}
        1 => members.push(r#""direction": "inwards""#.to_owned()),
        _ => members.push(format!(
            r#""direction": "{}""#,
            g.pick(&["down", "up", "north", "south", "west", "east"])
        )),
    }
    if members.len() > canonical_members.len() {
        canonical_members.push(members.last().cloned().unwrap());
    }
    let rule = object(g, &members, 6, reordered);
    let canonical_rule = format!("{{{}}}", canonical_members.join(", "));
    (rule, canonical_rule)
}

/// One generated document: every member in each of its forms, sound and unsound.
fn generate(g: &mut Generator) -> Generated {
    let version = g.pick(&[
        "1.20.60", "1.21.0", "1.21.79", "1.21.80", "1.21.80", "1.21.80",
    ]);
    let array_version = g.next(5) == 0;
    let identifier = g.pick(&[
        "demo:hedge",
        "demo:hedge",
        "a.b:c/d",
        "Demo:Lamp",
        "minecraft:culling.leaves",
        "lamp",
        "a:b:c",
        "a:b c",
        "",
    ]);
    let (mut float_cube, mut reordered) = (false, false);
    let count = g.next(4);
    // Each rule as written and as the writer spells it.
    let (mut rules, mut canonical_rules) = (Vec::new(), Vec::new());
    for index in 0..count {
        let (rule, canonical_rule) = generate_rule(g, index, &mut float_cube, &mut reordered);
        rules.push(rule);
        canonical_rules.push(canonical_rule);
    }
    let description = format!(r#""description": {{"identifier": "{identifier}"}}"#);
    let payload = |rules: &[String]| {
        [
            description.clone(),
            format!(r#""rules": [{}]"#, rules.join(", ")),
        ]
    };
    let written_version = if array_version {
        format!("[{}]", version.replace('.', ", "))
    } else {
        format!(r#""{version}""#)
    };
    let root = [
        format!(r#""format_version": {written_version}"#),
        format!(
            r#""minecraft:block_culling_rules": {}"#,
            object(g, &payload(&rules), 8, &mut reordered)
        ),
    ];
    let text = object(g, &root, 8, &mut reordered);
    let canonical_rules: Vec<&str> = canonical_rules.iter().map(String::as_str).collect();
    Generated {
        text,
        canonical: super::common::document(version, identifier, &canonical_rules),
        array_version,
        float_cube,
        reordered,
    }
}

/// `text` without the white space between its tokens.
fn compact(text: &str) -> String {
    let mut out = String::new();
    let (mut in_string, mut escaped) = (false, false);
    for c in text.chars() {
        if in_string {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
        } else if !c.is_whitespace() {
            in_string = c == '"';
            out.push(c);
        }
    }
    out
}

/// What the 1.20.60 → 1.21.80 step makes of a document below 1.21.80: the newest version,
/// and `cull_against_full_and_opaque: true` on every rule (the older client always did).
fn at_latest(document: &Value) -> Value {
    let mut root = document.as_object().unwrap().clone();
    root.insert("format_version", "1.21.80");
    let mut payload = root
        .get("minecraft:block_culling_rules")
        .and_then(Value::as_object)
        .unwrap()
        .clone();
    let mut rules = facet_value::VArray::new();
    for rule in payload.get("rules").and_then(Value::as_array).unwrap() {
        let mut rule = rule.as_object().unwrap().clone();
        rule.insert("cull_against_full_and_opaque", true);
        rules.push(Value::from(rule));
    }
    payload.insert("rules", rules);
    root.insert("minecraft:block_culling_rules", payload);
    root.into()
}

/// Asserts the reasons a generated document is refused are sound: every one is about a value,
/// none about a type, and each carries the file's path. Returns whether the document holds a
/// cube beyond 64 bits.
fn assert_refusal_is_sound(refusal: &Rejected, json: &str) -> bool {
    assert!(refusal.diagnostics.has_errors(), "{json}");
    // Every cube the generator writes is a number: none is refused for its
    // type, and an integer beyond 64 bits is refused for its value, as every
    // index beyond `int` is.
    let at_cube = |kind: Kind| {
        refusal
            .diagnostics
            .iter()
            .filter(|f| {
                f.code.kind() == kind && f.location.json_path.to_string().ends_with("/cube")
            })
            .count()
    };
    assert_eq!(at_cube(Kind::WrongType), 0, "{json}");
    if json.contains("18446744073709551616") {
        assert!(at_cube(Kind::InvalidValue) > 0, "{json}");
    }
    // Every reason has the file's path, and a closing line of vanilla's.
    assert!(
        refusal
            .diagnostics
            .iter()
            .all(|f| f.location.path.is_some()),
        "{json}"
    );
    json.contains("18446744073709551616")
}

#[test]
fn four_thousand_generated_documents_round_trip_or_are_refused() {
    let mut g = Generator(0x00C0_FFEE);
    let (mut loaded, mut rejected) = (0, 0);
    let (mut as_authored, mut upgraded) = (0, 0);
    let (mut array_versions, mut float_cubes, mut reorderings) = (0, 0, 0);
    let mut wide_cubes = 0;
    for _ in 0..4000 {
        let Generated {
            text: json,
            canonical,
            array_version,
            float_cube,
            reordered,
        } = generate(&mut g);
        // A base-game pack, so the `minecraft` namespace is no reason of its own.
        match load_in(&json, &base_game()) {
            Ok(document) => {
                loaded += 1;
                assert!(document.diagnostics().is_empty(), "{json}");
                let written = round_trips(document.value());
                let value = |text: &str| bedrock_addon_core::json::from_str::<Value>(text).unwrap();
                let latest = document.source_format_version == CullingRules::FORMAT_VERSION;
                // The recorded rewrites, none with a finding: an array version is
                // written as a string, a whole cube index written as a float is written as
                // an integer, and the members are written in vanilla's order. `canonical`
                // is the document with those three applied and nothing else.
                array_versions += usize::from(array_version);
                float_cubes += usize::from(float_cube);
                reorderings += usize::from(reordered);
                if !(array_version || float_cube || reordered) {
                    assert_eq!(compact(&json), compact(&canonical), "{json}");
                }
                // The generator's own two texts are one document.
                let mut same = value(&json).as_object().unwrap().clone();
                if array_version {
                    let numbers: Vec<String> = same
                        .get("format_version")
                        .and_then(Value::as_array)
                        .unwrap()
                        .iter()
                        .map(|n| n.as_number().unwrap().to_f64_lossy().to_string())
                        .collect();
                    same.insert("format_version", numbers.join(".").as_str());
                }
                assert_eq!(Value::from(same), value(&canonical), "{json}\n{canonical}");
                if latest {
                    // Write-back as authored: a document no step rewrote is written token
                    // for token as it was read, or as its canonical spelling.
                    as_authored += usize::from(!(array_version || float_cube || reordered));
                    assert_eq!(compact(&written), compact(&canonical), "{json}\n{written}");
                } else {
                    // The fourth recorded rewrite: a document below 1.21.80 is written at
                    // 1.21.80, the only version the writer has, with the flag the
                    // step gives every rule.
                    upgraded += 1;
                    assert_eq!(
                        value(&written),
                        at_latest(&value(&canonical)),
                        "{json}\n{written}"
                    );
                }
            }
            Err(refusal) => {
                rejected += 1;
                wide_cubes += usize::from(assert_refusal_is_sound(&refusal, &json));
            }
        }
    }
    eprintln!(
        "generated: {loaded} loaded ({as_authored} written as authored, {upgraded} upgraded, {array_versions} with an array version, {float_cubes} with a float cube, {reorderings} reordered), {rejected} rejected"
    );
    assert!(as_authored > 100, "documents at 1.21.80: {as_authored}");
    assert!(upgraded > 100, "documents below 1.21.80: {upgraded}");
    assert!(
        array_versions > 50,
        "loaded documents with an array version: {array_versions}"
    );
    assert!(
        float_cubes > 20,
        "loaded documents with a whole cube written as a float: {float_cubes}"
    );
    assert!(
        reorderings > 50,
        "loaded documents with members in another order: {reorderings}"
    );
    assert!(
        loaded > 300,
        "the generator makes sound documents: {loaded}"
    );
    assert!(rejected > 300, "and unsound ones: {rejected}");
    assert!(
        wide_cubes > 100,
        "documents with a cube beyond 64 bits: {wide_cubes}"
    );
}

#[test]
fn the_spellings_the_writer_does_not_keep() {
    // The three rewrites without a finding, one at a time.
    let written = |json: &str| {
        let document = load_in(json, &base_game()).unwrap_or_else(|r| panic!("{json}: {r:#?}"));
        assert!(document.diagnostics().is_empty(), "{json}");
        compact(&round_trips(document.value()))
    };
    let canonical = super::common::document(
        "1.21.80",
        "demo:hedge",
        &[
            r#"{"condition": "same_block", "cull_against_full_and_opaque": false, "geometry_part": {"bone": "b", "cube": 2, "face": "up"}, "direction": "north"}"#,
        ],
    );
    assert_eq!(written(&canonical), compact(&canonical));
    // An array `format_version` is written as the string.
    let array = canonical.replace(r#""1.21.80""#, "[1, 21, 80]");
    assert_ne!(array, canonical);
    assert_eq!(written(&array), compact(&canonical));
    // A whole cube index written as a float is written as an integer.
    for float in ["2.0", "2e0", "0.2e1"] {
        let json = canonical.replace(r#""cube": 2"#, &format!(r#""cube": {float}"#));
        assert_ne!(json, canonical);
        assert_eq!(written(&json), compact(&canonical), "{float}");
    }
    // The members are written in vanilla's order, at every level.
    let reordered = r#"{"minecraft:block_culling_rules": {"rules": [{"direction": "north", "geometry_part": {"face": "up", "cube": 2, "bone": "b"}, "cull_against_full_and_opaque": false, "condition": "same_block"}], "description": {"identifier": "demo:hedge"}}, "format_version": "1.21.80"}"#;
    assert_eq!(written(reordered), compact(&canonical));
}
