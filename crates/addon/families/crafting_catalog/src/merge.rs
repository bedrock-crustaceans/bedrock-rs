//! The catalogue's merge strategy: files merge in load order, the vanilla file first and the
//! top pack last, into four fixed categories; a problem drops one item, never a group or a
//! file.

use std::collections::HashSet;

use super::{
    CatalogItem, CatalogKey, Category, CategoryName, CraftingCatalog, Group, GroupIdentifier,
};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, JsonPath, Phase, VanillaText,
};
use bedrock_addon_core::document::{Document, Extra, MergeCx};
use bedrock_addon_core::id::ItemId;

/// The payload key: where every path of a merge finding starts.
const PAYLOAD_KEY: &str = "minecraft:crafting_items_catalog";

/// The item registry the merge resolves names against.
pub(super) trait Items {
    /// Whether `id` (compared without an aux value) is a registered item or block.
    fn is_registered(&self, id: &ItemId) -> bool;
}

/// The merged catalogue before any file: the four categories, in vanilla's order, empty.
pub(super) fn empty() -> CraftingCatalog {
    CraftingCatalog {
        categories: CategoryName::ALL
            .into_iter()
            .map(|category_name| Category {
                category_name,
                groups: Vec::new(),
                extra: Extra::default(),
            })
            .collect(),
        extra: Extra::default(),
    }
}

/// `text` names an aux value that is none: a third `:`-separated part that is no integer.
fn has_bad_aux(name: &ItemId) -> bool {
    name.as_str().matches(':').count() >= 2 && name.aux().is_none()
}

/// The state of one stack's merge.
pub(super) struct Merger<'a> {
    /// The registered items.
    items: &'a dyn Items,
    /// Every item placed so far, by identifier and aux value, across all categories, groups
    /// and files (vanilla's `mUsedItems`).
    used: HashSet<(ItemId, i16)>,
}

impl<'a> Merger<'a> {
    /// A merge that resolves names against `items`.
    pub(super) fn new(items: &'a dyn Items) -> Self {
        Self {
            items,
            used: HashSet::new(),
        }
    }

    /// Merges one file into `merged`, in document order.
    pub(super) fn layer(
        &mut self,
        merged: &mut CraftingCatalog,
        layer: &Document<CraftingCatalog>,
        cx: &mut MergeCx<'_, CatalogKey>,
    ) {
        let mut at = JsonPath::of_keys([PAYLOAD_KEY]);
        at.push("categories");
        for (index, category) in layer.categories.iter().enumerate() {
            // Every entry, repeated or not, goes to the one category its name selects.
            let Some(bucket) = merged.categories.get_mut(category.category_name as usize) else {
                continue;
            };
            let at = at.index(index).child("groups");
            for (index, group) in category.groups.iter().enumerate() {
                let mut at = at.index(index);
                let target = self.target(bucket, group, &at, cx);
                at.push("items");
                for (index, item) in group.items.iter().enumerate() {
                    let at = at.index(index);
                    self.item(&mut bucket.groups[target], item, at, cx);
                }
            }
        }
    }

    /// The index of the group of `bucket` the items of `group` are added to: for a named
    /// group, the group of that name (compared byte for byte) when the category has one, which
    /// keeps its place and its icon; else a new group at the end. Loose items always start a
    /// new group.
    fn target(
        &mut self,
        bucket: &mut Category,
        group: &Group,
        at: &JsonPath,
        cx: &mut MergeCx<'_, CatalogKey>,
    ) -> usize {
        let group_identifier = group.group_identifier.as_ref().map(|identifier| {
            (
                identifier,
                bucket.groups.iter().position(|merged| {
                    merged
                        .group_identifier
                        .as_ref()
                        .is_some_and(|of_merged| of_merged.name == identifier.name)
                }),
            )
        });
        let group_identifier = match group_identifier {
            // The later definition's icon is not looked at.
            Some((_, Some(existing))) => return existing,
            Some((identifier, None)) => {
                cx.claim(CatalogKey::Group {
                    category: bucket.category_name,
                    name: identifier.name.clone(),
                });
                let at = at.child("group_identifier").child("icon");
                Some(GroupIdentifier {
                    name: identifier.name.clone(),
                    icon: identifier
                        .icon
                        .as_ref()
                        .filter(|icon| self.resolves(icon, at, cx))
                        .map(placed),
                    extra: Extra::default(),
                })
            }
            None => None,
        };
        bucket.groups.push(Group {
            group_identifier,
            items: Vec::new(),
            extra: Extra::default(),
        });
        bucket.groups.len() - 1
    }

    /// Whether `item` names a registered item with a valid aux value; reports why not.
    /// Vanilla's item lookup by name logs a bad aux value and gives no item; an
    /// unknown name gives no item without a message (the crate reports a warning).
    fn resolves(&self, item: &CatalogItem, at: JsonPath, cx: &mut MergeCx<'_, CatalogKey>) -> bool {
        if has_bad_aux(&item.name) {
            cx.push_diagnostic(
                Diagnostic::error(DiagnosticCode::InvalidValue {
                    text: Some(VanillaText::Formatted {
                        template: "Invalid aux value in item ID \"{}\"",
                        args: vec![item.name.as_str().to_owned()],
                    }),
                })
                .at(at)
                .phase(Phase::Validate)
                .in_file(cx.origin()),
            );
            return false;
        }
        if !self.items.is_registered(&item.name) {
            // The lookup drops the item without a message; the game's
            // link pass (`LinkedAssetValidator`) holds every catalogue item and logs the miss
            // at the content log's error level, dropping nothing more.
            let text = bedrock_addon_core::diagnostic::validator::validator_text(
                bedrock_addon_core::diagnostic::validator::Validator::Missing,
                &at,
                1,
                item.name.as_str(),
            );
            let unresolved =
                Diagnostic::error(DiagnosticCode::UnresolvedReference { text: Some(text) })
                    .at(at)
                    .phase(Phase::Validate)
                    .in_file(cx.origin());
            cx.push_diagnostic(unresolved.detail(DiagnosticDetail::Reference {
                kind: "item",
                id: item.name.as_str().to_owned(),
                hint: None,
            }));
            return false;
        }
        true
    }

    /// Adds `item` to `group`, unless it does not resolve or is placed already: the first
    /// placement of an identifier and aux value wins, in the whole catalogue.
    fn item(
        &mut self,
        group: &mut Group,
        item: &CatalogItem,
        at: JsonPath,
        cx: &mut MergeCx<'_, CatalogKey>,
    ) {
        if !self.resolves(item, at.clone(), cx) {
            return;
        }
        let aux = item.name.aux().unwrap_or(0);
        if !self.used.insert((item.name.clone(), aux)) {
            cx.push_diagnostic(Diagnostic::error(DiagnosticCode::IdentifierCollision {
                    text: Some(VanillaText::Formatted {
                        template: "You can only add an item to the crafting item catalog once. Found duplicate {}",
                        args: vec![item.name.as_str().to_owned()],
                    }),
                }).at(at).phase(Phase::Validate).in_file(cx.origin()));
            return;
        }
        cx.claim(CatalogKey::Item {
            id: item.name.clone(),
            aux,
        });
        group.items.push(placed(item));
    }
}

/// `item` as the merged catalogue holds it: its name, without the unknown members of its
/// file (the layers keep those).
fn placed(item: &CatalogItem) -> CatalogItem {
    CatalogItem {
        name: item.name.clone(),
        extra: Extra::default(),
    }
}
