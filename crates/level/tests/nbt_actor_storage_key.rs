//! Follows up on `nbt_string_encoding.rs` against real world data, settling
//! the motivating case TODO.md's NBT-string task names: the Bedrock actor
//! storage key.
//!
//! `internalComponents.EntityStorageKeyComponent.StorageKey` is an NBT
//! *string* tag, present in real `actorprefix` records, and its payload is
//! eight bytes of raw binary that is frequently not valid UTF-8 -- this is
//! not a theoretical concern, it is what the game writes. These tests pin
//! that against the `v1_18_30` fixture, then confirm the resulting ergonomic
//! split: a `#[derive(Facet)]` struct field typed `String` rejects such a
//! payload, `BString` carries it through, and a whole record carrying one
//! round-trips byte for byte through the crate's raw `Value` path.

mod support;

use std::io::Cursor;

use bstr::{BStr, BString};
use facet::Facet;

/// Reads `internalComponents.EntityStorageKeyComponent` out of a decoded
/// actor record, if present in that shape.
fn entity_storage_key_component(v: &nbtx::Value) -> Option<&nbtx::Compound> {
    let nbtx::Value::Compound(root) = v else {
        return None;
    };
    let nbtx::Value::Compound(internal_components) = root.get(BStr::new("internalComponents"))?
    else {
        return None;
    };
    let nbtx::Value::Compound(component) =
        internal_components.get(BStr::new("EntityStorageKeyComponent"))?
    else {
        return None;
    };
    Some(component)
}

fn storage_key_bytes(component: &nbtx::Compound) -> Option<&[u8]> {
    match component.get(BStr::new("StorageKey"))? {
        nbtx::Value::String(s) => Some(s.as_slice()),
        _ => None,
    }
}

/// Formats bytes as space-separated hex, for failure messages that need to
/// name a specific record (`actorprefix` records have no chunk position to
/// print, unlike `fixtures_test.rs`/`data2d.rs`; the LevelDB key is the
/// closest equivalent identifier).
fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// A real `actorprefix` record whose `StorageKey` string tag is not valid
/// UTF-8: the record's LevelDB key (to identify it in a failure message),
/// the raw NBT bytes exactly as stored, the decoded
/// `EntityStorageKeyComponent` compound, and the offending `StorageKey`
/// bytes.
struct NonUtf8StorageKeyRecord {
    key: Vec<u8>,
    record: Vec<u8>,
    component: nbtx::Compound,
    storage_key: Vec<u8>,
}

/// Scans every `actorprefix` record in the `v1_18_30` fixture for the first
/// one whose `StorageKey` string tag is not valid UTF-8, and decodes it.
/// Shared by every test below so the find-decode-extract sequence lives in
/// one place.
fn find_nonutf8_storage_key_record() -> NonUtf8StorageKeyRecord {
    let fixture = support::fixtures()
        .into_iter()
        .find(|f| f.name == "v1_18_30")
        .expect("v1_18_30 fixture is part of the checked-in fixture set");

    let (_tmp, db) = fixture.open();
    let mut keys = db.keys().expect("iterate database");
    for kv in (&mut keys).by_ref() {
        let key = kv.key();
        if !key.starts_with(b"actorprefix") {
            continue;
        }
        let value = kv.value();
        let mut cur = Cursor::new(value.as_ref());
        let Ok(decoded) = nbtx::from_le_bytes::<nbtx::Value>(&mut cur) else {
            continue;
        };
        let Some(component) = entity_storage_key_component(&decoded) else {
            continue;
        };
        let Some(storage_key) = storage_key_bytes(component) else {
            continue;
        };
        if std::str::from_utf8(storage_key).is_err() {
            return NonUtf8StorageKeyRecord {
                key: key.as_ref().to_vec(),
                record: value.as_ref().to_vec(),
                component: component.clone(),
                storage_key: storage_key.to_vec(),
            };
        }
    }
    panic!("v1_18_30 has at least one actorprefix record with a non-UTF-8 StorageKey");
}

/// The motivating case itself: a real `actorprefix` record whose
/// `StorageKey` string tag holds eight bytes of raw binary that fail UTF-8
/// validation. This is what settles the open question in TODO.md -- the
/// storage key does live in a string tag, inside NBT, not only in LevelDB
/// key bytes and `digp` values.
#[test]
fn storage_key_is_a_real_nonutf8_string_tag_in_practice() {
    let found = find_nonutf8_storage_key_record();
    let key_hex = hex(&found.key);
    assert_eq!(
        found.storage_key.len(),
        8,
        "actorprefix {key_hex}: the storage key is always 8 raw bytes"
    );
    assert!(
        std::str::from_utf8(&found.storage_key).is_err(),
        "actorprefix {key_hex}: expected a non-UTF-8 StorageKey"
    );
}

/// A `#[derive(Facet)]` struct field typed `String` still validates UTF-8:
/// decoding a real `StorageKey` payload through it fails, exactly as
/// `nbt_string_encoding.rs` documents for synthetic payloads.
#[test]
fn typed_string_field_errors_on_a_real_storage_key() {
    #[derive(Facet, Debug)]
    #[facet(nbtx::allow_unknown_fields)]
    struct AsString {
        #[facet(rename = "StorageKey")]
        storage_key: String,
    }

    let found = find_nonutf8_storage_key_record();
    let key_hex = hex(&found.key);

    let result: Result<AsString, _> = nbtx::from_value(nbtx::Value::Compound(found.component));
    let err = result.expect_err(&format!(
        "actorprefix {key_hex}: a String field must reject non-UTF-8 bytes"
    ));
    // Confirms this is the UTF-8 validation rejecting the payload, not some
    // unrelated decode failure.
    let msg = err.to_string();
    assert!(
        msg.to_lowercase().contains("utf-8") || msg.to_lowercase().contains("utf8"),
        "actorprefix {key_hex}: expected a UTF-8 validation error, got: {msg}"
    );
}

/// The same field typed `BString` (what `Value::String` carries) accepts the
/// payload unchanged -- this is the "carries it through" side of the gap.
#[test]
fn bstring_field_carries_a_real_storage_key_through() {
    #[derive(Facet, Debug)]
    #[facet(nbtx::allow_unknown_fields)]
    struct AsBString {
        #[facet(rename = "StorageKey")]
        storage_key: BString,
    }

    let found = find_nonutf8_storage_key_record();
    let key_hex = hex(&found.key);

    let result: AsBString = nbtx::from_value(nbtx::Value::Compound(found.component))
        .unwrap_or_else(|e| {
            panic!("actorprefix {key_hex}: a BString field must accept the payload: {e}")
        });
    assert_eq!(
        result.storage_key.as_slice(),
        found.storage_key.as_slice(),
        "actorprefix {key_hex}: BString field must carry the StorageKey bytes through unchanged"
    );
}

/// A whole real `actorprefix` record carrying a non-UTF-8 `StorageKey`
/// round-trips byte for byte through the crate's raw `Value` path: decode,
/// re-encode, and the bytes on the wire come back unchanged. This is what
/// makes the raw layer's byte-identity guarantee (architecture decision 3)
/// hold for this record shape specifically, not just in the abstract.
#[test]
fn a_real_storage_key_record_round_trips_byte_identically() {
    let found = find_nonutf8_storage_key_record();
    let key_hex = hex(&found.key);

    let mut cur = Cursor::new(found.record.as_slice());
    let decoded: nbtx::Value = nbtx::from_le_bytes(&mut cur)
        .unwrap_or_else(|e| panic!("actorprefix {key_hex}: failed to decode: {e}"));
    let re_encoded = nbtx::to_le_bytes(&decoded)
        .unwrap_or_else(|e| panic!("actorprefix {key_hex}: failed to re-encode: {e}"));

    assert_eq!(
        re_encoded, found.record,
        "actorprefix {key_hex}: decode/encode must reproduce the record byte for byte"
    );
}
