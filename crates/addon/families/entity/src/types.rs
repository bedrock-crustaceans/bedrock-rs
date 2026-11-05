//! The family's types and their constructors.

use std::ops::Deref;

use bedrock_addon_core::document::ComponentMap;
use bedrock_addon_core::id::EntityId;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::MolangSource;
use bedrock_addon_core::schema;
use facet::Facet;
use indexmap::IndexMap;

use super::component::EntityComponent;

/// A behaviour-pack entity definition (`entities/*.json`, payload `minecraft:entity`), in the
/// newest format version (1.26.60).
///
/// Older documents are upgraded to it step by step ([`history`](crate::history)); a document written with
/// `"format_version": "beta"` is a [`Beta`] instead.
///
/// Built from [`Entity::new`] with struct-update syntax, as from any other crate:
///
/// ```
/// use bedrock_addon_family_entity::{Description, Entity, Scripts};
/// use bedrock_addon_core::id::EntityId;
///
/// let id = EntityId::parse("demo:walker");
/// let scripts = Scripts {
///     animate: Vec::new(),
///     ..Scripts::default()
/// };
/// let entity = Entity {
///     description: Description {
///         is_spawnable: Some(true),
///         scripts: Some(scripts),
///         ..Description::new(id.clone())
///     },
///     ..Entity::new(id)
/// };
/// assert_eq!(entity.description.identifier.as_str(), "demo:walker");
/// assert!(entity.components.is_empty());
/// ```
#[doc(alias = "minecraft:entity")]
#[derive(Clone, Debug, PartialEq)]
pub struct Entity {
    /// `description`: the identifier and the entity-wide settings.
    pub description: Description,
    /// `components`: the components and AI goals the entity always has, in file order.
    pub components: ComponentMap<EntityComponent>,
    /// `component_groups`: named sets of components the entity's events add and remove.
    pub component_groups: IndexMap<String, ComponentGroup>,
    /// `events`: what the entity does when an event fires, by event name.
    pub events: IndexMap<String, Event>,
    /// `brain`: the memories of the Brain Prototype experiment (format 1.21.130 onward).
    pub brain: Option<Brain>,
    /// `upgrades`: the handlers `description.entity_version` names, each an event node run
    /// once when a saved entity of a lower version loads (format 1.26.20 onward).
    pub upgrades: IndexMap<String, Value>,
    /// `__deprecated`: what an older document wrote that this version does not read (the
    /// `aliases` of a document below 1.21.10), kept for write-back.
    pub deprecated: Option<DeprecatedData>,
    /// Which optional members the document wrote, so an empty one is written back; not part of
    /// equality. Public only so that struct-update syntax (`..Entity::new(id)`) works outside
    /// the crate, where a `pub(crate)` field would refuse both a struct literal and the update
    /// syntax; `Default` records nothing written.
    #[doc(hidden)]
    pub written: Written,
}

/// Which optional members a document wrote: an empty map keeps the author's
/// presence. Not part of a value's equality: two entities equal in content are equal however
/// their documents spelled an empty member. Its flags are the crate's; outside it the record is
/// only carried along, through struct-update syntax.
#[doc(hidden)]
#[derive(Clone, Debug, Default)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "one independent presence flag per optional member"
)]
pub struct Written {
    /// `components`.
    pub(crate) components: bool,
    /// `component_groups`.
    pub(crate) component_groups: bool,
    /// `events`.
    pub(crate) events: bool,
    /// `scripts.animate`.
    pub(crate) animate: bool,
    /// The index each held entry of `scripts.animate` has in the file, when the reader left
    /// an element of the file out of the list; `None` when the list is the file's.
    pub(crate) animate_places: Option<Box<[usize]>>,
}

impl PartialEq for Written {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// `description` of an [`Entity`]: its identifier and entity-wide settings. Vanilla reads a
/// closed set of keys; any other key is reported, and vanilla's own `is_experimental` is kept
/// among the document's unknown members (`Document::unknown`).
#[derive(Clone, Debug, PartialEq)]
pub struct Description {
    /// `identifier`: `namespace:name`. A `minecraft:` identifier must override a vanilla
    /// entity, and from format 1.20.30 the name may not start with a digit.
    pub identifier: EntityId,
    /// `runtime_identifier`: the vanilla entity whose hard-coded behaviour a custom entity
    /// borrows.
    pub runtime_identifier: Option<EntityId>,
    /// `is_spawnable`: whether a spawn egg exists for the entity; `false` when absent.
    pub is_spawnable: Option<bool>,
    /// `is_summonable`: whether `/summon` may create it; `false` when absent.
    pub is_summonable: Option<bool>,
    /// `properties`: the entity's properties, by `namespace:name`.
    pub properties: IndexMap<String, Property>,
    /// `animations`: short name → animation or animation controller identifier.
    pub animations: IndexMap<String, String>,
    /// `scripts`: the animations the server runs.
    pub scripts: Option<Scripts>,
    /// `spawn_category`: the mob cap the entity counts against.
    pub spawn_category: Option<SpawnCategory>,
    /// `property_memory_mapping`: Brain memories bound to properties (Brain Prototype
    /// experiment), kept as written.
    pub property_memory_mapping: Option<Value>,
    /// `entity_version`: the upgrade handlers a saved entity runs, by version (format 1.26.20
    /// onward).
    pub entity_version: Option<EntityVersion>,
}

/// `description.properties.<name>`: one entity property.
#[derive(Clone, Debug, PartialEq)]
pub struct Property {
    /// `type`: what values the property holds.
    pub kind: PropertyType,
    /// `default`: the value a new entity starts with.
    pub default: Option<PropertyDefault>,
    /// `range`: the smallest and largest value of an `int` or `float` property.
    pub range: Option<[Value; 2]>,
    /// `values`: the values of an `enum` property.
    pub values: Vec<String>,
    /// `client_sync`: whether the client sees the value; `false` when absent.
    pub client_sync: Option<bool>,
}

/// `type` of a [`Property`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PropertyType {
    /// `"bool"`.
    Bool,
    /// `"int"`.
    Int,
    /// `"float"`.
    Float,
    /// `"enum"`: one of the property's `values`.
    Enum,
}

impl std::fmt::Display for PropertyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl PropertyType {
    /// The JSON spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Int => "int",
            Self::Float => "float",
            Self::Enum => "enum",
        }
    }
}

/// `default` of a [`Property`]: a value, or an expression evaluated when the entity is created.
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyDefault {
    /// A JSON boolean.
    Bool(bool),
    /// A JSON integer.
    Int(i32),
    /// A JSON number with a fraction.
    Float(f64),
    /// A string that is one of an `enum` property's `values`.
    Enum(String),
    /// Any other string: a Molang expression that may call only
    /// `query.had_component_group` and assigns nothing.
    Expression(MolangSource),
}

/// `description.scripts`: the scripts the server runs. Only `animate` is read server-side.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scripts {
    /// `animate`: the animations and controllers the server plays.
    pub animate: Vec<AnimateEntry>,
    /// Whether the document wrote `animate`, so an empty list is written back; not part of
    /// equality. Public only so that struct-update syntax (`..Scripts::default()`) works outside
    /// the crate.
    #[doc(hidden)]
    pub written: Written,
}

impl Scripts {
    /// The index the entry at `index` of `animate` has in the file it was read from: `index`
    /// itself for a list the reader took whole, or one changed since.
    pub(crate) fn place(&self, index: usize) -> usize {
        self.written
            .animate_places
            .as_deref()
            .filter(|places| places.len() == self.animate.len())
            .and_then(|places| places.get(index).copied())
            .unwrap_or(index)
    }
}

/// One entry of [`Scripts::animate`].
#[derive(Clone, Debug, PartialEq)]
pub enum AnimateEntry {
    /// `"name"`: always played. Vanilla looks the name up lower-cased.
    Always(String),
    /// `{ "name": "<expression>" }`: played while the expression is non-zero. Vanilla skips an
    /// object with more or fewer than one member.
    Conditional(IndexMap<String, MolangSource>),
}

/// `description.spawn_category`: the mob cap an entity counts against.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SpawnCategory {
    /// `"creature"`.
    Creature,
    /// `"monster"`.
    Monster,
    /// `"ambient"`.
    Ambient,
    /// `"water_ambient"`.
    WaterAmbient,
    /// `"water_creature"`.
    WaterCreature,
    /// `"axolotls"`.
    Axolotls,
    /// `"underground_water_creature"`.
    UndergroundWaterCreature,
    /// `"misc"`: the default.
    Misc,
}

impl std::fmt::Display for SpawnCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl SpawnCategory {
    /// Every category, in vanilla's order.
    pub const ALL: [Self; 8] = [
        Self::Creature,
        Self::Monster,
        Self::Ambient,
        Self::WaterAmbient,
        Self::WaterCreature,
        Self::Axolotls,
        Self::UndergroundWaterCreature,
        Self::Misc,
    ];

    /// The JSON spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Creature => "creature",
            Self::Monster => "monster",
            Self::Ambient => "ambient",
            Self::WaterAmbient => "water_ambient",
            Self::WaterCreature => "water_creature",
            Self::Axolotls => "axolotls",
            Self::UndergroundWaterCreature => "underground_water_creature",
            Self::Misc => "misc",
        }
    }

    /// The category a JSON spelling names.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.as_str() == name)
    }
}

/// `description.entity_version`: which upgrade handler a saved entity runs to reach each
/// version.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EntityVersion {
    /// `upgrades`: one entry per version, from 1 upward without gaps.
    pub upgrades: Vec<UpgradeEntry>,
}

/// One entry of [`EntityVersion::upgrades`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpgradeEntry {
    /// `upgrade_to`: the version the handler upgrades to (1 or more).
    pub upgrade_to: u32,
    /// `name`: the handler, a key of [`Entity::upgrades`].
    pub name: String,
}

/// `component_groups.<name>`: a named set of components the entity's events add and remove.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ComponentGroup {
    /// The group's components and AI goals, in file order.
    pub components: ComponentMap<EntityComponent>,
}

impl ComponentGroup {
    /// The group's components and AI goals, in file order.
    #[must_use]
    pub fn components(&self) -> &ComponentMap<EntityComponent> {
        &self.components
    }

    /// The group's components and AI goals, mutably.
    pub fn components_mut(&mut self) -> &mut ComponentMap<EntityComponent> {
        &mut self.components
    }
}

/// `events.<name>`: what the entity does when the event fires. The body is kept as written;
/// load-time validation checks its shape (`add`, `remove`, `sequence`, `randomize`,
/// `first_valid`, `trigger`, `filters`, `set_property` and the entity responses).
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub(crate) name: String,
    /// The event's body as written.
    pub node: Value,
}

/// `brain`: the Brain Prototype's memories.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct Brain {
    /// `memories`: the entity's memories, unique by name.
    pub memories: Vec<Memory>,
    /// `internal_memory_access_mappings`: memories the engine itself writes.
    #[facet(default)]
    pub internal_memory_access_mappings: Option<InternalMemoryAccess>,
}

/// One entry of [`Brain::memories`].
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct Memory {
    /// `name`: the memory's identifier.
    pub name: String,
    /// `type`: what the memory holds.
    #[facet(rename = "type")]
    pub kind: MemoryType,
    /// `persists`: whether the memory is saved with the entity; `false` when absent.
    #[facet(default)]
    pub persists: Option<bool>,
}

/// `type` of a [`Memory`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(rename_all = "snake_case")]
#[repr(u8)]
#[non_exhaustive]
pub enum MemoryType {
    /// `"bool"`.
    Bool,
    /// `"int"`.
    Int,
    /// `"float"`.
    Float,
    /// `"string"`.
    String,
    /// `"entity_ref"`.
    EntityRef,
    /// `"block_pos"`.
    BlockPos,
    /// `"block_pos_vec"`.
    BlockPosVec,
    /// `"poi_memory"`.
    PoiMemory,
}

impl MemoryType {
    /// Every type, in vanilla's order (values 1 to 8).
    pub const ALL: [Self; 8] = [
        Self::Bool,
        Self::Int,
        Self::Float,
        Self::String,
        Self::EntityRef,
        Self::BlockPos,
        Self::BlockPosVec,
        Self::PoiMemory,
    ];

    /// The JSON spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Int => "int",
            Self::Float => "float",
            Self::String => "string",
            Self::EntityRef => "entity_ref",
            Self::BlockPos => "block_pos",
            Self::BlockPosVec => "block_pos_vec",
            Self::PoiMemory => "poi_memory",
        }
    }

    /// The type a JSON spelling names.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.as_str() == name)
    }
}

/// `brain.internal_memory_access_mappings`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct InternalMemoryAccess {
    /// `beehive_has_nectar`: the `bool` memory the engine sets while the bee carries nectar.
    #[facet(default)]
    pub beehive_has_nectar: Option<String>,
}

/// `__deprecated`: the side-car of what an older document wrote and this version does not
/// read, kept so the entity writes back as loaded.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeprecatedData {
    /// `aliases` of a document below format 1.21.10, which the game still registers.
    pub aliases: IndexMap<String, Value>,
}

/// A beta document (`"format_version": "beta"`): the newest struct's shape, read with the beta
/// component set. Loaded only with the Beta APIs toggle or from a base-game pack.
#[derive(Clone, Debug, PartialEq)]
pub struct Beta(pub Entity);

impl Deref for Beta {
    type Target = Entity;

    fn deref(&self) -> &Entity {
        &self.0
    }
}

impl Entity {
    /// The component group named `name`.
    pub fn component_group(&self, name: &str) -> Option<&ComponentGroup> {
        self.component_groups.get(name)
    }

    /// The entity's properties, by name (`description.properties`).
    pub fn properties(&self) -> &IndexMap<String, Property> {
        &self.description.properties
    }

    /// Whether a spawn egg exists for the entity: `description.is_spawnable`, `false` when
    /// absent.
    pub fn is_spawnable(&self) -> bool {
        self.description.is_spawnable.unwrap_or(false)
    }

    /// Whether `/summon` may create the entity: `description.is_summonable`, `false` when
    /// absent.
    pub fn is_summonable(&self) -> bool {
        self.description.is_summonable.unwrap_or(false)
    }
}

impl Event {
    /// The event's name, its key in [`Entity::events`].
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Entity {
    /// An entity named `id` with an empty description, no components, groups or events.
    pub fn new(id: EntityId) -> Self {
        Self {
            description: Description::new(id),
            components: ComponentMap::new(),
            component_groups: IndexMap::new(),
            events: IndexMap::new(),
            brain: None,
            upgrades: IndexMap::new(),
            deprecated: None,
            written: Written {
                components: true,
                ..Written::default()
            },
        }
    }
}

impl Description {
    /// A description naming `id` and nothing else.
    pub fn new(id: EntityId) -> Self {
        Self {
            identifier: id,
            runtime_identifier: None,
            is_spawnable: None,
            is_summonable: None,
            properties: IndexMap::new(),
            animations: IndexMap::new(),
            scripts: None,
            spawn_category: None,
            property_memory_mapping: None,
            entity_version: None,
        }
    }
}

impl Event {
    /// The event `name` with the body `node` (`{ "add": { "component_groups": [...] } }`, …).
    pub fn new(name: impl Into<String>, node: Value) -> Self {
        Self {
            name: name.into(),
            node,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_type_and_spawn_category_display_their_json_spelling() {
        assert_eq!(PropertyType::Enum.to_string(), "enum");
        assert_eq!(PropertyType::Bool.to_string(), PropertyType::Bool.as_str());
        for category in SpawnCategory::ALL {
            assert_eq!(category.to_string(), category.as_str());
        }
        assert_eq!(SpawnCategory::WaterAmbient.to_string(), "water_ambient");
    }
}
