//! Writing [`LootTable`]: the JSON value, member by member, in declaration order with each
//! level's unknown members after its known ones. A member the value does not hold (`None`) is
//! not written.

use facet_value::{VArray, VObject};

use super::condition::{ConditionKind, LootCondition, ToolEnchantment};
use super::function::{
    BannerPattern, BannerType, EnchantOption, FunctionKind, ItemColor, ItemData, LootFunction,
    SpecificEnchant, StewEffect,
};
use super::{Entry, EntryKind, LootTable, Pool, RandomValueBounds, Tiers};
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::{Id, Kind};
use bedrock_addon_core::json::{EncodeError, Unrepresentable, Value, WriteStyle, write_json};
use bedrock_addon_core::shared::IntRange;

impl LootTable {
    /// Writes the table as JSON text.
    pub(crate) fn write_into(
        &self,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<(), EncodeError> {
        let value = table(self, &JsonPath::new())?;
        write_json(&value, out, style)
    }
}

type Written = Result<Value, EncodeError>;

/// A member with one JSON form that always exists: a scalar, a name, a list of strings, a
/// bounds value.
trait Plain {
    fn value(&self) -> Value;
}

impl Plain for i32 {
    fn value(&self) -> Value {
        Value::from(i64::from(*self))
    }
}

/// A real in its shortest form (`0.1`, not `0.10000000149011612`). A number that is not finite
/// has none: it is handed on as it is, and the JSON writer refuses it where it stands
/// (`EncodeError::NotRepresentable`).
impl Plain for f32 {
    fn value(&self) -> Value {
        let shortest = format!("{self:?}").parse().unwrap_or(f64::from(*self));
        Value::from(shortest)
    }
}

impl Plain for bool {
    fn value(&self) -> Value {
        Value::from(*self)
    }
}

impl Plain for String {
    fn value(&self) -> Value {
        Value::from(self.as_str())
    }
}

impl Plain for Vec<String> {
    fn value(&self) -> Value {
        let mut array = VArray::new();
        for text in self {
            array.push(Value::from(text.as_str()));
        }
        array.into()
    }
}

/// An identifier as written.
impl<K: Kind> Plain for Id<K> {
    fn value(&self) -> Value {
        Value::from(self.as_str())
    }
}

impl Plain for BannerType {
    fn value(&self) -> Value {
        Value::from(self.name())
    }
}

impl Plain for ItemColor {
    fn value(&self) -> Value {
        Value::from(self.name())
    }
}

impl Plain for RandomValueBounds {
    fn value(&self) -> Value {
        match self {
            Self::Exact(x) => x.value(),
            Self::Range { min, max } => {
                let mut object = VObject::new();
                if let Some(min) = min {
                    object.insert("min", min.value());
                }
                if let Some(max) = max {
                    object.insert("max", max.value());
                }
                object.into()
            }
        }
    }
}

/// One object being written.
struct Object<'a> {
    object: VObject,
    at: &'a JsonPath,
}

impl<'a> Object<'a> {
    fn new(at: &'a JsonPath) -> Self {
        Self {
            object: VObject::new(),
            at,
        }
    }

    /// Writes `key` with `value`.
    fn put(&mut self, key: &str, value: impl Into<Value>) {
        self.object.insert(key, value.into());
    }

    /// Writes `key` when `value` is held.
    fn set<T: Plain>(&mut self, key: &str, value: Option<&T>) {
        if let Some(value) = value {
            self.object.insert(key, value.value());
        }
    }

    /// Writes `key` when `value` is held, with `write` at `at/key`.
    fn opt<T: ?Sized>(
        &mut self,
        key: &str,
        value: Option<&T>,
        write: impl FnOnce(&T, &JsonPath) -> Written,
    ) -> Result<(), EncodeError> {
        if let Some(value) = value {
            let path = self.at.child(key);
            let written = write(value, &path)?;
            self.object.insert(key, written);
        }
        Ok(())
    }

    /// Writes `key` when the list is held: each element by `write` at its index.
    fn list<T>(
        &mut self,
        key: &str,
        list: Option<&Vec<T>>,
        write: impl Fn(&T, &JsonPath) -> Written,
    ) -> Result<(), EncodeError> {
        self.opt(key, list, |list, at| {
            let mut array = VArray::new();
            for (index, element) in list.iter().enumerate() {
                let path = at.index(index);
                array.push(write(element, &path)?);
            }
            Ok(array.into())
        })
    }

    /// Writes the unknown members after the known ones and closes the object.
    fn finish(mut self, extra: &Extra) -> Value {
        for (key, value) in extra.iter() {
            self.object.insert(key, value.clone());
        }
        self.object.into()
    }
}

fn int_range(range: &IntRange, at: &JsonPath) -> Written {
    Value::try_from(range).map_err(|_| EncodeError::NotRepresentable {
        at: at.clone(),
        what: Unrepresentable::Unsupported,
    })
}

// ---- the table -----------------------------------------------------------------------------

fn table(table: &LootTable, at: &JsonPath) -> Written {
    let mut o = Object::new(at);
    o.list("pools", table.pools.as_ref(), pool)?;
    Ok(o.finish(&table.extra))
}

fn pool(pool: &Pool, at: &JsonPath) -> Written {
    let mut o = Object::new(at);
    o.set("rolls", pool.rolls.as_ref());
    o.set("bonus_rolls", pool.bonus_rolls.as_ref());
    o.list("conditions", pool.conditions.as_ref(), condition)?;
    o.opt("tiers", pool.tiers.as_ref(), |tiers, at| {
        Ok(self::tiers(tiers, at))
    })?;
    o.list("entries", pool.entries.as_ref(), entry)?;
    Ok(o.finish(&pool.extra))
}

fn tiers(tiers: &Tiers, at: &JsonPath) -> Value {
    let mut o = Object::new(at);
    o.set("initial_range", tiers.initial_range.as_ref());
    o.set("bonus_rolls", tiers.bonus_rolls.as_ref());
    o.set("bonus_chance", tiers.bonus_chance.as_ref());
    o.finish(&tiers.extra)
}

fn entry(entry: &Entry, at: &JsonPath) -> Written {
    let mut o = Object::new(at);
    let functions = match &entry.kind {
        EntryKind::Item { name, functions } => {
            o.put("type", "item");
            o.set("name", name.as_ref());
            functions.as_ref()
        }
        EntryKind::LootTable { name } => {
            o.put("type", "loot_table");
            o.set("name", name.as_ref());
            None
        }
        EntryKind::Empty => {
            o.put("type", "empty");
            None
        }
    };
    o.set("weight", entry.weight.as_ref());
    o.set("quality", entry.quality.as_ref());
    o.list("functions", functions, function)?;
    o.list("conditions", entry.conditions.as_ref(), condition)?;
    o.list("pools", entry.pools.as_ref(), pool)?;
    Ok(o.finish(&entry.extra))
}

// ---- functions -----------------------------------------------------------------------------

fn function(function: &LootFunction, at: &JsonPath) -> Written {
    let mut o = Object::new(at);
    o.put("function", function.function_name());
    parameters(&mut o, &function.kind)?;
    o.list("conditions", function.conditions.as_ref(), condition)?;
    Ok(o.finish(&function.extra))
}

/// The parameters of a function, under the keys its parser reads.
fn parameters(o: &mut Object<'_>, kind: &FunctionKind) -> Result<(), EncodeError> {
    match kind {
        FunctionKind::SetCount { count } | FunctionKind::LootingEnchant { count } => {
            o.set("count", count.as_ref());
        }
        FunctionKind::SetData(data) | FunctionKind::RandomAuxValue(data) => item_data(o, data),
        FunctionKind::SetDamage { damage } => o.set("damage", damage.as_ref()),
        FunctionKind::EnchantWithLevels { levels, treasure } => {
            o.set("levels", levels.as_ref());
            o.set("treasure", treasure.as_ref());
        }
        FunctionKind::EnchantBookForTrading {
            base_cost,
            per_level_cost,
            base_random_cost,
            per_level_random_cost,
            enchantments,
        } => {
            o.set("base_cost", base_cost.as_ref());
            o.set("per_level_cost", per_level_cost.as_ref());
            o.set("base_random_cost", base_random_cost.as_ref());
            o.set("per_level_random_cost", per_level_random_cost.as_ref());
            o.list("enchantments", enchantments.as_ref(), |option, at| {
                Ok(enchant_option(option, at))
            })?;
        }
        FunctionKind::EnchantRandomly { treasure } => o.set("treasure", treasure.as_ref()),
        FunctionKind::FurnaceSmelt
        | FunctionKind::SetDataFromColorIndex
        | FunctionKind::RandomDye
        | FunctionKind::ExplosionDecay => {}
        FunctionKind::EnchantRandomGear { chance } => o.set("chance", chance.as_ref()),
        FunctionKind::RandomBlockState {
            block_state,
            values,
        } => {
            o.set("block_state", block_state.as_ref());
            o.set("values", values.as_ref());
        }
        FunctionKind::ExplorationMap { destination } => {
            o.set("destination", destination.as_ref());
        }
        FunctionKind::SetBannerDetails {
            banner_type,
            base_color,
            patterns,
        } => {
            o.set("type", banner_type.as_ref());
            o.set("base_color", base_color.as_ref());
            o.list("patterns", patterns.as_ref(), |pattern, at| {
                Ok(banner_pattern(pattern, at))
            })?;
        }
        FunctionKind::SetName { name } => o.set("name", name.as_ref()),
        FunctionKind::SetLore { lore } => o.set("lore", lore.as_ref()),
        FunctionKind::SpecificEnchants { enchants } => {
            o.list("enchants", enchants.as_ref(), specific_enchant)?;
        }
        FunctionKind::FillContainer { loot_table } => o.set("loot_table", loot_table.as_ref()),
        FunctionKind::SetActorId { id } => o.set("id", id.as_ref()),
        FunctionKind::SetBookContents {
            author,
            title,
            pages,
        } => {
            o.set("author", author.as_ref());
            o.set("title", title.as_ref());
            o.set("pages", pages.as_ref());
        }
        FunctionKind::SetStewEffect { effects } => {
            o.list("effects", effects.as_ref(), |effect, at| {
                Ok(stew_effect(effect, at))
            })?;
        }
        FunctionKind::SetOminousBottleAmplifier { amplifier } => {
            o.set("amplifier", amplifier.as_ref());
        }
        FunctionKind::SetArmorTrim { material, pattern } => {
            o.set("material", material.as_ref());
            o.set("pattern", pattern.as_ref());
        }
        FunctionKind::SetPotion { id } => o.set("id", id.as_ref()),
        FunctionKind::CarryOverBlockEntityData { dynamic_properties } => {
            o.set("dynamic_properties", dynamic_properties.as_ref());
        }
    }
    Ok(())
}

fn item_data(o: &mut Object<'_>, data: &ItemData) {
    o.set("data", data.data.as_ref());
    o.set("values", data.values.as_ref());
}

fn enchant_option(option: &EnchantOption, at: &JsonPath) -> Value {
    let mut o = Object::new(at);
    o.put("name", option.name.as_str());
    o.set("min", option.min.as_ref());
    o.set("max", option.max.as_ref());
    o.finish(&option.extra)
}

fn banner_pattern(pattern: &BannerPattern, at: &JsonPath) -> Value {
    let mut o = Object::new(at);
    o.put("pattern", pattern.pattern.as_str());
    o.set("color", pattern.color.as_ref());
    o.finish(&pattern.extra)
}

/// The string form when the enchant has no level and no other member, else the object.
fn specific_enchant(enchant: &SpecificEnchant, at: &JsonPath) -> Written {
    if enchant.level.is_none() && enchant.extra.is_empty() {
        return Ok(Value::from(enchant.id.as_str()));
    }
    let mut o = Object::new(at);
    o.put("id", enchant.id.as_str());
    o.opt("level", enchant.level.as_ref(), int_range)?;
    Ok(o.finish(&enchant.extra))
}

fn stew_effect(effect: &StewEffect, at: &JsonPath) -> Value {
    let mut o = Object::new(at);
    o.set("id", effect.id.as_ref());
    o.finish(&effect.extra)
}

// ---- conditions ----------------------------------------------------------------------------

fn condition(condition: &LootCondition, at: &JsonPath) -> Written {
    let mut o = Object::new(at);
    o.put("condition", condition.kind.name());
    match &condition.kind {
        ConditionKind::KilledByPlayer
        | ConditionKind::KilledByPlayerOrPets
        | ConditionKind::IsBaby => {}
        ConditionKind::KilledByEntity { entity_type }
        | ConditionKind::EntityKilled { entity_type }
        | ConditionKind::DamagedByEntity { entity_type }
        | ConditionKind::PassengerOfEntity { entity_type } => {
            o.set("entity_type", entity_type.as_ref());
        }
        ConditionKind::RandomChance { chance } => o.set("chance", chance.as_ref()),
        ConditionKind::RandomDifficultyChance {
            default_chance,
            peaceful,
            easy,
            normal,
            hard,
        } => {
            o.set("default_chance", default_chance.as_ref());
            o.set("peaceful", peaceful.as_ref());
            o.set("easy", easy.as_ref());
            o.set("normal", normal.as_ref());
            o.set("hard", hard.as_ref());
        }
        ConditionKind::RandomChanceWithLooting {
            chance,
            looting_multiplier,
        } => {
            o.set("chance", chance.as_ref());
            o.set("looting_multiplier", looting_multiplier.as_ref());
        }
        ConditionKind::RandomRegionalDifficultyChance { max_chance } => {
            o.set("max_chance", max_chance.as_ref());
        }
        ConditionKind::HasMarkVariant { value } | ConditionKind::HasVariant { value } => {
            o.set("value", value.as_ref());
        }
        ConditionKind::MatchTool {
            item,
            count,
            durability,
            enchantments,
            filter_any,
            filter_all,
            filter_none,
        } => {
            o.set("item", item.as_ref());
            o.opt("count", count.as_ref(), int_range)?;
            o.opt("durability", durability.as_ref(), int_range)?;
            o.list("enchantments", enchantments.as_ref(), tool_enchantment)?;
            o.set("minecraft:match_tool_filter_any", filter_any.as_ref());
            o.set("minecraft:match_tool_filter_all", filter_all.as_ref());
            o.set("minecraft:match_tool_filter_none", filter_none.as_ref());
        }
        ConditionKind::HasProperty { domain } => o.set("domain", domain.as_ref()),
        ConditionKind::BoolProperty { domain, value } => {
            o.set("domain", domain.as_ref());
            o.set("value", value.as_ref());
        }
        ConditionKind::IntProperty { domain, value } => {
            o.set("domain", domain.as_ref());
            o.set("value", value.as_ref());
        }
        ConditionKind::FloatProperty { domain, value } => {
            o.set("domain", domain.as_ref());
            o.set("value", value.as_ref());
        }
        ConditionKind::EnumProperty { domain, value } => {
            o.set("domain", domain.as_ref());
            o.set("value", value.as_ref());
        }
        ConditionKind::BiomeHasTag { tag } => o.set("tag", tag.as_ref()),
    }
    Ok(o.finish(&condition.extra))
}

fn tool_enchantment(enchantment: &ToolEnchantment, at: &JsonPath) -> Written {
    let mut o = Object::new(at);
    o.set("enchantment", enchantment.enchantment.as_ref());
    o.opt("levels", enchantment.levels.as_ref(), int_range)?;
    Ok(o.finish(&enchantment.extra))
}
