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

/// The ASCII prefix on a `digp` (actor digest) key. Unlike every other
/// chunk-scoped key, `digp` has no single tag byte after the coordinates --
/// this prefix stands in its place, and comes *before* the coordinates
/// rather than after, so it needs its own shape in [`Key::serialize`]/
/// [`Key::deserialize`] rather than flowing through [`KeyVariant::discriminant`]
/// and [`Key::known_tag`].
pub const ACTOR_DIGEST_PREFIX: &[u8; 4] = b"digp";

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum KeyVariant {
    Biome3d = 0x2b,
    ChunkVersion = 0x2c,
    HeightMap = 0x2d,
    SubChunk {
        index: i8,
    } = 0x2f,
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
    MetadataDictionaryKey = 0x3f,
    BlendingData = 0x40,
    ActorDigestVersion = 0x41,
    LegacyVersion = 0x76,
    AabbVolumes = 0x77,
    LocalPlayer,
    /// `digp`: the actor digest, a flat array of `actorprefix` storage keys
    /// belonging to this chunk. Chunk-scoped (its on-disk key carries the
    /// same X/Z/dimension coordinates as every other chunk key) but shaped
    /// differently on the wire -- see [`ACTOR_DIGEST_PREFIX`]. Per
    /// architecture decision 13 this is a recomputed index, not a preserved
    /// cache: a value read here is carried through this raw layer verbatim
    /// like everything else in a [`crate::chunk::ChunkRecords`] group, but a
    /// write path that edits actors must regenerate it rather than trust a
    /// stale copy (Phase 5's GC/maintenance work, not this layer's).
    ActorDigest,
}

impl KeyVariant {
    /// Whether this variant's on-disk key carries a real chunk position and
    /// dimension, as opposed to [`KeyVariant::LocalPlayer`], whose `Key` is
    /// a fixed placeholder because the underlying key is a bare string with
    /// no coordinates in it at all. Used to decide chunk record-group
    /// membership: only chunk-scoped variants can belong to one.
    ///
    /// Written as an exhaustive match with no wildcard arm, like
    /// [`Self::discriminant`] just below -- a variant added later must be
    /// placed here explicitly rather than silently inheriting `true` from a
    /// catch-all.
    pub const fn is_chunk_scoped(&self) -> bool {
        match self {
            KeyVariant::Biome3d => true,
            KeyVariant::ChunkVersion => true,
            KeyVariant::HeightMap => true,
            KeyVariant::SubChunk { .. } => true,
            KeyVariant::LegacyTerrain => true,
            KeyVariant::BlockEntity => true,
            KeyVariant::Entity => true,
            KeyVariant::PendingTicks => true,
            KeyVariant::BiomeState => true,
            KeyVariant::FinalizedState => true,
            KeyVariant::BorderBlocks => true,
            KeyVariant::HardcodedSpawnAreas => true,
            KeyVariant::RandomTicks => true,
            KeyVariant::Checksums => true,
            KeyVariant::GenerationSeed => true,
            KeyVariant::GeneratedPreCavesAndCliffsBlending => true,
            KeyVariant::BlendingBiomeHeight => true,
            KeyVariant::MetadataDictionaryKey => true,
            KeyVariant::BlendingData => true,
            KeyVariant::ActorDigestVersion => true,
            KeyVariant::LegacyVersion => true,
            KeyVariant::AabbVolumes => true,
            KeyVariant::LocalPlayer => false,
            // Unlike `LocalPlayer`, `digp`'s on-disk key really does carry
            // this chunk's X/Z/dimension -- it is the digest *of* this
            // chunk's actors, not a global record wearing a placeholder
            // position. `actorprefix` records are the ones with no
            // coordinates at all, and they are not `Key`s/`KeyVariant`s in
            // the first place (see `crate::actor`), so there is no arm for
            // them here.
            KeyVariant::ActorDigest => true,
        }
    }

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
            KeyVariant::MetadataDictionaryKey => 0x3f,
            KeyVariant::BlendingData => 0x40,
            KeyVariant::ActorDigestVersion => 0x41,
            KeyVariant::LegacyVersion => 0x76,
            KeyVariant::AabbVolumes => 0x77,
            // Neither of these has a real on-disk tag byte: `LocalPlayer` is
            // a bare string key, and `digp` is identified by
            // `ACTOR_DIGEST_PREFIX` instead (see its doc comment). Both are
            // excluded from `known_tag` below and special-cased in
            // `Key::serialize`/`Key::deserialize`, so this value is never
            // actually written to disk for either -- it only exists to keep
            // this match exhaustive.
            KeyVariant::LocalPlayer => u8::MAX,
            KeyVariant::ActorDigest => u8::MAX,
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
pub(crate) fn subchunk_key_index_is_offset(chunk_version: u8, dimension: Dimension) -> bool {
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

/// Highest dimension id [`known_dimension_id`] accepts as a real dimension field rather
/// than rejecting it back to string-key parsing.
///
/// Bedrock's own custom-dimension registration starts well above the vanilla ids and
/// assigns them sequentially: ids are handed out starting at 1000 and walk upward by one
/// each time a new one is needed, so no known world's dimension id comes anywhere close to
/// this bound, and a world would need thousands of distinct add-on dimensions to approach
/// it. It is also comfortably below
/// what four bytes of incidental string content would decode to: reinterpreted as a
/// little-endian `i32`, four printable-ASCII bytes land at 538 million or higher (`0x20`
/// is the lowest printable byte), so this bound keeps rejecting essentially all of that
/// collision space while accepting any id a real add-on world could plausibly register.
/// It is a heuristic, not a guarantee -- see the residual-ambiguity note on
/// [`Key::deserialize`] -- chosen to keep as much of the old check's discriminating power
/// as widening the accepted set allows.
const MAX_ACCEPTED_DIMENSION_ID: i32 = u16::MAX as i32;

/// Maps an on-disk dimension `i32` field to a [`Dimension`], if it is structurally
/// plausible as one: strictly between 0 (exclusive -- overworld is never written
/// explicitly, so a dimensioned key carrying literal 0 is not a valid chunk key) and
/// [`MAX_ACCEPTED_DIMENSION_ID`] (inclusive). Ids 1-3 map to the named
/// [`Dimension`] variants for the same reasons as before this bound existed; anything
/// else in range becomes [`Dimension::Other`], carrying the id unchanged. Shared by every
/// chunk-key shape that carries an explicit dimension field -- the tag-byte scheme and
/// `digp` alike.
fn known_dimension_id(id: i32) -> Option<Dimension> {
    if (1..=MAX_ACCEPTED_DIMENSION_ID).contains(&id) {
        Some(Dimension::from(id))
    } else {
        None
    }
}

impl Key {
    pub fn size_hint(&self) -> usize {
        let dim_size = if self.dimension == Dimension::Overworld {
            0
        } else {
            4
        };

        match &self.data {
            KeyVariant::LocalPlayer => LOCAL_PLAYER.len(),
            // Prefix instead of a tag byte, and no index byte -- see
            // `ACTOR_DIGEST_PREFIX`.
            KeyVariant::ActorDigest => ACTOR_DIGEST_PREFIX.len() + 4 + 4 + dim_size,
            KeyVariant::SubChunk { .. } => 4 + 4 + dim_size + 1 + 1,
            _ => 4 + 4 + dim_size + 1,
        }
    }

    pub fn serialize<W: Write>(&self, mut writer: W) -> Result<()> {
        // Both of these are string-shaped keys with no tag byte, not the
        // generic `x | z | dimension? | tag | index?` layout every other
        // variant shares below -- `LocalPlayer` has no coordinates at all,
        // and `digp` puts its identifying prefix *before* the coordinates
        // rather than a tag byte after them.
        if self.data == KeyVariant::LocalPlayer {
            return Ok(writer.write_all(LOCAL_PLAYER.as_bytes())?);
        }
        if self.data == KeyVariant::ActorDigest {
            writer.write_all(ACTOR_DIGEST_PREFIX)?;
            writer.write_i32::<LittleEndian>(self.chunk.0)?;
            writer.write_i32::<LittleEndian>(self.chunk.1)?;
            if self.dimension != Dimension::Overworld {
                writer.write_i32::<LittleEndian>(i32::from(self.dimension))?;
            }
            return Ok(());
        }

        writer.write_i32::<LittleEndian>(self.chunk.0)?;
        writer.write_i32::<LittleEndian>(self.chunk.1)?;

        if self.dimension != Dimension::Overworld {
            writer.write_i32::<LittleEndian>(i32::from(self.dimension))?;
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
    /// `LocalPlayer` and `ActorDigest` have no on-disk tag byte at all -- one is a
    /// bare string key, the other is identified by [`ACTOR_DIGEST_PREFIX`] instead
    /// -- so both are likewise excluded and handled by their own branches in
    /// [`Key::deserialize`].
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
            0x3f => KeyVariant::MetadataDictionaryKey,
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

        // `digp` has its own binary shape -- `ACTOR_DIGEST_PREFIX | x:i32 | z:i32 | dimension:i32?`
        // -- disjoint in length from every other chunk-key shape below (12/16 bytes here vs.
        // 9/10/13/14 there), so there is no ambiguity between the two checks on length alone.
        // Checked first because, unlike the tag-byte shapes, the identifying bytes come before
        // the coordinates rather than after them.
        //
        // This is still a heuristic, not a guarantee, exactly like the tag-byte shapes below: a
        // string key that happens to be exactly 12 or 16 bytes long and literally starts with
        // `digp` (and, at 16 bytes, whose bytes 12..16 collide with an accepted dimension id) is
        // indistinguishable from a real `digp` key on the wire and will be misparsed as one.
        if matches!(len, 12 | 16) {
            let mut prefix = [0u8; 4];
            reader.read_exact(&mut prefix)?;
            if &prefix == ACTOR_DIGEST_PREFIX {
                let x = reader.read_i32::<LittleEndian>()?;
                let z = reader.read_i32::<LittleEndian>()?;
                let chunk = ChunkPosition(x, z);

                let dimension = if len == 16 {
                    known_dimension_id(reader.read_i32::<LittleEndian>()?)
                } else {
                    Some(Dimension::Overworld)
                };

                if let Some(dimension) = dimension {
                    return Ok(Self {
                        chunk,
                        dimension,
                        data: KeyVariant::ActorDigest,
                    });
                }
            }
            reader.set_position(start_position);
        }

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
        // further signal to disambiguate. Accepting add-on ids weakened this: the dimension
        // field used to rule out all but 3 of the 2^32 possible i32 values (1, 2, 3), and now
        // rules out all but `MAX_ACCEPTED_DIMENSION_ID` of them -- see that constant's doc
        // comment for why the residual window is still a small, deliberately chosen slice of
        // the id space rather than the whole thing. The tag-byte check carries the remaining
        // discriminating power and still has to agree independently.
        if matches!(len, 9 | 10 | 13 | 14) {
            let x = reader.read_i32::<LittleEndian>()?;
            let z = reader.read_i32::<LittleEndian>()?;
            let chunk = ChunkPosition(x, z);

            let dimensioned = len == 13 || len == 14;
            let dimension = if dimensioned {
                // See `known_dimension_id` for exactly which ids are accepted and why.
                known_dimension_id(reader.read_i32::<LittleEndian>()?)
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

    /// `Key::serialize` previously fell through to the generic tag-byte shape
    /// for `LocalPlayer` (writing 8 zero coordinate bytes plus a `0xff` tag
    /// rather than `~local_player`'s 13 ASCII bytes), so it never actually
    /// round-tripped -- nothing exercised `serialize` on this variant before.
    /// Fixed alongside `digp`'s equally string-shaped key.
    #[test]
    fn local_player_key_round_trips() {
        let key = Key {
            chunk: ChunkPosition(0, 0),
            dimension: Dimension::Overworld,
            data: KeyVariant::LocalPlayer,
        };

        let mut buf = Vec::new();
        key.serialize(&mut buf).unwrap();
        assert_eq!(buf, LOCAL_PLAYER.as_bytes());
        assert_eq!(key.size_hint(), LOCAL_PLAYER.len());

        roundtrip(&key);
    }

    #[test]
    fn overworld_digp_key_12_bytes() {
        let mut buf = Vec::new();
        buf.extend_from_slice(ACTOR_DIGEST_PREFIX);
        buf.extend_from_slice(&3i32.to_le_bytes());
        buf.extend_from_slice(&(-7i32).to_le_bytes());

        let mut cursor = Cursor::new(buf.as_slice());
        let key = Key::deserialize(&mut cursor).unwrap();

        assert_eq!(key.chunk, ChunkPosition(3, -7));
        assert_eq!(key.dimension, Dimension::Overworld);
        assert_eq!(key.data, KeyVariant::ActorDigest);
        assert!(key.data.is_chunk_scoped());

        let mut re_encoded = Vec::new();
        key.serialize(&mut re_encoded).unwrap();
        assert_eq!(re_encoded, buf, "digp key must re-encode byte-identically");
        assert_eq!(key.size_hint(), buf.len());

        roundtrip(&key);
    }

    #[test]
    fn nether_digp_key_16_bytes() {
        let mut buf = Vec::new();
        buf.extend_from_slice(ACTOR_DIGEST_PREFIX);
        buf.extend_from_slice(&11i32.to_le_bytes());
        buf.extend_from_slice(&22i32.to_le_bytes());
        buf.extend_from_slice(&1i32.to_le_bytes()); // Nether

        let mut cursor = Cursor::new(buf.as_slice());
        let key = Key::deserialize(&mut cursor).unwrap();

        assert_eq!(key.chunk, ChunkPosition(11, 22));
        assert_eq!(key.dimension, Dimension::Nether);
        assert_eq!(key.data, KeyVariant::ActorDigest);

        let mut re_encoded = Vec::new();
        key.serialize(&mut re_encoded).unwrap();
        assert_eq!(re_encoded, buf, "digp key must re-encode byte-identically");

        roundtrip(&key);
    }

    #[test]
    fn digp_key_with_invalid_dimension_is_rejected() {
        let mut buf = Vec::new();
        buf.extend_from_slice(ACTOR_DIGEST_PREFIX);
        buf.extend_from_slice(&1i32.to_le_bytes());
        buf.extend_from_slice(&2i32.to_le_bytes());
        // Outside `MAX_ACCEPTED_DIMENSION_ID`, unlike the add-on-dimension-range case
        // covered by `digp_key_with_an_add_on_dimension_id_round_trips` below.
        buf.extend_from_slice(&(MAX_ACCEPTED_DIMENSION_ID + 1).to_le_bytes());

        let mut cursor = Cursor::new(buf.as_slice());
        assert!(Key::deserialize(&mut cursor).is_err());
    }

    /// A dimension id in the widened but still-bounded accepted range (here, a value in
    /// the id space real add-on dimensions actually use) parses as [`Dimension::Other`]
    /// rather than being rejected -- this is the behavior change this widening exists for.
    #[test]
    fn digp_key_with_an_add_on_dimension_id_round_trips() {
        let mut buf = Vec::new();
        buf.extend_from_slice(ACTOR_DIGEST_PREFIX);
        buf.extend_from_slice(&1i32.to_le_bytes());
        buf.extend_from_slice(&2i32.to_le_bytes());
        buf.extend_from_slice(&1000i32.to_le_bytes());

        let mut cursor = Cursor::new(buf.as_slice());
        let key = Key::deserialize(&mut cursor).unwrap();

        assert_eq!(key.dimension, Dimension::Other(1000));
        assert_eq!(key.data, KeyVariant::ActorDigest);

        let mut re_encoded = Vec::new();
        key.serialize(&mut re_encoded).unwrap();
        assert_eq!(re_encoded, buf, "digp key must re-encode byte-identically");

        roundtrip(&key);
    }

    /// A 12/16-byte key that merely starts with `digp` bytes but isn't
    /// actually the digest prefix (i.e. a string key that happens to share
    /// the first four bytes) must not be misparsed. This is a synthetic
    /// stress on the length/prefix check, distinct from the existing
    /// 8-byte `string_key_digp_prefix_is_not_a_chunk_key` case above, which
    /// exercises a length neither shape accepts.
    #[test]
    fn twelve_byte_key_with_wrong_prefix_is_not_a_digp_key() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"DIGP"); // wrong case, not ACTOR_DIGEST_PREFIX
        buf.extend_from_slice(&1i32.to_le_bytes());
        buf.extend_from_slice(&2i32.to_le_bytes());

        let mut cursor = Cursor::new(buf.as_slice());
        assert!(Key::deserialize(&mut cursor).is_err());
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
        // Negative ids are never valid: no real dimension id is negative, so
        // `known_dimension_id` rejects them regardless of magnitude.
        buf.extend_from_slice(&(-7i32).to_le_bytes());
        buf.push(0x2d); // HeightMap

        let mut cursor = Cursor::new(buf.as_slice());
        assert!(Key::deserialize(&mut cursor).is_err());
    }

    #[test]
    fn thirteen_byte_key_just_past_the_accepted_dimension_bound_is_rejected() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&1i32.to_le_bytes());
        buf.extend_from_slice(&2i32.to_le_bytes());
        buf.extend_from_slice(&(MAX_ACCEPTED_DIMENSION_ID + 1).to_le_bytes());
        buf.push(0x2d); // HeightMap

        let mut cursor = Cursor::new(buf.as_slice());
        assert!(Key::deserialize(&mut cursor).is_err());
    }

    #[test]
    fn thirteen_byte_key_at_the_accepted_dimension_bound_round_trips() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&1i32.to_le_bytes());
        buf.extend_from_slice(&2i32.to_le_bytes());
        buf.extend_from_slice(&MAX_ACCEPTED_DIMENSION_ID.to_le_bytes());
        buf.push(0x2d); // HeightMap

        let mut cursor = Cursor::new(buf.as_slice());
        let key = Key::deserialize(&mut cursor).unwrap();

        assert_eq!(key.dimension, Dimension::Other(MAX_ACCEPTED_DIMENSION_ID));
        assert_eq!(key.data, KeyVariant::HeightMap);

        roundtrip(&key);
    }

    /// A dimension id well inside the widened range but outside 0-3 parses as
    /// [`Dimension::Other`] and re-encodes to the identical bytes -- the widened
    /// round-trip pinned at the type level in `dimension.rs`, exercised here through
    /// the actual key wire format.
    #[test]
    fn thirteen_byte_key_with_an_add_on_dimension_id_round_trips() {
        let key = Key {
            chunk: ChunkPosition(5, -6),
            dimension: Dimension::Other(1000),
            data: KeyVariant::ChunkVersion,
        };

        roundtrip(&key);
    }

    #[test]
    fn fourteen_byte_subchunk_key_with_an_add_on_dimension_id_round_trips() {
        let key = Key {
            chunk: ChunkPosition(5, -6),
            dimension: Dimension::Other(2024),
            data: KeyVariant::SubChunk { index: -12 },
        };

        roundtrip(&key);
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

    #[test]
    fn is_chunk_scoped_is_false_only_for_local_player() {
        assert!(!KeyVariant::LocalPlayer.is_chunk_scoped());

        for variant in [
            KeyVariant::Biome3d,
            KeyVariant::ChunkVersion,
            KeyVariant::HeightMap,
            KeyVariant::SubChunk { index: 3 },
            KeyVariant::LegacyTerrain,
            KeyVariant::BlockEntity,
            KeyVariant::Entity,
            KeyVariant::FinalizedState,
            KeyVariant::Checksums,
            KeyVariant::LegacyVersion,
            KeyVariant::AabbVolumes,
            KeyVariant::ActorDigest,
        ] {
            assert!(
                variant.is_chunk_scoped(),
                "{variant:?} should be chunk-scoped"
            );
        }
    }
}
