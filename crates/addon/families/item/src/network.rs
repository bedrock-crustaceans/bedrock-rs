//! The client network NBT of an item.

use bedrock_addon_core::document::ComponentRef;

use super::Item;
use super::component::ItemComponent;
use super::legacy::ItemFormat;
use bedrock_addon_core::json::Value;

impl Item {
    /// The item's client properties as network NBT: `{components: {<name>: …,
    /// item_properties: {menu_category, icon, …}}}`. Each typed component is sent with its
    /// numbers as `Int` or `Float` as written; custom and unknown components are not sent. An
    /// item in the legacy item format is sent as a non-component-based entry: an empty
    /// compound.
    pub fn network_properties(&self) -> nbtx::Value {
        let mut root = nbtx::Compound::new();
        if self.legacy.format == ItemFormat::Legacy {
            return nbtx::Value::Compound(root);
        }
        let mut components = nbtx::Compound::new();
        let mut properties = nbtx::Compound::new();
        if let Ok(menu) =
            bedrock_addon_core::versioning::write_payload(&self.description.menu_category)
        {
            properties.insert("menu_category".into(), to_nbt(&menu));
        }
        for entry in self.components.iter() {
            let ComponentRef::Typed { name, value, .. } = entry else {
                continue;
            };
            let Ok(json) = value.write_component() else {
                continue;
            };
            if name == "minecraft:icon" {
                // the icon is an item property, not a component
                properties.insert("minecraft:icon".into(), to_nbt(&json));
                continue;
            }
            components.insert(name.into(), to_nbt(&json));
        }
        if let Some(ItemComponent::MaxStackSize(size)) =
            self.components.iter().find_map(|e| match e {
                ComponentRef::Typed { value, .. }
                    if matches!(value, ItemComponent::MaxStackSize(_)) =>
                {
                    Some(value)
                }
                _ => None,
            })
        {
            properties.insert("max_stack_size".into(), nbtx::Value::Int(size.value));
        }
        components.insert("item_properties".into(), nbtx::Value::Compound(properties));
        root.insert("components".into(), nbtx::Value::Compound(components));
        nbtx::Value::Compound(root)
    }
}

/// JSON as NBT: integers `Int` (or `Long` when out of range), reals `Float`, booleans `Byte`,
/// strings, arrays as lists (a list of mixed types as compounds of its items is not built: such
/// an array is dropped), objects as compounds; `null` is dropped.
fn to_nbt(value: &Value) -> nbtx::Value {
    if let Some(flag) = value.as_bool() {
        return nbtx::Value::Byte(i8::from(flag));
    }
    if let Some(number) = value.as_number() {
        if number.is_integer() {
            return match number.to_i32() {
                Some(n) => nbtx::Value::Int(n),
                None => nbtx::Value::Long(number.to_i64().unwrap_or_default()),
            };
        }
        return nbtx::Value::Float(number.to_f32().unwrap_or_default());
    }
    if let Some(text) = value.as_string() {
        return nbtx::Value::String(text.as_str().into());
    }
    if let Some(array) = value.as_array() {
        let items: Vec<nbtx::Value> = array.iter().map(to_nbt).collect();
        return nbtx::Value::List(nbtx::ValueList::try_from(items).unwrap_or_default());
    }
    let mut compound = nbtx::Compound::new();
    if let Some(object) = value.as_object() {
        for (key, member) in object {
            if !member.is_null() {
                compound.insert(key.as_str().into(), to_nbt(member));
            }
        }
    }
    nbtx::Value::Compound(compound)
}
