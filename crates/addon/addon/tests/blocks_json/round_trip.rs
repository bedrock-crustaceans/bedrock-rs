//! Round trips of the file type: vanilla's own files, files built by hand, and a few thousand
//! generated ones. A file writes back the members it holds, as it holds them, and what it
//! writes reads back to the same file and the same merged blocks.

use bedrock_addon::diagnostic::Diagnostics;
use bedrock_addon::document::{Content, Document};
use bedrock_addon::families::blocks_json::BlocksJson;
use bedrock_addon::json::{EncodeError, Value};
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

use super::common::{file, fixture, load_one};

/// Parses `text` as a user pack's `blocks.json`.
fn parse(text: &str) -> Result<Document<BlocksJson>, Diagnostics> {
    BlocksJson::parse(
        &PackPath::from_static("blocks.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .map_err(|rejected| rejected.diagnostics)
}

/// `(code, path)` of every finding.
fn codes(d: &Diagnostics) -> Vec<(&'static str, String)> {
    d.iter()
        .map(|f| (f.code.name(), f.location.json_path.to_string()))
        .collect()
}

/// Writes `document`, reads the text back and requires the same file, the same findings and an
/// identical second writing. The text.
#[track_caller]
fn round_trips(document: &Document<BlocksJson>) -> String {
    let written = document.to_json().unwrap_or_else(|e| panic!("{e}"));
    let read = parse(&written).unwrap_or_else(|d| panic!("written text is refused: {d:#?}"));
    assert_eq!(read.value(), document.value());
    assert_eq!(read.to_json().unwrap(), written, "a second writing differs");
    written
}

#[test]
fn vanilla_s_own_files_round_trip() {
    for (path, entries) in [
        ("bj01_vanilla_12650/packs/00_vanilla/blocks.json", 1352),
        ("bj02_vanilla_legacy/packs/00_vanilla/blocks.json", 0),
    ] {
        let text = fixture(path);
        let document = parse(&text).unwrap_or_else(|d| panic!("{path}: {d:#?}"));
        if entries != 0 {
            assert_eq!(document.entries.len(), entries, "{path}");
        }
        let written = round_trips(&document);
        // The merged blocks of the written text are those of the original.
        let (before, after) = (load_one(&text), load_one(&written));
        assert_eq!(before.registry().len(), after.registry().len(), "{path}");
        for (id, block) in before.registry() {
            assert_eq!(
                Some(block.value()),
                after.registry().get(id).map(Document::value),
                "{path}: {id:?}"
            );
        }
        // The findings too, but for a repeated key, which is written once.
        let repeated =
            |f: &&bedrock_addon::diagnostic::Diagnostic| f.code.name() != "DuplicateIdentifier";
        assert_eq!(
            before.registry().diagnostics().filter(repeated).count(),
            after.registry().diagnostics().count(),
            "{path}"
        );
    }
}

#[test]
fn a_file_writes_what_it_holds_and_nothing_else() {
    // Member order, an array version, unread members, values the client refuses: all as held.
    let text = r#"{
  "obsidian": {
    "sound": 5,
    "textures": {
      "up": "u",
      "down": "d",
      "side": "s"
    },
    "brightness_gamma": 0.7
  },
  "format_version": [
    1,
    1,
    0
  ],
  "x": 3,
  "wool": {
    "isotropic": {
      "side": null,
      "north": true
    }
  }
}
"#;
    let document = parse(text).unwrap();
    let written = round_trips(&document);
    // Everything is in file order, `format_version` included, untouched.
    assert_eq!(
        written,
        r#"{
  "obsidian": {
    "sound": 5,
    "textures": {
      "up": "u",
      "down": "d",
      "side": "s"
    },
    "brightness_gamma": 0.7
  },
  "format_version": [
    1,
    1,
    0
  ],
  "x": 3,
  "wool": {
    "isotropic": {
      "side": null,
      "north": true
    }
  }
}
"#
    );
    // A file without a version is written without one.
    let bare = parse(r#"{"dirt": {"textures": "d"}}"#).unwrap();
    assert_eq!(bare.format_version, None);
    assert!(!round_trips(&bare).contains("format_version"));
    // An empty file.
    let empty = parse("{}").unwrap();
    assert_eq!(round_trips(&empty), "{}\n");
}

#[test]
fn a_file_built_by_hand_round_trips() {
    let mut built = BlocksJson::new();
    built.entries.insert(
        "demo:lamp".to_owned(),
        bedrock_addon::json::from_str::<Value>(r#"{"textures": "lamp", "sound": "glass"}"#)
            .unwrap(),
    );
    let text = built.to_json().unwrap();
    assert_eq!(
        text,
        "{\n  \"format_version\": \"1.21.40\",\n  \"demo:lamp\": {\n    \"textures\": \"lamp\",\n    \"sound\": \"glass\"\n  }\n}\n"
    );
    let read = parse(&text).unwrap();
    assert_eq!(read.value(), &built);
    assert!(read.diagnostics().is_empty());
    assert_eq!(load_one(&text).block("demo:lamp").sound(), "glass");
    // Struct-update syntax works from outside the crate.
    let other = BlocksJson {
        format_version: None,
        ..built
    };
    assert_eq!(other.entries.len(), 1);
}

#[test]
fn format_version_is_written_where_the_author_put_it() {
    // The client's reader sorts the members, so the place of `format_version` means nothing
    // to it; the file holds the place and writes it back.
    for text in [
        r#"{"format_version": "1.21.40", "a": {}, "b": {}}"#,
        r#"{"a": {}, "format_version": "1.21.40", "b": {}}"#,
        r#"{"a": {}, "b": {}, "format_version": [1, 21, 40]}"#,
        r#"{"a": {}, "b": 3, "format_version": 7, "c": {}}"#,
    ] {
        let document = parse(text).unwrap();
        let written = round_trips(&document);
        assert_eq!(member_order(&written), member_order(text), "{text}");
        assert_eq!(
            bedrock_addon::json::from_str::<Value>(&written).unwrap(),
            bedrock_addon::json::from_str::<Value>(text).unwrap()
        );
    }
    // A repeated `format_version` is written once, where its last value stands, as every
    // repeated key is.
    let document =
        parse(r#"{"format_version": "1.1.0", "a": {}, "format_version": "1.21.40", "b": {}}"#)
            .unwrap();
    let written = round_trips(&document);
    assert_eq!(
        member_order(&written),
        member_order(r#"{"a": {}, "format_version": "1.21.40", "b": {}}"#)
    );
    assert_eq!(document.effective_version().to_string(), "1.21.40");
    // A place beyond the entries is the end; a file without a version has no place to keep.
    let mut built = BlocksJson::new();
    built.entries.insert("a".to_owned(), Value::from(1));
    built.format_version_index = 9;
    let written = built.to_json().unwrap();
    assert_eq!(
        member_order(&written),
        member_order(r#"{"a": 1, "format_version": "1.21.40"}"#)
    );
    assert_eq!(parse(&written).unwrap().format_version_index, 1);
    assert_eq!(
        parse(r#"{"a": {}, "b": {}}"#).unwrap().format_version_index,
        0
    );
}

#[test]
fn two_files_are_equal_when_the_version_is_written_at_the_same_place() {
    // `format_version_index` is compared as the place it writes: a number beyond the entries
    // is the end, and without a `format_version` it is no place at all. A file built by hand
    // therefore equals what its own text reads back to.
    let mut built = BlocksJson::new();
    built.entries.insert("a".to_owned(), Value::from(1));
    built.format_version_index = 9;
    let read = parse(&built.to_json().unwrap()).unwrap();
    assert_eq!(read.format_version_index, 1);
    assert_eq!(read.value(), &built);
    let at_the_end = BlocksJson {
        format_version_index: 1,
        ..built.clone()
    };
    assert_eq!(at_the_end, built);
    // Inside the entries every place is its own.
    let first = BlocksJson {
        format_version_index: 0,
        ..built.clone()
    };
    assert_ne!(first, built);
    assert_ne!(first, at_the_end);
    // Without a version the number says nothing.
    let unversioned = |index: usize| BlocksJson {
        format_version: None,
        format_version_index: index,
        ..built.clone()
    };
    assert_eq!(unversioned(0), unversioned(1));
    assert_eq!(unversioned(0), unversioned(9));
    let read = parse(&unversioned(9).to_json().unwrap()).unwrap();
    assert_eq!(read.value(), &unversioned(9));
    // The version and the entries are compared as before.
    assert_ne!(unversioned(0), first);
    let mut other = first.clone();
    other.entries.insert("b".to_owned(), Value::from(2));
    assert_ne!(other, first);
}

#[test]
fn an_entry_named_format_version_is_not_written() {
    // It would read back as the file's version, not as an entry.
    let mut built = BlocksJson::new();
    built
        .entries
        .insert("format_version".to_owned(), Value::from("1.1.0"));
    match built.to_json() {
        Err(EncodeError::NotRepresentable { at, .. }) => {
            assert_eq!(at.to_string(), "format_version")
        }
        other => panic!("{other:?}"),
    }
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

/// One generated file.
struct Generated {
    /// The file's text.
    text: String,
    /// The text without the earlier copy of every member an entry repeats: what the client's
    /// reader keeps of it (the last value, where it stands). `text` itself when
    /// no entry repeats a member.
    authored: String,
    /// Whether an entry repeats a member.
    repeats_a_member: bool,
}

/// One generated file: each member in each of its forms, sound and unsound.
fn generate(g: &mut Generator) -> Generated {
    let mut members = Vec::new();
    // The same members without the earlier copy of a repeated one.
    let mut authored = Vec::new();
    let mut repeats_a_member = false;
    let version = g.pick(&[
        "",
        r#""1.21.40""#,
        r#""1.21.40""#,
        "[1, 1, 0]",
        r#""1.26.0""#,
        r#""1.21""#,
        r#""*""#,
        "7",
        r#""beta""#,
        r#""1.20.50""#,
    ]);
    for index in 0..g.next(6) {
        let key = g.pick(&[
            "obsidian",
            "wool",
            "Wool",
            "stone",
            "minecraft:wool",
            "demo:lamp",
            "chalkboard",
            "lava_cauldron",
            "red_mushroom_block",
            "not_a_block",
            "skull",
            "tallgrass",
        ]);
        if g.next(10) == 0 {
            members.push(format!(
                r#""{key}{index}": {}"#,
                g.pick(&["3", "null", "[]", r#""x""#])
            ));
            authored.push(members.last().cloned().unwrap());
            continue;
        }
        let mut fields = Vec::new();
        let textures = [
            r#""t""#,
            r#"{"up": "u", "down": "d", "side": "s"}"#,
            r#"{"up": "u", "down": "d", "north": "n", "south": "s", "west": "w", "east": "e"}"#,
            r#"{"up": "u", "down": 5, "side": "s"}"#,
            "5",
            "null",
            r#"["t"]"#,
            "18446744073709551616",
        ];
        if g.next(3) != 0 {
            fields.push(format!(r#""textures": {}"#, g.pick(&textures)));
        }
        if g.next(4) == 0 {
            fields.push(format!(r#""carried_textures": {}"#, g.pick(&textures)));
        }
        if g.next(3) == 0 {
            fields.push(format!(
                r#""isotropic": {}"#,
                g.pick(&[
                    "true",
                    "false",
                    r#"{"up": true, "down": true}"#,
                    r#"{"side": true, "north": true}"#,
                    r#"{"side": null, "north": true}"#,
                    r#""yes""#,
                    "null",
                ])
            ));
        }
        if g.next(4) == 0 {
            fields.push(format!(
                r#""blockshape": {}"#,
                g.pick(&[r#""tree""#, r#""cross_texture""#, r#""nope""#, "null", "5"])
            ));
        }
        if g.next(3) == 0 {
            fields.push(format!(
                r#""ambient_occlusion_exponent": {}"#,
                g.pick(&[
                    "0.5",
                    "2",
                    "true",
                    r#""high""#,
                    "1e39",
                    "-1e39",
                    "null",
                    "1e999",
                    "-1e999",
                    "-18446744073709551616",
                ])
            ));
        }
        if g.next(2) == 0 {
            fields.push(format!(
                r#""sound": {}"#,
                g.pick(&[
                    r#""stone""#,
                    r#""""#,
                    "5",
                    "false",
                    "null",
                    "[]",
                    "18446744073709551616"
                ])
            ));
        }
        if g.next(6) == 0 {
            fields.push(format!(
                r#""{}": {}"#,
                g.pick(&["brightness_gamma", "sounds", "client_entity", "extra"]),
                g.pick(&[
                    "1",
                    r#""x""#,
                    r#"{"a": [1, 2.5, null]}"#,
                    "1e999",
                    r#"[{"b": -1e999}]"#,
                    "123456789012345678901234567890",
                    r#"{"a": [18446744073709551616]}"#,
                ])
            ));
        }
        // A member may come twice in one entry: the earlier copy is anywhere before the
        // last one, of any form.
        let kept = fields.clone();
        if !fields.is_empty() && g.next(8) == 0 {
            let last = g.next(fields.len());
            let name = fields[last].split(':').next().unwrap().to_owned();
            let earlier = g.pick(&[r#""earlier""#, "5", "null", r#"{"up": "u"}"#, "true"]);
            fields.insert(g.next(last + 1), format!("{name}: {earlier}"));
            repeats_a_member = true;
        }
        // The same key may come twice in one file.
        let suffix = if g.next(5) == 0 {
            String::new()
        } else {
            index.to_string()
        };
        let as_written = key.contains(':') || suffix.is_empty() || g.next(2) == 0;
        let name = if as_written {
            key.to_owned()
        } else {
            format!("demo:{key}{suffix}")
        };
        members.push(format!(r#""{name}": {{{}}}"#, fields.join(", ")));
        authored.push(format!(r#""{name}": {{{}}}"#, kept.join(", ")));
    }
    // The version stands first, as vanilla's files have it, or anywhere among the entries.
    if !version.is_empty() {
        let at = if g.next(2) == 0 {
            0
        } else {
            g.next(members.len() + 1)
        };
        members.insert(at, format!(r#""format_version": {version}"#));
        authored.insert(at, format!(r#""format_version": {version}"#));
    }
    Generated {
        text: format!("{{{}}}", members.join(", ")),
        authored: format!("{{{}}}", authored.join(", ")),
        repeats_a_member,
    }
}

/// Every number token of `text` (outside its strings), in order, as the double it spells.
fn number_tokens(text: &str) -> Vec<f64> {
    let mut numbers = Vec::new();
    let mut token = String::new();
    let (mut in_string, mut escaped) = (false, false);
    for c in text.chars().chain(std::iter::once(' ')) {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        if c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E') {
            token.push(c);
            continue;
        }
        if token.starts_with(|first: char| first == '-' || first.is_ascii_digit()) {
            numbers.push(token.parse().unwrap_or_else(|_| panic!("{token}")));
        }
        token.clear();
        in_string = c == '"';
    }
    numbers
}

/// The root keys of a file and, per entry, its member names, in the order of the text.
fn member_order(text: &str) -> Vec<(String, Vec<String>)> {
    let root = bedrock_addon::json::from_str::<Value>(text).unwrap();
    root.as_object()
        .unwrap()
        .iter()
        .map(|(key, entry)| {
            let members = entry
                .as_object()
                .map(|entry| {
                    entry
                        .iter()
                        .map(|(name, _)| name.as_str().to_owned())
                        .collect()
                })
                .unwrap_or_default();
            (key.as_str().to_owned(), members)
        })
        .collect()
}

#[test]
fn four_thousand_generated_files_round_trip() {
    let mut g = Generator(0x000B_10C5);
    let mut warned = 0;
    let (mut as_authored, mut repeated, mut clamped) = (0, 0, 0);
    let (mut version_not_first, mut repeated_member, mut wide) = (0, 0, 0);
    for round in 0..4000 {
        let Generated {
            text,
            authored,
            repeats_a_member,
        } = generate(&mut g);
        assert_eq!(authored != text, repeats_a_member, "{text}");
        let document = parse(&text).unwrap_or_else(|d| panic!("{text}\n{d:#?}"));
        if !document.diagnostics().is_empty() {
            warned += 1;
        }
        // Nothing the generator writes is refused: a number beyond a double is held finite.
        let written = document
            .to_json()
            .unwrap_or_else(|error| panic!("{error}: {text}"));
        // Write-back as authored: a file no finding says was rewritten is written member for
        // member as it was read. The recorded rewrites: a repeated key is written once, with
        // its last value (`DuplicateIdentifier`); a number beyond a double is written
        // finite (`ClampedValue`); and a member an entry repeats is written once,
        // where its last value stands, without a finding: `authored` is the file
        // without the earlier copies.
        let rewrites: Vec<&'static str> = codes(document.diagnostics())
            .into_iter()
            .map(|(code, _)| code)
            .filter(|code| matches!(*code, "DuplicateIdentifier" | "ClampedValue"))
            .collect();
        let value = |text: &str| bedrock_addon::json::from_str::<Value>(text).unwrap();
        if rewrites.is_empty() {
            if repeats_a_member {
                repeated_member += 1;
            } else {
                as_authored += 1;
            }
            assert_eq!(value(&written), value(&authored), "{text}\n{written}");
            assert_eq!(
                member_order(&written),
                member_order(&authored),
                "{text}\n{written}"
            );
            // A number is written as a number: an integer beyond 64 bits too, which is a
            // number token like any other and never a string.
            assert_eq!(
                number_tokens(&written),
                number_tokens(&authored),
                "{text}\n{written}"
            );
            wide += usize::from(
                authored.contains("18446744073709551616")
                    || authored.contains("123456789012345678901234567890"),
            );
            // `format_version` is a root member like any other: it is written where the
            // author put it.
            let authored = member_order(&authored);
            if authored
                .iter()
                .position(|(key, _)| key == "format_version")
                .is_some_and(|at| at > 0)
            {
                version_not_first += 1;
            }
        } else {
            repeated += usize::from(rewrites.contains(&"DuplicateIdentifier"));
            clamped += usize::from(rewrites.contains(&"ClampedValue"));
        }
        let read = parse(&written).unwrap_or_else(|d| panic!("{text}\n{written}\n{d:#?}"));
        assert_eq!(read.value(), document.value(), "{text}");
        assert_eq!(read.to_json().unwrap(), written, "{text}");
        // The same findings, but for the repeated keys, which are written once, and the
        // numbers beyond a double, which are written finite.
        let kept = |d: &Diagnostics| -> Vec<(&'static str, String)> {
            codes(d)
                .into_iter()
                .filter(|(code, _)| !matches!(*code, "DuplicateIdentifier" | "ClampedValue"))
                .collect()
        };
        assert_eq!(
            kept(read.diagnostics()),
            kept(document.diagnostics()),
            "{text}"
        );
        // And, for every fourth file, the same merged blocks.
        if round % 4 != 0 {
            continue;
        }
        let (before, after) = (load_one(&text), load_one(&written));
        assert_eq!(before.registry().len(), after.registry().len(), "{text}");
        for (id, block) in before.registry() {
            assert_eq!(
                Some(block.value()),
                after.registry().get(id).map(Document::value),
                "{text}"
            );
        }
    }
    assert!(warned > 1000, "the generator makes findings: {warned}");
    eprintln!(
        "generated: {as_authored} written as authored ({version_not_first} with the version not first), {repeated} with a repeated key, {repeated_member} with a member repeated inside an entry and no other rewrite, {clamped} with a clamped number, {wide} with an integer beyond 64 bits written as authored"
    );
    assert!(
        as_authored > 1000,
        "files written as authored: {as_authored}"
    );
    assert!(repeated > 100, "files with a repeated key: {repeated}");
    assert!(
        repeated_member > 100,
        "files with a member repeated inside an entry: {repeated_member}"
    );
    assert!(
        version_not_first > 100,
        "files written as authored whose version is not the first member: {version_not_first}"
    );
    assert!(
        clamped > 100,
        "files with a number beyond a double: {clamped}"
    );
    assert!(
        wide > 100,
        "files written as authored with an integer beyond 64 bits: {wide}"
    );
}

#[test]
fn an_integer_beyond_64_bits_is_a_number_wherever_it_stands() {
    // 2^64 and its like are number tokens: the client reads them as the numbers they are
    // (jsoncpp makes a double of an integer that fits no 64-bit type), and the file writes a
    // number back, never a string.
    const WIDE: &str = "18446744073709551616";
    let found_at = |loaded: &super::common::Loaded, at: &str| -> Vec<(&'static str, String)> {
        loaded
            .found()
            .into_iter()
            .filter(|f| f.at == at)
            .map(|f| (f.code, f.text))
            .collect()
    };
    let written = |text: &str| {
        let document = parse(text).unwrap_or_else(|d| panic!("{text}: {d:#?}"));
        let written = round_trips(&document);
        assert_eq!(number_tokens(&written), number_tokens(text), "{written}");
        assert!(!written.contains(&format!("\"{WIDE}\"")), "{written}");
        written
    };
    // `sound`: a number is no sound name, as `"sound": 5` is not.
    let text = file(&format!(r#""obsidian": {{"sound": {WIDE}}}"#));
    let loaded = load_one(&text);
    assert_eq!(
        found_at(&loaded, "obsidian/sound"),
        [("InvalidType", String::new())]
    );
    assert_eq!(loaded.block("obsidian").sound.as_deref(), Some(""));
    written(&text);
    // `textures`: refused with vanilla's warning, as `"textures": 5` is; never a
    // texture name.
    let text = file(&format!(
        r#""obsidian": {{"textures": {WIDE}, "carried_textures": {WIDE}}}"#
    ));
    let loaded = load_one(&text);
    assert_eq!(
        found_at(&loaded, "obsidian/textures"),
        [(
            "InvalidType",
            "'minecraft:obsidian' | Invalid or incomplete texture list.".to_owned()
        )]
    );
    assert_eq!(loaded.block("obsidian").textures, None);
    assert_eq!(loaded.block("obsidian").carried_textures, None);
    assert_eq!(found_at(&loaded, "obsidian/carried_textures").len(), 1);
    written(&text);
    // A face of a texture object.
    let text = file(&format!(
        r#""obsidian": {{"textures": {{"up": "u", "down": "d", "side": {WIDE}}}}}"#
    ));
    let loaded = load_one(&text);
    assert_eq!(loaded.block("obsidian").textures, None);
    assert_eq!(found_at(&loaded, "obsidian/textures").len(), 1);
    written(&text);
    // `ambient_occlusion_exponent`: a number, read as the nearest float; no finding.
    for (number, read) in [
        (WIDE.to_owned(), 18_446_744_073_709_551_616.0_f32),
        (format!("-{WIDE}"), -18_446_744_073_709_551_616.0_f32),
    ] {
        let text = file(&format!(
            r#""obsidian": {{"ambient_occlusion_exponent": {number}}}"#
        ));
        let loaded = load_one(&text);
        assert_eq!(
            found_at(&loaded, "obsidian/ambient_occlusion_exponent"),
            [],
            "{number}"
        );
        assert_eq!(
            loaded.block("obsidian").ambient_occlusion_exponent,
            Some(read),
            "{number}"
        );
        written(&text);
    }
    // A member the client never reads, and a number inside one: held and written as numbers.
    let text = file(&format!(
        r#""obsidian": {{"brightness_gamma": {WIDE}, "client_entity": {{"a": [1, 123456789012345678901234567890]}}}}"#
    ));
    let loaded = load_one(&text);
    let extra = &loaded.block("obsidian").extra;
    let gamma = extra
        .iter()
        .find(|(name, _)| *name == "brightness_gamma")
        .map(|(_, value)| value.clone())
        .expect("the unread member is kept");
    assert!(gamma.is_number(), "{gamma:?}");
    assert!(!gamma.is_string(), "{gamma:?}");
    written(&text);
    // `blockshape` and `isotropic`: a number is no shape name and no face list.
    let text = file(&format!(
        r#""obsidian": {{"blockshape": {WIDE}, "isotropic": {WIDE}}}"#
    ));
    let loaded = load_one(&text);
    assert_eq!(loaded.block("obsidian").blockshape, None);
    assert_eq!(loaded.block("obsidian").isotropic, None);
    written(&text);
    // An entry that is a number is no entry.
    let text = file(&format!(r#""obsidian": {WIDE}"#));
    let loaded = load_one(&text);
    assert!(loaded.get("obsidian").is_none());
    written(&text);
}

#[test]
fn a_number_beyond_f32_is_held_as_written_and_read_as_the_largest_float() {
    // The file keeps `1e39`; the client-side view is the largest float.
    let text = file(r#""obsidian": {"ambient_occlusion_exponent": 1e39}"#);
    let document = parse(&text).unwrap();
    let written = round_trips(&document);
    assert_eq!(
        load_one(&written)
            .block("obsidian")
            .ambient_occlusion_exponent,
        Some(f32::MAX)
    );
}

#[test]
fn a_number_beyond_a_double_is_held_as_the_largest_finite_one_and_written_back_deviation() {
    // `1e999` is no double: jsoncpp reads an infinity, which no JSON text holds. The file
    // holds the largest finite double of that sign wherever the number stands, with a
    // warning, so that what is written reads back equal.
    for (text, at) in [
        (
            r#"{"dirt":{"ambient_occlusion_exponent":1e999}}"#,
            "dirt/ambient_occlusion_exponent",
        ),
        (
            r#"{"dirt":{"ambient_occlusion_exponent":-1e999}}"#,
            "dirt/ambient_occlusion_exponent",
        ),
        (
            r#"{"dirt":{"brightness_gamma":1e999}}"#,
            "dirt/brightness_gamma",
        ),
        (
            r#"{"dirt":{"client_entity":{"a":[1,-1e999]}}}"#,
            "dirt/client_entity/a/1",
        ),
        (r#"{"dirt":{"textures":1e999}}"#, "dirt/textures"),
        (r#"{"dirt":{"sound":[1e999]}}"#, "dirt/sound/0"),
        (r#"{"format_version":1e999}"#, "format_version"),
        (r#"{"dirt":1e999}"#, "dirt"),
    ] {
        let document = parse(text).unwrap_or_else(|d| panic!("{text}: {d:#?}"));
        let clamped: Vec<_> = codes(document.diagnostics())
            .into_iter()
            .filter(|(code, path)| *code == "ClampedValue" && path == at)
            .collect();
        assert_eq!(clamped.len(), 1, "{text}: {:#?}", document.diagnostics());
        let written = round_trips(&document);
        let expected = text
            .replace("-1e999", "-1.7976931348623157e308")
            .replace("1e999", "1.7976931348623157e308");
        assert_eq!(
            bedrock_addon::json::from_str::<Value>(&written).unwrap(),
            bedrock_addon::json::from_str::<Value>(&expected).unwrap(),
            "{text}"
        );
    }
    // The client-side view of the exponent is the largest float of that sign, with the
    // reader's own warning beside the clamp's.
    for (number, read) in [("1e999", f32::MAX), ("-1e999", f32::MIN)] {
        let text = format!(r#"{{"dirt":{{"ambient_occlusion_exponent":{number}}}}}"#);
        let loaded = load_one(&text);
        let at_member: Vec<&str> = loaded
            .found()
            .iter()
            .filter(|f| f.at == "dirt/ambient_occlusion_exponent")
            .map(|f| f.code)
            .collect();
        assert_eq!(at_member, ["ClampedValue", "InvalidValue"]);
        assert_eq!(loaded.block("dirt").ambient_occlusion_exponent, Some(read));
        let written = parse(&text).unwrap().to_json().unwrap();
        assert_eq!(
            load_one(&written).block("dirt").ambient_occlusion_exponent,
            Some(read)
        );
    }
}

#[test]
fn an_integer_beyond_a_double_is_held_as_the_largest_finite_one_deviation() {
    // An integer token is a number however long it is. One of 310 digits is beyond
    // every double, as `1e999` is, and takes the same way: the largest finite double of its
    // sign, one `ClampedValue` at its path, and that number written back.
    let beyond = format!("1{}", "0".repeat(309));
    for (text, at, expected) in [
        (
            format!(r#"{{"dirt":{{"brightness_gamma":{beyond}}}}}"#),
            "dirt/brightness_gamma",
            r#"{"dirt":{"brightness_gamma":1.7976931348623157e308}}"#,
        ),
        (
            format!(r#"{{"dirt":{{"client_entity":{{"a":[1,-{beyond}]}}}}}}"#),
            "dirt/client_entity/a/1",
            r#"{"dirt":{"client_entity":{"a":[1,-1.7976931348623157e308]}}}"#,
        ),
        (
            format!(r#"{{"dirt":{{"sound":{beyond}}}}}"#),
            "dirt/sound",
            r#"{"dirt":{"sound":1.7976931348623157e308}}"#,
        ),
        (
            format!(r#"{{"format_version":{beyond}}}"#),
            "format_version",
            r#"{"format_version":1.7976931348623157e308}"#,
        ),
        (
            format!(r#"{{"dirt":-{beyond}}}"#),
            "dirt",
            r#"{"dirt":-1.7976931348623157e308}"#,
        ),
    ] {
        let document = parse(&text).unwrap_or_else(|d| panic!("{text}: {d:#?}"));
        let clamped: Vec<_> = codes(document.diagnostics())
            .into_iter()
            .filter(|(code, _)| *code == "ClampedValue")
            .collect();
        assert_eq!(clamped, [("ClampedValue", at.to_owned())], "{text}");
        let written = round_trips(&document);
        assert_eq!(
            bedrock_addon::json::from_str::<Value>(&written).unwrap(),
            bedrock_addon::json::from_str::<Value>(expected).unwrap(),
            "{text}"
        );
        // Never the digits, as a number or as a string.
        assert!(!written.contains(&beyond), "{written}");
    }
    // The other side of the bound: 309 digits spell the double 1e308, which is held and
    // written as the number it is: nothing is clamped.
    let within = format!("1{}", "0".repeat(308));
    let text = format!(r#"{{"dirt":{{"brightness_gamma":{within}}}}}"#);
    let document = parse(&text).unwrap();
    assert!(
        codes(document.diagnostics())
            .iter()
            .all(|(code, _)| *code != "ClampedValue"),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(
        bedrock_addon::json::from_str::<Value>(&round_trips(&document)).unwrap(),
        bedrock_addon::json::from_str::<Value>(r#"{"dirt":{"brightness_gamma":1e308}}"#).unwrap()
    );
    // The client-side view of the exponent is the largest float of the sign, with the
    // reader's own warning beside the clamp's, on the first read and on the read of
    // the written text.
    for (number, read) in [(beyond.clone(), f32::MAX), (format!("-{beyond}"), f32::MIN)] {
        let text = format!(r#"{{"dirt":{{"ambient_occlusion_exponent":{number}}}}}"#);
        let loaded = load_one(&text);
        let at_member: Vec<&str> = loaded
            .found()
            .iter()
            .filter(|f| f.at == "dirt/ambient_occlusion_exponent")
            .map(|f| f.code)
            .collect();
        assert_eq!(at_member, ["ClampedValue", "InvalidValue"]);
        assert_eq!(loaded.block("dirt").ambient_occlusion_exponent, Some(read));
        let written = parse(&text).unwrap().to_json().unwrap();
        assert_eq!(
            load_one(&written).block("dirt").ambient_occlusion_exponent,
            Some(read)
        );
    }
}

#[test]
fn a_repeated_key_is_written_once_where_its_last_value_stands() {
    // The recorded rewrite: the earlier values are gone, with a warning each.
    let text = r#"{"a": {"sound": "x", "textures": 1e999}, "b": {}, "a": {"sound": "y"}, "c": {}, "a": {"sound": "z"}}"#;
    let document = parse(text).unwrap();
    assert_eq!(
        codes(document.diagnostics())
            .iter()
            .filter(|(code, _)| *code == "DuplicateIdentifier")
            .count(),
        2
    );
    let written = round_trips(&document);
    assert_eq!(
        member_order(&written),
        member_order(r#"{"b": {}, "c": {}, "a": {"sound": "z"}}"#)
    );
    assert_eq!(
        bedrock_addon::json::from_str::<Value>(&written).unwrap(),
        bedrock_addon::json::from_str::<Value>(r#"{"b": {}, "c": {}, "a": {"sound": "z"}}"#)
            .unwrap()
    );
}

#[test]
fn a_member_repeated_inside_an_entry_is_written_once_where_its_last_value_stands() {
    // The client's reader keeps the last value of a member written twice, at every level;
    // the file holds what the reader keeps. No finding says so: the count is of the root's
    // keys.
    let text = r#"{"a": {"sound": "x", "textures": "t", "sound": "y", "extra": {"k": 1, "k": 2}}}"#;
    let kept = r#"{"a": {"textures": "t", "sound": "y", "extra": {"k": 2}}}"#;
    let document = parse(text).unwrap();
    assert_eq!(
        codes(document.diagnostics()),
        [
            ("MissingFormatVersion", String::new()),
            ("IgnoredMember", "a/extra".to_owned())
        ]
    );
    let written = round_trips(&document);
    assert_eq!(member_order(&written), member_order(kept));
    assert_eq!(
        bedrock_addon::json::from_str::<Value>(&written).unwrap(),
        bedrock_addon::json::from_str::<Value>(kept).unwrap()
    );
    assert_eq!(load_one(text).block("a").sound(), "y");
}

#[test]
fn a_key_is_written_as_the_author_wrote_it() {
    // The client lower-cases a bare key and resolves a legacy name; the file holds, and the
    // registry's origin names, the key as written.
    let text = r#"{"format_version": "1.21.40", "seaLantern": {"sound": "glass"}, "Minecraft:Stone": {"sound": "stone"}, "WOOL ": {}, "demo:Lamp": {"textures": "lamp"}}"#;
    let document = parse(text).unwrap();
    let written = round_trips(&document);
    assert_eq!(member_order(&written), member_order(text));
    assert_eq!(
        bedrock_addon::json::from_str::<Value>(&written).unwrap(),
        bedrock_addon::json::from_str::<Value>(text).unwrap()
    );
    let loaded = load_one(text);
    let lantern = loaded.block("minecraft:sealantern");
    assert_eq!(lantern.origin().key.as_deref(), Some("seaLantern"));
    assert_eq!(
        loaded.block("minecraft:stone").origin().key.as_deref(),
        Some("Minecraft:Stone")
    );
    assert_eq!(
        loaded.block("demo:Lamp").origin().key.as_deref(),
        Some("demo:Lamp")
    );
}
