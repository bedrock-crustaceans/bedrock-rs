#![allow(clippy::pedantic, missing_docs)]
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "full")]
//! What a stack load yields can be written: `non_finite.rs` proves each
//! document type's single-file entry; this file proves the stack accessors whose copies are
//! read by a parse job of their own (JSON UI, the three sound documents, `biomes_client.json`,
//! the two texture atlases, the flipbook list, the material files), and, as a control, four
//! accessors whose job calls the entry `non_finite.rs` proves.
//!
//! One test per file kind. A stack of two packs holds a copy of the file in each; a number no
//! double holds (`1e999`, `-1e999`, an integer of 400 digits) is put, one place and one pack
//! at a time, into the **lower** copy (in an unknown member, in a member kept as JSON, inside
//! a member the higher pack does not write) and into the **higher** copy. The accessor is
//! loaded, and for each text one of these holds:
//!
//! * **refused**: the diagnostics hold an error for that copy (its pack, its file) that the
//!   copy with `0` there does not have;
//! * **clamped**: a finding of the clamp code is at the number's own path, in the pack and
//!   the file that wrote it, and the copy with `0` there has none (**not taken** when no
//!   written text and no held value shows the clamped number);
//! * **not held**: what is yielded, written and found is that of `0` there, and of the
//!   number of the other sign.
//!
//! Whatever the outcome, every document the accessor yields is written (`to_json` is `Ok`),
//! its text loads through the type's own entry and is written the same again, and no value
//! the accessor hands out raw (a layer's JSON, a UI control) holds an infinity. Anything
//! else fails the test: a silent infinity, a silent clamp, a finding in the wrong pack.
//!
//! `audit:` lines (shown with `--show-output`) give the outcomes per copy and place.

#[path = "common/two_packs.rs"]
mod two_packs;

use std::collections::{BTreeMap, BTreeSet};

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::document::{Content, Merged, RawLayer};
use bedrock_addon::json::Value;
use bedrock_addon::load::LoadContext;
use bedrock_addon::pack::PackRef;
use bedrock_addon::storage::PackPath;

use two_packs::{behavior_stack, resource_stack};

/// A document an accessor yields, written, and what the reading of that text writes.
struct Written {
    name: String,
    text: Result<String, String>,
    again: Option<Result<String, String>>,
}

/// What an accessor yields of a stack.
#[derive(Default)]
struct Yielded {
    written: Vec<Written>,
    /// The values handed out as they are held: named, as JSON.
    held: Vec<(String, Value)>,
    findings: Vec<Diagnostic>,
}

impl Yielded {
    /// `value`, which the family writes to `path`: written, and the written text read back
    /// through the type's own entry and written again.
    fn write<T: Content>(&mut self, name: &str, path: &str, value: &T) {
        let text = value.to_json().map_err(|error| format!("{error:?}"));
        let again = text.as_ref().ok().map(|text| {
            let path = PackPath::new(path).expect("a pack path");
            match T::parse(&path, text.as_bytes(), &LoadContext::detached()) {
                Ok(document) => document
                    .value()
                    .to_json()
                    .map_err(|error| format!("{error:?}")),
                Err(rejected) => Err(format!(
                    "the written text does not load: {:?}",
                    rejected
                        .diagnostics
                        .iter()
                        .map(kind)
                        .collect::<Vec<String>>()
                )),
            }
        });
        self.written.push(Written {
            name: name.to_owned(),
            text,
            again,
        });
    }

    /// The layers of a merge: each copy's JSON as the accessor hands it out.
    fn layers<T: bedrock_addon::document::Merge>(
        &mut self,
        name: &str,
        merged: &Merged<T, RawLayer>,
    ) {
        for (index, layer) in merged.layers().iter().enumerate() {
            self.held
                .push((format!("{name} layer {index}"), layer.json.clone()));
        }
    }
}

/// A finding as `code severity at path in pack-index file`.
fn kind(finding: &Diagnostic) -> String {
    format!(
        "{} {:?} at `{}` in {:?} {}",
        finding.code.name(),
        finding.severity,
        finding.location.json_path,
        finding.location.pack.as_ref().and_then(PackRef::index),
        finding
            .location
            .path
            .as_ref()
            .map_or("-", |path| path.as_str()),
    )
}

/// The path of the first infinity `value` holds.
fn infinity_in(value: &Value) -> Option<String> {
    let mut open = vec![(String::new(), value)];
    while let Some((at, value)) = open.pop() {
        if let Some(number) = value.as_number() {
            if number.is_float() && !number.to_f64_lossy().is_finite() {
                return Some(at);
            }
        } else if let Some(array) = value.as_array() {
            for (index, element) in array.iter().enumerate() {
                open.push((format!("{at}/{index}"), element));
            }
        } else if let Some(object) = value.as_object() {
            for (key, member) in object {
                open.push((format!("{at}/{}", key.as_str()), member));
            }
        }
    }
    None
}

/// The largest finite double as the writers spell it.
const LARGEST: &str = "1.7976931348623157e308";

/// The codes a family reports a replaced number with (as `non_finite.rs`).
const CLAMP_CODES: &[&str] = &["InvalidValue", "ClampedValue"];

/// The numbers no double holds, each with a twin of the other sign.
fn numbers() -> Vec<(&'static str, String, &'static str)> {
    vec![
        ("1e999", "1e999".to_owned(), "-1e999"),
        ("-1e999", "-1e999".to_owned(), "1e999"),
        ("400 digits", "1234567890".repeat(40), "-1e999"),
    ]
}

/// One pack's copy of the probed file: its path, its text, and the places a number is put
/// (the text with `@` for the number, and the number's path in the file).
struct Copy<'a> {
    path: &'a str,
    text: &'a str,
    places: &'a [(&'a str, &'a str)],
}

/// An accessor of a stack of two packs, given the lower and the higher pack's files (path
/// and text): what it yields, and the two packs as the stack names them.
type Accessor = dyn Fn(&[(&str, &str)], &[(&str, &str)]) -> (Yielded, PackRef, PackRef);

/// The stack of a test: the two copies, the files both packs hold beside them, and the
/// accessor (given the lower and the higher pack's files).
struct Stack<'a> {
    family: &'a str,
    lower: Copy<'a>,
    higher: Copy<'a>,
    beside: &'a [(&'a str, &'a str)],
    load: &'a Accessor,
}

impl Stack<'_> {
    /// The accessor's yield with `lower` and `higher` as the two copies' texts.
    fn load(&self, lower: &str, higher: &str) -> (Yielded, PackRef, PackRef) {
        let mut low = self.beside.to_vec();
        low.push((self.lower.path, lower));
        let mut high = self.beside.to_vec();
        high.push((self.higher.path, higher));
        (self.load)(&low, &high)
    }
}

/// The faults of what is yielded, whatever the outcome: a document that is not written or
/// not written the same again, a held infinity.
fn unwritable(yielded: &Yielded) -> Option<String> {
    for written in &yielded.written {
        match (&written.text, &written.again) {
            (Err(error), _) => {
                return Some(format!(
                    "`{}` is yielded and not written: {error}",
                    written.name
                ));
            }
            (Ok(text), Some(again)) if again.as_ref() != Ok(text) => {
                return Some(format!(
                    "`{}`: the written text is not what its own reading writes:\n{text}\n{again:?}",
                    written.name
                ));
            }
            _ => {}
        }
    }
    for (name, value) in &yielded.held {
        if let Some(at) = infinity_in(value) {
            return Some(format!("`{name}` holds an infinity at `{at}`"));
        }
    }
    None
}

/// Everything a yield shows: the written texts, the held values, the findings.
fn shown(yielded: &Yielded) -> (Vec<String>, Vec<String>, Vec<String>) {
    (
        yielded
            .written
            .iter()
            .map(|written| format!("{}: {:?}", written.name, written.text))
            .collect(),
        yielded
            .held
            .iter()
            .map(|(name, value)| format!("{name}: {value:?}"))
            .collect(),
        yielded.findings.iter().map(kind).collect(),
    )
}

/// The findings of `yielded` in `pack`'s copy at `path`.
fn in_copy<'a>(yielded: &'a Yielded, pack: &PackRef, path: &str) -> Vec<&'a Diagnostic> {
    yielded
        .findings
        .iter()
        .filter(|finding| {
            finding.location.pack.as_ref() == Some(pack)
                && finding.location.path.as_ref().map(PackPath::as_str) == Some(path)
        })
        .collect()
}

/// What became of one text: `Ok` with the outcome's name, `Err` with the fault.
fn judge(
    stack: &Stack<'_>,
    in_lower: bool,
    place: (&str, &str),
    token: &str,
    twin: &str,
) -> Result<String, String> {
    let (template, at) = place;
    let load = |number: &str| {
        let text = template.replace('@', number);
        let (yielded, lower, higher) = if in_lower {
            stack.load(&text, stack.higher.text)
        } else {
            stack.load(stack.lower.text, &text)
        };
        (yielded, if in_lower { lower } else { higher })
    };
    let file = if in_lower {
        stack.lower.path
    } else {
        stack.higher.path
    };
    let (ours, pack) = load(token);
    if let Some(fault) = unwritable(&ours) {
        return Err(fault);
    }
    let (plain, plain_pack) = load("0");
    let is_clamp = |finding: &&Diagnostic| {
        CLAMP_CODES.contains(&finding.code.name()) && finding.location.json_path.to_string() == at
    };
    let clamps = in_copy(&ours, &pack, file).into_iter().filter(is_clamp);
    let known = in_copy(&plain, &plain_pack, file)
        .into_iter()
        .filter(is_clamp);
    if clamps.count() > known.count() {
        let visible = ours.written.iter().any(|written| {
            written
                .text
                .as_ref()
                .is_ok_and(|text| text.contains(LARGEST))
        }) || ours
            .held
            .iter()
            .any(|(_, value)| format!("{value:?}").contains(LARGEST));
        return Ok(if visible { "clamped" } else { "not taken" }.to_owned());
    }
    let errors = |yielded: &Yielded, pack: &PackRef| {
        in_copy(yielded, pack, file)
            .into_iter()
            .filter(|finding| finding.severity == Severity::Error)
            .count()
    };
    if errors(&ours, &pack) > errors(&plain, &plain_pack) {
        return Ok("refused".to_owned());
    }
    let (other, _) = load(twin);
    if shown(&ours) == shown(&plain) && shown(&ours) == shown(&other) {
        return Ok("not held".to_owned());
    }
    Err(format!(
        "neither an error for the copy nor a finding of the clamp at `{at}` in {file} of pack {:?}: {:?}",
        pack.index(),
        shown(&ours).2
    ))
}

/// The audit of one stack.
fn audit(stack: &Stack<'_>) {
    let family = stack.family;
    let (plain, lower, higher) = stack.load(stack.lower.text, stack.higher.text);
    assert_ne!(lower, higher, "{family}: two packs");
    if let Some(fault) = unwritable(&plain) {
        panic!("{family}: the stack as it is: {fault}");
    }
    assert!(
        !plain.written.is_empty() || !plain.held.is_empty(),
        "{family}: the accessor yields nothing of the stack"
    );
    for (pack, copy) in [(&lower, &stack.lower), (&higher, &stack.higher)] {
        let errors: Vec<String> = in_copy(&plain, pack, copy.path)
            .into_iter()
            .filter(|finding| finding.severity == Severity::Error)
            .map(kind)
            .collect();
        assert!(errors.is_empty(), "{family}: the copies load: {errors:?}");
    }
    let mut faults = Vec::new();
    let mut table: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut texts = 0;
    for (in_lower, copy) in [(true, &stack.lower), (false, &stack.higher)] {
        assert!(!copy.places.is_empty(), "{family}: no place for a number");
        for place in copy.places {
            assert_eq!(place.0.matches('@').count(), 1, "{family}: {}", place.0);
            let which = if in_lower { "lower" } else { "higher" };
            let row = table.entry(format!("{which} | {}", place.1)).or_default();
            for (name, token, twin) in numbers() {
                texts += 1;
                match judge(stack, in_lower, *place, &token, twin) {
                    Ok(outcome) => {
                        row.insert(outcome);
                    }
                    Err(fault) => {
                        row.insert("FAULT".to_owned());
                        let fault: String = fault.chars().take(700).collect();
                        faults.push(format!("`{name}` at `{}` of the {which} copy: {fault}", {
                            place.1
                        }));
                    }
                }
            }
        }
    }
    for (place, outcomes) in &table {
        let outcomes: Vec<&str> = outcomes.iter().map(String::as_str).collect();
        println!("audit: {family} | {place} | {}", outcomes.join(" / "));
    }
    let first: Vec<&str> = faults.iter().take(4).map(String::as_str).collect();
    assert!(
        faults.is_empty(),
        "{family}: {} of {texts} texts are at fault (places marked FAULT above); the first:\n{}",
        faults.len(),
        first.join("\n")
    );
}

/// Every finding the stack has for what was loaded.
fn findings<S: bedrock_addon::pack::Side>(
    stack: &bedrock_addon::pack::PackStack<S>,
) -> Vec<Diagnostic> {
    stack.report().iter().cloned().collect()
}

// --- the harness itself ----------------------------------------------------------------------

#[test]
fn an_infinity_in_a_held_value_is_found() {
    let finite: Value = facet_json::from_str(r#"{"a": [1, {"b": 1.5e300}]}"#).unwrap();
    assert_eq!(infinity_in(&finite), None);
    let infinite: Value = facet_json::from_str(r#"{"a": [1, {"b": -1e999}]}"#).unwrap();
    assert_eq!(infinity_in(&infinite).as_deref(), Some("/a/1/b"));
}

// --- copies read by a parse job of their own ---------------------------------------------------

/// `rp.ui()`: the files of the merge, the global variables, every layer and every control.
fn ui(lower: &[(&str, &str)], higher: &[(&str, &str)]) -> (Yielded, PackRef, PackRef) {
    let (stack, low, high) = resource_stack(lower, higher);
    let mut out = Yielded::default();
    let merged = stack.ui();
    let value = merged.value();
    for (path, file) in &value.files {
        out.write(path, path, file);
    }
    out.write(
        "global variables",
        "ui/_global_variables.json",
        &value.global_variables,
    );
    out.layers("ui", merged);
    for control in value.controls() {
        let name = format!("control {}.{:?}", control.namespace, control.key);
        out.held.push((name, control.raw.clone()));
    }
    out.findings = findings(&stack);
    (out, low, high)
}

const UI_LIST: (&str, &str) = ("ui/_ui_defs.json", r#"{"ui_defs": ["ui/a.json"]}"#);
const UI_FILE_LOW: &str = r#"{"namespace": "demo", "panel": {"type": "panel", "size": [5, 5]}, "lower_only": {"type": "label", "text": "x"}}"#;
const UI_FILE_HIGH: &str = r#"{"namespace": "demo", "panel": {"type": "panel"}}"#;

#[test]
fn ui_defs() {
    audit(&Stack {
        family: "ui_defs",
        lower: Copy {
            path: "ui/_ui_defs.json",
            text: r#"{"ui_defs": ["ui/a.json"], "note": {"kept": true}}"#,
            places: &[
                (r#"{"ui_defs": ["ui/a.json"], "zz_probe": @}"#, "zz_probe"),
                (
                    r#"{"ui_defs": ["ui/a.json"], "note": {"deep": {"n": @}}}"#,
                    "note/deep/n",
                ),
                (r#"{"ui_defs": ["ui/a.json", @]}"#, "ui_defs/1"),
            ],
        },
        higher: Copy {
            path: "ui/_ui_defs.json",
            text: r#"{"ui_defs": ["ui/a.json"]}"#,
            places: &[
                (
                    r#"{"ui_defs": ["ui/a.json"], "zz_probe": [0, @]}"#,
                    "zz_probe/1",
                ),
                (r#"{"ui_defs": ["ui/a.json", @]}"#, "ui_defs/1"),
            ],
        },
        beside: &[("ui/a.json", UI_FILE_HIGH)],
        load: &ui,
    });
}

#[test]
fn ui_global_variables() {
    audit(&Stack {
        family: "ui_global_variables",
        lower: Copy {
            path: "ui/_global_variables.json",
            text: r#"{"$a": 1, "$low": {"x": 1}}"#,
            places: &[
                (r#"{"$a": 1, "$low": {"x": @}}"#, "$low/x"),
                (r#"{"$a": 1, "$low": [0, @]}"#, "$low/1"),
                (r#"{"$a": @, "$low": {"x": 1}}"#, "$a"),
                (
                    r#"{"$a": 1, "zz_probe": {"deep": {"n": @}}}"#,
                    "zz_probe/deep/n",
                ),
            ],
        },
        higher: Copy {
            path: "ui/_global_variables.json",
            text: r#"{"$a": 2}"#,
            places: &[
                (r#"{"$a": @}"#, "$a"),
                (
                    r#"{"$a": 2, "zz_probe": {"deep": {"n": @}}}"#,
                    "zz_probe/deep/n",
                ),
            ],
        },
        beside: &[UI_LIST, ("ui/a.json", UI_FILE_HIGH)],
        load: &ui,
    });
}

#[test]
fn ui_file() {
    audit(&Stack {
        family: "ui_file",
        lower: Copy {
            path: "ui/a.json",
            text: UI_FILE_LOW,
            places: &[
                (
                    r#"{"namespace": "demo", "panel": {"type": "panel", "size": [5, 5]}, "lower_only": {"type": "label", "text": "x", "zz_probe": @}}"#,
                    "lower_only/zz_probe",
                ),
                (
                    r#"{"namespace": "demo", "panel": {"type": "panel", "size": [5, @]}, "lower_only": {"type": "label"}}"#,
                    "panel/size/1",
                ),
                (
                    r#"{"namespace": "demo", "panel": {"type": "panel"}, "lower_only": {"type": "label", "zz_probe": {"deep": {"n": @}}}}"#,
                    "lower_only/zz_probe/deep/n",
                ),
                (
                    r#"{"namespace": "demo", "panel": {"type": "panel"}, "zz_probe": @}"#,
                    "zz_probe",
                ),
            ],
        },
        higher: Copy {
            path: "ui/a.json",
            text: UI_FILE_HIGH,
            places: &[
                (
                    r#"{"namespace": "demo", "panel": {"type": "panel", "zz_probe": @}}"#,
                    "panel/zz_probe",
                ),
                (
                    r#"{"namespace": "demo", "panel": {"type": "panel", "size": [5, @]}}"#,
                    "panel/size/1",
                ),
            ],
        },
        beside: &[UI_LIST],
        load: &ui,
    });
}

/// `rp.sounds()`: the three merged documents and their layers.
fn sounds(lower: &[(&str, &str)], higher: &[(&str, &str)]) -> (Yielded, PackRef, PackRef) {
    let (stack, low, high) = resource_stack(lower, higher);
    let mut out = Yielded::default();
    let definitions = stack.sound_definitions();
    let bindings = stack.sounds_json();
    let music = stack.music();
    out.write(
        "sound definitions",
        "sounds/sound_definitions.json",
        definitions.value(),
    );
    out.write("sounds.json", "sounds.json", bindings.value());
    out.write(
        "music definitions",
        "sounds/music_definitions.json",
        music.value(),
    );
    out.layers("sound definitions", definitions);
    out.layers("sounds.json", bindings);
    out.layers("music definitions", music);
    out.findings = findings(&stack);
    (out, low, high)
}

#[test]
fn sound_definitions() {
    audit(&Stack {
        family: "sound_definitions",
        lower: Copy {
            path: "sounds/sound_definitions.json",
            text: r#"{"format_version": "1.20.20", "sound_definitions": {"low": {"sounds": ["a"]}, "both": {"sounds": ["b"]}}}"#,
            places: &[
                (
                    r#"{"format_version": "1.20.20", "sound_definitions": {"low": {"sounds": ["a"]}, "both": {"sounds": ["b"]}}, "zz_probe": @}"#,
                    "zz_probe",
                ),
                (
                    r#"{"format_version": "1.20.20", "sound_definitions": {"low": {"sounds": ["a"], "zz_probe": @}, "both": {"sounds": ["b"]}}}"#,
                    "sound_definitions/low/zz_probe",
                ),
                (
                    r#"{"format_version": "1.20.20", "sound_definitions": {"low": {"sounds": [{"name": "a", "zz_probe": {"deep": {"n": @}}}]}, "both": {"sounds": ["b"]}}}"#,
                    "sound_definitions/low/sounds/0/zz_probe/deep/n",
                ),
                (
                    r#"{"format_version": "1.20.20", "sound_definitions": {"low": {"sounds": ["a"]}, "both": {"sounds": ["b"], "zz_probe": [0, @]}}}"#,
                    "sound_definitions/both/zz_probe/1",
                ),
            ],
        },
        higher: Copy {
            path: "sounds/sound_definitions.json",
            text: r#"{"format_version": "1.20.20", "sound_definitions": {"both": {"sounds": ["c"]}}}"#,
            places: &[
                (
                    r#"{"format_version": "1.20.20", "sound_definitions": {"both": {"sounds": ["c"], "zz_probe": @}}}"#,
                    "sound_definitions/both/zz_probe",
                ),
                (
                    r#"{"format_version": "1.20.20", "sound_definitions": {"both": {"sounds": ["c"]}}, "zz_probe": {"deep": {"n": @}}}"#,
                    "zz_probe/deep/n",
                ),
            ],
        },
        beside: &[],
        load: &sounds,
    });
}

#[test]
fn sound_bindings() {
    audit(&Stack {
        family: "sounds",
        lower: Copy {
            path: "sounds.json",
            text: r#"{"block_sounds": {"stone": {"events": {"break": {"sound": "dig.stone", "volume": [0.5, 1]}}, "pitch": 1}}, "entity_sounds": {"defaults": {"events": {"ambient": "mob.pig.say"}}}}"#,
            places: &[
                (
                    r#"{"block_sounds": {"stone": {"events": {"break": {"sound": "dig.stone"}}, "pitch": 1}}, "zz_probe": @}"#,
                    "zz_probe",
                ),
                (
                    r#"{"block_sounds": {"stone": {"events": {"break": {"sound": "dig.stone"}}, "pitch": 1, "zz_probe": @}}}"#,
                    "block_sounds/stone/zz_probe",
                ),
                (
                    r#"{"block_sounds": {"stone": {"events": {"break": {"sound": "dig.stone", "zz_probe": {"deep": {"n": @}}}}}}}"#,
                    "block_sounds/stone/events/break/zz_probe/deep/n",
                ),
            ],
        },
        higher: Copy {
            path: "sounds.json",
            text: r#"{"block_sounds": {"dirt": {"events": {"break": "dig.dirt"}}}}"#,
            places: &[
                (
                    r#"{"block_sounds": {"dirt": {"events": {"break": "dig.dirt"}, "zz_probe": [0, @]}}}"#,
                    "block_sounds/dirt/zz_probe/1",
                ),
                (
                    r#"{"block_sounds": {"dirt": {"events": {"break": "dig.dirt"}}}, "zz_probe": @}"#,
                    "zz_probe",
                ),
            ],
        },
        beside: &[],
        load: &sounds,
    });
}

#[test]
fn music_definitions() {
    audit(&Stack {
        family: "music_definitions",
        lower: Copy {
            path: "sounds/music_definitions.json",
            text: r#"{"game": {"event_name": "music.game", "min_delay": 60, "max_delay": 180}, "low": {"event_name": "music.low"}}"#,
            places: &[
                (
                    r#"{"game": {"event_name": "music.game"}, "low": {"event_name": "music.low", "zz_probe": @}}"#,
                    "low/zz_probe",
                ),
                (
                    r#"{"game": {"event_name": "music.game", "zz_probe": [0, @]}, "low": {"event_name": "music.low"}}"#,
                    "game/zz_probe/1",
                ),
                (
                    r#"{"game": {"event_name": "music.game"}, "low": {"event_name": "music.low", "zz_probe": {"deep": {"n": @}}}}"#,
                    "low/zz_probe/deep/n",
                ),
            ],
        },
        higher: Copy {
            path: "sounds/music_definitions.json",
            text: r#"{"game": {"event_name": "music.game2"}}"#,
            places: &[(
                r#"{"game": {"event_name": "music.game2", "zz_probe": @}}"#,
                "game/zz_probe",
            )],
        },
        beside: &[],
        load: &sounds,
    });
}

/// `rp.legacy_client_biomes()`: the merged document and its layers.
fn legacy_client_biomes(
    lower: &[(&str, &str)],
    higher: &[(&str, &str)],
) -> (Yielded, PackRef, PackRef) {
    let (stack, low, high) = resource_stack(lower, higher);
    let mut out = Yielded::default();
    let merged = stack.legacy_client_biomes();
    out.write("biomes_client.json", "biomes_client.json", merged.value());
    out.layers("biomes_client.json", merged);
    out.findings = findings(&stack);
    (out, low, high)
}

#[test]
fn biomes_client() {
    audit(&Stack {
        family: "biomes_client",
        lower: Copy {
            path: "biomes_client.json",
            text: r##"{"biomes": {"plains": {"water_surface_color": "#44aff5"}, "default": {}}}"##,
            places: &[
                (
                    r##"{"biomes": {"plains": {"water_surface_color": "#44aff5"}, "default": {}}, "zz_probe": @}"##,
                    "zz_probe",
                ),
                (
                    r##"{"biomes": {"plains": {"water_surface_color": "#44aff5", "zz_probe": @}, "default": {}}}"##,
                    "biomes/plains/zz_probe",
                ),
                (
                    r##"{"biomes": {"plains": {"water_surface_color": "#44aff5"}, "default": {"zz_probe": {"deep": {"n": @}}}}}"##,
                    "biomes/default/zz_probe/deep/n",
                ),
            ],
        },
        higher: Copy {
            path: "biomes_client.json",
            text: r##"{"biomes": {"desert": {"water_surface_color": "#32a598"}}}"##,
            places: &[
                (
                    r##"{"biomes": {"desert": {"water_surface_color": "#32a598", "zz_probe": [0, @]}}}"##,
                    "biomes/desert/zz_probe/1",
                ),
                (
                    r##"{"biomes": {"desert": {"water_surface_color": "#32a598"}}, "zz_probe": @}"##,
                    "zz_probe",
                ),
            ],
        },
        beside: &[],
        load: &legacy_client_biomes,
    });
}

/// `rp.textures()`: the two atlases and the flipbook list, with their layers.
fn textures(lower: &[(&str, &str)], higher: &[(&str, &str)]) -> (Yielded, PackRef, PackRef) {
    let (stack, low, high) = resource_stack(lower, higher);
    let mut out = Yielded::default();
    let group = stack.texture_atlases();
    let flipbooks = stack.flipbooks();
    if let Some(atlas) = group.terrain() {
        out.write("terrain atlas", "textures/terrain_texture.json", atlas);
    }
    if let Some(atlas) = group.item() {
        out.write("item atlas", "textures/item_texture.json", atlas);
    }
    out.write(
        "flipbooks",
        "textures/flipbook_textures.json",
        flipbooks.value(),
    );
    out.layers("terrain atlas", &group.terrain_atlas);
    out.layers("item atlas", &group.item_atlas);
    out.layers("flipbooks", flipbooks);
    out.findings = findings(&stack);
    (out, low, high)
}

#[test]
fn terrain_texture() {
    audit(&Stack {
        family: "terrain_texture",
        lower: Copy {
            path: "textures/terrain_texture.json",
            text: r#"{"texture_name": "atlas.terrain", "texture_data": {"dirt": {"textures": "textures/blocks/dirt"}, "stone": {"textures": "textures/blocks/stone"}}}"#,
            places: &[
                (
                    r#"{"texture_name": "atlas.terrain", "texture_data": {"dirt": {"textures": "textures/blocks/dirt"}, "stone": {"textures": "textures/blocks/stone"}}, "zz_probe": @}"#,
                    "zz_probe",
                ),
                (
                    r#"{"texture_name": "atlas.terrain", "texture_data": {"dirt": {"textures": "textures/blocks/dirt", "zz_probe": @}, "stone": {"textures": "textures/blocks/stone"}}}"#,
                    "texture_data/dirt/zz_probe",
                ),
                (
                    r#"{"texture_name": "atlas.terrain", "texture_data": {"dirt": {"textures": ["textures/blocks/dirt", {"path": "textures/blocks/dirt2", "zz_probe": {"deep": {"n": @}}}]}, "stone": {"textures": "textures/blocks/stone"}}}"#,
                    "texture_data/dirt/textures/1/zz_probe/deep/n",
                ),
            ],
        },
        higher: Copy {
            path: "textures/terrain_texture.json",
            text: r#"{"texture_name": "atlas.terrain", "texture_data": {"stone": {"textures": "textures/blocks/stone2"}}}"#,
            places: &[
                (
                    r#"{"texture_name": "atlas.terrain", "texture_data": {"stone": {"textures": "textures/blocks/stone2", "zz_probe": [0, @]}}}"#,
                    "texture_data/stone/zz_probe/1",
                ),
                (
                    r#"{"texture_name": "atlas.terrain", "texture_data": {"stone": {"textures": "textures/blocks/stone2"}}, "zz_probe": @}"#,
                    "zz_probe",
                ),
            ],
        },
        beside: &[],
        load: &textures,
    });
}

#[test]
fn item_texture() {
    audit(&Stack {
        family: "item_texture",
        lower: Copy {
            path: "textures/item_texture.json",
            text: r#"{"texture_name": "atlas.items", "texture_data": {"apple": {"textures": "textures/items/apple"}, "stick": {"textures": "textures/items/stick"}}}"#,
            places: &[
                (
                    r#"{"texture_name": "atlas.items", "texture_data": {"apple": {"textures": "textures/items/apple", "zz_probe": @}, "stick": {"textures": "textures/items/stick"}}}"#,
                    "texture_data/apple/zz_probe",
                ),
                (
                    r#"{"texture_name": "atlas.items", "texture_data": {"apple": {"textures": "textures/items/apple"}, "stick": {"textures": "textures/items/stick"}}, "zz_probe": {"deep": {"n": @}}}"#,
                    "zz_probe/deep/n",
                ),
            ],
        },
        higher: Copy {
            path: "textures/item_texture.json",
            text: r#"{"texture_name": "atlas.items", "texture_data": {"stick": {"textures": "textures/items/stick2"}}}"#,
            places: &[(
                r#"{"texture_name": "atlas.items", "texture_data": {"stick": {"textures": "textures/items/stick2"}}, "zz_probe": @}"#,
                "zz_probe",
            )],
        },
        beside: &[],
        load: &textures,
    });
}

#[test]
fn flipbook_textures() {
    audit(&Stack {
        family: "flipbook_textures",
        lower: Copy {
            path: "textures/flipbook_textures.json",
            text: r#"[{"flipbook_texture": "textures/blocks/x", "atlas_tile": "x", "ticks_per_frame": 1}, {"flipbook_texture": "textures/blocks/low", "atlas_tile": "low"}]"#,
            places: &[
                (
                    r#"[{"flipbook_texture": "textures/blocks/x", "atlas_tile": "x", "ticks_per_frame": 1}, {"flipbook_texture": "textures/blocks/low", "atlas_tile": "low", "zz_probe": @}]"#,
                    "1/zz_probe",
                ),
                (
                    r#"[{"flipbook_texture": "textures/blocks/x", "atlas_tile": "x", "zz_probe": {"deep": {"n": @}}}, {"flipbook_texture": "textures/blocks/low", "atlas_tile": "low"}]"#,
                    "0/zz_probe/deep/n",
                ),
                (
                    r#"[{"flipbook_texture": "textures/blocks/x", "atlas_tile": "x"}, {"flipbook_texture": "textures/blocks/low", "atlas_tile": "low", "zz_probe": [0, @]}]"#,
                    "1/zz_probe/1",
                ),
            ],
        },
        higher: Copy {
            path: "textures/flipbook_textures.json",
            text: r#"[{"flipbook_texture": "textures/blocks/x", "atlas_tile": "x", "ticks_per_frame": 2}]"#,
            places: &[(
                r#"[{"flipbook_texture": "textures/blocks/x", "atlas_tile": "x", "ticks_per_frame": 2, "zz_probe": @}]"#,
                "0/zz_probe",
            )],
        },
        beside: &[],
        load: &textures,
    });
}

/// `rp.materials()`: every merged material file and its layers.
fn materials(lower: &[(&str, &str)], higher: &[(&str, &str)]) -> (Yielded, PackRef, PackRef) {
    let (stack, low, high) = resource_stack(lower, higher);
    let mut out = Yielded::default();
    let group = stack.materials();
    for (path, merged) in &group.files {
        out.write(path.as_str(), path.as_str(), merged.value());
        out.layers(path.as_str(), merged);
    }
    out.findings = findings(&stack);
    (out, low, high)
}

#[test]
fn material_file() {
    audit(&Stack {
        family: "material_file",
        lower: Copy {
            path: "materials/test.material",
            text: r#"{"materials": {"version": "1.0.0", "low": {"zz_unknown": {"a": [1, "x"]}}, "both": {}}}"#,
            places: &[
                (
                    r#"{"materials": {"version": "1.0.0", "low": {"zz_probe": @}, "both": {}}}"#,
                    "materials/low/zz_probe",
                ),
                (
                    r#"{"materials": {"version": "1.0.0", "low": {"zz_probe": {"deep": {"n": @}}}, "both": {}}}"#,
                    "materials/low/zz_probe/deep/n",
                ),
                (
                    r#"{"materials": {"version": "1.0.0", "low": {}, "both": {"zz_probe": [0, @]}}}"#,
                    "materials/both/zz_probe/1",
                ),
            ],
        },
        higher: Copy {
            path: "materials/test.material",
            text: r#"{"materials": {"version": "1.0.0", "both": {"zz_unknown": 1}}}"#,
            places: &[(
                r#"{"materials": {"version": "1.0.0", "both": {"zz_probe": @}}}"#,
                "materials/both/zz_probe",
            )],
        },
        beside: &[],
        load: &materials,
    });
}

// --- the control: accessors whose job calls the entry `non_finite.rs` proves -------------------

const ENTITY_LOW: &str = r#"{"format_version": "1.21.60", "minecraft:entity": {"description": {"identifier": "demo:low", "is_spawnable": true, "is_summonable": true}, "components": {"minecraft:health": {"value": 10, "max": 10}}, "events": {"demo:e": {"add": {"component_groups": []}}}}}"#;
const ENTITY_HIGH: &str = r#"{"format_version": "1.21.60", "minecraft:entity": {"description": {"identifier": "demo:high", "is_spawnable": true, "is_summonable": true}, "components": {"minecraft:health": {"value": 10, "max": 10}}}}"#;

#[test]
fn control_entities() {
    let load = |lower: &[(&str, &str)], higher: &[(&str, &str)]| {
        let (stack, low, high) = behavior_stack(lower, higher);
        let mut out = Yielded::default();
        for (id, document) in stack.entities().iter() {
            out.write(&format!("{id:?}"), "entities/x.json", document.value());
        }
        out.findings = findings(&stack);
        (out, low, high)
    };
    audit(&Stack {
        family: "control: entities",
        lower: Copy {
            path: "entities/low.json",
            text: ENTITY_LOW,
            places: &[
                (
                    r#"{"format_version": "1.21.60", "minecraft:entity": {"description": {"identifier": "demo:low", "is_spawnable": true, "is_summonable": true}, "components": {"minecraft:health": {"value": 10, "max": 10}}, "zz_probe": @}}"#,
                    "minecraft:entity/zz_probe",
                ),
                (
                    r#"{"format_version": "1.21.60", "minecraft:entity": {"description": {"identifier": "demo:low", "is_spawnable": true, "is_summonable": true}, "components": {"minecraft:health": {"value": 10, "max": 10}}, "events": {"demo:e": {"zz_probe": {"deep": {"n": @}}}}}}"#,
                    "minecraft:entity/events/demo:e/zz_probe/deep/n",
                ),
            ],
        },
        higher: Copy {
            path: "entities/high.json",
            text: ENTITY_HIGH,
            places: &[(
                r#"{"format_version": "1.21.60", "minecraft:entity": {"description": {"identifier": "demo:high", "is_spawnable": true, "is_summonable": true}, "components": {"minecraft:health": {"value": 10, "max": 10, "zz_probe": [0, @]}}}}"#,
                "minecraft:entity/components/minecraft:health/zz_probe/1",
            )],
        },
        beside: &[],
        load: &load,
    });
}

#[test]
fn control_loot_tables() {
    let load = |lower: &[(&str, &str)], higher: &[(&str, &str)]| {
        let (stack, low, high) = behavior_stack(lower, higher);
        let mut out = Yielded::default();
        for (path, document) in stack.loot_tables().iter() {
            out.write(path.as_str(), path.as_str(), document.value());
        }
        out.findings = findings(&stack);
        (out, low, high)
    };
    let table =
        r#"{"pools": [{"rolls": 1, "entries": [{"type": "item", "name": "minecraft:stone"}]}]}"#;
    audit(&Stack {
        family: "control: loot_tables",
        lower: Copy {
            path: "loot_tables/low.json",
            text: table,
            places: &[
                (
                    r#"{"pools": [{"rolls": 1, "entries": [{"type": "item", "name": "minecraft:stone"}]}], "zz_probe": @}"#,
                    "zz_probe",
                ),
                (
                    r#"{"pools": [{"rolls": 1, "entries": [{"type": "item", "name": "minecraft:stone", "zz_probe": {"deep": {"n": @}}}]}]}"#,
                    "pools/0/entries/0/zz_probe/deep/n",
                ),
            ],
        },
        higher: Copy {
            path: "loot_tables/high.json",
            text: table,
            places: &[(
                r#"{"pools": [{"rolls": 1, "entries": [{"type": "item", "name": "minecraft:stone"}], "zz_probe": [0, @]}]}"#,
                "pools/0/zz_probe/1",
            )],
        },
        beside: &[],
        load: &load,
    });
}

#[test]
fn control_splashes() {
    let load = |lower: &[(&str, &str)], higher: &[(&str, &str)]| {
        let (stack, low, high) = resource_stack(lower, higher);
        let mut out = Yielded::default();
        out.write("splashes.json", "splashes.json", stack.splashes().value());
        out.findings = findings(&stack);
        (out, low, high)
    };
    audit(&Stack {
        family: "control: splashes",
        lower: Copy {
            path: "splashes.json",
            text: r#"{"canMerge": true, "splashes": ["a"], "low": {"kept": true}}"#,
            places: &[
                (
                    r#"{"canMerge": true, "splashes": ["a"], "low": {"kept": @}}"#,
                    "low/kept",
                ),
                (r#"{"canMerge": true, "splashes": ["a", @]}"#, "splashes/1"),
            ],
        },
        higher: Copy {
            path: "splashes.json",
            text: r#"{"canMerge": true, "splashes": ["b"]}"#,
            places: &[(
                r#"{"canMerge": true, "splashes": ["b"], "zz_probe": {"deep": {"n": @}}}"#,
                "zz_probe/deep/n",
            )],
        },
        beside: &[],
        load: &load,
    });
}

#[test]
fn control_ui_texture_infos() {
    let load = |lower: &[(&str, &str)], higher: &[(&str, &str)]| {
        let (stack, low, high) = resource_stack(lower, higher);
        let mut out = Yielded::default();
        for (path, document) in stack.ui_texture_infos().iter() {
            out.write(path.as_str(), path.as_str(), document.value());
        }
        out.findings = findings(&stack);
        (out, low, high)
    };
    let info = r#"{"nineslice_size": 1, "base_size": [4, 4]}"#;
    audit(&Stack {
        family: "control: ui_texture_infos",
        lower: Copy {
            path: "textures/ui/low.json",
            text: info,
            places: &[
                (
                    r#"{"nineslice_size": 1, "base_size": [4, 4], "zz_probe": @}"#,
                    "zz_probe",
                ),
                (
                    r#"{"nineslice_size": 1, "base_size": [4, 4], "zz_probe": {"deep": {"n": @}}}"#,
                    "zz_probe/deep/n",
                ),
            ],
        },
        higher: Copy {
            path: "textures/ui/high.json",
            text: info,
            places: &[(
                r#"{"nineslice_size": 1, "base_size": [4, 4], "zz_probe": [0, @]}"#,
                "zz_probe/1",
            )],
        },
        beside: &[],
        load: &load,
    });
}
