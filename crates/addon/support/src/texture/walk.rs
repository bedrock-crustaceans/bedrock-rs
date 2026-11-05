//! The walk of the JSON files of the texture families.

use bedrock_addon_core::load::{Accept, AppDataPlace, Walk};
use bedrock_addon_core::pack_identity::PackPath;

/// The JSON files `accept` takes anywhere in the pack, lowest pack first; of the files an
/// active subpack shadows, those `shadowed` takes.
pub const fn walk(accept: fn(&PackPath) -> bool, shadowed: Option<fn(&PackPath) -> bool>) -> Walk {
    let walk = Walk::json_tree_in(&[""])
        .accept(Accept::Fn(accept))
        .app_data(&[], AppDataPlace::Before);
    match shadowed {
        Some(shadowed) => walk.shadowed(shadowed),
        None => walk,
    }
}
