//! The walk of the JSON files of the texture families.

use bedrock_addon_core::load::{AppDataPlace, ListDepth, Walk, WalkOrder};
use bedrock_addon_core::pack_identity::PackPath;

/// The JSON files `accept` takes anywhere in the pack, lowest pack first; of the files an
/// active subpack shadows, those `shadowed` takes.
pub const fn walk(accept: fn(&PackPath) -> bool, shadowed: Option<fn(&PackPath) -> bool>) -> Walk {
    Walk {
        directories: &[""],
        depth: ListDepth::Recursive,
        accept,
        order: WalkOrder::BottomUp,
        directory_major: false,
        app_data: &[],
        app_data_place: AppDataPlace::Before,
        in_pack: None,
        shadowed,
        read_error: None,
    }
}
