//! `\x7FMCB` binary documents → JSON text.
//!
//! An MCB document is a baked "Minecraft document" `{ format_version, <payload key> }`. Its
//! output here is JSON **text**, written by this module (not `facet_json::to_string`), so it
//! takes the same facet-json path as a `.json` file and `RawJson` isolation works. The writer
//! keeps the stored types in the lexical form: a float always gets a fraction or an exponent
//! and an integer never does, so the int/float tag survives the parse.
//!
//! The header is read (magic, format version, payload key). The body is typed per payload: it
//! has no type tags, its fields are in the order of the payload's schema, and a component is
//! keyed by the FNV-1a 32 hash of its name. Each layout is that of the game's own
//! documents, checked against their JSON twins (the loose file of the same path, which each
//! pack ships beside its archive); there is one module per payload. A body is decoded only
//! where those pairs show what every byte is; any other body is `UnsupportedBody`, which the pack layer turns into
//! `unsupported_file_format`.
//!
//! The decoder is picked by the table `ROWS`: payload key, the versions vanilla bakes it at,
//! decoder. It is not a per-family hook (`Family::MCB`, as the rework plan had it), on purpose:
//! a document is decoded before it is routed, by its own header key (the stack's read path
//! has no collection at hand, the writer's copy of a pack has no family at all), a decoder
//! yields an `McbValue` and needs its baked versions, which a family's version chain does not
//! carry, and a table assembled from the enabled families would stop decoding the documents
//! of a family whose feature is off. Splitting the rows out to the families is behaviour-bearing
//! for that last reason and is not done here.

mod camera;
mod catalog;
mod item;
mod particle;
mod poi;
mod reader;
mod trade;
mod voxel;

use std::fmt::Write as _;

use super::magic::MCB_MAGIC;
use super::{McbFault, StorageError};

/// The header of an MCB document: `\x7FMCB`, u16 major, u16 minor, u32 patch (the document's
/// `format_version`), then the payload key with a one-byte length.
// The layout is the same in every MCB document the game ships (565 of them); `1.26.30` with
// `minecraft:item` is the form its item files carry. The payload key's length is one
// byte.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct McbHeader<'a> {
    /// `format_version` major.
    pub major: u16,
    /// `format_version` minor.
    pub minor: u16,
    /// `format_version` patch.
    pub patch: u32,
    /// The document's payload key (`minecraft:item`, `particle_effect`, …).
    pub payload_key: &'a str,
    /// Where the body starts.
    pub body_offset: usize,
}

impl<'a> McbHeader<'a> {
    /// Reads the header of `bytes`; `Mcb { Header }` when the magic is missing or the header is
    /// cut short or its key is not UTF-8.
    pub fn parse(bytes: &'a [u8]) -> Result<McbHeader<'a>, StorageError> {
        const HEADER: StorageError = StorageError::Mcb {
            fault: McbFault::Header,
        };
        if !bytes.starts_with(&MCB_MAGIC) {
            return Err(HEADER);
        }
        let fixed = bytes.get(4..13).ok_or(HEADER)?;
        let major = u16::from_le_bytes([fixed[0], fixed[1]]);
        let minor = u16::from_le_bytes([fixed[2], fixed[3]]);
        let patch = u32::from_le_bytes([fixed[4], fixed[5], fixed[6], fixed[7]]);
        let key_len = usize::from(fixed[8]);
        let key = bytes.get(13..13 + key_len).ok_or(HEADER)?;
        let payload_key = std::str::from_utf8(key).map_err(|_| HEADER)?;
        Ok(McbHeader {
            major,
            minor,
            patch,
            payload_key,
            body_offset: 13 + key_len,
        })
    }

    /// The `format_version` text, `major.minor.patch`.
    pub fn format_version(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// A decoded MCB value, in the types an MCB body stores.
// No known layout holds a null or a 64-bit float; the writer is ready for both.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum McbValue {
    /// `null`.
    Null,
    /// A boolean.
    Bool(bool),
    /// A signed integer.
    Int(i64),
    /// An unsigned integer.
    UInt(u64),
    /// A 32-bit float, written in its shortest form (`0.1`, not `0.100000001…`).
    F32(f32),
    /// A 64-bit float.
    F64(f64),
    /// A string.
    String(Box<str>),
    /// An array.
    Array(Vec<McbValue>),
    /// An object, members in order.
    Object(Vec<(Box<str>, McbValue)>),
}

/// The JSON text of an MCB document: `{"format_version":"<version>","<payload key>":<body>}`.
/// Fails with `Mcb { Header }` for a bad header, `Mcb { UnsupportedBody }` for a body that is
/// not one of the known layouts, and `Mcb { NonFiniteFloat }` for a float JSON cannot write.
pub fn to_json_text(bytes: &[u8]) -> Result<String, StorageError> {
    let header = McbHeader::parse(bytes)?;
    let body = bytes.get(header.body_offset..).ok_or(reader::UNSUPPORTED)?;
    let body = decode_body(&header, body)?;
    write_json(&McbValue::Object(vec![
        (
            "format_version".into(),
            McbValue::String(header.format_version().into()),
        ),
        (header.payload_key.into(), body),
    ]))
}

/// A body decoder: the bytes after the header to the value of the payload key.
type Decoder = fn(&[u8]) -> Result<McbValue, StorageError>;

/// One payload the game bakes: its key, the format versions vanilla bakes it at (one layout
/// for all of them), and the decoder of that layout.
struct McbRow {
    payload_key: &'static str,
    versions: &'static [(u16, u16, u32)],
    decode: Decoder,
}

/// Every payload with a known layout.
const ROWS: &[McbRow] = &[
    McbRow {
        payload_key: "minecraft:voxel_shape",
        versions: &[(1, 21, 110)],
        decode: voxel::decode,
    },
    McbRow {
        payload_key: "tiers",
        versions: &[(1, 26, 10)],
        decode: trade::decode,
    },
    // Items are baked at 1.26.30, 1.26.40 and 1.26.50 with one layout.
    McbRow {
        payload_key: "minecraft:item",
        versions: &[(1, 26, 30), (1, 26, 40), (1, 26, 50)],
        decode: item::decode,
    },
    McbRow {
        payload_key: "minecraft:camera_entity",
        versions: &[(1, 26, 10)],
        decode: camera::decode,
    },
    // The Drop 4 experiment's freezing effects are baked at 1.26.50 under the one effect document.
    McbRow {
        payload_key: "particle_effect",
        versions: &[(1, 20, 80), (1, 26, 10), (1, 26, 30), (1, 26, 50)],
        decode: particle::decode,
    },
    // POI documents are baked at 1.26.40.
    McbRow {
        payload_key: "minecraft:poi_block",
        versions: &[(1, 26, 40)],
        decode: poi::decode_type,
    },
    McbRow {
        payload_key: "minecraft:poi_tag",
        versions: &[(1, 26, 40)],
        decode: poi::decode_tag,
    },
    // The crafting catalogue is baked at several versions; the older ones share the 1.21.60 layout.
    McbRow {
        payload_key: "minecraft:crafting_items_catalog",
        versions: &[
            (1, 21, 60),
            (1, 21, 70),
            (1, 21, 110),
            (1, 26, 10),
            (1, 26, 30),
            (1, 26, 40),
            (1, 26, 50),
        ],
        decode: catalog::decode,
    },
];

/// The decoder of `payload_key` at `version`, if vanilla bakes it there.
fn decoder_for(payload_key: &str, version: (u16, u16, u32)) -> Option<Decoder> {
    ROWS.iter()
        .find(|row| row.payload_key == payload_key && row.versions.contains(&version))
        .map(|row| row.decode)
}

/// The body of a document with `header`: the value of its payload key.
///
/// A layout is known for a payload at the format versions vanilla bakes it at (the header's
/// version is the document's `format_version`, and a body does not say which layout it is
/// in), so any other payload or version is unsupported.
fn decode_body(header: &McbHeader<'_>, body: &[u8]) -> Result<McbValue, StorageError> {
    let decode = decoder_for(
        header.payload_key,
        (header.major, header.minor, header.patch),
    )
    .ok_or(reader::UNSUPPORTED)?;
    decode(body)
}

/// `value` as compact JSON text. A float keeps a fraction or an exponent (`1.0`, `-0.0`,
/// `1e300`) and an integer never gets one; a NaN or infinite float is `Mcb { NonFiniteFloat }`.
pub fn write_json(value: &McbValue) -> Result<String, StorageError> {
    let mut out = String::new();
    write_value(&mut out, value)?;
    Ok(out)
}

fn write_value(out: &mut String, value: &McbValue) -> Result<(), StorageError> {
    match value {
        McbValue::Null => out.push_str("null"),
        McbValue::Bool(value) => out.push_str(if *value { "true" } else { "false" }),
        McbValue::Int(value) => {
            let _ = write!(out, "{value}");
        }
        McbValue::UInt(value) => {
            let _ = write!(out, "{value}");
        }
        // `Debug` of a finite float is its shortest round-tripping form and always has a `.`
        // or an `e` (`1.0`, `1e300`, `1.5e-7`), which is the lexical float tag.
        McbValue::F32(value) => write_float(out, value.is_finite(), format_args!("{value:?}"))?,
        McbValue::F64(value) => write_float(out, value.is_finite(), format_args!("{value:?}"))?,
        McbValue::String(text) => write_string(out, text),
        McbValue::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_value(out, item)?;
            }
            out.push(']');
        }
        McbValue::Object(members) => {
            out.push('{');
            for (index, (key, item)) in members.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_string(out, key);
                out.push(':');
                write_value(out, item)?;
            }
            out.push('}');
        }
    }
    Ok(())
}

fn write_float(
    out: &mut String,
    finite: bool,
    text: std::fmt::Arguments<'_>,
) -> Result<(), StorageError> {
    if !finite {
        return Err(StorageError::Mcb {
            fault: McbFault::NonFiniteFloat,
        });
    }
    let _ = out.write_fmt(text);
    Ok(())
}

/// A JSON string literal: `"` and `\` escaped, control characters as `\n`, `\t`, … or `\u00XX`.
fn write_string(out: &mut String, text: &str) {
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            ch if u32::from(ch) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", u32::from(ch));
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The dispatch as it was written before the table: the oracle the table is held to.
    fn by_match(payload_key: &str, version: (u16, u16, u32)) -> Option<Decoder> {
        match (payload_key, version) {
            ("minecraft:voxel_shape", (1, 21, 110)) => Some(voxel::decode),
            ("tiers", (1, 26, 10)) => Some(trade::decode),
            ("minecraft:item", (1, 26, 30 | 40 | 50)) => Some(item::decode),
            ("minecraft:camera_entity", (1, 26, 10)) => Some(camera::decode),
            ("particle_effect", (1, 20, 80) | (1, 26, 10 | 30 | 50)) => Some(particle::decode),
            ("minecraft:poi_block", (1, 26, 40)) => Some(poi::decode_type),
            ("minecraft:poi_tag", (1, 26, 40)) => Some(poi::decode_tag),
            (
                "minecraft:crafting_items_catalog",
                (1, 21, 60 | 70 | 110) | (1, 26, 10 | 30 | 40 | 50),
            ) => Some(catalog::decode),
            _ => None,
        }
    }

    /// Every key and version the match names, crossed, plus a key and versions it does not
    /// name: the table picks the decoder the match picked, or none where it picked none.
    #[test]
    fn the_table_picks_the_decoder_the_match_picked() {
        let keys = [
            "minecraft:voxel_shape",
            "tiers",
            "minecraft:item",
            "minecraft:camera_entity",
            "particle_effect",
            "minecraft:poi_block",
            "minecraft:poi_tag",
            "minecraft:crafting_items_catalog",
            "minecraft:block",
        ];
        let versions = [
            (1, 20, 80),
            (1, 21, 60),
            (1, 21, 70),
            (1, 21, 110),
            (1, 26, 10),
            (1, 26, 20),
            (1, 26, 30),
            (1, 26, 40),
            (1, 26, 50),
            (1, 26, 60),
        ];
        let mut named = 0;
        for key in keys {
            for version in versions {
                let expected = by_match(key, version);
                let found = decoder_for(key, version);
                named += usize::from(expected.is_some());
                match (expected, found) {
                    (None, None) => {}
                    (Some(expected), Some(found)) => assert!(
                        std::ptr::fn_addr_eq(expected, found),
                        "{key} {version:?}: another decoder"
                    ),
                    _ => panic!(
                        "{key} {version:?}: the match decodes {}, the table {}",
                        expected.is_some(),
                        found.is_some()
                    ),
                }
            }
        }
        // 1 + 1 + 3 + 1 + 4 + 1 + 1 + 7 pairs.
        assert_eq!(named, 19);
    }
}
