use std::{collections::HashMap, hash::DefaultHasher, iter::FusedIterator, ops::Index};

use serde::{Deserialize, Serialize};

use crate::error::{Result, LevelError};

pub enum SubChunkVersion {
    Legacy = 1,
    Limited = 8,
    Limitless = 9
}

impl TryFrom<u8> for SubChunkVersion {
    type Error = LevelError;

    fn try_from(v: u8) -> Result<Self> {
        Ok(match v {
            1 => Self::Legacy,
            8 => Self::Limited,
            9 => Self::Limitless,
            _ => return Err(LevelError::Invalid)
        })
    }
}

mod block_version {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn deserialize<'de, D>(de: D) -> Result<Option<[u8; 4]>, D::Error> 
    where
        D: Deserializer<'de>
    {
        let word = Option::<i32>::deserialize(de)?;
        Ok(word.map(i32::to_be_bytes))
    }

    #[allow(clippy::trivially_copy_pass_by_ref)] // Required by serde
    pub fn serialize<S>(v: &Option<[u8; 4]>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match v {
            Some(b) => ser.serialize_i32(i32::from_be_bytes(*b)),
            None => ser.serialize_none()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename = "")]
pub struct BlockEntry {
    pub name: String,
    #[serde(with = "block_version")]
    pub version: Option<[u8; 4]>,
    pub states: HashMap<String, nbtx::Value>
}

impl BlockEntry {
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        hasher.write(self.name.as_bytes());
        for (k, v) in &self.states {
            hasher.write(k.as_bytes());
            v.hash(&mut hasher);
        }

        hasher.finish();
    }
}

#[doc(alias = "storage record")]
#[derive(Debug, Clone, PartialEq)]
pub struct SubStorage {
    pub indices: Box<[u16; 4096]>,
    pub palette: Vec<BlockEntry>
}

impl SubStorage {
    pub fn iter(&self) -> LayerIter {
        LayerIter::from(self)
    }

    pub fn get<V>(&self, pos: V) -> Option<&BlockEntry> 
    {
        todo!();
    }

    pub fn palette(&self) -> &[BlockEntry] {
        &self.palette
    }

    pub fn palette_mut(&mut self) -> &mut [BlockEntry] {
        &mut self.palette
    }

    pub fn inidces(&self) -> &[u16; 4096] {
        &self.indices
    }

    pub fn indices_mut(&mut self) -> &mut [u16; 4096] {
        &mut self.indices
    }

    pub fn into_inner(self) -> Box<[u16; 4096]> {
        self.indices
    }

    pub fn empty() -> Self {
        Self {
            indices: Box::new([0; 4096]),
            palette: Vec::new()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.palette.is_empty()
    }
}

impl Default for SubStorage {
    fn default() -> Self {
        Self {
            indices: Box::new([0; 4096]),
            palette: Vec::new()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubChunk {
    pub version: SubChunkVersion,
    pub index: i8,
    pub layers: Vec<SubStorage>
}

impl SubChunk {
    pub fn empty(index: i8) -> Self {
        Self {
            index,
            layers: vec![SubStorage::empty()],
            version: SubChunkVersion::Limitless
        }
    }

    pub fn is_empty(&self) -> bool {
        self.layers.is_empty() || self.layers[0].is_empty()
    }

    pub fn version(&self) -> SubChunkVersion {
        self.version
    }

    pub fn index(&self) -> i8 {
        self.index
    }

    pub fn layers(&self) -> &[SubStorage] {
        &self.layers
    }

    pub fn layer(&self, index: usize) -> Option<&SubStorage> {
        self.layers.get(index)
    }

    pub fn layer_mut(&mut self, index: usize) -> Option<&mut SubStorage> {
        self.layers.get_mut(index)
    }
}

pub struct LayerIter<'s> {
    indices: std::slice::Iter<'s, u16>,
    palette: &'s [BlockEntry]
}

impl<'s> From<&'s SubStorage> for LayerIter<'s> {
    fn from(v: &'s SubStorage) -> Self {
        Self {
            indices: v.indices.iter(),
            palette: &v.palette
        }
    }
}

impl<'s> Iterator for LayerIter<'s> {
    type Item = &'s BlockEntry;

    fn next(&mut self) -> Option<&'s BlockEntry> {
        let idx = self.indices.next()?;
        self.palette.get(*idx as usize)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for LayerIter<'_> {
    fn len(&self) -> usize {
        self.indices.len()
    }
}

impl FusedIterator for LayerIter<'_> {}