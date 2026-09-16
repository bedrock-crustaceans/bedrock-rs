//! Raw-NBT access for the flat, database-wide string keys this crate has no
//! structured decode for yet: `scoreboard`, `BiomeData`, `mobevents`,
//! `schedulerWT`, `game_flatworldlayers`, the three dimension keys
//! (`Overworld`/`Nether`/`TheEnd`), `AutonomousEntities`,
//! `VILLAGE_<dimension>_*`/`mVillages`, `structuretemplate_<name>`, and
//! `tickingarea_<id>`. Structured decode of any of these is a Phase 8
//! stretch goal (see `TODO.md`); this module is the guaranteed baseline
//! Phase 6 asks for -- read and write the raw compound, unmodified, so a
//! caller can inspect or edit these records without this crate
//! understanding their content.
//!
//! # Corpus scan (evidence, not guesswork)
//!
//! A full key scan of all fourteen worlds (the thirteen imported fixtures
//! plus the 1.26 seed world, `tests/level.tar.gz`) found:
//!
//! | Key | Worlds present | Shape |
//! | --- | --- | --- |
//! | `scoreboard` | 14/14 | flat, one per world |
//! | `AutonomousEntities` | 14/14 | flat, one per world |
//! | `BiomeData` | 14/14 | flat, one per world |
//! | `mobevents` | 14/14 | flat, one per world |
//! | `schedulerWT` | 14/14 | flat, one per world |
//! | `Overworld` (dimension key) | 14/14 | flat, one per world |
//! | `TheEnd` (dimension key) | 5/14 (`v1_18`, `v1_18_30`, `v1_19_30`, `v1_20_81`, `1_26_seed`) | flat |
//! | `Nether` (dimension key) | 3/14 (`v1_18`, `v1_19_30`, `v1_20_81`) | flat |
//! | `game_flatworldlayers` | 0/14 | -- |
//! | `VILLAGE_*` / `mVillages` | 0/14 | -- |
//! | `structuretemplate_*` | 0/14 | -- |
//! | `tickingarea_*` | 0/14 | -- |
//!
//! Every present key's value is one uncompressed little-endian NBT
//! compound, exactly the shape [`crate::map_data`] and [`crate::portals`]
//! already handle for their own flat keys, decoded here the same way:
//! [`decode`]/[`encode`] round-trip real records byte-identically (pinned
//! in `tests/raw_keys.rs` against all 78 real records the scan found across
//! the five always-present keys (14 worlds each, 70 records) plus the two
//! dimension keys with partial coverage (`Nether`: 3, `TheEnd`: 5, 8
//! records)), backed by [`nbtx::Value`]'s order-preserving `Compound` (an
//! `IndexMap` by default -- see nbtx's own docs), the same mechanism
//! [`crate::map_data::MapRecord`] relies on.
//!
//! `VILLAGE_*`, `mVillages`, `structuretemplate_*`, and `tickingarea_*` are
//! all absent from the whole corpus: nothing here that touches them has
//! been checked against a real record, only against the on-disk key naming
//! documented publicly (`VILLAGE_<dimension-name>_DWELLERS`/`_INFO`/
//! `_PLAYERS`/`_POI`, one set per dimension that has ever had a village;
//! `structuretemplate_<name>`; `tickingarea_<id>`). [`village_key`],
//! [`structure_template_key`], and [`ticking_area_key`] build the expected
//! shape of those keys for a caller who wants to try reading one, but nothing
//! here asserts what payload they hold.
//!
//! # Write side
//!
//! [`write`] and [`RawCompound::new`] exist for a caller who wants to edit
//! one of these records; this crate's own write path (`Level`/`Chunk`)
//! does not touch any of them -- they are not chunk-scoped, so decision 13's
//! per-chunk preservation policy has nothing to say about them, and no
//! recomputation rule is proposed here for any of them either. A caller
//! that reads one, edits [`RawCompound::value_mut`], and writes it back is
//! the whole contract.

use std::io::Cursor;

use bedrock_shared::world::dimension::Dimension;

use crate::db::Database;
use crate::error::{Error, Result};
use crate::map_data::RetainedNbt;

pub use crate::key::{AUTONOMOUS_ENTITIES, VILLAGES};

/// `scoreboard`: objectives, criteria, entries, and display slots. Present
/// in 14/14 corpus worlds.
pub const SCOREBOARD: &str = "scoreboard";

/// `BiomeData`: per-biome-id overrides (every real record found carries
/// only `snowAccumulation` entries). Present in 14/14 corpus worlds.
pub const BIOME_DATA: &str = "BiomeData";

/// `mobevents`: booleans toggling the mob-event spawners
/// (`events_enabled`, `minecraft:ender_dragon_event`,
/// `minecraft:pillager_patrols_event`, `minecraft:wandering_trader_event`
/// observed). Present in 14/14 corpus worlds.
pub const MOB_EVENTS: &str = "mobevents";

/// `schedulerWT`: wandering-trader spawn-scheduling state
/// (`daysSinceLastWTSpawn`, `isSpawningWT`, `nextWTSpawnCheckTick`
/// observed). Present in 14/14 corpus worlds.
pub const SCHEDULER_WT: &str = "schedulerWT";

/// `game_flatworldlayers`: the flat-world layer JSON (not NBT -- unlike
/// every other key in this module, its value is a JSON string, not a
/// compound; [`decode`]/[`encode`] do not apply to it). Absent from the
/// whole corpus, so nothing here has been checked against a real record.
pub const GAME_FLATWORLDLAYERS: &str = "game_flatworldlayers";

/// ASCII prefix on a `VILLAGE_<dimension>_<DWELLERS|INFO|PLAYERS|POI>` key.
/// Absent from the whole corpus -- see the module docs.
pub const VILLAGE_PREFIX: &str = "VILLAGE_";

/// The four suffixes a real `VILLAGE_<dimension>_*` key is documented to
/// carry, per dimension. Unverified against a real record (see the module
/// docs).
pub const VILLAGE_SUFFIXES: [&str; 4] = ["DWELLERS", "INFO", "PLAYERS", "POI"];

/// ASCII prefix on a `structuretemplate_<name>` key: a saved structure
/// block's blocks/entities/palette. Absent from the whole corpus.
pub const STRUCTURE_TEMPLATE_PREFIX: &str = "structuretemplate_";

/// ASCII prefix on a `tickingarea_<id>` key: a persistent ticking-area
/// definition. Absent from the whole corpus.
pub const TICKING_AREA_PREFIX: &str = "tickingarea_";

/// Maps a [`Dimension`] to the exact bare string key its own flat,
/// non-chunk-scoped `Overworld`/`Nether`/`TheEnd` dimension record is
/// stored under (`DragonFight` state on `TheEnd`, `LimboEntities` on
/// every dimension observed -- see the corpus table in the module docs).
/// `None` for anything other than the three built-in dimensions: nothing
/// in the corpus or either reference names a dimension key for a custom
/// add-on dimension, and Bedrock's custom-dimension ids have no fixed
/// name to spell one with.
pub fn dimension_key_name(dimension: Dimension) -> Option<&'static str> {
    match dimension {
        Dimension::Overworld => Some("Overworld"),
        Dimension::Nether => Some("Nether"),
        Dimension::End => Some("TheEnd"),
        Dimension::Undefined | Dimension::Other(_) => None,
    }
}

/// The on-disk key bytes for `dimension`'s flat dimension record -- see
/// [`dimension_key_name`].
pub fn dimension_key(dimension: Dimension) -> Option<Vec<u8>> {
    dimension_key_name(dimension).map(|s| s.as_bytes().to_vec())
}

/// Builds a `VILLAGE_<dimension>_<suffix>` key. `suffix` is expected to be
/// one of [`VILLAGE_SUFFIXES`] but is not checked against it -- the whole
/// family is unverified against a real record (see the module docs), so
/// this does not pretend to reject a suffix the real game might still use.
/// Returns `None` if `dimension` has no [`dimension_key_name`].
pub fn village_key(dimension: Dimension, suffix: &str) -> Option<Vec<u8>> {
    let name = dimension_key_name(dimension)?;
    Some(format!("{VILLAGE_PREFIX}{name}_{suffix}").into_bytes())
}

/// Builds a `structuretemplate_<name>` key.
pub fn structure_template_key(name: &str) -> Vec<u8> {
    format!("{STRUCTURE_TEMPLATE_PREFIX}{name}").into_bytes()
}

/// Builds a `tickingarea_<id>` key.
pub fn ticking_area_key(id: &str) -> Vec<u8> {
    format!("{TICKING_AREA_PREFIX}{id}").into_bytes()
}

/// One decoded raw record: an opaque NBT compound under one of this
/// module's keys, with typed accessors deliberately absent -- this is the
/// pre-structured-decode baseline, so there is nothing to type yet. See
/// [`crate::map_data::MapRecord`], which this mirrors, and
/// [`RetainedNbt`], whose doc comment covers the retained-original-bytes
/// behavior both share.
#[derive(Debug, Clone, PartialEq)]
pub struct RawCompound {
    inner: RetainedNbt,
}

impl RawCompound {
    /// Wraps an already-decoded NBT value as a record with no original
    /// on-disk bytes to fall back to.
    pub fn new(value: nbtx::Value) -> Self {
        Self {
            inner: RetainedNbt::new(value),
        }
    }

    /// The record's full NBT value, verbatim.
    pub fn value(&self) -> &nbtx::Value {
        self.inner.value()
    }

    /// Mutable access to the record's NBT value. Clears the preserved
    /// original bytes -- see [`RetainedNbt`]'s doc comment -- so the next
    /// [`encode`] re-derives from the edited value instead of returning
    /// stale bytes.
    pub fn value_mut(&mut self) -> &mut nbtx::Value {
        self.inner.value_mut()
    }

    /// Unwraps into the decoded NBT value.
    pub fn into_value(self) -> nbtx::Value {
        self.inner.into_value()
    }
}

/// Decodes one of this module's records: one uncompressed little-endian
/// NBT compound with no trailing bytes. Shared by every key this module
/// names except [`GAME_FLATWORLDLAYERS`], whose value is JSON, not NBT.
pub fn decode(bytes: &[u8]) -> Result<RawCompound> {
    let mut cursor = Cursor::new(bytes);
    let value: nbtx::Value = nbtx::from_le_bytes(&mut cursor)?;
    if cursor.position() != bytes.len() as u64 {
        return Err(Error::Invalid(
            "raw NBT-keyed record has trailing bytes past its compound",
        ));
    }
    Ok(RawCompound {
        inner: RetainedNbt::decoded(value, bytes.to_vec()),
    })
}

/// Encodes one of this module's records: the inverse of [`decode`]. See
/// [`RetainedNbt`]'s doc comment for how a decoded-but-unmodified record's
/// original bytes are preferred over a re-derivation from `value`, and why
/// [`crate::map_data`] uses the same mechanism for the same reason.
pub fn encode(record: &RawCompound) -> Result<Vec<u8>> {
    record.inner.encode()
}

/// Reads and decodes the record at `key`, or `None` if it is absent.
pub fn read(db: &Database, key: &[u8]) -> Result<Option<RawCompound>> {
    let Some(bytes) = db.get(key)? else {
        return Ok(None);
    };
    Ok(Some(decode(&bytes)?))
}

/// Encodes and writes `record` at `key`, overwriting whatever is there.
pub fn write(db: &Database, key: &[u8], record: &RawCompound) -> Result<()> {
    db.insert(key, encode(record)?)
}

/// Every key in `db` carrying `prefix`, found by a full-database key scan
/// -- nothing in the format indexes `structuretemplate_*`/`tickingarea_*`
/// names the way `PositionTrackDB-LastId` indexes lodestone tracking
/// entries (see [`crate::position_track`]), so there is no cheaper way to
/// enumerate them. Order matches the database's own iteration order.
pub fn keys_with_prefix(db: &Database, prefix: &[u8]) -> Result<Vec<Vec<u8>>> {
    let mut out = Vec::new();
    let mut keys = db.keys()?;
    for kv in &mut keys {
        let k = kv.key();
        if k.starts_with(prefix) {
            out.push(k.to_vec());
        }
    }
    Ok(out)
}

/// Every `structuretemplate_<name>` key present in `db`, full key bytes
/// (the prefix is not stripped, since the corpus offers no real record to
/// confirm `<name>`'s own encoding, e.g. whether it may itself contain
/// `/`-separated path segments as public documentation suggests).
pub fn structure_template_keys(db: &Database) -> Result<Vec<Vec<u8>>> {
    keys_with_prefix(db, STRUCTURE_TEMPLATE_PREFIX.as_bytes())
}

/// Every `tickingarea_<id>` key present in `db`, full key bytes.
pub fn ticking_area_keys(db: &Database) -> Result<Vec<Vec<u8>>> {
    keys_with_prefix(db, TICKING_AREA_PREFIX.as_bytes())
}

/// Every `VILLAGE_*` key present in `db`, full key bytes.
pub fn village_keys(db: &Database) -> Result<Vec<Vec<u8>>> {
    keys_with_prefix(db, VILLAGE_PREFIX.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_value() -> nbtx::Value {
        let mut compound = nbtx::Compound::new();
        compound.insert("events_enabled".into(), nbtx::Value::Byte(1));
        compound.insert("minecraft:ender_dragon_event".into(), nbtx::Value::Byte(1));
        nbtx::Value::Compound(compound)
    }

    #[test]
    fn decode_then_encode_round_trips() {
        let bytes = nbtx::to_le_bytes(&sample_value()).unwrap();
        let record = decode(&bytes).unwrap();
        assert_eq!(encode(&record).unwrap(), bytes);
    }

    #[test]
    fn decode_rejects_trailing_bytes() {
        let mut bytes = nbtx::to_le_bytes(&sample_value()).unwrap();
        bytes.push(0xff);
        assert!(decode(&bytes).is_err());
    }

    #[test]
    fn value_mut_clears_original_and_re_derives() {
        let bytes = nbtx::to_le_bytes(&sample_value()).unwrap();
        let mut record = decode(&bytes).unwrap();
        assert_eq!(encode(&record).unwrap(), bytes);

        let nbtx::Value::Compound(compound) = record.value_mut() else {
            panic!("expected a compound");
        };
        compound.insert("extra".into(), nbtx::Value::Byte(0));

        let re_encoded = encode(&record).unwrap();
        assert_ne!(re_encoded, bytes);
        let re_decoded = decode(&re_encoded).unwrap();
        assert_eq!(re_decoded.value(), record.value());
    }

    #[test]
    fn dimension_key_names_match_the_corpus() {
        assert_eq!(dimension_key_name(Dimension::Overworld), Some("Overworld"));
        assert_eq!(dimension_key_name(Dimension::Nether), Some("Nether"));
        assert_eq!(dimension_key_name(Dimension::End), Some("TheEnd"));
        assert_eq!(dimension_key_name(Dimension::Other(1000)), None);
    }

    #[test]
    fn dimension_key_builds_the_bare_string_key() {
        assert_eq!(
            dimension_key(Dimension::Overworld),
            Some(b"Overworld".to_vec())
        );
        assert_eq!(dimension_key(Dimension::Other(1000)), None);
    }

    #[test]
    fn village_key_builds_the_documented_shape() {
        assert_eq!(
            village_key(Dimension::Overworld, "DWELLERS"),
            Some(b"VILLAGE_Overworld_DWELLERS".to_vec())
        );
        assert_eq!(village_key(Dimension::Other(1000), "DWELLERS"), None);
    }

    #[test]
    fn structure_template_and_ticking_area_keys_build_the_documented_shape() {
        assert_eq!(
            structure_template_key("mystructure"),
            b"structuretemplate_mystructure".to_vec()
        );
        assert_eq!(ticking_area_key("my area"), b"tickingarea_my area".to_vec());
    }
}
