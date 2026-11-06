//! Components about trading, bartering and taming: `admire_item`, `barter`, `trade_table`,
//! `tamemount`; and the components 1.26.60 types first: `inventory`, `shareables`,
//! `economy_trade_table`.
// Types and defaults follow the game's schemas (1.26.40 `admire_item`; 1.26.50 `barter`,
// the legacy tradeable component and `tamemount`).

use crate::component::descriptor_tags;
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::shared::{EventTrigger, IntRange, RangeForm};
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_shared_shapes::ItemDescriptor;
use facet::Facet;
use facet_value::VObject;

use super::parse::{ComponentCheck, Typed};
use super::value::visit_item_tags;

/// `minecraft:admire_item`: the entity admires a held item for a while, ignoring targets.
#[doc(alias = "minecraft:admire_item")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct AdmireItem {
    /// `cooldown_after_being_attacked`: seconds after being attacked during which it does not
    /// admire; default 10.
    #[facet(default = 10_i32)]
    pub cooldown_after_being_attacked: i32,
    /// `duration`: seconds the admiring lasts; default 10.
    #[facet(default = 10_i32)]
    pub duration: i32,
}

impl Typed for AdmireItem {}

/// `minecraft:barter`: the entity drops items from a loot table in exchange for a bartered
/// item.
#[doc(alias = "minecraft:barter")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Barter {
    /// `barter_table`: the loot table path; default none (empty).
    #[facet(default = String::new())]
    pub barter_table: String,
    /// `cooldown_after_being_attacked`: seconds after being attacked during which it does not
    /// barter; default 10.
    #[facet(default = 10_i32)]
    pub cooldown_after_being_attacked: i32,
}

impl Typed for Barter {}

/// `minecraft:trade_table`: the entity trades from a trade table (the legacy trading screen
/// unless `new_screen`).
#[doc(alias = "minecraft:trade_table")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct TradeTable {
    /// `convert_trades_economy`: the trades convert to the economy trade table; default `false`.
    #[facet(default = false)]
    pub convert_trades_economy: bool,
    /// `display_name`: the trading screen's title; default empty.
    #[facet(default = String::new())]
    pub display_name: String,
    /// `new_screen`: the newer trading screen is used; default `false`.
    #[facet(default = false)]
    pub new_screen: bool,
    /// `persist_trades`: the trades are kept when the entity changes; default `false`.
    #[facet(default = false)]
    pub persist_trades: bool,
    /// `table`: the trade table path; default none (empty).
    #[facet(default = String::new())]
    pub table: String,
}

impl Typed for TradeTable {}

/// `minecraft:tamemount`: the entity can be tamed by riding it and feeding it.
#[doc(alias = "minecraft:tamemount")]
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct Tamemount {
    /// `attempt_temper_mod`: temper added on each failed taming attempt; default 5.
    #[facet(default = 5_i32)]
    pub attempt_temper_mod: i32,
    /// `auto_reject_items`: items the entity refuses when fed.
    #[facet(default)]
    pub auto_reject_items: Vec<RejectItem>,
    /// `feed_items`: items that raise the temper when fed.
    #[facet(default)]
    pub feed_items: Vec<FeedItem>,
    /// `feed_text`: the interaction text for feeding; default empty.
    #[facet(default = String::new())]
    pub feed_text: String,
    /// `max_temper`: the highest starting temper; default 100.
    #[facet(default = 100_i32)]
    pub max_temper: i32,
    /// `min_temper`: the lowest starting temper; default 0.
    #[facet(default = 0_i32)]
    pub min_temper: i32,
    /// `ride_text`: the interaction text for mounting; default empty.
    #[facet(default = String::new())]
    pub ride_text: String,
    /// `tame_event`: the event fired when the entity is tamed; absent means none.
    pub tame_event: Option<EventTrigger>,
}

impl Default for Tamemount {
    fn default() -> Self {
        Self {
            attempt_temper_mod: 5,
            auto_reject_items: Vec::new(),
            feed_items: Vec::new(),
            feed_text: String::new(),
            max_temper: 100,
            min_temper: 0,
            ride_text: String::new(),
            tame_event: None,
        }
    }
}

/// One item of [`Tamemount::feed_items`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct FeedItem {
    /// `item`: the item; absent matches nothing.
    #[facet(schema::molang_with = descriptor_tags)]
    pub item: Option<ItemDescriptor>,
    /// `temper_mod`: temper added when it is fed; default 0.
    #[facet(default = 0_i32)]
    pub temper_mod: i32,
}

/// One item of [`Tamemount::auto_reject_items`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct RejectItem {
    /// `item`: the item; absent matches nothing.
    #[facet(schema::molang_with = descriptor_tags)]
    pub item: Option<ItemDescriptor>,
}

impl Typed for Tamemount {
    /// The legacy reader's camel-case spellings (`minTemper|min_temper`, …, and
    /// `temperMod|temper_mod` in each feed item).
    fn prepare(json: &mut VObject, cx: &mut ComponentCheck<'_>) -> bool {
        let mut changed = super::parse::legacy_spellings(
            json,
            &JsonPath::new(),
            &[
                ("minTemper", "min_temper"),
                ("maxTemper", "max_temper"),
                ("attemptTemperMod", "attempt_temper_mod"),
                ("feedItems", "feed_items"),
                ("autoRejectItems", "auto_reject_items"),
            ],
            cx,
        );
        if let Some(items) = json.get_mut("feed_items").and_then(|v| v.as_array_mut()) {
            for (i, item) in items.as_mut_slice().iter_mut().enumerate() {
                if let Some(item) = item.as_object_mut() {
                    let at = JsonPath::new().child("feed_items").index(i);
                    changed |= super::parse::legacy_spellings(
                        item,
                        &at,
                        &[("temperMod", "temper_mod")],
                        cx,
                    );
                }
            }
        }
        changed
    }

    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        let feed = JsonPath::new().child("feed_items");
        for (i, entry) in self.feed_items.iter().enumerate() {
            let mut at = feed.index(i);
            at.push("item");
            if let Some(item) = &entry.item {
                cx.extend(item.check(&at));
            }
        }
        let reject = JsonPath::new().child("auto_reject_items");
        for (i, entry) in self.auto_reject_items.iter().enumerate() {
            let mut at = reject.index(i);
            at.push("item");
            if let Some(item) = &entry.item {
                cx.extend(item.check(&at));
            }
        }
        if let Some(event) = &self.tame_event {
            cx.extend(event.check(&JsonPath::new().child("tame_event"), cx.unknown()));
        }
    }
}

impl MolangFields for Tamemount {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.enter("feed_items".into());
        for (i, entry) in self.feed_items.iter_mut().enumerate() {
            visitor.enter(i.into());
            visitor.enter("item".into());
            if let Some(item) = &mut entry.item {
                visit_item_tags(item, visitor);
            }
            visitor.leave();
            visitor.leave();
        }
        visitor.leave();
        visitor.enter("auto_reject_items".into());
        for (i, entry) in self.auto_reject_items.iter_mut().enumerate() {
            visitor.enter(i.into());
            visitor.enter("item".into());
            if let Some(item) = &mut entry.item {
                visit_item_tags(item, visitor);
            }
            visitor.leave();
            visitor.leave();
        }
        visitor.leave();
    }
}

/// `minecraft:inventory`: the entity's inventory.
// `inventory`: `container_type` defaults to the string "none"; the other defaults are 5,
// false, false, false and 0.
#[doc(alias = "minecraft:inventory")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Inventory {
    /// `container_type`: the kind of container (`horse`, `minecart_chest`, `chest_boat`,
    /// `minecart_hopper`, `inventory`, `container`, `hopper`, …); default `none`.
    #[facet(default = "none".to_owned())]
    pub container_type: String,
    /// `inventory_size`: the number of slots; default 5.
    #[facet(default = 5_i32)]
    pub inventory_size: i32,
    /// `can_be_siphoned_from`: a hopper can take items out; default `false`.
    #[facet(default = false)]
    pub can_be_siphoned_from: bool,
    /// `private`: the entity does not drop its inventory on death; default `false`.
    #[facet(default = false)]
    pub private: bool,
    /// `restrict_to_owner`: only the owner (or the entity) opens it; default `false`.
    #[facet(default = false)]
    pub restrict_to_owner: bool,
    /// `additional_slots_per_strength`: slots gained per extra strength; default 0.
    #[facet(default = 0_i32)]
    pub additional_slots_per_strength: i32,
}

impl Typed for Inventory {}

/// `minecraft:shareables`: the items the mob wants to share (and picks up).
// `shareables`: amounts default to −1, priority to 0 and the flags to false.
#[doc(alias = "minecraft:shareables")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Shareables {
    /// `singular_pickup`: it picks up no more of an item it already holds; default `false`.
    #[facet(default = false)]
    pub singular_pickup: bool,
    /// `all_items`: every other item is shareable too, at the lowest priority; default
    /// `false`.
    #[facet(default = false)]
    pub all_items: bool,
    /// `all_items_want_amount`: how many of such an item it wants; default −1.
    #[facet(default = -1_i32)]
    pub all_items_want_amount: i32,
    /// `all_items_surplus_amount`: how many of such an item are extra; default −1.
    #[facet(default = -1_i32)]
    pub all_items_surplus_amount: i32,
    /// `all_items_max_amount`: the most of such an item it holds; default −1.
    #[facet(default = -1_i32)]
    pub all_items_max_amount: i32,
    /// `items`: the items, matched from the first.
    #[facet(default)]
    pub items: Vec<ShareableItem>,
}

/// One item of [`Shareables`].
#[allow(
    clippy::struct_excessive_bools,
    reason = "the game's flags, one member each"
)]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ShareableItem {
    /// `item`: the item or item tag; absent is no item.
    #[facet(schema::molang_with = descriptor_tags)]
    pub item: Option<ItemDescriptor>,
    /// `want_amount`: how many it wants; default −1.
    #[facet(default = -1_i32)]
    pub want_amount: i32,
    /// `surplus_amount`: how many are extra; default −1.
    #[facet(default = -1_i32)]
    pub surplus_amount: i32,
    /// `max_amount`: the most it holds; default −1.
    #[facet(default = -1_i32)]
    pub max_amount: i32,
    /// `pickup_limit`: the most it picks up in one goal tick; default −1.
    #[facet(default = -1_i32)]
    pub pickup_limit: i32,
    /// `craft_into`: the item it crafts with this one; default none.
    #[facet(default = String::new())]
    pub craft_into: String,
    /// `priority`: which items it prefers, 0 first; default 0.
    #[facet(default = 0_i32)]
    pub priority: i32,
    /// `admire`: it admires the item after picking it up; default `false`.
    #[facet(default = false)]
    pub admire: bool,
    /// `barter`: it barters for the item; default `false`.
    #[facet(default = false)]
    pub barter: bool,
    /// `stored_in_inventory`: it puts the item in its inventory when it cannot equip it;
    /// default `false`.
    #[facet(default = false)]
    pub stored_in_inventory: bool,
    /// `consume_item`: it consumes the item; default `false`.
    #[facet(default = false)]
    pub consume_item: bool,
    /// `pickup_only`: it picks the item up and does not drop it; default `false`.
    #[facet(default = false)]
    pub pickup_only: bool,
}

impl Typed for Shareables {
    // Not KEEP_NESTED: the legacy reader is one of the game's schema readers, which validates
    // the whole tree before it parses: a nested value of another type never reaches this
    // read; from 1.15.0 it defaults the whole component.

    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        let list = JsonPath::new().child("items");
        for (i, entry) in self.items.iter().enumerate() {
            if let Some(item) = &entry.item {
                cx.extend(item.check(&list.index(i).child("item")));
            }
        }
    }
}

/// `minecraft:economy_trade_table`: the entity trades with players by the economy rules.
// `economy_trade_table`: `use_legacy_price_formula` defaults to false (an older document gets
// `true` from the 1.16.0 step).
#[allow(
    clippy::struct_excessive_bools,
    reason = "the game's flags, one member each"
)]
#[doc(alias = "minecraft:economy_trade_table")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct EconomyTradeTable {
    /// `display_name`: the localization key of the trade screen's name; default none.
    #[facet(default = String::new())]
    pub display_name: String,
    /// `table`: the trade table file; default none.
    #[facet(default = String::new())]
    pub table: String,
    /// `new_screen`: the newer trade screen is used; default `false`.
    #[facet(default = false)]
    pub new_screen: bool,
    /// `hero_demand_discount`: the demand discount while the player is Hero of the Village;
    /// default −4.
    #[facet(default = -4_i32)]
    pub hero_demand_discount: i32,
    /// `cured_discount`: the discount range of a cured villager; default −25 to −20.
    #[facet(default = IntRange::new(-25, -20, RangeForm::Array))]
    pub cured_discount: IntRange,
    /// `max_cured_discount`: the largest such range; default −25 to −20.
    #[facet(default = IntRange::new(-25, -20, RangeForm::Array))]
    pub max_cured_discount: IntRange,
    /// `nearby_cured_discount`: the discount per cured villager nearby (legacy pricing);
    /// default −20.
    #[facet(default = -20_i32)]
    pub nearby_cured_discount: i32,
    /// `max_nearby_cured_discount`: the most of that discount; default −200.
    #[facet(default = -200_i32)]
    pub max_nearby_cured_discount: i32,
    /// `persist_trades`: the offers are saved with the entity; default `false`.
    #[facet(default = false)]
    pub persist_trades: bool,
    /// `convert_trades_economy`: legacy trades become economy trades; default `false`.
    #[facet(default = false)]
    pub convert_trades_economy: bool,
    /// `show_trade_screen`: interacting opens the trade screen; default `true`.
    #[facet(default = true)]
    pub show_trade_screen: bool,
    /// `use_legacy_price_formula`: the legacy price formula is used; default `false`.
    #[facet(default = false)]
    pub use_legacy_price_formula: bool,
}

impl Typed for EconomyTradeTable {
    // Not KEEP_NESTED: the legacy reader is one of the game's schema readers, which validates
    // the whole tree before it parses: a nested value of another type never reaches this
    // read; from 1.15.0 it defaults the whole component.

    fn prepare(json: &mut VObject, cx: &mut ComponentCheck<'_>) -> bool {
        if !matches!(cx.mode(), ConvertMode::Lenient) {
            return false;
        }
        let mut changed = false;
        for key in ["cured_discount", "max_cured_discount"] {
            changed |= super::movement::legacy_range(json, key, true, &JsonPath::new(), cx);
        }
        changed
    }

    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        cx.range(
            &JsonPath::new().child("cured_discount"),
            &mut self.cured_discount,
        );
        cx.range(
            &JsonPath::new().child("max_cured_discount"),
            &mut self.max_cured_discount,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tame_mount_items_default_to_the_empty_object() {
        let feed = bedrock_addon_core::json::from_str::<FeedItem>("{}").expect("parses");
        assert_eq!(FeedItem::default(), feed);
        let reject = bedrock_addon_core::json::from_str::<RejectItem>("{}").expect("parses");
        assert_eq!(RejectItem::default(), reject);
    }
}

crate::trigger_fields!(Tamemount {
    tame_event: Option<EventTrigger>,
});
