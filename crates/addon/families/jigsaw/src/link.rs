//! The jigsaw references the link pass resolves: the crate-private checker `StackSet::link`
//! calls with the other families' registries. Every miss is an `UnresolvedReference` finding
//! in `Phase::Link`, one per member, naming the kind and the name of what is missing.

use super::{
    BlockEntityModifier, BlockPredicate, CappedDelegate, GroupedPoolAlias, Jigsaw, PoolAlias,
    PoolElement, Processor, ProcessorRule,
};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Location, Phase, Severity,
    VanillaText,
};
use bedrock_addon_core::document::Document;
use bedrock_addon_core::id::{
    BlockId, FeatureId, JigsawStructureId, Kind, LootTablePath, ProcessorListId, StructureId,
    TemplatePoolId, kind,
};

/// What the jigsaw references resolve against outside the group.
pub trait Targets {
    /// A block of the stack or a built-in one.
    fn block(&self, id: &BlockId) -> bool;
    /// A feature (`bp.features()`, app data included).
    fn feature(&self, id: &FeatureId) -> bool;
    /// A loot table file.
    fn loot_table(&self, path: &LootTablePath) -> bool;
    /// A structure template (`structures/<location>.mcstructure`).
    fn structure(&self, location: &StructureId) -> bool;
    /// A structure file the game's linked-asset check registers from the packs' files: any pack's `structures/<location>.nbt` or
    /// `.mcstructure`, whether or not generation reads it.
    fn structure_file(&self, location: &StructureId) -> bool {
        self.structure(location)
    }
    /// A template pool the game has outside the stack.
    fn pool(&self, _id: &TemplatePoolId) -> bool {
        false
    }
    /// A processor list the game has outside the stack.
    fn processor_list(&self, _id: &ProcessorListId) -> bool {
        false
    }
    /// A jigsaw structure, or a structure of the game's code a set can place, outside the
    /// stack.
    fn jigsaw_structure(&self, _id: &JigsawStructureId) -> bool {
        false
    }
}

const BLOCK: &str = <kind::Block as Kind>::NAME;
const FEATURE: &str = <kind::Feature as Kind>::NAME;
const LOOT_TABLE: &str = <kind::LootTable as Kind>::NAME;
const STRUCTURE: &str = <kind::Structure as Kind>::NAME;
const POOL: &str = <kind::TemplatePool as Kind>::NAME;
const PROCESSOR_LIST: &str = <kind::ProcessorList as Kind>::NAME;
const JIGSAW_STRUCTURE: &str = <kind::JigsawStructure as Kind>::NAME;

/// The template pool every pool falls back to by default, built into the game.
const EMPTY_POOL: &str = "minecraft:empty";

/// One document's findings.
struct Report<'a, T> {
    document: &'a Document<T>,
    out: &'a mut Diagnostics,
}

/// Whether a missing reference's error is kept with the document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kept {
    /// The finding is reported as it is, not marked kept.
    Dropped,
    /// The error is marked kept: the document and its list stay despite it.
    Kept,
}

impl<T> Report<'_, T> {
    /// An `UnresolvedReference` information at `at` with `text`, for the `kind` named `id`:
    /// the crate's own. The game writes the structure and feature texts only at generation
    /// time, never at load, so a miss here is not a content-log line of the game's.
    fn miss(&mut self, at: &JsonPath, kind: &'static str, id: &str, text: VanillaText) {
        self.miss_at(at, kind, id, text, Severity::Info);
    }

    /// [`Self::miss`] with `severity`.
    fn miss_at(
        &mut self,
        at: &JsonPath,
        kind: &'static str,
        id: &str,
        text: VanillaText,
        severity: Severity,
    ) {
        self.miss_kept(at, kind, id, text, severity, Kept::Dropped);
    }

    /// [`Self::miss_at`], a kept error when `kept` is [`Kept::Kept`].
    fn miss_kept(
        &mut self,
        at: &JsonPath,
        kind: &'static str,
        id: &str,
        text: VanillaText,
        severity: Severity,
        kept: Kept,
    ) {
        let origin = self.document.origin();
        let mut finding = Diagnostic::new(
            DiagnosticCode::UnresolvedReference { text: Some(text) },
            severity,
            Phase::Link,
            Location::new(
                Some(origin.pack.clone()),
                Some(origin.path.clone()),
                at.clone(),
                None,
            ),
        );
        finding.detail = Some(Box::new(DiagnosticDetail::Reference {
            kind,
            id: id.to_owned(),
            hint: None,
        }));
        if severity == Severity::Error && kept == Kept::Kept {
            finding = finding.into_kept();
        }
        self.out.push(finding);
    }

    /// The linked-asset validator's text: the member as vanilla writes the path
    /// (`-> elements[0] -> element -> processors`), then the identifier.
    fn missing_asset(&mut self, at: &JsonPath, payload: &JsonPath, kind: &'static str, id: &str) {
        self.validator(at, payload, "{}:  Missing referenced asset {}", kind, id);
    }

    /// The validator's `'{} Invalid asset path {}'` for a file
    /// reference (a structure, a loot table) no pack holds.
    fn invalid_asset_path(
        &mut self,
        at: &JsonPath,
        payload: &JsonPath,
        kind: &'static str,
        id: &str,
    ) {
        self.validator(at, payload, "{}:  Invalid asset path {}", kind, id);
    }

    /// The validator's line `template` for `id` at `at`.
    fn validator(
        &mut self,
        at: &JsonPath,
        payload: &JsonPath,
        template: &'static str,
        kind: &'static str,
        id: &str,
    ) {
        let text = VanillaText::Formatted {
            template,
            args: vec![vanilla_path(at, payload), id.to_owned()],
        };
        // The game's validator writes the line to the content log at the error level and
        // drops nothing.
        self.miss_at(at, kind, id, text, Severity::Error);
    }
}

/// `at` below `payload`, as vanilla's validator writes it: `-> key` per member, `[i]` after
/// an array member.
fn vanilla_path(at: &JsonPath, payload: &JsonPath) -> String {
    let relative = at.strip_prefix(payload);
    let mut out = String::new();
    for segment in relative.as_ref().unwrap_or(at).segments() {
        match segment {
            bedrock_addon_core::diagnostic::JsonPathSegment::Index(index) => {
                out.push('[');
                out.push_str(&index.to_string());
                out.push(']');
            }
            bedrock_addon_core::diagnostic::JsonPathSegment::Key(key) => {
                out.push_str(" -> ");
                out.push_str(key);
            }
        }
    }
    out.trim_start().to_owned()
}

/// `base` followed by `key`.
fn child(base: &JsonPath, key: &str) -> JsonPath {
    let mut path = base.clone();
    path.push(key);
    path
}

/// `base` followed by the element `i`.
fn item(base: &JsonPath, i: usize) -> JsonPath {
    let mut path = base.clone();
    path.push_index(i);
    path
}

/// The payload's path.
fn payload(key: &str) -> JsonPath {
    let mut path = JsonPath::new();
    path.push(key);
    path
}

/// Every unresolved reference of the group, in collection order.
pub fn link(jigsaw: &Jigsaw, targets: &dyn Targets) -> Diagnostics {
    let mut out = Diagnostics::default();
    link_structures(jigsaw, targets, &mut out);
    link_pools(jigsaw, targets, &mut out);
    link_sets(jigsaw, targets, &mut out);
    link_processors(jigsaw, targets, &mut out);
    out
}

/// A template pool of the group, vanilla's built-in empty pool, or one the game has outside
/// the stack.
fn pool_defined(jigsaw: &Jigsaw, targets: &dyn Targets, id: &TemplatePoolId) -> bool {
    id.as_str() == EMPTY_POOL || jigsaw.pools.contains(id) || targets.pool(id)
}

/// The pools an alias names, each with its member's path.
fn alias_pools<'a>(alias: &'a PoolAlias, at: &JsonPath) -> Vec<(JsonPath, &'a TemplatePoolId)> {
    let random = |targets: &'a [super::Weighted<TemplatePoolId>], at: &JsonPath| {
        targets
            .iter()
            .enumerate()
            .map(|(i, weighted)| {
                (
                    child(&item(&child(at, "targets"), i), "data"),
                    &weighted.data,
                )
            })
            .collect::<Vec<_>>()
    };
    match alias {
        PoolAlias::Direct { target, .. } => vec![(child(at, "target"), target)],
        PoolAlias::Random { targets, .. } => random(targets, at),
        PoolAlias::RandomGroup { groups, .. } => {
            let mut found = Vec::new();
            for (j, group) in groups.iter().enumerate() {
                let at_data = child(&item(&child(at, "groups"), j), "data");
                for (k, grouped) in group.data.iter().enumerate() {
                    let at_grouped = item(&at_data, k);
                    match grouped {
                        GroupedPoolAlias::Direct { target, .. } => {
                            found.push((child(&at_grouped, "target"), target));
                        }
                        GroupedPoolAlias::Random { targets, .. } => {
                            found.extend(random(targets, &at_grouped));
                        }
                    }
                }
            }
            found
        }
    }
}

/// Each structure's `start_pool` and its aliases' pools.
fn link_structures(jigsaw: &Jigsaw, targets: &dyn Targets, out: &mut Diagnostics) {
    for (_, document) in &jigsaw.structures {
        let at = payload("minecraft:jigsaw");
        let mut report = Report { document, out };
        if !pool_defined(jigsaw, targets, &document.start_pool) {
            report.missing_asset(
                &child(&at, "start_pool"),
                &at,
                POOL,
                document.start_pool.as_str(),
            );
        }
        let aliases = child(&at, "pool_aliases");
        for (i, alias) in document.pool_aliases.iter().flatten().enumerate() {
            for (path, target) in alias_pools(alias, &item(&aliases, i)) {
                if !pool_defined(jigsaw, targets, target) {
                    report.missing_asset(&path, &at, POOL, target.as_str());
                }
            }
        }
    }
}

/// Each pool's fallback, its elements' templates, processor lists and features.
fn link_pools(jigsaw: &Jigsaw, targets: &dyn Targets, out: &mut Diagnostics) {
    for (id, document) in &jigsaw.pools {
        let at = payload("minecraft:template_pool");
        let mut report = Report { document, out };
        if let Some(fallback) = &document.fallback
            && !pool_defined(jigsaw, targets, fallback)
        {
            report.missing_asset(&child(&at, "fallback"), &at, POOL, fallback.as_str());
        }
        let elements = child(&at, "elements");
        for (i, entry) in document.elements.iter().enumerate() {
            let at_element = child(&item(&elements, i), "element");
            match &entry.element {
                PoolElement::Single {
                    location,
                    processors,
                    ..
                } => {
                    // The text is the game's generation-failure line for a missing structure,
                    // written at generation time rather than at load; it is kept as the link
                    // finding's text, an information of the crate's own.
                    if !targets.structure(location) {
                        let at_location = child(&at_element, "location");
                        report.miss(
                            &at_location,
                            STRUCTURE,
                            location.as_str(),
                            VanillaText::Formatted {
                                template: "[JIGSAW GENERATION FAILURE] Structure name [{}] referenced in template pool [{}] does not exist.",
                                args: vec![location.as_str().to_owned(), id.as_str().to_owned()],
                            },
                        );
                        // The game's validator logs the file reference too, unless a pack
                        // holds the file.
                        if !targets.structure_file(location) {
                            report.invalid_asset_path(
                                &at_location,
                                &at,
                                STRUCTURE,
                                location.as_str(),
                            );
                        }
                    }
                    // `""` is no processors.
                    if let Some(list) = processors
                        && !list.as_str().is_empty()
                        && !jigsaw.processors.contains(list)
                        && !targets.processor_list(list)
                    {
                        report.missing_asset(
                            &child(&at_element, "processors"),
                            &at,
                            PROCESSOR_LIST,
                            list.as_str(),
                        );
                    }
                }
                // The text is the game's line for a feature that does not resolve, written
                // when the pool is generated; an information of the crate's own at link.
                PoolElement::Feature { feature, .. } if !targets.feature(feature) => {
                    report.miss(
                        &child(&at_element, "feature"),
                        FEATURE,
                        feature.as_str(),
                        VanillaText::Formatted {
                            template: "Cannot find 'feature' '{}' for 'feature_pool_element'. Make sure to include the 'namespace_name:' prefix.",
                            args: vec![feature.as_str().to_owned()],
                        },
                    );
                }
                _ => {}
            }
        }
    }
}

/// An entry naming no jigsaw structure is dropped by vanilla and logged by the
/// linked-asset validator.
fn link_sets(jigsaw: &Jigsaw, targets: &dyn Targets, out: &mut Diagnostics) {
    for (_, document) in &jigsaw.structure_sets {
        let at = payload("minecraft:structure_set");
        let mut report = Report { document, out };
        let structures = child(&at, "structures");
        for (i, entry) in document.structures.iter().enumerate() {
            if !jigsaw.structures.contains(&entry.structure)
                && !targets.jigsaw_structure(&entry.structure)
            {
                report.missing_asset(
                    &child(&item(&structures, i), "structure"),
                    &at,
                    JIGSAW_STRUCTURE,
                    entry.structure.as_str(),
                );
            }
        }
    }
}

/// The blocks and loot tables of each processor list.
fn link_processors(jigsaw: &Jigsaw, targets: &dyn Targets, out: &mut Diagnostics) {
    for (_, document) in &jigsaw.processors {
        let at = payload("minecraft:processor_list");
        let mut report = Report { document, out };
        let processors = child(&at, "processors");
        for (i, processor) in document.processors.iter().enumerate() {
            let at_processor = item(&processors, i);
            let at_delegate = child(&at_processor, "delegate");
            match processor {
                Processor::Rule { rules, .. }
                | Processor::Capped {
                    delegate: CappedDelegate::Rule { rules, .. },
                    ..
                } => {
                    let at_rules = match processor {
                        Processor::Rule { .. } => child(&at_processor, "rules"),
                        _ => child(&at_delegate, "rules"),
                    };
                    rules_references(rules, &at_rules, &at, targets, &mut report);
                }
                Processor::BlockIgnore { blocks, .. }
                | Processor::Capped {
                    delegate: CappedDelegate::BlockIgnore { blocks, .. },
                    ..
                } => {
                    let at_blocks = match processor {
                        Processor::BlockIgnore { .. } => child(&at_processor, "blocks"),
                        _ => child(&at_delegate, "blocks"),
                    };
                    block_references(blocks, &at_blocks, targets, &mut report);
                }
                _ => {}
            }
        }
    }
}

/// "Cannot find block '%s'", reported once per member.
fn missing_block<T>(
    id: &BlockId,
    at: &JsonPath,
    targets: &dyn Targets,
    report: &mut Report<'_, T>,
) {
    if !targets.block(id) {
        // The game logs it at the content log's error level, skips the rule and goes on with
        // the rest; the list is kept: a kept error.
        report.miss_kept(
            at,
            BLOCK,
            id.as_str(),
            VanillaText::Formatted {
                template: "Cannot find block '{}'",
                args: vec![id.as_str().to_owned()],
            },
            Severity::Error,
            Kept::Kept,
        );
    }
}

/// The game's validator line for a rule's block, a second finding beside the block lookup's
/// own (the block of a state object at its `name`, where the validator holds it).
fn validator_block<T>(
    id: &BlockId,
    at: &JsonPath,
    payload: &JsonPath,
    targets: &dyn Targets,
    report: &mut Report<'_, T>,
) {
    if !targets.block(id) {
        report.missing_asset(at, payload, BLOCK, id.as_str());
    }
}

/// The blocks of a `block_ignore` processor.
fn block_references<T>(
    blocks: &[BlockId],
    at: &JsonPath,
    targets: &dyn Targets,
    report: &mut Report<'_, T>,
) {
    for (i, block) in blocks.iter().enumerate() {
        missing_block(block, &item(at, i), targets, report);
    }
}

/// The blocks and loot tables of a list of rules.
fn rules_references<T>(
    rules: &[ProcessorRule],
    at: &JsonPath,
    payload: &JsonPath,
    targets: &dyn Targets,
    report: &mut Report<'_, T>,
) {
    for (i, rule) in rules.iter().enumerate() {
        let at_rule = item(at, i);
        for (key, predicate) in [
            ("input_predicate", &rule.input_predicate),
            ("location_predicate", &rule.location_predicate),
        ] {
            let at_predicate = child(&at_rule, key);
            match predicate {
                Some(
                    BlockPredicate::BlockMatch { block, .. }
                    | BlockPredicate::RandomBlockMatch { block, .. },
                ) => {
                    let at_block = child(&at_predicate, "block");
                    missing_block(block, &at_block, targets, report);
                    validator_block(block, &at_block, payload, targets, report);
                }
                Some(
                    BlockPredicate::BlockstateMatch { block_state, .. }
                    | BlockPredicate::RandomBlockstateMatch { block_state, .. },
                ) => {
                    let at_state = child(&at_predicate, "block_state");
                    missing_block(&block_state.name, &at_state, targets, report);
                    let at_name = child(&at_state, "name");
                    validator_block(&block_state.name, &at_name, payload, targets, report);
                }
                _ => {}
            }
        }
        let at_output = child(&at_rule, "output_state");
        missing_block(&rule.output_state.name, &at_output, targets, report);
        let at_name = child(&at_output, "name");
        validator_block(&rule.output_state.name, &at_name, payload, targets, report);
        // An empty loot table was dropped at load, so only a named one is looked up.
        if let Some(BlockEntityModifier::AppendLoot { loot_table, .. }) =
            &rule.block_entity_modifier
            && !targets.loot_table(loot_table)
        {
            // A file reference: the validator's "Invalid asset path".
            report.invalid_asset_path(
                &child(&child(&at_rule, "block_entity_modifier"), "loot_table"),
                payload,
                LOOT_TABLE,
                loot_table.as_str(),
            );
        }
    }
}
