//! The legacy parser: a hand-written walk over the document's JSON tree that reads the table,
//! its groups, trades and items with defaults, as vanilla's does. Scalars are read
//! the way `Json::Value::as*` converts them; a value no conversion takes is a warning
//! and reads as absent.

use super::v0::{Group, ItemList, Tier, Trade, TradeItem, TradeTable};
use crate::texts::{
    GROUPS_ARRAY, INVALID_WEIGHT, MUST_CONTAIN_TIERS, NO_OFFERS, NO_RECEIVES, TIERS_ARRAY,
    TRADES_ARRAY, formatted,
};
use crate::validate as rules;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::json::{JsonSlice, Value};

/// The members the legacy parser reads, per object.
const ROOT_MEMBERS: &[&str] = &["format_version", "tiers"];
/// A tier's members.
const TIER_MEMBERS: &[&str] = &["total_exp_required", "groups", "trades"];
/// A group's members.
const GROUP_MEMBERS: &[&str] = &["num_to_select", "trades"];
/// A trade's members.
const TRADE_MEMBERS: &[&str] = &[
    "wants",
    "gives",
    "trader_exp",
    "max_uses",
    "reward_exp",
    "weight",
];
/// An item's members.
const ITEM_MEMBERS: &[&str] = &[
    "quantity",
    "price_multiplier",
    "item",
    "functions",
    "filters",
    "biomes",
];

/// A part of the table a message may drop while the rest loads.
enum Read<T> {
    /// The part.
    Kept(T),
    /// A message dropped it.
    Dropped,
}

/// The state of one read: where findings go and whether one rejects the document.
pub(super) struct Reader<'d> {
    d: &'d mut Diagnostics,
    failed: bool,
}

/// Reads a legacy document (the whole text) whose path is `at`.
pub(super) fn read(
    payload: JsonSlice<'_>,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<TradeTable> {
    let Ok(root) = bedrock_addon_core::json::from_str_jsonc::<Value>(payload.text) else {
        d.push(
            Diagnostic::error(DiagnosticCode::MalformedInput {
                message: "the document is not JSON".to_owned(),
                text: None,
            })
            .at(at.clone()),
        );
        return None;
    };
    let mut reader = Reader { d, failed: false };
    let table = reader.table(&root, at);
    table.filter(|_| !reader.failed)
}

impl Reader<'_> {
    /// Pushes a finding.
    fn push(&mut self, finding: Diagnostic) {
        if finding.severity == Severity::Error {
            self.failed = true;
        }
        self.d.push(finding);
    }

    /// An error at `at`: the table is not loaded.
    fn error(&mut self, code: DiagnosticCode, at: JsonPath) {
        self.push(Diagnostic::error(code).at(at));
    }

    /// An information at `at`: the game's legacy parser reads on in silence, so the crate
    /// reports an information.
    fn warn(&mut self, code: DiagnosticCode, at: JsonPath) {
        self.push(Diagnostic::info(code).at(at));
    }

    /// A kept error at `at`: the game logs the line at the content log's error level, drops the
    /// trade or group and goes on with the table. The loader's validity check does not look at
    /// the logged errors, so the table loads.
    fn kept_error(&mut self, code: DiagnosticCode, at: JsonPath) {
        self.d.push_kept_error(Diagnostic::error(code).at(at));
    }

    /// The trade or group at `at` goes whole, as in the game, for what a line at one of its
    /// members says: noted at the trade or group itself, since the member's line does not
    /// say that the other members go with it.
    fn dropped(&mut self, at: &JsonPath) {
        self.push(Diagnostic::info(DiagnosticCode::DroppedField { text: None }).at(at.clone()));
    }

    /// The members of `object` not in `known`, each with an `UnrecognizedField` information:
    /// the legacy parser reads only the members it knows and ignores the rest. They are kept
    /// for the document's own shape, in the document's table; the upgraded table does not have
    /// them.
    fn unknown(&mut self, object: &facet_value::VObject, known: &[&str], at: &JsonPath) {
        for (key, value) in object {
            if !known.contains(&key.as_str()) {
                self.warn(
                    DiagnosticCode::UnrecognizedField { text: None },
                    at.child(key.as_str()),
                );
                self.d.unknown_mut().put(at, key.as_str(), value.clone());
            }
        }
    }

    /// The root. A message of the table parser itself rejects the table (vanilla returns an
    /// empty one).
    fn table(&mut self, root: &Value, at: &JsonPath) -> Option<TradeTable> {
        let Some(object) = root.as_object() else {
            let code = if root.is_null() {
                DiagnosticCode::MissingPayload {
                    text: Some(VanillaText::Fixed(MUST_CONTAIN_TIERS)),
                }
            } else {
                DiagnosticCode::InvalidType { text: None }
            };
            self.error(code, at.clone());
            return None;
        };
        self.unknown(object, ROOT_MEMBERS, at);
        // `format_version` is read only when it is a string.
        let format_version = object
            .get("format_version")
            .and_then(Value::as_string)
            .map(|s| s.as_str().to_owned());
        if format_version.is_none() && present(object.get("format_version")).is_some() {
            // Read as absent, and said.
            self.warn(
                DiagnosticCode::InvalidType { text: None },
                at.child("format_version"),
            );
        }
        let tiers_at = at.child("tiers");
        // The tiers are missing or not an array.
        let Some(tiers) = object.get("tiers") else {
            self.error(
                DiagnosticCode::MissingPayload {
                    text: Some(VanillaText::Fixed(TIERS_ARRAY)),
                },
                at.clone(),
            );
            return None;
        };
        let Some(tiers) = tiers.as_array() else {
            self.error(
                DiagnosticCode::InvalidType {
                    text: Some(VanillaText::Fixed(TIERS_ARRAY)),
                },
                tiers_at,
            );
            return None;
        };
        let mut table = TradeTable {
            format_version,
            tiers: Vec::with_capacity(tiers.len()),
        };
        for (i, tier) in tiers.iter().enumerate() {
            let tier = self.tier(tier, &tiers_at.index(i))?;
            table.tiers.push(tier);
        }
        Some(table)
    }

    /// One tier: `groups`, or (only when `groups` is null) the tier itself read as one group
    /// with its flat `trades`.
    fn tier(&mut self, json: &Value, at: &JsonPath) -> Option<Tier> {
        let Some(object) = json.as_object() else {
            self.error(DiagnosticCode::InvalidType { text: None }, at.clone());
            return None;
        };
        self.unknown(object, TIER_MEMBERS, at);
        let total_exp_required = self.uint(
            object.get("total_exp_required"),
            &at.child("total_exp_required"),
        );
        let mut tier = Tier {
            total_exp_required,
            groups: None,
            trades: None,
        };
        match present(object.get("groups")) {
            Some(groups) => {
                let groups_at = at.child("groups");
                // The groups are non-null and not an array.
                let Some(groups) = groups.as_array() else {
                    self.error(
                        DiagnosticCode::InvalidType {
                            text: Some(VanillaText::Fixed(GROUPS_ARRAY)),
                        },
                        groups_at,
                    );
                    return None;
                };
                let mut kept = Vec::with_capacity(groups.len());
                let mut places = Vec::with_capacity(groups.len());
                for (i, group) in groups.iter().enumerate() {
                    if let Read::Kept(group) = self.group(group, &groups_at.index(i))? {
                        kept.push(group);
                        places.push(i);
                    }
                }
                // What is told of a group from here on is at the index the file has.
                self.d.left_out(&groups_at, &places);
                tier.groups = Some(kept);
                // `trades` is read only when `groups` is null; beside `groups` it has no
                // effect, which the crate reports as a warning.
                if object.contains_key("trades") {
                    self.warn(
                        DiagnosticCode::DroppedField { text: None },
                        at.child("trades"),
                    );
                }
            }
            // The flat form: every trade is offered (`num_to_select` -1).
            None => tier.trades = self.trades(object.get("trades"), &at.child("trades"), -1),
        }
        Some(tier)
    }

    /// One group: `Dropped` when a message drops it alone, `None` when the table is rejected.
    fn group(&mut self, json: &Value, at: &JsonPath) -> Option<Read<Group>> {
        let empty = facet_value::VObject::new();
        let object = json.as_object().unwrap_or(&empty);
        self.unknown(object, GROUP_MEMBERS, at);
        let select_at = at.child("num_to_select");
        let num_to_select = self.int(object.get("num_to_select"), &select_at);
        // A missing or unreadable count selects -1, every trade.
        let selects = num_to_select.unwrap_or(-1);
        let Some(trades) = self.trades(object.get("trades"), &at.child("trades"), selects) else {
            self.dropped(at);
            return Some(Read::Dropped);
        };
        // The count is compared with the trades that were kept; vanilla returns an empty
        // table.
        if let Some(finding) = rules::selects_too_many(selects, trades.len(), &select_at) {
            self.push(finding);
            return None;
        }
        Some(Read::Kept(Group {
            num_to_select,
            trades,
        }))
    }

    /// A `trades` list of a group that selects `selects` trades: `None` when it is missing or
    /// not an array, and the group is dropped. Vanilla logs an error and the rest of the table
    /// loads, so it is a kept error.
    fn trades(&mut self, json: Option<&Value>, at: &JsonPath, selects: i32) -> Option<Vec<Trade>> {
        let text = Some(VanillaText::Fixed(TRADES_ARRAY));
        let Some(json) = json else {
            self.kept_error(DiagnosticCode::MissingRequiredField { text }, at.clone());
            return None;
        };
        let Some(trades) = json.as_array() else {
            self.kept_error(DiagnosticCode::InvalidType { text }, at.clone());
            return None;
        };
        let mut kept = Vec::with_capacity(trades.len());
        let mut places = Vec::with_capacity(trades.len());
        for (i, trade) in trades.iter().enumerate() {
            if let Some(trade) = self.trade(trade, &at.index(i), selects) {
                kept.push(trade);
                places.push(i);
            }
        }
        self.d.left_out(at, &places);
        Some(kept)
    }

    /// One trade of a group that selects `selects` trades; `None` when a message drops it
    /// (vanilla logs an error that drops the trade; the rest loads, so it is a kept error).
    fn trade(&mut self, json: &Value, at: &JsonPath, selects: i32) -> Option<Trade> {
        let empty = facet_value::VObject::new();
        let object = json.as_object().unwrap_or(&empty);
        self.unknown(object, TRADE_MEMBERS, at);
        // `{}` is the trade as styled JSON text; a number no double holds is shown as the
        // largest finite one, as it is read.
        let styled = || {
            let mut shown = json.clone();
            bedrock_addon_core::json::clamp_non_finite(&mut shown, None);
            bedrock_addon_core::json::to_json(&shown).unwrap_or_default()
        };
        let wants = self.entries(object.get("wants"), &at.child("wants"));
        if wants.is_empty() {
            let text = Some(formatted(NO_OFFERS, &[&styled().trim()]));
            self.kept_error(DiagnosticCode::InvalidValue { text }, at.child("wants"));
            self.dropped(at);
            return None;
        }
        let gives = self.entries(object.get("gives"), &at.child("gives"));
        if gives.is_empty() {
            let text = Some(formatted(NO_RECEIVES, &[&styled().trim()]));
            self.kept_error(DiagnosticCode::InvalidValue { text }, at.child("gives"));
            self.dropped(at);
            return None;
        }
        let trader_exp = self.uint(object.get("trader_exp"), &at.child("trader_exp"));
        let max_uses = self.int(object.get("max_uses"), &at.child("max_uses"));
        // `true` when null, else the member read as a boolean that defaults to `false`.
        let reward_exp = self.boolean(object.get("reward_exp"), &at.child("reward_exp"));
        let weight_at = at.child("weight");
        let weight = self.int(object.get("weight"), &weight_at);
        if let Some(weight) = weight {
            if weight <= 0 {
                let text = Some(formatted(INVALID_WEIGHT, &[&weight]));
                // The game logs an error and drops the trade: a kept error.
                self.kept_error(DiagnosticCode::InvalidValue { text }, weight_at);
                self.dropped(at);
                return None;
            }
            // A `weight` above 0 in a group that selects -1 (every trade) has no effect. The
            // game logs the text at the content log's warning level: a warning.
            if let Some(finding) =
                rules::weight_without_effect(selects, &weight_at, Severity::Warning)
            {
                self.push(finding);
            }
        }
        Some(Trade {
            wants,
            gives,
            trader_exp,
            max_uses,
            reward_exp,
            weight,
        })
    }

    /// A `wants` / `gives` list: each entry `{"choice": [...]}` when `choice` is a non-null
    /// array, otherwise one item.
    fn entries(&mut self, json: Option<&Value>, at: &JsonPath) -> Vec<ItemList> {
        let Some(entries) = json.and_then(Value::as_array) else {
            return Vec::new();
        };
        let mut kept = Vec::with_capacity(entries.len());
        let mut places = Vec::with_capacity(entries.len());
        let mut empty: Vec<JsonPath> = Vec::new();
        for (i, entry) in entries.iter().enumerate() {
            let entry_at = at.index(i);
            let choice = entry
                .as_object()
                .and_then(|object| object.get("choice"))
                .and_then(Value::as_array);
            match choice {
                Some(choice) => {
                    let choice_at = entry_at.child("choice");
                    let mut items = Vec::with_capacity(choice.len());
                    let mut item_places = Vec::with_capacity(choice.len());
                    for (c, item) in choice.iter().enumerate() {
                        if let Some(item) = self.item(item, &choice_at.index(c)) {
                            items.push(item);
                            item_places.push(c);
                        }
                    }
                    self.d.left_out(&choice_at, &item_places);
                    // Only `choice` is read of such an entry.
                    if let Some(object) = entry.as_object() {
                        self.unknown(object, &["choice"], &entry_at);
                    }
                    // An entry whose `choice` is empty is dropped. The crate tells the empty
                    // list where other entries stay (an element that is no item has said so
                    // itself).
                    if choice.is_empty() {
                        empty.push(choice_at.clone());
                    }
                    if !items.is_empty() {
                        kept.push(ItemList::Choice { choice: items });
                        places.push(i);
                    }
                }
                None => {
                    if let Some(item) = self.item(entry, &entry_at) {
                        kept.push(ItemList::Item(item));
                        places.push(i);
                    }
                }
            }
        }
        // A list left empty says so itself, at the list.
        if !kept.is_empty() {
            for at in empty {
                self.warn(DiagnosticCode::InvalidValue { text: None }, at);
            }
        }
        // Everything told of the entries at the file's indices is told.
        self.d.left_out(at, &places);
        kept
    }

    /// One item of a `wants` or `gives` list.
    fn item(&mut self, json: &Value, at: &JsonPath) -> Option<TradeItem> {
        let Some(object) = json.as_object() else {
            self.warn(DiagnosticCode::InvalidType { text: None }, at.clone());
            return None;
        };
        self.unknown(object, ITEM_MEMBERS, at);
        // The object form of a quantity is read for `min` and `max` only; the quantity is kept
        // as written, its other members with it.
        if let Some(range) = object.get("quantity").and_then(Value::as_object) {
            self.unknown(range, &["min", "max"], &at.child("quantity"));
        }
        let price_multiplier = self.float(
            object.get("price_multiplier"),
            &at.child("price_multiplier"),
        );
        Some(TradeItem {
            quantity: self.kept(object.get("quantity"), &at.child("quantity")),
            price_multiplier,
            item: self.string(object.get("item"), &at.child("item")),
            functions: self.kept(object.get("functions"), &at.child("functions")),
            filters: self.kept(object.get("filters"), &at.child("filters")),
            biomes: self.kept(object.get("biomes"), &at.child("biomes")),
        })
    }

    /// A member kept as written (`quantity`, `functions`, `filters`, `biomes`): absent when
    /// `null`. A number of it that no double holds is the largest finite double of its sign,
    /// with an `InvalidValue` warning at the number, so the loaded table holds no infinity that
    /// no writer spells.
    fn kept(&mut self, json: Option<&Value>, at: &JsonPath) -> Option<Value> {
        let mut value = present(json)?.clone();
        bedrock_addon_core::json::clamp_non_finite_warned(&mut value, None, at, self.d);
        Some(value)
    }

    // ---- scalar conversions (`Json::Value::as*`) -----------------------------------------
    //
    // Vanilla converts in silence; here every conversion that reads another value than the
    // file writes is a warning at the member: `InvalidType` for a value of another
    // JSON type, `InvalidValue` for a number the member does not hold.

    /// A float: absent or `null` reads as absent; a
    /// value that is no number reads as absent too (vanilla: 0.0) with an `InvalidType`
    /// warning; a number beyond a float is the largest finite float of its sign, with the
    /// warning of such a number.
    fn float(&mut self, json: Option<&Value>, at: &JsonPath) -> Option<f32> {
        let json = present(json)?;
        let Some(number) = json.as_number() else {
            self.warn(DiagnosticCode::InvalidType { text: None }, at.clone());
            return None;
        };
        let nearest = narrow(number.to_f64_lossy());
        if bedrock_addon_core::json::lossy::narrows(number.to_f64_lossy()) {
            // Held as a float, as in the game, and written so: noted.
            self.push(Diagnostic::info(DiagnosticCode::LossyConversion).at(at.clone()));
        }
        if nearest.is_infinite() {
            self.push(bedrock_addon_core::molang::beyond_f32_warning(
                at.clone(),
                None,
            ));
            return Some(f32::MAX.copysign(nearest));
        }
        Some(nearest)
    }

    /// An integer: an integer token, a real truncated toward zero (a fraction with an
    /// `InvalidValue` warning), a boolean as 0 / 1 (with an `InvalidType` warning); absent or
    /// `null` reads as absent.
    fn integer(&mut self, json: Option<&Value>, at: &JsonPath) -> Option<i64> {
        let json = present(json)?;
        if let Some(b) = json.as_bool() {
            self.warn(DiagnosticCode::InvalidType { text: None }, at.clone());
            return Some(i64::from(b));
        }
        let Some(number) = json.as_number() else {
            self.warn(DiagnosticCode::InvalidType { text: None }, at.clone());
            return None;
        };
        if !number.is_float()
            && let Some(n) = number.to_i64()
        {
            return Some(n);
        }
        // A real is truncated toward zero; one beyond `i64` saturates and then fits no
        // integer member, which says so.
        let x = number.to_f64_lossy();
        #[allow(clippy::cast_possible_truncation)]
        let n = x.trunc() as i64;
        if x.fract() != 0.0 {
            self.warn(DiagnosticCode::InvalidValue { text: None }, at.clone());
        }
        Some(n)
    }

    /// An `i32` (`asInt`).
    fn int(&mut self, json: Option<&Value>, at: &JsonPath) -> Option<i32> {
        let n = self.integer(json, at)?;
        let fits = i32::try_from(n).ok();
        if fits.is_none() {
            self.warn(DiagnosticCode::InvalidValue { text: None }, at.clone());
        }
        fits
    }

    /// A `u32` (`asUInt`).
    fn uint(&mut self, json: Option<&Value>, at: &JsonPath) -> Option<u32> {
        let n = self.integer(json, at)?;
        let fits = u32::try_from(n).ok();
        if fits.is_none() {
            self.warn(DiagnosticCode::InvalidValue { text: None }, at.clone());
        }
        fits
    }

    /// A boolean (`asBool`): a number is `true` when not zero, with an `InvalidType` warning.
    fn boolean(&mut self, json: Option<&Value>, at: &JsonPath) -> Option<bool> {
        let json = present(json)?;
        if let Some(flag) = json.as_bool() {
            return Some(flag);
        }
        self.warn(DiagnosticCode::InvalidType { text: None }, at.clone());
        json.as_number().map(|n| n.to_f64_lossy() != 0.0)
    }

    /// A string (`asString("")`): a number or a boolean reads as its text, with an
    /// `InvalidType` warning.
    fn string(&mut self, json: Option<&Value>, at: &JsonPath) -> Option<String> {
        let json = present(json)?;
        if let Some(text) = json.as_string() {
            return Some(text.as_str().to_owned());
        }
        // Not a string: said, whatever it reads as.
        self.warn(DiagnosticCode::InvalidType { text: None }, at.clone());
        if let Some(b) = json.as_bool() {
            return Some(b.to_string());
        }
        if json.as_number().is_some() {
            // A number no double holds has no text: the member reads as absent.
            return bedrock_addon_core::json::to_json(json)
                .ok()
                .map(|t| t.trim().to_owned());
        }
        None
    }
}

/// `json` unless it is absent or `null`.
fn present(json: Option<&Value>) -> Option<&Value> {
    json.filter(|v| !v.is_null())
}

/// The nearest `f32`.
#[allow(clippy::cast_possible_truncation)]
fn narrow(x: f64) -> f32 {
    x as f32
}
