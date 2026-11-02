//! The references of an item to other items, and the walk over the items of the behaviour
//! stack that every item rule of this group uses.

use std::collections::HashSet;

use bedrock_addon_core::diagnostic::{
    DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::document::{ComponentMap, Origin};
use bedrock_addon_core::id::ItemId;
use bedrock_addon_core::pack_identity::PackSide;

use super::super::{At, Findings, LinkCx, Member, States, Target, item_descriptor, kinds, unresolved};
use crate::families::item::LegacyItemData;
use crate::families::item::component::{Food, ItemComponent, Repairable, Shooter, StorageItem, Tags};
use crate::families::item::trim::TrimData;

/// One item of the behaviour stack, stable or beta: what a rule reads of it.
#[derive(Debug)]
pub(super) struct Entry<'a> {
    /// Where the item is defined.
    pub(super) origin: &'a Origin,
    /// The findings of its load, which know where the file has the entries of a list an
    /// upgrade step left an entry out of.
    pub(super) found: &'a Diagnostics,
    /// Its identifier.
    pub(super) id: &'a ItemId,
    /// Its components.
    pub(super) components: &'a ComponentMap<ItemComponent>,
    /// What the legacy reader took from the document.
    #[cfg_attr(
        not(feature = "block"),
        allow(dead_code, reason = "read by the rules of `item` and `block`")
    )]
    pub(super) legacy: &'a LegacyItemData,
}

/// Every effective item of the behaviour stack: the stable ones, then the beta ones.
pub(super) fn items<'a>(cx: &LinkCx<'a>) -> impl Iterator<Item = Entry<'a>> + 'a {
    let registry = cx.behavior().items();
    let stable = registry.iter().map(|(id, document)| Entry {
        origin: document.origin(),
        found: document.diagnostics(),
        id,
        components: &document.value().components,
        legacy: &document.value().legacy,
    });
    let beta = registry.beta().into_iter().flat_map(|beta| {
        beta.iter().map(|(id, document)| Entry {
            origin: document.origin(),
            found: document.diagnostics(),
            id,
            components: &document.value().components,
            legacy: &document.value().legacy,
        })
    });
    stable.chain(beta)
}

/// The path of the component `key` of an item.
pub(super) fn component(key: &str) -> JsonPath {
    at!["minecraft:item", "components", key]
}

/// The items' categories as the catalogue's placement rules read them: the stack's item
/// registry behind the catalogue family's `Definitions` (a newtype, since trait and registry
/// both live in other crates).
#[cfg(feature = "crafting_catalog")]
#[derive(Debug)]
pub(super) struct ItemDefinitions<'a>(pub(super) &'a crate::document::Registry<crate::families::item::Item>);

#[cfg(feature = "crafting_catalog")]
impl crate::families::crafting_catalog::Definitions for ItemDefinitions<'_> {
    fn menu_category(
        &self,
        id: &bedrock_addon_core::id::ItemId,
    ) -> Option<(&bedrock_addon_core::shared::MenuCategory, bool)> {
        self.0.get(id).map(|document| {
            let description = &document.description;
            (
                &description.menu_category,
                description.category_is_written(),
            )
        })
    }
}

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    // An item the catalogue places in another creative category, or another group, than
    // its own definition names: the catalogue family's checker, over the stack's item
    // definitions.
    #[cfg(feature = "crafting_catalog")]
    {
        let stack = cx.behavior();
        out.extend(crate::families::crafting_catalog::link_placements(
            stack.crafting_catalog(),
            &ItemDefinitions(stack.items()),
        ));
    }
    client_items(cx, out);
    for item in items(cx) {
        descriptors(cx, out, &item);
        trim_template(out, &item);
    }
}

/// The tag a smithing template item carries.
const TRIM_TEMPLATES: &str = "minecraft:trim_templates";

/// An item of a pack that carries the tag `minecraft:trim_templates` is no smithing template,
/// since the game's list of patterns is compiled in: the tag has no effect. An item that is
/// the template of one of the game's patterns (a pack that redefines it) is one. The game
/// says nothing: an information.
fn trim_template(out: &mut Findings, item: &Entry<'_>) {
    let Some(tags) = item.components.get::<Tags>() else {
        return;
    };
    let tagged = tags.tags.iter().any(|tag| tag == TRIM_TEMPLATES);
    if tagged && TrimData::vanilla().pattern_for_item(item.id).is_none() {
        let place = At::new(item.origin, component("minecraft:tags"), Severity::Info);
        out.push(place.diagnostic(DiagnosticCode::NoEffect { text: None }));
    }
}

/// An item file of a resource pack (`items/`, the legacy format) whose identifier no
/// behaviour pack and not the game defines is vanilla's line at the content log's error
/// level, after which the definition is dropped and the load goes on: a kept error.
/// Of several resource packs with a file at one path the uppermost is read. A file that does
/// not load (a component-format item, which vanilla refuses in a resource pack) names no
/// definition to ignore.
fn client_items(cx: &LinkCx<'_>, out: &mut Findings) {
    let mut seen = HashSet::new();
    for pack in cx.resource().stack_packs() {
        for file in pack.files_below("items") {
            // The item loader reads the files whose extension is exactly `json`.
            let json = file.key.as_str().rsplit_once('.').is_some_and(|(_, ext)| ext == "json");
            if !json || !seen.insert(&file.key) {
                continue;
            }
            let Some(bytes) = cx.bytes(PackSide::Resource, &pack.pack, &file.path, out) else {
                continue;
            };
            let Some(identifier) = crate::families::item::loaded_identifier(&bytes, &pack.context) else {
                continue;
            };
            if let Target::Missing(hint) = cx.item_definition(&ItemId::parse(identifier.as_str())) {
                let origin = Origin::new(
                    pack.pack.clone(),
                    PackSide::Resource,
                    file.path.clone(),
                    None,
                    Vec::new(),
                );
                let text = VanillaText::ItemNotInBehaviorPack {
                    identifier: identifier.clone(),
                };
                let path = at!["minecraft:item", "description", "identifier"];
                // The game logs the line at the error level and skips the item: the resource
                // pack's definition is dropped and the load goes on. The link pass refuses
                // nothing, so the drop is told as a kept error.
                let place = At::new(&origin, path, Severity::Error).with_text(text);
                out.push(unresolved(place, kinds::ITEM, &identifier, hint).into_kept());
            }
        }
    }
}

/// The item descriptors of `minecraft:repairable`, `minecraft:shooter`,
/// `minecraft:food` and `minecraft:storage_item` name items.
fn descriptors(cx: &LinkCx<'_>, out: &mut Findings, item: &Entry<'_>) {
    if let Some(repairable) = item.components.get::<Repairable>() {
        for (entry_index, entry) in repairable.repair_items.iter().enumerate() {
            for (index, descriptor) in entry.items.iter().enumerate() {
                let path = at![&component("minecraft:repairable"); "repair_items", entry_index, "items", index];
                // At the index the file has the entry at: the step that types the list
                // of a file below 1.19.83 leaves out an entry the newest type does not take.
                let path = item.found.place_of(&path);
                item_descriptor(cx, out, descriptor, descriptor_of(item.origin, &path));
            }
        }
    }
    if let Some(shooter) = item.components.get::<Shooter>() {
        for (index, ammunition) in shooter.ammunition.iter().enumerate() {
            let path = at![&component("minecraft:shooter"); "ammunition", index, "item"];
            item_descriptor(cx, out, &ammunition.item, descriptor_of(item.origin, &path));
        }
    }
    if let Some(food) = item.components.get::<Food>()
        && let Some(descriptor) = &food.using_converts_to
    {
        let path = at![&component("minecraft:food"); "using_converts_to"];
        item_descriptor(cx, out, descriptor, descriptor_of(item.origin, &path));
    }
    if let Some(storage) = item.components.get::<StorageItem>() {
        for (key, list) in [
            ("banned_items", &storage.banned_items),
            ("allowed_items", &storage.allowed_items),
        ] {
            for (index, descriptor) in list.iter().enumerate() {
                let path = at![&component("minecraft:storage_item"); key, index];
                item_descriptor(cx, out, descriptor, descriptor_of(item.origin, &path));
            }
        }
    }
}

/// A descriptor of `origin` at `path`: a name nothing defines is the linked-asset check's
/// error, a state of a block of the stack the crate's own information.
pub(super) fn descriptor_of<'r>(origin: &'r Origin, path: &'r JsonPath) -> Member<'r> {
    Member {
        origin,
        path,
        // The linked-asset check's grading: no path of the resolver reads another.
        severity: Severity::Error,
        validator: Some(1),
        states: States::Lint,
    }
}
