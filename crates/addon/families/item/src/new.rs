//! Constructing items.

use bedrock_addon_core::document::{ComponentMap, Extra};
use bedrock_addon_core::id::ItemId;
use bedrock_addon_core::shared::{CreativeCategory, MenuCategory};
use bedrock_addon_core::version::known;
use indexmap::IndexMap;

use super::history::write_component;
use super::load::gate;
use super::{Item, ItemDeprecatedData, ItemDescription, LegacyItemData};
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::load::LoadContext;

impl Item {
    /// The stack size vanilla uses: `minecraft:max_stack_size`, else 1 for an item worn in an
    /// armour slot (as of 1.26.45; which slots count is inferred: all but the off hand), else
    /// 64; for the legacy item format, its `minecraft:max_stack_size`.
    pub fn max_stack_size(&self) -> i32 {
        use super::component::{MaxStackSize, Wearable, WearableSlot};
        if let Some(format) = &self.legacy.format_item {
            return format.max_stack_size;
        }
        if let Some(size) = self.components.get::<MaxStackSize>() {
            return size.value;
        }
        match self.components.get::<Wearable>() {
            Some(wearable) if wearable.slot != WearableSlot::Offhand => 1,
            _ => 64,
        }
    }

    /// An item with this identifier and no components, shown only to commands (the creative
    /// category `none`).
    ///
    /// Written before it has a component, it is a document of the legacy item format (the
    /// game reads every document without a registered component so, at any version), and
    /// reads back as a legacy item; a `menu_category` set on it then has no text
    /// (`EncodeError::NotRepresentable` at `description/menu_category`), and neither has a
    /// custom component, a key no table knows or a side-car member set on it alone
    /// (`NotRepresentable` at `components`). With a registered component it is written in the
    /// newest format.
    pub fn new(identifier: ItemId) -> Item {
        Item {
            description: ItemDescription {
                identifier,
                menu_category: MenuCategory {
                    category: Some(CreativeCategory::ItemCommandOnly),
                    ..MenuCategory::default()
                },
                menu_category_presence: super::MenuCategoryPresence::Absent,
                extra: Extra::default(),
            },
            components: ComponentMap::new(),
            deprecated: ItemDeprecatedData::default(),
            events: None,
            legacy: LegacyItemData::default(),
            extra: Extra::default(),
        }
    }

    /// Runs the family's rules as if the item were loaded at the newest format version from a
    /// pack with `cx`: the component gates, the identifier rule and the validators.
    pub fn validate(&self, cx: &LoadContext) -> Diagnostics {
        let version = known::V1_26_50;
        let at = JsonPath::new().child("minecraft:item");
        let mut d = Diagnostics::default();
        let mut components = IndexMap::new();
        for (name, component) in super::history::from_component_map(&self.components) {
            if let Ok(json) = write_component(&component) {
                components.insert(name, json);
            }
        }
        let path = at.child("components");
        let _ = gate(
            &components,
            self.description.identifier.as_str(),
            version,
            cx,
            &path,
            &mut d,
        );
        super::validate::validate(self, version, cx, &at, &mut d);
        d
    }
}
