//! `MenuCategory`, `CreativeCategory`: where an item or block appears in the creative menu.

use facet::Facet;

use super::read::{Failure, ROOT, ShapeError, ShapeErrorKind, wrong_type};
use crate::document::Extra;
use crate::json::{StrictBool, Value};

/// An item's or block's creative-menu placement: as the game's item and block
/// loaders read it.
///
/// JSON: `{"category"?: category, "group"?: string, "is_hidden_in_commands"?: bool}`. The
/// constraints differ per family and stay there: an item's `category` is optional and its
/// `group` at most 256 characters and namespaced from 1.21.60; a block's `category` is
/// required and its `group` a namespaced name without spaces.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[doc(alias = "menu_category")]
pub struct MenuCategory {
    /// The creative tab (JSON `category`).
    #[facet(default)]
    pub category: Option<CreativeCategory>,
    /// The expandable group within the tab (JSON `group`): a vanilla `itemGroup.name.*` or a
    /// pack-defined group.
    #[facet(default)]
    pub group: Option<String>,
    /// Hidden from commands such as `/give` (JSON `is_hidden_in_commands`; a boolean only).
    #[facet(default)]
    pub is_hidden_in_commands: Option<StrictBool>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl MenuCategory {
    /// The `construction` category with no group.
    #[must_use]
    pub fn construction() -> MenuCategory {
        MenuCategory {
            category: Some(CreativeCategory::Construction),
            ..MenuCategory::default()
        }
    }
}

/// A creative tab, with the game's 1.21.110 numbering.
///
/// JSON: `construction`, `nature`, `equipment`, `items` and `none`. `none` is read as
/// [`CreativeCategory::ItemCommandOnly`], its meaning from 1.21.110; an item document of
/// 1.21.60–1.21.109 maps it to [`CreativeCategory::Undefined`], which is written as `none`
/// again. [`CreativeCategory::All`] has no JSON spelling and cannot be written.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
#[non_exhaustive]
pub enum CreativeCategory {
    /// Every tab (value 0); no JSON spelling.
    All,
    /// The construction tab (JSON `construction`).
    Construction,
    /// The nature tab (JSON `nature`).
    Nature,
    /// The equipment tab (JSON `equipment`).
    Equipment,
    /// The items tab (JSON `items`).
    Items,
    /// Only reachable by commands (JSON `none` from 1.21.110).
    ItemCommandOnly,
    /// No category (JSON `none` in 1.21.60–1.21.109).
    Undefined,
}

/// The JSON spellings vanilla reads, with the category each reads as.
const NAMES: [(&str, CreativeCategory); 5] = [
    ("construction", CreativeCategory::Construction),
    ("nature", CreativeCategory::Nature),
    ("equipment", CreativeCategory::Equipment),
    ("items", CreativeCategory::Items),
    ("none", CreativeCategory::ItemCommandOnly),
];

impl CreativeCategory {
    /// The JSON spelling; `None` for [`CreativeCategory::All`].
    #[must_use]
    pub fn name(self) -> Option<&'static str> {
        match self {
            Self::All => None,
            Self::Undefined => Some("none"),
            other => NAMES
                .iter()
                .find(|(_, category)| *category == other)
                .map(|(name, _)| *name),
        }
    }
}

impl TryFrom<Value> for CreativeCategory {
    type Error = ShapeError;

    fn try_from(value: Value) -> Result<Self, ShapeError> {
        // The game reads the category as an enum constraint over the category names.
        let name = value
            .as_string()
            .ok_or_else(|| wrong_type(ROOT, "a category name", &value))?;
        NAMES
            .iter()
            .find(|(n, _)| *n == name.as_str())
            .map(|(_, category)| *category)
            .ok_or_else(|| Failure::new(ShapeErrorKind::UnknownName).into())
    }
}

impl TryFrom<&CreativeCategory> for Value {
    type Error = ShapeError;

    fn try_from(category: &CreativeCategory) -> Result<Self, ShapeError> {
        category
            .name()
            .map(Value::from)
            .ok_or_else(|| Failure::new(ShapeErrorKind::NotRepresentable).into())
    }
}
