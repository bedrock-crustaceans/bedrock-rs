//! Format 1.21.120: `ambient_sounds` with `{asset, chance}` additions and underwater sounds;
//! the last format read from the whole document, envelope included.

use super::ClientBiomePayload;
use crate::ClientBiome as Newest;
use crate::component::ClientBiomeComponent;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

/// A client biome of format `[1.21.120, 1.21.130)`. A version between the two (1.21.125)
/// is read by this struct.
pub type ClientBiome = Legacy<ClientBiomePayload<ClientBiomeComponent>, 1, 21, 120>;

/// The step to 1.21.130: `description` and `components`
/// move into the payload struct; `format_version` stays in the envelope, and unknown members
/// of the root (reported when the document was read) have no place there.
impl LegacyStep<1, 21, 120> for ClientBiomePayload<ClientBiomeComponent> {
    type Next = Newest;

    fn step(this: ClientBiome, _: &mut UpgradeCx<'_>) -> Newest {
        let biome = this.0;
        Newest {
            description: biome.description,
            components: biome.components,
        }
    }
}
