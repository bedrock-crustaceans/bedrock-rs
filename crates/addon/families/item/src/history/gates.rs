//! The release gates of a component-based document: vanilla's item-level gate, and per
//! component its deprecation and its `released` / `requires` rows, the newest format's table
//! and the rows of the components only an older format binds.

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, VanillaText,
};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::load::{LoadContext, LoadRequirement};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::Family;
use indexmap::IndexMap;

use crate::component::ItemComponent;
use crate::history::ItemFamily;
use crate::texts;

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
    /// Its own deprecation text.
    pub(crate) deprecated_message: Option<&'static str>,
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
        deprecated_message: None,
    },
    GateRow {
        name: "minecraft:chargeable",
        released: None,
        requires: &[LCF],
        deprecated_since: Some(FormatVersion::new(1, 20, 50)),
        deprecated_message: None,
    },
    GateRow {
        name: "minecraft:custom_components",
        released: Some(FormatVersion::new(1, 21, 10)),
        requires: &[BETA_APIS],
        deprecated_since: Some(known::V1_21_90),
        deprecated_message: Some(texts::CUSTOM_COMPONENTS_DEPRECATED),
    },
    GateRow {
        name: "minecraft:on_use",
        released: None,
        requires: &[LCF],
        deprecated_since: Some(FormatVersion::new(1, 20, 40)),
        deprecated_message: None,
    },
    GateRow {
        name: "minecraft:on_use_on",
        released: None,
        requires: &[LCF],
        deprecated_since: Some(FormatVersion::new(1, 20, 40)),
        deprecated_message: None,
    },
    GateRow {
        name: "minecraft:render_offsets",
        released: None,
        requires: &[LCF],
        deprecated_since: Some(FormatVersion::new(1, 20, 10)),
        deprecated_message: None,
    },
    GateRow {
        name: "minecraft:use_duration",
        released: Some(FormatVersion::new(1, 20, 20)),
        requires: &[],
        deprecated_since: None,
        deprecated_message: None,
    },
    GateRow {
        name: "minecraft:weapon",
        released: None,
        requires: &[LCF],
        deprecated_since: Some(FormatVersion::new(1, 20, 40)),
        deprecated_message: None,
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
            deprecated_message: meta.deprecated_message,
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
        d.push(Diagnostic::error(DiagnosticCode::MissingToggles {
            text: Some(VanillaText::UseFormatVersion {
                what: "item",
                name: identifier.to_owned(),
                version: threshold,
            }),
        }));
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
            let text = row.deprecated_message.map_or_else(
                || VanillaText::Deprecated {
                    member: key.clone(),
                    version: original,
                },
                VanillaText::Fixed,
            );
            // The game logs "%s has been deprecated and is not available in json format %s"
            // at the content log's error level and removes the member; the item loads on: a
            // kept error. A row with its own text (`custom_components`) is a deprecated
            // member of the schema instead: that line is an error under every error mask, so
            // the item is refused (the game's own unit tests expect this text for a current
            // format version).
            let finding =
                Diagnostic::error(DiagnosticCode::DeprecatedMember { text: Some(text) }).at(path);
            if row.deprecated_message.is_some() {
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
            d.push(
                Diagnostic::error(DiagnosticCode::MissingToggles {
                    text: requirement_text(*failed, key, row.released),
                })
                .at(path),
            );
        }
    }
    (!rejected).then_some(dropped)
}

/// The text of a failed requirement.
fn requirement_text(
    requirement: LoadRequirement,
    name: &str,
    released: Option<FormatVersion>,
) -> Option<VanillaText> {
    use bedrock_addon_core::load::Experiment;
    let toggle = |experiment: Experiment| experiment.save_name().unwrap_or_default();
    let what = ItemFamily::COMPONENT_NOUN;
    let experiment = match requirement {
        LoadRequirement::LegacyCreatorFeatures => {
            // The crate's own text where the component has no release version.
            return Some(released.map_or_else(
                || VanillaText::ItemLegacyOnly {
                    component: name.to_owned(),
                },
                |version| VanillaText::UseFormatVersion {
                    what,
                    name: name.to_owned(),
                    version,
                },
            ));
        }
        LoadRequirement::BetaFeatures => Experiment::BetaApis,
        LoadRequirement::Experiment(experiment) => experiment,
        _ => return None,
    };
    Some(match released {
        Some(version) => VanillaText::TurnOnToggleBefore {
            what,
            name: name.to_owned(),
            version,
            toggle: toggle(experiment),
        },
        None => VanillaText::TurnOnToggle {
            what,
            name: name.to_owned(),
            toggle: toggle(experiment),
        },
    })
}
