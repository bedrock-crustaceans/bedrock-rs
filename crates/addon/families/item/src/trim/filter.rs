//! The two gates of world start: the world's base-game version, and the filter that keeps
//! only the patterns whose template item exists and carries the `minecraft:trim_templates`
//! tag.

use std::sync::OnceLock;

use super::TrimData;
use crate::Item;
use crate::component::Tags;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticDetail, Phase};
use bedrock_addon_core::document::Registry;
use bedrock_addon_core::id::ItemId;
use bedrock_addon_core::version::{BaseGameVersion, SemVersion};

/// The base-game version of the Trails & Tales update, 1.20.0, which introduced armour trims.
/// A world is compatible with the trim table from this version on.
const TRAILS_AND_TALES: SemVersion = SemVersion::new(1, 20, 0);

/// The tag a smithing template item must carry to be a trim template.
const TRIM_TEMPLATES: &str = "minecraft:trim_templates";

/// The tags of one item, as the template filter reads them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ItemTags {
    /// The tags, as written (`minecraft:trim_templates`).
    pub tags: Vec<String>,
}

impl ItemTags {
    /// The item carries `tag`, compared as written.
    pub fn contains(&self, tag: &str) -> bool {
        self.tags.iter().any(|own| own == tag)
    }
}

impl TrimData {
    /// The vanilla table a world of this base-game version starts with: the whole table when
    /// the world is compatible with 1.20.0 (Trails & Tales), otherwise empty registries, in
    /// which every lookup fails.
    ///
    /// A world of every version (`*`) or without a version is compatible, as for the crate's
    /// other base-game gates ([`BaseGameVersion::includes`]).
    // The game initialises both the pattern and the material registry only when the world's
    // base-game version is compatible with Trails & Tales.
    pub fn for_world(base_game_version: &BaseGameVersion) -> &'static TrimData {
        static EMPTY: OnceLock<TrimData> = OnceLock::new();
        if base_game_version.includes(&TRAILS_AND_TALES) {
            TrimData::vanilla()
        } else {
            EMPTY.get_or_init(TrimData::default)
        }
    }

    /// These registries after vanilla's template filter: a pattern stays only when its template
    /// item exists and carries the tag `minecraft:trim_templates`. The materials are not
    /// filtered.
    ///
    /// `items` are the stack's items, whose tags are those of the `minecraft:tags` component;
    /// an item the stack does not define is asked of `vanilla_items`, which gives the tags of
    /// a built-in item or `None` when there is no such item.
    ///
    /// Each pattern left out adds an `UnresolvedReference` info naming its template item to
    /// [`diagnostics`](Self::diagnostics), after the findings this value already has; the game
    /// reports nothing for it. Both maps are filled from the template items that
    /// pass.
    #[must_use]
    pub fn filtered(
        &self,
        items: &Registry<Item>,
        vanilla_items: impl Fn(&ItemId) -> Option<ItemTags>,
    ) -> TrimData {
        let mut filtered = TrimData {
            materials: self.materials.clone(),
            diagnostics: self.diagnostics.clone(),
            ..TrimData::default()
        };
        // Vanilla fills both maps from the pairs that pass.
        for pattern in self.by_item.values() {
            match template(&pattern.item_name, items, &vanilla_items) {
                Template::Tagged => {
                    filtered.insert_pattern(pattern.clone());
                }
                Template::Untagged => filtered
                    .diagnostics
                    .push(left_out("trim template", &pattern.item_name)),
                Template::Missing => filtered
                    .diagnostics
                    .push(left_out("item", &pattern.item_name)),
            }
        }
        filtered
    }
}

/// The finding for a pattern the filter leaves out: nothing of `kind` is named `item`. The
/// crate reports it as an information; the game reports nothing for it.
fn left_out(kind: &'static str, item: &ItemId) -> Diagnostic {
    Diagnostic::of(&core_codes::UNRESOLVED_REFERENCE)
        .phase(Phase::Link)
        .detail(DiagnosticDetail::Reference {
            kind,
            id: item.as_str().to_owned(),
            hint: None,
        })
}

/// What the filter finds for a template item.
enum Template {
    /// No such item.
    Missing,
    /// The item exists without the tag.
    Untagged,
    /// The item exists and carries the tag.
    Tagged,
}

/// Looks `item` up as the game's template filter does: by name in the item registry, then
/// for the `minecraft:trim_templates` tag. The stack's definition of an item is the
/// registered one; its tags are those of its `minecraft:tags` component.
fn template(
    item: &ItemId,
    items: &Registry<Item>,
    vanilla_items: &impl Fn(&ItemId) -> Option<ItemTags>,
) -> Template {
    let tagged = match items.get(item) {
        Some(document) => Some(
            document
                .components
                .get::<Tags>()
                .is_some_and(|own| own.tags.iter().any(|tag| tag == TRIM_TEMPLATES)),
        ),
        None => vanilla_items(item).map(|own| own.contains(TRIM_TEMPLATES)),
    };
    match tagged {
        Some(true) => Template::Tagged,
        Some(false) => Template::Untagged,
        None => Template::Missing,
    }
}
