//! Items: `Item`, `component::*`, `history::*`, `Beta`, and the armour trims (`trim`).
//!
//! `minecraft:item` of a behaviour pack, in the newest format version; older documents are
//! upgraded when they load.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![recursion_limit = "512"]

/// Armour trims (`TrimData`, `TrimPattern`, `TrimMaterial`): vanilla data compiled into the game.
pub mod trim;

/// The components of an item (`minecraft:item` → `components`).
pub mod component;
/// The older item formats, one struct per format version, and the chain between them.
pub mod history;

mod deprecated;
mod expressions;
mod legacy;
mod load;
#[cfg(feature = "nbt")]
mod network;
mod new;
mod sounds;
mod texts;
mod validate;
pub(crate) mod write;

#[cfg(test)]
mod tests;

pub use deprecated::ItemDeprecatedData;
pub use legacy::{
    ItemFormat, LegacyCamera, LegacyEffect, LegacyFood, LegacyFormatItem, LegacyItemData,
    LegacySeed,
};

#[doc(hidden)]
pub use load::LOADER;
#[allow(unused_imports, reason = "the link pass reads it")]
#[doc(hidden)]
pub use load::loaded_identifier;

use bedrock_addon_core::document::{ComponentMap, Extra};
use bedrock_addon_core::id::ItemId;
use bedrock_addon_core::shared::MenuCategory;

use bedrock_addon_core::json::Value;
use component::ItemComponent;

/// An item definition (`minecraft:item`), in the newest format version (1.26.50); an item
/// written in an older format is upgraded to it on load ([`history`] has the older structs).
///
/// Two item formats exist. A document is component based only when its `format_version` is
/// exactly 1.16.100 or 1.17.0 and later, **and** its `components` name at least one known
/// component (or a `tag:` key); everything else is the legacy item format of 1.10, read into
/// [`LegacyItemData::format_item`] with an empty component map ([`ItemFormat::Legacy`]).
///
/// JSON: `{"format_version": "1.26.50", "minecraft:item": {"description": …, "components":
/// {…}, "__deprecated_components"?: …, "events"?: …}}`.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "item")]
pub struct Item {
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
    /// Unknown members of the item object.
    pub extra: Extra,
}

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
    /// Unknown members.
    pub extra: Extra,
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
#[derive(Clone, Debug, PartialEq)]
pub struct Beta {
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
    /// Unknown members of the item object.
    pub extra: Extra,
}
