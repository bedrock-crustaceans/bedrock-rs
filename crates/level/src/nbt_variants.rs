//! Optional support for the two NBT wire encodings this crate's own disk
//! format never uses, for callers that need them anyway -- see the Phase 6
//! "NBT format variants" task in `TODO.md`.
//!
//! # What this crate reads and writes on disk
//!
//! Every record `bedrock_level` decodes from or encodes to a Bedrock world
//! is little-endian NBT with fixed-width integers (`nbtx::LittleEndian`,
//! used directly throughout this crate via `nbtx::from_le_bytes`/
//! `to_le_bytes`). That is the only encoding the on-disk format has ever
//! used, in every fixture this crate's corpus carries.
//!
//! # What this module adds
//!
//! `nbtx` (the vendored NBT codec this crate depends on, see its own docs)
//! already implements two more encodings as generic parameters on the same
//! codec: [`nbtx::BigEndian`] (Java Edition's disk/network format) and
//! [`nbtx::VarintEndian`] (Bedrock's *network* format -- the wire encoding
//! `ClientboundPacket`/`ServerboundPacket` NBT payloads use over a
//! connection, distinct from the little-endian disk format this crate
//! otherwise handles exclusively: short/float fields stay little-endian,
//! but `int`/`long` fields become LEB128 varints). Neither is reachable
//! from `bedrock_level`'s own read/write paths -- nothing in a LevelDB
//! world or a `level.dat` is ever big-endian or varint-encoded -- so this
//! module is a thin, opt-in wrapper around what `nbtx` already implements,
//! named and documented for the two situations a caller of this crate
//! could plausibly want them in: reading/writing a Java-format NBT file
//! (`big_endian`) or decoding/encoding a captured or synthesized Bedrock
//! network packet's NBT payload (`network`). Both are re-exports in
//! function form, not new codec logic -- see nbtx's own `BigEndian`/
//! `VarintEndian` docs for the exact wire layout each produces.
//!
//! # What this module does not add
//!
//! `nbtx` has no lossy embedded-NUL / invalid-UTF-8 -> raw-byte-string
//! degradation path for a `String`-typed field: a facet-derived struct
//! field must already be declared `bstr::BString` to accept non-UTF-8
//! bytes (the path this crate's own strict-decode fields already use for
//! the one real non-UTF-8 string tag found in the corpus, the actor
//! storage key -- see `TODO.md`'s "Settle what a Bedrock NBT string
//! actually encodes" task and `tests/nbt_actor_storage_key.rs`); a
//! `String`-typed field with invalid bytes simply fails to decode, with no
//! attribute or fallback that widens it to raw bytes automatically the way
//! [`nbtx::Attr::LenientWidth`] widens a numeric tag. This is a real gap in
//! the vendored codec, not something this module can paper over without
//! reimplementing string decoding -- which is out of scope here (see
//! `TODO.md`'s "NBT format variants" task and the repo's Conventions on not
//! reimplementing NBT codecs in this crate). `nbtx::Value::String` already
//! carries `bstr::BString` regardless, so this only affects a typed struct
//! field declared `String` rather than `BString`.

use facet::Facet;

use crate::error::Result;

/// Reads a value from big-endian NBT (fixed-width big-endian integers,
/// Java Edition's own disk/network format). Bedrock never writes this on
/// disk; this exists for a caller bridging to Java-format data.
pub fn from_big_endian_bytes<'f, T: Facet<'f>>(reader: &mut impl std::io::Read) -> Result<T> {
    Ok(nbtx::from_be_bytes(reader)?)
}

/// Serializes `value` as big-endian NBT -- the inverse of
/// [`from_big_endian_bytes`].
pub fn to_big_endian_bytes<'f>(value: &'f (impl Facet<'f> + ?Sized)) -> Result<Vec<u8>> {
    Ok(nbtx::to_be_bytes(value)?)
}

/// Reads a value from Bedrock's *network* NBT encoding: little-endian
/// shorts/floats, but LEB128 varints for `int`/`long` fields -- the
/// encoding `ClientboundPacket`/`ServerboundPacket` NBT payloads use over
/// the wire, distinct from the fixed-width little-endian this crate's own
/// disk format uses exclusively.
pub fn from_network_bytes<'f, T: Facet<'f>>(reader: &mut impl std::io::Read) -> Result<T> {
    Ok(nbtx::from_varint_bytes(reader)?)
}

/// Serializes `value` as Bedrock network NBT -- the inverse of
/// [`from_network_bytes`].
pub fn to_network_bytes<'f>(value: &'f (impl Facet<'f> + ?Sized)) -> Result<Vec<u8>> {
    Ok(nbtx::to_varint_bytes(value)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn sample() -> nbtx::Value {
        let mut compound = nbtx::Compound::new();
        compound.insert("shortField".into(), nbtx::Value::Short(1000));
        compound.insert("intField".into(), nbtx::Value::Int(70_000));
        compound.insert("longField".into(), nbtx::Value::Long(-5_000_000_000));
        compound.insert("floatField".into(), nbtx::Value::Float(1.5));
        compound.insert("stringField".into(), nbtx::Value::String("hello".into()));
        nbtx::Value::Compound(compound)
    }

    #[test]
    fn big_endian_round_trips() {
        let value = sample();
        let bytes = to_big_endian_bytes(&value).unwrap();
        let mut cursor = Cursor::new(bytes);
        let decoded: nbtx::Value = from_big_endian_bytes(&mut cursor).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn network_round_trips() {
        let value = sample();
        let bytes = to_network_bytes(&value).unwrap();
        let mut cursor = Cursor::new(bytes);
        let decoded: nbtx::Value = from_network_bytes(&mut cursor).unwrap();
        assert_eq!(decoded, value);
    }

    /// The three encodings this module and this crate together expose
    /// produce genuinely different bytes for the same value -- confirms
    /// this module is not silently falling back to the disk format.
    #[test]
    fn the_three_encodings_disagree_on_wire_bytes() {
        let value = sample();
        let disk = nbtx::to_le_bytes(&value).unwrap();
        let big_endian = to_big_endian_bytes(&value).unwrap();
        let network = to_network_bytes(&value).unwrap();

        assert_ne!(disk, big_endian);
        assert_ne!(disk, network);
        assert_ne!(big_endian, network);

        // The varint encoding of intField (70000, which needs 3 varint
        // bytes) and longField (a large negative i64, which zigzags to a
        // large positive value needing close to the max 10 varint bytes)
        // makes the network encoding a different *length* than the fixed
        // 4/8-byte little-endian disk encoding, not just different bytes
        // at the same offsets.
        assert_ne!(disk.len(), network.len());
    }

    /// Big-endian NBT still round-trips a non-UTF-8-adjacent but otherwise
    /// ordinary string; this is not the embedded-NUL degradation case the
    /// module docs say nbtx does not support (that needs a `BString`-typed
    /// field to accept invalid UTF-8 at all, unrelated to which endianness
    /// wraps it).
    #[test]
    fn big_endian_round_trips_a_string_with_an_embedded_nul() {
        let mut compound = nbtx::Compound::new();
        compound.insert("s".into(), nbtx::Value::String("a\0b".into()));
        let value = nbtx::Value::Compound(compound);

        let bytes = to_big_endian_bytes(&value).unwrap();
        let mut cursor = Cursor::new(bytes);
        let decoded: nbtx::Value = from_big_endian_bytes(&mut cursor).unwrap();
        assert_eq!(decoded, value);
    }
}
