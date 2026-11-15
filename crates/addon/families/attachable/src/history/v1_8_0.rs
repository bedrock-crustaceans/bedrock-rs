//! Format versions 1.8.0 up to 1.10.0, and the step to 1.10.0 (the shared one: vanilla's step
//! moves nothing of an attachable, whose `animation_controllers` stay a member 1.10.0 does not
//! have).

use bedrock_addon_core::id::AttachableId;
use bedrock_addon_shared_visual::history::v1_8_0::Described;

use super::AttachableFamily;

/// An attachable document of format version 1.8.0 up to 1.10.0.
pub type Attachable = Described<AttachableFamily>;

/// `description` at format versions 1.8.0 up to 1.10.0: animation controllers are a list of
/// their own (which an attachable cannot carry on to 1.10.0), render controllers are names
/// only, and `queryable_geometry`, `held_item_scale` and `item` do not exist.
pub type Description = bedrock_addon_shared_visual::history::v1_8_0::Description<AttachableId>;
