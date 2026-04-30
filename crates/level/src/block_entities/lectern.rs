use std::collections::HashMap;

use crate::{deserialize_bool, types::ItemStack};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
// #[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct BookPage {
    #[serde(rename = "photoname")]
    pub photo_name: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
// #[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct BookMetadata {
    pub author: String,
    pub xuid: String,
    pub title: String,
    pub pages: Vec<BookPage>,
    pub generation: i32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
// #[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Book {
    pub count: i8,
    pub name: String,
    pub damage: i16,
    #[serde(rename = "tag")]
    pub meta: BookMetadata,
    #[serde(deserialize_with = "deserialize_bool")]
    pub was_picked_up: bool,
}

impl From<Book> for ItemStack {
    fn from(value: Book) -> Self {
        ItemStack {
            was_picked_up: value.was_picked_up,
            block: None,
            can_destroy: None,
            can_place_on: None,
            count: value.count,
            damage: value.damage,
            name: value.name,
            tag: Some(todo!("implement nbtx::to_value")),
        }
    }
}

impl TryFrom<ItemStack> for Book {
    type Error = ();

    fn try_from(value: ItemStack) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
// #[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct LecternBook {
    pub total_pages: i32,
    pub page: i32,
    #[serde(deserialize_with = "deserialize_bool")]
    pub has_book: bool,
    // pub book: nbtx::Value,
    pub book: Book,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "deny-unknown-fields", serde(deny_unknown_fields))]
pub struct Lectern {
    #[serde(flatten)]
    pub book: Option<LecternBook>,
}
