#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockPosition(pub u8, pub u8, pub u8);

impl From<(u8, u8, u8)> for BlockPosition {
    fn from((x, y, z): (u8, u8, u8)) -> Self {
        Self(x, y, z)
    }
}

impl From<[u8; 3]> for BlockPosition {
    fn from([x, y, z]: [u8; 3]) -> Self {
        Self(x, y, z)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubChunkPosition(pub i32, pub i8, pub i32);

impl From<(i32, i8, i32)> for SubChunkPosition {
    fn from((x, y, z): (i32, i8, i32)) -> Self {
        Self(x, y, z)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkPosition(pub i32, pub i32);

impl From<(i32, i32)> for ChunkPosition {
    fn from((x, z): (i32, i32)) -> Self {
        Self(x, z)
    }
}
