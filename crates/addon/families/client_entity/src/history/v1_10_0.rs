//! Format versions 1.10.0 up to 1.26.0, and the step to 1.26.0.

use bedrock_addon_core::id::EntityId;
use bedrock_addon_shared_visual::NoOwn;
use bedrock_addon_shared_visual::history::v1_10_0::Described;

use super::ClientEntityFamily;

/// A client entity document of format version 1.10.0 up to 1.26.0.
pub type ClientEntity = Described<ClientEntityFamily>;

/// `description` at format versions 1.10.0 up to 1.26.0: as the newest
/// [`Description`](super::super::Description), except that each `scripts.initialize` and
/// `scripts.pre_animation` element is a Molang value of its own and `held_item_scale` and
/// `scripts.hide_held_items` do not exist.
pub type Description = bedrock_addon_shared_visual::history::v1_10_0::Description<EntityId, NoOwn>;
