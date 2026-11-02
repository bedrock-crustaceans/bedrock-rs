//! Constructors of the aim-assist documents.

use super::{AimAssistCategory, AimAssistPreset, Priorities};
use bedrock_addon_core::id::{AimAssistCategoryId, AimAssistPresetId};

impl AimAssistPreset {
    /// A preset named `identifier` that sets nothing.
    #[must_use]
    pub fn new(identifier: AimAssistPresetId) -> Self {
        Self {
            identifier,
            exclusion_settings: None,
            liquid_targeting_list: None,
            item_settings: None,
            default_item_settings: None,
            hand_settings: None,
        }
    }
}

impl AimAssistCategory {
    /// A category named `name` with `priorities`.
    #[must_use]
    pub fn new(name: AimAssistCategoryId, priorities: Priorities) -> Self {
        Self { name, priorities }
    }
}
