// The first probe itself: the judgement and the audit of one document type, and one test per
// type of `common/document_types.rs`. Included by `writable.rs`, `writable_2.rs`,
// `writable_3.rs` and `writable_4.rs`, each of which keeps its part of the types with
// `in_this_binary!`.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, Phase, Severity};
use bedrock_addon::json::Value;
use bedrock_addon::load::LoadContext;
use bedrock_addon::pack::PackType;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::{FormatVersion, FormatVersionInput};

use document_types::{beta, client, ddui, pack, poi};

use places::{
    Kind, Node, Outcome, code, content_in, kinds, leaves, lookup, nodes, non_strict_readings,
    number, outcome, parse, path_of, same_float, scalar, without,
};

/// The corpus files a family takes at most.
const FILES: usize = 40;
/// The numbers of one corpus file probed at most.
const NUMBERS_PER_FILE: usize = 200;
/// The places of one signature probed at most, over a family's corpus files.
const PER_SIGNATURE: usize = 2;
/// The bytes of probe text a family's corpus half loads at most (each text is loaded, its
/// written form loaded again).
const BUDGET: usize = 24 << 20;

/// What a number is replaced by.
const NUMBERS: &[&str] = &[
    "1e39",
    "-1e39",
    "1e999",
    "-1e999",
    "4294967296",
    "-2147483649",
    "0.5",
    "-1",
    "\"7\"",
    "true",
    "null",
];

/// The replacements a float member holds as the nearest `f32`, each with a second number that
/// is another `f32` (see `judge`).
const TWINS: &[(&str, &str)] = &[("4294967296", "4294968296"), ("-2147483649", "-2147484649")];

/// Places where the invariant does not hold for a recorded reason: the family, the file, the
/// end of the place's path (indices as `*`), the replacement (`*` for all) and the reason.
const EXEMPT: &[(&str, &str, &str, &str, &str)] = &[
    // A filter subject written as vanilla's number (`7`) is the subject of that number
    // (`damager`; vanilla's own re-serialised files write numbers) and is written
    // by its name: the same subject in the form the family writes.
    ("entity", "*", "/target", "7", "subject written by name"),
    // The same member under its other name: `subject` of a filter (16 places of
    // `villager_v2.json` alone, which the caps of the corpus sample left out).
    ("entity", "*", "/subject", "7", "subject written by name"),
    // A legacy quantity written as a text is a range (`"2-4"`, `"7"`), which the upgrade to
    // the newest format writes as `{"min": 7, "max": 7}`: the same quantity in the form
    // the newest format has.
    (
        "trade_table",
        "*",
        "/quantity",
        "\"7\"",
        "quantity written as a range",
    ),
    // A legacy group without `num_to_select` (a `null` is none) selects every trade, -1; the
    // newest format's default is 0, so the upgrade writes the -1.
    (
        "trade_table",
        "*",
        "groups/*/num_to_select",
        "null",
        "default selection count written",
    ),
    // The legacy particle parser stops at render parameters it cannot read, as the game
    // does; the components (curves, events) it does not get to are each noted where the file
    // has them (`DroppedField` at `particle_effect/components`): told at the member
    // that is lost, which this probe's rule of a finding ABOVE the place does not look at.
    (
        "particle",
        "particles/demo.particle.json",
        "description/basic_render_parameters",
        "7",
        "components after unreadable parameters are lost",
    ),
    (
        "particle",
        "particles/demo.particle.json",
        "description/basic_render_parameters/material",
        "7",
        "components after unreadable parameters are lost",
    ),
    // A cube of a file below 1.26.40 is written with the `pivot` the upgrade gives it, the
    // middle of the cube (`origin + size / 2`): a number beyond a float in `origin` or `size`
    // is held as the largest finite float and told at its element, and the pivot that then
    // moves is the cube's own, no value of the author's beside the member.
    (
        "geometry",
        "*",
        "cubes/*/origin/*",
        "1e39",
        "pivot given by the upgrade",
    ),
    (
        "geometry",
        "*",
        "cubes/*/origin/*",
        "-1e39",
        "pivot given by the upgrade",
    ),
    (
        "geometry",
        "*",
        "cubes/*/origin/*",
        "1e999",
        "pivot given by the upgrade",
    ),
    (
        "geometry",
        "*",
        "cubes/*/origin/*",
        "-1e999",
        "pivot given by the upgrade",
    ),
    (
        "geometry",
        "*",
        "cubes/*/size/*",
        "1e39",
        "pivot given by the upgrade",
    ),
    (
        "geometry",
        "*",
        "cubes/*/size/*",
        "-1e39",
        "pivot given by the upgrade",
    ),
    (
        "geometry",
        "*",
        "cubes/*/size/*",
        "1e999",
        "pivot given by the upgrade",
    ),
    (
        "geometry",
        "*",
        "cubes/*/size/*",
        "-1e999",
        "pivot given by the upgrade",
    ),
];

/// The families whose loaded document may hold entries the load refused with an ERROR and
/// keeps as written, each with its reason: the written text's load may give the errors the
/// text's own load gave, kind for kind (code and path, each known one counted once), and no
/// other. For every other family the written text loads without an error.
///
/// `sound_definitions` is not listed: its two findings of a
/// `sound_definitions` member that is no object are warnings of a document that loads.
const ERRORS_AS_READ: &[(&str, &str)] = &[("blocks_json", "refused entries kept as written")];

/// The families in which a `null` member is an absent member (an `Option` member
/// reads `null` as none, by the rule of the strict wrappers), so that a `null` put where a
/// number stood, written as no member, is the outcome "absent". The rule is a family's by
/// name: in a family not listed here a `null` that leaves no member and no finding is a
/// fault, and a family is added with the reader that makes it so. Listed are the
/// families whose typed structs read their members as `Option`s through facet; the others
/// refuse a `null`, hold it as written (a kept `Value`), or tell of it.
const NULL_IS_ABSENT: &[&str] = &[
    "atmosphere",
    "behavior_tree",
    "biome",
    "block",
    "block_cereal",
    "client_biome",
    "color_grading",
    "cubemap",
    "entity",
    "feature",
    "flipbook_textures",
    "geometry",
    "item",
    "item_beta",
    "lighting",
    "loot_table",
    "spawn_group",
    "spawn_rules",
    "trade_table",
    "water",
];

/// The families with a document that loads and is not written, for a recorded reason: the
/// family, its reason, and the phase of the warning the load gives at the path the writer
/// refuses at (`NotRepresentable`, `Unsupported`). The outcome is counted as "not written,
/// as the load says".
///
/// Item: a component-based document every component of which the load drops is an item
/// without one, and a text whose `components` name no component the game's routing test
/// knows is the legacy item format to the game: the document has no text. The load says so
/// at `components` in the validate phase (nothing is upgraded: the newest format has the
/// case too), where the writer refuses the item.
///
/// Fog and block are not listed: a block of an older format
/// that breaks a rule only the newest format has is written at its own format version, and a
/// fog file that breaks V15 or V17-V20 is refused at the load in every era (the older schemas
/// hold the same rules), so no text of either probe is left unwritten.
// The finding is matched by its code, its severity (the information of ours), its
// phase, its path and the start of its note, not by any information at the path.
const TOLD_NOT_WRITTEN: &[(&str, &str, Phase, &str, &str)] = &[(
    "item",
    "item without components",
    Phase::Validate,
    "InvalidValue",
    "no registered component is left",
)];

/// The families whose upgrade writes a Molang value given as `true` as the number it is, 1,
/// each with its reason. The outcome is "held": the written text has to hold exactly 1 at
/// the place.
///
/// Particle: the legacy parser takes a bool where a number or an expression stands; the
/// 1.20.80 reader does not, so the step to it writes the bool as its
/// number. The same value in the form the newest format has.
const BOOL_AS_NUMBER: &[(&str, &str)] = &[("particle", "bool written as number")];

/// The families whose documents below a format version are read by the game's non-strict
/// reader, with that version: a bool and a number read as each other, an integer token by its
/// low bits, a real by its truncation toward zero, silently. The outcome is "held" when the
/// written text holds one of those readings at the place (see [`non_strict_readings`]).
///
/// Block: the block loader reads the cereal-registered
/// descriptions of a document below 1.26.20 with the game's non-strict schema reader
/// (bool, unsigned and float reads); no log on a conversion.
const NON_STRICT_BELOW: &[(&str, FormatVersion)] = &[("block", FormatVersion::new(1, 26, 20))];

// --- the judgement ---------------------------------------------------------------------------

/// The unchanged document: what its load found and the scalars of what it writes.
struct Base {
    findings: Vec<String>,
    /// Each finding in full: its code with its text, its path and its detail.
    texts: Vec<String>,
    leaves: Vec<String>,
    /// What the unchanged document is written as.
    value: Value,
}

/// The scalars of `value` outside the member at `path`; `None` when it has no such member.
fn leaves_beside(value: &Value, path: &[String]) -> Option<Vec<String>> {
    fn walk(value: &Value, path: &[String], out: &mut Vec<String>) -> bool {
        let Some((first, rest)) = path.split_first() else {
            return true;
        };
        let mut found = false;
        if let Some(object) = value.as_object() {
            for (key, member) in object {
                if key.as_str() == first && !found {
                    found = walk(member, rest, out);
                } else {
                    leaves(member, out);
                }
            }
        } else if let Some(array) = value.as_array() {
            let at = first.parse::<usize>().ok();
            for (index, element) in array.as_slice().iter().enumerate() {
                if Some(index) == at {
                    found = walk(element, rest, out);
                } else {
                    leaves(element, out);
                }
            }
        }
        found
    }
    let mut out = Vec::new();
    walk(value, path, &mut out).then_some(out)
}

/// What the written text loses BESIDE the member a finding is at: the scalars the
/// unchanged document is written with outside the member that `value` is written with fewer
/// of. An element of a list is told of as the list is: a list is one value.
fn lost_beside(base: &Base, node: &Node, value: &Value) -> Vec<String> {
    let elements = node.index.iter().rev().take_while(|index| **index).count();
    let unit = &node.path[..node.path.len() - elements];
    let (Some(before), Some(mut after)) =
        (leaves_beside(&base.value, unit), leaves_beside(value, unit))
    else {
        // The member is at another place in one of the texts, or in none: not judged.
        return Vec::new();
    };
    // The version a family writes is no value the written text owes the author.
    let version = base
        .value
        .as_object()
        .and_then(|root| root.get("format_version"))
        .and_then(scalar);
    let mut lost: Vec<String> = before
        .into_iter()
        .filter(|leaf| version.as_ref() != Some(leaf))
        .filter(|leaf| match after.iter().position(|held| held == leaf) {
            Some(index) => {
                after.swap_remove(index);
                false
            }
            None => true,
        })
        .collect();
    lost.sort();
    lost.dedup();
    lost
}

/// A finding as one text: its code (with its message), path and detail.
fn finding_text(finding: &bedrock_addon::diagnostic::Diagnostic) -> String {
    format!(
        "{:?} at `{}`: {:?}",
        finding.code, finding.location.json_path, finding.detail
    )
}

/// What reading a text and writing it back gave: the written value, or the outcome's name
/// (`Ok`) or the fault (`Err`) that ends the judgement.
fn written(
    family: &str,
    read: &dyn Fn(&str) -> Outcome,
    text: &str,
) -> Result<(Outcome, Value), Result<&'static str, String>> {
    let ours = read(text);
    if !ours.loaded {
        return Err(Ok("refused"));
    }
    let written = match &ours.written {
        Ok(written) => written.clone(),
        Err(error) => {
            if error.contains("what: Unsupported")
                && TOLD_NOT_WRITTEN
                    .iter()
                    .filter(|(of, _, _, _, _)| *of == family)
                    .any(|(_, _, phase, code, note)| {
                        ours.findings.iter().any(|finding| {
                            // The load's own word that the newest format does not hold
                            // it: an information of ours.
                            finding.severity == Severity::Info
                                && finding.code.name() == *code
                                && matches!(
                                    finding.detail.as_deref(),
                                    Some(bedrock_addon::diagnostic::DiagnosticDetail::Note(text))
                                        if text.starts_with(note)
                                )
                                && finding.phase == *phase
                                && error.contains(&format!("at: {:?},", finding.location.json_path))
                        })
                    })
            {
                return Err(Ok("not written, as the load says"));
            }
            return Err(Err(format!(
                "loads and is not written: {error}; findings {:?}",
                kinds(&ours.findings)
            )));
        }
    };
    let again = read(&written);
    // An error of the written text's load that the load of the text itself did not give. (A
    // document that loads has no error, but for a file that holds entries of its
    // own: `blocks.json` refuses an entry with an error and keeps it as written, and
    // the written file says of that entry what the read one said.)
    // For that file type alone: every other family's written text loads without any error.
    // A kept error of the read is the game's line after which it keeps the
    // document; the written text keeps what was read, so its load says it again.
    let mut known = if ERRORS_AS_READ.iter().any(|(of, _)| *of == family) {
        kinds(&ours.findings)
    } else {
        let kept: Vec<Diagnostic> = ours.findings.iter().filter(|d| d.kept).cloned().collect();
        kinds(&kept)
    };
    // A kept error of the written text's load with the game's text of a kept error of the
    // read is the same line of the game's, wherever the crate places it: the written text
    // drops a value the game reads as absent (a loot pool's `entries` that is no array and
    // the missing member are one test and one line).
    let mut kept_lines: Vec<String> = ours.findings.iter().filter_map(kept_line).collect();
    let errors: Vec<Diagnostic> = again
        .findings
        .iter()
        .filter(|d| d.severity >= Severity::Error)
        .filter(|d| {
            let kind = format!("{} at `{}`", code(d), d.location.json_path);
            if let Some(index) = known.iter().position(|known| *known == kind) {
                known.swap_remove(index);
                return false;
            }
            if let Some(line) = kept_line(d)
                && let Some(index) = kept_lines.iter().position(|kept| *kept == line)
            {
                kept_lines.swap_remove(index);
                return false;
            }
            true
        })
        .cloned()
        .collect();
    if !again.loaded || !errors.is_empty() {
        return Err(Err(format!(
            "the written text does not load without an error: {:?}\n{written}",
            kinds(&errors)
        )));
    }
    if again.written.as_ref() != Ok(&written) {
        return Err(Err(format!(
            "the written text is not what its own reading writes; its findings: {:?}; first difference: {}",
            kinds(&again.findings),
            difference(
                &written,
                again.written.as_deref().unwrap_or("(not written)")
            )
        )));
    }
    let Some(value) = parse(&written) else {
        return Err(Err(format!("the written text is no JSON:\n{written}")));
    };
    Ok((ours, value))
}

/// Where two texts first differ: some characters of each from there.
fn difference(one: &str, other: &str) -> String {
    let same = one
        .bytes()
        .zip(other.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    let from = one[..same.min(one.len())]
        .char_indices()
        .rev()
        .nth(40)
        .map_or(0, |(at, _)| at);
    let part = |text: &str| -> String { text[from..].chars().take(160).collect() };
    format!("`{}` then `{}`", part(one), part(other))
}

/// Whether the written text holds the replacement at the place: as it is, as the `1` a
/// number family gives a `true`, as a reading below a family's strict version, or as the
/// nearest `f32` of a float member.
fn held_at_the_place(
    family: &str,
    read: &dyn Fn(&str) -> Outcome,
    template: (&str, &str),
    node: &Node,
    replacement: &str,
    value: &Value,
) -> bool {
    let text = format!("{}{replacement}{}", template.0, template.1);
    let at = lookup(value, &node.path);
    let expected = parse(replacement).and_then(|value| scalar(&value));
    let expected = expected.expect("a replacement is a scalar");
    if at.and_then(scalar).as_ref() == Some(&expected) {
        return true;
    }
    if replacement == "true"
        && BOOL_AS_NUMBER.iter().any(|(of, _)| *of == family)
        && at.and_then(number) == Some(1.0)
    {
        return true;
    }
    if let Some((_, below)) = NON_STRICT_BELOW.iter().find(|(of, _)| *of == family)
        && parse(&text)
            .as_ref()
            .and_then(Value::as_object)
            .and_then(|root| root.get("format_version"))
            .and_then(FormatVersionInput::from_json)
            .and_then(|input| input.version)
            .is_some_and(|version| version < *below)
        && let Some(held) = at.and_then(number)
        && non_strict_readings(replacement).contains(&held)
    {
        return true;
    }
    // A float member holds the nearest `f32` and writes its shortest spelling
    // (`4294967300.0` for `4294967296`). That is told from an integer member saturating at
    // a bound of its type (which is the same `f32`) by a second number 1000 further out: a
    // float holds it as another `f32`, a saturated integer as the same bound.
    if let Some((_, twin)) = TWINS.iter().find(|(of, _)| *of == replacement)
        && let Some(held) = at.and_then(number)
        && same_float(held, replacement)
        && let Ok((_, value)) =
            written(family, read, &format!("{}{twin}{}", template.0, template.1))
        && let Some(other) = lookup(&value, &node.path).and_then(number)
        && same_float(other, twin)
        && !same_float(other, replacement)
    {
        return true;
    }
    false
}

/// Whether the findings of a load of the written text tell of the replacement: at the place,
/// at a member above it, or at one the load drops whole. `Err` when they tell of it at the
/// member alone and the written text loses what stood beside it.
fn told_by_findings(
    base: &Base,
    node: &Node,
    ours: &Outcome,
    value: &Value,
) -> Result<Option<&'static str>, String> {
    // A finding at the place; or one the unchanged document does not have at an enclosing
    // member.
    let mut known = base.findings.clone();
    let mut known_texts = base.texts.clone();
    let (mut said_here, mut said_above) = (false, false);
    for finding in &ours.findings {
        let at = path_of(finding);
        // At the place: said, unless the unchanged document has the very finding there
        // (the same code, text and path), which then says nothing of the replacement.
        let text = finding_text(finding);
        let unchanged = known_texts.iter().position(|known| *known == text);
        if let Some(index) = unchanged {
            known_texts.swap_remove(index);
        }
        if at == node.path && unchanged.is_none() {
            said_here = true;
            continue;
        }
        let kind = format!("{} at `{}`", code(finding), finding.location.json_path);
        if let Some(index) = known.iter().position(|known| *known == kind) {
            known.swap_remove(index);
        } else if at.starts_with(&node.path) {
            // Below the place, where the family names the member of the long form a short
            // form stands for (`max_stack_size/value`).
            said_here = true;
        } else if !at.is_empty() && node.path.starts_with(&at) {
            // At an enclosing member.
            said_above = true;
        }
    }
    if said_above {
        return Ok(Some("said"));
    }
    if said_here {
        // Told at the member alone: what the written text loses BESIDE the member (a
        // component that goes whole for one member of another type) must be told above it.
        let lost = lost_beside(base, node, value);
        if !lost.is_empty() {
            return Err(format!(
                "told at the member alone, and the written text loses {lost:?} beside it; findings: {:?}",
                kinds(&ours.findings)
            ));
        }
        return Ok(Some("said"));
    }
    // Dropped with an enclosing member the load says it drops: a finding at a member above
    // the place that the written text does not have (a loot condition the game does not
    // register is skipped whole, with its warning, whatever its members hold).
    if ours.findings.iter().any(|finding| {
        let at = path_of(finding);
        !at.is_empty()
            && at.len() < node.path.len()
            && node.path.starts_with(&at)
            && lookup(value, &at).is_none()
    }) {
        return Ok(Some("said"));
    }
    Ok(None)
}

/// What became of `text`, which has `replacement` at `node`.
fn judge(
    family: &str,
    read: &dyn Fn(&str) -> Outcome,
    base: &Base,
    template: (&str, &str),
    node: &Node,
    replacement: &str,
) -> Result<&'static str, String> {
    let text = format!("{}{replacement}{}", template.0, template.1);
    let (ours, value) = match written(family, read, &text) {
        Ok(read) => read,
        Err(done) => return done,
    };
    let expected = parse(replacement).and_then(|value| scalar(&value));
    let expected = expected.expect("a replacement is a scalar");
    let at = lookup(&value, &node.path);
    if held_at_the_place(family, read, template, node, replacement, &value) {
        return Ok("held");
    }
    let null_is_absent = replacement == "null" && NULL_IS_ABSENT.contains(&family);
    if null_is_absent
        && node.index.last() == Some(&false)
        && at.is_none()
        && lookup(&value, &node.path[..node.path.len() - 1]).is_some_and(Value::is_object)
    {
        // Absent by the member, not by the family: the `null` is read as the
        // document WITHOUT the member is read. A member whose `null` is written as
        // another text than its absence (a component dropped whole for it, a default put
        // elsewhere) is no absent member, and falls through to the other outcomes.
        let null_at = Node {
            end: node.start + replacement.len(),
            ..node.clone()
        };
        let as_absent = without(&text, &null_at)
            .and_then(|absent| written(family, read, &absent).ok())
            .is_some_and(|(_, absent)| absent == value);
        if as_absent {
            return Ok("absent");
        }
    }
    if let Some(told) = told_by_findings(base, node, &ours, &value)? {
        return Ok(told);
    }
    // Held at another place: a scalar the unchanged document's written text has not.
    let mut ours_leaves = Vec::new();
    leaves(&value, &mut ours_leaves);
    let mut left = base.leaves.clone();
    for leaf in &ours_leaves {
        match left.iter().position(|known| known == leaf) {
            Some(index) => {
                left.swap_remove(index);
            }
            None if *leaf == expected => return Ok("held elsewhere"),
            None => {}
        }
    }
    // Held at another place by a float member, as the nearest `f32`: told from a saturated
    // integer by the second number, as at the place itself.
    if let Some((_, twin)) = TWINS.iter().find(|(of, _)| *of == replacement)
        && fresh(base, &value)
            .iter()
            .any(|held| same_float(*held, replacement))
        && let Ok((_, value)) =
            written(family, read, &format!("{}{twin}{}", template.0, template.1))
        && fresh(base, &value)
            .iter()
            .any(|other| same_float(*other, twin) && !same_float(*other, replacement))
    {
        return Ok("held elsewhere");
    }
    // A `null` whose member an upgrade step moved: nothing the unchanged document does not
    // write is written, so the member is absent wherever it would be.
    if null_is_absent && node.index.last() == Some(&false) && left.len() < base.leaves.len() {
        let mut new = ours_leaves.clone();
        for leaf in &base.leaves {
            if let Some(index) = new.iter().position(|known| known == leaf) {
                new.swap_remove(index);
            }
        }
        if new.is_empty() {
            return Ok("absent");
        }
    }
    let now = match at {
        None => "nothing".to_owned(),
        Some(at) => bedrock_addon::json::to_json(at)
            .unwrap_or_else(|error| format!("{error:?}"))
            .chars()
            .take(120)
            .collect(),
    };
    Err(format!(
        "the written text holds {now} there and no finding is at the place; findings: {:?}",
        kinds(&ours.findings)
    ))
}

/// The numbers of `value` the unchanged document's written text has not.
fn fresh(base: &Base, value: &Value) -> Vec<f64> {
    let mut all = Vec::new();
    leaves(value, &mut all);
    let mut left = base.leaves.clone();
    all.into_iter()
        .filter(|leaf| match left.iter().position(|known| known == leaf) {
            Some(index) => {
                left.swap_remove(index);
                false
            }
            None => true,
        })
        .filter_map(|leaf| leaf.strip_prefix("n:")?.parse().ok())
        .collect()
}

/// Whether a place is exempt (see [`EXEMPT`]).
fn exempt(family: &str, file: &str, node: &Node, replacement: &str) -> bool {
    let starred = node.starred().join("/");
    EXEMPT.iter().any(|(of, in_file, end, with, _entry)| {
        *of == family
            && (*in_file == "*" || *in_file == file)
            && starred.ends_with(end)
            && (*with == "*" || *with == replacement)
    })
}

/// The counts of a family's test.
#[derive(Default)]
struct Tally {
    texts: usize,
    outcomes: BTreeMap<&'static str, usize>,
    /// Places the caps left out.
    capped: usize,
    faults: Vec<String>,
    /// The faults by signature and replacement.
    kinds: BTreeMap<String, usize>,
    /// Member names the wrong-shape probes have seen.
    names: BTreeSet<String>,
    /// Places probed per signature.
    signatures: HashMap<String, usize>,
    /// Bytes of probe text left.
    budget: usize,
    /// Corpus files the family does not load as they are.
    unread: usize,
    /// The warnings and errors of the corpus files as they are, by code.
    vanilla: BTreeMap<String, usize>,
}

/// The unchanged document written once; `None`, with the fault noted, when that fails. Warnings
/// and errors of a corpus file (`capped`) are counted by code.
fn base_of(
    family: &str,
    file: &str,
    read: &dyn Fn(&str) -> Outcome,
    document: &str,
    capped: bool,
    tally: &mut Tally,
) -> Option<Base> {
    match written(family, read, document) {
        Ok((ours, value)) => {
            if capped {
                for finding in &ours.findings {
                    if finding.severity >= Severity::Warning {
                        let first =
                            tally.vanilla.is_empty() || !tally.vanilla.contains_key(&code(finding));
                        *tally.vanilla.entry(code(finding)).or_default() += 1;
                        if first {
                            println!(
                                "writable-vanilla: {family} | the first {} | {file} | {}",
                                code(finding),
                                finding.location.json_path
                            );
                        }
                    }
                }
            }
            let mut all = Vec::new();
            leaves(&value, &mut all);
            Some(Base {
                findings: kinds(&ours.findings),
                texts: ours.findings.iter().map(finding_text).collect(),
                leaves: all,
                value,
            })
        }
        Err(done) => {
            tally.faults.push(format!(
                "{family} | {file} | (the unchanged document) | {}",
                done.err()
                    .unwrap_or_default()
                    .chars()
                    .take(400)
                    .collect::<String>()
            ));
            *tally
                .kinds
                .entry("(the unchanged document)".to_owned())
                .or_default() += 1;
            None
        }
    }
}

/// Probes one document; `capped` for a corpus file.
fn probe(
    family: &str,
    file: &str,
    read: &dyn Fn(&str) -> Outcome,
    document: &str,
    capped: bool,
    tally: &mut Tally,
) {
    let plain = read(document);
    if !plain.loaded {
        assert!(
            capped,
            "{family}: the document does not load: {:?}",
            kinds(&plain.findings)
        );
        if tally.unread == 0 {
            println!(
                "writable-unread: {family} | {file} | {:?}",
                kinds(&plain.findings).iter().take(3).collect::<Vec<_>>()
            );
        }
        tally.unread += 1;
        return;
    }
    let Some(base) = base_of(family, file, read, document, capped, tally) else {
        return;
    };
    let mut numbers = 0;
    for node in nodes(document) {
        let replacements: Vec<&str> = match node.kind {
            Kind::Number => {
                let token = &document[node.start..node.end];
                let integer = !token.contains(['.', 'e', 'E']);
                let spelled = token.parse::<f64>().ok();
                NUMBERS
                    .iter()
                    .copied()
                    .filter(|with| *with != "0.5" || integer)
                    .filter(|with| with.parse::<f64>().ok() != spelled || spelled.is_none())
                    .collect()
            }
            Kind::String => vec!["7"],
            Kind::Object | Kind::Array if !node.path.is_empty() => vec!["7"],
            // A bool (not a `null`, which is no value to replace).
            Kind::Other if &document[node.start..node.end] != "null" => vec!["7"],
            Kind::Object | Kind::Array | Kind::Other => continue,
        };
        if node.kind == Kind::Number {
            if capped {
                numbers += 1;
                let seen = tally.signatures.entry(node.signature()).or_default();
                *seen += 1;
                if numbers > NUMBERS_PER_FILE || *seen > PER_SIGNATURE {
                    tally.capped += 1;
                    continue;
                }
            }
        } else if capped
            && !tally
                .names
                .insert(format!("{:?} {}", node.kind, node.name()))
        {
            // A corpus file: once per member name of the family. A small document is
            // probed at every place.
            continue;
        }
        let template = (&document[..node.start], &document[node.end..]);
        for replacement in replacements {
            if exempt(family, file, &node, replacement) {
                *tally.outcomes.entry("exempt").or_default() += 1;
                continue;
            }
            if capped {
                if tally.budget < document.len() {
                    tally.capped += 1;
                    continue;
                }
                tally.budget -= document.len();
            }
            tally.texts += 1;
            match judge(family, read, &base, template, &node, replacement) {
                Ok(outcome) => *tally.outcomes.entry(outcome).or_default() += 1,
                Err(fault) => {
                    let fault: String = fault.chars().take(400).collect();
                    *tally
                        .kinds
                        .entry(format!("{} <- {replacement}", node.signature()))
                        .or_default() += 1;
                    tally.faults.push(format!(
                        "{family} | {file} | {} | {replacement} | {fault}",
                        node.shown()
                    ));
                }
            }
        }
    }
}

// --- more small documents --------------------------------------------------------------------

/// More small documents of a family, probed in full like the one of the list: the members
/// whose faults this probe found in the vanilla files, so that they are probed on a machine
/// without the corpus.
fn more(family: &str) -> &'static [&'static str] {
    match family {
        "item" => &[
            r#"{"format_version": "1.21.0", "minecraft:item": {"description": {"identifier": "demo:y"}, "components": {"minecraft:cooldown": {"category": "x", "duration": 1.5}, "minecraft:food": {"nutrition": 4, "saturation_modifier": 0.6}, "minecraft:use_modifiers": {"use_duration": 1.6, "movement_modifier": 0.35}, "minecraft:icon": "stick", "minecraft:display_name": {"value": "Y"}, "minecraft:tags": {"tags": ["a"]}, "minecraft:durability": {"max_durability": 10}, "minecraft:throwable": {"launch_power_scale": 1.5, "max_launch_power": 2.5}}}}"#,
            r#"{"format_version": "1.10.0", "minecraft:item": {"description": {"identifier": "demo:z"}, "components": {"minecraft:max_damage": 10, "minecraft:use_duration": 32, "minecraft:food": {"nutrition": 2, "saturation_modifier": "low", "using_converts_to": "bowl", "effects": [{"name": "regeneration", "chance": 1.0, "duration": 5, "amplifier": 1}]}, "minecraft:seed": {"crop_result": "wheat"}}}}"#,
        ],
        "trade_table" => &[
            r#"{"tiers": [{"total_exp_required": 0, "groups": [{"num_to_select": 1, "trades": [{"wants": [{"item": "minecraft:emerald", "quantity": {"min": 1, "max": 3}, "price_multiplier": 0.05}], "gives": [{"choice": [{"item": "minecraft:book", "quantity": 2, "functions": [{"function": "enchant_book_for_trading", "base_cost": 2, "base_random_cost": 5, "per_level_random_cost": 10, "per_level_cost": 3}]}, {"item": "minecraft:bread", "filters": {"test": "is_biome", "value": 4}}]}], "trader_exp": 2, "max_uses": 12, "reward_exp": true, "weight": 4}, {"wants": [{"item": "minecraft:emerald"}], "gives": [{"item": "minecraft:cookie", "quantity": [2, 4]}]}]}]}]}"#,
        ],
        "animation" => &[
            r#"{"format_version": "1.8.0", "animations": {"animation.y": {"animation_length": 1.5, "loop": true, "bones": {"b": {"rotation": [0, 90, 0]}}}}}"#,
        ],
        "animation_controller" => &[
            r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.y": {"states": {"default": {"blend_transition": 0.2}, "curve": {"blend_transition": {"a": 0.5}}}}}}"#,
        ],
        "fog" => &[
            r##"{"format_version": "1.21.90", "minecraft:fog_settings": {"description": {"identifier": "demo:y"}, "distance": {"water": {"fog_start": 0.5, "fog_end": 60, "fog_color": "#44AFF5", "render_distance_type": "fixed", "transition_fog": {"init_fog": {"fog_start": 0.5, "fog_end": 2, "fog_color": [1, 1, 1], "render_distance_type": "fixed"}, "min_percent": 0.25, "mid_seconds": 5, "mid_percent": 0.6, "max_seconds": 30}}}, "volumetric": {"density": {"air": {"max_density": 0.1, "max_density_height": 56, "zero_density_height": 70}}, "media_coefficients": {"air": {"scattering": [0.02, 0.02, 0.02], "absorption": [0, 0, 0]}}}}}"##,
        ],
        "geometry" => &[
            r#"{"format_version": "1.8.0", "geometry.y:geometry.z": {"visible_bounds_width": 2, "visible_bounds_height": 3, "visible_bounds_offset": [0, 1.5, 0], "texturewidth": 64, "textureheight": 32, "bones": [{"name": "b", "pivot": [0, 24, 0], "cubes": [{"origin": [-3, 21, -3], "size": [6, 6, 6], "uv": [8, 4]}]}]}}"#,
            r#"{"format_version": "1.8.0", "geometry.y": {"visible_bounds_width": 2, "visible_bounds_height": 3, "visible_bounds_offset": [0, 1.5, 0], "texturewidth": 64, "textureheight": 32, "bones": [{"name": "b", "pivot": [0, 24, 0], "cubes": [{"origin": [-3, 21, -3], "size": [6, 6, 6], "uv": [8, 4]}]}]}}"#,
            r#"{"format_version": "1.12.0", "minecraft:geometry": [{"description": {"identifier": "geometry.y", "texture_width": 64, "texture_height": 32, "visible_bounds_width": 2, "visible_bounds_height": 3, "visible_bounds_offset": [0, 1.5, 0]}, "bones": [{"name": "b", "pivot": [0, 24, 0], "cubes": [{"origin": [-3, 21, -3], "size": [6, 6, 6], "uv": [8, 4], "inflate": 0.25}]}]}]}"#,
        ],
        "spawn_rules" => &[
            r#"{"format_version": "1.8.0", "minecraft:spawn_rules": {"description": {"identifier": "demo:y", "population_control": "animal"}, "conditions": [{"minecraft:weight": {"default": 8, "rarity": 2}, "minecraft:density_limit": {"surface": 4, "underground": 3}, "minecraft:brightness_filter": {"min": 7, "max": 15, "adjust_for_weather": false}, "minecraft:height_filter": {"min": 60, "max": 70}, "minecraft:distance_filter": {"min": 12, "max": 32}, "minecraft:herd": {"min_size": 2, "max_size": 3, "event": "e", "event_skip_count": 2, "initial_event": "i", "initial_event_count": 1}, "minecraft:permute_type": [{"weight": 10, "entity_type": "demo:z", "guaranteed_count": 1}], "minecraft:world_age_filter": {"min": 6, "max": 9}, "minecraft:delay_filter": {"min": 1, "max": 2, "identifier": "demo:d", "spawn_chance": 50}, "minecraft:spawns_above_block_filter": {"blocks": "minecraft:stone", "distance": 10}}, {"minecraft:herd": [{"min_size": 2, "max_size": 4}]}]}}"#,
        ],
        "spawn_group" => &[
            r#"{"format_version": "1.8.0", "minecraft:spawn_groups": {"description": {"identifier": "demo:y", "population_control": "monster"}, "conditions": [{"minecraft:weight": {"default": 8, "rarity": 2}, "minecraft:distance_filter": {"min": 12, "max": 32}, "minecraft:herd": {"min_size": 2, "max_size": 3}, "minecraft:permute_type": [{"weight": 10, "entity_type": "demo:z"}]}]}}"#,
        ],
        "block" => &[
            r#"{"format_version": "1.21.60", "minecraft:block": {"description": {"identifier": "demo:y"}, "components": {"minecraft:selection_box": {"origin": [-8, 0, -8], "size": [16, 16, 16]}, "minecraft:light_emission": 7, "minecraft:destructible_by_explosion": {"explosion_resistance": 3}, "minecraft:friction": 0.4}}}"#,
        ],
        "entity" => &[
            r#"{"format_version": "1.21.0", "minecraft:entity": {"description": {"identifier": "demo:y", "is_spawnable": true, "is_summonable": true}, "components": {"minecraft:interact": {"interactions": {"hurt_item": 3, "swing": true, "play_sounds": "eat", "interact_text": "t"}}, "minecraft:health": {"value": 4, "max": 6}}}}"#,
        ],
        "loot_table" => &[
            r#"{"pools": [{"rolls": 1, "entries": [{"type": "item", "name": "minecraft:stick", "weight": 3, "quality": 2, "functions": [{"function": "set_count", "count": {"min": 1, "max": 3}}, {"function": "set_potion", "id": "4"}]}]}]}"#,
        ],
        "feature" => &[
            r#"{"format_version": "1.13.0", "minecraft:scatter_feature": {"description": {"identifier": "demo:y"}, "places_feature": "demo:z", "iterations": 4, "scatter_chance": 25, "x": {"distribution": "uniform", "extent": [0, 15]}, "y": 3, "z": 5}}"#,
            r#"{"format_version": "1.13.0", "minecraft:single_block_feature": {"description": {"identifier": "demo:y"}, "places_block": {"name": "minecraft:cocoa", "states": {"age": 2}}, "enforce_placement_rules": true, "enforce_survivability_rules": true}}"#,
        ],
        "manifest" => &[
            r#"{"format_version": 2, "header": {"name": "n", "description": "d", "uuid": "b9c7b5a2-55a0-4a40-8e6b-1d4a2bd2f3c1", "version": [1, 2, 3], "min_engine_version": [1, 20, 0]}, "modules": [{"type": "data", "uuid": "0d1a2f6f-7f5c-4d4e-9a58-0c6a9d7b6e11", "version": [4, 5, 6]}], "dependencies": [{"uuid": "5f8d0c8e-3d0b-4b53-8f3a-7d7b6f6f2a10", "version": [7, 8, 9]}]}"#,
        ],
        _ => &[],
    }
}

// --- the corpus ------------------------------------------------------------------------------

/// Where a family's vanilla files are, below the corpus root: a directory (every `.json` and
/// `.material` file below it) or a file, each with the pack root its paths are relative to.
#[expect(
    clippy::too_many_lines,
    reason = "one match over the table of the vanilla files of each family: no seam to split at"
)]
fn sources(family: &str) -> &'static [(&'static str, &'static str)] {
    const BP: &str = "bedrock-samples/behavior_pack";
    const RP: &str = "bedrock-samples/resource_pack";
    const GDK_BP: &str = "gdk-retail/behavior_packs/vanilla";
    const GDK_RP: &str = "gdk-retail/resource_packs/vanilla";
    const DEFINITIONS: &str = "gdk-retail/definitions";
    const DDUI_RP: &str = "gdk-retail/resource_packs/vanilla_1.21.130";
    const PERSONA: &str = "gdk-retail/resource_packs/persona";
    match family {
        "block" => &[(BP, "blocks")],
        "voxel_shape" => &[(BP, "shapes")],
        "item" => &[(BP, "items")],
        "crafting_catalog" => &[(GDK_BP, "item_catalog")],
        "entity" => &[(BP, "entities")],
        "spawn_rules" => &[(BP, "spawn_rules")],
        "spawn_group" => &[(DEFINITIONS, "spawn_groups")],
        "biome" => &[(BP, "biomes")],
        "feature" => &[(DEFINITIONS, "features")],
        "feature_rule" => &[(DEFINITIONS, "feature_rules")],
        "loot_table" => &[(BP, "loot_tables")],
        "recipe" => &[(BP, "recipes")],
        "trade_table" => &[(BP, "trading")],
        "camera_preset" => &[(GDK_BP, "cameras/presets")],
        "aim_assist_preset" => &[(GDK_BP, "aim_assist/presets")],
        "aim_assist_categories" => &[(GDK_BP, "aim_assist/categories")],
        "behavior_tree" => &[(GDK_BP, "behavior_trees")],
        "manifest" => &[(BP, "manifest.json"), (RP, "manifest.json")],
        "blocks_json" => &[(RP, "blocks.json")],
        "attachable" => &[(RP, "attachables")],
        "client_entity" => &[(RP, "entity")],
        "client_biome" => &[(RP, "biomes")],
        "biomes_client" => &[(RP, "biomes_client.json")],
        "particle" => &[(RP, "particles")],
        "animation" => &[(RP, "animations")],
        "animation_controller" => &[(RP, "animation_controllers")],
        "geometry" => &[(RP, "models")],
        "render_controller" => &[(RP, "render_controllers")],
        "material_file" => &[(GDK_RP, "materials")],
        "fog" => &[(RP, "fogs")],
        "atmosphere" => &[(RP, "atmospherics")],
        "color_grading" => &[(RP, "color_grading")],
        "lighting" => &[(RP, "lighting")],
        "water" => &[(RP, "water")],
        "terrain_texture" => &[
            (RP, "textures/terrain_texture.json"),
            (RP, "textures/item_texture.json"),
        ],
        "flipbook_textures" => &[(RP, "textures/flipbook_textures.json")],
        "textures_list" => &[(RP, "textures/textures_list.json")],
        "sound_definitions" => &[(RP, "sounds/sound_definitions.json")],
        "sounds" => &[(RP, "sounds.json")],
        "music_definitions" => &[(RP, "sounds/music_definitions.json")],
        "ui_file" => &[(RP, "ui")],
        "ui_defs" => &[(RP, "ui/_ui_defs.json")],
        "ui_global_variables" => &[(RP, "ui/_global_variables.json")],
        "splashes" => &[(GDK_RP, "splashes.json")],
        "loading_messages" => &[(GDK_RP, "loading_messages.json")],
        "credits" => &[("gdk-retail", "credits/credits.json")],
        "font_metadata" => &[(GDK_RP, "font/font_metadata.json")],
        "emoticons" => &[(GDK_RP, "font/emoticons.json")],
        "ui_texture_info" => &[(RP, "textures/ui")],
        "ddui_root" => &[
            (DDUI_RP, "ddui/root"),
            (
                "gdk-dev/test/assets/OreUIDataDrivenScreenTests/TestRP",
                "ddui/root",
            ),
        ],
        "skin_pack" => &[
            ("gdk-retail/skin_packs/vanilla", "skins.json"),
            ("gdk-retail/skin_packs/custom", "skins.json"),
            ("gdk-retail/skin_packs/persona", "skins.json"),
        ],
        // A piece's files lie side by side in its folder and a source that is a directory
        // takes every `.json` below it, so the two types name their files one by one: of
        // the 66 metadata files the eleven that between them have every member the vanilla
        // pieces write, and every fifth of the others; every sixth of the 58 store files.
        "persona_piece_meta" => &[
            (PERSONA, "pieces/alex_hair/alex_hair.meta.json"),
            (PERSONA, "pieces/alex_pants/alex_pants.meta.json"),
            (PERSONA, "pieces/ari_pants/ari_pants.meta.json"),
            (PERSONA, "pieces/body/medium/medium.meta.json"),
            (
                PERSONA,
                "pieces/body/medium_skeleton/medium_skeleton.meta.json",
            ),
            (PERSONA, "pieces/body/small/small.meta.json"),
            (PERSONA, "pieces/body/smaller/smaller.meta.json"),
            (
                PERSONA,
                "pieces/body/smaller_skeleton/smaller_skeleton.meta.json",
            ),
            (PERSONA, "pieces/efe_hair/efe_hair.meta.json"),
            (PERSONA, "pieces/efe_skin/efe_skin.meta.json"),
            (
                PERSONA,
                "pieces/emote_followMe/communication_follow_me.meta.json",
            ),
            (
                PERSONA,
                "pieces/emote_overThere/communication_point.meta.json",
            ),
            (PERSONA, "pieces/emote_simpleClap/basic_clap.meta.json"),
            (PERSONA, "pieces/kai_shoes/kai_shoes.meta.json"),
            (PERSONA, "pieces/makena_shoes/makena_shoes.meta.json"),
            (PERSONA, "pieces/noor_shirt/noor_shirt.meta.json"),
            (PERSONA, "pieces/standard_eyes/standard_eyes.meta.json"),
            (PERSONA, "pieces/standard_mouth/standard_mouth.meta.json"),
            (PERSONA, "pieces/steve_mouth/steve_mouth.meta.meta.json"),
            (PERSONA, "pieces/straight/straight.meta.json"),
            (PERSONA, "pieces/sunny_hair/sunny_hair.meta.json"),
            (PERSONA, "pieces/zuri_hair/zuri_hair.meta.json"),
        ],
        "persona_piece_store_info" => &[
            (PERSONA, "pieces/alex_hair/alex_hair.store_info.json"),
            (PERSONA, "pieces/ari_pants/ari_pants.store_info.json"),
            (PERSONA, "pieces/efe_hair/efe_hair.store_info.json"),
            (
                PERSONA,
                "pieces/emote_followMe/communication_follow_me.store_info.json",
            ),
            (PERSONA, "pieces/kai_jacket/kai_jacket.store_info.json"),
            (PERSONA, "pieces/makena_shoes/makena_shoes.store_info.json"),
            (PERSONA, "pieces/noor_shoes/noor_shoes.store_info.json"),
            (PERSONA, "pieces/steve_mouth/steve_mouth.store_info.json"),
            (PERSONA, "pieces/sunny_hair/sunny_hair.store_info.json"),
            (PERSONA, "pieces/zuri_pants/zuri_pants.store_info.json"),
        ],
        "languages" => &[(RP, "texts/languages.json")],
        "language_names" => &[(RP, "texts/language_names.json")],
        _ => &[],
    }
}

/// Every `.json` and `.material` file below `at`, sorted.
fn files_below(at: &Path, out: &mut Vec<PathBuf>) {
    if at.is_file() {
        out.push(at.to_path_buf());
        return;
    }
    let mut entries: Vec<PathBuf> = std::fs::read_dir(at)
        .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            files_below(&path, out);
        } else if path
            .extension()
            .is_some_and(|ext| ext == "json" || ext == "material")
        {
            out.push(path);
        }
    }
}

/// The family's sample of the corpus: at most [`FILES`] files, evenly spaced over the sorted
/// list, each as its pack path and its text as plain JSON (comments gone, one line).
fn corpus(family: &str) -> Vec<(String, String)> {
    let Some(root) = std::env::var_os("BEDROCK_ADDON_CORPUS").filter(|root| !root.is_empty())
    else {
        return Vec::new();
    };
    let root = PathBuf::from(root);
    let mut all: Vec<(PathBuf, PathBuf)> = Vec::new();
    for (pack, below) in sources(family) {
        let pack = root.join(pack);
        let mut files = Vec::new();
        files_below(&pack.join(below), &mut files);
        all.extend(files.into_iter().map(|file| (pack.clone(), file)));
    }
    let step = all.len().div_ceil(FILES).max(1);
    all.into_iter()
        .step_by(step)
        .filter_map(|(pack, file)| {
            let bytes = std::fs::read(&file).ok()?;
            let text = String::from_utf8(bytes).ok()?;
            let value = parse(text.trim_start_matches('\u{feff}'))?;
            let plain = bedrock_addon::json::to_json(&value).ok()?;
            let path = file.strip_prefix(&pack).ok()?.to_str()?.replace('\\', "/");
            Some((path, plain))
        })
        .collect()
}

/// The probe of one document type: its small document at `path`, then its corpus files at
/// their own paths. A corpus file the family's context refuses (the `minecraft` namespace
/// outside the base game) is read with `vanilla`, the same context for a base-game pack.
fn audit(
    family: &str,
    path: &'static str,
    read: &dyn Fn(&PackPath, &str) -> Outcome,
    vanilla: &dyn Fn(&PackPath, &str) -> Outcome,
    document: &str,
) {
    let mut tally = Tally {
        budget: BUDGET,
        ..Tally::default()
    };
    let at = PackPath::from_static(path);
    for document in std::iter::once(&document).chain(more(family)) {
        let read = |text: &str| read(&at, text);
        probe(family, path, &read, document, false, &mut tally);
    }
    let files = corpus(family);
    let mut loaded = 0;
    for (file, text) in &files {
        let Ok(at) = PackPath::new(file) else {
            continue;
        };
        let before = tally.texts;
        let read = if read(&at, text).loaded {
            read
        } else {
            vanilla
        };
        probe(
            family,
            file,
            &|text| read(&at, text),
            text,
            true,
            &mut tally,
        );
        loaded += usize::from(tally.texts > before);
    }
    let outcomes: Vec<String> = tally
        .outcomes
        .iter()
        .map(|(name, count)| format!("{name} {count}"))
        .collect();
    println!(
        "writable: {family} | corpus files {loaded} of {} (not loaded as they are: {}) | texts {} | {} | left out by the caps {} | faults {}",
        files.len(),
        tally.unread,
        tally.texts,
        outcomes.join(", "),
        tally.capped,
        tally.faults.len()
    );
    if !tally.vanilla.is_empty() {
        // What the vanilla files say as they are: a fix of this probe's faults must not add
        // to it.
        let counts: Vec<String> = tally
            .vanilla
            .iter()
            .map(|(code, count)| format!("{code} {count}"))
            .collect();
        println!("writable-vanilla: {family} | {}", counts.join(", "));
    }
    for (kind, count) in &tally.kinds {
        println!("writable-fault: {family} | {kind} | {count}");
    }
    for fault in tally.faults.iter().take(300) {
        let line: String = fault.replace('\n', " ").chars().take(520).collect();
        println!("writable-at: {line}");
    }
    let shown: Vec<&str> = tally.faults.iter().take(6).map(String::as_str).collect();
    assert!(
        tally.faults.is_empty(),
        "{family}: {} of {} texts are at fault ({} kinds, the `writable-fault:` lines); the first:\n{}",
        tally.faults.len(),
        tally.texts,
        tally.kinds.len(),
        shown.join("\n")
    );
}

/// One test per document type read through `Content::parse` (the hand-written places of the
/// list are `non_finite.rs`'s).
macro_rules! families {
    ($( $test:ident: $type:ty, $path:literal $(in $cx:expr)?, $document:expr $(, [$($more:expr),* $(,)?])?; )*) => {
        $(
            // `in_this_binary!` keeps the test or leaves it to the other binary.
            in_this_binary! { $test {
                #[test]
                fn $test() {
                    #[allow(unused_variables, reason = "most families read in a detached context")]
                    let cx = LoadContext::detached();
                    $(let cx = $cx;)?
                    let mut vanilla = cx.clone();
                    vanilla.is_base_game_pack = true;
                    vanilla.is_trusted_pack = true;
                    audit(
                        stringify!($test),
                        $path,
                        &|path, text| content_in::<$type>(path, text, &cx),
                        &|path, text| content_in::<$type>(path, text, &vanilla),
                        $document,
                    );
                }
            } }
        )*
    };
}

document_types!(families);

/// The game line of a kept error, by which the written text's load and the read are matched:
/// only a finding with the game's sentence (`Diagnostic::vanilla`, a legacy code's game text or
/// a specific code's game sentence) has one; a kept error without it renders only its kind's
/// word and pairs with nothing.
fn kept_line(finding: &Diagnostic) -> Option<String> {
    let has_game_text = finding.vanilla().is_some();
    (finding.kept && has_game_text).then(|| finding.code.to_string())
}

#[cfg(test)]
mod kept_line_tests {
    use super::kept_line;
    use bedrock_addon::diagnostic::{
        Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
    };

    fn kept_at(code: DiagnosticCode, key: &str) -> Diagnostic {
        let mut path = JsonPath::new();
        path.push(key);
        let mut d = Diagnostics::default();
        d.push_kept_error(Diagnostic::new(
            code,
            Severity::Error,
            Phase::Validate,
            Location::new(None, None, path, None),
        ));
        d.iter().next().unwrap().clone()
    }

    /// Two kept errors without a game text render only their kind's word: they are no line
    /// of the game's to pair by.
    #[test]
    fn two_kept_errors_without_a_game_text_do_not_pair() {
        let a = kept_at(DiagnosticCode::InvalidType { text: None }, "a");
        let b = kept_at(DiagnosticCode::InvalidType { text: None }, "b");
        let (a, b) = (kept_line(&a), kept_line(&b));
        assert!(a.is_none() || a != b, "{a:?} pairs with {b:?}");
    }

    /// The same game text under two codes at two places is the one line.
    #[test]
    fn two_kept_errors_with_one_game_text_pair() {
        use bedrock_addon::diagnostic::VanillaText;
        let text = || {
            Some(VanillaText::Fixed(
                "There must be an \"entries\" array inside the loot table pool.",
            ))
        };
        let a = kept_at(DiagnosticCode::InvalidType { text: text() }, "entries");
        let b = kept_at(
            DiagnosticCode::MissingRequiredField { text: text() },
            "pool",
        );
        assert!(kept_line(&a).is_some());
        assert_eq!(kept_line(&a), kept_line(&b));
    }
}
