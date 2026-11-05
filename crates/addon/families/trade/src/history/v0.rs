//! `v0`: a trade table as vanilla's legacy parser reads it, for a document without
//! `format_version` or below 1.21.30.

use super::{convert, read};
use crate as latest;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Phase,
};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::ItemId;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned, write_payload,
};

/// The legacy slice's version.
pub const VERSION: FormatVersion = FormatVersion::new(0, 0, 0);

/// A legacy trade table: the document as written.
///
/// JSON: `{"format_version"?: "<version>", "tiers": [tier, …]}`.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
pub struct TradeTable {
    /// `format_version`, when it is a string (the legacy parser reads no other form).
    pub format_version: Option<String>,
    /// The tiers (JSON `tiers`).
    pub tiers: Vec<Tier>,
    /// The members the legacy parser does not read, kept for writing the document back in its
    /// own shape. The upgraded table cannot carry them.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// A legacy tier: groups, or one flat list of trades that are all offered.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
pub struct Tier {
    /// JSON `total_exp_required`; absent = the tier's 0-based index.
    pub total_exp_required: Option<u32>,
    /// JSON `groups`; when present, `trades` is not read.
    pub groups: Option<Vec<Group>>,
    /// JSON `trades`: one group that offers every trade.
    pub trades: Option<Vec<Trade>>,
    /// The members the legacy parser does not read, kept for writing the document back in its
    /// own shape. The upgraded table cannot carry them.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// A legacy group.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
pub struct Group {
    /// JSON `num_to_select`; absent = -1 (every trade).
    pub num_to_select: Option<i32>,
    /// JSON `trades`.
    pub trades: Vec<Trade>,
    /// The members the legacy parser does not read, kept for writing the document back in its
    /// own shape. The upgraded table cannot carry them.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// A legacy trade; the defaults are the newest ones.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
pub struct Trade {
    /// JSON `wants`.
    pub wants: Vec<ItemList>,
    /// JSON `gives`.
    pub gives: Vec<ItemList>,
    /// JSON `trader_exp` (default 1).
    pub trader_exp: Option<u32>,
    /// JSON `max_uses` (default 7).
    pub max_uses: Option<i32>,
    /// JSON `reward_exp` (default `true`).
    pub reward_exp: Option<bool>,
    /// JSON `weight` (default 1).
    pub weight: Option<i32>,
    /// The members the legacy parser does not read, kept for writing the document back in its
    /// own shape. The upgraded table cannot carry them.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// A legacy `wants` / `gives` entry.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(untagged)]
#[repr(u8)]
#[allow(
    clippy::large_enum_variant,
    reason = "the bare item is the common form; an entry is built once and stored"
)]
pub enum ItemList {
    /// `{"choice": [item, …]}`.
    Choice {
        /// JSON `choice`.
        choice: Vec<TradeItem>,
        /// The other members of the entry, which the legacy parser does not read.
        #[facet(skip, rename = "\u{0}extra")]
        extra: Extra,
    },
    /// A bare item.
    Item(TradeItem),
}

/// A legacy trade item, its members as written.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
pub struct TradeItem {
    /// JSON `quantity`: a number, `"a-b"`, `[a, b]` or `{"min", "max"}`; absent = 1.
    pub quantity: Option<Value>,
    /// JSON `price_multiplier` (default 0.0).
    pub price_multiplier: Option<f32>,
    /// JSON `item`, read as a string (`"name"` or `"name:aux"`).
    pub item: Option<String>,
    /// JSON `functions`: loot-table item functions, kept as written.
    pub functions: Option<Value>,
    /// JSON `filters`: a legacy filter, kept as written.
    pub filters: Option<Value>,
    /// JSON `biomes`: a biome name or a list of them, read only by the documents below 1.20.30.
    pub biomes: Option<Value>,
    /// The members the legacy parser does not read, kept for writing the document back in its
    /// own shape. The upgraded table cannot carry them.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl Versioned for TradeTable {
    const VERSION: FormatVersion = VERSION;
}

impl VersionStruct for TradeTable {
    /// The legacy parser over the whole document (the raw-text path).
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        read::read(payload, &cx.at, d)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

/// The step `v0 → v1_21_30`. Vanilla's own step is a pure move; the conversions live in its
/// legacy parser and are done here. Findings are at the members'
/// paths in the file as written: the reader has recorded where each element it kept is in
/// the file, and the step records what it rebuilds (a group that is not upgraded, the flat
/// `trades`, the items an item becomes), so what is told of the upgraded table afterwards is
/// at the file's member too.
impl Upgrade for TradeTable {
    type Next = latest::TradeTable;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> latest::TradeTable {
        let root = JsonPath::new();
        let tiers_at = root.child("tiers");
        let tiers = self
            .tiers
            .into_iter()
            .enumerate()
            .map(|(index, tier)| upgrade_tier(index, tier, &tiers_at.index(index), cx))
            .collect();
        latest::TradeTable { tiers }
    }
}

/// One tier.
fn upgrade_tier(index: usize, tier: Tier, at: &JsonPath, cx: &mut UpgradeCx<'_>) -> latest::Tier {
    let groups = match (tier.groups, tier.trades) {
        // A group without `num_to_select` gets the legacy default -1 written; the newest
        // default is 0.
        (Some(groups), _) => {
            let groups_at = at.child("groups");
            let mut kept = Vec::with_capacity(groups.len());
            let mut places = Vec::with_capacity(groups.len());
            for (g, group) in groups.into_iter().enumerate() {
                let trades_at = groups_at.index(g).child("trades");
                let Some(trades) = kept_trades(group.trades, &trades_at, cx) else {
                    continue;
                };
                kept.push(latest::Group {
                    num_to_select: Some(group.num_to_select.unwrap_or(-1)),
                    trades: upgrade_trades(trades, &trades_at, cx),
                });
                places.push(g);
            }
            // A group behind one that is not upgraded: named where the file has it.
            cx.diagnostics.left_out(&groups_at, &places);
            kept
        }
        // Flat `trades` become one group that offers every trade.
        (None, Some(trades)) => {
            let trades_at = at.child("trades");
            let group = kept_trades(trades, &trades_at, cx).map(|trades| latest::Group {
                num_to_select: Some(-1),
                trades: upgrade_trades(trades, &trades_at, cx),
            });
            if group.is_some() {
                // The one group's trades are the tier's own list in the file.
                let made = at.child("groups").index(0).child("trades");
                cx.diagnostics.moved(trades_at, made);
            }
            group.into_iter().collect()
        }
        (None, None) => Vec::new(),
    };
    latest::Tier {
        groups,
        // The default `total_exp_required` is the tier's 0-based index.
        total_exp_required: tier
            .total_exp_required
            .unwrap_or_else(|| u32::try_from(index).unwrap_or(u32::MAX)),
    }
}

/// The trades of a legacy group, `None` when it has none (an empty list, or every trade was
/// dropped as it was read): the newest format has no group without a trade
/// (`trades` needs at least one entry), so the group is not upgraded, with an `InvalidValue`
/// information at its `trades`. Without it the upgraded table would be written as a text this
/// loader refuses. The game's upgrade is a move that logs nothing, so the crate reports an
/// information.
fn kept_trades(trades: Vec<Trade>, at: &JsonPath, cx: &mut UpgradeCx<'_>) -> Option<Vec<Trade>> {
    if trades.is_empty() {
        cx.diagnostics.push(
            Diagnostic::info(DiagnosticCode::InvalidValue { text: None })
                .at(at.clone())
                .phase(Phase::Upgrade),
        );
        // The group's other members go with it: told at the group. The flat form
        // has no group of the file's.
        let mut group = at.clone();
        // `…/groups/<i>`: a group of a tier, not the flat list at the root.
        let in_groups = |path: &JsonPath| {
            let mut steps = path.segments().skip(1).peekable();
            std::iter::from_fn(|| {
                let step = steps.next()?;
                Some(
                    step == bedrock_addon_core::diagnostic::JsonPathSegment::Key("groups")
                        && steps.peek().is_some(),
                )
            })
            .any(|found| found)
        };
        if group.pop() && in_groups(&group) {
            // The game's upgrade says nothing here; the crate reports an information.
            cx.diagnostics.push(
                Diagnostic::info(DiagnosticCode::DroppedField { text: None })
                    .at(group)
                    .phase(Phase::Upgrade),
            );
        }
        return None;
    }
    Some(trades)
}

/// A group's trades; `at` is the path of the `trades` list.
fn upgrade_trades(trades: Vec<Trade>, at: &JsonPath, cx: &mut UpgradeCx<'_>) -> Vec<latest::Trade> {
    trades
        .into_iter()
        .enumerate()
        .map(|(i, trade)| {
            let trade_at = at.index(i);
            latest::Trade {
                wants: upgrade_lists(trade.wants, &trade_at.child("wants"), cx),
                gives: upgrade_lists(trade.gives, &trade_at.child("gives"), cx),
                trader_exp: trade.trader_exp,
                max_uses: trade.max_uses,
                reward_exp: trade.reward_exp,
                weight: trade.weight,
            }
        })
        .collect()
}

/// A `wants` / `gives` list; `at` is its path.
fn upgrade_lists(
    lists: Vec<ItemList>,
    at: &JsonPath,
    cx: &mut UpgradeCx<'_>,
) -> Vec<latest::ItemList> {
    lists
        .into_iter()
        .enumerate()
        .map(|(e, list)| {
            let entry_at = at.index(e);
            match list {
                // The entry's unknown members stay behind with the legacy struct.
                ItemList::Choice { choice, .. } => {
                    let choice_at = entry_at.child("choice");
                    let mut items = Vec::with_capacity(choice.len());
                    // The index of the file's element each item is made from.
                    let mut made_from = Vec::with_capacity(choice.len());
                    for (c, item) in choice.into_iter().enumerate() {
                        // The expansions of `choice` elements are flattened into the choice.
                        let made = upgrade_item(item, &choice_at.index(c), cx);
                        made_from.extend(std::iter::repeat_n(c, made.len()));
                        items.extend(made);
                    }
                    // Each item is named at the element of the file it is made from. The
                    // last first: an item's place is never one an earlier move filled.
                    for (now, was) in made_from.into_iter().enumerate().rev() {
                        cx.diagnostics
                            .moved(choice_at.index(was), choice_at.index(now));
                    }
                    latest::ItemList::Choice { choice: items }
                }
                ItemList::Item(item) => {
                    let mut items = upgrade_item(item, &entry_at, cx);
                    match items.pop() {
                        Some(item) if items.is_empty() => latest::ItemList::Item(item),
                        // A bare entry that expands (into five) becomes `{"choice": [...]}`.
                        last => {
                            items.extend(last);
                            // Each of them is the one item the file has at the entry.
                            let choice_at = entry_at.child("choice");
                            for now in 0..items.len() {
                                cx.diagnostics.moved(entry_at.clone(), choice_at.index(now));
                            }
                            latest::ItemList::Choice { choice: items }
                        }
                    }
                }
            }
        })
        .collect()
}

/// What the announcement of a legacy blob says of the member the written table has it as.
const WRITTEN_AS_FNCTNS: &str =
    "written back as `functions` of a 1.20.30 legacy document, which keeps its legacy evaluation";
const WRITTEN_AS_FLTRS: &str =
    "written back as `filters` of a 1.20.30 legacy document, which keeps its legacy evaluation";

/// One item of the file, at `at`: the items it becomes (five when the 1.18.20 gate expands
/// it, else itself). The gates run per `choice` element or on the entry itself, the expansion
/// first, then the `biomes` conversion.
fn upgrade_item(item: TradeItem, at: &JsonPath, cx: &mut UpgradeCx<'_>) -> Vec<latest::TradeItem> {
    // The quantity is converted once, for the item as written. Its expansions share it: vanilla
    // copies the misspelt key `quanitity` into them and so loses it; the crate keeps it.
    let quantity = item
        .quantity
        .as_ref()
        .and_then(|json| convert::quantity(json, &at.child("quantity"), cx.diagnostics));
    // Below 1.18.20 an item with a `trader_material_type` expands into several items.
    let expansion = (cx.original_version < convert::BIOME_FILTERS)
        .then(|| convert::material_choices(&item))
        .flatten();
    convert::tell_lost(&item, expansion.is_some(), at, cx);
    let present = |blob: &Option<Value>| blob.as_ref().is_some_and(|blob| !blob.is_null());
    let expanded = expansion.is_some();
    // Whether the `biomes` gate made the filters an item carries.
    let mut from_biomes = false;
    let items: Vec<latest::TradeItem> = expansion
        .unwrap_or_else(|| vec![item])
        .into_iter()
        .map(|mut item| {
            // Below 1.20.30 the `biomes` member becomes a filter.
            if cx.original_version < convert::GENERIC_FILTERS {
                let before = item.filters.clone();
                convert::biomes_to_filters(&mut item);
                from_biomes |= item.filters != before;
            }
            latest::TradeItem {
                quantity,
                item: ItemId::parse(item.item.unwrap_or_default()),
                // The typed members stay empty; the legacy blobs keep their legacy
                // evaluation.
                functions: None,
                filters: None,
                price_multiplier: item.price_multiplier,
                legacy_functions: item.functions,
                legacy_filters: item.filters,
            }
        })
        .collect();
    // The newest format has the blobs only under deprecated names the game rejects, so the
    // table is written back as a legacy document (`legacy_form`). Said here, at the
    // member of the file the blob comes from (the game says nothing while it upgrades).
    let functions = items
        .iter()
        .any(|item| present(&item.legacy_functions))
        .then_some(("functions", WRITTEN_AS_FNCTNS));
    let filters = items
        .iter()
        .any(|item| present(&item.legacy_filters))
        .then(|| {
            let from = if expanded {
                // The filters of a `trader_material_type` expansion (the file's own are
                // not copied into it, and told as dropped).
                "functions"
            } else if from_biomes {
                "biomes"
            } else {
                "filters"
            };
            (from, WRITTEN_AS_FLTRS)
        });
    for (from, written_as) in functions.into_iter().chain(filters) {
        cx.diagnostics.push(
            Diagnostic::info(DiagnosticCode::ValueRewritten { text: None })
                .at(at.child(from))
                .phase(Phase::Upgrade)
                .detail(DiagnosticDetail::Note(written_as)),
        );
    }
    items
}

/// The legacy document a newest table that holds a legacy blob (`legacy_functions`,
/// `legacy_filters`) is written as; `Ok(None)` when it holds none.
///
/// The newest format has the blobs only under `__DEP_fnctns` / `__DEP_fltrs`, which the game
/// binds and deprecates by name: a document that writes one is rejected.
/// So such a table is written in the legacy shape, the blobs as `functions` / `filters`, and
/// stamped with the legacy version from which the legacy parser reads both as written: 1.20.30
/// (`GENERIC_FILTERS`; from 1.18.20 no `trader_material_type` expansion, from 1.20.30 no
/// `biomes` conversion). Read back, the step makes the same table. A table that holds a typed
/// `functions` or `filters` beside a legacy blob has no form a reader reads back:
/// `NotRepresentable`.
pub(crate) fn legacy_form(table: &latest::TradeTable) -> Result<Option<TradeTable>, EncodeError> {
    let items = || {
        table
            .tiers
            .iter()
            .flat_map(|tier| &tier.groups)
            .flat_map(|group| &group.trades)
            .flat_map(|trade| trade.wants.iter().chain(&trade.gives))
            .flat_map(latest::ItemList::items)
    };
    if !items().any(|item| item.legacy_functions.is_some() || item.legacy_filters.is_some()) {
        return Ok(None);
    }
    if items().any(|item| item.functions.is_some() || item.filters.is_some()) {
        return Err(EncodeError::NotRepresentable {
            at: JsonPath::new(),
            what: bedrock_addon_core::json::Unrepresentable::Unsupported,
        });
    }
    let item = |item: &latest::TradeItem| -> Result<TradeItem, EncodeError> {
        Ok(TradeItem {
            quantity: item.quantity.as_ref().map(write_payload).transpose()?,
            price_multiplier: item.price_multiplier,
            item: Some(item.item.as_str().to_owned()),
            functions: item.legacy_functions.clone(),
            filters: item.legacy_filters.clone(),
            biomes: None,
            extra: Extra::default(),
        })
    };
    let lists = |lists: &[latest::ItemList]| -> Result<Vec<ItemList>, EncodeError> {
        lists
            .iter()
            .map(|list| {
                Ok(match list {
                    latest::ItemList::Choice { choice } => ItemList::Choice {
                        choice: choice.iter().map(item).collect::<Result<_, _>>()?,
                        extra: Extra::default(),
                    },
                    latest::ItemList::Item(one) => ItemList::Item(item(one)?),
                })
            })
            .collect()
    };
    let mut tiers = Vec::with_capacity(table.tiers.len());
    for tier in &table.tiers {
        let mut groups = Vec::with_capacity(tier.groups.len());
        for group in &tier.groups {
            let mut trades = Vec::with_capacity(group.trades.len());
            for trade in &group.trades {
                trades.push(Trade {
                    wants: lists(&trade.wants)?,
                    gives: lists(&trade.gives)?,
                    trader_exp: trade.trader_exp,
                    max_uses: trade.max_uses,
                    reward_exp: trade.reward_exp,
                    weight: trade.weight,
                    extra: Extra::default(),
                });
            }
            groups.push(Group {
                // Absent is -1 in the legacy parser and 0 in the newest format.
                num_to_select: Some(group.num_to_select()),
                trades,
                extra: Extra::default(),
            });
        }
        tiers.push(Tier {
            total_exp_required: Some(tier.total_exp_required),
            groups: Some(groups),
            trades: None,
            extra: Extra::default(),
        });
    }
    Ok(Some(TradeTable {
        format_version: Some(convert::GENERIC_FILTERS.to_string()),
        tiers,
        extra: Extra::default(),
    }))
}
