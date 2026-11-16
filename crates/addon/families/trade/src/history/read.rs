//! The legacy parser: vanilla's legacy reader of the table, its groups, trades and items, as
//! the derived read of the [`v0`](super::v0) types (`#[derive(Facet, Schema)]`): members in
//! declaration order, the unknown ones first (an information each, kept in the document's
//! table), a `null` member absent. Scalars are read the way `Json::Value::as*` converts them
//! (the `read_with` functions below); a value no conversion takes is an information and reads
//! as absent. The rules of each level (a group or trade the game drops, a message that rejects
//! the table) are the functions that read the level's list.

use super::v0::{Group, ItemList, Tier, Trade, TradeItem, TradeTable};
use crate::codes;
use crate::validate as rules;
use bedrock_addon_core::diagnostic::builders;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::kit::{Node, Profile, Read};
use bedrock_addon_core::json::parse::{Options, parse_slice};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::schema::{ElementFailure, FromJson, UnknownRule, read_document, read_list};
use bedrock_addon_core::versioning::{MemberFailure, NullHandling, Reading};

/// How the legacy parser reads: a `null` member is absent, and a member it cannot read is
/// absent too (told by the reads below).
const READING: Reading = Reading::OWN
    .with_null(NullHandling::AbsentSilently)
    .with_on_member_failure(MemberFailure::DefaultAndTell);

/// Reads a legacy document (the whole text) whose path is `at`.
pub(super) fn read(
    payload: JsonSlice<'_>,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<TradeTable> {
    let Ok(parsed) = parse_slice(payload, Options::default()) else {
        d.push(Diagnostic::of(builders::syntax("the document is not JSON")).at(at.clone()));
        return None;
    };
    let root = Node::parsed(&parsed, at.clone());
    // The game's parser reads only the members it knows and ignores the rest. They are kept
    // for the document's own shape; the upgraded table does not have them.
    let mut read = Read::new(READING, d)
        .profile(Profile::DECLARED_ORDER)
        .keeps_unknown(true)
        .unknown_severity(Severity::Info);
    // A message of the table parser itself rejects the table (vanilla returns an empty one).
    if !root.value().is_object() {
        let code = if root.value().is_null() {
            Code::from(&codes::MUST_CONTAIN_TIERS)
        } else {
            Code::from(&core_codes::INVALID_TYPE)
        };
        read.error(&root, code);
        return None;
    }
    let table = read_document::<TradeTable>(&root, &mut read);
    table.filter(|_| !read.failed())
}

/// Tells `code` at `node` as an error that rejects the table: vanilla returns an empty table
/// there, so nothing after it is read.
fn reject(read: &mut Read<'_>, node: &Node<'_>, code: Code) {
    read.error(node, code);
    read.stop();
}

/// An information of the crate's own at `node`: the game's legacy parser reads on in silence.
fn note(read: &mut Read<'_>, node: &Node<'_>, code: Code) {
    let mut finding = Diagnostic::info(code).at(node.path().clone());
    finding.location.span = node.span();
    read.push(finding);
}

/// A kept error at `at`: the game logs the line at the content log's error level, drops the
/// trade or group and goes on with the table. The loader's validity check does not look at
/// the logged errors, so the table loads.
fn kept_error(read: &mut Read<'_>, code: Code, at: JsonPath) {
    read.findings_mut()
        .push_kept_error(Diagnostic::error(code).at(at));
}

/// The trade or group at `node` goes whole, as in the game, for what a line at one of its
/// members says: noted at the trade or group itself, since the member's line does not say
/// that the other members go with it.
fn dropped(read: &mut Read<'_>, node: &Node<'_>) {
    let mut finding = Diagnostic::of(&core_codes::DROPPED_FIELD).at(node.path().clone());
    finding.location.span = node.span();
    read.push(finding);
}

/// An object to read the members of: `node` itself, or `{}` (at `node`'s path) where it is no
/// object, as the game reads a value that is none.
fn object_or_empty<'v>(node: &Node<'v>, empty: &'v Value) -> Node<'v> {
    if node.value().is_object() {
        node.clone()
    } else {
        Node::new(empty, node.path().clone())
    }
}

/// The member `key` of `node`, as written (`null` included); `None` when `node` is no object.
fn member<'v>(node: &Node<'v>, read: &mut Read<'_>, key: &str) -> Option<Node<'v>> {
    if !node.value().is_object() {
        return None;
    }
    read.object(node)?.get(key)
}

/// The root's rules (`schema::after_read` of [`TradeTable`]): `tiers` missing, or `null`
/// (read as absent, so the read did not see it), rejects the table.
pub(super) fn table_rules(_: &mut TradeTable, node: &Node<'_>, read: &mut Read<'_>) {
    if read.stopped() {
        return;
    }
    match member(node, read, "tiers") {
        None => reject(read, node, Code::from(&codes::TIERS_MISSING)),
        Some(tiers) if tiers.value().is_null() => {
            reject(read, &tiers, Code::from(&codes::TIERS_ARRAY));
        }
        Some(_) => {}
    }
}

/// `tiers`: a tier that is no object rejects the table.
pub(super) fn tiers(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Tier>> {
    if !node.value().is_array() {
        reject(read, node, Code::from(&codes::TIERS_ARRAY));
        return None;
    }
    read_list(node, read, ElementFailure::ByReading, |tier, read| {
        if read.stopped() {
            return None;
        }
        if !tier.value().is_object() {
            reject(read, tier, Code::from(&core_codes::INVALID_TYPE));
            return None;
        }
        Tier::from_json(tier, read)
    })
}

/// A tier's rule (`schema::after_read` of [`Tier`]): `trades` is read only when `groups` is
/// absent or `null`, as one group that offers every trade (`num_to_select` -1). Beside
/// `groups` it has no effect, which the crate notes.
pub(super) fn tier_rules(tier: &mut Tier, node: &Node<'_>, read: &mut Read<'_>) {
    if read.stopped() {
        return;
    }
    if tier.groups.is_none() {
        tier.trades = trades(node, read, -1);
        return;
    }
    // The member as written is not read (`schema::read_later` left its default).
    tier.trades = None;
    if let Some(trades) = member(node, read, "trades") {
        note(read, &trades, Code::from(&core_codes::DROPPED_FIELD));
    }
}

/// `groups`, non-null: no array rejects the table; a group a message drops is left out alone.
pub(super) fn groups(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Group>> {
    if !node.value().is_array() {
        reject(read, node, Code::from(&codes::GROUPS_ARRAY));
        return None;
    }
    read_list(node, read, ElementFailure::Drop, group)
}

/// One group: `None` when a message drops it (its `trades`) or rejects the table (a count
/// above the trades that were kept; vanilla returns an empty table).
fn group(node: &Node<'_>, read: &mut Read<'_>) -> Option<Group> {
    if read.stopped() {
        return None;
    }
    let empty = Value::from(facet_value::VObject::new());
    let node = object_or_empty(node, &empty);
    let mut group = Group::from_json(&node, read)?;
    // A missing or unreadable count selects -1, every trade.
    let selects = group.num_to_select.unwrap_or(-1);
    let Some(trades) = trades(&node, read, selects) else {
        dropped(read, &node);
        return None;
    };
    let select_at = node.path().child("num_to_select");
    if let Some(finding) = rules::selects_too_many(selects, trades.len(), &select_at) {
        read.push(finding);
        read.stop();
        return None;
    }
    group.trades = trades;
    Some(group)
}

/// The `trades` of the object `owner` (a group, or a flat tier) that selects `selects` trades:
/// `None` when it is missing or not an array, and the group is dropped. Vanilla logs an error
/// and the rest of the table loads, so it is a kept error.
fn trades(owner: &Node<'_>, read: &mut Read<'_>, selects: i32) -> Option<Vec<Trade>> {
    let Some(list) = member(owner, read, "trades") else {
        kept_error(
            read,
            Code::from(&codes::TRADES_MISSING),
            owner.path().child("trades"),
        );
        return None;
    };
    if !list.value().is_array() {
        kept_error(read, Code::from(&codes::TRADES_ARRAY), list.path().clone());
        return None;
    }
    read_list(&list, read, ElementFailure::Drop, |node, read| {
        trade(node, read, selects)
    })
}

/// A trade's members, in the order the game reads them ([`Trade`]'s declaration order).
const TRADE_MEMBERS: [&str; 6] = [
    "wants",
    "gives",
    "trader_exp",
    "max_uses",
    "reward_exp",
    "weight",
];

/// One trade of a group that selects `selects` trades; `None` when a message drops it
/// (vanilla logs an error that drops the trade; the rest loads, so it is a kept error).
fn trade(node: &Node<'_>, read: &mut Read<'_>, selects: i32) -> Option<Trade> {
    if read.stopped() {
        return None;
    }
    let written = node.value();
    let empty = Value::from(facet_value::VObject::new());
    let node = object_or_empty(node, &empty);
    let at = node.path();
    let mut attempt = read.fork();
    let trade = Trade::from_json(&node, &mut attempt);
    // The game reads `wants`, then `gives`, and returns at the first that holds no entry: what
    // the read told of the members after it is withdrawn.
    let ends_after = trade.as_ref().and_then(|trade| {
        (trade.wants.is_empty())
            .then_some(1)
            .or_else(|| trade.gives.is_empty().then_some(2))
    });
    if let Some(after) = ends_after {
        let unread: Vec<JsonPath> = TRADE_MEMBERS[after..]
            .iter()
            .map(|key| at.child(key))
            .collect();
        attempt.findings_mut().retain(|finding| {
            !unread
                .iter()
                .any(|member| finding.location.json_path.starts_with(member))
        });
    }
    read.commit(attempt);
    let trade = trade?;
    // `{}` is the trade as written, as styled JSON text; a number no double holds is shown as
    // the largest finite one, as it is read.
    let styled = || {
        let mut shown = written.clone();
        bedrock_addon_core::json::clamp_non_finite(&mut shown, None);
        bedrock_addon_core::json::to_json(&shown).unwrap_or_default()
    };
    let drops = |read: &mut Read<'_>, code: Code, key: &str| {
        kept_error(read, code, at.child(key));
        dropped(read, &node);
        None
    };
    if trade.wants.is_empty() {
        return drops(read, codes::NO_OFFERS.with([styled().trim()]), "wants");
    }
    if trade.gives.is_empty() {
        return drops(read, codes::NO_RECEIVES.with([styled().trim()]), "gives");
    }
    if let Some(weight) = trade.weight {
        if weight <= 0 {
            // The game logs an error and drops the trade: a kept error.
            let code = codes::INVALID_WEIGHT.with([weight.to_string()]);
            return drops(read, code, "weight");
        }
        // A `weight` above 0 in a group that selects -1 (every trade) has no effect. The
        // game logs the text at the content log's warning level: a warning.
        if let Some(finding) =
            rules::weight_without_effect(selects, &at.child("weight"), Severity::Warning)
        {
            read.push(finding);
        }
    }
    Some(trade)
}

/// A `wants` / `gives` list: each entry `{"choice": [...]}` when `choice` is an array,
/// otherwise one item; a list that is no array is empty.
pub(super) fn entries(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<ItemList>> {
    if !node.value().is_array() {
        return Some(Vec::new());
    }
    // The empty `choice` lists, told once the list is read.
    let mut empty: Vec<Diagnostic> = Vec::new();
    let kept = read_list(node, read, ElementFailure::Drop, |entry, read| {
        let choice = member(entry, read, "choice").filter(|choice| choice.value().is_array());
        let Some(choice) = choice else {
            return TradeItem::from_json(entry, read).map(ItemList::Item);
        };
        let items = read_list(&choice, read, ElementFailure::Drop, TradeItem::from_json)?;
        // Only `choice` is read of such an entry.
        let members = read.object(entry)?;
        for (key, value) in read.unknown(&members, &["choice"], UnknownRule::Keep) {
            read.findings_mut()
                .unknown_mut()
                .put(entry.path(), key, value.clone());
        }
        // An entry whose `choice` is empty is dropped. The crate tells the empty list where
        // other entries stay (an element that is no item has said so itself).
        if choice
            .value()
            .as_array()
            .is_some_and(facet_value::VArray::is_empty)
        {
            let mut finding =
                Diagnostic::info(Code::from(&core_codes::INVALID_VALUE)).at(choice.path().clone());
            finding.location.span = choice.span();
            empty.push(finding);
        }
        (!items.is_empty()).then_some(ItemList::Choice { choice: items })
    })?;
    // A list left empty says so itself, at the list.
    if !kept.is_empty() {
        for finding in empty {
            read.push(finding);
        }
    }
    Some(kept)
}

/// `quantity`: kept as written ([`kept`]); the object form is read for `min` and `max` only,
/// its other members told and kept with it.
pub(super) fn quantity(node: &Node<'_>, read: &mut Read<'_>) -> Option<Value> {
    if node.value().is_object() {
        let members = read.object(node)?;
        for (key, value) in read.unknown(&members, &["min", "max"], UnknownRule::Keep) {
            read.findings_mut()
                .unknown_mut()
                .put(node.path(), key, value.clone());
        }
    }
    kept(node, read)
}

/// A member kept as written (`quantity`, `functions`, `filters`, `biomes`). A number of it that
/// no double holds is the largest finite double of its sign, with an `InvalidValue`
/// information at the number, so the loaded table holds no infinity that no writer spells.
#[allow(
    clippy::unnecessary_wraps,
    reason = "a `schema::read_with` function: it reads every value"
)]
pub(super) fn kept(node: &Node<'_>, read: &mut Read<'_>) -> Option<Value> {
    let mut value = node.value().clone();
    bedrock_addon_core::json::clamp_non_finite_warned(
        &mut value,
        None,
        node.path(),
        read.findings_mut(),
    );
    Some(value)
}

// ---- scalar conversions (`Json::Value::as*`) -----------------------------------------
//
// Vanilla converts in silence; here every conversion that reads another value than the
// file writes is an information at the member: `InvalidType` for a value of another JSON
// type, `InvalidValue` for a number the member does not hold. A `null` member never reaches
// them: it is absent.

/// `format_version`, read only when it is a string; another value is told and absent.
pub(super) fn text(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    let text = node.value().as_string().map(|s| s.as_str().to_owned());
    if text.is_none() {
        note(read, node, Code::from(&core_codes::INVALID_TYPE));
    }
    text
}

/// A float: a value that is no number reads as absent (vanilla: 0.0) with an `InvalidType`
/// information; a number beyond a float is the largest finite float of its sign, with the
/// information of such a number.
pub(super) fn float(node: &Node<'_>, read: &mut Read<'_>) -> Option<f32> {
    let Some(number) = node.value().as_number() else {
        note(read, node, Code::from(&core_codes::INVALID_TYPE));
        return None;
    };
    let nearest = narrow(number.to_f64_lossy());
    if bedrock_addon_core::json::lossy::narrows(number.to_f64_lossy()) {
        // Held as a float, as in the game, and written so: noted.
        let mut finding = Diagnostic::of(&core_codes::LOSSY_CONVERSION).at(node.path().clone());
        finding.location.span = node.span();
        read.push(finding);
    }
    if nearest.is_infinite() {
        read.push(bedrock_addon_core::molang::beyond_f32_warning(
            node.path().clone(),
            node.span(),
        ));
        return Some(f32::MAX.copysign(nearest));
    }
    Some(nearest)
}

/// An integer: an integer token, a real truncated toward zero (a fraction with an
/// `InvalidValue` information), a boolean as 0 / 1 (with an `InvalidType` information).
fn integer(node: &Node<'_>, read: &mut Read<'_>) -> Option<i64> {
    let json = node.value();
    if let Some(b) = json.as_bool() {
        note(read, node, Code::from(&core_codes::INVALID_TYPE));
        return Some(i64::from(b));
    }
    let Some(number) = json.as_number() else {
        note(read, node, Code::from(&core_codes::INVALID_TYPE));
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
        note(read, node, Code::from(&core_codes::INVALID_VALUE));
    }
    Some(n)
}

/// An integer member of type `T` (`asInt`, `asUInt`); one the integer does not fit is told and
/// absent.
pub(super) fn fits<T: TryFrom<i64>>(node: &Node<'_>, read: &mut Read<'_>) -> Option<T> {
    let fits = T::try_from(integer(node, read)?).ok();
    if fits.is_none() {
        note(read, node, Code::from(&core_codes::INVALID_VALUE));
    }
    fits
}

/// A boolean (`asBool`): a number is `true` when not zero, with an `InvalidType` information.
pub(super) fn boolean(node: &Node<'_>, read: &mut Read<'_>) -> Option<bool> {
    let json = node.value();
    if let Some(flag) = json.as_bool() {
        return Some(flag);
    }
    note(read, node, Code::from(&core_codes::INVALID_TYPE));
    json.as_number().map(|n| n.to_f64_lossy() != 0.0)
}

/// A string (`asString("")`): a number or a boolean reads as its text, with an `InvalidType`
/// information.
pub(super) fn string(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    let json = node.value();
    if let Some(text) = json.as_string() {
        return Some(text.as_str().to_owned());
    }
    // Not a string: said, whatever it reads as.
    note(read, node, Code::from(&core_codes::INVALID_TYPE));
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

/// The nearest `f32`.
#[allow(clippy::cast_possible_truncation)]
fn narrow(x: f64) -> f32 {
    x as f32
}
