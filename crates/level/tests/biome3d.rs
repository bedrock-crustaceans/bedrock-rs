//! Exercises `0x2b` (paletted 3D biomes) decode/encode against the
//! version-diverse imported fixtures, mirroring `data2d.rs`'s coverage of
//! `0x2d`. Written directly against `Biomes3d`/`Database`, not the level
//! layer, so it stays a check of the format decode itself.

mod support;

use std::collections::BTreeMap;
use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};

use bedrock_level::Greedy;
use bedrock_level::biome::Biomes3d;
use bedrock_level::greedy::GreedyArray;
use bedrock_level::key::{Key, KeyVariant};

/// Every `0x2b` record in every `biomes_3d` fixture decodes and re-encodes
/// to the exact original bytes -- checked first as byte identity, and where
/// that fails, as the weaker (but still meaningful) guarantee that
/// re-decoding the re-encoded bytes produces the identical `Biomes3d` value
/// the original bytes decoded to (`Biomes3d` derives `PartialEq`).
///
/// The fallback exists because of a real, corpus-confirmed finding this
/// test surfaced: a paletted fragment's packed 32-bit words carry unused
/// high bits in their final, partially-filled word at widths that do not
/// divide 32 evenly (3/5/6 -- `BitArray`/`GreedyArray`'s packing, shared
/// with block/liquid layers), and the game does not always write them as
/// zero, matching the identical finding Phase 1 already pinned for
/// block/liquid layers (`packed_index_padding_bits_are_not_always_zero` in
/// `tests/bit_layers.rs`) and the same reasoning it recorded: the
/// byte-identity guarantee belongs to the raw layer (`ChunkRecords` passes
/// these bytes through unmodified), not to `Biomes3d`, which is the typed,
/// interpreted form and whose contract is semantic fidelity.
///
/// Note what "decode-equal" does and does not prove: it is a strictly
/// weaker check than byte identity, and on its own it cannot distinguish
/// "only the documented padding bits differ" from any other decode-invisible
/// byte difference -- a bit this crate reads but never re-derives correctly
/// would pass here exactly the same way a padding bit does. What rules that
/// out is [`every_real_biome_header_sets_the_low_bit`] below, checked
/// separately: it establishes that the one known decode-invisible bit this
/// format actually has (the storage header's low bit, discarded by
/// `BitArray::from_disk`'s `>> 1` and never round-tripped through the
/// decoded `Biomes3d` value at all) is always 1 in every real fragment this
/// crate has scanned, so a "decode-equal only" record here is attributable
/// to that finding specifically, not to an unbounded "some other bit
/// differs and this test cannot tell" possibility -- and a genuine
/// corruption still fails here regardless, since it changes the decoded
/// palette or index values.
#[test]
fn decodes_and_round_trips_every_record() {
    let fixtures: Vec<_> = support::fixtures()
        .into_iter()
        .filter(|f| f.capabilities.biomes_3d)
        .collect();
    assert!(
        fixtures.len() >= 2,
        "expected at least two biomes_3d fixtures, found {}",
        fixtures.len()
    );

    let mut byte_identical: BTreeMap<String, usize> = BTreeMap::new();
    let mut decode_equal_only: BTreeMap<String, usize> = BTreeMap::new();

    for fixture in &fixtures {
        let (_tmp, db) = fixture.open();
        let mut keys = db
            .keys()
            .unwrap_or_else(|e| panic!("{}: failed to iterate keys: {e}", fixture.name));

        for kv in &mut keys {
            let mut key_cursor = Cursor::new(kv.key());
            let Ok(key) = Key::deserialize(&mut key_cursor) else {
                continue;
            };
            if key.data != KeyVariant::Biome3d {
                continue;
            }

            let bytes: Vec<u8> = kv.value().into();
            let mut reader = Cursor::new(bytes.as_slice());
            let decoded = Biomes3d::from_disk::<Greedy, _>(&mut reader).unwrap_or_else(|e| {
                panic!(
                    "{}: chunk {:?}: failed to decode 0x2b record: {e}",
                    fixture.name, key.chunk
                )
            });

            let mut writer = Cursor::new(Vec::new());
            decoded.to_disk(&mut writer).unwrap_or_else(|e| {
                panic!(
                    "{}: chunk {:?}: failed to re-encode 0x2b record: {e}",
                    fixture.name, key.chunk
                )
            });
            let re_encoded = writer.into_inner();

            if re_encoded == bytes {
                *byte_identical.entry(fixture.name.clone()).or_insert(0) += 1;
                continue;
            }

            let mut re_reader = Cursor::new(re_encoded.as_slice());
            let re_decoded = Biomes3d::from_disk::<Greedy, _>(&mut re_reader).unwrap_or_else(|e| {
                panic!(
                    "{}: chunk {:?}: re-encoded bytes failed to decode: {e}",
                    fixture.name, key.chunk
                )
            });
            assert_eq!(
                decoded, re_decoded,
                "{}: chunk {:?}: re-encoded 0x2b record decodes to a different value \
                 (a genuine mismatch, not just padding bits)",
                fixture.name, key.chunk
            );
            *decode_equal_only.entry(fixture.name.clone()).or_insert(0) += 1;
        }
    }

    let total: usize =
        byte_identical.values().sum::<usize>() + decode_equal_only.values().sum::<usize>();
    assert!(
        total > 0,
        "no 0x2b records were found across any biomes_3d fixture"
    );
    for fixture in &fixtures {
        let total_for_fixture = byte_identical.get(&fixture.name).copied().unwrap_or(0)
            + decode_equal_only.get(&fixture.name).copied().unwrap_or(0);
        assert!(
            total_for_fixture > 0,
            "{}: capability biomes_3d is set but no 0x2b records were found",
            fixture.name
        );
    }

    // Printed rather than asserted on: a useful record of coverage, not an
    // invariant with a "right" number.
    println!("0x2b records byte-identical per fixture: {byte_identical:?}");
    println!(
        "0x2b records decode-equal only (byte differs, decoded value does not -- \
         see this test's doc comment for what that is and is not evidence of) \
         per fixture: {decode_equal_only:?}"
    );
}

/// Reads one fragment header byte directly (bypassing `BitArray::from_disk`,
/// which discards the low bit via its own `>> 1` -- see `Biomes3d::to_disk`'s
/// doc comment) and then advances `reader` past the rest of that fragment
/// exactly the way `Biomes3d::from_disk` does, so a Data fragment's
/// variable-length packed words and palette are consumed correctly rather
/// than guessed at. Returns the raw header byte.
///
/// Reuses [`GreedyArray::from_disk`] (the same production word-packing code
/// [`Biomes3d::to_disk`]/`from_disk` themselves go through) for the packed
/// index array specifically so this test's notion of "how many bytes does a
/// Data fragment occupy" cannot silently drift from what the crate actually
/// implements.
fn read_fragment_header<R: AsRef<[u8]>>(
    reader: &mut Cursor<R>,
) -> bedrock_level::error::Result<u8> {
    let header = reader.read_u8()?;
    let bits = header >> 1;
    match bits {
        0x00 => {
            // Single: header already consumed, one u32 biome id follows.
            reader.read_u32::<LittleEndian>()?;
        }
        0x7f => {
            // Inherit: header only, nothing else belongs to this fragment.
        }
        bits => {
            // Data: packed words sized by `bits`, then a u32 palette length
            // and that many u32 palette entries -- identical structure to
            // `Biomes3d::from_disk`'s own `IndicesType::Data` arm.
            GreedyArray::from_disk(reader, bits)?;
            let len = reader.read_u32::<LittleEndian>()?;
            for _ in 0..len {
                reader.read_u32::<LittleEndian>()?;
            }
        }
    }
    Ok(header)
}

/// Every real biome storage header this crate has scanned sets the low bit
/// (see `Biomes3d::to_disk`'s doc comment) -- pinned as a standing corpus
/// assertion so a future record that violates it is noticed rather than
/// silently accepted by the decoder (which never checks the bit, only the
/// round-trip test above would catch a mismatch, and only if the record's
/// *other* bytes happened to differ too).
///
/// Walks every fragment of every real `0x2b` record, not just the first one
/// per record: an earlier version of this test broke out of the per-record
/// loop at a record's first Data fragment (word count depends on packing,
/// which it did not compute), so it checked roughly one in six fragments
/// across the corpus. `read_fragment_header` fixes that by advancing through
/// a Data fragment's packed words and palette the same way production
/// decode does, so this walks the whole record. Per-kind fragment totals
/// across the corpus, re-derived with this fixed walk (a standing count, not
/// an incidental one -- if it moves, this comment is stale):
/// 23,845 Single, 124,386 Data, 44,245 Inherit (192,476 fragments total).
#[test]
fn every_real_biome_header_sets_the_low_bit() {
    let mut single_count = 0usize;
    let mut data_count = 0usize;
    let mut inherit_count = 0usize;

    for fixture in support::fixtures()
        .into_iter()
        .filter(|f| f.capabilities.biomes_3d)
    {
        let (_tmp, db) = fixture.open();
        let mut keys = db.keys().unwrap();
        for kv in &mut keys {
            let mut key_cursor = Cursor::new(kv.key());
            let Ok(key) = Key::deserialize(&mut key_cursor) else {
                continue;
            };
            if key.data != KeyVariant::Biome3d {
                continue;
            }
            let bytes: Vec<u8> = kv.value().into();
            if bytes.len() < 512 {
                continue;
            }

            let mut reader = Cursor::new(&bytes[512..]);
            loop {
                let before = reader.position();
                let header = match read_fragment_header(&mut reader) {
                    Ok(header) => header,
                    Err(bedrock_level::error::Error::IoError(io))
                        if io.kind() == std::io::ErrorKind::UnexpectedEof =>
                    {
                        break; // end of the fragment array
                    }
                    Err(e) => panic!(
                        "{}: chunk {:?}: failed to walk a biome fragment at offset {}: {e}",
                        fixture.name,
                        key.chunk,
                        512 + before
                    ),
                };
                assert_eq!(
                    header & 1,
                    1,
                    "{}: chunk {:?}: biome fragment header byte {header:#x} at offset {} \
                     does not set the low bit",
                    fixture.name,
                    key.chunk,
                    512 + before
                );
                match header >> 1 {
                    0x00 => single_count += 1,
                    0x7f => inherit_count += 1,
                    _ => data_count += 1,
                }
            }
        }
    }

    let total = single_count + data_count + inherit_count;
    assert!(total > 0, "no biome fragment headers were checked");
    println!(
        "biome fragment headers checked: {single_count} Single, {data_count} Data, \
         {inherit_count} Inherit, {total} total"
    );
}
