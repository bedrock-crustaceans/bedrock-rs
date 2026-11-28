//! Building subchunks, layers and biome records from their parts and taking them apart
//! again, for code that keeps blocks in structures of its own.

use std::io::Cursor;

use bedrock_level::Greedy;
use bedrock_level::biome::{BiomeArray, BiomeEncoding, Biomes3d};
use bedrock_level::subchunk::{BlockDef, Layer, PaletteEntry, SubChunk, SubChunkVersion};
use indexmap::IndexMap;

fn block(name: &str) -> PaletteEntry {
    PaletteEntry::Modern(BlockDef {
        name: name.to_owned(),
        version: Some(18_168_865),
        states: IndexMap::default(),
    })
}

#[test]
fn a_subchunk_built_from_parts_reads_back_the_same() {
    let mut indices = Box::new([0u16; 4096]);
    indices[5] = 1;
    let layer = Layer::from_parts(
        vec![block("minecraft:air"), block("minecraft:stone")],
        indices.clone(),
    )
    .unwrap();
    let chunk = SubChunk::new(SubChunkVersion::Limitless, -4, vec![layer]).unwrap();

    let mut out = Cursor::new(Vec::new());
    chunk.to_disk::<Greedy, _>(&mut out).unwrap();
    let read =
        SubChunk::from_disk::<Greedy, _>(&mut Cursor::new(out.into_inner().as_slice())).unwrap();
    assert_eq!(read, chunk);

    assert_eq!(read.index(), -4);
    assert_eq!(read.layers().len(), 1);
    let (palette, array) = read.into_layers().remove(0).into_parts();
    assert_eq!(palette[1].name(), Some("minecraft:stone"));
    assert_eq!(array.into_indices(), indices);
}

#[test]
fn a_layer_rejects_a_bad_palette_and_indices_past_its_end() {
    assert!(Layer::from_parts(Vec::new(), Box::new([0; 4096])).is_err());
    let numeric = PaletteEntry::Numeric { id: 1, data: 0 };
    assert!(Layer::from_parts(vec![numeric], Box::new([0; 4096])).is_err());
    let mut indices = Box::new([0u16; 4096]);
    indices[0] = 1;
    assert!(Layer::from_parts(vec![block("minecraft:air")], indices).is_err());
}

#[test]
fn a_subchunk_rejects_a_layer_count_its_version_cannot_store() {
    let layer = || Layer::from_parts(vec![block("minecraft:air")], Box::new([0; 4096])).unwrap();
    assert!(SubChunk::new(SubChunkVersion::Legacy, 0, vec![layer(), layer()]).is_err());
    assert!(SubChunk::new(SubChunkVersion::Legacy, 0, Vec::new()).is_err());
    assert!(SubChunk::new(SubChunkVersion::Legacy, 0, vec![layer()]).is_ok());
    assert!(SubChunk::new(SubChunkVersion::Limitless, 0, vec![layer(); 256]).is_err());
}

#[test]
fn biomes_built_from_parts_read_back_the_same() {
    let mut indices = Box::new([0u16; 4096]);
    indices[4095] = 1;
    let array = BiomeArray::from_parts(vec![1, 7], indices.clone()).unwrap();
    let biomes = Biomes3d::new(
        Box::new([3; 256]),
        smallvec::smallvec![
            BiomeEncoding::Palette(array),
            BiomeEncoding::Single(2),
            BiomeEncoding::Inherit,
        ],
    )
    .unwrap();

    let mut out = Cursor::new(Vec::new());
    biomes.to_disk(&mut out).unwrap();
    let read =
        Biomes3d::from_disk::<Greedy, _>(&mut Cursor::new(out.into_inner().as_slice())).unwrap();
    assert_eq!(read, biomes);

    let (heights, mut fragments) = read.into_parts();
    assert_eq!(heights[0], 3);
    assert_eq!(fragments[1], BiomeEncoding::Single(2));
    let BiomeEncoding::Palette(array) = fragments.remove(0) else {
        panic!("first fragment should be paletted");
    };
    let (palette, array) = array.into_parts();
    assert_eq!(palette, [1, 7]);
    assert_eq!(array.into_indices(), indices);
}

#[test]
fn a_biome_storage_rejects_indices_past_its_palette() {
    assert!(BiomeArray::from_parts(Vec::new(), Box::new([0; 4096])).is_err());
    assert!(BiomeArray::from_parts(vec![1], Box::new([1; 4096])).is_err());
}
