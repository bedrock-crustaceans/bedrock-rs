//! Writing a catalogue: every level writes its known members in vanilla's bind order (the
//! document's unknown members are merged after them by path; the reader keeps none). Nothing is
//! written that the value does not hold: no `group_identifier` for loose items, no `icon` a
//! group lacks.

use facet_value::{VArray, VObject};

use super::{CatalogItem, Category, CraftingCatalog, Group, GroupIdentifier};
use bedrock_addon_core::json::Value;

/// `values` as a JSON array.
fn array<T>(values: &[T], write: impl Fn(&T) -> Value) -> Value {
    let mut array = VArray::new();
    for value in values {
        array.push(write(value));
    }
    array.into()
}

impl CatalogItem {
    /// The bare string.
    fn write(&self) -> Value {
        Value::from(self.name.as_str())
    }
}

impl GroupIdentifier {
    fn write(&self) -> Value {
        let mut object = VObject::new();
        object.insert("name", self.name.as_str());
        if let Some(icon) = &self.icon {
            object.insert("icon", icon.write());
        }
        object.into()
    }
}

impl Group {
    fn write(&self) -> Value {
        let mut object = VObject::new();
        if let Some(identifier) = &self.group_identifier {
            object.insert("group_identifier", identifier.write());
        }
        object.insert("items", array(&self.items, CatalogItem::write));
        object.into()
    }
}

impl Category {
    fn write(&self) -> Value {
        let mut object = VObject::new();
        object.insert("category_name", self.category_name.name());
        object.insert("groups", array(&self.groups, Group::write));
        object.into()
    }
}

impl CraftingCatalog {
    /// The payload: `{ "categories": [...] }`.
    pub(super) fn payload(&self) -> Value {
        let mut object = VObject::new();
        object.insert("categories", array(&self.categories, Category::write));
        object.into()
    }
}
