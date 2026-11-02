// Armour trims: a pattern (from a smithing template item) plus a material (from an ingredient
// item) on an armour item stack. Bedrock compiles both lists into the engine and no pack can
// add, remove or change an entry, so this module is a data source, not a pack family: no
// `Family`, no `Registry`, no stack accessor, no chain and no step file.
// The module's rustdoc is on its declaration in `lib.rs`.

mod filter;
mod packet;
mod table;
mod texts;

pub use filter::ItemTags;
pub use packet::{MaterialEntry, PatternEntry};

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::id::{ItemId, TrimMaterialId, TrimPatternId};
use bedrock_addon_core::version::SemVersion;

/// One armour trim pattern (vanilla `TrimPattern`): its identifier and the smithing template
/// item that applies it.
///
/// Vanilla keeps one pattern per template item and one template item per pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrimPattern {
    /// The pattern, a bare name without namespace (`ward`): what an item's `Trim.Pattern` tag
    /// and a `set_armor_trim` loot function name.
    pub pattern_id: TrimPatternId,
    /// The smithing template item (`minecraft:ward_armor_trim_smithing_template`).
    pub item_name: ItemId,
}

impl TrimPattern {
    /// The engine version whose vanilla packs first ship this pattern: 1.20.0 for the first
    /// sixteen, 1.21.0 for `flow` and `bolt`; `None` for a pattern the vanilla table lacks.
    ///
    /// Vanilla has no per-entry version and always sends its whole table; this is for a tool
    /// that builds the lists for an older client.
    pub fn since_engine(&self) -> Option<SemVersion> {
        table::pattern_since(self.pattern_id.as_str())
    }
}

/// One armour trim material (vanilla `TrimMaterial`): its identifier, the colour of its hover
/// text and the ingredient item that applies it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrimMaterial {
    /// The material, a bare name without namespace (`quartz`): what an item's `Trim.Material`
    /// tag and a `set_armor_trim` loot function name.
    pub material_id: TrimMaterialId,
    /// The formatting code that colours the material's hover text (`§h`).
    pub color: FormatCode,
    /// The ingredient item (`minecraft:quartz`).
    pub item_name: ItemId,
}

impl TrimMaterial {
    /// The engine version whose vanilla packs first ship this material: 1.20.0 for the first
    /// ten, 1.21.50 for `resin`; `None` for a material the vanilla table lacks.
    ///
    /// Vanilla has no per-entry version and always sends its whole table; this is for a tool
    /// that builds the lists for an older client.
    pub fn since_engine(&self) -> Option<SemVersion> {
        table::material_since(self.material_id.as_str())
    }
}

/// A text formatting code as vanilla stores it: the section sign and one letter (`§h`).
///
/// The text is kept as given, so a code a server sent in another shape is not lost;
/// [`letter`](Self::letter) says whether it has the usual two-character shape.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FormatCode(String);

impl FormatCode {
    /// The code with this text, kept as given (`"§h"`).
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    /// The text as given.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The character after the section sign, when the text is exactly `§` and one character.
    pub fn letter(&self) -> Option<char> {
        let mut rest = self.0.strip_prefix('§')?.chars();
        let letter = rest.next()?;
        rest.next().is_none().then_some(letter)
    }
}

/// Both trim registries, as vanilla holds them once a world has started.
///
/// The patterns are two maps, template item → pattern and pattern → template item; the
/// materials are a list in build order, searched front to back. [`vanilla`](Self::vanilla) is
/// the engine's table, [`for_world`](Self::for_world) applies the world gate,
/// [`filtered`](Self::filtered) the template-item filter of world start and
/// [`from_packet`](Self::from_packet) builds the client's registries from a server's lists.
///
/// Two values are equal when their maps and their material lists are; the order of the
/// patterns is not compared (vanilla's are hash maps) and neither are the findings.
#[derive(Clone, Debug, Default)]
pub struct TrimData {
    /// The patterns by template item, as vanilla keeps them.
    by_item: IndexMap<ItemId, TrimPattern>,
    /// The patterns by pattern identifier, as vanilla keeps them.
    by_pattern: IndexMap<TrimPatternId, TrimPattern>,
    /// The materials, in build order.
    materials: Vec<TrimMaterial>,
    /// What building this value found.
    diagnostics: Diagnostics,
}

impl TrimData {
    /// The tables compiled into the engine: 18 patterns and 11 materials, in vanilla's build
    /// order.
    pub fn vanilla() -> &'static TrimData {
        table::vanilla()
    }

    /// The pattern a smithing template item applies.
    pub fn pattern_for_item(&self, item: &ItemId) -> Option<&TrimPattern> {
        self.by_item.get(item)
    }

    /// The pattern with this identifier (`ward`), compared as written.
    pub fn pattern(&self, id: &str) -> Option<&TrimPattern> {
        self.by_pattern
            .values()
            .find(|pattern| pattern.pattern_id.as_str() == id)
    }

    /// The first material whose ingredient is this item.
    pub fn material_for_item(&self, item: &ItemId) -> Option<&TrimMaterial> {
        self.materials
            .iter()
            .find(|material| material.item_name == *item)
    }

    /// The first material with this identifier (`quartz`), compared as written.
    pub fn material(&self, id: &str) -> Option<&TrimMaterial> {
        self.materials
            .iter()
            .find(|material| material.material_id.as_str() == id)
    }

    /// The patterns, one per template item: in build order for the vanilla table, by template
    /// item for one built from a packet.
    pub fn patterns(&self) -> impl ExactSizeIterator<Item = &TrimPattern> + '_ {
        self.by_item.values()
    }

    /// The materials, in build order.
    pub fn materials(&self) -> &[TrimMaterial] {
        &self.materials
    }

    /// What building this value found: a pattern [`filtered`](Self::filtered) left out, a
    /// template item [`from_packet`](Self::from_packet) met twice. Empty for the vanilla table.
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    /// Writes both maps for `pattern`, as vanilla's plain map assignment does: the entry of
    /// its template item and the entry of its identifier are replaced, each in its own map.
    /// Returns whether the template item already had a pattern.
    fn insert_pattern(&mut self, pattern: TrimPattern) -> bool {
        self.by_pattern
            .insert(pattern.pattern_id.clone(), pattern.clone());
        self.by_item
            .insert(pattern.item_name.clone(), pattern)
            .is_some()
    }
}

/// The maps and the material list; not the pattern order and not the findings.
impl PartialEq for TrimData {
    fn eq(&self, other: &Self) -> bool {
        self.by_item == other.by_item
            && self.by_pattern == other.by_pattern
            && self.materials == other.materials
    }
}

impl Eq for TrimData {}
