// The legacy probe itself: the documents, the judgement and the audit of one family. Included
// by `writable_legacy.rs` and by the binaries of the large families (`writable_legacy_*.rs`).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use bedrock_addon::json::Value;
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::pack::PackType;
use bedrock_addon::storage::PackPath;
use bedrock_addon::vanilla::ActorType;
use bedrock_addon::version::{FormatVersion, FormatVersionInput, MinEngineVersion};
use bedrock_addon::versioning::testing::{Checks, Run, step_output};
use bedrock_addon::versioning::{Upgrade, VersionStruct, Versioned};

use document_types::{client, pack};
use places::{
    Kind, Node, Outcome, code, content_in, nodes, non_strict_readings, outcome, parse, path_of,
    scalar, without,
};

/// What a value is replaced by, each with a second replacement of the same class (see
/// "carried" in the module docs) and its class.
const REPLACEMENTS: &[(&str, &str, Class)] = &[
    ("7", "8", Class::Integer),
    ("\"x\"", "\"y\"", Class::String),
    ("true", "false", Class::Bool),
    ("null", "null", Class::Null),
    ("[]", "[]", Class::EmptyArray),
    ("{}", "{}", Class::EmptyObject),
    ("[7]", "[8]", Class::Array),
    ("{\"a\": 1}", "{\"a\": 2}", Class::Object),
    ("1e39", "2e39", Class::Huge),
    ("-1e999", "-2e999", Class::Huge),
    ("0.5", "0.25", Class::Fraction),
    ("4294967296", "4294968296", Class::Wide),
    // Another value of the value's own class: a string where a string stands, an integer
    // where an integer stands (a member read as a constant, or as one of a set, whatever
    // it names).
    ("\"zz:bogus\"", "\"zz:other\"", Class::OtherString),
    ("123457", "123458", Class::OtherInt),
];

/// The class of a value: a replacement of the value's own class is not tried.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Class {
    Integer,
    Fraction,
    Wide,
    Huge,
    String,
    Bool,
    Null,
    EmptyArray,
    Array,
    EmptyObject,
    Object,
    /// A string that no document has (tried where a string stands, too).
    OtherString,
    /// An integer that no document has (tried where an integer stands, too).
    OtherInt,
}

/// The families whose upgrade writes a Molang value given as `true` as the number it is, 1
/// (`writable.rs`, `BOOL_AS_NUMBER`), each with its reason.
const BOOL_AS_NUMBER: &[(&str, &str)] = &[("particle", "bool written as number")];

/// The families whose documents below a format version are read by the game's non-strict
/// reader (`writable.rs`, `NON_STRICT_BELOW`): a bool and a number read as each other, an
/// integer by its low bits, a real by its truncation toward zero, silently.
const NON_STRICT_BELOW: &[(&str, FormatVersion)] = &[("block", FormatVersion::new(1, 26, 20))];

/// Whether what the written text holds of `replacement` is what the game's non-strict reader
/// reads it as: every fresh leaf one of its readings, or, with no fresh leaf, a value the
/// place's own value reads as too (`7` and `true` in a bool member are both `true`, and the
/// text `ours` is written as holds that bool).
fn read_non_strict(
    base: &Base<'_>,
    node: &Node,
    replacement: &str,
    new: &[String],
    ours: &[String],
) -> bool {
    let Some((_, below)) = NON_STRICT_BELOW.iter().find(|(of, _)| *of == base.family) else {
        return false;
    };
    let legacy = base
        .value
        .as_object()
        .and_then(|root| root.get("format_version"))
        .and_then(FormatVersionInput::from_json)
        .and_then(|input| input.version)
        .is_some_and(|version| version < *below);
    let numbers = non_strict_readings(replacement);
    if !legacy || numbers.is_empty() {
        return false;
    }
    // The game takes a number as `value != 0`.
    let truth = format!("b:{}", numbers.iter().any(|n| *n != 0.0));
    let reading = |leaf: &str| {
        leaf == truth
            || leaf
                .strip_prefix("n:")
                .and_then(|n| n.parse::<f64>().ok())
                .is_some_and(|n| numbers.contains(&n))
    };
    if new.is_empty() {
        let own = non_strict_readings(&base.text[node.start..node.end]);
        let same_truth = !own.is_empty()
            && own.iter().any(|n| *n != 0.0) == numbers.iter().any(|n| *n != 0.0)
            && ours.contains(&truth);
        return same_truth
            || own.iter().any(|n| numbers.contains(n))
            || parse(&base.text[node.start..node.end])
                .as_ref()
                .and_then(scalar)
                .is_some_and(|leaf| reading(&leaf));
    }
    new.iter().all(|leaf| reading(leaf))
}

/// Places where the invariant does not hold for a recorded reason: the family, the end of the
/// place's path (indices as `*`), the replacement (`*` for all) and the reason. See the entries; a place is listed only where the family's folder says
/// the game itself takes the value so. The replacement `(removed)` is the member taken out.
const EXEMPT: &[(&str, &str, &str, &str)] = &[
    // A filter subject written as vanilla's number (`7`) is the subject of that number
    // (`damager`; vanilla's own re-serialised files write numbers) and is written
    // by its name: the same subject in the form the family writes (`writable.rs`).
    ("entity", "/target", "7", "subject written by name"),
    ("entity", "/subject", "7", "subject written by name"),
    ("item", "/target", "7", "subject written by name"),
    ("block", "/target", "7", "subject written by name"),
    // block: a legacy block that holds an unknown member is written at its own
    // version (a cereal document with it is refused, 0x4 in a strict slice), without the
    // components the steps make again on reading it; the judge counts what it no longer
    // finds beside the member against it: (a) the scalars of the seeded copy of the payload
    // that the replacement stands for, (b) the synthesised components of the newest text.
    (
        "block",
        "minecraft:block/minecraft:block",
        "true",
        "legacy block written at its own version",
    ),
    (
        "block",
        "/menu_category/zz",
        "(present)",
        "legacy block written at its own version",
    ),
    // entity: places where the written text is right and nothing is to tell, and the
    // judge's rules have no outcome for it.
    //
    // (a) A member a step inserts when it is ABSENT and then MOVES, given as an empty value:
    // the empty value is kept as written. The judge compares with the document without the
    // member, which gets the step's default at the member's new place, and counts the
    // default's members as the author's ("collateral"; at the place itself it knows them to
    // be the member's own).
    (
        "entity",
        "minecraft:interact/*/on_interact",
        "{}",
        "value converted or moved by the upgrade",
    ),
    (
        "entity",
        "minecraft:behavior.follow_target_captain/leader_filters",
        "[]",
        "value converted or moved by the upgrade",
    ),
    (
        "entity",
        "minecraft:behavior.follow_target_captain/leader_filters",
        "{}",
        "value converted or moved by the upgrade",
    ),
    // (b) A conversion of the folder that carries the value into another type or unit, for
    // which the judge has no "carried": ticks × 0.05 of a number beyond an int, an integer
    // slot as its decimal string (`2 → "2"`), a bool `should_bounce` as its name.
    (
        "entity",
        "minecraft:behavior.explore_outskirts/wait_time",
        "4294967296",
        "value converted or moved by the upgrade",
    ),
    (
        "entity",
        "/equip_item_slot",
        "4294967296",
        "value converted or moved by the upgrade",
    ),
    (
        "entity",
        "/drop_item_slot",
        "7",
        "value converted or moved by the upgrade",
    ),
    (
        "entity",
        "/drop_item_slot",
        "4294967296",
        "value converted or moved by the upgrade",
    ),
    (
        "entity",
        "/drop_item_slot",
        "123457",
        "value converted or moved by the upgrade",
    ),
    (
        "entity",
        "/equip_item_slot",
        "123457",
        "value converted or moved by the upgrade",
    ),
    (
        "entity",
        "minecraft:projectile/should_bounce",
        "true",
        "value converted or moved by the upgrade",
    ),
    // (c) A component taken out that a step SPLITS into others, whose written form held a
    // default (the `false` of `pushable_by_entity`) equal to a scalar the author wrote in a
    // member the upgrade carries by its absence: nothing of the author's is lost.
    (
        "entity",
        "/minecraft:pushable",
        "(removed)",
        "value converted or moved by the upgrade",
    ),
    // The same for a `hurt_when_wet: true` taken out, or replaced by a value that makes no
    // rain entry: the entry the 1.12.0 step made of the `true` goes, and
    // its `damage_per_tick: 1`, the step's constant, is the one scalar the judge counts as
    // lost beside the component, which is told `DroppedComponent`. The constant is no value
    // of the author's. A row for each such replacement and no other: `true`, `false` and what is BELOW the
    // component are judged.
    (
        "entity",
        "/minecraft:hurt_when_wet",
        "(removed)",
        "taken-out component leaves a step constant",
    ),
    (
        "entity",
        "/minecraft:hurt_when_wet",
        "7",
        "taken-out component leaves a step constant",
    ),
    (
        "entity",
        "/minecraft:hurt_when_wet",
        "\"x\"",
        "taken-out component leaves a step constant",
    ),
    (
        "entity",
        "/minecraft:hurt_when_wet",
        "null",
        "taken-out component leaves a step constant",
    ),
    (
        "entity",
        "/minecraft:hurt_when_wet",
        "[]",
        "taken-out component leaves a step constant",
    ),
    (
        "entity",
        "/minecraft:hurt_when_wet",
        "{}",
        "taken-out component leaves a step constant",
    ),
    (
        "entity",
        "/minecraft:hurt_when_wet",
        "[7]",
        "taken-out component leaves a step constant",
    ),
    (
        "entity",
        "/minecraft:hurt_when_wet",
        "{\"a\": 1}",
        "taken-out component leaves a step constant",
    ),
    (
        "entity",
        "/minecraft:hurt_when_wet",
        "1e39",
        "taken-out component leaves a step constant",
    ),
    (
        "entity",
        "/minecraft:hurt_when_wet",
        "-1e999",
        "taken-out component leaves a step constant",
    ),
    (
        "entity",
        "/minecraft:hurt_when_wet",
        "0.5",
        "taken-out component leaves a step constant",
    ),
    (
        "entity",
        "/minecraft:hurt_when_wet",
        "4294967296",
        "taken-out component leaves a step constant",
    ),
    (
        "entity",
        "/minecraft:hurt_when_wet",
        "\"zz:bogus\"",
        "taken-out component leaves a step constant",
    ),
    (
        "entity",
        "/minecraft:hurt_when_wet",
        "123457",
        "taken-out component leaves a step constant",
    ),
    // entity: `true` put where the written text has `true` already, so that the
    // judge has no scalar to see it by. ((a) of that entry, a member read as its default
    // `true` when the file's is of another type, is an outcome of the judge:
    // "held", where the unchanged document was told at the place and this one is not.)
    // What stays of (a): the two members of `pushable`, read as `true` when they are of
    // another type (told there) and written as the PRESENCE of `pushable_by_entity` /
    // `pushable_by_block` so that the text holds no `true` to see it by.
    (
        "entity",
        "minecraft:pushable/is_pushable",
        "true",
        "no scalar left to see the value by",
    ),
    (
        "entity",
        "minecraft:pushable/is_pushable_by_piston",
        "true",
        "no scalar left to see the value by",
    ),
    // (b) A component of no typed version yet that is held as written, `true` included,
    // beside a `true` the step's own entries hold elsewhere.
    (
        "entity",
        "/minecraft:hurt_on_condition",
        "true",
        "no scalar left to see the value by",
    ),
    (
        "entity",
        "/minecraft:despawn",
        "true",
        "no scalar left to see the value by",
    ),
    (
        "entity",
        "minecraft:despawn/filters",
        "true",
        "no scalar left to see the value by",
    ),
    // (c) A bool `deals_damage` as its name (`true` → `"yes"`): a
    // conversion of the folder, as class (b) above.
    (
        "entity",
        "/deals_damage",
        "true",
        "no scalar left to see the value by",
    ),
    // (d) A member whose presence moves a sibling INTO its place (`attack_interval_min`
    // becomes `attack_interval.range_min`), which the judge then counts as gone
    // from beside it; and the one element of `on_death_array` that becomes an EMPTY
    // `on_death` (that element becomes the scalar, nothing is dropped).
    (
        "entity",
        "minecraft:behavior.nearest_attackable_target/attack_interval",
        "(present)",
        "no scalar left to see the value by",
    ),
    (
        "entity",
        "minecraft:experience_reward/on_death_array",
        "(present)",
        "no scalar left to see the value by",
    ),
    (
        "entity",
        "minecraft:experience_reward/on_death_array/*",
        "(present)",
        "no scalar left to see the value by",
    ),
    // animation controller: a 1.8 remap curve given as `{}` is the variable's
    // `remap_curve: {}` (`state.variables[name].remap_curve = v[i]`). The
    // judge compares with the document without the element, where the NEXT curve takes the
    // variable's name (`walk_0`), and counts that curve's keys as the author's.
    (
        "animation_controller",
        "/animations/*/walk/*",
        "{}",
        "curve takes the next variable name",
    ),
    // particle. A legacy `direction` of another length than three is told at the
    // list ("error reading array") and read as the default: an element of it is no place of
    // its own. A number beyond a float at `min_speed_threshold` is held as the largest finite
    // float and told at the member in the words that tell of a value of another type, which
    // the generated base has there.
    (
        "particle",
        "minecraft:emitter_shape_entity_aabb/direction/*",
        "*",
        "list read as the default",
    ),
    (
        "particle",
        "particle_appearance_billboard/direction/min_speed_threshold",
        "1e39",
        "list read as the default",
    ),
    (
        "particle",
        "particle_appearance_billboard/direction/min_speed_threshold",
        "-1e999",
        "list read as the default",
    ),
    // trade table. A `true` at an integer member is read as vanilla's integer read reads
    // it, 1, and told at the member (`InvalidType`) in the words that tell of any
    // value of another type; the judge takes the same words for no word of the 1.
    (
        "trade_table",
        "/total_exp_required",
        "true",
        "trade table upgrade",
    ),
    (
        "trade_table",
        "/num_to_select",
        "true",
        "trade table upgrade",
    ),
    // An item with the `trader_material_type` function below 1.18.20 is its five choices
    // `<item>:<n>`: the item's name is carried in each, and the function
    // goes into them; nothing of the author's is lost with the function there.
    (
        "trade_table",
        "/functions",
        "(present)",
        "trade table upgrade",
    ),
    (
        "trade_table",
        "/functions/*",
        "(present)",
        "trade table upgrade",
    ),
    (
        "trade_table",
        "/functions/*/function",
        "(present)",
        "trade table upgrade",
    ),
    // geometry: places where the written text is right and the judge counts a value
    // by its place. `preserve_model_pose` holds a file below 1.8 in its era's shape:
    // without it the file is written in the newest format, with the pivot the upgrade gives
    // each cube, a number equal to one the author wrote. An empty pre-1.8 cube is removed by
    // the cube pass, which moves the LAST cube into its place: the cubes are all written, in
    // another order.
    (
        "geometry",
        "geometry.pig/preserve_model_pose",
        "(present)",
        "cube pivot given by the upgrade",
    ),
    (
        "geometry",
        "geometry.pig/bones/*/cubes/*",
        "(present)",
        "cube pivot given by the upgrade",
    ),
    // the probe's "one more element" of a 1.8 `animations` list is an entry that
    // repeats its name with the same curves: the variables `walk_<i>` are written once, with
    // the curves both entries give. Taken out of the copy, a curve leaves the NEXT one to take
    // its variable's name and the judge counts the curve that is then written twice as lost.
    // A curve that IS written over by another is noted.
    (
        "animation_controller",
        "/animations/*/walk/*",
        "(present)",
        "repeated name written once",
    ),
    // item: places where the load tells what it does and the judge's rules have no
    // outcome for it.
    //
    // (a) An entry of `icon.textures` that is no string is written as the string it spells
    // (`3` → `"3"`, `null` → `""`), with `InvalidType` at the entry: the same finding
    // whatever the value, so the judge, whose base is the generated `"zz": 3`, sees no new
    // word for the next value.
    (
        "item",
        "minecraft:icon/textures/zz",
        "*",
        "component dropped or read as the default",
    ),
    // (b) The same of a `null` where the generated base holds a value of another type: the
    // base's component is dropped for that value (told at the member and at the
    // component), a `null` is read as the member's default with the same finding at the
    // member; and where the component so read lets the 1.21.60 split run, the file's
    // `storage_weight_limit` it writes over is told at that component.
    (
        "item",
        "minecraft:use_modifiers/emit_vibrations",
        "null",
        "component dropped or read as the default",
    ),
    (
        "item",
        "minecraft:storage_item/max_slots",
        "null",
        "component dropped or read as the default",
    ),
    (
        "item",
        "minecraft:storage_item/allow_nested_storage_items",
        "null",
        "component dropped or read as the default",
    ),
    // (c) A keyed custom component of the file beside the v1 list that names it: the keyed
    // one wins and the list's entry is told (`DroppedField` at `custom_components`); the
    // name the side-car then holds (`custom_components_v1`) is the scalar the judge misses
    // in the document WITHOUT the keyed component, where the list's entry becomes the
    // keyed one and no list is written.
    (
        "item",
        "components/demo:shake",
        "(removed)",
        "component dropped or read as the default",
    ),
];

/// Starting versions of a step whose document has no JSON text to compare: the family,
/// the version and why.
const NO_TEXT: &[(&str, &str, &str)] = &[
    (
        "jigsaw",
        "1.21.50",
        "the structure metadata registry is a binary file (`JigsawMetadata::to_json` refuses it)",
    ),
    (
        "sound_definitions",
        "1.0.0",
        "the 1.0.0 step reads the files without a `format_version`: its documents are the version-less ones, tallied as `none`",
    ),
];

/// The texts a family's probe loads at most.
const BUDGET: usize = 400_000;

/// The families of many texts: a place of theirs is probed once per reader, version, path
/// and set of sibling members. Every other family is probed at every place of every document.
const LARGE: &[&str] = &[
    "entity",
    "item",
    "block",
    "geometry",
    "sound_definitions",
    "animation",
];

// --- the families ------------------------------------------------------------------------------

/// One way to read a family's document: a name, and the reader.
type Reader = (&'static str, Box<dyn Fn(&str) -> Outcome>);

/// A family of the probe.
struct Family {
    name: &'static str,
    /// Its test folders below `tests/` and fixture folders below `tests/fixtures/`.
    folders: &'static [&'static str],
    /// The keys a step's payload is wrapped in to make a document.
    keys: &'static [&'static str],
    readers: Vec<Reader>,
    /// The steps of its chain that are run alone, for the members each adds (see
    /// [`seeded`]); a family without them is seeded from its whole chain.
    runs: Vec<StepRun>,
    /// The versions of the documents this test probes, from the first up to and without the
    /// second (a family of many texts is probed by more than one test binary).
    band: Option<(Triple, Triple)>,
}

/// A version as three numbers.
type Triple = (u32, u32, u32);

/// One step of a family's chain, run alone: the version it starts from, and what it makes of
/// a payload of a document of the version given (`None` when it does not take the payload).
struct StepRun {
    from: Triple,
    /// The version its output is of: the version of the next step.
    to: Triple,
    run: RunStep,
}

/// A step run alone: the payload and the version of its document in, the next payload out.
type RunStep = Box<dyn Fn(&Value, Option<Triple>) -> Option<Value>>;

/// The identifier a payload names (`description.identifier`), or none.
fn identifier(payload: &Value) -> String {
    payload
        .as_object()
        .and_then(|payload| payload.get("description"))
        .and_then(Value::as_object)
        .and_then(|description| description.get("identifier"))
        .and_then(Value::as_string)
        .map(|name| name.as_str().to_owned())
        .unwrap_or_default()
}

/// The step of `S`, run in the context `load` gives.
fn step<S>(load: fn() -> LoadContext) -> StepRun
where
    S: Upgrade + VersionStruct + 'static,
    S::Next: VersionStruct,
{
    let from = <S as Versioned>::VERSION;
    let to = <S::Next as Versioned>::VERSION;
    let narrow = |part: u32| u16::try_from(part).unwrap_or(u16::MAX);
    StepRun {
        from: (from.major.into(), from.minor.into(), from.patch.into()),
        to: (to.major.into(), to.minor.into(), to.patch.into()),
        run: Box::new(move |payload, version| {
            let identifier = identifier(payload);
            let checks = Checks {
                load: load(),
                identifier: &identifier,
                actor_type: ActorType::for_entity(&identifier, None),
            };
            let settings = Run {
                original_version: version
                    .map(|(major, minor, patch)| {
                        FormatVersion::new(narrow(major), narrow(minor), narrow(patch))
                    })
                    .filter(|version| *version >= from),
                skip_section_a: false,
            };
            step_output::<S>(payload, settings, &checks)
                .ok()
                .map(|output| output.payload)
        }),
    }
}

/// A reader of `T` at `path` in `cx`.
fn reader<T: bedrock_addon::document::Content + 'static>(
    name: &'static str,
    path: &'static str,
    cx: LoadContext,
) -> Reader {
    (
        name,
        Box::new(move |text| content_in::<T>(&PackPath::from_static(path), text, &cx)),
    )
}

/// One of the ten packs allowed the legacy creator features.
fn allow_listed() -> LoadContext {
    let mut cx = LoadContext::for_pack(
        &LoadOptions::default(),
        uuid::Uuid::from_u128(0xda09_e399_6f5e_4287_a013_4cf1_3de3_2166),
        PackType::Behavior,
        Some(MinEngineVersion::new(1, 26, 20)),
        false,
    );
    cx.molang_version = bedrock_addon::molang::pack_molang_version(cx.min_engine_version.as_ref());
    cx
}

/// The base game's own pack.
fn base_game() -> LoadContext {
    let mut cx = LoadContext::detached();
    cx.is_base_game_pack = true;
    cx.is_trusted_pack = true;
    cx
}

// --- the documents -----------------------------------------------------------------------------

/// A legacy document of a family: its version (as the file says it, or the step's), where it
/// is from and its text as plain JSON.
struct Legacy {
    version: String,
    from: String,
    text: String,
    /// The one member of the text that is probed, with what is below it: the member a
    /// generated input was made for (`None`: every place).
    only: Option<Vec<String>>,
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// A version text as three numbers (`1.10` is `1.10.0`).
fn version_of(text: &str) -> Option<(u32, u32, u32)> {
    let mut parts = text.split('.').map(|part| part.trim().parse::<u32>().ok());
    let major = parts.next()??;
    let minor = parts.next().unwrap_or(Some(0))?;
    let patch = parts.next().unwrap_or(Some(0))?;
    Some((major, minor, patch))
}

/// The `format_version` a document carries, as a text.
fn format_version(value: &Value) -> Option<String> {
    let member = value.as_object()?.get("format_version")?;
    if let Some(text) = member.as_string() {
        Some(text.as_str().to_owned())
    } else {
        member
            .as_number()
            .map(|number| format!("{}", number.to_f64_lossy()))
    }
}

/// `value` as plain JSON on one line.
fn plain(value: &Value) -> Option<String> {
    bedrock_addon::json::to_json(value).ok()
}

/// `key` as a JSON string (`None` for a key with a character JSON escapes).
fn quote(key: &str) -> Option<String> {
    (!key.contains(['"', '\\']) && !key.chars().any(char::is_control)).then(|| format!("\"{key}\""))
}

/// An object with `key: member` put before the members of `rest` (an object or nothing).
fn with_member(key: &str, member: &str, rest: Option<&str>) -> String {
    let key = quote(key).expect("a plain key");
    match rest.map(str::trim) {
        Some(rest) if rest.len() > 2 => format!("{{{key}: {member}, {}", &rest[1..]),
        _ => format!("{{{key}: {member}}}"),
    }
}

/// The texts a step's payload may be a document as: wrapped in each of the family's keys
/// (when it has none of them), itself, and itself with the step's version.
fn candidates(family: &Family, version: &str, payload: &Value) -> Vec<String> {
    let Some(text) = plain(payload) else {
        return Vec::new();
    };
    let quoted = format!("\"{version}\"");
    // The payload under each of the family's keys first: a family may take a root without
    // its key as the payload itself, which is no document an author writes.
    let mut out = Vec::new();
    let keyed = payload
        .as_object()
        .is_some_and(|object| family.keys.iter().any(|key| object.contains_key(key)));
    if !keyed {
        for key in family.keys {
            let inner = with_member(key, &text, None);
            out.push(with_member("format_version", &quoted, Some(&inner)));
            out.push(inner);
        }
    }
    out.push(text.clone());
    if payload.is_object() && format_version(payload).is_none() {
        out.push(with_member("format_version", &quoted, Some(&text)));
    }
    out
}

/// `value` with every scalar another one of its type (a number one higher, a string with a
/// suffix, a bool the other one).
fn another(value: &Value) -> Option<Value> {
    let text = plain(value)?;
    let mut out = String::new();
    let mut from = 0;
    for node in nodes(&text) {
        let token = &text[node.start..node.end];
        let other = match node.kind {
            Kind::Number => format!("{}", token.parse::<f64>().ok()? + 1.0),
            Kind::String => format!("{}_z\"", &token[..token.len() - 1]),
            Kind::Other if token == "true" => "false".to_owned(),
            Kind::Other if token == "false" => "true".to_owned(),
            _ => continue,
        };
        out.push_str(&text[from..node.start]);
        out.push_str(&other);
        from = node.end;
    }
    out.push_str(&text[from..]);
    parse(&out)
}

/// `document` with `value` at `path` (parents that are missing are made, as objects; an index
/// one past the end of a list appends).
fn set_at(document: &Value, path: &[String], value: Value) -> Value {
    let mut out = document.clone();
    set_in(&mut out, path, value);
    out
}

fn set_in(at: &mut Value, path: &[String], value: Value) {
    let Some((segment, rest)) = path.split_first() else {
        *at = value;
        return;
    };
    if let Some(array) = at.as_array_mut() {
        let Ok(index) = segment.parse::<usize>() else {
            return;
        };
        if index == array.len() {
            array.push(Value::NULL);
        }
        if let Some(element) = array.get_mut(index) {
            set_in(element, rest, value);
        }
        return;
    }
    if !at.is_object() {
        *at = parse("{}").expect("an empty object");
    }
    let Some(object) = at.as_object_mut() else {
        return;
    };
    if !object.contains_key(segment) {
        object.insert(segment.as_str(), Value::NULL);
    }
    if let Some(next) = object.get_mut(segment) {
        set_in(next, rest, value);
    }
}

/// Another value of the type of `value`: every scalar another one; an empty object or list
/// with a member or an element. `None` for a `null`.
fn different(value: &Value) -> Option<Value> {
    if value.is_null() {
        return None;
    }
    match another(value) {
        Some(other) if other != *value => Some(other),
        _ if value.is_object() => parse(r#"{"a": 1}"#),
        _ if value.is_array() => parse("[7]"),
        _ => None,
    }
}

/// A value of another type than `value`: a string for a number, a number for anything else.
fn of_another_type(value: &Value) -> Value {
    if value.is_number() {
        Value::from("x")
    } else {
        Value::from(7)
    }
}

/// Where a legacy document is from: its version, its origin, the texts it may be a document
/// as (the first a reader loads and writes is taken), and the one member below the payload
/// that is probed (`None`: every place).
struct Source {
    version: String,
    from: String,
    texts: Vec<String>,
    only: Option<Vec<String>>,
}

/// Every file below `at` with one of `extensions`, sorted.
fn files_below(at: &Path, extensions: &[&str], out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(at)
        .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            files_below(&path, extensions, out);
        } else if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| extensions.contains(&ext))
        {
            out.push(path);
        }
    }
}

/// The raw string literals of a Rust source that begin as a JSON object or array.
fn literals(source: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = source;
    while let Some(at) = rest.find("r#") {
        let after = &rest[at + 1..];
        let hashes = after.bytes().take_while(|byte| *byte == b'#').count();
        let body = &after[hashes..];
        let Some(body) = body.strip_prefix('"') else {
            rest = &rest[at + 2..];
            continue;
        };
        let close = format!("\"{}", "#".repeat(hashes));
        let Some(end) = body.find(&close) else {
            break;
        };
        let literal = &body[..end];
        if literal.trim_start().starts_with(['{', '[']) {
            out.push(literal);
        }
        rest = &body[end + close.len()..];
    }
    out
}

/// The documents of the family's tests and fixtures: every JSON literal of `tests/<folder>/`
/// (and `tests/<folder>.rs`), here and in the family's own crate
/// (`crates/addon/families/<folder>/tests/`, where the tests that need no pack live), and
/// every `.json` file of `tests/fixtures/<folder>/` under 8 KiB.
fn test_documents(family: &Family, out: &mut Vec<Source>) {
    let tests = manifest_dir().join("tests");
    let families = manifest_dir().join("../families");
    for folder in family.folders {
        let mut sources = Vec::new();
        files_below(&tests.join(folder), &["rs"], &mut sources);
        sources.push(tests.join(format!("{folder}.rs")));
        let own = families.join(folder).join("tests");
        files_below(&own.join(folder), &["rs"], &mut sources);
        sources.push(own.join(format!("{folder}.rs")));
        for source in sources {
            let Ok(text) = std::fs::read_to_string(&source) else {
                continue;
            };
            let name = source
                .strip_prefix(&tests)
                .or_else(|_| source.strip_prefix(&families))
                .unwrap_or(&source)
                .display();
            for (number, literal) in literals(&text).into_iter().enumerate() {
                let Some(value) = parse(literal) else {
                    continue;
                };
                let version = format_version(&value).unwrap_or_else(|| "none".to_owned());
                let Some(text) = plain(&value) else { continue };
                out.push(Source {
                    version,
                    from: format!("{name} literal {number}"),
                    texts: vec![text],
                    only: None,
                });
            }
        }
        let mut fixtures = Vec::new();
        files_below(
            &tests.join("fixtures").join(folder),
            &["json"],
            &mut fixtures,
        );
        for fixture in fixtures {
            let Ok(bytes) = std::fs::read(&fixture) else {
                continue;
            };
            if bytes.len() > 8 << 10 {
                continue;
            }
            let Ok(text) = String::from_utf8(bytes) else {
                continue;
            };
            let Some(value) = parse(text.trim_start_matches('\u{feff}')) else {
                continue;
            };
            let version = format_version(&value).unwrap_or_else(|| "none".to_owned());
            let Some(text) = plain(&value) else { continue };
            let name = fixture.strip_prefix(&tests).unwrap_or(&fixture).display();
            out.push(Source {
                version,
                from: name.to_string(),
                texts: vec![text],
                only: None,
            });
        }
    }
}

include!("more.rs");

// --- the judgement -----------------------------------------------------------------------------

/// Every scalar of `value` as a text that is equal for equal values, and every key (`k:`).
fn leaves(value: &Value, out: &mut Vec<String>) {
    if let Some(object) = value.as_object() {
        for (key, member) in object {
            out.push(format!("k:{}", key.as_str()));
            leaves(member, out);
        }
    } else if let Some(array) = value.as_array() {
        for element in array.as_slice() {
            leaves(element, out);
        }
    } else if let Some(leaf) = places::scalar(value) {
        out.push(leaf);
    }
}

/// The [`leaves`] of `value` outside the member or element at `path`; `false` when `value`
/// has no such place.
fn leaves_outside(value: &Value, path: &[String], out: &mut Vec<String>) -> bool {
    let Some((first, rest)) = path.split_first() else {
        return true;
    };
    let mut found = false;
    if let Some(object) = value.as_object() {
        for (key, member) in object {
            out.push(format!("k:{}", key.as_str()));
            if key.as_str() == first && !found {
                found = leaves_outside(member, rest, out);
            } else {
                leaves(member, out);
            }
        }
    } else if let Some(array) = value.as_array() {
        let at = first.parse::<usize>().ok();
        for (index, element) in array.as_slice().iter().enumerate() {
            if Some(index) == at {
                found = leaves_outside(element, rest, out);
            } else {
                leaves(element, out);
            }
        }
    }
    found
}

/// The `format_version` a written text has.
fn written_version(value: &Value) -> Option<String> {
    value
        .as_object()
        .and_then(|root| root.get("format_version"))
        .and_then(places::scalar)
}

/// The [`leaves`] of `value` outside the member or element at `skip`, without the members
/// a step of the upgrade synthesises: those that neither `file` nor `ours` has at the same
/// place and that `bare`, the document without anything of the author's, has there.
fn leaves_of_members_kept<'a>(
    value: &Value,
    file: Option<&'a Value>,
    ours: Option<&'a Value>,
    bare: Option<&'a Value>,
    skip: Option<&[String]>,
    out: &mut Vec<String>,
) {
    let (first, rest) = match skip {
        Some([]) => return,
        Some([first, rest @ ..]) => (Some(first.as_str()), Some(rest)),
        None => (None, None),
    };
    if let Some(object) = value.as_object() {
        for (key, member) in object {
            let key = key.as_str();
            let of =
                |other: Option<&'a Value>| -> Option<&'a Value> { other?.as_object()?.get(key) };
            let (file, ours, bare) = (of(file), of(ours), of(bare));
            if file.is_none() && ours.is_none() && bare.is_some() {
                continue;
            }
            let skip = if first == Some(key) { rest } else { None };
            leaves_of_members_kept(member, file, ours, bare, skip, out);
        }
    } else if let Some(array) = value.as_array() {
        for (index, element) in array.as_slice().iter().enumerate() {
            let of = |other: Option<&'a Value>| -> Option<&'a Value> {
                other?.as_array()?.as_slice().get(index)
            };
            let skip = if first.and_then(|at| at.parse().ok()) == Some(index) {
                rest
            } else {
                None
            };
            leaves_of_members_kept(element, of(file), of(ours), of(bare), skip, out);
        }
    } else if let Some(leaf) = places::scalar(value) {
        out.push(leaf);
    }
}

/// The path of every member and element of `value`, indices as `*`.
fn members(value: &Value, at: &str, out: &mut BTreeSet<String>) {
    if let Some(object) = value.as_object() {
        for (key, member) in object {
            let path = format!("{at}/{}", key.as_str());
            members(member, &path, out);
            out.insert(path);
        }
    } else if let Some(array) = value.as_array() {
        let path = format!("{at}/*");
        for element in array.as_slice() {
            members(element, &path, out);
            out.insert(path.clone());
        }
    }
}

/// The finding as one text: its code, path and message.
fn finding_text(finding: &bedrock_addon::diagnostic::Diagnostic) -> String {
    format!(
        "{:?} at `{}`: {:?}",
        finding.code, finding.location.json_path, finding.detail
    )
}

/// A text a family loads and writes: what it wrote and what it found.
struct Read {
    written: String,
    /// The written text as a value.
    value: Value,
    leaves: Vec<String>,
    members: BTreeSet<String>,
    findings: Vec<String>,
    /// The path of each finding.
    at: Vec<Vec<String>>,
}

impl Read {
    /// The paths of the findings `known` does not have.
    fn told_beyond(&self, known: &Read) -> Vec<&Vec<String>> {
        let mut left = known.findings.clone();
        self.findings
            .iter()
            .zip(&self.at)
            .filter(
                |(text, _)| match left.iter().position(|known| known == *text) {
                    Some(index) => {
                        left.swap_remove(index);
                        false
                    }
                    None => true,
                },
            )
            .map(|(_, at)| at)
            .collect()
    }
}

/// The mark of a load or a write that panicked, in the writer's error.
const PANICKED: &str = "PANICKED";

/// What a family made of `text`: `Err(None)` when it is refused, `Err(Some)` when it loads
/// and is not written.
fn read_text(
    read: &dyn Fn(&str) -> Outcome,
    text: &str,
) -> Result<(Read, Outcome), Option<Outcome>> {
    // A load or a write that panics is a document that loads and is not written: a fault
    // wherever it is met, with the panic's words.
    let ours = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| read(text)))
        .unwrap_or_else(|panic| {
            let words = panic
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| panic.downcast_ref::<&str>().copied())
                .unwrap_or("no words");
            Outcome {
                loaded: true,
                written: Err(format!("{PANICKED}: {words}")),
                findings: Vec::new(),
            }
        });
    if !ours.loaded {
        return Err(None);
    }
    let Ok(written) = ours.written.clone() else {
        return Err(Some(ours));
    };
    let value = parse(&written).expect("a written text is JSON");
    let mut all = Vec::new();
    leaves(&value, &mut all);
    let mut paths = BTreeSet::new();
    members(&value, "", &mut paths);
    Ok((
        Read {
            written,
            value,
            leaves: all,
            members: paths,
            findings: ours.findings.iter().map(finding_text).collect(),
            at: ours.findings.iter().map(path_of).collect(),
        },
        ours,
    ))
}

/// The entries of `ours` that `base` does not have, each known one counted once.
fn fresh(base: &[String], ours: &[String]) -> Vec<String> {
    let mut left = base.to_vec();
    ours.iter()
        .filter(|leaf| match left.iter().position(|known| known == *leaf) {
            Some(index) => {
                left.swap_remove(index);
                false
            }
            None => true,
        })
        .cloned()
        .collect()
}

/// Whether the number leaf `leaf` is the `f32` nearest to the number `token` spells.
#[expect(
    clippy::cast_possible_truncation,
    reason = "the nearest f32 is the question"
)]
#[expect(
    clippy::float_cmp,
    reason = "the two are the nearest f32 of a number: equal when they are the same f32"
)]
fn near(leaf: &str, token: &str) -> bool {
    let (Some(held), Ok(spelled)) = (
        leaf.strip_prefix("n:").and_then(|n| n.parse::<f64>().ok()),
        token.parse::<f64>(),
    ) else {
        return false;
    };
    (held as f32) == (spelled as f32) && (held as f32).is_finite()
}

/// Whether one of the fresh leaves holds `replacement`.
fn holds(family: &str, fresh: &[String], replacement: &str) -> bool {
    let has = |leaf: &str| fresh.iter().any(|known| known == leaf);
    match replacement {
        "7" | "[7]" => has("n:7.0"),
        "0.5" => has("n:0.5"),
        "1e39" => has("n:1e39"),
        "4294967296" => has("n:4294967296.0"),
        "true" => {
            has("b:true") || (BOOL_AS_NUMBER.iter().any(|(of, _)| *of == family) && has("n:1.0"))
        }
        "\"x\"" => has("s:x"),
        "\"zz:bogus\"" => has("s:zz:bogus"),
        "123457" => has("n:123457.0"),
        "null" => has("null"),
        "{\"a\": 1}" => has("k:a"),
        _ => false,
    }
}

/// The prefix of the leaves a replacement may be carried in (see "carried").
fn carried_in(class: Class) -> &'static [&'static str] {
    match class {
        Class::Integer | Class::Fraction | Class::Wide | Class::OtherInt => &["n:"],
        Class::String | Class::OtherString => &["s:", "k:"],
        _ => &[],
    }
}

/// The read of the document without the member at a node, once judged.
enum Without {
    /// Not asked for yet.
    Unknown,
    /// Refused, or not written.
    Unread,
    /// Loaded and written.
    Read(Read),
}

impl Without {
    /// The read, when the document without the member at `node` loads and is written; the
    /// document is read the first time this is asked.
    fn get(
        &mut self,
        read: &dyn Fn(&str) -> Outcome,
        base: &Base<'_>,
        node: &Node,
    ) -> Option<&Read> {
        if matches!(self, Without::Unknown) {
            *self = without(base.text, node)
                .and_then(|text| read_text(read, &text).ok())
                .map_or(Without::Unread, |read| Without::Read(read.0));
        }
        match self {
            Without::Read(read) => Some(read),
            Without::Unknown | Without::Unread => None,
        }
    }
}

/// The unchanged document.
struct Base<'a> {
    family: &'a str,
    text: &'a str,
    /// The text as a value.
    value: Value,
    /// Every value of the text.
    nodes: &'a [Node],
    read: Read,
    /// The scalars of the text, keys left out.
    scalars: Vec<String>,
    /// Whether the member at a path is not read as a whole (see [`not_read`]), once asked.
    ignored: std::cell::RefCell<BTreeMap<Vec<String>, bool>>,
    /// What the document is written as with nothing of the author's but its identifier
    /// ([`bare_text`]): the members the steps put into every document of its version.
    /// `None` when that document does not load or is not written.
    bare: Option<Value>,
}

/// `text` with nothing of the author's but the version and the identifier: every member of
/// the payload but `description` taken out, and every member of the description but
/// `identifier`. `None` when a member cannot be taken out.
fn bare_text(text: &str) -> Option<String> {
    let mut text = text.to_owned();
    loop {
        let all = places::nodes(&text);
        let more = all.iter().find(|node| match node.path.as_slice() {
            [payload, member] => payload != "format_version" && member != "description",
            [_, description, member] => description == "description" && member != "identifier",
            _ => false,
        });
        let Some(node) = more else {
            return Some(text);
        };
        if node.index.iter().any(|index| *index) {
            return None;
        }
        text = places::without(&text, node)?;
    }
}

/// The unchanged document of a judgement: its text, its values and what it is written as.
fn base_of<'a>(
    family: &'a str,
    text: &'a str,
    value: Value,
    nodes: &'a [Node],
    written: Read,
    read: &dyn Fn(&str) -> Outcome,
) -> Base<'a> {
    let bare = bare_text(text)
        .and_then(|bare| read_text(read, &bare).ok())
        .map(|(bare, _)| bare.value);
    Base {
        bare,
        family,
        text,
        scalars: scalars_of_all(&value),
        value,
        nodes,
        read: written,
        ignored: std::cell::RefCell::default(),
    }
}

/// Whether two written texts are the same document but for the order of its members.
fn same_but_order(a: &Read, b: &Read) -> bool {
    let sorted = |leaves: &[String]| {
        let mut all = leaves.to_vec();
        all.sort();
        all
    };
    a.written == b.written || (a.members == b.members && sorted(&a.leaves) == sorted(&b.leaves))
}

/// The codes that say a member is dropped or not read, whatever it holds: the only findings
/// that make a place "told as not read". A finding of any other code (the
/// `MolangNotChecked` INFO at the payload, most of all) says nothing of a value below it.
const NOT_READ: &[&str] = &[
    "DroppedField",
    // The name four of `DroppedField`'s sites have now (a member read and kept nowhere):
    // the same statement of the same members, no new place.
    "MemberNotKept",
    "DroppedComponent",
    "DeprecatedMember",
    "UnrecognizedField",
    "UnrecognizedComponent",
    "InvalidType",
    "InvalidValue",
    // A value its member's reader does not take (a number where a component's object is
    // read): the member is not read.
    "MalformedInput",
    // "Two versions of the same component found": the newer-named one of the file goes.
    "DuplicateComponentVersion",
    // A member the game never looks up (the particle's legacy `camera_distance_fade`).
    "NoEffect",
];

/// The scalars of `value` (no keys), sorted.
fn scalars_of(value: &Value) -> Vec<String> {
    let mut all = Vec::new();
    leaves(value, &mut all);
    all.retain(|leaf| !leaf.starts_with("k:"));
    all.sort();
    all
}

/// Whether the load says the place is not read ("told as not read"): a finding of a
/// [`NOT_READ`] code of this load at the place itself, or at an enclosing member that is not
/// read AS A WHOLE (the document with `7` and with `"x"` there, or the document without it,
/// is written as the unchanged one). A finding at an enclosing member that IS read excuses
/// nothing below it, also where a step moved the member and the written text has it at
/// another place (a component told as deprecated and carried into a side-car: what is below
/// it is written, or told where it is).
fn not_read(
    read: &dyn Fn(&str) -> Outcome,
    base: &Base<'_>,
    node: &Node,
    outcome: &Outcome,
) -> bool {
    for finding in &outcome.findings {
        if !NOT_READ.contains(&code(finding).as_str()) {
            continue;
        }
        let at = path_of(finding);
        if at.is_empty() || !node.path.starts_with(&at) {
            continue;
        }
        // "Members of component %s will be thrown away as part of the upgrade": the game's
        // line, told once at the component, of every member beside its value.
        if at == node.path
            || finding_text(finding).contains("will be thrown away as part of the upgrade")
        {
            return true;
        }
        if let Some(known) = base.ignored.borrow().get(&at) {
            if *known {
                return true;
            }
            continue;
        }
        let whole = base.nodes.iter().find(|other| other.path == at);
        let ignored = whole.is_some_and(|whole| {
            let unchanged = |text: &str| {
                read_text(read, text).is_ok_and(|(other, _)| other.written == base.read.written)
            };
            // Whatever it holds, or (an element that must be of its kind to load, as a
            // bone another bone replaces) with the member taken out.
            ["7", "\"x\""].iter().all(|value| {
                unchanged(&format!(
                    "{}{value}{}",
                    &base.text[..whole.start],
                    &base.text[whole.end..]
                ))
            }) || without(base.text, whole).is_some_and(|text| {
                // But for the order of the members (a bone that takes the place of the
                // one it replaces).
                read_text(read, &text).is_ok_and(|(other, _)| same_but_order(&other, &base.read))
            })
        });
        base.ignored.borrow_mut().insert(at, ignored);
        if ignored {
            return true;
        }
    }
    false
}

/// Whether the load has a `ClampedValue` finding at the place itself.
fn clamped_at(node: &Node, outcome: &Outcome) -> bool {
    outcome
        .findings
        .iter()
        .any(|finding| code(finding) == "ClampedValue" && path_of(finding) == node.path)
}

/// Whether `at` is a component of an entity (`minecraft:entity/components/<key>` or
/// `minecraft:entity/component_groups/<group>/<key>`): the place the information names when
/// a failed validation of the game's legacy reader defaults the whole component (no other
/// `ValueRewritten` above a place counts for the two rules that use it).
fn component_level(at: &[String]) -> bool {
    match at {
        [payload, section, _] => payload == "minecraft:entity" && section == "components",
        [payload, section, _, _] => payload == "minecraft:entity" && section == "component_groups",
        _ => false,
    }
}

/// Whether the load says at the place itself that its value is read as another (an information
/// `ValueRewritten` there): the same words whatever value stood there, so also where the
/// unchanged document was told so (as [`clamped_at`] for a clamp).
fn rewritten_at(node: &Node, outcome: &Outcome) -> bool {
    outcome
        .findings
        .iter()
        .any(|finding| code(finding) == "ValueRewritten" && path_of(finding) == node.path)
}

/// Whether a finding at `at` is at the member the place's value MOVED to ("said elsewhere"):
/// `at`, or a member above it, is no member of the file, and the unchanged
/// document's written text holds there exactly the scalars the file holds at the place.
fn moved_to(base: &Base<'_>, node: &Node, at: &[String]) -> bool {
    let Some(held) = places::lookup(&base.value, &node.path) else {
        return false;
    };
    let held = scalars_of(held);
    if held.is_empty() {
        return false;
    }
    (1..=at.len()).any(|length| {
        let member = &at[..length];
        places::lookup(&base.value, member).is_none()
            && places::lookup(&base.read.value, member)
                .is_some_and(|written| scalars_of(written) == held)
    })
}

/// The text of the unchanged document with `value` at `node`.
fn replaced(base: &Base<'_>, node: &Node, value: &str) -> String {
    format!(
        "{}{value}{}",
        &base.text[..node.start],
        &base.text[node.end..]
    )
}

/// The new findings of the load of the changed document, by where they are.
#[derive(Default)]
struct Newly {
    /// The ones at another place than the replaced one, as `code at path`.
    told: Vec<String>,
    /// The paths of those.
    elsewhere: Vec<Vec<String>>,
    /// Whether one of them is at the member the value moved to.
    moved: bool,
    /// One is at the place or below it.
    said_here: bool,
    /// One is at an enclosing member.
    said_above: bool,
}

/// The findings of the load of the document with `replacement` at `node` that the unchanged
/// document does not have.
fn new_findings(base: &Base<'_>, node: &Node, replacement: &str, outcome: &Outcome) -> Newly {
    let mut left = base.read.findings.clone();
    let mut newly = Newly::default();
    for finding in &outcome.findings {
        let text = finding_text(finding);
        if let Some(index) = left.iter().position(|known| *known == text) {
            left.swap_remove(index);
            continue;
        }
        let at = path_of(finding);
        if at.starts_with(&node.path) {
            newly.said_here = true;
            continue;
        }
        if !at.is_empty() && node.path.starts_with(&at) {
            newly.said_above = true;
            continue;
        }
        newly.elsewhere.push(at.clone());
        if moved_to(base, node, &at) {
            newly.moved = true;
            println!(
                "legacy-elsewhere: {} | {} | {replacement} | {} at `{}`",
                base.family,
                node.shown(),
                code(finding),
                finding.location.json_path
            );
        }
        newly.told.push(format!(
            "{} at `{}`",
            code(finding),
            finding.location.json_path
        ));
    }
    newly
}

/// Whether a leaf is the largest finite double of the negative sign.
fn clamped_low(leaves: &[String]) -> bool {
    leaves
        .iter()
        .any(|leaf| leaf == "n:-1.7976931348623157e308")
}

/// Whether a finding is at a member above `node`.
fn told_above(node: &Node, outcome: &Outcome) -> bool {
    outcome.findings.iter().any(|finding| {
        let at = path_of(finding);
        !at.is_empty() && node.path.starts_with(&at)
    })
}

/// A fault of a replacement: its kind and detail.
type Fault = (&'static str, String);

/// What `judge` knows of the changed document while it decides.
struct Changed<'a, 'b> {
    base: &'a Base<'b>,
    node: &'a Node,
    /// The load of the changed document.
    outcome: &'a Outcome,
    /// What the changed document is written as.
    ours: &'a Read,
    class: Class,
    replacement: &'a str,
    /// A second value of the replacement's class.
    twin: &'a str,
    /// The leaves the written text has and the unchanged document's does not.
    new: Vec<String>,
    /// The new findings at another place, as `code at path`.
    told: Vec<String>,
}

impl Changed<'_, '_> {
    /// The fault text: `what`, with what the written text lost and gained.
    fn detail(&self, what: &str) -> String {
        let gone = fresh(&self.ours.leaves, &self.base.read.leaves);
        format!(
            "{what}; written without {:?}, with {:?}; new findings elsewhere: {:?}",
            gone.iter().take(6).collect::<Vec<_>>(),
            self.new.iter().take(6).collect::<Vec<_>>(),
            self.told
        )
    }

    /// Whether the number class is of the ones a bound can clamp.
    fn is_number(&self) -> bool {
        matches!(
            self.class,
            Class::Integer | Class::Fraction | Class::Wide | Class::Huge | Class::OtherInt
        )
    }

    /// The verdict when the written text is the unchanged document's own.
    fn written_as_before(&self, read: &dyn Fn(&str) -> Outcome) -> Result<&'static str, Fault> {
        let (base, node, outcome) = (self.base, self.node, self.outcome);
        // A member the load says it does not read, whatever its value.
        if not_read(read, base, node, outcome) {
            return Ok("told as not read");
        }
        // Both loads say the enclosing component is rewritten as a whole (a legacy reader
        // whose failed validation keeps its defaults): nothing below it is read.
        let whole = |findings: &[String], at: &[Vec<String>]| {
            findings.iter().zip(at).any(|(text, at)| {
                component_level(at)
                    && *at != node.path
                    && node.path.starts_with(at)
                    && text.starts_with("ValueRewritten")
            })
        };
        let ours_at: Vec<Vec<String>> = outcome.findings.iter().map(path_of).collect();
        let ours_text: Vec<String> = outcome.findings.iter().map(finding_text).collect();
        if whole(&base.read.findings, &base.read.at) && whole(&ours_text, &ours_at) {
            return Ok("told as not read");
        }
        // The same value in another form (a short form for the long one the document has):
        // a second value of the replacement's class is written as another text.
        if !carried_in(self.class).is_empty()
            && let Ok((other, _)) = read_text(read, &replaced(base, node, self.twin))
            && other.written != self.ours.written
        {
            return Ok("carried");
        }
        if self.is_number() && clamped_at(node, outcome) {
            return Ok("said");
        }
        if rewritten_at(node, outcome) {
            return Ok("said");
        }
        // The UNCHANGED document was told at the place that its value is not read (a
        // generated input of another type, read as the member's default), this one is not
        // told, and the text the two are written as holds the replacement: the replacement
        // is the value the member was read as.
        let told_before = base
            .read
            .findings
            .iter()
            .zip(&base.read.at)
            .any(|(text, at)| {
                *at == node.path
                    && NOT_READ.iter().any(|code| text.starts_with(code))
                    && !outcome
                        .findings
                        .iter()
                        .any(|finding| finding_text(finding) == *text)
            });
        if told_before && holds(base.family, &base.read.leaves, self.replacement) {
            return Ok("held");
        }
        Err((
            "no-effect",
            self.detail("the written text is the unchanged document's"),
        ))
    }

    /// The members of the unchanged document's written text that the replacement loses and
    /// the document without the member (`removed`) keeps, other than what is the replaced
    /// member's own or what the load says it drops.
    fn lost_members(&self, removed: &Read) -> Vec<&String> {
        let (base, node) = (self.base, self.node);
        let own = format!("/{}", node.starred().join("/"));
        // The places the load says it drops or replaces, indices as `*`; a finding at the
        // payload key or above says nothing of a member.
        let dropped: Vec<String> = self
            .outcome
            .findings
            .iter()
            .filter(|finding| NOT_READ.contains(&code(finding).as_str()))
            .map(path_of)
            .filter(|at| at.len() > 2)
            .map(|at| {
                let starred: Vec<&str> = at
                    .iter()
                    .map(|segment| {
                        if segment.bytes().all(|byte| byte.is_ascii_digit()) {
                            "*"
                        } else {
                            segment.as_str()
                        }
                    })
                    .collect();
                format!("/{}", starred.join("/"))
            })
            .collect();
        base.read
            .members
            .iter()
            .filter(|member| {
                !self.ours.members.contains(*member) && removed.members.contains(*member)
            })
            // What is below the replaced place is the replaced member's own, also where
            // the document without the member gets a default there (a step that inserts
            // the member when it is absent).
            .filter(|member| {
                member
                    .strip_prefix(own.as_str())
                    .is_none_or(|rest| !rest.is_empty() && !rest.starts_with('/'))
            })
            // A member the load says it drops or replaces (a finding of a [`NOT_READ`]
            // code at it or above it, below the payload) is told of: the member a step
            // writes the replaced one over.
            .filter(|member| {
                !dropped.iter().any(|at| {
                    member
                        .strip_prefix(at.as_str())
                        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
                })
            })
            .collect()
    }

    /// Whether a float member holds the replacement as the nearest `f32`, or the value is
    /// carried as another: told from an integer member saturating at a bound (the same
    /// `f32`) by the second number, which a float holds as another.
    fn carried_by_the_twin(&self, read: &dyn Fn(&str) -> Outcome) -> Option<&'static str> {
        let base = self.base;
        let prefixes = carried_in(self.class);
        if prefixes.is_empty() {
            return None;
        }
        let (other, _) = read_text(read, &replaced(base, self.node, self.twin)).ok()?;
        let other_new = fresh(&base.read.leaves, &other.leaves);
        let of_type = |leaves: &[String]| -> Vec<String> {
            leaves
                .iter()
                .filter(|leaf| prefixes.iter().any(|prefix| leaf.starts_with(prefix)))
                .cloned()
                .collect()
        };
        if self.class == Class::Wide {
            (self.new.iter().any(|leaf| near(leaf, self.replacement))
                && other_new
                    .iter()
                    .any(|leaf| near(leaf, self.twin) && !near(leaf, self.replacement)))
            .then_some("held")
        } else {
            (of_type(&self.new) != of_type(&other_new)).then_some("carried")
        }
    }

    /// Whether the unchanged document was told at an enclosing member that its value is
    /// rewritten as a whole (a legacy reader whose failed validation keeps the component's
    /// defaults), this one is not: the replacement makes the component read, and what
    /// changes beside it is what that finding said.
    fn component_restored(&self) -> bool {
        let (base, node) = (self.base, self.node);
        base.read
            .findings
            .iter()
            .zip(&base.read.at)
            .any(|(text, at)| {
                component_level(at)
                    && *at != node.path
                    && node.path.starts_with(at)
                    && text.starts_with("ValueRewritten")
                    && !self
                        .outcome
                        .findings
                        .iter()
                        .any(|finding| finding_text(finding) == *text)
            })
    }
}

/// Whether the written text of a replacement told at the member alone loses nothing beside
/// it that no finding tells of.
fn told_alone(
    base: &Base<'_>,
    node: &Node,
    ours: &Read,
    elsewhere: &[Vec<String>],
    outcome: &Outcome,
    told: &[String],
) -> Result<(), Fault> {
    // A finding the unchanged document has too, at the member that holds a lost
    // scalar, tells of the loss as well (see `judge_removed`).
    let told_at: Vec<Vec<String>> = elsewhere
        .iter()
        .cloned()
        .chain(outcome.findings.iter().map(path_of))
        .collect();
    let lost = lost_beside(base, node, ours, &told_at);
    if !lost.is_empty() {
        return Err((
            "collateral",
            format!(
                "told at the member alone, and the written text loses {lost:?} beside it; new findings elsewhere: {told:?}"
            ),
        ));
    }
    Ok(())
}

/// What became of the document with `replacement` at `node`: the outcome, or the fault's kind
/// and detail.
fn judge(
    read: &dyn Fn(&str) -> Outcome,
    base: &Base<'_>,
    node: &Node,
    (replacement, twin, class): (&str, &str, Class),
    removed: &mut Without,
) -> Result<&'static str, Fault> {
    let (ours, outcome) = match read_text(read, &replaced(base, node, replacement)) {
        Ok(read) => (Some(read.0), read.1),
        Err(None) => return Ok("refused"),
        Err(Some(outcome)) => (None, outcome),
    };
    let Newly {
        told,
        elsewhere,
        moved,
        said_here,
        said_above,
    } = new_findings(base, node, replacement, &outcome);
    if said_above {
        return Ok("said");
    }
    if said_here {
        // Told at the member alone: what the written text loses BESIDE the member (a
        // component that goes whole for one member of another type) is told above the
        // member or at what is lost, or it is a fault.
        if let Some(ours) = &ours {
            told_alone(base, node, ours, &elsewhere, &outcome, &told)?;
        }
        return Ok("said");
    }
    if moved {
        // A finding the unchanged document does not have, at the member a step moved the
        // value to. A new finding at any OTHER place says nothing of this one.
        return Ok("said elsewhere");
    }
    let Some(ours) = ours else {
        return Err((
            "not-written",
            format!(
                "{}; new findings elsewhere: {told:?}",
                outcome.written.err().unwrap_or_default()
            ),
        ));
    };
    let new = fresh(&base.read.leaves, &ours.leaves);
    if holds(base.family, &new, replacement)
        || read_non_strict(base, node, replacement, &new, &ours.leaves)
    {
        return Ok("held");
    }
    // The document without the member: what a `null` there is read as, and what of `W` is
    // owed to the member.
    let removed = removed.get(read, base, node);
    // Held where the replaced value held the same scalar (`true` for `{"value": true}`, or
    // for a `"true"` the legacy reader takes as the bool): the written text has it and the
    // document without the member does not.
    if let Some(removed) = removed
        && holds(
            base.family,
            &fresh(&removed.leaves, &ours.leaves),
            replacement,
        )
    {
        return Ok("held");
    }
    let nothing = matches!(class, Class::Null | Class::EmptyArray | Class::EmptyObject);
    // The document without the member, but for the order of its members (a step that
    // inserts an absent member puts it last).
    if nothing
        && let Some(removed) = removed
        && same_but_order(removed, &ours)
    {
        return Ok("absent");
    }
    // A number no double holds is written as the largest finite double of its sign. Each
    // number has ONE finding (`one_finding_per_number.rs`): where the place is told of
    // already, by a finding the unchanged document has in the same words (a definition
    // that is no object, whatever it is), that finding stands for the clamp too.
    if replacement == "-1e999" && clamped_low(&new) && told_above(node, &outcome) {
        return Ok("said");
    }
    let changed = Changed {
        base,
        node,
        outcome: &outcome,
        ours: &ours,
        class,
        replacement,
        twin,
        new,
        told,
    };
    if ours.written == base.read.written {
        return changed.written_as_before(read);
    }
    if let Some(removed) = removed {
        let lost = changed.lost_members(removed);
        if !lost.is_empty() {
            return Err((
                "collateral",
                changed.detail(&format!(
                    "members the replaced one does not own are gone: {:?}",
                    lost.iter().take(6).collect::<Vec<_>>()
                )),
            ));
        }
    }
    if matches!(class, Class::EmptyArray | Class::EmptyObject) {
        return Ok("empty");
    }
    // A float member holds the nearest `f32`; told from an integer member saturating at a
    // bound (the same `f32`) by the second number, which a float holds as another.
    if let Some(verdict) = changed.carried_by_the_twin(read) {
        return Ok(verdict);
    }
    // A number the load holds at a bound of its range, and says so where it is: the same
    // words whatever number stood there, so also where the unchanged document was told so.
    if changed.is_number() && clamped_at(node, &outcome) {
        return Ok("said");
    }
    if rewritten_at(node, &outcome) || changed.component_restored() {
        return Ok("said");
    }
    Err(("changed", changed.detail("neither held nor carried")))
}

/// The scalars of `value` in the order of the text, keys left out.
fn scalars_of_all(value: &Value) -> Vec<String> {
    let mut all = Vec::new();
    leaves(value, &mut all);
    all.retain(|leaf| !leaf.starts_with("k:"));
    all
}

/// The scalars of `text`, keys left out.
fn scalars_in(text: &str) -> Vec<String> {
    let mut all = Vec::new();
    if let Some(value) = parse(text) {
        leaves(&value, &mut all);
    }
    all.retain(|leaf| !leaf.starts_with("k:"));
    all
}

/// How many of `all` are `leaf`.
fn count(all: &[String], leaf: &String) -> usize {
    all.iter().filter(|one| *one == leaf).count()
}

/// The scalars the author wrote OUTSIDE the member at `node` that the unchanged document's
/// written text holds and `ours` holds fewer of, with no finding of `told` at a member of the
/// file that holds one (and is not the place itself, nor below it). An element of a list is
/// told of as the list is: a list is one value, and a finding at one of its elements says
/// that the list is not what its member takes.
fn lost_beside(base: &Base<'_>, node: &Node, ours: &Read, told: &[Vec<String>]) -> Vec<String> {
    let elements = node.index.iter().rev().take_while(|index| **index).count();
    let unit = &node.path[..node.path.len() - elements];
    let node = base
        .nodes
        .iter()
        .find(|other| other.path == unit)
        .unwrap_or(node);
    let inside = scalars_in(&base.text[node.start..node.end]);
    let outside = fresh(&inside, &base.scalars);
    let held_outside = {
        let mut all = Vec::new();
        leaves_outside(&base.read.value, &node.path, &mut all).then_some(all)
    };
    let held = held_outside.as_deref().unwrap_or(&base.read.leaves);
    // A text written at the file's own format version (a document the newest format has no
    // text for) leaves the members a step of the upgrade SYNTHESISES to the reading, which
    // puts them there again. Such a member holds no scalar of the author's: it is one that
    // neither the file nor this text has and that the document without anything of the
    // author's (`Base::bare`) is written with too. A member a step moved or renamed holds
    // the author's value, is in no bare document, and counts.
    let at_its_own_version = written_version(&ours.value) != written_version(&base.read.value);
    let held_by_both = {
        let mut all = Vec::new();
        leaves_of_members_kept(
            &base.read.value,
            Some(&base.value),
            Some(&ours.value),
            base.bare.as_ref(),
            Some(&node.path),
            &mut all,
        );
        all
    };
    let held = if at_its_own_version {
        &held_by_both
    } else {
        held
    };
    // The version the file names is no value the written text owes the author.
    let version = base
        .value
        .as_object()
        .and_then(|root| root.get("format_version"))
        .and_then(places::scalar);
    let mut lost: Vec<String> = fresh(&ours.leaves, &base.read.leaves)
        .into_iter()
        .filter(|leaf| !leaf.starts_with("k:") && !inside.contains(leaf))
        .filter(|leaf| version.as_ref() != Some(leaf))
        .filter(|leaf| count(&ours.leaves, leaf) < count(&outside, leaf).min(count(held, leaf)))
        .filter(|leaf| {
            !told.iter().any(|at| {
                !at.is_empty()
                    && !at.starts_with(&node.path)
                    && !node.path.starts_with(at)
                    && places::lookup(&base.value, at)
                        .is_some_and(|member| scalars_of(member).contains(leaf))
            })
        })
        .collect();
    lost.sort();
    lost.dedup();
    lost
}

/// What the PRESENCE of the member at `node` costs the author: the scalars the author wrote
/// outside it that the document WITHOUT the member (`without`) is written with and the
/// document itself is written with fewer of. A fault unless a finding the document without
/// the member does not have is at an enclosing member (a component that goes whole for a
/// member it does not know is told at the component, not at the member alone), or at a
/// member of the file that holds a lost scalar (the member a step wrote over).
fn judge_displaced(base: &Base<'_>, node: &Node, without: &Read) -> Result<(), String> {
    let inside = scalars_in(&base.text[node.start..node.end]);
    let outside = fresh(&inside, &base.scalars);
    // What each text holds outside the member's own place: a default the document without
    // the member is written with THERE is no value of the author's.
    let beside = |read: &Read| {
        let mut all = Vec::new();
        if leaves_outside(&read.value, &node.path, &mut all) {
            all
        } else {
            read.leaves.clone()
        }
    };
    let (with, mut without_it) = (beside(&base.read), beside(without));
    // A document written at another format version than the one with the member (a block
    // with an unknown member is written at its own version, the one without it at
    // the newest) holds the members the steps of the upgrade synthesise only in the second:
    // they are no scalars of the author's (see `lost_beside`).
    if written_version(&without.value) != written_version(&base.read.value) {
        let mut kept = Vec::new();
        leaves_of_members_kept(
            &without.value,
            Some(&base.value),
            Some(&base.read.value),
            base.bare.as_ref(),
            Some(&node.path),
            &mut kept,
        );
        without_it = kept;
    }
    // The version the file names is no value the written text owes the author.
    let version = base
        .value
        .as_object()
        .and_then(|root| root.get("format_version"))
        .and_then(places::scalar);
    let mut lost: Vec<String> = fresh(&with, &without_it)
        .into_iter()
        .filter(|leaf| !leaf.starts_with("k:") && !inside.contains(leaf))
        .filter(|leaf| version.as_ref() != Some(leaf))
        .filter(|leaf| count(&with, leaf) < count(&outside, leaf).min(count(&without_it, leaf)))
        .collect();
    lost.sort();
    lost.dedup();
    if lost.is_empty() {
        return Ok(());
    }
    let told = base.read.told_beyond(without);
    let says = told.iter().any(|at| {
        !at.is_empty()
            && ((node.path.starts_with(at) && at.len() < node.path.len())
                || (!at.starts_with(&node.path)
                    && places::lookup(&base.value, at).is_some_and(|member| {
                        scalars_of(member).iter().any(|leaf| lost.contains(leaf))
                    })))
    });
    if says {
        return Ok(());
    }
    Err(format!(
        "with the member the written text loses {lost:?}, which the document without it is written with, and no finding above the member or at the lost one says so; findings the document without it does not have: {:?}",
        told.iter().map(|at| at.join("/")).collect::<Vec<_>>()
    ))
}

/// What became of the document without the member or element at `node`: a member the load
/// needs (the component is dropped without it) takes nothing of the author's with it in
/// silence. A fault is a scalar the author wrote outside the removed member that the written
/// text loses with no finding the unchanged document does not have.
fn judge_removed(
    read: &dyn Fn(&str) -> Outcome,
    base: &Base<'_>,
    node: &Node,
    removed: &mut Without,
) -> Result<&'static str, (&'static str, String)> {
    let Some(text) = without(base.text, node) else {
        return Ok("no member");
    };
    let (ours, outcome) = match read_text(read, &text) {
        Ok(read) => read,
        Err(None) => {
            *removed = Without::Unread;
            return Ok("refused");
        }
        Err(Some(outcome)) => {
            *removed = Without::Unread;
            let found: Vec<String> = outcome.findings.iter().map(finding_text).collect();
            if fresh(&base.read.findings, &found).is_empty() {
                return Err(("not-written", outcome.written.err().unwrap_or_default()));
            }
            return Ok("said");
        }
    };
    // The findings the unchanged document does not have, each with its path.
    let told: Vec<Vec<String>> = {
        let mut left = base.read.findings.clone();
        outcome
            .findings
            .iter()
            .filter(|finding| {
                let text = finding_text(finding);
                match left.iter().position(|known| *known == text) {
                    Some(index) => {
                        left.swap_remove(index);
                        false
                    }
                    None => true,
                }
            })
            .map(path_of)
            .collect()
    };
    // The scalars of the document outside the removed member, and inside it.
    let scalars = |text: &str| -> Vec<String> {
        let mut all = Vec::new();
        if let Some(value) = parse(text) {
            leaves(&value, &mut all);
        }
        all.retain(|leaf| !leaf.starts_with("k:"));
        all
    };
    let inside = scalars(&base.text[node.start..node.end]);
    let outside = fresh(&inside, &base.scalars);
    // A scalar the author wrote outside the member, of which the written text now holds
    // fewer than the author wrote there and than the unchanged document's text held (a
    // default the dropped component wrote is no loss of the author's).
    let count = |all: &[String], leaf: &String| all.iter().filter(|one| *one == leaf).count();
    // Where the written text has the member at the place the file has it, what it holds
    // outside that place is what is compared: a scalar the member itself was written as (a
    // number read as a whole one) is no scalar of the rest.
    let held_outside = parse(&base.read.written).and_then(|written| {
        let mut all = Vec::new();
        leaves_outside(&written, &node.path, &mut all).then_some(all)
    });
    let held = held_outside.as_deref().unwrap_or(&base.read.leaves);
    // The version the file says it has is no value the written text owes the author: a
    // family writes the newest version (a geometry its own era's while a member of that era
    // holds it there).
    let version = parse(base.text)
        .as_ref()
        .and_then(Value::as_object)
        .and_then(|root| root.get("format_version"))
        .and_then(places::scalar);
    let mut lost: Vec<String> = fresh(&ours.leaves, &base.read.leaves)
        .into_iter()
        .filter(|leaf| !leaf.starts_with("k:") && !inside.contains(leaf))
        .filter(|leaf| version.as_ref() != Some(leaf))
        .filter(|leaf| count(&ours.leaves, leaf) < count(&outside, leaf).min(count(held, leaf)))
        .collect();
    lost.dedup();
    *removed = Without::Read(ours);
    if lost.is_empty() {
        return Ok("removed");
    }
    // Said: a new finding above the removed member (the component that is dropped without
    // it), or at a member of the file that holds one of the lost scalars. A new finding at
    // any other place says nothing of the loss.
    // A document without its `format_version` is told of at the root (`MissingFormatVersion`):
    // it is read as the version the family takes for a file without one, which says what the
    // author's values become.
    let versionless = node.path == ["format_version"];
    // A finding the unchanged document has too, at the member of the file that holds a lost
    // scalar, tells of that loss all the same: the document without the member loses what it
    // loses there (a value the other document holds again as a default, `rideable`'s
    // `controlling_seat` 1 and `seat_count` 1, is the same scalar to the judge).
    let at_the_lost = outcome.findings.iter().map(path_of).any(|at| {
        !at.is_empty()
            && places::lookup(&base.value, &at)
                .is_some_and(|member| scalars_of(member).iter().any(|leaf| lost.contains(leaf)))
    });
    let says = at_the_lost
        || told.iter().any(|at| {
            (!at.is_empty() || versionless)
                && (node.path.starts_with(at)
                    || places::lookup(&base.value, at).is_some_and(|member| {
                        scalars_of(member).iter().any(|leaf| lost.contains(leaf))
                    }))
        });
    if says {
        return Ok("said");
    }
    Err((
        "collateral",
        format!(
            "without the member the written text loses {lost:?} and no finding above it or at the lost member says so; new findings: {told:?}"
        ),
    ))
}

/// The class of the value at `node`.
fn class_of(text: &str, node: &Node) -> Class {
    let token = &text[node.start..node.end];
    match node.kind {
        Kind::Number => {
            let spelled = token.parse::<f64>().unwrap_or(f64::NAN);
            if spelled.abs() > f64::from(f32::MAX) {
                Class::Huge
            } else if spelled.fract() != 0.0 {
                Class::Fraction
            } else if spelled.abs() > f64::from(i32::MAX) {
                Class::Wide
            } else {
                Class::Integer
            }
        }
        Kind::String => Class::String,
        Kind::Other if token == "null" => Class::Null,
        Kind::Other => Class::Bool,
        Kind::Array if token[1..token.len() - 1].trim().is_empty() => Class::EmptyArray,
        Kind::Array => Class::Array,
        Kind::Object if token[1..token.len() - 1].trim().is_empty() => Class::EmptyObject,
        Kind::Object => Class::Object,
    }
}

/// Whether a place is exempt (see [`EXEMPT`]).
fn exempt(family: &str, node: &Node, replacement: &str) -> bool {
    let starred = node.starred().join("/");
    EXEMPT.iter().any(|(of, end, with, _entry)| {
        *of == family && starred.ends_with(end) && (*with == "*" || *with == replacement)
    })
}

// --- the probe ---------------------------------------------------------------------------------

/// The counts of a family's test.
#[derive(Default)]
struct Tally {
    documents: usize,
    texts: usize,
    outcomes: BTreeMap<&'static str, usize>,
    faults: Vec<String>,
    /// The faults by version.
    by_version: BTreeMap<String, usize>,
    /// The documents by version.
    versions: BTreeMap<String, usize>,
    /// The places probed: reader, version, path with indices as `*`.
    places: BTreeSet<String>,
    capped: usize,
    /// The documents probed in which a legacy member stands beside its successor.
    beside: usize,
    /// The generated inputs probed: a member a step adds, already there; an unknown member;
    /// one more element.
    seeded: usize,
    unknown: usize,
    extra: usize,
    /// The seconds spent on the documents as they are (with the two below), on the inputs
    /// with a member a step adds, and on those with an unknown member or one more element;
    /// the texts of the last.
    seconds: [f64; 3],
    added_texts: usize,
}

/// The key of a place: the reader, the version, the path (indices as `*`) and the names of
/// the sibling members (a step reads a member by what stands beside it). A family of few
/// texts is probed at every place of every document; a generated input at its one member,
/// once per class of the value it was given.
fn place_key(family: &Family, name: &str, legacy: &Legacy, value: &Value, node: &Node) -> String {
    // A generated input's own member is probed once per place whatever stands beside it (in
    // the first document a step adds the member to, or that has the object or the list).
    let alone = legacy.only.as_ref() == Some(&node.path);
    let siblings = places::lookup(value, &node.path[..node.path.len().saturating_sub(1)])
        .and_then(Value::as_object)
        .filter(|_| !alone)
        .map(|object| {
            let mut keys: Vec<&str> = object.iter().map(|(key, _)| key.as_str()).collect();
            keys.sort_unstable();
            keys.join(",")
        })
        .unwrap_or_default();
    let document = if LARGE.contains(&family.name) {
        ""
    } else {
        legacy.text.as_str()
    };
    let only = legacy
        .only
        .as_ref()
        .and_then(|only| places::lookup(value, only))
        .map(|given| format!(" given {}", kind_name(given)))
        .unwrap_or_default();
    format!(
        "{name} {} {} [{siblings}]{only} {document}",
        legacy.version,
        node.starred().join("/")
    )
}

/// What kind of JSON value `value` is.
fn kind_name(value: &Value) -> &'static str {
    if value.is_object() {
        "an object"
    } else if value.is_array() {
        "a list"
    } else if value.is_string() {
        "a string"
    } else if value.is_number() {
        "a number"
    } else {
        "a bool or null"
    }
}

/// Probes one document with the readers that take it.
fn probe(family: &Family, legacy: &Legacy, tally: &mut Tally) {
    for (name, read) in &family.readers {
        probe_with(family, name, read.as_ref(), legacy, tally);
    }
}

/// The unchanged document read; `Err` with whether the document counts as probed when it
/// is not read: a load that panics is a fault noted, one that is refused is not probed.
fn read_unchanged(
    family: &Family,
    name: &str,
    read: &dyn Fn(&str) -> Outcome,
    legacy: &Legacy,
    tally: &mut Tally,
) -> Result<Read, bool> {
    match read_text(read, &legacy.text) {
        Ok((base, _)) => Ok(base),
        Err(Some(outcome))
            if outcome
                .written
                .as_ref()
                .is_err_and(|e| e.starts_with(PANICKED)) =>
        {
            tally.faults.push(format!(
                "{} | {} | {} | (as it is) | not-written | {} | in {} ({name})",
                family.name,
                legacy.version,
                legacy
                    .only
                    .as_ref()
                    .map(|only| only.join("/"))
                    .unwrap_or_default(),
                outcome.written.err().unwrap_or_default(),
                legacy.from
            ));
            Err(true)
        }
        Err(_) => Err(false),
    }
}

/// Notes a fault when the text the unchanged document is written as does not load again and
/// is written the same.
fn note_unstable(
    family: &Family,
    name: &str,
    read: &dyn Fn(&str) -> Outcome,
    legacy: &Legacy,
    base: &Read,
    tally: &mut Tally,
) {
    // The text a document is written as loads again and is written the same, for
    // every document as it is and every generated input: a step that carries a value the
    // newest format's reader does not take is a fault. But for an item written
    // without a component of the game's: that text reads as a legacy item (recorded
    // and not changed).
    let as_legacy_item = family.name == "item"
        && base
            .value
            .as_object()
            .and_then(|root| root.get("minecraft:item"))
            .and_then(Value::as_object)
            .and_then(|item| item.get("components"))
            .and_then(Value::as_object)
            .is_some_and(|components| {
                !components
                    .iter()
                    .any(|(key, _)| key.as_str().starts_with("minecraft:"))
            });
    let stable =
        read_text(read, &base.written).is_ok_and(|(again, _)| again.written == base.written);
    if !stable && !as_legacy_item {
        tally.faults.push(format!(
            "{} | {} | {} | (as it is) | not-reread | the written text is not what its own reading writes: {} | in {} ({name})",
            family.name,
            legacy.version,
            legacy.only.as_ref().map(|only| only.join("/")).unwrap_or_default(),
            base.written.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(300).collect::<String>(),
            legacy.from
        ));
    }
}

/// Probes the document at one of its members: the document without it, and with each
/// replacement of another class.
fn probe_node(
    family: &Family,
    name: &str,
    read: &dyn Fn(&str) -> Outcome,
    legacy: &Legacy,
    base: &Base<'_>,
    node: &Node,
    tally: &mut Tally,
) {
    let class = class_of(&legacy.text, node);
    let mut removed = Without::Unknown;
    let fault = |replacement: &str, kind: &str, detail: &str| -> String {
        let detail: String = detail.replace('\n', " ").chars().take(420).collect();
        format!(
            "{} | {} | {} | {replacement} | {kind} | {detail} | in {} ({name})",
            family.name,
            legacy.version,
            node.shown(),
            legacy.from
        )
    };
    if exempt(family.name, node, "(removed)") {
        *tally.outcomes.entry("exempt").or_default() += 1;
    } else if tally.texts < BUDGET {
        tally.texts += 1;
        match judge_removed(read, base, node, &mut removed) {
            Ok(outcome) => *tally.outcomes.entry(outcome).or_default() += 1,
            Err((kind, detail)) => {
                *tally.by_version.entry(legacy.version.clone()).or_default() += 1;
                tally.faults.push(fault("(removed)", kind, &detail));
            }
        }
        // What the member's presence costs the author elsewhere (the version the file
        // names is no member of the kind: without it the file is another document).
        if let Without::Read(without) = &removed
            && node.path != ["format_version"]
            && !exempt(family.name, node, "(present)")
            && let Err(detail) = judge_displaced(base, node, without)
        {
            *tally.by_version.entry(legacy.version.clone()).or_default() += 1;
            tally.faults.push(fault("(present)", "displaced", &detail));
        }
    }
    // A `null`, `[]` or `{}` that is written as the unchanged document is no fault
    // where the member is read (another replacement of it is told, held or refused):
    // the document had the defaults there.
    let mut nothing: Vec<String> = Vec::new();
    let mut read_at_all = false;
    for replacement in REPLACEMENTS {
        if replacement.2 == class {
            continue;
        }
        if exempt(family.name, node, replacement.0) {
            *tally.outcomes.entry("exempt").or_default() += 1;
            continue;
        }
        // `true` where the number 1 stands is the same value in a family that
        // writes a Molang `true` as 1: no replacement.
        if replacement.0 == "true"
            && BOOL_AS_NUMBER.iter().any(|(of, _)| *of == family.name)
            && legacy.text[node.start..node.end].parse::<f64>() == Ok(1.0)
        {
            continue;
        }
        if tally.texts >= BUDGET {
            tally.capped += 1;
            continue;
        }
        tally.texts += 1;
        match judge(read, base, node, *replacement, &mut removed) {
            Ok(outcome) => {
                read_at_all |= !matches!(outcome, "absent" | "empty");
                *tally.outcomes.entry(outcome).or_default() += 1;
            }
            Err((kind, detail)) => {
                let fault = fault(replacement.0, kind, &detail);
                let empty = matches!(
                    replacement.2,
                    Class::Null | Class::EmptyArray | Class::EmptyObject
                );
                if kind == "no-effect" && empty {
                    nothing.push(fault);
                } else {
                    *tally.by_version.entry(legacy.version.clone()).or_default() += 1;
                    tally.faults.push(fault);
                }
            }
        }
    }
    if read_at_all {
        *tally.outcomes.entry("empty").or_default() += nothing.len();
    } else {
        *tally.by_version.entry(legacy.version.clone()).or_default() += nothing.len();
        tally.faults.append(&mut nothing);
    }
}

/// The members no document has: an unknown member in every object, one more element in
/// every list. Each is a document of its own, probed at that member.
fn probe_made_members(
    family: &Family,
    name: &str,
    read: &dyn Fn(&str) -> Outcome,
    legacy: &Legacy,
    base: &Base<'_>,
    all: &[Node],
    tally: &mut Tally,
) {
    for node in all {
        let (mark, member, given) = match node.kind {
            Kind::Object => (UNKNOWN, "zz".to_owned(), Value::from(3)),
            Kind::Array => {
                let Some(last) = places::lookup(&base.value, &node.path)
                    .and_then(Value::as_array)
                    .and_then(|list| list.as_slice().last().map(|last| (list.len(), last)))
                else {
                    continue;
                };
                // An element that names itself is one more element under another
                // name: two elements of one name are ONE element to a family that keys
                // the list by name (a geometry's bones), and the copy would be probed
                // as that merge, not as one more element.
                let mut more = last.1.clone();
                if let Some(name) = more
                    .as_object()
                    .and_then(|element| element.get("name"))
                    .and_then(Value::as_string)
                    .map(|name| format!("{}_more", name.as_str()))
                    && let Some(element) = more.as_object_mut()
                {
                    element.insert("name", name);
                }
                (EXTRA, last.0.to_string(), more)
            }
            _ => continue,
        };
        let mut path = node.path.clone();
        path.push(member);
        let Some(text) = plain(&set_at(&base.value, &path, given)) else {
            continue;
        };
        let added = Legacy {
            version: legacy.version.clone(),
            from: format!("{} {mark} `{}`", legacy.from, path.join("/")),
            text,
            only: Some(path),
        };
        let (start, texts) = (std::time::Instant::now(), tally.texts);
        add(family, name, read, &base.read, &added, tally);
        tally.seconds[2] += start.elapsed().as_secs_f64();
        tally.added_texts += tally.texts - texts;
    }
}

/// Probes one document with one reader; returns whether the reader loads and writes it.
fn probe_with(
    family: &Family,
    name: &str,
    read: &dyn Fn(&str) -> Outcome,
    legacy: &Legacy,
    tally: &mut Tally,
) -> bool {
    let all = nodes(&legacy.text);
    let Some(value) = parse(&legacy.text) else {
        return false;
    };
    // A generated input whose one member is probed already is not loaded.
    if let Some(only) = &legacy.only {
        let known = all
            .iter()
            .find(|node| node.path == *only)
            .is_none_or(|node| {
                tally
                    .places
                    .contains(&place_key(family, name, legacy, &value, node))
            });
        if known {
            return false;
        }
    }
    let base = match read_unchanged(family, name, read, legacy, tally) {
        Ok(base) => base,
        Err(probed) => return probed,
    };
    note_unstable(family, name, read, legacy, &base, tally);
    let base = base_of(family.name, &legacy.text, value, &all, base, read);
    let mut counted = false;
    for node in &all {
        if node.path.is_empty() {
            continue;
        }
        if legacy
            .only
            .as_ref()
            .is_some_and(|only| !node.path.starts_with(only))
        {
            continue;
        }
        if !tally
            .places
            .insert(place_key(family, name, legacy, &base.value, node))
        {
            continue;
        }
        if !counted {
            counted = true;
            tally.documents += 1;
            tally.beside += usize::from(legacy.from.contains("beside its successor"));
            for (mark, counter) in [
                (SEEDED, &mut tally.seeded),
                (UNKNOWN, &mut tally.unknown),
                (EXTRA, &mut tally.extra),
            ] {
                *counter += usize::from(legacy.from.contains(mark));
            }
            *tally.versions.entry(legacy.version.clone()).or_default() += 1;
        }
        probe_node(family, name, read, legacy, &base, node, tally);
    }
    if legacy.only.is_none() {
        probe_made_members(family, name, read, legacy, &base, &all, tally);
    }
    true
}

/// The marks of the generated inputs in a document's origin.
const SEEDED: &str = "with a member a step adds, already there:";
const UNKNOWN: &str = "with an unknown member";
const EXTRA: &str = "with one more element";

/// Probes a document made of another by ADDING one member or element (`added.only`), at
/// that member. A document the reader refuses is "refused"; one it loads and does not write
/// must have a finding the document it was made of does not have.
fn add(
    family: &Family,
    name: &str,
    read: &dyn Fn(&str) -> Outcome,
    known: &Read,
    added: &Legacy,
    tally: &mut Tally,
) {
    if probe_with(family, name, read, added, tally) {
        return;
    }
    // Not probed: known already, refused, or loaded and not written.
    let Some(value) = parse(&added.text) else {
        return;
    };
    let all = nodes(&added.text);
    let Some(node) = all
        .iter()
        .find(|node| Some(&node.path) == added.only.as_ref())
    else {
        return;
    };
    if !tally
        .places
        .insert(place_key(family, name, added, &value, node))
    {
        return;
    }
    tally.texts += 1;
    match read_text(read, &added.text) {
        Err(None) => *tally.outcomes.entry("refused").or_default() += 1,
        Err(Some(outcome)) => {
            let found: Vec<String> = outcome.findings.iter().map(finding_text).collect();
            if fresh(&known.findings, &found).is_empty() {
                *tally.by_version.entry(added.version.clone()).or_default() += 1;
                tally.faults.push(format!(
                    "{} | {} | {} | (added) | not-written | {} | in {} ({name})",
                    family.name,
                    added.version,
                    node.shown(),
                    outcome.written.err().unwrap_or_default(),
                    added.from
                ));
            } else {
                *tally.outcomes.entry("said").or_default() += 1;
            }
        }
        Ok(_) => {}
    }
}

/// The members of `after` that `before` does not have, each with its path below `at` and
/// its value: what a step (or the whole chain) ADDS to a document. Objects are entered, and
/// lists of the same length; a `null` member is none.
fn added_members(
    before: &Value,
    after: &Value,
    at: &mut Vec<String>,
    out: &mut Vec<(Vec<String>, Value)>,
) {
    if let (Some(before), Some(after)) = (before.as_object(), after.as_object()) {
        for (key, member) in after {
            at.push(key.as_str().to_owned());
            match before.get(key.as_str()) {
                Some(had) => added_members(had, member, at, out),
                None if !member.is_null() => out.push((at.clone(), member.clone())),
                None => {}
            }
            at.pop();
        }
    } else if let (Some(before), Some(after)) = (before.as_array(), after.as_array())
        && before.len() == after.len()
    {
        for (index, (had, member)) in before.as_slice().iter().zip(after.as_slice()).enumerate() {
            at.push(index.to_string());
            added_members(had, member, at, out);
            at.pop();
        }
    }
}

/// The most members one document is seeded with.
const SEEDS: usize = 24;

/// The inputs made from what the family's steps ADD to `legacy`: for every member that the
/// document's own step, run alone, has in its output and the document does not have (and
/// every member the whole chain's written text has and the document does not), the document
/// with that member already there, holding another value of the same type, and one of
/// another type. Each is probed at that member: a step that writes over what the author
/// wrote there, or drops it, must say so.
fn seeded(family: &Family, legacy: &Legacy, out: &mut Vec<Legacy>) {
    let Some(document) = parse(&legacy.text) else {
        return;
    };
    let version = version_of(&legacy.version);
    let mut added: Vec<(Vec<String>, Value)> = Vec::new();
    // The document's own step, run alone on its payload: below one of the family's keys, or
    // the document itself.
    let own = family
        .runs
        .iter()
        .filter(|step| version.is_none_or(|version| step.from <= version))
        .max_by_key(|step| step.from)
        .filter(|_| version.is_some() || family.runs.len() == 1);
    if let Some(step) = own {
        let mut payloads: Vec<(Vec<String>, &Value)> = family
            .keys
            .iter()
            .filter_map(|key| {
                let payload = document.as_object()?.get(key)?;
                payload
                    .is_object()
                    .then(|| (vec![(*key).to_owned()], payload))
            })
            .collect();
        payloads.push((Vec::new(), &document));
        for (mut at, payload) in payloads {
            if let Some(output) = (step.run)(payload, version) {
                added_members(payload, &output, &mut at, &mut added);
                break;
            }
        }
    }
    // The whole chain: what the family writes for the document.
    for (_, read) in &family.readers {
        if let Ok((written, _)) = read_text(read.as_ref(), &legacy.text) {
            added_members(&document, &written.value, &mut Vec::new(), &mut added);
        }
    }
    added.retain(|(path, _)| path != &["format_version"]);
    let mut seen = BTreeSet::new();
    added.retain(|(path, _)| seen.insert(path.clone()));
    for (path, value) in added.into_iter().take(SEEDS) {
        let given = [different(&value), Some(of_another_type(&value))];
        for given in given.into_iter().flatten() {
            let Some(text) = plain(&set_at(&document, &path, given)) else {
                continue;
            };
            out.push(Legacy {
                version: legacy.version.clone(),
                from: format!("{} {SEEDED} `{}`", legacy.from, path.join("/")),
                text,
                only: Some(path.clone()),
            });
        }
    }
}

/// The versions the steps of a family start from, as they are and as the files spell them.
fn starting_versions(family: &Family) -> (Vec<Triple>, Vec<String>) {
    let mut starts: Vec<Triple> = Vec::new();
    for run in &family.runs {
        if !starts.contains(&run.from) {
            starts.push(run.from);
        }
    }
    let versions = starts
        .iter()
        .map(|(major, minor, patch)| format!("{major}.{minor}.{patch}"))
        .collect();
    (starts, versions)
}

/// The path of the member `only` of a source in `text`, as the text has it.
fn member_in_text(family: &Family, text: &str, only: &[String]) -> Option<Vec<String>> {
    nodes(text).into_iter().map(|node| node.path).find(|path| {
        *path == only
            || path
                .split_first()
                .is_some_and(|(key, rest)| rest == only && family.keys.contains(&key.as_str()))
    })
}

/// Prints what a family's probe counted.
fn report(family: &Family, tally: &Tally) {
    let outcomes: Vec<String> = tally
        .outcomes
        .iter()
        .map(|(name, count)| format!("{name} {count}"))
        .collect();
    let per_version: Vec<String> = tally
        .versions
        .iter()
        .map(|(version, count)| {
            format!(
                "{version}: {count} ({} faults)",
                tally.by_version.get(version).copied().unwrap_or(0)
            )
        })
        .collect();
    println!(
        "legacy: {} | documents {} | versions {} | places {} | texts {} | {} | left out by the cap {} | faults {}",
        family.name,
        tally.documents,
        tally.versions.len(),
        tally.places.len(),
        tally.texts,
        outcomes.join(", "),
        tally.capped,
        tally.faults.len()
    );
    println!(
        "legacy-versions: {} | {}",
        family.name,
        per_version.join(", ")
    );
    println!(
        "legacy-inputs: {} | probed with a legacy member beside its successor: {} | with a member a step adds, already there: {} | with an unknown member: {} | with one more element: {}",
        family.name, tally.beside, tally.seeded, tally.unknown, tally.extra
    );
    println!(
        "legacy-seconds: {} | the documents, with an unknown member and one more element: {:.0} | with a member a step adds: {:.0} | with an unknown member and one more element alone: {:.0} ({} texts)",
        family.name, tally.seconds[0], tally.seconds[1], tally.seconds[2], tally.added_texts
    );
    for fault in tally.faults.iter().take(2000) {
        println!("legacy-at: {fault}");
    }
}

/// The probe of one family.
fn audit(family: &Family) {
    let mut sources = Vec::new();
    for (version, text) in more(family.name) {
        let value = parse(text).unwrap_or_else(|| panic!("{}: no JSON: {text}", family.name));
        sources.push(Source {
            version: (*version).to_owned(),
            from: "more".to_owned(),
            texts: vec![plain(&value).expect("plain JSON")],
            only: None,
        });
    }
    test_documents(family, &mut sources);
    // The newest version of the chain: a document at or above it is no legacy document.
    let newest = family.runs.iter().map(|run| run.to).max();
    // A family every step of which starts from a real version reads no file without one.
    let versioned = family.runs.iter().all(|run| run.from >= (1, 1, 0));
    let (starts, versions) = starting_versions(family);
    let mut tally = Tally::default();
    for Source {
        version,
        from,
        texts,
        only,
    } in sources
    {
        if versioned && version == "none" {
            continue;
        }
        let legacy_version = match (version_of(&version), newest) {
            (Some(version), Some(newest)) => version < newest || starts.contains(&version),
            _ => true,
        };
        if !legacy_version {
            continue;
        }
        // The first candidate a reader loads and writes.
        let Some(text) = texts.into_iter().find(|text| {
            family
                .readers
                .iter()
                .any(|(_, read)| read_text(read.as_ref(), text).is_ok())
        }) else {
            continue;
        };
        if family.band.is_some_and(|(from, below)| {
            version_of(&version).is_none_or(|version| version < from || version >= below)
        }) {
            continue;
        }
        // The member of the payload a source is probed at, as the text has it: below one
        // of the family's keys, or at the root.
        let only = only.and_then(|only| member_in_text(family, &text, &only));
        let legacy = Legacy {
            version,
            from,
            text,
            only,
        };
        let start = std::time::Instant::now();
        probe(family, &legacy, &mut tally);
        tally.seconds[0] += start.elapsed().as_secs_f64();
        let start = std::time::Instant::now();
        let mut inputs = Vec::new();
        seeded(family, &legacy, &mut inputs);
        for input in &inputs {
            probe(family, input, &mut tally);
        }
        tally.seconds[1] += start.elapsed().as_secs_f64();
    }
    report(family, &tally);
    let missing: Vec<&String> = versions
        .iter()
        .filter(|version| !tally.versions.contains_key(*version))
        .filter(|version| {
            family.band.is_none_or(|(from, below)| {
                version_of(version).is_some_and(|version| version >= from && version < below)
            })
        })
        .filter(|version| {
            !NO_TEXT
                .iter()
                .any(|(of, at, _)| *of == family.name && at == *version)
        })
        .collect();
    assert!(
        missing.is_empty(),
        "{}: no document of the starting versions {missing:?} loads and is written",
        family.name
    );
    let shown: Vec<&str> = tally.faults.iter().take(6).map(String::as_str).collect();
    assert!(
        tally.faults.is_empty(),
        "{}: {} of {} texts are at fault (the `legacy-at:` lines); the first:\n{}",
        family.name,
        tally.faults.len(),
        tally.texts,
        shown.join("\n")
    );
}
