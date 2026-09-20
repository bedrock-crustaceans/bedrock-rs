//! Structured decode for the flat, database-wide string-keyed records that
//! [`crate::raw_keys`] exposes only as opaque NBT -- the Phase 8 stretch
//! goal named in `TODO.md`: `mobevents`, `schedulerWT`, `BiomeData`,
//! `AutonomousEntities`, and the three dimension keys
//! (`Overworld`/`Nether`/`TheEnd`). `scoreboard` is included too, per the
//! Phase 8 list entry that names it directly above this one.
//!
//! Every type here layers typed accessors over `RetainedNbt` the way
//! [`crate::map_data::MapRecord`] does: [`decode`] keeps the original bytes
//! alongside the parsed value, and a decoded-but-unmodified record
//! [`encode`]s back to those exact bytes; reaching into `value_mut()` drops
//! the retained bytes so the next encode re-derives from the edited value.
//! Nothing here models a field, a default, or a version gate the corpus
//! scan below did not show.
//!
//! # Corpus scan (14 worlds: 13 imported fixtures + `tests/level.tar.gz`)
//!
//! | Key | Worlds | Typed | Raw (unmodelable from this corpus) |
//! | --- | --- | --- | --- |
//! | `mobevents` | 14/14 | `events_enabled`, the three named event bytes | -- |
//! | `schedulerWT` | 14/14 | `daysSinceLastWTSpawn`, `isSpawningWT`, `nextWTSpawnCheckTick`, `existingWT` | -- |
//! | `TheEnd` | 5/14 | `data.DragonFight.*` | `data.LimboEntities` (always empty) |
//! | `Overworld` | 14/14 | -- | `data.LimboEntities` (always empty) |
//! | `Nether` | 3/14 | -- | `data.LimboEntities` (always empty) |
//! | `scoreboard` | 14/14 | `LastUniqueID` | `Criteria`/`DisplayObjectives`/`Entries`/`Objectives` (always empty) |
//! | `BiomeData` | 14/14 | `list[].id`, `snowAccumulation`, `foliageSnow` | -- |
//! | `AutonomousEntities` | 14/14 | -- | `AutonomousEntityList` (empty in 13/14; one full actor record in `v1_18`) |
//!
//! Every count above is the same scan `raw_keys.rs` cites; nothing here
//! changes what is present where, only how much of each present record gets
//! a typed accessor instead of a raw [`nbtx::Value`] escape hatch.
//!
//! `mobevents`'s `minecraft:ender_dragon_event` is `Option<bool>` because it
//! is absent in exactly one world (`v1_12`); every other named field in this
//! module is present on every record that carries the key. Every observed
//! boolean-as-`Byte` value in `mobevents` is `1` -- the crate infers `0`
//! means "disabled" from the tag being a boolean flag, not from an observed
//! `0` record. Each `mobevents` accessor's doc comment repeats this.
//!
//! `schedulerWT`'s `existingWT` is present in exactly 3 of the 14 worlds
//! (`v1_18`, `v1_18_30`, `v1_19_30`) and absent in worlds both older and
//! newer than those three -- content-dependent (whether a wandering trader
//! was pending when the world was saved), not version-gated, so it is not
//! treated as one.
//!
//! `TheEnd`'s `ExitPortalLocation` is present in exactly the 2 of 5
//! `DragonFight` records that also have `DragonSpawned = 1`. The corpus is 5
//! records, so this is an observed correlation, not a proven rule --
//! [`DragonFight::exit_portal_location`] is `Option` regardless of
//! `dragon_spawned`, and callers should not assume the correlation holds
//! outside this corpus.
//!
//! # `BiomeData.id`: kept as the tag the record was written with
//!
//! `list[].id` is a `Byte` in 13 of the 14 worlds and a `Short` in the one
//! 1.26 seed world -- only two data points (1.20.81 and 1.26) bracket
//! wherever the boundary actually falls, so this is not treated as a
//! version-gated field with a known cutover, just a tag that varies.
//! `nbtx`'s `lenient_width` widens on decode only and always re-emits the
//! tag it was told to decode as, which cannot stay byte-faithful for a
//! record whose declared tag varies by world: a `Byte`-tagged record
//! decoded through a `lenient_width(i16)` field would encode back out as
//! `Short`, changing bytes the corpus never asked to change.
//!
//! [`BiomeId`] is the choice made here: an enum that carries the tag a real
//! record was read with (`Byte(i8)` or `Short(i16)`), so decode-then-encode
//! stays byte-identical without touching the tag at all. This is option (a)
//! from the module's design brief, taken because it stays small -- one
//! two-variant enum plus a `value()` convenience -- and it costs nothing a
//! typed `i16` plus a write-width caveat would have saved, since nothing
//! here ever needs to *choose* a width; it only ever reports the one a real
//! record already has. A caller building a fresh [`BiomeEntry`] from
//! scratch picks a variant explicitly instead of the crate guessing one.
//!
//! # Overworld/Nether/TheEnd share one type
//!
//! [`DimensionRecord`] models all three dimension keys, because two of
//! their three fields overlap completely: `Overworld` and `Nether` carry
//! only `data.LimboEntities` (always empty in the corpus, kept raw per the
//! module docs above [`crate::raw_keys`] already established for it), and
//! `TheEnd` carries that plus `data.DragonFight`. `dragon_fight()` returns
//! `None` on every `Overworld`/`Nether` record and on any `TheEnd` record
//! whose `DragonFight` compound is missing or does not carry every field
//! this corpus showed as always present together.

use std::io::Cursor;

use bedrock_shared::world::dimension::Dimension;

use crate::db::Database;
use crate::error::{Error, Result};
use crate::map_data::{RetainedNbt, bool_byte, byte, field, float, int, list, long};
use crate::raw_keys;

/// Decodes one uncompressed little-endian NBT compound with no trailing
/// bytes -- the shape every record in this module has, matching
/// [`raw_keys::decode`].
fn decode_compound(bytes: &[u8]) -> Result<(nbtx::Value, Vec<u8>)> {
    let mut cursor = Cursor::new(bytes);
    let value: nbtx::Value = nbtx::from_le_bytes(&mut cursor)?;
    if cursor.position() != bytes.len() as u64 {
        return Err(Error::Invalid(
            "flat record has trailing bytes past its NBT compound",
        ));
    }
    Ok((value, bytes.to_vec()))
}

/// Defines one flat-record type's boilerplate: the `RetainedNbt`-backed
/// struct, `new`/`value`/`value_mut`, and `decode_*`/`encode_*` -- the part
/// that is identical for every type in this module, per the module's design
/// (typed accessors layered over `RetainedNbt` the way
/// [`crate::map_data::MapRecord`] does). A second, six-argument form of this
/// macro also generates `read_*`/`write_*` against a fixed on-disk key;
/// callers that need a non-fixed key (see [`DimensionRecord`], keyed per
/// call) use this three-argument form and hand-write their own read/write.
/// Per-field accessors are never generated here -- they differ by type and
/// stay in a plain `impl` block after each invocation.
macro_rules! flat_record {
    (
        $(#[$struct_meta:meta])*
        $name:ident,
        $decode:ident,
        $encode:ident $(,)?
    ) => {
        $(#[$struct_meta])*
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name {
            inner: RetainedNbt,
        }

        impl $name {
            /// Wraps an already-decoded NBT value as a record with no
            /// original on-disk bytes to fall back to.
            pub fn new(value: nbtx::Value) -> Self {
                Self {
                    inner: RetainedNbt::new(value),
                }
            }

            /// The record's full NBT value, verbatim.
            pub fn value(&self) -> &nbtx::Value {
                self.inner.value()
            }

            /// Mutable access to the record's NBT value. Clears the
            /// preserved original bytes -- see `RetainedNbt` -- so the
            /// next encode re-derives from the edited value.
            pub fn value_mut(&mut self) -> &mut nbtx::Value {
                self.inner.value_mut()
            }
        }

        #[doc = concat!("Decodes a [`", stringify!($name), "`] value.")]
        pub fn $decode(bytes: &[u8]) -> Result<$name> {
            let (value, original) = decode_compound(bytes)?;
            Ok($name {
                inner: RetainedNbt::decoded(value, original),
            })
        }

        #[doc = concat!(
            "Encodes a [`", stringify!($name), "`] value. The inverse of [`",
            stringify!($decode), "`]."
        )]
        pub fn $encode(record: &$name) -> Result<Vec<u8>> {
            record.inner.encode()
        }
    };

    (
        $(#[$struct_meta:meta])*
        $name:ident,
        $key:expr,
        $decode:ident,
        $encode:ident,
        $read:ident,
        $write:ident $(,)?
    ) => {
        flat_record!($(#[$struct_meta])* $name, $decode, $encode);

        #[doc = concat!("Reads the [`", stringify!($name), "`] record from `db`, if present.")]
        pub fn $read(db: &Database) -> Result<Option<$name>> {
            match db.get($key.as_bytes())? {
                Some(bytes) => Ok(Some($decode(&bytes)?)),
                None => Ok(None),
            }
        }

        #[doc = concat!(
            "Encodes and writes `record` at the [`", stringify!($name),
            "`] key, overwriting whatever is there."
        )]
        pub fn $write(db: &Database, record: &$name) -> Result<()> {
            db.insert($key.as_bytes(), $encode(record)?)
        }
    };
}

// ---------------------------------------------------------------------
// mobevents
// ---------------------------------------------------------------------

flat_record!(
    /// The decoded `mobevents` record: mob-event spawner toggles. Present in
    /// 14/14 corpus worlds; see the module docs for which fields are always
    /// present and which are `Option`.
    MobEvents,
    raw_keys::MOB_EVENTS,
    decode_mob_events,
    encode_mob_events,
    read_mob_events,
    write_mob_events,
);

impl MobEvents {
    /// `events_enabled`: present on every real record found. The corpus
    /// only ever observed `1` -- `0` (mob events globally off) is inferred
    /// from the field being a boolean flag, not from an observed record.
    pub fn events_enabled(&self) -> Option<bool> {
        bool_byte(self.value(), "events_enabled")
    }

    /// `minecraft:pillager_patrols_event`: present on every real record
    /// found. See [`Self::events_enabled`] for the "only `1` observed" note,
    /// which applies here too.
    pub fn pillager_patrols_event(&self) -> Option<bool> {
        bool_byte(self.value(), "minecraft:pillager_patrols_event")
    }

    /// `minecraft:wandering_trader_event`: present on every real record
    /// found. See [`Self::events_enabled`] for the "only `1` observed" note.
    pub fn wandering_trader_event(&self) -> Option<bool> {
        bool_byte(self.value(), "minecraft:wandering_trader_event")
    }

    /// `minecraft:ender_dragon_event`: present in 13 of the 14 corpus
    /// worlds, absent only in `v1_12`. See [`Self::events_enabled`] for the
    /// "only `1` observed" note, which applies to every record that does
    /// carry this field.
    pub fn ender_dragon_event(&self) -> Option<bool> {
        bool_byte(self.value(), "minecraft:ender_dragon_event")
    }
}

// ---------------------------------------------------------------------
// schedulerWT
// ---------------------------------------------------------------------

flat_record!(
    /// The decoded `schedulerWT` record: wandering-trader spawn-scheduling
    /// state. Present in 14/14 corpus worlds.
    SchedulerWt,
    raw_keys::SCHEDULER_WT,
    decode_scheduler_wt,
    encode_scheduler_wt,
    read_scheduler_wt,
    write_scheduler_wt,
);

impl SchedulerWt {
    /// `daysSinceLastWTSpawn`: present on every real record found, observed
    /// values 0-3.
    pub fn days_since_last_wt_spawn(&self) -> Option<i32> {
        int(self.value(), "daysSinceLastWTSpawn")
    }

    /// `isSpawningWT`: present on every real record found. Every observed
    /// value is `0` -- the corpus never shows a world mid-spawn.
    pub fn is_spawning_wt(&self) -> Option<bool> {
        bool_byte(self.value(), "isSpawningWT")
    }

    /// `nextWTSpawnCheckTick`: present on every real record found.
    pub fn next_wt_spawn_check_tick(&self) -> Option<i64> {
        long(self.value(), "nextWTSpawnCheckTick")
    }

    /// `existingWT`: present in exactly 3 of the 14 corpus worlds (`v1_18`,
    /// `v1_18_30`, `v1_19_30`), including worlds both older and newer than
    /// those three -- content-dependent (a wandering trader pending at save
    /// time), not version-gated, so this is not treated as one.
    pub fn existing_wt(&self) -> Option<i64> {
        long(self.value(), "existingWT")
    }
}

// ---------------------------------------------------------------------
// scoreboard
// ---------------------------------------------------------------------

flat_record!(
    /// The decoded `scoreboard` record. Present in 14/14 corpus worlds, 101
    /// bytes on every one of them: `LastUniqueID` is the only field any real
    /// record populates, so it is the only typed accessor -- `Criteria`,
    /// `DisplayObjectives`, `Entries`, and `Objectives` are empty lists
    /// (element tag `TAG_End`) in every corpus record and stay raw,
    /// unmodelable from this corpus.
    Scoreboard,
    raw_keys::SCOREBOARD,
    decode_scoreboard,
    encode_scoreboard,
    read_scoreboard,
    write_scoreboard,
);

impl Scoreboard {
    /// `LastUniqueID`: the next id the game will assign to a new
    /// objective/entry. Present and populated on every real record found.
    pub fn last_unique_id(&self) -> Option<i64> {
        long(self.value(), "LastUniqueID")
    }

    /// `Criteria`, verbatim -- empty (element tag `TAG_End`) on every real
    /// record found.
    pub fn criteria(&self) -> Option<&[nbtx::Value]> {
        list(self.value(), "Criteria")
    }

    /// `DisplayObjectives`, verbatim -- empty on every real record found.
    pub fn display_objectives(&self) -> Option<&[nbtx::Value]> {
        list(self.value(), "DisplayObjectives")
    }

    /// `Entries`, verbatim -- empty on every real record found.
    pub fn entries(&self) -> Option<&[nbtx::Value]> {
        list(self.value(), "Entries")
    }

    /// `Objectives`, verbatim -- empty on every real record found.
    pub fn objectives(&self) -> Option<&[nbtx::Value]> {
        list(self.value(), "Objectives")
    }
}

// ---------------------------------------------------------------------
// BiomeData
// ---------------------------------------------------------------------

/// `list[].id`'s tag, kept exactly as read -- see the module docs' "kept as
/// the tag the record was written with" section for why this is the chosen
/// shape rather than a single widened `i16`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BiomeId {
    /// Observed in 13 of the 14 corpus worlds.
    Byte(i8),
    /// Observed only in the 1.26 seed world.
    Short(i16),
}

impl BiomeId {
    /// The id widened to `i32`, regardless of which tag it was read as.
    pub fn value(self) -> i32 {
        match self {
            BiomeId::Byte(v) => i32::from(v),
            BiomeId::Short(v) => i32::from(v),
        }
    }
}

/// One `BiomeData.list` entry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BiomeEntry {
    pub id: BiomeId,
    /// Present on every real record found.
    pub snow_accumulation: f32,
    /// Present only in the `v1_18` world, with no discernible trigger --
    /// neither a version boundary nor a content correlation was found for
    /// it in the corpus.
    pub foliage_snow: Option<f32>,
}

impl BiomeEntry {
    fn from_value(value: &nbtx::Value) -> Option<Self> {
        let id = match field(value, "id")? {
            nbtx::Value::Byte(v) => BiomeId::Byte(*v),
            nbtx::Value::Short(v) => BiomeId::Short(*v),
            _ => return None,
        };
        let snow_accumulation = float(value, "snowAccumulation")?;
        let foliage_snow = float(value, "foliageSnow");
        Some(BiomeEntry {
            id,
            snow_accumulation,
            foliage_snow,
        })
    }
}

flat_record!(
    /// The decoded `BiomeData` record. Present in 14/14 corpus worlds.
    BiomeData,
    raw_keys::BIOME_DATA,
    decode_biome_data,
    encode_biome_data,
    read_biome_data,
    write_biome_data,
);

impl BiomeData {
    /// `list`, parsed into [`BiomeEntry`] where every entry's `id` and
    /// `snowAccumulation` are present and correctly typed -- an entry
    /// missing either is skipped rather than failing the whole record, per
    /// architecture decision 11's "degrade on unmodelled". No real record
    /// in the corpus has ever shown a malformed entry.
    pub fn entries(&self) -> Option<Vec<BiomeEntry>> {
        let items = list(self.value(), "list")?;
        Some(items.iter().filter_map(BiomeEntry::from_value).collect())
    }
}

// ---------------------------------------------------------------------
// AutonomousEntities
// ---------------------------------------------------------------------

flat_record!(
    /// The decoded `AutonomousEntities` record. Present in 14/14 corpus
    /// worlds: `AutonomousEntityList` is empty in 13 of the 14, and holds
    /// one full ender-dragon actor record in `v1_18`. The list stays raw
    /// either way -- its populated element is a whole actor payload,
    /// matching the precedent [`crate::actor`] already set for actor NBT.
    AutonomousEntities,
    raw_keys::AUTONOMOUS_ENTITIES,
    decode_autonomous_entities,
    encode_autonomous_entities,
    read_autonomous_entities,
    write_autonomous_entities,
);

impl AutonomousEntities {
    /// `AutonomousEntityList`, verbatim -- see the type docs for why this
    /// stays raw rather than being parsed element by element.
    pub fn entity_list(&self) -> Option<&[nbtx::Value]> {
        list(self.value(), "AutonomousEntityList")
    }
}

// ---------------------------------------------------------------------
// Overworld / Nether / TheEnd
// ---------------------------------------------------------------------

/// A `TheEnd` dimension record's `data.DragonFight` compound. Present on
/// every `TheEnd` record found (5/5); see [`DimensionRecord::dragon_fight`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragonFight {
    pub version: i8,
    pub killed: bool,
    pub spawned: bool,
    pub uuid: i64,
    /// `IsRespawning`.
    pub is_respawning: bool,
    /// `PreviouslyKilled`.
    pub previously_killed: bool,
    /// `ExitPortalLocation`, present in exactly the 2 of 5 corpus records
    /// that also have `spawned == true` -- see the module docs' note on
    /// this being an observed correlation, not a proven rule.
    pub exit_portal_location: Option<[i32; 3]>,
}

impl DragonFight {
    fn from_value(value: &nbtx::Value) -> Option<Self> {
        let version = byte(value, "DragonFightVersion")?;
        let killed = bool_byte(value, "DragonKilled")?;
        let spawned = bool_byte(value, "DragonSpawned")?;
        let uuid = long(value, "DragonUUID")?;
        let is_respawning = bool_byte(value, "IsRespawning")?;
        let previously_killed = bool_byte(value, "PreviouslyKilled")?;
        // `Gateways` (a `List<Int>`, 19-20 entries in the corpus) is
        // required for this compound to count as a `DragonFight` at all,
        // even though its content is not surfaced as a typed field: nothing
        // in the corpus gives a structural reason to interpret individual
        // gateway positions, matching `map_data`'s treatment of
        // `decorations`.
        field(value, "Gateways")?;
        let exit_portal_location = match field(value, "ExitPortalLocation") {
            Some(nbtx::Value::List(items)) if items.len() == 3 => {
                let mut out = [0i32; 3];
                for (slot, item) in out.iter_mut().zip(items.iter()) {
                    match item {
                        nbtx::Value::Int(v) => *slot = *v,
                        _ => return None,
                    }
                }
                Some(out)
            }
            Some(_) => return None,
            None => None,
        };
        Some(DragonFight {
            version,
            killed,
            spawned,
            uuid,
            is_respawning,
            previously_killed,
            exit_portal_location,
        })
    }
}

flat_record!(
    /// The decoded record stored under a dimension key (`Overworld`,
    /// `Nether`, or `TheEnd`) -- see [`raw_keys::dimension_key`] for
    /// building the on-disk key and the module docs for why all three
    /// share one type. `read`/`write` are hand-written rather than
    /// generated: unlike every other type in this module, a
    /// `DimensionRecord`'s on-disk key depends on the `Dimension` passed
    /// in per call rather than being fixed.
    DimensionRecord,
    decode_dimension_record,
    encode_dimension_record,
);

impl DimensionRecord {
    /// `data.LimboEntities`, verbatim -- always an empty list (element tag
    /// `TAG_End`) in every corpus record on all three dimension keys.
    pub fn limbo_entities(&self) -> Option<&[nbtx::Value]> {
        list(field(self.value(), "data")?, "LimboEntities")
    }

    /// `data.DragonFight`, parsed -- present only on `TheEnd` records (5/5
    /// of them); `None` on every `Overworld`/`Nether` record, and on any
    /// `TheEnd` record whose `DragonFight` compound does not carry every
    /// field this corpus showed as always present together.
    pub fn dragon_fight(&self) -> Option<DragonFight> {
        let data = field(self.value(), "data")?;
        DragonFight::from_value(field(data, "DragonFight")?)
    }
}

/// Reads `dimension`'s flat dimension record from `db`, if present. Returns
/// `Ok(None)` both when the record is absent and when `dimension` has no
/// fixed key name (see [`raw_keys::dimension_key_name`]) -- neither case
/// has data to report.
pub fn read_dimension_record(
    db: &Database,
    dimension: Dimension,
) -> Result<Option<DimensionRecord>> {
    let Some(key) = raw_keys::dimension_key(dimension) else {
        return Ok(None);
    };
    match db.get(&key)? {
        Some(bytes) => Ok(Some(decode_dimension_record(&bytes)?)),
        None => Ok(None),
    }
}

/// Encodes and writes `record` at `dimension`'s flat dimension key,
/// overwriting whatever is there. Fails if `dimension` has no fixed key
/// name (see [`raw_keys::dimension_key_name`]) -- there is nowhere to write
/// it.
pub fn write_dimension_record(
    db: &Database,
    dimension: Dimension,
    record: &DimensionRecord,
) -> Result<()> {
    let key = raw_keys::dimension_key(dimension)
        .ok_or(Error::Invalid("dimension has no fixed dimension-key name"))?;
    db.insert(&key, encode_dimension_record(record)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mob_events_sample() -> nbtx::Value {
        let mut compound = nbtx::Compound::new();
        compound.insert("events_enabled".into(), nbtx::Value::Byte(1));
        compound.insert(
            "minecraft:pillager_patrols_event".into(),
            nbtx::Value::Byte(1),
        );
        compound.insert(
            "minecraft:wandering_trader_event".into(),
            nbtx::Value::Byte(1),
        );
        compound.insert("minecraft:ender_dragon_event".into(), nbtx::Value::Byte(1));
        nbtx::Value::Compound(compound)
    }

    #[test]
    fn mob_events_decode_then_encode_round_trips() {
        let bytes = nbtx::to_le_bytes(&mob_events_sample()).unwrap();
        let record = decode_mob_events(&bytes).unwrap();
        assert_eq!(record.events_enabled(), Some(true));
        assert_eq!(record.pillager_patrols_event(), Some(true));
        assert_eq!(record.wandering_trader_event(), Some(true));
        assert_eq!(record.ender_dragon_event(), Some(true));
        assert_eq!(encode_mob_events(&record).unwrap(), bytes);
    }

    #[test]
    fn mob_events_missing_ender_dragon_event_is_none() {
        let mut compound = nbtx::Compound::new();
        compound.insert("events_enabled".into(), nbtx::Value::Byte(1));
        let value = nbtx::Value::Compound(compound);
        let bytes = nbtx::to_le_bytes(&value).unwrap();
        let record = decode_mob_events(&bytes).unwrap();
        assert_eq!(record.ender_dragon_event(), None);
        assert_eq!(record.events_enabled(), Some(true));
    }

    #[test]
    fn mob_events_value_mut_clears_retained_bytes() {
        let bytes = nbtx::to_le_bytes(&mob_events_sample()).unwrap();
        let mut record = decode_mob_events(&bytes).unwrap();
        let nbtx::Value::Compound(compound) = record.value_mut() else {
            panic!("expected a compound");
        };
        compound.insert("events_enabled".into(), nbtx::Value::Byte(0));
        assert_eq!(record.events_enabled(), Some(false));
        assert_ne!(encode_mob_events(&record).unwrap(), bytes);
    }

    fn scheduler_wt_sample(existing_wt: Option<i64>) -> nbtx::Value {
        let mut compound = nbtx::Compound::new();
        compound.insert("daysSinceLastWTSpawn".into(), nbtx::Value::Int(2));
        compound.insert("isSpawningWT".into(), nbtx::Value::Byte(0));
        compound.insert("nextWTSpawnCheckTick".into(), nbtx::Value::Long(123456789));
        if let Some(wt) = existing_wt {
            compound.insert("existingWT".into(), nbtx::Value::Long(wt));
        }
        nbtx::Value::Compound(compound)
    }

    #[test]
    fn scheduler_wt_decode_then_encode_round_trips_without_existing_wt() {
        let bytes = nbtx::to_le_bytes(&scheduler_wt_sample(None)).unwrap();
        let record = decode_scheduler_wt(&bytes).unwrap();
        assert_eq!(record.days_since_last_wt_spawn(), Some(2));
        assert_eq!(record.is_spawning_wt(), Some(false));
        assert_eq!(record.next_wt_spawn_check_tick(), Some(123456789));
        assert_eq!(record.existing_wt(), None);
        assert_eq!(encode_scheduler_wt(&record).unwrap(), bytes);
    }

    #[test]
    fn scheduler_wt_decode_then_encode_round_trips_with_existing_wt() {
        let bytes = nbtx::to_le_bytes(&scheduler_wt_sample(Some(987654321))).unwrap();
        let record = decode_scheduler_wt(&bytes).unwrap();
        assert_eq!(record.existing_wt(), Some(987654321));
        assert_eq!(encode_scheduler_wt(&record).unwrap(), bytes);
    }

    fn scoreboard_sample() -> nbtx::Value {
        let mut compound = nbtx::Compound::new();
        compound.insert("LastUniqueID".into(), nbtx::Value::Long(7));
        compound.insert("Criteria".into(), nbtx::Value::List(vec![]));
        compound.insert("DisplayObjectives".into(), nbtx::Value::List(vec![]));
        compound.insert("Entries".into(), nbtx::Value::List(vec![]));
        compound.insert("Objectives".into(), nbtx::Value::List(vec![]));
        nbtx::Value::Compound(compound)
    }

    #[test]
    fn scoreboard_decode_then_encode_round_trips() {
        let bytes = nbtx::to_le_bytes(&scoreboard_sample()).unwrap();
        let record = decode_scoreboard(&bytes).unwrap();
        assert_eq!(record.last_unique_id(), Some(7));
        assert_eq!(record.criteria(), Some(&[][..]));
        assert_eq!(record.display_objectives(), Some(&[][..]));
        assert_eq!(record.entries(), Some(&[][..]));
        assert_eq!(record.objectives(), Some(&[][..]));
        assert_eq!(encode_scoreboard(&record).unwrap(), bytes);
    }

    fn biome_entry_value(id: nbtx::Value, foliage_snow: Option<f32>) -> nbtx::Value {
        let mut compound = nbtx::Compound::new();
        compound.insert("id".into(), id);
        compound.insert("snowAccumulation".into(), nbtx::Value::Float(0.0));
        if let Some(f) = foliage_snow {
            compound.insert("foliageSnow".into(), nbtx::Value::Float(f));
        }
        nbtx::Value::Compound(compound)
    }

    #[test]
    fn biome_data_decode_then_encode_round_trips_byte_id() {
        let mut root = nbtx::Compound::new();
        root.insert(
            "list".into(),
            nbtx::Value::List(vec![biome_entry_value(nbtx::Value::Byte(1), None)]),
        );
        let bytes = nbtx::to_le_bytes(&nbtx::Value::Compound(root)).unwrap();
        let record = decode_biome_data(&bytes).unwrap();
        let entries = record.entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, BiomeId::Byte(1));
        assert_eq!(entries[0].id.value(), 1);
        assert_eq!(entries[0].foliage_snow, None);
        assert_eq!(encode_biome_data(&record).unwrap(), bytes);
    }

    #[test]
    fn biome_data_decode_then_encode_round_trips_short_id_and_foliage_snow() {
        let mut root = nbtx::Compound::new();
        root.insert(
            "list".into(),
            nbtx::Value::List(vec![biome_entry_value(nbtx::Value::Short(300), Some(0.5))]),
        );
        let bytes = nbtx::to_le_bytes(&nbtx::Value::Compound(root)).unwrap();
        let record = decode_biome_data(&bytes).unwrap();
        let entries = record.entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, BiomeId::Short(300));
        assert_eq!(entries[0].id.value(), 300);
        assert_eq!(entries[0].foliage_snow, Some(0.5));
        assert_eq!(encode_biome_data(&record).unwrap(), bytes);
    }

    #[test]
    fn autonomous_entities_decode_then_encode_round_trips_empty_list() {
        let mut root = nbtx::Compound::new();
        root.insert("AutonomousEntityList".into(), nbtx::Value::List(vec![]));
        let bytes = nbtx::to_le_bytes(&nbtx::Value::Compound(root)).unwrap();
        let record = decode_autonomous_entities(&bytes).unwrap();
        assert_eq!(record.entity_list(), Some(&[][..]));
        assert_eq!(encode_autonomous_entities(&record).unwrap(), bytes);
    }

    fn overworld_sample() -> nbtx::Value {
        let mut data = nbtx::Compound::new();
        data.insert("LimboEntities".into(), nbtx::Value::List(vec![]));
        let mut root = nbtx::Compound::new();
        root.insert("data".into(), nbtx::Value::Compound(data));
        nbtx::Value::Compound(root)
    }

    #[test]
    fn dimension_record_overworld_has_no_dragon_fight() {
        let bytes = nbtx::to_le_bytes(&overworld_sample()).unwrap();
        let record = decode_dimension_record(&bytes).unwrap();
        assert_eq!(record.limbo_entities(), Some(&[][..]));
        assert_eq!(record.dragon_fight(), None);
        assert_eq!(encode_dimension_record(&record).unwrap(), bytes);
    }

    fn the_end_sample(spawned: bool, exit_portal: bool) -> nbtx::Value {
        let mut dragon_fight = nbtx::Compound::new();
        dragon_fight.insert("DragonFightVersion".into(), nbtx::Value::Byte(1));
        dragon_fight.insert("DragonKilled".into(), nbtx::Value::Byte(0));
        dragon_fight.insert(
            "DragonSpawned".into(),
            nbtx::Value::Byte(if spawned { 1 } else { 0 }),
        );
        dragon_fight.insert("DragonUUID".into(), nbtx::Value::Long(-1));
        dragon_fight.insert(
            "Gateways".into(),
            nbtx::Value::List((0..20).map(nbtx::Value::Int).collect()),
        );
        dragon_fight.insert("IsRespawning".into(), nbtx::Value::Byte(0));
        dragon_fight.insert("PreviouslyKilled".into(), nbtx::Value::Byte(0));
        if exit_portal {
            dragon_fight.insert(
                "ExitPortalLocation".into(),
                nbtx::Value::List(vec![
                    nbtx::Value::Int(0),
                    nbtx::Value::Int(64),
                    nbtx::Value::Int(0),
                ]),
            );
        }
        let mut data = nbtx::Compound::new();
        data.insert("DragonFight".into(), nbtx::Value::Compound(dragon_fight));
        data.insert("LimboEntities".into(), nbtx::Value::List(vec![]));
        let mut root = nbtx::Compound::new();
        root.insert("data".into(), nbtx::Value::Compound(data));
        nbtx::Value::Compound(root)
    }

    #[test]
    fn dimension_record_the_end_with_exit_portal_when_spawned() {
        let bytes = nbtx::to_le_bytes(&the_end_sample(true, true)).unwrap();
        let record = decode_dimension_record(&bytes).unwrap();
        let fight = record.dragon_fight().unwrap();
        assert!(fight.spawned);
        assert_eq!(fight.exit_portal_location, Some([0, 64, 0]));
        assert_eq!(record.limbo_entities(), Some(&[][..]));
        assert_eq!(encode_dimension_record(&record).unwrap(), bytes);
    }

    #[test]
    fn dimension_record_the_end_without_exit_portal_when_not_spawned() {
        let bytes = nbtx::to_le_bytes(&the_end_sample(false, false)).unwrap();
        let record = decode_dimension_record(&bytes).unwrap();
        let fight = record.dragon_fight().unwrap();
        assert!(!fight.spawned);
        assert_eq!(fight.exit_portal_location, None);
        assert_eq!(encode_dimension_record(&record).unwrap(), bytes);
    }

    #[test]
    fn decode_rejects_trailing_bytes() {
        let mut bytes = nbtx::to_le_bytes(&mob_events_sample()).unwrap();
        bytes.push(0xff);
        assert!(decode_mob_events(&bytes).is_err());
    }

    #[test]
    fn write_dimension_record_rejects_dimension_with_no_key_name() {
        let db_err = raw_keys::dimension_key(Dimension::Other(1000));
        assert!(db_err.is_none());
    }
}
