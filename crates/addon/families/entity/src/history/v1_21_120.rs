//! Format 1.21.120: the struct and the step to 1.21.130.

super::legacy_struct!(1, 21, 120);

use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};
use facet_value::{VArray, VObject};

use super::ops::{Told, below, element, half, overwrite, section_b, sections, single_to_array};
use bedrock_addon_core::diagnostic::JsonPath;

/// 1.21.120 → 1.21.130.
impl Upgrade for Entity {
    type Next = super::v1_21_130::Entity;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut payload = self.0;
        // The conversions of this step whose legacy reader is one of the game's schema-driven
        // readers.
        super::ops::schema_conversions(&mut payload, super::v1_21_130::Entity::VERSION, cx);
        // B, the 1.21.130 component upgrades.
        if section_b(&payload, cx) {
            let mut told = Told::default();
            for section in sections(&mut payload) {
                let members = section.members;
                if let Some(ageable) = members.object("minecraft:ageable") {
                    let at = below(&section.at, &["minecraft:ageable"]);
                    with_result_item(ageable, "feed_items", &at, &mut told);
                }
                if let Some(breedable) = members.object("minecraft:breedable") {
                    // Whether the entries are elements in the file, or `breed_items` is the
                    // one entry.
                    let listed = breedable.get("breed_items").is_some_and(Value::is_array);
                    single_to_array(breedable, "breed_items");
                    let at = below(&section.at, &["minecraft:breedable"]);
                    with_result_item_of(breedable, "breed_items", listed, &at, &mut told);
                }
                if let Some(goal) = members.object("minecraft:behavior.nearest_attackable_target") {
                    let at = below(
                        &section.at,
                        &["minecraft:behavior.nearest_attackable_target"],
                    );
                    interval_range(goal, &at, &mut told);
                }
            }
            told.report(cx);
        }
        super::v1_21_130::Entity(payload)
    }
}

/// Each entry of `items` (a string or an object) becomes `{item, result_item:
/// <transform_to_item>}`; the root `transform_to_item` is then removed.
fn with_result_item(component: &mut VObject, items: &str, at: &JsonPath, told: &mut Told) {
    with_result_item_of(component, items, true, at, told);
}

/// [`with_result_item`]; `listed` is whether the entries are elements of `items` in the file
/// (`breed_items` may be the one entry). The crate's own note, told at the member (`at` is the
/// component's path): a `transform_to_item` that no entry takes (no list, an empty one) is dropped;
/// a `result_item` an entry had is replaced by it. Both are notes: informations about a value the
/// game discards itself.
fn with_result_item_of(
    component: &mut VObject,
    items: &str,
    listed: bool,
    at: &JsonPath,
    told: &mut Told,
) {
    // A `null` member is an absent one.
    let result = component
        .remove("transform_to_item")
        .filter(|result| !result.is_null());
    let list = component
        .get_mut(items)
        .and_then(|i| i.as_array_mut())
        .filter(|list| !list.is_empty());
    let Some(list) = list else {
        if result.is_some() {
            // No entry takes it: the member goes. A note (an information): the step is the
            // game's, and nothing shows that the game keeps the value where `items` is one
            // string (vanilla's own axolotl has it so) or is not there at all.
            told.dropped(below(at, &["transform_to_item"]));
        }
        return;
    };
    let mut out = VArray::new();
    for (index, entry) in list.iter().enumerate() {
        let mut object = entry.as_object().cloned().unwrap_or_else(|| {
            let mut object = VObject::new();
            object.insert("item", entry.clone());
            object
        });
        if let Some(result) = &result {
            let items_at = below(at, &[items]);
            let entry_at = if listed {
                element(&items_at, index)
            } else {
                items_at
            };
            overwrite(&mut object, "result_item", result.clone(), &entry_at, told);
        }
        out.push(object);
    }
    *list = out;
}

/// `attack_interval` (a number) with `attack_interval_min` becomes `{range_min, range_max}`; a
/// lone number is kept.
fn interval_range(goal: &mut VObject, at: &JsonPath, told: &mut Told) {
    let number = goal.get("attack_interval").is_some_and(Value::is_number);
    if !number || !goal.contains_key("attack_interval_min") {
        return;
    }
    // A minimum that is no number leaves the lone interval as it is.
    let Some(min) = half(goal, "attack_interval_min", at, told) else {
        return;
    };
    let Some(max) = goal.remove("attack_interval") else {
        return;
    };
    let mut range = VObject::new();
    range.insert("range_min", min);
    range.insert("range_max", max);
    goal.insert("attack_interval", range);
    told.moved(
        below(at, &["attack_interval"]),
        below(at, &["attack_interval", "range_max"]),
    );
    told.moved(
        below(at, &["attack_interval_min"]),
        below(at, &["attack_interval", "range_min"]),
    );
}
