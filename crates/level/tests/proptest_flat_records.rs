//! Round-trip and malformed-input property tests for the flat,
//! database-wide (not chunk-scoped) records: `map_<id>`
//! (`crates/level/src/map_data.rs`), `portals`
//! (`crates/level/src/portals.rs`), and `PosTrackDB-0x<id>` /
//! `PositionTrackDB-LastId` (`crates/level/src/position_track.rs`).

use bedrock_level::map_data::{
    decode as decode_map, encode as encode_map, id_from_key, key as map_key,
};
use bedrock_level::portals::{
    PortalRecord, Portals, decode as decode_portals, encode as encode_portals,
};
use bedrock_level::position_track::{
    Entry, LastId, decode_entry, decode_last_id, encode_entry, encode_last_id, id_string,
};
use proptest::prelude::*;

// ---------------------------------------------------------------------
// map_<id>
// ---------------------------------------------------------------------

/// A minimal map-record NBT compound: a `mapId` field plus a handful of
/// other primitive fields. `map_data::decode` always keeps the original
/// bytes (see `MapRecord::original`), so `encode` after `decode` reproduces
/// them verbatim regardless of field set or order -- the "empty list loses
/// its declared element type" carve-out documented on `map_data.rs` only
/// bites a record built fresh through `MapRecord::new`, which this
/// generator never does.
fn map_record_bytes() -> impl Strategy<Value = Vec<u8>> {
    (
        any::<i64>(),
        any::<i8>(),
        "[a-zA-Z0-9 ]{0,20}".prop_map(|s| s),
    )
        .prop_map(|(map_id, scale, name)| {
            let mut c = nbtx::Compound::new();
            c.insert("mapId".into(), nbtx::Value::Long(map_id));
            c.insert("scale".into(), nbtx::Value::Byte(scale));
            c.insert("name".into(), nbtx::Value::String(name.into()));
            nbtx::to_le_bytes(&nbtx::Value::Compound(c)).unwrap()
        })
}

proptest! {
    /// A decoded map record always re-encodes to its exact original bytes.
    #[test]
    fn map_record_round_trips(bytes in map_record_bytes()) {
        let record = decode_map(&bytes).unwrap();
        prop_assert_eq!(encode_map(&record).unwrap(), bytes);
    }

    /// Arbitrary bytes never panic `map_data::decode`.
    #[test]
    fn map_record_decode_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..80)) {
        let _ = decode_map(&bytes);
    }

    /// `map_<id>`/`id_from_key` round-trip every representable `i64` id
    /// (both positive and negative, matching real records' negative ids).
    #[test]
    fn map_key_round_trips(id in any::<i64>()) {
        let k = map_key(id);
        prop_assert_eq!(id_from_key(&k), Some(id));
    }
}

// ---------------------------------------------------------------------
// portals
// ---------------------------------------------------------------------

fn portal_record() -> impl Strategy<Value = PortalRecord> {
    (
        any::<i32>(),
        any::<i8>(),
        any::<i32>(),
        any::<i32>(),
        any::<i32>(),
        any::<i8>(),
        any::<i8>(),
    )
        .prop_map(|(dim_id, span, tp_x, tp_y, tp_z, xa, za)| PortalRecord {
            dim_id,
            span,
            tp_x,
            tp_y,
            tp_z,
            xa,
            za,
        })
}

proptest! {
    /// Any list of portal records round-trips byte-identically -- `Portals`
    /// is a fixed, order-preserving struct with no field-order ambiguity
    /// (contrast `map_data`).
    #[test]
    fn portals_round_trips(records in proptest::collection::vec(portal_record(), 0..6)) {
        let portals = Portals { records };
        let bytes = encode_portals(&portals).unwrap();
        prop_assert_eq!(decode_portals(&bytes).unwrap(), portals);
    }

    /// Arbitrary bytes never panic `portals::decode`.
    #[test]
    fn portals_decode_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..120)) {
        let _ = decode_portals(&bytes);
    }
}

// ---------------------------------------------------------------------
// PosTrackDB-0x<id> / PositionTrackDB-LastId
// ---------------------------------------------------------------------

fn position_track_entry() -> impl Strategy<Value = Entry> {
    (
        any::<i32>(),
        any::<u32>(),
        proptest::array::uniform3(any::<i32>()),
        any::<i8>(),
        any::<i8>(),
    )
        .prop_map(|(dim, id, pos, status, version)| {
            Entry::new(dim, id_string(id), pos, status, version)
        })
}

fn last_id() -> impl Strategy<Value = LastId> {
    (any::<u32>(), any::<i8>()).prop_map(|(id, version)| LastId {
        id: id_string(id),
        version,
    })
}

proptest! {
    /// Any well-formed entry (a 3-element `pos`, built through `Entry::new`)
    /// round-trips byte-identically.
    #[test]
    fn position_track_entry_round_trips(entry in position_track_entry()) {
        let bytes = encode_entry(&entry).unwrap();
        prop_assert_eq!(decode_entry(&bytes).unwrap(), entry);
    }

    /// Arbitrary bytes never panic `decode_entry`.
    #[test]
    fn position_track_entry_decode_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..80)) {
        let _ = decode_entry(&bytes);
    }

    /// Any `LastId` counter round-trips byte-identically.
    #[test]
    fn position_track_last_id_round_trips(value in last_id()) {
        let bytes = encode_last_id(&value).unwrap();
        prop_assert_eq!(decode_last_id(&bytes).unwrap(), value);
    }

    /// Arbitrary bytes never panic `decode_last_id`.
    #[test]
    fn position_track_last_id_decode_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..80)) {
        let _ = decode_last_id(&bytes);
    }
}
