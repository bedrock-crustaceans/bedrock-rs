//! Direct reading and writing of modern block palette entries.
//!
//! Almost every palette entry has the same shape: a root compound with a `name` string, a
//! `version` int and a `states` compound of scalar values, `states` and `version` in either
//! order. Going through [`nbtx::Value`] and reflection costs far more than the entry itself,
//! so this shape is handled here and anything else -- legacy `{name, val}` entries, unknown
//! fields, nested states -- is read again by nbtx from its start, so the result is the same
//! either way.

use std::io::{Cursor, Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use indexmap::IndexMap;
use nbtx::Value;

use crate::error::Result;
use crate::subchunk::{BlockDef, FieldOrder};

const TAG_END: u8 = 0;
const TAG_BYTE: u8 = 1;
const TAG_SHORT: u8 = 2;
const TAG_INT: u8 = 3;
const TAG_LONG: u8 = 4;
const TAG_FLOAT: u8 = 5;
const TAG_DOUBLE: u8 = 6;
const TAG_STRING: u8 = 8;
const TAG_COMPOUND: u8 = 10;

/// Entries larger than this go through nbtx, so its decode limits apply to them as before.
const MAX_DIRECT_LEN: u64 = 1 << 20;

/// Reads one modern entry of the common shape, with the order its `version` and `states`
/// were written in. Returns `None`, with the reader back where it started, for any other
/// shape and for malformed bytes, which the caller then hands to nbtx.
pub(crate) fn read_modern<R>(reader: &mut Cursor<R>) -> Option<(BlockDef, FieldOrder)>
where
    Cursor<R>: Read,
{
    let start = reader.position();
    let read = read_common(reader);
    if read.is_some() && reader.position() - start <= MAX_DIRECT_LEN {
        return read;
    }
    reader.set_position(start);
    None
}

/// Writes one modern entry in the given field order, byte for byte as nbtx writes it.
/// Returns `false`, having written nothing, for an entry with a state nbtx has to write.
pub(crate) fn write_modern<W: Write>(
    writer: &mut W,
    entry: &BlockDef,
    order: FieldOrder,
) -> Result<bool> {
    if !writes_directly(entry) {
        return Ok(false);
    }

    writer.write_u8(TAG_COMPOUND)?;
    write_str(writer, b"")?;
    write_key(writer, TAG_STRING, b"name")?;
    write_str(writer, entry.name.as_bytes())?;
    if order == FieldOrder::VersionThenStates {
        write_version(writer, entry.version)?;
    }
    write_key(writer, TAG_COMPOUND, b"states")?;
    for (key, value) in &entry.states {
        write_scalar(writer, key.as_bytes(), value)?;
    }
    writer.write_u8(TAG_END)?;
    if order == FieldOrder::StatesThenVersion {
        write_version(writer, entry.version)?;
    }
    writer.write_u8(TAG_END)?;
    Ok(true)
}

fn read_common<R: Read>(r: &mut R) -> Option<(BlockDef, FieldOrder)> {
    if r.read_u8().ok()? != TAG_COMPOUND {
        return None;
    }
    read_bytes(r)?;

    let (mut name, mut version, mut states) = (None, None, None);
    let mut order = FieldOrder::default();
    loop {
        let tag = r.read_u8().ok()?;
        if tag == TAG_END {
            break;
        }
        let mut key = [0; 8];
        let key = read_field_name(r, &mut key)?;
        match (tag, key) {
            (TAG_STRING, b"name") if name.is_none() => {
                name = Some(String::from_utf8(read_bytes(r)?).ok()?);
            }
            (TAG_INT, b"version") if version.is_none() => {
                version = Some(r.read_i32::<LittleEndian>().ok()?);
            }
            (TAG_COMPOUND, b"states") if states.is_none() => {
                if version.is_some() {
                    order = FieldOrder::VersionThenStates;
                }
                states = Some(read_states(r)?);
            }
            _ => return None,
        }
    }

    let block = BlockDef {
        name: name?,
        version,
        states: states.unwrap_or_default(),
    };
    Some((block, order))
}

fn read_states<R: Read>(r: &mut R) -> Option<IndexMap<String, Value>> {
    let mut states = IndexMap::new();
    loop {
        let tag = r.read_u8().ok()?;
        if tag == TAG_END {
            return Some(states);
        }
        let key = String::from_utf8(read_bytes(r)?).ok()?;
        let value = match tag {
            TAG_BYTE => Value::Byte(r.read_i8().ok()?),
            TAG_SHORT => Value::Short(r.read_i16::<LittleEndian>().ok()?),
            TAG_INT => Value::Int(r.read_i32::<LittleEndian>().ok()?),
            TAG_LONG => Value::Long(r.read_i64::<LittleEndian>().ok()?),
            TAG_FLOAT => Value::Float(r.read_f32::<LittleEndian>().ok()?),
            TAG_DOUBLE => Value::Double(r.read_f64::<LittleEndian>().ok()?),
            TAG_STRING => Value::String(read_bytes(r)?.into()),
            _ => return None,
        };
        if states.insert(key, value).is_some() {
            return None;
        }
    }
}

/// A field name of the root compound into `buf`, without allocating. A name longer than
/// `buf` is none of the known fields.
fn read_field_name<'b, R: Read>(r: &mut R, buf: &'b mut [u8; 8]) -> Option<&'b [u8]> {
    let len = usize::from(r.read_u16::<LittleEndian>().ok()?);
    let name = buf.get_mut(..len)?;
    r.read_exact(name).ok()?;
    Some(name)
}

/// A string payload: a little endian `u16` length and the bytes.
fn read_bytes<R: Read>(r: &mut R) -> Option<Vec<u8>> {
    let len = r.read_u16::<LittleEndian>().ok()?;
    let mut buf = vec![0; usize::from(len)];
    r.read_exact(&mut buf).ok()?;
    Some(buf)
}

/// Whether every state is a scalar and every string fits its `u16` length.
fn writes_directly(entry: &BlockDef) -> bool {
    let fits = |s: &[u8]| u16::try_from(s.len()).is_ok();
    fits(entry.name.as_bytes())
        && entry.states.iter().all(|(key, value)| {
            fits(key.as_bytes())
                && match value {
                    Value::String(s) => fits(s),
                    Value::Byte(_)
                    | Value::Short(_)
                    | Value::Int(_)
                    | Value::Long(_)
                    | Value::Float(_)
                    | Value::Double(_) => true,
                    _ => false,
                }
        })
}

fn write_version<W: Write>(w: &mut W, version: Option<i32>) -> Result<()> {
    if let Some(version) = version {
        write_key(w, TAG_INT, b"version")?;
        w.write_i32::<LittleEndian>(version)?;
    }
    Ok(())
}

fn write_scalar<W: Write>(w: &mut W, key: &[u8], value: &Value) -> Result<()> {
    match value {
        Value::Byte(v) => {
            write_key(w, TAG_BYTE, key)?;
            w.write_i8(*v)?;
        }
        Value::Short(v) => {
            write_key(w, TAG_SHORT, key)?;
            w.write_i16::<LittleEndian>(*v)?;
        }
        Value::Int(v) => {
            write_key(w, TAG_INT, key)?;
            w.write_i32::<LittleEndian>(*v)?;
        }
        Value::Long(v) => {
            write_key(w, TAG_LONG, key)?;
            w.write_i64::<LittleEndian>(*v)?;
        }
        Value::Float(v) => {
            write_key(w, TAG_FLOAT, key)?;
            w.write_f32::<LittleEndian>(*v)?;
        }
        Value::Double(v) => {
            write_key(w, TAG_DOUBLE, key)?;
            w.write_f64::<LittleEndian>(*v)?;
        }
        Value::String(v) => {
            write_key(w, TAG_STRING, key)?;
            write_str(w, v)?;
        }
        _ => unreachable!("only scalars are written directly"),
    }
    Ok(())
}

fn write_key<W: Write>(w: &mut W, tag: u8, key: &[u8]) -> Result<()> {
    w.write_u8(tag)?;
    write_str(w, key)
}

fn write_str<W: Write>(w: &mut W, s: &[u8]) -> Result<()> {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "`writes_directly` checked that every string fits"
    )]
    w.write_u16::<LittleEndian>(s.len() as u16)?;
    w.write_all(s)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subchunk::{PaletteEntry, read_palette_entry_nbtx, write_palette_entry_nbtx};

    fn block(states: &[(&str, Value)], version: Option<i32>) -> BlockDef {
        BlockDef {
            name: "minecraft:oak_stairs".to_owned(),
            version,
            states: states
                .iter()
                .map(|(k, v)| ((*k).to_owned(), v.clone()))
                .collect(),
        }
    }

    fn samples() -> Vec<BlockDef> {
        vec![
            block(&[], None),
            block(&[], Some(18_168_865)),
            block(
                &[
                    ("upside_down_bit", Value::Byte(1)),
                    ("weirdo_direction", Value::Int(3)),
                    (
                        "minecraft:cardinal_direction",
                        Value::String("north".into()),
                    ),
                    ("short", Value::Short(-2)),
                    ("long", Value::Long(1 << 40)),
                    ("float", Value::Float(0.5)),
                    ("double", Value::Double(-1.25)),
                ],
                Some(17_959_425),
            ),
            block(
                &[("nested", Value::List(nbtx::ValueList::Int(vec![1, 2, 3])))],
                Some(1),
            ),
        ]
    }

    const ORDERS: [FieldOrder; 2] = [FieldOrder::StatesThenVersion, FieldOrder::VersionThenStates];

    fn nbtx_bytes(entry: &BlockDef, order: FieldOrder) -> Vec<u8> {
        let mut out = Vec::new();
        write_palette_entry_nbtx(&mut out, &PaletteEntry::Modern(entry.clone()), order).unwrap();
        out
    }

    /// What the direct read and the nbtx read of the same bytes return.
    type Both = (
        Option<(BlockDef, FieldOrder)>,
        Result<(PaletteEntry, FieldOrder)>,
    );

    /// The direct read and the nbtx read of the same bytes: both succeed with the same entry
    /// and order, or the direct read declines.
    fn read_both(bytes: &[u8]) -> Both {
        let mut cursor = Cursor::new(bytes);
        let direct = read_modern(&mut cursor);
        if direct.is_some() {
            assert_eq!(
                cursor.position(),
                bytes.len() as u64,
                "entry not fully read"
            );
        } else {
            assert_eq!(cursor.position(), 0, "a declined read moved the reader");
        }
        (direct, read_palette_entry_nbtx(&mut Cursor::new(bytes)))
    }

    #[test]
    fn written_entries_match_nbtx_in_both_orders() {
        for entry in samples() {
            for order in ORDERS {
                let mut direct = Vec::new();
                if write_modern(&mut direct, &entry, order).unwrap() {
                    assert_eq!(direct, nbtx_bytes(&entry, order), "{entry:?} {order:?}");
                } else {
                    assert!(direct.is_empty());
                    assert!(entry.states.contains_key("nested"));
                }
            }
        }
    }

    #[test]
    fn read_entries_match_nbtx_in_both_orders() {
        for entry in samples() {
            for order in ORDERS {
                let (direct, generic) = read_both(&nbtx_bytes(&entry, order));
                let (generic, generic_order) = generic.unwrap();
                if let Some((direct, direct_order)) = direct {
                    assert_eq!(PaletteEntry::Modern(direct), generic);
                    assert_eq!(direct_order, generic_order);
                } else {
                    assert!(entry.states.contains_key("nested"));
                }
            }
        }
    }

    /// Shapes this module does not read itself are left to nbtx.
    #[test]
    fn other_shapes_are_declined() {
        let named = |tag: u8, key: &[u8], payload: &[u8]| {
            let mut v = vec![tag];
            v.extend(u16::try_from(key.len()).unwrap().to_le_bytes());
            v.extend(key);
            v.extend(payload);
            v
        };
        let name = named(TAG_STRING, b"name", b"\x03\x00abc");
        let compound = |fields: &[Vec<u8>]| {
            let mut v = vec![TAG_COMPOUND, 0, 0];
            for f in fields {
                v.extend(f);
            }
            v.push(TAG_END);
            v
        };
        let cases = [
            // A legacy entry.
            compound(&[name.clone(), named(TAG_SHORT, b"val", &[3, 0])]),
            // An unknown key.
            compound(&[name.clone(), named(TAG_INT, b"extra", &[1, 0, 0, 0])]),
            // A duplicate key.
            compound(&[name.clone(), name.clone()]),
            // A duplicate state.
            compound(&[
                name.clone(),
                named(
                    TAG_COMPOUND,
                    b"states",
                    &[
                        named(TAG_BYTE, b"s", &[1]),
                        named(TAG_BYTE, b"s", &[2]),
                        vec![TAG_END],
                    ]
                    .concat(),
                ),
            ]),
            // No name.
            compound(&[named(TAG_INT, b"version", &[1, 0, 0, 0])]),
            // A name that is not UTF-8.
            compound(&[named(TAG_STRING, b"name", b"\x01\x00\xff")]),
            // A version of the wrong type.
            compound(&[name.clone(), named(TAG_SHORT, b"version", &[1, 0])]),
            // Not a compound.
            named(TAG_INT, b"", &[1, 0, 0, 0]),
            // Cut short.
            compound(std::slice::from_ref(&name))[..6].to_vec(),
        ];
        for bytes in cases {
            let (direct, _) = read_both(&bytes);
            assert!(direct.is_none(), "{bytes:?} read directly");
        }
    }
}
