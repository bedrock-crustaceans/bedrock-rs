//! The aim-assist references the link pass resolves: the doc-hidden checker that `bedrock_addon`'s
//! `StackSet::link` calls with the other families' registries. Every miss is an
//! `UnresolvedReference` finding in `Phase::Link`; the game only records the names with its
//! linked-asset validator, so the crate's own check is what reports them.

use std::collections::{HashMap, HashSet};

use super::AimAssist;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::document::{Document, Origin};
use bedrock_addon_core::id::AimAssistPresetId;
use bedrock_addon_core::id::{Kind, kind};

/// What the aim-assist references resolve against outside the group. Each name is the key
/// vanilla's linked-asset validator registers: lower-cased, an item without its `:aux` part,
/// an entity without its `<event>` part.
pub trait Targets {
    /// A block of the stack or a built-in one.
    fn block(&self, key: &str) -> bool;
    /// An entity of the stack or a built-in one.
    fn entity(&self, key: &str) -> bool;
    /// An item of the stack or a built-in one.
    fn item(&self, key: &str) -> bool;
    /// A category the game defines outside the stack (`minecraft:default`, …).
    fn category(&self, key: &str) -> bool;
    /// An aim-assist preset the game defines outside the stack
    /// (`minecraft:aim_assist_default`).
    fn preset(&self, key: &str) -> bool;
}

/// The key vanilla registers for an item reference: lower-cased, and of three or more
/// `:`-separated parts only the text before the last `:`.
pub(crate) fn item_key(id: &str) -> String {
    let lowered = id.to_lowercase();
    if lowered.split(':').count() >= 3
        && let Some((head, _)) = lowered.rsplit_once(':')
    {
        return head.to_owned();
    }
    lowered
}

/// The key vanilla registers for an entity reference: lower-cased, only the text before the
/// first `<`.
pub(crate) fn entity_key(id: &str) -> String {
    let lowered = id.to_lowercase();
    match lowered.split_once('<') {
        Some((head, _)) => head.to_owned(),
        None => lowered,
    }
}

/// One step of a finding's path below the payload.
#[derive(Clone, Copy)]
enum Step<'a> {
    /// A member.
    Key(&'a str),
    /// An array element.
    Index(usize),
}

use Step::{Index, Key};

/// One document's findings.
struct Report<'a> {
    origin: &'a Origin,
    /// The payload's path.
    base: &'static str,
    out: &'a mut Diagnostics,
}

impl Report<'_> {
    /// An `UnresolvedReference` information for the `kind` named `id` at `path` below the
    /// payload. `link()` makes the references the game's linked-asset validator holds errors
    /// with its line (`with_validator_line`), and a preset's `aim_assist.preset` is checked by
    /// neither folder.
    fn miss(&mut self, path: &[Step<'_>], kind: &'static str, id: &str) {
        let mut at = JsonPath::new();
        at.push(self.base);
        for step in path {
            match *step {
                Index(index) => at.push_index(index),
                Key(key) => at.push(key),
            }
        }
        let mut finding = Diagnostic::new(
            DiagnosticCode::UnresolvedReference { text: None },
            Severity::Info,
            Phase::Link,
            Location::new(
                Some(self.origin.pack.clone()),
                Some(self.origin.path.clone()),
                at,
                None,
            ),
        );
        finding.detail = Some(Box::new(DiagnosticDetail::Reference {
            kind,
            id: id.to_owned(),
            hint: None,
        }));
        self.out.push(finding);
    }
}

/// The report of `document`, whose payload is under `base`.
fn report<'a, T>(
    document: &'a Document<T>,
    base: &'static str,
    out: &'a mut Diagnostics,
) -> Report<'a> {
    Report {
        origin: document.origin(),
        base,
        out,
    }
}

const BLOCK: &str = <kind::Block as Kind>::NAME;
const ENTITY: &str = <kind::Entity as Kind>::NAME;
const ITEM: &str = <kind::Item as Kind>::NAME;
const CATEGORY: &str = <kind::AimAssistCategory as Kind>::NAME;

/// The findings of the group's references against `targets`: the categories' priority keys
/// first, then each preset's exclusions, liquid items, item settings and category references.
/// Block tags and entity type families are plain strings and are not checked.
pub fn link(group: &AimAssist, targets: &dyn Targets) -> Diagnostics {
    let mut out = Diagnostics::default();
    // Category names are registered lower-cased, as every identifier is.
    let categories: HashSet<String> = group
        .categories
        .iter()
        .map(|(id, _)| id.as_str().to_lowercase())
        .collect();
    let category = |id: &str| {
        let key = id.to_lowercase();
        categories.contains(&key) || targets.category(&key)
    };

    // One file's categories share its document origin; each is reported by its own index in
    // the registry's order within that file. The files are counted by pack and path, one
    // probe per category.
    let mut index_in_file: HashMap<(Option<usize>, &str), usize> = HashMap::new();
    for (_, document) in &group.categories {
        let origin = document.origin();
        let next = index_in_file
            .entry((origin.pack.index(), origin.path.as_str()))
            .or_insert(0);
        let position = *next;
        *next += 1;
        let mut report = report(document, "minecraft:aim_assist_categories", &mut out);
        let priorities = &document.priorities;
        for key in priorities.entities.iter().flat_map(|map| map.keys()) {
            if !targets.entity(&entity_key(key)) {
                let path = [
                    Key("categories"),
                    Index(position),
                    Key("priorities"),
                    Key("entities"),
                    Key(key),
                ];
                report.miss(&path, ENTITY, key);
            }
        }
        for key in priorities.blocks.iter().flat_map(|map| map.keys()) {
            if !targets.block(&key.to_lowercase()) {
                let path = [
                    Key("categories"),
                    Index(position),
                    Key("priorities"),
                    Key("blocks"),
                    Key(key),
                ];
                report.miss(&path, BLOCK, key);
            }
        }
    }

    for (_, document) in &group.presets {
        let mut report = report(document, "minecraft:aim_assist_preset", &mut out);
        if let Some(settings) = &document.exclusion_settings {
            for (i, id) in settings.blocks.iter().flatten().enumerate() {
                if !targets.block(&id.as_str().to_lowercase()) {
                    let path = [Key("exclusion_settings"), Key("blocks"), Index(i)];
                    report.miss(&path, BLOCK, id.as_str());
                }
            }
            for (i, id) in settings.entities.iter().flatten().enumerate() {
                if !targets.entity(&entity_key(id.as_str())) {
                    let path = [Key("exclusion_settings"), Key("entities"), Index(i)];
                    report.miss(&path, ENTITY, id.as_str());
                }
            }
        }
        for (i, id) in document.liquid_targeting_list.iter().flatten().enumerate() {
            if !targets.item(&item_key(id.as_str())) {
                report.miss(&[Key("liquid_targeting_list"), Index(i)], ITEM, id.as_str());
            }
        }
        for (item, id) in document.item_settings.iter().flatten() {
            if !targets.item(&item_key(item)) {
                report.miss(&[Key("item_settings"), Key(item)], ITEM, item);
            }
            if !category(id.as_str()) {
                report.miss(&[Key("item_settings"), Key(item)], CATEGORY, id.as_str());
            }
        }
        // An absent member is the game's `minecraft:default` / `minecraft:empty_hand` at run
        // time and no reference of the document.
        for (key, id) in [
            ("default_item_settings", &document.default_item_settings),
            ("hand_settings", &document.hand_settings),
        ] {
            if let Some(id) = id.as_ref().filter(|id| !category(id.as_str())) {
                report.miss(&[Key(key)], CATEGORY, id.as_str());
            }
        }
    }
    out
}

/// The aim-assist preset a camera preset turns on (`aim_assist.preset`) must be an
/// aim-assist preset of the stack or one the game defines outside it; a miss is an
/// `UnresolvedReference` information at the member (the crate's own check; neither folder
/// checks it). Names are compared lower-cased, as vanilla registers every identifier and
/// reference.
///
/// `references` is one entry per camera preset that names an aim-assist preset: the preset's
/// document, whose origin locates the finding, and the name it gives.
pub fn link_camera_presets<'a, T: 'a>(
    references: impl IntoIterator<Item = (&'a Document<T>, &'a AimAssistPresetId)>,
    group: &AimAssist,
    targets: &dyn Targets,
) -> Diagnostics {
    let mut out = Diagnostics::default();
    let presets: HashSet<String> = group
        .presets
        .iter()
        .map(|(id, _)| id.as_str().to_lowercase())
        .collect();
    for (document, id) in references {
        let key = id.as_str().to_lowercase();
        if !presets.contains(&key) && !targets.preset(&key) {
            report(document, "minecraft:camera_preset", &mut out).miss(
                &[Key("aim_assist"), Key("preset")],
                <kind::AimAssistPreset as Kind>::NAME,
                id.as_str(),
            );
        }
    }
    out
}
