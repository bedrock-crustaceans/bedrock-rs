//! The family rules on [`Block`],
//! after the generic component checks and the Molang compile pass.
//!
//! "The block is not loaded" is an error. A rule whose game consequence is a logged error after
//! which the game carries on is a kept error, and a rule the game does not log at all is an
//! information of the crate's own. The game's texts are carried as `VanillaText`; findings of
//! the crate's own carry no text (their code and path say what they are).

use super::component::{
    BlockComponent, BlockEntity, BoundingBox, CollisionBoundingBox, CollisionBox, ConnectionRule,
    CraftingTable, DestructibleByExplosion, DestructibleByMining, DestructionParticles, Friction,
    Geometry, InstrumentSound, LightDampening, LightEmission, LiquidDetection,
    MaterialInstanceEntry, MaterialInstances, Movable, MovementType, NeighborChange,
    NeighborChangeDirection, PlacementFilter, PrecipitationBehavior, PrecipitationInteractions,
    RandomOffset, RedstoneConsumer, RedstoneProducer, RenderMethod, SelectionBox, Tags, Tick,
    TintMethod, Transformation, UvLock, Vec3, legacy_constraint,
};
use super::history::BlockFamily;
use super::read::{CEREAL, NEWEST};
use super::{Block, StateValues};
use bedrock_addon_core::diagnostic::cereal::{self, NumericRange};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::{ComponentMap, ComponentRef};
use bedrock_addon_core::id::syntax::{has_namespace, split_namespace};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::load::{Experiment, LoadContext};
use bedrock_addon_core::molang::{Molang, SourceForm};
use bedrock_addon_core::schema::{Schema, SchemaCx};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{
    Family, FieldMeta, Reading, ValidateCx, check_fields_since_engine,
};
use bedrock_addon_shared_shapes::{StateValue, event};

/// `Family::validate`.
pub(crate) fn validate(block: &mut Block, cx: &mut ValidateCx<'_>) {
    // Permutations whose section failed a gate are skipped (B.6).
    let mut skipped: Vec<usize> = cx.skipped.iter().filter_map(permutation_index).collect();
    #[cfg(feature = "molang")]
    {
        let failures = std::mem::take(&mut cx.molang_failures);
        skipped.extend(molang_failures(
            block,
            &failures,
            cx.load,
            &cx.at,
            cx.diagnostics,
        ));
    }
    // The payload's path: `minecraft:block`, also for a file below 1.26.20, which is read
    // whole (`read::read_file` gives the key).
    let at = cx.at.clone();
    skipped.extend(rules(block, cx.load, &at, cx.diagnostics));
    references(block, &at, cx.diagnostics);
    skipped.sort_unstable();
    skipped.dedup();
    let before = block.permutations.len();
    let kept: Vec<usize> = (0..before)
        .filter(|index| !skipped.contains(index))
        .collect();
    for index in skipped.iter().rev() {
        if *index < block.permutations.len() {
            block.permutations.remove(*index);
        }
    }
    // The permutation goes whole, with its condition and every other component: said where
    // the file has it (an information: the game skips it, and the gate's own finding is at
    // the component).
    for index in &skipped {
        let path = at.child("permutations").index(*index);
        cx.diagnostics.push(
            Diagnostic::info(DiagnosticCode::DroppedField { text: None })
                .at(path)
                .phase(Phase::Validate),
        );
    }
    if kept.len() < before {
        // The permutations left are known by the index the file has them at: what is told
        // of one from here on, and what the link pass finds in one, is told there.
        let list = at.child("permutations");
        cx.diagnostics.left_out(&list, &kept);
        // Where the file has the members the rules of 1.26.20 refuse, for the write's
        // refusal: the last step recorded them by the permutations as they were.
        block.refused_at.clear();
        for finding in refused_by_newest(block, &at) {
            let of_block = finding.location.json_path;
            let of_file = cx.diagnostics.source_of(&of_block);
            if of_file != of_block {
                block.refused_at.push((of_block, of_file));
            }
        }
    }
    // A block of a file below 1.26.20 is held, and written, in the shape of 1.26.20, whose
    // rules a file of that version is held to. What breaks one of them loads, as the game
    // reads the file at its own version, and has no text in that shape: `to_json` refuses
    // it, and the load says so with the rule's finding as an information of the upgrade (the
    // game, reading the file at its own version, says nothing).
    for mut finding in refused_by_newest(block, &at) {
        if told_at_the_component(&finding, &at, cx.diagnostics) {
            continue;
        }
        finding.severity = Severity::Info;
        finding.phase = Phase::Upgrade;
        cx.diagnostics.push(finding);
    }
}

/// Whether `finding`, of a member of `description/menu_category`, was told by the last step
/// of the chain at the `minecraft:creative_category` component the file has: a
/// finding of the upgrade with its code at that member of such a component.
fn told_at_the_component(finding: &Diagnostic, at: &JsonPath, found: &Diagnostics) -> bool {
    let menu = at.child("description").child("menu_category");
    let Some(member) = finding
        .location
        .json_path
        .strip_prefix(&menu)
        .filter(|member| !member.is_empty())
    else {
        return false;
    };
    let mut end = JsonPath::of_keys(["minecraft:creative_category"]);
    end.append(&member);
    found.iter().any(|known| {
        known.phase == Phase::Upgrade
            && known.code == finding.code
            && known.location.json_path.len() > end.len()
            && known.location.json_path.ends_with(&end)
    })
}

/// What the rules of the newest format (1.26.20) find in a block read from an older one that
/// the rules of its own version do not: where the written document (always of the newest
/// format) would be read as another block, or not at all. Empty for a block of the newest
/// format. The rules run on copies, in a detached context, with the block's own version and
/// with 1.26.20: a finding of the second run at a path the first has none at.
pub(crate) fn refused_by_newest(block: &Block, at: &JsonPath) -> Vec<Diagnostic> {
    if block.source_format_version >= CEREAL {
        return Vec::new();
    }
    let cx = LoadContext::detached();
    let run = |version: FormatVersion| {
        let mut copy = block.clone();
        copy.source_format_version = version;
        let mut found = Diagnostics::default();
        rules(&mut copy, &cx, at, &mut found);
        // A trial run: what it keeps of the block's members is not the block's.
        drop(found.take_unknown());
        found
    };
    let own = run(block.source_format_version);
    run(CEREAL)
        .into_iter()
        .filter(|finding| {
            // A finding of the block's own run that is a kept error does not stand for the
            // newest format's: the block loads, and the text of the newest format would not
            // (the legacy tick and selection box checks are kept errors).
            finding.severity >= Severity::Warning
                && !own.iter().any(|known| {
                    known.location.json_path == finding.location.json_path && !known.kept
                })
        })
        .collect()
}

/// `Block::validate`: the rules without a load (no gate skips, no Molang outcomes).
pub(crate) fn validate_detached(
    block: &mut Block,
    cx: &LoadContext,
    at: &JsonPath,
    d: &mut Diagnostics,
) {
    rules(block, cx, at, d);
}

/// The index of `[minecraft:block/]permutations/<i>/…`.
fn permutation_index(path: &JsonPath) -> Option<usize> {
    path.index_after("permutations")
}

/// Where the rules report.
struct Report<'a> {
    at: &'a JsonPath,
    d: &'a mut Diagnostics,
}

impl Report<'_> {
    /// The path of `rel` below the payload (a number after `permutations` is an index).
    fn path(&self, rel: &[&str]) -> JsonPath {
        let mut path = self.at.clone();
        for (i, key) in rel.iter().enumerate() {
            match key.parse::<usize>() {
                Ok(index) if i > 0 && rel[i - 1] == "permutations" => path.push_index(index),
                _ => path.push(key),
            }
        }
        path
    }

    /// A finding at `rel` below the payload.
    fn push(&mut self, code: DiagnosticCode, severity: Severity, rel: &[&str]) {
        let path = self.path(rel);
        self.d.push(
            Diagnostic::error(code)
                .severity(severity)
                .at(path)
                .phase(Phase::Validate),
        );
    }

    /// Keeps `value` as the unknown member `key` of the object at `rel` below the payload: a
    /// member the block reads and the game does not, written back as it was read.
    fn keep(&mut self, rel: &[&str], key: &str, value: Value) {
        let at = self.path(rel);
        self.d.unknown_mut().put(&at, key, value);
    }

    /// An error: the block is not loaded.
    fn error(&mut self, code: DiagnosticCode, rel: &[&str]) {
        self.push(code, Severity::Error, rel);
    }

    /// A warning.
    fn warn(&mut self, code: DiagnosticCode, rel: &[&str]) {
        self.push(code, Severity::Warning, rel);
    }
}

/// Vanilla's text with arguments.
#[allow(
    clippy::unnecessary_wraps,
    reason = "the codes' text fields are options"
)]
fn text(template: &'static str, args: &[&str]) -> Option<VanillaText> {
    Some(VanillaText::Formatted {
        template,
        args: args.iter().map(|a| (*a).to_owned()).collect(),
    })
}

/// Vanilla's fixed text.
#[allow(
    clippy::unnecessary_wraps,
    reason = "the codes' text fields are options"
)]
fn fixed(text: &'static str) -> Option<VanillaText> {
    Some(VanillaText::Fixed(text))
}

/// The section `None` (base) or `Some(i)` (permutation `i`) as a path prefix.
fn section_path(section: Option<usize>) -> Vec<String> {
    match section {
        None => vec!["components".to_owned()],
        Some(i) => vec![
            "permutations".to_owned(),
            i.to_string(),
            "components".to_owned(),
        ],
    }
}

/// `prefix` followed by `rest`, as the `&str`s `Report` takes.
fn rel<'a>(prefix: &'a [String], rest: &[&'a str]) -> Vec<&'a str> {
    prefix
        .iter()
        .map(String::as_str)
        .chain(rest.iter().copied())
        .collect()
}

/// The block's rules. Returns the permutations a rule skips.
fn rules(block: &mut Block, cx: &LoadContext, at: &JsonPath, d: &mut Diagnostics) -> Vec<usize> {
    let mut report = Report { at, d };
    let version = block.source_format_version;
    identifier(block, cx, &mut report);
    vanilla_data(block, cx, &mut report);
    menu_category(block, &mut report);
    states(block, &mut report);
    traits(block, cx, &mut report);
    engine_fields(block, cx, &mut report);
    empty_conditions(block, &mut report);
    let mut skipped = Vec::new();
    // Per-section component rules, base then permutations.
    component_rules(&mut block.components, None, version, cx, &mut report);
    if !experimental_components(&block.components, None, version, cx, &mut report) {
        // The base failed: an error is already reported.
    }
    for (index, permutation) in block.permutations.iter_mut().enumerate() {
        component_rules(
            &mut permutation.components,
            Some(index),
            version,
            cx,
            &mut report,
        );
        if !experimental_components(
            &permutation.components,
            Some(index),
            version,
            cx,
            &mut report,
        ) {
            skipped.push(index);
        }
    }
    creator_features(block, cx, &mut report);
    block_entity(block, &mut report);
    neighbor_change(block, &mut report);
    shared_values(block, &mut report);
    redstone_producer(block, &mut report);
    cross_components(block, &mut report);
    multi_block(block, &mut report);
    n_way_visual_rotation(block, &mut report);
    skipped
}

// ---- rule 1: the identifier --------------------------------------------------------------

/// The identifier is `ns:name` without whitespace and, outside the base game, not in the
/// `minecraft` namespace; a base-game block's is `minecraft:…`.
fn identifier(block: &Block, cx: &LoadContext, report: &mut Report<'_>) {
    let id = block.description.identifier.as_str();
    let (namespace, name) = split_namespace(id).unwrap_or(("", id));
    let shaped = !id.is_empty()
        && !id.chars().any(char::is_whitespace)
        && !namespace.is_empty()
        && !name.is_empty();
    let ok = if cx.is_base_game_pack {
        shaped && namespace == "minecraft"
    } else {
        shaped && namespace != "minecraft"
    };
    if !ok {
        report.error(
            DiagnosticCode::NamespaceRule {
                text: fixed(
                    "Must have a namespace and can't use the \"minecraft\" one. Example: \"namespace:my_block\"",
                ),
            },
            &["description", "identifier"],
        );
    }
}

/// `internal_vanilla_data` is base-game only; elsewhere it is an unknown member.
fn vanilla_data(block: &mut Block, cx: &LoadContext, report: &mut Report<'_>) {
    let cereal = block.source_format_version >= CEREAL;
    if cx.is_base_game_pack {
        // The game's description of this data allows one archetype at most and
        // `translucency` in `0..=1`; each is an error of the cereal document.
        let Some(data) = block
            .description
            .internal_vanilla_data
            .as_ref()
            .filter(|_| cereal)
        else {
            return;
        };
        if data.block_archetype.as_ref().is_some_and(|a| a.len() > 1) {
            // The game's archetype constraint logs this text.
            report.error(
                DiagnosticCode::InvalidValue {
                    text: fixed("\"block_archetype\" can only have a maximum of one component."),
                },
                &["description", "internal_vanilla_data", "block_archetype"],
            );
        }
        // One constraint for the check and its text.
        let translucency = NumericRange::inclusive(0.0, 1.0);
        if data
            .translucency
            .is_some_and(|t| !translucency.contains(f64::from(t.0)))
        {
            report.error(
                DiagnosticCode::InvalidValue {
                    text: constraint_text(translucency.code()),
                },
                &["description", "internal_vanilla_data", "translucency"],
            );
        }
        return;
    }
    if let Some(data) = block.description.internal_vanilla_data.take() {
        // The member is bound for base-game packs only: in a cereal document of another pack
        // the load fails. The legacy description reader takes its members by name and says
        // nothing of this one, so the crate reports it as an information there.
        report.push(
            DiagnosticCode::UnrecognizedField { text: None },
            if cereal {
                Severity::Error
            } else {
                Severity::Info
            },
            &["description", "internal_vanilla_data"],
        );
        if let Ok(json) = bedrock_addon_core::versioning::write_payload(&data) {
            report.keep(&["description"], "internal_vanilla_data", json);
        }
    }
}

/// A 1.26.20 block's `menu_category` needs its `category`; its `group` is a namespaced name
/// without whitespace (vanilla ones allowed). A violation fails the description: the block is
/// not loaded.
fn menu_category(block: &Block, report: &mut Report<'_>) {
    let Some(menu) = &block.description.menu_category else {
        return;
    };
    if block.source_format_version < CEREAL {
        return;
    }
    if menu.category.is_none() {
        report.error(
            DiagnosticCode::MissingRequiredField { text: None },
            &["description", "menu_category", "category"],
        );
    }
    if let Some(group) = &menu.group {
        let namespaced = has_namespace(group);
        if !namespaced || group.chars().any(char::is_whitespace) {
            report.error(
                DiagnosticCode::NamespaceRule { text: None },
                &["description", "menu_category", "group"],
            );
        }
    }
}

// ---- rule 3: states ------------------------------------------------------------------------

/// State names, value counts, duplicates, the bool-list rule and clashes with trait states. A
/// shape that is no state description is vanilla's "Failed to create Block State Description".
fn states(block: &Block, report: &mut Report<'_>) {
    let implied: Vec<&str> = block
        .description
        .traits
        .implied_states()
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    for (name, values) in &block.description.states {
        let rel = ["description", "states", name.as_str()];
        let (namespace, path) = split_namespace(name).unwrap_or(("", name));
        if name.chars().any(char::is_whitespace)
            || namespace.is_empty()
            || path.is_empty()
            || namespace == "minecraft"
        {
            report.error(DiagnosticCode::NamespaceRule { text: None }, &rel);
        }
        if implied.contains(&name.as_str()) {
            report.error(DiagnosticCode::InvalidValue { text: None }, &rel);
        }
        match values {
            StateValues::Invalid(_) => report.error(
                DiagnosticCode::InvalidType {
                    text: fixed("Failed to create Block State Description"),
                },
                &rel,
            ),
            other if !state_values_ok(other) => {
                report.error(DiagnosticCode::InvalidValue { text: None }, &rel);
            }
            _ => {}
        }
    }
}

/// The values of a state are 1 to 16 without duplicates, a boolean list exactly `[true,
/// false]`, a range `min <= max` of at most 16 values.
fn state_values_ok(values: &StateValues) -> bool {
    let unique = |list: &[StateValue]| list.iter().enumerate().all(|(i, v)| !list[..i].contains(v));
    match values {
        StateValues::Invalid(_) => false,
        StateValues::Bools(list) => list.len() == 2 && list[0] != list[1],
        StateValues::Range { min, max } => min <= max && i64::from(*max) - i64::from(*min) < 16,
        other => {
            let list = other.values();
            (1..=16).contains(&list.len()) && unique(&list)
        }
    }
}

// ---- rule 15: traits  -----------------------------------------------------

/// The toggle the trait gates name.
fn ucf_toggle() -> &'static str {
    Experiment::UpcomingCreatorFeatures
        .save_name()
        .unwrap_or_default()
}

/// Trait release gates, unknown traits, enabled states, `y_rotation_offset` and
/// `blocks_to_corner_with`.
#[allow(
    clippy::too_many_lines,
    reason = "one rule after another, in the order of the rule list"
)]
fn traits(block: &mut Block, cx: &LoadContext, report: &mut Report<'_>) {
    let version = block.source_format_version;
    let ucf = cx.experiments.contains(Experiment::UpcomingCreatorFeatures);
    let traits = &mut block.description.traits;
    let unknown: Vec<String> = report
        .d
        .unknown()
        .members_at(&report.path(&["description", "traits"]))
        .map(|(name, _)| name.to_owned())
        .collect();
    for name in &unknown {
        report.error(
            DiagnosticCode::InvalidEnumValue {
                text: text("Failed to find BlockTrait with identifier: '{}'", &[name]),
            },
            &["description", "traits", name],
        );
    }
    let gate = |report: &mut Report<'_>, name: &str, released: Option<FormatVersion>| {
        if released.is_some_and(|r| version >= r) || ucf {
            return;
        }
        let text = match released {
            Some(version) => VanillaText::TurnOnToggleBefore {
                what: "trait",
                name: name.to_owned(),
                version,
                toggle: ucf_toggle(),
            },
            None => VanillaText::TurnOnToggle {
                what: "trait",
                name: name.to_owned(),
                toggle: ucf_toggle(),
            },
        };
        report.error(
            DiagnosticCode::MissingToggles { text: Some(text) },
            &["description", "traits", name],
        );
    };
    let enabled = |report: &mut Report<'_>,
                   name: &str,
                   states: &[bedrock_addon_core::json::StrictString],
                   valid: &[&str]| {
        if states.is_empty() {
            report.error(
                DiagnosticCode::MissingRequiredField {
                    text: text(
                        "\"{}\" trait requires at least one 'enabled_state' to be specified.",
                        &[name],
                    ),
                },
                &["description", "traits", name, "enabled_states"],
            );
        }
        for state in states {
            if !valid.contains(&state.0.as_str()) {
                report.error(
                    DiagnosticCode::InvalidValue {
                        text: text(
                            "Invalid state option \"{}\" for \"{}\" trait.",
                            &[&state.0, name],
                        ),
                    },
                    &["description", "traits", name, "enabled_states"],
                );
            }
        }
    };
    if let Some(direction) = &mut traits.placement_direction {
        const NAME: &str = "minecraft:placement_direction";
        gate(report, NAME, Some(FormatVersion::new(1, 20, 20)));
        enabled(
            report,
            NAME,
            &direction.enabled_states,
            &[
                "minecraft:cardinal_direction",
                "minecraft:facing_direction",
                "minecraft:corner_and_cardinal_direction",
                "minecraft:sixteen_way_rotation",
            ],
        );
        let has = |s: &str| direction.enabled_states.iter().any(|e| e.0 == s);
        if let Some(offset) = direction.y_rotation_offset {
            let offset = offset.0;
            if !(has("minecraft:cardinal_direction")
                || has("minecraft:facing_direction")
                || has("minecraft:sixteen_way_rotation"))
            {
                report.error(
                    DiagnosticCode::InvalidValue {
                        text: fixed(
                            "\"y_rotation_offset\" can only be used when \"minecraft:cardinal_direction\", \"minecraft:facing_direction\", and/or \"minecraft:sixteen_way_rotation\" are enabled in \"enabled_states\".",
                        ),
                    },
                    &["description", "traits", NAME, "y_rotation_offset"],
                );
            } else if !(0.0..=360.0).contains(&offset) || offset % 90.0 != 0.0 {
                let snapped = ((offset / 90.0).round() * 90.0).clamp(0.0, 360.0);
                direction.y_rotation_offset = Some(bedrock_addon_core::json::StrictFloat(snapped));
                // The game logs this line at the content log's warning level and goes on with
                // the closest axis-aligned rotation: a warning.
                report.warn(
                    DiagnosticCode::ClampedValue {
                        text: fixed(
                            "\"minecraft:placement_direction\" trait \"y_rotation_offset\" must be a factor of 90.0. Defaulting to the nearest axis-aligned angle.",
                        ),
                    },
                    &["description", "traits", NAME, "y_rotation_offset"],
                );
            }
        }
        if !direction.blocks_to_corner_with().is_empty()
            && !has("minecraft:corner_and_cardinal_direction")
        {
            report.error(
                DiagnosticCode::InvalidValue {
                    text: fixed(
                        "\"blocks_to_corner_with\" can only be used when \"minecraft:corner_and_cardinal_direction\" is enabled in \"enabled_states\".",
                    ),
                },
                &["description", "traits", NAME, "blocks_to_corner_with"],
            );
        }
    }
    if let Some(position) = &traits.placement_position {
        const NAME: &str = "minecraft:placement_position";
        gate(report, NAME, Some(FormatVersion::new(1, 20, 20)));
        enabled(
            report,
            NAME,
            &position.enabled_states,
            &["minecraft:block_face", "minecraft:vertical_half"],
        );
    }
    if let Some(connection) = &traits.connection {
        const NAME: &str = "minecraft:connection";
        gate(report, NAME, Some(known::V1_26_0));
        // `vertical_connections` is bound for base-game packs only; elsewhere it is refused as
        // no state of the trait (the consequence is inferred from the game's constraint).
        if !cx.is_base_game_pack
            && connection
                .enabled_states
                .iter()
                .any(|s| s.0 == "minecraft:vertical_connections")
        {
            report.error(
                DiagnosticCode::InvalidValue {
                    text: text(
                        "Invalid state option \"{}\" for \"{}\" trait.",
                        &["minecraft:vertical_connections", NAME],
                    ),
                },
                &["description", "traits", NAME, "enabled_states"],
            );
        }
        enabled(
            report,
            NAME,
            &connection.enabled_states,
            &[
                "minecraft:cardinal_connections",
                "minecraft:vertical_connections",
            ],
        );
    }
    if let Some(multi) = &traits.multi_block {
        const NAME: &str = "minecraft:multi_block";
        // The game releases the trait at 1.26.40, but it registers a vertical 1.26.20 block
        // without the experiment, so the release gates the legacy path alone.
        if version < CEREAL {
            gate(report, NAME, Some(MULTI_BLOCK_RELEASED));
        }
        enabled(
            report,
            NAME,
            &multi.enabled_states,
            &["minecraft:multi_block_part"],
        );
        if !(2..=4).contains(&multi.parts()) {
            // The game binds `parts` with a numeric constraint of 2 to 4 under both error
            // masks, so a value outside it refuses the block.
            report.error(
                DiagnosticCode::InvalidValue { text: None },
                &["description", "traits", NAME, "parts"],
            );
        }
        match multi.direction.0.as_str() {
            // The horizontal gate is the Upcoming Creator Features validator's first check
            // (`creator_features`).
            "down" | "up" | "north" | "south" | "west" | "east" => {}
            _ => report.error(
                DiagnosticCode::InvalidValue { text: None },
                &["description", "traits", NAME, "direction"],
            ),
        }
    }
}

// ---- engine-gated members  --------------------------------------------------

/// The engine-gated trait states and description members (the components' own are in the
/// component table).
const SIXTEEN_WAY_ROTATION: &[FieldMeta] = &[FieldMeta {
    name: "",
    since_engine: known::V1_26_30,
}];
/// `minecraft:connection`'s `vertical_connections`.
const VERTICAL_CONNECTIONS: &[FieldMeta] = &[FieldMeta {
    name: "",
    since_engine: known::V1_26_50,
}];
/// `internal_vanilla_data.requires_correct_tool_for_drops`.
const VANILLA_DATA_FIELDS: &[FieldMeta] = &[FieldMeta {
    name: "requires_correct_tool_for_drops",
    since_engine: known::V1_26_50,
}];

/// `FieldSinceEngine` for every engine-gated member the block uses that is newer than its
/// pack's `min_engine_version`.
fn engine_fields(block: &Block, cx: &LoadContext, report: &mut Report<'_>) {
    let traits = &block.description.traits;
    let enables = |states: &[bedrock_addon_core::json::StrictString], name: &str| {
        states.iter().any(|s| s.0 == name)
    };
    if let Some(direction) = &traits.placement_direction
        && enables(&direction.enabled_states, "minecraft:sixteen_way_rotation")
    {
        let at = report.path(&[
            "description",
            "traits",
            "minecraft:placement_direction",
            "enabled_states",
        ]);
        check_fields_since_engine(SIXTEEN_WAY_ROTATION, |_| true, &at, cx, report.d);
    }
    if let Some(connection) = &traits.connection
        && enables(&connection.enabled_states, "minecraft:vertical_connections")
    {
        let at = report.path(&[
            "description",
            "traits",
            "minecraft:connection",
            "enabled_states",
        ]);
        check_fields_since_engine(VERTICAL_CONNECTIONS, |_| true, &at, cx, report.d);
    }
    if let Some(data) = &block.description.internal_vanilla_data {
        let at = report.path(&["description", "internal_vanilla_data"]);
        let present = data.requires_correct_tool_for_drops.is_some();
        check_fields_since_engine(VANILLA_DATA_FIELDS, |_| present, &at, cx, report.d);
    }
    let sections = std::iter::once((None, &block.components)).chain(
        block
            .permutations
            .iter()
            .enumerate()
            .map(|(i, p)| (Some(i), &p.components)),
    );
    for (section, map) in sections {
        let prefix = section_path(section);
        for entry in map.iter() {
            let ComponentRef::Typed { name, value, .. } = entry else {
                continue;
            };
            let Some(meta) = BlockFamily::components().get(name) else {
                continue;
            };
            if meta.fields.is_empty() {
                continue;
            }
            let json = value.write_component().unwrap_or(Value::NULL);
            let present = |field: &str| field.is_empty() || member(&json, field).is_some();
            let at = report.path(&rel(&prefix, &[name]));
            check_fields_since_engine(meta.fields, present, &at, cx, report.d);
        }
    }
}

/// The member at a dotted path.
fn member<'v>(json: &'v Value, dotted: &str) -> Option<&'v Value> {
    let mut value = json;
    for key in dotted.split('.') {
        value = value.as_object()?.get(key)?;
    }
    Some(value)
}

// ---- rule 19 (statically): empty conditions ------------------------------------------------

/// A 1.26.20 condition that is empty or at raw version −1 is no expression: not compiled, a
/// `DroppedField` finding, an information of the crate's own: the game is silent.
fn empty_conditions(block: &Block, report: &mut Report<'_>) {
    if block.source_format_version < CEREAL {
        return;
    }
    for (index, permutation) in block.permutations.iter().enumerate() {
        let Molang::Expr(source) = &permutation.condition else {
            continue;
        };
        let empty = source.is_empty()
            || matches!(
                source.form(),
                SourceForm::Object {
                    version: bedrock_addon_core::molang::RawVersion(-1)
                }
            );
        if empty {
            let i = index.to_string();
            // The empty node is not compiled and the game says nothing, so the crate reports
            // it as an information.
            report.push(
                DiagnosticCode::DroppedField { text: None },
                Severity::Info,
                &["permutations", &i, "condition"],
            );
        }
    }
}

// ---- per-component rules -------------------------------------------------------------------

/// Rules 7–10b and 13, the schema constraints (rule 9, documents from 1.26.20), the
/// format-version field gates, `destruction_particles`' format version, the base-game-only
/// members and the `snow_log_no_collision` spelling of the precipitation behaviour on one
/// section.
#[allow(
    clippy::too_many_lines,
    reason = "one rule after another, in the order of the rule list"
)]
fn component_rules(
    map: &mut ComponentMap<BlockComponent>,
    section: Option<usize>,
    version: FormatVersion,
    cx: &LoadContext,
    report: &mut Report<'_>,
) {
    let prefix = section_path(section);
    let cereal = version >= CEREAL;
    let mut dropped: Vec<&'static str> = Vec::new();
    // In a cereal document each of these rules is a Cereal constraint of the component's
    // description (an error under both masks: the block is not loaded), except
    // `minecraft:tags` (which has no constraint) and the format version of
    // `destruction_particles` (the component registry's). A legacy document's descriptions
    // bind most of them differently, as below.
    let mut drop = |report: &mut Report<'_>,
                    name: &'static str,
                    message: Option<VanillaText>,
                    field: &[&str]| {
        let mut path = rel(&prefix, &[name]);
        path.extend_from_slice(field);
        // A legacy document's descriptions bind the same size constraints for these four: a
        // failed constraint fails the description's load, which refuses the block.
        let constraint = if cereal {
            !matches!(name, "minecraft:tags" | "minecraft:destruction_particles")
        } else {
            matches!(
                name,
                "minecraft:crafting_table"
                    | "minecraft:liquid_detection"
                    | "minecraft:placement_filter"
                    | "minecraft:geometry"
                    | "minecraft:friction"
                    | "minecraft:light_emission"
            )
        };
        // The game's `destruction_particles` check logs the format-version line and fails the
        // component, which refuses the block. `minecraft:tags` has no constraint, and a legacy
        // document's descriptions bind none of the tick order, detailed geometry or single
        // bounding box constraints that exist only from 1.26.20: the drop is a lint of the
        // crate's own, an information. The legacy descriptions of crafting table, liquid
        // detection, placement filter and geometry do bind their size constraints, with the
        // values of the 1.26.20 components, so their drop refuses the block too, as do friction
        // and light emission, whose legacy descriptions bind the same ranges; the legacy
        // collision box and selection box bind none, the legacy tick's order check is a kept
        // error (below), and a legacy `uv_lock` list has no size constraint.
        let severity = if constraint || name == "minecraft:destruction_particles" {
            Severity::Error
        } else {
            Severity::Info
        };
        report.push(
            DiagnosticCode::InvalidValue { text: message },
            severity,
            &path,
        );
        dropped.push(name);
    };
    // Friction and light emission: bound by the legacy descriptions too, so checked in every
    // format. What the legacy reader holds here is what its non-strict read gave (`true` is
    // 1.0, `-1` is 255), held to the same range.
    if let Some(bound) = map.get::<Friction>().and_then(out_of_bounds) {
        drop(report, "minecraft:friction", constraint_text(bound), &[]);
    }
    if let Some(bound) = map.get::<LightEmission>().and_then(out_of_bounds) {
        // The legacy description binds the brightness to a closed `[0, 15]` range, the
        // 1.26.20 component to `max(15)` (the component's bound).
        let bound = if cereal {
            bound
        } else {
            NumericRange::inclusive(0.0, 15.0).code()
        };
        drop(
            report,
            "minecraft:light_emission",
            constraint_text(bound),
            &[],
        );
    }
    // Rule 9 and the schema constraints (cereal documents; a step clamps older values).
    if cereal {
        if let Some(bound) = map.get::<LightDampening>().and_then(out_of_bounds) {
            drop(
                report,
                "minecraft:light_dampening",
                constraint_text(bound),
                &[],
            );
        }
        if let Some(bound) = map.get::<RedstoneProducer>().and_then(out_of_bounds) {
            drop(
                report,
                "minecraft:redstone_producer",
                constraint_text(bound),
                &["power"],
            );
        }
        if let Some(bound) = map.get::<RedstoneConsumer>().and_then(out_of_bounds) {
            drop(
                report,
                "minecraft:redstone_consumer",
                constraint_text(bound),
                &["min_power"],
            );
        }
        if let Some(axis) = map.get::<Transformation>().and_then(|t| {
            t.rotation
                .and_then(|r| r.into_iter().find(|axis| axis % 90.0 != 0.0))
        }) {
            let text = constraint_text(cereal::not_multiple_of(f64::from(axis), 90.0));
            drop(report, "minecraft:transformation", text, &["rotation"]);
        }
        if let Some(DestructibleByExplosion::Params(p)) = map.get::<DestructibleByExplosion>()
            && let Some(bound) = out_of_bounds(p)
        {
            drop(
                report,
                "minecraft:destructible_by_explosion",
                constraint_text(bound),
                &["explosion_resistance"],
            );
        }
        if let Some(DestructibleByMining::Params(p)) = map.get::<DestructibleByMining>()
            && let Some(bound) = out_of_bounds(p)
        {
            drop(
                report,
                "minecraft:destructible_by_mining",
                constraint_text(bound),
                &["seconds_to_destroy"],
            );
        }
        if map.get::<ConnectionRule>().is_some_and(|c| {
            c.enabled_directions
                .as_ref()
                .is_some_and(|d| d.iter().enumerate().any(|(i, v)| d[..i].contains(v)))
        }) {
            drop(
                report,
                "minecraft:connection_rule",
                constraint_text(cereal::no_duplicates()),
                &["enabled_directions"],
            );
        }
        if let Some(sounds) = map.get::<InstrumentSound>() {
            // The game's map constraint: at least one entry, the keys `^(up|down)$`.
            let text = if sounds.0.is_empty() {
                Some(cereal::object_too_small(1))
            } else if sounds.0.keys().any(|k| k != "up" && k != "down") {
                Some(cereal::string_mismatch("^(up|down)$"))
            } else {
                None
            };
            if let Some(code) = text {
                drop(
                    report,
                    "minecraft:instrument_sound",
                    constraint_text(code),
                    &[],
                );
            }
        }
        if map
            .get::<Tags>()
            .is_some_and(|t| t.0.is_empty() || t.0.iter().any(String::is_empty))
        {
            drop(report, "minecraft:tags", None, &[]);
        }
        if let Some(Geometry::Params(params)) = map.get::<Geometry>() {
            if params
                .culling_shape
                .as_ref()
                .is_some_and(|s| !culling_shape_ok(s))
            {
                let text = constraint_text(cereal::string_mismatch(CULLING_SHAPE_REGEX));
                drop(report, "minecraft:geometry", text, &["culling_shape"]);
            } else if params
                .culling_layer
                .as_ref()
                .is_some_and(|l| !culling_layer_ok(l))
            {
                let text = constraint_text(cereal::string_mismatch(CULLING_LAYER_REGEX));
                drop(report, "minecraft:geometry", text, &["culling_layer"]);
            }
        }
        if map.get::<MaterialInstances>().is_some_and(|m| {
            m.0.values().any(|e| {
                matches!(e, MaterialInstanceEntry::Instance(i)
                    if i.ambient_occlusion.is_some_and(|a| !(0.0..=10.0).contains(&a)))
            })
        }) {
            let text = constraint_text(NumericRange::inclusive(0.0, 10.0).code());
            drop(report, "minecraft:material_instances", text, &[]);
        }
    }
    // Rule 7.
    // The game's crafting table description: the tag list holds at most 64 entries, each tag
    // at most 64 characters, and `table_name` at most 256 (the localisation-id constraint).
    if let Some(table) = map.get::<CraftingTable>() {
        // The game's string constraint counts bytes.
        let long_tag = table.crafting_tags.iter().position(|t| t.len() > 64);
        if table.crafting_tags.len() > 64 {
            drop(
                report,
                "minecraft:crafting_table",
                constraint_text(cereal::too_many_elements(64)),
                &["crafting_tags"],
            );
        } else if let Some(index) = long_tag {
            let index = index.to_string();
            drop(
                report,
                "minecraft:crafting_table",
                constraint_text(cereal::too_many_characters(64)),
                &["crafting_tags", index.as_str()],
            );
        } else if table
            .table_name
            .as_ref()
            .is_some_and(|name| name.len() > 256)
        {
            drop(
                report,
                "minecraft:crafting_table",
                constraint_text(cereal::too_many_characters(256)),
                &["table_name"],
            );
        }
    }
    // Rule 8.
    if let Some(rules) = map
        .get::<LiquidDetection>()
        .map(|l| l.detection_rules.len())
        .filter(|n| *n != 1)
    {
        // The game's sequence constraint: exactly one rule.
        let text = if rules == 0 {
            cereal::array_too_small(1)
        } else {
            cereal::too_many_elements(1)
        };
        drop(
            report,
            "minecraft:liquid_detection",
            constraint_text(text),
            &["detection_rules"],
        );
    }
    // Rules 10 and 10a: the collision boxes. The constraints of 1.26.20 and 1.26.60 are the
    // readers' (an error that fails the block); of a block that is no such
    // document the 1.26.20 one is checked here, as its text is written at 1.26.20 (what
    // `refused_by_newest` asks), and a legacy block keeps more than 16 boxes from nothing but
    // a dropped component.
    if let Some(CollisionBox::Boxes(boxes)) = map.get::<CollisionBox>() {
        if !cereal && boxes.len() > 16 {
            drop(report, "minecraft:collision_box", None, &[]);
        } else if cereal
            && version < NEWEST
            && let Some(mut finding) = legacy_constraint(
                boxes,
                false,
                &report.path(&rel(&prefix, &["minecraft:collision_box"])),
            )
        {
            finding.phase = Phase::Validate;
            report.d.push(finding);
        }
    }
    // A legacy document's selection box is tested as a unit-scaled origin and then origin plus
    // size against `[0, 1]` (the x and z shifted by 8). The game logs each failure at the
    // content log's error level, then stores the box as read: two kept errors, the component
    // stays.
    if !cereal && let Some(SelectionBox::Box(single)) = map.get::<SelectionBox>() {
        let (origin, size) = (single.origin(), single.size());
        let min = [origin[0] + 8.0, origin[1], origin[2] + 8.0];
        let max = [min[0] + size[0], min[1] + size[1], min[2] + size[2]];
        let unit = |v: [f32; 3]| v.iter().all(|c| (0.0..=1.0).contains(&(c * 0.0625)));
        for (ok, message) in [
            (
                unit(min),
                "minecraft:selection_box: origin can't be below (-8, 0, -8) and can't be more than (8, 16, 8)",
            ),
            (
                unit(max),
                "minecraft:selection_box: origin with added size can't be below (-8, 0, -8) and can't be more than (8, 16, 8)",
            ),
        ] {
            if !ok {
                let path = report.path(&rel(&prefix, &["minecraft:selection_box"]));
                report.d.push(
                    Diagnostic::error(DiagnosticCode::InvalidValue {
                        text: fixed(message),
                    })
                    .at(path)
                    .phase(Phase::Validate)
                    .into_kept(),
                );
            }
        }
    }
    if cereal
        && let Some(SelectionBox::Box(single)) = map.get::<SelectionBox>()
        && !inside(single, 16.0)
    {
        drop(
            report,
            "minecraft:selection_box",
            text(
                "\"origin\" + \"size\", must be inside the following limit: {} and {}",
                &["(0, 0, 0)", "(16, 16, 16)"],
            ),
            &[],
        );
    }
    // Rule 10b. A legacy document's tick description logs "%s: maximum ticks in interval range
    // must be more than or equal to minimum ticks" at the content log's error level and stores
    // both ends as read: a kept error, the component stays. `%s` is the component's name,
    // "minecraft:tick".
    if !cereal
        && map
            .get::<Tick>()
            .is_some_and(|t| t.interval_range[0] > t.interval_range[1])
    {
        let path = report.path(&rel(&prefix, &["minecraft:tick", "interval_range"]));
        report.d.push(
            Diagnostic::error(DiagnosticCode::InvalidValue {
                    text: text(
                        "{}: maximum ticks in interval range must be more than or equal to minimum ticks",
                        &["minecraft:tick"],
                    ),
                }).at(path).phase(Phase::Validate)
            .into_kept(),
        );
    } else if cereal
        && map
            .get::<Tick>()
            .is_some_and(|t| t.interval_range[0] > t.interval_range[1])
    {
        drop(
            report,
            "minecraft:tick",
            text(
                "Invalid interval_range: {}",
                &["The first value must be equal or lower than the second value"],
            ),
            &["interval_range"],
        );
    }
    // Rule 10: the geometry, material instance and placement filter limits. A legacy
    // document's geometry binds `bone_visibility` to at most 64 entries and puts no size
    // constraint on `uv_lock` (the 1.26.20 component has one), and refuses `bone_visibility`
    // beside `minecraft:geometry.cross`, `.full_block` or `.full_block_v1` with "invalid
    // bones: cannot use bone visibility feature with: <identifier>".
    if let Some(Geometry::Params(params)) = map.get::<Geometry>() {
        let vanilla = matches!(
            params.identifier.as_str(),
            "minecraft:geometry.full_block" | "minecraft:geometry.cross"
        ) || (!cereal && params.identifier == "minecraft:geometry.full_block_v1");
        if params.bone_visibility_len() > 64 {
            let text = constraint_text(cereal::too_many_map_elements());
            drop(report, "minecraft:geometry", text, &["bone_visibility"]);
        } else if matches!(&params.uv_lock, Some(UvLock::Bones(bones)) if bones.len() > 64) {
            if cereal {
                let text = constraint_text(cereal::too_many_elements(64));
                drop(report, "minecraft:geometry", text, &["uv_lock"]);
            } else {
                // The game keeps the component in silence, so the crate reports an information.
                report.push(
                    DiagnosticCode::InvalidValue {
                        text: constraint_text(cereal::too_many_elements(64)),
                    },
                    Severity::Info,
                    &rel(&prefix, &["minecraft:geometry", "uv_lock"]),
                );
            }
        } else if vanilla && params.bone_visibility_len() > 0 {
            let message = if cereal {
                fixed(
                    "Cannot use bone visibility feature with minecraft:geometry.cross or minecraft:geometry.full_block",
                )
            } else {
                text(
                    "invalid bones: cannot use bone visibility feature with: {}",
                    &[params.identifier.as_str()],
                )
            };
            drop(report, "minecraft:geometry", message, &["bone_visibility"]);
        }
    }
    if let Some(instances) = map.get::<MaterialInstances>() {
        // The map holds at most 64 entries, and its keys match a regex.
        let text = if instances.0.len() > 64 {
            Some(cereal::too_many_map_elements())
        } else if instances.0.keys().any(|k| !instance_key(k)) {
            Some(cereal::string_mismatch("^[a-zA-Z0-9_.:*]+$"))
        } else {
            None
        };
        if let Some(code) = text {
            drop(
                report,
                "minecraft:material_instances",
                constraint_text(code),
                &[],
            );
        }
    }
    if map.get::<PlacementFilter>().is_some_and(|f| {
        f.conditions().len() > 64 || f.conditions().iter().any(|c| c.block_filter().len() > 64)
    }) {
        // Both lists hold at most 64 entries.
        let text = constraint_text(cereal::too_many_elements(64));
        drop(report, "minecraft:placement_filter", text, &[]);
    }
    // destruction_particles needs format 1.21.70.
    if version < FormatVersion::new(1, 21, 70) && map.get::<DestructionParticles>().is_some() {
        drop(
            report,
            "minecraft:destruction_particles",
            text(
                "Format version {} is at least required to use {}",
                &["1.21.70", "minecraft:destruction_particles"],
            ),
            &[],
        );
    }
    for name in dropped {
        map.remove_named(name);
    }
    custom_parameters(map, &prefix, report);
    base_game_members(map, &prefix, cereal, cx, report);
    field_gates(map, &prefix, version, report);
    random_offset(map, &prefix, version, report);
    // The game reads `snow_log_no_collision` as `snowlogging`.
    if let Some(precipitation) = map.get_mut::<PrecipitationInteractions>()
        && precipitation.precipitation_behavior == PrecipitationBehavior::SnowLogNoCollision
    {
        precipitation.precipitation_behavior = PrecipitationBehavior::Snowlogging;
    }
}

/// The text of one of `diagnostic::cereal`'s `InvalidValue` codes.
fn constraint_text(code: DiagnosticCode) -> Option<VanillaText> {
    match code {
        DiagnosticCode::InvalidValue { text } => text,
        // The multiple-of definition held its template and arguments apart.
        code if code.is(&bedrock_addon_core::diagnostic::codes::NOT_MULTIPLE_OF) => code
            .args_of(&bedrock_addon_core::diagnostic::codes::NOT_MULTIPLE_OF)
            .and_then(|args| match args {
                [value, step] => Some(VanillaText::Formatted {
                    template: "{} is not multiple of {}",
                    args: vec![value.clone(), step.clone()],
                }),
                _ => None,
            }),
        // By legacy name: the definitions that were `InvalidValue`, by their game sentence.
        code if code.name() == "InvalidValue" => {
            code.vanilla().map(|line| VanillaText::Formatted {
                template: "{}",
                args: vec![line],
            })
        }
        _ => None,
    }
}

/// The finding of the bounds `T` states (`schema::min`, `schema::max`) on `value`, worded with
/// the game's range text; `None` when every member is inside its bounds. The rule that calls
/// it says where the finding goes, how severe it is and what a failed bound drops.
fn out_of_bounds<T: Schema + Clone>(value: &T) -> Option<DiagnosticCode> {
    let mut found = Diagnostics::default();
    let mut value = value.clone();
    {
        let mut cx =
            SchemaCx::new(Reading::DOCUMENT, JsonPath::new(), &mut found).with_range_text(true);
        value.check(&mut cx);
    }
    found.iter().next().map(|finding| finding.code.clone())
}

/// The `culling_shape` regex the game's constraint uses.
const CULLING_SHAPE_REGEX: &str =
    "^(?:minecraft:empty|minecraft:unit_cube|(?!minecraft:)[a-z0-9].*[a-z0-9])$";

/// The `culling_layer` regex the game's constraint uses.
const CULLING_LAYER_REGEX: &str = "^(?:minecraft:culling_layer.undefined|minecraft:culling_layer.leaves|(?!minecraft:)[a-z0-9].*[a-z0-9])$";

/// Custom-component parameters nest at most 2 deep (rule 10). The game's loader does not apply
/// its depth constraint to the component group, so the game
/// keeps the component in silence: the crate reports an information and drops nothing.
fn custom_parameters(
    map: &mut ComponentMap<BlockComponent>,
    prefix: &[String],
    report: &mut Report<'_>,
) {
    let deep: Vec<String> = map
        .iter()
        .filter_map(|entry| match entry {
            ComponentRef::Custom(custom) if depth(&custom.parameters) > 2 => {
                Some(custom.name.clone())
            }
            _ => None,
        })
        .collect();
    for name in deep {
        report.push(
            DiagnosticCode::InvalidValue { text: None },
            Severity::Info,
            &rel(prefix, &[&name]),
        );
    }
}

/// `material_instances.*.dithering` and `connection_rule.vanilla_connections_from` are read
/// only in base-game packs; elsewhere they are unknown members: in a cereal document an error
/// that refuses the block (as for `internal_vanilla_data`), in a legacy one silent (only the
/// 1.26.20 descriptions bind them), so the crate reports an information.
fn base_game_members(
    map: &mut ComponentMap<BlockComponent>,
    prefix: &[String],
    cereal: bool,
    cx: &LoadContext,
    report: &mut Report<'_>,
) {
    if cx.is_base_game_pack {
        return;
    }
    let severity = if cereal {
        Severity::Error
    } else {
        Severity::Info
    };
    if let Some(instances) = map.get_mut::<MaterialInstances>() {
        for (key, entry) in &mut instances.0 {
            if let MaterialInstanceEntry::Instance(instance) = entry
                && let Some(dithering) = instance.dithering.take()
            {
                report.keep(
                    &rel(prefix, &["minecraft:material_instances", key]),
                    "dithering",
                    Value::from(dithering),
                );
                report.push(
                    DiagnosticCode::UnrecognizedField { text: None },
                    severity,
                    &rel(prefix, &["minecraft:material_instances", key, "dithering"]),
                );
            }
        }
    }
    if let Some(rule) = map.get_mut::<ConnectionRule>()
        && let Some(from) = rule.vanilla_connections_from.take()
    {
        if let Ok(json) = bedrock_addon_core::versioning::write_payload(&from) {
            report.keep(
                &rel(prefix, &["minecraft:connection_rule"]),
                "vanilla_connections_from",
                json,
            );
        }
        report.push(
            DiagnosticCode::UnrecognizedField { text: None },
            severity,
            &rel(
                prefix,
                &["minecraft:connection_rule", "vanilla_connections_from"],
            ),
        );
    }
}

/// `^(?:minecraft:empty|minecraft:unit_cube|(?!minecraft:)[a-z0-9].*[a-z0-9])$`, the voxel
/// shape name check.
fn culling_shape_ok(shape: &str) -> bool {
    if shape == "minecraft:empty" || shape == "minecraft:unit_cube" {
        return true;
    }
    let edge = |c: Option<char>| c.is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    !shape.starts_with("minecraft:")
        && shape.chars().count() >= 2
        && edge(shape.chars().next())
        && edge(shape.chars().last())
}

/// [`CULLING_LAYER_REGEX`], as the game's `char` regex engine reads it: on bytes, with
/// ECMAScript's `.` any byte but `\n` and `\r`:
/// `minecraft:culling_layer`, one such byte, and `undefined` or `leaves`; or a value that does
/// not start with `minecraft:`, at least two bytes, whose first and last bytes are a lower-case
/// ASCII letter or digit and none between is `\n` or `\r`.
fn culling_layer_ok(layer: &str) -> bool {
    let bytes = layer.as_bytes();
    let dot = |b: u8| b != b'\n' && b != b'\r';
    let edge = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    let vanilla = bytes
        .strip_prefix(b"minecraft:culling_layer")
        .and_then(|rest| rest.split_first())
        .is_some_and(|(first, name)| dot(*first) && (name == b"undefined" || name == b"leaves"));
    vanilla
        || (!layer.starts_with("minecraft:")
            && bytes.len() >= 2
            && edge(bytes[0])
            && edge(bytes[bytes.len() - 1])
            && bytes[1..bytes.len() - 1].iter().all(|b| dot(*b)))
}

/// How deep a JSON value nests: 0 for a scalar, 1 more than its deepest member for a container.
fn depth(value: &Value) -> usize {
    if let Some(array) = value.as_array() {
        return 1 + array.iter().map(depth).max().unwrap_or(0);
    }
    if let Some(object) = value.as_object() {
        return 1 + object.values().map(depth).max().unwrap_or(0);
    }
    0
}

/// A box, x and z shifted by 8, inside `(0, 0, 0)–(16, height, 16)`.
fn inside(b: &BoundingBox, height: f32) -> bool {
    let (origin, size) = (b.origin(), b.size());
    let min = [origin[0] + 8.0, origin[1], origin[2] + 8.0];
    let max = [min[0] + size[0], min[1] + size[1], min[2] + size[2]];
    let limit = [16.0, height, 16.0];
    (0..3).all(|i| min[i] >= 0.0 && max[i] <= limit[i] && min[i] <= limit[i] && max[i] >= 0.0)
}

/// `^[a-zA-Z0-9_.:*]+$`.
fn instance_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | ':' | '*'))
}

/// Fields gated by the format version. Each gate is a legacy description's post-parse check
/// that logs at the content log's error level and fails the component's load, which refuses
/// the block: `uv_lock` of the geometry, `isotropic` and the `alpha_masked_tint` rules of the
/// material instances, and `particle_count` of the destruction particles. From 1.26.20 the
/// `alpha_masked_tint` texts are the cereal constraint's, the same refusal.
fn field_gates(
    map: &mut ComponentMap<BlockComponent>,
    prefix: &[String],
    version: FormatVersion,
    report: &mut Report<'_>,
) {
    let cereal = version >= CEREAL;
    let constraint = |report: &mut Report<'_>, message: Option<VanillaText>, path: &[&str]| {
        report.error(
            DiagnosticCode::InvalidValue { text: message },
            &rel(prefix, path),
        );
    };
    if version < known::V1_21_90
        && let Some(Geometry::Params(params)) = map.get_mut::<Geometry>()
        && params.uv_lock.take().is_some()
    {
        constraint(
            report,
            fixed(
                "Cannot use field \"uv_lock\" before 'format_version' 1.21.90 when 'minecraft:transformation' component was released.",
            ),
            &["minecraft:geometry", "uv_lock"],
        );
    }
    if let Some(instances) = map.get_mut::<MaterialInstances>() {
        for (key, entry) in &mut instances.0 {
            let MaterialInstanceEntry::Instance(instance) = entry else {
                continue;
            };
            if version < FormatVersion::new(1, 21, 70) && instance.isotropic.take().is_some() {
                constraint(
                    report,
                    fixed(
                        "'minecraft:material_instances' cannot use 'isotropic' member with 'format_version' < 1.21.70",
                    ),
                    &["minecraft:material_instances", key, "isotropic"],
                );
            }
            if instance.alpha_masked_tint == Some(true) {
                let tint = instance.tint_method.unwrap_or_default() != TintMethod::None;
                let opaque = instance.render_method.unwrap_or_default() == RenderMethod::Opaque;
                let message = if version < known::V1_21_120 {
                    Some(
                        "'alpha_masked_tint' is only supported at format_version 1.21.120 or higher",
                    )
                } else if !tint {
                    Some(if cereal {
                        "\"alpha_masked_tint\" requires a non-default \"tint_method\"."
                    } else {
                        "'alpha_masked_tint' requires a non-default 'tint_method'"
                    })
                } else if !opaque {
                    Some(if cereal {
                        "\"alpha_masked_tint\" requires \"render_method\" to be \"opaque\"."
                    } else {
                        "'alpha_masked_tint' requires 'render_method' to be 'opaque'"
                    })
                } else {
                    None
                };
                if let Some(message) = message {
                    instance.alpha_masked_tint = None;
                    constraint(
                        report,
                        fixed(message),
                        &["minecraft:material_instances", key, "alpha_masked_tint"],
                    );
                }
            }
        }
    }
    // The game's check never reads the experiments it is handed: the gate is the version
    // alone, and the text's "experimental creator features" is a leftover. A 1.21.90 document
    // keeps `particle_count` without the toggle.
    if let Some(particles) = map.get_mut::<DestructionParticles>()
        && particles.particle_count.is_some()
        && version < known::V1_21_90
    {
        particles.particle_count = None;
        constraint(
            report,
            text(
                "Format version {} is at least required to use {} and experimental creator features",
                &["1.21.90", "particle_count"],
            ),
            &["minecraft:destruction_particles", "particle_count"],
        );
    }
}

/// Rule 13: the offset keeps the collision box inside the cell. Below 1.26.20 each bound that
/// moves is clamped with a warning; from 1.26.20 the component is dropped.
fn random_offset(
    map: &mut ComponentMap<BlockComponent>,
    prefix: &[String],
    version: FormatVersion,
    report: &mut Report<'_>,
) {
    let boxes = match map.get::<CollisionBox>() {
        None => vec![CollisionBoundingBox::default()],
        Some(collision) => collision.boxes(),
    };
    if boxes.is_empty() {
        return;
    }
    // The boxes as 1.26.20 reads them (a box of 1.26.60 by its `origin`/`size` reading, the
    // inverse of the 1.26.20 step; that the 1.26.60 constraint measures the same way is
    // inferred).
    let boxes: Vec<(Vec3, Vec3)> = boxes
        .iter()
        .map(CollisionBoundingBox::legacy_origin_size)
        .collect();
    // The cell in sixteenths: x and z from −8 to 8, y from 0 to 16.
    let lower = |axis: usize| {
        boxes
            .iter()
            .map(|(origin, _)| origin[axis])
            .fold(f32::INFINITY, f32::min)
    };
    let upper = |axis: usize| {
        boxes
            .iter()
            .map(|(origin, size)| origin[axis] + size[axis].max(0.0))
            .fold(f32::NEG_INFINITY, f32::max)
    };
    let bounds = [(-8.0f32, 8.0f32), (0.0, 16.0), (-8.0, 8.0)];
    let Some(offset) = map.get_mut::<RandomOffset>() else {
        return;
    };
    let base = rel(prefix, &["minecraft:random_offset"]);
    let mut failed = false;
    for (axis, name, slot) in [
        (0usize, "x", &mut offset.x),
        (1, "y", &mut offset.y),
        (2, "z", &mut offset.z),
    ] {
        let Some(slot) = slot else { continue };
        let (cell_min, cell_max) = bounds[axis];
        let min_allowed = cell_min - lower(axis);
        let max_allowed = cell_max - upper(axis);
        for (bound, value, allowed, below) in [
            ("min", &mut slot.range.min, min_allowed, true),
            ("max", &mut slot.range.max, max_allowed, false),
        ] {
            let out = if below {
                *value < allowed
            } else {
                *value > allowed
            };
            if !out {
                continue;
            }
            let mut path = base.clone();
            path.extend([name, "range", bound]);
            if version >= CEREAL {
                failed = true;
                // The game binds this as a constraint of the component.
                report.error(
                    DiagnosticCode::InvalidValue {
                        text: text(
                            "\"random_offset pushes \"collision_box\" outside of unit cube for \"{}\" \"range\" \"{}\" of {}",
                            &[name, bound, &value.to_string()],
                        ),
                    },
                    &path,
                );
            } else {
                // `%f`: six decimals, as the game prints them. The game logs each bound at the
                // content log's warning level and clamps it: a warning.
                let from = format!("{:.6}", *value);
                *value = allowed;
                report.warn(
                    DiagnosticCode::ClampedValue {
                        text: text(
                            "Random offset pushes collision box outside of cube for '{}->{}'. Restricting value from {} to {}",
                            &[name, bound, &from, &format!("{allowed:.6}")],
                        ),
                    },
                    &path,
                );
            }
        }
    }
    if failed {
        map.remove_named("minecraft:random_offset");
    }
}

// ---- components behind Upcoming Creator Features with their own texts  ----

/// The legacy rows of a document below 1.26.20 the family gates with the registry's text:
/// `sound` (released 1.26.40) and `vibration_properties` (released 1.26.60), each behind
/// Upcoming Creator Features. An error, in the base and in a permutation alike: the block is
/// not loaded. Returns `false` when the section fails.
fn experimental_components(
    map: &ComponentMap<BlockComponent>,
    section: Option<usize>,
    version: FormatVersion,
    cx: &LoadContext,
    report: &mut Report<'_>,
) -> bool {
    if cx.experiments.contains(Experiment::UpcomingCreatorFeatures) {
        return true;
    }
    let prefix = section_path(section);
    // The game's prerelease component check fails the component load, and a permutation's
    // failure fails the block's load with it: the whole block is refused, in the base and in
    // a permutation alike.
    let severity = Severity::Error;
    let mut passed = true;
    // The legacy row of `sound` (released 1.26.40 behind Upcoming Creator Features): a
    // document below 1.26.20; the validator of 1.26.60 does not check `block_entity`, `sound`
    // or `instrument_sound`.
    if version < CEREAL && has(map, "minecraft:sound") {
        passed = false;
        report.push(
            DiagnosticCode::MissingToggles {
                text: Some(VanillaText::TurnOnToggleBefore {
                    what: "component",
                    name: "minecraft:sound".to_owned(),
                    version: SOUND_RELEASED,
                    toggle: ucf_toggle(),
                }),
            },
            severity,
            &rel(&prefix, &["minecraft:sound"]),
        );
    }
    // The legacy row of `vibration_properties` (released 1.26.60 behind Upcoming Creator
    // Features): a document below 1.26.20.
    if version < CEREAL && has(map, "minecraft:vibration_properties") {
        passed = false;
        report.push(
            DiagnosticCode::MissingToggles {
                text: Some(VanillaText::TurnOnToggleBefore {
                    what: "component",
                    name: "minecraft:vibration_properties".to_owned(),
                    version: NEWEST,
                    toggle: ucf_toggle(),
                }),
            },
            severity,
            &rel(&prefix, &["minecraft:vibration_properties"]),
        );
    }
    passed
}

/// The game's Upcoming Creator Features validator, which runs on every document, after its
/// upgrade, when Upcoming Creator Features are off: each check visits the base components, then each permutation,
/// and the validator stops at its first error, which fails the
/// block.
fn creator_features(block: &Block, cx: &LoadContext, report: &mut Report<'_>) {
    if cx.experiments.contains(Experiment::UpcomingCreatorFeatures) {
        return;
    }
    let version = block.source_format_version;
    // The first check: a horizontal multi-block below `HORIZONTAL_MULTI_BLOCK_RELEASE`.
    // Below 1.26.20 the legacy read's trait gate (`traits`) has refused the trait without the
    // experiment, and the validator never sees the block: one diagnostic.
    if version >= CEREAL
        && version < HORIZONTAL_MULTI_BLOCK_RELEASE
        && block
            .description
            .traits
            .multi_block
            .as_ref()
            .is_some_and(|m| matches!(m.direction.0.as_str(), "north" | "south" | "west" | "east"))
    {
        report.error(
            DiagnosticCode::MissingToggles {
                text: fixed(
                    "Upcoming Creator Features experiment is required for trait 'minecraft:multi_block' for horizontal directions",
                ),
            },
            &["description", "traits", "minecraft:multi_block", "direction"],
        );
        return;
    }
    let sections = std::iter::once((None, &block.components)).chain(
        block
            .permutations
            .iter()
            .enumerate()
            .map(|(i, p)| (Some(i), &p.components)),
    );
    let sections: Vec<_> = sections.collect();
    // `block_entity`'s `container` and `entity_storage`, in the base (a rejected key in
    // permutations).
    if let Some(entity) = block.components.get::<BlockEntity>() {
        for (member, set) in [
            ("container", entity.container.is_some()),
            ("entity_storage", entity.entity_storage()),
        ] {
            if set {
                report.error(
                    DiagnosticCode::MissingToggles {
                        text: text(
                            "To use '{}' in '{}', experimental creator features are required",
                            &[member, "minecraft:block_entity"],
                        ),
                    },
                    &["components", "minecraft:block_entity", member],
                );
                return;
            }
        }
    }
    // Each of these components, whatever its value. A document below 1.26.20 has none of them typed (`vibration_properties` is gated there
    // by its legacy row, `experimental_components`).
    for name in [
        "minecraft:enchantment_power_transmitter",
        "minecraft:vibration_properties",
        "minecraft:neighbor_change",
    ] {
        if version < CEREAL {
            continue;
        }
        for (section, map) in &sections {
            if has(map, name) {
                let prefix = section_path(*section);
                report.error(
                    DiagnosticCode::MissingToggles {
                        text: text(
                            "To use '{}', experimental creator features are required",
                            &[name],
                        ),
                    },
                    &rel(&prefix, &[name]),
                );
                return;
            }
        }
    }
    for (section, map) in sections {
        // `min`/`max` in a box of a document of 1.26.60 or later (an upgraded box is no box of
        // such a document).
        if version >= NEWEST
            && let Some(CollisionBox::Boxes(boxes)) = map.get::<CollisionBox>()
            && boxes.iter().any(CollisionBoundingBox::has_min_max)
        {
            let prefix = section_path(section);
            report.error(
                DiagnosticCode::MissingToggles {
                    text: fixed(
                        "Upcoming Creator Features experiment is required to use \"min\"/\"max\" in \"minecraft:collision_box\" from version 1.26.60 and after",
                    ),
                },
                &rel(&prefix, &["minecraft:collision_box"]),
            );
            return;
        }
    }
}

/// `block_entity.container`: `slot_count` in `1..=54`, `title` non-empty and at most 256
/// characters (the bound is from the member's help text), each an error; and a container
/// excludes a crafting table in the base components.
fn block_entity(block: &Block, report: &mut Report<'_>) {
    let Some(container) = block
        .components
        .get::<BlockEntity>()
        .and_then(|entity| entity.container.as_ref())
    else {
        return;
    };
    let at = ["components", "minecraft:block_entity", "container"];
    if out_of_bounds(container).is_some() {
        report.error(
            DiagnosticCode::InvalidValue { text: None },
            &[at[0], at[1], at[2], "slot_count"],
        );
    }
    if container
        .title
        .as_ref()
        .is_some_and(|title| title.is_empty() || title.chars().count() > 256)
    {
        report.error(
            DiagnosticCode::InvalidValue { text: None },
            &[at[0], at[1], at[2], "title"],
        );
    }
    if has(&block.components, "minecraft:crafting_table") {
        report.error(
            DiagnosticCode::InvalidValue {
                text: fixed(
                    "Block cannot have both a \"container\" in \"minecraft:block_entity\" and a \"minecraft:crafting_table\" component.",
                ),
            },
            &["components"],
        );
    }
}

/// `neighbor_change.neighbor_directions`: 1 to 6 directions (a set), and "The 'all' neighbor
/// direction must be used alone"; base, then permutations; each an error.
fn neighbor_change(block: &Block, report: &mut Report<'_>) {
    let sections = std::iter::once((None, &block.components)).chain(
        block
            .permutations
            .iter()
            .enumerate()
            .map(|(i, p)| (Some(i), &p.components)),
    );
    for (section, map) in sections {
        let Some(directions) = map
            .get::<NeighborChange>()
            .and_then(|c| c.neighbor_directions.as_ref())
        else {
            continue;
        };
        let prefix = section_path(section);
        let path = rel(
            &prefix,
            &["minecraft:neighbor_change", "neighbor_directions"],
        );
        let mut distinct: Vec<NeighborChangeDirection> = Vec::new();
        for direction in directions {
            if !distinct.contains(direction) {
                distinct.push(*direction);
            }
        }
        if distinct.is_empty() || distinct.len() > 6 {
            report.error(DiagnosticCode::InvalidValue { text: None }, &path);
        } else if distinct.len() > 1 && distinct.contains(&NeighborChangeDirection::All) {
            report.error(
                DiagnosticCode::InvalidValue {
                    text: fixed("The 'all' neighbor direction must be used alone"),
                },
                &path,
            );
        }
    }
}

/// The legacy release of the multi-block trait.
const MULTI_BLOCK_RELEASED: FormatVersion = known::V1_26_40;

/// The version from which a horizontal multi-block needs no experiment.
const HORIZONTAL_MULTI_BLOCK_RELEASE: FormatVersion = known::V1_26_50;

/// The version the legacy row of `sound` is released at.
const SOUND_RELEASED: FormatVersion = known::V1_26_40;

// ---- the shared values ---------------------------------------------------------------------

/// The findings of the shared descriptors the block holds: `blocks_to_corner_with`,
/// placement filters and item-specific mining speeds.
fn shared_values(block: &Block, report: &mut Report<'_>) {
    let mut found = Vec::new();
    if let Some(direction) = &block.description.traits.placement_direction {
        for (i, descriptor) in direction.blocks_to_corner_with().iter().enumerate() {
            let mut at = report.path(&[
                "description",
                "traits",
                "minecraft:placement_direction",
                "blocks_to_corner_with",
            ]);
            at.push_index(i);
            found.extend(descriptor.check(&at, Severity::Warning));
        }
    }
    let sections = std::iter::once((None, &block.components)).chain(
        block
            .permutations
            .iter()
            .enumerate()
            .map(|(i, p)| (Some(i), &p.components)),
    );
    for (section, map) in sections {
        let prefix = section_path(section);
        if let Some(filter) = map.get::<PlacementFilter>() {
            for (c, condition) in filter.conditions().iter().enumerate() {
                for (b, descriptor) in condition.block_filter().iter().enumerate() {
                    let mut at =
                        report.path(&rel(&prefix, &["minecraft:placement_filter", "conditions"]));
                    at.push_index(c);
                    at.push("block_filter");
                    at.push_index(b);
                    found.extend(descriptor.check(&at, Severity::Warning));
                }
            }
        }
        if let Some(DestructibleByMining::Params(mining)) = map.get::<DestructibleByMining>() {
            for (i, speed) in mining.item_specific_speeds.iter().enumerate() {
                let mut at = report.path(&rel(
                    &prefix,
                    &["minecraft:destructible_by_mining", "item_specific_speeds"],
                ));
                at.push_index(i);
                at.push("item");
                found.extend(speed.item.check(&at));
            }
        }
    }
    report.d.extend(found);
}

// ---- rule 4 and rule 12: across components --------------------------------------------------

/// Rule 4 (from 1.26.20): a permutation's `redstone_producer` needs one in the base.
fn redstone_producer(block: &Block, report: &mut Report<'_>) {
    if block.source_format_version < CEREAL || has(&block.components, "minecraft:redstone_producer")
    {
        return;
    }
    for (index, permutation) in block.permutations.iter().enumerate() {
        if has(&permutation.components, "minecraft:redstone_producer") {
            let i = index.to_string();
            report.error(
                DiagnosticCode::InvalidValue {
                    text: text(
                        "For 'format_version' >= {} the block cannot define '{}' in the 'permutations' array and not define it in the base 'components' object",
                        &["1.26.20", "minecraft:redstone_producer"],
                    ),
                },
                &[
                    "permutations",
                    &i,
                    "components",
                    "minecraft:redstone_producer",
                ],
            );
        }
    }
}

/// Whether a section has the component `name`.
fn has(map: &ComponentMap<BlockComponent>, name: &str) -> bool {
    map.iter().any(|entry| match entry {
        ComponentRef::Typed { name: n, .. }
        | ComponentRef::Dynamic { name: n, .. }
        | ComponentRef::Unknown { name: n, .. } => n == name,
        ComponentRef::Custom(c) => c.name == name,
        _ => false,
    })
}

/// The cross-component validators of rule 12, on the base merged with each permutation.
#[allow(
    clippy::too_many_lines,
    reason = "one rule after another, in the order of the rule list"
)]
fn cross_components(block: &Block, report: &mut Report<'_>) {
    let version = block.source_format_version;
    let cereal = version >= CEREAL;
    let base = &block.components;
    let sections: Vec<(Option<usize>, Option<&ComponentMap<BlockComponent>>)> =
        std::iter::once((None, None))
            .chain(
                block
                    .permutations
                    .iter()
                    .enumerate()
                    .map(|(i, p)| (Some(i), Some(&p.components))),
            )
            .collect();
    let mut seen: Vec<&'static str> = Vec::new();
    let mut fail = |report: &mut Report<'_>,
                    section: Option<usize>,
                    message: Option<VanillaText>,
                    key: &'static str| {
        if seen.contains(&key) {
            return;
        }
        seen.push(key);
        let prefix = section_path(section);
        report.error(
            DiagnosticCode::InvalidValue { text: message },
            &rel(&prefix, &[]),
        );
    };
    for (section, extra) in sections {
        let get = |name: &str| extra.is_some_and(|m| has(m, name)) || has(base, name);
        let geometry = get("minecraft:geometry");
        let materials = get("minecraft:material_instances");
        if version >= FormatVersion::new(1, 21, 80) && geometry != materials {
            let message = if cereal {
                "Block needs both a \"geometry\" and \"material_instances\" component."
            } else {
                "Block needs both a geometry and material instances component."
            };
            fail(report, section, fixed(message), "geometry");
        }
        if get("minecraft:flower_pottable") && !(geometry && materials) {
            fail(
                report,
                section,
                fixed(
                    "Block that uses \"flower_pottable\" needs both a \"geometry\" and \"material_instances\" component.",
                ),
                "flower_pottable",
            );
        }
        if !cereal && get("minecraft:item_visual") && !(geometry && materials) {
            fail(
                report,
                section,
                text(
                    "Block {} needs both a geometry and material instances component when using an Item Visual Component.",
                    &[block.description.identifier.as_str()],
                ),
                "item_visual",
            );
        }
        // Snow logging needs the collision box disabled.
        let precipitation = extra
            .and_then(|m| m.get::<PrecipitationInteractions>())
            .or_else(|| base.get::<PrecipitationInteractions>());
        let collision = extra
            .and_then(|m| m.get::<CollisionBox>())
            .or_else(|| base.get::<CollisionBox>());
        let snowlogging = precipitation.is_some_and(|p| {
            matches!(
                p.precipitation_behavior,
                PrecipitationBehavior::Snowlogging | PrecipitationBehavior::SnowLogNoCollision
            )
        });
        if snowlogging && collision.is_some_and(|c| *c != CollisionBox::Enabled(false)) {
            fail(
                report,
                section,
                fixed(
                    "Blocks with precipitation interactions \"snowlogging\" must have collision box disabled",
                ),
                "snowlogging",
            );
        }
        let full_block = extra
            .and_then(|m| m.get::<Geometry>())
            .or_else(|| base.get::<Geometry>())
            .is_some_and(|g| g.identifier() == "minecraft:geometry.full_block");
        if full_block && get("minecraft:random_offset") {
            fail(
                report,
                section,
                fixed(
                    "Block with \"geometry\" set at \"minecraft:geometry.full_block\" cannot have a \"random_offset\"",
                ),
                "random_offset",
            );
        }
    }
    if has(base, "minecraft:redstone_producer") && has(base, "minecraft:redstone_consumer") {
        let message = if cereal {
            "Block cannot have both \"minecraft:redstone_producer\" and \"minecraft:redstone_consumer\" components."
        } else {
            "Block cannot have both 'minecraft:redstone_producer' and 'minecraft:redstone_consumer' components"
        };
        fail(report, None, fixed(message), "redstone");
    }
}

// ---- rules 14 and 16: multi-block blocks ---------------------------------------------------

/// The multi-block rules: the base needs a `movable` that pops or is immovable, no
/// `random_offset` with a horizontal direction, no connection or placement-position trait,
/// placement direction only cardinal; its permutations may not define `movable`,
/// `replaceable`, `random_offset`, `placement_filter`, or with a vertical direction
/// `precipitation_interactions`.
fn multi_block(block: &Block, report: &mut Report<'_>) {
    if block.description.traits.multi_block.is_none() {
        return;
    }
    let traits = &block.description.traits;
    match block.components.get::<Movable>() {
        None => report.error(
            DiagnosticCode::MissingRequiredField {
                text: fixed(
                    "Blocks with trait 'minecraft:multi_block' must define component 'minecraft:movable'",
                ),
            },
            &["components"],
        ),
        Some(movable)
            if !matches!(
                movable.movement_type,
                MovementType::Popped | MovementType::Immovable
            ) =>
        {
            report.error(
                DiagnosticCode::InvalidValue {
                    text: fixed(
                        "Blocks with trait 'minecraft:multi_block' for component 'minecraft:movable' the 'movement_type' must either be 'popped' or 'immovable'",
                    ),
                },
                &["components", "minecraft:movable", "movement_type"],
            );
        }
        Some(_) => {}
    }
    // The 1.26.60 validator has no multi-block + `block_entity` rule; a horizontal direction
    // refuses a base `random_offset`.
    let horizontal = matches!(
        traits.multi_block.as_ref().map(|m| m.direction.0.as_str()),
        Some("north" | "south" | "west" | "east")
    );
    if horizontal && has(&block.components, "minecraft:random_offset") {
        report.error(
            DiagnosticCode::InvalidValue {
                text: fixed(
                    "Blocks with trait 'minecraft:multi_block' and a horizontal 'direction' cannot define component 'minecraft:random_offset'",
                ),
            },
            &["components", "minecraft:random_offset"],
        );
    }
    for (other, present) in [
        ("minecraft:connection", traits.connection.is_some()),
        (
            "minecraft:placement_position",
            traits.placement_position.is_some(),
        ),
    ] {
        if present {
            report.error(
                DiagnosticCode::InvalidValue {
                    text: text(
                        "A block with trait 'minecraft:multi_block' cannot also define trait '{}'",
                        &[other],
                    ),
                },
                &["description", "traits", other],
            );
        }
    }
    if let Some(direction) = &traits.placement_direction
        && direction
            .enabled_states
            .iter()
            .any(|s| s.0 != "minecraft:cardinal_direction")
    {
        report.error(
            DiagnosticCode::InvalidValue {
                text: fixed(
                    "A block with trait 'minecraft:multi_block' and trait 'minecraft:placement_direction' can only enable state 'minecraft:cardinal_direction'",
                ),
            },
            &[
                "description",
                "traits",
                "minecraft:placement_direction",
                "enabled_states",
            ],
        );
    }
    multi_block_permutations(block, horizontal, report);
}

/// The permutation constraints of a multi-block block.
fn multi_block_permutations(block: &Block, horizontal: bool, report: &mut Report<'_>) {
    for (index, permutation) in block.permutations.iter().enumerate() {
        for name in [
            "minecraft:movable",
            "minecraft:replaceable",
            "minecraft:random_offset",
            "minecraft:placement_filter",
            "minecraft:precipitation_interactions",
        ] {
            // `precipitation_interactions` only when the direction is vertical.
            if name == "minecraft:precipitation_interactions" && horizontal {
                continue;
            }
            if has(&permutation.components, name) {
                let i = index.to_string();
                report.error(
                    DiagnosticCode::InvalidValue {
                        text: text(
                            "Component '{}' is not valid in multi block permutations, only in the base BlockType components",
                            &[name],
                        ),
                    },
                    &["permutations", &i, "components", name],
                );
            }
        }
    }
}

// ---- rule 17: `geometry.n_way_visual_rotation` ---------------------------------------------

/// Base components only, not on a multi-block, at least one axis, each naming a state.
fn n_way_visual_rotation(block: &Block, report: &mut Report<'_>) {
    let rotation = |map: &ComponentMap<BlockComponent>| match map.get::<Geometry>() {
        Some(Geometry::Params(params)) => params.n_way_visual_rotation.clone(),
        _ => None,
    };
    for (index, permutation) in block.permutations.iter().enumerate() {
        if rotation(&permutation.components).is_some() {
            let i = index.to_string();
            report.error(
                DiagnosticCode::InvalidValue {
                    text: fixed(
                        "minecraft:geometry 'n_way_visual_rotation' may only be defined on the base block and is not allowed in permutations.",
                    ),
                },
                &[
                    "permutations",
                    &i,
                    "components",
                    "minecraft:geometry",
                    "n_way_visual_rotation",
                ],
            );
        }
    }
    let Some(base) = rotation(&block.components) else {
        return;
    };
    let rel = ["components", "minecraft:geometry", "n_way_visual_rotation"];
    if block.description.traits.multi_block.is_some() {
        report.error(
            DiagnosticCode::InvalidValue {
                text: fixed(
                    "'minecraft:geometry' field 'n_way_visual_rotation' cannot be defined for a block with trait 'minecraft:multi_block'",
                ),
            },
            &rel,
        );
    }
    let axes = [("x", &base.x), ("y", &base.y), ("z", &base.z)];
    if axes.iter().all(|(_, state)| state.is_none()) {
        report.error(
            DiagnosticCode::MissingRequiredField {
                text: fixed("'n_way_visual_rotation' must define at least one axis (x, y, or z)"),
            },
            &rel,
        );
    }
    let states = block.states_with_traits();
    for (axis, state) in axes {
        if let Some(state) = state
            && !states.contains_key(state)
        {
            report.error(
                DiagnosticCode::UnresolvedReference {
                    text: text(
                        "'minecraft:geometry' field 'n_way_visual_rotation.{}' references unknown state '{}'",
                        &[axis, state],
                    ),
                },
                &rel,
            );
        }
    }
}

// ---- rule 19: the outcomes of the compile pass ---------------------------------------------

/// The failure rules of the permutation conditions: in `[1.19.70, 1.26.20)` a rejected
/// condition rejects the block ("permutation condition failed to parse", beside the compiler's
/// own findings); below 1.19.70 it is compiled again without the allow-list and the permutation
/// is dropped when that compiles (`DroppedField`, the crate's own). Returns the permutations to drop.
#[cfg(feature = "molang")]
fn molang_failures(
    block: &Block,
    failures: &[bedrock_addon_core::molang::FieldOutcome],
    cx: &LoadContext,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Vec<usize> {
    use bedrock_addon_core::molang::{MolangFailure, MolangOutcome};
    let mut dropped = Vec::new();
    let mut report = Report { at, d };
    for failure in failures {
        let MolangOutcome::Failed(rule) = failure.outcome else {
            continue;
        };
        let Some(index) = permutation_index(&failure.path) else {
            continue;
        };
        if failure.path.last_key() != Some("condition") {
            continue;
        }
        let i = index.to_string();
        match rule {
            MolangFailure::RejectDocument => report.error(
                DiagnosticCode::InvalidValue {
                    text: fixed("permutation condition failed to parse"),
                },
                &["permutations", &i, "condition"],
            ),
            MolangFailure::DropEntry if compiles_unrestricted(block, index, cx) => {
                // The game drops the permutation without a word (the parser's lines are the
                // condition's own), so the crate reports an information.
                report.push(
                    DiagnosticCode::DroppedField { text: None },
                    Severity::Info,
                    &["permutations", &i],
                );
                dropped.push(index);
            }
            _ => {}
        }
    }
    dropped
}

/// Whether the condition of permutation `index` compiles under the unrestricted rule.
#[cfg(feature = "molang")]
fn compiles_unrestricted(block: &Block, index: usize, cx: &LoadContext) -> bool {
    use bedrock_addon_core::molang::{MolangContext, MolangFieldSpec, MolangOutcome, Side};
    use bedrock_addon_core::version::FormatVersionInput;
    let (Some(check), Some(permutation)) = (&cx.molang, block.permutations.get(index)) else {
        return false;
    };
    let spec = MolangFieldSpec::DEFAULT;
    let molang = MolangContext::new(
        cx.min_engine_version.as_ref(),
        cx.molang_experiments,
        &FormatVersionInput {
            version: Some(block.source_format_version),
            beta_literal: false,
            prerelease: false,
        },
        Some(cx.molang_version),
    );
    let mut scratch = Diagnostics::default();
    check.check(
        &permutation.condition,
        &spec,
        Side::Server,
        &molang,
        &bedrock_addon_core::diagnostic::Location::default(),
        &mut scratch,
    ) == MolangOutcome::Ok
}

#[cfg(test)]
mod tests {
    use crate::history::BlockFamily;
    use bedrock_addon_core::versioning::Family;

    /// The rows the family gates itself carry no generic gate: no `released`, no `requires`.
    #[test]
    fn the_rows_the_family_gates_have_no_generic_gate() {
        for name in [
            "minecraft:instrument_sound",
            "minecraft:sound",
            "minecraft:vibration_properties",
            "minecraft:block_entity",
        ] {
            let row = BlockFamily::components().get(name).expect("a row");
            assert_eq!(row.released, None, "{name}");
            assert!(row.requires.is_empty(), "{name}");
        }
    }
}

/// A reference of the block to something of its own that it does not have: an information
/// of the crate's own, the reference kept, at `at/<steps…>`. The game has no line for a
/// dangling material instance alias or event name.
fn dangling(d: &mut Diagnostics, at: &JsonPath, steps: &[Step<'_>], kind: &'static str, id: &str) {
    let mut path = at.clone();
    for step in steps {
        match step {
            Step::Key(key) => path.push(key),
            Step::Index(index) => path.push_index(*index),
        }
    }
    d.push(
        Diagnostic::info(DiagnosticCode::UnresolvedReference { text: None })
            .at(path)
            .phase(Phase::Validate)
            .detail(
                bedrock_addon_core::diagnostic::DiagnosticDetail::Reference {
                    kind,
                    id: id.to_owned(),
                    hint: None,
                },
            ),
    );
}

/// One step of a path below the payload.
#[derive(Clone, Copy)]
enum Step<'s> {
    Key(&'s str),
    Index(usize),
}

/// The references a block makes to its own parts. A material instance written as a string
/// names another instance of the same `material_instances` object; a deprecated
/// trigger that fires on the block itself, and a trigger inside an event handler, names one of
/// the block's event handlers.
#[allow(
    clippy::too_many_lines,
    reason = "one walk over the sections, then one over the handlers"
)]
fn references(block: &Block, at: &JsonPath, d: &mut Diagnostics) {
    use super::component::{
        DeprecatedBaseTrigger, DeprecatedDefinitionEvent, EmbeddedVisual, ItemVisual,
        MaterialInstanceEntry,
    };
    use Step::{Index, Key};

    let aliases = |d: &mut Diagnostics, base: &[Step<'_>], instances: &MaterialInstances| {
        for (name, entry) in &instances.0 {
            if let MaterialInstanceEntry::Alias(target) = entry
                && !instances.0.contains_key(target)
            {
                let mut steps = base.to_vec();
                steps.push(Key(name));
                dangling(d, at, &steps, "material instance", target);
            }
        }
    };
    let trigger = |d: &mut Diagnostics, base: &[Step<'_>], trigger: &DeprecatedBaseTrigger| {
        let own = matches!(
            trigger.target,
            None | Some(bedrock_addon_core::shared::FilterSubject::Self_)
        );
        if let Some(event) = &trigger.event
            && own
            && !event.0.is_empty()
            && !block.deprecated_event_handlers.contains_key(&event.0)
        {
            let mut steps = base.to_vec();
            steps.push(Key("event"));
            dangling(d, at, &steps, "event", &event.0);
        }
    };
    let sections = std::iter::once((Vec::new(), &block.components, &block.deprecated)).chain(
        block
            .permutations
            .iter()
            .enumerate()
            .map(|(index, permutation)| {
                (
                    vec![Key("permutations"), Index(index)],
                    &permutation.components,
                    &permutation.deprecated,
                )
            }),
    );
    for (base, components, deprecated) in sections {
        let below = |keys: &[&'static str]| {
            let mut steps = base.clone();
            steps.extend(keys.iter().map(|key| Key(key)));
            steps
        };
        if let Some(instances) = components.get::<MaterialInstances>() {
            aliases(
                d,
                &below(&["components", "minecraft:material_instances"]),
                instances,
            );
        }
        if let Some(visual) = components.get::<ItemVisual>() {
            let path = below(&["components", "minecraft:item_visual", "material_instances"]);
            aliases(d, &path, &visual.material_instances);
        }
        if let Some(visual) = components.get::<EmbeddedVisual>() {
            let path = below(&[
                "components",
                "minecraft:embedded_visual",
                "material_instances",
            ]);
            aliases(d, &path, &visual.material_instances);
        }
        for (key, fired) in [
            ("on_fall_on", &deprecated.on_fall_on),
            ("on_interact", &deprecated.on_interact),
            ("on_placed", &deprecated.on_placed),
            ("on_player_destroyed", &deprecated.on_player_destroyed),
            ("on_player_placing", &deprecated.on_player_placing),
            ("on_step_off", &deprecated.on_step_off),
            ("on_step_on", &deprecated.on_step_on),
        ] {
            if let Some(fired) = fired {
                trigger(d, &below(&["deprecated", key]), fired);
            }
        }
        if let Some(ticking) = &deprecated.queued_ticking {
            trigger(
                d,
                &below(&["deprecated", "queued_ticking", "on_tick"]),
                &ticking.on_tick,
            );
        }
        if let Some(ticking) = &deprecated.random_ticking {
            trigger(
                d,
                &below(&["deprecated", "random_ticking", "on_tick"]),
                &ticking.on_tick,
            );
        }
    }
    // The handlers' own triggers; a handler nests through `sequence` and `randomize`, walked
    // without recursion.
    let mut waiting: Vec<(Vec<Step<'_>>, &DeprecatedDefinitionEvent)> = block
        .deprecated_event_handlers
        .iter()
        .rev()
        .map(|(name, event)| (vec![Key("deprecated_event_handlers"), Key(name)], event))
        .collect();
    while let Some((path, event)) = waiting.pop() {
        // The game's event loader (shared with item events): an event with `randomize` and
        // `sequence` logs a warning-level line and loads the `randomize` alone. A present but
        // empty list is not told apart from an absent one in the model.
        let ignores_sequence = !event.randomize.is_empty() && !event.sequence.is_empty();
        if ignores_sequence {
            // (the findings of a block's handlers say `events`, the file's name)
            let mut warned = at.child("events");
            for step in &path[1..] {
                match step {
                    Key(key) => warned.push(key),
                    Index(index) => warned.push_index(*index),
                }
            }
            d.push(event::sequence_ignored(&warned).phase(Phase::Validate));
        }
        if let Some(fired) = &event.trigger {
            let own = matches!(
                fired.target,
                None | Some(bedrock_addon_core::shared::FilterSubject::Self_)
            );
            if let Some(name) = &fired.event
                && own
                && !name.0.is_empty()
                && !block.deprecated_event_handlers.contains_key(&name.0)
            {
                let mut steps = path.clone();
                steps.extend([Key("trigger"), Key("event")]);
                dangling(d, at, &steps, "event", &name.0);
            }
        }
        for (key, list) in [
            ("randomize", &event.randomize),
            ("sequence", &event.sequence),
        ] {
            if ignores_sequence && key == "sequence" {
                continue;
            }
            for (index, nested) in list.iter().enumerate().rev() {
                let mut steps = path.clone();
                steps.extend([Key(key), Index(index)]);
                waiting.push((steps, nested));
            }
        }
    }
}
