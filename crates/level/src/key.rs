use byteorder::{ReadBytesExt, WriteBytesExt};
use nbtx::LittleEndian;

use crate::{
    error::{Error, Result},
    types::ChunkPosition,
};
use bedrockrs_shared::read::SeekExt;
use bedrockrs_shared::world::dimension::Dimension;
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
    MetadataHash = 0x3d,
    GeneratedBlending = 0x3e,
    BlendingBiomeHeight = 0x3f,
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
            KeyVariant::MetadataHash => 0x3d,
            KeyVariant::GeneratedBlending => 0x3e,
            KeyVariant::BlendingBiomeHeight => 0x3f,
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

    pub fn deserialize<R>(reader: &mut Cursor<R>) -> Result<Key>
    where
        Cursor<R>: Seek + Read,
    {
        let start_position = reader.position();
        let len = reader.stream_len_ext()?;

        let x = reader.read_i32::<LittleEndian>()?;
        let z = reader.read_i32::<LittleEndian>()?;

        let chunk = ChunkPosition(x, z);
        let dimension = if len > 10 {
            Dimension::from(reader.read_u32::<LittleEndian>()? as i32)
        } else {
            Dimension::Overworld
        };

        let key_var = reader.read_u8()?;
        let data = match key_var {
            0x2f => KeyVariant::SubChunk {
                index: reader.read_i8()?,
            },
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
            0x3d => KeyVariant::MetadataHash,
            0x3e => KeyVariant::GeneratedBlending,
            0x3f => KeyVariant::BlendingBiomeHeight,
            0x40 => KeyVariant::BlendingData,
            0x41 => KeyVariant::ActorDigestVersion,
            0x76 => KeyVariant::LegacyVersion,
            0x77 => KeyVariant::AabbVolumes,
            _ => {
                // Check whether this was one of the strings
                reader.set_position(start_position);

                let mut string = String::with_capacity(len as usize);
                reader.read_to_string(&mut string)?;

                if string == LOCAL_PLAYER {
                    KeyVariant::LocalPlayer
                } else {
                    return Err(Error::Invalid("invalid leveldb database key type"));
                }
            }
        };

        let key = Self {
            chunk,
            dimension,
            data,
        };

        Ok(key)
    }
}
