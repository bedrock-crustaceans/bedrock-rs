//! The aim-assist references the link pass resolves: the doc-hidden checker that `bedrock_addon`'s
//! `StackSet::link` calls with the other families' registries. Every miss is an
//! `UnresolvedReference` finding in `Phase::Link`; the game only records the names with its
//! linked-asset validator, so the crate's own check is what reports them.

use std::collections::{HashMap, HashSet};

use super::{AimAssist, AimAssistPreset};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Phase,
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

/// An `UnresolvedReference` information in the file of `origin` for the `kind` named `id` at
/// `at`. `link()` makes the references the game's linked-asset
/// validator holds errors with its line (`with_validator_line`), and a preset's
/// `aim_assist.preset` is checked by neither folder.
fn miss(out: &mut Diagnostics, origin: &Origin, at: JsonPath, kind: &'static str, id: &str) {
    out.push(
        Diagnostic::info(DiagnosticCode::UnresolvedReference { text: None })
            .at(at)
            .phase(Phase::Link)
            .in_file(origin)
            .detail(DiagnosticDetail::Reference {
                kind,
                id: id.to_owned(),
                hint: None,
            }),
    );
}

/// The payload of a categories file.
const CATEGORIES: &str = "minecraft:aim_assist_categories";
/// The payload of a preset file.
const PRESET: &str = "minecraft:aim_assist_preset";

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
        let priorities = &document.priorities;
        let priorities_at = JsonPath::of_keys([CATEGORIES, "categories"])
            .index(position)
            .child("priorities");
        for key in priorities.entities.iter().flat_map(|map| map.keys()) {
            if !targets.entity(&entity_key(key)) {
                miss(
                    &mut out,
                    origin,
                    priorities_at.with(&["entities", key]),
                    ENTITY,
                    key,
                );
            }
        }
        for key in priorities.blocks.iter().flat_map(|map| map.keys()) {
            if !targets.block(&key.to_lowercase()) {
                miss(
                    &mut out,
                    origin,
                    priorities_at.with(&["blocks", key]),
                    BLOCK,
                    key,
                );
            }
        }
    }

    for (_, document) in &group.presets {
        link_preset(document, targets, &category, &mut out);
    }
    out
}

/// The findings of one preset's exclusions, liquid items, item settings and category
/// references; `category` says whether a category name resolves.
fn link_preset(
    document: &Document<AimAssistPreset>,
    targets: &dyn Targets,
    category: &dyn Fn(&str) -> bool,
    out: &mut Diagnostics,
) {
    let origin = document.origin();
    let preset = JsonPath::new().child(PRESET);
    if let Some(settings) = &document.exclusion_settings {
        for (i, id) in settings.blocks.iter().flatten().enumerate() {
            if !targets.block(&id.as_str().to_lowercase()) {
                miss(
                    out,
                    origin,
                    preset.with(&["exclusion_settings", "blocks"]).index(i),
                    BLOCK,
                    id.as_str(),
                );
            }
        }
        for (i, id) in settings.entities.iter().flatten().enumerate() {
            if !targets.entity(&entity_key(id.as_str())) {
                miss(
                    out,
                    origin,
                    preset.with(&["exclusion_settings", "entities"]).index(i),
                    ENTITY,
                    id.as_str(),
                );
            }
        }
    }
    for (i, id) in document.liquid_targeting_list.iter().flatten().enumerate() {
        if !targets.item(&item_key(id.as_str())) {
            miss(
                out,
                origin,
                preset.child("liquid_targeting_list").index(i),
                ITEM,
                id.as_str(),
            );
        }
    }
    for (item, id) in document.item_settings.iter().flatten() {
        if !targets.item(&item_key(item)) {
            miss(
                out,
                origin,
                preset.with(&["item_settings", item]),
                ITEM,
                item,
            );
        }
        if !category(id.as_str()) {
            miss(
                out,
                origin,
                preset.with(&["item_settings", item]),
                CATEGORY,
                id.as_str(),
            );
        }
    }
    // An absent member is the game's `minecraft:default` / `minecraft:empty_hand` at run
    // time and no reference of the document.
    for (key, id) in [
        ("default_item_settings", &document.default_item_settings),
        ("hand_settings", &document.hand_settings),
    ] {
        if let Some(id) = id.as_ref().filter(|id| !category(id.as_str())) {
            miss(out, origin, preset.child(key), CATEGORY, id.as_str());
        }
    }
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
            miss(
                &mut out,
                document.origin(),
                JsonPath::new()
                    .child("minecraft:camera_preset")
                    .child("aim_assist")
                    .child("preset"),
                <kind::AimAssistPreset as Kind>::NAME,
                id.as_str(),
            );
        }
    }
    out
}
