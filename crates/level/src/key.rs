use byteorder::{ReadBytesExt, WriteBytesExt};
use nbtx::LittleEndian;

use crate::{
    error::{Error, Result},
    types::ChunkPosition,
};
use bedrock_shared::read::SeekExt;
use bedrock_shared::world::dimension::Dimension;
use std::io::{Cursor, Read, Seek, Write};

pub const AUTONOMOUS_ENTITIES: &str = "AutonomousEntities";
pub const LOCAL_PLAYER: &str = "~local_player";
pub const VILLAGES: &str = "mVillages";

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum KeyVariant {
    Biome3d = 0x2b,
    ChunkVersion = 0x2c,
    HeightMap = 0x2d,
    SubChunk { index: i8 } = 0x2f,
    LegacyTerrain = 0x30,
    BlockEntity = 0x31,
    Entity = 0x32,
    PendingTicks = 0x33,
    BiomeState = 0x35,
    FinalizedState = 0x36,
    BorderBlocks = 0x38,
    HardcodedSpawnAreas = 0x39,
    RandomTicks = 0x3a,
    Checksums = 0x3b,
    GenerationSeed = 0x3c,
    GeneratedPreCavesAndCliffsBlending = 0x3d,
    BlendingBiomeHeight = 0x3e,
    MetadataHash = 0x3f,
    BlendingData = 0x40,
    ActorDigestVersion = 0x41,
    LegacyVersion = 0x76,
    AabbVolumes = 0x77,
    LocalPlayer,
}

impl KeyVariant {
    /// Returns the discriminant of `self`.
    pub const fn discriminant(&self) -> u8 {
        match self {
            KeyVariant::Biome3d => 0x2b,
            KeyVariant::ChunkVersion => 0x2c,
            KeyVariant::HeightMap => 0x2d,
            KeyVariant::SubChunk { .. } => 0x2f,
            KeyVariant::LegacyTerrain => 0x30,
            KeyVariant::BlockEntity => 0x31,
            KeyVariant::Entity => 0x32,
            KeyVariant::PendingTicks => 0x33,
            KeyVariant::BiomeState => 0x35,
            KeyVariant::FinalizedState => 0x36,
            KeyVariant::BorderBlocks => 0x38,
            KeyVariant::HardcodedSpawnAreas => 0x39,
            KeyVariant::RandomTicks => 0x3a,
            KeyVariant::Checksums => 0x3b,
            KeyVariant::GenerationSeed => 0x3c,
            KeyVariant::GeneratedPreCavesAndCliffsBlending => 0x3d,
            KeyVariant::BlendingBiomeHeight => 0x3e,
            KeyVariant::MetadataHash => 0x3f,
            KeyVariant::BlendingData => 0x40,
            KeyVariant::ActorDigestVersion => 0x41,
            KeyVariant::LegacyVersion => 0x76,
            KeyVariant::AabbVolumes => 0x77,
            KeyVariant::LocalPlayer => u8::MAX,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    /// The X and Z coordinates of the requested chunk.
    pub chunk: ChunkPosition,
    /// The dimension of the requested chunk.
    pub dimension: Dimension,
    /// The data to be requested of this chunk.
    pub data: KeyVariant,
}

/// Chunk-version window, inclusive, in which an Overworld subchunk key's
/// index byte is the subchunk's absolute vertical index plus 4, keeping the
/// extended floor's sections at non-negative bytes. Outside this window, or
/// outside the Overworld, the stored byte is the absolute index directly.
const SUBCHUNK_KEY_INDEX_OFFSET_WINDOW_START: u8 = 25;
/// See [`SUBCHUNK_KEY_INDEX_OFFSET_WINDOW_START`].
const SUBCHUNK_KEY_INDEX_OFFSET_WINDOW_END: u8 = 28;

/// The shift applied to a subchunk key's index byte inside
/// [`SUBCHUNK_KEY_INDEX_OFFSET_WINDOW_START`]..=[`SUBCHUNK_KEY_INDEX_OFFSET_WINDOW_END`].
const SUBCHUNK_KEY_INDEX_OFFSET: i8 = 4;

/// Whether `chunk_version`/`dimension` falls inside the window where a
/// subchunk key's index byte carries the `+4` offset.
fn subchunk_key_index_is_offset(chunk_version: u8, dimension: Dimension) -> bool {
    dimension == Dimension::Overworld
        && (SUBCHUNK_KEY_INDEX_OFFSET_WINDOW_START..=SUBCHUNK_KEY_INDEX_OFFSET_WINDOW_END)
            .contains(&chunk_version)
}

/// Converts a subchunk key's raw, on-disk index byte to the subchunk's
/// absolute vertical index.
///
/// `chunk_version` is the owning chunk's own version stamp (`0x2c`/`0x76`):
/// a key carries no version of its own, so this cannot be folded into
/// [`Key::deserialize`], which only ever sees the raw byte. Outside the
/// offset window, or outside the Overworld, this is the identity
/// conversion.
///
/// A version-9 subchunk record embeds this same absolute index directly, as
/// the byte immediately following the version and layer count. Where a
/// record's embedded byte and this conversion disagree, **the record's byte
/// is authoritative** -- this function is for a key read before its record
/// body, or paired with a version-1/8 record, neither of which carries an
/// embedded index to check against.
///
/// Saturates rather than overflowing: a raw index near `i8::MIN` inside the
/// offset window has no valid absolute equivalent representable in `i8`, and
/// no real subchunk index comes anywhere close, so clamping is harmless.
pub fn subchunk_index_from_key(raw_index: i8, chunk_version: u8, dimension: Dimension) -> i8 {
    if subchunk_key_index_is_offset(chunk_version, dimension) {
        raw_index.saturating_sub(SUBCHUNK_KEY_INDEX_OFFSET)
    } else {
        raw_index
    }
}

/// Inverse of [`subchunk_index_from_key`]: converts a subchunk's absolute
/// vertical index to the raw index byte that belongs on its key.
///
/// Saturates rather than overflowing, for the same reason as
/// [`subchunk_index_from_key`].
pub fn subchunk_index_to_key(absolute_index: i8, chunk_version: u8, dimension: Dimension) -> i8 {
    if subchunk_key_index_is_offset(chunk_version, dimension) {
        absolute_index.saturating_add(SUBCHUNK_KEY_INDEX_OFFSET)
    } else {
        absolute_index
    }
}

impl Key {
    pub fn size_hint(&self) -> usize {
        let dim_size = if self.dimension == Dimension::Overworld {
            0
        } else {
            4
        };

        let data_size = if let KeyVariant::SubChunk { .. } = &self.data {
            1
        } else {
            0
        };

        4 + 4 + dim_size + 1 + data_size
    }

    pub fn serialize<W: Write>(&self, mut writer: W) -> Result<()> {
        writer.write_i32::<LittleEndian>(self.chunk.0)?;
        writer.write_i32::<LittleEndian>(self.chunk.1)?;

        if self.dimension != Dimension::Overworld {
            writer.write_i32::<LittleEndian>(self.dimension as i32)?;
        }

        writer.write_u8(self.data.discriminant())?;
        if let KeyVariant::SubChunk { index } = self.data {
            writer.write_i8(index)?;
        }

        Ok(())
    }

    /// If this key is a [`KeyVariant::SubChunk`], returns the subchunk's
    /// absolute vertical index -- see [`subchunk_index_from_key`] for what
    /// `chunk_version` means and the invariant that a version-9 record's own
    /// embedded byte takes precedence over this conversion. Returns `None`
    /// for every other key variant.
    pub fn subchunk_absolute_index(&self, chunk_version: u8) -> Option<i8> {
        match self.data {
            KeyVariant::SubChunk { index } => Some(subchunk_index_from_key(
                index,
                chunk_version,
                self.dimension,
            )),
            _ => None,
        }
    }

    /// Maps a chunk key tag byte to its `KeyVariant`, if it is a recognized one.
    ///
    /// `SubChunk` is intentionally excluded: its wire tag additionally requires a
    /// trailing index byte, which is handled by the caller based on key length.
    /// `LocalPlayer` has no on-disk tag byte at all -- it is a string key -- so it
    /// is likewise excluded.
    fn known_tag(tag: u8) -> Option<KeyVariant> {
        Some(match tag {
            0x2b => KeyVariant::Biome3d,
            0x2c => KeyVariant::ChunkVersion,
            0x2d => KeyVariant::HeightMap,
            0x30 => KeyVariant::LegacyTerrain,
            0x31 => KeyVariant::BlockEntity,
            0x32 => KeyVariant::Entity,
            0x33 => KeyVariant::PendingTicks,
            0x35 => KeyVariant::BiomeState,
            0x36 => KeyVariant::FinalizedState,
            0x38 => KeyVariant::BorderBlocks,
            0x39 => KeyVariant::HardcodedSpawnAreas,
            0x3a => KeyVariant::RandomTicks,
            0x3b => KeyVariant::Checksums,
            0x3c => KeyVariant::GenerationSeed,
            0x3d => KeyVariant::GeneratedPreCavesAndCliffsBlending,
            0x3e => KeyVariant::BlendingBiomeHeight,
            0x3f => KeyVariant::MetadataHash,
            0x40 => KeyVariant::BlendingData,
            0x41 => KeyVariant::ActorDigestVersion,
            0x76 => KeyVariant::LegacyVersion,
            0x77 => KeyVariant::AabbVolumes,
            _ => return None,
        })
    }

    pub fn deserialize<R>(reader: &mut Cursor<R>) -> Result<Key>
    where
        Cursor<R>: Seek + Read,
    {
        let start_position = reader.position();
        let len = reader.stream_len_ext()?;

        // Chunk keys have a fixed binary shape: `x:i32 | z:i32 | dimension:i32? | tag:u8 | index:i8?`.
        // The dimension field is omitted for the overworld (dimension 0 is implied and never
        // written on disk), and the trailing index byte is present only for `SubChunk`. That
        // leaves exactly four valid lengths: 9/10 for the overworld, 13/14 for other dimensions.
        // Anything else -- including any of these four lengths whose tag byte (and, for 13/14,
        // whose dimension i32) doesn't decode to a known value -- is a string key instead.
        //
        // This is still a heuristic, not a guarantee: a string key that happens to be exactly
        // 9, 10, 13, or 14 bytes long, whose byte at the tag offset collides with a known tag
        // (and, for 13/14, whose bytes 8..12 collide with an accepted dimension id), is indistinguishable
        // from a real chunk key on the wire and will be misparsed as one. The format gives no
        // further signal to disambiguate.
        if matches!(len, 9 | 10 | 13 | 14) {
            let x = reader.read_i32::<LittleEndian>()?;
            let z = reader.read_i32::<LittleEndian>()?;
            let chunk = ChunkPosition(x, z);

            let dimensioned = len == 13 || len == 14;
            let dimension = if dimensioned {
                // Accepted ids are exactly the on-disk dimension ids the format defines
                // (1 = Nether, 2 = End) plus the undefined marker (3) that `serialize`
                // can emit. 0 (overworld) is never written explicitly, and anything else
                // is not a chunk key; reject and fall back to string-key parsing below.
                match reader.read_i32::<LittleEndian>()? {
                    1 => Some(Dimension::Nether),
                    2 => Some(Dimension::End),
                    3 => Some(Dimension::Undefined),
                    _ => None,
                }
            } else {
                Some(Dimension::Overworld)
            };

            if let Some(dimension) = dimension {
                let has_index = len == 10 || len == 14;
                let tag = reader.read_u8()?;

                let data = if tag == 0x2f {
                    // The index byte is mandatory for SubChunk and invalid for every other tag.
                    has_index
                        .then(|| reader.read_i8())
                        .transpose()?
                        .map(|index| KeyVariant::SubChunk { index })
                } else if has_index {
                    None
                } else {
                    Self::known_tag(tag)
                };

                if let Some(data) = data {
                    return Ok(Self {
                        chunk,
                        dimension,
                        data,
                    });
                }
            }
        }

        // Not a structurally valid chunk key: fall back to the known string keys.
        reader.set_position(start_position);

        let mut string = String::with_capacity(len as usize);
        reader.read_to_string(&mut string)?;

        if string == LOCAL_PLAYER {
            return Ok(Self {
                chunk: ChunkPosition(0, 0),
                dimension: Dimension::Overworld,
                data: KeyVariant::LocalPlayer,
            });
        }

        Err(Error::Invalid("invalid leveldb database key type"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(key: &Key) {
        let mut buf = Vec::new();
        key.serialize(&mut buf).unwrap();

        let mut cursor = Cursor::new(buf.as_slice());
        let decoded = Key::deserialize(&mut cursor).unwrap();

        assert_eq!(&decoded, key);
    }

    #[test]
    fn overworld_chunk_key_9_bytes() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&1i32.to_le_bytes());
        buf.extend_from_slice(&2i32.to_le_bytes());
        buf.push(0x2b); // Biome3d

        let mut cursor = Cursor::new(buf.as_slice());
        let key = Key::deserialize(&mut cursor).unwrap();

        assert_eq!(key.chunk, ChunkPosition(1, 2));
        assert_eq!(key.dimension, Dimension::Overworld);
        assert_eq!(key.data, KeyVariant::Biome3d);

        roundtrip(&key);
    }

    #[test]
    fn overworld_subchunk_key_10_bytes() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&(-4i32).to_le_bytes());
        buf.extend_from_slice(&5i32.to_le_bytes());
        buf.push(0x2f); // SubChunk
        buf.push(7i8 as u8);

        let mut cursor = Cursor::new(buf.as_slice());
        let key = Key::deserialize(&mut cursor).unwrap();

        assert_eq!(key.chunk, ChunkPosition(-4, 5));
        assert_eq!(key.dimension, Dimension::Overworld);
        assert_eq!(key.data, KeyVariant::SubChunk { index: 7 });

        roundtrip(&key);
    }

    #[test]
    fn nether_chunk_key_13_bytes() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&10i32.to_le_bytes());
        buf.extend_from_slice(&20i32.to_le_bytes());
        buf.extend_from_slice(&1i32.to_le_bytes()); // Nether
        buf.push(0x2d); // HeightMap

        let mut cursor = Cursor::new(buf.as_slice());
        let key = Key::deserialize(&mut cursor).unwrap();

        assert_eq!(key.chunk, ChunkPosition(10, 20));
        assert_eq!(key.dimension, Dimension::Nether);
        assert_eq!(key.data, KeyVariant::HeightMap);

        roundtrip(&key);
    }

    #[test]
    fn end_subchunk_key_14_bytes() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&(-1i32).to_le_bytes());
        buf.extend_from_slice(&(-2i32).to_le_bytes());
        buf.extend_from_slice(&2i32.to_le_bytes()); // End
        buf.push(0x2f); // SubChunk
        buf.push((-3i8) as u8);

        let mut cursor = Cursor::new(buf.as_slice());
        let key = Key::deserialize(&mut cursor).unwrap();

        assert_eq!(key.chunk, ChunkPosition(-1, -2));
        assert_eq!(key.dimension, Dimension::End);
        assert_eq!(key.data, KeyVariant::SubChunk { index: -3 });

        roundtrip(&key);
    }

    #[test]
    fn undefined_dimension_chunk_key_13_bytes_round_trips() {
        // `serialize` can emit dimension id 3 (Undefined), so `deserialize` must accept it.
        let key = Key {
            chunk: ChunkPosition(3, -8),
            dimension: Dimension::Undefined,
            data: KeyVariant::ChunkVersion,
        };

        roundtrip(&key);
    }

    #[test]
    fn undefined_dimension_subchunk_key_14_bytes_round_trips() {
        let key = Key {
            chunk: ChunkPosition(-9, 12),
            dimension: Dimension::Undefined,
            data: KeyVariant::SubChunk { index: 4 },
        };

        roundtrip(&key);
    }

    #[test]
    fn local_player_string_key() {
        let mut cursor = Cursor::new(LOCAL_PLAYER.as_bytes());
        let key = Key::deserialize(&mut cursor).unwrap();

        assert_eq!(key.data, KeyVariant::LocalPlayer);
    }

    #[test]
    fn string_key_biome_data_9_bytes_is_not_a_chunk_key() {
        // "BiomeData" happens to be 9 bytes, the same length as an overworld chunk key,
        // but its byte at the tag offset does not decode to a known `KeyVariant`.
        let mut cursor = Cursor::new(b"BiomeData".as_slice());
        assert!(Key::deserialize(&mut cursor).is_err());
    }

    #[test]
    fn string_key_scoreboard_10_bytes_is_not_a_chunk_key() {
        // "scoreboard" is 10 bytes, the same length as an overworld subchunk key, but its
        // tag byte is not 0x2f (SubChunk), so it is correctly rejected.
        let mut cursor = Cursor::new(b"scoreboard".as_slice());
        assert!(Key::deserialize(&mut cursor).is_err());
    }

    #[test]
    fn string_key_overworld_9_bytes_is_not_a_chunk_key() {
        let mut cursor = Cursor::new(b"Overworld".as_slice());
        assert!(Key::deserialize(&mut cursor).is_err());
    }

    #[test]
    fn string_key_digp_prefix_is_not_a_chunk_key() {
        let mut cursor = Cursor::new(b"digp\x00\x00\x00\x00".as_slice());
        assert!(Key::deserialize(&mut cursor).is_err());
    }

    #[test]
    fn ten_byte_key_with_non_subchunk_tag_is_rejected() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&1i32.to_le_bytes());
        buf.extend_from_slice(&2i32.to_le_bytes());
        buf.push(0x2b); // Biome3d, not SubChunk
        buf.push(0x00); // stray trailing byte

        let mut cursor = Cursor::new(buf.as_slice());
        assert!(Key::deserialize(&mut cursor).is_err());
    }

    #[test]
    fn thirteen_byte_key_with_invalid_dimension_is_rejected() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&1i32.to_le_bytes());
        buf.extend_from_slice(&2i32.to_le_bytes());
        buf.extend_from_slice(&7i32.to_le_bytes()); // not a valid dimension id
        buf.push(0x2d); // HeightMap

        let mut cursor = Cursor::new(buf.as_slice());
        assert!(Key::deserialize(&mut cursor).is_err());
    }

    #[test]
    fn subchunk_index_offset_applies_at_window_boundaries() {
        // Chunk version 25 is the first version in the offset window.
        assert_eq!(subchunk_index_from_key(4, 25, Dimension::Overworld), 0);
        assert_eq!(subchunk_index_to_key(0, 25, Dimension::Overworld), 4);

        // Chunk version 28 is the last version in the window.
        assert_eq!(subchunk_index_from_key(0, 28, Dimension::Overworld), -4);
        assert_eq!(subchunk_index_to_key(-4, 28, Dimension::Overworld), 0);
    }

    #[test]
    fn subchunk_index_offset_absent_just_outside_window() {
        // 24 and 29 are one step outside the window on either side.
        assert_eq!(subchunk_index_from_key(4, 24, Dimension::Overworld), 4);
        assert_eq!(subchunk_index_from_key(4, 29, Dimension::Overworld), 4);
    }

    #[test]
    fn subchunk_index_offset_absent_outside_overworld() {
        for dimension in [Dimension::Nether, Dimension::End, Dimension::Undefined] {
            assert_eq!(subchunk_index_from_key(4, 26, dimension), 4);
            assert_eq!(subchunk_index_to_key(4, 26, dimension), 4);
        }
    }

    #[test]
    fn subchunk_index_offset_round_trips_across_the_whole_window() {
        for chunk_version in 20..=35u8 {
            for raw in -20i8..20 {
                let absolute = subchunk_index_from_key(raw, chunk_version, Dimension::Overworld);
                let back_to_raw =
                    subchunk_index_to_key(absolute, chunk_version, Dimension::Overworld);
                assert_eq!(back_to_raw, raw);
            }
        }
    }

    #[test]
    fn subchunk_index_offset_saturates_instead_of_overflowing_at_the_extremes() {
        // Inside the window, `raw - 4` would overflow `i8` for raw in
        // -128..=-125, and `absolute + 4` would overflow for absolute in
        // 124..=127. The conversions saturate there instead of panicking or
        // silently wrapping, so round-trip identity does not hold at these
        // extremes -- only the saturated bound does.
        for raw in i8::MIN..=i8::MIN + 3 {
            assert_eq!(
                subchunk_index_from_key(raw, 25, Dimension::Overworld),
                i8::MIN
            );
        }
        for absolute in i8::MAX - 3..=i8::MAX {
            assert_eq!(
                subchunk_index_to_key(absolute, 25, Dimension::Overworld),
                i8::MAX
            );
        }
    }

    #[test]
    fn key_subchunk_absolute_index_uses_the_key_dimension() {
        let overworld = Key {
            chunk: ChunkPosition(0, 0),
            dimension: Dimension::Overworld,
            data: KeyVariant::SubChunk { index: 4 },
        };
        assert_eq!(overworld.subchunk_absolute_index(25), Some(0));

        // Same chunk version and raw index, but not the Overworld: the
        // key's own dimension must reach the conversion, not just the
        // window's chunk version, so no offset is applied here.
        let nether = Key {
            chunk: ChunkPosition(0, 0),
            dimension: Dimension::Nether,
            data: KeyVariant::SubChunk { index: 4 },
        };
        assert_eq!(nether.subchunk_absolute_index(25), Some(4));

        let non_subchunk = Key {
            chunk: ChunkPosition(0, 0),
            dimension: Dimension::Overworld,
            data: KeyVariant::ChunkVersion,
        };
        assert_eq!(non_subchunk.subchunk_absolute_index(25), None);
    }
}
