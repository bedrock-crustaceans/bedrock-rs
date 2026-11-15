//! Format versions 1.10.0 up to 1.26.0, and the step to 1.26.0.

use bedrock_addon_core::id::AttachableId;
use bedrock_addon_shared_visual::history::v1_10_0::Described;

use super::AttachableFamily;
use crate::ItemMember;

/// An attachable document of format version 1.10.0 up to 1.26.0.
pub type Attachable = Described<AttachableFamily>;

/// `description` at format versions 1.10.0 up to 1.26.0: as the newest
/// [`Description`](super::super::Description), except that each `scripts.initialize` and
/// `scripts.pre_animation` element is a Molang value of its own and `held_item_scale` and
/// `scripts.hide_held_items` do not exist.
pub type Description =
    bedrock_addon_shared_visual::history::v1_10_0::Description<AttachableId, ItemMember>;
