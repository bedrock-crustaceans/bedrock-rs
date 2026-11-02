//! `skins.json`. A skin pack with no access strategy cannot happen to a pack in memory and a
//! pack without an identity is the manifest's.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::{Content, Document, Placed};
use bedrock_addon::families::skin::{Skin, SkinPack, check_cape, check_texture};
use bedrock_addon::load::LoadContext;
use bedrock_addon::pack::{PackOrigin, PackType};
use bedrock_addon::storage::PackPath;

use super::common::{codes, json, located, pack_with, shown};

/// The example pack of learn.microsoft.com "Create a skin pack".
const DOCS: &str = r#"{
  "skins": [
    { "localization_name": "TemplateSkin1", "geometry": "geometry.humanoid.customSlim",
      "texture": "skin1.png", "type": "free" },
    { "localization_name": "TemplateSkin2", "geometry": "geometry.humanoid.custom",
      "texture": "skin2.png", "type": "free" }
  ],
  "serialize_name": "TemplateSkinPack",
  "localization_name": "TemplateSkinPack"
}"#;

fn parse(text: &str) -> Document<SkinPack> {
    SkinPack::parse(
        &PackPath::from_static("skins.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .expect("a skin list")
}

fn load(text: &str, geometry: Option<&str>, paths: &[&str]) -> Document<SkinPack> {
    SkinPack::load(
        Some(text.as_bytes()),
        geometry.map(str::as_bytes),
        paths.iter().copied(),
        &LoadContext::detached(),
    )
    .expect("the pack loads")
}

/// A pack named `p` with the given skin entries.
fn pack(skins: &str) -> String {
    format!(r#"{{ "serialize_name": "p", "localization_name": "p", "skins": [{skins}] }}"#)
}

fn skin(members: &str) -> String {
    format!(
        r#"{{ "localization_name": "a", "geometry": "geometry.humanoid.custom", "texture": "a.png"{members} }}"#
    )
}

fn names(doc: &Document<SkinPack>) -> Vec<String> {
    doc.skins
        .iter()
        .flatten()
        .map(|skin| skin.name().into_owned())
        .collect()
}

#[test]
fn sk01_the_documentation_s_example_pack() {
    // Through a real skin pack: the manifest module type `skin_pack` opens as a skin pack,
    // whose files a caller hands to `SkinPack::load`.
    let pack = pack_with(
        0,
        "skin_pack",
        &[],
        &[
            ("skins.json", DOCS.as_bytes()),
            ("skin1.png", b""),
            ("skin2.png", b""),
        ],
    );
    assert_eq!(pack.pack_type(), PackType::Skins);
    let bytes = pack.read(&PackPath::from_static("skins.json")).unwrap();
    let paths: Vec<&str> = pack.paths().map(PackPath::as_str).collect();
    let doc = SkinPack::load(Some(&bytes), None, paths, &LoadContext::detached()).unwrap();
    assert_eq!(codes(doc.diagnostics()), []);
    assert_eq!(names(&doc), ["TemplateSkin1", "TemplateSkin2"]);
    assert_eq!(doc.name(), "TemplateSkinPack");
    let first = &doc.skins.as_ref().unwrap()[0];
    assert!(first.uses_built_in_geometry());
    assert_eq!(first.texture_path(), "skin1.png");
    assert_eq!(first.cape_path(), None);
    assert_eq!(json(&doc.to_json().unwrap()), json(DOCS));
    assert_eq!(SkinPack::default().default_path().as_str(), "skins.json");
    assert_eq!(SkinPack::default().to_json().unwrap().trim(), "{}");
}

#[test]
fn sk02_k11_serialize_name_is_not_read_deviation() {
    for (members, mismatch) in [
        (r#""serialize_name": "x", "localization_name": "y""#, true),
        (r#""localization_name": "y""#, true),
        (r#""serialize_name": "y""#, true),
        // Neither name: the missing serialize_name is the finding, not the two being equal.
        (r#""localization_name": null"#, true),
        (r#""serialize_name": 3, "localization_name": "3""#, true),
        (r#""serialize_name": "y", "localization_name": "y""#, false),
    ] {
        let doc = parse(&format!(r#"{{ {members}, "skins": [] }}"#));
        let want: &[_] = if mismatch {
            &[("SkinPackNameMismatch", Severity::Info)]
        } else {
            &[]
        };
        assert_eq!(codes(doc.diagnostics()), want, "{members}");
    }
    let doc = parse(r#"{ "serialize_name": "x", "localization_name": "y" }"#);
    assert_eq!(
        (doc.serialize_name.as_deref(), doc.name().as_ref()),
        (Some("x"), "y"),
        "kept; the name is localization_name"
    );
}

#[test]
fn sk03_k6_an_entry_that_is_no_object() {
    let doc = parse(&pack(&format!(r#""x", {}, 3, null, []"#, skin(""))));
    assert_eq!(names(&doc), ["a"]);
    let found: Vec<_> = doc
        .diagnostics()
        .iter()
        .map(|d| (d.location.json_path.to_string(), shown(d)))
        .collect();
    let text = "Invalid entry in skins.json - Every entry under 'skins' must be a json object - it has to start with an \"{\" and end with a \"}\"";
    assert_eq!(
        found,
        ["skins/0", "skins/2", "skins/3", "skins/4"].map(|at| (
            at.to_owned(),
            ("InvalidType", Severity::Error, text.to_owned())
        ))
    );
    // The game logs the line at level 3 and goes on to the next entry.
    assert!(doc.diagnostics().iter().all(|d| d.kept));
}

#[test]
fn sk04_sk05_a_skin_whose_files_the_pack_lacks_is_left_out() {
    let text = pack(&format!(
        "{}, {}",
        skin(r#", "cape": "cape.png""#),
        skin("").replace("\"a\"", "\"b\"").replace("a.png", "b.png")
    ));
    let message =
        |doc: &Document<SkinPack>| doc.diagnostics().iter().map(shown).collect::<Vec<_>>();
    let invalid = |tail: &str| {
        vec![(
            "InvalidValue",
            Severity::Error,
            format!("Trying to add an invalid classic skin.{tail}"),
        )]
    };
    // sk04: the texture is missing.
    let doc = load(&text, None, &["cape.png", "b.png"]);
    assert_eq!(message(&doc), invalid(" Skin asset is missing from a."));
    assert_eq!(names(&doc), ["b"]);
    assert_eq!(located(doc.diagnostics())[0].2, "skins/0");
    // A kept error: the game logs the line at level 3 and loads the next skin.
    assert!(doc.diagnostics().iter().all(|d| d.kept));
    // sk05: the cape is missing.
    let doc = load(&text, None, &["a.png", "b.png"]);
    assert_eq!(message(&doc), invalid(" Cape is missing from a."));
    assert_eq!(names(&doc), ["b"]);
    // sk05b: both, in one line.
    let doc = load(&text, None, &["b.png"]);
    assert_eq!(
        message(&doc),
        invalid(" Skin asset is missing from a. Cape is missing from a.")
    );
    // Everything there: both skins, no finding. `parse` alone holds both whatever the files.
    let doc = load(&text, None, &["a.png", "b.png", "cape.png"]);
    assert_eq!(
        (names(&doc), codes(doc.diagnostics())),
        (vec!["a".to_owned(), "b".to_owned()], vec![])
    );
    assert_eq!(names(&parse(&text)), ["a", "b"]);
    assert_eq!(codes(parse(&text).diagnostics()), []);
    // A path is matched exactly.
    let doc = load(&text, None, &["A.png", "b.png", "sub/cape.png"]);
    assert_eq!(names(&doc), ["b"]);
}

#[test]
fn a_skin_without_a_texture_member_and_an_empty_cape() {
    let text = pack(
        r#"{ "localization_name": "n", "geometry": "geometry.humanoid.custom", "cape": "" }, { "localization_name": true, "geometry": "geometry.humanoid.custom", "texture": 5 }"#,
    );
    let doc = load(&text, None, &["x.png"]);
    assert_eq!(
        doc.diagnostics()
            .iter()
            .map(|d| d.code.to_string())
            .collect::<Vec<_>>(),
        [
            "Trying to add an invalid classic skin. Skin asset is missing from n.",
            "Trying to add an invalid classic skin. Skin asset is missing from true.",
        ],
        "an absent texture is the empty path; an empty cape is no cape; a name is read as a string"
    );
}

#[test]
fn sk06_k12_custom_geometry() {
    let text = pack(&format!(
        "{}, {}",
        skin("").replace("geometry.humanoid.custom", "geometry.custom.x"),
        skin("").replace("geometry.humanoid.custom", "geometry.humanoid.customSlim")
    ));
    let geometry = r#"{ "format_version": "1.8.0", "geometry.custom.x": { "bones": [] } }"#;
    let doc = load(&text, Some(geometry), &["a.png"]);
    assert_eq!(codes(doc.diagnostics()), []);
    assert!(!doc.skins.as_ref().unwrap()[0].uses_built_in_geometry());
    for missing in [None, Some(r#"{ "geometry.custom.y": {} }"#), Some("[]")] {
        let doc = load(&text, missing, &["a.png"]);
        assert_eq!(
            located(doc.diagnostics()),
            [(
                "InvalidValue",
                Severity::Info,
                "skins/0/geometry".to_owned()
            )],
            "{missing:?}"
        );
        assert_eq!(doc.skins.as_ref().unwrap().len(), 2, "the skin stays");
    }
    // The built-in names are matched byte for byte.
    let doc = load(
        &pack(&skin("").replace("geometry.humanoid.custom", "geometry.humanoid.customslim")),
        None,
        &["a.png"],
    );
    assert_eq!(codes(doc.diagnostics()), [("InvalidValue", Severity::Info)]);
}

/// A `geometry.json` that is not JSON is named once, at the file; the pack loads without custom
/// geometry (no K-rule rejects it), and no skin is blamed for a name nobody could look up.
#[test]
fn a_geometry_file_that_is_not_json_is_named_once_and_no_skin_is_blamed() {
    let custom = |name: &str| skin("").replace("geometry.humanoid.custom", name);
    let text = pack(&format!(
        "{}, {}, {}",
        custom("geometry.custom.x"),
        custom("geometry.custom.y"),
        custom("geometry.humanoid.customSlim")
    ));
    let doc = load(&text, Some("{ nope"), &["a.png"]);
    let found: Vec<_> = doc.diagnostics().iter().collect();
    assert_eq!(
        codes(found.iter().copied()),
        [("MalformedInput", Severity::Info)]
    );
    assert_eq!(
        found[0].location.path.as_ref().unwrap().as_str(),
        "geometry.json"
    );
    assert_eq!(found[0].location.json_path.to_string(), "");
    assert_eq!(doc.skins.as_ref().unwrap().len(), 3, "every skin stays");
    // A file that reads and lacks the names is K12 for each of them, and only for them.
    let doc = load(&text, Some(r#"{ "geometry.custom.y": {} }"#), &["a.png"]);
    assert_eq!(
        located(doc.diagnostics()),
        [(
            "InvalidValue",
            Severity::Info,
            "skins/0/geometry".to_owned()
        )]
    );
}

/// K7, K8 and K12 are reported at the entry's place in the file, whatever the game skipped
/// before it.
#[test]
fn k7_k8_k12_are_reported_at_the_file_s_index() {
    let custom = skin("").replace("geometry.humanoid.custom", "geometry.custom.x");
    let text = pack(&format!(
        r#""x", {{ "localization_name": "n" }}, 4, {}, {custom}"#,
        skin(r#", "cape": "c.png""#)
    ));
    let doc = load(&text, None, &["a.png"]);
    assert_eq!(
        located(doc.diagnostics()),
        [
            ("InvalidType", Severity::Error, "skins/0".to_owned()),
            ("InvalidType", Severity::Error, "skins/2".to_owned()),
            ("InvalidValue", Severity::Error, "skins/1".to_owned()),
            ("InvalidValue", Severity::Error, "skins/3".to_owned()),
            (
                "InvalidValue",
                Severity::Info,
                "skins/4/geometry".to_owned()
            ),
        ]
    );
    // Of an object `skins`, the place is the member.
    let text = format!(
        r#"{{ "serialize_name": "p", "localization_name": "p", "skins": {{ "b": {{ "texture": "no.png" }}, "a": 3, "c": {custom} }} }}"#
    );
    let doc = load(&text, None, &["a.png"]);
    assert_eq!(
        located(doc.diagnostics()),
        [
            ("InvalidValue", Severity::Info, "skins".to_owned()),
            ("InvalidType", Severity::Error, "skins/a".to_owned()),
            ("InvalidValue", Severity::Error, "skins/b".to_owned()),
            (
                "InvalidValue",
                Severity::Info,
                "skins/c/geometry".to_owned()
            ),
        ]
    );
}

#[test]
fn sk07_a_user_s_pack_is_free() {
    let doc = parse(&pack(&format!(
        "{}, {}",
        skin(r#", "type": "paid""#),
        skin("")
    )));
    let skins = doc.skins.as_ref().unwrap();
    for origin in [PackOrigin::User, PackOrigin::Dev] {
        assert_eq!(skins[0].effective_type(origin), "free", "{origin:?}");
        assert_eq!(skins[1].effective_type(origin), "free", "{origin:?}");
    }
    for origin in [
        PackOrigin::Package,
        PackOrigin::World,
        PackOrigin::PremiumCache,
    ] {
        assert_eq!(skins[0].effective_type(origin), "paid", "{origin:?}");
        assert_eq!(skins[1].effective_type(origin), "", "{origin:?}");
    }
    assert_eq!(skins[0].type_.as_deref(), Some("paid"), "as written");
}

#[test]
fn k10_a_type_that_is_neither_free_nor_paid() {
    for (written, effective, reported) in [
        ("\"free\"", "free", false),
        ("\"paid\"", "paid", false),
        ("\"Free\"", "Free", true),
        ("\"\"", "", true),
        ("true", "true", true),
        ("3", "", true),
        ("null", "", true),
    ] {
        let doc = parse(&pack(&skin(&format!(r#", "type": {written}"#))));
        let skin = &doc.skins.as_ref().unwrap()[0];
        assert_eq!(
            skin.effective_type(PackOrigin::Package),
            effective,
            "{written}"
        );
        let want: &[_] = if reported {
            &[("InvalidValue", Severity::Info, "skins/0/type".to_owned())]
        } else {
            &[]
        };
        assert_eq!(located(doc.diagnostics()), want, "{written}");
    }
}

#[test]
fn sk08_k3_k4_k5_a_pack_the_game_rejects() {
    let refused = |skins: Option<&[u8]>| {
        SkinPack::load(skins, None, ["a.png"], &LoadContext::detached())
            .expect_err("rejected")
            .diagnostics
            .iter()
            .map(shown)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        refused(None),
        [(
            "MissingRequiredField",
            Severity::Error,
            "Failed to load skinpack, skins.json not found".to_owned()
        )]
    );
    assert_eq!(
        refused(Some(b"{ nope")),
        [(
            "MalformedInput",
            Severity::Error,
            "Failed to load skinpack, failed to parse manifest".to_owned()
        )]
    );
    for root in ["[]", "3", "null", "\"s\""] {
        assert_eq!(
            refused(Some(root.as_bytes())),
            [(
                "InvalidType",
                Severity::Error,
                "Skin manifest failed to load.  Pack Name:  /nAuthor: , /nUUID: ".to_owned()
            )],
            "{root}"
        );
    }
}

#[test]
fn sk09_the_optional_members_are_kept_as_written() {
    let members = r#", "animations": { "move.arms": "animation.x" }, "enable_attachables": false, "hide_armor": "yes", "held_item_ignores_lighting": [1], "tag": 7"#;
    let text = pack(&skin(members));
    let doc = parse(&text);
    let first = &doc.skins.as_ref().unwrap()[0];
    assert_eq!(
        first.animations,
        Some(json(r#"{ "move.arms": "animation.x" }"#))
    );
    assert_eq!(first.enable_attachables, Some(json("false")));
    assert_eq!(first.hide_armor, Some(json("\"yes\"")));
    assert_eq!(first.held_item_ignores_lighting, Some(json("[1]")));
    assert_eq!(
        located(doc.diagnostics()),
        [(
            "UnrecognizedField",
            Severity::Info,
            "skins/0/tag".to_owned()
        )]
    );
    assert_eq!(json(&doc.to_json().unwrap()), json(&text));
    let plain = parse(&pack(&skin("")));
    assert_eq!(plain.skins.as_ref().unwrap()[0].animations, None);
    assert_eq!(
        json(&plain.to_json().unwrap()),
        json(&pack(&skin(""))),
        "nothing is invented"
    );
}

#[test]
fn sk10_k14_a_skins_object_gives_its_member_values() {
    let text = format!(
        r#"{{ "serialize_name": "p", "localization_name": "p", "skins": {{ "y": {}, "x": {}, "w": 4 }} }}"#,
        skin("").replace("\"a\"", "\"second\""),
        skin("").replace("\"a\"", "\"first\"")
    );
    let doc = parse(&text);
    assert_eq!(names(&doc), ["first", "second"], "by key");
    assert_eq!(
        located(doc.diagnostics()),
        [
            ("InvalidValue", Severity::Info, "skins".to_owned()),
            ("InvalidType", Severity::Error, "skins/w".to_owned()),
        ]
    );
    // A scalar gives none.
    for scalar in ["5", "\"s\"", "null", "true"] {
        let doc = parse(&format!(
            r#"{{ "serialize_name": "p", "localization_name": "p", "skins": {scalar} }}"#
        ));
        assert_eq!(doc.skins, Some(Vec::new()), "{scalar}");
        assert_eq!(
            located(doc.diagnostics()),
            [("InvalidValue", Severity::Info, "skins".to_owned())],
            "{scalar}"
        );
    }
    assert_eq!(
        parse(r#"{ "serialize_name": "p", "localization_name": "p" }"#).skins,
        None
    );
}

#[test]
fn k9_a_path_too_long_for_the_game_s_buffer_reads_as_empty() {
    for (length, fits) in [(1023, true), (1024, false)] {
        let long = "p".repeat(length);
        let text = pack(&skin(&format!(r#", "cape": "{long}""#)).replace("a.png", &long));
        let doc = parse(&text);
        let skin = &doc.skins.as_ref().unwrap()[0];
        assert_eq!(skin.texture_path() == long.as_str(), fits, "{length}");
        assert_eq!(skin.cape_path().is_some(), fits, "{length}");
        let want: &[_] = if fits {
            &[]
        } else {
            &[
                ("InvalidValue", Severity::Info, "skins/0/texture".to_owned()),
                ("InvalidValue", Severity::Info, "skins/0/cape".to_owned()),
            ]
        };
        assert_eq!(located(doc.diagnostics()), want, "{length}");
        assert_eq!(
            json(&doc.to_json().unwrap()),
            json(&text),
            "written back as it was"
        );
    }
}

#[test]
fn k13_creator_rules() {
    let many = |count: usize| pack(&vec![skin(""); count].join(", "));
    assert_eq!(codes(parse(&many(80)).diagnostics()), []);
    assert_eq!(
        located(parse(&many(81)).diagnostics()),
        [("SkinPackCreatorRule", Severity::Info, "skins".to_owned())]
    );
    let png = |width: u32, height: u32| {
        let mut bytes = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR".to_vec();
        bytes.extend(width.to_be_bytes());
        bytes.extend(height.to_be_bytes());
        bytes
    };
    for (width, height, texture, cape) in [
        (64, 64, true, false),
        (128, 128, true, false),
        (64, 32, false, true),
        (32, 64, false, false),
        (128, 64, false, false),
        (256, 256, false, false),
    ] {
        let rule = |found: bedrock_addon::diagnostic::Diagnostics| {
            codes(&found) == [("SkinPackCreatorRule", Severity::Info)]
        };
        assert_eq!(
            !rule(check_texture(&png(width, height))),
            texture,
            "{width}x{height}"
        );
        assert_eq!(
            !rule(check_cape(&png(width, height))),
            cape,
            "{width}x{height}"
        );
    }
    assert_eq!(codes(&check_texture(b"not a png")), [], "not judged");
}

#[test]
fn string_members_of_another_type_are_read_as_the_game_reads_them_and_written_back() {
    let text = r#"{ "serialize_name": "p", "localization_name": true, "extra": 1, "skins": [{ "localization_name": false, "geometry": 3, "texture": null, "cape": true, "type": [] }] }"#;
    let doc = parse(text);
    assert_eq!(doc.name(), "true");
    let skin: &Skin = &doc.skins.as_ref().unwrap()[0];
    assert_eq!(
        (
            skin.name().as_ref(),
            skin.geometry_name().as_ref(),
            skin.texture_path().as_ref()
        ),
        ("false", "", "")
    );
    assert_eq!(skin.cape_path().as_deref(), Some("true"));
    assert_eq!(skin.localization_name, None);
    assert_eq!(json(&doc.to_json().unwrap()), json(text));
    assert_eq!(doc.extra.get("extra"), Some(&json("1")));
}

/// Write-back as authored: the review's probes are written back as the same JSON
/// values. A root that is no object is refused (K5), so there is nothing to write.
#[test]
fn skin_lists_are_written_as_authored() {
    for text in [
        "{}",
        r#"{"skins":[{"localization_name":"a","texture":"a.png"}]}"#,
        r#"{"serialize_name":5,"localization_name":true,"skins":[{"localization_name":true,"geometry":5,"texture":["x"],"type":null,"cape":{}}, "x", 4]}"#,
        r#"{"skins":{"b":{"texture":"b.png"},"a":{"texture":"a.png"}}}"#,
        r#"{"skins":5}"#,
        r#"{"skins":[]}"#,
    ] {
        let document = parse(text);
        let written = document.to_json().unwrap();
        assert_eq!(json(&written), json(text), "wrote {written}");
        assert_eq!(parse(&written).to_json().unwrap(), written, "{text}");
    }
    // A view that was changed writes itself; a file the fields say whole keeps nothing.
    let mut pack = parse(r#"{"skins":[{"texture":"a.png"}, 4]}"#).into_value();
    assert!(pack.written.is_some());
    pack.localization_name = Some("p".to_owned());
    assert_eq!(
        json(&pack.to_json().unwrap()),
        json(r#"{"localization_name":"p","skins":[{"texture":"a.png"}]}"#)
    );
    assert_eq!(parse(r#"{"skins":[{"texture":"a.png"}]}"#).written, None);
}

#[test]
fn generated_skin_lists_round_trip() {
    let values = [
        "\"s\"",
        "\"\"",
        "\"free\"",
        "3",
        "true",
        "null",
        "[]",
        "{}",
        "{\"k\": 1}",
        "[\"a\"]",
    ];
    let members = [
        "localization_name",
        "geometry",
        "texture",
        "type",
        "cape",
        "animations",
        "hide_armor",
        "other",
    ];
    let mut count = 0;
    for member in members {
        for value in values {
            for pack_name in values {
                for skins in [
                    "[{entry}]",
                    "{\"k\": {entry}}",
                    "[{entry}, 4, {entry}]",
                    "7",
                ] {
                    let entry = format!(r#"{{ "{member}": {value}, "texture": "t.png" }}"#);
                    let text = format!(
                        r#"{{ "localization_name": {pack_name}, "serialize_name": {value}, "skins": {} }}"#,
                        skins.replace("{entry}", &entry)
                    );
                    let first = parse(&text);
                    let written = first.to_json().unwrap();
                    // As authored: no generated document is rewritten.
                    assert_eq!(json(&written), json(&text), "{text}");
                    let second = parse(&written);
                    assert_eq!(first.value(), second.value(), "{text}");
                    assert_eq!(second.to_json().unwrap(), written, "{text}");
                    count += 1;
                }
            }
        }
    }
    assert!(count >= 3000, "{count} documents");
    println!("{count} generated documents written as authored");
}

/// A skin pack loads in time linear in its skins: the load is called directly, on texts built
/// before any clock starts.
#[test]
fn a_hundred_thousand_skins_load_in_linear_time() {
    let build = |entries: usize| {
        let skins: Vec<String> = (0..entries)
            .map(|i| format!(r#"{{ "localization_name": "s{i}", "geometry": "geometry.c{i}", "texture": "t{i}.png", "cape": "c{i}.png" }}"#))
            .collect();
        let text = pack(&skins.join(","));
        let models: Vec<String> = (0..entries)
            .map(|i| format!(r#""geometry.c{i}": {{}}"#))
            .collect();
        let geometry = format!("{{{}}}", models.join(","));
        let paths: Vec<String> = (0..entries)
            .flat_map(|i| [format!("t{i}.png"), format!("c{i}.png")])
            .collect();
        (text, geometry, paths, entries)
    };
    // 25 000 skins against a hundred thousand.
    crate::cost::assert_linear_at(
        "SkinPack::load",
        25_000,
        build,
        |(text, geometry, paths, entries)| {
            let doc = SkinPack::load(
                Some(text.as_bytes()),
                Some(geometry.as_bytes()),
                paths.iter().map(String::as_str),
                &LoadContext::detached(),
            )
            .unwrap();
            assert_eq!(doc.skins.as_ref().unwrap().len(), *entries);
            // K13 alone: more skins than creator tools allow.
            assert_eq!(doc.diagnostics().len(), 1);
        },
    );
}
