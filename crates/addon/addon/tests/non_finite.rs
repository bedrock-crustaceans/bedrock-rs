#![cfg(feature = "full")]
//! A document that loads can be written: a number no double
//! holds (`1e999`, `-1e999`, an integer of 400 digits) never loads as an infinity that
//! `to_json` then refuses with `NotRepresentable`.
//!
//! One test per document type. Each takes a small document its family loads and puts the
//! number, one place at a time, into **every object of that document**, as an unknown member
//! (`"zz_probe": N`), two levels down inside an unknown member (`"zz_probe": {"deep": {"n":
//! N}}`) and as an element of an array inside one (`"zz_probe": [0, N]`). The documents hold
//! the members their family keeps as JSON on purpose (events, components kept as written,
//! `attribs`, …), so those objects are probed with the rest. For each text one of these holds:
//!
//! * **refused**: the load yields no document;
//! * **not held**: the document loads and the number is not part of it (the member is not
//!   kept, whatever it holds: the document and its findings are those of `0` there): what is
//!   written is the same for the number and for its twin of the other sign,
//!   and it reads back to the same value;
//! * **as written**: the family keeps the author's token and writes it back;
//! * **not taken**: the same, but the document differs from the one with `0` at that place:
//!   then a finding at the number's own path says so;
//! * **clamped**: the document loads, `to_json` is `Ok`, the written text reads back to the
//!   same value (see below), and a finding of the family's clamp code (`InvalidValue`, or
//!   `ClampedValue` for `blocks.json`) is at the number's own path;
//! * **told by the member**: the same, but the finding at the number's own path is one the
//!   document with `0` there has too, and it is about the value (not about a member name
//!   the family does not know): the member is of another type than a number, the family
//!   says so and keeps what is written. A number is told once.
//!
//! `EXACT` pins the outcome of the objects whose member the family does not keep.
//!
//! Anything else fails the family's test: a load followed by a refused write, a written text
//! that does not read back equal, a number replaced or a document changed without a finding
//! at the number's path.
//!
//! "Reads back to the same value" is judged on what is written: the written text loads and its
//! value is written as the same text again. (The values themselves are not compared: a file
//! of an older format version is upgraded by its first reading and written at the newest
//! one, and a member the upgrade made is marked as made by it only in the first value.)
//!
//! `audit:` lines (shown with `--show-output`) give the outcome per object of each document.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use bedrock_addon::diagnostic::Diagnostic;
use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::load::LoadContext;
use bedrock_addon::pack::PackType;
use bedrock_addon::storage::PackPath;

#[macro_use]
#[path = "common/document_types.rs"]
mod document_types;
use document_types::{beta, client, ddui, pack, poi};

/// What a family made of one text.
struct Outcome {
    loaded: bool,
    /// The value written back, or the writer's error.
    written: Result<String, String>,
    findings: Vec<Diagnostic>,
}

fn outcome<T>(
    result: Result<Document<T>, Rejected>,
    write: impl Fn(&T) -> Result<String, String>,
) -> Outcome {
    match result {
        Ok(document) => Outcome {
            loaded: true,
            written: write(document.value()),
            findings: document.diagnostics().iter().cloned().collect(),
        },
        Err(rejected) => Outcome {
            loaded: false,
            written: Err("not loaded".to_owned()),
            findings: rejected.diagnostics.iter().cloned().collect(),
        },
    }
}

/// The family's public entry for one file.
fn content_in<T: Content>(path: &'static str, text: &str, cx: &LoadContext) -> Outcome {
    outcome(
        T::parse(&PackPath::from_static(path), text.as_bytes(), cx),
        |value| value.to_json().map_err(|error| format!("{error:?}")),
    )
}

// --- the probes ------------------------------------------------------------------------------

/// The numbers no double holds, each with a twin of the other sign: a document that holds
/// the number is written differently for the two.
fn numbers() -> Vec<(&'static str, String, &'static str)> {
    vec![
        ("1e999", "1e999".to_owned(), "-1e999"),
        ("-1e999", "-1e999".to_owned(), "1e999"),
        ("400 digits", "1234567890".repeat(40), "-1e999"),
    ]
}

/// The three shapes a number is inserted in: the member's text around the number and the
/// number's path below the member.
const SHAPES: &[(&str, &str, &str, &str)] = &[
    ("member", "", "", "zz_probe"),
    ("nested", "{\"deep\": {\"n\": ", "}}", "zz_probe/deep/n"),
    ("array", "[0, ", "]", "zz_probe/1"),
];

/// An object of a document: its path and where a member can be added at its end.
struct Object {
    path: Vec<String>,
    /// Offset of the closing brace.
    close: usize,
    empty: bool,
}

/// Every object of `text` (plain JSON, as the documents of this file are).
fn objects(text: &str) -> Vec<Object> {
    fn skip(bytes: &[u8], mut at: usize) -> usize {
        while bytes[at].is_ascii_whitespace() {
            at += 1;
        }
        at
    }
    fn string(bytes: &[u8], at: usize) -> usize {
        assert_eq!(bytes[at], b'"');
        let mut end = at + 1;
        while bytes[end] != b'"' {
            end += if bytes[end] == b'\\' { 2 } else { 1 };
        }
        end + 1
    }
    fn value(text: &str, at: usize, path: &mut Vec<String>, out: &mut Vec<Object>) -> usize {
        let bytes = text.as_bytes();
        let at = skip(bytes, at);
        match bytes[at] {
            b'{' => {
                let mut next = skip(bytes, at + 1);
                let empty = bytes[next] == b'}';
                while bytes[next] != b'}' {
                    let end = string(bytes, next);
                    path.push(text[next + 1..end - 1].to_owned());
                    next = skip(bytes, end);
                    assert_eq!(bytes[next], b':');
                    next = skip(bytes, value(text, next + 1, path, out));
                    path.pop();
                    if bytes[next] == b',' {
                        next = skip(bytes, next + 1);
                    }
                }
                out.push(Object {
                    path: path.clone(),
                    close: next,
                    empty,
                });
                next + 1
            }
            b'[' => {
                let mut next = skip(bytes, at + 1);
                let mut index = 0;
                while bytes[next] != b']' {
                    path.push(index.to_string());
                    next = skip(bytes, value(text, next, path, out));
                    path.pop();
                    index += 1;
                    if bytes[next] == b',' {
                        next = skip(bytes, next + 1);
                    }
                }
                next + 1
            }
            b'"' => string(bytes, at),
            _ => {
                let mut end = at;
                while !matches!(bytes.get(end), None | Some(b',' | b'}' | b']'))
                    && !bytes[end].is_ascii_whitespace()
                {
                    end += 1;
                }
                end
            }
        }
    }
    let mut out = Vec::new();
    value(&format!("{text} "), 0, &mut Vec::new(), &mut out);
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

/// A place a number is put: a text with `@` for the number, and the number's path.
struct Place {
    /// The object probed (or the name of a hand-written place) and the shape.
    object: String,
    shape: &'static str,
    template: String,
    path: String,
}

/// The keys of a place as `JsonPath` prints them: joined with `/`, a `~` or `/` inside a key
/// written `~0` or `~1` (the legacy texture map has the key `textures/old`).
fn json_path_text(keys: &[String]) -> String {
    let keys: Vec<String> = keys
        .iter()
        .map(|key| key.replace('~', "~0").replace('/', "~1"))
        .collect();
    keys.join("/")
}

fn places(document: &str, more: &[(&'static str, &'static str, &'static str)]) -> Vec<Place> {
    let mut out = Vec::new();
    for object in objects(document) {
        for (shape, before, after, below) in SHAPES {
            let comma = if object.empty { "" } else { ", " };
            let mut template = document[..object.close].to_owned();
            write!(template, "{comma}\"zz_probe\": {before}@{after}").unwrap();
            template.push_str(&document[object.close..]);
            // `below` is the probe's own path under the object, already slash-joined.
            let mut path = json_path_text(&object.path);
            if !path.is_empty() {
                path.push('/');
            }
            path.push_str(below);
            out.push(Place {
                object: if object.path.is_empty() {
                    "(root)".to_owned()
                } else {
                    object.path.join("/")
                },
                shape,
                template,
                path,
            });
        }
    }
    for (name, template, path) in more {
        assert_eq!(template.matches('@').count(), 1, "{name}");
        out.push(Place {
            object: (*name).to_owned(),
            shape: "kept",
            template: (*template).to_owned(),
            path: (*path).to_owned(),
        });
    }
    out
}

/// The variant name of a finding's code.
fn code(finding: &Diagnostic) -> String {
    finding.code.id().to_owned()
}

/// Whether `finding` is at the number: at its path, or at the end of it where the family's
/// paths start below the root (a `blocks.json` entry), the whole probe included.
fn is_at(finding: &Diagnostic, place: &Place) -> bool {
    let at = finding.location.json_path.to_string();
    let below = place
        .path
        .find("zz_probe")
        .map_or(place.path.as_str(), |start| &place.path[start..]);
    at == place.path
        || (at.ends_with(below)
            && place.path.ends_with(at.as_str())
            && place.path[..place.path.len() - at.len()].ends_with('/'))
}

/// The codes a family reports a replaced number with (`ClampedValue` is the client
/// block's; every other family's is `InvalidValue`).
const CLAMP_CODES: &[&str] = &["core/invalid_value", "core/clamped_value"];

/// The codes that say a member's name is not known, and nothing of its value: a number in
/// such a member is the clamp's to tell (core's `json::spoken_for`).
const NAME_CODES: &[&str] = &[
    "core/unrecognized_field",
    "core/unrecognized_component",
    "core/unknown_sound",
    "core/unknown_sound_worded",
    "sounds_json/not_an_event",
];

/// The objects whose unknown member the family does not keep, and the one outcome of the
/// nine texts of each: `(document type, object, outcome)`.
///
/// "not taken (InvalidValue)": the family says the member's name is not known and does not
/// keep it; the warning of the clamp at the number is the only word about the number. "not
/// held": the family refuses the entry the number is in with an error at the entry, and the
/// clamp's warning is left out. A manifest's `metadata` has both: the member is told as not
/// kept whatever it holds (`DroppedField`; it said nothing before), so a number a
/// double holds is "not held", and one no double holds has the clamp's warning beside.
const EXACT: &[(&str, &str, &str)] = &[
    // The biome and client-biome slices read with the strict mask: an unknown member or
    // component refuses the document.
    ("biome", "(root)", "refused"),
    ("biome", "minecraft:biome/components", "refused"),
    ("biome_beta", "minecraft:biome/components", "refused"),
    // A `MinecraftDocumentInput` refuses a number no double holds.
    ("block", "(root)", "refused"),
    ("client_biome", "(root)", "refused"),
    (
        "dimension",
        "minecraft:dimension/components/minecraft:dimension_bounds",
        "refused",
    ),
    (
        "manifest",
        "metadata",
        "not held / not taken (core/invalid_value)",
    ),
    // a third root member is refused (it was the raw-text path of 1.10.0).
    ("terrain_texture", "texture_data", "not held"),
    // A member of another type than a number, kept as written: told by the member.
    (
        "music_definitions",
        "(root)",
        "clamped (core/invalid_value) / told by the member (core/invalid_value)",
    ),
    (
        "sound_definitions",
        "sound_definitions",
        "clamped (core/invalid_value) / told by the member (sound_definitions/malformed_sound)",
    ),
];

/// The findings as `code at path`.
fn kinds(outcome: &Outcome) -> Vec<String> {
    outcome
        .findings
        .iter()
        .map(|d| format!("{} at `{}`", code(d), d.location.json_path))
        .collect()
}

/// What became of one text: `Ok` with the outcome's name, `Err` with the fault.
fn judge(
    read: &dyn Fn(&str) -> Outcome,
    place: &Place,
    token: &str,
    twin: &str,
) -> Result<String, String> {
    let ours = read(&place.template.replace('@', token));
    if !ours.loaded {
        return Ok("refused".to_owned());
    }
    let written = match &ours.written {
        Ok(written) => written,
        Err(error) => return Err(format!("loads and is not written: {error}")),
    };
    let again = read(written);
    if !again.loaded {
        return Err(format!(
            "the written text does not load: {:?}\n{written}",
            kinds(&again)
        ));
    }
    if again.written.as_ref() != Ok(written) {
        return Err(format!(
            "the written text is not what its own reading writes:\n{written}\n{:?}",
            again.written
        ));
    }
    if written.contains(token) {
        return Ok("as written".to_owned());
    }
    // The findings at the number that the same document with `0` there does not have (a
    // finding the `0` has too, a number where a string belongs, is counted once for it).
    let plain = read(&place.template.replace('@', "0"));
    let known = kinds(&plain);
    let mut left = known.clone();
    let mut at_number: Vec<String> = Vec::new();
    for finding in ours.findings.iter().filter(|d| is_at(d, place)) {
        let kind = format!("{} at `{}`", code(finding), finding.location.json_path);
        match left.iter().position(|known| *known == kind) {
            Some(index) => {
                left.swap_remove(index);
            }
            None => at_number.push(code(finding)),
        }
    }
    at_number.sort();
    at_number.dedup();
    let other = read(&place.template.replace('@', twin));
    if other.loaded && other.written.as_ref() == Ok(written) {
        // Not part of the document: because the member is not kept whatever it holds, or
        // because the family says at the number that it does not take it.
        if plain.written.as_ref() == Ok(written) && kinds(&ours) == known {
            return Ok("not held".to_owned());
        }
        if !at_number.is_empty() {
            return Ok(format!("not taken ({})", at_number.join(", ")));
        }
        return Err(format!(
            "the number changes what is loaded without a finding at `{}`: {:?}\nwith 0: {:?}\n{written}",
            place.path,
            kinds(&ours),
            known
        ));
    }
    at_number.retain(|code| CLAMP_CODES.contains(&code.as_str()));
    if at_number.is_empty() {
        // The member's own word about the value, which `0` gets too.
        let mut own: Vec<String> = ours
            .findings
            .iter()
            .filter(|d| d.location.json_path.to_string() == place.path)
            .map(code)
            .filter(|code| !NAME_CODES.contains(&code.as_str()))
            .collect();
        own.sort();
        own.dedup();
        if !own.is_empty() {
            return Ok(format!("told by the member ({})", own.join(", ")));
        }
        return Err(format!(
            "the number is replaced without a finding at `{}`: {:?}",
            place.path,
            kinds(&ours)
        ));
    }
    Ok(format!("clamped ({})", at_number.join(", ")))
}

/// The audit of one document type.
fn audit(
    family: &str,
    read: &dyn Fn(&str) -> Outcome,
    document: &str,
    more: &[(&'static str, &'static str, &'static str)],
) {
    let plain = read(document);
    assert!(
        plain.loaded,
        "{family}: the document does not load: {:?}",
        plain.findings
    );
    let written = plain
        .written
        .as_ref()
        .unwrap_or_else(|error| panic!("{family}: the document is not written: {error}"));
    let again = read(written);
    assert!(again.loaded, "{family}: {written}: {:?}", again.findings);
    assert_eq!(again.written.as_ref(), Ok(written), "{family}");

    let places = places(document, more);
    assert!(!places.is_empty(), "{family}: no place for a number");
    let mut faults = Vec::new();
    // Per object: the outcomes of its nine texts.
    let mut table: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for place in &places {
        for (name, token, twin) in numbers() {
            let row = table.entry(place.object.as_str()).or_default();
            match judge(read, place, &token, twin) {
                Ok(outcome) => {
                    row.insert(outcome);
                }
                Err(fault) => {
                    row.insert("FAULT".to_owned());
                    let fault: String = fault.chars().take(600).collect();
                    faults.push(format!(
                        "`{name}` at `{}` ({}): {fault}",
                        place.path, place.shape
                    ));
                }
            }
        }
    }
    let mut pinned = 0;
    for (object, outcomes) in &table {
        let outcomes: Vec<&str> = outcomes.iter().map(String::as_str).collect();
        let outcomes = outcomes.join(" / ");
        println!("audit: {family} | {object} | {outcomes}");
        for (_, _, expected) in EXACT
            .iter()
            .filter(|(of, at, _)| *of == family && at == object)
        {
            pinned += 1;
            assert_eq!(outcomes, *expected, "{family}: the outcome at `{object}`");
        }
        if outcomes.contains("told by the member") {
            assert!(
                EXACT
                    .iter()
                    .any(|(of, at, _)| *of == family && at == object),
                "{family}: `{object}` is told by the member and is not listed in EXACT"
            );
        }
    }
    assert_eq!(
        pinned,
        EXACT.iter().filter(|(of, _, _)| *of == family).count(),
        "{family}: an object of EXACT is not one of the document"
    );
    // Every fault is counted; the first few are shown in full.
    let shown: Vec<&str> = faults.iter().take(4).map(String::as_str).collect();
    assert!(
        faults.is_empty(),
        "{family}: {} of {} texts are at fault (objects marked FAULT above); the first:\n{}",
        faults.len(),
        places.len() * 3,
        shown.join("\n")
    );
}

/// One test per document type read through `Content::parse`: its name, type, path and a
/// document it loads, then hand-written places (name, text with `@`, path of the number).
macro_rules! families {
    ($( $test:ident: $type:ty, $path:literal $(in $cx:expr)?, $document:expr $(, [$($more:expr),* $(,)?])?; )*) => {
        $(
            #[test]
            fn $test() {
                #[allow(unused_variables, reason = "most families read in a detached context")]
                let cx = LoadContext::detached();
                $(let cx = $cx;)?
                audit(
                    stringify!($test),
                    &|text| content_in::<$type>($path, text, &cx),
                    $document,
                    &[$($($more),*)?],
                );
            }
        )*
    };
}

// --- the harness itself ----------------------------------------------------------------------

#[test]
fn every_object_of_a_document_is_a_place() {
    let found = objects(r#"{"a": {"b": [1, {"c": "}\""}, {}]}, "d": {}}"#);
    let paths: Vec<String> = found.iter().map(|o| o.path.join("/")).collect();
    assert_eq!(paths, ["", "a", "a/b/1", "a/b/2", "d"]);
    let all = places(r#"{"a": {}}"#, &[]);
    let texts: Vec<&str> = all.iter().map(|p| p.template.as_str()).collect();
    assert_eq!(
        texts,
        [
            r#"{"a": {}, "zz_probe": @}"#,
            r#"{"a": {}, "zz_probe": {"deep": {"n": @}}}"#,
            r#"{"a": {}, "zz_probe": [0, @]}"#,
            r#"{"a": {"zz_probe": @}}"#,
            r#"{"a": {"zz_probe": {"deep": {"n": @}}}}"#,
            r#"{"a": {"zz_probe": [0, @]}}"#,
        ]
    );
    assert_eq!(all[4].path, "a/zz_probe/deep/n");
}

// --- the families ----------------------------------------------------------------------------

// Not here, and why:
// * `LangFile`, `Function` (text) and `Structure` (NBT) are no JSON documents;
//   `JigsawMetadata` is a binary file whose `to_json` is `NotRepresentable` by rule.
// * A persona piece's geometry and emote animation have no parse entry of their own: they are
//   read by `PersonaPiece::load_all` through the reader the piece's meta file is read with
//   (`persona_piece_meta` below; `skin::persona::read`, one clamp for all four files).
// * The script metadata readers and `WorldPacks` have no writer.

document_types!(families);

/// The saved pack settings have an entry of their own (`SavedSettings::parse`).
#[test]
fn pack_settings() {
    use bedrock_addon::pack::settings::SavedSettings;
    audit(
        "pack_settings",
        &|text| {
            outcome(SavedSettings::parse(text.as_bytes()), |value| {
                value.to_json().map_err(|error| format!("{error:?}"))
            })
        },
        r#"{"format_version": "1.21.100", "minecraft:pack_settings": {"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "values": {"banana:slider": 2.5}}]}}"#,
        &[(
            "a saved value",
            r#"{"format_version": "1.21.100", "minecraft:pack_settings": {"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "values": {"banana:slider": @}}]}}"#,
            "minecraft:pack_settings/settings/0/values/banana:slider",
        )],
    );
}
