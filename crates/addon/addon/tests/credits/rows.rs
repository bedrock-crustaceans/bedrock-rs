//! `credits/credits.json` and the files a resource pack carries in vain.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::{Content, Document, Placed};
use bedrock_addon::families::credits::{Credits, check_starter_seeds};
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::storage::PackPath;

use super::common::{
    app_data, codes, in_vain, json, located, pack_number, shown, stack_of, stack_with,
    subpack_stacks,
};

const CREDITS: &str = r#"[
  { "section": "Mojang", "disciplines": [
    { "discipline": "Design", "titles": [
      { "title": "Game Designer", "names": ["A. Person", "B. Person"] } ] } ] },
  { "section": "Thanks" }
]"#;

fn parse(text: &str) -> Document<Credits> {
    Credits::parse(
        &PackPath::from_static("credits/credits.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .expect("a credits file")
}

fn texts(doc: &Document<Credits>) -> Vec<(&'static str, Severity, String)> {
    doc.diagnostics().iter().map(shown).collect()
}

#[test]
fn cr01_a_pack_s_credits_are_ignored_and_the_file_parses_on_its_own_deviation() {
    let stack = stack_of(&[&[("credits/credits.json", CREDITS)]]);
    let merged = stack.credits();
    assert_eq!(merged.value(), &Credits::default(), "no pack layer is read");
    assert!(merged.layers().is_empty());
    // The file is the pack's, carried in vain: the stack says so, not the credits.
    assert_eq!(codes(merged.diagnostics()), []);
    let vain = in_vain(&stack);
    let found: Vec<_> = vain.iter().collect();
    assert_eq!(
        codes(found.iter().copied()),
        [("IgnoredFile", Severity::Info)]
    );
    assert_eq!(
        found[0].location.path.as_ref().unwrap().as_str(),
        "credits/credits.json"
    );
    assert_eq!(pack_number(found[0], 1), Some(0));
    // The parsed model of the file.
    let doc = parse(CREDITS);
    assert_eq!(codes(doc.diagnostics()), []);
    let section = &doc.sections[0];
    assert_eq!(section.section.as_deref(), Some("Mojang"));
    let title = &section.disciplines.as_ref().unwrap()[0]
        .titles
        .as_ref()
        .unwrap()[0];
    assert_eq!(title.title.as_deref(), Some("Game Designer"));
    assert_eq!(title.names.as_ref().unwrap(), &["A. Person", "B. Person"]);
    assert_eq!(doc.sections[1].disciplines, None);
    assert_eq!(json(&doc.to_json().unwrap()), json(CREDITS));
}

#[test]
fn the_stack_s_credits_are_the_game_s_own_copy() {
    let options = app_data(&[("credits/credits.json", CREDITS), ("credits/end.txt", "x")]);
    let stack = stack_with(&[&[("credits/credits.json", b"[]")], &[]], options);
    let merged = stack.credits();
    assert_eq!(merged.sections.len(), 2);
    assert_eq!(merged.layers().len(), 1);
    assert!(merged.layers()[0].origin().pack.is_app_data());
    assert_eq!(codes(merged.diagnostics()), []);
    assert_eq!(
        codes(in_vain(&stack).iter()),
        [("IgnoredFile", Severity::Info)]
    );
    // Without the app data's copy: no credits, and no finding of a stack without the file.
    let stack = stack_with(&[&[]], LoadOptions::default());
    assert_eq!(stack.credits().value(), &Credits::default());
    assert_eq!(codes(stack.credits().diagnostics()), []);
    assert_eq!(codes(in_vain(&stack).iter()), []);
}

/// The game's nine checks are `handleAssert`s of a developer build (`CreditsRenderer`'s file, section,
/// discipline and title-group parsers), no content-log line: informations
/// of ours with the game's texts.
#[test]
fn the_game_s_own_checks_are_informations_on_the_app_data_copy() {
    let stack = stack_with(&[&[]], app_data(&[("credits/credits.json", "{}")]));
    assert_eq!(
        stack.credits().diagnostics().map(shown).collect::<Vec<_>>(),
        [(
            "InvalidType",
            Severity::Info,
            "The root must be an array".to_owned()
        )]
    );
    let stack = stack_with(&[&[]], app_data(&[("credits/credits.json", "")]));
    assert_eq!(
        stack.credits().diagnostics().map(shown).collect::<Vec<_>>(),
        [(
            "MalformedInput",
            Severity::Error,
            "File is not supposed to be empty".to_owned()
        )]
    );
}

#[test]
fn cr02_a_root_object() {
    let stack = stack_of(&[&[("credits/credits.json", "{}")]]);
    assert_eq!(
        codes(in_vain(&stack).iter()),
        [("IgnoredFile", Severity::Info)]
    );
    let doc = parse("{}");
    assert_eq!(
        texts(&doc),
        [(
            "InvalidType",
            Severity::Info,
            "The root must be an array".to_owned()
        )]
    );
    // No section; the root is kept as written beside them.
    assert_eq!(doc.sections, []);
    assert_eq!(doc.written, Some(json("{}")));
    assert_eq!(doc.to_json().unwrap().trim(), "{}");
}

#[test]
fn the_shape_checks_carry_the_game_s_texts() {
    let doc = parse(
        r#"[ {}, 3,
             { "section": "s", "disciplines": 5 },
             { "section": true, "disciplines": [ {}, { "discipline": "d", "titles": {} },
               { "titles": [ null, { "title": "t", "names": "n" }, { "title": 1, "names": ["a", 2, false] } ] } ] } ]"#,
    );
    let found: Vec<(String, String)> = doc
        .diagnostics()
        .iter()
        .map(|d| (d.location.json_path.to_string(), d.code.to_string()))
        .collect();
    let want = [
        ("0", "The section data should not be empty"),
        ("1", "The section data should not be empty"),
        ("2/disciplines", "\"disciplines\" must be an array"),
        ("3/disciplines/0", "The discipline data should not be empty"),
        ("3/disciplines/1/titles", "\"titles\" must be an array"),
        (
            "3/disciplines/2/titles/0",
            "The title object should not be empty",
        ),
        (
            "3/disciplines/2/titles/1/names",
            "\"names\" must be an array",
        ),
    ];
    assert_eq!(
        found,
        want.map(|(at, text)| (at.to_owned(), text.to_owned()))
    );
    assert!(
        doc.diagnostics()
            .iter()
            .all(|d| d.severity == Severity::Info)
    );
    assert_eq!(doc.sections.len(), 2);
    assert_eq!(doc.sections[1].section.as_deref(), Some("true"));
    let titles = doc.sections[1].disciplines.as_ref().unwrap()[1]
        .titles
        .as_ref()
        .unwrap();
    assert_eq!(titles[1].names.as_ref().unwrap(), &["a", "", "false"]);
    assert_eq!(titles[1].title.as_deref(), Some(""));
}

#[test]
fn an_empty_file_and_a_file_that_is_not_json_are_refused() {
    for (bytes, text) in [
        (&b""[..], "File is not supposed to be empty"),
        (&b"[ nope"[..], "Failed to load json file"),
    ] {
        let rejected = Credits::parse(
            &PackPath::from_static("credits/credits.json"),
            bytes,
            &LoadContext::detached(),
        )
        .unwrap_err();
        let found: Vec<_> = rejected.diagnostics.iter().map(shown).collect();
        assert_eq!(
            found,
            [("MalformedInput", Severity::Error, text.to_owned())]
        );
    }
}

#[test]
fn unknown_members_and_absent_members_are_written_back_as_they_were() {
    let text = r#"[ { "note": 1 }, { "section": "s", "disciplines": [ { "x": [] , "titles": [ { "names": [], "y": {} } ] } ] } ]"#;
    let doc = parse(text);
    assert_eq!(json(&doc.to_json().unwrap()), json(text));
    assert_eq!(Credits::default().to_json().unwrap().trim(), "[]");
    assert_eq!(
        Credits::default().default_path().as_str(),
        "credits/credits.json"
    );
}

#[test]
fn generated_credits_round_trip() {
    let values = [
        "\"t\"",
        "3",
        "null",
        "true",
        "[]",
        "{}",
        "[\"a\", 1]",
        "[{\"k\": 1}]",
    ];
    let mut count = 0;
    for section in values {
        for disciplines in values {
            for titles in values {
                for names in values {
                    let text = format!(
                        r#"[{{"section": {section}, "disciplines": {disciplines}}}, {{"e": 1, "disciplines": [{{"discipline": {section}, "titles": {titles}}}, {{"titles": [{{"title": {titles}, "names": {names}}}]}}]}}]"#
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
    assert!(count >= 4000, "{count} documents");
    println!("{count} generated documents written as authored");
}

#[test]
fn tb01_ap01_ap03_files_only_the_game_s_data_supplies_are_ignored_in_a_pack_deviation() {
    for name in [
        "texture_backwards_compatibility_mapping.json",
        "manifest_publish.json",
        "StarterSeeds.txt",
        "CreativeStarterSeeds.txt",
    ] {
        // Not JSON: the file is named, never parsed.
        let stack = stack_of(&[
            &[("other.txt", "")],
            &[(name, "{ not json")],
            &[("z.txt", "")],
        ]);
        let vain = in_vain(&stack);
        let found: Vec<_> = vain.iter().collect();
        assert_eq!(
            codes(found.iter().copied()),
            [("IgnoredFile", Severity::Info)],
            "{name}"
        );
        assert_eq!(found[0].location.path.as_ref().unwrap().as_str(), name);
        assert_eq!(pack_number(found[0], 3), Some(1), "{name}");
        // Only at the pack root, and only by the exact name.
        let nested = format!("sub/{name}");
        let near = format!("{name}.bak");
        let stack = stack_of(&[&[(nested.as_str(), ""), (near.as_str(), "")]]);
        assert_eq!(codes(in_vain(&stack).iter()), [], "{name}");
    }
}

#[test]
fn every_ignored_file_of_every_pack_is_named_lowest_pack_first() {
    let stack = stack_of(&[
        &[("StarterSeeds.txt", "1"), ("credits/credits.json", "[]")],
        &[("manifest_publish.json", "{}"), ("StarterSeeds.txt", "2")],
    ]);
    let found: Vec<(usize, String)> = in_vain(&stack)
        .iter()
        .map(|d| {
            (
                pack_number(d, 2).unwrap(),
                d.location.path.as_ref().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            (0, "credits/credits.json".to_owned()),
            (0, "StarterSeeds.txt".to_owned()),
            (1, "StarterSeeds.txt".to_owned()),
            (1, "manifest_publish.json".to_owned()),
        ]
    );
}

/// C1 to C5 name the file where it lies: a copy inside the active subpack is reported at the
/// subpack's path, not at the pack root.
#[test]
fn a_copy_in_the_active_subpack_is_reported_at_its_own_path() {
    let set = subpack_stacks(&[
        ("subpacks/hi/credits/credits.json", b"[]"),
        ("subpacks/hi/StarterSeeds.txt", b"1"),
        ("manifest_publish.json", b"{}"),
        ("subpacks/hi/persona/a.json", b"{}"),
    ]);
    let found: Vec<String> = in_vain(&set.resource)
        .iter()
        .map(|d| {
            assert_eq!(d.code.name(), "IgnoredFile");
            d.location.path.as_ref().unwrap().to_string()
        })
        .collect();
    assert_eq!(
        found,
        [
            "subpacks/hi/credits/credits.json",
            "subpacks/hi/StarterSeeds.txt",
            "manifest_publish.json",
            "subpacks/hi/persona/a.json",
        ]
    );
}

/// A blank line a client picks fails `LevelSettings::parseSeedString` and logs no line) and meets only the developer assert "Failed to parse the starter seed from file":
/// ours, an information.
#[test]
fn ap02_starter_seeds_on_their_own() {
    let found = check_starter_seeds(b"123\r\nabc\r\n\r\n-5");
    assert_eq!(
        located(&found),
        [("InvalidValue", Severity::Info, "2".to_owned())]
    );
    assert_eq!(
        check_starter_seeds(b"1\n2\n").len(),
        0,
        "a final line break adds no line"
    );
    assert_eq!(check_starter_seeds(b"1\n \t\n2\n\n").len(), 2);
    assert_eq!(
        check_starter_seeds(b"").len(),
        1,
        "an empty file is one blank seed"
    );
}

#[test]
fn pe01_a_persona_folder_in_a_resource_pack_is_not_read() {
    let stack = stack_of(&[&[
        ("persona/steve_geo_v5.json", "{}"),
        ("persona/a/b.png", ""),
        ("persona.json", "{}"),
        ("personal/x.json", "{}"),
    ]]);
    let found: Vec<String> = in_vain(&stack)
        .iter()
        .map(|d| {
            assert_eq!((d.code.name(), d.severity), ("IgnoredFile", Severity::Info));
            d.location.path.as_ref().unwrap().to_string()
        })
        .collect();
    assert_eq!(found, ["persona/a/b.png", "persona/steve_geo_v5.json"]);
}

// the client reads the starter seeds from its own (global) resource
// packs; only a pack that is not the game's own carries them in vain.
#[test]
fn c4_c5_a_base_game_pack_s_starter_seeds_are_the_game_s_own() {
    use bedrock_addon::pack::{OpenOptions, Pack, PackOrigin, PackStack};
    let files = |n: usize| {
        let mut all = vec![(
            PackPath::from_static("manifest.json"),
            bytes::Bytes::from(super::common::manifest(n, "resources", &[])),
        )];
        for name in [
            "StarterSeeds.txt",
            "CreativeStarterSeeds.txt",
            "manifest_publish.json",
        ] {
            all.push((
                PackPath::new(name).unwrap(),
                bytes::Bytes::from_static(b"1"),
            ));
        }
        all
    };
    let open = |n: usize, origin: PackOrigin| {
        let options = OpenOptions {
            origin,
            ..OpenOptions::default()
        };
        Pack::from_files(files(n), &options).expect("the pack opens")
    };
    // Top first: a world pack above a base-game pack.
    let packs = vec![open(1, PackOrigin::World), open(0, PackOrigin::Package)];
    let stack =
        PackStack::<bedrock_addon::pack::Resource>::of(packs, LoadOptions::default()).unwrap();
    let found: Vec<(usize, String)> = in_vain(&stack)
        .iter()
        .map(|d| {
            (
                pack_number(d, 2).unwrap(),
                d.location.path.as_ref().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            (0, "manifest_publish.json".to_owned()),
            (1, "CreativeStarterSeeds.txt".to_owned()),
            (1, "StarterSeeds.txt".to_owned()),
            (1, "manifest_publish.json".to_owned()),
        ],
        "the game's own pack: only the file nothing reads"
    );
}
