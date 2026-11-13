//! Loading a spawn group in the spawn tests.

use bedrock_addon::document::{Content, Document, Rejected};
use bedrock_addon::families::spawn_groups::SpawnGroup;
use bedrock_addon::storage::PackPath;

use super::common::context;

/// Loads `text` as the spawn group `definitions/spawn_groups/x.json`.
pub fn load_group(text: &str) -> Result<Document<SpawnGroup>, Rejected> {
    SpawnGroup::parse(
        &PackPath::from_static("definitions/spawn_groups/x.json"),
        text.as_bytes(),
        &context(),
    )
}
