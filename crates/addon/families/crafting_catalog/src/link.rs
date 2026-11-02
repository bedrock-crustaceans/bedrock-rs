//! The catalogue's link rules (the placement checks and the group-name lint): crate-private
//! checkers `StackSet::link` calls (`pack::link::content`) with the stack's item definitions and
//! its `.lang` keys. Nothing here changes the merged catalogue.

use std::collections::HashMap;

use super::{CatalogItem, CatalogKey, Category, CraftingCatalog, Group};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Location, Phase, Severity,
    VanillaText,
};
use bedrock_addon_core::document::Merged;
use bedrock_addon_core::id::ItemId;
use bedrock_addon_core::shared::{CreativeCategory, MenuCategory};

/// The item definitions the placement rules compare against.
pub trait Definitions {
    /// The `menu_category` of the definition of `id` (compared without an aux value), when
    /// a pack of the stack defines the item, and whether its author wrote the category
    /// (`ItemDescription::category_is_written`).
    fn menu_category(&self, id: &ItemId) -> Option<(&MenuCategory, bool)>;
}

/// The namespace whose items vanilla exempts from the placement rules: a 10-byte prefix
/// compare on the catalogue string.
const VANILLA_PREFIX: &str = "minecraft:";

/// Where the entries of the merged catalogue are written, by the file that placed them: what
/// a link finding names as its place. Built for a file when the first finding in it is told,
/// so that a checker costs the catalogue once, however many findings it tells.
struct Places<'m> {
    merged: &'m Merged<CraftingCatalog>,
    /// Per layer of `merged`, in its order: the path of each entry of that file, the first
    /// of a key kept.
    by_layer: Vec<Option<HashMap<CatalogKey, JsonPath>>>,
}

impl<'m> Places<'m> {
    fn new(merged: &'m Merged<CraftingCatalog>) -> Self {
        Self {
            merged,
            by_layer: merged.layers().iter().map(|_| None).collect(),
        }
    }

    /// Every entry of `file` with its path: the first item of an identifier and aux value
    /// (the one the merge placed) and the name of the first group of a name in its category.
    fn written(file: &CraftingCatalog) -> HashMap<CatalogKey, JsonPath> {
        let mut written = HashMap::new();
        for (category_index, category) in file.categories.iter().enumerate() {
            for (group_index, group) in category.groups.iter().enumerate() {
                let mut at = JsonPath::new();
                at.push("minecraft:crafting_items_catalog");
                at.push("categories");
                at.push_index(category_index);
                at.push("groups");
                at.push_index(group_index);
                if let Some(name) = group.name() {
                    let key = CatalogKey::Group {
                        category: category.category_name,
                        name: name.to_owned(),
                    };
                    written.entry(key).or_insert_with(|| {
                        let mut at = at.clone();
                        at.push("group_identifier");
                        at.push("name");
                        at
                    });
                }
                for (index, item) in group.items.iter().enumerate() {
                    written.entry(item_key(item)).or_insert_with(|| {
                        let mut at = at.clone();
                        at.push("items");
                        at.push_index(index);
                        at
                    });
                }
            }
        }
        written
    }

    /// A link finding at the entry `key` of the merged catalogue: in the file that placed
    /// it, at the path it is written at there.
    fn finding(
        &mut self,
        code: DiagnosticCode,
        severity: Severity,
        key: &CatalogKey,
    ) -> Diagnostic {
        let merged = self.merged;
        let origin = merged.origin(key);
        let layer = origin.and_then(|origin| {
            merged
                .layers()
                .iter()
                .position(|layer| layer.origin() == origin)
        });
        let at = layer
            .and_then(|index| {
                let file = merged.layers().get(index)?;
                let written = self
                    .by_layer
                    .get_mut(index)?
                    .get_or_insert_with(|| Self::written(file.value()));
                written.get(key).cloned()
            })
            .unwrap_or_default();
        Diagnostic::new(
            code,
            severity,
            Phase::Link,
            Location::new(
                origin.map(|origin| origin.pack.clone()),
                origin.map(|origin| origin.path.clone()),
                at,
                None,
            ),
        )
    }
}

/// The key of a placed item.
fn item_key(item: &CatalogItem) -> CatalogKey {
    CatalogKey::Item {
        id: item.name.clone(),
        aux: item.name.aux().unwrap_or(0),
    }
}

/// The placement rules for one placed item.
fn placement(
    index: &mut Places<'_>,
    category: &Category,
    group: &Group,
    item: &CatalogItem,
    definitions: &dyn Definitions,
    out: &mut Diagnostics,
) {
    let written = item.name.as_str();
    if written.starts_with(VANILLA_PREFIX) {
        return;
    }
    let Some((own, category_written)) = definitions.menu_category(&item.name) else {
        return;
    };
    // The definition gave no `menu_category`: vanilla's `Undefined`, and from 1.21.110 the
    // default `none` (`ItemCommandOnly`), which the item records as unwritten. A `none` its
    // author wrote is checked like any other category.
    let Some(own_category) = own
        .category
        .filter(|category| *category != CreativeCategory::Undefined && category_written)
    else {
        return;
    };
    let placed = category.category_name.creative_category();
    // The game writes both lines (the category's and the group's) to the content log at its
    // warning level, so both findings are warnings.
    let mut warn = |template, args: Vec<String>| {
        index.finding(
            DiagnosticCode::InvalidValue {
                text: Some(VanillaText::Formatted { template, args }),
            },
            Severity::Warning,
            &item_key(item),
        )
    };
    if own_category != placed {
        // The names are `stringFromCreativeItemCategory`'s: the menu category spellings.
        let name = |category: CreativeCategory| category.name().unwrap_or_default().to_owned();
        out.push(warn(
            "The item {} was created with the category set to '{}', and is now being set to '{}'",
            vec![written.to_owned(), name(own_category), name(placed)],
        ));
    }
    let own_group = own.group.as_deref().filter(|group| !group.is_empty());
    let target = group.group_identifier.as_ref().map(|i| i.name.as_str());
    if let (Some(own_group), Some(target)) = (own_group, target)
        && own_group != target
    {
        out.push(warn(
            "The item {} was created with the group set to '{}', but is now being set to '{}'",
            vec![written.to_owned(), own_group.to_owned(), target.to_owned()],
        ));
    }
}

/// An item whose own definition names another creative category, or
/// another group, than the catalogue places it in is a warning with vanilla's text; the item
/// stays where the catalogue put it. Skipped for a catalogue string starting with
/// `minecraft:` and for a definition without a category (`Undefined`, or `none`).
pub fn link_placements(
    merged: &Merged<CraftingCatalog>,
    definitions: &dyn Definitions,
) -> Diagnostics {
    let mut out = Diagnostics::default();
    let mut places = Places::new(merged);
    for category in &merged.categories {
        for group in &category.groups {
            for item in &group.items {
                placement(&mut places, category, group, item, definitions, &mut out);
            }
        }
    }
    out
}

/// The group-name lint: a named group
/// outside the `minecraft` namespace whose name, which is its localisation key, no loaded
/// `.lang` file defines is an `UnresolvedLangKey` info.
pub fn link_group_names(
    merged: &Merged<CraftingCatalog>,
    has_key: &dyn Fn(&str) -> bool,
) -> Diagnostics {
    let mut out = Diagnostics::default();
    let mut places = Places::new(merged);
    for category in &merged.categories {
        let names = category
            .groups
            .iter()
            .filter_map(|group| group.group_identifier.as_ref())
            .map(|identifier| identifier.name.as_str());
        for name in names {
            if name.starts_with(VANILLA_PREFIX) || has_key(name) {
                continue;
            }
            let key = CatalogKey::Group {
                category: category.category_name,
                name: name.to_owned(),
            };
            let mut lint = places.finding(DiagnosticCode::UnresolvedLangKey, Severity::Info, &key);
            lint.detail = Some(Box::new(DiagnosticDetail::Reference {
                kind: "lang key",
                id: name.to_owned(),
                hint: None,
            }));
            out.push(lint);
        }
    }
    out
}
