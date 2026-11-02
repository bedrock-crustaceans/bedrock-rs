//! Writing a catalogue: every level writes its known members in vanilla's bind order, then
//! the unknown members it kept. Nothing is written that the value does not hold: no
//! `group_identifier` for loose items, no `icon` a group lacks.

use facet_value::{VArray, VObject};

use super::load::CatalogFamily;
use super::{CatalogItem, Category, CraftingCatalog, Group, GroupIdentifier, V1_26_50};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::Family;

/// Appends the unknown members of a level after its known ones.
fn with_extra(mut object: VObject, extra: &Extra) -> Value {
    for (key, value) in extra.iter() {
        object.insert(key, value.clone());
    }
    object.into()
}

/// `values` as a JSON array.
fn array<T>(values: &[T], write: impl Fn(&T) -> Value) -> Value {
    let mut array = VArray::new();
    for value in values {
        array.push(write(value));
    }
    array.into()
}

impl CatalogItem {
    /// The bare string; the `{ "name": … }` form only when it has unknown members to keep.
    fn write(&self) -> Value {
        if self.extra.is_empty() {
            return Value::from(self.name.as_str());
        }
        let mut object = VObject::new();
        object.insert("name", self.name.as_str());
        with_extra(object, &self.extra)
    }
}

impl GroupIdentifier {
    fn write(&self) -> Value {
        let mut object = VObject::new();
        object.insert("name", self.name.as_str());
        if let Some(icon) = &self.icon {
            object.insert("icon", icon.write());
        }
        with_extra(object, &self.extra)
    }
}

impl Group {
    fn write(&self) -> Value {
        let mut object = VObject::new();
        if let Some(identifier) = &self.group_identifier {
            object.insert("group_identifier", identifier.write());
        }
        object.insert("items", array(&self.items, CatalogItem::write));
        with_extra(object, &self.extra)
    }
}

impl Category {
    fn write(&self) -> Value {
        let mut object = VObject::new();
        object.insert("category_name", self.category_name.name());
        object.insert("groups", array(&self.groups, Group::write));
        with_extra(object, &self.extra)
    }
}

impl CraftingCatalog {
    /// The payload: `{ "categories": [...] }` and the unknown members.
    pub(super) fn payload(&self) -> Value {
        let mut object = VObject::new();
        object.insert("categories", array(&self.categories, Category::write));
        with_extra(object, &self.extra)
    }

    /// The whole document: `format_version` 1.26.50, the newest struct's version and vanilla's
    /// own stamp, then the payload. A read never keeps an unknown member; an `extra` a caller
    /// filled by hand is written after the known members of its level, and the strict 1.26.50
    /// slice refuses it on the next read.
    pub(super) fn document(&self) -> Value {
        let mut root = VObject::new();
        root.insert("format_version", V1_26_50.to_string());
        root.insert(CatalogFamily::PAYLOAD_KEY, self.payload());
        root.into()
    }
}
