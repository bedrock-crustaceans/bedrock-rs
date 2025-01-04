#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KeyTypeTag {
    Data3D = 43,  // 2B  "+"  Heightmap (256x2 bytes)
    Version = 44, // 2C  ","  1 byte version
    Data2D = 45,  // 2D  "-"  Heightmap (256x2 bytes)

    Data2DLegacy = 46, // 2E  "."  Heightmap (256x2 bytes)
    // 2D biomes (256x4 bytes)
    // Each entry of the biome array contains a biome ID in the first byte, and the final 3 bytes are red/green/blue respectively. No longer written since v1.0.0.
    SubChunkPrefix = 47, // 2F  "/"  Subchunk version (1 byte), version-dependent data
    // Terrain for a 16×16×16 subchunk
    LegacyTerrain = 48, // 30  "0"  Block IDs (32768 bytes)
    // Block meta (32768 nibbles)
    // Block sky light (32768 nibbles)
    // Block light (32768 nibbles)
    // Heightmap (256x1 bytes)
    // 2D biomes (256x4 bytes)
    // Data ordered in XZY order, unlike Java.
    // No longer written since v1.0.0.
    // Biomes are IDs plus RGB colours similar to Data2DLegacy.
    BlockEntity = 49, // 31  "1"  List of NBT compound roots
    // Block entity data (little-endian NBT)
    Entity = 50, // 32  "2"  List of NBT compound roots
    // Entity data (little-endian NBT)
    PendingTicks = 51, // 33  "3"  List of NBT compound roots
    // Pending tick data (little-endian NBT)
    LegacyBlockExtraData = 52, // 34  "4"  Entry count (4 bytes)
    // Entries: Key (4 bytes), Value (2 bytes).
    // Array of blocks that appear in the same place as other blocks.
    // Used for grass appearing inside snow layers prior to v1.2.13.
    // No longer written as of v1.2.13.
    BiomeState = 53, // 35  "5"  Biome palette with 25 entries. Biome IDs written as integers.

    FinalizedState = 54, // 36  "6"  4 bytes (32-bit little endian integer)

    ConversionData = 55, // 37  "7"  No longer used

    BorderBlocks = 56, // 38  "8"  Education Edition feature

    HardcodedSpawners = 57, // 39  "9"  Bounding boxes for structure spawns stored in binary format

    RandomTicks = 58, // 3A  ":"  List of NBT compound roots
    // Random tick data (little-endian NBT)
    Checksums = 59, // 3B  ";"  xxHash checksums of other chunk records. No longer written as of v1.18.0.

    MetaDataHash = 61, // 3D  "="  Metadata hash

    GeneratedPreCavesAndCliffsBlending = 62, // 3E  ">"  Not used

    BlendingBiomeHeight = 63, // 3F  "?"  Not used

    BlendingData = 64, // 40  "@"  Blending data

    ActorDigestVersion = 65, // 41  "A"  Actor digest version

    LegacyVersion = 118, // 76  "v"  1 byte; moved to Version in v1.16.100
}

impl KeyTypeTag {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            43 => Some(KeyTypeTag::Data3D),
            44 => Some(KeyTypeTag::Version),
            45 => Some(KeyTypeTag::Data2D),
            46 => Some(KeyTypeTag::Data2DLegacy),
            47 => Some(KeyTypeTag::SubChunkPrefix),
            48 => Some(KeyTypeTag::LegacyTerrain),
            49 => Some(KeyTypeTag::BlockEntity),
            50 => Some(KeyTypeTag::Entity),
            51 => Some(KeyTypeTag::PendingTicks),
            52 => Some(KeyTypeTag::LegacyBlockExtraData),
            53 => Some(KeyTypeTag::BiomeState),
            54 => Some(KeyTypeTag::FinalizedState),
            55 => Some(KeyTypeTag::ConversionData),
            56 => Some(KeyTypeTag::BorderBlocks),
            57 => Some(KeyTypeTag::HardcodedSpawners),
            58 => Some(KeyTypeTag::RandomTicks),
            59 => Some(KeyTypeTag::Checksums),
            61 => Some(KeyTypeTag::MetaDataHash),
            62 => Some(KeyTypeTag::GeneratedPreCavesAndCliffsBlending),
            63 => Some(KeyTypeTag::BlendingBiomeHeight),
            64 => Some(KeyTypeTag::BlendingData),
            65 => Some(KeyTypeTag::ActorDigestVersion),
            118 => Some(KeyTypeTag::LegacyVersion),
            _ => None,
        }
    }
    pub fn to_byte(&self) -> u8 {
        *self as u8
    }
}
