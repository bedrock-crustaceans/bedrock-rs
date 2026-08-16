//! Pins the widened `Dimension`/`Key::deserialize` behavior from the
//! "accept dimension ids outside the vanilla three" task against every
//! imported fixture. All fixtures are vanilla worlds, so none of them
//! carries an add-on dimension -- the widened path (`Dimension::Other`) has
//! no real-world coverage in this corpus, which is confirmed by scan here
//! rather than assumed. What the corpus *can* pin is that widening the
//! accepted dimension-id range changed nothing about which keys parse as
//! chunk keys on these worlds, and that the real string keys the old
//! structural check was designed to keep out (`~local_player` chief among
//! them, since it happens to be exactly 13 bytes) still fall through
//! correctly now that the id check accepts far more values than before.

mod support;

use std::io::Cursor;

use bedrock_level::key::{Key, KeyVariant};
use bedrock_shared::world::dimension::Dimension;

/// Re-implements just enough of the *pre-widening* dimension-id check (ids
/// 1, 2, or 3 only, matching what `known_dimension_id` accepted before this
/// task) to recompute how many keys a fixture would have parsed as chunk or
/// `digp` keys under the old, narrower bound. Deliberately independent of
/// `bedrock_level::key` so the comparison in
/// `widening_does_not_change_which_keys_parse_on_any_fixture` is not
/// circular -- it does not call the widened code to establish the
/// pre-widening baseline. Covers both key shapes `Key::deserialize` accepts
/// (the tag-byte chunk-key shape and the `digp` shape), since both share the
/// same dimension-id check and both must be counted for the "before" total
/// to be comparable to the widened code's "after" total.
fn parses_as_chunk_key_under_the_old_narrow_bound(key_bytes: &[u8]) -> bool {
    use byteorder::{LittleEndian, ReadBytesExt};

    const KNOWN_TAGS: &[u8] = &[
        0x2b, 0x2c, 0x2d, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x35, 0x36, 0x38, 0x39, 0x3a, 0x3b, 0x3c,
        0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x76, 0x77,
    ];

    let len = key_bytes.len();

    if matches!(len, 12 | 16) && key_bytes.starts_with(bedrock_level::key::ACTOR_DIGEST_PREFIX) {
        let mut cursor = Cursor::new(&key_bytes[4..]);
        // x, z
        if cursor.read_i32::<LittleEndian>().is_err() || cursor.read_i32::<LittleEndian>().is_err()
        {
            return false;
        }
        return if len == 16 {
            matches!(cursor.read_i32::<LittleEndian>(), Ok(1..=3))
        } else {
            true
        };
    }

    if !matches!(len, 9 | 10 | 13 | 14) {
        return false;
    }

    let mut cursor = Cursor::new(key_bytes);
    // x, z
    if cursor.read_i32::<LittleEndian>().is_err() || cursor.read_i32::<LittleEndian>().is_err() {
        return false;
    }

    let dimensioned = len == 13 || len == 14;
    if dimensioned {
        let Ok(id) = cursor.read_i32::<LittleEndian>() else {
            return false;
        };
        if !matches!(id, 1..=3) {
            return false;
        }
    }

    let has_index = len == 10 || len == 14;
    let Ok(tag) = cursor.read_u8() else {
        return false;
    };

    if tag == 0x2f {
        has_index && cursor.read_i8().is_ok()
    } else if has_index {
        false
    } else {
        KNOWN_TAGS.contains(&tag)
    }
}

/// No vanilla fixture contains a chunk key whose dimension id falls in the
/// range this task newly accepts (outside 0-3). This is the corpus fact
/// that justifies calling the widened path untested-by-fixture in the task
/// report: it is confirmed absent here, not merely assumed absent.
#[test]
fn no_fixture_contains_an_add_on_dimension_id() {
    let mut total_other = 0usize;

    for fixture in support::fixtures() {
        let (_tmp, db) = fixture.open();
        let mut keys = db
            .keys()
            .unwrap_or_else(|e| panic!("{}: failed to iterate keys: {e}", fixture.name));

        for kv in &mut keys {
            let key_bytes = kv.key();
            let mut cursor = Cursor::new(key_bytes.as_ref());
            if let Ok(key) = Key::deserialize(&mut cursor)
                && matches!(key.dimension, Dimension::Other(_))
            {
                total_other += 1;
                eprintln!(
                    "{}: unexpected add-on dimension {:?} on key {:?}",
                    fixture.name, key.dimension, key.data
                );
            }
        }
    }

    assert_eq!(
        total_other, 0,
        "no imported fixture is expected to carry an add-on dimension id; \
         if this fires, the fixture set gained real add-on coverage and the \
         corpus claim in this test's doc comment is now stale"
    );
}

/// Widening the accepted dimension-id range must not change which keys
/// parse as chunk keys (or `digp` keys) on a corpus with no add-on
/// dimensions: every key that parsed before must still parse, and (per the
/// previous test) nothing new starts parsing that wasn't already going to
/// parse under the pre-widening bound, since no fixture key's dimension
/// field falls in the newly accepted range. This test pins that equality
/// directly, per fixture, rather than relying on the two other tests'
/// conclusions holding it up.
#[test]
fn widening_does_not_change_which_keys_parse_on_any_fixture() {
    for fixture in support::fixtures() {
        let (_tmp, db) = fixture.open();
        let mut keys = db
            .keys()
            .unwrap_or_else(|e| panic!("{}: failed to iterate keys: {e}", fixture.name));

        let mut before = 0usize;
        let mut after = 0usize;

        for kv in &mut keys {
            let key_bytes = kv.key();

            if parses_as_chunk_key_under_the_old_narrow_bound(key_bytes.as_ref()) {
                before += 1;
            }

            let mut cursor = Cursor::new(key_bytes.as_ref());
            // Match the same "structural chunk/digp key" surface the old-bound
            // helper checks -- excludes the `LocalPlayer` string-key fallback,
            // which neither the old nor the new dimension-id bound touches.
            if matches!(Key::deserialize(&mut cursor), Ok(key) if key.data != KeyVariant::LocalPlayer)
            {
                after += 1;
            }
        }

        assert_eq!(
            before, after,
            "{}: widened dimension-id acceptance changed the number of keys \
             parsing as chunk/digp keys ({before} before, {after} after) -- \
             expected no change on a fixture with no add-on dimension ids",
            fixture.name
        );
        assert!(
            before > 0,
            "{}: fixture yielded no chunk keys at all, so this comparison is vacuous",
            fixture.name
        );
    }
}

/// `~local_player` is exactly 13 bytes -- one of the four structurally
/// chunk-key-shaped lengths -- so it is the sharpest real-world check that
/// widening the accepted dimension-id range did not also widen away the
/// tag-byte check that keeps it falling through to the string-key path.
/// Every fixture has exactly one such key (`db.rs`'s single-player local
/// storage), so this runs against real on-disk bytes, not a synthetic
/// buffer.
#[test]
fn local_player_key_still_falls_back_on_every_fixture() {
    let mut seen = 0usize;

    for fixture in support::fixtures() {
        let (_tmp, db) = fixture.open();
        let mut keys = db
            .keys()
            .unwrap_or_else(|e| panic!("{}: failed to iterate keys: {e}", fixture.name));

        for kv in &mut keys {
            let key_bytes = kv.key();
            if key_bytes.as_ref() != bedrock_level::key::LOCAL_PLAYER.as_bytes() {
                continue;
            }

            seen += 1;
            let mut cursor = Cursor::new(key_bytes.as_ref());
            let key = Key::deserialize(&mut cursor)
                .unwrap_or_else(|e| panic!("{}: ~local_player failed to parse: {e}", fixture.name));
            assert_eq!(
                key.data,
                KeyVariant::LocalPlayer,
                "{}: ~local_player was misparsed as a chunk key",
                fixture.name
            );
        }
    }

    assert!(
        seen > 0,
        "no fixture contained a ~local_player key, so this test proved nothing"
    );
}
