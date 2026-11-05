//! The family's types and their constructors.

use bedrock_addon_core::document::ComponentMap;
use bedrock_addon_core::id::ItemId;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::shared::{CreativeCategory, MenuCategory};
use bedrock_addon_core::version::known;
use indexmap::IndexMap;

use super::component::ItemComponent;
use super::history::gate;
use super::{ItemDeprecatedData, LegacyItemData};
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::load::LoadContext;

/// An item document of a modern format: the members the newest struct ([`Item`]), the beta
/// struct ([`Beta`]) and the 1.26.30 struct ([`history::v1_26_30::Item`](crate::history::v1_26_30::Item))
/// share, one struct per format version `V` ([`pack`](crate::history::pack)ed; [`BETA`] for the
/// beta struct).
///
/// JSON: `{"format_version": …, "minecraft:item": {"description": …, "components": {…},
/// "__deprecated_components"?: …, "events"?: …}}`.
#[derive(Clone, Debug, PartialEq)]
pub struct ModernItem<const V: u32> {
    /// The identifier and creative-menu placement (JSON `description`).
    pub description: ItemDescription,
    /// The components, in file order (JSON `components`).
    pub components: ComponentMap<ItemComponent>,
    /// Data older formats moved out of the components (JSON `__deprecated_components`).
    pub deprecated: ItemDeprecatedData,
    /// The legacy `events` object, kept as written (JSON `events`).
    pub events: Option<Value>,
    /// What vanilla's legacy reader takes from the same document; not written.
    pub legacy: LegacyItemData,
}

/// The [`ModernItem`] version of the beta struct ([`BETA_VERSION`](bedrock_addon_core::versioning::BETA_VERSION),
/// whose parts do not [`pack`](crate::history::pack)).
#[doc(hidden)]
pub const BETA: u32 = u32::MAX;

/// The format version a [`ModernItem`] version stands for.
pub(crate) const fn modern_version(packed: u32) -> bedrock_addon_core::version::FormatVersion {
    if packed == BETA {
        bedrock_addon_core::versioning::BETA_VERSION
    } else {
        crate::history::unpack(packed)
    }
}

impl<const V: u32> ModernItem<V> {
    /// The same members as the struct of another modern format: a step that moves every
    /// member unchanged.
    pub(crate) fn into_format<const W: u32>(self) -> ModernItem<W> {
        ModernItem {
            description: self.description,
            components: self.components,
            deprecated: self.deprecated,
            events: self.events,
            legacy: self.legacy,
        }
    }
}

/// An item definition (`minecraft:item`), in the newest format version (1.26.50); an item
/// written in an older format is upgraded to it on load ([`history`](crate::history) has the older structs).
///
/// Two item formats exist. A document is component based only when its `format_version` is
/// exactly 1.16.100 or 1.17.0 and later, **and** its `components` name at least one known
/// component (or a `tag:` key); everything else is the legacy item format of 1.10, read into
/// [`LegacyItemData::format_item`] with an empty component map ([`ItemFormat::Legacy`](crate::ItemFormat::Legacy)).
///
/// JSON: `{"format_version": "1.26.50", "minecraft:item": {"description": …, "components":
/// {…}, "__deprecated_components"?: …, "events"?: …}}`.
#[doc(alias = "item")]
pub type Item = ModernItem<{ crate::history::pack(1, 26, 50) }>;

/// An item's `description`.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemDescription {
    /// The identifier, `ns:name` (JSON `identifier`, required).
    pub identifier: ItemId,
    /// The creative-menu placement (JSON `menu_category`); its `category` always holds the
    /// value vanilla uses: the written one, else `none` (command only) for a document of
    /// 1.21.110 and later, `Undefined` for 1.21.60–1.21.109, `items` below.
    pub menu_category: MenuCategory,
    /// How much of `menu_category` the document wrote itself: what tells a `none` the author
    /// wrote from the `none` an absent member means, and what the writer leaves out again.
    pub menu_category_presence: MenuCategoryPresence,
}

/// How much of `description.menu_category` an item document wrote.
///
/// From format 1.21.110 an absent `category` means `none`, so
/// [`ItemDescription::menu_category`] holds `none` whether the author wrote it or not; this
/// records which. The writer never writes a `none` or a `menu_category` the document did not
/// have; any other category, a group and `is_hidden_in_commands` are always written. A
/// document of an older format is upgraded with the category its own format gives it, which
/// counts as written (the newest format would read its absence as `none`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MenuCategoryPresence {
    /// The document has no `menu_category` (also the state of a new item).
    #[default]
    Absent,
    /// The document has a `menu_category` without `category`.
    WithoutCategory,
    /// The document has `menu_category.category`.
    Category,
}

impl MenuCategoryPresence {
    /// The presence of a `menu_category` as it was read, before a default was filled in.
    pub(crate) fn of(menu: Option<&MenuCategory>) -> MenuCategoryPresence {
        match menu {
            None => MenuCategoryPresence::Absent,
            Some(menu) if menu.category.is_none() => MenuCategoryPresence::WithoutCategory,
            Some(_) => MenuCategoryPresence::Category,
        }
    }
}

impl ItemDescription {
    /// Whether the creative category is one the document states: it wrote `category`, or the
    /// category is not the `none` an absent member means. `false` is an item whose author
    /// left the category out and so never asked for a place in the creative menu.
    pub fn category_is_written(&self) -> bool {
        self.menu_category_presence == MenuCategoryPresence::Category
            || self.menu_category.category
                != Some(bedrock_addon_core::shared::CreativeCategory::ItemCommandOnly)
    }
}

/// A beta item definition (`"format_version": "beta"`): [`Item`]'s members. The beta struct
/// re-binds only two components of the 1.26.50 format:
/// `allow_off_hand`, whose `value` defaults to true (false in a stable document), and
/// `record`, whose `sound_event` takes any sound name (a level sound event's name in a stable
/// document); the other sound members take any sound name from 1.26.50 already. Reached
/// through `Registry::beta()`.
pub type Beta = ModernItem<BETA>;

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
            },
            components: ComponentMap::new(),
            deprecated: ItemDeprecatedData::default(),
            events: None,
            legacy: LegacyItemData::default(),
        }
    }

    /// Runs the family's rules as if the item were loaded at the newest format version from a
    /// pack with `cx`: the component gates, the identifier rule and the validators.
    pub fn validate(&self, cx: &LoadContext) -> Diagnostics {
        let version = known::V1_26_50;
        let at = JsonPath::new().child("minecraft:item");
        let mut d = Diagnostics::default();
        let mut components = IndexMap::new();
        for (name, written) in super::history::written_components(&self.components) {
            if let Ok(json) = written {
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
