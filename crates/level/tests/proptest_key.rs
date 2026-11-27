//! Round-trip and malformed-input property tests for the raw key codec
//! (`Key::serialize`/`Key::deserialize`, `crates/level/src/key.rs`) and the
//! subchunk-key index conversions it exposes.
//!
//! Phase 7 (`crates/level/TODO.md`): decode ∘ encode is the identity on
//! well-formed input, and malformed input never panics.

use std::io::Cursor;

use bedrock_level::key::{Key, KeyVariant, subchunk_index_from_key, subchunk_index_to_key};
use bedrock_level::types::ChunkPosition;
use bedrock_shared::world::dimension::Dimension;
use proptest::prelude::*;

/// Every chunk-scoped tag-byte `KeyVariant` (excludes `SubChunk`, which
/// additionally carries an index byte and is generated separately, and
/// `LocalPlayer`/`ActorDigest`, which have no tag byte at all).
fn tag_variant() -> impl Strategy<Value = KeyVariant> {
    prop_oneof![
        Just(KeyVariant::Biome3d),
        Just(KeyVariant::ChunkVersion),
        Just(KeyVariant::HeightMap),
        Just(KeyVariant::LegacyTerrain),
        Just(KeyVariant::BlockEntity),
        Just(KeyVariant::Entity),
        Just(KeyVariant::PendingTicks),
        Just(KeyVariant::LegacyBlockExtraData),
        Just(KeyVariant::BiomeState),
        Just(KeyVariant::FinalizedState),
        Just(KeyVariant::ConversionData),
        Just(KeyVariant::BorderBlocks),
        Just(KeyVariant::HardcodedSpawnAreas),
        Just(KeyVariant::RandomTicks),
        Just(KeyVariant::Checksums),
        Just(KeyVariant::GenerationSeed),
        Just(KeyVariant::GeneratedPreCavesAndCliffsBlending),
        Just(KeyVariant::BlendingBiomeHeight),
        Just(KeyVariant::MetadataDictionaryKey),
        Just(KeyVariant::BlendingData),
        Just(KeyVariant::ActorDigestVersion),
        Just(KeyVariant::LegacyVersion),
        Just(KeyVariant::AabbVolumes),
    ]
}

/// A dimension in the accepted range: the four named variants plus a handful
/// of `Other` ids, all comfortably inside `MAX_ACCEPTED_DIMENSION_ID` (see
/// `key.rs`).
fn dimension() -> impl Strategy<Value = Dimension> {
    prop_oneof![
        Just(Dimension::Overworld),
        Just(Dimension::Nether),
        Just(Dimension::End),
        Just(Dimension::Undefined),
        (4i32..=10_000).prop_map(Dimension::from),
    ]
}

fn chunk_position() -> impl Strategy<Value = ChunkPosition> {
    (any::<i32>(), any::<i32>()).prop_map(|(x, z)| ChunkPosition(x, z))
}

/// Reimplements `key::subchunk_key_index_is_offset`'s public contract (that
/// function itself is `pub(crate)`, so this crate's own external tests
/// cannot call it): the on-disk index byte carries a `+4` offset only in the
/// Overworld, for chunk versions 25 through 28 inclusive -- see the module
/// docs on `subchunk_index_from_key`/`subchunk_index_to_key` for the window
/// this mirrors.
fn is_offset_window(chunk_version: u8, dimension: Dimension) -> bool {
    dimension == Dimension::Overworld && (25..=28).contains(&chunk_version)
}

fn any_key() -> impl Strategy<Value = Key> {
    (
        chunk_position(),
        dimension(),
        prop_oneof![
            3 => tag_variant(),
            1 => any::<i8>().prop_map(|index| KeyVariant::SubChunk { index }),
            1 => Just(KeyVariant::LocalPlayer),
            1 => Just(KeyVariant::ActorDigest),
        ],
    )
        .prop_map(|(chunk, dimension, data)| Key {
            chunk,
            dimension,
            data,
        })
}

proptest! {
    /// `Key::deserialize(Key::serialize(k)) == k` for every well-formed key
    /// this crate can construct, across every variant, every accepted
    /// dimension, and arbitrary chunk coordinates. `LocalPlayer` carries a
    /// fixed placeholder position/dimension on the way back out (its wire
    /// form is a bare string with no coordinates at all), so it is checked
    /// separately below rather than folded into this generator.
    #[test]
    fn key_round_trips_through_serialize_then_deserialize(key in any_key()) {
        if key.data == KeyVariant::LocalPlayer {
            return Ok(());
        }

        let mut buf = Vec::new();
        key.serialize(&mut buf).unwrap();

        let mut cursor = Cursor::new(buf.as_slice());
        let decoded = Key::deserialize(&mut cursor).unwrap();
        prop_assert_eq!(decoded, key);
    }

    /// `subchunk_index_to_key` and `subchunk_index_from_key` are exact
    /// inverses for every `(absolute_index, chunk_version, dimension)`
    /// triple -- the `+4`/`-4` offset window is symmetric by construction,
    /// and outside it both are the identity. Saturation only matters at the
    /// extreme ends of `i8`, which `absolute_index` sweeps over here too.
    #[test]
    fn subchunk_index_key_conversion_round_trips(
        absolute_index in any::<i8>(),
        chunk_version in any::<u8>(),
        dimension in dimension(),
    ) {
        let raw = subchunk_index_to_key(absolute_index, chunk_version, dimension);
        let back = subchunk_index_from_key(raw, chunk_version, dimension);

        if is_offset_window(chunk_version, dimension) {
            // Only fails to round-trip if the forward conversion itself
            // saturated: adding the offset pushed it past `i8::MAX`.
            if absolute_index <= i8::MAX - 4 {
                prop_assert_eq!(back, absolute_index);
            }
        } else {
            prop_assert_eq!(raw, absolute_index);
            prop_assert_eq!(back, absolute_index);
        }
    }

    /// A `SubChunk` key's on-disk index byte round-trips through
    /// `Key::subchunk_absolute_index` exactly the same way
    /// `subchunk_index_from_key` does directly -- the method is a thin
    /// wrapper, so this pins that it stays one.
    #[test]
    fn subchunk_absolute_index_matches_the_free_function(
        raw_index in any::<i8>(),
        chunk_version in any::<u8>(),
        dimension in dimension(),
        chunk in chunk_position(),
    ) {
        let key = Key {
            chunk,
            dimension,
            data: KeyVariant::SubChunk { index: raw_index },
        };
        prop_assert_eq!(
            key.subchunk_absolute_index(chunk_version),
            Some(subchunk_index_from_key(raw_index, chunk_version, dimension))
        );
    }

    /// Arbitrary bytes of any length must never panic `Key::deserialize` --
    /// per architecture decision 11, an unrecognized byte sequence falls
    /// back to being treated as a string key (structurally always valid,
    /// since a `Key` cannot itself represent an error for "not a chunk
    /// key"), never a panic.
    #[test]
    fn deserialize_never_panics_on_arbitrary_bytes(bytes in proptest::collection::vec(any::<u8>(), 0..64)) {
        let mut cursor = Cursor::new(bytes.as_slice());
        let _ = Key::deserialize(&mut cursor);
    }

    /// A truncated well-formed key (any prefix of a real encoding) must
    /// likewise never panic, whatever `deserialize` decides to do with it.
    #[test]
    fn deserialize_never_panics_on_truncated_well_formed_keys(
        key in any_key(),
        cut in 0usize..32,
    ) {
        let mut buf = Vec::new();
        key.serialize(&mut buf).unwrap();
        let truncated = &buf[..cut.min(buf.len())];

        let mut cursor = Cursor::new(truncated);
        let _ = Key::deserialize(&mut cursor);
    }
}

/// `LocalPlayer` is not chunk-scoped and has no wire coordinates -- checked
/// on its own since its round trip is "the fixed placeholder value decodes
/// back to itself", not "the same `Key` I started from" (see
/// `KeyVariant::LocalPlayer`'s doc comment for why it carries a placeholder
/// chunk/dimension at all).
#[test]
fn local_player_key_round_trips() {
    let key = Key {
        chunk: ChunkPosition(0, 0),
        dimension: Dimension::Overworld,
        data: KeyVariant::LocalPlayer,
    };
    let mut buf = Vec::new();
    key.serialize(&mut buf).unwrap();
    assert_eq!(buf, b"~local_player");

    let mut cursor = Cursor::new(buf.as_slice());
    let decoded = Key::deserialize(&mut cursor).unwrap();
    assert_eq!(decoded.data, KeyVariant::LocalPlayer);
}
