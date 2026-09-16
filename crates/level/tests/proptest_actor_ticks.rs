//! Round-trip and malformed-input property tests for actor storage
//! (`digp` digest, `actorprefix` records, the legacy inline `0x32` entity
//! list, `0x41` ActorDigestVersion -- `crates/level/src/actor.rs`) and the
//! `0x33`/`0x3a` tick-list records (`crates/level/src/ticks.rs`).

mod support;

use bedrock_level::actor::{
    ActorStorageKey, decode_actor_digest, decode_actor_digest_version, decode_actor_record,
    decode_legacy_entities, encode_actor_digest, encode_actor_digest_version, encode_actor_record,
    encode_legacy_entities,
};
use bedrock_level::ticks::{decode_tick_list, encode_tick_list};
use proptest::prelude::*;
use support::nbt::nbt_leaf;

/// A small, depth-bounded NBT value: `nbt_leaf()` primitives, plus one level
/// of `Compound` and homogeneous `List` nesting. Enough to exercise
/// order-preserving compound re-encoding and list-of-compound content
/// (the actual shape both actor records and tick entries use) without a
/// full unbounded recursive generator.
fn nbt_compound(depth: u32) -> impl Strategy<Value = nbtx::Value> {
    let field_name = "[A-Za-z]{1,10}";
    if depth == 0 {
        proptest::collection::vec((field_name, nbt_leaf()), 0..5)
            .prop_map(|fields| {
                let mut c = nbtx::Compound::new();
                for (name, value) in fields {
                    c.insert(name.into(), value);
                }
                nbtx::Value::Compound(c)
            })
            .boxed()
    } else {
        proptest::collection::vec((field_name, nbt_value(depth - 1)), 0..4)
            .prop_map(|fields| {
                let mut c = nbtx::Compound::new();
                for (name, value) in fields {
                    c.insert(name.into(), value);
                }
                nbtx::Value::Compound(c)
            })
            .boxed()
    }
}

/// A homogeneous `List` -- every real NBT list carries one element type, and
/// `nbtx` derives the on-disk element tag from the first element, so a
/// mixed-type list would not round-trip through this crate's own encoder
/// either; the generator only ever produces homogeneous ones.
fn nbt_list(depth: u32) -> impl Strategy<Value = nbtx::Value> {
    nbt_leaf_or_compound(depth)
        .prop_flat_map(|sample| {
            let strat = match sample {
                nbtx::Value::Byte(_) => any::<i8>().prop_map(nbtx::Value::Byte).boxed(),
                nbtx::Value::Short(_) => any::<i16>().prop_map(nbtx::Value::Short).boxed(),
                nbtx::Value::Int(_) => any::<i32>().prop_map(nbtx::Value::Int).boxed(),
                nbtx::Value::Long(_) => any::<i64>().prop_map(nbtx::Value::Long).boxed(),
                nbtx::Value::Float(_) => any::<f32>().prop_map(nbtx::Value::Float).boxed(),
                nbtx::Value::String(_) => "[a-zA-Z0-9]{0,10}"
                    .prop_map(|s| nbtx::Value::String(s.into()))
                    .boxed(),
                _ => Just(sample.clone()).boxed(),
            };
            proptest::collection::vec(strat, 0..4)
        })
        .prop_map(nbtx::Value::List)
}

fn nbt_leaf_or_compound(depth: u32) -> impl Strategy<Value = nbtx::Value> {
    if depth == 0 {
        nbt_leaf().boxed()
    } else {
        prop_oneof![nbt_leaf(), nbt_compound(depth - 1)].boxed()
    }
}

fn nbt_value(depth: u32) -> impl Strategy<Value = nbtx::Value> {
    if depth == 0 {
        nbt_leaf().boxed()
    } else {
        prop_oneof![
            2 => nbt_leaf(),
            1 => nbt_compound(depth - 1),
            1 => nbt_list(depth - 1),
        ]
        .boxed()
    }
}

/// A top-level record value is always a `Compound` -- the root tag every
/// real actor/tick record uses.
fn nbt_compound_root() -> impl Strategy<Value = nbtx::Value> {
    nbt_compound(2)
}

// ---------------------------------------------------------------------
// digp (actor digest)
// ---------------------------------------------------------------------

proptest! {
    /// Any flat list of 8-byte storage keys round-trips byte-identically --
    /// there is no invalid encoding for this record short of a length that
    /// isn't a multiple of 8.
    #[test]
    fn actor_digest_round_trips(
        keys in proptest::collection::vec(proptest::array::uniform8(any::<u8>()), 0..20),
    ) {
        let keys: Vec<ActorStorageKey> = keys;
        let bytes = encode_actor_digest(&keys);
        prop_assert_eq!(decode_actor_digest(&bytes).unwrap(), keys);
    }

    /// A length that is not a multiple of 8 is rejected, never panics.
    #[test]
    fn actor_digest_decode_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..40)) {
        let result = decode_actor_digest(&bytes);
        if !bytes.len().is_multiple_of(8) {
            prop_assert!(result.is_err());
        }
    }

    // ---------------------------------------------------------------------
    // actorprefix records and the legacy inline 0x32 entity list
    // ---------------------------------------------------------------------

    /// Any NBT compound round-trips byte-identically through
    /// `decode_actor_record`/`encode_actor_record`.
    #[test]
    fn actor_record_round_trips(value in nbt_compound_root()) {
        let bytes = encode_actor_record(&value).unwrap();
        prop_assert_eq!(decode_actor_record(&bytes).unwrap(), value);
    }

    /// Arbitrary bytes never panic `decode_actor_record`.
    #[test]
    fn actor_record_decode_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..80)) {
        let _ = decode_actor_record(&bytes);
    }

    /// Any list of NBT compounds concatenates and round-trips byte-
    /// identically through the legacy `0x32` codec.
    #[test]
    fn legacy_entities_round_trips(values in proptest::collection::vec(nbt_compound_root(), 0..4)) {
        let bytes = encode_legacy_entities(&values).unwrap();
        prop_assert_eq!(decode_legacy_entities(&bytes).unwrap(), values);
    }

    /// Arbitrary bytes never panic `decode_legacy_entities`.
    #[test]
    fn legacy_entities_decode_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..80)) {
        let _ = decode_legacy_entities(&bytes);
    }

    // ---------------------------------------------------------------------
    // ActorDigestVersion (0x41)
    // ---------------------------------------------------------------------

    /// Any single byte round-trips.
    #[test]
    fn actor_digest_version_round_trips(version in any::<u8>()) {
        let bytes = encode_actor_digest_version(version);
        prop_assert_eq!(decode_actor_digest_version(&bytes).unwrap(), version);
    }

    /// Any length other than exactly one byte is rejected, never panics.
    #[test]
    fn actor_digest_version_decode_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..8)) {
        let result = decode_actor_digest_version(&bytes);
        if bytes.len() != 1 {
            prop_assert!(result.is_err());
        }
    }

    // ---------------------------------------------------------------------
    // TickList (0x33 PendingTicks / 0x3a RandomTicks)
    // ---------------------------------------------------------------------

    /// Any NBT compound round-trips byte-identically through the tick-list
    /// codec -- `TickList` wraps the decoded value verbatim, so this holds
    /// regardless of whether the value matches the real `currentTick`/
    /// `tickList` shape.
    #[test]
    fn tick_list_round_trips(value in nbt_compound_root()) {
        let bytes = nbtx::to_le_bytes(&value).unwrap();
        let list = decode_tick_list(&bytes).unwrap();
        prop_assert_eq!(encode_tick_list(&list).unwrap(), bytes);
    }

    /// Arbitrary bytes never panic `decode_tick_list`.
    #[test]
    fn tick_list_decode_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..80)) {
        let _ = decode_tick_list(&bytes);
    }
}
