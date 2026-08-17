//! `BiomeIdsTable`: the id-to-name registry for a world's addon-registered
//! custom biomes.
//!
//! A single, flat, database-wide key (not chunk-scoped, not a
//! [`crate::key::Key`]) holding one uncompressed little-endian NBT
//! compound: `{list: [{id: Short, name: String}, ...]}`. Needed to resolve
//! biome ids in a world using addon biomes, which are numbered from a high
//! base outside the vanilla range -- every other biome id this crate
//! reads (2D per-column bytes, 3D palette entries) is purely numeric on
//! disk, so a world with no addon biomes never writes this key at all.
//!
//! # Absent from the corpus, implemented from agreeing references
//!
//! Zero real records: a full scan of all 14 worlds (13 fixtures plus the
//! 1.26 seed) found no `BiomeIdsTable` key in any of them -- consistent
//! with the key only existing for worlds that actually register a custom
//! biome, none of which the corpus happens to include. The rule this task
//! applies for a key with no real record to check a layout against: model
//! it from the reference implementations only when independent references
//! agree on the exact shape, and leave it raw otherwise. Two independent
//! references agree here: one reads `list` as a list of `{name, id}`
//! compounds resolved by field name (order-agnostic on read) with no
//! writer of its own; the other both reads and writes
//! `{id: Short, name: String}` in that field order, wrapped under a bare
//! `list` key with no other sibling field. [`decode`] accepts either field
//! order (matching the order-agnostic reference); [`encode`] writes `id`
//! before `name`, the one reference that commits to an order.
//!
//! `DimensionNameIdTable`, grouped with this key under the same original
//! task ("BiomeIdsTable / DimensionNameIdTable custom-biome registry"), is
//! **not** implemented here. It resolves a different thing entirely --
//! addon-registered *dimension* names to dimension ids
//! (`{entries: {<dimension name>: id, ...}}`), not biome names -- and
//! unlike `BiomeIdsTable` it is single-sourced: one reference implements
//! it (reading and writing, gated to worlds newer than a specific game
//! version), and the other has no trace of it anywhere in its source. The
//! same rule that lets `BiomeIdsTable` be modeled holds `DimensionNameIdTable`
//! back: a single source is exactly the case it does not cover, so a
//! `DimensionNameIdTable` value is left raw -- carried through a
//! read-modify-write untouched like any other key this crate does not
//! parse, per architecture decision 7 -- rather than modeled from one
//! uncorroborated reading.
//!
//! Unknown fields on either NBT type here are tolerated on decode
//! (matching this crate's `player.rs` convention for typed NBT records)
//! rather than rejected, per architecture decision 11 -- "degrade on
//! unmodelled" -- but are not preserved through a decode then encode
//! cycle, since nothing in the corpus has ever shown one (there being no
//! corpus record to show one in, for this particular key).

use std::io::Cursor;

use facet::Facet;

use crate::db::Database;
use crate::error::{Error, Result};

/// The on-disk key this whole table is stored under.
pub const KEY: &[u8] = b"BiomeIdsTable";

/// One entry: a custom biome's numeric id and its registered name.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
#[facet(nbtx::allow_unknown_fields)]
pub struct BiomeIdEntry {
    pub id: i16,
    pub name: String,
}

#[derive(Facet, Debug, Clone, PartialEq)]
#[facet(nbtx::allow_unknown_fields)]
struct BiomeIdsTableRoot {
    list: Vec<BiomeIdEntry>,
}

/// The decoded `BiomeIdsTable` record: every custom biome's id/name pair,
/// in on-disk order.
#[derive(Debug, Clone, PartialEq)]
pub struct BiomeIdsTable {
    pub entries: Vec<BiomeIdEntry>,
}

/// Decodes a `BiomeIdsTable` value.
pub fn decode(bytes: &[u8]) -> Result<BiomeIdsTable> {
    let mut cursor = Cursor::new(bytes);
    let root: BiomeIdsTableRoot = nbtx::from_le_bytes(&mut cursor)?;
    if cursor.position() != bytes.len() as u64 {
        return Err(Error::Invalid(
            "BiomeIdsTable record has trailing bytes past its NBT compound",
        ));
    }
    Ok(BiomeIdsTable { entries: root.list })
}

/// Encodes a `BiomeIdsTable` value. The inverse of [`decode`] -- see the
/// module docs for why `id` is written before `name` within each entry.
pub fn encode(table: &BiomeIdsTable) -> Result<Vec<u8>> {
    let root = BiomeIdsTableRoot {
        list: table.entries.clone(),
    };
    Ok(nbtx::to_le_bytes(&root)?)
}

/// Reads the `BiomeIdsTable` record from `db`, if present.
pub fn read(db: &Database) -> Result<Option<BiomeIdsTable>> {
    match db.get(KEY)? {
        Some(bytes) => Ok(Some(decode(&bytes)?)),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> BiomeIdsTable {
        BiomeIdsTable {
            entries: vec![
                BiomeIdEntry {
                    id: 100,
                    name: "custom:frozen_meadow".to_string(),
                },
                BiomeIdEntry {
                    id: 101,
                    name: "custom:ashen_plains".to_string(),
                },
            ],
        }
    }

    #[test]
    fn decode_then_encode_round_trips() {
        let table = sample();
        let bytes = encode(&table).unwrap();
        assert_eq!(decode(&bytes).unwrap(), table);
    }

    #[test]
    fn encode_writes_id_before_name() {
        let table = BiomeIdsTable {
            entries: vec![BiomeIdEntry {
                id: 100,
                name: "x".to_string(),
            }],
        };
        let bytes = encode(&table).unwrap();
        let id_pos = bytes
            .windows(2)
            .position(|w| w == b"id")
            .expect("id field name present");
        let name_pos = bytes
            .windows(4)
            .position(|w| w == b"name")
            .expect("name field name present");
        assert!(id_pos < name_pos);
    }

    #[test]
    fn decode_accepts_name_before_id_too() {
        // The order-agnostic reference reads by field name regardless of
        // order; a hand-built record with the fields swapped must still
        // decode correctly.
        let mut entry = nbtx::Compound::new();
        entry.insert("name".into(), nbtx::Value::String("x".into()));
        entry.insert("id".into(), nbtx::Value::Short(7));
        let mut root = nbtx::Compound::new();
        root.insert(
            "list".into(),
            nbtx::Value::List(vec![nbtx::Value::Compound(entry)]),
        );
        let bytes = nbtx::to_le_bytes(&nbtx::Value::Compound(root)).unwrap();

        let table = decode(&bytes).unwrap();
        assert_eq!(
            table.entries,
            vec![BiomeIdEntry {
                id: 7,
                name: "x".to_string(),
            }]
        );
    }

    #[test]
    fn empty_table_round_trips() {
        let table = BiomeIdsTable { entries: vec![] };
        let bytes = encode(&table).unwrap();
        assert_eq!(decode(&bytes).unwrap(), table);
    }

    #[test]
    fn decode_rejects_malformed_bytes() {
        assert!(decode(&[0xff]).is_err());
        assert!(decode(&[]).is_err());
    }

    #[test]
    fn decode_rejects_trailing_bytes() {
        let mut bytes = encode(&sample()).unwrap();
        bytes.push(0xff);
        assert!(decode(&bytes).is_err());
    }
}
