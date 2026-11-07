//! The release gates of a component-based document: vanilla's item-level gate, and per
//! component its deprecation and its `released` / `requires` rows, the newest format's table
//! and the rows of the components only an older format binds.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::load::{LoadContext, LoadRequirement};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::Family;
use indexmap::IndexMap;

use crate::codes;
use crate::component::ItemComponent;
use crate::history::ItemFamily;

/// A gate row of a component no newer format binds (or binds under another name).
#[derive(Clone, Copy)]
pub(crate) struct GateRow {
    /// The JSON key.
    pub(crate) name: &'static str,
    /// The format version from which the component is released.
    pub(crate) released: Option<FormatVersion>,
    /// Unlocks it before `released`.
    pub(crate) requires: &'static [LoadRequirement],
    /// The version it was deprecated in.
    pub(crate) deprecated_since: Option<FormatVersion>,
    /// Whether its deprecation is the schema's deprecated-member line (cereal's `member "<key>"
    /// is deprecated`, which refuses the item) in place of the generic one (a kept error).
    pub(crate) deprecated_member: bool,
}

const LCF: LoadRequirement = LoadRequirement::LegacyCreatorFeatures;
const BETA_APIS: LoadRequirement =
    LoadRequirement::Experiment(bedrock_addon_core::load::Experiment::BetaApis);

/// The registered components the newest format does not bind.
pub(crate) const HISTORY_ROWS: &[GateRow] = &[
    GateRow {
        name: "minecraft:armor",
        released: None,
        requires: &[LCF],
        deprecated_since: Some(FormatVersion::new(1, 20, 20)),
        deprecated_member: false,
    },
    GateRow {
        name: "minecraft:chargeable",
        released: None,
        requires: &[LCF],
        deprecated_since: Some(FormatVersion::new(1, 20, 50)),
        deprecated_member: false,
    },
    GateRow {
        name: "minecraft:custom_components",
        released: Some(FormatVersion::new(1, 21, 10)),
        requires: &[BETA_APIS],
        deprecated_since: Some(known::V1_21_90),
        deprecated_member: true,
    },
    GateRow {
        name: "minecraft:on_use",
        released: None,
        requires: &[LCF],
        deprecated_since: Some(FormatVersion::new(1, 20, 40)),
        deprecated_member: false,
    },
    GateRow {
        name: "minecraft:on_use_on",
        released: None,
        requires: &[LCF],
        deprecated_since: Some(FormatVersion::new(1, 20, 40)),
        deprecated_member: false,
    },
    GateRow {
        name: "minecraft:render_offsets",
        released: None,
        requires: &[LCF],
        deprecated_since: Some(FormatVersion::new(1, 20, 10)),
        deprecated_member: false,
    },
    GateRow {
        name: "minecraft:use_duration",
        released: Some(FormatVersion::new(1, 20, 20)),
        requires: &[],
        deprecated_since: None,
        deprecated_member: false,
    },
    GateRow {
        name: "minecraft:weapon",
        released: None,
        requires: &[LCF],
        deprecated_since: Some(FormatVersion::new(1, 20, 40)),
        deprecated_member: false,
    },
];

/// The gate row of `name`: the newest format's table, else [`HISTORY_ROWS`].
fn row(name: &str) -> Option<GateRow> {
    if let Some(meta) = ItemComponent::TABLE.get(name) {
        return Some(GateRow {
            name: meta.name,
            released: meta.released,
            requires: meta.requires,
            deprecated_since: meta.deprecated_since,
            // A table row with a text of its own is a deprecated member of the schema.
            deprecated_member: meta.deprecated_message.is_some(),
        });
    }
    HISTORY_ROWS.iter().find(|row| row.name == name).copied()
}

/// The smallest `released` of any component: below it, vanilla's item-level gate rejects every
/// component-based item of a pack without the legacy creator features (computed from the component
/// table, as the game does).
pub(crate) fn release_threshold(
    rows: impl Iterator<Item = Option<FormatVersion>>,
) -> Option<FormatVersion> {
    rows.flatten().min()
}

/// The threshold of the item tables.
fn item_threshold() -> Option<FormatVersion> {
    release_threshold(
        ItemComponent::TABLE
            .0
            .iter()
            .map(|meta| meta.released)
            .chain(HISTORY_ROWS.iter().map(|row| row.released)),
    )
}

/// The parse-time gates of a component-based document: vanilla's item-level release
/// gate, then per component as written its deprecation, then its `released` / `requires`.
/// Returns the keys to drop, or `None` when the item is not loaded (an error says why).
pub(crate) fn gate(
    components: &IndexMap<String, Value>,
    identifier: &str,
    original: FormatVersion,
    cx: &LoadContext,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<Vec<String>> {
    gate_with(
        components,
        identifier,
        original,
        cx,
        at,
        d,
        item_threshold(),
        row,
    )
}

/// [`gate`] over the given threshold and rows (tests lower the table).
#[allow(clippy::too_many_arguments, reason = "the gate's inputs")]
pub(crate) fn gate_with(
    components: &IndexMap<String, Value>,
    identifier: &str,
    original: FormatVersion,
    cx: &LoadContext,
    at: &JsonPath,
    d: &mut Diagnostics,
    threshold: Option<FormatVersion>,
    row: fn(&str) -> Option<GateRow>,
) -> Option<Vec<String>> {
    let lcf = LoadRequirement::LegacyCreatorFeatures.satisfied(cx);
    // The game refuses an item below the minimum format version released for any component
    // unless the legacy creator features are on.
    if !lcf
        && let Some(threshold) = threshold
        && original < threshold
    {
        d.push(Diagnostic::error(
            core_codes::TOGGLE_USE_FORMAT_VERSION
                .with(["item", identifier, threshold.to_string().as_str()])
                .into(),
        ));
        return None;
    }
    let mut dropped = Vec::new();
    let mut rejected = false;
    for key in components.keys() {
        let Some(row) = row(key) else {
            continue;
        };
        // vanilla's armor fixer removes it from the raw JSON before the check
        if key == "minecraft:armor" && original < FormatVersion::new(1, 20, 20) {
            continue;
        }
        let path = at.child(key);
        if let Some(since) = row.deprecated_since
            && original >= since
        {
            let code = if row.deprecated_member {
                core_codes::MEMBER_DEPRECATED.with([key])
            } else {
                core_codes::DEPRECATED_IN_FORMAT.with([key.clone(), original.to_string()])
            };
            // The game logs "%s has been deprecated and is not available in json format %s"
            // at the content log's error level and removes the member; the item loads on: a
            // kept error. A row of the schema's own line (`custom_components`) is a deprecated
            // member of the schema instead: that line is an error under every error mask, so
            // the item is refused (the game's own unit tests expect this text for a current
            // format version).
            let finding = Diagnostic::error(code.into()).at(path);
            if row.deprecated_member {
                d.push(finding);
            } else {
                d.push_kept_error(finding);
            }
            dropped.push(key.clone());
            continue;
        }
        let released = row.released.is_some_and(|r| original >= r);
        if released {
            continue;
        }
        if let Some(failed) = row.requires.iter().find(|r| !r.satisfied(cx)) {
            rejected = true;
            d.push(Diagnostic::error(requirement_code(*failed, key, row.released)).at(path));
        }
    }
    (!rejected).then_some(dropped)
}

/// The code of a failed requirement: the game's line for it, or the bare code where the game
/// has none.
fn requirement_code(
    requirement: LoadRequirement,
    name: &str,
    released: Option<FormatVersion>,
) -> DiagnosticCode {
    use bedrock_addon_core::load::Experiment;
    let toggle = |experiment: Experiment| experiment.save_name().unwrap_or_default();
    let what = ItemFamily::COMPONENT_NOUN;
    let experiment = match requirement {
        LoadRequirement::LegacyCreatorFeatures => {
            // The crate's own text where the component has no release version.
            return released
                .map_or_else(
                    || codes::LEGACY_ONLY.with([name]),
                    |version| {
                        core_codes::TOGGLE_USE_FORMAT_VERSION.with([
                            what,
                            name,
                            version.to_string().as_str(),
                        ])
                    },
                )
                .into();
        }
        LoadRequirement::BetaFeatures => Experiment::BetaApis,
        LoadRequirement::Experiment(experiment) => experiment,
        _ => return (&core_codes::MISSING_TOGGLES).into(),
    };
    match released {
        Some(version) => core_codes::TURN_ON_TOGGLE_BEFORE.with([
            what,
            name,
            version.to_string().as_str(),
            toggle(experiment),
        ]),
        None => core_codes::TURN_ON_TOGGLE.with([what, name, toggle(experiment)]),
    }
    .into()
}
