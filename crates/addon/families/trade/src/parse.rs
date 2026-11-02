//! Parsing a trade table from format 1.21.30 on: vanilla's strict cereal slice
//! (`v1_21_30`). The members that take two JSON forms (a
//! `wants` / `gives` entry, a quantity) are held as raw slices and parsed on their own.

// The slice is strict: `UnrecognizedField` is an error.

use facet::Facet;
use facet_json::RawJson;

use super::{Group, ItemList, Quantity, Tier, Trade, TradeItem, TradeTable};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, JsonPathSegment, Location, Phase, Severity,
    code_for_deserialize_error_kind,
};
use bedrock_addon_core::id::ItemId;
use bedrock_addon_core::json::{
    JsonSlice, NullRule, SliceError, StrictBool, StrictFloat, StrictInt, Value, parse_at,
    reject_nulls,
};
use bedrock_addon_core::shared::Filter;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::ParseCx;

use super::texts::{child, element};
use super::validate as rules;

/// The cereal slice's version (`v1_21_30::TradeTableData::VERSION`).
pub(crate) const CEREAL: FormatVersion = FormatVersion::new(1, 21, 30);

/// A tier as the text has it.
#[derive(Facet)]
struct TierWire<'a> {
    // Required.
    groups: Vec<GroupWire<'a>>,
    // Required.
    total_exp_required: StrictInt<u32>,
}

/// A group as the text has it.
#[derive(Facet)]
struct GroupWire<'a> {
    // Optional, default 0.
    #[facet(default)]
    num_to_select: Option<StrictInt<i32>>,
    // Required.
    trades: Vec<TradeWire<'a>>,
}

/// A trade as the text has it.
#[derive(Facet)]
struct TradeWire<'a> {
    // Required.
    wants: Vec<RawJson<'a>>,
    gives: Vec<RawJson<'a>>,
    // Optional.
    #[facet(default)]
    trader_exp: Option<StrictInt<u32>>,
    #[facet(default)]
    max_uses: Option<StrictInt<i32>>,
    #[facet(default)]
    reward_exp: Option<StrictBool>,
    #[facet(default)]
    weight: Option<StrictInt<i32>>,
}

/// The object form of an entry.
#[derive(Facet)]
struct ChoiceWire<'a> {
    // Required.
    choice: Vec<RawJson<'a>>,
}

/// An item as the text has it.
#[derive(Facet)]
struct TradeItemWire<'a> {
    #[facet(default)]
    quantity: Option<RawJson<'a>>,
    // Required; takes a string only.
    item: ItemId,
    #[facet(default)]
    functions: Option<Vec<Value>>,
    #[facet(default)]
    filters: Option<Filter>,
    #[facet(default)]
    price_multiplier: Option<StrictFloat<f32>>,
    // Read, then reported as deprecated by name.
    #[facet(default, rename = "__DEP_fnctns")]
    legacy_functions: Option<Value>,
    #[facet(default, rename = "__DEP_fltrs")]
    legacy_filters: Option<Value>,
}

/// The object form of a quantity.
#[derive(Facet)]
struct RangeWire<'a> {
    #[facet(default)]
    min: Option<RawJson<'a>>,
    #[facet(default)]
    max: Option<RawJson<'a>>,
}

/// The state of one parse: the findings and whether one rejects the document.
struct Run<'d> {
    d: &'d mut Diagnostics,
    failed: bool,
    /// The slice's null rule.
    rule: Option<NullRule>,
    /// The document was written at 1.21.30 or later, read by the cereal reader itself (not a
    /// legacy table the steps brought here).
    cereal: bool,
}

impl Run<'_> {
    /// An error at `at`: a cereal constraint the value breaks.
    fn error(&mut self, code: DiagnosticCode, at: JsonPath) {
        self.d.push(Diagnostic::new(
            code,
            Severity::Error,
            Phase::Parse,
            Location::new(None, None, at, None),
        ));
        self.failed = true;
    }

    /// Parses `T` from `slice`, whose path is `at`; a failure is one error at the failing member.
    fn typed<'a, T: Facet<'a>>(&mut self, slice: JsonSlice<'a>, at: &JsonPath) -> Option<T> {
        match parse_at::<T>(slice) {
            // Rule N2: a `null` where the wire struct reads a value is an `InvalidType`
            // error at it, and the table is rejected.
            Ok(value)
                if self.rule == Some(NullRule::Rejected)
                    && reject_nulls(&value, slice, at, self.d) =>
            {
                self.failed = true;
                None
            }
            Ok(value) => Some(value),
            Err(error) => {
                self.d.push(slice_error(&error, at, slice));
                self.failed = true;
                None
            }
        }
    }
}

/// Parses the `tiers` payload of a document from 1.21.30 on.
pub(crate) fn parse_table(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<TradeTable> {
    let mut run = Run {
        d,
        failed: false,
        rule: cx.load.null_rule,
        cereal: cx.original_version >= CEREAL,
    };
    // Vanilla's validation reports every unknown member, also beside another failure.
    if let Ok(json) = bedrock_addon_core::json::from_str_jsonc::<Value>(payload.text) {
        let mut unknown = Vec::new();
        unknown_tiers(&json, &cx.at, &mut unknown);
        for at in unknown {
            run.d.push(Diagnostic::new(
                DiagnosticCode::UnrecognizedField { text: None },
                Severity::Error,
                Phase::Parse,
                Location::new(None, None, at, None),
            ));
            run.failed = true;
        }
    }
    let tiers: Vec<TierWire<'_>> = run.typed(payload, &cx.at)?;
    let mut table = TradeTable::default();
    for (t, tier) in tiers.into_iter().enumerate() {
        let tier_at = element(&cx.at, t);
        let mut groups = Vec::with_capacity(tier.groups.len());
        for (g, group) in tier.groups.into_iter().enumerate() {
            let group_at = element(&child(&tier_at, "groups"), g);
            let list_at = child(&group_at, "trades");
            let num_to_select = group.num_to_select.map(|n| n.0);
            // The schema default is 0.
            let selects = num_to_select.unwrap_or(0);
            // The sequence must hold at least one element.
            if group.trades.is_empty() {
                run.error(DiagnosticCode::InvalidValue { text: None }, list_at.clone());
            }
            if let Some(finding) = rules::selects_too_many(
                selects,
                group.trades.len(),
                &child(&group_at, "num_to_select"),
            ) {
                run.d.push(finding);
                run.failed = true;
            }
            let mut trades = Vec::with_capacity(group.trades.len());
            for (i, trade) in group.trades.into_iter().enumerate() {
                let trade_at = element(&list_at, i);
                trades.push(parse_trade(&mut run, payload, &trade, &trade_at, selects));
            }
            groups.push(Group {
                num_to_select,
                trades,
            });
        }
        table.tiers.push(Tier {
            groups,
            total_exp_required: tier.total_exp_required.0,
        });
    }
    (!run.failed).then_some(table)
}

// ---- unknown members (the strict mask: `UnrecognizedField` is an error) -----------------

/// The members of a tier.
const TIER_MEMBERS: &[&str] = &["groups", "total_exp_required"];
/// The members of a group.
const GROUP_MEMBERS: &[&str] = &["num_to_select", "trades"];
/// The members of a trade.
const TRADE_MEMBERS: &[&str] = &[
    "wants",
    "gives",
    "trader_exp",
    "max_uses",
    "reward_exp",
    "weight",
];
/// The members of a `{"choice"}` entry.
const CHOICE_MEMBERS: &[&str] = &["choice"];
/// The members of an item.
const ITEM_MEMBERS: &[&str] = &[
    "quantity",
    "item",
    "functions",
    "filters",
    "price_multiplier",
    "__DEP_fnctns",
    "__DEP_fltrs",
];
/// The members of a quantity's object form.
const RANGE_MEMBERS: &[&str] = &["min", "max"];

/// The elements of `json` when it is an array, with their paths below `at`.
fn elements(json: Option<&Value>, at: JsonPath) -> impl Iterator<Item = (&Value, JsonPath)> {
    json.and_then(Value::as_array)
        .into_iter()
        .flat_map(|array| array.iter().enumerate())
        .map(move |(i, value)| (value, element(&at, i)))
}

/// The members of the object `json` not in `known`, pushed to `out`; the members that are.
fn members<'v>(
    json: &'v Value,
    at: &JsonPath,
    known: &[&str],
    out: &mut Vec<JsonPath>,
) -> Option<&'v facet_value::VObject> {
    let object = json.as_object()?;
    for (key, _) in object {
        if !known.contains(&key.as_str()) {
            out.push(child(at, key.as_str()));
        }
    }
    Some(object)
}

/// The unknown members of the `tiers` array `json`, at any depth, in document order. A member
/// of the wrong JSON type is the typed parse's to reject and is not walked; a filter checks
/// its own members (`Filter::check`).
fn unknown_tiers(json: &Value, at: &JsonPath, out: &mut Vec<JsonPath>) {
    for (tier, tier_at) in elements(Some(json), at.clone()) {
        let Some(tier) = members(tier, &tier_at, TIER_MEMBERS, out) else {
            continue;
        };
        for (group, group_at) in elements(tier.get("groups"), child(&tier_at, "groups")) {
            let Some(group) = members(group, &group_at, GROUP_MEMBERS, out) else {
                continue;
            };
            for (trade, trade_at) in elements(group.get("trades"), child(&group_at, "trades")) {
                let Some(trade) = members(trade, &trade_at, TRADE_MEMBERS, out) else {
                    continue;
                };
                for key in ["wants", "gives"] {
                    for (entry, entry_at) in elements(trade.get(key), child(&trade_at, key)) {
                        unknown_entry(entry, &entry_at, out);
                    }
                }
            }
        }
    }
}

/// The unknown members of a `wants` / `gives` entry.
fn unknown_entry(json: &Value, at: &JsonPath, out: &mut Vec<JsonPath>) {
    let Some(object) = json.as_object() else {
        return;
    };
    if object.contains_key("choice") {
        members(json, at, CHOICE_MEMBERS, out);
        for (item, item_at) in elements(object.get("choice"), child(at, "choice")) {
            unknown_item(item, &item_at, out);
        }
    } else {
        unknown_item(json, at, out);
    }
}

/// The unknown members of an item.
fn unknown_item(json: &Value, at: &JsonPath, out: &mut Vec<JsonPath>) {
    let Some(item) = members(json, at, ITEM_MEMBERS, out) else {
        return;
    };
    if let Some(quantity) = item.get("quantity") {
        members(quantity, &child(at, "quantity"), RANGE_MEMBERS, out);
    }
}

/// One trade, its entries parsed from their slices.
fn parse_trade<'a>(
    run: &mut Run<'_>,
    payload: JsonSlice<'a>,
    trade: &TradeWire<'a>,
    at: &JsonPath,
    selects: i32,
) -> Trade {
    // `wants` and `gives` must hold at least one element.
    for (entries, key) in [(&trade.wants, "wants"), (&trade.gives, "gives")] {
        if entries.is_empty() {
            run.error(DiagnosticCode::InvalidValue { text: None }, child(at, key));
        }
    }
    if let Some(weight) = trade.weight {
        let weight_at = child(at, "weight");
        // A number must be at least 1.
        if weight.0 < 1 {
            run.error(DiagnosticCode::InvalidValue { text: None }, weight_at);
        } else if let Some(finding) =
            // The cereal slice has no such check, so the crate reports an information.
            rules::weight_without_effect(selects, &weight_at, Severity::Info)
        {
            run.d.push(finding);
        }
    }
    let mut lists = |entries: &[RawJson<'a>], key: &str| -> Vec<ItemList> {
        let list_at = child(at, key);
        entries
            .iter()
            .enumerate()
            .filter_map(|(e, raw)| {
                let slice = payload.nested(raw)?;
                parse_list(run, slice, &element(&list_at, e))
            })
            .collect()
    };
    let wants = lists(&trade.wants, "wants");
    let gives = lists(&trade.gives, "gives");
    Trade {
        wants,
        gives,
        trader_exp: trade.trader_exp.map(|n| n.0),
        max_uses: trade.max_uses.map(|n| n.0),
        reward_exp: trade.reward_exp.map(|b| b.0),
        weight: trade.weight.map(|n| n.0),
    }
}

/// One `wants` / `gives` entry: `{"choice": [...]}` when the object has `choice`, else one item
/// (a bare item reads as a list of one).
fn parse_list(run: &mut Run<'_>, slice: JsonSlice<'_>, at: &JsonPath) -> Option<ItemList> {
    if has_member(slice, "choice") {
        let wire: ChoiceWire<'_> = run.typed(slice, at)?;
        let choice_at = child(at, "choice");
        let choice = wire
            .choice
            .iter()
            .enumerate()
            .filter_map(|(c, raw)| {
                let item = slice.nested(raw)?;
                parse_item(run, item, &element(&choice_at, c))
            })
            .collect();
        Some(ItemList::Choice { choice })
    } else {
        parse_item(run, slice, at).map(ItemList::Item)
    }
}

/// One trade item.
fn parse_item(run: &mut Run<'_>, slice: JsonSlice<'_>, at: &JsonPath) -> Option<TradeItem> {
    let wire: TradeItemWire<'_> = run.typed(slice, at)?;
    let quantity = match &wire.quantity {
        Some(raw) => {
            let quantity = slice.nested(raw)?;
            Some(parse_quantity(run, quantity, &child(at, "quantity"))?)
        }
        None => None,
    };
    // The hidden names are read and deprecated by name: written in a 1.21.30+ document they are
    // an error under both masks that rejects the table. Moved here by the legacy steps they
    // carry the legacy evaluation and are announced with the same text as a warning.
    for (written, member) in [
        (wire.legacy_functions.is_some(), "__DEP_fnctns"),
        (wire.legacy_filters.is_some(), "__DEP_fltrs"),
    ] {
        if written {
            let mut finding = Diagnostic::deprecated_member(member, child(at, member));
            if run.cereal {
                run.failed = true;
            } else {
                // A table upgraded from the legacy form holds the hidden names by the upgrade's
                // move, which logs nothing, so the crate reports an information.
                finding.severity = Severity::Info;
            }
            run.d.push(finding);
        }
    }
    Some(TradeItem {
        quantity,
        item: wire.item,
        functions: wire.functions,
        filters: wire.filters,
        price_multiplier: wire.price_multiplier.map(|x| x.0),
        legacy_functions: wire.legacy_functions,
        legacy_filters: wire.legacy_filters,
    })
}

/// A quantity: an integer, or `{"min", "max"}`.
fn parse_quantity(run: &mut Run<'_>, slice: JsonSlice<'_>, at: &JsonPath) -> Option<Quantity> {
    if !slice.text.trim_start().starts_with('{') {
        return quantity_int(run, slice, at).map(Quantity::Exact);
    }
    let wire: RangeWire<'_> = run.typed(slice, at)?;
    let mut bound = |raw: &Option<RawJson<'_>>, key: &str| match raw {
        None => Some(None),
        Some(raw) => quantity_int(run, slice.nested(raw)?, &child(at, key)).map(Some),
    };
    let min = bound(&wire.min, "min");
    let max = bound(&wire.max, "max");
    Some(Quantity::Range {
        min: min?,
        max: max?,
    })
}

/// One integer of a quantity (the integer form, or a bound of the object form): a `u32` with
/// minimum of 1 in each place. An integer below 1 breaks the constraint
/// (`InvalidValue`; 0 is kept, a negative one has no value to keep); any other value is of the
/// wrong type (`InvalidType`).
fn quantity_int(run: &mut Run<'_>, slice: JsonSlice<'_>, at: &JsonPath) -> Option<u32> {
    match parse_at::<StrictInt<u32>>(slice) {
        Ok(StrictInt(0)) => {
            run.error(DiagnosticCode::InvalidValue { text: None }, at.clone());
            Some(0)
        }
        Ok(StrictInt(n)) => Some(n),
        Err(_) => {
            let negative = parse_at::<StrictInt<i64>>(slice).is_ok_and(|n| n.0 < 0);
            let code = if negative {
                DiagnosticCode::InvalidValue { text: None }
            } else {
                DiagnosticCode::InvalidType { text: None }
            };
            run.error(code, at.clone());
            None
        }
    }
}

/// Whether the object `slice` has the member `key`.
fn has_member(slice: JsonSlice<'_>, key: &str) -> bool {
    bedrock_addon_core::json::from_str_borrowed_jsonc::<indexmap::IndexMap<String, RawJson<'_>>>(
        slice.text,
    )
    .is_ok_and(|members| members.contains_key(key))
}

/// The scalar members of the typed structs, each with the path that leads to it from its
/// slice's root (`None` stands for an array index): the `tiers` payload (`Vec<TierWire>`) and
/// what it nests, and `TradeItemWire`. A quantity's bounds are read from their own slices.
const SCALAR_MEMBERS: &[(&[Option<&str>], &[&str])] = &[
    (&[], &["item", "price_multiplier"]),
    (&[None], &["total_exp_required"]),
    (&[None, Some("groups"), None], &["num_to_select"]),
    (
        &[None, Some("groups"), None, Some("trades"), None],
        &["trader_exp", "max_uses", "reward_exp", "weight"],
    ),
];

/// The index in `segments` (the path of a failed typed parse, relative to its slice) of the
/// scalar member the failure belongs to, if it is below one. A `None` of a prefix is an array
/// index, a `Some` the member of that key.
pub(super) fn scalar_member(segments: &[JsonPathSegment<'_>]) -> Option<usize> {
    SCALAR_MEMBERS.iter().find_map(|(prefix, members)| {
        let JsonPathSegment::Key(member) = segments.get(prefix.len())? else {
            return None;
        };
        let leads =
            prefix
                .iter()
                .zip(segments)
                .all(|(expected, segment)| match (expected, segment) {
                    (None, segment) => matches!(segment, JsonPathSegment::Index(_)),
                    (Some(key), JsonPathSegment::Key(segment)) => key == segment,
                    (Some(_), JsonPathSegment::Index(_)) => false,
                });
        (leads && members.contains(member)).then_some(prefix.len())
    })
}

/// The finding of a failed typed parse of `slice`, at `base` followed by the failing member.
fn slice_error(error: &SliceError, base: &JsonPath, slice: JsonSlice<'_>) -> Diagnostic {
    let blank = slice.text.trim().is_empty();
    let code = code_for_deserialize_error_kind(&error.error.kind, !error.path.is_empty(), blank);
    // A slice is JSON the envelope probe has read, so a failure at its root is never a syntax
    // error: the value has the wrong JSON type for its place (the `tiers` payload, a `wants` /
    // `gives` entry, a `choice` element).
    let code = match code {
        DiagnosticCode::MalformedInput { .. } if error.path.is_empty() => {
            DiagnosticCode::InvalidType { text: None }
        }
        other => other,
    };
    let mut at = base.clone();
    at.append(&error.path);
    // An array or object given to a scalar member is reported at its first element; the
    // finding is the member's.
    let segments: Vec<JsonPathSegment<'_>> = error.path.segments().collect();
    if let Some(member) = scalar_member(&segments) {
        for _ in member + 1..segments.len() {
            at.pop();
        }
    }
    Diagnostic::new(
        code,
        Severity::Error,
        Phase::Parse,
        Location::new(None, None, at, Some(error.span)),
    )
}
