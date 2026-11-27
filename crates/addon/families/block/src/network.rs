//! The client network form of a block (11): what the server sends in the start-game block
//! properties.

use nbtx::{Compound, Value as Nbt, ValueList};

use super::Block;
use super::component::BlockComponent;
use bedrock_addon_core::document::{ComponentMap, ComponentRef};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::{Molang, MolangSource, MolangVersion, RawVersion};
use bedrock_addon_shared_shapes::StateValue;

impl Block {
    /// The block as the client network NBT (11): `{components, menu_category {category, group,
    /// is_hidden_in_commands}, permutations [{condition, components}], properties [{name,
    /// enum}], molangVersion, vanillaBlockData {block_id}}`.
    ///
    /// Components are sent by name with their JSON as NBT (objects as compounds, integers as
    /// `Int`, reals as `Float`, booleans as `Byte`; a component that is not an object is sent as
    /// `{"value": …}`); custom components are sent with their parameters; unknown components are
    /// not sent. A state's values are a `String`, `Int` or `Byte` list. `molangVersion` is the
    /// version of the first permutation condition that has one, else 13.
    pub fn to_network_nbt(&self) -> Nbt {
        let mut root = Compound::default();
        root.insert("components".into(), components(&self.components));
        let mut menu = Compound::default();
        let category = self
            .description
            .menu_category
            .as_ref()
            .and_then(|m| m.category)
            .and_then(bedrock_addon_shared_shapes::CreativeCategory::name)
            .unwrap_or("none");
        let group = self
            .description
            .menu_category
            .as_ref()
            .and_then(|m| m.group.clone())
            .unwrap_or_default();
        let hidden = self
            .description
            .menu_category
            .as_ref()
            .and_then(|m| m.is_hidden_in_commands.as_ref().map(|b| b.0))
            .unwrap_or(false);
        menu.insert("category".into(), Nbt::String(category.into()));
        menu.insert("group".into(), Nbt::String(group.into()));
        menu.insert("is_hidden_in_commands".into(), Nbt::Byte(i8::from(hidden)));
        root.insert("menu_category".into(), Nbt::Compound(menu));
        let permutations: Vec<Compound> = self
            .permutations
            .iter()
            .map(|permutation| {
                let mut entry = Compound::default();
                entry.insert(
                    "condition".into(),
                    Nbt::String(condition_text(&permutation.condition).into()),
                );
                entry.insert("components".into(), components(&permutation.components));
                entry
            })
            .collect();
        root.insert(
            "permutations".into(),
            Nbt::List(if permutations.is_empty() {
                ValueList::Compound(Vec::new())
            } else {
                ValueList::Compound(permutations)
            }),
        );
        let properties: Vec<Compound> = self
            .states_with_traits()
            .iter()
            .map(|(name, values)| {
                let mut property = Compound::default();
                property.insert("name".into(), Nbt::String(name.as_str().into()));
                property.insert("enum".into(), Nbt::List(state_list(&values.values())));
                property
            })
            .collect();
        root.insert(
            "properties".into(),
            Nbt::List(ValueList::Compound(properties)),
        );
        root.insert(
            "molangVersion".into(),
            Nbt::Int(i32::from(self.molang_version().0)),
        );
        let mut vanilla = Compound::default();
        let block_id = self
            .description
            .internal_vanilla_data
            .as_ref()
            .map_or(0, super::VanillaBlockData::block_id);
        vanilla.insert("block_id".into(), Nbt::Int(block_id));
        root.insert("vanillaBlockData".into(), Nbt::Compound(vanilla));
        Nbt::Compound(root)
    }

    /// The Molang version the client compiles the conditions at.
    fn molang_version(&self) -> RawVersion {
        self.permutations
            .iter()
            .find_map(|p| p.condition.source().and_then(MolangSource::raw_version))
            .unwrap_or(RawVersion::from(MolangVersion::V13))
    }
}

/// A condition's text: the expression, or a constant written as Molang.
fn condition_text(condition: &Molang) -> String {
    match condition {
        Molang::Expr(source) => source.as_str().to_owned(),
        Molang::Const(value) => value.to_string(),
        Molang::Bool(value) => if *value { "1" } else { "0" }.to_owned(),
        _ => String::new(),
    }
}

/// A component map as a compound; unknown entries are left out.
fn components(map: &ComponentMap<BlockComponent>) -> Nbt {
    let mut compound = Compound::default();
    for entry in map.iter() {
        let (name, json) = match entry {
            ComponentRef::Typed { name, value, .. } => match value.write_component() {
                Ok(json) => (name.to_owned(), json),
                Err(_) => continue,
            },
            ComponentRef::Custom(custom) => (custom.name.clone(), custom.parameters.clone()),
            _ => continue,
        };
        let value = if json.is_object() {
            to_nbt(&json)
        } else {
            let mut wrapped = Compound::default();
            wrapped.insert("value".into(), to_nbt(&json));
            Nbt::Compound(wrapped)
        };
        compound.insert(name.into(), value);
    }
    Nbt::Compound(compound)
}

/// JSON as NBT (a real number narrowed to `Float`, as the client reads it): objects as compounds, arrays as lists (of one tag), integers as `Int` (`Long`
/// beyond 32 bits), reals as `Float`, booleans as `Byte`, `null` as an empty compound.
#[allow(
    clippy::cast_possible_truncation,
    reason = "component reals are `f32` in vanilla"
)]
fn to_nbt(json: &Value) -> Nbt {
    if let Some(b) = json.as_bool() {
        return Nbt::Byte(i8::from(b));
    }
    if let Some(n) = json.as_number() {
        if n.is_integer() {
            return n
                .to_i32()
                .map(Nbt::Int)
                .or_else(|| n.to_i64().map(Nbt::Long))
                .unwrap_or(Nbt::Float(n.to_f64_lossy() as f32));
        }
        return Nbt::Float(n.to_f64_lossy() as f32);
    }
    if let Some(s) = json.as_string() {
        return Nbt::String(s.as_str().into());
    }
    if let Some(array) = json.as_array() {
        let values: Vec<Nbt> = array.iter().map(to_nbt).collect();
        return Nbt::List(ValueList::try_from(values).unwrap_or_default());
    }
    let mut compound = Compound::default();
    if let Some(object) = json.as_object() {
        for (key, value) in object {
            compound.insert(key.as_str().into(), to_nbt(value));
        }
    }
    Nbt::Compound(compound)
}

/// A state's values as a list: strings as `String`, integers as `Int`, booleans as `Byte`.
fn state_list(values: &[StateValue]) -> ValueList {
    let nbt: Vec<Nbt> = values
        .iter()
        .map(|value| match value {
            StateValue::Bool(b) => Nbt::Byte(i8::from(*b)),
            StateValue::Int(i) => Nbt::Int(*i),
            StateValue::String(s) => Nbt::String(s.as_str().into()),
            _ => Nbt::Byte(0),
        })
        .collect();
    ValueList::try_from(nbt).unwrap_or_default()
}
