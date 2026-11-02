//! Writing a recipe back as JSON: the members the author wrote, in the order vanilla reads
//! them, unknown members after the known ones of their level (the Value level).

use facet_value::{VArray, VObject};

use super::{Description, Ingredient, Output, Recipe, Tags, Unlock};
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::{EncodeError, Value, WriteStyle, write_json};
use bedrock_addon_core::molang::write_list;

impl Recipe {
    /// Writes the recipe as JSON text to `out`.
    pub(super) fn write_into(
        &self,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<(), EncodeError> {
        write_json(&self.to_value()?, out, style)
    }

    /// The document as a JSON value.
    fn to_value(&self) -> Result<Value, EncodeError> {
        let mut payload = VObject::new();
        super::new::each!(self, r => {
            description(&mut payload, &r.description);
            if let Some(tags) = &r.tags {
                payload.insert("tags", tags_value(tags));
            }
        });
        match self {
            Self::Shaped(r) => {
                optional(&mut payload, "priority", r.priority.map(int));
                optional(
                    &mut payload,
                    "assume_symmetry",
                    r.assume_symmetry.map(Value::from),
                );
                let pattern: Vec<Value> = r
                    .pattern
                    .iter()
                    .map(|row| Value::from(row.as_str()))
                    .collect();
                payload.insert("pattern", VArray::from(pattern));
                let mut key = VObject::new();
                for (name, ingredient) in &r.key {
                    key.insert(name.as_str(), ingredient_value(ingredient)?);
                }
                payload.insert("key", key);
                payload.insert("result", output_value(&r.result)?);
                optional(
                    &mut payload,
                    "unlock",
                    r.unlock.as_ref().map(unlock_value).transpose()?,
                );
            }
            Self::Shapeless(r) => {
                optional(&mut payload, "priority", r.priority.map(int));
                payload.insert("ingredients", list(&r.ingredients)?);
                payload.insert("result", output_value(&r.result)?);
                optional(
                    &mut payload,
                    "unlock",
                    r.unlock.as_ref().map(unlock_value).transpose()?,
                );
            }
            Self::Furnace(r) => {
                optional(&mut payload, "priority", r.priority.map(int));
                payload.insert("input", ingredient_value(&r.input)?);
                payload.insert("output", ingredient_value(&r.output)?);
                optional(
                    &mut payload,
                    "unlock",
                    r.unlock.as_ref().map(unlock_value).transpose()?,
                );
            }
            Self::BrewingMix(r) => {
                payload.insert("input", ingredient_value(&r.input)?);
                payload.insert("reagent", ingredient_value(&r.reagent)?);
                payload.insert("output", ingredient_value(&r.output)?);
            }
            Self::BrewingContainer(r) => {
                payload.insert("input", ingredient_value(&r.input)?);
                payload.insert("reagent", ingredient_value(&r.reagent)?);
                payload.insert("output", ingredient_value(&r.output)?);
            }
            Self::MaterialReduction(r) => {
                payload.insert("input", ingredient_value(&r.input)?);
                optional(
                    &mut payload,
                    "output",
                    r.output.as_ref().map(output_value).transpose()?,
                );
            }
            Self::SmithingTransform(r) => {
                optional(
                    &mut payload,
                    "template",
                    r.template.as_ref().map(ingredient_value).transpose()?,
                );
                payload.insert("base", ingredient_value(&r.base)?);
                payload.insert("addition", ingredient_value(&r.addition)?);
                payload.insert("result", ingredient_value(&r.result)?);
            }
            Self::SmithingTrim(r) => {
                payload.insert("template", ingredient_value(&r.template)?);
                payload.insert("base", ingredient_value(&r.base)?);
                payload.insert("addition", ingredient_value(&r.addition)?);
            }
        }
        let mut root = VObject::new();
        super::new::each!(self, r => {
            extend(&mut payload, &r.extra);
            if let Some(version) = r.format_version {
                root.insert("format_version", version.to_string());
            }
            root.insert(self.kind().payload_key(), payload);
            extend(&mut root, &r.root_extra);
        });
        Ok(root.into_value())
    }
}

fn description(payload: &mut VObject, description: &Description) {
    let mut object = VObject::new();
    object.insert("identifier", description.identifier.as_str());
    extend(&mut object, &description.extra);
    payload.insert("description", object);
}

fn tags_value(tags: &Tags) -> Value {
    match tags {
        Tags::One(tag) => Value::from(tag.as_str()),
        Tags::Many(tags) => {
            let tags: Vec<Value> = tags.iter().map(|tag| Value::from(tag.as_str())).collect();
            VArray::from(tags).into_value()
        }
    }
}

fn int(value: i32) -> Value {
    Value::from(i128::from(value))
}

fn optional(object: &mut VObject, key: &str, value: Option<Value>) {
    if let Some(value) = value {
        object.insert(key, value);
    }
}

fn extend(object: &mut VObject, extra: &Extra) {
    for (key, value) in extra.iter() {
        object.insert(key, value.clone());
    }
}

fn ingredient_value(ingredient: &Ingredient) -> Result<Value, EncodeError> {
    match ingredient {
        Ingredient::Name(name) => Ok(Value::from(name.as_str())),
        Ingredient::Object(object) => {
            let mut value = VObject::new();
            if let Some(data) = &object.data {
                let data = write_list(data).map_err(|what| EncodeError::NotRepresentable {
                    at: JsonPath::new(),
                    what,
                })?;
                value.insert("data", data);
            }
            optional(&mut value, "count", object.count.map(int));
            optional(&mut value, "item", object.item.as_deref().map(Value::from));
            optional(&mut value, "tag", object.tag.as_deref().map(Value::from));
            extend(&mut value, &object.extra);
            Ok(value.into_value())
        }
    }
}

fn list(ingredients: &[Ingredient]) -> Result<Value, EncodeError> {
    let items = ingredients
        .iter()
        .map(ingredient_value)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(VArray::from(items).into_value())
}

fn output_value(output: &Output) -> Result<Value, EncodeError> {
    match output {
        Output::One(item) => ingredient_value(item),
        Output::Many(items) => list(items),
    }
}

fn unlock_value(unlock: &Unlock) -> Result<Value, EncodeError> {
    match unlock {
        Unlock::Ingredients(items) => list(items),
        Unlock::Malformed(value) => Ok(value.clone()),
        Unlock::Context { context, extra } => {
            let mut object = VObject::new();
            object.insert("context", context.as_str());
            extend(&mut object, extra);
            Ok(object.into_value())
        }
    }
}
