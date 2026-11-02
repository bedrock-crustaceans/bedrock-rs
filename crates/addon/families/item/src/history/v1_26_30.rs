//! The 1.26.30 format (`v1_26_30::ItemDocument`, the first modern slice, `FIRST_MODERN_SLICE`):
//! the members of [`crate::Item`], whose four sound members
//! (`use_modifiers.start_sound`, `swing_sounds.*`, `durability_thresholds[].sound_event`) bind
//! a `LevelSoundEvent`. The step to 1.26.50 turns each into its name.

use bedrock_addon_core::document::{ComponentMap, Extra};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, VersionStruct, Versioned};

use crate::component::ItemComponent;
use crate::{ItemDeprecatedData, ItemDescription, LegacyItemData};
use bedrock_addon_core::diagnostic::Diagnostics;

/// An item document of the 1.26.30 format (also 1.26.31–1.26.49): [`crate::Item`]'s
/// members, the sound members holding level sound event names.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    /// The identifier and creative-menu placement (JSON `description`).
    pub description: ItemDescription,
    /// The components, in file order (JSON `components`).
    pub components: ComponentMap<ItemComponent>,
    /// Data older formats moved out of the components (the step from 1.26.0 fills it; 1.26.30
    /// binds the side-car only as deprecated names).
    pub deprecated: ItemDeprecatedData,
    /// The legacy `events` object, kept as written (JSON `events`).
    pub events: Option<Value>,
    /// What vanilla's legacy reader takes from the same document; not written.
    pub legacy: LegacyItemData,
    /// Unknown members of the item object.
    pub extra: Extra,
}

impl Versioned for Item {
    const VERSION: FormatVersion = FormatVersion::new(1, 26, 30);
}

impl VersionStruct for Item {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let n = crate::write::parse_newest(payload, Self::VERSION, cx, d)?;
        Some(Item {
            description: n.description,
            components: n.components,
            deprecated: n.deprecated,
            events: n.events,
            legacy: n.legacy,
            extra: n.extra,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        crate::write::newest_value(
            &self.description,
            &self.components,
            &self.deprecated,
            self.events.as_ref(),
            &self.legacy,
            &self.extra,
        )
    }
}

impl MolangFields for Item {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.nested("components", &mut self.components);
        self.legacy.expressions.visit_molang(visitor);
    }
}
