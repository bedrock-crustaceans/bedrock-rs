//! The crafting item catalogue (`CraftingCatalog`): `item_catalog/crafting_item_catalog.json` of a
//! behaviour pack (`minecraft:crafting_items_catalog`), which places items into the groups of the
//! four creative categories. Two structs of the same members: `v1_21_60` `[1.21.60, 1.26.50)`,
//! read under the unstrict mask, and `v1_26_50` from 1.26.50 to the engine, under the strict
//! one, joined by a plain copy; the files of a stack merge bottom-up from the vanilla
//! catalogue.

#![cfg_attr(docsrs, feature(doc_cfg))]

/// The catalogue as written at each format: the two structs of the chain and the document a
/// file parses to before its step ([`CatalogAny`](history::CatalogAny)).
pub mod history;
mod link;
mod load;
mod merge;
mod new;
mod parse;
mod stack;
mod write;

#[cfg(test)]
mod tests;

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::ItemId;
use bedrock_addon_core::shared::CreativeCategory;
use bedrock_addon_core::version::FormatVersion;

// The checkers the link pass calls (`pack::link::content`).
#[allow(unused_imports, reason = "the link pass reads them")]
#[doc(hidden)]
pub use link::{Definitions, link_group_names, link_placements};
#[doc(hidden)]
pub use stack::load;

/// The first struct version (`v1_21_60::CraftingCatalogDocument::VERSION`), which is also the
/// oldest `format_version` vanilla reads.
pub(crate) const V1_21_60: FormatVersion = FormatVersion::new(1, 21, 60);

/// The newest struct version (`v1_26_50::CraftingCatalogDocument::VERSION`), the
/// `format_version` of vanilla's own 1.26.50 catalogue.
pub(crate) const V1_26_50: FormatVersion = bedrock_addon_core::version::known::V1_26_50;

/// The path of the one file a behaviour pack's catalogue is read from.
pub(crate) const PATH: &str = "item_catalog/crafting_item_catalog.json";

/// A crafting item catalogue (`minecraft:crafting_items_catalog`): the groups a pack adds to
/// the four creative categories, and the items in each group.
///
/// JSON: `{"format_version": "1.21.60", "minecraft:crafting_items_catalog": {"categories":
/// [{"category_name": "construction", "groups": [{"group_identifier": {"name": "ns:group",
/// "icon": "ns:item"}, "items": ["ns:item", {"name": "ns:other:1"}]}]}]}}`. Every
/// `format_version` from 1.21.60 to the engine version reads the same members.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "crafting_item_catalog")]
#[doc(alias = "minecraft:crafting_items_catalog")]
pub struct CraftingCatalog {
    /// The categories groups are added to (JSON `categories`, required). A file may list a
    /// category several times; every entry adds to the same category.
    pub categories: Vec<Category>,
    /// Unknown members of the payload.
    pub extra: Extra,
}

/// The groups one entry of `categories` adds to a creative category.
#[derive(Clone, Debug, PartialEq)]
pub struct Category {
    /// The category the groups go to (JSON `category_name`, required).
    pub category_name: CategoryName,
    /// The groups added to it, in order (JSON `groups`, required).
    pub groups: Vec<Group>,
    /// Unknown members.
    pub extra: Extra,
}

/// One of the four creative categories a catalogue can add to; a pack cannot add, rename or
/// reorder them. The JSON names are lower-case and compared with case.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum CategoryName {
    /// `"construction"`.
    Construction = 0,
    /// `"nature"`.
    Nature = 1,
    /// `"equipment"`.
    Equipment = 2,
    /// `"items"`.
    Items = 3,
}

impl CategoryName {
    /// The four categories, in vanilla's order.
    pub const ALL: [CategoryName; 4] = [
        Self::Construction,
        Self::Nature,
        Self::Equipment,
        Self::Items,
    ];

    /// The JSON name (`"construction"`).
    pub fn name(self) -> &'static str {
        match self {
            Self::Construction => "construction",
            Self::Nature => "nature",
            Self::Equipment => "equipment",
            Self::Items => "items",
        }
    }

    /// The category a JSON name selects; `None` for any other string (`"Construction"`
    /// included).
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|category| category.name() == name)
    }

    /// The creative-menu category of an item definition's `menu_category` this category is.
    ///
    /// The game numbers the categories from one (Construction 1 up to Items 4), so its value is
    /// the discriminant here plus one.
    pub fn creative_category(self) -> CreativeCategory {
        match self {
            Self::Construction => CreativeCategory::Construction,
            Self::Nature => CreativeCategory::Nature,
            Self::Equipment => CreativeCategory::Equipment,
            Self::Items => CreativeCategory::Items,
        }
    }
}

/// A group of items in a category: collapsible under a name and an icon when it has a
/// `group_identifier`, else a run of loose items.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Group {
    /// The group's name and icon (JSON `group_identifier`); without it the items are added
    /// as loose items.
    pub group_identifier: Option<GroupIdentifier>,
    /// The items of the group, in order (JSON `items`, required).
    pub items: Vec<CatalogItem>,
    /// Unknown members.
    pub extra: Extra,
}

/// The name and icon of a collapsible group.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupIdentifier {
    /// The group's name (JSON `name`, required): `namespace:name`, which is also the
    /// localisation key the creative menu shows it by. A group of the same name in the same
    /// category, in this or a lower pack, is the same group; names compare byte for byte.
    pub name: String,
    /// The item or block that represents the group (JSON `icon`). A group keeps the icon of
    /// its first definition.
    pub icon: Option<CatalogItem>,
    /// Unknown members.
    pub extra: Extra,
}

/// One item or block of the catalogue.
///
/// JSON: `"ns:item"`, `"ns:item:1"` (with an aux value) or `{"name": "ns:item"}`. Both forms
/// read to the same value; it is written as the string unless it holds unknown members.
/// Two items are equal when their names are written the same, the aux value included.
#[derive(Clone, Debug)]
pub struct CatalogItem {
    /// The item (JSON `name`, or the string itself): `namespace:name`, optionally followed
    /// by `:aux`. The namespace is required.
    pub name: ItemId,
    /// Unknown members of the object form.
    pub extra: Extra,
}

/// By the name as written: `ItemId` alone compares without the aux value.
impl PartialEq for CatalogItem {
    fn eq(&self, other: &Self) -> bool {
        self.name.as_str() == other.name.as_str() && self.extra == other.extra
    }
}

/// What an entry of the merged catalogue is keyed by, for
/// [`Merged::origin`](bedrock_addon_core::document::Merged::origin): the pack that first defined a named
/// group, or the pack whose file placed an item.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CatalogKey {
    /// A named group of a category.
    Group {
        /// The category.
        category: CategoryName,
        /// The group's name.
        name: String,
    },
    /// A placed item: its identifier and aux value, which is what makes a second placement
    /// a duplicate.
    Item {
        /// The identifier; an aux value in its text does not count.
        id: ItemId,
        /// The aux value (`0` when the item names none).
        aux: i16,
    },
}
