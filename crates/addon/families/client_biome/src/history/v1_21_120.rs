//! Format 1.21.120: `ambient_sounds` with `{asset, chance}` additions and underwater sounds;
//! the last format read from the whole document, envelope included.

use super::legacy_document;
use crate::ClientBiome as Newest;
use crate::component::ClientBiomeComponent;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};

legacy_document! {
    /// A client biome of format `[1.21.120, 1.21.130)`. A version between the two (1.21.125)
    /// is read by this struct.
    version = (1, 21, 120), set = ClientBiomeComponent
}

/// The step to 1.21.130: `description` and `components`
/// move into the payload struct; `format_version` stays in the envelope, and unknown members
/// of the root (reported when the document was read) have no place there.
impl Upgrade for ClientBiome {
    type Next = Newest;

    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Newest {
        Newest {
            description: self.description,
            components: self.components,
        }
    }
}
