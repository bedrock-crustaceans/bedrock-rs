//! Produces the normalized text dump this harness diffs against an oracle's
//! own dump of the same world. Every line is independent and the whole
//! output is sorted before being written, so a line-oriented diff is a
//! structural diff: two dumps agree exactly when their sorted line sets are
//! equal.
//!
//! Line shapes (tab-separated fields, one record per line):
//!
//! ```text
//! BLOCK        <dim> <cx> <cz> <x> <y> <z> <layer> <block-key>
//! PALETTE      <dim> <cx> <cz> <subchunk-y> <layer> <block-key>
//! BIOME2D      <dim> <cx> <cz> <x> <z> <biome-id>
//! BIOME3D      <dim> <cx> <cz> <subchunk-y> <fragment-kind> <detail>
//! BLOCKENTITY  <dim> <cx> <cz> <id> <x> <y> <z>
//! ENTITY       <dim> <cx> <cz> <identifier> <x> <y> <z>
//! ```
//!
//! `<block-key>` is `name;prop=val,prop=val` (properties sorted by name) for
//! a modern entry, `name#val=N` for a pre-flattening legacy entry, and
//! `numeric#id=N,data=N` for a pre-palette entry with no name at all -- see
//! the crate's `PaletteEntry` for what each form means.
//!
//! # Known coverage gap
//!
//! A paletted 3D biome fragment's per-voxel indices are not reachable
//! through the public API this harness is restricted to (only the
//! fragment's palette contents are, via `BiomeArray::palette`), so
//! `BIOME3D` reports each fragment's *shape* (inherited / single id /
//! palette contents) rather than an exploded per-voxel id. `BIOME2D`,
//! which does carry a real per-column id, has no such gap.

use std::collections::HashSet;
use std::fmt::Write as _;

use bedrock_level::actor::decode_legacy_entities;
use bedrock_level::biome::BiomeEncoding;
use bedrock_level::key::KeyVariant;
use bedrock_level::level::Level;
use bedrock_level::level_chunk::{Chunk, ChunkBiomes};
use bedrock_level::subchunk::{Layer, PaletteEntry};
use bedrock_level::types::BlockPosition;

/// Walks every dimension and chunk position this level's index knows about,
/// decoding chunks through the ordinary level layer, and returns the sorted
/// dump lines.
///
/// A chunk that fails to decode is not silently skipped: it is reported as
/// its own `CHUNKERROR` line so a decode regression shows up as a diff
/// rather than a quietly shrunken dump.
pub fn dump_level(level: &Level) -> Vec<String> {
    let mut lines = Vec::new();

    for &dimension in level.dimensions() {
        for position in level.chunk_positions(dimension) {
            let dim_id = i32::from(dimension);
            match level.chunk(dimension, position) {
                Ok(Some(chunk)) => dump_chunk(&chunk, dim_id, &mut lines),
                Ok(None) => {}
                Err(err) => {
                    lines.push(format!(
                        "CHUNKERROR\t{dim_id}\t{}\t{}\t{err}",
                        position.0, position.1
                    ));
                }
            }
        }
    }

    lines.sort();
    lines.dedup();
    lines
}

fn dump_chunk(chunk: &Chunk, dim_id: i32, out: &mut Vec<String>) {
    let cx = chunk.position().0;
    let cz = chunk.position().1;

    dump_terrain(chunk, dim_id, cx, cz, out);
    dump_biomes(chunk, dim_id, cx, cz, out);
    dump_block_entities(chunk, dim_id, cx, cz, out);
    dump_entities(chunk, dim_id, cx, cz, out);
}

/// A parsed `BLOCK` line, for the harness's position-paired
/// expected-difference checks in `run.rs`. Borrows `content` out of the
/// line it was parsed from rather than cloning it.
pub struct BlockLine<'a> {
    pub dim: i32,
    pub cx: i32,
    pub cz: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub layer: usize,
    pub content: &'a str,
}

/// Parses a `BLOCK` line back into its fields, or `None` if `line` is not
/// one (including if it is a `PALETTE`/`CHUNKERROR`/... line instead).
pub fn parse_block_line(line: &str) -> Option<BlockLine<'_>> {
    let mut fields = line.split('\t');
    if fields.next()? != "BLOCK" {
        return None;
    }
    Some(BlockLine {
        dim: fields.next()?.parse().ok()?,
        cx: fields.next()?.parse().ok()?,
        cz: fields.next()?.parse().ok()?,
        x: fields.next()?.parse().ok()?,
        y: fields.next()?.parse().ok()?,
        z: fields.next()?.parse().ok()?,
        layer: fields.next()?.parse().ok()?,
        content: fields.next()?,
    })
}

/// A parsed `PALETTE` line; see [`parse_block_line`].
pub struct PaletteLine<'a> {
    pub dim: i32,
    pub cx: i32,
    pub cz: i32,
    pub subchunk_y: i32,
    pub layer: usize,
    pub content: &'a str,
}

/// Parses a `PALETTE` line back into its fields, or `None` if `line` is
/// not one.
pub fn parse_palette_line(line: &str) -> Option<PaletteLine<'_>> {
    let mut fields = line.split('\t');
    if fields.next()? != "PALETTE" {
        return None;
    }
    Some(PaletteLine {
        dim: fields.next()?.parse().ok()?,
        cx: fields.next()?.parse().ok()?,
        cz: fields.next()?.parse().ok()?,
        subchunk_y: fields.next()?.parse().ok()?,
        layer: fields.next()?.parse().ok()?,
        content: fields.next()?,
    })
}

/// The position key a BLOCK line's first seven fields spell out, shared
/// between this module and the harness's position-paired
/// expected-difference checks (`run.rs`) so the two never drift out of
/// sync with each other's idea of "the same position."
pub fn position_key(dim_id: i32, cx: i32, cz: i32, x: i32, y: i32, z: i32, layer: usize) -> String {
    format!("{dim_id}\t{cx}\t{cz}\t{x}\t{y}\t{z}\t{layer}")
}

/// A position-keyed `BlockDef::version` lookup, restricted to modern
/// entries whose block name is in `names`. **Not** part of the main
/// comparison dump: `block_key` deliberately excludes version (see its
/// doc comment -- diffing on it collapses agreement to near zero, because
/// the java oracle restamps every block's version on any round trip
/// regardless of content). This exists only so a narrowly-scoped
/// expected-difference check can verify a *specific* version transition
/// it claims, for the handful of block names that check cares about,
/// without paying that cost on the 10-million-line comparison.
pub fn state_versions(
    level: &Level,
    names: &[&str],
) -> std::collections::BTreeMap<String, Option<i32>> {
    let mut out = std::collections::BTreeMap::new();

    for &dimension in level.dimensions() {
        for position in level.chunk_positions(dimension) {
            let dim_id = i32::from(dimension);
            let Ok(Some(chunk)) = level.chunk(dimension, position) else {
                continue;
            };
            let cx = chunk.position().0;
            let cz = chunk.position().1;

            for subchunk_y in -32i8..=32 {
                for layer_idx in 0..2usize {
                    let Some(layer) = chunk.layer(subchunk_y, layer_idx) else {
                        continue;
                    };
                    for x in 0..16u8 {
                        for z in 0..16u8 {
                            for local_y in 0..16u8 {
                                let Some(PaletteEntry::Modern(block)) =
                                    layer.get(BlockPosition(x, local_y, z))
                                else {
                                    continue;
                                };
                                if !names.contains(&block.name.as_str()) {
                                    continue;
                                }
                                let y = i32::from(subchunk_y) * 16 + i32::from(local_y);
                                let key = position_key(
                                    dim_id,
                                    cx,
                                    cz,
                                    i32::from(x),
                                    y,
                                    i32::from(z),
                                    layer_idx,
                                );
                                out.insert(key, block.version);
                            }
                        }
                    }
                }
            }
        }
    }

    out
}

fn dump_terrain(chunk: &Chunk, dim_id: i32, cx: i32, cz: i32, out: &mut Vec<String>) {
    // Every chunk version this crate decodes uses one of two terrain
    // shapes (see `Terrain`); both expose the same per-subchunk `layer`
    // accessor, so a single sweep over a plausible subchunk-index range
    // covers both without matching on which shape this chunk has.
    for subchunk_y in -32i8..=32 {
        for layer_idx in 0..2usize {
            let Some(layer) = chunk.layer(subchunk_y, layer_idx) else {
                continue;
            };

            // Emitted as a sorted set of content keys with no index at
            // all -- deliberately, and a plain sort was not enough on its
            // own. Two encoders can agree on every block's content while
            // building a subchunk's palette in a different order (this
            // crate's own writer never promises to preserve the order it
            // read a palette in either), which a content sort does fix.
            // But a run of this harness against the java oracle also
            // found it compacting a subchunk's palette down to only the
            // entries its index array still references, dropping orphaned
            // ones this crate preserves as read -- and dropping even one
            // entry shifts every alphabetically-later index for the rest
            // of the palette, which a sorted-but-still-indexed line would
            // still report as a false divergence for every one of those
            // entries instead of the one that is actually missing. An
            // index carries no comparison value a BLOCK line does not
            // already carry more precisely, so it is dropped rather than
            // patched around a second time.
            let mut keys: Vec<String> = layer.palette().iter().map(block_key).collect();
            keys.sort();
            keys.dedup();
            for key in &keys {
                out.push(format!(
                    "PALETTE\t{dim_id}\t{cx}\t{cz}\t{subchunk_y}\t{layer_idx}\t{key}"
                ));
            }

            dump_layer_positions(layer, dim_id, cx, cz, subchunk_y, layer_idx, out);
        }
    }
}

fn dump_layer_positions(
    layer: &Layer,
    dim_id: i32,
    cx: i32,
    cz: i32,
    subchunk_y: i8,
    layer_idx: usize,
    out: &mut Vec<String>,
) {
    for x in 0..16u8 {
        for z in 0..16u8 {
            for local_y in 0..16u8 {
                let Some(entry) = layer.get(BlockPosition(x, local_y, z)) else {
                    continue;
                };
                let y = i32::from(subchunk_y) * 16 + i32::from(local_y);
                out.push(format!(
                    "BLOCK\t{dim_id}\t{cx}\t{cz}\t{x}\t{y}\t{z}\t{layer_idx}\t{}",
                    block_key(entry)
                ));
            }
        }
    }
}

/// Renders a palette entry to the normalized key described in this module's
/// docs. Properties are sorted by name so key order (which `BlockDef`
/// otherwise preserves faithfully, see its doc comment) never causes a
/// spurious diff between two structurally identical states.
///
/// Deliberately excludes `BlockDef::version`: a run of this harness against
/// a java-oracle round trip confirmed the oracle's writer restamps every
/// block's version field on any conversion, `INPUT`-to-`INPUT` included
/// (observed: every block in a 1.16 fixture moved from `17825808` to
/// `17876992`, content-identical blocks like plain `stone` included).
/// Diffing on it would turn every single line into a divergence and bury
/// the comparison this harness exists to make; the version is real
/// evidence for specific, root-caused divergences (see
/// `run.rs`'s expected-difference reasons) but not part of a block's
/// identity for this key.
fn block_key(entry: &PaletteEntry) -> String {
    match entry {
        PaletteEntry::Modern(block) => {
            let mut props: Vec<(&str, String)> = block
                .states
                .iter()
                .map(|(k, v)| (k.as_str(), format!("{v:?}")))
                .collect();
            props.sort_by(|a, b| a.0.cmp(b.0));
            let mut key = block.name.clone();
            if !props.is_empty() {
                key.push(';');
                for (i, (k, v)) in props.iter().enumerate() {
                    if i > 0 {
                        key.push(',');
                    }
                    let _ = write!(key, "{k}={v}");
                }
            }
            key
        }
        PaletteEntry::Legacy { name, val } => format!("{name}#val={val}"),
        PaletteEntry::Numeric { id, data } => format!("numeric#id={id},data={data}"),
    }
}

fn dump_biomes(chunk: &Chunk, dim_id: i32, cx: i32, cz: i32, out: &mut Vec<String>) {
    match chunk.biomes() {
        Some(ChunkBiomes::Columns(heightmap)) => {
            let biomes = heightmap.biomes();
            for x in 0..16usize {
                for z in 0..16usize {
                    if let Some(id) = biomes.id_at(x, z) {
                        out.push(format!("BIOME2D\t{dim_id}\t{cx}\t{cz}\t{x}\t{z}\t{id}"));
                    }
                }
            }
        }
        Some(ChunkBiomes::Volumes(volumes)) => {
            for (i, fragment) in volumes.fragments().iter().enumerate() {
                let subchunk_y = i; // relative fragment order, bottom to top
                let (kind, detail) = match fragment {
                    BiomeEncoding::Inherit => ("inherit", String::new()),
                    BiomeEncoding::Single(id) => ("single", id.to_string()),
                    BiomeEncoding::Palette(array) => {
                        let mut ids: Vec<u32> = array.palette().to_vec();
                        ids.sort_unstable();
                        (
                            "palette",
                            ids.iter().map(u32::to_string).collect::<Vec<_>>().join(","),
                        )
                    }
                };
                out.push(format!(
                    "BIOME3D\t{dim_id}\t{cx}\t{cz}\t{subchunk_y}\t{kind}\t{detail}"
                ));
            }
        }
        None => {}
    }
}

fn dump_block_entities(chunk: &Chunk, dim_id: i32, cx: i32, cz: i32, out: &mut Vec<String>) {
    let Some(bytes) = chunk.unmodelled_records().get(KeyVariant::BlockEntity) else {
        return;
    };
    let Ok(values) = decode_legacy_entities(bytes) else {
        out.push(format!("BLOCKENTITYERROR\t{dim_id}\t{cx}\t{cz}"));
        return;
    };
    for value in values {
        let id = nbt_string(&value, &["id"]).unwrap_or_else(|| "?".to_string());
        let x = nbt_int(&value, &["x"]).unwrap_or_default();
        let y = nbt_int(&value, &["y"]).unwrap_or_default();
        let z = nbt_int(&value, &["z"]).unwrap_or_default();
        out.push(format!(
            "BLOCKENTITY\t{dim_id}\t{cx}\t{cz}\t{id}\t{x}\t{y}\t{z}"
        ));
    }
}

fn dump_entities(chunk: &Chunk, dim_id: i32, cx: i32, cz: i32, out: &mut Vec<String>) {
    let mut seen = HashSet::new();
    for record in &chunk.actors().resolved {
        dump_one_entity(&record.value, dim_id, cx, cz, &mut seen, out);
    }
    for value in &chunk.actors().legacy {
        dump_one_entity(value, dim_id, cx, cz, &mut seen, out);
    }
}

fn dump_one_entity(
    value: &nbtx::Value,
    dim_id: i32,
    cx: i32,
    cz: i32,
    seen: &mut HashSet<String>,
    out: &mut Vec<String>,
) {
    let identifier = nbt_string(value, &["identifier", "id"]).unwrap_or_else(|| "?".to_string());
    let (x, y, z) = nbt_pos(value).unwrap_or((0.0, 0.0, 0.0));
    let line = format!("ENTITY\t{dim_id}\t{cx}\t{cz}\t{identifier}\t{x}\t{y}\t{z}");
    if seen.insert(line.clone()) {
        out.push(line);
    }
}

fn nbt_string(value: &nbtx::Value, keys: &[&str]) -> Option<String> {
    let nbtx::Value::Compound(map) = value else {
        return None;
    };
    for key in keys {
        if let Some(nbtx::Value::String(s)) = map.get(key.as_bytes() as &[u8]) {
            return Some(s.to_string());
        }
    }
    None
}

fn nbt_int(value: &nbtx::Value, keys: &[&str]) -> Option<i64> {
    let nbtx::Value::Compound(map) = value else {
        return None;
    };
    for key in keys {
        match map.get(key.as_bytes() as &[u8]) {
            Some(nbtx::Value::Int(v)) => return Some(i64::from(*v)),
            Some(nbtx::Value::Byte(v)) => return Some(i64::from(*v)),
            Some(nbtx::Value::Short(v)) => return Some(i64::from(*v)),
            Some(nbtx::Value::Long(v)) => return Some(*v),
            _ => {}
        }
    }
    None
}

/// Bedrock entities store their position as a `Pos` list of three floats.
fn nbt_pos(value: &nbtx::Value) -> Option<(f64, f64, f64)> {
    let nbtx::Value::Compound(map) = value else {
        return None;
    };
    let nbtx::Value::List(list) = map.get(b"Pos" as &[u8])? else {
        return None;
    };
    if list.len() != 3 {
        return None;
    }
    let get = |v: &nbtx::Value| -> Option<f64> {
        match v {
            nbtx::Value::Float(f) => Some(f64::from(*f)),
            nbtx::Value::Double(d) => Some(*d),
            _ => None,
        }
    };
    Some((
        get(&list.get(0)?)?,
        get(&list.get(1)?)?,
        get(&list.get(2)?)?,
    ))
}
