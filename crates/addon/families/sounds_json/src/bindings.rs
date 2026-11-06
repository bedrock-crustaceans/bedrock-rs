//! `sounds.json`: which sound each game event plays, for the world, for blocks and for
//! entities.

use indexmap::IndexMap;

use super::events::{LEGACY_AMBIENT_EVENTS, is_level_sound_event};
use bedrock_addon_core::diagnostic::{DiagnosticCode, Diagnostics, Severity, VanillaText};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::jsoncpp::fork::{self, Stored};
use bedrock_addon_core::json::kit::sound::read::{integer, string};
use bedrock_addon_core::json::kit::sound::report::{Reporter, check_unknown};
use bedrock_addon_core::molang::{MolangSource, RawVersion};

/// Vanilla's text for a table without its `events` object.
const EXPECTED_EVENTS: &str = "Expected \"events\" object";
/// Vanilla's text for `individual_named_sounds` without its `sounds` object.
const EXPECTED_SOUNDS: &str = "Expected \"sounds\" object";
/// Vanilla's text for a variant map without a key.
const EXPECTED_KEY: &str = "Expected \"key\" expression for entity with sound variants";
/// Vanilla's text for a variant key without a map.
const EXPECTED_MAP: &str = "Expected \"map\" list for entity with sound variant key";
/// Vanilla's text for a block sound whose `base` names no block sound.
const UNKNOWN_BASE: &str = "Unknown block base sound '{}'";
/// Vanilla's text for a block sound whose `base` chain returns to a sound it passed.
const RECURSIVE_BASE: &str = "Block base sound '{}' is recursively used";

/// The event bindings of the resource packs, `sounds.json` (`#[doc(alias)]` `sounds.json`):
/// which sound definition each level sound event plays.
///
/// [`json`](Self::json) is the document as merged, which is what the value writes back.
#[doc(alias = "sounds.json")]
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct SoundBindings {
    /// `individual_event_sounds`: the sounds of events that belong to no block or entity.
    pub individual_event_sounds: EventSounds,
    /// `individual_named_sounds`: sounds played by name (biome ambience).
    pub individual_named_sounds: NamedSounds,
    /// `block_sounds`: the sounds of each block sound type (`stone`, `wood`), as the type
    /// writes them. An event a type lacks is its `base` chain's:
    /// [`block_sound`](Self::block_sound) answers with the chain.
    pub block_sounds: IndexMap<String, BlockSounds>,
    /// `entity_sounds`.
    pub entity_sounds: EntitySounds,
    /// `interactive_sounds`: what an entity or a block sounds like when something acts on it.
    pub interactive_sounds: InteractiveSounds,
    /// The document.
    pub(crate) json: Value,
}

impl SoundBindings {
    /// The document as JSON: what the typed members were read from and what the value writes.
    pub fn json(&self) -> &Value {
        &self.json
    }

    /// The sound the block sound type `block` plays for the level sound event `event` (its
    /// lower-cased name): the type's own, else the nearest one along its `base` chain.
    ///
    /// How far each type's chain reaches is worked out once, when the document is read, and
    /// kept beside the type. A caller that afterwards edits a `base`, or adds or removes a
    /// type of [`block_sounds`](Self::block_sounds), calls
    /// [`resolve_block_bases`](Self::resolve_block_bases) before it asks again: until then
    /// the answer follows the chains as they were read.
    pub fn block_sound(&self, block: &str, event: &str) -> Option<&SoundRef> {
        inherited(&self.block_sounds, block, event)
    }

    /// [`block_sound`](Self::block_sound) in `interactive_sounds.block_sounds`, with the same
    /// rule for a table edited after the load.
    pub fn interactive_block_sound(&self, block: &str, event: &str) -> Option<&SoundRef> {
        inherited(&self.interactive_sounds.block_sounds, block, event)
    }

    /// Works out again how far the `base` chain of every block sound type reaches, in
    /// [`block_sounds`](Self::block_sounds) and in `interactive_sounds.block_sounds`: what a
    /// caller runs after it edited a `base` or the set of types, so that
    /// [`block_sound`](Self::block_sound) and
    /// [`interactive_block_sound`](Self::interactive_block_sound) answer for the tables as
    /// they are now. A chain ends, as at load, at a base that names no type or at the first
    /// type it passed already; nothing is reported and [`json`](Self::json) is not touched.
    pub fn resolve_block_bases(&mut self) {
        for table in [
            &mut self.block_sounds,
            &mut self.interactive_sounds.block_sounds,
        ] {
            let resolved = resolve_bases(table);
            for (entry, (inherits, _)) in table.values_mut().zip(resolved) {
                entry.inherits = inherits;
            }
        }
    }

    /// The value of a document, read as it is: vanilla's per-pack transform does not run and
    /// the findings of the read are dropped. `Content::parse` reads a file with both.
    #[allow(
        clippy::needless_pass_by_value,
        reason = "the signature of the other two documents' `from_json`"
    )]
    pub fn from_json(json: Value) -> Self {
        read_root(&json, true, &mut Diagnostics::default())
    }
}

/// A volume or pitch range: one of the two bounds is picked each time a sound plays.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SoundRange {
    /// The lower bound.
    pub min: f32,
    /// The upper bound.
    pub max: f32,
}

impl SoundRange {
    /// The range of a table that sets none: −1.0 for both bounds, which the engine reads as
    /// its own default.
    pub const UNSET: Self = Self {
        min: -1.0,
        max: -1.0,
    };
    /// The range an interactive table starts from: 1.0 for both bounds.
    pub const ONE: Self = Self { min: 1.0, max: 1.0 };

    /// Whether this is [`UNSET`](Self::UNSET).
    pub fn is_unset(self) -> bool {
        self == Self::UNSET
    }
}

/// One bound sound: a sound definition's event name with its volume and pitch.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct SoundRef {
    /// The sound event name in `sound_definitions.json`; `""` plays nothing.
    pub sound: String,
    /// `volume`, or the table's when the sound sets none.
    pub volume: SoundRange,
    /// `pitch`, or the table's when the sound sets none.
    pub pitch: SoundRange,
}

/// A table of level sound events (`individual_event_sounds`, an entity, a variant).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct EventSounds {
    /// `volume`: what the table's sounds take when they set none.
    pub volume: SoundRange,
    /// `pitch`: what the table's sounds take when they set none.
    pub pitch: SoundRange,
    /// `events`: the sound of each level sound event, by its lower-cased name.
    pub events: IndexMap<String, SoundRef>,
}

impl Default for EventSounds {
    fn default() -> Self {
        Self {
            volume: SoundRange::UNSET,
            pitch: SoundRange::UNSET,
            events: IndexMap::new(),
        }
    }
}

/// `individual_named_sounds`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct NamedSounds {
    /// `volume`.
    pub volume: SoundRange,
    /// `pitch`.
    pub pitch: SoundRange,
    /// `sounds`: the sound of each name, compared with case. The sixteen legacy ambient
    /// events of `individual_event_sounds` are also listed here under their event names
    /// unless a sound of that name exists.
    pub sounds: IndexMap<String, SoundRef>,
}

impl Default for NamedSounds {
    fn default() -> Self {
        Self {
            volume: SoundRange::UNSET,
            pitch: SoundRange::UNSET,
            sounds: IndexMap::new(),
        }
    }
}

/// One block sound type.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct BlockSounds {
    /// `base`: the block sound type whose events this one takes where it has none; `""` for
    /// none. After an edit of it, [`SoundBindings::resolve_block_bases`] makes the lookups
    /// follow the new chain.
    pub base: String,
    /// The type's own table. The events of its `base` chain are not copied in: a table of
    /// many types that inherit from each other would hold every event many times over.
    pub sounds: EventSounds,
    /// How many types of the `base` chain the type inherits from: the chain up to a base that
    /// names no type, or to the first type it passed already.
    ///
    /// A cache: it is worked out for the whole table when the document is read, and not
    /// again when a caller edits [`base`](Self::base) or the table's types.
    /// [`SoundBindings::resolve_block_bases`] works it out again; until then the lookups
    /// follow the chain this far and no further.
    pub(crate) inherits: usize,
}

/// `entity_sounds`.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct EntitySounds {
    /// `defaults`: the sounds of an entity without an entry.
    pub defaults: EventSounds,
    /// `entities`: the sounds of each entity type.
    pub entities: IndexMap<String, EntitySoundEntry>,
}

/// One entity's sounds.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct EntitySoundEntry {
    /// The entity's own table.
    pub sounds: EventSounds,
    /// `variants`: other tables, picked by a Molang key.
    pub variants: Option<SoundVariants<EventSounds>>,
    /// `__molang_version`: the Molang version the variant key is parsed with. Vanilla writes
    /// the supplying pack's into every `entity_sounds` entry with a variant key before the
    /// packs are merged.
    pub molang_version: Option<i32>,
}

/// The sound variants of an entity: a Molang key that names one entry of the map.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct SoundVariants<T> {
    /// `key`: the Molang expression whose string result names the variant; `None` for the
    /// `null` a merge can leave here. Compiled for its findings only, never evaluated.
    pub key: Option<MolangSource>,
    /// `map`: the table of each variant.
    pub map: IndexMap<String, T>,
}

/// `interactive_sounds`.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct InteractiveSounds {
    /// `block_sounds`, read as the top-level `block_sounds` are.
    pub block_sounds: IndexMap<String, BlockSounds>,
    /// `entity_sounds`.
    pub entity_sounds: InteractiveEntitySounds,
}

/// `interactive_sounds.entity_sounds`.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct InteractiveEntitySounds {
    /// `defaults`.
    pub defaults: InteractiveEventSounds,
    /// `entities`.
    pub entities: IndexMap<String, InteractiveEntityEntry>,
}

/// A table of events whose sound depends on what is acted on.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct InteractiveEventSounds {
    /// `volume`; 1.0 when the table sets none.
    pub volume: SoundRange,
    /// `pitch`; 1.0 when the table sets none.
    pub pitch: SoundRange,
    /// `events`: per level sound event, the sound of each surface (`default`, or a block
    /// sound type).
    pub events: IndexMap<String, IndexMap<String, SoundRef>>,
}

impl Default for InteractiveEventSounds {
    fn default() -> Self {
        Self {
            volume: SoundRange::ONE,
            pitch: SoundRange::ONE,
            events: IndexMap::new(),
        }
    }
}

/// One entity of `interactive_sounds.entity_sounds.entities`.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct InteractiveEntityEntry {
    /// The entity's own table.
    pub sounds: InteractiveEventSounds,
    /// `variants`.
    pub variants: Option<SoundVariants<InteractiveEventSounds>>,
    /// `__molang_version` as the author wrote it; the variant key is parsed with version 0
    /// without one (vanilla never writes it into this table).
    pub molang_version: Option<i32>,
}

/// Sets `range` from `value` and says whether it did. `null` and an
/// empty array set nothing.
fn parse_range(value: &Value, range: &mut SoundRange, report: &mut Reporter<'_>) -> bool {
    match fork::stored(value) {
        Stored::Null => false,
        Stored::Integer | Stored::Real | Stored::Bool => {
            if value.is_bool() {
                // A bool is numeric in the game's jsoncpp variant; the crate reports it as an
                // information.
                report.here(DiagnosticCode::InvalidValue { text: None }, Severity::Info);
            }
            let bound = fork::as_float(value, 0.0);
            *range = SoundRange {
                min: bound,
                max: bound,
            };
            true
        }
        Stored::Array => {
            let elements = fork::elements(value);
            match elements.as_slice() {
                [] => false,
                [only] => {
                    let bound = fork::as_float(only, 0.0);
                    *range = SoundRange {
                        min: bound,
                        max: bound,
                    };
                    true
                }
                [first, second, ..] => {
                    let (mut min, mut max) =
                        (fork::as_float(first, 0.0), fork::as_float(second, 0.0));
                    if max < min {
                        std::mem::swap(&mut min, &mut max);
                    }
                    *range = SoundRange { min, max };
                    true
                }
            }
        }
        Stored::Object => {
            if fork::has_member(value, "min") {
                range.min = fork::as_float(fork::member(value, "min"), 0.0);
            }
            if fork::has_member(value, "max") {
                range.max = fork::as_float(fork::member(value, "max"), 0.0);
            }
            // The maximum is never below the minimum.
            range.max = range.min.max(range.max);
            true
        }
        Stored::String => parse_range_text(fork::text(value).unwrap_or_default(), range),
    }
}

/// A range given as a string: the numbers between `[` and `]`, one for both
/// bounds or two in ascending order.
fn parse_range_text(text: &str, range: &mut SoundRange) -> bool {
    let inner = text.trim().trim_start_matches('[').trim_end_matches(']');
    let mut numbers = inner.split(',').map(|part| part.trim().parse::<f32>());
    match (numbers.next(), numbers.next()) {
        (Some(Ok(only)), None) => {
            *range = SoundRange {
                min: only,
                max: only,
            };
            true
        }
        (Some(Ok(a)), Some(Ok(b))) => {
            *range = SoundRange {
                min: a.min(b),
                max: a.max(b),
            };
            true
        }
        _ => false,
    }
}

/// The `volume` and `pitch` of a table, over what it starts from.
fn table_ranges(
    value: &Value,
    start: SoundRange,
    report: &mut Reporter<'_>,
) -> (SoundRange, SoundRange) {
    let (mut volume, mut pitch) = (start, start);
    report.enter("volume");
    parse_range(fork::member(value, "volume"), &mut volume, report);
    report.leave();
    report.enter("pitch");
    parse_range(fork::member(value, "pitch"), &mut pitch, report);
    report.leave();
    (volume, pitch)
}

/// A sound: a name, or an object with `sound`, `volume` and `pitch`; the
/// ranges it does not set are the table's.
fn read_sound(
    value: &Value,
    volume: SoundRange,
    pitch: SoundRange,
    report: &mut Reporter<'_>,
) -> SoundRef {
    if value.is_object() {
        // `sounds` beside `sound` occurs in 88 entries of the game's own file (the `fall` events
        // of `interactive_sounds.block_sounds`); nothing reads it.
        check_unknown(value, &["sound", "volume", "pitch"], &["sounds"], report);
        let sound = string(value, "sound", report);
        let (volume, pitch) = {
            let (mut v, mut p) = (volume, pitch);
            report.enter("volume");
            parse_range(fork::member(value, "volume"), &mut v, report);
            report.leave();
            report.enter("pitch");
            parse_range(fork::member(value, "pitch"), &mut p, report);
            report.leave();
            (v, p)
        };
        return SoundRef {
            sound,
            volume,
            pitch,
        };
    }
    if !matches!(fork::stored(value), Stored::Null | Stored::String) {
        // A number or array reads as `""` and a bool as its spelling; the game says nothing, so
        // the crate reports it as an information.
        report.here(DiagnosticCode::InvalidValue { text: None }, Severity::Info);
    }
    SoundRef {
        sound: fork::to_text(value),
        volume,
        pitch,
    }
}

/// Vanilla's text for an event name it does not know.
const NOT_AN_EVENT: &str = "Event name '{}' is not a valid LevelSoundEvent";

/// The finding of an event name vanilla does not know; the name is the
/// lower-cased key. In an `events` table the game writes vanilla's text to the content log at
/// the error level, drops the entry and loads the file: a kept error; `interactive_sounds`
/// looks the name up without a word, so the finding there is an information of the crate's
/// own.
fn unknown_event(name: &str, logged: bool, report: &mut Reporter<'_>) {
    if logged {
        report.kept(DiagnosticCode::UnknownSound {
            text: Some(VanillaText::Formatted {
                template: NOT_AN_EVENT,
                args: vec![name.to_owned()],
            }),
        });
    } else {
        report.here(DiagnosticCode::UnknownSound { text: None }, Severity::Info);
    }
}

/// Reads an event table: nothing for a value that is no object; else the table's ranges and its
/// `events`, which must be an object unless `events_optional`.
fn read_event_sounds(
    value: &Value,
    events_optional: bool,
    known: &[&str],
    report: &mut Reporter<'_>,
) -> EventSounds {
    let mut table = EventSounds::default();
    if !value.is_object() {
        return table;
    }
    check_unknown(value, known, &[], report);
    (table.volume, table.pitch) = table_ranges(value, SoundRange::UNSET, report);
    let events = fork::member(value, "events");
    if !events.is_object() {
        if !events_optional {
            // The game writes an error line to the content log; the table is skipped and the
            // file loads: a kept error.
            report.kept(DiagnosticCode::MissingRequiredField {
                text: Some(VanillaText::Fixed(EXPECTED_EVENTS)),
            });
        }
        return table;
    }
    report.enter("events");
    // Sorted key order: of two keys equal without case, the later one stands.
    for key in fork::member_names(events) {
        report.enter(&key);
        let event = key.to_lowercase();
        // `undefined` is the enum's `Undefined` value, which this lookup refuses too.
        if is_level_sound_event(&event) && event != "undefined" {
            let sound = read_sound(
                fork::member(events, &key),
                table.volume,
                table.pitch,
                report,
            );
            table.events.insert(event, sound);
        } else {
            unknown_event(&event, true, report);
        }
        report.leave();
    }
    report.leave();
    table
}

/// Reads the named sounds and the legacy ambient fallback after them.
fn read_named_sounds(
    value: &Value,
    events: &EventSounds,
    report: &mut Reporter<'_>,
) -> NamedSounds {
    let mut named = NamedSounds::default();
    if value.is_object() {
        check_unknown(value, &["volume", "pitch", "sounds"], &[], report);
        (named.volume, named.pitch) = table_ranges(value, SoundRange::UNSET, report);
        let sounds = fork::member(value, "sounds");
        if sounds.is_object() {
            report.enter("sounds");
            for key in fork::member_names(sounds) {
                report.enter(&key);
                let sound = read_sound(
                    fork::member(sounds, &key),
                    named.volume,
                    named.pitch,
                    report,
                );
                named.sounds.insert(key.clone(), sound);
                report.leave();
            }
            report.leave();
        } else {
            // The game writes an error line to the content log and the file loads: a kept
            // error.
            report.kept(DiagnosticCode::MissingRequiredField {
                text: Some(VanillaText::Fixed(EXPECTED_SOUNDS)),
            });
        }
    }
    for event in LEGACY_AMBIENT_EVENTS {
        if let Some(sound) = events.events.get(*event) {
            named
                .sounds
                .entry((*event).to_owned())
                .or_insert_with(|| sound.clone());
        }
    }
    named
}

/// The sound of `event` for the type `block` of `table`: its own, else the nearest along the
/// part of its `base` chain it inherits from.
fn inherited<'a>(
    table: &'a IndexMap<String, BlockSounds>,
    block: &str,
    event: &str,
) -> Option<&'a SoundRef> {
    let mut entry = table.get(block)?;
    let mut left = entry.inherits;
    loop {
        if let Some(sound) = entry.sounds.events.get(event) {
            return Some(sound);
        }
        if left == 0 {
            return None;
        }
        left -= 1;
        entry = table.get(&entry.base)?;
    }
}

/// Where the `base` chain of a block sound type ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChainEnd {
    /// At a type without a base.
    Clean,
    /// At a base that names no type: the type that names it.
    Missing(usize),
    /// At a type the chain passed already: that type.
    Cycle(usize),
}

/// For every type of `blocks`, by index: how many types of its `base` chain it inherits from
/// and where the chain ends. Each type is resolved once: the chains are walked with an
/// explicit stack, a type on the stack that is reached again closes a cycle, and a type
/// resolved earlier ends the walk with its own answer.
// The game walks each entry's chain with a visited set; the result per entry is the same.
fn resolve_bases(blocks: &IndexMap<String, BlockSounds>) -> Vec<(usize, ChainEnd)> {
    #[derive(Clone, Copy, PartialEq)]
    enum Mark {
        New,
        OnStack(usize),
        Done,
    }
    let base_of = |index: usize| -> Option<Option<usize>> {
        let base = blocks[index].base.as_str();
        (!base.is_empty()).then(|| blocks.get_index_of(base))
    };
    let mut marks = vec![Mark::New; blocks.len()];
    let mut resolved = vec![(0, ChainEnd::Clean); blocks.len()];
    let mut stack: Vec<usize> = Vec::new();
    for start in 0..blocks.len() {
        if marks[start] != Mark::New {
            continue;
        }
        let mut at = start;
        // What the walk's last type inherits and where its chain ends.
        let mut tail = loop {
            marks[at] = Mark::OnStack(stack.len());
            stack.push(at);
            match base_of(at) {
                None => break (0, ChainEnd::Clean),
                Some(None) => break (0, ChainEnd::Missing(at)),
                Some(Some(base)) => match marks[base] {
                    Mark::New => at = base,
                    Mark::Done => {
                        let (inherits, end) = resolved[base];
                        break (inherits + 1, end);
                    }
                    Mark::OnStack(position) => {
                        // The types from `position` up are a cycle: each inherits from the
                        // others and ends at itself.
                        let members = stack.split_off(position);
                        for member in &members {
                            resolved[*member] = (members.len() - 1, ChainEnd::Cycle(*member));
                            marks[*member] = Mark::Done;
                        }
                        // A type below them enters the cycle at `base`.
                        break (members.len(), ChainEnd::Cycle(base));
                    }
                },
            }
        };
        while let Some(index) = stack.pop() {
            resolved[index] = tail;
            marks[index] = Mark::Done;
            tail = (tail.0 + 1, tail.1);
        }
    }
    resolved
}

/// The name of the type at `index` of `blocks`.
fn name_at(blocks: &IndexMap<String, BlockSounds>, index: usize) -> String {
    blocks
        .get_index(index)
        .map_or_else(String::new, |(name, _)| name.clone())
}

/// Reads the block sounds and resolves their bases: every member of the section is a block
/// sound type; a type with a `base` need not have `events`, and takes the events of its base
/// chain that it does not define itself (answered on lookup, [`inherited`]).
///
/// With `whole`, the section is the merged one and a `base` that names no type is vanilla's
/// error; a single pack's copy may name a type of a lower pack, so there it is no finding.
fn read_block_sounds(
    value: &Value,
    whole: bool,
    report: &mut Reporter<'_>,
) -> IndexMap<String, BlockSounds> {
    let mut blocks: IndexMap<String, BlockSounds> = IndexMap::new();
    for name in fork::member_names(value) {
        report.enter(&name);
        let entry = fork::member(value, &name);
        let base = fork::to_text(fork::member(entry, "base"));
        let sounds = read_event_sounds(
            entry,
            !base.is_empty(),
            &["base", "volume", "pitch", "events"],
            report,
        );
        blocks.insert(
            name.clone(),
            BlockSounds {
                base,
                sounds,
                inherits: 0,
            },
        );
        report.leave();
    }
    let resolved = resolve_bases(&blocks);
    for (index, (inherits, end)) in resolved.into_iter().enumerate() {
        blocks[index].inherits = inherits;
        let code = match end {
            ChainEnd::Clean => continue,
            ChainEnd::Missing(_) if !whole => continue,
            ChainEnd::Missing(last) => DiagnosticCode::UnresolvedReference {
                text: Some(VanillaText::Formatted {
                    template: UNKNOWN_BASE,
                    args: vec![blocks[last].base.clone()],
                }),
            },
            ChainEnd::Cycle(entry) => DiagnosticCode::ReferenceCycle {
                text: Some(VanillaText::Formatted {
                    template: RECURSIVE_BASE,
                    args: vec![name_at(&blocks, entry)],
                }),
            },
        };
        let block = name_at(&blocks, index);
        // The game writes an error line to the content log and the file loads: a kept error.
        report.enter(&block);
        report.enter("base");
        report.kept(code);
        report.leave();
        report.leave();
    }
    blocks
}

/// The `variants` of an entity entry: the map first, each variant read by `read` with fresh
/// table defaults, then the key. `None` for an entry without a
/// `variants` member.
fn read_variants<T>(
    entry: &Value,
    read: impl Fn(&Value, &mut Reporter<'_>) -> T,
    report: &mut Reporter<'_>,
) -> Option<SoundVariants<T>> {
    if !fork::has_member(entry, "variants") {
        return None;
    }
    let variants = fork::member(entry, "variants");
    report.enter("variants");
    check_unknown(variants, &["key", "map"], &[], report);
    let mut map = IndexMap::new();
    let listed = fork::member(variants, "map");
    report.enter("map");
    if listed.is_object() {
        for name in fork::member_names(listed) {
            report.enter(&name);
            map.insert(name.clone(), read(fork::member(listed, &name), report));
            report.leave();
        }
    }
    report.leave();
    let key = fork::member(variants, "key");
    let key = if key.is_null() {
        if !map.is_empty() {
            // The game writes an error line to the content log: a kept error.
            report.kept(DiagnosticCode::MissingRequiredField {
                text: Some(VanillaText::Fixed(EXPECTED_KEY)),
            });
        }
        None
    } else {
        if map.is_empty() {
            // The game writes an error line to the content log: a kept error.
            report.kept(DiagnosticCode::MissingRequiredField {
                text: Some(VanillaText::Fixed(EXPECTED_MAP)),
            });
        }
        // The entry's `__molang_version` as an integer defaulting to 0, narrowed to a 16-bit
        // integer as the game narrows it.
        #[allow(
            clippy::cast_possible_truncation,
            reason = "vanilla's MolangVersion is a short"
        )]
        let version = fork::as_int(fork::member(entry, "__molang_version"), 0) as i16;
        Some(MolangSource::string(
            fork::to_text(key),
            RawVersion(version),
        ))
    };
    report.leave();
    Some(SoundVariants { key, map })
}

/// The `__molang_version` of an entity entry, when it has the member.
fn molang_version(entry: &Value, report: &mut Reporter<'_>) -> Option<i32> {
    fork::has_member(entry, "__molang_version")
        .then(|| integer(entry, "__molang_version", 0, report))
}

/// The members an event table of an entity may have.
const ENTITY_MEMBERS: &[&str] = &["volume", "pitch", "events", "variants", "__molang_version"];
/// The members a plain event table may have.
const TABLE_MEMBERS: &[&str] = &["volume", "pitch", "events"];

/// Reads the entity sounds.
fn read_entity_sounds(value: &Value, report: &mut Reporter<'_>) -> EntitySounds {
    let mut out = EntitySounds::default();
    if !value.is_object() {
        return out;
    }
    check_unknown(value, &["defaults", "entities"], &[], report);
    report.enter("defaults");
    out.defaults = read_event_sounds(
        fork::member(value, "defaults"),
        false,
        TABLE_MEMBERS,
        report,
    );
    report.leave();
    let entities = fork::member(value, "entities");
    report.enter("entities");
    for name in fork::member_names(entities) {
        report.enter(&name);
        let entry = fork::member(entities, &name);
        out.entities.insert(
            name.clone(),
            EntitySoundEntry {
                sounds: read_event_sounds(entry, false, ENTITY_MEMBERS, report),
                variants: read_variants(
                    entry,
                    |variant, report| read_event_sounds(variant, false, TABLE_MEMBERS, report),
                    report,
                ),
                molang_version: molang_version(entry, report),
            },
        );
        report.leave();
    }
    report.leave();
    out
}

/// Reads an interactive event table: nothing for a value that is no object; `events` is optional;
/// an inner key `""` is skipped.
fn read_interactive_events(
    value: &Value,
    known: &[&str],
    report: &mut Reporter<'_>,
) -> InteractiveEventSounds {
    let mut table = InteractiveEventSounds::default();
    if !value.is_object() {
        return table;
    }
    check_unknown(value, known, &[], report);
    (table.volume, table.pitch) = table_ranges(value, SoundRange::ONE, report);
    let events = fork::member(value, "events");
    report.enter("events");
    for key in fork::member_names(events) {
        report.enter(&key);
        let event = key.to_lowercase();
        if is_level_sound_event(&event) {
            let surfaces = fork::member(events, &key);
            let mut sounds = IndexMap::new();
            for surface in fork::member_names(surfaces) {
                if surface.is_empty() {
                    continue;
                }
                report.enter(&surface);
                let sound = read_sound(
                    fork::member(surfaces, &surface),
                    table.volume,
                    table.pitch,
                    report,
                );
                sounds.insert(surface.clone(), sound);
                report.leave();
            }
            table.events.insert(event, sounds);
        } else {
            // The game drops it silently here, with no content-log line; the crate reports it.
            unknown_event(&event, false, report);
        }
        report.leave();
    }
    report.leave();
    table
}

/// Reads `interactive_sounds.entity_sounds`.
fn read_interactive_entities(value: &Value, report: &mut Reporter<'_>) -> InteractiveEntitySounds {
    let mut out = InteractiveEntitySounds::default();
    if !value.is_object() {
        return out;
    }
    check_unknown(value, &["defaults", "entities"], &[], report);
    report.enter("defaults");
    out.defaults = read_interactive_events(fork::member(value, "defaults"), TABLE_MEMBERS, report);
    report.leave();
    let entities = fork::member(value, "entities");
    report.enter("entities");
    for name in fork::member_names(entities) {
        report.enter(&name);
        let entry = fork::member(entities, &name);
        let variants = read_variants(
            entry,
            |variant, report| read_interactive_events(variant, TABLE_MEMBERS, report),
            report,
        );
        let version = molang_version(entry, report);
        if version.is_none() && variants.as_ref().is_some_and(|v| v.key.is_some()) {
            // the key is parsed with Molang version 0.
            report.at(
                "variants",
                DiagnosticCode::InvalidValue { text: None },
                Severity::Info,
            );
        }
        out.entities.insert(
            name.clone(),
            InteractiveEntityEntry {
                sounds: read_interactive_events(entry, ENTITY_MEMBERS, report),
                variants,
                molang_version: version,
            },
        );
        report.leave();
    }
    report.leave();
    out
}

/// `_loadSounds`: the five sections of the document, in vanilla's order. `whole` says the
/// document is the merge of the whole stack, not one pack's copy.
pub(crate) fn read_root(root: &Value, whole: bool, diagnostics: &mut Diagnostics) -> SoundBindings {
    let mut report = Reporter::new(diagnostics);
    check_unknown(
        root,
        &[
            "individual_event_sounds",
            "individual_named_sounds",
            "block_sounds",
            "entity_sounds",
            "interactive_sounds",
        ],
        &["format_version"],
        &mut report,
    );
    report.enter("individual_event_sounds");
    let individual_event_sounds = read_event_sounds(
        fork::member(root, "individual_event_sounds"),
        false,
        TABLE_MEMBERS,
        &mut report,
    );
    report.leave();
    report.enter("individual_named_sounds");
    let individual_named_sounds = read_named_sounds(
        fork::member(root, "individual_named_sounds"),
        &individual_event_sounds,
        &mut report,
    );
    report.leave();
    report.enter("block_sounds");
    let block_sounds = read_block_sounds(fork::member(root, "block_sounds"), whole, &mut report);
    report.leave();
    report.enter("entity_sounds");
    let entity_sounds = read_entity_sounds(fork::member(root, "entity_sounds"), &mut report);
    report.leave();
    report.enter("interactive_sounds");
    let interactive = fork::member(root, "interactive_sounds");
    // `entities` is the member the game's pre-merge transform writes and nothing reads.
    check_unknown(
        interactive,
        &["block_sounds", "entity_sounds", "entities"],
        &[],
        &mut report,
    );
    report.enter("block_sounds");
    let interactive_blocks = read_block_sounds(
        fork::member(interactive, "block_sounds"),
        whole,
        &mut report,
    );
    report.leave();
    report.enter("entity_sounds");
    let interactive_entities =
        read_interactive_entities(fork::member(interactive, "entity_sounds"), &mut report);
    report.leave();
    report.leave();
    SoundBindings {
        individual_event_sounds,
        individual_named_sounds,
        block_sounds,
        entity_sounds,
        interactive_sounds: InteractiveSounds {
            block_sounds: interactive_blocks,
            entity_sounds: interactive_entities,
        },
        json: root.clone(),
    }
}

/// The pre-merge transform of one pack's copy, for each of `entity_sounds` and `interactive_sounds`
/// that the copy has: its `entities` is made to exist (`null` when absent), every entity with
/// `variants` gets a `variants.key` (`null` when absent), and one with a key that is not `null`
/// gets the pack's Molang version as `__molang_version`, over whatever the author wrote. The second
/// table is one the loader never reads.
///
/// A copy without the section is left without it: twelve of the game's own slices ship no
/// `entity_sounds`, and a `null` inserted for them would clear the entity table of the game's
/// own stack, which the game visibly keeps.
pub(crate) fn pre_merge_transform(root: &mut Value, molang_version: i16) {
    for section in ["entity_sounds", "interactive_sounds"] {
        if !fork::has_member(root, section) {
            continue;
        }
        let entities = fork::index_mut(fork::index_mut(root, section), "entities");
        let Some(entities) = entities.as_object_mut() else {
            continue;
        };
        for entity in entities.values_mut() {
            if !fork::has_member(entity, "variants") {
                continue;
            }
            let key = fork::index_mut(fork::index_mut(entity, "variants"), "key");
            if !key.is_null() {
                *fork::index_mut(entity, "__molang_version") =
                    Value::from(i64::from(molang_version));
            }
        }
    }
}
