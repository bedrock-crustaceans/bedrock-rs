//! `0x30` LegacyTerrain decode/encode: synthetic round-trips against a hand-built column,
//! since no fixture in the corpus carries this era (the oldest chunk version among the
//! imported worlds is 15 -- see `crates/level/TODO.md`'s Phase 2 tasks). The established
//! 49,152-byte region is confirmed by one reference implementation, which actually parses
//! it; the other only declares the record's tag and never reads its payload at all, so it
//! confirms nothing about the byte layout. The tail past that point is not established by
//! either and is only ever exercised here as opaque bytes.

use std::io::Cursor;

use bedrock_level::legacy_terrain::{COLUMN_HEIGHT, ESTABLISHED_LEN, LegacyTerrain, SLICE_COUNT};
use bedrock_level::types::BlockPosition;

/// Builds a synthetic `0x30` value: 32,768 ids and 16,384 nibbles from the given functions
/// (called with the raw on-disk index, `0..32768`), followed by `tail`.
fn legacy_terrain_bytes(
    id_at: impl Fn(usize) -> u8,
    data_at: impl Fn(usize) -> u8,
    tail: &[u8],
) -> Vec<u8> {
    let mut out = Vec::with_capacity(ESTABLISHED_LEN + tail.len());
    for i in 0..16 * 16 * COLUMN_HEIGHT {
        out.push(id_at(i));
    }
    for pair in 0..16 * 16 * COLUMN_HEIGHT / 2 {
        let low = data_at(pair * 2) & 0x0f;
        let high = data_at(pair * 2 + 1) & 0x0f;
        out.push(low | (high << 4));
    }
    out.extend_from_slice(tail);
    out
}

/// A full record -- established region plus a recognizable tail -- round-trips through
/// `from_disk`/`to_disk` byte-identically, tail included, even though the tail is never
/// interpreted.
#[test]
fn round_trips_byte_identical_with_a_synthetic_tail() {
    let tail = b"not parsed, only carried";
    let bytes = legacy_terrain_bytes(|i| (i % 200) as u8, |i| (i % 11) as u8, tail);

    let terrain = LegacyTerrain::from_disk(&mut Cursor::new(bytes.as_slice())).unwrap();
    assert_eq!(terrain.tail(), tail);
    assert_eq!(terrain.slices().len(), SLICE_COUNT);

    let mut out = Cursor::new(Vec::new());
    terrain.to_disk(&mut out).unwrap();
    assert_eq!(out.into_inner(), bytes);
}

/// An empty tail (a record that is exactly the established length) round-trips too -- the
/// tail is genuinely optional bytes, not a region assumed to be non-empty.
#[test]
fn round_trips_with_no_tail() {
    let bytes = legacy_terrain_bytes(|i| i as u8, |i| (i % 16) as u8, &[]);
    assert_eq!(bytes.len(), ESTABLISHED_LEN);

    let terrain = LegacyTerrain::from_disk(&mut Cursor::new(bytes.as_slice())).unwrap();
    assert!(terrain.tail().is_empty());

    let mut out = Cursor::new(Vec::new());
    terrain.to_disk(&mut out).unwrap();
    assert_eq!(out.into_inner(), bytes);
}

/// Order-convention pin, hand-computed against the whole 128-tall column rather than a
/// single 16-tall slice: a block at `(x, y, z) = (2, 45, 9)` sits at raw on-disk index
/// `x*16*128 + z*128 + y` (X-major, then Z, then Y fastest over the full column height) --
/// `2*2048 + 9*128 + 45 = 5293`. That falls in vertical slice `45 / 16 = 2` at local Y
/// `45 % 16 = 13`.
#[test]
fn order_convention_places_a_known_id_in_the_right_slice_and_local_position() {
    let target = 2usize * 16 * COLUMN_HEIGHT + 9 * COLUMN_HEIGHT + 45;
    assert_eq!(target, 5293);

    let bytes = legacy_terrain_bytes(|i| if i == target { 0xcd } else { 0 }, |_| 0, &[]);
    let terrain = LegacyTerrain::from_disk(&mut Cursor::new(bytes.as_slice())).unwrap();

    let slice = &terrain.slices()[2];
    assert_eq!(
        slice.get(BlockPosition(2, 13, 9)).unwrap().numeric(),
        Some((0xcd, 0))
    );
    // The slice directly below/above the target holds nothing at the same local position.
    assert_eq!(
        terrain.slices()[1]
            .get(BlockPosition(2, 13, 9))
            .unwrap()
            .numeric(),
        Some((0, 0))
    );
    assert_eq!(
        terrain.slices()[3]
            .get(BlockPosition(2, 13, 9))
            .unwrap()
            .numeric(),
        Some((0, 0))
    );
}

/// Nibble-packing pin against the full column, positions derived by hand rather than by
/// running the module's own unravelling formula: raw index 200 (even) is the low nibble of
/// nibble byte 100, raw index 201 (odd) is its high nibble -- the same convention as the
/// non-paletted `0x2f` layout, just over a taller column.
///
/// By hand: with the column X-major then Z then Y fastest over height 128, raw index 200
/// is `x=0` (`200 < 2048`), then `z = 200 / 128 = 1` with `128` left over, so `y = 200 -
/// 128*1 = 72`. That is vertical slice `72 / 16 = 4` at local Y `72 - 4*16 = 8`, i.e.
/// `(x, y, z) = (0, 8, 1)` in slice 4. Raw index 201 is the very next block up: still `x=0,
/// z=1`, `y = 73`, the same slice 4 at local Y `73 - 64 = 9`, i.e. `(0, 9, 1)`.
#[test]
fn nibble_packing_pins_even_low_odd_high() {
    let bytes = legacy_terrain_bytes(
        |_| 1,
        |i| {
            if i == 200 {
                4
            } else if i == 201 {
                11
            } else {
                0
            }
        },
        &[],
    );

    assert_eq!(bytes[16 * 16 * COLUMN_HEIGHT + 100], 0xb4);

    let terrain = LegacyTerrain::from_disk(&mut Cursor::new(bytes.as_slice())).unwrap();
    assert_eq!(
        terrain.slices()[4]
            .get(BlockPosition(0, 8, 1))
            .unwrap()
            .numeric(),
        Some((1, 4))
    );
    assert_eq!(
        terrain.slices()[4]
            .get(BlockPosition(0, 9, 1))
            .unwrap()
            .numeric(),
        Some((1, 11))
    );
}

/// A record shorter than the established 49,152 bytes -- even with no tail expected -- fails
/// to decode rather than silently returning a partially zero-filled column.
#[test]
fn truncated_established_region_errors() {
    let mut bytes = legacy_terrain_bytes(|i| i as u8, |i| (i % 16) as u8, &[]);
    bytes.truncate(bytes.len() - 1);
    assert!(LegacyTerrain::from_disk(&mut Cursor::new(bytes.as_slice())).is_err());
}
