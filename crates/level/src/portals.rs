//! `portals`: the world's Nether-portal registry.
//!
//! A single, flat, database-wide key (`b"portals"`, not chunk-scoped and
//! not a [`crate::key::Key`]) holding one uncompressed little-endian NBT
//! compound: a root `data` compound wrapping a `PortalRecords` list. Each
//! list entry names one built portal frame and the coordinates a player
//! teleporting into it should land at.
//!
//! # Corpus
//!
//! Present in 4 of the 14 worlds scanned (`v1_18`, `v1_18_30`, `v1_19_30`,
//! and the 1.26 seed world), 5 `PortalRecords` entries total (the seed
//! world carries two). Every entry's field set and order agrees exactly
//! with the originally proposed layout --
//! `{DimId: Int, Span: Byte, TpX: Int, TpY: Int, TpZ: Int, Xa: Byte,
//! Za: Byte}`, in that field order with no exceptions -- and with both
//! reference implementations read against for this crate (one models this
//! exact field set and order for both read and write; the other's
//! independent NBT dump matches the same field set). [`Portals`] is
//! therefore a typed struct rather than an order-preserving
//! [`nbtx::Value`] (contrast [`crate::map_data`], where real records
//! disagree on field order): `decode_then_encode_reproduces_every_real_portals_record`
//! pins byte-identical re-encoding against all 4 real records.
//!
//! Unknown fields on a `PortalRecords` entry or on the root compound are
//! tolerated on decode (matching this crate's `player.rs` convention for
//! typed NBT records) rather than rejected, per architecture decision 11 --
//! "degrade on unmodelled" -- but are not preserved through a decode then
//! encode cycle, since nothing in the corpus has ever shown one.

use std::io::Cursor;

use facet::Facet;

use crate::db::Database;
use crate::error::{Error, Result};

/// The on-disk key this whole record is stored under.
pub const KEY: &[u8] = b"portals";

/// One `PortalRecords` entry: a built Nether-portal frame and the
/// coordinates a teleport into it lands at.
#[derive(Facet, Debug, Clone, Copy, PartialEq, Eq)]
#[facet(nbtx::allow_unknown_fields)]
#[facet(rename_all = "PascalCase")]
pub struct PortalRecord {
    /// The dimension id the portal was built in.
    pub dim_id: i32,
    /// The portal frame's width or height, depending on orientation.
    pub span: i8,
    pub tp_x: i32,
    pub tp_y: i32,
    pub tp_z: i32,
    /// Teleport-target X axis offset/orientation within the frame.
    pub xa: i8,
    /// Teleport-target Z axis offset/orientation within the frame.
    pub za: i8,
}

#[derive(Facet, Debug, Clone, PartialEq)]
#[facet(nbtx::allow_unknown_fields)]
struct PortalsInner {
    #[facet(rename = "PortalRecords")]
    portal_records: Vec<PortalRecord>,
}

#[derive(Facet, Debug, Clone, PartialEq)]
#[facet(nbtx::allow_unknown_fields)]
struct PortalsRoot {
    data: PortalsInner,
}

/// The decoded `portals` record: every built portal's record, in on-disk
/// order.
#[derive(Debug, Clone, PartialEq)]
pub struct Portals {
    pub records: Vec<PortalRecord>,
}

/// Decodes a `portals` value.
///
/// # Errors
///
/// Returns an error if `bytes` is not a valid little-endian NBT compound of
/// the expected shape, or [`Error::Invalid`] if bytes remain after it.
pub fn decode(bytes: &[u8]) -> Result<Portals> {
    let mut cursor = Cursor::new(bytes);
    let root: PortalsRoot = nbtx::from_le_bytes(&mut cursor)?;
    if cursor.position() != bytes.len() as u64 {
        return Err(Error::Invalid(
            "portals record has trailing bytes past its NBT compound",
        ));
    }
    Ok(Portals {
        records: root.data.portal_records,
    })
}

/// Encodes a `portals` value. The inverse of [`decode`].
///
/// # Errors
///
/// Returns an error if NBT serialization of the record fails.
pub fn encode(portals: &Portals) -> Result<Vec<u8>> {
    let root = PortalsRoot {
        data: PortalsInner {
            portal_records: portals.records.clone(),
        },
    };
    Ok(nbtx::to_le_bytes(&root)?)
}

/// Reads the `portals` record from `db`, if present.
///
/// # Errors
///
/// Returns an error if reading from `db` fails or if the stored value does
/// not [`decode`].
pub fn read(db: &Database) -> Result<Option<Portals>> {
    match db.get(KEY)? {
        Some(bytes) => Ok(Some(decode(&bytes)?)),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Portals {
        Portals {
            records: vec![PortalRecord {
                dim_id: 0,
                span: 9,
                tp_x: -68,
                tp_y: 5,
                tp_z: -34,
                xa: 0,
                za: 1,
            }],
        }
    }

    #[test]
    fn decode_then_encode_round_trips() {
        let portals = sample();
        let bytes = encode(&portals).unwrap();
        assert_eq!(decode(&bytes).unwrap(), portals);
    }

    /// From the corpus: `v1_18`'s `portals` record, 96 bytes.
    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "the real record's bytes are inlined as a fixture"
    )]
    fn decode_then_encode_reproduces_every_real_portals_record() {
        let real_records: &[(&[u8], &[PortalRecord])] = &[
            (
                &[
                    0x0a, 0x00, 0x00, 0x0a, 0x04, 0x00, 0x64, 0x61, 0x74, 0x61, 0x09, 0x0d, 0x00,
                    0x50, 0x6f, 0x72, 0x74, 0x61, 0x6c, 0x52, 0x65, 0x63, 0x6f, 0x72, 0x64, 0x73,
                    0x0a, 0x01, 0x00, 0x00, 0x00, 0x03, 0x05, 0x00, 0x44, 0x69, 0x6d, 0x49, 0x64,
                    0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x00, 0x53, 0x70, 0x61, 0x6e, 0x02, 0x03,
                    0x03, 0x00, 0x54, 0x70, 0x58, 0x04, 0x00, 0x00, 0x00, 0x03, 0x03, 0x00, 0x54,
                    0x70, 0x59, 0x47, 0x00, 0x00, 0x00, 0x03, 0x03, 0x00, 0x54, 0x70, 0x5a, 0xef,
                    0xff, 0xff, 0xff, 0x01, 0x02, 0x00, 0x58, 0x61, 0x00, 0x01, 0x02, 0x00, 0x5a,
                    0x61, 0x01, 0x00, 0x00, 0x00,
                ][..],
                &[PortalRecord {
                    dim_id: 1,
                    span: 2,
                    tp_x: 4,
                    tp_y: 71,
                    tp_z: -17,
                    xa: 0,
                    za: 1,
                }][..],
            ),
            (
                &[
                    0x0a, 0x00, 0x00, 0x0a, 0x04, 0x00, 0x64, 0x61, 0x74, 0x61, 0x09, 0x0d, 0x00,
                    0x50, 0x6f, 0x72, 0x74, 0x61, 0x6c, 0x52, 0x65, 0x63, 0x6f, 0x72, 0x64, 0x73,
                    0x0a, 0x01, 0x00, 0x00, 0x00, 0x03, 0x05, 0x00, 0x44, 0x69, 0x6d, 0x49, 0x64,
                    0x00, 0x00, 0x00, 0x00, 0x01, 0x04, 0x00, 0x53, 0x70, 0x61, 0x6e, 0x07, 0x03,
                    0x03, 0x00, 0x54, 0x70, 0x58, 0x0b, 0x00, 0x00, 0x00, 0x03, 0x03, 0x00, 0x54,
                    0x70, 0x59, 0x05, 0x00, 0x00, 0x00, 0x03, 0x03, 0x00, 0x54, 0x70, 0x5a, 0x4e,
                    0x00, 0x00, 0x00, 0x01, 0x02, 0x00, 0x58, 0x61, 0x01, 0x01, 0x02, 0x00, 0x5a,
                    0x61, 0x00, 0x00, 0x00, 0x00,
                ][..],
                &[PortalRecord {
                    dim_id: 0,
                    span: 7,
                    tp_x: 11,
                    tp_y: 5,
                    tp_z: 78,
                    xa: 1,
                    za: 0,
                }][..],
            ),
            (
                &[
                    0x0a, 0x00, 0x00, 0x0a, 0x04, 0x00, 0x64, 0x61, 0x74, 0x61, 0x09, 0x0d, 0x00,
                    0x50, 0x6f, 0x72, 0x74, 0x61, 0x6c, 0x52, 0x65, 0x63, 0x6f, 0x72, 0x64, 0x73,
                    0x0a, 0x01, 0x00, 0x00, 0x00, 0x03, 0x05, 0x00, 0x44, 0x69, 0x6d, 0x49, 0x64,
                    0x00, 0x00, 0x00, 0x00, 0x01, 0x04, 0x00, 0x53, 0x70, 0x61, 0x6e, 0x09, 0x03,
                    0x03, 0x00, 0x54, 0x70, 0x58, 0xbc, 0xff, 0xff, 0xff, 0x03, 0x03, 0x00, 0x54,
                    0x70, 0x59, 0x05, 0x00, 0x00, 0x00, 0x03, 0x03, 0x00, 0x54, 0x70, 0x5a, 0xde,
                    0xff, 0xff, 0xff, 0x01, 0x02, 0x00, 0x58, 0x61, 0x00, 0x01, 0x02, 0x00, 0x5a,
                    0x61, 0x01, 0x00, 0x00, 0x00,
                ][..],
                &[PortalRecord {
                    dim_id: 0,
                    span: 9,
                    tp_x: -68,
                    tp_y: 5,
                    tp_z: -34,
                    xa: 0,
                    za: 1,
                }][..],
            ),
            (
                &[
                    0x0a, 0x00, 0x00, 0x0a, 0x04, 0x00, 0x64, 0x61, 0x74, 0x61, 0x09, 0x0d, 0x00,
                    0x50, 0x6f, 0x72, 0x74, 0x61, 0x6c, 0x52, 0x65, 0x63, 0x6f, 0x72, 0x64, 0x73,
                    0x0a, 0x02, 0x00, 0x00, 0x00, 0x03, 0x05, 0x00, 0x44, 0x69, 0x6d, 0x49, 0x64,
                    0x00, 0x00, 0x00, 0x00, 0x01, 0x04, 0x00, 0x53, 0x70, 0x61, 0x6e, 0x01, 0x03,
                    0x03, 0x00, 0x54, 0x70, 0x58, 0x4b, 0x00, 0x00, 0x00, 0x03, 0x03, 0x00, 0x54,
                    0x70, 0x59, 0x46, 0x00, 0x00, 0x00, 0x03, 0x03, 0x00, 0x54, 0x70, 0x5a, 0xbb,
                    0x00, 0x00, 0x00, 0x01, 0x02, 0x00, 0x58, 0x61, 0x01, 0x01, 0x02, 0x00, 0x5a,
                    0x61, 0x00, 0x00, 0x03, 0x05, 0x00, 0x44, 0x69, 0x6d, 0x49, 0x64, 0x00, 0x00,
                    0x00, 0x00, 0x01, 0x04, 0x00, 0x53, 0x70, 0x61, 0x6e, 0x01, 0x03, 0x03, 0x00,
                    0x54, 0x70, 0x58, 0x4b, 0x00, 0x00, 0x00, 0x03, 0x03, 0x00, 0x54, 0x70, 0x59,
                    0x46, 0x00, 0x00, 0x00, 0x03, 0x03, 0x00, 0x54, 0x70, 0x5a, 0xbd, 0x00, 0x00,
                    0x00, 0x01, 0x02, 0x00, 0x58, 0x61, 0x00, 0x01, 0x02, 0x00, 0x5a, 0x61, 0x01,
                    0x00, 0x00, 0x00,
                ][..],
                &[
                    PortalRecord {
                        dim_id: 0,
                        span: 1,
                        tp_x: 75,
                        tp_y: 70,
                        tp_z: 187,
                        xa: 1,
                        za: 0,
                    },
                    PortalRecord {
                        dim_id: 0,
                        span: 1,
                        tp_x: 75,
                        tp_y: 70,
                        tp_z: 189,
                        xa: 0,
                        za: 1,
                    },
                ][..],
            ),
        ];

        for (bytes, records) in real_records {
            let decoded = decode(bytes).unwrap();
            assert_eq!(decoded.records, *records);
            assert_eq!(&encode(&decoded).unwrap(), bytes);
        }
    }

    #[test]
    fn decode_rejects_malformed_bytes() {
        assert!(decode(&[0xff]).is_err());
        assert!(decode(&[]).is_err());
    }

    #[test]
    fn decode_rejects_trailing_bytes() {
        let mut bytes = encode(&sample()).unwrap();
        bytes.push(0xff);
        assert!(decode(&bytes).is_err());
    }
}
