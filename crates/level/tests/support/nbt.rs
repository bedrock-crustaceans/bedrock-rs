//! A small, shared primitive-typed NBT leaf generator, used by property
//! tests that need *some* varied NBT field content but have no structural
//! reason to prefer one primitive tag mix over another (`proptest_actor_ticks.rs`'s
//! bounded-depth record generator and `proptest_chunk_records.rs`'s
//! `MetadataDictionary` entry generator both built their own near-identical
//! version of this before it moved here).

use proptest::prelude::*;

/// One primitive-typed NBT value: `Byte`, `Short`, `Int`, `Long`, `Float`,
/// or a short printable `String`. Deliberately excludes `Double`,
/// `ByteArray`/`IntArray`/`LongArray`, `List`, and `Compound` -- callers
/// that need nested or list-shaped content build that themselves on top of
/// this leaf, the same way both original call sites did.
pub fn nbt_leaf() -> impl Strategy<Value = nbtx::Value> {
    prop_oneof![
        any::<i8>().prop_map(nbtx::Value::Byte),
        any::<i16>().prop_map(nbtx::Value::Short),
        any::<i32>().prop_map(nbtx::Value::Int),
        any::<i64>().prop_map(nbtx::Value::Long),
        any::<f32>().prop_map(nbtx::Value::Float),
        "[a-zA-Z0-9 ]{0,16}".prop_map(|s| nbtx::Value::String(s.into())),
    ]
}
