//! Ingredients: vanilla's item-name grammar, potion names and the derived views of an
//! ingredient (item, aux, count).

use super::{Ingredient, IngredientObject, ItemName};
use bedrock_addon_core::molang::{Molang, MolangList};

/// The aux value that stands for every aux value of an item (vanilla's wildcard 32767).
pub(super) const WILDCARD_AUX: i32 = 32767;

/// The prefix of a potion name (`minecraft:potion_type:awkward`).
const POTION_PREFIX: &str = "minecraft:potion_type:";

/// What vanilla logs about an item text. Each is a log line only: parsing goes on with what
/// the text yields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ItemIdProblem {
    /// `Invalid aux value in item ID "%s"`: the aux token is not an integer in 0..=255.
    InvalidAux,
    /// `Empty namespace in item ID "%s"`.
    EmptyNamespace,
    /// `Excessive number of tokens in item ID "%s"`: four or more `:`-separated tokens.
    ExcessiveTokens,
}

impl ItemIdProblem {
    /// Vanilla's text, with `{}` where the item text goes.
    pub(super) fn template(self) -> &'static str {
        match self {
            Self::InvalidAux => "Invalid aux value in item ID \"{}\"",
            Self::EmptyNamespace => "Empty namespace in item ID \"{}\"",
            Self::ExcessiveTokens => "Excessive number of tokens in item ID \"{}\"",
        }
    }
}

/// The potion `text` names (`minecraft:potion_type:awkward` → `minecraft:awkward`), if it is one
/// of vanilla's potions.
///
/// Vanilla tries the whole string as a potion name before it splits it as an item. The form
/// accepted here is the `minecraft:potion_type:<name>` one that vanilla's own recipe files use,
/// with the name compared without case (vanilla ships `Thick` and `Awkward`) against the list
/// of potions the game knows.
pub(super) fn potion(text: &str) -> Option<String> {
    let name = text.strip_prefix(POTION_PREFIX)?;
    let id = format!("minecraft:{}", name.to_lowercase());
    bedrock_addon_core::vanilla::ids()
        .potion_effects
        .contains(&id.as_str())
        .then_some(id)
}

impl ItemName {
    /// Splits `text` as vanilla's item-name parser does, or names the potion it is.
    ///
    /// One token is `minecraft:<token>`. With two or more tokens whose second is a number, the
    /// second is the aux value of `minecraft:<first>` and the rest is ignored; otherwise two
    /// tokens are `<namespace>:<name>`, three add the aux value, and of four or more the first
    /// three count. Namespace and name are lower-cased, an `item.` prefix is dropped, and the
    /// aux value must be an integer from 0 to 255.
    pub fn parse(text: &str) -> ItemName {
        parse_item(text).0
    }

    /// `namespace:name`.
    pub fn id(&self) -> String {
        format!("{}:{}", self.namespace, self.name)
    }
}

/// [`ItemName::parse`] with what vanilla logs about the text.
///
/// The text is split on `:`; a missing namespace is `minecraft`, a numeric token is tested as
/// an optionally signed run of digits and then range-checked to 0..=255, and an `item.` prefix
/// of the name is dropped. Empty tokens are kept as empty names.
pub(super) fn parse_item(text: &str) -> (ItemName, Vec<ItemIdProblem>) {
    if let Some(potion) = potion(text) {
        let name = ItemName {
            namespace: "minecraft".to_owned(),
            name: "potion".to_owned(),
            aux: None,
            potion: Some(potion),
        };
        return (name, Vec::new());
    }
    let tokens: Vec<&str> = text.split(':').collect();
    let mut problems = Vec::new();
    let (namespace, name, aux) = match tokens.as_slice() {
        [name] => ("minecraft", *name, None),
        [name, aux, ..] if is_number(aux) => ("minecraft", *name, Some(*aux)),
        [namespace, name] => (*namespace, *name, None),
        [namespace, name, aux] => (*namespace, *name, Some(*aux)),
        [namespace, name, aux, ..] => {
            problems.push(ItemIdProblem::ExcessiveTokens);
            (*namespace, *name, Some(*aux))
        }
        [] => ("minecraft", "", None),
    };
    let aux = aux.and_then(|aux| {
        let value = aux.parse::<i32>().ok().filter(|v| (0..=255).contains(v));
        if value.is_none() {
            problems.push(ItemIdProblem::InvalidAux);
        }
        value
    });
    if namespace.is_empty() {
        problems.push(ItemIdProblem::EmptyNamespace);
    }
    let name = name.to_lowercase();
    let name = name
        .strip_prefix("item.")
        .map_or(name.clone(), str::to_owned);
    let item = ItemName {
        namespace: namespace.to_lowercase(),
        name,
        aux,
        potion: None,
    };
    (item, problems)
}

/// Whether `token` is a number to vanilla's item-name parser: an optional sign and at least one
/// digit.
fn is_number(token: &str) -> bool {
    let digits = token.strip_prefix(['-', '+']).unwrap_or(token);
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

/// A constant `data` value truncated toward zero, saturating at the `i32` range; NaN is 0.
#[allow(
    clippy::cast_possible_truncation,
    reason = "vanilla truncates the float toward zero and saturates"
)]
pub(super) fn truncate(value: f32) -> i32 {
    value as i32
}

/// The value of a constant `data` (a number or a bool); `None` for an expression or a list.
pub(super) fn constant_data(data: &MolangList) -> Option<i32> {
    match data {
        MolangList::One(Molang::Const(value)) => Some(truncate(*value)),
        MolangList::One(Molang::Bool(value)) => Some(i32::from(*value)),
        _ => None,
    }
}

impl Ingredient {
    /// The item it names, split as vanilla splits it; `None` for an object without a string
    /// `item` (a tag ingredient).
    pub fn item_name(&self) -> Option<ItemName> {
        match self {
            Self::Name(text) => Some(ItemName::parse(text)),
            Self::Object(object) => object.item.as_deref().map(ItemName::parse),
        }
    }

    /// The potion the string form names (`minecraft:awkward`).
    pub fn potion(&self) -> Option<String> {
        match self {
            Self::Name(text) => potion(text),
            Self::Object(_) => None,
        }
    }

    /// The item tag of a tag ingredient: an object whose `item` is not a string.
    pub fn tag(&self) -> Option<&str> {
        match self {
            Self::Object(object) if object.item.is_none() => object.tag.as_deref(),
            _ => None,
        }
    }

    /// How many: `count`, default 1.
    pub fn count(&self) -> i32 {
        match self {
            Self::Object(IngredientObject {
                count: Some(count), ..
            }) => *count,
            _ => 1,
        }
    }

    /// The aux value: an inline aux in the item text wins, then what `data` evaluated to (a
    /// constant `data` of a value built by hand counts too), else 32767, any aux.
    pub fn aux(&self) -> i32 {
        if let Some(aux) = self.item_name().and_then(|name| name.aux) {
            return aux;
        }
        match self {
            Self::Object(object) => object
                .data_value
                .or_else(|| object.data.as_ref().and_then(constant_data))
                .unwrap_or(WILDCARD_AUX),
            Self::Name(_) => WILDCARD_AUX,
        }
    }

    /// The aux value of a result item: [`aux`](Self::aux), with 32767 read as 0.
    pub fn result_aux(&self) -> i32 {
        match self.aux() {
            WILDCARD_AUX => 0,
            aux => aux,
        }
    }
}

impl Default for Ingredient {
    /// An empty name: not a valid ingredient until it is given one.
    fn default() -> Self {
        Self::Name(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_token_defaults_to_the_minecraft_namespace() {
        let (name, problems) = parse_item("stick");
        assert_eq!(name.id(), "minecraft:stick");
        assert!(problems.is_empty());
    }

    #[test]
    fn a_numeric_second_token_is_the_aux_of_a_minecraft_item() {
        let (name, problems) = parse_item("coal:1:junk");
        assert_eq!((name.id().as_str(), name.aux), ("minecraft:coal", Some(1)));
        assert!(problems.is_empty());
    }

    #[test]
    fn potion_names_are_potions() {
        let (name, _) = parse_item("minecraft:potion_type:awkward");
        assert_eq!(name.id(), "minecraft:potion");
        assert_eq!(name.potion.as_deref(), Some("minecraft:awkward"));
        assert!(potion("minecraft:potion_type:nonsense").is_none());
    }

    #[test]
    fn potion_names_ignore_case() {
        // vanilla's `brew_water_glow_stone.json` names `minecraft:potion_type:Thick`
        assert_eq!(
            potion("minecraft:potion_type:Thick").as_deref(),
            Some("minecraft:thick")
        );
        assert!(parse_item("minecraft:potion_type:Awkward").1.is_empty());
    }

    #[test]
    fn truncation_saturates_and_maps_nan_to_zero() {
        assert_eq!(truncate(2.9), 2);
        assert_eq!(truncate(-2.9), -2);
        assert_eq!(truncate(f32::NAN), 0);
        assert_eq!(truncate(1e20), i32::MAX);
    }
}
