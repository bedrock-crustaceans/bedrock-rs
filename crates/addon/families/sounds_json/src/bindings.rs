//! `sounds.json`: which sound each game event plays, for the world, for blocks and for
//! entities.

#![allow(
    clippy::unnecessary_wraps,
    reason = "the shape of a `schema::read_with` function, which never fails here"
)]

use facet::Facet;
use indexmap::IndexMap;

use super::codes;
use super::events::{LEGACY_AMBIENT_EVENTS, is_level_sound_event};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Phase, Severity};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::jsoncpp::fork::{self, Stored};
use bedrock_addon_core::json::kit::{MemberOrder, Node, Objects, Profile, Read};
use bedrock_addon_core::molang::{MolangSource, RawVersion};
use bedrock_addon_core::schema::{self, FromJson, Schema, read_document};
use bedrock_addon_core::versioning::{NullHandling, Reading, Scalars};

/// What a read of the whole stack's merge puts `true` in scope under: a block sound `base` that
/// names no type is then vanilla's error.
const WHOLE: &str = "whole";

/// The event bindings of the resource packs, `sounds.json` (`#[doc(alias)]` `sounds.json`):
/// which sound definition each level sound event plays.
///
/// [`json`](Self::json) is the document as merged, which is what the value writes back.
#[doc(alias = "sounds.json")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
// `format_version` is in vanilla's own files; nothing reads it.
#[facet(schema::quiet = "format_version", schema::after_read = legacy_ambient)]
#[non_exhaustive]
pub struct SoundBindings {
    /// `individual_event_sounds`: the sounds of events that belong to no block or entity.
    #[facet(default)]
    pub individual_event_sounds: EventSounds,
    /// `individual_named_sounds`: sounds played by name (biome ambience).
    #[facet(default)]
    pub individual_named_sounds: NamedSounds,
    /// `block_sounds`: the sounds of each block sound type (`stone`, `wood`), as the type
    /// writes them. An event a type lacks is its `base` chain's:
    /// [`block_sound`](Self::block_sound) answers with the chain.
    #[facet(default, schema::read_with = block_table)]
    pub block_sounds: IndexMap<String, BlockSounds>,
    /// `entity_sounds`.
    #[facet(default)]
    pub entity_sounds: EntitySounds,
    /// `interactive_sounds`: what an entity or a block sounds like when something acts on it.
    #[facet(default)]
    pub interactive_sounds: InteractiveSounds,
    /// The document.
    #[facet(skip)]
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
#[derive(Clone, Copy, Debug, PartialEq, Facet)]
pub struct SoundRange {
    /// The lower bound.
    pub min: f32,
    /// The upper bound.
    pub max: f32,
}

bedrock_addon_core::impl_schema_leaf!(SoundRange);
bedrock_addon_core::impl_no_molang!(SoundRange);

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
///
/// Read from a name, or from an object with `sound`, `volume` and `pitch`; a range it does not
/// set is the one its table puts in scope.
#[derive(Clone, Debug, PartialEq, Facet)]
#[non_exhaustive]
pub struct SoundRef {
    /// The sound event name in `sound_definitions.json`; `""` plays nothing.
    pub sound: String,
    /// `volume`, or the table's when the sound sets none.
    pub volume: SoundRange,
    /// `pitch`, or the table's when the sound sets none.
    pub pitch: SoundRange,
}

bedrock_addon_core::impl_schema_leaf!(SoundRef);
bedrock_addon_core::impl_no_molang!(SoundRef);

impl FromJson for SoundRef {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let (volume, pitch) = (table_range(read, "volume"), table_range(read, "pitch"));
        if !node.value().is_object() {
            // A number or array reads as `""` and a bool as its spelling (`asString`); the game
            // says nothing, so the crate tells it as an information.
            let sound = String::from_json(node, read)?;
            return Some(Self {
                sound,
                volume,
                pitch,
            });
        }
        let written = SoundObject::from_json(node, read)?;
        Some(Self {
            sound: written.sound,
            volume: written.volume.unwrap_or(volume),
            pitch: written.pitch.unwrap_or(pitch),
        })
    }
}

/// A sound written as an object.
#[derive(Debug, PartialEq, Facet, Schema)]
// `sounds` beside `sound` occurs in 88 entries of the game's own file (the `fall` events of
// `interactive_sounds.block_sounds`); nothing reads it.
#[facet(schema::quiet = "sounds")]
struct SoundObject {
    #[facet(default)]
    sound: String,
    #[facet(schema::read_with = own_volume)]
    volume: Option<SoundRange>,
    #[facet(schema::read_with = own_pitch)]
    pitch: Option<SoundRange>,
}

/// A table of level sound events (`individual_event_sounds`, an entity, a variant). Its
/// `volume` and `pitch` are in scope for its sounds.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::after_read = events_object)]
#[non_exhaustive]
pub struct EventSounds {
    /// `volume`: what the table's sounds take when they set none.
    #[facet(
        default = SoundRange::UNSET,
        schema::read_with = range_over_unset,
        schema::scope
    )]
    pub volume: SoundRange,
    /// `pitch`: what the table's sounds take when they set none.
    #[facet(
        default = SoundRange::UNSET,
        schema::read_with = range_over_unset,
        schema::scope
    )]
    pub pitch: SoundRange,
    /// `events`: the sound of each level sound event, by its lower-cased name.
    #[facet(default, schema::read_with = events)]
    pub events: IndexMap<String, SoundRef>,
}

/// `individual_named_sounds`. Its `volume` and `pitch` are in scope for its sounds.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::after_read = sounds_object)]
#[non_exhaustive]
pub struct NamedSounds {
    /// `volume`.
    #[facet(
        default = SoundRange::UNSET,
        schema::read_with = range_over_unset,
        schema::scope
    )]
    pub volume: SoundRange,
    /// `pitch`.
    #[facet(
        default = SoundRange::UNSET,
        schema::read_with = range_over_unset,
        schema::scope
    )]
    pub pitch: SoundRange,
    /// `sounds`: the sound of each name, compared with case. The sixteen legacy ambient
    /// events of `individual_event_sounds` are also listed here under their event names
    /// unless a sound of that name exists.
    #[facet(default)]
    pub sounds: IndexMap<String, SoundRef>,
}

/// One block sound type. Its `base` is in scope for its table, whose `events` it makes
/// optional.
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
#[non_exhaustive]
pub struct BlockSounds {
    /// `base`: the block sound type whose events this one takes where it has none; `""` for
    /// none. After an edit of it, [`SoundBindings::resolve_block_bases`] makes the lookups
    /// follow the new chain.
    #[facet(default, schema::read_with = silent_text, schema::scope)]
    pub base: String,
    /// The type's own table. The events of its `base` chain are not copied in: a table of
    /// many types that inherit from each other would hold every event many times over.
    #[facet(flatten)]
    pub sounds: EventSounds,
    /// How many types of the `base` chain the type inherits from: the chain up to a base that
    /// names no type, or to the first type it passed already.
    ///
    /// A cache: it is worked out for the whole table when the document is read, and not
    /// again when a caller edits [`base`](Self::base) or the table's types.
    /// [`SoundBindings::resolve_block_bases`] works it out again; until then the lookups
    /// follow the chain this far and no further.
    #[facet(skip)]
    pub(crate) inherits: usize,
}

/// `entity_sounds`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[non_exhaustive]
pub struct EntitySounds {
    /// `defaults`: the sounds of an entity without an entry.
    #[facet(default)]
    pub defaults: EventSounds,
    /// `entities`: the sounds of each entity type.
    #[facet(default)]
    pub entities: IndexMap<String, EntitySoundEntry>,
}

/// One entity's sounds.
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
#[facet(schema::after_read = entity_key)]
#[non_exhaustive]
pub struct EntitySoundEntry {
    /// The entity's own table.
    #[facet(flatten)]
    pub sounds: EventSounds,
    /// `variants`: other tables, picked by a Molang key.
    #[facet(schema::reads_null)]
    pub variants: Option<SoundVariants<EventSounds>>,
    /// `__molang_version`: the Molang version the variant key is parsed with. Vanilla writes
    /// the supplying pack's into every `entity_sounds` entry with a variant key before the
    /// packs are merged.
    #[facet(rename = "__molang_version", schema::reads_null)]
    pub molang_version: Option<i32>,
}

/// The sound variants of an entity: a Molang key that names one entry of the map. Each
/// variant's table starts from fresh ranges.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::after_read = key_and_map)]
#[non_exhaustive]
pub struct SoundVariants<T> {
    /// `key`: the Molang expression whose string result names the variant; `None` for the
    /// `null` a merge can leave here. Compiled for its findings only, never evaluated.
    #[facet(schema::read_with = key_source)]
    pub key: Option<MolangSource>,
    /// `map`: the table of each variant.
    #[facet(default)]
    pub map: IndexMap<String, T>,
}

/// `interactive_sounds`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
// `entities` is the member the game's pre-merge transform writes and nothing reads.
#[facet(schema::quiet = "entities")]
#[non_exhaustive]
pub struct InteractiveSounds {
    /// `block_sounds`, read as the top-level `block_sounds` are.
    #[facet(default, schema::read_with = block_table)]
    pub block_sounds: IndexMap<String, BlockSounds>,
    /// `entity_sounds`.
    #[facet(default)]
    pub entity_sounds: InteractiveEntitySounds,
}

/// `interactive_sounds.entity_sounds`.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[non_exhaustive]
pub struct InteractiveEntitySounds {
    /// `defaults`.
    #[facet(default)]
    pub defaults: InteractiveEventSounds,
    /// `entities`.
    #[facet(default)]
    pub entities: IndexMap<String, InteractiveEntityEntry>,
}

/// A table of events whose sound depends on what is acted on. Its `volume` and `pitch` are in
/// scope for its sounds.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[non_exhaustive]
pub struct InteractiveEventSounds {
    /// `volume`; 1.0 when the table sets none.
    #[facet(default = SoundRange::ONE, schema::read_with = range_over_one, schema::scope)]
    pub volume: SoundRange,
    /// `pitch`; 1.0 when the table sets none.
    #[facet(default = SoundRange::ONE, schema::read_with = range_over_one, schema::scope)]
    pub pitch: SoundRange,
    /// `events`: per level sound event, the sound of each surface (`default`, or a block
    /// sound type).
    #[facet(default, schema::read_with = interactive_events)]
    pub events: IndexMap<String, IndexMap<String, SoundRef>>,
}

/// One entity of `interactive_sounds.entity_sounds.entities`: the game reads its variants
/// before its own table.
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
#[facet(schema::after_read = interactive_entity_key)]
#[non_exhaustive]
pub struct InteractiveEntityEntry {
    /// `variants`.
    #[facet(schema::reads_null)]
    pub variants: Option<SoundVariants<InteractiveEventSounds>>,
    /// `__molang_version` as the author wrote it; the variant key is parsed with version 0
    /// without one (vanilla never writes it into this table).
    #[facet(rename = "__molang_version", schema::reads_null)]
    pub molang_version: Option<i32>,
    /// The entity's own table.
    #[facet(flatten)]
    pub sounds: InteractiveEventSounds,
}

/// `_loadSounds`: the five sections of `root`, in vanilla's order, read as the game's jsoncpp
/// fork reads them: members asked for by name, an object iterated in key order, a value that
/// is no object one with no members, each scalar coerced (`Scalars::Fork`) and a `null` the
/// default, every finding in the validate phase and an unknown member an information. `whole`
/// says the document is the merge of the whole stack, not one pack's copy.
pub(crate) fn read_root(root: &Value, whole: bool, diagnostics: &mut Diagnostics) -> SoundBindings {
    let reading = Reading::OWN
        .with_scalars(Scalars::Fork)
        .with_null(NullHandling::AbsentSilently);
    let profile = Profile::DECLARED_ORDER
        .with_members(MemberOrder::Named)
        .with_objects(Objects::Fork);
    let mut read = Read::new(reading, diagnostics)
        .profile(profile)
        .phase(Phase::Validate)
        .unknown_severity(Severity::Info);
    let node = Node::new(root, JsonPath::new());
    let mut value: SoundBindings = read
        .scoped(WHOLE, whole, |read| read_document(&node, read))
        .unwrap_or_default();
    value.json = root.clone();
    value
}

/// The range `name` of the table the value is read in ([`Read::scope`]); unset outside one.
fn table_range(read: &Read<'_>, name: &str) -> SoundRange {
    read.scope::<SoundRange>(name)
        .copied()
        .unwrap_or(SoundRange::UNSET)
}

/// A sound's own `volume`, over its table's.
fn own_volume(node: &Node<'_>, read: &mut Read<'_>) -> Option<SoundRange> {
    let start = table_range(read, "volume");
    Some(parse_range(node, start, read))
}

/// A sound's own `pitch`, over its table's.
fn own_pitch(node: &Node<'_>, read: &mut Read<'_>) -> Option<SoundRange> {
    let start = table_range(read, "pitch");
    Some(parse_range(node, start, read))
}

/// A table's range, over the unset one.
fn range_over_unset(node: &Node<'_>, read: &mut Read<'_>) -> Option<SoundRange> {
    Some(parse_range(node, SoundRange::UNSET, read))
}

/// An interactive table's range, over 1.0.
fn range_over_one(node: &Node<'_>, read: &mut Read<'_>) -> Option<SoundRange> {
    Some(parse_range(node, SoundRange::ONE, read))
}

/// A range over `start`: `null` and an empty array set nothing; a number (a bool too, numeric
/// in the game's jsoncpp variant, which the crate tells as an information) both bounds; an
/// array its first two elements in ascending order; an object its `min` and `max` over
/// `start`, the maximum never below the minimum; a string the numbers between `[` and `]`.
fn parse_range(node: &Node<'_>, start: SoundRange, read: &mut Read<'_>) -> SoundRange {
    let value = node.value();
    let both = |bound: f32| SoundRange {
        min: bound,
        max: bound,
    };
    match fork::stored(value) {
        Stored::Null => start,
        Stored::Integer | Stored::Real | Stored::Bool => {
            if value.is_bool() {
                read.push(Diagnostic::info(&core_codes::INVALID_VALUE).at(node.path().clone()));
            }
            both(fork::as_float(value, 0.0))
        }
        Stored::Array => match fork::elements(value).as_slice() {
            [] => start,
            [only] => both(fork::as_float(only, 0.0)),
            [first, second, ..] => {
                let (a, b) = (fork::as_float(first, 0.0), fork::as_float(second, 0.0));
                if b < a {
                    SoundRange { min: b, max: a }
                } else {
                    SoundRange { min: a, max: b }
                }
            }
        },
        Stored::Object => {
            let mut range = start;
            if fork::has_member(value, "min") {
                range.min = fork::as_float(fork::member(value, "min"), 0.0);
            }
            if fork::has_member(value, "max") {
                range.max = fork::as_float(fork::member(value, "max"), 0.0);
            }
            range.max = range.min.max(range.max);
            range
        }
        Stored::String => parse_range_text(fork::text(value).unwrap_or_default()).unwrap_or(start),
    }
}

/// A range given as a string: the numbers between `[` and `]`, one for both bounds or two in
/// ascending order.
fn parse_range_text(text: &str) -> Option<SoundRange> {
    let inner = text.trim().trim_start_matches('[').trim_end_matches(']');
    let mut numbers = inner.split(',').map(|part| part.trim().parse::<f32>());
    match (numbers.next(), numbers.next()) {
        (Some(Ok(only)), None) => Some(SoundRange {
            min: only,
            max: only,
        }),
        (Some(Ok(a)), Some(Ok(b))) => Some(SoundRange {
            min: a.min(b),
            max: a.max(b),
        }),
        _ => None,
    }
}

/// A string read by `asString` without a word (a `base`).
fn silent_text(node: &Node<'_>, _: &mut Read<'_>) -> Option<String> {
    Some(fork::to_text(node.value()))
}

/// A variant key, read by `asString` without a word, at Molang version 0 until its entity
/// says its own ([`stamp`]).
fn key_source(node: &Node<'_>, _: &mut Read<'_>) -> Option<MolangSource> {
    Some(MolangSource::string(
        fork::to_text(node.value()),
        RawVersion(0),
    ))
}

/// An `events` table, by the lower-cased name of each event in key order (of two keys equal
/// without case the later stands). A name vanilla does not know (`undefined`, the enum's
/// `Undefined`, too) is dropped with vanilla's text, which the game writes to the content log
/// at the error level before it loads the file: a kept error.
fn events(node: &Node<'_>, read: &mut Read<'_>) -> Option<IndexMap<String, SoundRef>> {
    let members = read.object(node)?;
    let mut table = IndexMap::new();
    for (key, member) in read.ordered_entries(&members) {
        let event = key.to_lowercase();
        if is_level_sound_event(&event) && event != "undefined" {
            table.insert(event, SoundRef::from_json(&member, read)?);
        } else {
            let code = codes::NOT_AN_EVENT.with([event]);
            read.push(Diagnostic::of(code).at(member.path().clone()));
        }
    }
    Some(table)
}

/// An interactive `events` table: per event, the sound of each surface but `""`. The game drops
/// a name it does not know without a word here; the crate tells it as an information.
fn interactive_events(
    node: &Node<'_>,
    read: &mut Read<'_>,
) -> Option<IndexMap<String, IndexMap<String, SoundRef>>> {
    let members = read.object(node)?;
    let mut table = IndexMap::new();
    for (key, member) in read.ordered_entries(&members) {
        let event = key.to_lowercase();
        if !is_level_sound_event(&event) {
            read.push(Diagnostic::info(&core_codes::UNKNOWN_SOUND).at(member.path().clone()));
            continue;
        }
        let surfaces = read.object(&member)?;
        let mut sounds = IndexMap::new();
        for (surface, sound) in read.ordered_entries(&surfaces) {
            if !surface.is_empty() {
                sounds.insert(surface.to_owned(), SoundRef::from_json(&sound, read)?);
            }
        }
        table.insert(event, sounds);
    }
    Some(table)
}

/// A table without its `events` object, but for a block sound type's with a `base`: the game
/// writes an error line to the content log, skips the table and loads the file, a kept error.
fn events_object(_: &mut EventSounds, node: &Node<'_>, read: &mut Read<'_>) {
    let based = read
        .scope::<String>("base")
        .is_some_and(|base| !base.is_empty());
    if !based && node.value().is_object() && !fork::member(node.value(), "events").is_object() {
        read.push(Diagnostic::of(&codes::EXPECTED_EVENTS).at(node.path().clone()));
    }
}

/// `individual_named_sounds` without its `sounds` object: a content-log error, the file loads.
fn sounds_object(_: &mut NamedSounds, node: &Node<'_>, read: &mut Read<'_>) {
    if node.value().is_object() && !fork::member(node.value(), "sounds").is_object() {
        read.push(Diagnostic::of(&codes::EXPECTED_SOUNDS).at(node.path().clone()));
    }
}

/// A variant map without a key, or a key without a map: each a content-log error.
fn key_and_map<T>(variants: &mut SoundVariants<T>, node: &Node<'_>, read: &mut Read<'_>) {
    let code = match (&variants.key, variants.map.is_empty()) {
        (None, false) => &codes::EXPECTED_KEY,
        (Some(_), true) => &codes::EXPECTED_MAP,
        _ => return,
    };
    read.push(Diagnostic::of(code).at(node.path().clone()));
}

/// The variant key parsed with the entry's `__molang_version` (0 without one), narrowed to a
/// 16-bit integer as the game narrows it.
fn stamp<T>(variants: Option<&mut SoundVariants<T>>, version: Option<i32>) {
    if let Some(key) = variants.and_then(|variants| variants.key.as_mut()) {
        #[allow(
            clippy::cast_possible_truncation,
            reason = "vanilla's MolangVersion is a short"
        )]
        let version = RawVersion(version.unwrap_or(0) as i16);
        *key = MolangSource::string(key.as_str().to_owned(), version);
    }
}

/// An entity's variant key, at the entry's Molang version.
fn entity_key(entry: &mut EntitySoundEntry, _: &Node<'_>, _: &mut Read<'_>) {
    stamp(entry.variants.as_mut(), entry.molang_version);
}

/// An interactive entity's variant key, at the entry's Molang version; without one it is
/// parsed with version 0, which the crate tells as an information at `variants`.
fn interactive_entity_key(
    entry: &mut InteractiveEntityEntry,
    node: &Node<'_>,
    read: &mut Read<'_>,
) {
    stamp(entry.variants.as_mut(), entry.molang_version);
    let keyed = entry.variants.as_ref().is_some_and(|v| v.key.is_some());
    if entry.molang_version.is_none() && keyed {
        let at = node.path().child("variants");
        read.push(Diagnostic::info(&core_codes::INVALID_VALUE).at(at));
    }
}

/// The legacy ambient fallback: the sixteen legacy ambient events of `individual_event_sounds`
/// are named sounds too, unless a sound of that name exists.
fn legacy_ambient(bindings: &mut SoundBindings, _: &Node<'_>, _: &mut Read<'_>) {
    let events = &bindings.individual_event_sounds.events;
    for event in LEGACY_AMBIENT_EVENTS {
        if let Some(sound) = events.get(*event) {
            bindings
                .individual_named_sounds
                .sounds
                .entry((*event).to_owned())
                .or_insert_with(|| sound.clone());
        }
    }
}

/// A `block_sounds` section with its bases resolved: every member is a block sound type, which
/// takes the events of its base chain that it does not define itself (answered on lookup,
/// [`inherited`]). In the stack's merge ([`WHOLE`]) a `base` that names no type is vanilla's
/// error; a single pack's copy may name a type of a lower pack, so there it is no finding. A
/// chain that returns to a type it passed is vanilla's error in both. Each is a content-log
/// error after which the file loads: a kept error.
fn block_table(node: &Node<'_>, read: &mut Read<'_>) -> Option<IndexMap<String, BlockSounds>> {
    let mut blocks = IndexMap::<String, BlockSounds>::from_json(node, read)?;
    let whole = read.scope::<bool>(WHOLE).copied().unwrap_or(true);
    for (index, (inherits, end)) in resolve_bases(&blocks).into_iter().enumerate() {
        blocks[index].inherits = inherits;
        let code = match end {
            ChainEnd::Clean => continue,
            ChainEnd::Missing(_) if !whole => continue,
            ChainEnd::Missing(last) => codes::UNKNOWN_BASE.with([blocks[last].base.clone()]),
            ChainEnd::Cycle(entry) => codes::RECURSIVE_BASE.with([name_at(&blocks, entry)]),
        };
        let at = node.path().child(&name_at(&blocks, index)).child("base");
        read.push(Diagnostic::of(code).at(at));
    }
    Some(blocks)
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
