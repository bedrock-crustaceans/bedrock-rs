//! Constructing catalogues
//! and the inherent helpers that read them.

use std::fmt;

use super::{CatalogItem, Category, CategoryName, CraftingCatalog, Group, GroupIdentifier};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::ItemId;

impl CraftingCatalog {
    /// A catalogue without categories.
    pub fn new() -> Self {
        Self::default()
    }

    /// Every group this catalogue adds to `category`, in order, over all of its entries for
    /// that category.
    pub fn groups(&self, category: CategoryName) -> impl Iterator<Item = &Group> + '_ {
        self.categories
            .iter()
            .filter(move |entry| entry.category_name == category)
            .flat_map(|entry| &entry.groups)
    }

    /// The first group named `name` this catalogue adds to `category`; names compare byte for
    /// byte, as the merge compares them.
    pub fn group(&self, category: CategoryName, name: &str) -> Option<&Group> {
        self.groups(category)
            .find(|group| group.name() == Some(name))
    }
}

impl Category {
    /// An entry adding no group yet to `category_name`.
    pub fn new(category_name: CategoryName) -> Self {
        Self {
            category_name,
            groups: Vec::new(),
            extra: Extra::default(),
        }
    }
}

impl Group {
    /// A collapsible group named `name` (`namespace:name`), without an icon or items.
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            group_identifier: Some(GroupIdentifier::new(name)),
            ..Self::default()
        }
    }

    /// A group of loose items, without items yet (also [`Group::default`]).
    pub fn loose() -> Self {
        Self::default()
    }

    /// The group's name; `None` for loose items.
    pub fn name(&self) -> Option<&str> {
        self.group_identifier
            .as_ref()
            .map(|identifier| identifier.name.as_str())
    }
}

impl GroupIdentifier {
    /// The identifier of a group named `name` (`namespace:name`), without an icon.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            icon: None,
            extra: Extra::default(),
        }
    }
}

impl CatalogItem {
    /// The item `name` (`namespace:name`, or `namespace:name:aux`).
    pub fn new(name: ItemId) -> Self {
        Self {
            name,
            extra: Extra::default(),
        }
    }
}

impl From<ItemId> for CatalogItem {
    fn from(name: ItemId) -> Self {
        Self::new(name)
    }
}

/// The JSON name.
impl fmt::Display for CategoryName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}
