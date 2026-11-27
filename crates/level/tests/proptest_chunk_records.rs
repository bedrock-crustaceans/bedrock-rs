//! Round-trip and malformed-input property tests for the small fixed-shape
//! and count-prefixed chunk records: `0x38` `BorderBlocks`, `0x77`
//! `AabbVolumes`, `0x3b` Checksums, `0x39` `HardcodedSpawnAreas`, `0x36`
//! `FinalizedState`, and the (not chunk-scoped, but decode/encode-symmetric
//! the same way) `LevelChunkMetaDataDictionary`.

pub mod support;

use bedrock_level::aabb_volumes::{
    AabbVolumes, AabbVolumesEntry, decode_aabb_volumes, encode_aabb_volumes,
};
use bedrock_level::border_blocks::{decode_border_blocks, encode_border_blocks};
use bedrock_level::checksums::{ChecksumRecord, decode_checksums, encode_checksums};
use bedrock_level::finalized_state::{
    FinalizedState, decode_finalized_state, encode_finalized_state,
};
use bedrock_level::hardcoded_spawn_areas::{
    HardcodedSpawnArea, decode_hardcoded_spawn_areas, encode_hardcoded_spawn_areas,
};
use bedrock_level::metadata_dictionary::{
    DICTIONARY_KEY_LEN, MetadataDictionary, MetadataDictionaryEntry,
};
use proptest::prelude::*;
use support::nbt::nbt_leaf;

// ---------------------------------------------------------------------
// BorderBlocks (0x38)
// ---------------------------------------------------------------------

proptest! {
    /// Any list of up to 255 positions round-trips byte-identically through
    /// encode/decode.
    #[test]
    fn border_blocks_round_trips(positions in proptest::collection::vec(any::<u8>(), 0..=255)) {
        let bytes = encode_border_blocks(&positions).unwrap();
        prop_assert_eq!(decode_border_blocks(&bytes).unwrap(), positions);
    }

    /// More than 255 positions cannot be encoded (the count is a `u8`) --
    /// `encode_border_blocks` errors rather than truncating or panicking.
    #[test]
    fn border_blocks_encode_rejects_too_many_positions(
        positions in proptest::collection::vec(any::<u8>(), 256..300),
    ) {
        prop_assert!(encode_border_blocks(&positions).is_err());
    }

    /// Arbitrary bytes never panic `decode_border_blocks`.
    #[test]
    fn border_blocks_decode_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..300)) {
        let _ = decode_border_blocks(&bytes);
    }
}

// ---------------------------------------------------------------------
// AabbVolumes (0x77)
// ---------------------------------------------------------------------

fn aabb_entry() -> impl Strategy<Value = AabbVolumesEntry> {
    (any::<i32>(), proptest::array::uniform6(any::<i32>()))
        .prop_map(|(id, fields)| AabbVolumesEntry { id, fields })
}

fn aabb_volumes() -> impl Strategy<Value = AabbVolumes> {
    (
        (any::<i32>(), any::<i32>(), any::<i32>()),
        proptest::collection::vec(any::<u8>(), 0..20),
        proptest::collection::vec(aabb_entry(), 0..5),
    )
        .prop_flat_map(|(header, structure_id, entries)| {
            let trailer_len = 4 * (3 * entries.len() + 2);
            proptest::collection::vec(any::<u8>(), trailer_len).prop_map(move |trailer| {
                AabbVolumes {
                    header,
                    structure_id: structure_id.clone(),
                    entries: entries.clone(),
                    trailer,
                }
            })
        })
}

proptest! {
    /// Any well-formed value (trailer length matching the entry-count
    /// formula, as the type's own docs require) round-trips byte-
    /// identically.
    #[test]
    fn aabb_volumes_round_trips(value in aabb_volumes()) {
        let bytes = encode_aabb_volumes(&value).unwrap();
        prop_assert_eq!(decode_aabb_volumes(&bytes).unwrap(), value);
    }

    /// Arbitrary bytes never panic `decode_aabb_volumes`.
    #[test]
    fn aabb_volumes_decode_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..200)) {
        let _ = decode_aabb_volumes(&bytes);
    }

    /// A truncated well-formed record never panics either.
    #[test]
    fn aabb_volumes_never_panics_on_truncated_records(value in aabb_volumes(), cut in 0usize..200) {
        let bytes = encode_aabb_volumes(&value).unwrap();
        let truncated = &bytes[..cut.min(bytes.len())];
        let _ = decode_aabb_volumes(truncated);
    }
}

// ---------------------------------------------------------------------
// Checksums (0x3b)
// ---------------------------------------------------------------------

fn checksum_record() -> impl Strategy<Value = ChecksumRecord> {
    (
        any::<i16>(),
        any::<u8>(),
        proptest::array::uniform8(any::<u8>()),
    )
        .prop_map(|(tag, subtag, hash)| ChecksumRecord { tag, subtag, hash })
}

proptest! {
    /// Any list of records round-trips byte-identically.
    #[test]
    fn checksums_round_trips(records in proptest::collection::vec(checksum_record(), 0..10)) {
        let bytes = encode_checksums(&records).unwrap();
        prop_assert_eq!(decode_checksums(&bytes).unwrap(), records);
    }

    /// Arbitrary bytes never panic `decode_checksums`.
    #[test]
    fn checksums_decode_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..200)) {
        let _ = decode_checksums(&bytes);
    }
}

// ---------------------------------------------------------------------
// HardcodedSpawnAreas (0x39)
// ---------------------------------------------------------------------

fn spawn_area() -> impl Strategy<Value = HardcodedSpawnArea> {
    (proptest::array::uniform6(any::<i32>()), any::<u8>())
        .prop_map(|(bounds, tag)| HardcodedSpawnArea { bounds, tag })
}

proptest! {
    /// Any list of areas round-trips byte-identically.
    #[test]
    fn hardcoded_spawn_areas_round_trips(areas in proptest::collection::vec(spawn_area(), 0..10)) {
        let bytes = encode_hardcoded_spawn_areas(&areas).unwrap();
        prop_assert_eq!(decode_hardcoded_spawn_areas(&bytes).unwrap(), areas);
    }

    /// Arbitrary bytes never panic `decode_hardcoded_spawn_areas`.
    #[test]
    fn hardcoded_spawn_areas_decode_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..200)) {
        let _ = decode_hardcoded_spawn_areas(&bytes);
    }
}

// ---------------------------------------------------------------------
// FinalizedState (0x36)
// ---------------------------------------------------------------------

fn finalized_state() -> impl Strategy<Value = FinalizedState> {
    prop_oneof![
        any::<i8>().prop_map(FinalizedState::Byte),
        any::<i32>().prop_map(FinalizedState::Int),
    ]
}

proptest! {
    /// Both on-disk widths round-trip byte-identically for any value.
    #[test]
    fn finalized_state_round_trips(value in finalized_state()) {
        let bytes = encode_finalized_state(&value);
        prop_assert_eq!(decode_finalized_state(&bytes).unwrap(), value);
    }

    /// Any length other than 1 or 4 bytes is rejected, never panics.
    #[test]
    fn finalized_state_decode_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..10)) {
        let result = decode_finalized_state(&bytes);
        if bytes.len() != 1 && bytes.len() != 4 {
            prop_assert!(result.is_err());
        }
    }
}

// ---------------------------------------------------------------------
// MetadataDictionary (LevelChunkMetaDataDictionary)
// ---------------------------------------------------------------------

/// A small, bounded NBT compound value -- enough variety (`nbt_leaf()`'s
/// primitive-typed fields) to exercise the order-preserving round trip
/// without needing a full recursive NBT generator.
fn dictionary_entry_value() -> impl Strategy<Value = nbtx::Value> {
    let field_name = "[A-Za-z]{1,12}";
    proptest::collection::vec((field_name, nbt_leaf()), 0..6).prop_map(|fields| {
        let mut compound = nbtx::Compound::new();
        for (name, value) in fields {
            compound.insert(name.into(), value);
        }
        nbtx::Value::Compound(compound)
    })
}

fn dictionary_key() -> impl Strategy<Value = [u8; DICTIONARY_KEY_LEN]> {
    proptest::array::uniform8(any::<u8>())
}

fn metadata_dictionary() -> impl Strategy<Value = MetadataDictionary> {
    proptest::collection::vec((dictionary_key(), dictionary_entry_value()), 0..8).prop_map(
        |entries| {
            let mut dict = MetadataDictionary::new();
            for (key, value) in entries {
                dict.insert(key, MetadataDictionaryEntry::new(value));
            }
            dict
        },
    )
}

proptest! {
    /// Any dictionary built from distinct-or-not keys round-trips byte-
    /// identically: `Value::Compound` is order-preserving, and `insert`
    /// replacing in place (rather than duplicating) matches what `decode`
    /// would produce from the re-encoded bytes.
    #[test]
    fn metadata_dictionary_round_trips(dict in metadata_dictionary()) {
        let bytes = dict.encode().unwrap();
        prop_assert_eq!(MetadataDictionary::decode(&bytes).unwrap(), dict);
    }

    /// Arbitrary bytes never panic `MetadataDictionary::decode`.
    #[test]
    fn metadata_dictionary_decode_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..300)) {
        let _ = MetadataDictionary::decode(&bytes);
    }

    /// A truncated well-formed dictionary never panics either.
    #[test]
    fn metadata_dictionary_never_panics_on_truncated_input(
        dict in metadata_dictionary(),
        cut in 0usize..300,
    ) {
        let bytes = dict.encode().unwrap();
        let truncated = &bytes[..cut.min(bytes.len())];
        let _ = MetadataDictionary::decode(truncated);
    }
}
